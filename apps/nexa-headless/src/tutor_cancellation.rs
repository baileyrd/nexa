//! Headless binding for the Tutor Generation cancellation control plane.

use nexa_domain::{
    CorrelationId, ModelId, ModelInvocationId, ModelProviderId, SessionId, TraceId, WorkflowId,
};
use nexa_orchestrator::{
    plan_workflow_cancellation, ActiveCancellationTarget, CancellationSemantics,
    CancellationTarget, InteractionWorkflow, WorkflowCancellationPlan, WorkflowState,
};
use nexa_orchestrator_runtime::{WorkflowCancellationExecution, WorkflowTaskGroup};
use nexa_tutor::cancellation::{
    request_tutor_generation_cancellation, TutorGenerationCancellationAcknowledgement,
    TutorGenerationCancellationPort, TutorGenerationCancellationRequest,
    TUTOR_GENERATION_CANCELLATION_V1,
};
use std::sync::{Arc, Mutex};

/// Closed, content-free failures at the headless Tutor Generation boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TutorGenerationCancellationCompositionError {
    InvalidWorkflow,
    AssociationMismatch,
    UnsupportedVersion,
    ControlFailure,
    RuntimeFailure,
    ConflictingExecution,
}

impl std::fmt::Display for TutorGenerationCancellationCompositionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidWorkflow => "invalid cancelled workflow",
            Self::AssociationMismatch => "tutor cancellation association mismatch",
            Self::UnsupportedVersion => "unsupported tutor cancellation version",
            Self::ControlFailure => "tutor cancellation control failure",
            Self::RuntimeFailure => "tutor cancellation runtime failure",
            Self::ConflictingExecution => "conflicting tutor cancellation execution",
        })
    }
}
impl std::error::Error for TutorGenerationCancellationCompositionError {}

/// Immutable evidence that the exact control request was accepted and its owned task joined.
///
/// This does not prove that `LanguageModelProvider::generate` or provider work stopped,
/// joined, or emitted no later output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorGenerationCancellationEvidence {
    runtime: WorkflowCancellationExecution,
    request: TutorGenerationCancellationRequest,
    acknowledgement: TutorGenerationCancellationAcknowledgement,
}
impl TutorGenerationCancellationEvidence {
    pub const fn runtime(&self) -> &WorkflowCancellationExecution {
        &self.runtime
    }
    pub const fn request(&self) -> &TutorGenerationCancellationRequest {
        &self.request
    }
    /// Acceptance of the request only; this is not provider-generation completion evidence.
    pub const fn acknowledgement(&self) -> &TutorGenerationCancellationAcknowledgement {
        &self.acknowledgement
    }
}

enum ExecutionState {
    Ready,
    Running,
    Succeeded(Box<TutorGenerationCancellationEvidence>),
    Failed,
}

/// Owns one exact Tutor Generation cancellation-control operation.
pub struct TutorGenerationCancellationComposition<P> {
    workflow: InteractionWorkflow,
    request: TutorGenerationCancellationRequest,
    plan: WorkflowCancellationPlan,
    port: Arc<Mutex<P>>,
    terminal: ExecutionState,
    #[cfg(test)]
    probe: Option<TestProbe>,
    #[cfg(test)]
    fault: Option<TestFault>,
}

#[cfg(test)]
struct TestProbe {
    waiting: std::sync::mpsc::Sender<()>,
    proceed: tokio::sync::oneshot::Receiver<()>,
    dropped: std::sync::mpsc::Sender<()>,
}
#[cfg(test)]
struct DropProbe(std::sync::mpsc::Sender<()>);
#[cfg(test)]
impl Drop for DropProbe {
    fn drop(&mut self) {
        let _ = self.0.send(());
    }
}
#[cfg(test)]
#[derive(Clone, Copy)]
enum TestFault {
    Join,
    Coverage,
}

