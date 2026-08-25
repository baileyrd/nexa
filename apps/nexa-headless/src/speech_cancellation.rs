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
    SpeechCancellationAggregateEvidence, SpeechCancellationCoordinator,
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
#[derive(Clone, Debug, PartialEq)]
pub struct SpeechInteractionCancellationEvidence {
    runtime: WorkflowCancellationExecution,
    speech: SpeechCancellationAggregateEvidence,
    behavior: AvatarCancellationEvidence,
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
    speech_id: SpeechId,
    speech_request: SpeechCancellationRequest,
    participants: Vec<Arc<dyn SpeechCancellationParticipant>>,
    behavior_request: AvatarRequest,
    behavior_preview: AvatarReport,
    adapter: Arc<Mutex<A>>,
    plan: WorkflowCancellationPlan,
    terminal: ExecutionState,
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
        SpeechCancellationCoordinator::new(
            speech_id,
            participants.iter().map(|participant| participant.as_ref()),
        )
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
            speech_id,
            speech_request,
            participants,
            behavior_request,
            behavior_preview,
            adapter: Arc::new(Mutex::new(adapter)),
            plan,
            terminal: ExecutionState::Ready,
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
        let participants = self.participants.clone();
        let speech_id = self.speech_id;
        let request = self.speech_request;
        if tasks
            .spawn_for_target(CancellationTarget::Speech, move |token| async move {
                token.cancelled().await;
                let coordinator = SpeechCancellationCoordinator::new(
                    speech_id,
                    participants.iter().map(|participant| participant.as_ref()),
                );
                let result = match coordinator {
                    Ok(coordinator) => coordinator.cancel(request).await,
                    Err(error) => Err(error),
                };
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
        if tasks
            .spawn_for_target(CancellationTarget::Behavior, move |token| async move {
                token.cancelled().await;
                let invoked = request.clone();
                let report = adapter
                    .lock()
                    .ok()
                    .map(|mut adapter| adapter.handle(request));
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
        if speech.speech_id != self.speech_id
            || invoked != self.behavior_request
            || report != self.behavior_preview
            || report.terminal_status() != RuntimeStatus::Cancelled
            || runtime.target_outcomes().len() != 2
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
    use uuid::Uuid;

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
    fn composition() -> SpeechInteractionCancellationComposition<FakeAvatarAdapter> {
        let (w, s, c, t) = ids();
        let request = SpeechCancellationRequest::new(speech_id());
        SpeechInteractionCancellationComposition::new(
            workflow(true),
            w,
            s,
            c,
            t,
            speech_id(),
            request,
            participants(ScriptedSpeechCancellationOutcome::Acknowledged(
                SpeechCancellationAcknowledgement::for_request(&request),
            )),
            message_id(),
            cancellation(),
            adapter(true),
        )
        .unwrap()
    }

    #[test]
    fn canonical_two_target_success_is_joined_exact_and_idempotent() {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                let mut value = composition();
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
                let request = SpeechCancellationRequest::new(speech_id());
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
                assert_eq!(evidence.runtime().target_outcomes().len(), 2);
                assert!(evidence
                    .runtime()
                    .target_outcomes()
                    .iter()
                    .all(|e| e.outcome() == CancellationTargetExecutionOutcome::Stopped));
                assert_eq!(evidence.behavior().cancellation_request_count(), 1);
                assert_eq!(
                    value
                        .execute(request, message_id(), cancellation())
                        .await
                        .unwrap(),
                    evidence
                );
                assert_eq!(value.inspect_adapter(|a| a.requests().len()).unwrap(), 1);
                let mut conflict = cancellation();
                conflict.reason = "conflict".into();
                assert_eq!(
                    value.execute(request, message_id(), conflict).await,
                    Err(SpeechInteractionCancellationError::ConflictingExecution)
                );
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
            ] {
                assert!(!diagnostic.contains(canary));
            }
        }
    }
}
