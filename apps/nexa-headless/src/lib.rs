//! Minimal headless composition of workflow-owned Behavior cancellation.
#![forbid(unsafe_code)]

mod tutor_cancellation;
pub use tutor_cancellation::{
    TutorGenerationCancellationComposition, TutorGenerationCancellationCompositionError,
    TutorGenerationCancellationEvidence,
};

use nexa_avatar::{AvatarPort, AvatarReport, AvatarRequest};
use nexa_domain::{CorrelationId, MessageId, SessionId, TraceId, WorkflowId};
use nexa_nbp::{AvatarCapability, BehaviorCancel, RuntimeStatus};
use nexa_orchestrator::{
    plan_workflow_cancellation, ActiveCancellationTarget, CancellationSemantics,
    CancellationTarget, InteractionWorkflow, WorkflowCancellationPlan,
};
use nexa_orchestrator_runtime::{
    WorkflowCancellationExecution, WorkflowTaskGroup, WorkflowTaskGroupError,
};
use std::sync::{Arc, Mutex};

/// Closed, content-free failures at the application composition boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BehaviorCancellationError {
    InvalidWorkflow,
    AssociationMismatch,
    CapabilityUnavailable,
    InvalidPreview,
    RuntimeFailure,
    ConflictingExecution,
}

impl std::fmt::Display for BehaviorCancellationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidWorkflow => "invalid cancelled workflow",
            Self::AssociationMismatch => "behavior cancellation association mismatch",
            Self::CapabilityUnavailable => "avatar cancellation capability unavailable",
            Self::InvalidPreview => "invalid avatar cancellation preview",
            Self::RuntimeFailure => "behavior cancellation runtime failure",
            Self::ConflictingExecution => "conflicting behavior cancellation execution",
        })
    }
}

impl std::error::Error for BehaviorCancellationError {}

#[derive(Clone, Debug, PartialEq)]
pub struct AvatarCancellationEvidence {
    requests: Vec<AvatarRequest>,
    report: AvatarReport,
}

