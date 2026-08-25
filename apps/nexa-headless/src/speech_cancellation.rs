use crate::AvatarCancellationEvidence;
use nexa_avatar::{AvatarPort, AvatarReport, AvatarRequest};
use nexa_domain::{CorrelationId, MessageId, SessionId, SpeechId, TraceId, WorkflowId};
use nexa_nbp::{AvatarCapability, BehaviorCancel, RuntimeStatus};
use nexa_orchestrator::{
    plan_workflow_cancellation, ActiveCancellationTarget, CancellationSemantics,
    CancellationTarget, InteractionWorkflow, WorkflowCancellationPlan, WorkflowState,
};
use nexa_orchestrator_runtime::{WorkflowCancellationExecution, WorkflowTaskGroup};
use nexa_speech::{
    OwnedSpeechCancellationCoordinator, SpeechCancellationAggregateEvidence,
    SpeechCancellationParticipant, SpeechCancellationRequest, SPEECH_CANCELLATION_V1,
};
use std::sync::{Arc, Mutex};

/// Closed, content-free failures for the bounded Speech plus Behavior binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpeechInteractionCancellationError {
    InvalidWorkflow,
    AssociationMismatch,
    InvalidSpeech,
    InvalidSpeechParticipants,
    BehaviorCapabilityUnavailable,
    InvalidBehaviorPreview,
    RuntimeFailure,
    ConflictingExecution,
}

impl std::fmt::Display for SpeechInteractionCancellationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidWorkflow => "invalid cancelled workflow",
            Self::AssociationMismatch => "speech interaction cancellation association mismatch",
            Self::InvalidSpeech => "invalid speech cancellation request",
            Self::InvalidSpeechParticipants => "invalid speech cancellation participants",
            Self::BehaviorCapabilityUnavailable => "behavior cancellation capability unavailable",
            Self::InvalidBehaviorPreview => "invalid behavior cancellation preview",
            Self::RuntimeFailure => "speech interaction cancellation runtime failure",
            Self::ConflictingExecution => "conflicting speech interaction cancellation execution",
        })
    }
}

impl std::error::Error for SpeechInteractionCancellationError {}

/// Immutable evidence for the exact joined two-target application operation.
///
/// This proves only terminalization of the four Speech control futures and the
/// Behavior adapter cancellation task. It does not prove that external provider,
/// audio, device, process, renderer, or other real-world work stopped.
#[derive(Clone, PartialEq)]
pub struct SpeechInteractionCancellationEvidence {
    runtime: WorkflowCancellationExecution,
    speech: SpeechCancellationAggregateEvidence,
    behavior: AvatarCancellationEvidence,
}

impl std::fmt::Debug for SpeechInteractionCancellationEvidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SpeechInteractionCancellationEvidence { redacted }")
    }
}

impl SpeechInteractionCancellationEvidence {
    pub const fn runtime(&self) -> &WorkflowCancellationExecution {
        &self.runtime
    }
    pub const fn speech(&self) -> &SpeechCancellationAggregateEvidence {
        &self.speech
    }
    pub const fn behavior(&self) -> &AvatarCancellationEvidence {
        &self.behavior
    }
}

enum ExecutionState {
    Ready,
    Running,
    Succeeded(Box<SpeechInteractionCancellationEvidence>),
    Failed,
}

/// Owns one exact, canonical Speech and Behavior cancellation execution.
pub struct SpeechInteractionCancellationComposition<A> {
    workflow: InteractionWorkflow,
    speech_request: SpeechCancellationRequest,
    speech_coordinator: Option<OwnedSpeechCancellationCoordinator>,
    behavior_request: AvatarRequest,
    behavior_preview: AvatarReport,
    adapter: Arc<Mutex<A>>,
    plan: WorkflowCancellationPlan,
    terminal: ExecutionState,
    #[cfg(test)]
    test_seam: TestExecutionSeam,
}

#[cfg(test)]
#[derive(Clone, Default)]
struct TestExecutionSeam {
    behavior_panics: bool,
    behavior_panics_when: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
    omit_behavior_result: bool,
    runtime_evidence_mismatch: bool,
    behavior_active: Option<Arc<std::sync::atomic::AtomicUsize>>,
    behavior_pending: bool,
}

#[cfg(test)]
struct ActiveBehaviorProbe(Arc<std::sync::atomic::AtomicUsize>);

#[cfg(test)]
impl Drop for ActiveBehaviorProbe {
    fn drop(&mut self) {
        self.0.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }
}

