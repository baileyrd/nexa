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

/// Closed, canonical order of the four speech-owned cancellation surfaces.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SpeechCancellationSurface {
    Synthesis,
    QueuedAudio,
    Playback,
    VisemeTimeline,
}

impl SpeechCancellationSurface {
    pub const ALL: [Self; 4] = [
        Self::Synthesis,
        Self::QueuedAudio,
        Self::Playback,
        Self::VisemeTimeline,
    ];
}

/// Side-effect-free, content-free declaration made before cancellation begins.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpeechCancellationCapability {
    pub contract_version: ProtocolVersion,
    pub speech_id: SpeechId,
    pub surface: SpeechCancellationSurface,
    pub cancellable: bool,
}

impl SpeechCancellationCapability {
    pub const fn cancellable(speech_id: SpeechId, surface: SpeechCancellationSurface) -> Self {
        Self {
            contract_version: SPEECH_CANCELLATION_V1,
            speech_id,
            surface,
            cancellable: true,
        }
    }

    fn validate(&self) -> Result<(), SpeechCancellationCoordinatorError> {
        if self.contract_version != SPEECH_CANCELLATION_V1 {
            Err(SpeechCancellationCoordinatorError::UnsupportedVersion)
        } else if !self.cancellable {
            Err(SpeechCancellationCoordinatorError::NonCancellableSurface)
        } else {
            Ok(())
        }
    }
}

impl<'de> Deserialize<'de> for SpeechCancellationCapability {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            contract_version: ProtocolVersion,
            speech_id: SpeechId,
            surface: SpeechCancellationSurface,
            cancellable: bool,
        }
        let wire = Wire::deserialize(deserializer)?;
        let value = Self {
            contract_version: wire.contract_version,
            speech_id: wire.speech_id,
            surface: wire.surface,
            cancellable: wire.cancellable,
        };
        // Capability evidence describes what a participant supports. A
        // non-cancellable declaration is valid evidence, even though the
        // coordinator cannot use it for a required surface.
        if value.contract_version != SPEECH_CANCELLATION_V1 {
            return Err(serde::de::Error::custom(
                "unsupported speech cancellation capability version",
            ));
        }
        Ok(value)
    }
}

/// One exact participant. Capability inspection must be pure and non-activating.
pub trait SpeechCancellationParticipant: SpeechCancellationService {
    fn cancellation_capability(&self) -> SpeechCancellationCapability;
}

/// Immutable evidence for one canonical speech-owned surface.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpeechSurfaceCancellationEvidence {
    pub contract_version: ProtocolVersion,
    pub speech_id: SpeechId,
    pub surface: SpeechCancellationSurface,
    pub acknowledgement: SpeechCancellationAcknowledgement,
}

impl<'de> Deserialize<'de> for SpeechSurfaceCancellationEvidence {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            contract_version: ProtocolVersion,
            speech_id: SpeechId,
            surface: SpeechCancellationSurface,
            acknowledgement: SpeechCancellationAcknowledgement,
        }
        let wire = Wire::deserialize(deserializer)?;
        if wire.contract_version != SPEECH_CANCELLATION_V1
            || wire.acknowledgement.contract_version != wire.contract_version
            || wire.acknowledgement.speech_id != wire.speech_id
        {
            return Err(serde::de::Error::custom(
                "invalid speech surface cancellation evidence",
            ));
        }
        Ok(Self {
            contract_version: wire.contract_version,
            speech_id: wire.speech_id,
            surface: wire.surface,
            acknowledgement: wire.acknowledgement,
        })
    }
}

/// Closed aggregate state. `Stopped` is deliberately scoped to coordinator work.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpeechCancellationAggregateKind {
    Stopped,
}

/// Canonically ordered evidence for the complete four-surface coordination.
///
/// `Stopped` proves only that the four speech-owned control dependencies accepted
/// cancellation and their coordinator futures terminalized. It does not prove a
/// provider, device, process, or external request stopped, and it does not cover
/// speech-dependent gestures (which remain Behavior-owned).
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpeechCancellationAggregateEvidence {
    pub contract_version: ProtocolVersion,
    pub speech_id: SpeechId,
    pub kind: SpeechCancellationAggregateKind,
    pub surfaces: Vec<SpeechSurfaceCancellationEvidence>,
}

