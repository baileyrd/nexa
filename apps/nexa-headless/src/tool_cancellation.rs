//! Headless application-control binding for Tool Execution cancellation.
//!
//! This module neither executes tools nor enforces policy or sandboxing. It binds an already
//! admitted ADR-0065 control to the orchestrator's Tool Execution cancellation target.

use nexa_domain::{CorrelationId, SessionId, TraceId, WorkflowId};
use nexa_labs::{
    admit_tool_execution, cancel_tool_execution, AdmittedToolExecution,
    CancellationSemantics as ToolSemantics, RiskClass, ToolAdmissionRequest, ToolAssociation,
    ToolCancellationCapability, ToolCancellationControl, ToolCancellationEvidence,
    ToolCancellationOutcomeKind, TOOL_EXECUTION_SECURITY_V1,
};
use nexa_orchestrator::{
    plan_workflow_cancellation, ActiveCancellationTarget, CancellationDirective,
    CancellationSemantics, CancellationTarget, InteractionWorkflow, WorkflowCancellationPlan,
    WorkflowState,
};
use nexa_orchestrator_runtime::{
    CancellationTargetExecutionOutcome, WorkflowCancellationExecution, WorkflowTaskGroup,
};
use std::sync::{Arc, Mutex};

/// Closed, content-free failures at the headless Tool Execution boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolExecutionCancellationCompositionError {
    InvalidWorkflow,
    AssociationMismatch,
    AdmissionRejected,
    UnsupportedVersion,
    ControlFailure,
    RuntimeFailure,
    ConflictingExecution,
}
impl std::fmt::Display for ToolExecutionCancellationCompositionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidWorkflow => "invalid cancelled workflow",
            Self::AssociationMismatch => "tool cancellation association mismatch",
            Self::AdmissionRejected => "tool admission rejected",
            Self::UnsupportedVersion => "unsupported tool cancellation version",
            Self::ControlFailure => "tool cancellation control failure",
            Self::RuntimeFailure => "tool cancellation runtime failure",
            Self::ConflictingExecution => "conflicting tool cancellation execution",
        })
    }
}
impl std::error::Error for ToolExecutionCancellationCompositionError {}

/// Immutable evidence for one exact admitted Tool Execution control operation.
///
/// Structural admission does not prove authentic or fresh policy, a real confirmation identity,
/// or sandbox enforcement. `Accepted` proves only dependency acceptance and control-future
/// terminalization; `DeclaredNonCancellable` proves only the declared semantics. Neither outcome
/// proves that Tool Execution occurred or that external work stopped.
#[derive(Clone, Eq, PartialEq)]
pub struct ToolExecutionCancellationEvidence {
    runtime: WorkflowCancellationExecution,
    cancellation: ToolCancellationEvidence,
    association: ToolAssociation,
    risk: RiskClass,
}
impl std::fmt::Debug for ToolExecutionCancellationEvidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolExecutionCancellationEvidence")
            .field("runtime", &self.runtime)
            .field("cancellation_kind", &self.cancellation.kind)
            .field("association", &"REDACTED")
            .field("risk", &self.risk)
            .finish()
    }
}
impl ToolExecutionCancellationEvidence {
    pub const fn runtime(&self) -> &WorkflowCancellationExecution {
        &self.runtime
    }
    pub const fn cancellation(&self) -> &ToolCancellationEvidence {
        &self.cancellation
    }
    pub const fn association(&self) -> &ToolAssociation {
        &self.association
    }
    pub const fn risk(&self) -> RiskClass {
        self.risk
    }
}

enum ExecutionState {
    Ready,
    Running,
    Succeeded(Box<ToolExecutionCancellationEvidence>),
    Failed,
}

/// Owns one admitted Tool Execution cancellation-control operation and its runtime task.
pub struct ToolExecutionCancellationComposition<C> {
    workflow: InteractionWorkflow,
    admitted: AdmittedToolExecution,
    capability: ToolCancellationCapability,
    plan: WorkflowCancellationPlan,
    control: Arc<C>,
    terminal: ExecutionState,
    #[cfg(test)]
    fault: Option<TestFault>,
}
#[cfg(test)]
#[derive(Clone, Copy)]
enum TestFault {
    Join,
    Coverage,
    Evidence,
}