impl AvatarCancellationEvidence {
    pub fn request(&self) -> &AvatarRequest {
        &self.requests[0]
    }
    pub const fn report(&self) -> &AvatarReport {
        &self.report
    }
    pub fn requests(&self) -> &[AvatarRequest] {
        &self.requests
    }
    pub fn cancellation_request_count(&self) -> usize {
        self.requests
            .iter()
            .filter(|request| matches!(request, AvatarRequest::Cancel { .. }))
            .count()
    }
    pub fn submit_request_count(&self) -> usize {
        self.requests
            .iter()
            .filter(|request| matches!(request, AvatarRequest::Submit { .. }))
            .count()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BehaviorCancellationEvidence {
    runtime: WorkflowCancellationExecution,
    avatar: AvatarCancellationEvidence,
}

impl BehaviorCancellationEvidence {
    pub const fn runtime(&self) -> &WorkflowCancellationExecution {
        &self.runtime
    }
    pub const fn avatar(&self) -> &AvatarCancellationEvidence {
        &self.avatar
    }
}

/// Owns one adapter and one exact Behavior-cancellation operation.
pub struct BehaviorCancellationComposition<A> {
    workflow: InteractionWorkflow,
    request: AvatarRequest,
    plan: WorkflowCancellationPlan,
    adapter: Arc<Mutex<A>>,
    preview: AvatarReport,
    terminal: ExecutionState,
    #[cfg(test)]
    lifecycle_probe: Option<TestLifecycleProbe>,
}

#[cfg(test)]
struct TestLifecycleProbe {
    started_waiting: std::sync::mpsc::Sender<()>,
    allow_cancellation: tokio::sync::oneshot::Receiver<()>,
    task_dropped: std::sync::mpsc::Sender<()>,
}

#[cfg(test)]
struct TestTaskDropProbe(std::sync::mpsc::Sender<()>);

#[cfg(test)]
impl Drop for TestTaskDropProbe {
    fn drop(&mut self) {
        let _ = self.0.send(());
    }
}

enum ExecutionState {
    Ready,
    Running,
    Succeeded(Box<BehaviorCancellationEvidence>),
    Failed,
}

impl<A: AvatarPort + Send + 'static> BehaviorCancellationComposition<A> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        workflow: InteractionWorkflow,
        workflow_id: WorkflowId,
        session_id: SessionId,
        correlation_id: CorrelationId,
        trace_id: TraceId,
        message_id: MessageId,
        cancellation: BehaviorCancel,
        adapter: A,
    ) -> Result<Self, BehaviorCancellationError> {
        if workflow.state() != nexa_orchestrator::WorkflowState::Cancelled {
            return Err(BehaviorCancellationError::InvalidWorkflow);
        }
        if (
            workflow.workflow_id(),
            workflow.session_id(),
            workflow.correlation_id(),
            workflow.trace_id(),
        ) != (workflow_id, session_id, correlation_id, trace_id)
        {
            return Err(BehaviorCancellationError::AssociationMismatch);
        }
        if !adapter
            .capabilities()
            .supports(AvatarCapability::Cancellation)
        {
            return Err(BehaviorCancellationError::CapabilityUnavailable);
        }
        let request = AvatarRequest::Cancel {
            message_id,
            cancellation,
        };
        let preview = adapter.preview(&request);
        let behavior_id = match &request {
            AvatarRequest::Cancel { cancellation, .. } => cancellation.behavior_id,
            AvatarRequest::Submit { .. } => unreachable!(),
        };
        if preview.message_id != message_id
            || preview.behavior_id != behavior_id
            || preview.terminal_status() != RuntimeStatus::Cancelled
        {
            return Err(BehaviorCancellationError::InvalidPreview);
        }
        let plan = plan_workflow_cancellation(
            &workflow,
            workflow_id,
            session_id,
            correlation_id,
            trace_id,
            &[ActiveCancellationTarget::new(
                CancellationTarget::Behavior,
                CancellationSemantics::Cancellable,
            )],
        )
        .map_err(|_| BehaviorCancellationError::InvalidWorkflow)?;
        Ok(Self {
            workflow,
            request,
            plan,
            adapter: Arc::new(Mutex::new(adapter)),
            preview,
            terminal: ExecutionState::Ready,
            #[cfg(test)]
            lifecycle_probe: None,
        })
    }

    pub const fn plan(&self) -> &WorkflowCancellationPlan {
        &self.plan
    }

    /// Provides bounded, read-only access to the owned adapter without exposing its lock.
    pub fn inspect_adapter<R>(
        &self,
        inspect: impl FnOnce(&A) -> R,
    ) -> Result<R, BehaviorCancellationError> {
        self.adapter
            .lock()
            .map(|adapter| inspect(&adapter))
            .map_err(|_| BehaviorCancellationError::RuntimeFailure)
    }

