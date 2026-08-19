//! Typed event envelopes and a minimal runtime-neutral local bus.
#![forbid(unsafe_code)]

use nexa_domain::{
    AssessmentId, AssessmentItemInstanceId, AttemptId, BehaviorId, CompetencyId, CorrelationId,
    EndpointId, EventId, EvidenceId, LessonId, LessonStepId, LessonTransitionId, MasteryScore,
    MessageId, ProtocolVersion, ResponseId, SemanticKey, Sequence, SessionId, StudentId, SubjectId,
    Timestamp, TraceId,
};
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// The closed event catalog required by the Phase 1 vertical slice.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    #[serde(rename = "system.ready")]
    SystemReady,
    #[serde(rename = "system.error")]
    SystemError,
    #[serde(rename = "session.started")]
    SessionStarted,
    #[serde(rename = "session.ended")]
    SessionEnded,
    #[serde(rename = "student.text.submitted")]
    StudentTextSubmitted,
    #[serde(rename = "tutor.response.started")]
    TutorResponseStarted,
    #[serde(rename = "tutor.response.completed")]
    TutorResponseCompleted,
    #[serde(rename = "tutor.response.failed")]
    TutorResponseFailed,
    #[serde(rename = "speech.synthesis.started")]
    SpeechSynthesisStarted,
    #[serde(rename = "speech.synthesis.completed")]
    SpeechSynthesisCompleted,
    #[serde(rename = "speech.playback.started")]
    SpeechPlaybackStarted,
    #[serde(rename = "speech.playback.completed")]
    SpeechPlaybackCompleted,
    #[serde(rename = "avatar.state.changed")]
    AvatarStateChanged,
    #[serde(rename = "avatar.behavior.accepted")]
    AvatarBehaviorAccepted,
    #[serde(rename = "avatar.behavior.started")]
    AvatarBehaviorStarted,
    #[serde(rename = "avatar.behavior.completed")]
    AvatarBehaviorCompleted,
    #[serde(rename = "avatar.behavior.cancelled")]
    AvatarBehaviorCancelled,
    #[serde(rename = "avatar.behavior.degraded")]
    AvatarBehaviorDegraded,
    #[serde(rename = "avatar.behavior.failed")]
    AvatarBehaviorFailed,
    #[serde(rename = "competency.evidence.added")]
    CompetencyEvidenceAdded,
    #[serde(rename = "competency.updated")]
    CompetencyUpdated,
    #[serde(rename = "pedagogy.decision.made")]
    PedagogyDecisionMade,
    #[serde(rename = "lesson.lifecycle.changed")]
    LessonLifecycleChanged,
    #[serde(rename = "lesson.transition.applied")]
    LessonTransitionApplied,
    #[serde(rename = "assessment.response.evaluated")]
    AssessmentResponseEvaluated,
    #[serde(rename = "assessment.completed")]
    AssessmentCompleted,
}

/// Operational, non-domain envelope metadata. Never place secrets here.
pub type EventMetadata = serde_json::Map<String, Value>;

/// One immutable typed fact. Sequence scope is `(source, session_id)`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Event<T> {
    pub event_version: ProtocolVersion,
    pub event_id: EventId,
    event_type: EventKind,
    pub timestamp: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<SessionId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence: Option<Sequence>,
    pub source: EndpointId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<SubjectId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<CorrelationId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causation_id: Option<EventId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<TraceId>,
    pub payload: T,
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub metadata: EventMetadata,
}

/// A type-safe event payload declares its immutable catalog identity.
pub trait DomainEvent:
    Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static
{
    const KIND: EventKind;
}

impl<T: DomainEvent> Event<T> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        event_version: ProtocolVersion,
        event_id: EventId,
        timestamp: Timestamp,
        session_id: Option<SessionId>,
        sequence: Option<Sequence>,
        source: EndpointId,
        subject: Option<SubjectId>,
        correlation_id: Option<CorrelationId>,
        causation_id: Option<EventId>,
        trace_id: Option<TraceId>,
        payload: T,
        metadata: EventMetadata,
    ) -> Self {
        Self {
            event_version,
            event_id,
            event_type: T::KIND,
            timestamp,
            session_id,
            sequence,
            source,
            subject,
            correlation_id,
            causation_id,
            trace_id,
            payload,
            metadata,
        }
    }

    pub const fn event_type(&self) -> EventKind {
        self.event_type
    }
}

