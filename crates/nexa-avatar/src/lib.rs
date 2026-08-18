//! Renderer-neutral ports between NBP semantics and avatar runtime adapters.
#![forbid(unsafe_code)]

use nexa_domain::{BehaviorId, EndpointId, MessageId, SemanticKey, Sequence};
pub use nexa_nbp::AvatarCapability;
use nexa_nbp::{
    BehaviorCancel, BehaviorCommand, ErrorSeverity, NbpMessage, Payload, RuntimeAck,
    RuntimeCapabilities, RuntimeError, RuntimeState, RuntimeStatus,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

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
    pub fn as_nbp(&self, avatar_id: SemanticKey) -> RuntimeCapabilities {
        RuntimeCapabilities::new(avatar_id, self.iter())
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
            Payload::RuntimeCapabilities(_) => Err(Self::Error::OutputPayload {
                actual: "runtime.capabilities",
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

/// Renderer-neutral lifecycle result emitted synchronously by the core port.
/// `Accepted` means ownership was taken; it never implies completion.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AvatarReport {
    pub message_id: MessageId,
    pub behavior_id: BehaviorId,
    lifecycle: Vec<RuntimeStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<RuntimeState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RuntimeError>,
}

impl<'de> Deserialize<'de> for AvatarReport {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Wire {
            message_id: MessageId,
            behavior_id: BehaviorId,
            lifecycle: Vec<RuntimeStatus>,
            state: Option<RuntimeState>,
            error: Option<RuntimeError>,
        }
        let wire = Wire::deserialize(deserializer)?;
        let valid_shape = matches!(
            wire.lifecycle.as_slice(),
            [
                RuntimeStatus::Accepted,
                RuntimeStatus::Started,
                RuntimeStatus::Completed
            ] | [RuntimeStatus::Cancelled]
                | [RuntimeStatus::Degraded]
                | [RuntimeStatus::Rejected]
                | [
                    RuntimeStatus::Accepted,
                    RuntimeStatus::Started,
                    RuntimeStatus::Failed
                ]
        );
        let terminal = wire.lifecycle.last().copied();
        let error_required = matches!(
            terminal,
            Some(RuntimeStatus::Degraded | RuntimeStatus::Rejected | RuntimeStatus::Failed)
        );
        let state_valid = if terminal == Some(RuntimeStatus::Completed) {
            wire.state.is_some() && wire.error.is_none()
        } else {
            wire.state.is_none() && (wire.error.is_some() == error_required)
        };
        let identities_valid = wire.state.as_ref().is_none_or(|state| {
            state.message_id == wire.message_id && state.behavior_id == Some(wire.behavior_id)
        }) && wire.error.as_ref().is_none_or(|error| {
            error.message_id == wire.message_id && error.behavior_id == Some(wire.behavior_id)
        });
        if !(valid_shape && state_valid && identities_valid) {
            return Err(serde::de::Error::custom("invalid avatar report lifecycle"));
        }
        Ok(Self {
            message_id: wire.message_id,
            behavior_id: wire.behavior_id,
            lifecycle: wire.lifecycle,
            state: wire.state,
            error: wire.error,
        })
    }
}
impl AvatarReport {
    fn with_statuses(
        message_id: MessageId,
        behavior_id: BehaviorId,
        lifecycle: Vec<RuntimeStatus>,
    ) -> Self {
        Self {
            message_id,
            behavior_id,
            lifecycle,
            state: None,
            error: None,
        }
    }
    pub fn completed(
        message_id: MessageId,
        avatar_id: SemanticKey,
        command: &BehaviorCommand,
    ) -> Self {
        let mut value = Self::with_statuses(
            message_id,
            command.behavior_id,
            vec![
                RuntimeStatus::Accepted,
                RuntimeStatus::Started,
                RuntimeStatus::Completed,
            ],
        );
        value.state = Some(RuntimeState {
            message_id,
            avatar_id,
            state: command.state,
            behavior_id: Some(command.behavior_id),
        });
        value
    }
    pub fn cancelled(message_id: MessageId, behavior_id: BehaviorId) -> Self {
        Self::with_statuses(message_id, behavior_id, vec![RuntimeStatus::Cancelled])
    }
    pub fn degraded(
        message_id: MessageId,
        behavior_id: BehaviorId,
        code: SemanticKey,
        message: String,
    ) -> Self {
        let mut value = Self::with_statuses(message_id, behavior_id, vec![RuntimeStatus::Degraded]);
        value.error = Some(RuntimeError {
            message_id,
            code,
            severity: ErrorSeverity::Warning,
            behavior_id: Some(behavior_id),
            message,
            recoverable: true,
        });
        value
    }
    pub fn rejected(
        message_id: MessageId,
        behavior_id: BehaviorId,
        code: SemanticKey,
        message: String,
    ) -> Self {
        let mut value = Self::with_statuses(message_id, behavior_id, vec![RuntimeStatus::Rejected]);
        value.error = Some(RuntimeError {
            message_id,
            code,
            severity: ErrorSeverity::Error,
            behavior_id: Some(behavior_id),
            message,
            recoverable: false,
        });
        value
    }
    pub fn failed(
        message_id: MessageId,
        behavior_id: BehaviorId,
        code: SemanticKey,
        message: String,
    ) -> Self {
        let mut value = Self::with_statuses(
            message_id,
            behavior_id,
            vec![
                RuntimeStatus::Accepted,
                RuntimeStatus::Started,
                RuntimeStatus::Failed,
            ],
        );
        value.error = Some(RuntimeError {
            message_id,
            code,
            severity: ErrorSeverity::Error,
            behavior_id: Some(behavior_id),
            message,
            recoverable: false,
        });
        value
    }
    pub fn terminal_status(&self) -> RuntimeStatus {
        self.lifecycle[self.lifecycle.len() - 1]
    }
    pub fn lifecycle(&self) -> &[RuntimeStatus] {
        &self.lifecycle
    }
    pub fn state(&self) -> Option<&RuntimeState> {
        self.state.as_ref()
    }
    pub fn error(&self) -> Option<&RuntimeError> {
        self.error.as_ref()
    }

    /// Converts each semantic lifecycle fact to a governed NBP output. IDs and the first
    /// output sequence are supplied by the composition root; the core never invents identity.
    pub fn to_nbp_messages(
        &self,
        input: &NbpMessage,
        source: EndpointId,
        first_sequence: Sequence,
        message_ids: impl IntoIterator<Item = MessageId>,
    ) -> Result<Vec<NbpMessage>, OutputConversionError> {
        let mut ids = message_ids.into_iter();
        let mut payloads: Vec<Payload> = self
            .lifecycle
            .iter()
            .map(|status| {
                Payload::RuntimeAck(RuntimeAck {
                    message_id: self.message_id,
                    behavior_id: Some(self.behavior_id),
                    status: *status,
                })
            })
            .collect();
        if let Some(state) = &self.state {
            payloads.push(Payload::RuntimeState(state.clone()));
        }
        if let Some(error) = &self.error {
            payloads.push(Payload::RuntimeError(error.clone()));
        }
        payloads
            .into_iter()
            .enumerate()
            .map(|(index, payload)| {
                let message_id = ids
                    .next()
                    .ok_or(OutputConversionError::InsufficientMessageIds)?;
                NbpMessage::new(
                    input.nbp_version,
                    message_id,
                    input.timestamp,
                    input.session_id,
                    Sequence::new(
                        first_sequence
                            .get()
                            .checked_add(index as u64)
                            .ok_or(OutputConversionError::SequenceOverflow)?,
                    ),
                    source.clone(),
                    Some(input.source.clone()),
                    input.correlation_id,
                    payload,
                    Default::default(),
                )
                .map_err(OutputConversionError::Protocol)
            })
            .collect()
    }
}

#[derive(Debug, Error)]
pub enum OutputConversionError {
    #[error("not enough message identities supplied for avatar outputs")]
    InsufficientMessageIds,
    #[error("avatar output sequence exceeds u64::MAX")]
    SequenceOverflow,
    #[error(transparent)]
    Protocol(#[from] nexa_nbp::NbpError),
}

/// Inbound command port owned here and implemented by renderer/runtime adapters.
pub trait AvatarPort {
    fn capabilities(&self) -> AvatarCapabilities;
    fn submit(&mut self, message_id: MessageId, command: BehaviorCommand) -> AvatarReport;
    fn cancel(&mut self, message_id: MessageId, cancellation: BehaviorCancel) -> AvatarReport;

    /// Determines the synchronous outcome without performing the command. Implementations must
    /// return the same report from `handle`; composition roots use this to validate identities
    /// before allowing externally visible adapter mutation.
    fn preview(&self, request: &AvatarRequest) -> AvatarReport;

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
    submit_outcome: FakeSubmitOutcome,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FakeSubmitOutcome {
    #[default]
    Complete,
    Reject,
    Fail,
}

impl FakeAvatarAdapter {
    pub fn new(avatar_id: SemanticKey, capabilities: AvatarCapabilities) -> Self {
        Self {
            avatar_id,
            capabilities,
            requests: Vec::new(),
            submit_outcome: FakeSubmitOutcome::Complete,
        }
    }

    pub fn with_submit_outcome(mut self, outcome: FakeSubmitOutcome) -> Self {
        self.submit_outcome = outcome;
        self
    }

    pub fn requests(&self) -> &[AvatarRequest] {
        &self.requests
    }
}

impl AvatarPort for FakeAvatarAdapter {
    fn capabilities(&self) -> AvatarCapabilities {
        self.capabilities.clone()
    }

    fn preview(&self, request: &AvatarRequest) -> AvatarReport {
        let mut preview = self.clone();
        preview.handle(request.clone())
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
            match self.submit_outcome {
                FakeSubmitOutcome::Complete => {
                    AvatarReport::completed(message_id, self.avatar_id.clone(), &command)
                }
                FakeSubmitOutcome::Reject => AvatarReport::rejected(
                    message_id,
                    command.behavior_id,
                    SemanticKey::new("avatar.fake.rejected").expect("static key is valid"),
                    "deterministic fake rejection".into(),
                ),
                FakeSubmitOutcome::Fail => AvatarReport::failed(
                    message_id,
                    command.behavior_id,
                    SemanticKey::new("avatar.fake.failed").expect("static key is valid"),
                    "deterministic fake failure".into(),
                ),
            }
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
