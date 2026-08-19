//! Pure, deterministic pedagogy decisions over read-only mastery projections.
#![forbid(unsafe_code)]

use nexa_domain::{CompetencyId, ProtocolVersion, StudentId};
use nexa_student::{CompetencyStatus, MasteryState};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

/// Closed v1 instructional-option vocabulary. These are routing intents, not lesson execution.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstructionalOption {
    Introduce,
    Explain,
    Demonstrate,
    Practice,
    Hint,
    Clarify,
    Reinforce,
    Review,
    Challenge,
    Assess,
    Retry,
    Advance,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecentOutcome {
    Success,
    PartialSuccess,
    Failure,
}

/// Stable, machine-readable explanation codes. Variants are additive only in a new policy version.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RationaleCode {
    NoEvidence,
    InsufficientEvidence,
    LowModelConfidence,
    RecentSuccess,
    RecentPartialSuccess,
    RecentFailure,
    RepeatedFailure,
    RetryLimitReached,
    MasteryThresholdMet,
    CompetencyMastered,
    PreferredOptionUnavailable,
    ProjectionStatusControls,
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum PedagogyError {
    #[error("policy version {actual:?} does not match supported version {expected:?}")]
    PolicyVersionMismatch {
        expected: ProtocolVersion,
        actual: ProtocolVersion,
    },
    #[error("mastery projection policy version {actual:?} is unsupported; expected {expected:?}")]
    ProjectionVersionMismatch {
        expected: ProtocolVersion,
        actual: ProtocolVersion,
    },
    #[error("pedagogy input is invalid: {message}")]
    InvalidInput { message: &'static str },
    #[error("none of the available instructional options is a safe v1 fallback")]
    NoAvailableOption,
}

/// Validated policy input. Fields are private so a valid value cannot become contradictory.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "InputWire")]
pub struct PedagogyInput {
    policy_version: ProtocolVersion,
    mastery: MasteryState,
    recent_outcome: Option<RecentOutcome>,
    attempt_count: u32,
    consecutive_failures: u32,
    available_options: BTreeSet<InstructionalOption>,
}

#[derive(Deserialize)]
struct InputWire {
    policy_version: ProtocolVersion,
    mastery: MasteryState,
    recent_outcome: Option<RecentOutcome>,
    attempt_count: u32,
    consecutive_failures: u32,
    available_options: BTreeSet<InstructionalOption>,
}

impl TryFrom<InputWire> for PedagogyInput {
    type Error = PedagogyError;

    fn try_from(value: InputWire) -> Result<Self, Self::Error> {
        if value.available_options.is_empty() {
            return Err(PedagogyError::InvalidInput {
                message: "available_options must not be empty",
            });
        }
        if (value.attempt_count == 0) != value.recent_outcome.is_none() {
            return Err(PedagogyError::InvalidInput {
                message: "attempt_count must be zero exactly when recent_outcome is absent",
            });
        }
        if value.consecutive_failures > value.attempt_count {
            return Err(PedagogyError::InvalidInput {
                message: "consecutive_failures exceeds attempt_count",
            });
        }
        if value.recent_outcome == Some(RecentOutcome::Failure) && value.consecutive_failures == 0 {
            return Err(PedagogyError::InvalidInput {
                message: "a recent failure requires at least one consecutive failure",
            });
        }
        if value.recent_outcome != Some(RecentOutcome::Failure) && value.consecutive_failures != 0 {
            return Err(PedagogyError::InvalidInput {
                message: "consecutive failures require a recent failure",
            });
        }
        if value.mastery.status() == CompetencyStatus::Mastered
            && (value.mastery.evidence_count() < PedagogyPolicyV1::MINIMUM_EVIDENCE
                || value.mastery.mastery().get() < PedagogyPolicyV1::MASTERY_THRESHOLD)
        {
            return Err(PedagogyError::InvalidInput {
                message: "mastered status contradicts v1 evidence or mastery boundaries",
            });
        }
        Ok(Self {
            policy_version: value.policy_version,
            mastery: value.mastery,
            recent_outcome: value.recent_outcome,
            attempt_count: value.attempt_count,
            consecutive_failures: value.consecutive_failures,
            available_options: value.available_options,
        })
    }
}

impl PedagogyInput {
    pub fn new(
        policy_version: ProtocolVersion,
        mastery: MasteryState,
        recent_outcome: Option<RecentOutcome>,
        attempt_count: u32,
        consecutive_failures: u32,
        available_options: impl IntoIterator<Item = InstructionalOption>,
    ) -> Result<Self, PedagogyError> {
        Self::try_from(InputWire {
            policy_version,
            mastery,
            recent_outcome,
            attempt_count,
            consecutive_failures,
            available_options: available_options.into_iter().collect(),
        })
    }

    pub fn mastery(&self) -> &MasteryState {
        &self.mastery
    }
    pub fn available_options(&self) -> &BTreeSet<InstructionalOption> {
        &self.available_options
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PedagogyDecision {
    pub policy_version: ProtocolVersion,
    pub student_id: StudentId,
    pub competency_id: CompetencyId,
    pub selected_option: InstructionalOption,
    pub rationale_codes: Vec<RationaleCode>,
}

/// A dependency-light synchronous policy port. Persistence and execution belong above this crate.
pub trait PedagogyPolicy {
    fn version(&self) -> ProtocolVersion;
    fn decide(&self, input: &PedagogyInput) -> Result<PedagogyDecision, PedagogyError>;
}

/// Version 1.0 thresholds: evidence 2, confidence 0.60, mastery 0.85, repeated failures 2, retries 3.
#[derive(Clone, Copy, Debug, Default)]
pub struct PedagogyPolicyV1;

impl PedagogyPolicyV1 {
    pub const MINIMUM_EVIDENCE: u32 = 2;
    pub const LOW_CONFIDENCE: f64 = 0.60;
    pub const MASTERY_THRESHOLD: f64 = 0.85;
    pub const REPEATED_FAILURES: u32 = 2;
    pub const MAX_ATTEMPTS: u32 = 3;

    fn choose(
        input: &PedagogyInput,
        preferences: &[InstructionalOption],
        reasons: &mut Vec<RationaleCode>,
    ) -> Result<InstructionalOption, PedagogyError> {
        if let Some(option) = preferences
            .iter()
            .find(|o| input.available_options.contains(o))
        {
            if Some(option) != preferences.first() {
                reasons.push(RationaleCode::PreferredOptionUnavailable);
            }
            return Ok(*option);
        }
        // The final stable fallback order is independent of caller insertion order.
        reasons.push(RationaleCode::PreferredOptionUnavailable);
        input
            .available_options
            .iter()
            .next()
            .copied()
            .ok_or(PedagogyError::NoAvailableOption)
    }
}

impl PedagogyPolicy for PedagogyPolicyV1 {
    fn version(&self) -> ProtocolVersion {
        ProtocolVersion::new(1, 0)
    }

    fn decide(&self, input: &PedagogyInput) -> Result<PedagogyDecision, PedagogyError> {
        if input.policy_version != self.version() {
            return Err(PedagogyError::PolicyVersionMismatch {
                expected: self.version(),
                actual: input.policy_version,
            });
        }
        if input.mastery.policy_version() != ProtocolVersion::new(1, 0) {
            return Err(PedagogyError::ProjectionVersionMismatch {
                expected: ProtocolVersion::new(1, 0),
                actual: input.mastery.policy_version(),
            });
        }

        let mut reasons = Vec::new();
        let preferences: &[InstructionalOption] =
            if input.mastery.status() == CompetencyStatus::Mastered {
                reasons.extend([
                    RationaleCode::CompetencyMastered,
                    RationaleCode::ProjectionStatusControls,
                ]);
                &[
                    InstructionalOption::Advance,
                    InstructionalOption::Challenge,
                    InstructionalOption::Assess,
                ]
            } else if input.mastery.evidence_count() < Self::MINIMUM_EVIDENCE {
                reasons.push(if input.mastery.evidence_count() == 0 {
                    RationaleCode::NoEvidence
                } else {
                    RationaleCode::InsufficientEvidence
                });
                if input.mastery.evidence_count() == 0 {
                    &[
                        InstructionalOption::Introduce,
                        InstructionalOption::Explain,
                        InstructionalOption::Demonstrate,
                        InstructionalOption::Assess,
                    ]
                } else {
                    &[
                        InstructionalOption::Assess,
                        InstructionalOption::Practice,
                        InstructionalOption::Clarify,
                    ]
                }
            } else if input.mastery.model_confidence().get() < Self::LOW_CONFIDENCE {
                reasons.push(RationaleCode::LowModelConfidence);
                &[
                    InstructionalOption::Assess,
                    InstructionalOption::Clarify,
                    InstructionalOption::Review,
                ]
            } else if input.recent_outcome == Some(RecentOutcome::Failure)
                && input.attempt_count >= Self::MAX_ATTEMPTS
            {
                reasons.extend([
                    RationaleCode::RecentFailure,
                    RationaleCode::RetryLimitReached,
                ]);
                &[
                    InstructionalOption::Reinforce,
                    InstructionalOption::Review,
                    InstructionalOption::Explain,
                    InstructionalOption::Demonstrate,
                ]
            } else if input.recent_outcome == Some(RecentOutcome::Failure)
                && input.consecutive_failures >= Self::REPEATED_FAILURES
            {
                reasons.extend([RationaleCode::RecentFailure, RationaleCode::RepeatedFailure]);
                &[
                    InstructionalOption::Clarify,
                    InstructionalOption::Hint,
                    InstructionalOption::Retry,
                    InstructionalOption::Demonstrate,
                ]
            } else if input.recent_outcome == Some(RecentOutcome::Failure) {
                reasons.push(RationaleCode::RecentFailure);
                &[
                    InstructionalOption::Hint,
                    InstructionalOption::Retry,
                    InstructionalOption::Clarify,
                ]
            } else if input.recent_outcome == Some(RecentOutcome::PartialSuccess) {
                reasons.push(RationaleCode::RecentPartialSuccess);
                &[
                    InstructionalOption::Practice,
                    InstructionalOption::Clarify,
                    InstructionalOption::Reinforce,
                ]
            } else if input.mastery.mastery().get() >= Self::MASTERY_THRESHOLD {
                reasons.extend([
                    RationaleCode::MasteryThresholdMet,
                    RationaleCode::ProjectionStatusControls,
                ]);
                &[
                    InstructionalOption::Assess,
                    InstructionalOption::Challenge,
                    InstructionalOption::Practice,
                ]
            } else if input.recent_outcome == Some(RecentOutcome::Success) {
                reasons.push(RationaleCode::RecentSuccess);
                &[
                    InstructionalOption::Challenge,
                    InstructionalOption::Reinforce,
                    InstructionalOption::Practice,
                ]
            } else {
                reasons.push(RationaleCode::InsufficientEvidence);
                &[
                    InstructionalOption::Review,
                    InstructionalOption::Assess,
                    InstructionalOption::Practice,
                ]
            };
        let selected_option = Self::choose(input, preferences, &mut reasons)?;
        reasons.sort();
        reasons.dedup();
        Ok(PedagogyDecision {
            policy_version: self.version(),
            student_id: input.mastery.student_id(),
            competency_id: input.mastery.competency_id(),
            selected_option,
            rationale_codes: reasons,
        })
    }
}