impl<'de, T: DomainEvent> Deserialize<'de> for Event<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct WireEvent<T> {
            event_version: ProtocolVersion,
            event_id: EventId,
            event_type: EventKind,
            timestamp: Timestamp,
            session_id: Option<SessionId>,
            sequence: Option<Sequence>,
            source: EndpointId,
            subject: Option<SubjectId>,
            correlation_id: Option<CorrelationId>,
            causation_id: Option<EventId>,
            trace_id: Option<TraceId>,
            payload: T,
            #[serde(default)]
            metadata: EventMetadata,
        }

        let wire = WireEvent::<T>::deserialize(deserializer)?;
        if wire.event_type != T::KIND {
            return Err(de::Error::custom(format_args!(
                "event_type {:?} does not match payload kind {:?}",
                wire.event_type,
                T::KIND
            )));
        }
        Ok(Self::new(
            wire.event_version,
            wire.event_id,
            wire.timestamp,
            wire.session_id,
            wire.sequence,
            wire.source,
            wire.subject,
            wire.correlation_id,
            wire.causation_id,
            wire.trace_id,
            wire.payload,
            wire.metadata,
        ))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionStarted {
    pub mode: String,
}
impl DomainEvent for SessionStarted {
    const KIND: EventKind = EventKind::SessionStarted;
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionEnded {
    pub reason: String,
}
impl DomainEvent for SessionEnded {
    const KIND: EventKind = EventKind::SessionEnded;
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StudentTextSubmitted {
    pub text: String,
}
impl DomainEvent for StudentTextSubmitted {
    const KIND: EventKind = EventKind::StudentTextSubmitted;
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TutorResponseCompleted {
    pub text: String,
}
impl DomainEvent for TutorResponseCompleted {
    const KIND: EventKind = EventKind::TutorResponseCompleted;
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SpeechSynthesisCompleted {
    pub speech_ref: String,
}
impl DomainEvent for SpeechSynthesisCompleted {
    const KIND: EventKind = EventKind::SpeechSynthesisCompleted;
}
macro_rules! avatar_lifecycle {
    ($name:ident, $kind:ident) => {
        #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
        pub struct $name {
            pub message_id: MessageId,
            pub behavior_id: BehaviorId,
        }
        impl DomainEvent for $name {
            const KIND: EventKind = EventKind::$kind;
        }
    };
}
avatar_lifecycle!(AvatarBehaviorAccepted, AvatarBehaviorAccepted);
avatar_lifecycle!(AvatarBehaviorStarted, AvatarBehaviorStarted);
avatar_lifecycle!(AvatarBehaviorCompleted, AvatarBehaviorCompleted);
avatar_lifecycle!(AvatarBehaviorCancelled, AvatarBehaviorCancelled);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AvatarBehaviorDegraded {
    pub message_id: MessageId,
    pub behavior_id: BehaviorId,
    pub reason: SemanticKey,
}
impl DomainEvent for AvatarBehaviorDegraded {
    const KIND: EventKind = EventKind::AvatarBehaviorDegraded;
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AvatarBehaviorFailed {
    pub message_id: MessageId,
    pub behavior_id: BehaviorId,
    pub reason: SemanticKey,
}
impl DomainEvent for AvatarBehaviorFailed {
    const KIND: EventKind = EventKind::AvatarBehaviorFailed;
}

/// Privacy-minimal notification that immutable learning evidence was accepted.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CompetencyEvidenceAdded {
    pub evidence_id: EvidenceId,
    pub student_id: StudentId,
    pub competency_id: CompetencyId,
    pub evidence_type: SemanticKey,
    pub outcome: SemanticKey,
}
impl DomainEvent for CompetencyEvidenceAdded {
    const KIND: EventKind = EventKind::CompetencyEvidenceAdded;
}

/// Notification that replayable evidence changed a mastery projection.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompetencyUpdated {
    pub evidence_id: EvidenceId,
    pub student_id: StudentId,
    pub competency_id: CompetencyId,
    pub previous_mastery: MasteryScore,
    pub new_mastery: MasteryScore,
    pub policy_version: ProtocolVersion,
}
impl DomainEvent for CompetencyUpdated {
    const KIND: EventKind = EventKind::CompetencyUpdated;
}

/// Privacy-minimal deterministic evaluation fact; response content and answer keys are excluded.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentEvaluationOutcome {
    Correct,
    Partial,
    Incorrect,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssessmentResponseEvaluated {
    pub attempt_id: AttemptId,
    pub assessment_id: AssessmentId,
    pub item_instance_id: AssessmentItemInstanceId,
    pub response_id: ResponseId,
    pub score: MasteryScore,
    pub outcome: AssessmentEvaluationOutcome,
    pub policy_version: ProtocolVersion,
}
impl DomainEvent for AssessmentResponseEvaluated {
    const KIND: EventKind = EventKind::AssessmentResponseEvaluated;
}

/// Privacy-minimal terminal assessment fact.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "AssessmentCompletedWire")]
pub struct AssessmentCompleted {
    pub attempt_id: AttemptId,
    pub assessment_id: AssessmentId,
    pub score: MasteryScore,
    pub passing_score: MasteryScore,
    pub passed: bool,
    pub policy_version: ProtocolVersion,
}
#[derive(Deserialize)]
struct AssessmentCompletedWire {
    attempt_id: AttemptId,
    assessment_id: AssessmentId,
    score: MasteryScore,
    passing_score: MasteryScore,
    passed: bool,
    policy_version: ProtocolVersion,
}
impl TryFrom<AssessmentCompletedWire> for AssessmentCompleted {
    type Error = &'static str;
    fn try_from(w: AssessmentCompletedWire) -> Result<Self, Self::Error> {
        if w.passed != (w.score.get() >= w.passing_score.get()) {
            return Err("passed must agree with score and passing_score");
        }
        Ok(Self {
            attempt_id: w.attempt_id,
            assessment_id: w.assessment_id,
            score: w.score,
            passing_score: w.passing_score,
            passed: w.passed,
            policy_version: w.policy_version,
        })
    }
}
impl DomainEvent for AssessmentCompleted {
    const KIND: EventKind = EventKind::AssessmentCompleted;
}

/// Privacy-minimal pedagogy routing fact. Vocabulary remains semantic to avoid a crate cycle.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "PedagogyDecisionMadeWire")]
pub struct PedagogyDecisionMade {
    pub student_id: StudentId,
    pub competency_id: CompetencyId,
    pub selected_option: SemanticKey,
    rationale_codes: Vec<SemanticKey>,
    pub policy_version: ProtocolVersion,
}

#[derive(Deserialize)]
struct PedagogyDecisionMadeWire {
    student_id: StudentId,
    competency_id: CompetencyId,
    selected_option: SemanticKey,
    rationale_codes: Vec<SemanticKey>,
    policy_version: ProtocolVersion,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("rationale_codes must be nonempty, sorted, and unique")]
pub struct InvalidPedagogyDecisionMade;

impl TryFrom<PedagogyDecisionMadeWire> for PedagogyDecisionMade {
    type Error = InvalidPedagogyDecisionMade;