    pub async fn execute(
        &mut self,
        message_id: MessageId,
        cancellation: BehaviorCancel,
    ) -> Result<BehaviorCancellationEvidence, BehaviorCancellationError> {
        let requested = AvatarRequest::Cancel {
            message_id,
            cancellation,
        };
        if requested != self.request {
            return Err(BehaviorCancellationError::ConflictingExecution);
        }
        match &self.terminal {
            ExecutionState::Succeeded(evidence) => return Ok((**evidence).clone()),
            ExecutionState::Running | ExecutionState::Failed => {
                return Err(BehaviorCancellationError::RuntimeFailure)
            }
            ExecutionState::Ready => {}
        }

        // Terminalize before any task is spawned. If this caller future is dropped, retrying
        // cannot start another operation and the local task group's Drop aborts owned work.
        self.terminal = ExecutionState::Running;

        let mut tasks = WorkflowTaskGroup::new(self.workflow);
        let adapter = Arc::clone(&self.adapter);
        let request = self.request.clone();
        let invocation = Arc::new(Mutex::new(None));
        let task_invocation = Arc::clone(&invocation);
        #[cfg(test)]
        let task_probe = self
            .lifecycle_probe
            .as_ref()
            .map(|probe| (probe.started_waiting.clone(), probe.task_dropped.clone()));
        if tasks
            .spawn_for_target(CancellationTarget::Behavior, move |token| async move {
                #[cfg(test)]
                let _drop_probe = if let Some((started_waiting, task_dropped)) = task_probe {
                    let guard = TestTaskDropProbe(task_dropped);
                    let mut cancellation_wait = Box::pin(token.cancelled());
                    let mut started_waiting = Some(started_waiting);
                    std::future::poll_fn(|context| {
                        let status = std::future::Future::poll(cancellation_wait.as_mut(), context);
                        if status.is_pending() {
                            if let Some(started_waiting) = started_waiting.take() {
                                let _ = started_waiting.send(());
                            }
                        }
                        status
                    })
                    .await;
                    Some(guard)
                } else {
                    token.cancelled().await;
                    None
                };
                #[cfg(not(test))]
                token.cancelled().await;
                let invoked_request = request.clone();
                let result = match adapter.lock() {
                    Ok(mut adapter) => adapter.handle(request),
                    Err(_) => return,
                };
                if let Ok(mut invocation) = task_invocation.lock() {
                    *invocation = Some((invoked_request, result));
                }
            })
            .is_err()
        {
            self.terminal = ExecutionState::Failed;
            return Err(BehaviorCancellationError::RuntimeFailure);
        }
        #[cfg(test)]
        if let Some(probe) = self.lifecycle_probe.take() {
            if probe.allow_cancellation.await.is_err() {
                self.terminal = ExecutionState::Failed;
                return Err(BehaviorCancellationError::RuntimeFailure);
            }
        }
        let runtime = match tasks.execute_cancellation_plan(&self.plan).await {
            Ok(runtime) => runtime,
            Err(error) => {
                self.terminal = ExecutionState::Failed;
                return Err(map_runtime_error(error));
            }
        };
        let actual = invocation.lock().ok().and_then(|value| value.clone());
        let Some((invoked_request, avatar_report)) = actual else {
            self.terminal = ExecutionState::Failed;
            return Err(BehaviorCancellationError::RuntimeFailure);
        };
        if invoked_request != self.request
            || avatar_report != self.preview
            || avatar_report.message_id != self.request.message_id()
            || avatar_report.terminal_status() != RuntimeStatus::Cancelled
        {
            self.terminal = ExecutionState::Failed;
            return Err(BehaviorCancellationError::RuntimeFailure);
        }
        let evidence = BehaviorCancellationEvidence {
            runtime,
            avatar: AvatarCancellationEvidence {
                requests: vec![invoked_request],
                report: avatar_report,
            },
        };
        self.terminal = ExecutionState::Succeeded(Box::new(evidence.clone()));
        Ok(evidence)
    }
}