impl<'de> Deserialize<'de> for SpeechCancellationAggregateEvidence {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            contract_version: ProtocolVersion,
            speech_id: SpeechId,
            kind: SpeechCancellationAggregateKind,
            surfaces: Vec<SpeechSurfaceCancellationEvidence>,
        }
        let wire = Wire::deserialize(deserializer)?;
        if wire.contract_version != SPEECH_CANCELLATION_V1
            || wire.surfaces.len() != SpeechCancellationSurface::ALL.len()
            || wire
                .surfaces
                .iter()
                .map(|e| e.surface)
                .ne(SpeechCancellationSurface::ALL)
            || wire.surfaces.iter().any(|e| {
                e.contract_version != wire.contract_version || e.speech_id != wire.speech_id
            })
        {
            return Err(serde::de::Error::custom(
                "invalid speech cancellation aggregate evidence",
            ));
        }
        Ok(Self {
            contract_version: wire.contract_version,
            speech_id: wire.speech_id,
            kind: wire.kind,
            surfaces: wire.surfaces,
        })
    }
}

/// Closed, content-free coordinator errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpeechCancellationCoordinatorError {
    UnsupportedVersion,
    InvalidCapabilitySet,
    MissingSurface,
    DuplicateSurface,
    NonCancellableSurface,
    AssociationMismatch,
    DependencyFailure,
    AcknowledgementMismatch,
    AggregateFailure,
}

impl fmt::Display for SpeechCancellationCoordinatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnsupportedVersion => "unsupported speech cancellation coordinator version",
            Self::InvalidCapabilitySet => "invalid speech cancellation capability set",
            Self::MissingSurface => "missing speech cancellation surface",
            Self::DuplicateSurface => "duplicate speech cancellation surface",
            Self::NonCancellableSurface => "non-cancellable speech cancellation surface",
            Self::AssociationMismatch => "speech cancellation coordinator association mismatch",
            Self::DependencyFailure => "speech cancellation coordinator dependency failure",
            Self::AcknowledgementMismatch => {
                "speech cancellation coordinator acknowledgement mismatch"
            }
            Self::AggregateFailure => "speech cancellation coordinator aggregate failure",
        })
    }
}

impl std::error::Error for SpeechCancellationCoordinatorError {}

/// A fully preflighted, provider-neutral composite coordinator.
pub struct SpeechCancellationCoordinator<'a> {
    speech_id: SpeechId,
    participants: Vec<(
        &'a dyn SpeechCancellationParticipant,
        SpeechCancellationCapability,
    )>,
}

/// An owned, fully preflighted coordinator for composition roots that must move
/// the exact participant association into a workflow task.
///
/// Construction performs the same capability discovery and canonicalization as
/// [`SpeechCancellationCoordinator::new`]. Subsequent cancellation never
/// rediscovers capabilities.
pub struct OwnedSpeechCancellationCoordinator {
    speech_id: SpeechId,
    participants: Vec<(
        Arc<dyn SpeechCancellationParticipant>,
        SpeechCancellationCapability,
    )>,
}

impl OwnedSpeechCancellationCoordinator {
    pub fn new(
        speech_id: SpeechId,
        participants: impl IntoIterator<Item = Arc<dyn SpeechCancellationParticipant>>,
    ) -> Result<Self, SpeechCancellationCoordinatorError> {
        let mut values = participants
            .into_iter()
            .map(|participant| {
                let capability = participant.cancellation_capability();
                (participant, capability)
            })
            .collect::<Vec<_>>();
        validate_and_canonicalize(speech_id, &mut values)?;
        Ok(Self {
            speech_id,
            participants: values,
        })
    }

