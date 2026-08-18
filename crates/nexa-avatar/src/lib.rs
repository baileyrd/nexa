//! Renderer-neutral ports between NBP semantics and avatar runtime adapters.
#![forbid(unsafe_code)]

use nexa_domain::{BehaviorId, MessageId, SemanticKey};
use nexa_nbp::{
    BehaviorCancel, BehaviorCommand, ErrorSeverity, NbpMessage, Payload, RuntimeAck, RuntimeError,
    RuntimeState, RuntimeStatus,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

/// Semantic facilities which an avatar adapter can advertise.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AvatarCapability {
    BehaviorState,
    Emotion,
    Gaze,
    Gesture,
    Speech,
    Visemes,
    Cancellation,
}

/// A deterministic, renderer-independent capability declaration.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AvatarCapabilities {
    supported: BTreeSet<AvatarCapability>,
}

impl AvatarCapabilities {
    pub fn new(supported: impl IntoIterator<Item = AvatarCapability>) -> Self {
        Self {
            supported: supported.into_iter().collect(),
        }
    }

    pub fn supports(&self, capability: AvatarCapability) -> bool {
        self.supported.contains(&capability)
    }

    pub fn iter(&self) -> impl Iterator<Item = AvatarCapability> + '_ {
        self.supported.iter().copied()
    }
}

/// The only commands accepted by an avatar implementation boundary.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "request_type", content = "request", rename_all = "snake_case")]
pub enum AvatarRequest {
    Submit {
        message_id: MessageId,
        command: BehaviorCommand,
    },
    Cancel {
        message_id: MessageId,
        cancellation: BehaviorCancel,
    },
}

