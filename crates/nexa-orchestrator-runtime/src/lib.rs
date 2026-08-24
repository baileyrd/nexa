//! Tokio-backed ownership and cancellation for one interaction workflow.
#![forbid(unsafe_code)]

use nexa_orchestrator::{CancellationTarget, InteractionWorkflow};
use std::future::Future;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

/// Whether an owned task group is still accepting work or has begun termination.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowTaskGroupState {
    Accepting,
    Cancelling,
    Draining,
    Cancelled,
    Drained,
}

/// Why all work owned by a workflow task group stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowTaskCompletionKind {
    Cancelled,
    Drained,
}

/// Content-free evidence that every task associated with the exact workflow has stopped.
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

/// A closed task-group operation failure with no task or panic content.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowTaskGroupError {
    NotAcceptingTasks,
    TaskJoinFailure,
    ConflictingCompletion,
}

impl std::fmt::Display for WorkflowTaskGroupError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::NotAcceptingTasks => "workflow task group is not accepting tasks",
            Self::TaskJoinFailure => "owned workflow task failed to join",
            Self::ConflictingCompletion => "workflow task group already completed differently",
        })
    }
}

impl std::error::Error for WorkflowTaskGroupError {}

/// The owned Tokio task group for one exact workflow identity association.
///
/// Dropping the group calls `JoinSet::abort_all`; outstanding work is never detached.
pub struct WorkflowTaskGroup {
    workflow: InteractionWorkflow,
    cancellation: CancellationToken,
    target_cancellations: [CancellationToken; 5],
    tasks: JoinSet<Option<CancellationTarget>>,
    target_task_counts: [usize; 5],
    state: WorkflowTaskGroupState,
    terminal: Option<Result<WorkflowTaskCompletion, WorkflowTaskGroupError>>,
}

impl WorkflowTaskGroup {
    pub fn new(workflow: InteractionWorkflow) -> Self {
        let cancellation = CancellationToken::new();
        Self {
            workflow,
            target_cancellations: std::array::from_fn(|_| cancellation.child_token()),
            cancellation,
            tasks: JoinSet::new(),
            target_task_counts: [0; 5],
            state: WorkflowTaskGroupState::Accepting,
            terminal: None,
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

    /// Returns the number of tasks currently owned for one closed subsystem target.
    pub const fn target_task_count(&self, target: CancellationTarget) -> usize {
        self.target_task_counts[target_index(target)]
    }

    pub fn is_cancellation_requested(&self) -> bool {
        self.cancellation.is_cancelled()
    }

    /// Spawns owned work and supplies it a child of this workflow's root token.
    pub fn spawn<F, Fut>(&mut self, task: F) -> Result<(), WorkflowTaskGroupError>
    where
        F: FnOnce(CancellationToken) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        if self.state != WorkflowTaskGroupState::Accepting {
            return Err(WorkflowTaskGroupError::NotAcceptingTasks);
        }
        let token = self.cancellation.child_token();
        self.tasks.spawn(async move {
            task(token).await;
            None
        });
        Ok(())
    }

    /// Spawns work associated with exactly one target and supplies a private task token.
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
        self.tasks.spawn(async move {
            task(token).await;
            Some(target)
        });
        self.target_task_counts[index] += 1;
        Ok(())
    }

    /// Requests cancellation once and waits until every owned task has stopped.
    pub async fn cancel_and_wait(
        &mut self,
    ) -> Result<WorkflowTaskCompletion, WorkflowTaskGroupError> {
        if let Some(result) = self.terminal {
            return match result {
                Ok(evidence) if evidence.kind == WorkflowTaskCompletionKind::Cancelled => {
                    Ok(evidence)
                }
                Ok(_) => Err(WorkflowTaskGroupError::ConflictingCompletion),
                Err(error) => Err(error),
            };
        }
        if self.state == WorkflowTaskGroupState::Draining {
            return Err(WorkflowTaskGroupError::ConflictingCompletion);
        }
        self.state = WorkflowTaskGroupState::Cancelling;
        self.cancellation.cancel();
        self.finish(WorkflowTaskCompletionKind::Cancelled).await
    }

    /// Waits for natural task completion without requesting cancellation.
    pub async fn drain(&mut self) -> Result<WorkflowTaskCompletion, WorkflowTaskGroupError> {
        if let Some(result) = self.terminal {
            return match result {
                Ok(evidence) if evidence.kind == WorkflowTaskCompletionKind::Drained => {
                    Ok(evidence)
                }
                Ok(_) => Err(WorkflowTaskGroupError::ConflictingCompletion),
                Err(error) => Err(error),
            };
        }
        if self.state == WorkflowTaskGroupState::Cancelling {
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
        while let Some(result) = self.tasks.join_next().await {
            match result {
                Ok(Some(target)) => self.target_task_counts[target_index(target)] -= 1,
                Ok(None) => {}
                Err(_) => failed = true,
            }
        }
        // A failed join cannot return its private association, but no tasks remain owned.
        self.target_task_counts = [0; 5];
        self.state = match kind {
            WorkflowTaskCompletionKind::Cancelled => WorkflowTaskGroupState::Cancelled,
            WorkflowTaskCompletionKind::Drained => WorkflowTaskGroupState::Drained,
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