    /// Invokes the stable, canonically preflighted participant set exactly once.
    pub async fn cancel(
        &self,
        request: SpeechCancellationRequest,
    ) -> Result<SpeechCancellationAggregateEvidence, SpeechCancellationCoordinatorError> {
        cancel_preflighted(self.speech_id, &self.participants, request).await
    }
}

fn validate_and_canonicalize<P>(
    speech_id: SpeechId,
    values: &mut [(P, SpeechCancellationCapability)],
) -> Result<(), SpeechCancellationCoordinatorError> {
    for (_, capability) in values.iter() {
        capability.validate()?;
        if capability.speech_id != speech_id {
            return Err(SpeechCancellationCoordinatorError::AssociationMismatch);
        }
    }
    values.sort_by_key(|(_, capability)| capability.surface);
    for pair in values.windows(2) {
        if pair[0].1.surface == pair[1].1.surface {
            return Err(SpeechCancellationCoordinatorError::DuplicateSurface);
        }
    }
    if values.len() != SpeechCancellationSurface::ALL.len() {
        return Err(SpeechCancellationCoordinatorError::MissingSurface);
    }
    if values
        .iter()
        .map(|value| value.1.surface)
        .ne(SpeechCancellationSurface::ALL)
    {
        return Err(SpeechCancellationCoordinatorError::InvalidCapabilitySet);
    }
    Ok(())
}

impl<'a> SpeechCancellationCoordinator<'a> {
    /// Inspects every capability and establishes canonical order without mutation.
    pub fn new(
        speech_id: SpeechId,
        participants: impl IntoIterator<Item = &'a dyn SpeechCancellationParticipant>,
    ) -> Result<Self, SpeechCancellationCoordinatorError> {
        let mut values = participants
            .into_iter()
            .map(|p| (p, p.cancellation_capability()))
            .collect::<Vec<_>>();
        validate_and_canonicalize(speech_id, &mut values)?;
        Ok(Self {
            speech_id,
            participants: values,
        })
    }

    /// Invokes every exact participant once and owns all futures until terminal/drop.
    pub async fn cancel(
        &self,
        request: SpeechCancellationRequest,
    ) -> Result<SpeechCancellationAggregateEvidence, SpeechCancellationCoordinatorError> {
        cancel_preflighted(self.speech_id, &self.participants, request).await
    }
}

async fn cancel_preflighted<P>(
    speech_id: SpeechId,
    participants: &[(P, SpeechCancellationCapability)],
    request: SpeechCancellationRequest,
) -> Result<SpeechCancellationAggregateEvidence, SpeechCancellationCoordinatorError>
where
    P: ParticipantHandle,
{
    request
        .validate()
        .map_err(|_| SpeechCancellationCoordinatorError::UnsupportedVersion)?;
    if request.speech_id != speech_id {
        return Err(SpeechCancellationCoordinatorError::AssociationMismatch);
    }
    let mut futures = participants
        .iter()
        .map(|(participant, _)| Some(participant.participant().request_cancellation(request)))
        .collect::<Vec<_>>();
    let mut outcomes = (0..futures.len()).map(|_| None).collect::<Vec<_>>();
    std::future::poll_fn(|cx| {
        let mut pending = false;
        for (index, future) in futures.iter_mut().enumerate() {
            if let Some(value) = future.as_mut() {
                match value.as_mut().poll(cx) {
                    std::task::Poll::Ready(outcome) => {
                        outcomes[index] = Some(outcome);
                        *future = None;
                    }
                    std::task::Poll::Pending => pending = true,
                }
            }
        }
        if outcomes
            .iter()
            .flatten()
            .any(|o| matches!(o, SpeechCancellationServiceOutcome::DependencyFailure))
        {
            // Dropping all remaining owned futures is the safe terminal path.
            futures.clear();
            std::task::Poll::Ready(())
        } else if pending {
            std::task::Poll::Pending
        } else {
            std::task::Poll::Ready(())
        }
    })
    .await;
    if outcomes
        .iter()
        .flatten()
        .any(|o| matches!(o, SpeechCancellationServiceOutcome::DependencyFailure))
    {
        return Err(SpeechCancellationCoordinatorError::DependencyFailure);
    }
    if outcomes.iter().any(Option::is_none) {
        return Err(SpeechCancellationCoordinatorError::AggregateFailure);
    }
    let mut surfaces = Vec::with_capacity(4);
    for ((_, capability), outcome) in participants.iter().zip(outcomes) {
        let SpeechCancellationServiceOutcome::Acknowledged(acknowledgement) =
            outcome.expect("complete outcome")
        else {
            unreachable!()
        };
        if acknowledgement.contract_version != capability.contract_version
            || acknowledgement.speech_id != speech_id
        {
            return Err(SpeechCancellationCoordinatorError::AcknowledgementMismatch);
        }
        surfaces.push(SpeechSurfaceCancellationEvidence {
            contract_version: capability.contract_version,
            speech_id,
            surface: capability.surface,
            acknowledgement,
        });
    }
    Ok(SpeechCancellationAggregateEvidence {
        contract_version: SPEECH_CANCELLATION_V1,
        speech_id,
        kind: SpeechCancellationAggregateKind::Stopped,
        surfaces,
    })
}

