//! Nexa Behavior Protocol (NBP) semantic messages, independent of renderers and transports.
#![forbid(unsafe_code)]

use nexa_domain::{
    BehaviorId, Confidence, CorrelationId, DurationMs, EndpointId, MessageId, ProtocolVersion,
    SemanticKey, Sequence, SessionId, Timestamp,
};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum NbpError {
    #[error("unsupported NBP major version {major}")]
    UnsupportedVersion { major: u16 },
    #[error("message_type {declared:?} does not match payload type {actual:?}")]
    MessageTypeMismatch {
        declared: MessageType,
        actual: MessageType,
    },
    #[error("priority must be in the inclusive range 0..=100")]
    InvalidPriority,
    #[error("extension key must contain a namespace and local name separated by '.'")]
    InvalidExtensionKey,
    #[error("extension values must be JSON objects")]
    InvalidExtensionValue,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MessageType {
    #[serde(rename = "behavior.command")]
    BehaviorCommand,
    #[serde(rename = "behavior.cancel")]
    BehaviorCancel,
    #[serde(rename = "runtime.ack")]
    RuntimeAck,
    #[serde(rename = "runtime.state")]
    RuntimeState,
    #[serde(rename = "runtime.error")]
    RuntimeError,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BehaviorState {
    Idle,
    Attentive,
    Listening,
    Thinking,
    Speaking,
    Explaining,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Interruptibility {
    Immediate,
    WordBoundary,
    PhraseBoundary,
    SentenceBoundary,
    NonInterruptible,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmotionPreset {
    Neutral,
    Focused,
    Encouraging,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GazeTarget {
    Student,
    Camera,
    CanvasObject,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GestureKind {
    Idle,
    Nod,
    Point,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpeechStyle {
    Neutral,
    Instructional,
    Encouraging,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CancellationMode {
    Immediate,
    Graceful,
    PhraseBoundary,
    SentenceBoundary,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeStatus {
    Accepted,
    Queued,
    Started,
    Completed,
    Cancelled,
    Rejected,
    Degraded,
    Failed,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorSeverity {
    Warning,
    Error,
    Fatal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Priority(u8);
impl Priority {
    pub fn new(value: u8) -> Result<Self, NbpError> {
        if value <= 100 {
            Ok(Self(value))
        } else {
            Err(NbpError::InvalidPriority)
        }
    }
    pub const fn get(self) -> u8 {
        self.0
    }
}
impl Default for Priority {
    fn default() -> Self {
        Self(50)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Emotion {
    pub preset: EmotionPreset,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intensity: Option<Confidence>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Gaze {
    pub target_type: GazeTarget,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<SemanticKey>,
    pub intensity: Confidence,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<DurationMs>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_time_ms: Option<DurationMs>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Gesture {
    #[serde(rename = "type")]
    pub kind: GestureKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<SemanticKey>,
    pub intensity: Confidence,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<DurationMs>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Speech {
    pub text: String,
    pub style: SpeechStyle,
    pub allow_interruption: bool,
    pub emit_visemes: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BehaviorCommand {
    pub behavior_id: BehaviorId,
    pub state: BehaviorState,
    #[serde(default)]
    pub priority: Priority,
    pub interruptibility: Interruptibility,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emotion: Option<Emotion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gaze: Option<Gaze>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gesture: Option<Gesture>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speech: Option<Speech>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BehaviorCancel {
    pub behavior_id: BehaviorId,
    pub reason: String,
    pub transition: CancellationMode,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RuntimeAck {
    pub message_id: MessageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behavior_id: Option<BehaviorId>,
    pub status: RuntimeStatus,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RuntimeState {
    pub avatar_id: SemanticKey,
    pub state: BehaviorState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behavior_id: Option<BehaviorId>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RuntimeError {
    pub code: SemanticKey,
    pub severity: ErrorSeverity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behavior_id: Option<BehaviorId>,
    pub message: String,
    pub recoverable: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Payload {
    BehaviorCommand(BehaviorCommand),
    BehaviorCancel(BehaviorCancel),
    RuntimeAck(RuntimeAck),
    RuntimeState(RuntimeState),
    RuntimeError(RuntimeError),
}
impl Payload {
    pub const fn message_type(&self) -> MessageType {
        match self {
            Self::BehaviorCommand(_) => MessageType::BehaviorCommand,
            Self::BehaviorCancel(_) => MessageType::BehaviorCancel,
            Self::RuntimeAck(_) => MessageType::RuntimeAck,
            Self::RuntimeState(_) => MessageType::RuntimeState,
            Self::RuntimeError(_) => MessageType::RuntimeError,
        }
    }
}

/// Namespaced optional renderer/vendor hints; core semantics must never depend on them.
pub type Extensions = BTreeMap<String, Map<String, Value>>;

#[derive(Clone, Debug, PartialEq)]
pub struct NbpMessage {
    pub nbp_version: ProtocolVersion,
    pub message_id: MessageId,
    pub timestamp: Timestamp,
    pub session_id: SessionId,
    pub sequence: Sequence,
    pub source: EndpointId,
    pub target: Option<EndpointId>,
    pub correlation_id: Option<CorrelationId>,
    pub payload: Payload,
    pub extensions: Extensions,
}
impl NbpMessage {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        nbp_version: ProtocolVersion,
        message_id: MessageId,
        timestamp: Timestamp,
        session_id: SessionId,
        sequence: Sequence,
        source: EndpointId,
        target: Option<EndpointId>,
        correlation_id: Option<CorrelationId>,
        payload: Payload,
        extensions: Extensions,
    ) -> Result<Self, NbpError> {
        if nbp_version.major() != 1 {
            return Err(NbpError::UnsupportedVersion {
                major: nbp_version.major(),
            });
        }
        validate_extensions(&extensions)?;
        Ok(Self {
            nbp_version,
            message_id,
            timestamp,
            session_id,
            sequence,
            source,
            target,
            correlation_id,
            payload,
            extensions,
        })
    }
    pub const fn message_type(&self) -> MessageType {
        self.payload.message_type()
    }
}
fn validate_extensions(extensions: &Extensions) -> Result<(), NbpError> {
    for (key, value) in extensions {
        let valid = key
            .split_once('.')
            .is_some_and(|(a, b)| !a.is_empty() && !b.is_empty());
        if !valid {
            return Err(NbpError::InvalidExtensionKey);
        }
        if value.is_empty() { /* empty objects are valid */ }
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct WireMessage {
    nbp_version: ProtocolVersion,
    message_id: MessageId,
    message_type: MessageType,
    timestamp: Timestamp,
    session_id: SessionId,
    sequence: Sequence,
    source: EndpointId,
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<EndpointId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    correlation_id: Option<CorrelationId>,
    payload: Value,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    extensions: Extensions,
}
impl Serialize for NbpMessage {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        WireMessage {
            nbp_version: self.nbp_version,
            message_id: self.message_id,
            message_type: self.message_type(),
            timestamp: self.timestamp,
            session_id: self.session_id,
            sequence: self.sequence,
            source: self.source.clone(),
            target: self.target.clone(),
            correlation_id: self.correlation_id,
            payload: match &self.payload {
                Payload::BehaviorCommand(v) => serde_json::to_value(v),
                Payload::BehaviorCancel(v) => serde_json::to_value(v),
                Payload::RuntimeAck(v) => serde_json::to_value(v),
                Payload::RuntimeState(v) => serde_json::to_value(v),
                Payload::RuntimeError(v) => serde_json::to_value(v),
            }
            .map_err(serde::ser::Error::custom)?,
            extensions: self.extensions.clone(),
        }
        .serialize(s)
    }
}
impl<'de> Deserialize<'de> for NbpMessage {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let w = WireMessage::deserialize(d)?;
        let payload = match w.message_type {
            MessageType::BehaviorCommand => {
                serde_json::from_value(w.payload).map(Payload::BehaviorCommand)
            }
            MessageType::BehaviorCancel => {
                serde_json::from_value(w.payload).map(Payload::BehaviorCancel)
            }
            MessageType::RuntimeAck => serde_json::from_value(w.payload).map(Payload::RuntimeAck),
            MessageType::RuntimeState => {
                serde_json::from_value(w.payload).map(Payload::RuntimeState)
            }
            MessageType::RuntimeError => {
                serde_json::from_value(w.payload).map(Payload::RuntimeError)
            }
        }
        .map_err(de::Error::custom)?;
        Self::new(
            w.nbp_version,
            w.message_id,
            w.timestamp,
            w.session_id,
            w.sequence,
            w.source,
            w.target,
            w.correlation_id,
            payload,
            w.extensions,
        )
        .map_err(de::Error::custom)
    }
}