impl<P: TutorGenerationCancellationPort + Send + 'static>
    TutorGenerationCancellationComposition<P>
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        workflow: InteractionWorkflow,
        workflow_id: WorkflowId,
        session_id: SessionId,
        correlation_id: CorrelationId,
        trace_id: TraceId,
        request: TutorGenerationCancellationRequest,
        invocation_id: ModelInvocationId,
        provider_id: ModelProviderId,
        model_id: ModelId,
        port: P,
    ) -> Result<Self, TutorGenerationCancellationCompositionError> {
        if workflow.state() != WorkflowState::Cancelled {
            return Err(TutorGenerationCancellationCompositionError::InvalidWorkflow);
        }
        if (
            workflow.workflow_id(),
            workflow.session_id(),
            workflow.correlation_id(),
            workflow.trace_id(),
        ) != (workflow_id, session_id, correlation_id, trace_id)
            || (request.invocation_id, request.provider_id, request.model_id)
                != (invocation_id, provider_id, model_id)
        {
            return Err(TutorGenerationCancellationCompositionError::AssociationMismatch);
        }
        if request.contract_version != TUTOR_GENERATION_CANCELLATION_V1 {
            return Err(TutorGenerationCancellationCompositionError::UnsupportedVersion);
        }
        let plan = plan_workflow_cancellation(
            &workflow,
            workflow_id,
            session_id,
            correlation_id,
            trace_id,
            &[ActiveCancellationTarget::new(
                CancellationTarget::TutorGeneration,
                CancellationSemantics::Cancellable,
            )],
        )
        .map_err(|_| TutorGenerationCancellationCompositionError::InvalidWorkflow)?;
        Ok(Self {
            workflow,
            request,
            plan,
            port: Arc::new(Mutex::new(port)),
            terminal: ExecutionState::Ready,
            #[cfg(test)]
            probe: None,
            #[cfg(test)]
            fault: None,
        })
    }

    pub const fn plan(&self) -> &WorkflowCancellationPlan {
        &self.plan
    }

    /// Bounded read-only inspection of deterministic dependency evidence.
    pub fn inspect_port<R>(
        &self,
        inspect: impl FnOnce(&P) -> R,
    ) -> Result<R, TutorGenerationCancellationCompositionError> {
        self.port
            .lock()
            .map(|port| inspect(&port))
            .map_err(|_| TutorGenerationCancellationCompositionError::RuntimeFailure)
    }

    pub async fn execute(
        &mut self,
        request: TutorGenerationCancellationRequest,
    ) -> Result<TutorGenerationCancellationEvidence, TutorGenerationCancellationCompositionError>
    {
        if request != self.request {
            return Err(TutorGenerationCancellationCompositionError::ConflictingExecution);
        }
        match &self.terminal {
            ExecutionState::Succeeded(value) => return Ok((**value).clone()),
            ExecutionState::Running | ExecutionState::Failed => {
                return Err(TutorGenerationCancellationCompositionError::RuntimeFailure)
            }
            ExecutionState::Ready => {}
        }
        // Terminalize first: dropping this future aborts the local task group and forbids retry.
        self.terminal = ExecutionState::Running;
        let mut tasks = WorkflowTaskGroup::new(self.workflow);
        let port = Arc::clone(&self.port);
        let exact = self.request.clone();
        let observed = Arc::new(Mutex::new(None));
        let task_observed = Arc::clone(&observed);
        #[cfg(test)]
        let (probe, proceed) = self.probe.take().map_or((None, None), |probe| {
            (Some((probe.waiting, probe.dropped)), Some(probe.proceed))
        });
        #[cfg(test)]
        let fault = self.fault;
        tasks
            .spawn_for_target(
                CancellationTarget::TutorGeneration,
                move |token| async move {
                    #[cfg(test)]
                    let _drop_probe = if let Some((waiting, dropped)) = probe {
                        let guard = DropProbe(dropped);
                        let mut wait = Box::pin(token.cancelled());
                        let mut signal = Some(waiting);
                        std::future::poll_fn(|cx| {
                            let polled = std::future::Future::poll(wait.as_mut(), cx);
                            if polled.is_pending() {
                                if let Some(signal) = signal.take() {
                                    let _ = signal.send(());
                                }
                            }
                            polled
                        })
                        .await;
                        Some(guard)
                    } else {
                        token.cancelled().await;
                        None
                    };
                    #[cfg(not(test))]
                    token.cancelled().await;
                    #[cfg(test)]
                    if matches!(fault, Some(TestFault::Join)) {
                        panic!("injected join failure");
                    }
                    let result = port.lock().ok().map(|mut port| {
                        request_tutor_generation_cancellation(
                            &mut *port,
                            &exact,
                            exact.invocation_id,
                            exact.provider_id,
                            exact.model_id,
                        )
                    });
                    if let Ok(mut slot) = task_observed.lock() {
                        *slot = result;
                    }
                },
            )
            .map_err(|_| {
                self.terminal = ExecutionState::Failed;
                TutorGenerationCancellationCompositionError::RuntimeFailure
            })?;
        #[cfg(test)]
        if let Some(proceed) = proceed {
            if proceed.await.is_err() {
                self.terminal = ExecutionState::Failed;
                return Err(TutorGenerationCancellationCompositionError::RuntimeFailure);
            }
        }
        #[cfg(test)]
        if matches!(self.fault, Some(TestFault::Coverage)) {
            tasks
                .spawn_for_target(CancellationTarget::Retrieval, |_| async {})
                .unwrap();
        }
        let runtime = tasks
            .execute_cancellation_plan(&self.plan)
            .await
            .map_err(|_| {
                self.terminal = ExecutionState::Failed;
                TutorGenerationCancellationCompositionError::RuntimeFailure
            })?;
        let acknowledgement = observed
            .lock()
            .ok()
            .and_then(|mut value| value.take())
            .ok_or_else(|| {
                self.terminal = ExecutionState::Failed;
                TutorGenerationCancellationCompositionError::ControlFailure
            })?
            .map_err(|_| {
                self.terminal = ExecutionState::Failed;
                TutorGenerationCancellationCompositionError::ControlFailure
            })?;
        let evidence = TutorGenerationCancellationEvidence {
            runtime,
            request: self.request.clone(),
            acknowledgement,
        };
        self.terminal = ExecutionState::Succeeded(Box::new(evidence.clone()));
        Ok(evidence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexa_domain::ProtocolVersion;
    use nexa_orchestrator::CancellationDirective;
    use nexa_orchestrator_runtime::CancellationTargetExecutionOutcome;
    use nexa_tutor::cancellation::{
        ScriptedTutorGenerationCancellationOutcome as Outcome,
        ScriptedTutorGenerationCancellationPort as Port,
        TutorGenerationCancellationDependencyError,
    };
    use nexa_tutor::model::{
        ModelCapabilities, ModelDescriptor, ModelErrorKind, PrivacyClass, ScriptedModelProvider,
        ScriptedOutcome,
    };
    use std::future::Future;
    use uuid::Uuid;

    #[derive(Clone, Debug)]
    struct SharedPort(Arc<Mutex<Port>>);

    impl SharedPort {
        fn new(outcomes: impl IntoIterator<Item = Outcome>) -> Self {
            Self(Arc::new(Mutex::new(Port::new(outcomes))))
        }

        fn accounting(&self) -> (usize, usize, usize) {
            let port = self.0.lock().unwrap();
            (
                port.received_requests().len(),
                port.consumed_outcomes(),
                port.remaining_outcomes(),
            )
        }
    }

    impl TutorGenerationCancellationPort for SharedPort {
        fn request_cancellation(
            &mut self,
            request: &TutorGenerationCancellationRequest,
        ) -> Result<
            TutorGenerationCancellationAcknowledgement,
            TutorGenerationCancellationDependencyError,
        > {
            self.0.lock().unwrap().request_cancellation(request)
        }
    }

    fn id<T>(n: u128, make: impl Fn(Uuid) -> Result<T, nexa_domain::ValueError>) -> T {
        make(Uuid::from_u128(n)).unwrap()
    }
    fn ids() -> (WorkflowId, SessionId, CorrelationId, TraceId) {
        (
            id(1, WorkflowId::new),
            id(2, SessionId::new),
            id(3, CorrelationId::new),
            id(4, TraceId::new),
        )
    }
    fn workflow(cancelled: bool) -> InteractionWorkflow {
        let (w, s, c, t) = ids();
        let value = InteractionWorkflow::new(w, s, c, t);
        if cancelled {
            value.cancel().unwrap()
        } else {
            value
        }
    }
    fn request() -> TutorGenerationCancellationRequest {
        TutorGenerationCancellationRequest::new(
            id(5, ModelInvocationId::new),
            id(6, ModelProviderId::new),
            id(7, ModelId::new),
        )
    }
    fn composition(
        outcomes: impl IntoIterator<Item = Outcome>,
    ) -> TutorGenerationCancellationComposition<Port> {
        let (w, s, c, t) = ids();
        let r = request();
        TutorGenerationCancellationComposition::new(
            workflow(true),
            w,
            s,
            c,
            t,
            r.clone(),
            r.invocation_id,
            r.provider_id,
            r.model_id,
            Port::new(outcomes),
        )
        .unwrap()
    }
    fn ack() -> TutorGenerationCancellationAcknowledgement {
        TutorGenerationCancellationAcknowledgement::for_request(&request())
    }
    async fn receive(rx: &std::sync::mpsc::Receiver<()>) {
        loop {
            match rx.try_recv() {
                Ok(()) => return,
                Err(std::sync::mpsc::TryRecvError::Empty) => tokio::task::yield_now().await,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => panic!("probe disconnected"),
            }
        }
    }

    #[test]
    fn constructor_preflights_all_associations_and_builds_only_canonical_tutor_plan() {
        let good = composition([Outcome::Acknowledged(ack())]);
        assert_eq!(good.plan().directives().len(), 1);
        assert_eq!(
            good.plan().directives()[0].target(),
            CancellationTarget::TutorGeneration
        );
        assert_eq!(
            good.plan().directives()[0].directive(),
            CancellationDirective::RequestCancellation
        );
        let (w, s, c, t) = ids();
        let r = request();
        let assert_preflight_failure =
            |workflow, workflow_id, session_id, correlation_id, trace_id, request, expected| {
                let port = SharedPort::new([Outcome::Acknowledged(ack())]);
                let observable = port.clone();
                assert!(matches!(
                    TutorGenerationCancellationComposition::new(
                        workflow,
                        workflow_id,
                        session_id,
                        correlation_id,
                        trace_id,
                        request,
                        r.invocation_id,
                        r.provider_id,
                        r.model_id,
                        port,
                    ),
                    Err(error) if error == expected
                ));
                assert_eq!(observable.accounting(), (0, 0, 1));
            };
        assert_preflight_failure(
            workflow(false),
            w,
            s,
            c,
            t,
            r.clone(),
            TutorGenerationCancellationCompositionError::InvalidWorkflow,
        );
        for changed in 0..7 {
            let mut rq = r.clone();
            let mut wi = w;
            let mut si = s;
            let mut ci = c;
            let mut ti = t;
            match changed {
                0 => wi = id(20, WorkflowId::new),
                1 => si = id(20, SessionId::new),
                2 => ci = id(20, CorrelationId::new),
                3 => ti = id(20, TraceId::new),
                4 => rq.invocation_id = id(20, ModelInvocationId::new),
                5 => rq.provider_id = id(20, ModelProviderId::new),
                _ => rq.model_id = id(20, ModelId::new),
            }
            assert_preflight_failure(
                workflow(true),
                wi,
                si,
                ci,
                ti,
                rq,
                TutorGenerationCancellationCompositionError::AssociationMismatch,
            );
        }
        let mut invalid = r.clone();
        invalid.contract_version = ProtocolVersion::new(2, 0);
        assert_preflight_failure(
            workflow(true),
            w,
            s,
            c,
            t,
            invalid,
            TutorGenerationCancellationCompositionError::UnsupportedVersion,
        );
    }

    #[tokio::test]
    async fn task_waits_then_calls_once_and_returns_exact_limited_evidence() {
        let mut c = composition([Outcome::Acknowledged(ack())]);
        let (waiting_rx, proceed_tx) = {
            let (waiting_tx, waiting_rx) = std::sync::mpsc::channel();
            let (proceed_tx, proceed_rx) = tokio::sync::oneshot::channel();
            let (drop_tx, _) = std::sync::mpsc::channel();
            c.probe = Some(TestProbe {
                waiting: waiting_tx,
                proceed: proceed_rx,
                dropped: drop_tx,
            });
            (waiting_rx, proceed_tx)
        };
        let port = Arc::clone(&c.port);
        let mut future = Box::pin(c.execute(request()));
        assert!(
            std::future::poll_fn(|cx| match future.as_mut().poll(cx) {
                std::task::Poll::Pending => std::task::Poll::Ready(true),
                std::task::Poll::Ready(_) => std::task::Poll::Ready(false),
            })
            .await
        );
        receive(&waiting_rx).await;
        assert_eq!(port.lock().unwrap().received_requests().len(), 0);
        proceed_tx.send(()).unwrap();
        let evidence = future.await.unwrap();
        assert_eq!(evidence.request(), &request());
        assert_eq!(evidence.acknowledgement(), &ack());
        assert_eq!(evidence.runtime().target_outcomes().len(), 1);
        assert_eq!(
            evidence.runtime().target_outcomes()[0].target(),
            CancellationTarget::TutorGeneration
        );
        assert_eq!(
            evidence.runtime().target_outcomes()[0].outcome(),
            CancellationTargetExecutionOutcome::Stopped
        );
        assert_eq!(evidence.runtime().accepted_unclassified_task_count(), 0);
        assert_eq!(
            c.inspect_port(|p| (p.received_requests().to_vec(), p.consumed_outcomes()))
                .unwrap(),
            (vec![request()], 1)
        );
    }

    #[tokio::test]
    async fn success_is_idempotent_and_conflict_has_no_mutation() {
        let mut c = composition([Outcome::Acknowledged(ack())]);
        let first = c.execute(request()).await.unwrap();
        assert_eq!(c.execute(request()).await.unwrap(), first);
        let mut other = request();
        other.model_id = id(30, ModelId::new);
        assert_eq!(
            c.execute(other).await,
            Err(TutorGenerationCancellationCompositionError::ConflictingExecution)
        );
        assert_eq!(
            c.inspect_port(|p| (p.received_requests().len(), p.consumed_outcomes()))
                .unwrap(),
            (1, 1)
        );
    }

    #[tokio::test]
    async fn control_path_does_not_consume_or_reinterpret_generation_work() {
        let provider = ScriptedModelProvider::new(
            ModelDescriptor::new(
                request().provider_id,
                request().model_id,
                PrivacyClass::LocalOnly,
                ModelCapabilities {
                    streaming: false,
                    structured_output: true,
                    tool_calling: false,
                    vision: false,
                    context_window_tokens: 4_096,
                    maximum_output_tokens: 512,
                },
            )
            .unwrap(),
            [ScriptedOutcome::Error(ModelErrorKind::Cancelled)],
        )
        .unwrap();
        let mut composition = composition([Outcome::Acknowledged(ack())]);
        let evidence = composition.execute(request()).await.unwrap();
        assert_eq!(provider.remaining(), 1);
        assert_eq!(
            evidence.runtime().target_outcomes()[0].outcome(),
            CancellationTargetExecutionOutcome::Stopped
        );
        assert_eq!(evidence.acknowledgement(), &ack());
    }

    #[tokio::test]
    async fn dependency_exhaustion_failure_and_ack_reassociations_terminalize() {
        let cases = [
            None,
            Some(Outcome::DependencyFailure),
            Some(Outcome::Acknowledged(
                TutorGenerationCancellationAcknowledgement {
                    contract_version: ProtocolVersion::new(2, 0),
                    ..ack()
                },
            )),
            Some(Outcome::Acknowledged(
                TutorGenerationCancellationAcknowledgement {
                    invocation_id: id(40, ModelInvocationId::new),
                    ..ack()
                },
            )),
            Some(Outcome::Acknowledged(
                TutorGenerationCancellationAcknowledgement {
                    provider_id: id(40, ModelProviderId::new),
                    ..ack()
                },
            )),
            Some(Outcome::Acknowledged(
                TutorGenerationCancellationAcknowledgement {
                    model_id: id(40, ModelId::new),
                    ..ack()
                },
            )),
        ];
        for outcome in cases {
            let expected = if outcome.is_none() {
                (1, 0, 0)
            } else {
                (1, 1, 0)
            };
            let mut c = composition(outcome);
            assert_eq!(
                c.execute(request()).await,
                Err(TutorGenerationCancellationCompositionError::ControlFailure)
            );
            let accounting = c
                .inspect_port(|p| {
                    (
                        p.received_requests().len(),
                        p.consumed_outcomes(),
                        p.remaining_outcomes(),
                    )
                })
                .unwrap();
            assert_eq!(accounting, expected);
            assert_eq!(
                c.execute(request()).await,
                Err(TutorGenerationCancellationCompositionError::RuntimeFailure)
            );
            assert_eq!(
                c.inspect_port(|p| {
                    (
                        p.received_requests().len(),
                        p.consumed_outcomes(),
                        p.remaining_outcomes(),
                    )
                })
                .unwrap(),
                accounting
            );
        }
    }

    #[tokio::test]
    async fn runtime_coverage_and_join_failures_are_closed_and_terminal() {
        for fault in [TestFault::Coverage, TestFault::Join] {
            let mut c = composition([Outcome::Acknowledged(ack())]);
            c.fault = Some(fault);
            assert_eq!(
                c.execute(request()).await,
                Err(TutorGenerationCancellationCompositionError::RuntimeFailure)
            );
            assert_eq!(
                c.execute(request()).await,
                Err(TutorGenerationCancellationCompositionError::RuntimeFailure)
            );
        }
    }

    #[tokio::test]
    async fn caller_drop_aborts_waiting_owned_work_and_retry_is_forbidden() {
        let mut c = composition([Outcome::Acknowledged(ack())]);
        let (waiting_tx, waiting_rx) = std::sync::mpsc::channel();
        let (_proceed_tx, proceed_rx) = tokio::sync::oneshot::channel();
        let (drop_tx, drop_rx) = std::sync::mpsc::channel();
        c.probe = Some(TestProbe {
            waiting: waiting_tx,
            proceed: proceed_rx,
            dropped: drop_tx,
        });
        let mut future = Box::pin(c.execute(request()));
        std::future::poll_fn(|cx| {
            let _ = future.as_mut().poll(cx);
            std::task::Poll::Ready(())
        })
        .await;
        receive(&waiting_rx).await;
        drop(future);
        receive(&drop_rx).await;
        assert_eq!(
            c.inspect_port(|p| (p.received_requests().len(), p.consumed_outcomes()))
                .unwrap(),
            (0, 0)
        );
        assert_eq!(
            c.execute(request()).await,
            Err(TutorGenerationCancellationCompositionError::RuntimeFailure)
        );
    }

    #[test]
    fn errors_are_closed_and_content_free() {
        let cases = [
            (
                TutorGenerationCancellationCompositionError::InvalidWorkflow,
                "InvalidWorkflow",
                "invalid cancelled workflow",
            ),
            (
                TutorGenerationCancellationCompositionError::AssociationMismatch,
                "AssociationMismatch",
                "tutor cancellation association mismatch",
            ),
            (
                TutorGenerationCancellationCompositionError::UnsupportedVersion,
                "UnsupportedVersion",
                "unsupported tutor cancellation version",
            ),
            (
                TutorGenerationCancellationCompositionError::ControlFailure,
                "ControlFailure",
                "tutor cancellation control failure",
            ),
            (
                TutorGenerationCancellationCompositionError::RuntimeFailure,
                "RuntimeFailure",
                "tutor cancellation runtime failure",
            ),
            (
                TutorGenerationCancellationCompositionError::ConflictingExecution,
                "ConflictingExecution",
                "conflicting tutor cancellation execution",
            ),
        ];
        let forbidden = [
            "private_prompt",
            "private_output",
            "private_endpoint",
            "client_secret",
            "private_credential",
            "provider_payload",
            "runtime_detail",
            "task_detail",
        ];
        for (error, debug, display) in cases {
            assert_eq!(format!("{error:?}"), debug);
            assert_eq!(error.to_string(), display);
            let diagnostics = format!("{error:?} {error}");
            for sentinel in forbidden {
                assert!(!diagnostics.contains(sentinel));
            }
        }
    }
}