impl<A: AvatarPort + Send + 'static> SpeechInteractionCancellationComposition<A> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        workflow: InteractionWorkflow,
        workflow_id: WorkflowId,
        session_id: SessionId,
        correlation_id: CorrelationId,
        trace_id: TraceId,
        speech_id: SpeechId,
        speech_request: SpeechCancellationRequest,
        participants: Vec<Arc<dyn SpeechCancellationParticipant>>,
        behavior_message_id: MessageId,
        behavior_cancellation: BehaviorCancel,
        adapter: A,
    ) -> Result<Self, SpeechInteractionCancellationError> {
        if workflow.state() != WorkflowState::Cancelled {
            return Err(SpeechInteractionCancellationError::InvalidWorkflow);
        }
        if (
            workflow.workflow_id(),
            workflow.session_id(),
            workflow.correlation_id(),
            workflow.trace_id(),
        ) != (workflow_id, session_id, correlation_id, trace_id)
        {
            return Err(SpeechInteractionCancellationError::AssociationMismatch);
        }
        if speech_request.contract_version != SPEECH_CANCELLATION_V1
            || speech_request.speech_id != speech_id
        {
            return Err(SpeechInteractionCancellationError::InvalidSpeech);
        }
        let speech_coordinator =
            OwnedSpeechCancellationCoordinator::new(speech_id, participants)
                .map_err(|_| SpeechInteractionCancellationError::InvalidSpeechParticipants)?;
        if !adapter
            .capabilities()
            .supports(AvatarCapability::Cancellation)
        {
            return Err(SpeechInteractionCancellationError::BehaviorCapabilityUnavailable);
        }
        let behavior_request = AvatarRequest::Cancel {
            message_id: behavior_message_id,
            cancellation: behavior_cancellation,
        };
        let behavior_preview = adapter.preview(&behavior_request);
        let behavior_id = match &behavior_request {
            AvatarRequest::Cancel { cancellation, .. } => cancellation.behavior_id,
            AvatarRequest::Submit { .. } => unreachable!(),
        };
        if behavior_preview.message_id != behavior_message_id
            || behavior_preview.behavior_id != behavior_id
            || behavior_preview.terminal_status() != RuntimeStatus::Cancelled
        {
            return Err(SpeechInteractionCancellationError::InvalidBehaviorPreview);
        }
        let plan = plan_workflow_cancellation(
            &workflow,
            workflow_id,
            session_id,
            correlation_id,
            trace_id,
            &[
                ActiveCancellationTarget::new(
                    CancellationTarget::Speech,
                    CancellationSemantics::Cancellable,
                ),
                ActiveCancellationTarget::new(
                    CancellationTarget::Behavior,
                    CancellationSemantics::Cancellable,
                ),
            ],
        )
        .map_err(|_| SpeechInteractionCancellationError::InvalidWorkflow)?;
        Ok(Self {
            workflow,
            speech_request,
            speech_coordinator: Some(speech_coordinator),
            behavior_request,
            behavior_preview,
            adapter: Arc::new(Mutex::new(adapter)),
            plan,
            terminal: ExecutionState::Ready,
            #[cfg(test)]
            test_seam: TestExecutionSeam::default(),
        })
    }

    pub const fn plan(&self) -> &WorkflowCancellationPlan {
        &self.plan
    }

    pub fn inspect_adapter<R>(
        &self,
        inspect: impl FnOnce(&A) -> R,
    ) -> Result<R, SpeechInteractionCancellationError> {
        self.adapter
            .lock()
            .map(|adapter| inspect(&adapter))
            .map_err(|_| SpeechInteractionCancellationError::RuntimeFailure)
    }

    pub async fn execute(
        &mut self,
        speech_request: SpeechCancellationRequest,
        behavior_message_id: MessageId,
        behavior_cancellation: BehaviorCancel,
    ) -> Result<SpeechInteractionCancellationEvidence, SpeechInteractionCancellationError> {
        let behavior_request = AvatarRequest::Cancel {
            message_id: behavior_message_id,
            cancellation: behavior_cancellation,
        };
        if speech_request != self.speech_request || behavior_request != self.behavior_request {
            return Err(SpeechInteractionCancellationError::ConflictingExecution);
        }
        match &self.terminal {
            ExecutionState::Succeeded(evidence) => return Ok((**evidence).clone()),
            ExecutionState::Running | ExecutionState::Failed => {
                return Err(SpeechInteractionCancellationError::RuntimeFailure)
            }
            ExecutionState::Ready => {}
        }
        // Terminalize before spawning. Dropping this future drops the group, which aborts
        // and owns both tasks; retry can never consume another dependency outcome.
        self.terminal = ExecutionState::Running;
        let mut tasks = WorkflowTaskGroup::new(self.workflow);
        let speech_result = Arc::new(Mutex::new(None));
        let task_speech_result = Arc::clone(&speech_result);
        let Some(coordinator) = self.speech_coordinator.take() else {
            self.terminal = ExecutionState::Failed;
            return Err(SpeechInteractionCancellationError::RuntimeFailure);
        };
        let request = self.speech_request;
        if tasks
            .spawn_for_target(CancellationTarget::Speech, move |token| async move {
                token.cancelled().await;
                let result = coordinator.cancel(request).await;
                if let Ok(mut slot) = task_speech_result.lock() {
                    *slot = Some(result);
                }
            })
            .is_err()
        {
            self.terminal = ExecutionState::Failed;
            return Err(SpeechInteractionCancellationError::RuntimeFailure);
        }
        let behavior_result = Arc::new(Mutex::new(None));
        let task_behavior_result = Arc::clone(&behavior_result);
        let adapter = Arc::clone(&self.adapter);
        let request = self.behavior_request.clone();
        #[cfg(test)]
        let behavior_seam = self.test_seam.clone();
        if tasks
            .spawn_for_target(CancellationTarget::Behavior, move |token| async move {
                token.cancelled().await;
                let invoked = request.clone();
                let report = adapter
                    .lock()
                    .ok()
                    .map(|mut adapter| adapter.handle(request));
                #[cfg(test)]
                {
                    let _active_probe = behavior_seam.behavior_active.as_ref().map(|active| {
                        active.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        ActiveBehaviorProbe(Arc::clone(active))
                    });
                    if let Some(ready) = behavior_seam.behavior_panics_when {
                        while !ready() {
                            tokio::task::yield_now().await;
                        }
                        panic!("test-only behavior task failure after both targets started");
                    }
                    if behavior_seam.behavior_panics {
                        panic!("test-only behavior task failure");
                    }
                    if behavior_seam.behavior_pending {
                        std::future::pending::<()>().await;
                    }
                    if behavior_seam.omit_behavior_result {
                        return;
                    }
                }
                if let (Some(report), Ok(mut slot)) = (report, task_behavior_result.lock()) {
                    *slot = Some((invoked, report));
                }
            })
            .is_err()
        {
            self.terminal = ExecutionState::Failed;
            return Err(SpeechInteractionCancellationError::RuntimeFailure);
        }
        let runtime = match tasks.execute_cancellation_plan(&self.plan).await {
            Ok(value) => value,
            Err(_) => {
                self.terminal = ExecutionState::Failed;
                return Err(SpeechInteractionCancellationError::RuntimeFailure);
            }
        };
        let speech = speech_result
            .lock()
            .ok()
            .and_then(|mut value| value.take())
            .and_then(Result::ok);
        let behavior = behavior_result
            .lock()
            .ok()
            .and_then(|mut value| value.take());
        let (Some(speech), Some((invoked, report))) = (speech, behavior) else {
            self.terminal = ExecutionState::Failed;
            return Err(SpeechInteractionCancellationError::RuntimeFailure);
        };
        #[cfg(test)]
        let runtime_evidence_mismatch = self.test_seam.runtime_evidence_mismatch;
        #[cfg(not(test))]
        let runtime_evidence_mismatch = false;
        if speech.speech_id != self.speech_request.speech_id
            || invoked != self.behavior_request
            || report != self.behavior_preview
            || report.terminal_status() != RuntimeStatus::Cancelled
            || runtime.workflow() != self.workflow
            || runtime
                .target_outcomes()
                .iter()
                .map(|evidence| (evidence.target(), evidence.outcome()))
                .ne([
                    (
                        CancellationTarget::Speech,
                        nexa_orchestrator_runtime::CancellationTargetExecutionOutcome::Stopped,
                    ),
                    (
                        CancellationTarget::Behavior,
                        nexa_orchestrator_runtime::CancellationTargetExecutionOutcome::Stopped,
                    ),
                ])
            || runtime_evidence_mismatch
        {
            self.terminal = ExecutionState::Failed;
            return Err(SpeechInteractionCancellationError::RuntimeFailure);
        }
        let evidence = SpeechInteractionCancellationEvidence {
            runtime,
            speech,
            behavior: AvatarCancellationEvidence {
                requests: vec![invoked],
                report,
            },
        };
        self.terminal = ExecutionState::Succeeded(Box::new(evidence.clone()));
        Ok(evidence)
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
    use nexa_speech::{
        ScriptedSpeechCancellationOutcome, ScriptedSpeechCancellationParticipant,
        SpeechCancellationAcknowledgement, SpeechCancellationCapability, SpeechCancellationSurface,
    };
    use std::str::FromStr;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use uuid::Uuid;

    #[derive(Clone)]
    struct InspectableAvatar {
        inner: Arc<Mutex<InspectableAvatarState>>,
        capable: bool,
        preview_valid: bool,
        actual_matches: bool,
    }

    #[derive(Default)]
    struct InspectableAvatarState {
        requests: Vec<AvatarRequest>,
    }

    impl InspectableAvatar {
        fn new(capable: bool, preview_valid: bool, actual_matches: bool) -> Self {
            Self {
                inner: Arc::new(Mutex::new(InspectableAvatarState::default())),
                capable,
                preview_valid,
                actual_matches,
            }
        }

        fn request_count(&self) -> usize {
            self.inner.lock().unwrap().requests.len()
        }
    }

    impl AvatarPort for InspectableAvatar {
        fn capabilities(&self) -> AvatarCapabilities {
            AvatarCapabilities::new(self.capable.then_some(AvatarCapability::Cancellation))
        }

        fn submit(
            &mut self,
            _message_id: MessageId,
            _command: nexa_nbp::BehaviorCommand,
        ) -> AvatarReport {
            unreachable!()
        }

        fn cancel(&mut self, message_id: MessageId, cancellation: BehaviorCancel) -> AvatarReport {
            self.inner
                .lock()
                .unwrap()
                .requests
                .push(AvatarRequest::Cancel {
                    message_id,
                    cancellation: cancellation.clone(),
                });
            let actual_id = if self.actual_matches {
                cancellation.behavior_id
            } else {
                BehaviorId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249099").unwrap()
            };
            AvatarReport::cancelled(message_id, actual_id)
        }

        fn preview(&self, request: &AvatarRequest) -> AvatarReport {
            let AvatarRequest::Cancel {
                message_id,
                cancellation,
            } = request
            else {
                unreachable!()
            };
            let preview_id = if self.preview_valid {
                cancellation.behavior_id
            } else {
                BehaviorId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249098").unwrap()
            };
            AvatarReport::cancelled(*message_id, preview_id)
        }
    }

    fn ids() -> (WorkflowId, SessionId, CorrelationId, TraceId) {
        (
            WorkflowId::new(Uuid::from_u128(1)).unwrap(),
            SessionId::new(Uuid::from_u128(2)).unwrap(),
            CorrelationId::new(Uuid::from_u128(3)).unwrap(),
            TraceId::new(Uuid::from_u128(4)).unwrap(),
        )
    }
    fn speech_id() -> SpeechId {
        SpeechId::new(Uuid::from_u128(5)).unwrap()
    }
    fn message_id() -> MessageId {
        MessageId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249002").unwrap()
    }
    fn cancellation() -> BehaviorCancel {
        BehaviorCancel {
            behavior_id: BehaviorId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249001").unwrap(),
            reason: "private-behavior-reason".into(),
            transition: CancellationMode::Graceful,
        }
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
    fn adapter(capable: bool) -> FakeAvatarAdapter {
        FakeAvatarAdapter::new(
            SemanticKey::new("headless-avatar").unwrap(),
            AvatarCapabilities::new(capable.then_some(AvatarCapability::Cancellation)),
        )
    }
    fn participants(
        outcome: ScriptedSpeechCancellationOutcome,
    ) -> Vec<Arc<dyn SpeechCancellationParticipant>> {
        SpeechCancellationSurface::ALL
            .into_iter()
            .rev()
            .map(|surface| {
                Arc::new(ScriptedSpeechCancellationParticipant::new(
                    SpeechCancellationCapability::cancellable(speech_id(), surface),
                    [outcome],
                )) as Arc<dyn SpeechCancellationParticipant>
            })
            .collect()
    }
    fn tracked_participants(
        capabilities: impl IntoIterator<Item = SpeechCancellationCapability>,
        outcomes: impl IntoIterator<Item = ScriptedSpeechCancellationOutcome> + Clone,
    ) -> (
        Vec<Arc<ScriptedSpeechCancellationParticipant>>,
        Vec<Arc<dyn SpeechCancellationParticipant>>,
    ) {
        let tracked = capabilities
            .into_iter()
            .map(|capability| {
                Arc::new(ScriptedSpeechCancellationParticipant::new(
                    capability,
                    outcomes.clone(),
                ))
            })
            .collect::<Vec<_>>();
        let erased = tracked
            .iter()
            .cloned()
            .map(|value| value as Arc<dyn SpeechCancellationParticipant>)
            .collect();
        (tracked, erased)
    }
    #[test]
    fn canonical_two_target_success_is_joined_exact_and_idempotent() {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                let request = SpeechCancellationRequest::new(speech_id());
                let (tracked, erased) = tracked_participants(
                    SpeechCancellationSurface::ALL
                        .into_iter()
                        .rev()
                        .map(|surface| {
                            SpeechCancellationCapability::cancellable(speech_id(), surface)
                        }),
                    [ScriptedSpeechCancellationOutcome::Acknowledged(
                        SpeechCancellationAcknowledgement::for_request(&request),
                    )],
                );
                let (w, s, c, t) = ids();
                let mut value = SpeechInteractionCancellationComposition::new(
                    workflow(true),
                    w,
                    s,
                    c,
                    t,
                    speech_id(),
                    request,
                    erased,
                    message_id(),
                    cancellation(),
                    adapter(true),
                )
                .unwrap();
                assert_eq!(
                    value
                        .plan()
                        .directives()
                        .iter()
                        .map(|d| (d.target(), d.directive()))
                        .collect::<Vec<_>>(),
                    vec![
                        (
                            CancellationTarget::Speech,
                            CancellationDirective::RequestCancellation
                        ),
                        (
                            CancellationTarget::Behavior,
                            CancellationDirective::RequestCancellation
                        )
                    ]
                );
                let evidence = value
                    .execute(request, message_id(), cancellation())
                    .await
                    .unwrap();
                assert_eq!(
                    evidence
                        .speech()
                        .surfaces
                        .iter()
                        .map(|e| e.surface)
                        .collect::<Vec<_>>(),
                    SpeechCancellationSurface::ALL
                );
                assert_eq!(
                    evidence
                        .runtime()
                        .target_outcomes()
                        .iter()
                        .map(|outcome| (outcome.target(), outcome.outcome()))
                        .collect::<Vec<_>>(),
                    vec![
                        (
                            CancellationTarget::Speech,
                            CancellationTargetExecutionOutcome::Stopped,
                        ),
                        (
                            CancellationTarget::Behavior,
                            CancellationTargetExecutionOutcome::Stopped,
                        ),
                    ]
                );
                assert_eq!(evidence.runtime().workflow(), workflow(true));
                assert_eq!(evidence.speech().speech_id, speech_id());
                assert!(evidence.speech().surfaces.iter().all(|surface| {
                    surface.acknowledgement
                        == SpeechCancellationAcknowledgement::for_request(&request)
                }));
                assert!(evidence
                    .runtime()
                    .target_outcomes()
                    .iter()
                    .all(|e| e.outcome() == CancellationTargetExecutionOutcome::Stopped));
                assert_eq!(evidence.behavior().cancellation_request_count(), 1);
                assert_eq!(
                    evidence.behavior().request(),
                    &AvatarRequest::Cancel {
                        message_id: message_id(),
                        cancellation: cancellation()
                    }
                );
                assert_eq!(
                    evidence.behavior().report(),
                    &adapter(true).preview(&AvatarRequest::Cancel {
                        message_id: message_id(),
                        cancellation: cancellation()
                    })
                );
                let diagnostic = format!("{evidence:?}");
                assert_eq!(
                    diagnostic,
                    "SpeechInteractionCancellationEvidence { redacted }"
                );
                assert!(!diagnostic.contains("private-behavior-reason"));
                assert_eq!(
                    value
                        .execute(request, message_id(), cancellation())
                        .await
                        .unwrap(),
                    evidence
                );
                assert_eq!(value.inspect_adapter(|a| a.requests().len()).unwrap(), 1);
                for participant in &tracked {
                    assert_eq!(participant.received_requests(), vec![request]);
                    assert_eq!(participant.consumed_outcome_count(), 1);
                    assert_eq!(participant.remaining_outcome_count(), 0);
                    assert_eq!(participant.active_future_count(), 0);
                }
                let mut conflict = cancellation();
                conflict.reason = "conflict".into();
                assert_eq!(
                    value.execute(request, message_id(), conflict).await,
                    Err(SpeechInteractionCancellationError::ConflictingExecution)
                );
                assert_eq!(value.inspect_adapter(|a| a.requests().len()).unwrap(), 1);
                for participant in tracked {
                    assert_eq!(participant.received_requests(), vec![request]);
                    assert_eq!(participant.consumed_outcome_count(), 1);
                    assert_eq!(participant.remaining_outcome_count(), 0);
                    assert_eq!(participant.active_future_count(), 0);
                }
            });
    }

    #[test]
    fn construction_preflight_rejects_without_consuming_dependencies() {
        let (w, s, c, t) = ids();
        let request = SpeechCancellationRequest::new(speech_id());
        let cases = [
            SpeechInteractionCancellationComposition::new(
                workflow(false),
                w,
                s,
                c,
                t,
                speech_id(),
                request,
                participants(ScriptedSpeechCancellationOutcome::Pending),
                message_id(),
                cancellation(),
                adapter(true),
            )
            .map(|_| ()),
            SpeechInteractionCancellationComposition::new(
                workflow(true),
                w,
                s,
                c,
                t,
                speech_id(),
                request,
                Vec::new(),
                message_id(),
                cancellation(),
                adapter(true),
            )
            .map(|_| ()),
            SpeechInteractionCancellationComposition::new(
                workflow(true),
                w,
                s,
                c,
                t,
                speech_id(),
                request,
                participants(ScriptedSpeechCancellationOutcome::Pending),
                message_id(),
                cancellation(),
                adapter(false),
            )
            .map(|_| ()),
        ];
        assert_eq!(
            cases[0],
            Err(SpeechInteractionCancellationError::InvalidWorkflow)
        );
        assert_eq!(
            cases[1],
            Err(SpeechInteractionCancellationError::InvalidSpeechParticipants)
        );
        assert_eq!(
            cases[2],
            Err(SpeechInteractionCancellationError::BehaviorCapabilityUnavailable)
        );
    }

    #[test]
    fn every_participant_preflight_rejection_is_non_consuming() {
        let (w, s, c, t) = ids();
        let request = SpeechCancellationRequest::new(speech_id());
        let other_speech = SpeechId::new(Uuid::from_u128(99)).unwrap();
        let unsupported = nexa_domain::ProtocolVersion::new(9, 9);
        let capability_cases = vec![
            SpeechCancellationSurface::ALL[..3]
                .iter()
                .map(|&surface| SpeechCancellationCapability::cancellable(speech_id(), surface))
                .collect(),
            vec![
                SpeechCancellationCapability::cancellable(
                    speech_id(),
                    SpeechCancellationSurface::Synthesis
                );
                4
            ],
            SpeechCancellationSurface::ALL
                .into_iter()
                .map(|surface| SpeechCancellationCapability {
                    contract_version: unsupported,
                    speech_id: speech_id(),
                    surface,
                    cancellable: true,
                })
                .collect(),
            SpeechCancellationSurface::ALL
                .into_iter()
                .map(|surface| SpeechCancellationCapability {
                    contract_version: SPEECH_CANCELLATION_V1,
                    speech_id: speech_id(),
                    surface,
                    cancellable: surface != SpeechCancellationSurface::Playback,
                })
                .collect(),
            SpeechCancellationSurface::ALL
                .into_iter()
                .map(|surface| SpeechCancellationCapability::cancellable(other_speech, surface))
                .collect(),
        ];
        for capabilities in capability_cases {
            let (tracked, erased) =
                tracked_participants(capabilities, [ScriptedSpeechCancellationOutcome::Pending]);
            assert_eq!(
                SpeechInteractionCancellationComposition::new(
                    workflow(true),
                    w,
                    s,
                    c,
                    t,
                    speech_id(),
                    request,
                    erased,
                    message_id(),
                    cancellation(),
                    adapter(true)
                )
                .map(|_| ()),
                Err(SpeechInteractionCancellationError::InvalidSpeechParticipants)
            );
            for participant in tracked {
                assert!(participant.received_requests().is_empty());
                assert_eq!(participant.consumed_outcome_count(), 0);
                assert_eq!(participant.active_future_count(), 0);
            }
        }

        let (tracked, erased) = tracked_participants(
            SpeechCancellationSurface::ALL
                .into_iter()
                .map(|surface| SpeechCancellationCapability::cancellable(speech_id(), surface)),
            [ScriptedSpeechCancellationOutcome::Pending],
        );
        let mut wrong_version = request;
        wrong_version.contract_version = unsupported;
        for (workflow_id, supplied_request) in [
            (WorkflowId::new(Uuid::from_u128(88)).unwrap(), request),
            (w, wrong_version),
            (w, SpeechCancellationRequest::new(other_speech)),
        ] {
            assert!(SpeechInteractionCancellationComposition::new(
                workflow(true),
                workflow_id,
                s,
                c,
                t,
                speech_id(),
                supplied_request,
                erased.clone(),
                message_id(),
                cancellation(),
                adapter(true)
            )
            .is_err());
        }
        for participant in tracked {
            assert!(participant.received_requests().is_empty());
            assert_eq!(participant.consumed_outcome_count(), 0);
            assert_eq!(participant.active_future_count(), 0);
        }
    }

    #[test]
    fn acknowledgement_mismatch_and_exhaustion_are_terminal_with_exact_accounting() {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                for outcome in [
                    ScriptedSpeechCancellationOutcome::Acknowledged(
                        SpeechCancellationAcknowledgement::for_request(
                            &SpeechCancellationRequest::new(
                                SpeechId::new(Uuid::from_u128(99)).unwrap(),
                            ),
                        ),
                    ),
                    ScriptedSpeechCancellationOutcome::DependencyFailure,
                ] {
                    let (tracked, erased) = tracked_participants(
                        SpeechCancellationSurface::ALL.into_iter().map(|surface| {
                            SpeechCancellationCapability::cancellable(speech_id(), surface)
                        }),
                        [outcome],
                    );
                    let (w, s, c, t) = ids();
                    let request = SpeechCancellationRequest::new(speech_id());
                    let mut value = SpeechInteractionCancellationComposition::new(
                        workflow(true),
                        w,
                        s,
                        c,
                        t,
                        speech_id(),
                        request,
                        erased,
                        message_id(),
                        cancellation(),
                        adapter(true),
                    )
                    .unwrap();
                    assert_eq!(
                        value.execute(request, message_id(), cancellation()).await,
                        Err(SpeechInteractionCancellationError::RuntimeFailure)
                    );
                    assert_eq!(
                        value.execute(request, message_id(), cancellation()).await,
                        Err(SpeechInteractionCancellationError::RuntimeFailure)
                    );
                    assert_eq!(value.inspect_adapter(|a| a.requests().len()).unwrap(), 1);
                    for participant in tracked {
                        assert_eq!(participant.received_requests(), vec![request]);
                        assert_eq!(participant.consumed_outcome_count(), 1);
                        assert_eq!(participant.remaining_outcome_count(), 0);
                        assert_eq!(participant.active_future_count(), 0);
                    }
                }

                let (tracked, erased) = tracked_participants(
                    SpeechCancellationSurface::ALL.into_iter().map(|surface| {
                        SpeechCancellationCapability::cancellable(speech_id(), surface)
                    }),
                    std::iter::empty::<ScriptedSpeechCancellationOutcome>(),
                );
                let (w, s, c, t) = ids();
                let request = SpeechCancellationRequest::new(speech_id());
                let avatar = InspectableAvatar::new(true, true, true);
                let avatar_handle = avatar.clone();
                let mut value = SpeechInteractionCancellationComposition::new(
                    workflow(true),
                    w,
                    s,
                    c,
                    t,
                    speech_id(),
                    request,
                    erased,
                    message_id(),
                    cancellation(),
                    avatar,
                )
                .unwrap();
                assert_eq!(
                    value.execute(request, message_id(), cancellation()).await,
                    Err(SpeechInteractionCancellationError::RuntimeFailure)
                );
                assert_eq!(avatar_handle.request_count(), 1);
                for participant in &tracked {
                    assert_eq!(participant.received_requests(), vec![request]);
                    assert_eq!(participant.consumed_outcome_count(), 0);
                    assert_eq!(participant.remaining_outcome_count(), 0);
                    assert_eq!(participant.active_future_count(), 0);
                }
                assert_eq!(
                    value.execute(request, message_id(), cancellation()).await,
                    Err(SpeechInteractionCancellationError::RuntimeFailure)
                );
                assert_eq!(avatar_handle.request_count(), 1);
                assert!(tracked
                    .iter()
                    .all(|participant| participant.received_requests() == vec![request]));
            });
    }

    #[test]
    fn runtime_evidence_mismatch_is_terminal_with_no_residual_work() {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                let request = SpeechCancellationRequest::new(speech_id());
                let (tracked, erased) = tracked_participants(
                    SpeechCancellationSurface::ALL.into_iter().map(|surface| {
                        SpeechCancellationCapability::cancellable(speech_id(), surface)
                    }),
                    [ScriptedSpeechCancellationOutcome::Acknowledged(
                        SpeechCancellationAcknowledgement::for_request(&request),
                    )],
                );
                let avatar = InspectableAvatar::new(true, true, true);
                let avatar_handle = avatar.clone();
                let (w, s, c, t) = ids();
                let mut value = SpeechInteractionCancellationComposition::new(
                    workflow(true),
                    w,
                    s,
                    c,
                    t,
                    speech_id(),
                    request,
                    erased,
                    message_id(),
                    cancellation(),
                    avatar,
                )
                .unwrap();
                value.test_seam.runtime_evidence_mismatch = true;
                assert_eq!(
                    value.execute(request, message_id(), cancellation()).await,
                    Err(SpeechInteractionCancellationError::RuntimeFailure)
                );
                assert_eq!(avatar_handle.request_count(), 1);
                for participant in &tracked {
                    assert_eq!(participant.received_requests(), vec![request]);
                    assert_eq!(participant.consumed_outcome_count(), 1);
                    assert_eq!(participant.remaining_outcome_count(), 0);
                    assert_eq!(participant.active_future_count(), 0);
                }
                assert_eq!(
                    value.execute(request, message_id(), cancellation()).await,
                    Err(SpeechInteractionCancellationError::RuntimeFailure)
                );
                assert_eq!(avatar_handle.request_count(), 1);
            });
    }

    #[test]
    fn caller_drop_aborts_pending_speech_and_prevents_retry() {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                let (tracked, erased) = tracked_participants(
                    SpeechCancellationSurface::ALL.into_iter().map(|surface| {
                        SpeechCancellationCapability::cancellable(speech_id(), surface)
                    }),
                    [ScriptedSpeechCancellationOutcome::Pending],
                );
                let (w, s, c, t) = ids();
                let request = SpeechCancellationRequest::new(speech_id());
                let mut value = SpeechInteractionCancellationComposition::new(
                    workflow(true),
                    w,
                    s,
                    c,
                    t,
                    speech_id(),
                    request,
                    erased,
                    message_id(),
                    cancellation(),
                    adapter(true),
                )
                .unwrap();
                let behavior_active = Arc::new(AtomicUsize::new(0));
                value.test_seam.behavior_active = Some(Arc::clone(&behavior_active));
                value.test_seam.behavior_pending = true;
                let mut execution = Box::pin(value.execute(request, message_id(), cancellation()));
                loop {
                    tokio::select! {
                        _ = &mut execution => panic!("pending script unexpectedly completed"),
                        _ = tokio::task::yield_now() => {}
                    }
                    if tracked
                        .iter()
                        .all(|participant| participant.active_future_count() == 1)
                        && behavior_active.load(Ordering::SeqCst) == 1
                    {
                        break;
                    }
                }
                drop(execution);
                tokio::task::yield_now().await;
                assert!(tracked
                    .iter()
                    .all(|participant| participant.active_future_count() == 0));
                assert_eq!(behavior_active.load(Ordering::SeqCst), 0);
                assert_eq!(value.inspect_adapter(|a| a.requests().len()).unwrap(), 1);
                assert_eq!(
                    value.execute(request, message_id(), cancellation()).await,
                    Err(SpeechInteractionCancellationError::RuntimeFailure)
                );
                assert!(tracked
                    .iter()
                    .all(|participant| participant.consumed_outcome_count() == 1));
                assert!(tracked
                    .iter()
                    .all(|participant| participant.received_requests() == vec![request]));
            });
    }

    #[test]
    fn every_construction_failure_has_zero_dependency_side_effects() {
        let (w, s, c, t) = ids();
        let request = SpeechCancellationRequest::new(speech_id());
        let other_speech = SpeechId::new(Uuid::from_u128(99)).unwrap();
        let unsupported = nexa_domain::ProtocolVersion::new(9, 9);
        let valid_capabilities = || {
            SpeechCancellationSurface::ALL
                .into_iter()
                .map(|surface| SpeechCancellationCapability::cancellable(speech_id(), surface))
                .collect::<Vec<_>>()
        };
        enum Case {
            Association(
                InteractionWorkflow,
                WorkflowId,
                SessionId,
                CorrelationId,
                TraceId,
            ),
            Request(SpeechCancellationRequest),
            Capabilities(Vec<SpeechCancellationCapability>),
            Behavior(bool, bool),
        }
        let mut wrong_version = request;
        wrong_version.contract_version = unsupported;
        let cases = vec![
            Case::Association(workflow(false), w, s, c, t),
            Case::Association(
                workflow(true),
                WorkflowId::new(Uuid::from_u128(88)).unwrap(),
                s,
                c,
                t,
            ),
            Case::Association(
                workflow(true),
                w,
                SessionId::new(Uuid::from_u128(89)).unwrap(),
                c,
                t,
            ),
            Case::Association(
                workflow(true),
                w,
                s,
                CorrelationId::new(Uuid::from_u128(90)).unwrap(),
                t,
            ),
            Case::Association(
                workflow(true),
                w,
                s,
                c,
                TraceId::new(Uuid::from_u128(91)).unwrap(),
            ),
            Case::Request(wrong_version),
            Case::Request(SpeechCancellationRequest::new(other_speech)),
            Case::Capabilities(valid_capabilities()[..3].to_vec()),
            Case::Capabilities(vec![valid_capabilities()[0]; 4]),
            Case::Capabilities(
                valid_capabilities()
                    .into_iter()
                    .map(|mut value| {
                        value.contract_version = unsupported;
                        value
                    })
                    .collect(),
            ),
            Case::Capabilities(
                valid_capabilities()
                    .into_iter()
                    .map(|mut value| {
                        value.cancellable = false;
                        value
                    })
                    .collect(),
            ),
            Case::Capabilities(
                valid_capabilities()
                    .into_iter()
                    .map(|mut value| {
                        value.speech_id = other_speech;
                        value
                    })
                    .collect(),
            ),
            Case::Behavior(false, true),
            Case::Behavior(true, false),
        ];
        for case in cases {
            let (
                workflow_value,
                workflow_id,
                session_id,
                correlation_id,
                trace_id,
                supplied_request,
                capabilities,
                capable,
                preview,
            ) = match case {
                Case::Association(value, workflow_id, session_id, correlation_id, trace_id) => (
                    value,
                    workflow_id,
                    session_id,
                    correlation_id,
                    trace_id,
                    request,
                    valid_capabilities(),
                    true,
                    true,
                ),
                Case::Request(value) => (
                    workflow(true),
                    w,
                    s,
                    c,
                    t,
                    value,
                    valid_capabilities(),
                    true,
                    true,
                ),
                Case::Capabilities(values) => {
                    (workflow(true), w, s, c, t, request, values, true, true)
                }
                Case::Behavior(capable, preview) => (
                    workflow(true),
                    w,
                    s,
                    c,
                    t,
                    request,
                    valid_capabilities(),
                    capable,
                    preview,
                ),
            };
            let (tracked, erased) =
                tracked_participants(capabilities, [ScriptedSpeechCancellationOutcome::Pending]);
            let avatar = InspectableAvatar::new(capable, preview, true);
            let avatar_handle = avatar.clone();
            assert!(SpeechInteractionCancellationComposition::new(
                workflow_value,
                workflow_id,
                session_id,
                correlation_id,
                trace_id,
                speech_id(),
                supplied_request,
                erased,
                message_id(),
                cancellation(),
                avatar,
            )
            .is_err());
            assert_eq!(avatar_handle.request_count(), 0);
            for participant in tracked {
                assert!(participant.received_requests().is_empty());
                assert_eq!(participant.consumed_outcome_count(), 0);
                assert_eq!(participant.active_future_count(), 0);
            }
        }
    }

    #[test]
    fn behavior_mismatch_task_failure_and_missing_result_are_terminal_and_joined() {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                for seam in 0..3 {
                    let request = SpeechCancellationRequest::new(speech_id());
                    let acknowledged = ScriptedSpeechCancellationOutcome::Acknowledged(
                        SpeechCancellationAcknowledgement::for_request(&request),
                    );
                    let (tracked, erased) = tracked_participants(
                        SpeechCancellationSurface::ALL.into_iter().map(|surface| {
                            SpeechCancellationCapability::cancellable(speech_id(), surface)
                        }),
                        [acknowledged],
                    );
                    let avatar = InspectableAvatar::new(true, true, seam != 0);
                    let avatar_handle = avatar.clone();
                    let (w, s, c, t) = ids();
                    let mut value = SpeechInteractionCancellationComposition::new(
                        workflow(true),
                        w,
                        s,
                        c,
                        t,
                        speech_id(),
                        request,
                        erased,
                        message_id(),
                        cancellation(),
                        avatar,
                    )
                    .unwrap();
                    value.test_seam.behavior_panics = seam == 1;
                    value.test_seam.omit_behavior_result = seam == 2;
                    assert_eq!(
                        value.execute(request, message_id(), cancellation()).await,
                        Err(SpeechInteractionCancellationError::RuntimeFailure)
                    );
                    assert_eq!(avatar_handle.request_count(), 1);
                    for participant in tracked {
                        assert_eq!(participant.received_requests(), vec![request]);
                        assert_eq!(participant.consumed_outcome_count(), 1);
                        assert_eq!(participant.remaining_outcome_count(), 0);
                        assert_eq!(participant.active_future_count(), 0);
                    }
                    assert_eq!(
                        value.execute(request, message_id(), cancellation()).await,
                        Err(SpeechInteractionCancellationError::RuntimeFailure)
                    );
                    assert_eq!(avatar_handle.request_count(), 1);
                }
            });
    }

    #[test]
    fn behavior_failure_after_both_targets_start_aborts_pending_speech() {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                let request = SpeechCancellationRequest::new(speech_id());
                let (tracked, erased) = tracked_participants(
                    SpeechCancellationSurface::ALL.into_iter().map(|surface| {
                        SpeechCancellationCapability::cancellable(speech_id(), surface)
                    }),
                    [ScriptedSpeechCancellationOutcome::Pending],
                );
                let avatar = InspectableAvatar::new(true, true, true);
                let avatar_handle = avatar.clone();
                let (w, s, c, t) = ids();
                let mut value = SpeechInteractionCancellationComposition::new(
                    workflow(true),
                    w,
                    s,
                    c,
                    t,
                    speech_id(),
                    request,
                    erased,
                    message_id(),
                    cancellation(),
                    avatar,
                )
                .unwrap();
                let active = Arc::new(AtomicUsize::new(0));
                value.test_seam.behavior_active = Some(Arc::clone(&active));
                let participants = tracked.clone();
                value.test_seam.behavior_panics_when = Some(Arc::new(move || {
                    participants
                        .iter()
                        .all(|participant| participant.active_future_count() == 1)
                }));

                assert_eq!(
                    value.execute(request, message_id(), cancellation()).await,
                    Err(SpeechInteractionCancellationError::RuntimeFailure)
                );
                assert_eq!(active.load(Ordering::SeqCst), 0);
                assert_eq!(avatar_handle.request_count(), 1);
                for participant in &tracked {
                    assert_eq!(participant.received_requests(), vec![request]);
                    assert_eq!(participant.consumed_outcome_count(), 1);
                    assert_eq!(participant.remaining_outcome_count(), 0);
                    assert_eq!(participant.active_future_count(), 0);
                }
                assert_eq!(
                    value.execute(request, message_id(), cancellation()).await,
                    Err(SpeechInteractionCancellationError::RuntimeFailure)
                );
                assert_eq!(avatar_handle.request_count(), 1);
                assert_eq!(active.load(Ordering::SeqCst), 0);
            });
    }

    #[test]
    fn participant_order_and_separate_constructions_produce_identical_evidence() {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                async fn execute_with_order(
                    order: impl IntoIterator<Item = SpeechCancellationSurface>,
                ) -> SpeechInteractionCancellationEvidence {
                    let request = SpeechCancellationRequest::new(speech_id());
                    let (tracked, erased) = tracked_participants(
                        order.into_iter().map(|surface| {
                            SpeechCancellationCapability::cancellable(speech_id(), surface)
                        }),
                        [ScriptedSpeechCancellationOutcome::Acknowledged(
                            SpeechCancellationAcknowledgement::for_request(&request),
                        )],
                    );
                    let (w, s, c, t) = ids();
                    let mut value = SpeechInteractionCancellationComposition::new(
                        workflow(true),
                        w,
                        s,
                        c,
                        t,
                        speech_id(),
                        request,
                        erased,
                        message_id(),
                        cancellation(),
                        adapter(true),
                    )
                    .unwrap();
                    let evidence = value
                        .execute(request, message_id(), cancellation())
                        .await
                        .unwrap();
                    for (participant, surface) in tracked
                        .iter()
                        .zip(tracked.iter().map(|p| p.capability().surface))
                    {
                        assert_eq!(participant.capability().surface, surface);
                        assert_eq!(participant.received_requests(), vec![request]);
                        assert_eq!(participant.consumed_outcome_count(), 1);
                        assert_eq!(participant.remaining_outcome_count(), 0);
                        assert_eq!(participant.active_future_count(), 0);
                    }
                    evidence
                }

                let canonical = execute_with_order(SpeechCancellationSurface::ALL).await;
                let reversed =
                    execute_with_order(SpeechCancellationSurface::ALL.into_iter().rev()).await;
                let separate = execute_with_order(SpeechCancellationSurface::ALL).await;
                assert_eq!(canonical, reversed);
                assert_eq!(canonical, separate);
            });
    }

    #[test]
    fn speech_failure_is_terminal_and_public_errors_are_content_free() {
        let (w, s, c, t) = ids();
        let request = SpeechCancellationRequest::new(speech_id());
        let mut value = SpeechInteractionCancellationComposition::new(
            workflow(true),
            w,
            s,
            c,
            t,
            speech_id(),
            request,
            participants(ScriptedSpeechCancellationOutcome::DependencyFailure),
            message_id(),
            cancellation(),
            adapter(true),
        )
        .unwrap();
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                assert_eq!(
                    value.execute(request, message_id(), cancellation()).await,
                    Err(SpeechInteractionCancellationError::RuntimeFailure)
                );
                assert_eq!(
                    value.execute(request, message_id(), cancellation()).await,
                    Err(SpeechInteractionCancellationError::RuntimeFailure)
                );
                assert_eq!(value.inspect_adapter(|a| a.requests().len()).unwrap(), 1);
            });
        for error in [
            SpeechInteractionCancellationError::InvalidWorkflow,
            SpeechInteractionCancellationError::AssociationMismatch,
            SpeechInteractionCancellationError::InvalidSpeech,
            SpeechInteractionCancellationError::InvalidSpeechParticipants,
            SpeechInteractionCancellationError::BehaviorCapabilityUnavailable,
            SpeechInteractionCancellationError::InvalidBehaviorPreview,
            SpeechInteractionCancellationError::RuntimeFailure,
            SpeechInteractionCancellationError::ConflictingExecution,
        ] {
            let diagnostic = format!("{error:?} {error}");
            for canary in [
                "learner-private",
                "synthesized-private",
                "private-behavior-reason",
                "audio-private",
                "provider-private",
                "endpoint-private",
                "credential-private",
                "task-identifier-private",
                "dependency-error-private",
            ] {
                assert!(!diagnostic.contains(canary));
            }
        }
    }
}
