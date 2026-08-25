//! Tokio-backed ownership and cancellation for one interaction workflow.
#![forbid(unsafe_code)]

use nexa_orchestrator::{
    CancellationDirective, CancellationTarget, InteractionWorkflow, WorkflowCancellationPlan,
    CANCELLATION_PROPAGATION_V1,
};
use std::{collections::HashMap, future::Future};
use tokio::task::{AbortHandle, Id, JoinSet};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowTaskGroupState {
    Accepting,
    Cancelling,
    Draining,
    ExecutingPlan,
    Cancelled,
    Drained,
    PlanExecuted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowTaskCompletionKind {
    Cancelled,
    Drained,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkflowTaskCompletion {
    workflow: InteractionWorkflow,
    kind: WorkflowTaskCompletionKind,
}
impl WorkflowTaskCompletion {
    pub const fn workflow(&self) -> InteractionWorkflow {
        self.workflow
    }
    pub const fn kind(&self) -> WorkflowTaskCompletionKind {
        self.kind
    }
}

/// The closed outcome for one target in an accepted exact plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CancellationTargetExecutionOutcome {
    Stopped,
    ReportedNonCancellable { owned_task_count: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CancellationTargetExecutionEvidence {
    target: CancellationTarget,
    outcome: CancellationTargetExecutionOutcome,
}
impl CancellationTargetExecutionEvidence {
    pub const fn target(&self) -> CancellationTarget {
        self.target
    }
    pub const fn outcome(&self) -> CancellationTargetExecutionOutcome {
        self.outcome
    }
}

/// Immutable evidence for one accepted exact-plan execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowCancellationExecution {
    workflow: InteractionWorkflow,
    target_outcomes: Vec<CancellationTargetExecutionEvidence>,
    accepted_unclassified_task_count: usize,
}
impl WorkflowCancellationExecution {
    pub const fn workflow(&self) -> InteractionWorkflow {
        self.workflow
    }
    pub fn target_outcomes(&self) -> &[CancellationTargetExecutionEvidence] {
        &self.target_outcomes
    }
    pub const fn accepted_unclassified_task_count(&self) -> usize {
        self.accepted_unclassified_task_count
    }
    /// A successful result proves this count is zero.
    pub const fn remaining_unclassified_task_count(&self) -> usize {
        0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowTaskGroupError {
    NotAcceptingTasks,
    UnsupportedPlanVersion,
    AssociationMismatch,
    InvalidPlan,
    PlanCoverageMismatch,
    TaskJoinFailure,
    ConflictingCompletion,
}
impl std::fmt::Display for WorkflowTaskGroupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NotAcceptingTasks => "workflow task group is not accepting tasks",
            Self::UnsupportedPlanVersion => "unsupported workflow cancellation plan version",
            Self::AssociationMismatch => "workflow cancellation plan association mismatch",
            Self::InvalidPlan => "invalid workflow cancellation plan",
            Self::PlanCoverageMismatch => "workflow cancellation plan coverage mismatch",
            Self::TaskJoinFailure => "owned workflow task failed to join",
            Self::ConflictingCompletion => "workflow task group already completed differently",
        })
    }
}
impl std::error::Error for WorkflowTaskGroupError {}

#[derive(Clone, Copy)]
enum TaskAssociation {
    Unclassified,
    Target(CancellationTarget),
}

struct OwnedTask {
    association: TaskAssociation,
    abort_handle: AbortHandle,
}

pub struct WorkflowTaskGroup {
    workflow: InteractionWorkflow,
    cancellation: CancellationToken,
    unclassified_cancellation: CancellationToken,
    target_cancellations: [CancellationToken; 5],
    tasks: JoinSet<()>,
    associations: HashMap<Id, OwnedTask>,
    target_task_counts: [usize; 5],
    unclassified_task_count: usize,
    state: WorkflowTaskGroupState,
    terminal: Option<Result<WorkflowTaskCompletion, WorkflowTaskGroupError>>,
    accepted_plan: Option<WorkflowCancellationPlan>,
    plan_terminal: Option<Result<WorkflowCancellationExecution, WorkflowTaskGroupError>>,
    plan_outcomes: Vec<CancellationTargetExecutionEvidence>,
    accepted_unclassified_task_count: usize,
    plan_join_failed: bool,
}