    fn try_from(value: PedagogyDecisionMadeWire) -> Result<Self, Self::Error> {
        if value.rationale_codes.is_empty()
            || !value
                .rationale_codes
                .windows(2)
                .all(|pair| pair[0] < pair[1])
        {
            return Err(InvalidPedagogyDecisionMade);
        }
        Ok(Self {
            student_id: value.student_id,
            competency_id: value.competency_id,
            selected_option: value.selected_option,
            rationale_codes: value.rationale_codes,
            policy_version: value.policy_version,
        })
    }
}

impl PedagogyDecisionMade {
    pub fn new(
        student_id: StudentId,
        competency_id: CompetencyId,
        selected_option: SemanticKey,
        rationale_codes: Vec<SemanticKey>,
        policy_version: ProtocolVersion,
    ) -> Result<Self, InvalidPedagogyDecisionMade> {
        Self::try_from(PedagogyDecisionMadeWire {
            student_id,
            competency_id,
            selected_option,
            rationale_codes,
            policy_version,
        })
    }

    pub fn rationale_codes(&self) -> &[SemanticKey] {
        &self.rationale_codes
    }
}
impl DomainEvent for PedagogyDecisionMade {
    const KIND: EventKind = EventKind::PedagogyDecisionMade;
}

/// Privacy-minimal lesson lifecycle fact; content and student profile data are excluded.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LessonEventLifecycle {
    NotStarted,
    Active,
    Waiting,
    Completed,
    Blocked,
    Abandoned,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "LessonLifecycleChangedWire")]
pub struct LessonLifecycleChanged {
    pub transition_id: LessonTransitionId,
    pub student_id: StudentId,
    pub lesson_id: LessonId,
    pub from: LessonEventLifecycle,
    pub to: LessonEventLifecycle,
    pub policy_version: ProtocolVersion,
}
#[derive(Deserialize)]
struct LessonLifecycleChangedWire {
    transition_id: LessonTransitionId,
    student_id: StudentId,
    lesson_id: LessonId,
    from: LessonEventLifecycle,
    to: LessonEventLifecycle,
    policy_version: ProtocolVersion,
}
impl TryFrom<LessonLifecycleChangedWire> for LessonLifecycleChanged {
    type Error = InvalidLessonLifecycleChanged;