fn map_runtime_error(error: WorkflowTaskGroupError) -> BehaviorCancellationError {
    match error {
        WorkflowTaskGroupError::ConflictingCompletion => {
            BehaviorCancellationError::ConflictingExecution
        }
        _ => BehaviorCancellationError::RuntimeFailure,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexa_avatar::{AvatarCapabilities, FakeAvatarAdapter};
    use nexa_domain::{BehaviorId, SemanticKey};
    use nexa_nbp::CancellationMode;
    use nexa_orchestrator::CancellationDirective;
    use nexa_orchestrator_runtime::CancellationTargetExecutionOutcome;
    use std::str::FromStr;
    use uuid::Uuid;

    fn ids() -> (WorkflowId, SessionId, CorrelationId, TraceId) {
        (
            WorkflowId::new(Uuid::from_u128(1)).unwrap(),
            SessionId::new(Uuid::from_u128(2)).unwrap(),
            CorrelationId::new(Uuid::from_u128(3)).unwrap(),
            TraceId::new(Uuid::from_u128(4)).unwrap(),
        )
    }

    fn message_id() -> MessageId {
        MessageId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249002").unwrap()
    }

    fn cancellation() -> BehaviorCancel {
        BehaviorCancel {
            behavior_id: BehaviorId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249001").unwrap(),
            reason: "new turn".into(),
            transition: CancellationMode::Graceful,
        }
    }

    fn workflow(cancelled: bool) -> InteractionWorkflow {
        let (workflow, session, correlation, trace) = ids();
        let value = InteractionWorkflow::new(workflow, session, correlation, trace);
        if cancelled {
            value.cancel().unwrap()
        } else {
            value
        }
    }

    fn adapter(capable: bool) -> FakeAvatarAdapter {
        FakeAvatarAdapter::new(
            SemanticKey::new("headless-avatar").unwrap(),
            AvatarCapabilities::new(capable.then_some(AvatarCapability::Cancellation)),
        )
    }

    fn composition() -> BehaviorCancellationComposition<FakeAvatarAdapter> {
        let (workflow_id, session_id, correlation_id, trace_id) = ids();
        BehaviorCancellationComposition::new(
            workflow(true),
            workflow_id,
            session_id,
            correlation_id,
            trace_id,
            message_id(),
            cancellation(),
            adapter(true),
        )
        .unwrap()
    }

    #[test]
    fn canonical_behavior_plan_executes_exactly_once_and_preserves_identity() {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                let mut composition = composition();
                assert_eq!(composition.plan().directives().len(), 1);
                assert_eq!(
                    composition.plan().directives()[0].target(),
                    CancellationTarget::Behavior
                );
                assert_eq!(
                    composition.plan().directives()[0].directive(),
                    CancellationDirective::RequestCancellation
                );

                let evidence = composition
                    .execute(message_id(), cancellation())
                    .await
                    .unwrap();
                assert_eq!(evidence.runtime().workflow(), workflow(true));
                assert_eq!(evidence.runtime().target_outcomes().len(), 1);
                assert_eq!(
                    evidence.runtime().target_outcomes()[0].outcome(),
                    CancellationTargetExecutionOutcome::Stopped
                );
                assert_eq!(evidence.avatar().cancellation_request_count(), 1);
                assert_eq!(evidence.avatar().submit_request_count(), 0);
                assert_eq!(evidence.avatar().report().message_id, message_id());
                assert_eq!(
                    evidence.avatar().report().behavior_id,
                    cancellation().behavior_id
                );
                assert_eq!(
                    evidence.avatar().report().terminal_status(),
                    RuntimeStatus::Cancelled
                );
                assert_eq!(
                    composition
                        .inspect_adapter(|adapter| adapter.requests().to_vec())
                        .unwrap(),
                    vec![AvatarRequest::Cancel {
                        message_id: message_id(),
                        cancellation: cancellation(),
                    }]
                );

                let repeat = composition
                    .execute(message_id(), cancellation())
                    .await
                    .unwrap();
                assert_eq!(repeat, evidence);
                let mut conflict = cancellation();
                conflict.reason = "different".into();
                assert_eq!(
                    composition.execute(message_id(), conflict).await,
                    Err(BehaviorCancellationError::ConflictingExecution)
                );
                assert_eq!(
                    composition
                        .inspect_adapter(|adapter| adapter.requests().to_vec())
                        .unwrap()
                        .len(),
                    1
                );
            });
    }

    #[test]
    fn workflow_capability_and_all_associations_fail_during_preflight() {
        let (workflow_id, session_id, correlation_id, trace_id) = ids();
        let build = |workflow, w, s, c, t, capable| {
            BehaviorCancellationComposition::new(
                workflow,
                w,
                s,
                c,
                t,
                message_id(),
                cancellation(),
                adapter(capable),
            )
            .map(|_| ())
        };
        assert_eq!(
            build(
                workflow(false),
                workflow_id,
                session_id,
                correlation_id,
                trace_id,
                true
            ),
            Err(BehaviorCancellationError::InvalidWorkflow)
        );
        assert_eq!(
            build(
                workflow(true),
                workflow_id,
                session_id,
                correlation_id,
                trace_id,
                false
            ),
            Err(BehaviorCancellationError::CapabilityUnavailable)
        );
        let wrong = Uuid::from_u128(99);
        assert_eq!(
            build(
                workflow(true),
                WorkflowId::new(wrong).unwrap(),
                session_id,
                correlation_id,
                trace_id,
                true
            ),
            Err(BehaviorCancellationError::AssociationMismatch)
        );
        assert_eq!(
            build(
                workflow(true),
                workflow_id,
                SessionId::new(wrong).unwrap(),
                correlation_id,
                trace_id,
                true
            ),
            Err(BehaviorCancellationError::AssociationMismatch)
        );
        assert_eq!(
            build(
                workflow(true),
                workflow_id,
                session_id,
                CorrelationId::new(wrong).unwrap(),
                trace_id,
                true
            ),
            Err(BehaviorCancellationError::AssociationMismatch)
        );
        assert_eq!(
            build(
                workflow(true),
                workflow_id,
                session_id,
                correlation_id,
                TraceId::new(wrong).unwrap(),
                true
            ),
            Err(BehaviorCancellationError::AssociationMismatch)
        );
    }

    #[test]
    fn errors_have_exact_content_free_debug_and_display() {
        let cases = [
            (
                BehaviorCancellationError::InvalidWorkflow,
                "InvalidWorkflow",
                "invalid cancelled workflow",
            ),
            (
                BehaviorCancellationError::AssociationMismatch,
                "AssociationMismatch",
                "behavior cancellation association mismatch",
            ),
            (
                BehaviorCancellationError::CapabilityUnavailable,
                "CapabilityUnavailable",
                "avatar cancellation capability unavailable",
            ),
            (
                BehaviorCancellationError::InvalidPreview,
                "InvalidPreview",
                "invalid avatar cancellation preview",
            ),
            (
                BehaviorCancellationError::RuntimeFailure,
                "RuntimeFailure",
                "behavior cancellation runtime failure",
            ),
            (
                BehaviorCancellationError::ConflictingExecution,
                "ConflictingExecution",
                "conflicting behavior cancellation execution",
            ),
        ];
        for (error, debug, display) in cases {
            assert_eq!(format!("{error:?}"), debug);
            assert_eq!(error.to_string(), display);
        }
    }

    struct InvalidPreviewAdapter {
        preview: AvatarReport,
    }

    impl AvatarPort for InvalidPreviewAdapter {
        fn capabilities(&self) -> AvatarCapabilities {
            AvatarCapabilities::new([AvatarCapability::Cancellation])
        }
        fn preview(&self, _: &AvatarRequest) -> AvatarReport {
            self.preview.clone()
        }
        fn submit(&mut self, _: MessageId, _: nexa_nbp::BehaviorCommand) -> AvatarReport {
            unreachable!()
        }
        fn cancel(&mut self, _: MessageId, _: BehaviorCancel) -> AvatarReport {
            unreachable!()
        }
    }

    #[test]
    fn invalid_preview_identity_or_status_fails_before_adapter_mutation() {
        let (workflow_id, session_id, correlation_id, trace_id) = ids();
        let previews = [
            AvatarReport::cancelled(
                MessageId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249099").unwrap(),
                cancellation().behavior_id,
            ),
            AvatarReport::cancelled(
                message_id(),
                BehaviorId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249099").unwrap(),
            ),
            AvatarReport::rejected(
                message_id(),
                cancellation().behavior_id,
                SemanticKey::new("invalid.preview").unwrap(),
                "must not escape".into(),
            ),
        ];
        for preview in previews {
            assert!(matches!(
                BehaviorCancellationComposition::new(
                    workflow(true),
                    workflow_id,
                    session_id,
                    correlation_id,
                    trace_id,
                    message_id(),
                    cancellation(),
                    InvalidPreviewAdapter { preview },
                ),
                Err(BehaviorCancellationError::InvalidPreview)
            ));
        }
    }

    #[derive(Clone)]
    struct DivergentAdapter {
        actual: AvatarReport,
        requests: Arc<Mutex<Vec<AvatarRequest>>>,
    }

    impl AvatarPort for DivergentAdapter {
        fn capabilities(&self) -> AvatarCapabilities {
            AvatarCapabilities::new([AvatarCapability::Cancellation])
        }
        fn preview(&self, _: &AvatarRequest) -> AvatarReport {
            AvatarReport::cancelled(message_id(), cancellation().behavior_id)
        }
        fn submit(&mut self, _: MessageId, _: nexa_nbp::BehaviorCommand) -> AvatarReport {
            unreachable!()
        }
        fn cancel(&mut self, message_id: MessageId, cancellation: BehaviorCancel) -> AvatarReport {
            self.requests.lock().unwrap().push(AvatarRequest::Cancel {
                message_id,
                cancellation,
            });
            self.actual.clone()
        }
    }

    #[test]
    fn divergent_actual_reports_fail_closed_and_terminalize_after_one_mutation() {
        let reports = [
            AvatarReport::cancelled(
                MessageId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249099").unwrap(),
                cancellation().behavior_id,
            ),
            AvatarReport::cancelled(
                message_id(),
                BehaviorId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249099").unwrap(),
            ),
            AvatarReport::rejected(
                message_id(),
                cancellation().behavior_id,
                SemanticKey::new("private.adapter.failure").unwrap(),
                "private adapter details".into(),
            ),
        ];
        for actual in reports {
            let requests = Arc::new(Mutex::new(Vec::new()));
            let (workflow_id, session_id, correlation_id, trace_id) = ids();
            let mut composition = BehaviorCancellationComposition::new(
                workflow(true),
                workflow_id,
                session_id,
                correlation_id,
                trace_id,
                message_id(),
                cancellation(),
                DivergentAdapter {
                    actual,
                    requests: Arc::clone(&requests),
                },
            )
            .unwrap();
            tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap()
                .block_on(async {
                    let error = composition
                        .execute(message_id(), cancellation())
                        .await
                        .unwrap_err();
                    assert_eq!(error, BehaviorCancellationError::RuntimeFailure);
                    assert!(!format!("{error:?} {error}").contains("private"));
                    assert_eq!(requests.lock().unwrap().len(), 1);
                    assert_eq!(
                        composition.execute(message_id(), cancellation()).await,
                        Err(BehaviorCancellationError::RuntimeFailure)
                    );
                    assert_eq!(requests.lock().unwrap().len(), 1);
                });
        }
    }

    #[derive(Clone)]
    struct BlockingProbeAdapter {
        invoked: std::sync::mpsc::Sender<()>,
        release: Arc<Mutex<std::sync::mpsc::Receiver<()>>>,
        requests: Arc<Mutex<Vec<AvatarRequest>>>,
    }

    impl AvatarPort for BlockingProbeAdapter {
        fn capabilities(&self) -> AvatarCapabilities {
            AvatarCapabilities::new([AvatarCapability::Cancellation])
        }
        fn preview(&self, _: &AvatarRequest) -> AvatarReport {
            AvatarReport::cancelled(message_id(), cancellation().behavior_id)
        }
        fn submit(&mut self, _: MessageId, _: nexa_nbp::BehaviorCommand) -> AvatarReport {
            unreachable!()
        }
        fn cancel(&mut self, message_id: MessageId, cancellation: BehaviorCancel) -> AvatarReport {
            self.requests.lock().unwrap().push(AvatarRequest::Cancel {
                message_id,
                cancellation: cancellation.clone(),
            });
            self.invoked.send(()).unwrap();
            self.release.lock().unwrap().recv().unwrap();
            AvatarReport::cancelled(message_id, cancellation.behavior_id)
        }
    }

    #[test]
    fn target_task_waits_for_cancellation_and_is_joined_before_success() {
        let (invoked_tx, invoked_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let (completed_tx, completed_rx) = std::sync::mpsc::channel();
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let (allow_tx, allow_rx) = tokio::sync::oneshot::channel();
        let (dropped_tx, _dropped_rx) = std::sync::mpsc::channel();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let adapter = BlockingProbeAdapter {
            invoked: invoked_tx,
            release: Arc::new(Mutex::new(release_rx)),
            requests: Arc::clone(&requests),
        };
        std::thread::scope(|scope| {
            scope.spawn(move || {
                let (workflow_id, session_id, correlation_id, trace_id) = ids();
                let mut composition = BehaviorCancellationComposition::new(
                    workflow(true),
                    workflow_id,
                    session_id,
                    correlation_id,
                    trace_id,
                    message_id(),
                    cancellation(),
                    adapter,
                )
                .unwrap();
                composition.lifecycle_probe = Some(TestLifecycleProbe {
                    started_waiting: started_tx,
                    allow_cancellation: allow_rx,
                    task_dropped: dropped_tx,
                });
                let result = tokio::runtime::Builder::new_current_thread()
                    .build()
                    .unwrap()
                    .block_on(composition.execute(message_id(), cancellation()));
                completed_tx.send(result).unwrap();
            });
            started_rx.recv().unwrap();
            assert!(invoked_rx.try_recv().is_err());
            assert!(requests.lock().unwrap().is_empty());
            assert!(completed_rx.try_recv().is_err());
            allow_tx.send(()).unwrap();
            invoked_rx.recv().unwrap();
            assert_eq!(requests.lock().unwrap().len(), 1);
            assert!(completed_rx.try_recv().is_err());
            release_tx.send(()).unwrap();
            let evidence = completed_rx.recv().unwrap().unwrap();
            assert_eq!(evidence.avatar().requests().len(), 1);
        });
    }

    #[test]
    fn dropping_in_flight_execution_aborts_owned_task_without_detachment() {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let adapter = DivergentAdapter {
            actual: AvatarReport::cancelled(message_id(), cancellation().behavior_id),
            requests: Arc::clone(&requests),
        };
        let (workflow_id, session_id, correlation_id, trace_id) = ids();
        let mut composition = BehaviorCancellationComposition::new(
            workflow(true),
            workflow_id,
            session_id,
            correlation_id,
            trace_id,
            message_id(),
            cancellation(),
            adapter,
        )
        .unwrap();
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let (_allow_tx, allow_rx) = tokio::sync::oneshot::channel();
        let (task_dropped_tx, task_dropped_rx) = std::sync::mpsc::channel();
        let (drop_execution_tx, drop_execution_rx) = tokio::sync::oneshot::channel();
        let (completed_tx, completed_rx) = std::sync::mpsc::channel();
        composition.lifecycle_probe = Some(TestLifecycleProbe {
            started_waiting: started_tx,
            allow_cancellation: allow_rx,
            task_dropped: task_dropped_tx,
        });

        std::thread::scope(|scope| {
            scope.spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .build()
                    .unwrap();
                runtime.block_on(async {
                    let mut execution = Box::pin(composition.execute(message_id(), cancellation()));
                    tokio::select! {
                        result = &mut execution => panic!("execution completed before drop: {result:?}"),
                        result = drop_execution_rx => result.unwrap(),
                    }
                    drop(execution);
                    tokio::task::yield_now().await;
                    assert_eq!(
                        composition.execute(message_id(), cancellation()).await,
                        Err(BehaviorCancellationError::RuntimeFailure)
                    );
                    completed_tx.send(()).unwrap();
                });
            });

            started_rx.recv().unwrap();
            assert!(requests.lock().unwrap().is_empty());
            drop_execution_tx.send(()).unwrap();
            task_dropped_rx.recv().unwrap();
            completed_rx.recv().unwrap();
            assert!(requests.lock().unwrap().is_empty());
        });
    }
}