trait ParticipantHandle {
    fn participant(&self) -> &dyn SpeechCancellationParticipant;
}

impl ParticipantHandle for &dyn SpeechCancellationParticipant {
    fn participant(&self) -> &dyn SpeechCancellationParticipant {
        *self
    }
}

impl ParticipantHandle for Arc<dyn SpeechCancellationParticipant> {
    fn participant(&self) -> &dyn SpeechCancellationParticipant {
        self.as_ref()
    }
}

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
            contract_version: SPEECH_CANCELLATION_V1,
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

impl fmt::Debug for ScriptedSpeechCancellationService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScriptedSpeechCancellationService")
            .field("received_request_count", &self.received_requests().len())
            .field("consumed_outcome_count", &self.consumed_outcome_count())
            .field("remaining_outcome_count", &self.remaining_outcome_count())
            .field("active_future_count", &self.active_future_count())
            .finish()
    }
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

/// Deterministic per-surface participant sharing the scripted service accounting.
#[derive(Clone)]
pub struct ScriptedSpeechCancellationParticipant {
    capability: SpeechCancellationCapability,
    service: ScriptedSpeechCancellationService,
}

impl fmt::Debug for ScriptedSpeechCancellationParticipant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScriptedSpeechCancellationParticipant")
            .field("capability", &self.capability)
            .field("service", &self.service)
            .finish()
    }
}

impl ScriptedSpeechCancellationParticipant {
    pub fn new(
        capability: SpeechCancellationCapability,
        outcomes: impl IntoIterator<Item = ScriptedSpeechCancellationOutcome>,
    ) -> Self {
        Self {
            capability,
            service: ScriptedSpeechCancellationService::new(outcomes),
        }
    }

    pub const fn capability(&self) -> SpeechCancellationCapability {
        self.capability
    }
    pub fn received_requests(&self) -> Vec<SpeechCancellationRequest> {
        self.service.received_requests()
    }
    pub fn consumed_outcome_count(&self) -> usize {
        self.service.consumed_outcome_count()
    }
    pub fn remaining_outcome_count(&self) -> usize {
        self.service.remaining_outcome_count()
    }
    pub fn active_future_count(&self) -> usize {
        self.service.active_future_count()
    }
}

impl SpeechCancellationService for ScriptedSpeechCancellationParticipant {
    fn request_cancellation(
        &self,
        request: SpeechCancellationRequest,
    ) -> SpeechCancellationFuture<'_> {
        self.service.request_cancellation(request)
    }
}

impl SpeechCancellationParticipant for ScriptedSpeechCancellationParticipant {
    fn cancellation_capability(&self) -> SpeechCancellationCapability {
        self.capability
    }
}

// ---------------------------------------------------------------------------
// Speech input service foundation (ADR-0067)