impl WorkflowTaskGroup {
    pub fn new(workflow: InteractionWorkflow) -> Self {
        let cancellation = CancellationToken::new();
        Self {
            workflow,
            unclassified_cancellation: cancellation.child_token(),
            target_cancellations: std::array::from_fn(|_| cancellation.child_token()),
            cancellation,
            tasks: JoinSet::new(),
            associations: HashMap::new(),
            target_task_counts: [0; 5],
            unclassified_task_count: 0,
            state: WorkflowTaskGroupState::Accepting,
            terminal: None,
            accepted_plan: None,
            plan_terminal: None,
            plan_outcomes: Vec::new(),
            accepted_unclassified_task_count: 0,
            plan_join_failed: false,
        }
    }
    pub const fn workflow(&self) -> InteractionWorkflow {
        self.workflow
    }
    pub const fn state(&self) -> WorkflowTaskGroupState {
        self.state
    }
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }
    pub const fn target_task_count(&self, target: CancellationTarget) -> usize {
        self.target_task_counts[target_index(target)]
    }
    pub const fn unclassified_task_count(&self) -> usize {
        self.unclassified_task_count
    }
    pub fn is_cancellation_requested(&self) -> bool {
        self.cancellation.is_cancelled()
    }

    pub fn spawn<F, Fut>(&mut self, task: F) -> Result<(), WorkflowTaskGroupError>
    where
        F: FnOnce(CancellationToken) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        if self.state != WorkflowTaskGroupState::Accepting {
            return Err(WorkflowTaskGroupError::NotAcceptingTasks);
        }
        let token = self.unclassified_cancellation.child_token();
        let handle = self.tasks.spawn(async move { task(token).await });
        self.associations.insert(
            handle.id(),
            OwnedTask {
                association: TaskAssociation::Unclassified,
                abort_handle: handle,
            },
        );
        self.unclassified_task_count += 1;
        Ok(())
    }

    pub fn spawn_for_target<F, Fut>(
        &mut self,
        target: CancellationTarget,
        task: F,
    ) -> Result<(), WorkflowTaskGroupError>
    where
        F: FnOnce(CancellationToken) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        if self.state != WorkflowTaskGroupState::Accepting {
            return Err(WorkflowTaskGroupError::NotAcceptingTasks);
        }
        let index = target_index(target);
        let token = self.target_cancellations[index].child_token();
        let handle = self.tasks.spawn(async move { task(token).await });
        self.associations.insert(
            handle.id(),
            OwnedTask {
                association: TaskAssociation::Target(target),
                abort_handle: handle,
            },
        );
        self.target_task_counts[index] += 1;
        Ok(())
    }

    /// Atomically accepts and executes one canonical plan covering exactly all live targets.
    pub async fn execute_cancellation_plan(
        &mut self,
        plan: &WorkflowCancellationPlan,
    ) -> Result<WorkflowCancellationExecution, WorkflowTaskGroupError> {
        if let Some(accepted) = &self.accepted_plan {
            if accepted != plan {
                return Err(WorkflowTaskGroupError::ConflictingCompletion);
            }
            if let Some(result) = &self.plan_terminal {
                return result.clone();
            }
        } else {
            self.preflight(plan)?;
            self.accepted_plan = Some(plan.clone());
            self.state = WorkflowTaskGroupState::ExecutingPlan;
            self.accepted_unclassified_task_count = self.unclassified_task_count;
            self.plan_outcomes = plan
                .directives()
                .iter()
                .map(|directive| {
                    let count = self.target_task_counts[target_index(directive.target())];
                    let outcome = match directive.directive() {
                        CancellationDirective::RequestCancellation => {
                            CancellationTargetExecutionOutcome::Stopped
                        }
                        CancellationDirective::ReportNonCancellable => {
                            CancellationTargetExecutionOutcome::ReportedNonCancellable {
                                owned_task_count: count,
                            }
                        }
                    };
                    CancellationTargetExecutionEvidence {
                        target: directive.target(),
                        outcome,
                    }
                })
                .collect();
            for directive in plan.directives() {
                if directive.directive() == CancellationDirective::RequestCancellation {
                    self.target_cancellations[target_index(directive.target())].cancel();
                }
            }
            self.unclassified_cancellation.cancel();
        }

        while self.required_plan_tasks_remain() {
            let Some(result) = self.tasks.join_next_with_id().await else {
                break;
            };
            match result {
                Ok((id, ())) => self.remove_association(id),
                Err(error) => {
                    let failure_is_required = self
                        .associations
                        .get(&error.id())
                        .is_some_and(|task| self.association_is_required(task.association));
                    self.remove_association(error.id());
                    self.plan_join_failed |= failure_is_required;
                    if failure_is_required {
                        // A required target can no longer satisfy the accepted plan. Abort only
                        // the remaining work required by this plan, then drain it. Reported
                        // non-cancellable work remains privately owned and unaffected.
                        self.abort_required_plan_tasks();
                    }
                }
            }
        }
        let result = if self.plan_join_failed {
            Err(WorkflowTaskGroupError::TaskJoinFailure)
        } else {
            Ok(WorkflowCancellationExecution {
                workflow: self.workflow,
                target_outcomes: self.plan_outcomes.clone(),
                accepted_unclassified_task_count: self.accepted_unclassified_task_count,
            })
        };
        self.state = WorkflowTaskGroupState::PlanExecuted;
        self.plan_terminal = Some(result.clone());
        result
    }

    fn preflight(&self, plan: &WorkflowCancellationPlan) -> Result<(), WorkflowTaskGroupError> {
        self.preflight_with_invariants(
            plan,
            plan.version() == CANCELLATION_PROPAGATION_V1,
            plan.directives()
                .iter()
                .all(|directive| directive.version() == CANCELLATION_PROPAGATION_V1),
            plan.directives()
                .windows(2)
                .all(|pair| pair[0].target() < pair[1].target()),
        )
    }

    fn preflight_with_invariants(
        &self,
        plan: &WorkflowCancellationPlan,
        supported_plan_version: bool,
        supported_directive_versions: bool,
        canonical_directives: bool,
    ) -> Result<(), WorkflowTaskGroupError> {
        if self.state != WorkflowTaskGroupState::Accepting {
            return Err(WorkflowTaskGroupError::ConflictingCompletion);
        }
        if !supported_plan_version {
            return Err(WorkflowTaskGroupError::UnsupportedPlanVersion);
        }
        if (
            plan.workflow_id(),
            plan.session_id(),
            plan.correlation_id(),
            plan.trace_id(),
        ) != (
            self.workflow.workflow_id(),
            self.workflow.session_id(),
            self.workflow.correlation_id(),
            self.workflow.trace_id(),
        ) {
            return Err(WorkflowTaskGroupError::AssociationMismatch);
        }
        if plan.directives().len() > 5 || !supported_directive_versions || !canonical_directives {
            return Err(WorkflowTaskGroupError::InvalidPlan);
        }
        let covered = plan.directives().iter().fold([false; 5], |mut set, d| {
            set[target_index(d.target())] = true;
            set
        });
        if (0..5).any(|index| covered[index] != (self.target_task_counts[index] != 0)) {
            return Err(WorkflowTaskGroupError::PlanCoverageMismatch);
        }
        Ok(())
    }

    fn required_plan_tasks_remain(&self) -> bool {
        self.unclassified_task_count != 0
            || self.accepted_plan.as_ref().is_some_and(|plan| {
                plan.directives().iter().any(|d| {
                    d.directive() == CancellationDirective::RequestCancellation
                        && self.target_task_counts[target_index(d.target())] != 0
                })
            })
    }

    fn association_is_required(&self, association: TaskAssociation) -> bool {
        match association {
            TaskAssociation::Unclassified => true,
            TaskAssociation::Target(target) => self.accepted_plan.as_ref().is_some_and(|plan| {
                plan.directives().iter().any(|directive| {
                    directive.target() == target
                        && directive.directive() == CancellationDirective::RequestCancellation
                })
            }),
        }
    }

    fn abort_required_plan_tasks(&self) {
        for task in self.associations.values() {
            if self.association_is_required(task.association) {
                task.abort_handle.abort();
            }
        }
    }

    fn remove_association(&mut self, id: Id) {
        match self.associations.remove(&id).map(|task| task.association) {
            Some(TaskAssociation::Unclassified) => self.unclassified_task_count -= 1,
            Some(TaskAssociation::Target(target)) => {
                self.target_task_counts[target_index(target)] -= 1
            }
            None => {}
        }
    }

    pub async fn cancel_and_wait(
        &mut self,
    ) -> Result<WorkflowTaskCompletion, WorkflowTaskGroupError> {
        if self.accepted_plan.is_some() {
            return Err(WorkflowTaskGroupError::ConflictingCompletion);
        }
        if let Some(result) = self.terminal {
            return match result {
                Ok(e) if e.kind == WorkflowTaskCompletionKind::Cancelled => Ok(e),
                Ok(_) => Err(WorkflowTaskGroupError::ConflictingCompletion),
                Err(e) => Err(e),
            };
        }
        if self.state != WorkflowTaskGroupState::Accepting
            && self.state != WorkflowTaskGroupState::Cancelling
        {
            return Err(WorkflowTaskGroupError::ConflictingCompletion);
        }
        self.state = WorkflowTaskGroupState::Cancelling;
        self.cancellation.cancel();
        self.finish(WorkflowTaskCompletionKind::Cancelled).await
    }
    pub async fn drain(&mut self) -> Result<WorkflowTaskCompletion, WorkflowTaskGroupError> {
        if self.accepted_plan.is_some() {
            return Err(WorkflowTaskGroupError::ConflictingCompletion);
        }
        if let Some(result) = self.terminal {
            return match result {
                Ok(e) if e.kind == WorkflowTaskCompletionKind::Drained => Ok(e),
                Ok(_) => Err(WorkflowTaskGroupError::ConflictingCompletion),
                Err(e) => Err(e),
            };
        }
        if self.state != WorkflowTaskGroupState::Accepting
            && self.state != WorkflowTaskGroupState::Draining
        {
            return Err(WorkflowTaskGroupError::ConflictingCompletion);
        }
        self.state = WorkflowTaskGroupState::Draining;
        self.finish(WorkflowTaskCompletionKind::Drained).await
    }
    async fn finish(
        &mut self,
        kind: WorkflowTaskCompletionKind,
    ) -> Result<WorkflowTaskCompletion, WorkflowTaskGroupError> {
        let mut failed = false;
        while let Some(result) = self.tasks.join_next_with_id().await {
            match result {
                Ok((id, ())) => self.remove_association(id),
                Err(e) => {
                    self.remove_association(e.id());
                    failed = true;
                }
            }
        }
        self.target_task_counts = [0; 5];
        self.unclassified_task_count = 0;
        self.state = if kind == WorkflowTaskCompletionKind::Cancelled {
            WorkflowTaskGroupState::Cancelled
        } else {
            WorkflowTaskGroupState::Drained
        };
        let result = if failed {
            Err(WorkflowTaskGroupError::TaskJoinFailure)
        } else {
            Ok(WorkflowTaskCompletion {
                workflow: self.workflow,
                kind,
            })
        };
        self.terminal = Some(result);
        result
    }
}

