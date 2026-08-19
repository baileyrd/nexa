//! Typed event envelopes and a minimal runtime-neutral local bus.
#![forbid(unsafe_code)]

use nexa_domain::{
    BehaviorId, CompetencyId, CorrelationId, EndpointId, EventId, EvidenceId, MasteryScore,
    MessageId, ProtocolVersion, SemanticKey, Sequence, SessionId, StudentId, SubjectId, Timestamp,
    TraceId,
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

/// Privacy-minimal pedagogy routing fact. Vocabulary remains semantic to avoid a crate cycle.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PedagogyDecisionMade {
    pub student_id: StudentId,
    pub competency_id: CompetencyId,
    pub selected_option: SemanticKey,
    pub rationale_codes: Vec<SemanticKey>,
    pub policy_version: ProtocolVersion,
}
impl DomainEvent for PedagogyDecisionMade {
    const KIND: EventKind = EventKind::PedagogyDecisionMade;
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