use nexa_domain::SpeechInputOperationId;
use std::task::{Context, Poll, Waker};

pub const SPEECH_INPUT_V1: ProtocolVersion = ProtocolVersion::new(1, 0);
pub const MAX_SPEECH_INPUT_TEXT_BYTES: usize = 16 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpeechInputRequest {
    pub contract_version: ProtocolVersion,
    pub speech_id: SpeechId,
    pub operation_id: SpeechInputOperationId,
}

impl SpeechInputRequest {
    pub const fn new(speech_id: SpeechId, operation_id: SpeechInputOperationId) -> Self {
        Self {
            contract_version: SPEECH_INPUT_V1,
            speech_id,
            operation_id,
        }
    }
    fn validate(&self) -> Result<(), SpeechInputError> {
        if self.contract_version == SPEECH_INPUT_V1 {
            Ok(())
        } else {
            Err(SpeechInputError::UnsupportedVersion)
        }
    }
}

impl<'de> Deserialize<'de> for SpeechInputRequest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            contract_version: ProtocolVersion,
            speech_id: SpeechId,
            operation_id: SpeechInputOperationId,
        }
        let wire = Wire::deserialize(deserializer)?;
        let value = Self {
            contract_version: wire.contract_version,
            speech_id: wire.speech_id,
            operation_id: wire.operation_id,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

/// Successful, bounded input evidence. Transcript content is intentionally
/// omitted from `Debug` and `Display`; callers may access it explicitly.
#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpeechInputEvidence {
    pub contract_version: ProtocolVersion,
    pub speech_id: SpeechId,
    pub operation_id: SpeechInputOperationId,
    transcript: String,
}

impl SpeechInputEvidence {
    pub fn new(
        request: &SpeechInputRequest,
        transcript: impl Into<String>,
    ) -> Result<Self, SpeechInputError> {
        request.validate()?;
        let transcript = transcript.into();
        if transcript.is_empty() || transcript.len() > MAX_SPEECH_INPUT_TEXT_BYTES {
            return Err(SpeechInputError::InvalidEvidence);
        }
        Ok(Self {
            contract_version: SPEECH_INPUT_V1,
            speech_id: request.speech_id,
            operation_id: request.operation_id,
            transcript,
        })
    }
    pub fn transcript(&self) -> &str {
        &self.transcript
    }
    fn validate(&self) -> Result<(), SpeechInputError> {
        if self.contract_version != SPEECH_INPUT_V1 {
            Err(SpeechInputError::UnsupportedVersion)
        } else if self.transcript.is_empty() || self.transcript.len() > MAX_SPEECH_INPUT_TEXT_BYTES
        {
            Err(SpeechInputError::InvalidEvidence)
        } else {
            Ok(())
        }
    }
}

impl fmt::Debug for SpeechInputEvidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpeechInputEvidence")
            .field("contract_version", &self.contract_version)
            .field("speech_id", &self.speech_id)
            .field("operation_id", &self.operation_id)
            .field("transcript", &"[REDACTED]")
            .finish()
    }
}
impl fmt::Display for SpeechInputEvidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bounded speech input evidence")
    }
}