impl<C: ToolCancellationControl + 'static> ToolExecutionCancellationComposition<C> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        workflow: InteractionWorkflow,
        workflow_id: WorkflowId,
        session_id: SessionId,
        correlation_id: CorrelationId,
        trace_id: TraceId,
        admission: ToolAdmissionRequest,
        capability: ToolCancellationCapability,
        control: C,
    ) -> Result<Self, ToolExecutionCancellationCompositionError> {
        if workflow.state() != WorkflowState::Cancelled {
            return Err(ToolExecutionCancellationCompositionError::InvalidWorkflow);
        }
        if (
            workflow.workflow_id(),
            workflow.session_id(),
            workflow.correlation_id(),
            workflow.trace_id(),
        ) != (workflow_id, session_id, correlation_id, trace_id)
        {
            return Err(ToolExecutionCancellationCompositionError::AssociationMismatch);
        }
        // The complete value is admitted unchanged before executable state or the dependency is kept.
        let admitted = admit_tool_execution(&admission)
            .map_err(|_| ToolExecutionCancellationCompositionError::AdmissionRejected)?;
        if capability.contract_version != TOOL_EXECUTION_SECURITY_V1 {
            return Err(ToolExecutionCancellationCompositionError::UnsupportedVersion);
        }
        if capability.association != *admitted.association() {
            return Err(ToolExecutionCancellationCompositionError::AssociationMismatch);
        }
        let semantics = match capability.semantics {
            ToolSemantics::Cancellable => CancellationSemantics::Cancellable,
            ToolSemantics::NonCancellable => CancellationSemantics::NonCancellable,
        };
        let plan = plan_workflow_cancellation(
            &workflow,
            workflow_id,
            session_id,
            correlation_id,
            trace_id,
            &[ActiveCancellationTarget::new(
                CancellationTarget::ToolExecution,
                semantics,
            )],
        )
        .map_err(|_| ToolExecutionCancellationCompositionError::InvalidWorkflow)?;
        let expected = match capability.semantics {
            ToolSemantics::Cancellable => CancellationDirective::RequestCancellation,
            ToolSemantics::NonCancellable => CancellationDirective::ReportNonCancellable,
        };
        if plan.directives().len() != 1
            || plan.directives()[0].target() != CancellationTarget::ToolExecution
            || plan.directives()[0].directive() != expected
        {
            return Err(ToolExecutionCancellationCompositionError::InvalidWorkflow);
        }
        Ok(Self {
            workflow,
            admitted,
            capability,
            plan,
            control: Arc::new(control),
            terminal: ExecutionState::Ready,
            #[cfg(test)]
            fault: None,
        })
    }

    pub const fn plan(&self) -> &WorkflowCancellationPlan {
        &self.plan
    }
    /// Bounded read-only inspection for deterministic controls and tests.
    pub fn inspect_control<R>(&self, inspect: impl FnOnce(&C) -> R) -> R {
        inspect(&self.control)
    }

    pub async fn execute(
        &mut self,
        capability: ToolCancellationCapability,
    ) -> Result<ToolExecutionCancellationEvidence, ToolExecutionCancellationCompositionError> {
        if capability != self.capability {
            return Err(ToolExecutionCancellationCompositionError::ConflictingExecution);
        }
        match &self.terminal {
            ExecutionState::Succeeded(value) => return Ok((**value).clone()),
            ExecutionState::Running | ExecutionState::Failed => {
                return Err(ToolExecutionCancellationCompositionError::RuntimeFailure)
            }
            ExecutionState::Ready => {}
        }
        // Terminalize before awaiting. Caller drop drops the group, aborts owned work, and forbids retry.
        self.terminal = ExecutionState::Running;
        let result = self.execute_once().await;
        match result {
            Ok(evidence) => {
                self.terminal = ExecutionState::Succeeded(Box::new(evidence.clone()));
                Ok(evidence)
            }
            Err(error) => {
                self.terminal = ExecutionState::Failed;
                Err(error)
            }
        }
    }

    async fn execute_once(
        &mut self,
    ) -> Result<ToolExecutionCancellationEvidence, ToolExecutionCancellationCompositionError> {
        let mut tasks = WorkflowTaskGroup::new(self.workflow);
        let observed = Arc::new(Mutex::new(None));
        let task_observed = Arc::clone(&observed);
        let control = Arc::clone(&self.control);
        let capability = self.capability.clone();
        let admitted = self.admitted.clone();
        #[cfg(test)]
        let fault = self.fault;

        let (release, started, completed) = if capability.semantics == ToolSemantics::NonCancellable
        {
            let (release_tx, release_rx) = tokio::sync::oneshot::channel();
            let (started_tx, started_rx) = tokio::sync::oneshot::channel();
            let (completed_tx, completed_rx) = tokio::sync::oneshot::channel();
            tasks
                .spawn_for_target(CancellationTarget::ToolExecution, move |token| async move {
                    let _ = started_tx.send(token.is_cancelled());
                    let _ = release_rx.await;
                    #[cfg(test)]
                    if matches!(fault, Some(TestFault::Join)) {
                        panic!("injected");
                    }
                    let _ = completed_tx.send(token.is_cancelled());
                })
                .map_err(|_| ToolExecutionCancellationCompositionError::RuntimeFailure)?;
            (Some(release_tx), Some(started_rx), Some(completed_rx))
        } else {
            tasks
                .spawn_for_target(CancellationTarget::ToolExecution, move |token| async move {
                    token.cancelled().await;
                    #[cfg(test)]
                    if matches!(fault, Some(TestFault::Join)) {
                        panic!("injected");
                    }
                    let result =
                        cancel_tool_execution(&capability, &admitted, control.as_ref()).await;
                    if let Ok(mut slot) = task_observed.lock() {
                        *slot = Some(result);
                    }
                })
                .map_err(|_| ToolExecutionCancellationCompositionError::RuntimeFailure)?;
            (None, None, None)
        };
        if let Some(started) = started {
            if started
                .await
                .map_err(|_| ToolExecutionCancellationCompositionError::RuntimeFailure)?
            {
                return Err(ToolExecutionCancellationCompositionError::RuntimeFailure);
            }
        }
        #[cfg(test)]
        if matches!(self.fault, Some(TestFault::Coverage)) {
            tasks
                .spawn_for_target(CancellationTarget::Behavior, |_| async {})
                .unwrap();
        }
        let runtime = tasks
            .execute_cancellation_plan(&self.plan)
            .await
            .map_err(|_| ToolExecutionCancellationCompositionError::RuntimeFailure)?;
        let cancellation = if self.capability.semantics == ToolSemantics::Cancellable {
            observed
                .lock()
                .ok()
                .and_then(|mut v| v.take())
                .ok_or(ToolExecutionCancellationCompositionError::ControlFailure)?
                .map_err(|_| ToolExecutionCancellationCompositionError::ControlFailure)?
        } else {
            cancel_tool_execution(&self.capability, &self.admitted, self.control.as_ref())
                .await
                .map_err(|_| ToolExecutionCancellationCompositionError::ControlFailure)?
        };
        if let Some(release) = release {
            let _ = release.send(());
        }
        if let Some(completed) = completed {
            if completed
                .await
                .map_err(|_| ToolExecutionCancellationCompositionError::RuntimeFailure)?
            {
                return Err(ToolExecutionCancellationCompositionError::RuntimeFailure);
            }
        }
        let expected_outcome = match self.capability.semantics {
            ToolSemantics::Cancellable => CancellationTargetExecutionOutcome::Stopped,
            ToolSemantics::NonCancellable => {
                CancellationTargetExecutionOutcome::ReportedNonCancellable {
                    owned_task_count: 1,
                }
            }
        };
        let expected_kind = match self.capability.semantics {
            ToolSemantics::Cancellable => ToolCancellationOutcomeKind::Accepted,
            ToolSemantics::NonCancellable => ToolCancellationOutcomeKind::DeclaredNonCancellable,
        };
        #[cfg(test)]
        let injected_evidence_failure = matches!(self.fault, Some(TestFault::Evidence));
        #[cfg(not(test))]
        let injected_evidence_failure = false;
        if runtime.target_outcomes().len() != 1
            || runtime.target_outcomes()[0].target() != CancellationTarget::ToolExecution
            || runtime.target_outcomes()[0].outcome() != expected_outcome
            || runtime.accepted_unclassified_task_count() != 0
            || cancellation.kind != expected_kind
            || cancellation.association != *self.admitted.association()
            || injected_evidence_failure
        {
            return Err(ToolExecutionCancellationCompositionError::RuntimeFailure);
        }
        Ok(ToolExecutionCancellationEvidence {
            runtime,
            cancellation,
            association: self.admitted.association().clone(),
            risk: self.admitted.risk(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexa_domain::{
        EnvironmentInstanceId, LabSessionId, ProtocolVersion, SemanticKey, ToolExecutionId,
        ToolRequestId,
    };
    use nexa_labs::*;
    use nexa_orchestrator::CancellationDirective;
    use uuid::Uuid;

    fn id<T>(n: u128, f: impl Fn(Uuid) -> Result<T, nexa_domain::ValueError>) -> T {
        f(Uuid::from_u128(n)).unwrap()
    }
    fn association() -> ToolAssociation {
        ToolAssociation {
            lab_session_id: id(11, LabSessionId::new),
            tool_request_id: id(12, ToolRequestId::new),
            tool_execution_id: id(13, ToolExecutionId::new),
            environment_instance_id: id(14, EnvironmentInstanceId::new),
            tool: SemanticKey::new("sensitive-tool").unwrap(),
            operation: SemanticKey::new("sensitive-operation").unwrap(),
            request_content_digest: RequestContentDigest::new([0xa5; 32]),
        }
    }
    fn admission() -> ToolAdmissionRequest {
        let a = association();
        ToolAdmissionRequest {
            contract_version: TOOL_EXECUTION_SECURITY_V1,
            association: a.clone(),
            sandbox: SandboxDeclaration {
                contract_version: TOOL_EXECUTION_SECURITY_V1,
                association: a.clone(),
                host_filesystem_access: false,
                host_network_access: false,
                privileged: false,
                root: false,
                bounds: ResourceBounds {
                    cpu_millis: 1,
                    memory_bytes: 1,
                    storage_bytes: 1,
                    process_count: 1,
                    execution_time_millis: 1,
                    output_bytes: 1,
                },
                network_policy: NetworkPolicy::DenyAll,
                authorized_mounts: vec![],
                authorized_capabilities: vec![],
            },
            risk_classification: RiskClassificationEvidence {
                contract_version: TOOL_EXECUTION_SECURITY_V1,
                association: a.clone(),
                risk: RiskClass::ReadOnly,
            },
            authorization: AuthorizationDecision {
                contract_version: TOOL_EXECUTION_SECURITY_V1,
                association: a.clone(),
                risk: RiskClass::ReadOnly,
                decision: PolicyDecision::Allow,
            },
            assessment: AssessmentDecision {
                contract_version: TOOL_EXECUTION_SECURITY_V1,
                association: a,
                risk: RiskClass::ReadOnly,
                decision: PolicyDecision::Allow,
            },
            confirmation: None,
            tutor_preference: TutorPreference::Prefer,
        }
    }
    fn workflow(cancelled: bool) -> InteractionWorkflow {
        let w = InteractionWorkflow::new(
            id(1, WorkflowId::new),
            id(2, SessionId::new),
            id(3, CorrelationId::new),
            id(4, TraceId::new),
        );
        if cancelled {
            w.cancel().unwrap()
        } else {
            w
        }
    }
    fn capability(semantics: ToolSemantics) -> ToolCancellationCapability {
        ToolCancellationCapability {
            contract_version: TOOL_EXECUTION_SECURITY_V1,
            association: association(),
            semantics,
        }
    }
    fn control(
        outcomes: impl IntoIterator<Item = ScriptedCancellationOutcome>,
    ) -> ScriptedToolCancellationControl {
        ScriptedToolCancellationControl::new(outcomes)
    }
    fn composition(
        semantics: ToolSemantics,
        outcomes: impl IntoIterator<Item = ScriptedCancellationOutcome>,
    ) -> ToolExecutionCancellationComposition<ScriptedToolCancellationControl> {
        let w = workflow(true);
        ToolExecutionCancellationComposition::new(
            w,
            w.workflow_id(),
            w.session_id(),
            w.correlation_id(),
            w.trace_id(),
            admission(),
            capability(semantics),
            control(outcomes),
        )
        .unwrap()
    }

    #[test]
    fn construction_rejects_workflow_identity_admission_capability_and_version_preflight() {
        let w = workflow(false);
        let c = control([]);
        assert!(matches!(
            ToolExecutionCancellationComposition::new(
                w,
                w.workflow_id(),
                w.session_id(),
                w.correlation_id(),
                w.trace_id(),
                admission(),
                capability(ToolSemantics::Cancellable),
                c
            ),
            Err(ToolExecutionCancellationCompositionError::InvalidWorkflow)
        ));
        let w = workflow(true);
        for n in 0..4 {
            let mut ids = (
                w.workflow_id(),
                w.session_id(),
                w.correlation_id(),
                w.trace_id(),
            );
            match n {
                0 => ids.0 = id(21, WorkflowId::new),
                1 => ids.1 = id(22, SessionId::new),
                2 => ids.2 = id(23, CorrelationId::new),
                _ => ids.3 = id(24, TraceId::new),
            }
            assert!(matches!(
                ToolExecutionCancellationComposition::new(
                    w,
                    ids.0,
                    ids.1,
                    ids.2,
                    ids.3,
                    admission(),
                    capability(ToolSemantics::Cancellable),
                    control([])
                ),
                Err(ToolExecutionCancellationCompositionError::AssociationMismatch)
            ));
        }
        let mut denied = admission();
        denied.assessment.decision = PolicyDecision::Deny;
        assert!(matches!(
            ToolExecutionCancellationComposition::new(
                w,
                w.workflow_id(),
                w.session_id(),
                w.correlation_id(),
                w.trace_id(),
                denied,
                capability(ToolSemantics::Cancellable),
                control([])
            ),
            Err(ToolExecutionCancellationCompositionError::AdmissionRejected)
        ));
        let mut mismatched = capability(ToolSemantics::Cancellable);
        mismatched.association.request_content_digest = RequestContentDigest::new([8; 32]);
        assert!(matches!(
            ToolExecutionCancellationComposition::new(
                w,
                w.workflow_id(),
                w.session_id(),
                w.correlation_id(),
                w.trace_id(),
                admission(),
                mismatched,
                control([])
            ),
            Err(ToolExecutionCancellationCompositionError::AssociationMismatch)
        ));
        let mut unsupported = capability(ToolSemantics::Cancellable);
        unsupported.contract_version = ProtocolVersion::new(2, 0);
        assert!(matches!(
            ToolExecutionCancellationComposition::new(
                w,
                w.workflow_id(),
                w.session_id(),
                w.correlation_id(),
                w.trace_id(),
                admission(),
                unsupported,
                control([])
            ),
            Err(ToolExecutionCancellationCompositionError::UnsupportedVersion)
        ));
    }

    #[test]
    fn both_semantics_produce_the_exact_singleton_plan() {
        for (semantics, directive) in [
            (
                ToolSemantics::Cancellable,
                CancellationDirective::RequestCancellation,
            ),
            (
                ToolSemantics::NonCancellable,
                CancellationDirective::ReportNonCancellable,
            ),
        ] {
            let c = composition(semantics, []);
            assert_eq!(c.plan().directives().len(), 1);
            assert_eq!(
                c.plan().directives()[0].target(),
                CancellationTarget::ToolExecution
            );
            assert_eq!(c.plan().directives()[0].directive(), directive);
            assert_eq!(
                c.inspect_control(|v| (v.received().len(), v.remaining_outcomes())),
                (0, 0)
            );
        }
    }

    #[tokio::test]
    async fn cancellable_success_is_exact_and_idempotent() {
        let ack = ToolCancellationAcknowledgement {
            contract_version: TOOL_EXECUTION_SECURITY_V1,
            association: association(),
        };
        let mut c = composition(
            ToolSemantics::Cancellable,
            [ScriptedCancellationOutcome::Acknowledged(ack)],
        );
        let evidence = c
            .execute(capability(ToolSemantics::Cancellable))
            .await
            .unwrap();
        assert_eq!(
            evidence.cancellation().kind,
            ToolCancellationOutcomeKind::Accepted
        );
        assert_eq!(evidence.association(), &association());
        assert_eq!(
            evidence.runtime().target_outcomes()[0].outcome(),
            CancellationTargetExecutionOutcome::Stopped
        );
        assert_eq!(
            c.inspect_control(|v| (
                v.received().len(),
                v.remaining_outcomes(),
                v.active_futures()
            )),
            (1, 0, 0)
        );
        let diagnostic = format!("{evidence:?}");
        assert!(!diagnostic.contains("sensitive-tool"));
        assert!(!diagnostic.contains("sensitive-operation"));
        assert!(!diagnostic.contains("165"));
        assert_eq!(
            c.execute(capability(ToolSemantics::Cancellable))
                .await
                .unwrap(),
            evidence
        );
        assert_eq!(c.inspect_control(|v| v.received().len()), 1);
        let mut conflicting = capability(ToolSemantics::Cancellable);
        conflicting.association.operation = SemanticKey::new("conflict").unwrap();
        assert_eq!(
            c.execute(conflicting).await,
            Err(ToolExecutionCancellationCompositionError::ConflictingExecution)
        );
    }

    #[tokio::test]
    async fn failures_terminalize_without_additional_dependency_work() {
        for outcomes in [vec![], vec![ScriptedCancellationOutcome::DependencyFailure]] {
            let mut c = composition(ToolSemantics::Cancellable, outcomes);
            assert_eq!(
                c.execute(capability(ToolSemantics::Cancellable)).await,
                Err(ToolExecutionCancellationCompositionError::ControlFailure)
            );
            let count = c.inspect_control(|v| v.received().len());
            assert_eq!(
                c.execute(capability(ToolSemantics::Cancellable)).await,
                Err(ToolExecutionCancellationCompositionError::RuntimeFailure)
            );
            assert_eq!(c.inspect_control(|v| v.received().len()), count);
        }
        for fault in [TestFault::Join, TestFault::Coverage, TestFault::Evidence] {
            let ack = ToolCancellationAcknowledgement {
                contract_version: TOOL_EXECUTION_SECURITY_V1,
                association: association(),
            };
            let mut c = composition(
                ToolSemantics::Cancellable,
                [ScriptedCancellationOutcome::Acknowledged(ack)],
            );
            c.fault = Some(fault);
            assert!(c
                .execute(capability(ToolSemantics::Cancellable))
                .await
                .is_err());
            let count = c.inspect_control(|v| v.received().len());
            assert!(c
                .execute(capability(ToolSemantics::Cancellable))
                .await
                .is_err());
            assert_eq!(c.inspect_control(|v| v.received().len()), count);
        }
    }

    #[tokio::test]
    async fn non_cancellable_reports_owned_work_without_control_request() {
        let mut c = composition(
            ToolSemantics::NonCancellable,
            [ScriptedCancellationOutcome::DependencyFailure],
        );
        let evidence = c
            .execute(capability(ToolSemantics::NonCancellable))
            .await
            .unwrap();
        assert_eq!(
            evidence.cancellation().kind,
            ToolCancellationOutcomeKind::DeclaredNonCancellable
        );
        assert_eq!(
            evidence.runtime().target_outcomes()[0].outcome(),
            CancellationTargetExecutionOutcome::ReportedNonCancellable {
                owned_task_count: 1
            }
        );
        assert_eq!(
            c.inspect_control(|v| (v.received().len(), v.remaining_outcomes())),
            (0, 1)
        );
        assert_eq!(
            c.execute(capability(ToolSemantics::NonCancellable))
                .await
                .unwrap(),
            evidence
        );
    }

    #[test]
    fn diagnostics_are_closed_and_digest_is_redacted() {
        for error in [
            ToolExecutionCancellationCompositionError::InvalidWorkflow,
            ToolExecutionCancellationCompositionError::AssociationMismatch,
            ToolExecutionCancellationCompositionError::AdmissionRejected,
            ToolExecutionCancellationCompositionError::UnsupportedVersion,
            ToolExecutionCancellationCompositionError::ControlFailure,
            ToolExecutionCancellationCompositionError::RuntimeFailure,
            ToolExecutionCancellationCompositionError::ConflictingExecution,
        ] {
            let text = format!("{error:?} {error}");
            assert!(!text.contains("sensitive"));
            assert!(!text.contains("165"));
        }
    }
}