    fn try_from(w: LessonLifecycleChangedWire) -> Result<Self, Self::Error> {
        if w.from == w.to {
            return Err(InvalidLessonLifecycleChanged);
        }
        Ok(Self {
            transition_id: w.transition_id,
            student_id: w.student_id,
            lesson_id: w.lesson_id,
            from: w.from,
            to: w.to,
            policy_version: w.policy_version,
        })
    }
}
#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("lesson lifecycle change must use known, distinct lifecycle states")]
pub struct InvalidLessonLifecycleChanged;

impl LessonLifecycleChanged {
    pub fn new(
        transition_id: LessonTransitionId,
        student_id: StudentId,
        lesson_id: LessonId,
        from: LessonEventLifecycle,
        to: LessonEventLifecycle,
        policy_version: ProtocolVersion,
    ) -> Result<Self, InvalidLessonLifecycleChanged> {
        Self::try_from(LessonLifecycleChangedWire {
            transition_id,
            student_id,
            lesson_id,
            from,
            to,
            policy_version,
        })
    }
}
impl DomainEvent for LessonLifecycleChanged {
    const KIND: EventKind = EventKind::LessonLifecycleChanged;
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "LessonTransitionAppliedWire")]
pub struct LessonTransitionApplied {
    pub transition_id: LessonTransitionId,
    pub student_id: StudentId,
    pub lesson_id: LessonId,
    pub from_step_id: Option<LessonStepId>,
    pub to_step_id: Option<LessonStepId>,
    rationale_codes: Vec<SemanticKey>,
    pub policy_version: ProtocolVersion,
}
#[derive(Deserialize)]
struct LessonTransitionAppliedWire {
    transition_id: LessonTransitionId,
    student_id: StudentId,
    lesson_id: LessonId,
    from_step_id: Option<LessonStepId>,
    to_step_id: Option<LessonStepId>,
    rationale_codes: Vec<SemanticKey>,
    policy_version: ProtocolVersion,
}
impl TryFrom<LessonTransitionAppliedWire> for LessonTransitionApplied {
    type Error = InvalidLessonTransitionEvent;
    fn try_from(w: LessonTransitionAppliedWire) -> Result<Self, Self::Error> {
        if w.rationale_codes.is_empty() || !w.rationale_codes.windows(2).all(|x| x[0] < x[1]) {
            return Err(InvalidLessonTransitionEvent);
        }
        Ok(Self {
            transition_id: w.transition_id,
            student_id: w.student_id,
            lesson_id: w.lesson_id,
            from_step_id: w.from_step_id,
            to_step_id: w.to_step_id,
            rationale_codes: w.rationale_codes,
            policy_version: w.policy_version,
        })
    }
}
#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("lesson transition rationale_codes must be nonempty, sorted, and unique")]
pub struct InvalidLessonTransitionEvent;
impl LessonTransitionApplied {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        transition_id: LessonTransitionId,
        student_id: StudentId,
        lesson_id: LessonId,
        from_step_id: Option<LessonStepId>,
        to_step_id: Option<LessonStepId>,
        rationale_codes: Vec<SemanticKey>,
        policy_version: ProtocolVersion,
    ) -> Result<Self, InvalidLessonTransitionEvent> {
        Self::try_from(LessonTransitionAppliedWire {
            transition_id,
            student_id,
            lesson_id,
            from_step_id,
            to_step_id,
            rationale_codes,
            policy_version,
        })
    }
    pub fn rationale_codes(&self) -> &[SemanticKey] {
        &self.rationale_codes
    }
}
impl DomainEvent for LessonTransitionApplied {
    const KIND: EventKind = EventKind::LessonTransitionApplied;
}