impl<'de> Deserialize<'de> for SpeechInputEvidence {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            contract_version: ProtocolVersion,
            speech_id: SpeechId,
            operation_id: SpeechInputOperationId,
            transcript: String,
        }
        let wire = Wire::deserialize(deserializer)?;
        let value = Self {
            contract_version: wire.contract_version,
            speech_id: wire.speech_id,
            operation_id: wire.operation_id,
            transcript: wire.transcript,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpeechInputCancellationEvidence {
    pub contract_version: ProtocolVersion,
    pub speech_id: SpeechId,
    pub operation_id: SpeechInputOperationId,
}
impl SpeechInputCancellationEvidence {
    pub const fn for_request(r: &SpeechInputRequest) -> Self {
        Self {
            contract_version: SPEECH_INPUT_V1,
            speech_id: r.speech_id,
            operation_id: r.operation_id,
        }
    }
}
impl<'de> Deserialize<'de> for SpeechInputCancellationEvidence {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct W {
            contract_version: ProtocolVersion,
            speech_id: SpeechId,
            operation_id: SpeechInputOperationId,
        }
        let w = W::deserialize(d)?;
        if w.contract_version != SPEECH_INPUT_V1 {
            return Err(serde::de::Error::custom("unsupported speech input version"));
        }
        Ok(Self {
            contract_version: w.contract_version,
            speech_id: w.speech_id,
            operation_id: w.operation_id,
        })
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SpeechInputFailure {
    Unavailable,
    DependencyFailure,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    tag = "kind",
    content = "evidence",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum SpeechInputOutcome {
    Success(SpeechInputEvidence),
    Cancelled(SpeechInputCancellationEvidence),
    Failure(SpeechInputFailure),
}
impl fmt::Display for SpeechInputOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Success(_) => "speech input succeeded",
            Self::Cancelled(_) => "speech input cancelled",
            Self::Failure(_) => "speech input failed",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpeechInputError {
    UnsupportedVersion,
    AssociationMismatch,
    InvalidEvidence,
}
impl fmt::Display for SpeechInputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnsupportedVersion => "unsupported speech input version",
            Self::AssociationMismatch => "speech input association mismatch",
            Self::InvalidEvidence => "invalid speech input evidence",
        })
    }
}
impl std::error::Error for SpeechInputError {}

pub type SpeechInputFuture<'a> = Pin<Box<dyn Future<Output = SpeechInputOutcome> + Send + 'a>>;
pub type SpeechInputCancellationFuture<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
pub trait SpeechInputCancellationSignal: Send + Sync {
    fn is_cancelled(&self) -> bool;
    fn cancelled(&self) -> SpeechInputCancellationFuture<'_>;
}
pub trait SpeechInputService: Send + Sync {
    fn input(
        &self,
        request: SpeechInputRequest,
        cancellation: Arc<dyn SpeechInputCancellationSignal>,
    ) -> SpeechInputFuture<'_>;
}