impl AvatarRequest {
    pub const fn message_id(&self) -> MessageId {
        match self {
            Self::Submit { message_id, .. } | Self::Cancel { message_id, .. } => *message_id,
        }
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum RequestConversionError {
    #[error("NBP payload {actual} cannot be submitted to the avatar command port")]
    OutputPayload { actual: &'static str },
}

impl TryFrom<&NbpMessage> for AvatarRequest {
    type Error = RequestConversionError;

    fn try_from(message: &NbpMessage) -> Result<Self, Self::Error> {
        match &message.payload {
            Payload::BehaviorCommand(command) => Ok(Self::Submit {
                message_id: message.message_id,
                command: command.clone(),
            }),
            Payload::BehaviorCancel(cancellation) => Ok(Self::Cancel {
                message_id: message.message_id,
                cancellation: cancellation.clone(),
            }),
            Payload::RuntimeAck(_) => Err(Self::Error::OutputPayload {
                actual: "runtime.ack",
            }),
            Payload::RuntimeState(_) => Err(Self::Error::OutputPayload {
                actual: "runtime.state",
            }),
            Payload::RuntimeError(_) => Err(Self::Error::OutputPayload {
                actual: "runtime.error",
            }),
        }
    }
}

/// Renderer-neutral response emitted synchronously by the initial port.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AvatarReport {
    pub acknowledgement: RuntimeAck,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<RuntimeState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RuntimeError>,
}

impl AvatarReport {
    pub fn accepted(
        message_id: MessageId,
        avatar_id: SemanticKey,
        command: &BehaviorCommand,
    ) -> Self {
        Self {
            acknowledgement: RuntimeAck {
                message_id,
                behavior_id: Some(command.behavior_id),
                status: RuntimeStatus::Accepted,
            },
            state: Some(RuntimeState {
                avatar_id,
                state: command.state,
                behavior_id: Some(command.behavior_id),
            }),
            error: None,
        }
    }

    pub fn cancelled(message_id: MessageId, behavior_id: BehaviorId) -> Self {
        Self {
            acknowledgement: RuntimeAck {
                message_id,
                behavior_id: Some(behavior_id),
                status: RuntimeStatus::Cancelled,
            },
            state: None,
            error: None,
        }
    }

    pub fn degraded(
        message_id: MessageId,
        behavior_id: BehaviorId,
        code: SemanticKey,
        message: String,
    ) -> Self {
        Self {
            acknowledgement: RuntimeAck {
                message_id,
                behavior_id: Some(behavior_id),
                status: RuntimeStatus::Degraded,
            },
            state: None,
            error: Some(RuntimeError {
                code,
                severity: ErrorSeverity::Warning,
                behavior_id: Some(behavior_id),
                message,
                recoverable: true,
            }),
        }
    }
}

/// Inbound command port owned here and implemented by renderer/runtime adapters.
pub trait AvatarPort {
    fn capabilities(&self) -> AvatarCapabilities;
    fn submit(&mut self, message_id: MessageId, command: BehaviorCommand) -> AvatarReport;
    fn cancel(&mut self, message_id: MessageId, cancellation: BehaviorCancel) -> AvatarReport;

    fn handle(&mut self, request: AvatarRequest) -> AvatarReport {
        match request {
            AvatarRequest::Submit {
                message_id,
                command,
            } => self.submit(message_id, command),
            AvatarRequest::Cancel {
                message_id,
                cancellation,
            } => self.cancel(message_id, cancellation),
        }
    }
}

/// Deterministic adapter for contract, orchestrator, and headless tests.
#[derive(Clone, Debug)]
pub struct FakeAvatarAdapter {
    avatar_id: SemanticKey,
    capabilities: AvatarCapabilities,
    requests: Vec<AvatarRequest>,
}

impl FakeAvatarAdapter {
    pub fn new(avatar_id: SemanticKey, capabilities: AvatarCapabilities) -> Self {
        Self {
            avatar_id,
            capabilities,
            requests: Vec::new(),
        }
    }

    pub fn requests(&self) -> &[AvatarRequest] {
        &self.requests
    }
}

impl AvatarPort for FakeAvatarAdapter {
    fn capabilities(&self) -> AvatarCapabilities {
        self.capabilities.clone()
    }

    fn submit(&mut self, message_id: MessageId, command: BehaviorCommand) -> AvatarReport {
        let missing = missing_required_capabilities(&command, &self.capabilities);
        let first_missing = missing.iter().next();
        self.requests.push(AvatarRequest::Submit {
            message_id,
            command: command.clone(),
        });
        if let Some(capability) = first_missing {
            AvatarReport::degraded(
                message_id,
                command.behavior_id,
                SemanticKey::new("avatar.capability.unsupported").expect("static key is valid"),
                format!("{capability:?} is unsupported; semantic command was degraded"),
            )
        } else {
            AvatarReport::accepted(message_id, self.avatar_id.clone(), &command)
        }
    }

    fn cancel(&mut self, message_id: MessageId, cancellation: BehaviorCancel) -> AvatarReport {
        let behavior_id = cancellation.behavior_id;
        self.requests.push(AvatarRequest::Cancel {
            message_id,
            cancellation,
        });
        if self.capabilities.supports(AvatarCapability::Cancellation) {
            AvatarReport::cancelled(message_id, behavior_id)
        } else {
            AvatarReport::degraded(
                message_id,
                behavior_id,
                SemanticKey::new("avatar.cancellation.unsupported").expect("static key is valid"),
                "cancellation is unsupported by this adapter".into(),
            )
        }
    }
}

/// Returns the semantic facilities required by a behavior command.
pub fn required_capabilities(command: &BehaviorCommand) -> AvatarCapabilities {
    let mut values = vec![AvatarCapability::BehaviorState];
    values.extend(
        command
            .emotion
            .is_some()
            .then_some(AvatarCapability::Emotion),
    );
    values.extend(command.gaze.is_some().then_some(AvatarCapability::Gaze));
    values.extend(
        command
            .gesture
            .is_some()
            .then_some(AvatarCapability::Gesture),
    );
    values.extend(command.speech.is_some().then_some(AvatarCapability::Speech));
    values.extend(
        command
            .speech
            .as_ref()
            .is_some_and(|speech| speech.emit_visemes)
            .then_some(AvatarCapability::Visemes),
    );
    AvatarCapabilities::new(values)
}

/// Returns every command requirement not advertised by an adapter.
///
/// Keeping this comparison beside [`required_capabilities`] ensures adapters do
/// not accidentally validate only optional portions of a behavior command.
pub fn missing_required_capabilities(
    command: &BehaviorCommand,
    available: &AvatarCapabilities,
) -> AvatarCapabilities {
    AvatarCapabilities::new(
        required_capabilities(command)
            .iter()
            .filter(|capability| !available.supports(*capability)),
    )
}