/// A subscriber callback failure. Other subscribers are still called.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("subscriber failed: {message}")]
pub struct SubscriberError {
    pub message: String,
}

/// Aggregate publication failures after fan-out completes.
#[derive(Debug, Error)]
pub enum PublishError {
    #[error("event payload encoding failed: {0}")]
    Encoding(#[source] serde_json::Error),
    #[error("{failed_subscribers} subscriber(s) failed")]
    Subscribers { failed_subscribers: usize },
}

type Handler = Arc<dyn Fn(&Event<Value>) -> Result<(), SubscriberError> + Send + Sync>;
#[derive(Clone)]
struct Subscriber {
    kind: Option<EventKind>,
    handler: Handler,
}

/// A deterministic synchronous adapter for tests and local composition.
///
/// It is intentionally non-durable; it attempts matching callbacks in registration order.
#[derive(Clone, Default)]
pub struct InProcessEventBus {
    subscribers: Arc<Mutex<Vec<Subscriber>>>,
}
impl InProcessEventBus {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn subscribe<F>(&self, kind: Option<EventKind>, handler: F)
    where
        F: Fn(&Event<Value>) -> Result<(), SubscriberError> + Send + Sync + 'static,
    {
        self.subscribers
            .lock()
            .expect("subscriber mutex poisoned")
            .push(Subscriber {
                kind,
                handler: Arc::new(handler),
            });
    }
    pub fn publish<T: Serialize>(&self, event: &Event<T>) -> Result<(), PublishError> {
        let erased = Event {
            event_version: event.event_version,
            event_id: event.event_id,
            event_type: event.event_type,
            timestamp: event.timestamp,
            session_id: event.session_id,
            sequence: event.sequence,
            source: event.source.clone(),
            subject: event.subject.clone(),
            correlation_id: event.correlation_id,
            causation_id: event.causation_id,
            trace_id: event.trace_id,
            payload: serde_json::to_value(&event.payload).map_err(PublishError::Encoding)?,
            metadata: event.metadata.clone(),
        };
        let subscribers = self
            .subscribers
            .lock()
            .expect("subscriber mutex poisoned")
            .clone();
        let failed_subscribers = subscribers
            .iter()
            .filter(|subscriber| {
                subscriber.kind.is_none() || subscriber.kind == Some(erased.event_type)
            })
            .filter(|subscriber| (subscriber.handler)(&erased).is_err())
            .count();
        if failed_subscribers == 0 {
            Ok(())
        } else {
            Err(PublishError::Subscribers { failed_subscribers })
        }
    }
}
