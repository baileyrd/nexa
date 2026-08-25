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
    MissingResult,
    RuntimeEvidence,
    HoldNonCancellable,
    HoldBeforePlan,
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
                    #[cfg(test)]
                    if matches!(fault, Some(TestFault::HoldNonCancellable)) {
                        std::future::pending::<()>().await;
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
                    #[cfg(test)]
                    let record_result = !matches!(fault, Some(TestFault::MissingResult));
                    #[cfg(not(test))]
                    let record_result = true;
                    if record_result {
                        if let Ok(mut slot) = task_observed.lock() {
                            *slot = Some(result);
                        }
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
        if matches!(self.fault, Some(TestFault::HoldBeforePlan)) {
            std::future::pending::<()>().await;
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
        #[cfg(test)]
        let injected_runtime_failure = matches!(self.fault, Some(TestFault::RuntimeEvidence));
        #[cfg(not(test))]
        let injected_runtime_failure = false;
        if runtime.target_outcomes().len() != 1
            || runtime.target_outcomes()[0].target() != CancellationTarget::ToolExecution
            || runtime.target_outcomes()[0].outcome() != expected_outcome
            || runtime.accepted_unclassified_task_count() != 0
            || cancellation.kind != expected_kind
            || cancellation.association != *self.admitted.association()
            || injected_evidence_failure
            || injected_runtime_failure
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
        let mut conflicting = capability(ToolSemantics::NonCancellable);
        reassociate(&mut conflicting.association, 0);
        assert_eq!(
            c.execute(conflicting).await,
            Err(ToolExecutionCancellationCompositionError::ConflictingExecution)
        );
        assert_eq!(
            c.inspect_control(|v| (
                v.received().len(),
                v.remaining_outcomes(),
                v.active_futures()
            )),
            (0, 1, 0)
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

    fn workflow_in(state: WorkflowState) -> InteractionWorkflow {
        use WorkflowState::*;
        let mut w = workflow(false);
        if state == Created {
            return w;
        }
        if state == Failed {
            return w.fail().unwrap();
        }
        for next in [
            NormalizingInput,
            PreparingContext,
            SelectingPedagogy,
            RetrievingKnowledge,
            GeneratingTutorResponse,
            ExecutingTools,
            PlanningResponse,
            Speaking,
            WaitingForStudent,
            Completed,
        ] {
            w = w.advance(next).unwrap();
            if next == state {
                return w;
            }
        }
        unreachable!()
    }

    fn assert_preflight_rejected(
        w: InteractionWorkflow,
        admission: ToolAdmissionRequest,
        capability: ToolCancellationCapability,
        expected: ToolExecutionCancellationCompositionError,
    ) {
        let control = control([ScriptedCancellationOutcome::DependencyFailure]);
        let observer = control.clone();
        let result = ToolExecutionCancellationComposition::new(
            w,
            w.workflow_id(),
            w.session_id(),
            w.correlation_id(),
            w.trace_id(),
            admission,
            capability,
            control,
        );
        assert!(matches!(result, Err(error) if error == expected));
        assert_eq!(
            (
                observer.received().len(),
                observer.remaining_outcomes(),
                observer.active_futures(),
                observer.dropped_futures(),
            ),
            (0, 1, 0, 0),
            "preflight must not create or consume dependency/runtime work"
        );
    }

    #[test]
    fn every_non_cancelled_workflow_state_is_rejected_without_side_effects() {
        use WorkflowState::*;
        for state in [
            Created,
            NormalizingInput,
            PreparingContext,
            SelectingPedagogy,
            RetrievingKnowledge,
            GeneratingTutorResponse,
            ExecutingTools,
            PlanningResponse,
            Speaking,
            WaitingForStudent,
            Completed,
            Failed,
        ] {
            assert_preflight_rejected(
                workflow_in(state),
                admission(),
                capability(ToolSemantics::Cancellable),
                ToolExecutionCancellationCompositionError::InvalidWorkflow,
            );
        }
    }

    fn reassociate(a: &mut ToolAssociation, field: usize) {
        match field {
            0 => a.lab_session_id = id(31, LabSessionId::new),
            1 => a.tool_request_id = id(32, ToolRequestId::new),
            2 => a.tool_execution_id = id(33, ToolExecutionId::new),
            3 => a.environment_instance_id = id(34, EnvironmentInstanceId::new),
            4 => a.tool = SemanticKey::new("other-tool").unwrap(),
            5 => a.operation = SemanticKey::new("other-operation").unwrap(),
            6 => a.request_content_digest = RequestContentDigest::new([0x5a; 32]),
            _ => unreachable!(),
        }
    }

    #[test]
    fn every_capability_association_field_and_version_is_preflighted_without_work() {
        for field in 0..7 {
            let mut cap = capability(ToolSemantics::Cancellable);
            reassociate(&mut cap.association, field);
            assert_preflight_rejected(
                workflow(true),
                admission(),
                cap,
                ToolExecutionCancellationCompositionError::AssociationMismatch,
            );
        }
        let mut cap = capability(ToolSemantics::Cancellable);
        cap.contract_version = ProtocolVersion::new(9, 9);
        assert_preflight_rejected(
            workflow(true),
            admission(),
            cap,
            ToolExecutionCancellationCompositionError::UnsupportedVersion,
        );
    }

    #[test]
    fn every_workflow_identity_reference_is_independently_side_effect_free() {
        let w = workflow(true);
        for field in 0..4 {
            let mut ids = (
                w.workflow_id(),
                w.session_id(),
                w.correlation_id(),
                w.trace_id(),
            );
            match field {
                0 => ids.0 = id(41, WorkflowId::new),
                1 => ids.1 = id(42, SessionId::new),
                2 => ids.2 = id(43, CorrelationId::new),
                _ => ids.3 = id(44, TraceId::new),
            }
            let dependency = control([ScriptedCancellationOutcome::DependencyFailure]);
            let observer = dependency.clone();
            let result = ToolExecutionCancellationComposition::new(
                w,
                ids.0,
                ids.1,
                ids.2,
                ids.3,
                admission(),
                capability(ToolSemantics::Cancellable),
                dependency,
            );
            assert!(matches!(
                result,
                Err(ToolExecutionCancellationCompositionError::AssociationMismatch)
            ));
            assert_eq!(
                (
                    observer.received().len(),
                    observer.remaining_outcomes(),
                    observer.active_futures(),
                    observer.dropped_futures()
                ),
                (0, 1, 0, 0)
            );
        }
    }

    #[test]
    fn every_admission_rejection_category_is_closed_and_side_effect_free() {
        let mut cases = Vec::new();
        let mut r = admission();
        r.contract_version = ProtocolVersion::new(2, 0);
        cases.push(r);
        for unrestricted in 0..4 {
            let mut r = admission();
            match unrestricted {
                0 => r.sandbox.host_filesystem_access = true,
                1 => r.sandbox.host_network_access = true,
                2 => r.sandbox.privileged = true,
                _ => r.sandbox.root = true,
            }
            cases.push(r);
        }
        for bound in 0..6 {
            let mut r = admission();
            match bound {
                0 => r.sandbox.bounds.cpu_millis = 0,
                1 => r.sandbox.bounds.memory_bytes = 0,
                2 => r.sandbox.bounds.storage_bytes = 0,
                3 => r.sandbox.bounds.process_count = 0,
                4 => r.sandbox.bounds.execution_time_millis = 0,
                _ => r.sandbox.bounds.output_bytes = 0,
            }
            cases.push(r);
        }
        let mut inconsistent = admission();
        inconsistent.sandbox.network_policy = NetworkPolicy::AllowListed { targets: vec![] };
        cases.push(inconsistent);
        for member in 0..4 {
            let mut r = admission();
            match member {
                0 => reassociate(&mut r.sandbox.association, 0),
                1 => reassociate(&mut r.risk_classification.association, 1),
                2 => reassociate(&mut r.authorization.association, 2),
                _ => reassociate(&mut r.assessment.association, 3),
            }
            cases.push(r);
        }
        for member in 0..2 {
            let mut r = admission();
            if member == 0 {
                r.authorization.risk = RiskClass::Mutating;
            } else {
                r.assessment.risk = RiskClass::Mutating;
            }
            cases.push(r);
        }
        for member in 0..2 {
            let mut r = admission();
            if member == 0 {
                r.authorization.decision = PolicyDecision::Deny;
            } else {
                r.assessment.decision = PolicyDecision::Deny;
            }
            r.tutor_preference = TutorPreference::Prefer;
            cases.push(r);
        }
        for risk in [RiskClass::Destructive, RiskClass::Privileged] {
            let mut r = admission();
            r.risk_classification.risk = risk;
            r.authorization.risk = risk;
            r.assessment.risk = risk;
            cases.push(r);
        }
        let mut required = admission();
        required.authorization.decision = PolicyDecision::ConfirmationRequired;
        cases.push(required);
        for request in cases {
            assert_preflight_rejected(
                workflow(true),
                request,
                capability(ToolSemantics::Cancellable),
                ToolExecutionCancellationCompositionError::AdmissionRejected,
            );
        }
    }

    #[tokio::test]
    async fn cancellable_waits_for_token_and_caller_drop_aborts_pending_control() {
        let mut waiting = composition(
            ToolSemantics::Cancellable,
            [ScriptedCancellationOutcome::DependencyFailure],
        );
        waiting.fault = Some(TestFault::HoldBeforePlan);
        let waiting_observer = waiting.inspect_control(Clone::clone);
        let mut before_token = Box::pin(waiting.execute(capability(ToolSemantics::Cancellable)));
        tokio::select! {
            biased;
            result = &mut before_token => panic!("pre-token barrier completed: {result:?}"),
            _ = tokio::task::yield_now() => {}
        }
        assert_eq!(
            (
                waiting_observer.received().len(),
                waiting_observer.remaining_outcomes(),
                waiting_observer.active_futures()
            ),
            (0, 1, 0),
            "the dependency must remain untouched before plan-driven token cancellation"
        );
        drop(before_token);

        let mut c = composition(
            ToolSemantics::Cancellable,
            [ScriptedCancellationOutcome::Pending],
        );
        let observer = c.inspect_control(Clone::clone);
        let mut execution = Box::pin(c.execute(capability(ToolSemantics::Cancellable)));
        tokio::select! {
            biased;
            result = &mut execution => panic!("pending dependency completed: {result:?}"),
            _ = tokio::task::yield_now() => {}
        }
        assert_eq!(
            (
                observer.received().len(),
                observer.remaining_outcomes(),
                observer.active_futures()
            ),
            (1, 0, 1)
        );
        drop(execution);
        tokio::task::yield_now().await;
        assert_eq!(
            (
                observer.received().len(),
                observer.active_futures(),
                observer.dropped_futures()
            ),
            (1, 0, 1)
        );
    }

    #[tokio::test]
    async fn acknowledgements_reject_version_and_every_reassociated_field_terminally() {
        let mut acknowledgements = Vec::new();
        let mut wrong_version = ToolCancellationAcknowledgement {
            contract_version: ProtocolVersion::new(2, 0),
            association: association(),
        };
        acknowledgements.push(wrong_version.clone());
        wrong_version.contract_version = TOOL_EXECUTION_SECURITY_V1;
        for field in 0..7 {
            let mut ack = wrong_version.clone();
            reassociate(&mut ack.association, field);
            acknowledgements.push(ack);
        }
        for ack in acknowledgements {
            let mut c = composition(
                ToolSemantics::Cancellable,
                [ScriptedCancellationOutcome::Acknowledged(ack)],
            );
            assert_eq!(
                c.execute(capability(ToolSemantics::Cancellable)).await,
                Err(ToolExecutionCancellationCompositionError::ControlFailure)
            );
            assert_eq!(
                c.inspect_control(|v| (
                    v.received().len(),
                    v.remaining_outcomes(),
                    v.active_futures()
                )),
                (1, 0, 0)
            );
            assert_eq!(
                c.execute(capability(ToolSemantics::Cancellable)).await,
                Err(ToolExecutionCancellationCompositionError::RuntimeFailure)
            );
            assert_eq!(c.inspect_control(|v| v.received().len()), 1);
        }
    }

    #[tokio::test]
    async fn all_runtime_failure_categories_are_terminal_and_accounted() {
        for fault in [
            TestFault::Join,
            TestFault::Coverage,
            TestFault::Evidence,
            TestFault::MissingResult,
            TestFault::RuntimeEvidence,
        ] {
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
            let accounting = c.inspect_control(|v| {
                (
                    v.received().len(),
                    v.remaining_outcomes(),
                    v.active_futures(),
                )
            });
            assert_eq!(accounting.2, 0);
            assert_eq!(
                c.execute(capability(ToolSemantics::Cancellable)).await,
                Err(ToolExecutionCancellationCompositionError::RuntimeFailure)
            );
            assert_eq!(
                c.inspect_control(|v| (
                    v.received().len(),
                    v.remaining_outcomes(),
                    v.active_futures()
                )),
                accounting
            );
        }
    }

    #[tokio::test]
    async fn non_cancellable_drop_and_failures_never_invoke_control() {
        let mut dropped = composition(
            ToolSemantics::NonCancellable,
            [ScriptedCancellationOutcome::DependencyFailure],
        );
        dropped.fault = Some(TestFault::HoldNonCancellable);
        let mut execution = Box::pin(dropped.execute(capability(ToolSemantics::NonCancellable)));
        tokio::select! {
            biased;
            result = &mut execution => panic!("held placeholder completed: {result:?}"),
            _ = tokio::task::yield_now() => {}
        }
        drop(execution);
        assert_eq!(
            dropped.inspect_control(|v| (
                v.received().len(),
                v.remaining_outcomes(),
                v.active_futures()
            )),
            (0, 1, 0)
        );
        for fault in [
            TestFault::Join,
            TestFault::Coverage,
            TestFault::Evidence,
            TestFault::RuntimeEvidence,
        ] {
            let mut c = composition(
                ToolSemantics::NonCancellable,
                [ScriptedCancellationOutcome::DependencyFailure],
            );
            c.fault = Some(fault);
            assert!(c
                .execute(capability(ToolSemantics::NonCancellable))
                .await
                .is_err());
            assert_eq!(
                c.inspect_control(|v| (
                    v.received().len(),
                    v.remaining_outcomes(),
                    v.active_futures()
                )),
                (0, 1, 0)
            );
        }
    }

    #[tokio::test]
    async fn exact_admitted_association_is_reused_and_security_rules_are_preserved() {
        let admitted_association = association();
        let ack = ToolCancellationAcknowledgement {
            contract_version: TOOL_EXECUTION_SECURITY_V1,
            association: admitted_association.clone(),
        };
        let mut c = composition(
            ToolSemantics::Cancellable,
            [ScriptedCancellationOutcome::Acknowledged(ack)],
        );
        let evidence = c
            .execute(capability(ToolSemantics::Cancellable))
            .await
            .unwrap();
        assert_eq!(evidence.association(), &admitted_association);
        assert_eq!(
            c.inspect_control(|v| v.received()[0].association.clone()),
            admitted_association
        );

        for decision_owner in 0..2 {
            let mut denied = admission();
            denied.tutor_preference = TutorPreference::Prefer;
            if decision_owner == 0 {
                denied.authorization.decision = PolicyDecision::Deny;
            } else {
                denied.assessment.decision = PolicyDecision::Deny;
            }
            assert!(admit_tool_execution(&denied).is_err());
        }
        for risk in [RiskClass::Destructive, RiskClass::Privileged] {
            let mut request = admission();
            request.risk_classification.risk = risk;
            request.authorization.risk = risk;
            request.assessment.risk = risk;
            assert!(admit_tool_execution(&request).is_err());
            request.confirmation = Some(ConfirmationEvidence {
                contract_version: TOOL_EXECUTION_SECURITY_V1,
                association: request.association.clone(),
                risk,
                authorization_decision: PolicyDecision::Allow,
                assessment_decision: PolicyDecision::Allow,
                confirmed: true,
            });
            assert!(admit_tool_execution(&request).is_ok());
        }
        let mut policy_required = admission();
        policy_required.assessment.decision = PolicyDecision::ConfirmationRequired;
        assert!(admit_tool_execution(&policy_required).is_err());
        policy_required.confirmation = Some(ConfirmationEvidence {
            contract_version: TOOL_EXECUTION_SECURITY_V1,
            association: policy_required.association.clone(),
            risk: RiskClass::ReadOnly,
            authorization_decision: PolicyDecision::Allow,
            assessment_decision: PolicyDecision::ConfirmationRequired,
            confirmed: true,
        });
        assert!(admit_tool_execution(&policy_required).is_ok());
    }
}