const fn target_index(target: CancellationTarget) -> usize {
    match target {
        CancellationTarget::Retrieval => 0,
        CancellationTarget::TutorGeneration => 1,
        CancellationTarget::Speech => 2,
        CancellationTarget::Behavior => 3,
        CancellationTarget::ToolExecution => 4,
    }
}
impl Drop for WorkflowTaskGroup {
    fn drop(&mut self) {
        self.tasks.abort_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexa_orchestrator::{
        plan_workflow_cancellation, ActiveCancellationTarget, CancellationSemantics,
    };
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    fn workflow() -> InteractionWorkflow {
        use nexa_domain::{CorrelationId, SessionId, TraceId, WorkflowId};
        use uuid::Uuid;

        InteractionWorkflow::new(
            WorkflowId::new(Uuid::from_u128(1)).unwrap(),
            SessionId::new(Uuid::from_u128(2)).unwrap(),
            CorrelationId::new(Uuid::from_u128(3)).unwrap(),
            TraceId::new(Uuid::from_u128(4)).unwrap(),
        )
    }

    fn valid_plan(workflow: InteractionWorkflow) -> WorkflowCancellationPlan {
        plan_workflow_cancellation(
            &workflow.cancel().unwrap(),
            workflow.workflow_id(),
            workflow.session_id(),
            workflow.correlation_id(),
            workflow.trace_id(),
            &[ActiveCancellationTarget::new(
                CancellationTarget::Retrieval,
                CancellationSemantics::Cancellable,
            )],
        )
        .unwrap()
    }

    #[tokio::test]
    async fn unsupported_and_noncanonical_preflight_are_side_effect_free() {
        let polls = Arc::new(AtomicUsize::new(0));
        let polls_in_task = Arc::clone(&polls);
        let expected = workflow();
        let plan = valid_plan(expected);
        let mut group = WorkflowTaskGroup::new(expected);
        group
            .spawn_for_target(CancellationTarget::Retrieval, move |_| {
                std::future::poll_fn(move |_| {
                    polls_in_task.fetch_add(1, Ordering::SeqCst);
                    std::task::Poll::Pending
                })
            })
            .unwrap();
        let before = (
            group.task_count(),
            group.target_task_count(CancellationTarget::Retrieval),
            group.unclassified_task_count(),
        );

        assert_eq!(
            group.preflight_with_invariants(&plan, false, true, true),
            Err(WorkflowTaskGroupError::UnsupportedPlanVersion)
        );
        assert_eq!(
            group.preflight_with_invariants(&plan, true, true, false),
            Err(WorkflowTaskGroupError::InvalidPlan)
        );
        assert_eq!(polls.load(Ordering::SeqCst), 0);
        assert_eq!(
            (
                group.task_count(),
                group.target_task_count(CancellationTarget::Retrieval),
                group.unclassified_task_count(),
            ),
            before
        );
        assert_eq!(group.state(), WorkflowTaskGroupState::Accepting);
        assert!(!group.is_cancellation_requested());
        assert_eq!(group.spawn(|_| async {}), Ok(()));
        assert_eq!(
            group.spawn_for_target(CancellationTarget::Speech, |_| async {}),
            Ok(())
        );
    }
}
