//! Provider-neutral asynchronous Speech cancellation-control contracts.
#![forbid(unsafe_code)]

use nexa_domain::{ProtocolVersion, SpeechId};
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    collections::VecDeque,
    fmt,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

/// The only supported Speech cancellation-control contract version.
pub const SPEECH_CANCELLATION_V1: ProtocolVersion = ProtocolVersion::new(1, 0);

/// A content-free request associated with exactly one speech operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpeechCancellationRequest {
    pub contract_version: ProtocolVersion,
    pub speech_id: SpeechId,
}

impl SpeechCancellationRequest {
    pub const fn new(speech_id: SpeechId) -> Self {
        Self {
            contract_version: SPEECH_CANCELLATION_V1,
            speech_id,
        }
    }

    fn validate(&self) -> Result<(), SpeechCancellationError> {
        if self.contract_version == SPEECH_CANCELLATION_V1 {
            Ok(())
        } else {
            Err(SpeechCancellationError::UnsupportedVersion)
        }
    }
}

impl<'de> Deserialize<'de> for SpeechCancellationRequest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            contract_version: ProtocolVersion,
            speech_id: SpeechId,
        }
        let wire = Wire::deserialize(deserializer)?;
        let request = Self {
            contract_version: wire.contract_version,
            speech_id: wire.speech_id,
        };
        request.validate().map_err(serde::de::Error::custom)?;
        Ok(request)
    }
}

/// Immutable evidence that the dependency accepted one exact control request.
///
/// Acceptance does not prove that generation, queued audio, playback, visemes,
/// gestures, a provider, device, thread, process, or external request stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpeechCancellationAcknowledgement {
    pub contract_version: ProtocolVersion,
    pub speech_id: SpeechId,
}

impl SpeechCancellationAcknowledgement {
    pub const fn for_request(request: &SpeechCancellationRequest) -> Self {
        Self {
            contract_version: request.contract_version,
            speech_id: request.speech_id,
        }
    }
}

impl<'de> Deserialize<'de> for SpeechCancellationAcknowledgement {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            contract_version: ProtocolVersion,
            speech_id: SpeechId,
        }
        let wire = Wire::deserialize(deserializer)?;
        if wire.contract_version != SPEECH_CANCELLATION_V1 {
            return Err(serde::de::Error::custom(
                "unsupported speech cancellation version",
            ));
        }
        Ok(Self {
            contract_version: wire.contract_version,
            speech_id: wire.speech_id,
        })
    }
}

/// Closed result returned by a Speech cancellation dependency.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "acknowledgement", rename_all = "snake_case")]
pub enum SpeechCancellationServiceOutcome {
    Acknowledged(SpeechCancellationAcknowledgement),
    DependencyFailure,
}

impl fmt::Display for SpeechCancellationServiceOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Acknowledged(_) => f.write_str("speech cancellation acknowledged"),
            Self::DependencyFailure => f.write_str("speech cancellation dependency failed"),
        }
    }
}

/// Closed, content-free host-operation errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpeechCancellationError {
    UnsupportedVersion,
    AssociationMismatch,
    DependencyFailure,
    AcknowledgementMismatch,
}

impl fmt::Display for SpeechCancellationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnsupportedVersion => "unsupported speech cancellation version",
            Self::AssociationMismatch => "speech cancellation host association mismatch",
            Self::DependencyFailure => "speech cancellation dependency failed",
            Self::AcknowledgementMismatch => "speech cancellation acknowledgement mismatch",
        })
    }
}

impl std::error::Error for SpeechCancellationError {}

/// An erased asynchronous Speech cancellation-control operation.
pub type SpeechCancellationFuture<'a> =
    Pin<Box<dyn Future<Output = SpeechCancellationServiceOutcome> + Send + 'a>>;

/// Object-safe asynchronous provider-neutral cancellation control.
pub trait SpeechCancellationService: Send + Sync {
    fn request_cancellation(
        &self,
        request: SpeechCancellationRequest,
    ) -> SpeechCancellationFuture<'_>;
}