pub async fn request_speech_input(
    service: &dyn SpeechInputService,
    request: SpeechInputRequest,
    speech_id: SpeechId,
    operation_id: SpeechInputOperationId,
    cancellation: Arc<dyn SpeechInputCancellationSignal>,
) -> Result<SpeechInputOutcome, SpeechInputError> {
    request.validate()?;
    if request.speech_id != speech_id || request.operation_id != operation_id {
        return Err(SpeechInputError::AssociationMismatch);
    }
    let outcome = service.input(request, cancellation).await;
    match &outcome {
        SpeechInputOutcome::Success(e) => {
            e.validate()?;
            if e.speech_id != speech_id || e.operation_id != operation_id {
                return Err(SpeechInputError::AssociationMismatch);
            }
        }
        SpeechInputOutcome::Cancelled(e) => {
            if e.contract_version != SPEECH_INPUT_V1 {
                return Err(SpeechInputError::UnsupportedVersion);
            }
            if e.speech_id != speech_id || e.operation_id != operation_id {
                return Err(SpeechInputError::AssociationMismatch);
            }
        }
        SpeechInputOutcome::Failure(_) => {}
    }
    Ok(outcome)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScriptedSpeechInputOutcome {
    Success(SpeechInputEvidence),
    Unavailable,
    DependencyFailure,
    WaitForCancellation,
}
#[derive(Default)]
struct SpeechInputScriptState {
    outcomes: VecDeque<ScriptedSpeechInputOutcome>,
    received: Vec<SpeechInputRequest>,
    consumed: usize,
    active: usize,
}
#[derive(Clone, Default)]
pub struct ScriptedSpeechInputService {
    state: Arc<Mutex<SpeechInputScriptState>>,
}
impl ScriptedSpeechInputService {
    pub fn new(outcomes: impl IntoIterator<Item = ScriptedSpeechInputOutcome>) -> Self {
        Self {
            state: Arc::new(Mutex::new(SpeechInputScriptState {
                outcomes: outcomes.into_iter().collect(),
                ..Default::default()
            })),
        }
    }
    pub fn received_requests(&self) -> Vec<SpeechInputRequest> {
        self.state
            .lock()
            .expect("speech input script poisoned")
            .received
            .clone()
    }
    pub fn consumed_outcome_count(&self) -> usize {
        self.state
            .lock()
            .expect("speech input script poisoned")
            .consumed
    }
    pub fn remaining_outcome_count(&self) -> usize {
        self.state
            .lock()
            .expect("speech input script poisoned")
            .outcomes
            .len()
    }
    pub fn active_future_count(&self) -> usize {
        self.state
            .lock()
            .expect("speech input script poisoned")
            .active
    }
}
struct ActiveSpeechInput(Arc<Mutex<SpeechInputScriptState>>);
impl Drop for ActiveSpeechInput {
    fn drop(&mut self) {
        self.0.lock().expect("speech input script poisoned").active -= 1;
    }
}
impl SpeechInputService for ScriptedSpeechInputService {
    fn input(
        &self,
        request: SpeechInputRequest,
        cancellation: Arc<dyn SpeechInputCancellationSignal>,
    ) -> SpeechInputFuture<'_> {
        let state = Arc::clone(&self.state);
        Box::pin(async move {
            {
                let mut s = state.lock().expect("speech input script poisoned");
                s.received.push(request);
                s.active += 1;
            }
            let _active = ActiveSpeechInput(Arc::clone(&state));
            if cancellation.is_cancelled() {
                return SpeechInputOutcome::Cancelled(
                    SpeechInputCancellationEvidence::for_request(&request),
                );
            }
            let outcome = {
                let mut s = state.lock().expect("speech input script poisoned");
                let o = s.outcomes.pop_front();
                if o.is_some() {
                    s.consumed += 1;
                }
                o
            };
            match outcome {
                Some(ScriptedSpeechInputOutcome::Success(e)) => SpeechInputOutcome::Success(e),
                Some(ScriptedSpeechInputOutcome::Unavailable) => {
                    SpeechInputOutcome::Failure(SpeechInputFailure::Unavailable)
                }
                Some(ScriptedSpeechInputOutcome::DependencyFailure) | None => {
                    SpeechInputOutcome::Failure(SpeechInputFailure::DependencyFailure)
                }
                Some(ScriptedSpeechInputOutcome::WaitForCancellation) => {
                    cancellation.cancelled().await;
                    SpeechInputOutcome::Cancelled(SpeechInputCancellationEvidence::for_request(
                        &request,
                    ))
                }
            }
        })
    }
}

#[derive(Clone, Default)]
pub struct ManualSpeechInputCancellation {
    inner: Arc<Mutex<ManualCancellationState>>,
}
#[derive(Default)]
struct ManualCancellationState {
    cancelled: bool,
    wakers: Vec<Waker>,
}
impl ManualSpeechInputCancellation {
    pub fn cancel(&self) {
        let wakers = {
            let mut s = self.inner.lock().expect("cancellation signal poisoned");
            s.cancelled = true;
            std::mem::take(&mut s.wakers)
        };
        for w in wakers {
            w.wake();
        }
    }
}
struct WaitCancellation {
    inner: Arc<Mutex<ManualCancellationState>>,
}
impl Future for WaitCancellation {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let mut s = self.inner.lock().expect("cancellation signal poisoned");
        if s.cancelled {
            Poll::Ready(())
        } else {
            s.wakers.push(cx.waker().clone());
            Poll::Pending
        }
    }
}
impl SpeechInputCancellationSignal for ManualSpeechInputCancellation {
    fn is_cancelled(&self) -> bool {
        self.inner
            .lock()
            .expect("cancellation signal poisoned")
            .cancelled
    }
    fn cancelled(&self) -> SpeechInputCancellationFuture<'_> {
        Box::pin(WaitCancellation {
            inner: Arc::clone(&self.inner),
        })
    }
}