/// Preflight one exact association, await one service call, and validate its evidence.
pub async fn request_speech_cancellation(
    service: &dyn SpeechCancellationService,
    request: SpeechCancellationRequest,
    speech_id: SpeechId,
) -> Result<SpeechCancellationAcknowledgement, SpeechCancellationError> {
    request.validate()?;
    if request.speech_id != speech_id {
        return Err(SpeechCancellationError::AssociationMismatch);
    }
    match service.request_cancellation(request).await {
        SpeechCancellationServiceOutcome::Acknowledged(acknowledgement)
            if acknowledgement.contract_version == request.contract_version
                && acknowledgement.speech_id == speech_id =>
        {
            Ok(acknowledgement)
        }
        SpeechCancellationServiceOutcome::Acknowledged(_) => {
            Err(SpeechCancellationError::AcknowledgementMismatch)
        }
        SpeechCancellationServiceOutcome::DependencyFailure => {
            Err(SpeechCancellationError::DependencyFailure)
        }
    }
}

/// One deterministic FIFO dependency instruction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScriptedSpeechCancellationOutcome {
    Acknowledged(SpeechCancellationAcknowledgement),
    DependencyFailure,
    Pending,
}

impl fmt::Display for ScriptedSpeechCancellationOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Acknowledged(_) => "scripted speech cancellation acknowledgement",
            Self::DependencyFailure => "scripted speech cancellation dependency failure",
            Self::Pending => "scripted speech cancellation pending",
        })
    }
}

#[derive(Default)]
struct ScriptState {
    outcomes: VecDeque<ScriptedSpeechCancellationOutcome>,
    received: Vec<SpeechCancellationRequest>,
    consumed: usize,
    active: usize,
}

/// Deterministic FIFO service with bounded read-only accounting.
///
/// It creates no task or thread. Dropping its returned future removes all active work.
#[derive(Clone, Default)]
pub struct ScriptedSpeechCancellationService {
    state: Arc<Mutex<ScriptState>>,
}

impl ScriptedSpeechCancellationService {
    pub fn new(outcomes: impl IntoIterator<Item = ScriptedSpeechCancellationOutcome>) -> Self {
        Self {
            state: Arc::new(Mutex::new(ScriptState {
                outcomes: outcomes.into_iter().collect(),
                ..ScriptState::default()
            })),
        }
    }

    pub fn received_requests(&self) -> Vec<SpeechCancellationRequest> {
        self.state
            .lock()
            .expect("script state poisoned")
            .received
            .clone()
    }

    pub fn consumed_outcome_count(&self) -> usize {
        self.state.lock().expect("script state poisoned").consumed
    }

    pub fn remaining_outcome_count(&self) -> usize {
        self.state
            .lock()
            .expect("script state poisoned")
            .outcomes
            .len()
    }

    pub fn active_future_count(&self) -> usize {
        self.state.lock().expect("script state poisoned").active
    }
}

struct ActiveFuture(Arc<Mutex<ScriptState>>);

impl Drop for ActiveFuture {
    fn drop(&mut self) {
        self.0.lock().expect("script state poisoned").active -= 1;
    }
}

impl SpeechCancellationService for ScriptedSpeechCancellationService {
    fn request_cancellation(
        &self,
        request: SpeechCancellationRequest,
    ) -> SpeechCancellationFuture<'_> {
        let state = Arc::clone(&self.state);
        Box::pin(async move {
            let outcome = {
                let mut state = state.lock().expect("script state poisoned");
                state.received.push(request);
                state.active += 1;
                let outcome = state.outcomes.pop_front();
                if outcome.is_some() {
                    state.consumed += 1;
                }
                outcome
            };
            let _active = ActiveFuture(Arc::clone(&state));
            match outcome {
                Some(ScriptedSpeechCancellationOutcome::Acknowledged(value)) => {
                    SpeechCancellationServiceOutcome::Acknowledged(value)
                }
                Some(ScriptedSpeechCancellationOutcome::DependencyFailure) | None => {
                    SpeechCancellationServiceOutcome::DependencyFailure
                }
                Some(ScriptedSpeechCancellationOutcome::Pending) => std::future::pending().await,
            }
        })
    }
}
