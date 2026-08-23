//! Provider-neutral, synchronous response planning over caller-supplied text.
#![forbid(unsafe_code)]

pub mod admission;
pub mod authorization;
pub mod availability;
pub mod generation;
pub mod model;
pub mod prompt;
pub mod registry;
pub mod remote_prompt;
pub mod selection;
pub mod tokenization;
pub mod usage;

use nexa_domain::{
    CitationId, CitationSetId, ClaimId, ContextPackageId, CourseId, EvidenceId,
    HybridRetrievalResultId, InteractionId, LessonId, ProtocolVersion, RetrievalQueryId, SessionId,
    StudentId, TutorResponseId, TutorSectionId,
};
use nexa_knowledge::{CitationResolutionStatus, CitationResult, ContextPackage};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};
use thiserror::Error;

macro_rules! validating_deserialize {
    ($name:ident, $validator:ident, {$($field:ident: $ty:ty),* $(,)?}) => {
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                #[derive(Deserialize)]
                #[serde(deny_unknown_fields)]
                struct Wire { $($field: $ty),* }
                let w = Wire::deserialize(d)?;
                let value = Self { $($field: w.$field),* };
                value.$validator().map_err(serde::de::Error::custom)?;
                Ok(value)
            }
        }
    };
}

pub const V1: ProtocolVersion = ProtocolVersion::new(1, 0);
pub const MAX_SECTIONS: usize = 32;
pub const MAX_SECTION_BYTES: usize = 16_384;
pub const MAX_RESPONSE_BYTES: usize = 65_536;
pub const MAX_REFERENCES_PER_SECTION: usize = 32;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SectionKind {
    Explanation,
    WorkedExample,
    Hint,
    CheckForUnderstanding,
    Remediation,
    Summary,
    NextStepGuidance,
    SafetyRefusal,
    ConstrainedResponse,
}
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Explain,
    Demonstrate,
    Hint,
    CheckUnderstanding,
    Remediate,
    Summarize,
    GuideNextStep,
    Refuse,
    Constrain,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafetyClassification {
    Ordinary,
    AssessmentProtected,
    RefusalRequired,
    ConstrainedRequired,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentRestriction {
    None,
    WithholdAnswers,
    WithholdHiddenEvaluation,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Accepted,
    Constrained,
    Refused,
}
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rationale {
    Validated,
    AssessmentProtection,
    SafetyConstraint,
    SafetyRefusal,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub student_id: StudentId,
    pub course_id: CourseId,
    pub lesson_id: LessonId,
    pub session_id: SessionId,
}

/// Reference-only evidence selected and validated by student/pedagogy owners.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionEvidence {
    pub learner_state_evidence_id: EvidenceId,
    pub pedagogy_decision_evidence_id: EvidenceId,
    pub scope: Scope,
    pub learner_state_version: ProtocolVersion,
    pub pedagogy_policy_version: ProtocolVersion,
    pub decision_evidence_version: ProtocolVersion,
    pub allowed_section_kinds: BTreeSet<SectionKind>,
    pub minimum_scaffolding: u8,
    pub maximum_scaffolding: u8,
    pub assessment_restriction: AssessmentRestriction,
}
impl DecisionEvidence {
    pub(crate) fn validate(&self) -> Result<(), TutorError> {
        if self.learner_state_version != V1
            || self.pedagogy_policy_version != V1
            || self.decision_evidence_version != V1
        {
            return Err(TutorError::UnsupportedVersion);
        }
        if self.allowed_section_kinds.is_empty()
            || self.minimum_scaffolding > self.maximum_scaffolding
            || self.maximum_scaffolding > 10
        {
            return Err(TutorError::InvalidEvidence);
        }
        Ok(())
    }
}
validating_deserialize!(DecisionEvidence, validate, {
    learner_state_evidence_id: EvidenceId,
    pedagogy_decision_evidence_id: EvidenceId,
    scope: Scope,
    learner_state_version: ProtocolVersion,
    pedagogy_policy_version: ProtocolVersion,
    decision_evidence_version: ProtocolVersion,
    allowed_section_kinds: BTreeSet<SectionKind>,
    minimum_scaffolding: u8,
    maximum_scaffolding: u8,
    assessment_restriction: AssessmentRestriction
});

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct InertText(String);
impl InertText {
    pub fn new(value: impl Into<String>) -> Result<Self, TutorError> {
        let v = value.into();
        if v.is_empty()
            || v.len() > MAX_SECTION_BYTES
            || v.chars().any(char::is_control)
            || [
                "<script",
                "javascript:",
                "://",
                "file:",
                "provider_payload",
                "tool_call",
                "BEGIN PRIVATE KEY",
            ]
            .iter()
            .any(|x| v.to_ascii_lowercase().contains(&x.to_ascii_lowercase()))
        {
            Err(TutorError::UnsafeContent)
        } else {
            Ok(Self(v))
        }
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl<'de> Deserialize<'de> for InertText {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}
impl fmt::Debug for InertText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("InertText([REDACTED])")
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CitationBinding {
    pub claim_id: ClaimId,
    pub citation_id: CitationId,
    pub claim_position: u32,
    pub citation_position: u32,
}
impl CitationBinding {
    fn validate(&self) -> Result<(), TutorError> {
        if self.claim_position == 0 || self.citation_position == 0 {
            Err(TutorError::CitationMismatch)
        } else {
            Ok(())
        }
    }
}
validating_deserialize!(CitationBinding, validate, {
    claim_id: ClaimId, citation_id: CitationId, claim_position: u32, citation_position: u32
});

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SectionRequest {
    pub section_id: TutorSectionId,
    pub kind: SectionKind,
    pub content: InertText,
    pub claims: Vec<ClaimId>,
    pub citations: Vec<CitationBinding>,
    pub citations_required: bool,
    pub pedagogy_decision_evidence_id: EvidenceId,
    pub safety: SafetyClassification,
    pub capability: Capability,
    pub scaffolding: u8,
    pub assessment_restriction: AssessmentRestriction,
}
impl SectionRequest {
    fn validate(&self) -> Result<(), TutorError> {
        let unique_claims: BTreeSet<_> = self.claims.iter().collect();
        let unique_citations: BTreeSet<_> = self.citations.iter().map(|b| b.citation_id).collect();
        if self.scaffolding > 10
            || self.claims.len() > MAX_REFERENCES_PER_SECTION
            || self.citations.len() > MAX_REFERENCES_PER_SECTION
            || unique_claims.len() != self.claims.len()
            || unique_citations.len() != self.citations.len()
            || self.citations_required && self.citations.is_empty()
            || self.claims.is_empty() && !self.citations.is_empty()
            || self
                .citations
                .iter()
                .any(|b| !unique_claims.contains(&b.claim_id))
            || self.citations.iter().any(|x| x.validate().is_err())
        {
            Err(TutorError::InvalidStructure)
        } else {
            Ok(())
        }
    }
}
validating_deserialize!(SectionRequest, validate, {
    section_id: TutorSectionId, kind: SectionKind, content: InertText,
    claims: Vec<ClaimId>, citations: Vec<CitationBinding>, citations_required: bool,
    pedagogy_decision_evidence_id: EvidenceId, safety: SafetyClassification,
    capability: Capability, scaffolding: u8, assessment_restriction: AssessmentRestriction
});
impl fmt::Debug for SectionRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SectionRequest")
            .field("section_id", &self.section_id)
            .field("kind", &self.kind)
            .field("content", &"[REDACTED]")
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlanningRequest {
    pub contract_version: ProtocolVersion,
    pub response_id: TutorResponseId,
    pub interaction_id: InteractionId,
    pub scope: Scope,
    pub context_package_id: ContextPackageId,
    pub citation_set_id: CitationSetId,
    pub hybrid_result_id: HybridRetrievalResultId,
    pub query_id: RetrievalQueryId,
    pub response_policy_version: ProtocolVersion,
    pub safety_policy_version: ProtocolVersion,
    pub citation_policy_version: ProtocolVersion,
    pub governance_policy_version: ProtocolVersion,
    pub limits: ResponseLimits,
    pub permitted_capabilities: BTreeSet<Capability>,
    pub evidence: DecisionEvidence,
    pub sections: Vec<SectionRequest>,
}
impl PlanningRequest {
    fn validate_wire(&self) -> Result<(), TutorError> {
        if [
            self.contract_version,
            self.response_policy_version,
            self.safety_policy_version,
            self.citation_policy_version,
            self.governance_policy_version,
        ]
        .iter()
        .any(|version| *version != V1)
        {
            return Err(TutorError::UnsupportedVersion);
        }
        self.limits.validate()?;
        self.evidence.validate()?;
        if self.scope != self.evidence.scope {
            return Err(TutorError::ProvenanceMismatch);
        }
        if self.permitted_capabilities.is_empty()
            || self.sections.is_empty()
            || self.sections.len() > self.limits.maximum_sections
            || self.sections.iter().any(|s| s.validate().is_err())
            || self.sections.iter().any(|s| {
                s.content.as_str().len() > self.limits.maximum_section_bytes
                    || s.claims.len() > self.limits.maximum_references_per_section
                    || s.citations.len() > self.limits.maximum_references_per_section
            })
            || self
                .sections
                .iter()
                .try_fold(0usize, |total, s| {
                    total.checked_add(s.content.as_str().len())
                })
                .is_none_or(|total| total > self.limits.maximum_response_bytes)
        {
            return Err(TutorError::InvalidStructure);
        }
        Ok(())
    }
}
validating_deserialize!(PlanningRequest, validate_wire, {
    contract_version: ProtocolVersion, response_id: TutorResponseId,
    interaction_id: InteractionId, scope: Scope, context_package_id: ContextPackageId,
    citation_set_id: CitationSetId, hybrid_result_id: HybridRetrievalResultId,
    query_id: RetrievalQueryId, response_policy_version: ProtocolVersion,
    safety_policy_version: ProtocolVersion, citation_policy_version: ProtocolVersion,
    governance_policy_version: ProtocolVersion, limits: ResponseLimits,
    permitted_capabilities: BTreeSet<Capability>, evidence: DecisionEvidence,
    sections: Vec<SectionRequest>
});
impl fmt::Debug for PlanningRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlanningRequest")
            .field("response_id", &self.response_id)
            .field("section_count", &self.sections.len())
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseLimits {
    pub maximum_sections: usize,
    pub maximum_section_bytes: usize,
    pub maximum_response_bytes: usize,
    pub maximum_references_per_section: usize,
}
validating_deserialize!(ResponseLimits, validate, {
    maximum_sections: usize, maximum_section_bytes: usize, maximum_response_bytes: usize,
    maximum_references_per_section: usize
});
impl ResponseLimits {
    pub(crate) fn validate(&self) -> Result<(), TutorError> {
        if self.maximum_sections == 0
            || self.maximum_sections > MAX_SECTIONS
            || self.maximum_section_bytes == 0
            || self.maximum_section_bytes > MAX_SECTION_BYTES
            || self.maximum_response_bytes == 0
            || self.maximum_response_bytes > MAX_RESPONSE_BYTES
            || self.maximum_references_per_section == 0
            || self.maximum_references_per_section > MAX_REFERENCES_PER_SECTION
        {
            Err(TutorError::InvalidLimit)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlannedSection {
    pub section_id: TutorSectionId,
    pub position: u32,
    pub kind: SectionKind,
    pub content: InertText,
    pub claims: Vec<ClaimId>,
    pub citations: Vec<CitationBinding>,
    pub citations_required: bool,
    pub pedagogy_decision_evidence_id: EvidenceId,
    pub safety: SafetyClassification,
    pub capability: Capability,
    pub scaffolding: u8,
    pub assessment_restriction: AssessmentRestriction,
}
impl PlannedSection {
    fn validate(&self) -> Result<(), TutorError> {
        if self.position == 0
            || self.scaffolding > 10
            || self.citations_required && self.citations.is_empty()
            || self.claims.is_empty() && !self.citations.is_empty()
            || self.citations.iter().any(|x| x.validate().is_err())
        {
            Err(TutorError::InvalidStructure)
        } else {
            Ok(())
        }
    }
}
validating_deserialize!(PlannedSection, validate, {
    section_id: TutorSectionId, position: u32, kind: SectionKind, content: InertText,
    claims: Vec<ClaimId>, citations: Vec<CitationBinding>, citations_required: bool,
    pedagogy_decision_evidence_id: EvidenceId, safety: SafetyClassification,
    capability: Capability, scaffolding: u8, assessment_restriction: AssessmentRestriction
});
impl fmt::Debug for PlannedSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlannedSection")
            .field("section_id", &self.section_id)
            .field("position", &self.position)
            .field("content", &"[REDACTED]")
            .finish()
    }
}

/// Reference-only copy of the governed citation decision used for replay.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CitationDecisionAnchor {
    pub claim_id: ClaimId,
    pub citation_id: CitationId,
    pub claim_position: u32,
    pub citation_position: u32,
    pub resolved: bool,
}
impl CitationDecisionAnchor {
    fn validate(&self) -> Result<(), TutorError> {
        if !self.resolved || self.claim_position == 0 || self.citation_position == 0 {
            Err(TutorError::CitationMismatch)
        } else {
            Ok(())
        }
    }
}
validating_deserialize!(CitationDecisionAnchor, validate, {
    claim_id: ClaimId, citation_id: CitationId, claim_position: u32,
    citation_position: u32, resolved: bool
});

/// Independent authoritative manifest copied from the validated citation result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CitationManifestEntry {
    pub claim_id: ClaimId,
    pub citation_id: CitationId,
    pub claim_position: u32,
    pub citation_position: u32,
    pub resolved: bool,
}
impl CitationManifestEntry {
    fn validate(&self) -> Result<(), TutorError> {
        if !self.resolved || self.claim_position == 0 || self.citation_position == 0 {
            Err(TutorError::CitationMismatch)
        } else {
            Ok(())
        }
    }
}
validating_deserialize!(CitationManifestEntry, validate, {
    claim_id: ClaimId, citation_id: CitationId, claim_position: u32,
    citation_position: u32, resolved: bool
});

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TutorResponse {
    pub contract_version: ProtocolVersion,
    pub response_id: TutorResponseId,
    pub interaction_id: InteractionId,
    pub scope: Scope,
    pub context_package_id: ContextPackageId,
    pub citation_set_id: CitationSetId,
    pub hybrid_result_id: HybridRetrievalResultId,
    pub query_id: RetrievalQueryId,
    pub response_policy_version: ProtocolVersion,
    pub safety_policy_version: ProtocolVersion,
    pub citation_policy_version: ProtocolVersion,
    pub governance_policy_version: ProtocolVersion,
    pub limits: ResponseLimits,
    pub permitted_capabilities: BTreeSet<Capability>,
    pub evidence: DecisionEvidence,
    pub context_was_empty: bool,
    pub ordered_section_ids: Vec<TutorSectionId>,
    pub citation_anchors: Vec<CitationDecisionAnchor>,
    pub citation_manifest: Vec<CitationManifestEntry>,
    pub status: ResponseStatus,
    pub rationale: Vec<Rationale>,
    pub sections: Vec<PlannedSection>,
    pub replay_anchor: String,
}
impl fmt::Debug for TutorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TutorResponse")
            .field("response_id", &self.response_id)
            .field("status", &self.status)
            .field("sections", &"[REDACTED]")
            .finish()
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum TutorError {
    #[error("unsupported contract or policy version")]
    UnsupportedVersion,
    #[error("invalid bounded limit")]
    InvalidLimit,
    #[error("invalid or contradictory evidence")]
    InvalidEvidence,
    #[error("identity is duplicate or inconsistent")]
    IdentityConflict,
    #[error("scope or provenance does not agree")]
    ProvenanceMismatch,
    #[error("citation is unresolved, unknown, duplicated, or reassociated")]
    CitationMismatch,
    #[error("section capability is not permitted")]
    UnsupportedCapability,
    #[error("caller content violates inert-content policy")]
    UnsafeContent,
    #[error("response structure is invalid")]
    InvalidStructure,
    #[error("replay anchor mismatch")]
    ReplayMismatch,
}

fn expected_capability(k: SectionKind) -> Capability {
    match k {
        SectionKind::Explanation => Capability::Explain,
        SectionKind::WorkedExample => Capability::Demonstrate,
        SectionKind::Hint => Capability::Hint,
        SectionKind::CheckForUnderstanding => Capability::CheckUnderstanding,
        SectionKind::Remediation => Capability::Remediate,
        SectionKind::Summary => Capability::Summarize,
        SectionKind::NextStepGuidance => Capability::GuideNextStep,
        SectionKind::SafetyRefusal => Capability::Refuse,
        SectionKind::ConstrainedResponse => Capability::Constrain,
    }
}
fn safety_is_compatible(
    kind: SectionKind,
    safety: SafetyClassification,
    assessment_restriction: AssessmentRestriction,
) -> bool {
    match kind {
        SectionKind::SafetyRefusal => safety == SafetyClassification::RefusalRequired,
        SectionKind::ConstrainedResponse
            if assessment_restriction != AssessmentRestriction::None =>
        {
            safety == SafetyClassification::AssessmentProtected
        }
        SectionKind::ConstrainedResponse => safety == SafetyClassification::ConstrainedRequired,
        SectionKind::Hint | SectionKind::CheckForUnderstanding
            if assessment_restriction != AssessmentRestriction::None =>
        {
            safety == SafetyClassification::AssessmentProtected
        }
        _ => {
            safety == SafetyClassification::Ordinary
                && assessment_restriction == AssessmentRestriction::None
        }
    }
}
fn anchor<T: Serialize>(value: &T) -> Result<String, TutorError> {
    let bytes = serde_json::to_vec(value).map_err(|_| TutorError::InvalidStructure)?;
    Ok(Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

pub fn plan_response(
    request: &PlanningRequest,
    context: &ContextPackage,
    citations: &CitationResult,
) -> Result<TutorResponse, TutorError> {
    request.validate(context, citations)?;
    let status = if request
        .sections
        .iter()
        .all(|s| s.kind == SectionKind::SafetyRefusal)
    {
        ResponseStatus::Refused
    } else if request.sections.iter().any(|s| {
        matches!(s.kind, SectionKind::ConstrainedResponse)
            || matches!(
                s.safety,
                SafetyClassification::ConstrainedRequired
                    | SafetyClassification::AssessmentProtected
            )
    }) {
        ResponseStatus::Constrained
    } else {
        ResponseStatus::Accepted
    };
    let rationale = match status {
        ResponseStatus::Accepted => vec![Rationale::Validated],
        ResponseStatus::Constrained => vec![if request.evidence.assessment_restriction
            != AssessmentRestriction::None
        {
            Rationale::AssessmentProtection
        } else {
            Rationale::SafetyConstraint
        }],
        ResponseStatus::Refused => vec![Rationale::SafetyRefusal],
    };
    let sections = request
        .sections
        .iter()
        .enumerate()
        .map(|(i, s)| PlannedSection {
            section_id: s.section_id,
            position: (i + 1) as u32,
            kind: s.kind,
            content: s.content.clone(),
            claims: s.claims.clone(),
            citations: s.citations.clone(),
            citations_required: s.citations_required,
            pedagogy_decision_evidence_id: s.pedagogy_decision_evidence_id,
            safety: s.safety,
            capability: s.capability,
            scaffolding: s.scaffolding,
            assessment_restriction: s.assessment_restriction,
        })
        .collect();
    let mut out = TutorResponse {
        contract_version: V1,
        response_id: request.response_id,
        interaction_id: request.interaction_id,
        scope: request.scope.clone(),
        context_package_id: request.context_package_id,
        citation_set_id: request.citation_set_id,
        hybrid_result_id: request.hybrid_result_id,
        query_id: request.query_id,
        response_policy_version: request.response_policy_version,
        safety_policy_version: request.safety_policy_version,
        citation_policy_version: request.citation_policy_version,
        governance_policy_version: request.governance_policy_version,
        limits: request.limits,
        permitted_capabilities: request.permitted_capabilities.clone(),
        evidence: request.evidence.clone(),
        context_was_empty: context.included.is_empty(),
        ordered_section_ids: request.sections.iter().map(|s| s.section_id).collect(),
        citation_anchors: request
            .sections
            .iter()
            .flat_map(|s| s.citations.iter())
            .map(|b| CitationDecisionAnchor {
                claim_id: b.claim_id,
                citation_id: b.citation_id,
                claim_position: b.claim_position,
                citation_position: b.citation_position,
                resolved: true,
            })
            .collect(),
        citation_manifest: citations
            .claims
            .iter()
            .filter(|claim| claim.status == CitationResolutionStatus::Resolved)
            .flat_map(|claim| {
                claim
                    .citations
                    .iter()
                    .map(|citation| CitationManifestEntry {
                        claim_id: claim.claim_id,
                        citation_id: citation.citation_id,
                        claim_position: claim.claim_position,
                        citation_position: citation.citation_position,
                        resolved: true,
                    })
            })
            .collect(),
        status,
        rationale,
        sections,
        replay_anchor: String::new(),
    };
    out.replay_anchor = out.compute_anchor()?;
    Ok(out)
}

impl PlanningRequest {
    pub fn validate(
        &self,
        context: &ContextPackage,
        citations: &CitationResult,
    ) -> Result<(), TutorError> {
        if [
            self.contract_version,
            self.response_policy_version,
            self.safety_policy_version,
            self.citation_policy_version,
            self.governance_policy_version,
        ]
        .iter()
        .any(|v| *v != V1)
        {
            return Err(TutorError::UnsupportedVersion);
        }
        self.limits.validate()?;
        self.evidence.validate()?;
        context
            .validate()
            .map_err(|_| TutorError::ProvenanceMismatch)?;
        citations
            .validate()
            .map_err(|_| TutorError::CitationMismatch)?;
        if self.scope != self.evidence.scope
            || (
                self.context_package_id,
                self.hybrid_result_id,
                self.query_id,
            ) != (
                context.context_package_id,
                context.hybrid_result_id,
                context.query_id,
            )
            || (
                self.citation_set_id,
                self.context_package_id,
                self.hybrid_result_id,
                self.query_id,
            ) != (
                citations.citation_set_id,
                citations.context_package_id,
                citations.hybrid_result_id,
                citations.query_id,
            )
            || self.governance_policy_version != context.governance_policy_version
            || self.citation_policy_version != citations.citation_policy_version
        {
            return Err(TutorError::ProvenanceMismatch);
        }
        if self.sections.is_empty()
            || self.sections.len() > self.limits.maximum_sections
            || self.permitted_capabilities.is_empty()
        {
            return Err(TutorError::InvalidLimit);
        }
        let refusal = self.sections.iter().any(|s| {
            s.kind == SectionKind::SafetyRefusal
                || s.safety == SafetyClassification::RefusalRequired
        });
        if refusal
            && self.sections.iter().any(|s| {
                s.kind != SectionKind::SafetyRefusal
                    || s.safety != SafetyClassification::RefusalRequired
            })
        {
            return Err(TutorError::InvalidEvidence);
        }
        let mut section_ids = BTreeSet::new();
        let mut total = 0usize;
        let claim_map: BTreeMap<_, _> = citations.claims.iter().map(|c| (c.claim_id, c)).collect();
        let mut all_citations = BTreeSet::new();
        for s in &self.sections {
            if !section_ids.insert(s.section_id) {
                return Err(TutorError::IdentityConflict);
            }
            if s.content.as_str().len() > self.limits.maximum_section_bytes
                || s.claims.len() > self.limits.maximum_references_per_section
                || s.citations.len() > self.limits.maximum_references_per_section
            {
                return Err(TutorError::InvalidLimit);
            }
            total = total
                .checked_add(s.content.as_str().len())
                .ok_or(TutorError::InvalidLimit)?;
            if expected_capability(s.kind) != s.capability
                || !self.permitted_capabilities.contains(&s.capability)
            {
                return Err(TutorError::UnsupportedCapability);
            }
            if !self.evidence.allowed_section_kinds.contains(&s.kind)
                || s.pedagogy_decision_evidence_id != self.evidence.pedagogy_decision_evidence_id
                || s.scaffolding < self.evidence.minimum_scaffolding
                || s.scaffolding > self.evidence.maximum_scaffolding
            {
                return Err(TutorError::InvalidEvidence);
            }
            if s.citations_required && s.citations.is_empty() {
                return Err(TutorError::CitationMismatch);
            }
            let mut local_claims = BTreeSet::new();
            if s.claims.iter().any(|id| !local_claims.insert(*id)) {
                return Err(TutorError::IdentityConflict);
            }
            let mut local_cites = BTreeSet::new();
            for b in &s.citations {
                if !local_cites.insert(b.citation_id) || !all_citations.insert(b.citation_id) {
                    return Err(TutorError::CitationMismatch);
                }
                let c = claim_map
                    .get(&b.claim_id)
                    .ok_or(TutorError::CitationMismatch)?;
                if c.status != CitationResolutionStatus::Resolved
                    || c.claim_position != b.claim_position
                    || !s.claims.contains(&b.claim_id)
                    || !c.citations.iter().any(|x| {
                        x.citation_id == b.citation_id && x.citation_position == b.citation_position
                    })
                {
                    return Err(TutorError::CitationMismatch);
                }
            }
            let protected = self.evidence.assessment_restriction != AssessmentRestriction::None;
            if s.assessment_restriction != self.evidence.assessment_restriction
                || protected
                    && !matches!(
                        s.kind,
                        SectionKind::Hint
                            | SectionKind::CheckForUnderstanding
                            | SectionKind::ConstrainedResponse
                            | SectionKind::SafetyRefusal
                    )
            {
                return Err(TutorError::InvalidEvidence);
            }
            if !safety_is_compatible(s.kind, s.safety, self.evidence.assessment_restriction) {
                return Err(TutorError::InvalidEvidence);
            }
        }
        if total > self.limits.maximum_response_bytes {
            return Err(TutorError::InvalidLimit);
        }
        if context.included.is_empty()
            && self.sections.iter().any(|s| {
                !matches!(
                    s.kind,
                    SectionKind::SafetyRefusal | SectionKind::ConstrainedResponse
                )
            })
        {
            return Err(TutorError::CitationMismatch);
        }
        Ok(())
    }
}
impl TutorResponse {
    fn compute_anchor(&self) -> Result<String, TutorError> {
        #[derive(Serialize)]
        struct A<'a> {
            contract_version: ProtocolVersion,
            response_id: TutorResponseId,
            interaction_id: InteractionId,
            scope: &'a Scope,
            context_package_id: ContextPackageId,
            citation_set_id: CitationSetId,
            hybrid_result_id: HybridRetrievalResultId,
            query_id: RetrievalQueryId,
            response_policy_version: ProtocolVersion,
            safety_policy_version: ProtocolVersion,
            citation_policy_version: ProtocolVersion,
            governance_policy_version: ProtocolVersion,
            limits: ResponseLimits,
            permitted_capabilities: &'a BTreeSet<Capability>,
            evidence: &'a DecisionEvidence,
            context_was_empty: bool,
            ordered_section_ids: &'a [TutorSectionId],
            citation_anchors: &'a [CitationDecisionAnchor],
            citation_manifest: &'a [CitationManifestEntry],
            status: ResponseStatus,
            rationale: &'a [Rationale],
            sections: &'a [PlannedSection],
        }
        anchor(&A {
            contract_version: self.contract_version,
            response_id: self.response_id,
            interaction_id: self.interaction_id,
            scope: &self.scope,
            context_package_id: self.context_package_id,
            citation_set_id: self.citation_set_id,
            hybrid_result_id: self.hybrid_result_id,
            query_id: self.query_id,
            response_policy_version: self.response_policy_version,
            safety_policy_version: self.safety_policy_version,
            citation_policy_version: self.citation_policy_version,
            governance_policy_version: self.governance_policy_version,
            limits: self.limits,
            permitted_capabilities: &self.permitted_capabilities,
            evidence: &self.evidence,
            context_was_empty: self.context_was_empty,
            ordered_section_ids: &self.ordered_section_ids,
            citation_anchors: &self.citation_anchors,
            citation_manifest: &self.citation_manifest,
            status: self.status,
            rationale: &self.rationale,
            sections: &self.sections,
        })
    }
    pub fn validate(&self) -> Result<(), TutorError> {
        if self.contract_version != V1
            || self.response_policy_version != V1
            || self.safety_policy_version != V1
            || self.citation_policy_version != V1
            || self.governance_policy_version != V1
        {
            return Err(TutorError::UnsupportedVersion);
        }
        self.limits.validate()?;
        self.evidence.validate()?;
        if self.sections.is_empty() || self.sections.len() > self.limits.maximum_sections {
            return Err(TutorError::InvalidStructure);
        }
        if self.scope != self.evidence.scope || self.permitted_capabilities.is_empty() {
            return Err(TutorError::ProvenanceMismatch);
        }
        let mut ids = BTreeSet::new();
        let mut bindings = Vec::new();
        let mut global_citations = BTreeSet::new();
        let mut total: usize = 0;
        for (i, s) in self.sections.iter().enumerate() {
            s.validate()?;
            if s.position as usize != i + 1
                || !ids.insert(s.section_id)
                || self.ordered_section_ids.get(i) != Some(&s.section_id)
                || expected_capability(s.kind) != s.capability
                || !self.permitted_capabilities.contains(&s.capability)
                || !self.evidence.allowed_section_kinds.contains(&s.kind)
                || s.pedagogy_decision_evidence_id != self.evidence.pedagogy_decision_evidence_id
                || s.scaffolding < self.evidence.minimum_scaffolding
                || s.scaffolding > self.evidence.maximum_scaffolding
                || s.assessment_restriction != self.evidence.assessment_restriction
                || s.content.as_str().len() > self.limits.maximum_section_bytes
                || s.claims.len() > self.limits.maximum_references_per_section
                || s.citations.len() > self.limits.maximum_references_per_section
            {
                return Err(TutorError::InvalidStructure);
            }
            let mut claims = BTreeSet::new();
            let mut citations = BTreeSet::new();
            if s.claims.iter().any(|x| !claims.insert(*x)) {
                return Err(TutorError::IdentityConflict);
            }
            for b in &s.citations {
                if !claims.contains(&b.claim_id)
                    || !citations.insert(b.citation_id)
                    || !global_citations.insert(b.citation_id)
                {
                    return Err(TutorError::CitationMismatch);
                }
                bindings.push(CitationDecisionAnchor {
                    claim_id: b.claim_id,
                    citation_id: b.citation_id,
                    claim_position: b.claim_position,
                    citation_position: b.citation_position,
                    resolved: true,
                });
            }
            total = total
                .checked_add(s.content.as_str().len())
                .ok_or(TutorError::InvalidLimit)?;
        }
        if self.ordered_section_ids.len() != self.sections.len()
            || bindings != self.citation_anchors
            || self
                .citation_anchors
                .iter()
                .any(|a| !a.resolved || a.claim_position == 0 || a.citation_position == 0)
        {
            return Err(TutorError::CitationMismatch);
        }
        if self.citation_manifest.len() > MAX_SECTIONS * MAX_REFERENCES_PER_SECTION {
            return Err(TutorError::InvalidLimit);
        }
        let mut manifest_claim_citations = BTreeSet::new();
        let mut manifest_citation_ids = BTreeSet::new();
        for entry in &self.citation_manifest {
            entry.validate()?;
            if !manifest_claim_citations.insert((entry.claim_id, entry.citation_id))
                || !manifest_citation_ids.insert(entry.citation_id)
            {
                return Err(TutorError::CitationMismatch);
            }
        }
        if bindings.iter().any(|binding| {
            !self.citation_manifest.iter().any(|entry| {
                entry.claim_id == binding.claim_id
                    && entry.citation_id == binding.citation_id
                    && entry.claim_position == binding.claim_position
                    && entry.citation_position == binding.citation_position
                    && entry.resolved
            })
        }) {
            return Err(TutorError::CitationMismatch);
        }
        let refusal = self.sections.iter().any(|s| {
            s.kind == SectionKind::SafetyRefusal
                || s.safety == SafetyClassification::RefusalRequired
        });
        if refusal
            && self.sections.iter().any(|s| {
                s.kind != SectionKind::SafetyRefusal
                    || s.safety != SafetyClassification::RefusalRequired
            })
        {
            return Err(TutorError::InvalidEvidence);
        }
        for s in &self.sections {
            if !safety_is_compatible(s.kind, s.safety, self.evidence.assessment_restriction) {
                return Err(TutorError::InvalidEvidence);
            }
        }
        let expected_status = if refusal {
            ResponseStatus::Refused
        } else if self.sections.iter().any(|s| {
            s.kind == SectionKind::ConstrainedResponse
                || s.safety == SafetyClassification::AssessmentProtected
        }) {
            ResponseStatus::Constrained
        } else {
            ResponseStatus::Accepted
        };
        let expected_rationale = match expected_status {
            ResponseStatus::Accepted => vec![Rationale::Validated],
            ResponseStatus::Refused => vec![Rationale::SafetyRefusal],
            ResponseStatus::Constrained => vec![if self.evidence.assessment_restriction
                != AssessmentRestriction::None
            {
                Rationale::AssessmentProtection
            } else {
                Rationale::SafetyConstraint
            }],
        };
        if self.status != expected_status || self.rationale != expected_rationale {
            return Err(TutorError::InvalidEvidence);
        }
        if self.context_was_empty
            && self.sections.iter().any(|s| {
                !matches!(
                    s.kind,
                    SectionKind::SafetyRefusal | SectionKind::ConstrainedResponse
                )
            })
        {
            return Err(TutorError::CitationMismatch);
        }
        if total > self.limits.maximum_response_bytes
            || self.compute_anchor()? != self.replay_anchor
        {
            return Err(TutorError::ReplayMismatch);
        }
        Ok(())
    }
}
impl<'de> Deserialize<'de> for TutorResponse {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct W {
            contract_version: ProtocolVersion,
            response_id: TutorResponseId,
            interaction_id: InteractionId,
            scope: Scope,
            context_package_id: ContextPackageId,
            citation_set_id: CitationSetId,
            hybrid_result_id: HybridRetrievalResultId,
            query_id: RetrievalQueryId,
            response_policy_version: ProtocolVersion,
            safety_policy_version: ProtocolVersion,
            citation_policy_version: ProtocolVersion,
            governance_policy_version: ProtocolVersion,
            limits: ResponseLimits,
            permitted_capabilities: BTreeSet<Capability>,
            evidence: DecisionEvidence,
            context_was_empty: bool,
            ordered_section_ids: Vec<TutorSectionId>,
            citation_anchors: Vec<CitationDecisionAnchor>,
            citation_manifest: Vec<CitationManifestEntry>,
            status: ResponseStatus,
            rationale: Vec<Rationale>,
            sections: Vec<PlannedSection>,
            replay_anchor: String,
        }
        let w = W::deserialize(d)?;
        let x = Self {
            contract_version: w.contract_version,
            response_id: w.response_id,
            interaction_id: w.interaction_id,
            scope: w.scope,
            context_package_id: w.context_package_id,
            citation_set_id: w.citation_set_id,
            hybrid_result_id: w.hybrid_result_id,
            query_id: w.query_id,
            response_policy_version: w.response_policy_version,
            safety_policy_version: w.safety_policy_version,
            citation_policy_version: w.citation_policy_version,
            governance_policy_version: w.governance_policy_version,
            limits: w.limits,
            permitted_capabilities: w.permitted_capabilities,
            evidence: w.evidence,
            context_was_empty: w.context_was_empty,
            ordered_section_ids: w.ordered_section_ids,
            citation_anchors: w.citation_anchors,
            citation_manifest: w.citation_manifest,
            status: w.status,
            rationale: w.rationale,
            sections: w.sections,
            replay_anchor: w.replay_anchor,
        };
        x.validate().map_err(serde::de::Error::custom)?;
        Ok(x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexa_knowledge::{resolve_citations, CitationRequest, ContentHash};
    use serde_json::json;
    use uuid::Uuid;
    fn id<T>(n: u128, make: impl FnOnce(Uuid) -> Result<T, nexa_domain::ValueError>) -> T {
        make(Uuid::from_u128(n)).unwrap()
    }
    fn response() -> TutorResponse {
        let scope = Scope {
            student_id: id(1, StudentId::new),
            course_id: id(2, CourseId::new),
            lesson_id: id(3, LessonId::new),
            session_id: id(4, SessionId::new),
        };
        let evidence = DecisionEvidence {
            learner_state_evidence_id: id(5, EvidenceId::new),
            pedagogy_decision_evidence_id: id(6, EvidenceId::new),
            scope: scope.clone(),
            learner_state_version: V1,
            pedagogy_policy_version: V1,
            decision_evidence_version: V1,
            allowed_section_kinds: [SectionKind::Explanation].into(),
            minimum_scaffolding: 1,
            maximum_scaffolding: 3,
            assessment_restriction: AssessmentRestriction::None,
        };
        let mut r = TutorResponse {
            contract_version: V1,
            response_id: id(7, TutorResponseId::new),
            interaction_id: id(8, InteractionId::new),
            scope,
            context_package_id: id(9, ContextPackageId::new),
            citation_set_id: id(10, CitationSetId::new),
            hybrid_result_id: id(11, HybridRetrievalResultId::new),
            query_id: id(12, RetrievalQueryId::new),
            response_policy_version: V1,
            safety_policy_version: V1,
            citation_policy_version: V1,
            governance_policy_version: V1,
            limits: ResponseLimits {
                maximum_sections: 2,
                maximum_section_bytes: 100,
                maximum_response_bytes: 200,
                maximum_references_per_section: 2,
            },
            permitted_capabilities: [Capability::Explain].into(),
            evidence,
            context_was_empty: false,
            ordered_section_ids: vec![id(13, TutorSectionId::new)],
            citation_anchors: vec![],
            citation_manifest: vec![],
            status: ResponseStatus::Accepted,
            rationale: vec![Rationale::Validated],
            sections: vec![PlannedSection {
                section_id: id(13, TutorSectionId::new),
                position: 1,
                kind: SectionKind::Explanation,
                content: InertText::new("Caller supplied explanation.").unwrap(),
                claims: vec![],
                citations: vec![],
                citations_required: false,
                pedagogy_decision_evidence_id: id(6, EvidenceId::new),
                safety: SafetyClassification::Ordinary,
                capability: Capability::Explain,
                scaffolding: 2,
                assessment_restriction: AssessmentRestriction::None,
            }],
            replay_anchor: String::new(),
        };
        r.replay_anchor = r.compute_anchor().unwrap();
        r
    }
    #[test]
    fn stable_round_trip_and_unknown_field_rejection() {
        let r = response();
        let wire = serde_json::to_string(&r).unwrap();
        assert_eq!(serde_json::from_str::<TutorResponse>(&wire).unwrap(), r);
        let bad = wire.replacen('{', "{\"unknown\":true,", 1);
        assert!(serde_json::from_str::<TutorResponse>(&bad).is_err())
    }
    #[test]
    fn coordinated_reorder_status_and_governance_tampering_fail() {
        let r = response();
        let mut v = serde_json::to_value(r).unwrap();
        v["status"] = "refused".into();
        assert!(serde_json::from_value::<TutorResponse>(v).is_err());
        let mut v = serde_json::to_value(response()).unwrap();
        v["governance_policy_version"] = "0.1".into();
        assert!(serde_json::from_value::<TutorResponse>(v).is_err())
    }
    fn recompute(r: &mut TutorResponse) {
        r.replay_anchor = r.compute_anchor().unwrap();
    }
    fn planning_wire() -> serde_json::Value {
        let r = response();
        let mut value = serde_json::to_value(&r).unwrap();
        let object = value.as_object_mut().unwrap();
        for response_only in [
            "context_was_empty",
            "ordered_section_ids",
            "citation_anchors",
            "citation_manifest",
            "status",
            "rationale",
            "replay_anchor",
        ] {
            object.remove(response_only);
        }
        object["sections"][0]
            .as_object_mut()
            .unwrap()
            .remove("position");
        value
    }
    fn planning_request(mut r: TutorResponse) -> PlanningRequest {
        r.replay_anchor = r.compute_anchor().unwrap();
        serde_json::from_value({
            let mut value = serde_json::to_value(r).unwrap();
            let object = value.as_object_mut().unwrap();
            for response_only in [
                "context_was_empty",
                "ordered_section_ids",
                "citation_anchors",
                "citation_manifest",
                "status",
                "rationale",
                "replay_anchor",
            ] {
                object.remove(response_only);
            }
            object["sections"][0]
                .as_object_mut()
                .unwrap()
                .remove("position");
            value
        })
        .unwrap()
    }
    fn governed_inputs() -> (ContextPackage, CitationResult) {
        let fingerprint = serde_json::to_value(ContentHash::sha256(b"a")).unwrap();
        let context: ContextPackage = serde_json::from_value(json!({
            "contract_version":"1.0", "context_package_id":id(9, ContextPackageId::new),
            "hybrid_result_id":id(11, HybridRetrievalResultId::new),
            "query_id":id(12, RetrievalQueryId::new), "assembly_policy_version":"1.0",
            "governance_policy_version":"1.0", "tokenizer_profile_id":"test",
            "tokenizer_fingerprint":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "maximum_tokens":10, "maximum_chunk_tokens":null, "maximum_chunks":null,
            "ordering_policy":"hybrid_rank", "packing_policy":"ranked_greedy_whole_chunk",
            "accounting":{"policy_version":"1.0","fixed_package_overhead":0,"per_chunk_overhead":0,"separator_tokens":0,"metadata_reference_overhead":0},
            "hybrid_candidate_count":1, "used_tokens":1, "remaining_tokens":9,
            "included":[{"chunk_id":"00000000-0000-0000-0000-000000000021","artifact_id":"00000000-0000-0000-0000-000000000022","source_id":"00000000-0000-0000-0000-000000000023","source_version":1,"hybrid_rank":1,"content_fingerprint":fingerprint,"exact_token_count":1,"overhead_tokens":0,"total_contribution":1,"final_context_position":1}],
            "exclusions":[], "content":[{"chunk_id":"00000000-0000-0000-0000-000000000021","final_context_position":1,"content":"a"}]
        })).unwrap();
        let citation_request: CitationRequest = serde_json::from_value(json!({
            "contract_version":"1.0", "citation_set_id":id(10, CitationSetId::new),
            "context_package_id":id(9, ContextPackageId::new),
            "hybrid_result_id":id(11, HybridRetrievalResultId::new),
            "query_id":id(12, RetrievalQueryId::new), "citation_policy_version":"1.0",
            "locator_policy_version":"1.0", "governance_policy_version":"1.0",
            "integrity_profile_version":"1.0", "maximum_citations":2,
            "maximum_citations_per_claim":2,
            "claims":[{"claim_id":"00000000-0000-0000-0000-000000000024","evidence":[]}]
        }))
        .unwrap();
        let citations = resolve_citations(&citation_request, &context).unwrap();
        (context, citations)
    }

    struct AdmissionFixture {
        descriptor: crate::model::ModelDescriptor,
        request: crate::model::ModelRequest,
        response: crate::model::ModelResponse,
        compilation: crate::prompt::PromptCompilationResult,
        authority: crate::admission::TrustedPlanningAuthority,
        context: ContextPackage,
        citations: CitationResult,
        section: serde_json::Value,
    }

    impl AdmissionFixture {
        fn admit(
            &self,
        ) -> Result<crate::admission::AdmissionResult, crate::admission::AdmissionError> {
            crate::admission::admit_model_output(
                &self.descriptor,
                &self.request,
                &self.response,
                &self.compilation,
                &self.authority,
                &self.context,
                &self.citations,
            )
        }

        fn set_candidate(&mut self, candidate: serde_json::Value) {
            self.response.output =
                crate::model::RawModelOutput::new(serde_json::to_string(&candidate).unwrap())
                    .unwrap();
        }

        fn set_sections(&mut self, sections: Vec<serde_json::Value>) {
            self.set_candidate(json!({"candidate_schema_version":"1.0", "sections":sections}));
        }
    }

    fn admission_fixture() -> AdmissionFixture {
        use crate::model::{
            FinishReason, ModelCapabilities, ModelDescriptor, ModelRequest, ModelResponse,
            PrivacyClass, RawModelOutput, RequiredCapabilities, MODEL_INVOCATION_V1,
        };
        use crate::prompt::{
            compile_prompt, PromptCompilationRequest, PromptContent, PromptLayer, PromptLayerKind,
            PromptLimits, PROMPT_COMPILATION_V1,
        };
        use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId};

        let planning = planning_request(response());
        let authority = crate::admission::TrustedPlanningAuthority {
            contract_version: planning.contract_version,
            response_id: planning.response_id,
            interaction_id: planning.interaction_id,
            scope: planning.scope.clone(),
            context_package_id: planning.context_package_id,
            citation_set_id: planning.citation_set_id,
            hybrid_result_id: planning.hybrid_result_id,
            query_id: planning.query_id,
            response_policy_version: planning.response_policy_version,
            safety_policy_version: planning.safety_policy_version,
            citation_policy_version: planning.citation_policy_version,
            governance_policy_version: planning.governance_policy_version,
            limits: planning.limits,
            permitted_capabilities: planning.permitted_capabilities.clone(),
            evidence: planning.evidence.clone(),
        };
        let layer = |kind, text| PromptLayer {
            kind,
            classification: kind.classification(),
            content: PromptContent::new(text).unwrap(),
        };
        let compilation = compile_prompt(&PromptCompilationRequest {
            contract_version: PROMPT_COMPILATION_V1,
            prompt_package_version: V1,
            context_builder_version: V1,
            output_schema_version: V1,
            limits: PromptLimits {
                maximum_layer_bytes: 1000,
                maximum_compiled_bytes: 10000,
            },
            layers: vec![
                layer(
                    PromptLayerKind::PlatformContract,
                    "distinctive private platform prompt prompt-private-sentinel",
                ),
                layer(PromptLayerKind::NexaIdentity, "identity"),
                layer(PromptLayerKind::Policy, "policy"),
                layer(PromptLayerKind::Pedagogy, "pedagogy"),
                layer(
                    PromptLayerKind::StudentInput,
                    "distinctive private learner prompt learner-private-sentinel learner-sentinel",
                ),
                layer(PromptLayerKind::OutputContract, "output"),
            ],
        })
        .unwrap();
        let descriptor = ModelDescriptor::new(
            id(50, ModelProviderId::new),
            id(51, ModelId::new),
            PrivacyClass::LocalOnly,
            ModelCapabilities {
                streaming: false,
                structured_output: true,
                tool_calling: false,
                vision: false,
                context_window_tokens: 20000,
                maximum_output_tokens: 1000,
            },
        )
        .unwrap();
        let request = ModelRequest {
            invocation_id: id(52, ModelInvocationId::new),
            provider_id: descriptor.provider_id,
            model_id: descriptor.model_id,
            contract_version: MODEL_INVOCATION_V1,
            input: compilation.model_input.clone(),
            required_capabilities: RequiredCapabilities {
                structured_output: true,
                tool_calling: false,
                vision: false,
            },
            maximum_output_tokens: 1000,
        };
        let section = serde_json::to_value(&planning.sections[0]).unwrap();
        let output = RawModelOutput::new(
            serde_json::to_string(
                &json!({"candidate_schema_version":"1.0", "sections":[section.clone()]}),
            )
            .unwrap(),
        )
        .unwrap();
        let response = ModelResponse {
            invocation_id: request.invocation_id,
            provider_id: request.provider_id,
            model_id: request.model_id,
            contract_version: MODEL_INVOCATION_V1,
            output,
            finish_reason: FinishReason::Complete,
            reported_usage: None,
        };
        let (context, citations) = governed_inputs();
        AdmissionFixture {
            descriptor,
            request,
            response,
            compilation,
            authority,
            context,
            citations,
            section,
        }
    }

    struct CountingProvider {
        descriptor: crate::model::ModelDescriptor,
        response: crate::model::ModelResponse,
        calls: std::sync::atomic::AtomicUsize,
    }

    impl CountingProvider {
        fn new(fixture: &AdmissionFixture) -> Self {
            Self {
                descriptor: fixture.descriptor.clone(),
                response: fixture.response.clone(),
                calls: std::sync::atomic::AtomicUsize::new(0),
            }
        }

        fn calls(&self) -> usize {
            self.calls.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    impl crate::model::LanguageModelProvider for CountingProvider {
        fn descriptor(&self) -> &crate::model::ModelDescriptor {
            &self.descriptor
        }

        fn generate(
            &self,
            _request: &crate::model::ModelRequest,
        ) -> Result<crate::model::ModelResponse, crate::model::ModelError> {
            self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(self.response.clone())
        }
    }

    struct SentinelTokenizer {
        inner: crate::tokenization::ScriptedModelInputTokenizer,
        private_diagnostic: String,
    }

    impl crate::tokenization::ModelInputTokenizer for SentinelTokenizer {
        fn descriptor(&self) -> &crate::model::ModelDescriptor {
            self.inner.descriptor()
        }

        fn count_input_tokens(
            &self,
            input: &crate::model::ModelInput,
        ) -> Result<u32, crate::tokenization::ModelInputTokenizationError> {
            assert!(!self.private_diagnostic.is_empty());
            self.inner.count_input_tokens(input)
        }
    }

    struct SentinelProvider {
        inner: crate::model::ScriptedModelProvider,
        endpoint: String,
        credential: String,
        private_diagnostic: String,
    }

    impl crate::model::LanguageModelProvider for SentinelProvider {
        fn descriptor(&self) -> &crate::model::ModelDescriptor {
            self.inner.descriptor()
        }

        fn generate(
            &self,
            request: &crate::model::ModelRequest,
        ) -> Result<crate::model::ModelResponse, crate::model::ModelError> {
            assert!(!self.endpoint.is_empty());
            assert!(!self.credential.is_empty());
            assert!(!self.private_diagnostic.is_empty());
            self.inner.generate(request)
        }
    }

    struct ObservingTokenizer {
        inner: crate::tokenization::ScriptedModelInputTokenizer,
        observed: std::sync::Mutex<Vec<crate::model::ModelInput>>,
    }

    impl ObservingTokenizer {
        fn remaining(&self) -> usize {
            self.inner.remaining().unwrap()
        }

        fn observed(&self) -> Vec<crate::model::ModelInput> {
            self.observed.lock().unwrap().clone()
        }
    }

    impl crate::tokenization::ModelInputTokenizer for ObservingTokenizer {
        fn descriptor(&self) -> &crate::model::ModelDescriptor {
            self.inner.descriptor()
        }

        fn count_input_tokens(
            &self,
            input: &crate::model::ModelInput,
        ) -> Result<u32, crate::tokenization::ModelInputTokenizationError> {
            self.observed.lock().unwrap().push(input.clone());
            self.inner.count_input_tokens(input)
        }
    }

    struct ObservingProvider {
        inner: crate::model::ScriptedModelProvider,
        observed: std::sync::Mutex<Vec<crate::model::ModelRequest>>,
    }

    impl ObservingProvider {
        fn remaining(&self) -> usize {
            self.inner.remaining()
        }

        fn observed(&self) -> Vec<crate::model::ModelRequest> {
            self.observed.lock().unwrap().clone()
        }
    }

    impl crate::model::LanguageModelProvider for ObservingProvider {
        fn descriptor(&self) -> &crate::model::ModelDescriptor {
            self.inner.descriptor()
        }

        fn generate(
            &self,
            request: &crate::model::ModelRequest,
        ) -> Result<crate::model::ModelResponse, crate::model::ModelError> {
            self.observed.lock().unwrap().push(request.clone());
            self.inner.generate(request)
        }
    }

    struct SentinelUncheckedProvider {
        inner: UncheckedScriptedProvider,
        endpoint: String,
        credential: String,
        private_diagnostic: String,
    }

    impl SentinelUncheckedProvider {
        fn remaining(&self) -> usize {
            self.inner.remaining()
        }
    }

    impl crate::model::LanguageModelProvider for SentinelUncheckedProvider {
        fn descriptor(&self) -> &crate::model::ModelDescriptor {
            self.inner.descriptor()
        }

        fn generate(
            &self,
            request: &crate::model::ModelRequest,
        ) -> Result<crate::model::ModelResponse, crate::model::ModelError> {
            assert!(!self.endpoint.is_empty());
            assert!(!self.credential.is_empty());
            assert!(!self.private_diagnostic.is_empty());
            self.inner.generate(request)
        }
    }

    fn assert_content_free_diagnostics(
        error: &(impl std::fmt::Debug + std::fmt::Display),
        sentinels: &[&str],
    ) {
        let debug = format!("{error:?}");
        let display = error.to_string();
        for sentinel in sentinels {
            assert!(!debug.contains(sentinel), "Debug leaked {sentinel}");
            assert!(!display.contains(sentinel), "Display leaked {sentinel}");
        }
    }

    struct UncheckedScriptedProvider {
        descriptor: crate::model::ModelDescriptor,
        outcomes: std::sync::Mutex<std::collections::VecDeque<crate::model::ScriptedOutcome>>,
    }

    impl UncheckedScriptedProvider {
        fn new(
            descriptor: crate::model::ModelDescriptor,
            outcomes: impl IntoIterator<Item = crate::model::ScriptedOutcome>,
        ) -> Self {
            Self {
                descriptor,
                outcomes: std::sync::Mutex::new(outcomes.into_iter().collect()),
            }
        }

        fn remaining(&self) -> usize {
            self.outcomes.lock().unwrap().len()
        }
    }

    impl crate::model::LanguageModelProvider for UncheckedScriptedProvider {
        fn descriptor(&self) -> &crate::model::ModelDescriptor {
            &self.descriptor
        }

        fn generate(
            &self,
            _request: &crate::model::ModelRequest,
        ) -> Result<crate::model::ModelResponse, crate::model::ModelError> {
            match self.outcomes.lock().unwrap().pop_front().unwrap() {
                crate::model::ScriptedOutcome::Response(response) => Ok(response),
                crate::model::ScriptedOutcome::Error(kind) => {
                    Err(crate::model::ModelError::new(kind))
                }
            }
        }
    }

    fn registry_with_untouched_sentinels(
        fixture: &AdmissionFixture,
        selected_outcomes: impl IntoIterator<Item = crate::model::ScriptedOutcome>,
    ) -> (
        crate::registry::ModelRegistry,
        std::sync::Arc<crate::model::ScriptedModelProvider>,
        std::sync::Arc<crate::model::ScriptedModelProvider>,
        std::sync::Arc<crate::model::ScriptedModelProvider>,
    ) {
        use crate::model::{LanguageModelProvider, ScriptedModelProvider};
        use std::sync::Arc;

        let selected = Arc::new(
            ScriptedModelProvider::new(fixture.descriptor.clone(), selected_outcomes).unwrap(),
        );
        let (other, remote) = untouched_sentinel_providers(fixture);
        let registry = crate::registry::ModelRegistry::try_from_providers([
            selected.clone() as Arc<dyn LanguageModelProvider>,
            other.clone(),
            remote.clone(),
        ])
        .unwrap();
        (registry, selected, other, remote)
    }

    fn untouched_sentinel_providers(
        fixture: &AdmissionFixture,
    ) -> (
        std::sync::Arc<crate::model::ScriptedModelProvider>,
        std::sync::Arc<crate::model::ScriptedModelProvider>,
    ) {
        use crate::model::{PrivacyClass, ScriptedModelProvider, ScriptedOutcome};
        use nexa_domain::{ModelId, ModelProviderId};
        use std::sync::Arc;

        let mut other_descriptor = fixture.descriptor.clone();
        other_descriptor.provider_id = id(60, ModelProviderId::new);
        other_descriptor.model_id = id(61, ModelId::new);
        let other = Arc::new(
            ScriptedModelProvider::new(
                other_descriptor,
                [ScriptedOutcome::Error(
                    crate::model::ModelErrorKind::Internal,
                )],
            )
            .unwrap(),
        );
        let mut remote_descriptor = fixture.descriptor.clone();
        remote_descriptor.provider_id = id(40, ModelProviderId::new);
        remote_descriptor.model_id = id(41, ModelId::new);
        remote_descriptor.privacy_class = PrivacyClass::ApprovedRemote;
        let remote = Arc::new(
            ScriptedModelProvider::new(
                remote_descriptor,
                [ScriptedOutcome::Error(
                    crate::model::ModelErrorKind::Internal,
                )],
            )
            .unwrap(),
        );
        (other, remote)
    }

    #[allow(clippy::too_many_arguments)]
    fn available_local_usage_wrapper(
        registry: &crate::registry::ModelRegistry,
        invocation_id: nexa_domain::ModelInvocationId,
        requirements: &crate::selection::ModelSelectionRequirements,
        tokenization_contract_version: nexa_domain::ProtocolVersion,
        tokenizer: &dyn crate::tokenization::ModelInputTokenizer,
        compilation: &crate::prompt::PromptCompilationResult,
        authority: &crate::admission::TrustedPlanningAuthority,
        context: &nexa_knowledge::ContextPackage,
        citations: &nexa_knowledge::CitationResult,
    ) -> Result<
        crate::generation::TokenizedInvocationAdmissionResult,
        crate::generation::AvailableLocalUsageValidatedTokenizedInvocationAdmissionError,
    > {
        let availability = crate::availability::ModelAvailabilitySnapshot::new(
            registry
                .inventory()
                .iter()
                .map(|descriptor| crate::availability::ModelAvailabilityEntry {
                    provider_id: descriptor.provider_id,
                    model_id: descriptor.model_id,
                    state: crate::availability::ModelAvailabilityState::Available,
                })
                .collect(),
        )
        .unwrap();
        crate::generation::select_available_local_model_tokenize_invoke_validate_reported_usage_and_admit(
            registry,
            invocation_id,
            requirements,
            &availability,
            tokenization_contract_version,
            tokenizer,
            compilation,
            authority,
            context,
            citations,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn authorized_remote_usage_wrapper(
        registry: &crate::registry::ModelRegistry,
        invocation_id: nexa_domain::ModelInvocationId,
        requirements: &crate::selection::ModelSelectionRequirements,
        tokenization_contract_version: nexa_domain::ProtocolVersion,
        tokenizer: &dyn crate::tokenization::ModelInputTokenizer,
        compilation: &crate::prompt::PromptCompilationResult,
        authority: &crate::admission::TrustedPlanningAuthority,
        context: &nexa_knowledge::ContextPackage,
        citations: &nexa_knowledge::CitationResult,
    ) -> Result<
        crate::generation::TokenizedInvocationAdmissionResult,
        crate::generation::AuthorizedAvailableRemoteUsageValidatedTokenizedInvocationAdmissionError,
    > {
        let entries: Vec<_> = registry
            .inventory()
            .iter()
            .filter(|descriptor| {
                matches!(
                    descriptor.privacy_class,
                    crate::model::PrivacyClass::ApprovedRemote
                        | crate::model::PrivacyClass::RestrictedRemote
                )
            })
            .map(
                |descriptor| crate::authorization::RemoteModelAuthorizationEntry {
                    provider_id: descriptor.provider_id,
                    model_id: descriptor.model_id,
                    privacy_class: descriptor.privacy_class,
                },
            )
            .collect();
        let authorization = crate::authorization::RemoteModelAuthorization::new(
            compilation.replay_anchor.clone(),
            entries.clone(),
        )
        .unwrap();
        let availability = crate::availability::ModelAvailabilitySnapshot::new(
            entries
                .into_iter()
                .map(|entry| crate::availability::ModelAvailabilityEntry {
                    provider_id: entry.provider_id,
                    model_id: entry.model_id,
                    state: crate::availability::ModelAvailabilityState::Available,
                })
                .collect(),
        )
        .unwrap();
        crate::generation::select_authorized_available_remote_model_tokenize_invoke_validate_reported_usage_and_admit(
            registry,
            invocation_id,
            requirements,
            &availability,
            &authorization,
            tokenization_contract_version,
            tokenizer,
            compilation,
            authority,
            context,
            citations,
        )
    }

    #[test]
    fn admission_rejects_descriptor_request_and_response_identity_mismatches() {
        use crate::admission::AdmissionError;
        use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId};
        let mut f = admission_fixture();
        f.request.provider_id = id(60, ModelProviderId::new);
        assert_eq!(f.admit(), Err(AdmissionError::InvalidDescriptorRequest));
        let mut f = admission_fixture();
        f.request.model_id = id(61, ModelId::new);
        assert_eq!(f.admit(), Err(AdmissionError::InvalidDescriptorRequest));
        let mut f = admission_fixture();
        f.response.invocation_id = id(62, ModelInvocationId::new);
        assert_eq!(
            f.admit(),
            Err(AdmissionError::ModelResponseIdentityMismatch)
        );
        let mut f = admission_fixture();
        f.response.provider_id = id(63, ModelProviderId::new);
        assert_eq!(
            f.admit(),
            Err(AdmissionError::ModelResponseIdentityMismatch)
        );
        let mut f = admission_fixture();
        f.response.model_id = id(64, ModelId::new);
        assert_eq!(
            f.admit(),
            Err(AdmissionError::ModelResponseIdentityMismatch)
        );
    }

    #[test]
    fn admission_rejects_unsupported_invocation_contract_versions() {
        use crate::admission::AdmissionError;
        let mut f = admission_fixture();
        f.descriptor.contract_version = ProtocolVersion::new(2, 0);
        assert_eq!(f.admit(), Err(AdmissionError::InvalidDescriptorRequest));
        let mut f = admission_fixture();
        f.request.contract_version = ProtocolVersion::new(2, 0);
        assert_eq!(f.admit(), Err(AdmissionError::InvalidDescriptorRequest));
        let mut f = admission_fixture();
        f.response.contract_version = ProtocolVersion::new(2, 0);
        assert_eq!(f.admit(), Err(AdmissionError::UnsupportedVersion));
    }

    #[test]
    fn admission_requires_structured_output_and_host_limits_are_authoritative() {
        use crate::admission::AdmissionError;
        let mut f = admission_fixture();
        f.request.required_capabilities.structured_output = false;
        assert_eq!(f.admit(), Err(AdmissionError::UnsupportedStructuredOutput));
        let mut f = admission_fixture();
        f.descriptor.capabilities.structured_output = false;
        assert_eq!(f.admit(), Err(AdmissionError::UnsupportedStructuredOutput));
        let mut f = admission_fixture();
        f.descriptor.capabilities.maximum_output_tokens = 999;
        assert_eq!(f.admit(), Err(AdmissionError::InvalidDescriptorRequest));
        let mut f = admission_fixture();
        f.descriptor.capabilities.context_window_tokens =
            f.request.input.as_str().len() as u32 + 999;
        assert_eq!(f.admit(), Err(AdmissionError::InvalidDescriptorRequest));
        for field in [
            "context_window_tokens",
            "maximum_output_tokens",
            "token_limit",
            "usage",
        ] {
            let mut f = admission_fixture();
            let mut candidate =
                json!({"candidate_schema_version":"1.0", "sections":[f.section.clone()]});
            candidate[field] = json!(1);
            f.set_candidate(candidate);
            assert_eq!(
                f.admit(),
                Err(AdmissionError::InvalidCandidateSchema),
                "{field}"
            );
        }
    }

    #[test]
    fn admission_rejects_prompt_input_replay_and_intrinsic_compilation_tampering() {
        use crate::admission::AdmissionError;
        use crate::model::ModelInput;
        let mut f = admission_fixture();
        let mut bytes = f.request.input.as_str().as_bytes().to_vec();
        *bytes.last_mut().unwrap() ^= 1;
        f.request.input = ModelInput::new(String::from_utf8(bytes).unwrap()).unwrap();
        assert_eq!(
            f.admit(),
            Err(AdmissionError::PromptAssociationReplayMismatch)
        );
        for mutation in 0..4 {
            let mut f = admission_fixture();
            match mutation {
                0 => f.compilation.replay_anchor = "a".repeat(64),
                1 => f.compilation.compiled_bytes += 1,
                2 => f.compilation.manifest[0].content_bytes += 1,
                _ => f.compilation.model_input = ModelInput::new("tampered envelope").unwrap(),
            }
            assert_eq!(
                f.admit(),
                Err(AdmissionError::PromptAssociationReplayMismatch)
            );
        }
    }

    #[test]
    fn admission_rejects_every_unsupported_compilation_and_candidate_version() {
        use crate::admission::AdmissionError;
        for field in 0..4 {
            let mut f = admission_fixture();
            match field {
                0 => f.compilation.contract_version = ProtocolVersion::new(2, 0),
                1 => f.compilation.prompt_package_version = ProtocolVersion::new(2, 0),
                2 => f.compilation.context_builder_version = ProtocolVersion::new(2, 0),
                _ => f.compilation.output_schema_version = ProtocolVersion::new(2, 0),
            }
            assert_eq!(f.admit(), Err(AdmissionError::UnsupportedVersion));
        }
        let mut f = admission_fixture();
        f.set_candidate(json!({"candidate_schema_version":"2.0", "sections":[f.section.clone()]}));
        assert_eq!(f.admit(), Err(AdmissionError::UnsupportedVersion));
    }

    #[test]
    fn admission_rejects_malformed_truncated_and_trailing_json() {
        use crate::admission::AdmissionError;
        use crate::model::RawModelOutput;
        for (wire, expected) in [
            ("not json", AdmissionError::MalformedSyntax),
            (
                "{\"candidate_schema_version\":\"1.0\",\"sections\":[",
                AdmissionError::MalformedSyntax,
            ),
            (
                "{\"candidate_schema_version\":\"1.0\",\"sections\":[]} trailing",
                AdmissionError::MalformedSyntax,
            ),
        ] {
            let mut f = admission_fixture();
            f.response.output = RawModelOutput::new(wire).unwrap();
            assert_eq!(f.admit(), Err(expected));
        }
    }

    #[test]
    fn admission_candidate_schema_is_closed_at_every_level() {
        use crate::admission::AdmissionError;
        let attempts = [
            "response_id",
            "provider_id",
            "model_id",
            "invocation_id",
            "policies",
            "limits",
            "decision_evidence",
            "usage",
            "actions",
            "tools",
            "renderer_commands",
        ];
        for field in attempts {
            let mut f = admission_fixture();
            let mut candidate =
                json!({"candidate_schema_version":"1.0", "sections":[f.section.clone()]});
            candidate[field] = json!("host-owned");
            f.set_candidate(candidate);
            assert_eq!(
                f.admit(),
                Err(AdmissionError::InvalidCandidateSchema),
                "{field}"
            );
        }
        let mut f = admission_fixture();
        f.section["unknown_section_field"] = json!(true);
        f.set_sections(vec![f.section.clone()]);
        assert_eq!(f.admit(), Err(AdmissionError::InvalidCandidateSchema));
        let mut f = admission_fixture();
        f.section["kind"] = json!("renderer_command");
        f.set_sections(vec![f.section.clone()]);
        assert_eq!(f.admit(), Err(AdmissionError::InvalidCandidateSchema));
        let mut f = admission_fixture();
        f.set_candidate(json!({"candidate_schema_version":"1.0", "sections":[]}));
        assert_eq!(f.admit(), Err(AdmissionError::InvalidCandidateSchema));
        let mut f = admission_fixture();
        f.set_candidate(planning_wire());
        assert_eq!(f.admit(), Err(AdmissionError::InvalidCandidateSchema));
    }

    #[test]
    fn admission_enforces_raw_section_response_and_count_limits() {
        use crate::admission::AdmissionError;
        use crate::model::{RawModelOutput, MAX_MODEL_OUTPUT_BYTES};
        assert!(RawModelOutput::new("x".repeat(MAX_MODEL_OUTPUT_BYTES + 1)).is_err());
        let mut f = admission_fixture();
        f.authority.limits.maximum_section_bytes = 3;
        assert_eq!(f.admit(), Err(AdmissionError::PlanningEvidenceProvenance));
        let mut f = admission_fixture();
        f.authority.limits.maximum_response_bytes = f.section["content"].as_str().unwrap().len();
        let mut second = f.section.clone();
        second["section_id"] = json!(Uuid::from_u128(80));
        f.set_sections(vec![f.section.clone(), second]);
        assert_eq!(f.admit(), Err(AdmissionError::PlanningEvidenceProvenance));
        let mut f = admission_fixture();
        f.authority.limits.maximum_sections = 1;
        let mut second = f.section.clone();
        second["section_id"] = json!(Uuid::from_u128(81));
        f.set_sections(vec![f.section.clone(), second]);
        assert_eq!(f.admit(), Err(AdmissionError::PlanningEvidenceProvenance));
        let mut f = admission_fixture();
        f.authority.limits.maximum_references_per_section = 1;
        f.section["claims"] = json!([Uuid::from_u128(82), Uuid::from_u128(83)]);
        f.set_sections(vec![f.section.clone()]);
        assert_eq!(f.admit(), Err(AdmissionError::PlanningEvidenceProvenance));
    }

    #[test]
    fn admission_delegates_policy_pedagogy_assessment_and_safety_checks() {
        use crate::admission::AdmissionError;
        for mutation in 0..6 {
            let mut f = admission_fixture();
            match mutation {
                0 => f.section["capability"] = json!("summarize"),
                1 => {
                    f.authority.evidence.allowed_section_kinds = [SectionKind::Summary].into();
                }
                2 => f.section["scaffolding"] = json!(9),
                3 => f.section["pedagogy_decision_evidence_id"] = json!(Uuid::from_u128(84)),
                4 => f.section["assessment_restriction"] = json!("withhold_answers"),
                _ => f.section["safety"] = json!("refusal_required"),
            }
            f.set_sections(vec![f.section.clone()]);
            assert_eq!(
                f.admit(),
                Err(AdmissionError::PolicyPedagogySafetyCapability),
                "mutation {mutation}"
            );
        }
    }

    #[test]
    fn admission_delegates_citation_grounding_and_reference_checks() {
        use crate::admission::AdmissionError;
        let mut f = admission_fixture();
        f.section["citations_required"] = json!(true);
        f.set_sections(vec![f.section.clone()]);
        assert_eq!(f.admit(), Err(AdmissionError::InvalidCandidateSchema));
        let mut f = admission_fixture();
        f.section["claims"] = json!([Uuid::from_u128(85)]);
        f.section["citations"] = json!([{
            "claim_id":Uuid::from_u128(85), "citation_id":Uuid::from_u128(86),
            "claim_position":1, "citation_position":1
        }]);
        f.set_sections(vec![f.section.clone()]);
        assert_eq!(f.admit(), Err(AdmissionError::CitationGroundingReference));
    }

    #[test]
    fn admission_delegates_all_planning_provenance_mismatches() {
        use crate::admission::AdmissionError;
        use nexa_domain::{
            CitationSetId, ContextPackageId, HybridRetrievalResultId, RetrievalQueryId,
        };
        for mutation in 0..6 {
            let mut f = admission_fixture();
            match mutation {
                0 => f.authority.scope.student_id = id(90, StudentId::new),
                1 => f.authority.context_package_id = id(91, ContextPackageId::new),
                2 => f.authority.citation_set_id = id(92, CitationSetId::new),
                3 => f.authority.hybrid_result_id = id(93, HybridRetrievalResultId::new),
                4 => f.authority.query_id = id(94, RetrievalQueryId::new),
                _ => f.authority.governance_policy_version = ProtocolVersion::new(2, 0),
            }
            let expected = if mutation == 0 || mutation == 5 {
                if mutation == 5 {
                    AdmissionError::UnsupportedVersion
                } else {
                    AdmissionError::PlanningEvidenceProvenance
                }
            } else {
                AdmissionError::PlanningEvidenceProvenance
            };
            assert_eq!(f.admit(), Err(expected), "mutation {mutation}");
        }
    }

    #[test]
    fn admission_raw_hash_and_standalone_result_replay_are_bound() {
        use crate::admission::{AdmissionError, AdmissionResult};
        let f = admission_fixture();
        let first = f.admit().unwrap();
        let mut changed = admission_fixture();
        changed.section["content"] = json!("A different valid candidate section.");
        changed.set_sections(vec![changed.section.clone()]);
        let second = changed.admit().unwrap();
        assert_ne!(
            first.evidence.raw_output_sha256,
            second.evidence.raw_output_sha256
        );

        let mut invalid_nested = serde_json::to_value(&first).unwrap();
        invalid_nested["response"]["sections"][0]["content"] = json!("tampered nested section");
        assert!(serde_json::from_value::<AdmissionResult>(invalid_nested).is_err());

        let mut disagreement = serde_json::to_value(&first).unwrap();
        disagreement["evidence"]["tutor_response_replay_anchor"] = json!("a".repeat(64));
        assert!(serde_json::from_value::<AdmissionResult>(disagreement).is_err());

        let mut incomplete = admission_fixture();
        incomplete.response.finish_reason = crate::model::FinishReason::OutputLimit;
        assert_eq!(incomplete.admit(), Err(AdmissionError::IncompleteOutput));
    }

    #[test]
    fn admission_privacy_and_provider_non_consumption_are_explicit() {
        use crate::admission::AdmissionError;
        use crate::model::{LanguageModelProvider, ScriptedModelProvider, ScriptedOutcome};
        let f = admission_fixture();
        let provider = ScriptedModelProvider::new(
            f.descriptor.clone(),
            [ScriptedOutcome::Response(f.response.clone())],
        )
        .unwrap();
        assert!(f.admit().is_ok());
        assert_eq!(provider.remaining(), 1);
        assert!(provider.generate(&f.request).is_ok());

        let mut f = admission_fixture();
        let secret = "distinctive-raw-output-secret";
        f.response.output =
            crate::model::RawModelOutput::new(format!("{{\"unknown\":\"{secret}\"}}")).unwrap();
        let error = f.admit().unwrap_err();
        assert_eq!(error, AdmissionError::InvalidCandidateSchema);
        for diagnostic in [
            format!("{error}"),
            format!("{error:?}"),
            format!("{:?}", f.response),
            format!("{:?}", f.request),
            format!("{:?}", f.authority),
        ] {
            assert!(!diagnostic.contains(secret));
            assert!(!diagnostic.contains("distinctive private platform prompt"));
            assert!(!diagnostic.contains("distinctive private learner prompt"));
            assert!(!diagnostic.contains("Caller supplied explanation"));
            assert!(!diagnostic.contains("a\"}"));
        }
        let serde_error = serde_json::from_str::<crate::admission::AdmissionResult>(secret)
            .unwrap_err()
            .to_string();
        assert!(!serde_error.contains(secret));
    }

    #[test]
    fn invocation_admission_is_single_attempt_and_matches_direct_admission() {
        use crate::generation::invoke_and_admit_model_output;
        use crate::model::{ScriptedModelProvider, ScriptedOutcome};
        let f = admission_fixture();
        let expected = f.admit().unwrap();
        let provider = ScriptedModelProvider::new(
            f.descriptor.clone(),
            [
                ScriptedOutcome::Response(f.response.clone()),
                ScriptedOutcome::Error(crate::model::ModelErrorKind::Internal),
            ],
        )
        .unwrap();
        let actual = invoke_and_admit_model_output(
            &provider,
            &f.request,
            &f.compilation,
            &f.authority,
            &f.context,
            &f.citations,
        )
        .unwrap();
        assert_eq!(actual, expected);
        assert_eq!(provider.remaining(), 1);
        assert_eq!(actual.evidence.provider_id, f.request.provider_id);
        assert_eq!(actual.evidence.model_id, f.request.model_id);
        assert_eq!(actual.evidence.invocation_id, f.request.invocation_id);
        assert_eq!(
            actual.evidence.prompt_compilation_replay_anchor,
            f.compilation.replay_anchor
        );
        assert_eq!(actual.evidence.raw_output_sha256.len(), 64);
        assert_eq!(
            actual.evidence.tutor_response_replay_anchor,
            actual.response.replay_anchor
        );
        assert_eq!(actual.evidence.admission_replay_anchor.len(), 64);
    }

    #[test]
    fn invocation_admission_preflight_failures_do_not_consume_outcomes() {
        use crate::admission::AdmissionError;
        use crate::generation::{
            invoke_and_admit_model_output,
            tokenize_invoke_and_admit_model_output_with_token_capacity,
            tokenize_invoke_validate_reported_usage_and_admit_model_output_with_token_capacity,
            InvocationAdmissionError, TokenizedInvocationAdmissionError,
            UsageValidatedTokenizedInvocationAdmissionError,
        };
        use crate::model::ModelInput;
        use crate::tokenization::{
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome, MODEL_INPUT_TOKENIZATION_V1,
        };
        use nexa_domain::{
            CitationSetId, ContextPackageId, HybridRetrievalResultId, ModelId, ModelProviderId,
            RetrievalQueryId, StudentId,
        };

        // Each index isolates one host-controlled preflight class. The counting provider is
        // intentionally non-validating so invalid descriptors can reach coordinator preflight.
        for mutation in 0..33 {
            let mut f = admission_fixture();
            let tokenizer_descriptor = f.descriptor.clone();
            let expected = match mutation {
                0 => {
                    f.descriptor.contract_version = ProtocolVersion::new(2, 0);
                    AdmissionError::InvalidDescriptorRequest
                }
                1 => {
                    f.descriptor.capabilities.streaming = true;
                    AdmissionError::InvalidDescriptorRequest
                }
                2 => {
                    f.descriptor.capabilities.structured_output = false;
                    AdmissionError::UnsupportedStructuredOutput
                }
                3 => {
                    f.request.contract_version = ProtocolVersion::new(2, 0);
                    AdmissionError::InvalidDescriptorRequest
                }
                4 => {
                    f.request.provider_id = id(900, ModelProviderId::new);
                    AdmissionError::InvalidDescriptorRequest
                }
                5 => {
                    f.request.model_id = id(901, ModelId::new);
                    AdmissionError::InvalidDescriptorRequest
                }
                6 => {
                    f.request.required_capabilities.structured_output = false;
                    AdmissionError::UnsupportedStructuredOutput
                }
                7 => {
                    f.request.required_capabilities.tool_calling = true;
                    AdmissionError::InvalidDescriptorRequest
                }
                8 => {
                    f.request.maximum_output_tokens =
                        f.descriptor.capabilities.maximum_output_tokens + 1;
                    AdmissionError::InvalidDescriptorRequest
                }
                9 => {
                    f.descriptor.capabilities.context_window_tokens =
                        f.request.input.as_str().len() as u32;
                    AdmissionError::InvalidDescriptorRequest
                }
                10..=13 => {
                    match mutation {
                        10 => f.compilation.contract_version = ProtocolVersion::new(2, 0),
                        11 => f.compilation.prompt_package_version = ProtocolVersion::new(2, 0),
                        12 => f.compilation.context_builder_version = ProtocolVersion::new(2, 0),
                        _ => f.compilation.output_schema_version = ProtocolVersion::new(2, 0),
                    }
                    AdmissionError::UnsupportedVersion
                }
                14..=17 => {
                    match mutation {
                        14 => f.compilation.manifest[0].content_bytes += 1,
                        15 => f.compilation.compiled_bytes += 1,
                        16 => f.compilation.replay_anchor = "a".repeat(64),
                        _ => f.compilation.model_input = ModelInput::new("tampered input").unwrap(),
                    }
                    AdmissionError::PromptAssociationReplayMismatch
                }
                18 => {
                    f.request.input = ModelInput::new("exact input mismatch").unwrap();
                    AdmissionError::PromptAssociationReplayMismatch
                }
                19 => {
                    f.authority.response_policy_version = ProtocolVersion::new(2, 0);
                    AdmissionError::UnsupportedVersion
                }
                20 => {
                    f.authority.permitted_capabilities.clear();
                    AdmissionError::PolicyPedagogySafetyCapability
                }
                21 => {
                    f.authority.evidence.scope.student_id = id(902, StudentId::new);
                    AdmissionError::PlanningEvidenceProvenance
                }
                22 => {
                    f.authority.limits.maximum_sections = 0;
                    AdmissionError::PlanningEvidenceProvenance
                }
                23 => {
                    f.authority.context_package_id = id(903, ContextPackageId::new);
                    AdmissionError::PlanningEvidenceProvenance
                }
                24 => {
                    f.authority.citation_set_id = id(904, CitationSetId::new);
                    AdmissionError::PlanningEvidenceProvenance
                }
                25 => {
                    f.authority.hybrid_result_id = id(905, HybridRetrievalResultId::new);
                    AdmissionError::PlanningEvidenceProvenance
                }
                26 => {
                    f.authority.query_id = id(906, RetrievalQueryId::new);
                    AdmissionError::PlanningEvidenceProvenance
                }
                27 => {
                    f.authority.governance_policy_version = ProtocolVersion::new(2, 0);
                    AdmissionError::UnsupportedVersion
                }
                28 => {
                    f.authority.citation_policy_version = ProtocolVersion::new(2, 0);
                    AdmissionError::UnsupportedVersion
                }
                29 => {
                    f.context.governance_policy_version = ProtocolVersion::new(2, 0);
                    AdmissionError::PlanningEvidenceProvenance
                }
                30 => {
                    f.citations.citation_policy_version = ProtocolVersion::new(2, 0);
                    AdmissionError::CitationGroundingReference
                }
                31 => {
                    f.context.maximum_tokens = 0;
                    AdmissionError::PlanningEvidenceProvenance
                }
                _ => {
                    f.citations.maximum_citations = 0;
                    AdmissionError::CitationGroundingReference
                }
            };
            // Governance and citation policy versions are fixed to V1 by both evidence
            // contracts, so a differing value is intrinsically invalid before association.
            let provider = CountingProvider::new(&f);
            assert_eq!(
                invoke_and_admit_model_output(
                    &provider,
                    &f.request,
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                ),
                Err(InvocationAdmissionError::Preflight(expected)),
                "mutation {mutation}"
            );
            assert_eq!(provider.calls(), 0, "mutation {mutation}");

            let tokenizer = ScriptedModelInputTokenizer::new(
                tokenizer_descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(1)],
            )
            .unwrap();
            let provider = CountingProvider::new(&f);
            assert_eq!(
                tokenize_invoke_and_admit_model_output_with_token_capacity(
                    MODEL_INPUT_TOKENIZATION_V1,
                    &tokenizer,
                    &provider,
                    &f.request,
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                ),
                Err(TokenizedInvocationAdmissionError::Preflight(expected)),
                "tokenized mutation {mutation}"
            );
            assert_eq!(tokenizer.remaining().unwrap(), 1, "mutation {mutation}");
            assert_eq!(provider.calls(), 0, "mutation {mutation}");

            let tokenizer = ScriptedModelInputTokenizer::new(
                tokenizer_descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(1)],
            )
            .unwrap();
            let provider = CountingProvider::new(&f);
            assert_eq!(
                tokenize_invoke_validate_reported_usage_and_admit_model_output_with_token_capacity(
                    MODEL_INPUT_TOKENIZATION_V1,
                    &tokenizer,
                    &provider,
                    &f.request,
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                ),
                Err(UsageValidatedTokenizedInvocationAdmissionError::Preflight(
                    expected
                )),
                "usage-validated mutation {mutation}"
            );
            assert_eq!(tokenizer.remaining().unwrap(), 1, "mutation {mutation}");
            assert_eq!(provider.calls(), 0, "mutation {mutation}");
        }
    }

    #[test]
    fn invocation_errors_consume_one_outcome_and_preserve_closed_kind() {
        use crate::generation::{invoke_and_admit_model_output, InvocationAdmissionError};
        use crate::model::{ModelErrorKind, ScriptedModelProvider, ScriptedOutcome};
        for kind in [
            ModelErrorKind::Timeout,
            ModelErrorKind::Unavailable,
            ModelErrorKind::RateLimited,
            ModelErrorKind::Internal,
        ] {
            let f = admission_fixture();
            let provider = ScriptedModelProvider::new(
                f.descriptor.clone(),
                [
                    ScriptedOutcome::Error(kind),
                    ScriptedOutcome::Error(ModelErrorKind::Internal),
                ],
            )
            .unwrap();
            assert_eq!(
                invoke_and_admit_model_output(
                    &provider,
                    &f.request,
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                ),
                Err(InvocationAdmissionError::Invocation(kind))
            );
            assert_eq!(provider.remaining(), 1);
        }
    }

    fn tokenization_evidence(
        fixture: &AdmissionFixture,
        count: u32,
    ) -> crate::tokenization::ModelInputTokenizationEvidence {
        use crate::tokenization::{
            tokenize_model_input, ScriptedModelInputTokenizer, ScriptedTokenizationOutcome,
            MODEL_INPUT_TOKENIZATION_V1,
        };

        let tokenizer = ScriptedModelInputTokenizer::new(
            fixture.descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(count)],
        )
        .unwrap();
        tokenize_model_input(
            MODEL_INPUT_TOKENIZATION_V1,
            &fixture.descriptor,
            &fixture.request.input,
            &tokenizer,
        )
        .unwrap()
    }

    #[test]
    fn token_capacity_composition_validates_before_provider_consumption() {
        use crate::admission::AdmissionError;
        use crate::generation::{
            invoke_and_admit_model_output_with_token_capacity,
            TokenCapacityInvocationAdmissionError,
        };
        use crate::tokenization::ModelRequestTokenCapacityError;

        let mut preflight = admission_fixture();
        let evidence = tokenization_evidence(&preflight, 1);
        preflight.compilation.replay_anchor = "a".repeat(64);
        let provider = CountingProvider::new(&preflight);
        assert_eq!(
            invoke_and_admit_model_output_with_token_capacity(
                &provider,
                &preflight.request,
                &evidence,
                &preflight.compilation,
                &preflight.authority,
                &preflight.context,
                &preflight.citations,
            ),
            Err(TokenCapacityInvocationAdmissionError::Preflight(
                AdmissionError::PromptAssociationReplayMismatch
            ))
        );
        assert_eq!(provider.calls(), 0);

        for count in [
            preflight.descriptor.capabilities.context_window_tokens,
            u32::MAX,
        ] {
            let fixture = admission_fixture();
            let evidence = tokenization_evidence(&fixture, count);
            let provider = CountingProvider::new(&fixture);
            assert_eq!(
                invoke_and_admit_model_output_with_token_capacity(
                    &provider,
                    &fixture.request,
                    &evidence,
                    &fixture.compilation,
                    &fixture.authority,
                    &fixture.context,
                    &fixture.citations,
                ),
                Err(TokenCapacityInvocationAdmissionError::TokenCapacity(
                    ModelRequestTokenCapacityError::ExactCapacity
                ))
            );
            assert_eq!(provider.calls(), 0);
        }

        let fixture = admission_fixture();
        let mut other = admission_fixture();
        other.descriptor.model_id = id(9_990, nexa_domain::ModelId::new);
        other.request.model_id = other.descriptor.model_id;
        let mismatched = tokenization_evidence(&other, 1);
        let provider = CountingProvider::new(&fixture);
        assert!(matches!(
            invoke_and_admit_model_output_with_token_capacity(
                &provider,
                &fixture.request,
                &mismatched,
                &fixture.compilation,
                &fixture.authority,
                &fixture.context,
                &fixture.citations,
            ),
            Err(TokenCapacityInvocationAdmissionError::TokenCapacity(
                ModelRequestTokenCapacityError::TokenizationEvidence(_)
            ))
        ));
        assert_eq!(provider.calls(), 0);
    }

    #[test]
    fn token_capacity_composition_equality_invokes_once_and_reuses_admission() {
        use crate::generation::invoke_and_admit_model_output_with_token_capacity;

        let fixture = admission_fixture();
        let exact_input_tokens = fixture.descriptor.capabilities.context_window_tokens
            - fixture.request.maximum_output_tokens;
        let evidence = tokenization_evidence(&fixture, exact_input_tokens);
        let provider = CountingProvider::new(&fixture);
        let result = invoke_and_admit_model_output_with_token_capacity(
            &provider,
            &fixture.request,
            &evidence,
            &fixture.compilation,
            &fixture.authority,
            &fixture.context,
            &fixture.citations,
        )
        .unwrap();
        assert_eq!(provider.calls(), 1);
        assert_eq!(result, fixture.admit().unwrap());
    }

    #[test]
    fn token_capacity_composition_preserves_provider_and_admission_failures() {
        use crate::admission::AdmissionError;
        use crate::generation::{
            invoke_and_admit_model_output_with_token_capacity,
            TokenCapacityInvocationAdmissionError,
        };
        use crate::model::{
            ModelErrorKind, RawModelOutput, ScriptedModelProvider, ScriptedOutcome,
        };

        let fixture = admission_fixture();
        let evidence = tokenization_evidence(&fixture, 1);
        let provider = ScriptedModelProvider::new(
            fixture.descriptor.clone(),
            [
                ScriptedOutcome::Error(ModelErrorKind::Unavailable),
                ScriptedOutcome::Error(ModelErrorKind::Internal),
            ],
        )
        .unwrap();
        assert_eq!(
            invoke_and_admit_model_output_with_token_capacity(
                &provider,
                &fixture.request,
                &evidence,
                &fixture.compilation,
                &fixture.authority,
                &fixture.context,
                &fixture.citations,
            ),
            Err(TokenCapacityInvocationAdmissionError::Invocation(
                ModelErrorKind::Unavailable
            ))
        );
        assert_eq!(provider.remaining(), 1);

        let mut fixture = admission_fixture();
        fixture.response.output = RawModelOutput::new("not json").unwrap();
        let evidence = tokenization_evidence(&fixture, 1);
        let provider = CountingProvider::new(&fixture);
        assert_eq!(
            invoke_and_admit_model_output_with_token_capacity(
                &provider,
                &fixture.request,
                &evidence,
                &fixture.compilation,
                &fixture.authority,
                &fixture.context,
                &fixture.citations,
            ),
            Err(TokenCapacityInvocationAdmissionError::Admission(
                AdmissionError::MalformedSyntax
            ))
        );
        assert_eq!(provider.calls(), 1);
    }

    #[test]
    fn token_capacity_composition_diagnostics_are_content_free() {
        use crate::generation::{
            invoke_and_admit_model_output_with_token_capacity,
            TokenCapacityInvocationAdmissionError,
        };
        use crate::model::{
            LanguageModelProvider, ModelError, ModelErrorKind, ModelInput, ModelRequest,
            ModelResponse, RawModelOutput,
        };

        struct PrivateDiagnosticProvider {
            descriptor: crate::model::ModelDescriptor,
            private_diagnostic: String,
        }

        impl LanguageModelProvider for PrivateDiagnosticProvider {
            fn descriptor(&self) -> &crate::model::ModelDescriptor {
                &self.descriptor
            }

            fn generate(&self, _request: &ModelRequest) -> Result<ModelResponse, ModelError> {
                assert!(!self.private_diagnostic.is_empty());
                Err(ModelError::new(ModelErrorKind::Internal))
            }
        }

        let prompt_secret = "capacity-composition-prompt-private-sentinel";
        let mut preflight = admission_fixture();
        preflight.request.input = ModelInput::new(prompt_secret).unwrap();
        let evidence = tokenization_evidence(&preflight, 1);
        let preflight_provider = CountingProvider::new(&preflight);
        let preflight_error = invoke_and_admit_model_output_with_token_capacity(
            &preflight_provider,
            &preflight.request,
            &evidence,
            &preflight.compilation,
            &preflight.authority,
            &preflight.context,
            &preflight.citations,
        )
        .unwrap_err();
        assert!(matches!(
            preflight_error,
            TokenCapacityInvocationAdmissionError::Preflight(_)
        ));

        let capacity = admission_fixture();
        let evidence = tokenization_evidence(
            &capacity,
            capacity.descriptor.capabilities.context_window_tokens,
        );
        let capacity_provider = CountingProvider::new(&capacity);
        let capacity_error = invoke_and_admit_model_output_with_token_capacity(
            &capacity_provider,
            &capacity.request,
            &evidence,
            &capacity.compilation,
            &capacity.authority,
            &capacity.context,
            &capacity.citations,
        )
        .unwrap_err();
        assert!(matches!(
            capacity_error,
            TokenCapacityInvocationAdmissionError::TokenCapacity(_)
        ));

        let provider_secret = "capacity-composition-provider-diagnostic-private-sentinel";
        let invocation = admission_fixture();
        let evidence = tokenization_evidence(&invocation, 1);
        let invocation_provider = PrivateDiagnosticProvider {
            descriptor: invocation.descriptor.clone(),
            private_diagnostic: provider_secret.into(),
        };
        let invocation_error = invoke_and_admit_model_output_with_token_capacity(
            &invocation_provider,
            &invocation.request,
            &evidence,
            &invocation.compilation,
            &invocation.authority,
            &invocation.context,
            &invocation.citations,
        )
        .unwrap_err();
        assert_eq!(
            invocation_error,
            TokenCapacityInvocationAdmissionError::Invocation(ModelErrorKind::Internal)
        );

        let output_secret = "capacity-composition-model-output-private-sentinel";
        let mut admission = admission_fixture();
        admission.response.output = RawModelOutput::new(output_secret).unwrap();
        let evidence = tokenization_evidence(&admission, 1);
        let admission_provider = CountingProvider::new(&admission);
        let admission_error = invoke_and_admit_model_output_with_token_capacity(
            &admission_provider,
            &admission.request,
            &evidence,
            &admission.compilation,
            &admission.authority,
            &admission.context,
            &admission.citations,
        )
        .unwrap_err();
        assert!(matches!(
            admission_error,
            TokenCapacityInvocationAdmissionError::Admission(_)
        ));

        for error in [
            preflight_error,
            capacity_error,
            invocation_error,
            admission_error,
        ] {
            let debug = format!("{error:?}");
            let display = format!("{error}");
            for sentinel in [prompt_secret, output_secret, provider_secret] {
                assert!(!debug.contains(sentinel));
                assert!(!display.contains(sentinel));
            }
        }
        assert_eq!(preflight_provider.calls(), 0);
        assert_eq!(capacity_provider.calls(), 0);
        assert_eq!(admission_provider.calls(), 1);
    }

    #[test]
    fn tokenized_invocation_orders_counting_capacity_invocation_and_admission() {
        use crate::admission::AdmissionError;
        use crate::generation::{
            tokenize_invoke_and_admit_model_output_with_token_capacity,
            TokenizedInvocationAdmissionError,
        };
        use crate::model::{
            ModelErrorKind, RawModelOutput, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::tokenization::{
            ModelInputTokenizationError, ModelRequestTokenCapacityError,
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome,
            TokenizeAndValidateModelRequestCapacityError, MODEL_INPUT_TOKENIZATION_V1,
        };

        for outcome in [
            ScriptedTokenizationOutcome::Error,
            ScriptedTokenizationOutcome::TokenCount(0),
        ] {
            let fixture = admission_fixture();
            let tokenizer =
                ScriptedModelInputTokenizer::new(fixture.descriptor.clone(), [outcome]).unwrap();
            let provider = CountingProvider::new(&fixture);
            assert!(matches!(
                tokenize_invoke_and_admit_model_output_with_token_capacity(
                    MODEL_INPUT_TOKENIZATION_V1,
                    &tokenizer,
                    &provider,
                    &fixture.request,
                    &fixture.compilation,
                    &fixture.authority,
                    &fixture.context,
                    &fixture.citations,
                ),
                Err(TokenizedInvocationAdmissionError::TokenizationCapacity(
                    TokenizeAndValidateModelRequestCapacityError::Tokenization(_)
                ))
            ));
            assert_eq!(tokenizer.remaining().unwrap(), 0);
            assert_eq!(provider.calls(), 0);
        }

        let fixture = admission_fixture();
        let tokenizer = ScriptedModelInputTokenizer::new(fixture.descriptor.clone(), []).unwrap();
        let provider = CountingProvider::new(&fixture);
        assert_eq!(
            tokenize_invoke_and_admit_model_output_with_token_capacity(
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &provider,
                &fixture.request,
                &fixture.compilation,
                &fixture.authority,
                &fixture.context,
                &fixture.citations,
            ),
            Err(TokenizedInvocationAdmissionError::TokenizationCapacity(
                TokenizeAndValidateModelRequestCapacityError::Tokenization(
                    ModelInputTokenizationError::ScriptExhausted
                )
            ))
        );
        assert_eq!(provider.calls(), 0);

        for count in [
            fixture.descriptor.capabilities.context_window_tokens,
            u32::MAX,
        ] {
            let fixture = admission_fixture();
            let tokenizer = ScriptedModelInputTokenizer::new(
                fixture.descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(count)],
            )
            .unwrap();
            let provider = CountingProvider::new(&fixture);
            assert_eq!(
                tokenize_invoke_and_admit_model_output_with_token_capacity(
                    MODEL_INPUT_TOKENIZATION_V1,
                    &tokenizer,
                    &provider,
                    &fixture.request,
                    &fixture.compilation,
                    &fixture.authority,
                    &fixture.context,
                    &fixture.citations,
                ),
                Err(TokenizedInvocationAdmissionError::TokenizationCapacity(
                    TokenizeAndValidateModelRequestCapacityError::TokenCapacity(
                        ModelRequestTokenCapacityError::ExactCapacity
                    )
                ))
            );
            assert_eq!(tokenizer.remaining().unwrap(), 0);
            assert_eq!(provider.calls(), 0);
        }

        for exact_boundary in [false, true] {
            let fixture = admission_fixture();
            let count = if exact_boundary {
                fixture.descriptor.capabilities.context_window_tokens
                    - fixture.request.maximum_output_tokens
            } else {
                1
            };
            let tokenizer = ScriptedModelInputTokenizer::new(
                fixture.descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(count)],
            )
            .unwrap();
            let provider = CountingProvider::new(&fixture);
            let result = tokenize_invoke_and_admit_model_output_with_token_capacity(
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &provider,
                &fixture.request,
                &fixture.compilation,
                &fixture.authority,
                &fixture.context,
                &fixture.citations,
            )
            .unwrap();
            assert_eq!(tokenizer.remaining().unwrap(), 0);
            assert_eq!(provider.calls(), 1);
            assert_eq!(result.admission, fixture.admit().unwrap());
            result
                .tokenization_evidence
                .validate_for(&fixture.descriptor, &fixture.request.input)
                .unwrap();
            let wire = serde_json::to_string(&result.tokenization_evidence).unwrap();
            assert_eq!(
                serde_json::from_str::<crate::tokenization::ModelInputTokenizationEvidence>(&wire)
                    .unwrap(),
                result.tokenization_evidence
            );
        }

        let fixture = admission_fixture();
        let tokenizer = ScriptedModelInputTokenizer::new(
            fixture.descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(1)],
        )
        .unwrap();
        let provider = ScriptedModelProvider::new(
            fixture.descriptor.clone(),
            [
                ScriptedOutcome::Error(ModelErrorKind::Unavailable),
                ScriptedOutcome::Error(ModelErrorKind::Internal),
            ],
        )
        .unwrap();
        assert_eq!(
            tokenize_invoke_and_admit_model_output_with_token_capacity(
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &provider,
                &fixture.request,
                &fixture.compilation,
                &fixture.authority,
                &fixture.context,
                &fixture.citations,
            ),
            Err(TokenizedInvocationAdmissionError::Invocation(
                ModelErrorKind::Unavailable
            ))
        );
        assert_eq!(tokenizer.remaining().unwrap(), 0);
        assert_eq!(provider.remaining(), 1);

        let mut fixture = admission_fixture();
        fixture.response.output = RawModelOutput::new("not json").unwrap();
        let tokenizer = ScriptedModelInputTokenizer::new(
            fixture.descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(1)],
        )
        .unwrap();
        let provider = CountingProvider::new(&fixture);
        assert_eq!(
            tokenize_invoke_and_admit_model_output_with_token_capacity(
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &provider,
                &fixture.request,
                &fixture.compilation,
                &fixture.authority,
                &fixture.context,
                &fixture.citations,
            ),
            Err(TokenizedInvocationAdmissionError::Admission(
                AdmissionError::MalformedSyntax
            ))
        );
        assert_eq!(tokenizer.remaining().unwrap(), 0);
        assert_eq!(provider.calls(), 1);
    }

    #[test]
    fn tokenized_invocation_tokenizer_preflight_and_diagnostics_are_closed() {
        use crate::generation::{
            tokenize_invoke_and_admit_model_output_with_token_capacity,
            TokenizedInvocationAdmissionError,
        };
        use crate::model::{LanguageModelProvider, ModelError, ModelErrorKind, ModelRequest};
        use crate::tokenization::{
            ModelInputTokenizationError, ModelInputTokenizer, ScriptedModelInputTokenizer,
            ScriptedTokenizationOutcome, MODEL_INPUT_TOKENIZATION_V1,
        };

        struct InternalTokenizer {
            descriptor: crate::model::ModelDescriptor,
            private: &'static str,
        }
        impl ModelInputTokenizer for InternalTokenizer {
            fn descriptor(&self) -> &crate::model::ModelDescriptor {
                &self.descriptor
            }
            fn count_input_tokens(
                &self,
                _input: &crate::model::ModelInput,
            ) -> Result<u32, ModelInputTokenizationError> {
                assert!(!self.private.is_empty());
                Err(ModelInputTokenizationError::Internal)
            }
        }
        struct PrivateProvider {
            descriptor: crate::model::ModelDescriptor,
            private: &'static str,
        }
        impl LanguageModelProvider for PrivateProvider {
            fn descriptor(&self) -> &crate::model::ModelDescriptor {
                &self.descriptor
            }
            fn generate(
                &self,
                _request: &ModelRequest,
            ) -> Result<crate::model::ModelResponse, ModelError> {
                assert!(!self.private.is_empty());
                Err(ModelError::new(ModelErrorKind::Internal))
            }
        }

        let fixture = admission_fixture();
        let tokenizer = ScriptedModelInputTokenizer::new(
            fixture.descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(1)],
        )
        .unwrap();
        let provider = CountingProvider::new(&fixture);
        assert!(matches!(
            tokenize_invoke_and_admit_model_output_with_token_capacity(
                ProtocolVersion::new(2, 0),
                &tokenizer,
                &provider,
                &fixture.request,
                &fixture.compilation,
                &fixture.authority,
                &fixture.context,
                &fixture.citations,
            ),
            Err(TokenizedInvocationAdmissionError::TokenizationCapacity(_))
        ));
        assert_eq!(tokenizer.remaining().unwrap(), 1);
        assert_eq!(provider.calls(), 0);

        let mut other = admission_fixture();
        other.descriptor.model_id = id(9_991, nexa_domain::ModelId::new);
        let tokenizer = ScriptedModelInputTokenizer::new(
            other.descriptor,
            [ScriptedTokenizationOutcome::TokenCount(1)],
        )
        .unwrap();
        let provider = CountingProvider::new(&fixture);
        assert!(matches!(
            tokenize_invoke_and_admit_model_output_with_token_capacity(
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &provider,
                &fixture.request,
                &fixture.compilation,
                &fixture.authority,
                &fixture.context,
                &fixture.citations,
            ),
            Err(TokenizedInvocationAdmissionError::TokenizationCapacity(_))
        ));
        assert_eq!(tokenizer.remaining().unwrap(), 1);
        assert_eq!(provider.calls(), 0);

        let sentinels = [
            "prompt-private-sentinel",
            "learner-private-sentinel",
            "knowledge-private-sentinel",
            "output-private-sentinel",
            "credential-private-sentinel",
            "endpoint-private-sentinel",
            "tokenizer-private-sentinel",
            "provider-private-sentinel",
        ];

        let mut mismatched = admission_fixture();
        mismatched.descriptor.model_id = id(9_992, nexa_domain::ModelId::new);
        let tokenizer = ScriptedModelInputTokenizer::new(
            fixture.descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(1)],
        )
        .unwrap();
        let provider = CountingProvider::new(&mismatched);
        let preflight_error = tokenize_invoke_and_admit_model_output_with_token_capacity(
            MODEL_INPUT_TOKENIZATION_V1,
            &tokenizer,
            &provider,
            &fixture.request,
            &fixture.compilation,
            &fixture.authority,
            &fixture.context,
            &fixture.citations,
        )
        .unwrap_err();
        assert!(matches!(
            preflight_error,
            TokenizedInvocationAdmissionError::Preflight(_)
        ));
        assert_eq!(tokenizer.remaining().unwrap(), 1);
        assert_eq!(provider.calls(), 0);

        let tokenizer = InternalTokenizer {
            descriptor: fixture.descriptor.clone(),
            private: sentinels[6],
        };
        let provider = CountingProvider::new(&fixture);
        let tokenization_error = tokenize_invoke_and_admit_model_output_with_token_capacity(
            MODEL_INPUT_TOKENIZATION_V1,
            &tokenizer,
            &provider,
            &fixture.request,
            &fixture.compilation,
            &fixture.authority,
            &fixture.context,
            &fixture.citations,
        )
        .unwrap_err();
        assert_eq!(provider.calls(), 0);

        let tokenizer = ScriptedModelInputTokenizer::new(
            fixture.descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(1)],
        )
        .unwrap();
        let provider = PrivateProvider {
            descriptor: fixture.descriptor.clone(),
            private: sentinels[7],
        };
        let invocation_error = tokenize_invoke_and_admit_model_output_with_token_capacity(
            MODEL_INPUT_TOKENIZATION_V1,
            &tokenizer,
            &provider,
            &fixture.request,
            &fixture.compilation,
            &fixture.authority,
            &fixture.context,
            &fixture.citations,
        )
        .unwrap_err();

        let mut malformed = admission_fixture();
        malformed.response.output = crate::model::RawModelOutput::new(sentinels[3]).unwrap();
        let tokenizer = ScriptedModelInputTokenizer::new(
            malformed.descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(1)],
        )
        .unwrap();
        let provider = CountingProvider::new(&malformed);
        let admission_error = tokenize_invoke_and_admit_model_output_with_token_capacity(
            MODEL_INPUT_TOKENIZATION_V1,
            &tokenizer,
            &provider,
            &malformed.request,
            &malformed.compilation,
            &malformed.authority,
            &malformed.context,
            &malformed.citations,
        )
        .unwrap_err();
        assert!(matches!(
            admission_error,
            TokenizedInvocationAdmissionError::Admission(_)
        ));
        assert_eq!(provider.calls(), 1);

        for error in [
            preflight_error,
            tokenization_error,
            invocation_error,
            admission_error,
        ] {
            let diagnostics = format!("{error:?} {error}");
            for sentinel in sentinels {
                assert!(!diagnostics.contains(sentinel));
            }
        }
    }

    #[test]
    fn usage_validated_tokenized_invocation_enforces_order_and_exact_usage() {
        use crate::generation::{
            tokenize_invoke_and_admit_model_output_with_token_capacity,
            tokenize_invoke_validate_reported_usage_and_admit_model_output_with_token_capacity,
            UsageValidatedTokenizedInvocationAdmissionError as Error,
        };
        use crate::model::{
            ModelErrorKind, ModelUsage, RawModelOutput, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::tokenization::{
            ModelInputTokenizationError, ModelRequestTokenCapacityError,
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome,
            TokenizeAndValidateModelRequestCapacityError, MODEL_INPUT_TOKENIZATION_V1,
        };
        use crate::usage::ModelResponseReportedUsageValidationError as UsageError;

        let invoke = |fixture: &AdmissionFixture, count, outcomes| {
            let tokenizer = ScriptedModelInputTokenizer::new(
                fixture.descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(count)],
            )
            .unwrap();
            let provider =
                ScriptedModelProvider::new(fixture.descriptor.clone(), outcomes).unwrap();
            let result =
                tokenize_invoke_validate_reported_usage_and_admit_model_output_with_token_capacity(
                    MODEL_INPUT_TOKENIZATION_V1,
                    &tokenizer,
                    &provider,
                    &fixture.request,
                    &fixture.compilation,
                    &fixture.authority,
                    &fixture.context,
                    &fixture.citations,
                );
            (result, tokenizer.remaining().unwrap(), provider.remaining())
        };

        for reported_usage in [
            None,
            Some(ModelUsage {
                input_tokens: 7,
                output_tokens: 1,
            }),
        ] {
            let mut fixture = admission_fixture();
            fixture.response.reported_usage = reported_usage;
            let expected_admission = fixture.admit().unwrap();
            let direct_tokenizer = ScriptedModelInputTokenizer::new(
                fixture.descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(7)],
            )
            .unwrap();
            let direct_provider = ScriptedModelProvider::new(
                fixture.descriptor.clone(),
                [ScriptedOutcome::Response(fixture.response.clone())],
            )
            .unwrap();
            let direct = tokenize_invoke_and_admit_model_output_with_token_capacity(
                MODEL_INPUT_TOKENIZATION_V1,
                &direct_tokenizer,
                &direct_provider,
                &fixture.request,
                &fixture.compilation,
                &fixture.authority,
                &fixture.context,
                &fixture.citations,
            )
            .unwrap();
            let (result, tokenizer_remaining, provider_remaining) = invoke(
                &fixture,
                7,
                vec![ScriptedOutcome::Response(fixture.response.clone())],
            );
            let result = result.unwrap();
            assert_eq!(result.tokenization_evidence, direct.tokenization_evidence);
            assert_eq!(result.admission, expected_admission);
            assert_eq!(result.admission, direct.admission);
            assert_eq!((tokenizer_remaining, provider_remaining), (0, 0));
            assert_eq!(direct_tokenizer.remaining().unwrap(), 0);
            assert_eq!(direct_provider.remaining(), 0);
        }

        for reported in [6, 8] {
            let mut fixture = admission_fixture();
            fixture.response.reported_usage = Some(ModelUsage {
                input_tokens: reported,
                output_tokens: 1,
            });
            let (result, tokenizer_remaining, provider_remaining) = invoke(
                &fixture,
                7,
                vec![ScriptedOutcome::Response(fixture.response.clone())],
            );
            assert_eq!(
                result,
                Err(Error::ReportedUsage(UsageError::InputTokenCountMismatch))
            );
            assert_eq!((tokenizer_remaining, provider_remaining), (0, 0));
        }

        let fixture = admission_fixture();
        let (result, tokenizer_remaining, provider_remaining) = invoke(
            &fixture,
            fixture.descriptor.capabilities.context_window_tokens,
            vec![ScriptedOutcome::Response(fixture.response.clone())],
        );
        assert_eq!(
            result,
            Err(Error::TokenizationCapacity(
                TokenizeAndValidateModelRequestCapacityError::TokenCapacity(
                    ModelRequestTokenCapacityError::ExactCapacity
                )
            ))
        );
        assert_eq!((tokenizer_remaining, provider_remaining), (0, 1));

        // Every failure reachable while producing exact tokenization evidence retains its
        // unchanged nested error and cannot consume a provider outcome.
        let fixture = admission_fixture();
        let cases = [
            (
                ProtocolVersion::new(2, 0),
                fixture.descriptor.clone(),
                vec![ScriptedTokenizationOutcome::TokenCount(7)],
                ModelInputTokenizationError::UnsupportedVersion,
                1,
            ),
            (
                MODEL_INPUT_TOKENIZATION_V1,
                {
                    let mut descriptor = fixture.descriptor.clone();
                    descriptor.provider_id = id(700, nexa_domain::ModelProviderId::new);
                    descriptor
                },
                vec![ScriptedTokenizationOutcome::TokenCount(7)],
                ModelInputTokenizationError::InvalidDescriptor,
                1,
            ),
            (
                MODEL_INPUT_TOKENIZATION_V1,
                fixture.descriptor.clone(),
                vec![ScriptedTokenizationOutcome::Error],
                ModelInputTokenizationError::TokenizerFailure,
                0,
            ),
            (
                MODEL_INPUT_TOKENIZATION_V1,
                fixture.descriptor.clone(),
                vec![],
                ModelInputTokenizationError::ScriptExhausted,
                0,
            ),
            (
                MODEL_INPUT_TOKENIZATION_V1,
                fixture.descriptor.clone(),
                vec![ScriptedTokenizationOutcome::TokenCount(0)],
                ModelInputTokenizationError::InvalidEvidence,
                0,
            ),
        ];
        for (version, descriptor, outcomes, expected, remaining) in cases {
            let tokenizer = ScriptedModelInputTokenizer::new(descriptor, outcomes).unwrap();
            let provider = ScriptedModelProvider::new(
                fixture.descriptor.clone(),
                [ScriptedOutcome::Response(fixture.response.clone())],
            )
            .unwrap();
            assert_eq!(
                tokenize_invoke_validate_reported_usage_and_admit_model_output_with_token_capacity(
                    version,
                    &tokenizer,
                    &provider,
                    &fixture.request,
                    &fixture.compilation,
                    &fixture.authority,
                    &fixture.context,
                    &fixture.citations,
                ),
                Err(Error::TokenizationCapacity(
                    TokenizeAndValidateModelRequestCapacityError::Tokenization(expected)
                ))
            );
            assert_eq!(tokenizer.remaining().unwrap(), remaining);
            assert_eq!(provider.remaining(), 1);
        }

        let fixture = admission_fixture();
        let (result, tokenizer_remaining, provider_remaining) = invoke(
            &fixture,
            7,
            vec![
                ScriptedOutcome::Error(ModelErrorKind::Unavailable),
                ScriptedOutcome::Response(fixture.response.clone()),
            ],
        );
        assert_eq!(result, Err(Error::Invocation(ModelErrorKind::Unavailable)));
        assert_eq!((tokenizer_remaining, provider_remaining), (0, 1));

        let mut fixture = admission_fixture();
        fixture.response.reported_usage = Some(ModelUsage {
            input_tokens: 7,
            output_tokens: fixture.request.maximum_output_tokens + 1,
        });
        fixture.response.output = RawModelOutput::new("not json").unwrap();
        let tokenizer = ScriptedModelInputTokenizer::new(
            fixture.descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(7)],
        )
        .unwrap();
        let provider = CountingProvider::new(&fixture);
        let result =
            tokenize_invoke_validate_reported_usage_and_admit_model_output_with_token_capacity(
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &provider,
                &fixture.request,
                &fixture.compilation,
                &fixture.authority,
                &fixture.context,
                &fixture.citations,
            );
        assert_eq!(
            result,
            Err(Error::ReportedUsage(UsageError::Response(
                ModelErrorKind::InvalidResponse
            )))
        );
        assert_eq!(tokenizer.remaining().unwrap(), 0);
        assert_eq!(provider.calls(), 1);

        for mutation in 0..2 {
            let mut fixture = admission_fixture();
            fixture.response.reported_usage = Some(ModelUsage {
                input_tokens: 6,
                output_tokens: 1,
            });
            fixture.response.output = RawModelOutput::new("not json").unwrap();
            let expected = if mutation == 0 {
                fixture.response.invocation_id = id(702, nexa_domain::ModelInvocationId::new);
                ModelErrorKind::IdentityMismatch
            } else {
                fixture.response.contract_version = ProtocolVersion::new(2, 0);
                ModelErrorKind::UnsupportedVersion
            };
            let tokenizer = ScriptedModelInputTokenizer::new(
                fixture.descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(7)],
            )
            .unwrap();
            let provider = CountingProvider::new(&fixture);
            let result =
                tokenize_invoke_validate_reported_usage_and_admit_model_output_with_token_capacity(
                    MODEL_INPUT_TOKENIZATION_V1,
                    &tokenizer,
                    &provider,
                    &fixture.request,
                    &fixture.compilation,
                    &fixture.authority,
                    &fixture.context,
                    &fixture.citations,
                );
            assert_eq!(
                result,
                Err(Error::ReportedUsage(UsageError::Response(expected)))
            );
            assert_eq!(tokenizer.remaining().unwrap(), 0);
            assert_eq!(provider.calls(), 1);
        }

        let mut fixture = admission_fixture();
        fixture.response.output = RawModelOutput::new("not json").unwrap();
        let (result, tokenizer_remaining, provider_remaining) = invoke(
            &fixture,
            7,
            vec![ScriptedOutcome::Response(fixture.response.clone())],
        );
        assert_eq!(
            result,
            Err(Error::Admission(
                crate::admission::AdmissionError::MalformedSyntax
            ))
        );
        assert_eq!((tokenizer_remaining, provider_remaining), (0, 0));
    }

    #[test]
    fn usage_validated_tokenized_invocation_error_diagnostics_are_content_free() {
        use crate::generation::{
            tokenize_invoke_validate_reported_usage_and_admit_model_output_with_token_capacity,
            UsageValidatedTokenizedInvocationAdmissionError as Error,
        };
        use crate::model::{ModelErrorKind, ModelUsage, RawModelOutput, ScriptedOutcome};
        use crate::tokenization::{
            ModelInputTokenizationError, ModelRequestTokenCapacityError,
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome,
            TokenizeAndValidateModelRequestCapacityError, MODEL_INPUT_TOKENIZATION_V1,
        };
        use crate::usage::ModelResponseReportedUsageValidationError as UsageError;

        struct DiagnosticProvider {
            inner: UncheckedScriptedProvider,
            endpoint: String,
            credential: String,
            private_diagnostic: String,
            usage_adjacent: String,
        }
        impl crate::model::LanguageModelProvider for DiagnosticProvider {
            fn descriptor(&self) -> &crate::model::ModelDescriptor {
                self.inner.descriptor()
            }

            fn generate(
                &self,
                request: &crate::model::ModelRequest,
            ) -> Result<crate::model::ModelResponse, crate::model::ModelError> {
                assert!(!self.endpoint.is_empty());
                assert!(!self.credential.is_empty());
                assert!(!self.private_diagnostic.is_empty());
                assert!(!self.usage_adjacent.is_empty());
                self.inner.generate(request)
            }
        }

        let sentinels = [
            "prompt-private-sentinel",
            "learner-sentinel",
            "context-sentinel",
            "response-output-sentinel",
            "usage-adjacent-sentinel",
            "tokenizer-provider-sentinel",
            "endpoint-sentinel",
            "credential-sentinel",
        ];
        let sentinel_fixture = |mut fixture: AdmissionFixture| {
            fixture.context.tokenizer_profile_id = "context-sentinel".into();
            fixture.response.output =
                RawModelOutput::new("response-output-sentinel usage-adjacent-sentinel not json")
                    .unwrap();
            assert!(fixture.request.input.as_str().contains(sentinels[0]));
            assert!(fixture.request.input.as_str().contains(sentinels[1]));
            assert!(fixture.context.tokenizer_profile_id.contains(sentinels[2]));
            assert!(fixture.response.output.as_str().contains(sentinels[3]));
            assert!(fixture.response.output.as_str().contains(sentinels[4]));
            fixture
        };
        let run = |fixture: &AdmissionFixture,
                   version,
                   descriptor,
                   token_outcomes,
                   provider_outcomes,
                   expected,
                   tokenizer_remaining,
                   provider_remaining| {
            let tokenizer = SentinelTokenizer {
                inner: ScriptedModelInputTokenizer::new(descriptor, token_outcomes).unwrap(),
                private_diagnostic: sentinels[5].into(),
            };
            let provider = DiagnosticProvider {
                inner: UncheckedScriptedProvider::new(
                    fixture.descriptor.clone(),
                    provider_outcomes,
                ),
                endpoint: sentinels[6].into(),
                credential: sentinels[7].into(),
                private_diagnostic: sentinels[5].into(),
                usage_adjacent: sentinels[4].into(),
            };
            let error =
                tokenize_invoke_validate_reported_usage_and_admit_model_output_with_token_capacity(
                    version,
                    &tokenizer,
                    &provider,
                    &fixture.request,
                    &fixture.compilation,
                    &fixture.authority,
                    &fixture.context,
                    &fixture.citations,
                )
                .unwrap_err();
            assert_eq!(error, expected);
            assert_eq!(tokenizer.inner.remaining().unwrap(), tokenizer_remaining);
            assert_eq!(provider.inner.remaining(), provider_remaining);
            for diagnostic in [format!("{error:?}"), format!("{error}")] {
                for sentinel in sentinels {
                    assert!(!diagnostic.contains(sentinel));
                }
            }
        };

        let mut preflight = sentinel_fixture(admission_fixture());
        preflight.compilation.contract_version = ProtocolVersion::new(2, 0);
        run(
            &preflight,
            MODEL_INPUT_TOKENIZATION_V1,
            preflight.descriptor.clone(),
            vec![ScriptedTokenizationOutcome::TokenCount(7)],
            vec![ScriptedOutcome::Response(preflight.response.clone())],
            Error::Preflight(crate::admission::AdmissionError::UnsupportedVersion),
            1,
            1,
        );

        let fixture = sentinel_fixture(admission_fixture());
        let tokenization_cases = [
            (
                ProtocolVersion::new(2, 0),
                fixture.descriptor.clone(),
                vec![ScriptedTokenizationOutcome::TokenCount(7)],
                TokenizeAndValidateModelRequestCapacityError::Tokenization(
                    ModelInputTokenizationError::UnsupportedVersion,
                ),
                1,
            ),
            (
                MODEL_INPUT_TOKENIZATION_V1,
                {
                    let mut descriptor = fixture.descriptor.clone();
                    descriptor.provider_id = id(703, nexa_domain::ModelProviderId::new);
                    descriptor
                },
                vec![ScriptedTokenizationOutcome::TokenCount(7)],
                TokenizeAndValidateModelRequestCapacityError::Tokenization(
                    ModelInputTokenizationError::InvalidDescriptor,
                ),
                1,
            ),
            (
                MODEL_INPUT_TOKENIZATION_V1,
                fixture.descriptor.clone(),
                vec![ScriptedTokenizationOutcome::Error],
                TokenizeAndValidateModelRequestCapacityError::Tokenization(
                    ModelInputTokenizationError::TokenizerFailure,
                ),
                0,
            ),
            (
                MODEL_INPUT_TOKENIZATION_V1,
                fixture.descriptor.clone(),
                vec![],
                TokenizeAndValidateModelRequestCapacityError::Tokenization(
                    ModelInputTokenizationError::ScriptExhausted,
                ),
                0,
            ),
            (
                MODEL_INPUT_TOKENIZATION_V1,
                fixture.descriptor.clone(),
                vec![ScriptedTokenizationOutcome::TokenCount(0)],
                TokenizeAndValidateModelRequestCapacityError::Tokenization(
                    ModelInputTokenizationError::InvalidEvidence,
                ),
                0,
            ),
            (
                MODEL_INPUT_TOKENIZATION_V1,
                fixture.descriptor.clone(),
                vec![ScriptedTokenizationOutcome::TokenCount(
                    fixture.descriptor.capabilities.context_window_tokens,
                )],
                TokenizeAndValidateModelRequestCapacityError::TokenCapacity(
                    ModelRequestTokenCapacityError::ExactCapacity,
                ),
                0,
            ),
        ];
        for (version, descriptor, outcomes, expected, remaining) in tokenization_cases {
            run(
                &fixture,
                version,
                descriptor,
                outcomes,
                vec![ScriptedOutcome::Response(fixture.response.clone())],
                Error::TokenizationCapacity(expected),
                remaining,
                1,
            );
        }

        run(
            &fixture,
            MODEL_INPUT_TOKENIZATION_V1,
            fixture.descriptor.clone(),
            vec![ScriptedTokenizationOutcome::TokenCount(7)],
            vec![ScriptedOutcome::Error(ModelErrorKind::Internal)],
            Error::Invocation(ModelErrorKind::Internal),
            0,
            0,
        );

        for mutation in 0..4 {
            let mut response_fixture = sentinel_fixture(admission_fixture());
            response_fixture.response.reported_usage = Some(ModelUsage {
                input_tokens: 6,
                output_tokens: 1,
            });
            let expected = match mutation {
                0 => {
                    response_fixture
                        .response
                        .reported_usage
                        .as_mut()
                        .unwrap()
                        .output_tokens = response_fixture.request.maximum_output_tokens + 1;
                    UsageError::Response(ModelErrorKind::InvalidResponse)
                }
                1 => {
                    response_fixture.response.invocation_id =
                        id(704, nexa_domain::ModelInvocationId::new);
                    UsageError::Response(ModelErrorKind::IdentityMismatch)
                }
                2 => {
                    response_fixture.response.contract_version = ProtocolVersion::new(2, 0);
                    UsageError::Response(ModelErrorKind::UnsupportedVersion)
                }
                _ => UsageError::InputTokenCountMismatch,
            };
            run(
                &response_fixture,
                MODEL_INPUT_TOKENIZATION_V1,
                response_fixture.descriptor.clone(),
                vec![ScriptedTokenizationOutcome::TokenCount(7)],
                vec![ScriptedOutcome::Response(response_fixture.response.clone())],
                Error::ReportedUsage(expected),
                0,
                0,
            );
        }

        run(
            &fixture,
            MODEL_INPUT_TOKENIZATION_V1,
            fixture.descriptor.clone(),
            vec![ScriptedTokenizationOutcome::TokenCount(7)],
            vec![ScriptedOutcome::Response(fixture.response.clone())],
            Error::Admission(crate::admission::AdmissionError::MalformedSyntax),
            0,
            0,
        );
    }

    #[test]
    fn post_invocation_admission_failure_consumes_one_outcome_without_retry() {
        use crate::admission::AdmissionError;
        use crate::generation::{invoke_and_admit_model_output, InvocationAdmissionError};
        use crate::model::{FinishReason, ModelUsage, RawModelOutput};
        use nexa_domain::{ModelId, ModelInvocationId};

        for mutation in 0..19 {
            let mut f = admission_fixture();
            let expected = match mutation {
                0 => {
                    f.response.output = RawModelOutput::new("not json").unwrap();
                    AdmissionError::MalformedSyntax
                }
                1 => {
                    f.response.output =
                        RawModelOutput::new("{\"candidate_schema_version\":\"1.0\",\"sections\":[")
                            .unwrap();
                    AdmissionError::MalformedSyntax
                }
                2 => {
                    f.response.output = RawModelOutput::new(
                        "{\"candidate_schema_version\":\"1.0\",\"sections\":[]} trailing",
                    )
                    .unwrap();
                    AdmissionError::MalformedSyntax
                }
                3 => {
                    f.set_candidate(
                        json!({"candidate_schema_version":"2.0", "sections":[f.section.clone()]}),
                    );
                    AdmissionError::UnsupportedVersion
                }
                4 => {
                    f.set_candidate(json!({"candidate_schema_version":"1.0", "sections":[f.section.clone()], "response_id":"model-owned"}));
                    AdmissionError::InvalidCandidateSchema
                }
                5 => {
                    f.section["unknown_section_field"] = json!(true);
                    f.set_sections(vec![f.section.clone()]);
                    AdmissionError::InvalidCandidateSchema
                }
                6 => {
                    f.authority.limits.maximum_section_bytes = 3;
                    AdmissionError::PlanningEvidenceProvenance
                }
                7 => {
                    f.authority.limits.maximum_response_bytes =
                        f.section["content"].as_str().unwrap().len();
                    let mut second = f.section.clone();
                    second["section_id"] = json!(Uuid::from_u128(910));
                    f.set_sections(vec![f.section.clone(), second]);
                    AdmissionError::PlanningEvidenceProvenance
                }
                8 => {
                    f.authority.limits.maximum_sections = 1;
                    let mut second = f.section.clone();
                    second["section_id"] = json!(Uuid::from_u128(911));
                    f.set_sections(vec![f.section.clone(), second]);
                    AdmissionError::PlanningEvidenceProvenance
                }
                9 => {
                    f.authority.limits.maximum_references_per_section = 1;
                    f.section["claims"] = json!([Uuid::from_u128(912), Uuid::from_u128(913)]);
                    f.set_sections(vec![f.section.clone()]);
                    AdmissionError::PlanningEvidenceProvenance
                }
                10 => {
                    f.section["capability"] = json!("summarize");
                    f.set_sections(vec![f.section.clone()]);
                    AdmissionError::PolicyPedagogySafetyCapability
                }
                11 => {
                    f.section["scaffolding"] = json!(9);
                    f.set_sections(vec![f.section.clone()]);
                    AdmissionError::PolicyPedagogySafetyCapability
                }
                12 => {
                    f.section["pedagogy_decision_evidence_id"] = json!(Uuid::from_u128(918));
                    f.set_sections(vec![f.section.clone()]);
                    AdmissionError::PolicyPedagogySafetyCapability
                }
                13 => {
                    f.section["assessment_restriction"] = json!("withhold_answers");
                    f.set_sections(vec![f.section.clone()]);
                    AdmissionError::PolicyPedagogySafetyCapability
                }
                14 => {
                    f.section["safety"] = json!("refusal_required");
                    f.set_sections(vec![f.section.clone()]);
                    AdmissionError::PolicyPedagogySafetyCapability
                }
                15 => {
                    f.section["claims"] = json!([Uuid::from_u128(914)]);
                    f.section["citations"] = json!([{"claim_id":Uuid::from_u128(914), "citation_id":Uuid::from_u128(915), "claim_position":1, "citation_position":1}]);
                    f.set_sections(vec![f.section.clone()]);
                    AdmissionError::CitationGroundingReference
                }
                16 => {
                    f.response.invocation_id = id(916, ModelInvocationId::new);
                    AdmissionError::ModelResponseIdentityMismatch
                }
                17 => {
                    f.response.model_id = id(917, ModelId::new);
                    AdmissionError::ModelResponseIdentityMismatch
                }
                _ => {
                    f.response.contract_version = ProtocolVersion::new(2, 0);
                    f.response.reported_usage = Some(ModelUsage {
                        input_tokens: 1,
                        output_tokens: 1,
                    });
                    AdmissionError::UnsupportedVersion
                }
            };
            if mutation == 6 {
                // The planner's response-size limit bounds the admitted raw section content;
                // RawModelOutput's larger intrinsic wire cap cannot be bypassed by construction,
                // so an over-cap raw response cannot exist for a LanguageModelProvider to return.
            }
            // Host-owned provenance mismatches are deliberately absent: shared preflight
            // rejects them before invocation, as covered by the preflight table above.
            let provider = CountingProvider::new(&f);
            assert_eq!(provider.calls(), 0, "preflight mutation {mutation}");
            assert_eq!(
                invoke_and_admit_model_output(
                    &provider,
                    &f.request,
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                ),
                Err(InvocationAdmissionError::Admission(expected)),
                "mutation {mutation}"
            );
            assert_eq!(provider.calls(), 1, "mutation {mutation}");
        }

        let mut f = admission_fixture();
        f.response.finish_reason = FinishReason::OutputLimit;
        let provider = CountingProvider::new(&f);
        assert_eq!(
            invoke_and_admit_model_output(
                &provider,
                &f.request,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations
            ),
            Err(InvocationAdmissionError::Admission(
                AdmissionError::IncompleteOutput
            ))
        );
        assert_eq!(provider.calls(), 1);
    }

    #[test]
    fn invocation_admission_diagnostics_are_content_free() {
        use crate::generation::{invoke_and_admit_model_output, InvocationAdmissionError};
        use crate::model::{
            ModelErrorKind, ModelInput, RawModelOutput, ScriptedModelProvider, ScriptedOutcome,
        };

        let f = admission_fixture();
        let provider = ScriptedModelProvider::new(
            f.descriptor.clone(),
            [ScriptedOutcome::Error(ModelErrorKind::Unavailable)],
        )
        .unwrap();
        let invocation_error = invoke_and_admit_model_output(
            &provider,
            &f.request,
            &f.compilation,
            &f.authority,
            &f.context,
            &f.citations,
        )
        .unwrap_err();
        assert_eq!(
            invocation_error,
            InvocationAdmissionError::Invocation(ModelErrorKind::Unavailable)
        );

        let prompt_secret = "coordinator-distinctive-prompt-secret";
        let mut preflight = admission_fixture();
        preflight.request.input = ModelInput::new(prompt_secret).unwrap();
        let preflight_provider = CountingProvider::new(&preflight);
        let preflight_error = invoke_and_admit_model_output(
            &preflight_provider,
            &preflight.request,
            &preflight.compilation,
            &preflight.authority,
            &preflight.context,
            &preflight.citations,
        )
        .unwrap_err();

        let context_secret = "coordinator-distinctive-governed-context-secret";
        let mut governed = admission_fixture();
        governed.context.content[0].content = context_secret.into();
        let governed_provider = CountingProvider::new(&governed);
        let governed_error = invoke_and_admit_model_output(
            &governed_provider,
            &governed.request,
            &governed.compilation,
            &governed.authority,
            &governed.context,
            &governed.citations,
        )
        .unwrap_err();

        let raw_secret = "coordinator-distinctive-raw-output-secret";
        let mut post = admission_fixture();
        post.response.output =
            RawModelOutput::new(format!("{{\"unknown\":\"{raw_secret}\"}}")).unwrap();
        let post_provider = CountingProvider::new(&post);
        let post_error = invoke_and_admit_model_output(
            &post_provider,
            &post.request,
            &post.compilation,
            &post.authority,
            &post.context,
            &post.citations,
        )
        .unwrap_err();

        let secrets = [
            prompt_secret,
            raw_secret,
            context_secret,
            "distinctive private platform prompt",
            "distinctive private learner prompt",
            "Caller supplied explanation",
        ];
        let diagnostics = [
            format!("{invocation_error}"),
            format!("{invocation_error:?}"),
            format!("{preflight_error}"),
            format!("{preflight_error:?}"),
            format!("{governed_error}"),
            format!("{governed_error:?}"),
            format!("{post_error}"),
            format!("{post_error:?}"),
            format!("{:?}", preflight.request),
            format!("{:?}", post.response),
            format!("{:?}", post.authority),
            format!("{:?}", post.context),
            format!("{:?}", governed.context),
            format!("{:?}", post.citations),
        ];
        for diagnostic in diagnostics {
            for secret in secrets {
                assert!(!diagnostic.contains(secret), "diagnostic leaked {secret}");
            }
        }
        assert_eq!(preflight_provider.calls(), 0);
        assert_eq!(governed_provider.calls(), 0);
        assert_eq!(post_provider.calls(), 1);
    }

    #[test]
    fn model_output_admission_is_closed_bound_and_delegates_to_planner() {
        use crate::admission::{
            admit_model_output, AdmissionError, AdmissionResult, TrustedPlanningAuthority,
        };
        use crate::model::{
            FinishReason, ModelCapabilities, ModelDescriptor, ModelRequest, ModelResponse,
            PrivacyClass, RawModelOutput, RequiredCapabilities, MODEL_INVOCATION_V1,
        };
        use crate::prompt::{
            compile_prompt, PromptCompilationRequest, PromptContent, PromptLayer, PromptLayerKind,
            PromptLimits, PROMPT_COMPILATION_V1,
        };
        use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId};

        let planning = planning_request(response());
        let authority = TrustedPlanningAuthority {
            contract_version: planning.contract_version,
            response_id: planning.response_id,
            interaction_id: planning.interaction_id,
            scope: planning.scope.clone(),
            context_package_id: planning.context_package_id,
            citation_set_id: planning.citation_set_id,
            hybrid_result_id: planning.hybrid_result_id,
            query_id: planning.query_id,
            response_policy_version: planning.response_policy_version,
            safety_policy_version: planning.safety_policy_version,
            citation_policy_version: planning.citation_policy_version,
            governance_policy_version: planning.governance_policy_version,
            limits: planning.limits,
            permitted_capabilities: planning.permitted_capabilities.clone(),
            evidence: planning.evidence.clone(),
        };
        let layer = |kind, text| PromptLayer {
            kind,
            classification: kind.classification(),
            content: PromptContent::new(text).unwrap(),
        };
        let compilation = compile_prompt(&PromptCompilationRequest {
            contract_version: PROMPT_COMPILATION_V1,
            prompt_package_version: V1,
            context_builder_version: V1,
            output_schema_version: V1,
            limits: PromptLimits {
                maximum_layer_bytes: 1000,
                maximum_compiled_bytes: 10000,
            },
            layers: vec![
                layer(PromptLayerKind::PlatformContract, "platform"),
                layer(PromptLayerKind::NexaIdentity, "identity"),
                layer(PromptLayerKind::Policy, "policy"),
                layer(PromptLayerKind::Pedagogy, "pedagogy"),
                layer(PromptLayerKind::StudentInput, "input"),
                layer(PromptLayerKind::OutputContract, "output"),
            ],
        })
        .unwrap();
        let descriptor = ModelDescriptor::new(
            id(50, ModelProviderId::new),
            id(51, ModelId::new),
            PrivacyClass::LocalOnly,
            ModelCapabilities {
                streaming: false,
                structured_output: true,
                tool_calling: false,
                vision: false,
                context_window_tokens: 20000,
                maximum_output_tokens: 1000,
            },
        )
        .unwrap();
        let request = ModelRequest {
            invocation_id: id(52, ModelInvocationId::new),
            provider_id: descriptor.provider_id,
            model_id: descriptor.model_id,
            contract_version: MODEL_INVOCATION_V1,
            input: compilation.model_input.clone(),
            required_capabilities: RequiredCapabilities {
                structured_output: true,
                tool_calling: false,
                vision: false,
            },
            maximum_output_tokens: 1000,
        };
        let candidate = json!({"candidate_schema_version":"1.0","sections":planning.sections});
        let output = RawModelOutput::new(serde_json::to_string(&candidate).unwrap()).unwrap();
        let response = ModelResponse {
            invocation_id: request.invocation_id,
            provider_id: request.provider_id,
            model_id: request.model_id,
            contract_version: MODEL_INVOCATION_V1,
            output,
            finish_reason: FinishReason::Complete,
            reported_usage: None,
        };
        let (context, citations) = governed_inputs();
        let admitted = admit_model_output(
            &descriptor,
            &request,
            &response,
            &compilation,
            &authority,
            &context,
            &citations,
        )
        .unwrap();
        assert_eq!(
            admitted.response,
            plan_response(&planning, &context, &citations).unwrap()
        );
        assert_eq!(admitted.evidence.provider_id, descriptor.provider_id);
        assert!(!format!("{admitted:?}").contains("Caller supplied explanation"));
        let wire = serde_json::to_string(&admitted).unwrap();
        assert_eq!(
            serde_json::from_str::<AdmissionResult>(&wire).unwrap(),
            admitted
        );

        let mut limited = response.clone();
        limited.finish_reason = FinishReason::OutputLimit;
        assert_eq!(
            admit_model_output(
                &descriptor,
                &request,
                &limited,
                &compilation,
                &authority,
                &context,
                &citations
            ),
            Err(AdmissionError::IncompleteOutput)
        );
        for bad in [
            json!({"candidate_schema_version":"1.0","sections":[],"response_id":planning.response_id}),
            json!({"candidate_schema_version":"1.0","sections":[]}),
        ] {
            let mut invalid = response.clone();
            invalid.output = RawModelOutput::new(serde_json::to_string(&bad).unwrap()).unwrap();
            assert!(admit_model_output(
                &descriptor,
                &request,
                &invalid,
                &compilation,
                &authority,
                &context,
                &citations
            )
            .is_err());
        }
        let mut tampered: serde_json::Value = serde_json::from_str(&wire).unwrap();
        tampered["evidence"]["provider_id"] =
            serde_json::to_value(id(90, ModelProviderId::new)).unwrap();
        assert!(serde_json::from_value::<AdmissionResult>(tampered).is_err());
    }
    #[test]
    fn planning_wire_rejects_all_intrinsic_contract_violations() {
        assert!(serde_json::from_value::<PlanningRequest>(planning_wire()).is_ok());
        for version in [
            "contract_version",
            "response_policy_version",
            "safety_policy_version",
            "citation_policy_version",
            "governance_policy_version",
        ] {
            let mut value = planning_wire();
            value[version] = "2.0".into();
            assert!(serde_json::from_value::<PlanningRequest>(value).is_err());
        }
        let mut value = planning_wire();
        value["permitted_capabilities"] = serde_json::json!([]);
        assert!(serde_json::from_value::<PlanningRequest>(value).is_err());

        let mut value = planning_wire();
        value["limits"]["maximum_sections"] = 1.into();
        let section = value["sections"][0].clone();
        value["sections"].as_array_mut().unwrap().push(section);
        assert!(serde_json::from_value::<PlanningRequest>(value).is_err());

        let mut value = planning_wire();
        value["sections"][0]["citations_required"] = true.into();
        assert!(serde_json::from_value::<PlanningRequest>(value).is_err());

        let claim = serde_json::to_value(id(40, ClaimId::new)).unwrap();
        let citation = serde_json::to_value(id(41, CitationId::new)).unwrap();
        let binding = serde_json::json!({
            "claim_id": claim, "citation_id": citation,
            "claim_position": 1, "citation_position": 1
        });
        let mut value = planning_wire();
        value["sections"][0]["claims"] = serde_json::json!([claim, claim]);
        value["sections"][0]["citations"] = serde_json::json!([binding, binding]);
        assert!(serde_json::from_value::<PlanningRequest>(value).is_err());

        let mut value = planning_wire();
        value["limits"]["maximum_section_bytes"] = 5.into();
        assert!(serde_json::from_value::<PlanningRequest>(value).is_err());
        let mut value = planning_wire();
        value["limits"]["maximum_response_bytes"] = 5.into();
        assert!(serde_json::from_value::<PlanningRequest>(value).is_err());
    }
    #[test]
    fn standalone_wire_rejects_intrinsically_invalid_values() {
        assert!(serde_json::from_str::<InertText>("\"tool_call\"").is_err());
        let mut v = serde_json::to_value(response()).unwrap();
        v["sections"][0]["position"] = 0.into();
        assert!(serde_json::from_value::<TutorResponse>(v).is_err());
        let mut v = serde_json::to_value(response()).unwrap();
        v["evidence"]["maximum_scaffolding"] = 11.into();
        assert!(serde_json::from_value::<TutorResponse>(v).is_err());
        let mut v = serde_json::to_value(response()).unwrap();
        v["limits"]["maximum_sections"] = 0.into();
        assert!(serde_json::from_value::<TutorResponse>(v).is_err());
        let mut v = serde_json::to_value(response()).unwrap();
        v["sections"][0]["citations_required"] = true.into();
        assert!(serde_json::from_value::<TutorResponse>(v).is_err());
    }
    #[test]
    fn recomputed_anchor_cannot_hide_semantic_tampering() {
        let mut r = response();
        r.status = ResponseStatus::Refused;
        recompute(&mut r);
        assert_eq!(r.validate(), Err(TutorError::InvalidEvidence));

        let mut r = response();
        r.ordered_section_ids[0] = id(99, TutorSectionId::new);
        recompute(&mut r);
        assert!(r.validate().is_err());

        let mut r = response();
        r.sections[0].citations_required = true;
        recompute(&mut r);
        assert!(r.validate().is_err());

        let mut r = response();
        r.sections[0].assessment_restriction = AssessmentRestriction::WithholdAnswers;
        recompute(&mut r);
        assert_eq!(r.validate(), Err(TutorError::InvalidStructure));
    }
    #[test]
    fn retained_citation_anchors_reject_reassociation() {
        let mut r = response();
        let claim = id(40, ClaimId::new);
        let citation = id(41, CitationId::new);
        r.sections[0].claims = vec![claim];
        r.sections[0].citations = vec![CitationBinding {
            claim_id: claim,
            citation_id: citation,
            claim_position: 1,
            citation_position: 1,
        }];
        r.sections[0].citations_required = true;
        r.citation_anchors = vec![CitationDecisionAnchor {
            claim_id: claim,
            citation_id: citation,
            claim_position: 1,
            citation_position: 1,
            resolved: true,
        }];
        r.citation_manifest = vec![CitationManifestEntry {
            claim_id: claim,
            citation_id: citation,
            claim_position: 1,
            citation_position: 1,
            resolved: true,
        }];
        recompute(&mut r);
        assert_eq!(r.validate(), Ok(()));

        r.sections[0].citations[0].citation_id = id(42, CitationId::new);
        recompute(&mut r);
        assert_eq!(r.validate(), Err(TutorError::CitationMismatch));
    }
    #[test]
    fn independent_manifest_rejects_fabrication_and_global_duplicates() {
        let mut r = response();
        let claim = id(40, ClaimId::new);
        let citation = id(41, CitationId::new);
        let binding = CitationBinding {
            claim_id: claim,
            citation_id: citation,
            claim_position: 1,
            citation_position: 1,
        };
        r.sections[0].claims = vec![claim];
        r.sections[0].citations = vec![binding.clone()];
        r.citation_anchors = vec![CitationDecisionAnchor {
            claim_id: claim,
            citation_id: citation,
            claim_position: 1,
            citation_position: 1,
            resolved: true,
        }];
        recompute(&mut r);
        assert_eq!(r.validate(), Err(TutorError::CitationMismatch));

        r.citation_manifest = vec![CitationManifestEntry {
            claim_id: claim,
            citation_id: citation,
            claim_position: 1,
            citation_position: 1,
            resolved: true,
        }];
        let mut duplicate = r.sections[0].clone();
        duplicate.section_id = id(14, TutorSectionId::new);
        duplicate.position = 2;
        r.sections.push(duplicate);
        r.ordered_section_ids.push(id(14, TutorSectionId::new));
        r.citation_anchors.push(r.citation_anchors[0].clone());
        recompute(&mut r);
        assert_eq!(r.validate(), Err(TutorError::CitationMismatch));
    }
    #[test]
    fn assessment_protected_hint_and_check_are_exactly_compatible() {
        for (kind, capability) in [
            (SectionKind::Hint, Capability::Hint),
            (
                SectionKind::CheckForUnderstanding,
                Capability::CheckUnderstanding,
            ),
        ] {
            let mut r = response();
            r.sections[0].kind = kind;
            r.sections[0].capability = capability;
            r.sections[0].safety = SafetyClassification::AssessmentProtected;
            r.sections[0].assessment_restriction = AssessmentRestriction::WithholdAnswers;
            r.permitted_capabilities = [capability].into();
            r.evidence.allowed_section_kinds = [kind].into();
            r.evidence.assessment_restriction = AssessmentRestriction::WithholdAnswers;
            r.status = ResponseStatus::Constrained;
            r.rationale = vec![Rationale::AssessmentProtection];
            recompute(&mut r);
            assert_eq!(r.validate(), Ok(()));

            let mut downgraded = r.clone();
            downgraded.sections[0].safety = SafetyClassification::Ordinary;
            recompute(&mut downgraded);
            assert_eq!(downgraded.validate(), Err(TutorError::InvalidEvidence));
            let mut mismatch = r;
            mismatch.sections[0].assessment_restriction =
                AssessmentRestriction::WithholdHiddenEvaluation;
            recompute(&mut mismatch);
            assert!(mismatch.validate().is_err());
        }
    }
    #[test]
    fn planner_enforces_the_shared_exact_safety_pairings() {
        let (context, citations) = governed_inputs();
        for (kind, capability) in [
            (SectionKind::Hint, Capability::Hint),
            (
                SectionKind::CheckForUnderstanding,
                Capability::CheckUnderstanding,
            ),
        ] {
            let mut fixture = response();
            fixture.sections[0].kind = kind;
            fixture.sections[0].capability = capability;
            fixture.sections[0].safety = SafetyClassification::AssessmentProtected;
            fixture.sections[0].assessment_restriction = AssessmentRestriction::WithholdAnswers;
            fixture.permitted_capabilities = [capability].into();
            fixture.evidence.allowed_section_kinds = [kind].into();
            fixture.evidence.assessment_restriction = AssessmentRestriction::WithholdAnswers;

            let valid = planning_request(fixture.clone());
            let planned = plan_response(&valid, &context, &citations).unwrap();
            assert_eq!(planned.validate(), Ok(()));
            let wire = serde_json::to_string(&planned).unwrap();
            assert!(serde_json::from_str::<TutorResponse>(&wire).is_ok());

            fixture.sections[0].safety = SafetyClassification::Ordinary;
            let downgraded = planning_request(fixture);
            assert_eq!(
                plan_response(&downgraded, &context, &citations),
                Err(TutorError::InvalidEvidence)
            );
        }

        for invalid_safety in [
            SafetyClassification::Ordinary,
            SafetyClassification::AssessmentProtected,
        ] {
            let mut fixture = response();
            fixture.sections[0].kind = SectionKind::ConstrainedResponse;
            fixture.sections[0].capability = Capability::Constrain;
            fixture.sections[0].safety = invalid_safety;
            fixture.permitted_capabilities = [Capability::Constrain].into();
            fixture.evidence.allowed_section_kinds = [SectionKind::ConstrainedResponse].into();
            let invalid = planning_request(fixture);
            assert_eq!(
                plan_response(&invalid, &context, &citations),
                Err(TutorError::InvalidEvidence)
            );
        }
    }

    #[test]
    fn every_accepted_planner_fixture_validates_and_round_trips() {
        let (context, citations) = governed_inputs();
        let mut fixtures = vec![response()];

        let mut constrained = response();
        constrained.sections[0].kind = SectionKind::ConstrainedResponse;
        constrained.sections[0].capability = Capability::Constrain;
        constrained.sections[0].safety = SafetyClassification::ConstrainedRequired;
        constrained.permitted_capabilities = [Capability::Constrain].into();
        constrained.evidence.allowed_section_kinds = [SectionKind::ConstrainedResponse].into();
        fixtures.push(constrained);

        let mut refused = response();
        refused.sections[0].kind = SectionKind::SafetyRefusal;
        refused.sections[0].capability = Capability::Refuse;
        refused.sections[0].safety = SafetyClassification::RefusalRequired;
        refused.permitted_capabilities = [Capability::Refuse].into();
        refused.evidence.allowed_section_kinds = [SectionKind::SafetyRefusal].into();
        fixtures.push(refused);

        for fixture in fixtures {
            let planned = plan_response(&planning_request(fixture), &context, &citations).unwrap();
            assert_eq!(planned.validate(), Ok(()));
            let wire = serde_json::to_string(&planned).unwrap();
            assert!(serde_json::from_str::<TutorResponse>(&wire).is_ok());
        }
    }
    #[test]
    fn safety_precedence_is_refusal_only_and_exact() {
        let mut refused = response();
        refused.sections[0].kind = SectionKind::SafetyRefusal;
        refused.sections[0].capability = Capability::Refuse;
        refused.sections[0].safety = SafetyClassification::RefusalRequired;
        refused.permitted_capabilities = [Capability::Refuse].into();
        refused.evidence.allowed_section_kinds = [SectionKind::SafetyRefusal].into();
        refused.status = ResponseStatus::Refused;
        refused.rationale = vec![Rationale::SafetyRefusal];
        recompute(&mut refused);
        assert_eq!(refused.validate(), Ok(()));

        let mut mixed = refused.clone();
        let mut ordinary = response().sections.remove(0);
        ordinary.section_id = id(14, TutorSectionId::new);
        ordinary.position = 2;
        mixed.sections.push(ordinary);
        mixed.ordered_section_ids.push(id(14, TutorSectionId::new));
        mixed.limits.maximum_sections = 2;
        mixed.permitted_capabilities.insert(Capability::Explain);
        mixed
            .evidence
            .allowed_section_kinds
            .insert(SectionKind::Explanation);
        recompute(&mut mixed);
        assert_eq!(mixed.validate(), Err(TutorError::InvalidEvidence));

        let mut constrained = response();
        constrained.sections[0].kind = SectionKind::ConstrainedResponse;
        constrained.sections[0].capability = Capability::Constrain;
        constrained.sections[0].safety = SafetyClassification::ConstrainedRequired;
        constrained.permitted_capabilities = [Capability::Constrain].into();
        constrained.evidence.allowed_section_kinds = [SectionKind::ConstrainedResponse].into();
        constrained.status = ResponseStatus::Constrained;
        constrained.rationale = vec![Rationale::SafetyConstraint];
        recompute(&mut constrained);
        assert_eq!(constrained.validate(), Ok(()));
    }
    #[test]
    fn content_debug_and_errors_are_redacted() {
        let secret = "very secret lesson text";
        let t = InertText::new(secret).unwrap();
        assert!(!format!("{t:?}").contains(secret));
        assert_eq!(
            InertText::new("https://provider.example/key").unwrap_err(),
            TutorError::UnsafeContent
        );
        assert!(!TutorError::UnsafeContent
            .to_string()
            .contains("provider.example"))
    }

    fn local_selection_requirements() -> crate::selection::ModelSelectionRequirements {
        crate::selection::ModelSelectionRequirements::new(
            crate::model::RequiredCapabilities {
                structured_output: true,
                tool_calling: false,
                vision: false,
            },
            1000,
            vec![crate::model::PrivacyClass::LocalOnly],
        )
        .unwrap()
    }

    #[test]
    fn local_tokenized_selection_returns_exact_evidence_and_direct_admission() {
        use crate::generation::select_local_model_tokenize_invoke_and_admit;
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::tokenization::{
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome, MODEL_INPUT_TOKENIZATION_V1,
        };
        use nexa_domain::{ModelId, ModelProviderId};
        use std::sync::Arc;

        for reverse in [false, true] {
            let f = admission_fixture();
            let mut selected_descriptor = f.descriptor.clone();
            selected_descriptor.provider_id = id(9_700, ModelProviderId::new);
            selected_descriptor.model_id = id(9_701, ModelId::new);
            let mut selected_response = f.response.clone();
            selected_response.provider_id = selected_descriptor.provider_id;
            selected_response.model_id = selected_descriptor.model_id;
            let mut selected_request = f.request.clone();
            selected_request.provider_id = selected_descriptor.provider_id;
            selected_request.model_id = selected_descriptor.model_id;
            let expected_admission = crate::admission::admit_model_output(
                &selected_descriptor,
                &selected_request,
                &selected_response,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap();
            let selected = Arc::new(
                ScriptedModelProvider::new(
                    selected_descriptor.clone(),
                    [ScriptedOutcome::Response(selected_response)],
                )
                .unwrap(),
            );
            let mut other_descriptor = f.descriptor.clone();
            other_descriptor.provider_id = id(9_750, ModelProviderId::new);
            other_descriptor.model_id = id(9_751, ModelId::new);
            let other = Arc::new(
                ScriptedModelProvider::new(
                    other_descriptor,
                    [ScriptedOutcome::Error(
                        crate::model::ModelErrorKind::Internal,
                    )],
                )
                .unwrap(),
            );
            let token_count = if reverse {
                f.descriptor.capabilities.context_window_tokens
                    - local_selection_requirements().maximum_output_tokens
            } else {
                1
            };
            let tokenizer = ScriptedModelInputTokenizer::new(
                selected_descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(token_count)],
            )
            .unwrap();
            let mut remote_descriptor = f.descriptor.clone();
            remote_descriptor.provider_id = id(9_800, ModelProviderId::new);
            remote_descriptor.model_id = id(9_801, ModelId::new);
            remote_descriptor.privacy_class = PrivacyClass::ApprovedRemote;
            let remote = Arc::new(
                ScriptedModelProvider::new(
                    remote_descriptor,
                    [ScriptedOutcome::Error(
                        crate::model::ModelErrorKind::Internal,
                    )],
                )
                .unwrap(),
            );
            let mut providers: Vec<Arc<dyn LanguageModelProvider>> =
                vec![other.clone(), remote.clone(), selected.clone()];
            if reverse {
                providers.reverse();
            }
            let registry = ModelRegistry::try_from_providers(providers).unwrap();

            let result = select_local_model_tokenize_invoke_and_admit(
                &registry,
                f.request.invocation_id,
                &local_selection_requirements(),
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap();

            assert_eq!(result.admission, expected_admission);
            result
                .tokenization_evidence
                .validate_for(&selected_descriptor, &f.compilation.model_input)
                .unwrap();
            assert_eq!(result.tokenization_evidence.input_token_count, token_count);
            assert_eq!(tokenizer.remaining().unwrap(), 0);
            assert_eq!(selected.remaining(), 0);
            assert_eq!(other.remaining(), 1);
            assert_eq!(remote.remaining(), 1);
        }
    }

    #[test]
    fn local_tokenized_selection_rejects_requirements_and_selection_before_dependencies() {
        use crate::generation::{
            select_local_model_tokenize_invoke_and_admit, SelectedTokenizedInvocationAdmissionError,
        };
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::selection::ModelSelectionError;
        use crate::tokenization::{
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome, MODEL_INPUT_TOKENIZATION_V1,
        };
        use std::sync::Arc;

        for privacy in [
            vec![],
            vec![PrivacyClass::ApprovedRemote],
            vec![PrivacyClass::LocalOnly, PrivacyClass::ApprovedRemote],
            vec![PrivacyClass::LocalOnly, PrivacyClass::LocalOnly],
        ] {
            let f = admission_fixture();
            let provider = Arc::new(
                ScriptedModelProvider::new(
                    f.descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                provider.clone() as Arc<dyn LanguageModelProvider>
            ])
            .unwrap();
            let tokenizer = ScriptedModelInputTokenizer::new(
                f.descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(1)],
            )
            .unwrap();
            let mut requirements = local_selection_requirements();
            requirements.privacy_preference = privacy;
            assert_eq!(
                select_local_model_tokenize_invoke_and_admit(
                    &registry,
                    f.request.invocation_id,
                    &requirements,
                    MODEL_INPUT_TOKENIZATION_V1,
                    &tokenizer,
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                ),
                Err(SelectedTokenizedInvocationAdmissionError::InvalidLocalOnlyRequirements)
            );
            assert_eq!(tokenizer.remaining().unwrap(), 1);
            assert_eq!(provider.remaining(), 1);
        }

        let f = admission_fixture();
        let mut ineligible_descriptor = f.descriptor.clone();
        ineligible_descriptor.privacy_class = PrivacyClass::ApprovedRemote;
        let ineligible = Arc::new(
            ScriptedModelProvider::new(
                ineligible_descriptor,
                [ScriptedOutcome::Response(f.response.clone())],
            )
            .unwrap(),
        );
        let registry = ModelRegistry::try_from_providers([
            ineligible.clone() as Arc<dyn LanguageModelProvider>
        ])
        .unwrap();
        let tokenizer = ScriptedModelInputTokenizer::new(
            f.descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(1)],
        )
        .unwrap();
        assert_eq!(
            select_local_model_tokenize_invoke_and_admit(
                &registry,
                f.request.invocation_id,
                &local_selection_requirements(),
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            ),
            Err(SelectedTokenizedInvocationAdmissionError::Selection(
                ModelSelectionError::NoEligibleModel
            ))
        );
        assert_eq!(tokenizer.remaining().unwrap(), 1);
        assert_eq!(ineligible.remaining(), 1);
    }

    #[test]
    fn local_tokenized_selection_preserves_exact_once_failures_and_closed_diagnostics() {
        use crate::generation::{
            select_local_model_tokenize_invoke_and_admit,
            SelectedTokenizedInvocationAdmissionError, TokenizedInvocationAdmissionError,
        };
        use crate::model::{
            LanguageModelProvider, ModelErrorKind, RawModelOutput, ScriptedModelProvider,
            ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::tokenization::{
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome, MODEL_INPUT_TOKENIZATION_V1,
        };
        use nexa_domain::{ModelId, ProtocolVersion};
        use std::sync::Arc;

        for wrong_version in [true, false] {
            let f = admission_fixture();
            let provider = Arc::new(
                ScriptedModelProvider::new(
                    f.descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                provider.clone() as Arc<dyn LanguageModelProvider>
            ])
            .unwrap();
            let mut tokenizer_descriptor = f.descriptor.clone();
            if !wrong_version {
                tokenizer_descriptor.model_id = id(9_900, ModelId::new);
            }
            let tokenizer = ScriptedModelInputTokenizer::new(
                tokenizer_descriptor,
                [ScriptedTokenizationOutcome::TokenCount(1)],
            )
            .unwrap();
            let version = if wrong_version {
                ProtocolVersion::new(2, 0)
            } else {
                MODEL_INPUT_TOKENIZATION_V1
            };
            assert!(matches!(
                select_local_model_tokenize_invoke_and_admit(
                    &registry,
                    f.request.invocation_id,
                    &local_selection_requirements(),
                    version,
                    &tokenizer,
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                ),
                Err(
                    SelectedTokenizedInvocationAdmissionError::TokenizedInvocationAdmission(
                        TokenizedInvocationAdmissionError::TokenizationCapacity(_)
                    )
                )
            ));
            assert_eq!(tokenizer.remaining().unwrap(), 1);
            assert_eq!(provider.remaining(), 1);
        }

        for outcome in [
            ScriptedTokenizationOutcome::Error,
            ScriptedTokenizationOutcome::TokenCount(u32::MAX),
        ] {
            let f = admission_fixture();
            let provider = Arc::new(
                ScriptedModelProvider::new(
                    f.descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                provider.clone() as Arc<dyn LanguageModelProvider>
            ])
            .unwrap();
            let tokenizer =
                ScriptedModelInputTokenizer::new(f.descriptor.clone(), [outcome]).unwrap();
            assert!(select_local_model_tokenize_invoke_and_admit(
                &registry,
                f.request.invocation_id,
                &local_selection_requirements(),
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .is_err());
            assert_eq!(tokenizer.remaining().unwrap(), 0);
            assert_eq!(provider.remaining(), 1);
        }

        let f = admission_fixture();
        let provider = Arc::new(
            ScriptedModelProvider::new(
                f.descriptor.clone(),
                [
                    ScriptedOutcome::Error(ModelErrorKind::Unavailable),
                    ScriptedOutcome::Error(ModelErrorKind::Internal),
                ],
            )
            .unwrap(),
        );
        let registry =
            ModelRegistry::try_from_providers([provider.clone() as Arc<dyn LanguageModelProvider>])
                .unwrap();
        let tokenizer = ScriptedModelInputTokenizer::new(
            f.descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(1)],
        )
        .unwrap();
        let provider_error = select_local_model_tokenize_invoke_and_admit(
            &registry,
            f.request.invocation_id,
            &local_selection_requirements(),
            MODEL_INPUT_TOKENIZATION_V1,
            &tokenizer,
            &f.compilation,
            &f.authority,
            &f.context,
            &f.citations,
        )
        .unwrap_err();
        assert_eq!(tokenizer.remaining().unwrap(), 0);
        assert_eq!(provider.remaining(), 1);

        let mut malformed = admission_fixture();
        malformed.response.output = RawModelOutput::new("model-output-private-sentinel").unwrap();
        malformed.context.tokenizer_profile_id = "knowledge-private-sentinel".into();
        let provider = Arc::new(SentinelProvider {
            inner: ScriptedModelProvider::new(
                malformed.descriptor.clone(),
                [ScriptedOutcome::Response(malformed.response.clone())],
            )
            .unwrap(),
            endpoint: "endpoint-private-sentinel".into(),
            credential: "credential-private-sentinel".into(),
            private_diagnostic: "provider-private-sentinel".into(),
        });
        let registry =
            ModelRegistry::try_from_providers([provider.clone() as Arc<dyn LanguageModelProvider>])
                .unwrap();
        let tokenizer = SentinelTokenizer {
            inner: ScriptedModelInputTokenizer::new(
                malformed.descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(1)],
            )
            .unwrap(),
            private_diagnostic: "tokenizer-private-sentinel".into(),
        };
        let admission_error = select_local_model_tokenize_invoke_and_admit(
            &registry,
            malformed.request.invocation_id,
            &local_selection_requirements(),
            MODEL_INPUT_TOKENIZATION_V1,
            &tokenizer,
            &malformed.compilation,
            &malformed.authority,
            &malformed.context,
            &malformed.citations,
        )
        .unwrap_err();
        assert_eq!(tokenizer.inner.remaining().unwrap(), 0);
        assert_eq!(provider.inner.remaining(), 0);
        for error in [provider_error, admission_error] {
            let diagnostics = format!("{error:?} {error}");
            for sentinel in [
                "model-output-private-sentinel",
                "prompt-private-sentinel",
                "learner-private-sentinel",
                "knowledge-private-sentinel",
                "endpoint-private-sentinel",
                "credential-private-sentinel",
                "tokenizer-private-sentinel",
                "provider-private-sentinel",
            ] {
                assert!(!diagnostics.contains(sentinel));
            }
        }
    }

    #[test]
    fn selected_usage_validated_tokenized_composition_rejects_every_requirement_and_selection_category(
    ) {
        use crate::generation::{
            select_local_model_tokenize_invoke_validate_reported_usage_and_admit,
            SelectedUsageValidatedTokenizedInvocationAdmissionError,
        };
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::selection::ModelSelectionError;
        use crate::tokenization::{
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome, MODEL_INPUT_TOKENIZATION_V1,
        };
        use std::sync::Arc;

        let f = admission_fixture();
        let (registry, provider, other, remote) =
            registry_with_untouched_sentinels(&f, [ScriptedOutcome::Response(f.response.clone())]);
        for mutation in 0..7 {
            let tokenizer = ScriptedModelInputTokenizer::new(
                f.descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(1)],
            )
            .unwrap();
            let mut requirements = local_selection_requirements();
            match mutation {
                0 => requirements.contract_version = ProtocolVersion::new(2, 0),
                1 => requirements.maximum_output_tokens = 0,
                2 => requirements.required_capabilities.structured_output = false,
                3 => requirements.privacy_preference.clear(),
                4 => requirements.privacy_preference = vec![PrivacyClass::ApprovedRemote],
                5 => {
                    requirements.privacy_preference =
                        vec![PrivacyClass::LocalOnly, PrivacyClass::ApprovedRemote]
                }
                _ => {
                    requirements.privacy_preference =
                        vec![PrivacyClass::LocalOnly, PrivacyClass::LocalOnly]
                }
            }
            assert_eq!(
                select_local_model_tokenize_invoke_validate_reported_usage_and_admit(
                    &registry,
                    f.request.invocation_id,
                    &requirements,
                    MODEL_INPUT_TOKENIZATION_V1,
                    &tokenizer,
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                ),
                Err(SelectedUsageValidatedTokenizedInvocationAdmissionError::InvalidLocalOnlyRequirements)
            );
            assert_eq!(tokenizer.remaining().unwrap(), 1);
            assert_eq!(provider.remaining(), 1);
            assert_eq!(other.remaining(), 1);
            assert_eq!(remote.remaining(), 1);
        }

        let tokenizer = ScriptedModelInputTokenizer::new(
            f.descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(1)],
        )
        .unwrap();
        assert_eq!(
            select_local_model_tokenize_invoke_validate_reported_usage_and_admit(
                &ModelRegistry::try_from_providers(std::iter::empty::<
                    Arc<dyn LanguageModelProvider>,
                >(),)
                .unwrap(),
                f.request.invocation_id,
                &local_selection_requirements(),
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            ),
            Err(
                SelectedUsageValidatedTokenizedInvocationAdmissionError::Selection(
                    ModelSelectionError::NoEligibleModel
                )
            )
        );
        assert_eq!(tokenizer.remaining().unwrap(), 1);
        assert_eq!(provider.remaining(), 1);

        // Privacy, capability, output, and conservative context exclusions all retain both
        // dependencies. No other ADR-0027 error is reachable from an immutable valid registry
        // after the explicit-local requirements gate.
        for mutation in 0..4 {
            let mut descriptor = f.descriptor.clone();
            let mut requirements = local_selection_requirements();
            match mutation {
                0 => descriptor.privacy_class = PrivacyClass::ApprovedRemote,
                1 => descriptor.capabilities.structured_output = false,
                2 => {
                    requirements.maximum_output_tokens =
                        descriptor.capabilities.maximum_output_tokens + 1
                }
                _ => {
                    descriptor.capabilities.context_window_tokens =
                        f.compilation.model_input.as_str().len() as u32
                }
            }
            let ineligible = Arc::new(
                ScriptedModelProvider::new(
                    descriptor,
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                ineligible.clone() as Arc<dyn LanguageModelProvider>
            ])
            .unwrap();
            let tokenizer = ScriptedModelInputTokenizer::new(
                f.descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(1)],
            )
            .unwrap();
            assert_eq!(
                select_local_model_tokenize_invoke_validate_reported_usage_and_admit(
                    &registry,
                    f.request.invocation_id,
                    &requirements,
                    MODEL_INPUT_TOKENIZATION_V1,
                    &tokenizer,
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                ),
                Err(
                    SelectedUsageValidatedTokenizedInvocationAdmissionError::Selection(
                        ModelSelectionError::NoEligibleModel
                    )
                ),
                "eligibility mutation {mutation}"
            );
            assert_eq!(tokenizer.remaining().unwrap(), 1);
            assert_eq!(ineligible.remaining(), 1);
        }

        // Canonical identity, not insertion order, selects the same local provider; the other
        // eligible local and an ineligible remote retain their complete queues.
        for reverse in [false, true] {
            let mut f = admission_fixture();
            let mut selected_descriptor = f.descriptor.clone();
            selected_descriptor.provider_id = id(88_200, nexa_domain::ModelProviderId::new);
            selected_descriptor.model_id = id(88_200, nexa_domain::ModelId::new);
            let mut other_descriptor = selected_descriptor.clone();
            other_descriptor.provider_id = id(88_201, nexa_domain::ModelProviderId::new);
            other_descriptor.model_id = id(88_201, nexa_domain::ModelId::new);
            let mut remote_descriptor = selected_descriptor.clone();
            remote_descriptor.provider_id = id(88_199, nexa_domain::ModelProviderId::new);
            remote_descriptor.model_id = id(88_199, nexa_domain::ModelId::new);
            remote_descriptor.privacy_class = PrivacyClass::ApprovedRemote;
            f.response.provider_id = selected_descriptor.provider_id;
            f.response.model_id = selected_descriptor.model_id;
            let selected = Arc::new(
                ScriptedModelProvider::new(
                    selected_descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let other = Arc::new(
                ScriptedModelProvider::new(
                    other_descriptor,
                    [ScriptedOutcome::Error(
                        crate::model::ModelErrorKind::Internal,
                    )],
                )
                .unwrap(),
            );
            let remote = Arc::new(
                ScriptedModelProvider::new(
                    remote_descriptor,
                    [ScriptedOutcome::Error(
                        crate::model::ModelErrorKind::Internal,
                    )],
                )
                .unwrap(),
            );
            let providers: Vec<Arc<dyn LanguageModelProvider>> = if reverse {
                vec![other.clone(), remote.clone(), selected.clone()]
            } else {
                vec![selected.clone(), remote.clone(), other.clone()]
            };
            let registry = ModelRegistry::try_from_providers(providers).unwrap();
            let tokenizer = ScriptedModelInputTokenizer::new(
                selected_descriptor,
                [ScriptedTokenizationOutcome::TokenCount(7)],
            )
            .unwrap();
            select_local_model_tokenize_invoke_validate_reported_usage_and_admit(
                &registry,
                f.request.invocation_id,
                &local_selection_requirements(),
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap();
            assert_eq!(tokenizer.remaining().unwrap(), 0);
            assert_eq!(selected.remaining(), 0);
            assert_eq!(other.remaining(), 1);
            assert_eq!(remote.remaining(), 1);
        }
    }

    #[test]
    fn selected_usage_validated_tokenized_composition_proves_nested_ordering_and_exact_success() {
        use crate::generation::{
            select_local_model_tokenize_invoke_validate_reported_usage_and_admit,
            tokenize_invoke_validate_reported_usage_and_admit_model_output_with_token_capacity,
            SelectedUsageValidatedTokenizedInvocationAdmissionError,
            UsageValidatedTokenizedInvocationAdmissionError,
        };
        use crate::model::{
            LanguageModelProvider, ModelErrorKind, ModelRequest, ModelUsage, RawModelOutput,
            ScriptedModelProvider, ScriptedOutcome, MODEL_INVOCATION_V1,
        };
        use crate::registry::ModelRegistry;
        use crate::tokenization::{
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome, MODEL_INPUT_TOKENIZATION_V1,
        };
        use crate::usage::ModelResponseReportedUsageValidationError;
        use std::sync::Arc;

        for reported in [None, Some(7), Some(6), Some(8)] {
            let mut f = admission_fixture();
            f.response.reported_usage = reported.map(|input_tokens| ModelUsage {
                input_tokens,
                output_tokens: 1,
            });
            let provider = Arc::new(
                ScriptedModelProvider::new(
                    f.descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                provider.clone() as Arc<dyn LanguageModelProvider>
            ])
            .unwrap();
            let tokenizer = ScriptedModelInputTokenizer::new(
                f.descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(7)],
            )
            .unwrap();
            let result = select_local_model_tokenize_invoke_validate_reported_usage_and_admit(
                &registry,
                f.request.invocation_id,
                &local_selection_requirements(),
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            );
            if matches!(reported, None | Some(7)) {
                let result = result.unwrap();
                assert_eq!(result.admission, f.admit().unwrap());
                assert_eq!(result.tokenization_evidence.input_token_count, 7);
                result
                    .tokenization_evidence
                    .validate_for(&f.descriptor, &f.compilation.model_input)
                    .unwrap();

                let request = ModelRequest {
                    invocation_id: f.request.invocation_id,
                    provider_id: f.descriptor.provider_id,
                    model_id: f.descriptor.model_id,
                    contract_version: MODEL_INVOCATION_V1,
                    input: f.compilation.model_input.clone(),
                    required_capabilities: local_selection_requirements().required_capabilities,
                    maximum_output_tokens: local_selection_requirements().maximum_output_tokens,
                };
                let direct_tokenizer = ScriptedModelInputTokenizer::new(
                    f.descriptor.clone(),
                    [ScriptedTokenizationOutcome::TokenCount(7)],
                )
                .unwrap();
                let direct_provider = ScriptedModelProvider::new(
                    f.descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap();
                let direct = tokenize_invoke_validate_reported_usage_and_admit_model_output_with_token_capacity(
                    MODEL_INPUT_TOKENIZATION_V1,
                    &direct_tokenizer,
                    &direct_provider,
                    &request,
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                )
                .unwrap();
                assert_eq!(result, direct);
                assert_eq!(direct_tokenizer.remaining().unwrap(), 0);
                assert_eq!(direct_provider.remaining(), 0);
            } else {
                assert_eq!(
                    result,
                    Err(SelectedUsageValidatedTokenizedInvocationAdmissionError::
                        UsageValidatedTokenizedInvocationAdmission(
                            UsageValidatedTokenizedInvocationAdmissionError::ReportedUsage(
                                ModelResponseReportedUsageValidationError::InputTokenCountMismatch
                            )
                        ))
                );
            }
            assert_eq!(tokenizer.remaining().unwrap(), 0);
            assert_eq!(provider.remaining(), 0);
        }

        use crate::admission::AdmissionError;
        use crate::tokenization::{
            ModelInputTokenizationError, ModelRequestTokenCapacityError,
            TokenizeAndValidateModelRequestCapacityError as Capacity,
        };
        use nexa_domain::{ModelProviderId, ProtocolVersion};

        // Every admission-preflight class reachable after valid local selection is preserved
        // under the exact ADR-0046 `Preflight` nesting and consumes neither dependency.
        for mutation in 0..5 {
            let mut f = admission_fixture();
            let expected = match mutation {
                0 => {
                    f.compilation.contract_version = ProtocolVersion::new(2, 0);
                    AdmissionError::UnsupportedVersion
                }
                1 => {
                    f.compilation.compiled_bytes += 1;
                    AdmissionError::PromptAssociationReplayMismatch
                }
                2 => {
                    f.authority.permitted_capabilities.clear();
                    AdmissionError::PolicyPedagogySafetyCapability
                }
                3 => {
                    f.context.maximum_tokens = 0;
                    AdmissionError::PlanningEvidenceProvenance
                }
                _ => {
                    f.citations.maximum_citations = 0;
                    AdmissionError::CitationGroundingReference
                }
            };
            let provider = Arc::new(
                ScriptedModelProvider::new(
                    f.descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                provider.clone() as Arc<dyn LanguageModelProvider>
            ])
            .unwrap();
            let tokenizer = ScriptedModelInputTokenizer::new(
                f.descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(7)],
            )
            .unwrap();
            assert_eq!(select_local_model_tokenize_invoke_validate_reported_usage_and_admit(
                &registry, f.request.invocation_id, &local_selection_requirements(),
                MODEL_INPUT_TOKENIZATION_V1, &tokenizer, &f.compilation, &f.authority,
                &f.context, &f.citations,
            ), Err(SelectedUsageValidatedTokenizedInvocationAdmissionError::
                UsageValidatedTokenizedInvocationAdmission(
                    UsageValidatedTokenizedInvocationAdmissionError::Preflight(expected))));
            assert_eq!(tokenizer.remaining().unwrap(), 1);
            assert_eq!(provider.remaining(), 1);
        }

        // Exact tokenization leaves are distinct cases rather than an accidental repeated range.
        for mutation in 0..5 {
            let f = admission_fixture();
            let mut tokenizer_descriptor = f.descriptor.clone();
            let (version, outcomes, expected, tokenizer_remaining) = match mutation {
                0 => (
                    ProtocolVersion::new(2, 0),
                    vec![ScriptedTokenizationOutcome::TokenCount(7)],
                    Capacity::Tokenization(ModelInputTokenizationError::UnsupportedVersion),
                    1,
                ),
                1 => {
                    tokenizer_descriptor.provider_id = id(88_001, ModelProviderId::new);
                    (
                        MODEL_INPUT_TOKENIZATION_V1,
                        vec![ScriptedTokenizationOutcome::TokenCount(7)],
                        Capacity::Tokenization(ModelInputTokenizationError::InvalidDescriptor),
                        1,
                    )
                }
                2 => (
                    MODEL_INPUT_TOKENIZATION_V1,
                    vec![ScriptedTokenizationOutcome::Error],
                    Capacity::Tokenization(ModelInputTokenizationError::TokenizerFailure),
                    0,
                ),
                3 => (
                    MODEL_INPUT_TOKENIZATION_V1,
                    vec![],
                    Capacity::Tokenization(ModelInputTokenizationError::ScriptExhausted),
                    0,
                ),
                _ => (
                    MODEL_INPUT_TOKENIZATION_V1,
                    vec![ScriptedTokenizationOutcome::TokenCount(0)],
                    Capacity::Tokenization(ModelInputTokenizationError::InvalidEvidence),
                    0,
                ),
            };
            let (registry, provider, other, remote) = registry_with_untouched_sentinels(
                &f,
                [ScriptedOutcome::Response(f.response.clone())],
            );
            let tokenizer =
                ScriptedModelInputTokenizer::new(tokenizer_descriptor, outcomes).unwrap();
            assert_eq!(
                select_local_model_tokenize_invoke_validate_reported_usage_and_admit(
                    &registry, f.request.invocation_id, &local_selection_requirements(), version,
                    &tokenizer, &f.compilation, &f.authority, &f.context, &f.citations,
                ),
                Err(SelectedUsageValidatedTokenizedInvocationAdmissionError::
                    UsageValidatedTokenizedInvocationAdmission(
                        UsageValidatedTokenizedInvocationAdmissionError::TokenizationCapacity(expected)
                    )),
                "tokenization mutation {mutation}"
            );
            assert_eq!(tokenizer.remaining().unwrap(), tokenizer_remaining);
            assert_eq!(provider.remaining(), 1);
            assert_eq!(other.remaining(), 1);
            assert_eq!(remote.remaining(), 1);
        }

        // Checked exact capacity succeeds at equality, and fails identically for one-token
        // excess and checked-add overflow. Only equality reaches the selected provider.
        for (input_tokens, succeeds) in [
            (
                admission_fixture()
                    .descriptor
                    .capabilities
                    .context_window_tokens
                    - local_selection_requirements().maximum_output_tokens,
                true,
            ),
            (
                admission_fixture()
                    .descriptor
                    .capabilities
                    .context_window_tokens
                    - local_selection_requirements().maximum_output_tokens
                    + 1,
                false,
            ),
            (u32::MAX, false),
        ] {
            let f = admission_fixture();
            let (registry, provider, other, remote) = registry_with_untouched_sentinels(
                &f,
                [ScriptedOutcome::Response(f.response.clone())],
            );
            let tokenizer = ScriptedModelInputTokenizer::new(
                f.descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(input_tokens)],
            )
            .unwrap();
            let result = select_local_model_tokenize_invoke_validate_reported_usage_and_admit(
                &registry,
                f.request.invocation_id,
                &local_selection_requirements(),
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            );
            if succeeds {
                assert_eq!(
                    result.unwrap().tokenization_evidence.input_token_count,
                    input_tokens
                );
                assert_eq!(provider.remaining(), 0);
            } else {
                assert_eq!(
                    result,
                    Err(SelectedUsageValidatedTokenizedInvocationAdmissionError::
                        UsageValidatedTokenizedInvocationAdmission(
                            UsageValidatedTokenizedInvocationAdmissionError::TokenizationCapacity(
                                Capacity::TokenCapacity(ModelRequestTokenCapacityError::ExactCapacity)
                            )
                        ))
                );
                assert_eq!(provider.remaining(), 1);
            }
            assert_eq!(tokenizer.remaining().unwrap(), 0);
            assert_eq!(other.remaining(), 1);
            assert_eq!(remote.remaining(), 1);
        }

        let f = admission_fixture();
        let provider = Arc::new(
            ScriptedModelProvider::new(
                f.descriptor.clone(),
                [
                    ScriptedOutcome::Error(ModelErrorKind::Unavailable),
                    ScriptedOutcome::Response(f.response.clone()),
                ],
            )
            .unwrap(),
        );
        let registry =
            ModelRegistry::try_from_providers([provider.clone() as Arc<dyn LanguageModelProvider>])
                .unwrap();
        let tokenizer = ScriptedModelInputTokenizer::new(
            f.descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(7)],
        )
        .unwrap();
        assert_eq!(select_local_model_tokenize_invoke_validate_reported_usage_and_admit(
            &registry, f.request.invocation_id, &local_selection_requirements(), MODEL_INPUT_TOKENIZATION_V1,
            &tokenizer, &f.compilation, &f.authority, &f.context, &f.citations,
        ), Err(SelectedUsageValidatedTokenizedInvocationAdmissionError::UsageValidatedTokenizedInvocationAdmission(
            UsageValidatedTokenizedInvocationAdmissionError::Invocation(ModelErrorKind::Unavailable))));
        assert_eq!(tokenizer.remaining().unwrap(), 0);
        assert_eq!(provider.remaining(), 1);

        // Response validation precedes usage equality and admission; admission sees only a
        // response that is valid under ADR-0045.
        for mutation in 0..4 {
            let mut f = admission_fixture();
            f.response.reported_usage = Some(ModelUsage {
                input_tokens: 6,
                output_tokens: 1,
            });
            let expected = match mutation {
                0 => {
                    f.response.invocation_id = id(88_010, nexa_domain::ModelInvocationId::new);
                    ModelErrorKind::IdentityMismatch
                }
                1 => {
                    f.response.contract_version = ProtocolVersion::new(2, 0);
                    ModelErrorKind::UnsupportedVersion
                }
                2 => {
                    f.response.reported_usage.as_mut().unwrap().output_tokens =
                        local_selection_requirements().maximum_output_tokens + 1;
                    ModelErrorKind::InvalidResponse
                }
                _ => {
                    f.response.output = RawModelOutput::new("not json").unwrap();
                    ModelErrorKind::InvalidResponse
                }
            };
            let provider = Arc::new(CountingProvider::new(&f));
            let (other, remote) = untouched_sentinel_providers(&f);
            let registry = ModelRegistry::try_from_providers([
                provider.clone() as Arc<dyn LanguageModelProvider>,
                other.clone(),
                remote.clone(),
            ])
            .unwrap();
            let tokenizer = ScriptedModelInputTokenizer::new(
                f.descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(7)],
            )
            .unwrap();
            let result = select_local_model_tokenize_invoke_validate_reported_usage_and_admit(
                &registry,
                f.request.invocation_id,
                &local_selection_requirements(),
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            );
            let expected = if mutation == 3 {
                // Raw structure is ADR-0045-valid and therefore reaches unchanged admission.
                Err(SelectedUsageValidatedTokenizedInvocationAdmissionError::UsageValidatedTokenizedInvocationAdmission(
                    UsageValidatedTokenizedInvocationAdmissionError::ReportedUsage(
                        ModelResponseReportedUsageValidationError::InputTokenCountMismatch)))
            } else {
                Err(SelectedUsageValidatedTokenizedInvocationAdmissionError::UsageValidatedTokenizedInvocationAdmission(
                    UsageValidatedTokenizedInvocationAdmissionError::ReportedUsage(
                        ModelResponseReportedUsageValidationError::Response(expected))))
            };
            assert_eq!(result, expected);
            assert_eq!(tokenizer.remaining().unwrap(), 0);
            assert_eq!(provider.calls(), 1);
            assert_eq!(other.remaining(), 1);
            assert_eq!(remote.remaining(), 1);
        }

        let mut f = admission_fixture();
        f.response.output = RawModelOutput::new("not json").unwrap();
        let (registry, provider, other, remote) =
            registry_with_untouched_sentinels(&f, [ScriptedOutcome::Response(f.response.clone())]);
        let tokenizer = ScriptedModelInputTokenizer::new(
            f.descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(7)],
        )
        .unwrap();
        assert_eq!(select_local_model_tokenize_invoke_validate_reported_usage_and_admit(
            &registry, f.request.invocation_id, &local_selection_requirements(), MODEL_INPUT_TOKENIZATION_V1,
            &tokenizer, &f.compilation, &f.authority, &f.context, &f.citations,
        ), Err(SelectedUsageValidatedTokenizedInvocationAdmissionError::UsageValidatedTokenizedInvocationAdmission(
            UsageValidatedTokenizedInvocationAdmissionError::Admission(AdmissionError::MalformedSyntax))));
        assert_eq!(tokenizer.remaining().unwrap(), 0);
        assert_eq!(provider.remaining(), 0);
        assert_eq!(other.remaining(), 1);
        assert_eq!(remote.remaining(), 1);
    }

    #[test]
    fn selected_usage_validated_tokenized_composition_proves_multi_invalid_precedence() {
        use crate::admission::AdmissionError;
        use crate::generation::{
            select_local_model_tokenize_invoke_validate_reported_usage_and_admit,
            SelectedUsageValidatedTokenizedInvocationAdmissionError as Outer,
            UsageValidatedTokenizedInvocationAdmissionError as Inner,
        };
        use crate::model::{
            LanguageModelProvider, ModelErrorKind, ModelUsage, PrivacyClass, RawModelOutput,
            ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::selection::ModelSelectionError;
        use crate::tokenization::{
            ModelInputTokenizationError, ScriptedModelInputTokenizer, ScriptedTokenizationOutcome,
            TokenizeAndValidateModelRequestCapacityError as Capacity, MODEL_INPUT_TOKENIZATION_V1,
        };
        use crate::usage::ModelResponseReportedUsageValidationError as Usage;
        use nexa_domain::ProtocolVersion;
        use std::sync::Arc;

        // Selection wins even though every downstream input is invalid. The ineligible provider
        // carries a real queued outcome and is part of the failing registry call.
        let mut f = admission_fixture();
        f.descriptor.privacy_class = PrivacyClass::ApprovedRemote;
        f.compilation.contract_version = ProtocolVersion::new(2, 0);
        f.response.contract_version = ProtocolVersion::new(2, 0);
        f.response.output = RawModelOutput::new("not json").unwrap();
        let provider = Arc::new(
            ScriptedModelProvider::new(
                f.descriptor.clone(),
                [ScriptedOutcome::Error(ModelErrorKind::Internal)],
            )
            .unwrap(),
        );
        let registry =
            ModelRegistry::try_from_providers([provider.clone() as Arc<dyn LanguageModelProvider>])
                .unwrap();
        let tokenizer = ScriptedModelInputTokenizer::new(
            f.descriptor.clone(),
            [ScriptedTokenizationOutcome::Error],
        )
        .unwrap();
        assert_eq!(
            select_local_model_tokenize_invoke_validate_reported_usage_and_admit(
                &registry,
                f.request.invocation_id,
                &local_selection_requirements(),
                ProtocolVersion::new(2, 0),
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            ),
            Err(Outer::Selection(ModelSelectionError::NoEligibleModel))
        );
        assert_eq!(tokenizer.remaining().unwrap(), 1);
        assert_eq!(provider.remaining(), 1);

        // Each later row deliberately keeps every downstream stage invalid. Exact consumption
        // proves the first failing stage wins and neither sentinel provider can be attempted.
        for stage in [0, 1, 2, 4] {
            let mut f = admission_fixture();
            f.response.reported_usage = Some(ModelUsage {
                input_tokens: 6,
                output_tokens: 1,
            });
            f.response.output = RawModelOutput::new("not json").unwrap();
            let mut version = MODEL_INPUT_TOKENIZATION_V1;
            let mut token_outcome = ScriptedTokenizationOutcome::TokenCount(7);
            let mut selected_outcome = ScriptedOutcome::Error(ModelErrorKind::Unavailable);
            let expected = match stage {
                0 => {
                    f.compilation.contract_version = ProtocolVersion::new(2, 0);
                    version = ProtocolVersion::new(2, 0);
                    token_outcome = ScriptedTokenizationOutcome::Error;
                    Inner::Preflight(AdmissionError::UnsupportedVersion)
                }
                1 => {
                    token_outcome = ScriptedTokenizationOutcome::Error;
                    Inner::TokenizationCapacity(Capacity::Tokenization(
                        ModelInputTokenizationError::TokenizerFailure,
                    ))
                }
                2 => Inner::Invocation(ModelErrorKind::Unavailable),
                _ => {
                    selected_outcome = ScriptedOutcome::Response(f.response.clone());
                    Inner::ReportedUsage(Usage::InputTokenCountMismatch)
                }
            };
            let (registry, selected, other, remote) =
                registry_with_untouched_sentinels(&f, [selected_outcome]);
            let tokenizer =
                ScriptedModelInputTokenizer::new(f.descriptor.clone(), [token_outcome]).unwrap();
            assert_eq!(
                select_local_model_tokenize_invoke_validate_reported_usage_and_admit(
                    &registry,
                    f.request.invocation_id,
                    &local_selection_requirements(),
                    version,
                    &tokenizer,
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                ),
                Err(Outer::UsageValidatedTokenizedInvocationAdmission(expected)),
                "precedence stage {stage}"
            );
            let dependencies_reached = stage >= 1;
            let provider_reached = stage >= 2;
            assert_eq!(
                tokenizer.remaining().unwrap(),
                usize::from(!dependencies_reached)
            );
            assert_eq!(selected.remaining(), usize::from(!provider_reached));
            assert_eq!(other.remaining(), 1);
            assert_eq!(remote.remaining(), 1);
        }

        // Reported-response validation wins over both reported-usage equality and admission.
        // This provider intentionally returns the invalid response without validating it during
        // invocation, so the exact error proves that the ADR-0045 reported-response layer ran.
        let mut f = admission_fixture();
        f.response.invocation_id = id(88_301, nexa_domain::ModelInvocationId::new);
        f.response.reported_usage = Some(ModelUsage {
            input_tokens: 6,
            output_tokens: 1,
        });
        f.response.output = RawModelOutput::new("not json").unwrap();
        let selected = Arc::new(UncheckedScriptedProvider::new(
            f.descriptor.clone(),
            [ScriptedOutcome::Response(f.response.clone())],
        ));
        let (other, remote) = untouched_sentinel_providers(&f);
        let registry = ModelRegistry::try_from_providers([
            selected.clone() as Arc<dyn LanguageModelProvider>,
            other.clone(),
            remote.clone(),
        ])
        .unwrap();
        let tokenizer = ScriptedModelInputTokenizer::new(
            f.descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(7)],
        )
        .unwrap();
        assert_eq!(
            select_local_model_tokenize_invoke_validate_reported_usage_and_admit(
                &registry,
                f.request.invocation_id,
                &local_selection_requirements(),
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            ),
            Err(Outer::UsageValidatedTokenizedInvocationAdmission(
                Inner::ReportedUsage(Usage::Response(ModelErrorKind::IdentityMismatch))
            ))
        );
        assert_eq!(tokenizer.remaining().unwrap(), 0);
        assert_eq!(selected.remaining(), 0);
        assert_eq!(other.remaining(), 1);
        assert_eq!(remote.remaining(), 1);
    }

    #[test]
    fn selected_usage_validated_tokenized_composition_diagnostics_are_content_free() {
        use crate::generation::{
            select_local_model_tokenize_invoke_validate_reported_usage_and_admit,
            SelectedUsageValidatedTokenizedInvocationAdmissionError as Outer,
            UsageValidatedTokenizedInvocationAdmissionError as Inner,
        };
        use crate::model::{
            LanguageModelProvider, ModelErrorKind, ModelUsage, RawModelOutput, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::selection::ModelSelectionError;
        use crate::tokenization::{
            ModelInputTokenizationError, ScriptedModelInputTokenizer, ScriptedTokenizationOutcome,
            TokenizeAndValidateModelRequestCapacityError as Capacity, MODEL_INPUT_TOKENIZATION_V1,
        };
        use crate::usage::ModelResponseReportedUsageValidationError as Usage;
        use nexa_domain::ProtocolVersion;
        use std::sync::Arc;

        struct DiagnosticProvider {
            inner: UncheckedScriptedProvider,
            endpoint: &'static str,
            credential: &'static str,
            private_diagnostic: &'static str,
            usage_adjacent: &'static str,
        }
        impl LanguageModelProvider for DiagnosticProvider {
            fn descriptor(&self) -> &crate::model::ModelDescriptor {
                self.inner.descriptor()
            }
            fn generate(
                &self,
                request: &crate::model::ModelRequest,
            ) -> Result<crate::model::ModelResponse, crate::model::ModelError> {
                assert!(!self.endpoint.is_empty());
                assert!(!self.credential.is_empty());
                assert!(!self.private_diagnostic.is_empty());
                assert!(!self.usage_adjacent.is_empty());
                self.inner.generate(request)
            }
        }

        let sentinels = [
            "prompt-private-sentinel",
            "learner-private-sentinel",
            "knowledge-private-sentinel",
            "response-private-sentinel",
            "usage-adjacent-sentinel",
            "tokenizer-private-sentinel",
            "provider-private-sentinel",
            "endpoint-private-sentinel",
            "credential-private-sentinel",
        ];
        let assert_closed = |error: Outer| {
            for diagnostic in [format!("{error:?}"), format!("{error}")] {
                for sentinel in sentinels {
                    assert!(!diagnostic.contains(sentinel), "leaked {sentinel}");
                }
            }
        };

        let mut base = admission_fixture();
        base.context.tokenizer_profile_id = sentinels[2].into();
        assert!(base.compilation.model_input.as_str().contains(sentinels[0]));
        assert!(base.compilation.model_input.as_str().contains(sentinels[1]));

        // Outer requirement and selection categories are produced by the wrapper itself.
        let provider = Arc::new(DiagnosticProvider {
            inner: UncheckedScriptedProvider::new(
                base.descriptor.clone(),
                [ScriptedOutcome::Response(base.response.clone())],
            ),
            endpoint: sentinels[7],
            credential: sentinels[8],
            private_diagnostic: sentinels[6],
            usage_adjacent: sentinels[4],
        });
        let registry =
            ModelRegistry::try_from_providers([provider.clone() as Arc<dyn LanguageModelProvider>])
                .unwrap();
        let tokenizer = SentinelTokenizer {
            inner: ScriptedModelInputTokenizer::new(
                base.descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(7)],
            )
            .unwrap(),
            private_diagnostic: sentinels[5].into(),
        };
        let mut invalid = local_selection_requirements();
        invalid.privacy_preference.clear();
        let error = select_local_model_tokenize_invoke_validate_reported_usage_and_admit(
            &registry,
            base.request.invocation_id,
            &invalid,
            MODEL_INPUT_TOKENIZATION_V1,
            &tokenizer,
            &base.compilation,
            &base.authority,
            &base.context,
            &base.citations,
        )
        .unwrap_err();
        assert_eq!(error, Outer::InvalidLocalOnlyRequirements);
        assert_eq!(tokenizer.inner.remaining().unwrap(), 1);
        assert_eq!(provider.inner.remaining(), 1);
        assert_closed(error);

        let mut ineligible_descriptor = base.descriptor.clone();
        ineligible_descriptor.privacy_class = crate::model::PrivacyClass::ApprovedRemote;
        let ineligible = Arc::new(DiagnosticProvider {
            inner: UncheckedScriptedProvider::new(
                ineligible_descriptor,
                [ScriptedOutcome::Response(base.response.clone())],
            ),
            endpoint: sentinels[7],
            credential: sentinels[8],
            private_diagnostic: sentinels[6],
            usage_adjacent: sentinels[4],
        });
        let ineligible_registry = ModelRegistry::try_from_providers([
            ineligible.clone() as Arc<dyn LanguageModelProvider>
        ])
        .unwrap();
        let error = select_local_model_tokenize_invoke_validate_reported_usage_and_admit(
            &ineligible_registry,
            base.request.invocation_id,
            &local_selection_requirements(),
            MODEL_INPUT_TOKENIZATION_V1,
            &tokenizer,
            &base.compilation,
            &base.authority,
            &base.context,
            &base.citations,
        )
        .unwrap_err();
        assert_eq!(
            error,
            Outer::Selection(ModelSelectionError::NoEligibleModel)
        );
        assert_eq!(tokenizer.inner.remaining().unwrap(), 1);
        assert_eq!(ineligible.inner.remaining(), 1);
        assert_closed(error);

        // Every nested category below is reached through a fresh wrapper call carrying all
        // sentinel-bearing prompt/context/tokenizer/provider/endpoint/credential state.
        for mutation in 0..7 {
            let mut f = admission_fixture();
            f.context.tokenizer_profile_id = sentinels[2].into();
            f.response.output =
                RawModelOutput::new(format!("{} {} not json", sentinels[3], sentinels[4])).unwrap();
            let (version, token_outcome, provider_outcome, expected) = match mutation {
                0 => {
                    f.compilation.contract_version = ProtocolVersion::new(2, 0);
                    (
                        MODEL_INPUT_TOKENIZATION_V1,
                        ScriptedTokenizationOutcome::TokenCount(7),
                        ScriptedOutcome::Response(f.response.clone()),
                        Inner::Preflight(crate::admission::AdmissionError::UnsupportedVersion),
                    )
                }
                1 => (
                    ProtocolVersion::new(2, 0),
                    ScriptedTokenizationOutcome::TokenCount(7),
                    ScriptedOutcome::Response(f.response.clone()),
                    Inner::TokenizationCapacity(Capacity::Tokenization(
                        ModelInputTokenizationError::UnsupportedVersion,
                    )),
                ),
                2 => (
                    MODEL_INPUT_TOKENIZATION_V1,
                    ScriptedTokenizationOutcome::Error,
                    ScriptedOutcome::Response(f.response.clone()),
                    Inner::TokenizationCapacity(Capacity::Tokenization(
                        ModelInputTokenizationError::TokenizerFailure,
                    )),
                ),
                3 => (
                    MODEL_INPUT_TOKENIZATION_V1,
                    ScriptedTokenizationOutcome::TokenCount(7),
                    ScriptedOutcome::Error(ModelErrorKind::Internal),
                    Inner::Invocation(ModelErrorKind::Internal),
                ),
                4 => {
                    f.response.invocation_id = id(88_100, nexa_domain::ModelInvocationId::new);
                    (
                        MODEL_INPUT_TOKENIZATION_V1,
                        ScriptedTokenizationOutcome::TokenCount(7),
                        ScriptedOutcome::Response(f.response.clone()),
                        Inner::ReportedUsage(Usage::Response(ModelErrorKind::IdentityMismatch)),
                    )
                }
                5 => {
                    f.response.reported_usage = Some(ModelUsage {
                        input_tokens: 6,
                        output_tokens: 1,
                    });
                    (
                        MODEL_INPUT_TOKENIZATION_V1,
                        ScriptedTokenizationOutcome::TokenCount(7),
                        ScriptedOutcome::Response(f.response.clone()),
                        Inner::ReportedUsage(Usage::InputTokenCountMismatch),
                    )
                }
                _ => (
                    MODEL_INPUT_TOKENIZATION_V1,
                    ScriptedTokenizationOutcome::TokenCount(7),
                    ScriptedOutcome::Response(f.response.clone()),
                    Inner::Admission(crate::admission::AdmissionError::MalformedSyntax),
                ),
            };
            let provider = Arc::new(DiagnosticProvider {
                inner: UncheckedScriptedProvider::new(f.descriptor.clone(), [provider_outcome]),
                endpoint: sentinels[7],
                credential: sentinels[8],
                private_diagnostic: sentinels[6],
                usage_adjacent: sentinels[4],
            });
            let registry = ModelRegistry::try_from_providers([
                provider.clone() as Arc<dyn LanguageModelProvider>
            ])
            .unwrap();
            let tokenizer = SentinelTokenizer {
                inner: ScriptedModelInputTokenizer::new(f.descriptor.clone(), [token_outcome])
                    .unwrap(),
                private_diagnostic: sentinels[5].into(),
            };
            let error = select_local_model_tokenize_invoke_validate_reported_usage_and_admit(
                &registry,
                f.request.invocation_id,
                &local_selection_requirements(),
                version,
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap_err();
            assert_eq!(
                error,
                Outer::UsageValidatedTokenizedInvocationAdmission(expected)
            );
            assert_eq!(
                tokenizer.inner.remaining().unwrap(),
                usize::from(mutation < 2)
            );
            assert_eq!(provider.inner.remaining(), usize::from(mutation < 3));
            assert_closed(error);
        }

        // Complete the remaining reachable leaves with the same real sentinel-bearing wrapper
        // call. These are kept explicit so diagnostics coverage cannot be inferred from a lower
        // level helper's tests.
        for mutation in 0..10 {
            let mut f = admission_fixture();
            f.context.tokenizer_profile_id = sentinels[2].into();
            f.response.output =
                RawModelOutput::new(format!("{} {} not json", sentinels[3], sentinels[4])).unwrap();
            let mut tokenizer_descriptor = f.descriptor.clone();
            let version = MODEL_INPUT_TOKENIZATION_V1;
            let mut outcomes = vec![ScriptedTokenizationOutcome::TokenCount(7)];
            let expected = match mutation {
                0 => {
                    f.compilation.compiled_bytes += 1;
                    Inner::Preflight(
                        crate::admission::AdmissionError::PromptAssociationReplayMismatch,
                    )
                }
                1 => {
                    f.authority.permitted_capabilities.clear();
                    Inner::Preflight(
                        crate::admission::AdmissionError::PolicyPedagogySafetyCapability,
                    )
                }
                2 => {
                    f.context.maximum_tokens = 0;
                    Inner::Preflight(crate::admission::AdmissionError::PlanningEvidenceProvenance)
                }
                3 => {
                    f.citations.maximum_citations = 0;
                    Inner::Preflight(crate::admission::AdmissionError::CitationGroundingReference)
                }
                4 => {
                    tokenizer_descriptor.provider_id =
                        id(88_401, nexa_domain::ModelProviderId::new);
                    Inner::TokenizationCapacity(Capacity::Tokenization(
                        ModelInputTokenizationError::InvalidDescriptor,
                    ))
                }
                5 => {
                    outcomes.clear();
                    Inner::TokenizationCapacity(Capacity::Tokenization(
                        ModelInputTokenizationError::ScriptExhausted,
                    ))
                }
                6 => {
                    outcomes = vec![ScriptedTokenizationOutcome::TokenCount(0)];
                    Inner::TokenizationCapacity(Capacity::Tokenization(
                        ModelInputTokenizationError::InvalidEvidence,
                    ))
                }
                7 => {
                    outcomes = vec![ScriptedTokenizationOutcome::TokenCount(u32::MAX)];
                    Inner::TokenizationCapacity(Capacity::TokenCapacity(
                        crate::tokenization::ModelRequestTokenCapacityError::ExactCapacity,
                    ))
                }
                8 => {
                    f.response.contract_version = ProtocolVersion::new(2, 0);
                    Inner::ReportedUsage(Usage::Response(ModelErrorKind::UnsupportedVersion))
                }
                _ => {
                    f.response.reported_usage = Some(ModelUsage {
                        input_tokens: 7,
                        output_tokens: local_selection_requirements().maximum_output_tokens + 1,
                    });
                    Inner::ReportedUsage(Usage::Response(ModelErrorKind::InvalidResponse))
                }
            };
            let provider = Arc::new(DiagnosticProvider {
                inner: UncheckedScriptedProvider::new(
                    f.descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                ),
                endpoint: sentinels[7],
                credential: sentinels[8],
                private_diagnostic: sentinels[6],
                usage_adjacent: sentinels[4],
            });
            let registry = ModelRegistry::try_from_providers([
                provider.clone() as Arc<dyn LanguageModelProvider>
            ])
            .unwrap();
            let tokenizer = SentinelTokenizer {
                inner: ScriptedModelInputTokenizer::new(tokenizer_descriptor, outcomes).unwrap(),
                private_diagnostic: sentinels[5].into(),
            };
            let error = select_local_model_tokenize_invoke_validate_reported_usage_and_admit(
                &registry,
                f.request.invocation_id,
                &local_selection_requirements(),
                version,
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap_err();
            assert_eq!(
                error,
                Outer::UsageValidatedTokenizedInvocationAdmission(expected),
                "diagnostic mutation {mutation}"
            );
            assert_eq!(
                tokenizer.inner.remaining().unwrap(),
                usize::from(mutation <= 4)
            );
            assert_eq!(provider.inner.remaining(), usize::from(mutation < 8));
            assert_closed(error);
        }
    }

    #[test]
    fn available_local_tokenized_selection_rejects_requirements_and_availability_first() {
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilityError, ModelAvailabilitySnapshot,
            ModelAvailabilityState,
        };
        use crate::generation::{
            select_available_local_model_tokenize_invoke_and_admit,
            AvailableLocalTokenizedInvocationAdmissionError,
        };
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::tokenization::{
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome, MODEL_INPUT_TOKENIZATION_V1,
        };
        use nexa_domain::{ModelId, ModelProviderId, ProtocolVersion};
        use std::sync::Arc;

        let f = admission_fixture();
        let provider = Arc::new(
            ScriptedModelProvider::new(
                f.descriptor.clone(),
                [ScriptedOutcome::Response(f.response.clone())],
            )
            .unwrap(),
        );
        let registry =
            ModelRegistry::try_from_providers([provider.clone() as Arc<dyn LanguageModelProvider>])
                .unwrap();
        let tokenizer = ScriptedModelInputTokenizer::new(
            f.descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(1)],
        )
        .unwrap();

        for mutation in 0..9 {
            let mut requirements = local_selection_requirements();
            match mutation {
                0 => requirements.contract_version = ProtocolVersion::new(2, 0),
                1 => requirements.maximum_output_tokens = 0,
                2 => requirements.required_capabilities.structured_output = false,
                3 => requirements.privacy_preference.clear(),
                4 => requirements.privacy_preference = vec![PrivacyClass::ApprovedRemote],
                5 => requirements.privacy_preference = vec![PrivacyClass::RestrictedRemote],
                6 => requirements
                    .privacy_preference
                    .push(PrivacyClass::ApprovedRemote),
                7 => requirements
                    .privacy_preference
                    .push(PrivacyClass::RestrictedRemote),
                _ => requirements
                    .privacy_preference
                    .push(PrivacyClass::LocalOnly),
            }
            assert_eq!(
                select_available_local_model_tokenize_invoke_and_admit(
                    &registry,
                    f.request.invocation_id,
                    &requirements,
                    &ModelAvailabilitySnapshot::new(vec![]).unwrap(),
                    MODEL_INPUT_TOKENIZATION_V1,
                    &tokenizer,
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                ),
                Err(AvailableLocalTokenizedInvocationAdmissionError::InvalidLocalOnlyRequirements)
            );
        }

        let unknown = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: id(9_991, ModelProviderId::new),
            model_id: id(9_991, ModelId::new),
            state: ModelAvailabilityState::Available,
        }])
        .unwrap();
        let mut unsupported = ModelAvailabilitySnapshot::new(vec![]).unwrap();
        unsupported.contract_version = ProtocolVersion::new(2, 0);
        let duplicate_entry = ModelAvailabilityEntry {
            provider_id: f.descriptor.provider_id,
            model_id: f.descriptor.model_id,
            state: ModelAvailabilityState::Available,
        };
        let duplicate = ModelAvailabilitySnapshot {
            contract_version: crate::availability::MODEL_AVAILABILITY_V1,
            entries: vec![duplicate_entry, duplicate_entry],
        };
        for (snapshot, expected) in [
            (unknown, ModelAvailabilityError::RegistryInconsistency),
            (
                unsupported,
                ModelAvailabilityError::UnsupportedAvailabilityVersion,
            ),
            (duplicate, ModelAvailabilityError::InvalidAvailability),
            (
                ModelAvailabilitySnapshot::new(vec![]).unwrap(),
                ModelAvailabilityError::Selection(
                    crate::selection::ModelSelectionError::NoEligibleModel,
                ),
            ),
        ] {
            assert_eq!(
                select_available_local_model_tokenize_invoke_and_admit(
                    &registry,
                    f.request.invocation_id,
                    &local_selection_requirements(),
                    &snapshot,
                    MODEL_INPUT_TOKENIZATION_V1,
                    &tokenizer,
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                ),
                Err(
                    AvailableLocalTokenizedInvocationAdmissionError::AvailabilitySelection(
                        expected
                    )
                )
            );
        }
        assert_eq!(tokenizer.remaining().unwrap(), 1);
        assert_eq!(provider.remaining(), 1);

        // An unavailable or missing local model never reaches tokenization or invocation.
        let unavailable = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: f.descriptor.provider_id,
            model_id: f.descriptor.model_id,
            state: ModelAvailabilityState::Unavailable,
        }])
        .unwrap();
        assert!(select_available_local_model_tokenize_invoke_and_admit(
            &registry,
            f.request.invocation_id,
            &local_selection_requirements(),
            &unavailable,
            MODEL_INPUT_TOKENIZATION_V1,
            &tokenizer,
            &f.compilation,
            &f.authority,
            &f.context,
            &f.citations,
        )
        .is_err());
        assert_eq!(tokenizer.remaining().unwrap(), 1);
        assert_eq!(provider.remaining(), 1);

        // Keep otherwise-valid identities type checked in this focused closed-error test.
        let _: ModelId = f.descriptor.model_id;
    }

    #[test]
    fn available_local_tokenized_selection_chooses_next_available_and_returns_exact_evidence() {
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::select_available_local_model_tokenize_invoke_and_admit;
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::tokenization::{
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome, MODEL_INPUT_TOKENIZATION_V1,
        };
        use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId};
        use std::sync::Arc;

        let mut f = admission_fixture();
        let invocation_id = id(9_980, ModelInvocationId::new);
        let token_count = f.descriptor.capabilities.context_window_tokens - 1000;
        let mut first_descriptor = f.descriptor.clone();
        first_descriptor.provider_id = id(9_970, ModelProviderId::new);
        first_descriptor.model_id = id(9_970, ModelId::new);
        let mut selected_descriptor = f.descriptor.clone();
        selected_descriptor.provider_id = id(9_971, ModelProviderId::new);
        selected_descriptor.model_id = id(9_971, ModelId::new);
        f.response.invocation_id = invocation_id;
        f.response.provider_id = selected_descriptor.provider_id;
        f.response.model_id = selected_descriptor.model_id;
        let mut selected_request = f.request.clone();
        selected_request.invocation_id = invocation_id;
        selected_request.provider_id = selected_descriptor.provider_id;
        selected_request.model_id = selected_descriptor.model_id;
        let expected_admission = crate::admission::admit_model_output(
            &selected_descriptor,
            &selected_request,
            &f.response,
            &f.compilation,
            &f.authority,
            &f.context,
            &f.citations,
        )
        .unwrap();
        let first = Arc::new(
            ScriptedModelProvider::new(
                first_descriptor.clone(),
                [ScriptedOutcome::Response(f.response.clone())],
            )
            .unwrap(),
        );
        let selected = Arc::new(
            ScriptedModelProvider::new(
                selected_descriptor.clone(),
                [ScriptedOutcome::Response(f.response.clone())],
            )
            .unwrap(),
        );
        let mut remote_descriptor = f.descriptor.clone();
        remote_descriptor.provider_id = id(9_972, ModelProviderId::new);
        remote_descriptor.model_id = id(9_972, ModelId::new);
        remote_descriptor.privacy_class = PrivacyClass::ApprovedRemote;
        let remote = Arc::new(
            ScriptedModelProvider::new(
                remote_descriptor.clone(),
                [ScriptedOutcome::Response(f.response.clone())],
            )
            .unwrap(),
        );
        let registry = ModelRegistry::try_from_providers([
            first.clone() as Arc<dyn LanguageModelProvider>,
            selected.clone() as Arc<dyn LanguageModelProvider>,
            remote.clone() as Arc<dyn LanguageModelProvider>,
        ])
        .unwrap();
        let availability = ModelAvailabilitySnapshot::new(vec![
            ModelAvailabilityEntry {
                provider_id: selected_descriptor.provider_id,
                model_id: selected_descriptor.model_id,
                state: ModelAvailabilityState::Available,
            },
            ModelAvailabilityEntry {
                provider_id: remote_descriptor.provider_id,
                model_id: remote_descriptor.model_id,
                state: ModelAvailabilityState::Available,
            },
        ])
        .unwrap();
        let tokenizer = ScriptedModelInputTokenizer::new(
            selected_descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(token_count)],
        )
        .unwrap();
        let result = select_available_local_model_tokenize_invoke_and_admit(
            &registry,
            invocation_id,
            &local_selection_requirements(),
            &availability,
            MODEL_INPUT_TOKENIZATION_V1,
            &tokenizer,
            &f.compilation,
            &f.authority,
            &f.context,
            &f.citations,
        )
        .unwrap();
        assert_eq!(result.admission, expected_admission);
        result
            .tokenization_evidence
            .validate_for(&selected_descriptor, &f.compilation.model_input)
            .unwrap();
        assert_eq!(result.tokenization_evidence.input_token_count, token_count);
        assert_eq!(tokenizer.remaining().unwrap(), 0);
        assert_eq!(selected.remaining(), 0);
        assert_eq!(first.remaining(), 1);
        assert_eq!(remote.remaining(), 1);
    }

    #[test]
    fn available_local_tokenized_selection_preserves_exact_tokenization_failures() {
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::{
            select_available_local_model_tokenize_invoke_and_admit,
            AvailableLocalTokenizedInvocationAdmissionError as Outer,
            TokenizedInvocationAdmissionError as Inner,
        };
        use crate::model::{
            LanguageModelProvider, RawModelOutput, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::tokenization::{
            ModelInputTokenizationError, ModelRequestTokenCapacityError,
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome,
            TokenizeAndValidateModelRequestCapacityError as Capacity, MODEL_INPUT_TOKENIZATION_V1,
        };
        use nexa_domain::{ModelId, ProtocolVersion};
        use std::sync::Arc;

        for (wrong_version, expected) in [
            (true, ModelInputTokenizationError::UnsupportedVersion),
            (false, ModelInputTokenizationError::InvalidDescriptor),
        ] {
            let mut f = admission_fixture();
            f.context.tokenizer_profile_id = "knowledge-private-sentinel".into();
            f.response.output = RawModelOutput::new("model-output-private-sentinel").unwrap();
            let provider = Arc::new(
                ScriptedModelProvider::new(
                    f.descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                provider.clone() as Arc<dyn LanguageModelProvider>
            ])
            .unwrap();
            let availability = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
                provider_id: f.descriptor.provider_id,
                model_id: f.descriptor.model_id,
                state: ModelAvailabilityState::Available,
            }])
            .unwrap();
            let mut tokenizer_descriptor = f.descriptor.clone();
            if !wrong_version {
                tokenizer_descriptor.model_id = id(10_001, ModelId::new);
            }
            let tokenizer = ScriptedModelInputTokenizer::new(
                tokenizer_descriptor,
                [ScriptedTokenizationOutcome::TokenCount(1)],
            )
            .unwrap();
            let version = if wrong_version {
                ProtocolVersion::new(2, 0)
            } else {
                MODEL_INPUT_TOKENIZATION_V1
            };

            let error = select_available_local_model_tokenize_invoke_and_admit(
                &registry,
                f.request.invocation_id,
                &local_selection_requirements(),
                &availability,
                version,
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap_err();
            assert_eq!(
                error,
                Outer::TokenizedInvocationAdmission(Inner::TokenizationCapacity(
                    Capacity::Tokenization(expected)
                ))
            );
            assert_eq!(tokenizer.remaining().unwrap(), 1);
            assert_eq!(provider.remaining(), 1);
            assert_content_free_available_local_error(error);
        }

        for (outcome, expected) in [
            (
                ScriptedTokenizationOutcome::Error,
                Capacity::Tokenization(ModelInputTokenizationError::TokenizerFailure),
            ),
            (
                ScriptedTokenizationOutcome::TokenCount(
                    admission_fixture()
                        .descriptor
                        .capabilities
                        .context_window_tokens
                        - local_selection_requirements().maximum_output_tokens
                        + 1,
                ),
                Capacity::TokenCapacity(ModelRequestTokenCapacityError::ExactCapacity),
            ),
        ] {
            let mut f = admission_fixture();
            f.context.tokenizer_profile_id = "knowledge-private-sentinel".into();
            f.response.output = RawModelOutput::new("model-output-private-sentinel").unwrap();
            let provider = Arc::new(
                ScriptedModelProvider::new(
                    f.descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                provider.clone() as Arc<dyn LanguageModelProvider>
            ])
            .unwrap();
            let availability = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
                provider_id: f.descriptor.provider_id,
                model_id: f.descriptor.model_id,
                state: ModelAvailabilityState::Available,
            }])
            .unwrap();
            let tokenizer = SentinelTokenizer {
                inner: ScriptedModelInputTokenizer::new(f.descriptor.clone(), [outcome]).unwrap(),
                private_diagnostic: "tokenizer-private-sentinel".into(),
            };
            let error = select_available_local_model_tokenize_invoke_and_admit(
                &registry,
                f.request.invocation_id,
                &local_selection_requirements(),
                &availability,
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap_err();
            assert_eq!(
                error,
                Outer::TokenizedInvocationAdmission(Inner::TokenizationCapacity(expected))
            );
            assert_eq!(tokenizer.inner.remaining().unwrap(), 0);
            assert_eq!(provider.remaining(), 1);
            assert_content_free_available_local_error(error);
        }
    }

    fn assert_content_free_available_local_error(
        error: crate::generation::AvailableLocalTokenizedInvocationAdmissionError,
    ) {
        let diagnostics = format!("{error:?} {error}");
        for sentinel in [
            "model-output-private-sentinel",
            "prompt-private-sentinel",
            "learner-private-sentinel",
            "knowledge-private-sentinel",
            "endpoint-private-sentinel",
            "credential-private-sentinel",
            "tokenizer-private-sentinel",
            "provider-private-sentinel",
        ] {
            assert!(!diagnostics.contains(sentinel), "leaked {sentinel}");
        }
    }

    #[test]
    fn available_local_tokenized_selection_consumes_only_selected_dependencies_on_late_failures() {
        use crate::admission::AdmissionError;
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::{
            select_available_local_model_tokenize_invoke_and_admit,
            AvailableLocalTokenizedInvocationAdmissionError as Outer,
            TokenizedInvocationAdmissionError as Inner,
        };
        use crate::model::{
            LanguageModelProvider, ModelErrorKind, PrivacyClass, RawModelOutput,
            ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::tokenization::{
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome, MODEL_INPUT_TOKENIZATION_V1,
        };
        use nexa_domain::{ModelId, ModelProviderId};
        use std::sync::Arc;

        for admission_failure in [false, true] {
            let mut f = admission_fixture();
            f.context.tokenizer_profile_id = "knowledge-private-sentinel".into();
            let selected_descriptor = f.descriptor.clone();
            let selected_outcome = if admission_failure {
                let mut response = f.response.clone();
                response.output = RawModelOutput::new("model-output-private-sentinel").unwrap();
                ScriptedOutcome::Response(response)
            } else {
                ScriptedOutcome::Error(ModelErrorKind::Unavailable)
            };
            let selected = Arc::new(SentinelProvider {
                inner: ScriptedModelProvider::new(
                    selected_descriptor.clone(),
                    [
                        selected_outcome,
                        ScriptedOutcome::Error(ModelErrorKind::Internal),
                    ],
                )
                .unwrap(),
                endpoint: "endpoint-private-sentinel".into(),
                credential: "credential-private-sentinel".into(),
                private_diagnostic: "provider-private-sentinel".into(),
            });
            let mut other_descriptor = f.descriptor.clone();
            other_descriptor.provider_id = id(10_010, ModelProviderId::new);
            other_descriptor.model_id = id(10_010, ModelId::new);
            let other = Arc::new(
                ScriptedModelProvider::new(
                    other_descriptor,
                    [ScriptedOutcome::Error(ModelErrorKind::Internal)],
                )
                .unwrap(),
            );
            let mut remote_descriptor = f.descriptor.clone();
            remote_descriptor.provider_id = id(10_011, ModelProviderId::new);
            remote_descriptor.model_id = id(10_011, ModelId::new);
            remote_descriptor.privacy_class = PrivacyClass::ApprovedRemote;
            let remote = Arc::new(
                ScriptedModelProvider::new(
                    remote_descriptor.clone(),
                    [ScriptedOutcome::Error(ModelErrorKind::Internal)],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                remote.clone() as Arc<dyn LanguageModelProvider>,
                other.clone() as Arc<dyn LanguageModelProvider>,
                selected.clone() as Arc<dyn LanguageModelProvider>,
            ])
            .unwrap();
            let availability = ModelAvailabilitySnapshot::new(vec![
                ModelAvailabilityEntry {
                    provider_id: selected_descriptor.provider_id,
                    model_id: selected_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
                ModelAvailabilityEntry {
                    provider_id: other.descriptor().provider_id,
                    model_id: other.descriptor().model_id,
                    state: ModelAvailabilityState::Available,
                },
                ModelAvailabilityEntry {
                    provider_id: remote_descriptor.provider_id,
                    model_id: remote_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
            ])
            .unwrap();
            let tokenizer = SentinelTokenizer {
                inner: ScriptedModelInputTokenizer::new(
                    selected_descriptor,
                    [ScriptedTokenizationOutcome::TokenCount(1)],
                )
                .unwrap(),
                private_diagnostic: "tokenizer-private-sentinel".into(),
            };

            let error = select_available_local_model_tokenize_invoke_and_admit(
                &registry,
                f.request.invocation_id,
                &local_selection_requirements(),
                &availability,
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap_err();
            let expected = if admission_failure {
                Outer::TokenizedInvocationAdmission(Inner::Admission(
                    AdmissionError::MalformedSyntax,
                ))
            } else {
                Outer::TokenizedInvocationAdmission(Inner::Invocation(ModelErrorKind::Unavailable))
            };
            assert_eq!(error, expected);
            assert_eq!(tokenizer.inner.remaining().unwrap(), 0);
            assert_eq!(selected.inner.remaining(), 1);
            assert_eq!(other.remaining(), 1);
            assert_eq!(remote.remaining(), 1);
            assert_content_free_available_local_error(error);
        }
    }

    #[test]
    fn local_selection_composes_canonical_selection_request_invocation_and_admission() {
        use crate::generation::select_local_model_invoke_and_admit;
        use crate::model::{LanguageModelProvider, ScriptedModelProvider, ScriptedOutcome};
        use crate::registry::ModelRegistry;
        use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId};
        use std::sync::Arc;

        for reverse in [false, true] {
            let f = admission_fixture();
            let invocation_id = id(800, ModelInvocationId::new);
            let mut selected_descriptor = f.descriptor.clone();
            selected_descriptor.provider_id = id(40, ModelProviderId::new);
            selected_descriptor.model_id = id(41, ModelId::new);
            let mut selected_response = f.response.clone();
            selected_response.invocation_id = invocation_id;
            selected_response.provider_id = selected_descriptor.provider_id;
            selected_response.model_id = selected_descriptor.model_id;
            let selected = Arc::new(
                ScriptedModelProvider::new(
                    selected_descriptor.clone(),
                    [ScriptedOutcome::Response(selected_response)],
                )
                .unwrap(),
            );
            let mut other_descriptor = f.descriptor.clone();
            other_descriptor.provider_id = id(60, ModelProviderId::new);
            other_descriptor.model_id = id(61, ModelId::new);
            let other = Arc::new(
                ScriptedModelProvider::new(
                    other_descriptor,
                    [ScriptedOutcome::Error(
                        crate::model::ModelErrorKind::Internal,
                    )],
                )
                .unwrap(),
            );
            let mut remote_descriptor = f.descriptor.clone();
            remote_descriptor.provider_id = id(1, ModelProviderId::new);
            remote_descriptor.model_id = id(1, ModelId::new);
            remote_descriptor.privacy_class = crate::model::PrivacyClass::ApprovedRemote;
            let remote = Arc::new(
                ScriptedModelProvider::new(
                    remote_descriptor,
                    [ScriptedOutcome::Error(
                        crate::model::ModelErrorKind::Internal,
                    )],
                )
                .unwrap(),
            );
            let mut providers: Vec<Arc<dyn LanguageModelProvider>> =
                vec![selected.clone(), other.clone(), remote.clone()];
            if reverse {
                providers.reverse();
            }
            let registry = ModelRegistry::try_from_providers(providers).unwrap();
            let result = select_local_model_invoke_and_admit(
                &registry,
                invocation_id,
                &local_selection_requirements(),
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap();
            assert_eq!(result.evidence.provider_id, selected_descriptor.provider_id);
            assert_eq!(result.evidence.model_id, selected_descriptor.model_id);
            assert_eq!(result.evidence.invocation_id, invocation_id);
            assert_eq!(
                result.evidence.prompt_compilation_replay_anchor,
                f.compilation.replay_anchor
            );
            assert_eq!(selected.remaining(), 0);
            assert_eq!(other.remaining(), 1);
            assert_eq!(remote.remaining(), 1);
        }
    }

    #[test]
    fn local_selection_rejects_nonlocal_and_malformed_requirements_without_invocation() {
        use crate::generation::{
            select_local_model_invoke_and_admit, SelectedInvocationAdmissionError,
        };
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use nexa_domain::{ModelInvocationId, ProtocolVersion};
        use std::sync::Arc;

        let f = admission_fixture();
        for mutation in 0..9 {
            let provider = Arc::new(
                ScriptedModelProvider::new(
                    f.descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let handle: Arc<dyn LanguageModelProvider> = provider.clone();
            let registry = ModelRegistry::try_from_providers([handle]).unwrap();
            let mut requirements = local_selection_requirements();
            match mutation {
                0 => requirements.contract_version = ProtocolVersion::new(2, 0),
                1 => requirements.maximum_output_tokens = 0,
                2 => requirements.required_capabilities.structured_output = false,
                3 => requirements.privacy_preference.clear(),
                4 => requirements.privacy_preference = vec![PrivacyClass::ApprovedRemote],
                5 => requirements.privacy_preference = vec![PrivacyClass::RestrictedRemote],
                6 => requirements
                    .privacy_preference
                    .push(PrivacyClass::ApprovedRemote),
                7 => requirements
                    .privacy_preference
                    .push(PrivacyClass::RestrictedRemote),
                _ => requirements
                    .privacy_preference
                    .push(PrivacyClass::LocalOnly),
            }
            assert_eq!(
                select_local_model_invoke_and_admit(
                    &registry,
                    id(801, ModelInvocationId::new),
                    &requirements,
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                ),
                Err(SelectedInvocationAdmissionError::InvalidLocalOnlyRequirements)
            );
            assert_eq!(provider.remaining(), 1, "mutation {mutation}");
        }
    }

    #[test]
    fn local_selection_and_single_attempt_failures_never_fallback() {
        use crate::generation::{
            select_local_model_invoke_and_admit, InvocationAdmissionError,
            SelectedInvocationAdmissionError,
        };
        use crate::model::{
            LanguageModelProvider, ModelErrorKind, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::selection::ModelSelectionError;
        use nexa_domain::{ModelInvocationId, ModelProviderId};
        use std::sync::Arc;

        let f = admission_fixture();
        let invocation_id = id(802, ModelInvocationId::new);
        let empty = ModelRegistry::try_from_providers(std::iter::empty()).unwrap();
        assert_eq!(
            select_local_model_invoke_and_admit(
                &empty,
                invocation_id,
                &local_selection_requirements(),
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations
            ),
            Err(SelectedInvocationAdmissionError::Selection(
                ModelSelectionError::NoEligibleModel
            ))
        );

        let mut first_descriptor = f.descriptor.clone();
        first_descriptor.provider_id = id(30, ModelProviderId::new);
        let first = Arc::new(
            ScriptedModelProvider::new(
                first_descriptor,
                [ScriptedOutcome::Error(ModelErrorKind::Unavailable)],
            )
            .unwrap(),
        );
        let mut second_descriptor = f.descriptor.clone();
        second_descriptor.provider_id = id(31, ModelProviderId::new);
        let second = Arc::new(
            ScriptedModelProvider::new(
                second_descriptor,
                [ScriptedOutcome::Response(f.response.clone())],
            )
            .unwrap(),
        );
        let registry = ModelRegistry::try_from_providers([
            first.clone() as Arc<dyn LanguageModelProvider>,
            second.clone() as Arc<dyn LanguageModelProvider>,
        ])
        .unwrap();
        assert_eq!(
            select_local_model_invoke_and_admit(
                &registry,
                invocation_id,
                &local_selection_requirements(),
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations
            ),
            Err(SelectedInvocationAdmissionError::InvocationAdmission(
                InvocationAdmissionError::Invocation(ModelErrorKind::Unavailable)
            ))
        );
        assert_eq!(first.remaining(), 0);
        assert_eq!(second.remaining(), 1);
    }

    #[test]
    fn local_selection_ineligible_and_remote_only_registries_consume_nothing() {
        use crate::generation::{
            select_local_model_invoke_and_admit, SelectedInvocationAdmissionError,
        };
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::selection::ModelSelectionError;
        use nexa_domain::{ModelInvocationId, ModelProviderId};
        use std::sync::Arc;

        for mutation in 0..4 {
            let f = admission_fixture();
            let mut descriptor = f.descriptor.clone();
            match mutation {
                0 => descriptor.privacy_class = PrivacyClass::ApprovedRemote,
                1 => descriptor.capabilities.structured_output = false,
                2 => descriptor.capabilities.maximum_output_tokens = 999,
                _ => {
                    descriptor.capabilities.context_window_tokens =
                        f.compilation.model_input.as_str().len() as u32 + 999
                }
            }
            descriptor.provider_id = id(70 + mutation, ModelProviderId::new);
            let provider = Arc::new(
                ScriptedModelProvider::new(
                    descriptor,
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                provider.clone() as Arc<dyn LanguageModelProvider>
            ])
            .unwrap();
            assert_eq!(
                select_local_model_invoke_and_admit(
                    &registry,
                    id(804, ModelInvocationId::new),
                    &local_selection_requirements(),
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                ),
                Err(SelectedInvocationAdmissionError::Selection(
                    ModelSelectionError::NoEligibleModel
                )),
                "mutation {mutation}"
            );
            assert_eq!(provider.remaining(), 1, "mutation {mutation}");
        }
    }

    #[test]
    fn local_selection_preflight_association_matrix_consumes_no_provider() {
        use crate::admission::AdmissionError;
        use crate::generation::{
            select_local_model_invoke_and_admit, InvocationAdmissionError,
            SelectedInvocationAdmissionError,
        };
        use crate::model::{
            LanguageModelProvider, ModelInput, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use nexa_domain::{
            CitationSetId, ContextPackageId, ModelId, ModelInvocationId, ModelProviderId,
            ProtocolVersion, StudentId,
        };
        use std::sync::Arc;

        for mutation in 0..17 {
            let mut f = admission_fixture();
            let secret = "distinctive-selected-composition-secret";
            let expected = match mutation {
                0 => {
                    f.compilation.manifest[0].content_bytes += 1;
                    AdmissionError::PromptAssociationReplayMismatch
                }
                1 => {
                    f.compilation.compiled_bytes += 1;
                    AdmissionError::PromptAssociationReplayMismatch
                }
                2 => {
                    f.compilation.replay_anchor = "a".repeat(64);
                    AdmissionError::PromptAssociationReplayMismatch
                }
                3 => {
                    f.compilation.model_input = ModelInput::new(secret).unwrap();
                    AdmissionError::PromptAssociationReplayMismatch
                }
                4 => {
                    f.compilation.contract_version = ProtocolVersion::new(2, 0);
                    AdmissionError::UnsupportedVersion
                }
                5 => {
                    f.compilation.prompt_package_version = ProtocolVersion::new(2, 0);
                    AdmissionError::UnsupportedVersion
                }
                6 => {
                    f.compilation.context_builder_version = ProtocolVersion::new(2, 0);
                    AdmissionError::UnsupportedVersion
                }
                7 => {
                    f.compilation.output_schema_version = ProtocolVersion::new(2, 0);
                    AdmissionError::UnsupportedVersion
                }
                8 => {
                    f.authority.response_policy_version = ProtocolVersion::new(2, 0);
                    AdmissionError::UnsupportedVersion
                }
                9 => {
                    f.authority.permitted_capabilities.clear();
                    AdmissionError::PolicyPedagogySafetyCapability
                }
                10 => {
                    f.authority.evidence.scope.student_id = id(820, StudentId::new);
                    AdmissionError::PlanningEvidenceProvenance
                }
                11 => {
                    f.authority.context_package_id = id(821, ContextPackageId::new);
                    AdmissionError::PlanningEvidenceProvenance
                }
                12 => {
                    f.authority.citation_set_id = id(822, CitationSetId::new);
                    AdmissionError::PlanningEvidenceProvenance
                }
                13 => {
                    f.context.governance_policy_version = ProtocolVersion::new(2, 0);
                    AdmissionError::PlanningEvidenceProvenance
                }
                14 => {
                    f.context.maximum_tokens = 0;
                    AdmissionError::PlanningEvidenceProvenance
                }
                15 => {
                    f.citations.citation_policy_version = ProtocolVersion::new(2, 0);
                    AdmissionError::CitationGroundingReference
                }
                _ => {
                    f.citations.maximum_citations = 0;
                    AdmissionError::CitationGroundingReference
                }
            };
            let selected = Arc::new(
                ScriptedModelProvider::new(
                    f.descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let mut other_descriptor = f.descriptor.clone();
            other_descriptor.provider_id = id(900, ModelProviderId::new);
            other_descriptor.model_id = id(901, ModelId::new);
            let other = Arc::new(
                ScriptedModelProvider::new(
                    other_descriptor,
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let mut remote_descriptor = f.descriptor.clone();
            remote_descriptor.provider_id = id(902, ModelProviderId::new);
            remote_descriptor.model_id = id(903, ModelId::new);
            remote_descriptor.privacy_class = PrivacyClass::ApprovedRemote;
            let remote = Arc::new(
                ScriptedModelProvider::new(
                    remote_descriptor,
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                selected.clone() as Arc<dyn LanguageModelProvider>,
                other.clone() as Arc<dyn LanguageModelProvider>,
                remote.clone() as Arc<dyn LanguageModelProvider>,
            ])
            .unwrap();
            let error = select_local_model_invoke_and_admit(
                &registry,
                id(803, ModelInvocationId::new),
                &local_selection_requirements(),
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap_err();
            assert_eq!(
                error,
                SelectedInvocationAdmissionError::InvocationAdmission(
                    InvocationAdmissionError::Preflight(expected)
                ),
                "mutation {mutation}"
            );
            assert_eq!(selected.remaining(), 1, "mutation {mutation}");
            assert_eq!(other.remaining(), 1, "mutation {mutation}");
            assert_eq!(remote.remaining(), 1, "mutation {mutation}");
            let diagnostic = format!("{error} {error:?}");
            for sensitive in [secret, "distinctive private platform prompt"] {
                assert!(!diagnostic.contains(sensitive), "mutation {mutation}");
            }
        }
    }

    #[test]
    fn local_selection_post_invocation_admission_failures_are_single_attempt() {
        use crate::admission::AdmissionError;
        use crate::generation::{
            select_local_model_invoke_and_admit, InvocationAdmissionError,
            SelectedInvocationAdmissionError,
        };
        use crate::model::{
            FinishReason, LanguageModelProvider, ModelErrorKind, PrivacyClass, RawModelOutput,
            ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId};
        use std::sync::Arc;

        for mutation in 0..5 {
            let mut f = admission_fixture();
            let invocation_id = id(805, ModelInvocationId::new);
            f.response.invocation_id = invocation_id;
            f.response.provider_id = f.descriptor.provider_id;
            f.response.model_id = f.descriptor.model_id;
            let expected = match mutation {
                0 => {
                    f.response.output = RawModelOutput::new("not json").unwrap();
                    AdmissionError::MalformedSyntax
                }
                1 => {
                    f.response.output =
                        RawModelOutput::new("{\"candidate_schema_version\":\"1.0\",\"sections\":[")
                            .unwrap();
                    AdmissionError::MalformedSyntax
                }
                2 => {
                    f.response.output = RawModelOutput::new(
                        "{\"candidate_schema_version\":\"1.0\",\"sections\":[]} trailing",
                    )
                    .unwrap();
                    AdmissionError::MalformedSyntax
                }
                3 => {
                    f.set_candidate(json!({"candidate_schema_version":"1.0", "sections":[f.section.clone()], "unknown":"closed"}));
                    AdmissionError::InvalidCandidateSchema
                }
                4 => {
                    f.response.finish_reason = FinishReason::OutputLimit;
                    AdmissionError::IncompleteOutput
                }
                _ => unreachable!(),
            };
            let selected = Arc::new(
                ScriptedModelProvider::new(
                    f.descriptor.clone(),
                    [
                        ScriptedOutcome::Response(f.response.clone()),
                        ScriptedOutcome::Error(ModelErrorKind::Internal),
                    ],
                )
                .unwrap(),
            );
            let mut other_descriptor = f.descriptor.clone();
            other_descriptor.provider_id = id(910, ModelProviderId::new);
            other_descriptor.model_id = id(911, ModelId::new);
            let other = Arc::new(
                ScriptedModelProvider::new(
                    other_descriptor,
                    [ScriptedOutcome::Error(ModelErrorKind::Internal)],
                )
                .unwrap(),
            );
            let mut remote_descriptor = f.descriptor.clone();
            remote_descriptor.provider_id = id(912, ModelProviderId::new);
            remote_descriptor.model_id = id(913, ModelId::new);
            remote_descriptor.privacy_class = PrivacyClass::ApprovedRemote;
            let remote = Arc::new(
                ScriptedModelProvider::new(
                    remote_descriptor,
                    [ScriptedOutcome::Error(ModelErrorKind::Internal)],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                selected.clone() as Arc<dyn LanguageModelProvider>,
                other.clone() as Arc<dyn LanguageModelProvider>,
                remote.clone() as Arc<dyn LanguageModelProvider>,
            ])
            .unwrap();
            let error = select_local_model_invoke_and_admit(
                &registry,
                invocation_id,
                &local_selection_requirements(),
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap_err();
            assert_eq!(
                error,
                SelectedInvocationAdmissionError::InvocationAdmission(
                    InvocationAdmissionError::Admission(expected)
                ),
                "mutation {mutation}"
            );
            assert_eq!(selected.remaining(), 1, "mutation {mutation}");
            assert_eq!(other.remaining(), 1, "mutation {mutation}");
            assert_eq!(remote.remaining(), 1, "mutation {mutation}");
            let diagnostic = format!("{error} {error:?}");
            for sensitive in [
                "not json",
                "candidate_schema_version",
                "trailing",
                "closed",
                "distinctive private platform prompt",
            ] {
                assert!(!diagnostic.contains(sensitive), "mutation {mutation}");
            }
        }
    }

    #[test]
    fn local_selection_response_identity_mismatch_reaches_admission_once() {
        use crate::admission::AdmissionError;
        use crate::generation::{
            select_local_model_invoke_and_admit, InvocationAdmissionError,
            SelectedInvocationAdmissionError,
        };
        use crate::model::{
            LanguageModelProvider, ModelErrorKind, PrivacyClass, ScriptedModelProvider,
            ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId};
        use std::sync::Arc;

        let mut f = admission_fixture();
        let invocation_id = id(806, ModelInvocationId::new);
        f.response.invocation_id = id(807, ModelInvocationId::new);
        let selected = Arc::new(CountingProvider::new(&f));
        let mut other_descriptor = f.descriptor.clone();
        other_descriptor.provider_id = id(920, ModelProviderId::new);
        other_descriptor.model_id = id(921, ModelId::new);
        let other = Arc::new(
            ScriptedModelProvider::new(
                other_descriptor,
                [ScriptedOutcome::Error(ModelErrorKind::Internal)],
            )
            .unwrap(),
        );
        let mut remote_descriptor = f.descriptor.clone();
        remote_descriptor.provider_id = id(922, ModelProviderId::new);
        remote_descriptor.model_id = id(923, ModelId::new);
        remote_descriptor.privacy_class = PrivacyClass::ApprovedRemote;
        let remote = Arc::new(
            ScriptedModelProvider::new(
                remote_descriptor,
                [ScriptedOutcome::Error(ModelErrorKind::Internal)],
            )
            .unwrap(),
        );
        let registry = ModelRegistry::try_from_providers([
            selected.clone() as Arc<dyn LanguageModelProvider>,
            other.clone() as Arc<dyn LanguageModelProvider>,
            remote.clone() as Arc<dyn LanguageModelProvider>,
        ])
        .unwrap();
        let error = select_local_model_invoke_and_admit(
            &registry,
            invocation_id,
            &local_selection_requirements(),
            &f.compilation,
            &f.authority,
            &f.context,
            &f.citations,
        )
        .unwrap_err();
        assert_eq!(
            error,
            SelectedInvocationAdmissionError::InvocationAdmission(
                InvocationAdmissionError::Admission(AdmissionError::ModelResponseIdentityMismatch)
            )
        );
        assert_eq!(selected.calls(), 1);
        assert_eq!(other.remaining(), 1);
        assert_eq!(remote.remaining(), 1);
        assert!(!format!("{error} {error:?}").contains("distinctive private"));
    }

    #[test]
    fn available_local_selection_excludes_unavailable_canonical_first_and_admits_once() {
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::select_available_local_model_invoke_and_admit;
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId};
        use std::sync::Arc;

        for reverse in [false, true] {
            let f = admission_fixture();
            let invocation_id = id(930, ModelInvocationId::new);
            let mut unavailable_descriptor = f.descriptor.clone();
            unavailable_descriptor.provider_id = id(10, ModelProviderId::new);
            unavailable_descriptor.model_id = id(10, ModelId::new);
            let unavailable = Arc::new(
                ScriptedModelProvider::new(
                    unavailable_descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let mut selected_descriptor = f.descriptor.clone();
            selected_descriptor.provider_id = id(20, ModelProviderId::new);
            selected_descriptor.model_id = id(20, ModelId::new);
            let mut selected_response = f.response.clone();
            selected_response.provider_id = selected_descriptor.provider_id;
            selected_response.model_id = selected_descriptor.model_id;
            selected_response.invocation_id = invocation_id;
            let selected = Arc::new(
                ScriptedModelProvider::new(
                    selected_descriptor.clone(),
                    [ScriptedOutcome::Response(selected_response)],
                )
                .unwrap(),
            );
            let mut remote_descriptor = f.descriptor.clone();
            remote_descriptor.provider_id = id(1, ModelProviderId::new);
            remote_descriptor.model_id = id(1, ModelId::new);
            remote_descriptor.privacy_class = PrivacyClass::ApprovedRemote;
            let remote = Arc::new(
                ScriptedModelProvider::new(
                    remote_descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let mut providers: Vec<Arc<dyn LanguageModelProvider>> =
                vec![unavailable.clone(), selected.clone(), remote.clone()];
            if reverse {
                providers.reverse();
            }
            let registry = ModelRegistry::try_from_providers(providers).unwrap();
            let availability = ModelAvailabilitySnapshot::new(vec![
                ModelAvailabilityEntry {
                    provider_id: unavailable_descriptor.provider_id,
                    model_id: unavailable_descriptor.model_id,
                    state: ModelAvailabilityState::Unavailable,
                },
                ModelAvailabilityEntry {
                    provider_id: selected_descriptor.provider_id,
                    model_id: selected_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
                ModelAvailabilityEntry {
                    provider_id: remote_descriptor.provider_id,
                    model_id: remote_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
            ])
            .unwrap();
            let result = select_available_local_model_invoke_and_admit(
                &registry,
                invocation_id,
                &local_selection_requirements(),
                &availability,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap();
            assert_eq!(
                (result.evidence.provider_id, result.evidence.model_id),
                (
                    selected_descriptor.provider_id,
                    selected_descriptor.model_id
                )
            );
            assert_eq!(result.evidence.invocation_id, invocation_id);
            assert_eq!(
                result.evidence.prompt_compilation_replay_anchor,
                f.compilation.replay_anchor
            );
            assert_eq!(selected.remaining(), 0);
            assert_eq!(unavailable.remaining(), 1);
            assert_eq!(remote.remaining(), 1);
        }
    }

    #[test]
    fn available_local_selection_rejects_invalid_explicit_local_requirements_before_consumption() {
        use crate::availability::ModelAvailabilitySnapshot;
        use crate::generation::{
            select_available_local_model_invoke_and_admit, AvailableLocalInvocationAdmissionError,
        };
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use nexa_domain::{ModelInvocationId, ProtocolVersion};
        use std::sync::Arc;

        for mutation in 0..9 {
            let f = admission_fixture();
            let provider = Arc::new(
                ScriptedModelProvider::new(
                    f.descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                provider.clone() as Arc<dyn LanguageModelProvider>
            ])
            .unwrap();
            let mut requirements = local_selection_requirements();
            match mutation {
                0 => requirements.contract_version = ProtocolVersion::new(2, 0),
                1 => requirements.maximum_output_tokens = 0,
                2 => requirements.required_capabilities.structured_output = false,
                3 => requirements.privacy_preference.clear(),
                4 => requirements.privacy_preference = vec![PrivacyClass::ApprovedRemote],
                5 => requirements.privacy_preference = vec![PrivacyClass::RestrictedRemote],
                6 => requirements
                    .privacy_preference
                    .push(PrivacyClass::ApprovedRemote),
                7 => requirements
                    .privacy_preference
                    .push(PrivacyClass::RestrictedRemote),
                _ => requirements
                    .privacy_preference
                    .push(PrivacyClass::LocalOnly),
            }
            assert_eq!(
                select_available_local_model_invoke_and_admit(
                    &registry,
                    id(931, ModelInvocationId::new),
                    &requirements,
                    &ModelAvailabilitySnapshot::new(vec![]).unwrap(),
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                ),
                Err(AvailableLocalInvocationAdmissionError::InvalidLocalOnlyRequirements)
            );
            assert_eq!(provider.remaining(), 1);
        }
    }

    #[test]
    fn available_local_selection_preserves_nested_availability_errors_and_non_consumption() {
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilityError, ModelAvailabilitySnapshot,
            ModelAvailabilityState,
        };
        use crate::generation::{
            select_available_local_model_invoke_and_admit, AvailableLocalInvocationAdmissionError,
        };
        use crate::model::{LanguageModelProvider, ScriptedModelProvider, ScriptedOutcome};
        use crate::registry::ModelRegistry;
        use crate::selection::ModelSelectionError;
        use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId, ProtocolVersion};
        use std::sync::Arc;

        let f = admission_fixture();
        let provider = Arc::new(
            ScriptedModelProvider::new(
                f.descriptor.clone(),
                [ScriptedOutcome::Response(f.response.clone())],
            )
            .unwrap(),
        );
        let registry =
            ModelRegistry::try_from_providers([provider.clone() as Arc<dyn LanguageModelProvider>])
                .unwrap();
        let omitted = ModelAvailabilitySnapshot::new(vec![]).unwrap();
        let unavailable = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: f.descriptor.provider_id,
            model_id: f.descriptor.model_id,
            state: ModelAvailabilityState::Unavailable,
        }])
        .unwrap();
        let unknown = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: id(999, ModelProviderId::new),
            model_id: id(999, ModelId::new),
            state: ModelAvailabilityState::Available,
        }])
        .unwrap();
        let mut unsupported = omitted.clone();
        unsupported.contract_version = ProtocolVersion::new(2, 0);
        let duplicate = ModelAvailabilitySnapshot {
            contract_version: crate::availability::MODEL_AVAILABILITY_V1,
            entries: vec![unavailable.entries[0], unavailable.entries[0]],
        };
        let mut noncanonical = duplicate.clone();
        noncanonical.entries = vec![
            ModelAvailabilityEntry {
                provider_id: id(3, ModelProviderId::new),
                model_id: id(3, ModelId::new),
                state: ModelAvailabilityState::Available,
            },
            ModelAvailabilityEntry {
                provider_id: id(2, ModelProviderId::new),
                model_id: id(2, ModelId::new),
                state: ModelAvailabilityState::Available,
            },
        ];
        for (snapshot, expected) in [
            (
                omitted,
                ModelAvailabilityError::Selection(ModelSelectionError::NoEligibleModel),
            ),
            (
                unavailable,
                ModelAvailabilityError::Selection(ModelSelectionError::NoEligibleModel),
            ),
            (unknown, ModelAvailabilityError::RegistryInconsistency),
            (
                unsupported,
                ModelAvailabilityError::UnsupportedAvailabilityVersion,
            ),
            (duplicate, ModelAvailabilityError::InvalidAvailability),
            (noncanonical, ModelAvailabilityError::InvalidAvailability),
        ] {
            assert_eq!(
                select_available_local_model_invoke_and_admit(
                    &registry,
                    id(932, ModelInvocationId::new),
                    &local_selection_requirements(),
                    &snapshot,
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                ),
                Err(AvailableLocalInvocationAdmissionError::AvailabilitySelection(expected))
            );
            assert_eq!(provider.remaining(), 1);
        }
    }

    #[test]
    fn available_local_selection_invocation_and_admission_failures_are_single_attempt() {
        use crate::admission::AdmissionError;
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::{
            select_available_local_model_invoke_and_admit, AvailableLocalInvocationAdmissionError,
            InvocationAdmissionError,
        };
        use crate::model::{
            LanguageModelProvider, ModelErrorKind, RawModelOutput, ScriptedModelProvider,
            ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use nexa_domain::{ModelInvocationId, ModelProviderId};
        use std::sync::Arc;

        for admission_failure in [false, true] {
            let mut f = admission_fixture();
            let invocation_id = id(933, ModelInvocationId::new);
            f.response.invocation_id = invocation_id;
            if admission_failure {
                f.response.output =
                    RawModelOutput::new("private-output-sentinel not json").unwrap();
            }
            let selected = Arc::new(
                ScriptedModelProvider::new(
                    f.descriptor.clone(),
                    [
                        if admission_failure {
                            ScriptedOutcome::Response(f.response.clone())
                        } else {
                            ScriptedOutcome::Error(ModelErrorKind::Unavailable)
                        },
                        ScriptedOutcome::Error(ModelErrorKind::Internal),
                    ],
                )
                .unwrap(),
            );
            let mut other_descriptor = f.descriptor.clone();
            other_descriptor.provider_id = id(940, ModelProviderId::new);
            let other = Arc::new(
                ScriptedModelProvider::new(
                    other_descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                selected.clone() as Arc<dyn LanguageModelProvider>,
                other.clone() as Arc<dyn LanguageModelProvider>,
            ])
            .unwrap();
            let availability = ModelAvailabilitySnapshot::new(vec![
                ModelAvailabilityEntry {
                    provider_id: f.descriptor.provider_id,
                    model_id: f.descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
                ModelAvailabilityEntry {
                    provider_id: other_descriptor.provider_id,
                    model_id: other_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
            ])
            .unwrap();
            let error = select_available_local_model_invoke_and_admit(
                &registry,
                invocation_id,
                &local_selection_requirements(),
                &availability,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap_err();
            let nested = if admission_failure {
                InvocationAdmissionError::Admission(AdmissionError::MalformedSyntax)
            } else {
                InvocationAdmissionError::Invocation(ModelErrorKind::Unavailable)
            };
            assert_eq!(
                error,
                AvailableLocalInvocationAdmissionError::InvocationAdmission(nested)
            );
            assert_eq!(selected.remaining(), 1);
            assert_eq!(other.remaining(), 1);
            let diagnostics = format!("{error:?} {error}");
            assert!(!diagnostics.contains("private-output-sentinel"));
            assert!(!diagnostics.contains("distinctive private platform prompt"));
        }
    }

    #[test]
    fn available_local_selection_preflight_failures_preserve_all_scripted_outcomes() {
        use crate::admission::AdmissionError;
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::{
            select_available_local_model_invoke_and_admit, AvailableLocalInvocationAdmissionError,
            InvocationAdmissionError,
        };
        use crate::model::{
            LanguageModelProvider, ModelInput, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use nexa_domain::{
            ModelId, ModelInvocationId, ModelProviderId, ProtocolVersion, StudentId,
        };
        use std::sync::Arc;

        for mutation in 0..5 {
            let mut f = admission_fixture();
            let secret = "available-preflight-input-sentinel";
            let expected = match mutation {
                0 => {
                    f.compilation.manifest[0].content_bytes += 1;
                    AdmissionError::PromptAssociationReplayMismatch
                }
                1 => {
                    f.compilation.model_input = ModelInput::new(secret).unwrap();
                    AdmissionError::PromptAssociationReplayMismatch
                }
                2 => {
                    f.authority.evidence.scope.student_id = id(950, StudentId::new);
                    AdmissionError::PlanningEvidenceProvenance
                }
                3 => {
                    f.context.governance_policy_version = ProtocolVersion::new(2, 0);
                    AdmissionError::PlanningEvidenceProvenance
                }
                _ => {
                    f.citations.maximum_citations = 0;
                    AdmissionError::CitationGroundingReference
                }
            };
            let selected = Arc::new(
                ScriptedModelProvider::new(
                    f.descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let mut other_descriptor = f.descriptor.clone();
            other_descriptor.provider_id = id(951, ModelProviderId::new);
            other_descriptor.model_id = id(951, ModelId::new);
            let other = Arc::new(
                ScriptedModelProvider::new(
                    other_descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let mut remote_descriptor = f.descriptor.clone();
            remote_descriptor.provider_id = id(952, ModelProviderId::new);
            remote_descriptor.model_id = id(952, ModelId::new);
            remote_descriptor.privacy_class = PrivacyClass::ApprovedRemote;
            let remote = Arc::new(
                ScriptedModelProvider::new(
                    remote_descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                selected.clone() as Arc<dyn LanguageModelProvider>,
                other.clone() as Arc<dyn LanguageModelProvider>,
                remote.clone() as Arc<dyn LanguageModelProvider>,
            ])
            .unwrap();
            let availability = ModelAvailabilitySnapshot::new(vec![
                ModelAvailabilityEntry {
                    provider_id: f.descriptor.provider_id,
                    model_id: f.descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
                ModelAvailabilityEntry {
                    provider_id: other_descriptor.provider_id,
                    model_id: other_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
                ModelAvailabilityEntry {
                    provider_id: remote_descriptor.provider_id,
                    model_id: remote_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
            ])
            .unwrap();
            let error = select_available_local_model_invoke_and_admit(
                &registry,
                id(953, ModelInvocationId::new),
                &local_selection_requirements(),
                &availability,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap_err();
            assert_eq!(
                error,
                AvailableLocalInvocationAdmissionError::InvocationAdmission(
                    InvocationAdmissionError::Preflight(expected)
                ),
                "mutation {mutation}"
            );
            assert_eq!(selected.remaining(), 1, "mutation {mutation}");
            assert_eq!(other.remaining(), 1, "mutation {mutation}");
            assert_eq!(remote.remaining(), 1, "mutation {mutation}");
            let diagnostics = format!("{error} {error:?}");
            for sensitive in [
                secret,
                "distinctive private platform prompt",
                "learner-visible-request",
                "knowledge excerpt",
                "citation",
            ] {
                assert!(!diagnostics.contains(sensitive), "mutation {mutation}");
            }
        }
    }

    #[test]
    fn available_local_selection_post_invocation_admission_matrix_is_single_attempt() {
        use crate::admission::AdmissionError;
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::{
            select_available_local_model_invoke_and_admit, AvailableLocalInvocationAdmissionError,
            InvocationAdmissionError,
        };
        use crate::model::{
            FinishReason, LanguageModelProvider, ModelErrorKind, PrivacyClass, RawModelOutput,
            ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId};
        use std::sync::Arc;

        for mutation in 0..6 {
            let mut f = admission_fixture();
            let invocation_id = id(960, ModelInvocationId::new);
            f.response.invocation_id = invocation_id;
            f.response.provider_id = f.descriptor.provider_id;
            f.response.model_id = f.descriptor.model_id;
            let expected = match mutation {
                0 => {
                    f.response.output =
                        RawModelOutput::new("available-malformed-sentinel").unwrap();
                    AdmissionError::MalformedSyntax
                }
                1 => {
                    f.response.output =
                        RawModelOutput::new("{\"candidate_schema_version\":\"1.0\",\"sections\":[")
                            .unwrap();
                    AdmissionError::MalformedSyntax
                }
                2 => {
                    f.response.output = RawModelOutput::new(
                        "{\"candidate_schema_version\":\"1.0\",\"sections\":[]} trailing-sentinel",
                    )
                    .unwrap();
                    AdmissionError::MalformedSyntax
                }
                3 => {
                    f.set_candidate(json!({"candidate_schema_version":"1.0", "sections":[f.section.clone()], "unknown_candidate_sentinel":true}));
                    AdmissionError::InvalidCandidateSchema
                }
                4 => {
                    let mut section = f.section.clone();
                    section["unknown_section_sentinel"] = json!(true);
                    f.set_candidate(
                        json!({"candidate_schema_version":"1.0", "sections":[section]}),
                    );
                    AdmissionError::InvalidCandidateSchema
                }
                5 => {
                    f.response.finish_reason = FinishReason::OutputLimit;
                    AdmissionError::IncompleteOutput
                }
                _ => unreachable!(),
            };
            let selected = Arc::new(
                ScriptedModelProvider::new(
                    f.descriptor.clone(),
                    [
                        ScriptedOutcome::Response(f.response.clone()),
                        ScriptedOutcome::Error(ModelErrorKind::Internal),
                    ],
                )
                .unwrap(),
            );
            let mut other_descriptor = f.descriptor.clone();
            other_descriptor.provider_id = id(964, ModelProviderId::new);
            other_descriptor.model_id = id(964, ModelId::new);
            let other = Arc::new(
                ScriptedModelProvider::new(
                    other_descriptor.clone(),
                    [ScriptedOutcome::Error(ModelErrorKind::Internal)],
                )
                .unwrap(),
            );
            let mut remote_descriptor = f.descriptor.clone();
            remote_descriptor.provider_id = id(965, ModelProviderId::new);
            remote_descriptor.model_id = id(965, ModelId::new);
            remote_descriptor.privacy_class = PrivacyClass::ApprovedRemote;
            let remote = Arc::new(
                ScriptedModelProvider::new(
                    remote_descriptor.clone(),
                    [ScriptedOutcome::Error(ModelErrorKind::Internal)],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                selected.clone() as Arc<dyn LanguageModelProvider>,
                other.clone() as Arc<dyn LanguageModelProvider>,
                remote.clone() as Arc<dyn LanguageModelProvider>,
            ])
            .unwrap();
            let availability = ModelAvailabilitySnapshot::new(vec![
                ModelAvailabilityEntry {
                    provider_id: f.descriptor.provider_id,
                    model_id: f.descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
                ModelAvailabilityEntry {
                    provider_id: other_descriptor.provider_id,
                    model_id: other_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
                ModelAvailabilityEntry {
                    provider_id: remote_descriptor.provider_id,
                    model_id: remote_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
            ])
            .unwrap();
            let error = select_available_local_model_invoke_and_admit(
                &registry,
                invocation_id,
                &local_selection_requirements(),
                &availability,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap_err();
            assert_eq!(
                error,
                AvailableLocalInvocationAdmissionError::InvocationAdmission(
                    InvocationAdmissionError::Admission(expected)
                ),
                "mutation {mutation}"
            );
            assert_eq!(selected.remaining(), 1, "mutation {mutation}");
            assert_eq!(other.remaining(), 1, "mutation {mutation}");
            assert_eq!(remote.remaining(), 1, "mutation {mutation}");
            let diagnostics = format!("{error} {error:?}");
            for sensitive in [
                "available-malformed-sentinel",
                "trailing-sentinel",
                "unknown_candidate_sentinel",
                "unknown_section_sentinel",
                "distinctive private platform prompt",
            ] {
                assert!(!diagnostics.contains(sensitive), "mutation {mutation}");
            }
        }
    }

    #[test]
    fn available_local_selection_response_identity_mismatch_uses_constructed_request() {
        use crate::admission::AdmissionError;
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::{
            select_available_local_model_invoke_and_admit, AvailableLocalInvocationAdmissionError,
            InvocationAdmissionError,
        };
        use crate::model::{
            LanguageModelProvider, ModelErrorKind, PrivacyClass, ScriptedModelProvider,
            ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId};
        use std::sync::Arc;

        for mutation in 0..3 {
            let mut f = admission_fixture();
            let invocation_id = id(970, ModelInvocationId::new);
            f.response.invocation_id = invocation_id;
            f.response.provider_id = f.descriptor.provider_id;
            f.response.model_id = f.descriptor.model_id;
            match mutation {
                0 => f.response.invocation_id = id(971, ModelInvocationId::new),
                1 => f.response.provider_id = id(972, ModelProviderId::new),
                _ => f.response.model_id = id(973, ModelId::new),
            }
            let selected = Arc::new(UncheckedScriptedProvider::new(
                f.descriptor.clone(),
                [
                    ScriptedOutcome::Response(f.response.clone()),
                    ScriptedOutcome::Error(ModelErrorKind::Internal),
                ],
            ));
            let mut other_descriptor = f.descriptor.clone();
            other_descriptor.provider_id = id(974, ModelProviderId::new);
            other_descriptor.model_id = id(974, ModelId::new);
            let other = Arc::new(
                ScriptedModelProvider::new(
                    other_descriptor.clone(),
                    [ScriptedOutcome::Error(ModelErrorKind::Internal)],
                )
                .unwrap(),
            );
            let mut remote_descriptor = f.descriptor.clone();
            remote_descriptor.provider_id = id(975, ModelProviderId::new);
            remote_descriptor.model_id = id(975, ModelId::new);
            remote_descriptor.privacy_class = PrivacyClass::ApprovedRemote;
            let remote = Arc::new(
                ScriptedModelProvider::new(
                    remote_descriptor.clone(),
                    [ScriptedOutcome::Error(ModelErrorKind::Internal)],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                selected.clone() as Arc<dyn LanguageModelProvider>,
                other.clone() as Arc<dyn LanguageModelProvider>,
                remote.clone() as Arc<dyn LanguageModelProvider>,
            ])
            .unwrap();
            let availability = ModelAvailabilitySnapshot::new(vec![
                ModelAvailabilityEntry {
                    provider_id: f.descriptor.provider_id,
                    model_id: f.descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
                ModelAvailabilityEntry {
                    provider_id: other_descriptor.provider_id,
                    model_id: other_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
                ModelAvailabilityEntry {
                    provider_id: remote_descriptor.provider_id,
                    model_id: remote_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
            ])
            .unwrap();
            let error = select_available_local_model_invoke_and_admit(
                &registry,
                invocation_id,
                &local_selection_requirements(),
                &availability,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap_err();
            assert_eq!(
                error,
                AvailableLocalInvocationAdmissionError::InvocationAdmission(
                    InvocationAdmissionError::Admission(
                        AdmissionError::ModelResponseIdentityMismatch
                    )
                ),
                "mutation {mutation}"
            );
            assert_eq!(selected.remaining(), 1, "mutation {mutation}");
            assert_eq!(other.remaining(), 1, "mutation {mutation}");
            assert_eq!(remote.remaining(), 1, "mutation {mutation}");
            assert!(!format!("{error} {error:?}").contains("distinctive private"));
        }
    }

    #[test]
    fn available_local_selection_enforces_eligibility_and_exact_context_boundary() {
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::{
            select_available_local_model_invoke_and_admit, AvailableLocalInvocationAdmissionError,
        };
        use crate::model::{LanguageModelProvider, ScriptedModelProvider, ScriptedOutcome};
        use crate::registry::ModelRegistry;
        use crate::selection::ModelSelectionError;
        use nexa_domain::ModelInvocationId;
        use std::sync::Arc;

        for mutation in 0..4 {
            let mut f = admission_fixture();
            let invocation_id = id(934, ModelInvocationId::new);
            f.response.invocation_id = invocation_id;
            let mut descriptor = f.descriptor.clone();
            match mutation {
                0 => descriptor.capabilities.structured_output = false,
                1 => descriptor.capabilities.maximum_output_tokens = 999,
                2 => {
                    descriptor.capabilities.context_window_tokens =
                        f.compilation.model_input.as_str().len() as u32 + 999
                }
                _ => {
                    descriptor.capabilities.context_window_tokens =
                        f.compilation.model_input.as_str().len() as u32 + 1000
                }
            }
            let provider = Arc::new(
                ScriptedModelProvider::new(
                    descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                provider.clone() as Arc<dyn LanguageModelProvider>
            ])
            .unwrap();
            let availability = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
                provider_id: descriptor.provider_id,
                model_id: descriptor.model_id,
                state: ModelAvailabilityState::Available,
            }])
            .unwrap();
            let result = select_available_local_model_invoke_and_admit(
                &registry,
                invocation_id,
                &local_selection_requirements(),
                &availability,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            );
            if mutation == 3 {
                assert!(result.is_ok());
                assert_eq!(provider.remaining(), 0);
            } else {
                assert_eq!(
                    result,
                    Err(
                        AvailableLocalInvocationAdmissionError::AvailabilitySelection(
                            crate::availability::ModelAvailabilityError::Selection(
                                ModelSelectionError::NoEligibleModel
                            )
                        )
                    )
                );
                assert_eq!(provider.remaining(), 1);
            }
        }
    }
    #[test]
    fn response_planner_has_no_generation_or_async_surface() {
        let source = include_str!("lib.rs");
        let prohibited = [
            ["req", "west"].concat(),
            ["tok", "io::"].concat(),
            ["asy", "nc fn"].concat(),
            ["model", "_provider"].concat(),
            ["generate", "_prose"].concat(),
            ["semantic", "_entailment"].concat(),
            ["learner_state", "_inference"].concat(),
        ];
        for prohibited in prohibited {
            assert!(
                !source.contains(&prohibited),
                "found prohibited implementation marker"
            )
        }
    }

    fn remote_selection_requirements(
        privacy: Vec<crate::model::PrivacyClass>,
    ) -> crate::selection::ModelSelectionRequirements {
        crate::selection::ModelSelectionRequirements::new(
            crate::model::RequiredCapabilities {
                structured_output: true,
                tool_calling: false,
                vision: false,
            },
            1000,
            privacy,
        )
        .unwrap()
    }

    /// Test-only provider which makes the composition boundary observable without
    /// changing the provider-neutral production port.
    struct RecordingModelProvider {
        descriptor: crate::model::ModelDescriptor,
        requests: std::sync::Mutex<Vec<crate::model::ModelRequest>>,
        outcomes: std::sync::Mutex<std::collections::VecDeque<crate::model::ScriptedOutcome>>,
    }

    impl RecordingModelProvider {
        fn new(
            descriptor: crate::model::ModelDescriptor,
            outcomes: impl IntoIterator<Item = crate::model::ScriptedOutcome>,
        ) -> Self {
            Self {
                descriptor,
                requests: std::sync::Mutex::new(Vec::new()),
                outcomes: std::sync::Mutex::new(outcomes.into_iter().collect()),
            }
        }

        fn requests(&self) -> Vec<crate::model::ModelRequest> {
            self.requests.lock().unwrap().clone()
        }

        fn remaining(&self) -> usize {
            self.outcomes.lock().unwrap().len()
        }
    }

    impl crate::model::LanguageModelProvider for RecordingModelProvider {
        fn descriptor(&self) -> &crate::model::ModelDescriptor {
            &self.descriptor
        }

        fn generate(
            &self,
            request: &crate::model::ModelRequest,
        ) -> Result<crate::model::ModelResponse, crate::model::ModelError> {
            request.validate_for(&self.descriptor)?;
            self.requests.lock().unwrap().push(request.clone());
            match self.outcomes.lock().unwrap().pop_front() {
                Some(crate::model::ScriptedOutcome::Response(response)) => {
                    response.validate_for(request)?;
                    Ok(response)
                }
                Some(crate::model::ScriptedOutcome::Error(kind)) => {
                    Err(crate::model::ModelError::new(kind))
                }
                None => Err(crate::model::ModelError::new(
                    crate::model::ModelErrorKind::ScriptExhausted,
                )),
            }
        }
    }

    #[test]
    fn authorized_available_remote_invocation_constructs_exact_request_and_admits_for_both_remote_classes(
    ) {
        use crate::authorization::{RemoteModelAuthorization, RemoteModelAuthorizationEntry};
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::select_authorized_available_remote_model_invoke_and_admit;
        use crate::model::{
            LanguageModelProvider, ModelRequest, ModelResponse, PrivacyClass, ScriptedOutcome,
            MODEL_INVOCATION_V1,
        };
        use crate::registry::ModelRegistry;
        use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId};
        use std::sync::Arc;

        for (privacy, preferences) in [
            (
                PrivacyClass::ApprovedRemote,
                vec![PrivacyClass::ApprovedRemote, PrivacyClass::RestrictedRemote],
            ),
            (
                PrivacyClass::RestrictedRemote,
                vec![PrivacyClass::RestrictedRemote, PrivacyClass::ApprovedRemote],
            ),
        ] {
            for reverse_registry in [false, true] {
                let f = admission_fixture();
                let invocation_id = id(1032, ModelInvocationId::new);
                let mut selected_descriptor = f.descriptor.clone();
                selected_descriptor.provider_id = id(20, ModelProviderId::new);
                selected_descriptor.model_id = id(20, ModelId::new);
                selected_descriptor.privacy_class = privacy;
                let mut response = f.response.clone();
                response.invocation_id = invocation_id;
                response.provider_id = selected_descriptor.provider_id;
                response.model_id = selected_descriptor.model_id;
                let selected = Arc::new(RecordingModelProvider::new(
                    selected_descriptor.clone(),
                    [
                        ScriptedOutcome::Response(response),
                        ScriptedOutcome::Error(crate::model::ModelErrorKind::Internal),
                    ],
                ));
                let mut other_descriptor = selected_descriptor.clone();
                other_descriptor.provider_id = id(30, ModelProviderId::new);
                other_descriptor.model_id = id(30, ModelId::new);
                let mut unauthorized_descriptor = selected_descriptor.clone();
                unauthorized_descriptor.provider_id = id(40, ModelProviderId::new);
                unauthorized_descriptor.model_id = id(40, ModelId::new);
                let mut unavailable_descriptor = selected_descriptor.clone();
                unavailable_descriptor.provider_id = id(50, ModelProviderId::new);
                unavailable_descriptor.model_id = id(50, ModelId::new);
                let mut local_descriptor = selected_descriptor.clone();
                local_descriptor.provider_id = id(60, ModelProviderId::new);
                local_descriptor.model_id = id(60, ModelId::new);
                local_descriptor.privacy_class = PrivacyClass::LocalOnly;
                let make_unselected = |descriptor| {
                    Arc::new(RecordingModelProvider::new(
                        descriptor,
                        [
                            ScriptedOutcome::Error(crate::model::ModelErrorKind::Internal),
                            ScriptedOutcome::Error(crate::model::ModelErrorKind::Unavailable),
                        ],
                    ))
                };
                let other = make_unselected(other_descriptor.clone());
                let unauthorized = make_unselected(unauthorized_descriptor.clone());
                let unavailable = make_unselected(unavailable_descriptor.clone());
                let local = make_unselected(local_descriptor.clone());
                let mut handles: Vec<Arc<dyn LanguageModelProvider>> = vec![
                    selected.clone(),
                    other.clone(),
                    unauthorized.clone(),
                    unavailable.clone(),
                    local.clone(),
                ];
                if reverse_registry {
                    handles.reverse();
                }
                let registry = ModelRegistry::try_from_providers(handles).unwrap();
                let availability = ModelAvailabilitySnapshot::new(vec![
                    ModelAvailabilityEntry {
                        provider_id: selected_descriptor.provider_id,
                        model_id: selected_descriptor.model_id,
                        state: ModelAvailabilityState::Available,
                    },
                    ModelAvailabilityEntry {
                        provider_id: other_descriptor.provider_id,
                        model_id: other_descriptor.model_id,
                        state: ModelAvailabilityState::Available,
                    },
                    ModelAvailabilityEntry {
                        provider_id: unauthorized_descriptor.provider_id,
                        model_id: unauthorized_descriptor.model_id,
                        state: ModelAvailabilityState::Available,
                    },
                    ModelAvailabilityEntry {
                        provider_id: unavailable_descriptor.provider_id,
                        model_id: unavailable_descriptor.model_id,
                        state: ModelAvailabilityState::Unavailable,
                    },
                    ModelAvailabilityEntry {
                        provider_id: local_descriptor.provider_id,
                        model_id: local_descriptor.model_id,
                        state: ModelAvailabilityState::Available,
                    },
                ])
                .unwrap();
                let authorization = RemoteModelAuthorization::new(
                    f.compilation.replay_anchor.clone(),
                    vec![
                        RemoteModelAuthorizationEntry {
                            provider_id: selected_descriptor.provider_id,
                            model_id: selected_descriptor.model_id,
                            privacy_class: privacy,
                        },
                        RemoteModelAuthorizationEntry {
                            provider_id: other_descriptor.provider_id,
                            model_id: other_descriptor.model_id,
                            privacy_class: privacy,
                        },
                        RemoteModelAuthorizationEntry {
                            provider_id: unavailable_descriptor.provider_id,
                            model_id: unavailable_descriptor.model_id,
                            privacy_class: privacy,
                        },
                    ],
                )
                .unwrap();
                let requirements = remote_selection_requirements(preferences.clone());
                let result = select_authorized_available_remote_model_invoke_and_admit(
                    &registry,
                    invocation_id,
                    &requirements,
                    &availability,
                    &authorization,
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                )
                .unwrap();
                let expected_request = ModelRequest {
                    invocation_id,
                    provider_id: selected_descriptor.provider_id,
                    model_id: selected_descriptor.model_id,
                    contract_version: MODEL_INVOCATION_V1,
                    input: f.compilation.model_input.clone(),
                    required_capabilities: requirements.required_capabilities.clone(),
                    maximum_output_tokens: requirements.maximum_output_tokens,
                };
                let expected_response = ModelResponse {
                    invocation_id,
                    provider_id: selected_descriptor.provider_id,
                    model_id: selected_descriptor.model_id,
                    ..f.response.clone()
                };
                let direct = crate::admission::admit_model_output(
                    &selected_descriptor,
                    &expected_request,
                    &expected_response,
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                )
                .unwrap();
                assert_eq!(result, direct);
                assert_eq!(result.evidence.invocation_id, invocation_id);
                assert_eq!(
                    result.evidence.prompt_compilation_replay_anchor,
                    f.compilation.replay_anchor
                );
                assert_eq!(selected.remaining(), 1);
                assert_eq!(selected.requests(), vec![expected_request]);
                for provider in [&other, &unauthorized, &unavailable, &local] {
                    assert_eq!(provider.remaining(), 2);
                    assert!(provider.requests().is_empty());
                }
            }
        }
    }

    #[test]
    fn authorized_available_remote_invocation_selection_denials_are_nested_and_non_consuming() {
        use crate::authorization::{
            RemoteAuthorizationError, RemoteModelAuthorization, RemoteModelAuthorizationEntry,
        };
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::{
            select_authorized_available_remote_model_invoke_and_admit,
            AuthorizedAvailableRemoteInvocationAdmissionError,
        };
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::selection::ModelSelectionError;
        use nexa_domain::ModelInvocationId;
        use std::sync::Arc;

        let f = admission_fixture();
        let mut descriptor = f.descriptor.clone();
        descriptor.privacy_class = PrivacyClass::ApprovedRemote;
        let provider = Arc::new(
            ScriptedModelProvider::new(
                descriptor.clone(),
                [ScriptedOutcome::Error(
                    crate::model::ModelErrorKind::Internal,
                )],
            )
            .unwrap(),
        );
        let registry =
            ModelRegistry::try_from_providers([provider.clone() as Arc<dyn LanguageModelProvider>])
                .unwrap();
        let entry = RemoteModelAuthorizationEntry {
            provider_id: descriptor.provider_id,
            model_id: descriptor.model_id,
            privacy_class: descriptor.privacy_class,
        };
        let available = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: descriptor.provider_id,
            model_id: descriptor.model_id,
            state: ModelAvailabilityState::Available,
        }])
        .unwrap();
        let authorized =
            RemoteModelAuthorization::new(f.compilation.replay_anchor.clone(), vec![entry])
                .unwrap();
        let empty_authorization =
            RemoteModelAuthorization::new(f.compilation.replay_anchor.clone(), vec![]).unwrap();
        let omitted = ModelAvailabilitySnapshot::new(vec![]).unwrap();
        let unavailable = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: descriptor.provider_id,
            model_id: descriptor.model_id,
            state: ModelAvailabilityState::Unavailable,
        }])
        .unwrap();

        for (availability, authorization, expected) in [
            (
                &available,
                &empty_authorization,
                RemoteAuthorizationError::Selection(ModelSelectionError::NoEligibleModel),
            ),
            (
                &omitted,
                &authorized,
                RemoteAuthorizationError::Selection(ModelSelectionError::NoEligibleModel),
            ),
            (
                &unavailable,
                &authorized,
                RemoteAuthorizationError::Selection(ModelSelectionError::NoEligibleModel),
            ),
        ] {
            assert_eq!(
                select_authorized_available_remote_model_invoke_and_admit(
                    &registry,
                    id(1033, ModelInvocationId::new),
                    &remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]),
                    availability,
                    authorization,
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                ),
                Err(AuthorizedAvailableRemoteInvocationAdmissionError::AuthorizationAvailabilitySelection(expected))
            );
            assert_eq!(provider.remaining(), 1);
        }

        for privacy in [
            vec![],
            vec![PrivacyClass::LocalOnly],
            vec![PrivacyClass::ApprovedRemote, PrivacyClass::LocalOnly],
            vec![PrivacyClass::ApprovedRemote, PrivacyClass::ApprovedRemote],
        ] {
            let mut requirements =
                remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]);
            requirements.privacy_preference = privacy;
            assert_eq!(
                select_authorized_available_remote_model_invoke_and_admit(
                    &registry,
                    id(1034, ModelInvocationId::new),
                    &requirements,
                    &available,
                    &authorized,
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                ),
                Err(AuthorizedAvailableRemoteInvocationAdmissionError::AuthorizationAvailabilitySelection(RemoteAuthorizationError::InvalidRemoteRequirements))
            );
            assert_eq!(provider.remaining(), 1);
        }

        let mut unsupported = remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]);
        unsupported.contract_version = nexa_domain::ProtocolVersion::new(99, 0);
        assert_eq!(
            select_authorized_available_remote_model_invoke_and_admit(
                &registry, id(1034, ModelInvocationId::new), &unsupported, &available,
                &authorized, &f.compilation, &f.authority, &f.context, &f.citations,
            ),
            Err(AuthorizedAvailableRemoteInvocationAdmissionError::AuthorizationAvailabilitySelection(
                RemoteAuthorizationError::InvalidRemoteRequirements
            ))
        );
        assert_eq!(provider.remaining(), 1);
    }

    #[test]
    fn authorized_available_remote_invocation_preflight_invocation_and_admission_are_single_attempt(
    ) {
        use crate::admission::AdmissionError;
        use crate::authorization::{RemoteModelAuthorization, RemoteModelAuthorizationEntry};
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::{
            select_authorized_available_remote_model_invoke_and_admit,
            AuthorizedAvailableRemoteInvocationAdmissionError, InvocationAdmissionError,
        };
        use crate::model::{
            LanguageModelProvider, ModelErrorKind, ModelRequest, PrivacyClass, RawModelOutput,
            ScriptedOutcome, MODEL_INVOCATION_V1,
        };
        use crate::registry::ModelRegistry;
        use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId};
        use std::sync::Arc;

        for mode in 0..3 {
            let mut f = admission_fixture();
            let invocation_id = id(1035, ModelInvocationId::new);
            f.descriptor.provider_id = id(70, ModelProviderId::new);
            f.descriptor.model_id = id(70, ModelId::new);
            f.descriptor.privacy_class = PrivacyClass::RestrictedRemote;
            f.response.invocation_id = invocation_id;
            f.response.provider_id = f.descriptor.provider_id;
            f.response.model_id = f.descriptor.model_id;
            if mode == 0 {
                f.authority.context_package_id = id(9999, nexa_domain::ContextPackageId::new);
            }
            if mode == 2 {
                f.response.output = RawModelOutput::new("private-response-sentinel").unwrap();
            }
            let first = if mode == 1 {
                ScriptedOutcome::Error(ModelErrorKind::Unavailable)
            } else {
                ScriptedOutcome::Response(f.response.clone())
            };
            let selected = Arc::new(RecordingModelProvider::new(
                f.descriptor.clone(),
                [first, ScriptedOutcome::Error(ModelErrorKind::Internal)],
            ));

            let mut descriptors = Vec::new();
            for (number, privacy) in [
                (80_u128, PrivacyClass::RestrictedRemote),
                (90, PrivacyClass::ApprovedRemote),
                (100, PrivacyClass::RestrictedRemote),
                (110, PrivacyClass::LocalOnly),
            ] {
                let mut descriptor = f.descriptor.clone();
                descriptor.provider_id = id(number, ModelProviderId::new);
                descriptor.model_id = id(number, ModelId::new);
                descriptor.privacy_class = privacy;
                descriptors.push(descriptor);
            }
            let providers: Vec<Arc<RecordingModelProvider>> = descriptors
                .iter()
                .cloned()
                .map(|descriptor| {
                    Arc::new(RecordingModelProvider::new(
                        descriptor,
                        [
                            ScriptedOutcome::Error(ModelErrorKind::Internal),
                            ScriptedOutcome::Error(ModelErrorKind::Unavailable),
                        ],
                    ))
                })
                .collect();
            let registry = ModelRegistry::try_from_providers(
                std::iter::once(selected.clone() as Arc<dyn LanguageModelProvider>).chain(
                    providers
                        .iter()
                        .cloned()
                        .map(|provider| provider as Arc<dyn LanguageModelProvider>),
                ),
            )
            .unwrap();
            let availability = ModelAvailabilitySnapshot::new(vec![
                ModelAvailabilityEntry {
                    provider_id: f.descriptor.provider_id,
                    model_id: f.descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
                ModelAvailabilityEntry {
                    provider_id: descriptors[0].provider_id,
                    model_id: descriptors[0].model_id,
                    state: ModelAvailabilityState::Available,
                },
                ModelAvailabilityEntry {
                    provider_id: descriptors[1].provider_id,
                    model_id: descriptors[1].model_id,
                    state: ModelAvailabilityState::Available,
                },
                ModelAvailabilityEntry {
                    provider_id: descriptors[2].provider_id,
                    model_id: descriptors[2].model_id,
                    state: ModelAvailabilityState::Unavailable,
                },
                ModelAvailabilityEntry {
                    provider_id: descriptors[3].provider_id,
                    model_id: descriptors[3].model_id,
                    state: ModelAvailabilityState::Available,
                },
            ])
            .unwrap();
            let authorization = RemoteModelAuthorization::new(
                f.compilation.replay_anchor.clone(),
                vec![
                    RemoteModelAuthorizationEntry {
                        provider_id: f.descriptor.provider_id,
                        model_id: f.descriptor.model_id,
                        privacy_class: PrivacyClass::RestrictedRemote,
                    },
                    RemoteModelAuthorizationEntry {
                        provider_id: descriptors[0].provider_id,
                        model_id: descriptors[0].model_id,
                        privacy_class: PrivacyClass::RestrictedRemote,
                    },
                    RemoteModelAuthorizationEntry {
                        provider_id: descriptors[2].provider_id,
                        model_id: descriptors[2].model_id,
                        privacy_class: PrivacyClass::RestrictedRemote,
                    },
                ],
            )
            .unwrap();
            let error = select_authorized_available_remote_model_invoke_and_admit(
                &registry,
                invocation_id,
                &remote_selection_requirements(vec![PrivacyClass::RestrictedRemote]),
                &availability,
                &authorization,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap_err();
            let expected = match mode {
                0 => {
                    InvocationAdmissionError::Preflight(AdmissionError::PlanningEvidenceProvenance)
                }
                1 => InvocationAdmissionError::Invocation(ModelErrorKind::Unavailable),
                _ => InvocationAdmissionError::Admission(AdmissionError::MalformedSyntax),
            };
            assert_eq!(
                error,
                AuthorizedAvailableRemoteInvocationAdmissionError::InvocationAdmission(expected)
            );
            assert_eq!(selected.remaining(), if mode == 0 { 2 } else { 1 });
            let expected_requests = if mode == 0 {
                vec![]
            } else {
                vec![ModelRequest {
                    invocation_id,
                    provider_id: f.descriptor.provider_id,
                    model_id: f.descriptor.model_id,
                    contract_version: MODEL_INVOCATION_V1,
                    input: f.compilation.model_input.clone(),
                    required_capabilities: remote_selection_requirements(vec![
                        PrivacyClass::RestrictedRemote,
                    ])
                    .required_capabilities,
                    maximum_output_tokens: 1000,
                }]
            };
            assert_eq!(selected.requests(), expected_requests);
            for provider in &providers {
                assert_eq!(provider.remaining(), 2);
                assert!(provider.requests().is_empty());
            }
            let diagnostics = format!("{error:?} {error}");
            for sentinel in [
                "private-response-sentinel",
                "distinctive private learner prompt",
                "distinctive private platform prompt",
            ] {
                assert!(!diagnostics.contains(sentinel));
            }
        }
    }

    #[test]
    fn authorized_available_remote_invocation_observes_exact_request_and_both_privacy_orders() {
        use crate::authorization::{RemoteModelAuthorization, RemoteModelAuthorizationEntry};
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::select_authorized_available_remote_model_invoke_and_admit;
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedOutcome, MODEL_INVOCATION_V1,
        };
        use crate::registry::ModelRegistry;
        use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId};
        use std::sync::Arc;

        for (preferences, expected_privacy, expected_number) in [
            (
                vec![PrivacyClass::ApprovedRemote, PrivacyClass::RestrictedRemote],
                PrivacyClass::ApprovedRemote,
                40_u128,
            ),
            (
                vec![PrivacyClass::RestrictedRemote, PrivacyClass::ApprovedRemote],
                PrivacyClass::RestrictedRemote,
                50_u128,
            ),
        ] {
            let f = admission_fixture();
            let invocation_id = id(1040, ModelInvocationId::new);
            let mut providers = Vec::new();
            let mut recordings = Vec::new();
            let mut entries = Vec::new();
            let mut states = Vec::new();
            for (number, privacy) in [
                (40_u128, PrivacyClass::ApprovedRemote),
                (50, PrivacyClass::RestrictedRemote),
            ] {
                let mut descriptor = f.descriptor.clone();
                descriptor.provider_id = id(number, ModelProviderId::new);
                descriptor.model_id = id(number, ModelId::new);
                descriptor.privacy_class = privacy;
                let mut response = f.response.clone();
                response.invocation_id = invocation_id;
                response.provider_id = descriptor.provider_id;
                response.model_id = descriptor.model_id;
                let provider = Arc::new(RecordingModelProvider::new(
                    descriptor.clone(),
                    [
                        ScriptedOutcome::Response(response),
                        ScriptedOutcome::Error(crate::model::ModelErrorKind::Internal),
                    ],
                ));
                providers.push(provider.clone() as Arc<dyn LanguageModelProvider>);
                recordings.push((number, provider));
                entries.push(RemoteModelAuthorizationEntry {
                    provider_id: descriptor.provider_id,
                    model_id: descriptor.model_id,
                    privacy_class: privacy,
                });
                states.push(ModelAvailabilityEntry {
                    provider_id: descriptor.provider_id,
                    model_id: descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                });
            }
            providers.reverse();
            let registry = ModelRegistry::try_from_providers(providers).unwrap();
            let availability = ModelAvailabilitySnapshot::new(states).unwrap();
            let authorization =
                RemoteModelAuthorization::new(f.compilation.replay_anchor.clone(), entries)
                    .unwrap();
            let requirements = remote_selection_requirements(preferences);
            select_authorized_available_remote_model_invoke_and_admit(
                &registry,
                invocation_id,
                &requirements,
                &availability,
                &authorization,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap();

            for (number, provider) in recordings {
                if number == expected_number {
                    assert_eq!(provider.remaining(), 1);
                    assert_eq!(provider.descriptor.privacy_class, expected_privacy);
                    assert_eq!(
                        provider.requests(),
                        vec![crate::model::ModelRequest {
                            invocation_id,
                            provider_id: provider.descriptor.provider_id,
                            model_id: provider.descriptor.model_id,
                            contract_version: MODEL_INVOCATION_V1,
                            input: f.compilation.model_input.clone(),
                            required_capabilities: requirements.required_capabilities.clone(),
                            maximum_output_tokens: requirements.maximum_output_tokens,
                        }]
                    );
                } else {
                    assert!(provider.requests().is_empty());
                    assert_eq!(provider.remaining(), 2);
                }
            }
        }
    }

    #[test]
    fn authorized_available_remote_invocation_canonical_provider_then_model_tie_break() {
        use crate::authorization::{RemoteModelAuthorization, RemoteModelAuthorizationEntry};
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::select_authorized_available_remote_model_invoke_and_admit;
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId};
        use std::sync::Arc;

        for reverse in [false, true] {
            let f = admission_fixture();
            let invocation_id = id(1041, ModelInvocationId::new);
            let provider_id = id(60, ModelProviderId::new);
            let mut handles: Vec<Arc<ScriptedModelProvider>> = Vec::new();
            let mut entries = Vec::new();
            let mut states = Vec::new();
            for model_number in [62_u128, 61] {
                let mut descriptor = f.descriptor.clone();
                descriptor.provider_id = provider_id;
                descriptor.model_id = id(model_number, ModelId::new);
                descriptor.privacy_class = PrivacyClass::ApprovedRemote;
                let mut response = f.response.clone();
                response.invocation_id = invocation_id;
                response.provider_id = provider_id;
                response.model_id = descriptor.model_id;
                handles.push(Arc::new(
                    ScriptedModelProvider::new(
                        descriptor.clone(),
                        [ScriptedOutcome::Response(response)],
                    )
                    .unwrap(),
                ));
                entries.push(RemoteModelAuthorizationEntry {
                    provider_id,
                    model_id: descriptor.model_id,
                    privacy_class: descriptor.privacy_class,
                });
                states.push(ModelAvailabilityEntry {
                    provider_id,
                    model_id: descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                });
            }
            if reverse {
                handles.reverse();
            }
            let registry = ModelRegistry::try_from_providers(
                handles
                    .iter()
                    .cloned()
                    .map(|p| p as Arc<dyn LanguageModelProvider>),
            )
            .unwrap();
            let authorization =
                RemoteModelAuthorization::new(f.compilation.replay_anchor.clone(), entries)
                    .unwrap();
            let availability = ModelAvailabilitySnapshot::new(states).unwrap();
            select_authorized_available_remote_model_invoke_and_admit(
                &registry,
                invocation_id,
                &remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]),
                &availability,
                &authorization,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap();
            for provider in handles {
                assert_eq!(
                    provider.remaining(),
                    usize::from(provider.descriptor().model_id != id(61, ModelId::new))
                );
            }
        }
    }

    #[test]
    fn authorized_available_remote_invocation_prompt_and_registry_association_fail_closed() {
        use crate::authorization::{
            RemoteAuthorizationError, RemoteModelAuthorization, RemoteModelAuthorizationEntry,
        };
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::{
            select_authorized_available_remote_model_invoke_and_admit,
            AuthorizedAvailableRemoteInvocationAdmissionError as Outer,
        };
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId};
        use std::sync::Arc;

        let f = admission_fixture();
        let mut descriptor = f.descriptor.clone();
        descriptor.privacy_class = PrivacyClass::ApprovedRemote;
        let provider = Arc::new(
            ScriptedModelProvider::new(
                descriptor.clone(),
                [ScriptedOutcome::Error(
                    crate::model::ModelErrorKind::Internal,
                )],
            )
            .unwrap(),
        );
        let registry =
            ModelRegistry::try_from_providers([provider.clone() as Arc<dyn LanguageModelProvider>])
                .unwrap();
        let availability = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: descriptor.provider_id,
            model_id: descriptor.model_id,
            state: ModelAvailabilityState::Available,
        }])
        .unwrap();
        let entry = RemoteModelAuthorizationEntry {
            provider_id: descriptor.provider_id,
            model_id: descriptor.model_id,
            privacy_class: descriptor.privacy_class,
        };
        let requirements = remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]);
        let call = |authorization: &RemoteModelAuthorization,
                    compilation: &crate::prompt::PromptCompilationResult| {
            select_authorized_available_remote_model_invoke_and_admit(
                &registry,
                id(1042, ModelInvocationId::new),
                &requirements,
                &availability,
                authorization,
                compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
        };

        let mut tampered = f.compilation.clone();
        tampered.compiled_bytes += 1;
        let valid = RemoteModelAuthorization::new(f.compilation.replay_anchor.clone(), vec![entry])
            .unwrap();
        assert_eq!(
            call(&valid, &tampered),
            Err(Outer::AuthorizationAvailabilitySelection(
                RemoteAuthorizationError::PromptCompilationAssociation
            ))
        );
        let mismatch = RemoteModelAuthorization::new("a".repeat(64), vec![entry]).unwrap();
        assert_eq!(
            call(&mismatch, &f.compilation),
            Err(Outer::AuthorizationAvailabilitySelection(
                RemoteAuthorizationError::PromptCompilationAssociation
            ))
        );
        for bad_entry in [
            RemoteModelAuthorizationEntry {
                provider_id: id(9990, ModelProviderId::new),
                ..entry
            },
            RemoteModelAuthorizationEntry {
                model_id: id(9991, ModelId::new),
                ..entry
            },
            RemoteModelAuthorizationEntry {
                privacy_class: PrivacyClass::RestrictedRemote,
                ..entry
            },
        ] {
            let bad =
                RemoteModelAuthorization::new(f.compilation.replay_anchor.clone(), vec![bad_entry])
                    .unwrap();
            assert_eq!(
                call(&bad, &f.compilation),
                Err(Outer::AuthorizationAvailabilitySelection(
                    RemoteAuthorizationError::AuthorizationRegistryInconsistency
                ))
            );
        }
        assert_eq!(provider.remaining(), 1);
    }

    #[test]
    fn authorized_available_remote_invocation_preserves_eligibility_and_never_invokes_local() {
        use crate::authorization::{
            RemoteAuthorizationError, RemoteModelAuthorization, RemoteModelAuthorizationEntry,
        };
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::{
            select_authorized_available_remote_model_invoke_and_admit,
            AuthorizedAvailableRemoteInvocationAdmissionError as Outer,
        };
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::selection::ModelSelectionError;
        use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId};
        use std::sync::Arc;

        for mutation in 0..6 {
            let f = admission_fixture();
            let invocation_id = id(1043, ModelInvocationId::new);
            let mut descriptor = f.descriptor.clone();
            descriptor.privacy_class = PrivacyClass::ApprovedRemote;
            let input_units = f.compilation.model_input.as_str().len() as u32;
            let mut requirements =
                remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]);
            match mutation {
                0 => descriptor.capabilities.structured_output = false,
                1 => {
                    requirements.required_capabilities.tool_calling = true;
                    descriptor.capabilities.tool_calling = false;
                }
                2 => {
                    requirements.required_capabilities.vision = true;
                    descriptor.capabilities.vision = false;
                }
                3 => {
                    descriptor.capabilities.maximum_output_tokens =
                        requirements.maximum_output_tokens - 1
                }
                4 => {
                    descriptor.capabilities.context_window_tokens =
                        input_units + requirements.maximum_output_tokens
                }
                _ => {
                    descriptor.capabilities.context_window_tokens =
                        input_units + requirements.maximum_output_tokens - 1
                }
            }
            let mut response = f.response.clone();
            response.invocation_id = invocation_id;
            let remote = Arc::new(
                ScriptedModelProvider::new(
                    descriptor.clone(),
                    [ScriptedOutcome::Response(response)],
                )
                .unwrap(),
            );
            let mut local_descriptor = f.descriptor.clone();
            local_descriptor.provider_id = id(1044, ModelProviderId::new);
            local_descriptor.model_id = id(1044, ModelId::new);
            local_descriptor.privacy_class = PrivacyClass::LocalOnly;
            let local = Arc::new(
                ScriptedModelProvider::new(
                    local_descriptor.clone(),
                    [ScriptedOutcome::Error(
                        crate::model::ModelErrorKind::Internal,
                    )],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                remote.clone() as Arc<dyn LanguageModelProvider>,
                local.clone() as Arc<dyn LanguageModelProvider>,
            ])
            .unwrap();
            let availability = ModelAvailabilitySnapshot::new(vec![
                ModelAvailabilityEntry {
                    provider_id: descriptor.provider_id,
                    model_id: descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
                ModelAvailabilityEntry {
                    provider_id: local_descriptor.provider_id,
                    model_id: local_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
            ])
            .unwrap();
            let authorization = RemoteModelAuthorization::new(
                f.compilation.replay_anchor.clone(),
                vec![RemoteModelAuthorizationEntry {
                    provider_id: descriptor.provider_id,
                    model_id: descriptor.model_id,
                    privacy_class: descriptor.privacy_class,
                }],
            )
            .unwrap();
            let result = select_authorized_available_remote_model_invoke_and_admit(
                &registry,
                invocation_id,
                &requirements,
                &availability,
                &authorization,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            );
            if mutation == 4 {
                assert!(result.is_ok());
                assert_eq!(remote.remaining(), 0);
            } else {
                assert_eq!(
                    result,
                    Err(Outer::AuthorizationAvailabilitySelection(
                        RemoteAuthorizationError::Selection(ModelSelectionError::NoEligibleModel)
                    ))
                );
                assert_eq!(remote.remaining(), 1);
            }
            assert_eq!(local.remaining(), 1);
        }
    }

    #[test]
    fn authorized_available_remote_invocation_disjoint_gates_are_non_consuming() {
        use crate::authorization::{
            RemoteAuthorizationError, RemoteModelAuthorization, RemoteModelAuthorizationEntry,
        };
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::{
            select_authorized_available_remote_model_invoke_and_admit,
            AuthorizedAvailableRemoteInvocationAdmissionError as Outer,
        };
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::selection::ModelSelectionError;
        use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId};
        use std::sync::Arc;

        let f = admission_fixture();
        let mut descriptors = Vec::new();
        let mut providers = Vec::new();
        for number in [1050_u128, 1051] {
            let mut descriptor = f.descriptor.clone();
            descriptor.provider_id = id(number, ModelProviderId::new);
            descriptor.model_id = id(number, ModelId::new);
            descriptor.privacy_class = PrivacyClass::ApprovedRemote;
            providers.push(Arc::new(
                ScriptedModelProvider::new(
                    descriptor.clone(),
                    [ScriptedOutcome::Error(
                        crate::model::ModelErrorKind::Internal,
                    )],
                )
                .unwrap(),
            ));
            descriptors.push(descriptor);
        }
        let registry = ModelRegistry::try_from_providers(
            providers
                .iter()
                .cloned()
                .map(|p| p as Arc<dyn LanguageModelProvider>),
        )
        .unwrap();
        let authorization = RemoteModelAuthorization::new(
            f.compilation.replay_anchor.clone(),
            vec![RemoteModelAuthorizationEntry {
                provider_id: descriptors[0].provider_id,
                model_id: descriptors[0].model_id,
                privacy_class: descriptors[0].privacy_class,
            }],
        )
        .unwrap();
        let availability = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: descriptors[1].provider_id,
            model_id: descriptors[1].model_id,
            state: ModelAvailabilityState::Available,
        }])
        .unwrap();
        assert_eq!(
            select_authorized_available_remote_model_invoke_and_admit(
                &registry,
                id(1052, ModelInvocationId::new),
                &remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]),
                &availability,
                &authorization,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations
            ),
            Err(Outer::AuthorizationAvailabilitySelection(
                RemoteAuthorizationError::Selection(ModelSelectionError::NoEligibleModel)
            ))
        );
        assert!(providers.iter().all(|provider| provider.remaining() == 1));
    }

    #[test]
    fn authorized_available_remote_invocation_diagnostics_redact_all_content_categories() {
        use crate::authorization::RemoteAuthorizationError;
        use crate::availability::ModelAvailabilityError;
        use crate::generation::{
            AuthorizedAvailableRemoteInvocationAdmissionError as Outer, InvocationAdmissionError,
        };
        use crate::model::ModelErrorKind;
        use crate::selection::ModelSelectionError;

        let errors = [
            Outer::AuthorizationAvailabilitySelection(
                RemoteAuthorizationError::PromptCompilationAssociation,
            ),
            Outer::AuthorizationAvailabilitySelection(
                RemoteAuthorizationError::AuthorizationRegistryInconsistency,
            ),
            Outer::AuthorizationAvailabilitySelection(
                RemoteAuthorizationError::AvailabilitySelection(
                    ModelAvailabilityError::RegistryInconsistency,
                ),
            ),
            Outer::AuthorizationAvailabilitySelection(RemoteAuthorizationError::Selection(
                ModelSelectionError::NoEligibleModel,
            )),
            Outer::InvocationAdmission(InvocationAdmissionError::Invocation(
                ModelErrorKind::Internal,
            )),
        ];
        for error in errors {
            let diagnostics = format!("{error:?} {error}");
            for sentinel in [
                "prompt-private-sentinel",
                "response-private-sentinel",
                "learner-private-sentinel",
                "knowledge-private-sentinel",
                "credential-private-sentinel",
                "endpoint-private-sentinel",
                "provider-private-sentinel",
            ] {
                assert!(!diagnostics.contains(sentinel));
            }
        }
    }

    #[test]
    fn authorized_remote_tokenized_composition_denials_preserve_exact_categories_and_dependencies()
    {
        use crate::authorization::{
            RemoteAuthorizationError, RemoteModelAuthorization, RemoteModelAuthorizationEntry,
        };
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilityError, ModelAvailabilitySnapshot,
            ModelAvailabilityState,
        };
        use crate::generation::{
            select_authorized_available_remote_model_tokenize_invoke_and_admit,
            AuthorizedAvailableRemoteTokenizedInvocationAdmissionError as Outer,
        };
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::selection::ModelSelectionError;
        use crate::tokenization::{
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome, MODEL_INPUT_TOKENIZATION_V1,
        };
        use nexa_domain::{ModelId, ModelProviderId, ProtocolVersion};
        use std::sync::Arc;

        let f = admission_fixture();
        let mut descriptor = f.descriptor.clone();
        descriptor.privacy_class = PrivacyClass::ApprovedRemote;
        let provider = Arc::new(
            ScriptedModelProvider::new(
                descriptor.clone(),
                [ScriptedOutcome::Response(f.response.clone())],
            )
            .unwrap(),
        );
        let registry =
            ModelRegistry::try_from_providers([provider.clone() as Arc<dyn LanguageModelProvider>])
                .unwrap();
        let entry = RemoteModelAuthorizationEntry {
            provider_id: descriptor.provider_id,
            model_id: descriptor.model_id,
            privacy_class: descriptor.privacy_class,
        };
        let valid_authorization =
            RemoteModelAuthorization::new(f.compilation.replay_anchor.clone(), vec![entry])
                .unwrap();
        let valid_availability = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: descriptor.provider_id,
            model_id: descriptor.model_id,
            state: ModelAvailabilityState::Available,
        }])
        .unwrap();
        let tokenizer = ScriptedModelInputTokenizer::new(
            descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(1)],
        )
        .unwrap();
        let requirements = remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]);
        let call = |requirements: &crate::selection::ModelSelectionRequirements,
                    availability: &ModelAvailabilitySnapshot,
                    authorization: &RemoteModelAuthorization,
                    compilation: &crate::prompt::PromptCompilationResult| {
            select_authorized_available_remote_model_tokenize_invoke_and_admit(
                &registry,
                f.request.invocation_id,
                requirements,
                availability,
                authorization,
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
        };

        let mut unsupported_requirements = requirements.clone();
        unsupported_requirements.contract_version = ProtocolVersion::new(2, 0);
        let mut local_only_requirements = requirements.clone();
        local_only_requirements.privacy_preference = vec![PrivacyClass::LocalOnly];
        let mut mixed_requirements = requirements.clone();
        mixed_requirements.privacy_preference =
            vec![PrivacyClass::ApprovedRemote, PrivacyClass::LocalOnly];
        let mut empty_requirements = requirements.clone();
        empty_requirements.privacy_preference.clear();
        let mut duplicate_requirements = requirements.clone();
        duplicate_requirements.privacy_preference =
            vec![PrivacyClass::ApprovedRemote, PrivacyClass::ApprovedRemote];
        let mut unsupported_authorization = valid_authorization.clone();
        unsupported_authorization.contract_version = ProtocolVersion::new(2, 0);
        let mut malformed_authorization = valid_authorization.clone();
        malformed_authorization.prompt_compilation_replay_anchor =
            "authorization-private-sentinel".into();
        let wrong_anchor = RemoteModelAuthorization::new("a".repeat(64), vec![entry]).unwrap();
        let bad_registry = RemoteModelAuthorization::new(
            f.compilation.replay_anchor.clone(),
            vec![RemoteModelAuthorizationEntry {
                model_id: id(99_001, ModelId::new),
                ..entry
            }],
        )
        .unwrap();
        let privacy_mismatch = RemoteModelAuthorization::new(
            f.compilation.replay_anchor.clone(),
            vec![RemoteModelAuthorizationEntry {
                privacy_class: PrivacyClass::RestrictedRemote,
                ..entry
            }],
        )
        .unwrap();
        let empty_authorization =
            RemoteModelAuthorization::new(f.compilation.replay_anchor.clone(), vec![]).unwrap();
        let mut unsupported_availability = valid_availability.clone();
        unsupported_availability.contract_version = ProtocolVersion::new(2, 0);
        let duplicate_availability = ModelAvailabilitySnapshot {
            contract_version: crate::availability::MODEL_AVAILABILITY_V1,
            entries: vec![valid_availability.entries[0], valid_availability.entries[0]],
        };
        let unavailable = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            state: ModelAvailabilityState::Unavailable,
            ..valid_availability.entries[0]
        }])
        .unwrap();
        let missing = ModelAvailabilitySnapshot::new(vec![]).unwrap();
        let unknown = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: id(99_002, ModelProviderId::new),
            model_id: id(99_002, ModelId::new),
            state: ModelAvailabilityState::Available,
        }])
        .unwrap();

        for (r, a, auth, expected) in [
            (
                &unsupported_requirements,
                &valid_availability,
                &valid_authorization,
                RemoteAuthorizationError::InvalidRemoteRequirements,
            ),
            (
                &local_only_requirements,
                &valid_availability,
                &valid_authorization,
                RemoteAuthorizationError::InvalidRemoteRequirements,
            ),
            (
                &mixed_requirements,
                &valid_availability,
                &valid_authorization,
                RemoteAuthorizationError::InvalidRemoteRequirements,
            ),
            (
                &empty_requirements,
                &valid_availability,
                &valid_authorization,
                RemoteAuthorizationError::InvalidRemoteRequirements,
            ),
            (
                &duplicate_requirements,
                &valid_availability,
                &valid_authorization,
                RemoteAuthorizationError::InvalidRemoteRequirements,
            ),
            (
                &requirements,
                &valid_availability,
                &unsupported_authorization,
                RemoteAuthorizationError::UnsupportedAuthorizationVersion,
            ),
            (
                &requirements,
                &valid_availability,
                &malformed_authorization,
                RemoteAuthorizationError::InvalidAuthorizationEvidence,
            ),
            (
                &requirements,
                &valid_availability,
                &wrong_anchor,
                RemoteAuthorizationError::PromptCompilationAssociation,
            ),
            (
                &requirements,
                &valid_availability,
                &bad_registry,
                RemoteAuthorizationError::AuthorizationRegistryInconsistency,
            ),
            (
                &requirements,
                &valid_availability,
                &privacy_mismatch,
                RemoteAuthorizationError::AuthorizationRegistryInconsistency,
            ),
            (
                &requirements,
                &unsupported_availability,
                &valid_authorization,
                RemoteAuthorizationError::AvailabilitySelection(
                    ModelAvailabilityError::UnsupportedAvailabilityVersion,
                ),
            ),
            (
                &requirements,
                &duplicate_availability,
                &valid_authorization,
                RemoteAuthorizationError::AvailabilitySelection(
                    ModelAvailabilityError::InvalidAvailability,
                ),
            ),
            (
                &requirements,
                &unknown,
                &valid_authorization,
                RemoteAuthorizationError::AvailabilitySelection(
                    ModelAvailabilityError::RegistryInconsistency,
                ),
            ),
            (
                &requirements,
                &unavailable,
                &valid_authorization,
                RemoteAuthorizationError::Selection(ModelSelectionError::NoEligibleModel),
            ),
            (
                &requirements,
                &missing,
                &valid_authorization,
                RemoteAuthorizationError::Selection(ModelSelectionError::NoEligibleModel),
            ),
            (
                &requirements,
                &valid_availability,
                &empty_authorization,
                RemoteAuthorizationError::Selection(ModelSelectionError::NoEligibleModel),
            ),
        ] {
            let error = Outer::AuthorizationAvailabilitySelection(expected);
            assert_eq!(call(r, a, auth, &f.compilation), Err(error));
            assert!(!format!("{error:?} {error}").contains("authorization-private-sentinel"));
            assert_eq!(tokenizer.remaining().unwrap(), 1);
            assert_eq!(provider.remaining(), 1);
        }
        let mut tampered_compilation = f.compilation.clone();
        tampered_compilation.compiled_bytes += 1;
        assert_eq!(
            call(
                &requirements,
                &valid_availability,
                &valid_authorization,
                &tampered_compilation
            ),
            Err(Outer::AuthorizationAvailabilitySelection(
                RemoteAuthorizationError::PromptCompilationAssociation
            ))
        );
        assert_eq!(tokenizer.remaining().unwrap(), 1);
        assert_eq!(provider.remaining(), 1);
    }

    #[test]
    fn authorized_remote_tokenized_composition_is_exact_single_attempt_and_content_free() {
        use crate::admission::AdmissionError;
        use crate::authorization::{RemoteModelAuthorization, RemoteModelAuthorizationEntry};
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::{
            select_authorized_available_remote_model_tokenize_invoke_and_admit,
            AuthorizedAvailableRemoteTokenizedInvocationAdmissionError as Outer,
            TokenizedInvocationAdmissionError as Inner,
        };
        use crate::model::{
            LanguageModelProvider, ModelErrorKind, PrivacyClass, RawModelOutput,
            ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::tokenization::{
            ModelInputTokenizationError, ModelRequestTokenCapacityError,
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome,
            TokenizeAndValidateModelRequestCapacityError as Capacity, MODEL_INPUT_TOKENIZATION_V1,
        };
        use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId, ProtocolVersion};
        use std::sync::Arc;

        for (mode, reverse_registry_order) in (0..7).flat_map(|mode| [(mode, false), (mode, true)])
        {
            let mut f = admission_fixture();
            f.context.tokenizer_profile_id = "knowledge-private-sentinel".into();
            let invocation_id = id(
                99_100 + mode * 2 + u128::from(reverse_registry_order),
                ModelInvocationId::new,
            );
            let mut selected_descriptor = f.descriptor.clone();
            selected_descriptor.provider_id = id(99_110, ModelProviderId::new);
            selected_descriptor.model_id = id(99_110, ModelId::new);
            selected_descriptor.privacy_class = PrivacyClass::ApprovedRemote;
            f.response.invocation_id = invocation_id;
            f.response.provider_id = selected_descriptor.provider_id;
            f.response.model_id = selected_descriptor.model_id;
            if mode == 6 {
                f.response.output = RawModelOutput::new("model-output-private-sentinel").unwrap();
            }
            let outcome = if mode == 5 {
                ScriptedOutcome::Error(ModelErrorKind::Unavailable)
            } else {
                ScriptedOutcome::Response(f.response.clone())
            };
            let selected = Arc::new(SentinelProvider {
                inner: ScriptedModelProvider::new(
                    selected_descriptor.clone(),
                    [outcome, ScriptedOutcome::Error(ModelErrorKind::Internal)],
                )
                .unwrap(),
                endpoint: "endpoint-private-sentinel".into(),
                credential: "credential-private-sentinel".into(),
                private_diagnostic: "provider-private-sentinel".into(),
            });
            let mut other_descriptor = selected_descriptor.clone();
            other_descriptor.provider_id = id(99_120, ModelProviderId::new);
            other_descriptor.model_id = id(99_120, ModelId::new);
            let other = Arc::new(
                ScriptedModelProvider::new(
                    other_descriptor.clone(),
                    [ScriptedOutcome::Error(ModelErrorKind::Internal)],
                )
                .unwrap(),
            );
            let mut local_descriptor = selected_descriptor.clone();
            local_descriptor.provider_id = id(99_130, ModelProviderId::new);
            local_descriptor.model_id = id(99_130, ModelId::new);
            local_descriptor.privacy_class = PrivacyClass::LocalOnly;
            let local = Arc::new(
                ScriptedModelProvider::new(
                    local_descriptor,
                    [ScriptedOutcome::Error(ModelErrorKind::Internal)],
                )
                .unwrap(),
            );
            let providers: Vec<Arc<dyn LanguageModelProvider>> = if reverse_registry_order {
                vec![other.clone(), local.clone(), selected.clone()]
            } else {
                vec![selected.clone(), local.clone(), other.clone()]
            };
            let registry = ModelRegistry::try_from_providers(providers).unwrap();
            let availability = ModelAvailabilitySnapshot::new(vec![
                ModelAvailabilityEntry {
                    provider_id: selected_descriptor.provider_id,
                    model_id: selected_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
                ModelAvailabilityEntry {
                    provider_id: other_descriptor.provider_id,
                    model_id: other_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
            ])
            .unwrap();
            let authorization = RemoteModelAuthorization::new(
                f.compilation.replay_anchor.clone(),
                vec![
                    RemoteModelAuthorizationEntry {
                        provider_id: selected_descriptor.provider_id,
                        model_id: selected_descriptor.model_id,
                        privacy_class: selected_descriptor.privacy_class,
                    },
                    RemoteModelAuthorizationEntry {
                        provider_id: other_descriptor.provider_id,
                        model_id: other_descriptor.model_id,
                        privacy_class: other_descriptor.privacy_class,
                    },
                ],
            )
            .unwrap();
            let requirements = remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]);
            let exact = selected_descriptor.capabilities.context_window_tokens
                - requirements.maximum_output_tokens;
            let tokenizer_descriptor = if mode == 1 {
                other_descriptor.clone()
            } else {
                selected_descriptor.clone()
            };
            let token_outcome = match mode {
                2 => ScriptedTokenizationOutcome::Error,
                3 => ScriptedTokenizationOutcome::TokenCount(exact + 1),
                _ => ScriptedTokenizationOutcome::TokenCount(exact),
            };
            let tokenizer = SentinelTokenizer {
                inner: ScriptedModelInputTokenizer::new(tokenizer_descriptor, [token_outcome])
                    .unwrap(),
                private_diagnostic: "tokenizer-private-sentinel".into(),
            };
            let version = if mode == 0 {
                ProtocolVersion::new(2, 0)
            } else {
                MODEL_INPUT_TOKENIZATION_V1
            };
            let result = select_authorized_available_remote_model_tokenize_invoke_and_admit(
                &registry,
                invocation_id,
                &requirements,
                &availability,
                &authorization,
                version,
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            );
            match mode {
                0 => assert_eq!(
                    result,
                    Err(Outer::TokenizedInvocationAdmission(
                        Inner::TokenizationCapacity(Capacity::Tokenization(
                            ModelInputTokenizationError::UnsupportedVersion
                        ))
                    ))
                ),
                1 => assert_eq!(
                    result,
                    Err(Outer::TokenizedInvocationAdmission(
                        Inner::TokenizationCapacity(Capacity::Tokenization(
                            ModelInputTokenizationError::InvalidDescriptor
                        ))
                    ))
                ),
                2 => assert_eq!(
                    result,
                    Err(Outer::TokenizedInvocationAdmission(
                        Inner::TokenizationCapacity(Capacity::Tokenization(
                            ModelInputTokenizationError::TokenizerFailure
                        ))
                    ))
                ),
                3 => assert_eq!(
                    result,
                    Err(Outer::TokenizedInvocationAdmission(
                        Inner::TokenizationCapacity(Capacity::TokenCapacity(
                            ModelRequestTokenCapacityError::ExactCapacity
                        ))
                    ))
                ),
                4 => {
                    let result = result.clone().unwrap();
                    assert_eq!(result.tokenization_evidence.input_token_count, exact);
                    let request = crate::model::ModelRequest {
                        invocation_id,
                        provider_id: selected_descriptor.provider_id,
                        model_id: selected_descriptor.model_id,
                        contract_version: crate::model::MODEL_INVOCATION_V1,
                        input: f.compilation.model_input.clone(),
                        required_capabilities: requirements.required_capabilities.clone(),
                        maximum_output_tokens: requirements.maximum_output_tokens,
                    };
                    assert_eq!(
                        result.admission,
                        crate::admission::admit_model_output(
                            &selected_descriptor,
                            &request,
                            &f.response,
                            &f.compilation,
                            &f.authority,
                            &f.context,
                            &f.citations
                        )
                        .unwrap()
                    );
                    result
                        .tokenization_evidence
                        .validate_for(&selected_descriptor, &f.compilation.model_input)
                        .unwrap();
                }
                5 => assert_eq!(
                    result,
                    Err(Outer::TokenizedInvocationAdmission(Inner::Invocation(
                        ModelErrorKind::Unavailable
                    )))
                ),
                _ => assert_eq!(
                    result,
                    Err(Outer::TokenizedInvocationAdmission(Inner::Admission(
                        AdmissionError::MalformedSyntax
                    )))
                ),
            }
            assert_eq!(tokenizer.inner.remaining().unwrap(), usize::from(mode < 2));
            assert_eq!(selected.inner.remaining(), if mode < 4 { 2 } else { 1 });
            assert_eq!(other.remaining(), 1);
            assert_eq!(local.remaining(), 1);
            if let Err(error) = result {
                let diagnostics = format!("{error:?} {error}");
                for sentinel in [
                    "prompt-private-sentinel",
                    "learner-private-sentinel",
                    "knowledge-private-sentinel",
                    "authorization-private-sentinel",
                    "tokenizer-private-sentinel",
                    "provider-private-sentinel",
                    "endpoint-private-sentinel",
                    "credential-private-sentinel",
                    "model-output-private-sentinel",
                ] {
                    assert!(!diagnostics.contains(sentinel), "leaked {sentinel}");
                }
            }
        }
    }

    fn filtered_remote_fixture(
        privacy: crate::model::PrivacyClass,
    ) -> crate::remote_prompt::RemotePromptFilterResult {
        filtered_remote_fixture_with_source(privacy).1
    }

    fn filtered_remote_fixture_with_source(
        privacy: crate::model::PrivacyClass,
    ) -> (
        crate::prompt::PromptCompilationResult,
        crate::remote_prompt::RemotePromptFilterResult,
    ) {
        use crate::prompt::{
            compile_prompt, PromptCompilationRequest, PromptContent, PromptLayer, PromptLayerKind,
            PromptLimits, PROMPT_COMPILATION_V1,
        };
        use crate::remote_prompt::{
            filter_and_compile_remote_prompt, RemotePromptDisclosurePolicy,
            RemotePromptLayerDisposition, RemotePromptLayerRule,
        };

        let layer = |kind, text| PromptLayer {
            kind,
            classification: kind.classification(),
            content: PromptContent::new(text).unwrap(),
        };
        let source = PromptCompilationRequest {
            contract_version: PROMPT_COMPILATION_V1,
            prompt_package_version: V1,
            context_builder_version: V1,
            output_schema_version: V1,
            limits: PromptLimits {
                maximum_layer_bytes: 1000,
                maximum_compiled_bytes: 10000,
            },
            layers: vec![
                layer(PromptLayerKind::PlatformContract, "prompt-private-sentinel"),
                layer(PromptLayerKind::NexaIdentity, "identity-private-sentinel"),
                layer(PromptLayerKind::Policy, "policy"),
                layer(PromptLayerKind::Pedagogy, "pedagogy"),
                layer(PromptLayerKind::LearnerContext, "learner-private-sentinel"),
                layer(
                    PromptLayerKind::GovernedKnowledgeContext,
                    "knowledge-private-sentinel",
                ),
                layer(
                    PromptLayerKind::ConversationContext,
                    "conversation-private-sentinel",
                ),
                layer(
                    PromptLayerKind::PermittedToolContext,
                    "tool-private-sentinel",
                ),
                layer(PromptLayerKind::StudentInput, "input"),
                layer(PromptLayerKind::OutputContract, "output"),
            ],
        };
        let policy = RemotePromptDisclosurePolicy::new(
            privacy,
            crate::prompt::CANONICAL_LAYER_ORDER
                .into_iter()
                .map(|kind| RemotePromptLayerRule {
                    kind,
                    disposition: if matches!(
                        kind,
                        PromptLayerKind::GovernedKnowledgeContext
                            | PromptLayerKind::ConversationContext
                            | PromptLayerKind::PermittedToolContext
                    ) {
                        RemotePromptLayerDisposition::Omit
                    } else {
                        RemotePromptLayerDisposition::Include
                    },
                })
                .collect(),
        )
        .unwrap();
        let source_compilation = compile_prompt(&source).unwrap();
        let filtered = filter_and_compile_remote_prompt(&source, &policy).unwrap();
        (source_compilation, filtered)
    }

    /// Provider used to observe ADR-0025 admission failures without performing adapter-side
    /// response validation before the host admission boundary.
    struct RawRecordingModelProvider {
        descriptor: crate::model::ModelDescriptor,
        requests: std::sync::Mutex<Vec<crate::model::ModelRequest>>,
        outcomes: std::sync::Mutex<
            std::collections::VecDeque<
                Result<crate::model::ModelResponse, crate::model::ModelErrorKind>,
            >,
        >,
    }

    impl RawRecordingModelProvider {
        fn new(
            descriptor: crate::model::ModelDescriptor,
            outcomes: impl IntoIterator<
                Item = Result<crate::model::ModelResponse, crate::model::ModelErrorKind>,
            >,
        ) -> Self {
            Self {
                descriptor,
                requests: std::sync::Mutex::new(Vec::new()),
                outcomes: std::sync::Mutex::new(outcomes.into_iter().collect()),
            }
        }

        fn request_count(&self) -> usize {
            self.requests.lock().unwrap().len()
        }
        fn remaining(&self) -> usize {
            self.outcomes.lock().unwrap().len()
        }
    }

    impl crate::model::LanguageModelProvider for RawRecordingModelProvider {
        fn descriptor(&self) -> &crate::model::ModelDescriptor {
            &self.descriptor
        }

        fn generate(
            &self,
            request: &crate::model::ModelRequest,
        ) -> Result<crate::model::ModelResponse, crate::model::ModelError> {
            self.requests.lock().unwrap().push(request.clone());
            match self.outcomes.lock().unwrap().pop_front() {
                Some(Ok(response)) => Ok(response),
                Some(Err(kind)) => Err(crate::model::ModelError::new(kind)),
                None => Err(crate::model::ModelError::new(
                    crate::model::ModelErrorKind::ScriptExhausted,
                )),
            }
        }
    }

    #[test]
    fn filtered_authorized_available_remote_invocation_constructs_exact_filtered_request_for_both_privacy_classes(
    ) {
        use crate::admission::admit_model_output;
        use crate::authorization::{RemoteModelAuthorization, RemoteModelAuthorizationEntry};
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::select_filtered_authorized_available_remote_model_invoke_and_admit;
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedOutcome, MODEL_INVOCATION_V1,
        };
        use crate::registry::ModelRegistry;
        use nexa_domain::{ModelInvocationId, ModelProviderId};
        use std::sync::Arc;

        for (offset, privacy) in [PrivacyClass::ApprovedRemote, PrivacyClass::RestrictedRemote]
            .into_iter()
            .enumerate()
        {
            let f = admission_fixture();
            let filtered = filtered_remote_fixture(privacy);
            let invocation_id = id(1200 + offset as u128, ModelInvocationId::new);
            let mut descriptor = f.descriptor.clone();
            descriptor.provider_id = id(1210 + offset as u128, ModelProviderId::new);
            descriptor.privacy_class = privacy;
            let mut response = f.response.clone();
            response.invocation_id = invocation_id;
            response.provider_id = descriptor.provider_id;
            let provider = Arc::new(RecordingModelProvider::new(
                descriptor.clone(),
                [
                    ScriptedOutcome::Response(response.clone()),
                    ScriptedOutcome::Error(crate::model::ModelErrorKind::Internal),
                ],
            ));
            let handle: Arc<dyn LanguageModelProvider> = provider.clone();
            let registry = ModelRegistry::try_from_providers([Arc::clone(&handle)]).unwrap();
            let availability = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
                provider_id: descriptor.provider_id,
                model_id: descriptor.model_id,
                state: ModelAvailabilityState::Available,
            }])
            .unwrap();
            let authorization = RemoteModelAuthorization::new(
                filtered.filtered_compilation.replay_anchor.clone(),
                vec![RemoteModelAuthorizationEntry {
                    provider_id: descriptor.provider_id,
                    model_id: descriptor.model_id,
                    privacy_class: privacy,
                }],
            )
            .unwrap();
            let requirements = remote_selection_requirements(vec![privacy]);

            let admitted = select_filtered_authorized_available_remote_model_invoke_and_admit(
                &registry,
                invocation_id,
                &requirements,
                &availability,
                &authorization,
                &filtered,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap();

            assert_eq!(provider.remaining(), 1);
            assert!(Arc::ptr_eq(
                &registry
                    .resolve(descriptor.provider_id, descriptor.model_id)
                    .unwrap(),
                &handle
            ));
            assert_eq!(provider.requests().len(), 1);
            let request = &provider.requests()[0];
            assert_eq!(request.invocation_id, invocation_id);
            assert_eq!(request.provider_id, descriptor.provider_id);
            assert_eq!(request.model_id, descriptor.model_id);
            assert_eq!(request.contract_version, MODEL_INVOCATION_V1);
            assert_eq!(request.input, filtered.filtered_compilation.model_input);
            assert_eq!(
                request.required_capabilities,
                requirements.required_capabilities
            );
            assert_eq!(
                request.maximum_output_tokens,
                requirements.maximum_output_tokens
            );
            let expected = admit_model_output(
                &descriptor,
                request,
                &response,
                &filtered.filtered_compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap();
            assert_eq!(admitted, expected);
            assert!(request.input.as_str().contains("learner-private-sentinel"));
            for omitted in [
                "knowledge-private-sentinel",
                "conversation-private-sentinel",
                "tool-private-sentinel",
            ] {
                assert!(!request.input.as_str().contains(omitted));
            }
        }
    }

    #[test]
    fn filtered_authorized_available_remote_invocation_all_preselection_gates_are_non_consuming() {
        use crate::authorization::{RemoteModelAuthorization, RemoteModelAuthorizationEntry};
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::select_filtered_authorized_available_remote_model_invoke_and_admit;
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::prompt::PromptLayerKind;
        use crate::registry::ModelRegistry;
        use nexa_domain::{ModelInvocationId, ModelProviderId, ProtocolVersion};
        use std::sync::Arc;

        #[derive(Clone, Copy)]
        enum Gate {
            Policy,
            Inventory,
            Partition,
            CompilationEvidence,
            NestedCompilation,
            FinalAnchor,
            EmptyPrivacy,
            Local,
            Mixed,
            MultipleRemote,
            Mismatch,
            Version,
            SourceAuthorization,
            EmptyAuthorization,
            UnknownAuthorization,
            PrivacyDisagreement,
            Unavailable,
            AvailabilityOmission,
            UnknownAvailability,
            Capability,
            Context,
            Output,
        }
        let gates = [
            Gate::Policy,
            Gate::Inventory,
            Gate::Partition,
            Gate::CompilationEvidence,
            Gate::NestedCompilation,
            Gate::FinalAnchor,
            Gate::EmptyPrivacy,
            Gate::Local,
            Gate::Mixed,
            Gate::MultipleRemote,
            Gate::Mismatch,
            Gate::Version,
            Gate::SourceAuthorization,
            Gate::EmptyAuthorization,
            Gate::UnknownAuthorization,
            Gate::PrivacyDisagreement,
            Gate::Unavailable,
            Gate::AvailabilityOmission,
            Gate::UnknownAvailability,
            Gate::Capability,
            Gate::Context,
            Gate::Output,
        ];
        for (index, gate) in gates.into_iter().enumerate() {
            let f = admission_fixture();
            let mut filtered = filtered_remote_fixture(PrivacyClass::ApprovedRemote);
            let mut descriptor = f.descriptor.clone();
            descriptor.privacy_class = PrivacyClass::ApprovedRemote;
            if matches!(gate, Gate::Capability) {
                descriptor.capabilities.vision = false;
            }
            if matches!(gate, Gate::Context) {
                descriptor.capabilities.context_window_tokens = 1000;
            }
            if matches!(gate, Gate::Output) {
                descriptor.capabilities.maximum_output_tokens = 1;
            }
            let provider = Arc::new(
                ScriptedModelProvider::new(
                    descriptor.clone(),
                    [ScriptedOutcome::Error(
                        crate::model::ModelErrorKind::Internal,
                    )],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                provider.clone() as Arc<dyn LanguageModelProvider>
            ])
            .unwrap();
            let state = if matches!(gate, Gate::Unavailable) {
                ModelAvailabilityState::Unavailable
            } else {
                ModelAvailabilityState::Available
            };
            let availability_id = if matches!(gate, Gate::UnknownAvailability) {
                id(999, ModelProviderId::new)
            } else {
                descriptor.provider_id
            };
            let availability =
                ModelAvailabilitySnapshot::new(if matches!(gate, Gate::AvailabilityOmission) {
                    vec![]
                } else {
                    vec![ModelAvailabilityEntry {
                        provider_id: availability_id,
                        model_id: descriptor.model_id,
                        state,
                    }]
                })
                .unwrap();
            let authorization_anchor = if matches!(gate, Gate::SourceAuthorization) {
                filtered.evidence.source_compilation_replay_anchor.clone()
            } else {
                filtered.filtered_compilation.replay_anchor.clone()
            };
            let authorization_id = if matches!(gate, Gate::UnknownAuthorization) {
                id(998, ModelProviderId::new)
            } else {
                descriptor.provider_id
            };
            let auth_privacy = if matches!(gate, Gate::PrivacyDisagreement) {
                PrivacyClass::RestrictedRemote
            } else {
                PrivacyClass::ApprovedRemote
            };
            let mut authorization = RemoteModelAuthorization::new(
                authorization_anchor,
                vec![RemoteModelAuthorizationEntry {
                    provider_id: authorization_id,
                    model_id: descriptor.model_id,
                    privacy_class: auth_privacy,
                }],
            )
            .unwrap();
            if matches!(gate, Gate::EmptyAuthorization) {
                authorization.entries.clear();
            }
            let mut requirements =
                remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]);
            match gate {
                Gate::Policy => filtered.evidence.policy_replay_anchor = "0".repeat(64),
                Gate::Inventory => {
                    filtered.evidence.source_present_layer_kinds.pop();
                }
                Gate::Partition => filtered
                    .evidence
                    .included_layer_kinds
                    .push(PromptLayerKind::LearnerContext),
                Gate::CompilationEvidence => {
                    filtered.evidence.filtered_compilation_replay_anchor = "0".repeat(64)
                }
                Gate::NestedCompilation => {
                    filtered.filtered_compilation.replay_anchor = "0".repeat(64)
                }
                Gate::FinalAnchor => filtered.evidence.filter_replay_anchor = "0".repeat(64),
                Gate::EmptyPrivacy => requirements.privacy_preference.clear(),
                Gate::Local => requirements.privacy_preference = vec![PrivacyClass::LocalOnly],
                Gate::Mixed => {
                    requirements.privacy_preference =
                        vec![PrivacyClass::LocalOnly, PrivacyClass::ApprovedRemote]
                }
                Gate::MultipleRemote => {
                    requirements.privacy_preference =
                        vec![PrivacyClass::ApprovedRemote, PrivacyClass::RestrictedRemote]
                }
                Gate::Mismatch => {
                    requirements.privacy_preference = vec![PrivacyClass::RestrictedRemote]
                }
                Gate::Version => requirements.contract_version = ProtocolVersion::new(9, 9),
                Gate::Capability => requirements.required_capabilities.vision = true,
                Gate::Output => requirements.maximum_output_tokens = 1000,
                _ => {}
            }
            let result = select_filtered_authorized_available_remote_model_invoke_and_admit(
                &registry,
                id(1400 + index as u128, ModelInvocationId::new),
                &requirements,
                &availability,
                &authorization,
                &filtered,
                &f.authority,
                &f.context,
                &f.citations,
            );
            assert!(result.is_err(), "gate {index} must fail closed");
            assert_eq!(
                provider.remaining(),
                1,
                "gate {index} consumed a provider outcome"
            );
        }
    }

    #[test]
    fn filtered_authorized_available_remote_invocation_filtered_authorization_does_not_bind_source_compilation(
    ) {
        use crate::authorization::{
            select_authorized_available_remote_model, RemoteAuthorizationError,
            RemoteModelAuthorization, RemoteModelAuthorizationEntry,
        };
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::model::{LanguageModelProvider, PrivacyClass, ScriptedModelProvider};
        use crate::registry::ModelRegistry;
        use std::sync::Arc;

        let f = admission_fixture();
        let (source, filtered) = filtered_remote_fixture_with_source(PrivacyClass::ApprovedRemote);
        let mut descriptor = f.descriptor;
        descriptor.privacy_class = PrivacyClass::ApprovedRemote;
        let provider = Arc::new(ScriptedModelProvider::new(descriptor.clone(), []).unwrap());
        let registry =
            ModelRegistry::try_from_providers([provider as Arc<dyn LanguageModelProvider>])
                .unwrap();
        let availability = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: descriptor.provider_id,
            model_id: descriptor.model_id,
            state: ModelAvailabilityState::Available,
        }])
        .unwrap();
        let authorization = RemoteModelAuthorization::new(
            filtered.filtered_compilation.replay_anchor,
            vec![RemoteModelAuthorizationEntry {
                provider_id: descriptor.provider_id,
                model_id: descriptor.model_id,
                privacy_class: PrivacyClass::ApprovedRemote,
            }],
        )
        .unwrap();

        assert_eq!(
            select_authorized_available_remote_model(
                &registry,
                &remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]),
                &availability,
                &authorization,
                &source,
            )
            .unwrap_err(),
            RemoteAuthorizationError::PromptCompilationAssociation
        );
    }

    #[test]
    fn filtered_authorized_available_remote_invocation_adr0025_preflight_is_non_consuming() {
        use crate::admission::AdmissionError;
        use crate::authorization::{RemoteModelAuthorization, RemoteModelAuthorizationEntry};
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::{
            select_filtered_authorized_available_remote_model_invoke_and_admit,
            FilteredAuthorizedAvailableRemoteInvocationAdmissionError as Outer,
            InvocationAdmissionError,
        };
        use crate::model::{LanguageModelProvider, PrivacyClass, ScriptedOutcome};
        use crate::registry::ModelRegistry;
        use nexa_domain::{CitationSetId, ContextPackageId, ModelInvocationId, ProtocolVersion};
        use std::sync::Arc;

        for case in 0..5 {
            let f = admission_fixture();
            let filtered = filtered_remote_fixture(PrivacyClass::ApprovedRemote);
            let mut descriptor = f.descriptor.clone();
            descriptor.privacy_class = PrivacyClass::ApprovedRemote;
            if case == 4 {
                descriptor.capabilities.structured_output = false;
            }
            let provider = Arc::new(RecordingModelProvider::new(
                descriptor.clone(),
                [ScriptedOutcome::Error(
                    crate::model::ModelErrorKind::Internal,
                )],
            ));
            let registry = ModelRegistry::try_from_providers([
                provider.clone() as Arc<dyn LanguageModelProvider>
            ])
            .unwrap();
            let availability = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
                provider_id: descriptor.provider_id,
                model_id: descriptor.model_id,
                state: ModelAvailabilityState::Available,
            }])
            .unwrap();
            let authorization = RemoteModelAuthorization::new(
                filtered.filtered_compilation.replay_anchor.clone(),
                vec![RemoteModelAuthorizationEntry {
                    provider_id: descriptor.provider_id,
                    model_id: descriptor.model_id,
                    privacy_class: PrivacyClass::ApprovedRemote,
                }],
            )
            .unwrap();
            let mut authority = f.authority.clone();
            let mut requirements =
                remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]);
            match case {
                0 => authority.contract_version = ProtocolVersion::new(9, 9),
                1 => authority.context_package_id = id(991, ContextPackageId::new),
                2 => authority.citation_set_id = id(992, CitationSetId::new),
                3 | 4 => requirements.required_capabilities.structured_output = false,
                _ => unreachable!(),
            }
            let expected = match case {
                0 => AdmissionError::UnsupportedVersion,
                1 | 2 => AdmissionError::PlanningEvidenceProvenance,
                3 | 4 => AdmissionError::UnsupportedStructuredOutput,
                _ => unreachable!(),
            };
            assert_eq!(
                select_filtered_authorized_available_remote_model_invoke_and_admit(
                    &registry,
                    id(1500 + case, ModelInvocationId::new),
                    &requirements,
                    &availability,
                    &authorization,
                    &filtered,
                    &authority,
                    &f.context,
                    &f.citations
                ),
                Err(Outer::InvocationAdmission(
                    InvocationAdmissionError::Preflight(expected)
                ))
            );
            assert!(provider.requests().is_empty());
            assert_eq!(provider.remaining(), 1);
        }
    }

    #[test]
    fn filtered_authorized_available_remote_invocation_is_exactly_once_strict_and_has_no_fallback()
    {
        use crate::authorization::{RemoteModelAuthorization, RemoteModelAuthorizationEntry};
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::{
            select_filtered_authorized_available_remote_model_invoke_and_admit,
            FilteredAuthorizedAvailableRemoteInvocationAdmissionError as Outer,
            InvocationAdmissionError,
        };
        use crate::model::{
            FinishReason, LanguageModelProvider, ModelErrorKind, PrivacyClass, RawModelOutput,
        };
        use crate::registry::ModelRegistry;
        use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId};
        use std::sync::Arc;

        #[derive(Clone, Copy)]
        enum Case {
            Invocation,
            Provider,
            Model,
            InvocationIdentity,
            Incomplete,
            Malformed,
            InvalidCandidate,
        }
        for (index, case) in [
            Case::Invocation,
            Case::Provider,
            Case::Model,
            Case::InvocationIdentity,
            Case::Incomplete,
            Case::Malformed,
            Case::InvalidCandidate,
        ]
        .into_iter()
        .enumerate()
        {
            let f = admission_fixture();
            let filtered = filtered_remote_fixture(PrivacyClass::ApprovedRemote);
            let invocation_id = id(1300 + index as u128, ModelInvocationId::new);
            let mut selected_descriptor = f.descriptor.clone();
            selected_descriptor.provider_id = id(10, ModelProviderId::new);
            selected_descriptor.model_id = id(10, ModelId::new);
            selected_descriptor.privacy_class = PrivacyClass::ApprovedRemote;
            let mut other_descriptor = selected_descriptor.clone();
            other_descriptor.provider_id = id(20, ModelProviderId::new);
            other_descriptor.model_id = id(20, ModelId::new);
            let mut response = f.response.clone();
            response.invocation_id = invocation_id;
            response.provider_id = selected_descriptor.provider_id;
            response.model_id = selected_descriptor.model_id;
            match case {
                Case::Provider => response.provider_id = other_descriptor.provider_id,
                Case::Model => response.model_id = other_descriptor.model_id,
                Case::InvocationIdentity => {
                    response.invocation_id = id(9999, ModelInvocationId::new)
                }
                Case::Incomplete => response.finish_reason = FinishReason::OutputLimit,
                Case::Malformed => {
                    response.output = RawModelOutput::new(concat!(
                        "{sentinel-malformed prompt-private-sentinel ",
                        "learner-private-sentinel conversation-private-sentinel ",
                        "knowledge-private-sentinel tool-private-sentinel ",
                        "endpoint-private-sentinel credential-private-sentinel ",
                        "provider-private-sentinel"
                    ))
                    .unwrap()
                }
                Case::InvalidCandidate => {
                    response.output = RawModelOutput::new(
                        "{\"candidate_schema_version\":\"1.0\",\"sections\":[]}",
                    )
                    .unwrap()
                }
                Case::Invocation => {}
            }
            let first = if matches!(case, Case::Invocation) {
                Err(ModelErrorKind::Internal)
            } else {
                Ok(response)
            };
            let selected = Arc::new(RawRecordingModelProvider::new(
                selected_descriptor.clone(),
                [first, Err(ModelErrorKind::Unavailable)],
            ));
            let other = Arc::new(RawRecordingModelProvider::new(
                other_descriptor.clone(),
                [
                    Err(ModelErrorKind::Internal),
                    Err(ModelErrorKind::Unavailable),
                ],
            ));
            let handles: Vec<Arc<dyn LanguageModelProvider>> = if index % 2 == 0 {
                vec![other.clone(), selected.clone()]
            } else {
                vec![selected.clone(), other.clone()]
            };
            let registry = ModelRegistry::try_from_providers(handles).unwrap();
            let availability = ModelAvailabilitySnapshot::new(vec![
                ModelAvailabilityEntry {
                    provider_id: other_descriptor.provider_id,
                    model_id: other_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
                ModelAvailabilityEntry {
                    provider_id: selected_descriptor.provider_id,
                    model_id: selected_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
            ])
            .unwrap();
            let authorization = RemoteModelAuthorization::new(
                filtered.filtered_compilation.replay_anchor.clone(),
                vec![
                    RemoteModelAuthorizationEntry {
                        provider_id: other_descriptor.provider_id,
                        model_id: other_descriptor.model_id,
                        privacy_class: PrivacyClass::ApprovedRemote,
                    },
                    RemoteModelAuthorizationEntry {
                        provider_id: selected_descriptor.provider_id,
                        model_id: selected_descriptor.model_id,
                        privacy_class: PrivacyClass::ApprovedRemote,
                    },
                ],
            )
            .unwrap();
            let error = select_filtered_authorized_available_remote_model_invoke_and_admit(
                &registry,
                invocation_id,
                &remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]),
                &availability,
                &authorization,
                &filtered,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap_err();
            assert!(
                matches!(
                    error,
                    Outer::InvocationAdmission(InvocationAdmissionError::Invocation(_))
                ) || matches!(
                    error,
                    Outer::InvocationAdmission(InvocationAdmissionError::Admission(_))
                )
            );
            assert_eq!(
                selected.request_count(),
                1,
                "selected provider must be invoked exactly once"
            );
            assert_eq!(
                selected.remaining(),
                1,
                "selected provider's second outcome must remain"
            );
            assert_eq!(
                other.request_count(),
                0,
                "no fallback or non-selected invocation"
            );
            assert_eq!(
                other.remaining(),
                2,
                "non-selected provider state must be preserved"
            );
            let debug = format!("{error:?}");
            let display = format!("{error}");
            for sentinel in [
                "prompt-private-sentinel",
                "learner-private-sentinel",
                "conversation-private-sentinel",
                "knowledge-private-sentinel",
                "tool-private-sentinel",
                "endpoint-private-sentinel",
                "credential-private-sentinel",
                "provider-private-sentinel",
                "sentinel-malformed",
            ] {
                assert!(!debug.contains(sentinel));
                assert!(!display.contains(sentinel));
            }
        }
    }

    #[test]
    fn filtered_authorized_available_remote_invocation_success_is_canonical_and_exactly_once() {
        use crate::admission::admit_model_output;
        use crate::authorization::{RemoteModelAuthorization, RemoteModelAuthorizationEntry};
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::select_filtered_authorized_available_remote_model_invoke_and_admit;
        use crate::model::{LanguageModelProvider, ModelErrorKind, PrivacyClass};
        use crate::registry::ModelRegistry;
        use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId};
        use std::sync::Arc;

        for reverse_insertion in [false, true] {
            let f = admission_fixture();
            let filtered = filtered_remote_fixture(PrivacyClass::ApprovedRemote);
            let invocation_id = id(1600 + reverse_insertion as u128, ModelInvocationId::new);
            let mut selected_descriptor = f.descriptor.clone();
            selected_descriptor.provider_id = id(10, ModelProviderId::new);
            selected_descriptor.model_id = id(10, ModelId::new);
            selected_descriptor.privacy_class = PrivacyClass::ApprovedRemote;
            let mut other_descriptor = selected_descriptor.clone();
            other_descriptor.provider_id = id(20, ModelProviderId::new);
            other_descriptor.model_id = id(20, ModelId::new);
            let mut response = f.response.clone();
            response.invocation_id = invocation_id;
            response.provider_id = selected_descriptor.provider_id;
            response.model_id = selected_descriptor.model_id;
            let selected = Arc::new(RawRecordingModelProvider::new(
                selected_descriptor.clone(),
                [Ok(response.clone()), Err(ModelErrorKind::Unavailable)],
            ));
            let other = Arc::new(RawRecordingModelProvider::new(
                other_descriptor.clone(),
                [
                    Err(ModelErrorKind::Internal),
                    Err(ModelErrorKind::Unavailable),
                ],
            ));
            let handles: Vec<Arc<dyn LanguageModelProvider>> = if reverse_insertion {
                vec![other.clone(), selected.clone()]
            } else {
                vec![selected.clone(), other.clone()]
            };
            let registry = ModelRegistry::try_from_providers(handles).unwrap();
            let availability = ModelAvailabilitySnapshot::new(vec![
                ModelAvailabilityEntry {
                    provider_id: other_descriptor.provider_id,
                    model_id: other_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
                ModelAvailabilityEntry {
                    provider_id: selected_descriptor.provider_id,
                    model_id: selected_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
            ])
            .unwrap();
            let authorization = RemoteModelAuthorization::new(
                filtered.filtered_compilation.replay_anchor.clone(),
                vec![
                    RemoteModelAuthorizationEntry {
                        provider_id: other_descriptor.provider_id,
                        model_id: other_descriptor.model_id,
                        privacy_class: PrivacyClass::ApprovedRemote,
                    },
                    RemoteModelAuthorizationEntry {
                        provider_id: selected_descriptor.provider_id,
                        model_id: selected_descriptor.model_id,
                        privacy_class: PrivacyClass::ApprovedRemote,
                    },
                ],
            )
            .unwrap();

            let actual = select_filtered_authorized_available_remote_model_invoke_and_admit(
                &registry,
                invocation_id,
                &remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]),
                &availability,
                &authorization,
                &filtered,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap();
            let requests = selected.requests.lock().unwrap();
            assert_eq!(requests.len(), 1);
            let expected = admit_model_output(
                &selected_descriptor,
                &requests[0],
                &response,
                &filtered.filtered_compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap();
            assert_eq!(actual, expected);
            assert_eq!(selected.remaining(), 1);
            assert_eq!(other.request_count(), 0);
            assert_eq!(other.remaining(), 2);
        }
    }

    #[test]
    fn filtered_authorized_available_remote_invocation_selection_failure_is_non_consuming() {
        use crate::authorization::{RemoteModelAuthorization, RemoteModelAuthorizationEntry};
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::{
            select_filtered_authorized_available_remote_model_invoke_and_admit,
            FilteredAuthorizedAvailableRemoteInvocationAdmissionError as Outer,
        };
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::remote_prompt::FilteredRemoteSelectionError;
        use nexa_domain::ModelInvocationId;
        use std::sync::Arc;

        let f = admission_fixture();
        let mut descriptor = f.descriptor.clone();
        descriptor.privacy_class = PrivacyClass::ApprovedRemote;
        let provider = Arc::new(
            ScriptedModelProvider::new(
                descriptor.clone(),
                [ScriptedOutcome::Error(
                    crate::model::ModelErrorKind::Internal,
                )],
            )
            .unwrap(),
        );
        let registry =
            ModelRegistry::try_from_providers([provider.clone() as Arc<dyn LanguageModelProvider>])
                .unwrap();
        let availability = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: descriptor.provider_id,
            model_id: descriptor.model_id,
            state: ModelAvailabilityState::Available,
        }])
        .unwrap();
        let mut filtered = filtered_remote_fixture(PrivacyClass::ApprovedRemote);
        let authorization = RemoteModelAuthorization::new(
            filtered.filtered_compilation.replay_anchor.clone(),
            vec![RemoteModelAuthorizationEntry {
                provider_id: descriptor.provider_id,
                model_id: descriptor.model_id,
                privacy_class: descriptor.privacy_class,
            }],
        )
        .unwrap();
        filtered.evidence.filter_replay_anchor = "0".repeat(64);
        assert_eq!(
            select_filtered_authorized_available_remote_model_invoke_and_admit(
                &registry,
                id(1220, ModelInvocationId::new),
                &remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]),
                &availability,
                &authorization,
                &filtered,
                &f.authority,
                &f.context,
                &f.citations,
            ),
            Err(Outer::FilteredSelection(
                FilteredRemoteSelectionError::FilterEvidence
            ))
        );
        assert_eq!(provider.remaining(), 1);
    }

    #[test]
    fn filtered_authorized_available_remote_invocation_is_single_attempt_and_diagnostics_are_content_free(
    ) {
        use crate::generation::{
            FilteredAuthorizedAvailableRemoteInvocationAdmissionError as Outer,
            InvocationAdmissionError,
        };
        use crate::model::ModelErrorKind;
        use crate::remote_prompt::FilteredRemoteSelectionError;

        let errors = [
            Outer::FilteredSelection(FilteredRemoteSelectionError::FilterEvidence),
            Outer::FilteredSelection(FilteredRemoteSelectionError::FilterPrivacyRequirements),
            Outer::InvocationAdmission(InvocationAdmissionError::Invocation(
                ModelErrorKind::Internal,
            )),
        ];
        for error in errors {
            let diagnostics = format!("{error:?} {error}");
            for sentinel in [
                "prompt-private-sentinel",
                "learner-private-sentinel",
                "conversation-private-sentinel",
                "knowledge-private-sentinel",
                "tool-private-sentinel",
                "endpoint-private-sentinel",
                "credential-private-sentinel",
                "provider-private-sentinel",
            ] {
                assert!(!diagnostics.contains(sentinel));
            }
        }
    }

    #[test]
    fn filtered_remote_tokenized_composition_gates_dependencies_at_adr_0034() {
        use crate::authorization::{RemoteModelAuthorization, RemoteModelAuthorizationEntry};
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::{
            select_filtered_authorized_available_remote_model_tokenize_invoke_and_admit,
            FilteredAuthorizedAvailableRemoteTokenizedInvocationAdmissionError as Outer,
        };
        use crate::model::{
            LanguageModelProvider, ModelErrorKind, PrivacyClass, ScriptedModelProvider,
            ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::remote_prompt::FilteredRemoteSelectionError;
        use crate::tokenization::{
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome, MODEL_INPUT_TOKENIZATION_V1,
        };
        use nexa_domain::ModelInvocationId;
        use std::sync::Arc;

        let f = admission_fixture();
        let mut descriptor = f.descriptor.clone();
        descriptor.privacy_class = PrivacyClass::ApprovedRemote;
        let provider = Arc::new(
            ScriptedModelProvider::new(
                descriptor.clone(),
                [ScriptedOutcome::Error(ModelErrorKind::Internal)],
            )
            .unwrap(),
        );
        let tokenizer = ScriptedModelInputTokenizer::new(
            descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(1)],
        )
        .unwrap();
        let registry =
            ModelRegistry::try_from_providers([provider.clone() as Arc<dyn LanguageModelProvider>])
                .unwrap();
        let availability = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: descriptor.provider_id,
            model_id: descriptor.model_id,
            state: ModelAvailabilityState::Available,
        }])
        .unwrap();
        let mut filtered = filtered_remote_fixture(PrivacyClass::ApprovedRemote);
        let authorization = RemoteModelAuthorization::new(
            filtered.filtered_compilation.replay_anchor.clone(),
            vec![RemoteModelAuthorizationEntry {
                provider_id: descriptor.provider_id,
                model_id: descriptor.model_id,
                privacy_class: descriptor.privacy_class,
            }],
        )
        .unwrap();
        filtered.evidence.filter_replay_anchor = "0".repeat(64);

        assert_eq!(
            select_filtered_authorized_available_remote_model_tokenize_invoke_and_admit(
                &registry,
                id(1700, ModelInvocationId::new),
                &remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]),
                &availability,
                &authorization,
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &filtered,
                &f.authority,
                &f.context,
                &f.citations,
            ),
            Err(Outer::FilteredSelection(
                FilteredRemoteSelectionError::FilterEvidence
            ))
        );
        assert_eq!(tokenizer.remaining().unwrap(), 1);
        assert_eq!(provider.remaining(), 1);
    }

    #[test]
    fn filtered_remote_tokenized_composition_returns_exact_filtered_evidence_and_admission() {
        use crate::admission::admit_model_output;
        use crate::authorization::{RemoteModelAuthorization, RemoteModelAuthorizationEntry};
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::select_filtered_authorized_available_remote_model_tokenize_invoke_and_admit;
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::tokenization::{
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome, MODEL_INPUT_TOKENIZATION_V1,
        };
        use std::sync::Arc;

        for reverse in [false, true] {
            let f = admission_fixture();
            let filtered = filtered_remote_fixture(PrivacyClass::ApprovedRemote);
            let mut descriptor = f.descriptor.clone();
            descriptor.privacy_class = PrivacyClass::ApprovedRemote;
            let selected = Arc::new(
                ScriptedModelProvider::new(
                    descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let mut other_descriptor = descriptor.clone();
            other_descriptor.provider_id = id(999, nexa_domain::ModelProviderId::new);
            other_descriptor.model_id = id(999, nexa_domain::ModelId::new);
            let other = Arc::new(
                ScriptedModelProvider::new(
                    other_descriptor.clone(),
                    [ScriptedOutcome::Error(
                        crate::model::ModelErrorKind::Internal,
                    )],
                )
                .unwrap(),
            );
            let providers: Vec<Arc<dyn LanguageModelProvider>> = if reverse {
                vec![other.clone(), selected.clone()]
            } else {
                vec![selected.clone(), other.clone()]
            };
            let registry = ModelRegistry::try_from_providers(providers).unwrap();
            let availability = ModelAvailabilitySnapshot::new(vec![
                ModelAvailabilityEntry {
                    provider_id: descriptor.provider_id,
                    model_id: descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
                ModelAvailabilityEntry {
                    provider_id: other_descriptor.provider_id,
                    model_id: other_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
            ])
            .unwrap();
            let authorization = RemoteModelAuthorization::new(
                filtered.filtered_compilation.replay_anchor.clone(),
                vec![
                    RemoteModelAuthorizationEntry {
                        provider_id: descriptor.provider_id,
                        model_id: descriptor.model_id,
                        privacy_class: descriptor.privacy_class,
                    },
                    RemoteModelAuthorizationEntry {
                        provider_id: other_descriptor.provider_id,
                        model_id: other_descriptor.model_id,
                        privacy_class: other_descriptor.privacy_class,
                    },
                ],
            )
            .unwrap();
            let tokenizer = ScriptedModelInputTokenizer::new(
                descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(1)],
            )
            .unwrap();
            let result =
                select_filtered_authorized_available_remote_model_tokenize_invoke_and_admit(
                    &registry,
                    f.request.invocation_id,
                    &remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]),
                    &availability,
                    &authorization,
                    MODEL_INPUT_TOKENIZATION_V1,
                    &tokenizer,
                    &filtered,
                    &f.authority,
                    &f.context,
                    &f.citations,
                )
                .unwrap();
            result
                .tokenization_evidence
                .validate_for(&descriptor, &filtered.filtered_compilation.model_input)
                .unwrap();
            let mut expected_request = f.request.clone();
            expected_request.input = filtered.filtered_compilation.model_input.clone();
            let expected = admit_model_output(
                &descriptor,
                &expected_request,
                &f.response,
                &filtered.filtered_compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap();
            assert_eq!(result.admission, expected);
            assert_eq!(tokenizer.remaining().unwrap(), 0);
            assert_eq!(selected.remaining(), 0);
            assert_eq!(other.remaining(), 1);
        }
    }

    #[test]
    fn filtered_remote_tokenized_composition_denials_preserve_exact_categories_and_dependencies() {
        use crate::authorization::{
            RemoteAuthorizationError, RemoteModelAuthorization, RemoteModelAuthorizationEntry,
        };
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilityError, ModelAvailabilitySnapshot,
            ModelAvailabilityState,
        };
        use crate::generation::{
            select_filtered_authorized_available_remote_model_tokenize_invoke_and_admit,
            FilteredAuthorizedAvailableRemoteTokenizedInvocationAdmissionError as Outer,
        };
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::selection::ModelSelectionError;
        use crate::tokenization::{
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome, MODEL_INPUT_TOKENIZATION_V1,
        };
        use nexa_domain::{ModelId, ModelProviderId, ProtocolVersion};
        use std::sync::Arc;

        let f = admission_fixture();
        let filtered = filtered_remote_fixture(PrivacyClass::ApprovedRemote);
        let mut descriptor = f.descriptor.clone();
        descriptor.privacy_class = PrivacyClass::ApprovedRemote;
        let provider = Arc::new(
            ScriptedModelProvider::new(
                descriptor.clone(),
                [ScriptedOutcome::Response(f.response.clone())],
            )
            .unwrap(),
        );
        let registry =
            ModelRegistry::try_from_providers([provider.clone() as Arc<dyn LanguageModelProvider>])
                .unwrap();
        let entry = RemoteModelAuthorizationEntry {
            provider_id: descriptor.provider_id,
            model_id: descriptor.model_id,
            privacy_class: descriptor.privacy_class,
        };
        let valid_authorization = RemoteModelAuthorization::new(
            filtered.filtered_compilation.replay_anchor.clone(),
            vec![entry],
        )
        .unwrap();
        let valid_availability = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: descriptor.provider_id,
            model_id: descriptor.model_id,
            state: ModelAvailabilityState::Available,
        }])
        .unwrap();
        let tokenizer = ScriptedModelInputTokenizer::new(
            descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(1)],
        )
        .unwrap();
        let requirements = remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]);
        let call =
            |requirements: &crate::selection::ModelSelectionRequirements,
             availability: &ModelAvailabilitySnapshot,
             authorization: &RemoteModelAuthorization,
             filtered_result: &crate::remote_prompt::RemotePromptFilterResult| {
                select_filtered_authorized_available_remote_model_tokenize_invoke_and_admit(
                    &registry,
                    f.request.invocation_id,
                    requirements,
                    availability,
                    authorization,
                    MODEL_INPUT_TOKENIZATION_V1,
                    &tokenizer,
                    filtered_result,
                    &f.authority,
                    &f.context,
                    &f.citations,
                )
            };

        // Every malformed, tampered, incomplete, duplicate, non-canonical, or reassociated
        // ADR-0033 result is rejected before either dependency is consumed.
        for mutation in 0..16 {
            let mut invalid = filtered.clone();
            match mutation {
                0 => invalid.policy.contract_version = ProtocolVersion::new(2, 0),
                1 => invalid.evidence.contract_version = ProtocolVersion::new(2, 0),
                2 => {
                    invalid.policy.rules.pop();
                }
                3 => invalid.policy.rules.push(invalid.policy.rules[0]),
                4 => invalid.policy.rules.swap(0, 1),
                5 => {
                    invalid.policy.rules[0].disposition =
                        crate::remote_prompt::RemotePromptLayerDisposition::Omit
                }
                6 => invalid.policy.target_privacy_class = PrivacyClass::RestrictedRemote,
                7 => invalid.evidence.target_privacy_class = PrivacyClass::RestrictedRemote,
                8 => {
                    invalid.evidence.source_present_layer_kinds.pop();
                }
                9 => invalid
                    .evidence
                    .source_present_layer_kinds
                    .push(invalid.evidence.source_present_layer_kinds[0]),
                10 => invalid.evidence.source_present_layer_kinds.swap(0, 1),
                11 => {
                    invalid.evidence.included_layer_kinds.pop();
                }
                12 => invalid
                    .evidence
                    .omitted_layer_kinds
                    .push(invalid.evidence.included_layer_kinds[0]),
                13 => invalid.evidence.policy_replay_anchor = "a".repeat(64),
                14 => invalid.evidence.source_compilation_replay_anchor = "b".repeat(64),
                _ => invalid.evidence.filtered_compilation_replay_anchor = "c".repeat(64),
            }
            assert_eq!(
                call(
                    &requirements,
                    &valid_availability,
                    &valid_authorization,
                    &invalid,
                ),
                Err(Outer::FilteredSelection(
                    crate::remote_prompt::FilteredRemoteSelectionError::FilterEvidence,
                )),
                "filter mutation {mutation}",
            );
            assert_eq!(tokenizer.remaining().unwrap(), 1);
            assert_eq!(provider.remaining(), 1);
        }

        let mut unsupported_requirements = requirements.clone();
        unsupported_requirements.contract_version = ProtocolVersion::new(2, 0);
        let mut local_only_requirements = requirements.clone();
        local_only_requirements.privacy_preference = vec![PrivacyClass::LocalOnly];
        let mut mixed_requirements = requirements.clone();
        mixed_requirements.privacy_preference =
            vec![PrivacyClass::ApprovedRemote, PrivacyClass::LocalOnly];
        let mut empty_requirements = requirements.clone();
        empty_requirements.privacy_preference.clear();
        let mut duplicate_requirements = requirements.clone();
        duplicate_requirements.privacy_preference =
            vec![PrivacyClass::ApprovedRemote, PrivacyClass::ApprovedRemote];
        let mut unsupported_authorization = valid_authorization.clone();
        unsupported_authorization.contract_version = ProtocolVersion::new(2, 0);
        let mut malformed_authorization = valid_authorization.clone();
        malformed_authorization.prompt_compilation_replay_anchor =
            "authorization-private-sentinel".into();
        let wrong_anchor = RemoteModelAuthorization::new("a".repeat(64), vec![entry]).unwrap();
        let source_compilation_authorization = RemoteModelAuthorization::new(
            filtered.evidence.source_compilation_replay_anchor.clone(),
            vec![entry],
        )
        .unwrap();
        let bad_registry = RemoteModelAuthorization::new(
            filtered.filtered_compilation.replay_anchor.clone(),
            vec![RemoteModelAuthorizationEntry {
                model_id: id(99_001, ModelId::new),
                ..entry
            }],
        )
        .unwrap();
        let privacy_mismatch = RemoteModelAuthorization::new(
            filtered.filtered_compilation.replay_anchor.clone(),
            vec![RemoteModelAuthorizationEntry {
                privacy_class: PrivacyClass::RestrictedRemote,
                ..entry
            }],
        )
        .unwrap();
        let empty_authorization = RemoteModelAuthorization::new(
            filtered.filtered_compilation.replay_anchor.clone(),
            vec![],
        )
        .unwrap();
        let mut unsupported_availability = valid_availability.clone();
        unsupported_availability.contract_version = ProtocolVersion::new(2, 0);
        let duplicate_availability = ModelAvailabilitySnapshot {
            contract_version: crate::availability::MODEL_AVAILABILITY_V1,
            entries: vec![valid_availability.entries[0], valid_availability.entries[0]],
        };
        let unavailable = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            state: ModelAvailabilityState::Unavailable,
            ..valid_availability.entries[0]
        }])
        .unwrap();
        let missing = ModelAvailabilitySnapshot::new(vec![]).unwrap();
        let unknown = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: id(99_002, ModelProviderId::new),
            model_id: id(99_002, ModelId::new),
            state: ModelAvailabilityState::Available,
        }])
        .unwrap();

        for (r, a, auth, expected) in [
            (
                &unsupported_requirements,
                &valid_availability,
                &valid_authorization,
                RemoteAuthorizationError::InvalidRemoteRequirements,
            ),
            (
                &local_only_requirements,
                &valid_availability,
                &valid_authorization,
                RemoteAuthorizationError::InvalidRemoteRequirements,
            ),
            (
                &mixed_requirements,
                &valid_availability,
                &valid_authorization,
                RemoteAuthorizationError::InvalidRemoteRequirements,
            ),
            (
                &empty_requirements,
                &valid_availability,
                &valid_authorization,
                RemoteAuthorizationError::InvalidRemoteRequirements,
            ),
            (
                &duplicate_requirements,
                &valid_availability,
                &valid_authorization,
                RemoteAuthorizationError::InvalidRemoteRequirements,
            ),
            (
                &requirements,
                &valid_availability,
                &unsupported_authorization,
                RemoteAuthorizationError::UnsupportedAuthorizationVersion,
            ),
            (
                &requirements,
                &valid_availability,
                &malformed_authorization,
                RemoteAuthorizationError::InvalidAuthorizationEvidence,
            ),
            (
                &requirements,
                &valid_availability,
                &wrong_anchor,
                RemoteAuthorizationError::PromptCompilationAssociation,
            ),
            (
                &requirements,
                &valid_availability,
                &source_compilation_authorization,
                RemoteAuthorizationError::PromptCompilationAssociation,
            ),
            (
                &requirements,
                &valid_availability,
                &bad_registry,
                RemoteAuthorizationError::AuthorizationRegistryInconsistency,
            ),
            (
                &requirements,
                &valid_availability,
                &privacy_mismatch,
                RemoteAuthorizationError::AuthorizationRegistryInconsistency,
            ),
            (
                &requirements,
                &unsupported_availability,
                &valid_authorization,
                RemoteAuthorizationError::AvailabilitySelection(
                    ModelAvailabilityError::UnsupportedAvailabilityVersion,
                ),
            ),
            (
                &requirements,
                &duplicate_availability,
                &valid_authorization,
                RemoteAuthorizationError::AvailabilitySelection(
                    ModelAvailabilityError::InvalidAvailability,
                ),
            ),
            (
                &requirements,
                &unknown,
                &valid_authorization,
                RemoteAuthorizationError::AvailabilitySelection(
                    ModelAvailabilityError::RegistryInconsistency,
                ),
            ),
            (
                &requirements,
                &unavailable,
                &valid_authorization,
                RemoteAuthorizationError::Selection(ModelSelectionError::NoEligibleModel),
            ),
            (
                &requirements,
                &missing,
                &valid_authorization,
                RemoteAuthorizationError::Selection(ModelSelectionError::NoEligibleModel),
            ),
            (
                &requirements,
                &valid_availability,
                &empty_authorization,
                RemoteAuthorizationError::Selection(ModelSelectionError::NoEligibleModel),
            ),
        ] {
            let error = if expected == RemoteAuthorizationError::InvalidRemoteRequirements {
                Outer::FilteredSelection(
                    crate::remote_prompt::FilteredRemoteSelectionError::FilterPrivacyRequirements,
                )
            } else {
                Outer::FilteredSelection(
                    crate::remote_prompt::FilteredRemoteSelectionError::AuthorizedSelection(
                        expected,
                    ),
                )
            };
            assert_eq!(call(r, a, auth, &filtered), Err(error));
            assert!(!format!("{error:?} {error}").contains("authorization-private-sentinel"));
            assert_eq!(tokenizer.remaining().unwrap(), 1);
            assert_eq!(provider.remaining(), 1);
        }
        let mut tampered_compilation = filtered.clone();
        tampered_compilation.filtered_compilation.compiled_bytes += 1;
        assert_eq!(
            call(
                &requirements,
                &valid_availability,
                &valid_authorization,
                &tampered_compilation
            ),
            Err(Outer::FilteredSelection(
                crate::remote_prompt::FilteredRemoteSelectionError::FilterEvidence,
            ))
        );
        assert_eq!(tokenizer.remaining().unwrap(), 1);
        assert_eq!(provider.remaining(), 1);
    }

    #[test]
    fn filtered_remote_tokenized_composition_is_exact_single_attempt_and_content_free() {
        use crate::admission::AdmissionError;
        use crate::authorization::{RemoteModelAuthorization, RemoteModelAuthorizationEntry};
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::{
            select_filtered_authorized_available_remote_model_tokenize_invoke_and_admit,
            FilteredAuthorizedAvailableRemoteTokenizedInvocationAdmissionError as Outer,
            TokenizedInvocationAdmissionError as Inner,
        };
        use crate::model::{
            LanguageModelProvider, ModelErrorKind, PrivacyClass, RawModelOutput,
            ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::tokenization::{
            ModelInputTokenizationError, ModelRequestTokenCapacityError,
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome,
            TokenizeAndValidateModelRequestCapacityError as Capacity, MODEL_INPUT_TOKENIZATION_V1,
        };
        use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId, ProtocolVersion};
        use std::sync::Arc;

        for (mode, reverse_registry_order) in (0..7).flat_map(|mode| [(mode, false), (mode, true)])
        {
            let mut f = admission_fixture();
            let filtered = filtered_remote_fixture(PrivacyClass::ApprovedRemote);
            f.context.tokenizer_profile_id = "knowledge-private-sentinel".into();
            let invocation_id = id(
                99_100 + mode * 2 + u128::from(reverse_registry_order),
                ModelInvocationId::new,
            );
            let mut selected_descriptor = f.descriptor.clone();
            selected_descriptor.provider_id = id(99_110, ModelProviderId::new);
            selected_descriptor.model_id = id(99_110, ModelId::new);
            selected_descriptor.privacy_class = PrivacyClass::ApprovedRemote;
            f.response.invocation_id = invocation_id;
            f.response.provider_id = selected_descriptor.provider_id;
            f.response.model_id = selected_descriptor.model_id;
            if mode == 6 {
                f.response.output = RawModelOutput::new("model-output-private-sentinel").unwrap();
            }
            let outcome = if mode == 5 {
                ScriptedOutcome::Error(ModelErrorKind::Unavailable)
            } else {
                ScriptedOutcome::Response(f.response.clone())
            };
            let selected = Arc::new(SentinelProvider {
                inner: ScriptedModelProvider::new(
                    selected_descriptor.clone(),
                    [outcome, ScriptedOutcome::Error(ModelErrorKind::Internal)],
                )
                .unwrap(),
                endpoint: "endpoint-private-sentinel".into(),
                credential: "credential-private-sentinel".into(),
                private_diagnostic: "provider-private-sentinel".into(),
            });
            let mut other_descriptor = selected_descriptor.clone();
            other_descriptor.provider_id = id(99_120, ModelProviderId::new);
            other_descriptor.model_id = id(99_120, ModelId::new);
            let other = Arc::new(
                ScriptedModelProvider::new(
                    other_descriptor.clone(),
                    [ScriptedOutcome::Error(ModelErrorKind::Internal)],
                )
                .unwrap(),
            );
            let mut local_descriptor = selected_descriptor.clone();
            local_descriptor.provider_id = id(99_130, ModelProviderId::new);
            local_descriptor.model_id = id(99_130, ModelId::new);
            local_descriptor.privacy_class = PrivacyClass::LocalOnly;
            let local = Arc::new(
                ScriptedModelProvider::new(
                    local_descriptor,
                    [ScriptedOutcome::Error(ModelErrorKind::Internal)],
                )
                .unwrap(),
            );
            let providers: Vec<Arc<dyn LanguageModelProvider>> = if reverse_registry_order {
                vec![other.clone(), local.clone(), selected.clone()]
            } else {
                vec![selected.clone(), local.clone(), other.clone()]
            };
            let registry = ModelRegistry::try_from_providers(providers).unwrap();
            let availability = ModelAvailabilitySnapshot::new(vec![
                ModelAvailabilityEntry {
                    provider_id: selected_descriptor.provider_id,
                    model_id: selected_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
                ModelAvailabilityEntry {
                    provider_id: other_descriptor.provider_id,
                    model_id: other_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
            ])
            .unwrap();
            let authorization = RemoteModelAuthorization::new(
                filtered.filtered_compilation.replay_anchor.clone(),
                vec![
                    RemoteModelAuthorizationEntry {
                        provider_id: selected_descriptor.provider_id,
                        model_id: selected_descriptor.model_id,
                        privacy_class: selected_descriptor.privacy_class,
                    },
                    RemoteModelAuthorizationEntry {
                        provider_id: other_descriptor.provider_id,
                        model_id: other_descriptor.model_id,
                        privacy_class: other_descriptor.privacy_class,
                    },
                ],
            )
            .unwrap();
            let requirements = remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]);
            let exact = selected_descriptor.capabilities.context_window_tokens
                - requirements.maximum_output_tokens;
            let tokenizer_descriptor = if mode == 1 {
                other_descriptor.clone()
            } else {
                selected_descriptor.clone()
            };
            let token_outcome = match mode {
                2 => ScriptedTokenizationOutcome::Error,
                3 => ScriptedTokenizationOutcome::TokenCount(exact + 1),
                _ => ScriptedTokenizationOutcome::TokenCount(exact),
            };
            let tokenizer = SentinelTokenizer {
                inner: ScriptedModelInputTokenizer::new(tokenizer_descriptor, [token_outcome])
                    .unwrap(),
                private_diagnostic: "tokenizer-private-sentinel".into(),
            };
            let version = if mode == 0 {
                ProtocolVersion::new(2, 0)
            } else {
                MODEL_INPUT_TOKENIZATION_V1
            };
            let result =
                select_filtered_authorized_available_remote_model_tokenize_invoke_and_admit(
                    &registry,
                    invocation_id,
                    &requirements,
                    &availability,
                    &authorization,
                    version,
                    &tokenizer,
                    &filtered,
                    &f.authority,
                    &f.context,
                    &f.citations,
                );
            match mode {
                0 => assert_eq!(
                    result,
                    Err(Outer::TokenizedInvocationAdmission(
                        Inner::TokenizationCapacity(Capacity::Tokenization(
                            ModelInputTokenizationError::UnsupportedVersion
                        ))
                    ))
                ),
                1 => assert_eq!(
                    result,
                    Err(Outer::TokenizedInvocationAdmission(
                        Inner::TokenizationCapacity(Capacity::Tokenization(
                            ModelInputTokenizationError::InvalidDescriptor
                        ))
                    ))
                ),
                2 => assert_eq!(
                    result,
                    Err(Outer::TokenizedInvocationAdmission(
                        Inner::TokenizationCapacity(Capacity::Tokenization(
                            ModelInputTokenizationError::TokenizerFailure
                        ))
                    ))
                ),
                3 => assert_eq!(
                    result,
                    Err(Outer::TokenizedInvocationAdmission(
                        Inner::TokenizationCapacity(Capacity::TokenCapacity(
                            ModelRequestTokenCapacityError::ExactCapacity
                        ))
                    ))
                ),
                4 => {
                    let result = result.clone().unwrap();
                    assert_eq!(result.tokenization_evidence.input_token_count, exact);
                    let request = crate::model::ModelRequest {
                        invocation_id,
                        provider_id: selected_descriptor.provider_id,
                        model_id: selected_descriptor.model_id,
                        contract_version: crate::model::MODEL_INVOCATION_V1,
                        input: filtered.filtered_compilation.model_input.clone(),
                        required_capabilities: requirements.required_capabilities.clone(),
                        maximum_output_tokens: requirements.maximum_output_tokens,
                    };
                    assert_eq!(
                        result.admission,
                        crate::admission::admit_model_output(
                            &selected_descriptor,
                            &request,
                            &f.response,
                            &filtered.filtered_compilation,
                            &f.authority,
                            &f.context,
                            &f.citations
                        )
                        .unwrap()
                    );
                    result
                        .tokenization_evidence
                        .validate_for(
                            &selected_descriptor,
                            &filtered.filtered_compilation.model_input,
                        )
                        .unwrap();
                }
                5 => assert_eq!(
                    result,
                    Err(Outer::TokenizedInvocationAdmission(Inner::Invocation(
                        ModelErrorKind::Unavailable
                    )))
                ),
                _ => assert_eq!(
                    result,
                    Err(Outer::TokenizedInvocationAdmission(Inner::Admission(
                        AdmissionError::MalformedSyntax
                    )))
                ),
            }
            assert_eq!(tokenizer.inner.remaining().unwrap(), usize::from(mode < 2));
            assert_eq!(selected.inner.remaining(), if mode < 4 { 2 } else { 1 });
            assert_eq!(other.remaining(), 1);
            assert_eq!(local.remaining(), 1);
            if let Err(error) = result {
                let diagnostics = format!("{error:?} {error}");
                for sentinel in [
                    "prompt-private-sentinel",
                    "learner-private-sentinel",
                    "knowledge-private-sentinel",
                    "authorization-private-sentinel",
                    "tokenizer-private-sentinel",
                    "provider-private-sentinel",
                    "endpoint-private-sentinel",
                    "credential-private-sentinel",
                    "model-output-private-sentinel",
                ] {
                    assert!(!diagnostics.contains(sentinel), "leaked {sentinel}");
                }
            }
        }
    }
    #[test]
    fn available_local_usage_validated_tokenized_selection_rejects_requirements_and_availability_first(
    ) {
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilityError, ModelAvailabilitySnapshot,
            ModelAvailabilityState,
        };
        use crate::generation::{
            select_available_local_model_tokenize_invoke_validate_reported_usage_and_admit,
            AvailableLocalUsageValidatedTokenizedInvocationAdmissionError,
        };
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::tokenization::{
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome, MODEL_INPUT_TOKENIZATION_V1,
        };
        use nexa_domain::{ModelId, ModelProviderId, ProtocolVersion};
        use std::sync::Arc;

        let f = admission_fixture();
        let provider = Arc::new(
            ScriptedModelProvider::new(
                f.descriptor.clone(),
                [ScriptedOutcome::Response(f.response.clone())],
            )
            .unwrap(),
        );
        let registry =
            ModelRegistry::try_from_providers([provider.clone() as Arc<dyn LanguageModelProvider>])
                .unwrap();
        let tokenizer = ScriptedModelInputTokenizer::new(
            f.descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(1)],
        )
        .unwrap();

        for mutation in 0..9 {
            let mut requirements = local_selection_requirements();
            match mutation {
                0 => requirements.contract_version = ProtocolVersion::new(2, 0),
                1 => requirements.maximum_output_tokens = 0,
                2 => requirements.required_capabilities.structured_output = false,
                3 => requirements.privacy_preference.clear(),
                4 => requirements.privacy_preference = vec![PrivacyClass::ApprovedRemote],
                5 => requirements.privacy_preference = vec![PrivacyClass::RestrictedRemote],
                6 => requirements
                    .privacy_preference
                    .push(PrivacyClass::ApprovedRemote),
                7 => requirements
                    .privacy_preference
                    .push(PrivacyClass::RestrictedRemote),
                _ => requirements
                    .privacy_preference
                    .push(PrivacyClass::LocalOnly),
            }
            assert_eq!(
                select_available_local_model_tokenize_invoke_validate_reported_usage_and_admit(
                    &registry,
                    f.request.invocation_id,
                    &requirements,
                    &ModelAvailabilitySnapshot::new(vec![]).unwrap(),
                    MODEL_INPUT_TOKENIZATION_V1,
                    &tokenizer,
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                ),
                Err(AvailableLocalUsageValidatedTokenizedInvocationAdmissionError::InvalidLocalOnlyRequirements)
            );
        }

        let unknown = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: id(9_991, ModelProviderId::new),
            model_id: id(9_991, ModelId::new),
            state: ModelAvailabilityState::Available,
        }])
        .unwrap();
        let mut unsupported = ModelAvailabilitySnapshot::new(vec![]).unwrap();
        unsupported.contract_version = ProtocolVersion::new(2, 0);
        let duplicate_entry = ModelAvailabilityEntry {
            provider_id: f.descriptor.provider_id,
            model_id: f.descriptor.model_id,
            state: ModelAvailabilityState::Available,
        };
        let duplicate = ModelAvailabilitySnapshot {
            contract_version: crate::availability::MODEL_AVAILABILITY_V1,
            entries: vec![duplicate_entry, duplicate_entry],
        };
        for (snapshot, expected) in [
            (unknown, ModelAvailabilityError::RegistryInconsistency),
            (
                unsupported,
                ModelAvailabilityError::UnsupportedAvailabilityVersion,
            ),
            (duplicate, ModelAvailabilityError::InvalidAvailability),
            (
                ModelAvailabilitySnapshot::new(vec![]).unwrap(),
                ModelAvailabilityError::Selection(
                    crate::selection::ModelSelectionError::NoEligibleModel,
                ),
            ),
        ] {
            assert_eq!(
                select_available_local_model_tokenize_invoke_validate_reported_usage_and_admit(
                    &registry,
                    f.request.invocation_id,
                    &local_selection_requirements(),
                    &snapshot,
                    MODEL_INPUT_TOKENIZATION_V1,
                    &tokenizer,
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                ),
                Err(
                    AvailableLocalUsageValidatedTokenizedInvocationAdmissionError::AvailabilitySelection(
                        expected
                    )
                )
            );
        }
        assert_eq!(tokenizer.remaining().unwrap(), 1);
        assert_eq!(provider.remaining(), 1);

        // An unavailable or missing local model never reaches tokenization or invocation.
        let unavailable = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: f.descriptor.provider_id,
            model_id: f.descriptor.model_id,
            state: ModelAvailabilityState::Unavailable,
        }])
        .unwrap();
        assert_eq!(
            select_available_local_model_tokenize_invoke_validate_reported_usage_and_admit(
                &registry,
                f.request.invocation_id,
                &local_selection_requirements(),
                &unavailable,
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            ),
            Err(
                AvailableLocalUsageValidatedTokenizedInvocationAdmissionError::AvailabilitySelection(
                    ModelAvailabilityError::Selection(
                        crate::selection::ModelSelectionError::NoEligibleModel,
                    ),
                ),
            )
        );
        assert_eq!(tokenizer.remaining().unwrap(), 1);
        assert_eq!(provider.remaining(), 1);

        // Keep otherwise-valid identities type checked in this focused closed-error test.
        let _: ModelId = f.descriptor.model_id;
    }

    #[test]
    fn available_local_usage_validated_tokenized_selection_is_order_independent_and_pre_tokenization(
    ) {
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::select_available_local_model_tokenize_invoke_validate_reported_usage_and_admit;
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::tokenization::{
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome, MODEL_INPUT_TOKENIZATION_V1,
        };
        use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId};
        use std::sync::Arc;

        for reverse_registry in [false, true] {
            for reverse_snapshot in [false, true] {
                let mut f = admission_fixture();
                let invocation_id = id(9_980, ModelInvocationId::new);
                let mut byte_ineligible = f.descriptor.clone();
                byte_ineligible.provider_id = id(9_970, ModelProviderId::new);
                byte_ineligible.model_id = id(9_970, ModelId::new);
                byte_ineligible.capabilities.context_window_tokens = f
                    .compilation
                    .compiled_bytes
                    .checked_add(local_selection_requirements().maximum_output_tokens)
                    .unwrap()
                    - 1;
                let mut selected_descriptor = f.descriptor.clone();
                selected_descriptor.provider_id = id(9_971, ModelProviderId::new);
                selected_descriptor.model_id = id(9_971, ModelId::new);
                let mut remote_descriptor = f.descriptor.clone();
                remote_descriptor.provider_id = id(9_972, ModelProviderId::new);
                remote_descriptor.model_id = id(9_972, ModelId::new);
                remote_descriptor.privacy_class = PrivacyClass::ApprovedRemote;
                f.response.invocation_id = invocation_id;
                f.response.provider_id = selected_descriptor.provider_id;
                f.response.model_id = selected_descriptor.model_id;

                let rejected = Arc::new(
                    ScriptedModelProvider::new(
                        byte_ineligible.clone(),
                        [ScriptedOutcome::Error(
                            crate::model::ModelErrorKind::Internal,
                        )],
                    )
                    .unwrap(),
                );
                let selected = Arc::new(
                    ScriptedModelProvider::new(
                        selected_descriptor.clone(),
                        [ScriptedOutcome::Response(f.response.clone())],
                    )
                    .unwrap(),
                );
                let remote = Arc::new(
                    ScriptedModelProvider::new(
                        remote_descriptor.clone(),
                        [ScriptedOutcome::Error(
                            crate::model::ModelErrorKind::Internal,
                        )],
                    )
                    .unwrap(),
                );
                let providers: Vec<Arc<dyn LanguageModelProvider>> = if reverse_registry {
                    vec![remote.clone(), selected.clone(), rejected.clone()]
                } else {
                    vec![rejected.clone(), selected.clone(), remote.clone()]
                };
                let registry = ModelRegistry::try_from_providers(providers).unwrap();
                let mut entries = vec![
                    ModelAvailabilityEntry {
                        provider_id: byte_ineligible.provider_id,
                        model_id: byte_ineligible.model_id,
                        state: ModelAvailabilityState::Available,
                    },
                    ModelAvailabilityEntry {
                        provider_id: selected_descriptor.provider_id,
                        model_id: selected_descriptor.model_id,
                        state: ModelAvailabilityState::Available,
                    },
                    ModelAvailabilityEntry {
                        provider_id: remote_descriptor.provider_id,
                        model_id: remote_descriptor.model_id,
                        state: ModelAvailabilityState::Available,
                    },
                ];
                if reverse_snapshot {
                    entries.reverse();
                }
                let availability = ModelAvailabilitySnapshot::new(entries).unwrap();
                let tokenizer = ScriptedModelInputTokenizer::new(
                    selected_descriptor.clone(),
                    [ScriptedTokenizationOutcome::TokenCount(7)],
                )
                .unwrap();
                let result =
                    select_available_local_model_tokenize_invoke_validate_reported_usage_and_admit(
                        &registry,
                        invocation_id,
                        &local_selection_requirements(),
                        &availability,
                        MODEL_INPUT_TOKENIZATION_V1,
                        &tokenizer,
                        &f.compilation,
                        &f.authority,
                        &f.context,
                        &f.citations,
                    )
                    .unwrap();
                result
                    .tokenization_evidence
                    .validate_for(&selected_descriptor, &f.compilation.model_input)
                    .unwrap();
                assert_eq!(result.tokenization_evidence.input_token_count, 7);
                assert_eq!(tokenizer.remaining().unwrap(), 0);
                assert_eq!(selected.remaining(), 0);
                assert_eq!(rejected.remaining(), 1);
                assert_eq!(remote.remaining(), 1);
            }
        }
    }

    #[test]
    fn available_local_usage_validated_tokenized_composition_proves_nested_ordering_and_exact_success(
    ) {
        use crate::generation::{
            tokenize_invoke_validate_reported_usage_and_admit_model_output_with_token_capacity,
            AvailableLocalUsageValidatedTokenizedInvocationAdmissionError,
            UsageValidatedTokenizedInvocationAdmissionError,
        };
        use crate::model::{
            LanguageModelProvider, ModelErrorKind, ModelRequest, ModelUsage, RawModelOutput,
            ScriptedModelProvider, ScriptedOutcome, MODEL_INVOCATION_V1,
        };
        use crate::registry::ModelRegistry;
        use crate::tokenization::{
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome, MODEL_INPUT_TOKENIZATION_V1,
        };
        use crate::usage::ModelResponseReportedUsageValidationError;
        use std::sync::Arc;

        for reported in [None, Some(7), Some(6), Some(8)] {
            let mut f = admission_fixture();
            f.response.reported_usage = reported.map(|input_tokens| ModelUsage {
                input_tokens,
                output_tokens: 1,
            });
            let provider = Arc::new(
                ScriptedModelProvider::new(
                    f.descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                provider.clone() as Arc<dyn LanguageModelProvider>
            ])
            .unwrap();
            let tokenizer = ScriptedModelInputTokenizer::new(
                f.descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(7)],
            )
            .unwrap();
            let result = available_local_usage_wrapper(
                &registry,
                f.request.invocation_id,
                &local_selection_requirements(),
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            );
            if matches!(reported, None | Some(7)) {
                let result = result.unwrap();
                assert_eq!(result.admission, f.admit().unwrap());
                assert_eq!(result.tokenization_evidence.input_token_count, 7);
                result
                    .tokenization_evidence
                    .validate_for(&f.descriptor, &f.compilation.model_input)
                    .unwrap();

                let request = ModelRequest {
                    invocation_id: f.request.invocation_id,
                    provider_id: f.descriptor.provider_id,
                    model_id: f.descriptor.model_id,
                    contract_version: MODEL_INVOCATION_V1,
                    input: f.compilation.model_input.clone(),
                    required_capabilities: local_selection_requirements().required_capabilities,
                    maximum_output_tokens: local_selection_requirements().maximum_output_tokens,
                };
                let direct_tokenizer = ScriptedModelInputTokenizer::new(
                    f.descriptor.clone(),
                    [ScriptedTokenizationOutcome::TokenCount(7)],
                )
                .unwrap();
                let direct_provider = ScriptedModelProvider::new(
                    f.descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap();
                let direct = tokenize_invoke_validate_reported_usage_and_admit_model_output_with_token_capacity(
                    MODEL_INPUT_TOKENIZATION_V1,
                    &direct_tokenizer,
                    &direct_provider,
                    &request,
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                )
                .unwrap();
                assert_eq!(result, direct);
                assert_eq!(direct_tokenizer.remaining().unwrap(), 0);
                assert_eq!(direct_provider.remaining(), 0);
            } else {
                assert_eq!(
                    result,
                    Err(AvailableLocalUsageValidatedTokenizedInvocationAdmissionError::
                        UsageValidatedTokenizedInvocationAdmission(
                            UsageValidatedTokenizedInvocationAdmissionError::ReportedUsage(
                                ModelResponseReportedUsageValidationError::InputTokenCountMismatch
                            )
                        ))
                );
            }
            assert_eq!(tokenizer.remaining().unwrap(), 0);
            assert_eq!(provider.remaining(), 0);
        }

        use crate::admission::AdmissionError;
        use crate::tokenization::{
            ModelInputTokenizationError, ModelRequestTokenCapacityError,
            TokenizeAndValidateModelRequestCapacityError as Capacity,
        };
        use nexa_domain::{ModelProviderId, ProtocolVersion};

        // Every admission-preflight class reachable after valid local selection is preserved
        // under the exact ADR-0046 `Preflight` nesting and consumes neither dependency.
        for mutation in 0..5 {
            let mut f = admission_fixture();
            let expected = match mutation {
                0 => {
                    f.compilation.contract_version = ProtocolVersion::new(2, 0);
                    AdmissionError::UnsupportedVersion
                }
                1 => {
                    f.compilation.compiled_bytes += 1;
                    AdmissionError::PromptAssociationReplayMismatch
                }
                2 => {
                    f.authority.permitted_capabilities.clear();
                    AdmissionError::PolicyPedagogySafetyCapability
                }
                3 => {
                    f.context.maximum_tokens = 0;
                    AdmissionError::PlanningEvidenceProvenance
                }
                _ => {
                    f.citations.maximum_citations = 0;
                    AdmissionError::CitationGroundingReference
                }
            };
            let provider = Arc::new(
                ScriptedModelProvider::new(
                    f.descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                provider.clone() as Arc<dyn LanguageModelProvider>
            ])
            .unwrap();
            let tokenizer = ScriptedModelInputTokenizer::new(
                f.descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(7)],
            )
            .unwrap();
            assert_eq!(available_local_usage_wrapper(
                &registry, f.request.invocation_id, &local_selection_requirements(),
                MODEL_INPUT_TOKENIZATION_V1, &tokenizer, &f.compilation, &f.authority,
                &f.context, &f.citations,
            ), Err(AvailableLocalUsageValidatedTokenizedInvocationAdmissionError::
                UsageValidatedTokenizedInvocationAdmission(
                    UsageValidatedTokenizedInvocationAdmissionError::Preflight(expected))));
            assert_eq!(tokenizer.remaining().unwrap(), 1);
            assert_eq!(provider.remaining(), 1);
        }

        // Exact tokenization leaves are distinct cases rather than an accidental repeated range.
        for mutation in 0..5 {
            let f = admission_fixture();
            let mut tokenizer_descriptor = f.descriptor.clone();
            let (version, outcomes, expected, tokenizer_remaining) = match mutation {
                0 => (
                    ProtocolVersion::new(2, 0),
                    vec![ScriptedTokenizationOutcome::TokenCount(7)],
                    Capacity::Tokenization(ModelInputTokenizationError::UnsupportedVersion),
                    1,
                ),
                1 => {
                    tokenizer_descriptor.provider_id = id(88_001, ModelProviderId::new);
                    (
                        MODEL_INPUT_TOKENIZATION_V1,
                        vec![ScriptedTokenizationOutcome::TokenCount(7)],
                        Capacity::Tokenization(ModelInputTokenizationError::InvalidDescriptor),
                        1,
                    )
                }
                2 => (
                    MODEL_INPUT_TOKENIZATION_V1,
                    vec![ScriptedTokenizationOutcome::Error],
                    Capacity::Tokenization(ModelInputTokenizationError::TokenizerFailure),
                    0,
                ),
                3 => (
                    MODEL_INPUT_TOKENIZATION_V1,
                    vec![],
                    Capacity::Tokenization(ModelInputTokenizationError::ScriptExhausted),
                    0,
                ),
                _ => (
                    MODEL_INPUT_TOKENIZATION_V1,
                    vec![ScriptedTokenizationOutcome::TokenCount(0)],
                    Capacity::Tokenization(ModelInputTokenizationError::InvalidEvidence),
                    0,
                ),
            };
            let (registry, provider, other, remote) = registry_with_untouched_sentinels(
                &f,
                [ScriptedOutcome::Response(f.response.clone())],
            );
            let tokenizer =
                ScriptedModelInputTokenizer::new(tokenizer_descriptor, outcomes).unwrap();
            assert_eq!(
                available_local_usage_wrapper(
                    &registry, f.request.invocation_id, &local_selection_requirements(), version,
                    &tokenizer, &f.compilation, &f.authority, &f.context, &f.citations,
                ),
                Err(AvailableLocalUsageValidatedTokenizedInvocationAdmissionError::
                    UsageValidatedTokenizedInvocationAdmission(
                        UsageValidatedTokenizedInvocationAdmissionError::TokenizationCapacity(expected)
                    )),
                "tokenization mutation {mutation}"
            );
            assert_eq!(tokenizer.remaining().unwrap(), tokenizer_remaining);
            assert_eq!(provider.remaining(), 1);
            assert_eq!(other.remaining(), 1);
            assert_eq!(remote.remaining(), 1);
        }

        // Checked exact capacity succeeds at equality, and fails identically for one-token
        // excess and checked-add overflow. Only equality reaches the selected provider.
        for (input_tokens, succeeds) in [
            (
                admission_fixture()
                    .descriptor
                    .capabilities
                    .context_window_tokens
                    - local_selection_requirements().maximum_output_tokens,
                true,
            ),
            (
                admission_fixture()
                    .descriptor
                    .capabilities
                    .context_window_tokens
                    - local_selection_requirements().maximum_output_tokens
                    + 1,
                false,
            ),
            (u32::MAX, false),
        ] {
            let f = admission_fixture();
            let (registry, provider, other, remote) = registry_with_untouched_sentinels(
                &f,
                [ScriptedOutcome::Response(f.response.clone())],
            );
            let tokenizer = ScriptedModelInputTokenizer::new(
                f.descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(input_tokens)],
            )
            .unwrap();
            let result = available_local_usage_wrapper(
                &registry,
                f.request.invocation_id,
                &local_selection_requirements(),
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            );
            if succeeds {
                assert_eq!(
                    result.unwrap().tokenization_evidence.input_token_count,
                    input_tokens
                );
                assert_eq!(provider.remaining(), 0);
            } else {
                assert_eq!(
                    result,
                    Err(AvailableLocalUsageValidatedTokenizedInvocationAdmissionError::
                        UsageValidatedTokenizedInvocationAdmission(
                            UsageValidatedTokenizedInvocationAdmissionError::TokenizationCapacity(
                                Capacity::TokenCapacity(ModelRequestTokenCapacityError::ExactCapacity)
                            )
                        ))
                );
                assert_eq!(provider.remaining(), 1);
            }
            assert_eq!(tokenizer.remaining().unwrap(), 0);
            assert_eq!(other.remaining(), 1);
            assert_eq!(remote.remaining(), 1);
        }

        let f = admission_fixture();
        let provider = Arc::new(
            ScriptedModelProvider::new(
                f.descriptor.clone(),
                [
                    ScriptedOutcome::Error(ModelErrorKind::Unavailable),
                    ScriptedOutcome::Response(f.response.clone()),
                ],
            )
            .unwrap(),
        );
        let registry =
            ModelRegistry::try_from_providers([provider.clone() as Arc<dyn LanguageModelProvider>])
                .unwrap();
        let tokenizer = ScriptedModelInputTokenizer::new(
            f.descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(7)],
        )
        .unwrap();
        assert_eq!(available_local_usage_wrapper(
            &registry, f.request.invocation_id, &local_selection_requirements(), MODEL_INPUT_TOKENIZATION_V1,
            &tokenizer, &f.compilation, &f.authority, &f.context, &f.citations,
        ), Err(AvailableLocalUsageValidatedTokenizedInvocationAdmissionError::UsageValidatedTokenizedInvocationAdmission(
            UsageValidatedTokenizedInvocationAdmissionError::Invocation(ModelErrorKind::Unavailable))));
        assert_eq!(tokenizer.remaining().unwrap(), 0);
        assert_eq!(provider.remaining(), 1);

        // Response validation precedes usage equality and admission; admission sees only a
        // response that is valid under ADR-0045.
        for mutation in 0..4 {
            let mut f = admission_fixture();
            f.response.reported_usage = Some(ModelUsage {
                input_tokens: 6,
                output_tokens: 1,
            });
            let expected = match mutation {
                0 => {
                    f.response.invocation_id = id(88_010, nexa_domain::ModelInvocationId::new);
                    ModelErrorKind::IdentityMismatch
                }
                1 => {
                    f.response.contract_version = ProtocolVersion::new(2, 0);
                    ModelErrorKind::UnsupportedVersion
                }
                2 => {
                    f.response.reported_usage.as_mut().unwrap().output_tokens =
                        local_selection_requirements().maximum_output_tokens + 1;
                    ModelErrorKind::InvalidResponse
                }
                _ => {
                    f.response.output = RawModelOutput::new("not json").unwrap();
                    ModelErrorKind::InvalidResponse
                }
            };
            let provider = Arc::new(CountingProvider::new(&f));
            let (other, remote) = untouched_sentinel_providers(&f);
            let registry = ModelRegistry::try_from_providers([
                provider.clone() as Arc<dyn LanguageModelProvider>,
                other.clone(),
                remote.clone(),
            ])
            .unwrap();
            let tokenizer = ScriptedModelInputTokenizer::new(
                f.descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(7)],
            )
            .unwrap();
            let result = available_local_usage_wrapper(
                &registry,
                f.request.invocation_id,
                &local_selection_requirements(),
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            );
            let expected = if mutation == 3 {
                // Raw structure is ADR-0045-valid and therefore reaches unchanged admission.
                Err(AvailableLocalUsageValidatedTokenizedInvocationAdmissionError::UsageValidatedTokenizedInvocationAdmission(
                    UsageValidatedTokenizedInvocationAdmissionError::ReportedUsage(
                        ModelResponseReportedUsageValidationError::InputTokenCountMismatch)))
            } else {
                Err(AvailableLocalUsageValidatedTokenizedInvocationAdmissionError::UsageValidatedTokenizedInvocationAdmission(
                    UsageValidatedTokenizedInvocationAdmissionError::ReportedUsage(
                        ModelResponseReportedUsageValidationError::Response(expected))))
            };
            assert_eq!(result, expected);
            assert_eq!(tokenizer.remaining().unwrap(), 0);
            assert_eq!(provider.calls(), 1);
            assert_eq!(other.remaining(), 1);
            assert_eq!(remote.remaining(), 1);
        }

        let mut f = admission_fixture();
        f.response.output = RawModelOutput::new("not json").unwrap();
        let (registry, provider, other, remote) =
            registry_with_untouched_sentinels(&f, [ScriptedOutcome::Response(f.response.clone())]);
        let tokenizer = ScriptedModelInputTokenizer::new(
            f.descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(7)],
        )
        .unwrap();
        assert_eq!(available_local_usage_wrapper(
            &registry, f.request.invocation_id, &local_selection_requirements(), MODEL_INPUT_TOKENIZATION_V1,
            &tokenizer, &f.compilation, &f.authority, &f.context, &f.citations,
        ), Err(AvailableLocalUsageValidatedTokenizedInvocationAdmissionError::UsageValidatedTokenizedInvocationAdmission(
            UsageValidatedTokenizedInvocationAdmissionError::Admission(AdmissionError::MalformedSyntax))));
        assert_eq!(tokenizer.remaining().unwrap(), 0);
        assert_eq!(provider.remaining(), 0);
        assert_eq!(other.remaining(), 1);
        assert_eq!(remote.remaining(), 1);
    }

    #[test]
    fn available_local_usage_validated_tokenized_composition_proves_multi_invalid_precedence() {
        use crate::admission::AdmissionError;
        use crate::generation::{
            AvailableLocalUsageValidatedTokenizedInvocationAdmissionError as Outer,
            UsageValidatedTokenizedInvocationAdmissionError as Inner,
        };
        use crate::model::{
            LanguageModelProvider, ModelErrorKind, ModelUsage, PrivacyClass, RawModelOutput,
            ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::selection::ModelSelectionError;
        use crate::tokenization::{
            ModelInputTokenizationError, ScriptedModelInputTokenizer, ScriptedTokenizationOutcome,
            TokenizeAndValidateModelRequestCapacityError as Capacity, MODEL_INPUT_TOKENIZATION_V1,
        };
        use crate::usage::ModelResponseReportedUsageValidationError as Usage;
        use nexa_domain::ProtocolVersion;
        use std::sync::Arc;

        // The outer requirements gate wins over invalid availability, preflight evidence,
        // tokenizer failure, and a real queued provider outcome in the same call.
        let mut f = admission_fixture();
        f.compilation.contract_version = ProtocolVersion::new(2, 0);
        let selected = Arc::new(
            ScriptedModelProvider::new(
                f.descriptor.clone(),
                [ScriptedOutcome::Error(ModelErrorKind::Internal)],
            )
            .unwrap(),
        );
        let registry =
            ModelRegistry::try_from_providers([selected.clone() as Arc<dyn LanguageModelProvider>])
                .unwrap();
        let duplicate = crate::availability::ModelAvailabilityEntry {
            provider_id: f.descriptor.provider_id,
            model_id: f.descriptor.model_id,
            state: crate::availability::ModelAvailabilityState::Available,
        };
        let invalid_availability = crate::availability::ModelAvailabilitySnapshot {
            contract_version: crate::availability::MODEL_AVAILABILITY_V1,
            entries: vec![duplicate, duplicate],
        };
        let tokenizer = ScriptedModelInputTokenizer::new(
            f.descriptor.clone(),
            [ScriptedTokenizationOutcome::Error],
        )
        .unwrap();
        let mut malformed = local_selection_requirements();
        malformed.privacy_preference.clear();
        assert_eq!(
            crate::generation::select_available_local_model_tokenize_invoke_validate_reported_usage_and_admit(
                &registry, f.request.invocation_id, &malformed, &invalid_availability,
                ProtocolVersion::new(2, 0), &tokenizer, &f.compilation, &f.authority,
                &f.context, &f.citations,
            ),
            Err(Outer::InvalidLocalOnlyRequirements)
        );
        assert_eq!(tokenizer.remaining().unwrap(), 1);
        assert_eq!(selected.remaining(), 1);

        // Selection wins even though every downstream input is invalid. The ineligible provider
        // carries a real queued outcome and is part of the failing registry call.
        let mut f = admission_fixture();
        f.descriptor.privacy_class = PrivacyClass::ApprovedRemote;
        f.compilation.contract_version = ProtocolVersion::new(2, 0);
        f.response.contract_version = ProtocolVersion::new(2, 0);
        f.response.output = RawModelOutput::new("not json").unwrap();
        let provider = Arc::new(
            ScriptedModelProvider::new(
                f.descriptor.clone(),
                [ScriptedOutcome::Error(ModelErrorKind::Internal)],
            )
            .unwrap(),
        );
        let registry =
            ModelRegistry::try_from_providers([provider.clone() as Arc<dyn LanguageModelProvider>])
                .unwrap();
        let tokenizer = ScriptedModelInputTokenizer::new(
            f.descriptor.clone(),
            [ScriptedTokenizationOutcome::Error],
        )
        .unwrap();
        assert_eq!(
            available_local_usage_wrapper(
                &registry,
                f.request.invocation_id,
                &local_selection_requirements(),
                ProtocolVersion::new(2, 0),
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            ),
            Err(Outer::AvailabilitySelection(
                crate::availability::ModelAvailabilityError::Selection(
                    ModelSelectionError::NoEligibleModel,
                ),
            ))
        );
        assert_eq!(tokenizer.remaining().unwrap(), 1);
        assert_eq!(provider.remaining(), 1);

        // Each later row deliberately keeps every downstream stage invalid. Exact consumption
        // proves the first failing stage wins and neither sentinel provider can be attempted.
        for stage in [0, 1, 2, 4] {
            let mut f = admission_fixture();
            f.response.reported_usage = Some(ModelUsage {
                input_tokens: 6,
                output_tokens: 1,
            });
            f.response.output = RawModelOutput::new("not json").unwrap();
            let mut version = MODEL_INPUT_TOKENIZATION_V1;
            let mut token_outcome = ScriptedTokenizationOutcome::TokenCount(7);
            let mut selected_outcome = ScriptedOutcome::Error(ModelErrorKind::Unavailable);
            let expected = match stage {
                0 => {
                    f.compilation.contract_version = ProtocolVersion::new(2, 0);
                    version = ProtocolVersion::new(2, 0);
                    token_outcome = ScriptedTokenizationOutcome::Error;
                    Inner::Preflight(AdmissionError::UnsupportedVersion)
                }
                1 => {
                    token_outcome = ScriptedTokenizationOutcome::Error;
                    Inner::TokenizationCapacity(Capacity::Tokenization(
                        ModelInputTokenizationError::TokenizerFailure,
                    ))
                }
                2 => Inner::Invocation(ModelErrorKind::Unavailable),
                _ => {
                    selected_outcome = ScriptedOutcome::Response(f.response.clone());
                    Inner::ReportedUsage(Usage::InputTokenCountMismatch)
                }
            };
            let (registry, selected, other, remote) =
                registry_with_untouched_sentinels(&f, [selected_outcome]);
            let tokenizer =
                ScriptedModelInputTokenizer::new(f.descriptor.clone(), [token_outcome]).unwrap();
            assert_eq!(
                available_local_usage_wrapper(
                    &registry,
                    f.request.invocation_id,
                    &local_selection_requirements(),
                    version,
                    &tokenizer,
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                ),
                Err(Outer::UsageValidatedTokenizedInvocationAdmission(expected)),
                "precedence stage {stage}"
            );
            let dependencies_reached = stage >= 1;
            let provider_reached = stage >= 2;
            assert_eq!(
                tokenizer.remaining().unwrap(),
                usize::from(!dependencies_reached)
            );
            assert_eq!(selected.remaining(), usize::from(!provider_reached));
            assert_eq!(other.remaining(), 1);
            assert_eq!(remote.remaining(), 1);
        }

        // Reported-response validation wins over both reported-usage equality and admission.
        // This provider intentionally returns the invalid response without validating it during
        // invocation, so the exact error proves that the ADR-0045 reported-response layer ran.
        let mut f = admission_fixture();
        f.response.invocation_id = id(88_301, nexa_domain::ModelInvocationId::new);
        f.response.reported_usage = Some(ModelUsage {
            input_tokens: 6,
            output_tokens: 1,
        });
        f.response.output = RawModelOutput::new("not json").unwrap();
        let selected = Arc::new(UncheckedScriptedProvider::new(
            f.descriptor.clone(),
            [ScriptedOutcome::Response(f.response.clone())],
        ));
        let (other, remote) = untouched_sentinel_providers(&f);
        let registry = ModelRegistry::try_from_providers([
            selected.clone() as Arc<dyn LanguageModelProvider>,
            other.clone(),
            remote.clone(),
        ])
        .unwrap();
        let tokenizer = ScriptedModelInputTokenizer::new(
            f.descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(7)],
        )
        .unwrap();
        assert_eq!(
            available_local_usage_wrapper(
                &registry,
                f.request.invocation_id,
                &local_selection_requirements(),
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            ),
            Err(Outer::UsageValidatedTokenizedInvocationAdmission(
                Inner::ReportedUsage(Usage::Response(ModelErrorKind::IdentityMismatch))
            ))
        );
        assert_eq!(tokenizer.remaining().unwrap(), 0);
        assert_eq!(selected.remaining(), 0);
        assert_eq!(other.remaining(), 1);
        assert_eq!(remote.remaining(), 1);
    }

    #[test]
    fn available_local_usage_validated_tokenized_keeps_every_non_selected_class_untouched() {
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::{
            AvailableLocalUsageValidatedTokenizedInvocationAdmissionError as Outer,
            UsageValidatedTokenizedInvocationAdmissionError as Inner,
        };
        use crate::model::{
            LanguageModelProvider, ModelErrorKind, ModelUsage, PrivacyClass, RawModelOutput,
            ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::tokenization::{
            ModelInputTokenizationError, ScriptedModelInputTokenizer, ScriptedTokenizationOutcome,
            TokenizeAndValidateModelRequestCapacityError as Capacity, MODEL_INPUT_TOKENIZATION_V1,
        };
        use crate::usage::ModelResponseReportedUsageValidationError as Usage;
        use nexa_domain::{ModelId, ModelProviderId, ProtocolVersion};
        use std::sync::Arc;

        // Success plus each post-selection failure family uses a fresh five-state inventory.
        for case in 0..7 {
            let mut f = admission_fixture();
            let mut available_descriptor = f.descriptor.clone();
            available_descriptor.provider_id = id(90_010, ModelProviderId::new);
            available_descriptor.model_id = id(90_010, ModelId::new);
            let mut unavailable_descriptor = f.descriptor.clone();
            unavailable_descriptor.provider_id = id(90_020, ModelProviderId::new);
            unavailable_descriptor.model_id = id(90_020, ModelId::new);
            let mut omitted_descriptor = f.descriptor.clone();
            omitted_descriptor.provider_id = id(90_030, ModelProviderId::new);
            omitted_descriptor.model_id = id(90_030, ModelId::new);
            let mut remote_descriptor = f.descriptor.clone();
            remote_descriptor.provider_id = id(90_040, ModelProviderId::new);
            remote_descriptor.model_id = id(90_040, ModelId::new);
            remote_descriptor.privacy_class = PrivacyClass::ApprovedRemote;

            let mut token_outcome = ScriptedTokenizationOutcome::TokenCount(7);
            let mut selected_outcome = ScriptedOutcome::Response(f.response.clone());
            let expected = match case {
                0 => None,
                1 => {
                    f.compilation.contract_version = ProtocolVersion::new(2, 0);
                    Some(Inner::Preflight(
                        crate::admission::AdmissionError::UnsupportedVersion,
                    ))
                }
                2 => {
                    token_outcome = ScriptedTokenizationOutcome::Error;
                    Some(Inner::TokenizationCapacity(Capacity::Tokenization(
                        ModelInputTokenizationError::TokenizerFailure,
                    )))
                }
                3 => {
                    selected_outcome = ScriptedOutcome::Error(ModelErrorKind::Unavailable);
                    Some(Inner::Invocation(ModelErrorKind::Unavailable))
                }
                4 => {
                    f.response.invocation_id = id(90_050, nexa_domain::ModelInvocationId::new);
                    selected_outcome = ScriptedOutcome::Response(f.response.clone());
                    Some(Inner::ReportedUsage(Usage::Response(
                        ModelErrorKind::IdentityMismatch,
                    )))
                }
                5 => {
                    f.response.reported_usage = Some(ModelUsage {
                        input_tokens: 6,
                        output_tokens: 1,
                    });
                    selected_outcome = ScriptedOutcome::Response(f.response.clone());
                    Some(Inner::ReportedUsage(Usage::InputTokenCountMismatch))
                }
                _ => {
                    f.response.output = RawModelOutput::new("not json").unwrap();
                    selected_outcome = ScriptedOutcome::Response(f.response.clone());
                    Some(Inner::Admission(
                        crate::admission::AdmissionError::MalformedSyntax,
                    ))
                }
            };
            let selected = Arc::new(UncheckedScriptedProvider::new(
                f.descriptor.clone(),
                [selected_outcome],
            ));
            let available = Arc::new(UncheckedScriptedProvider::new(
                available_descriptor.clone(),
                [ScriptedOutcome::Error(ModelErrorKind::Internal)],
            ));
            let unavailable = Arc::new(UncheckedScriptedProvider::new(
                unavailable_descriptor.clone(),
                [ScriptedOutcome::Error(ModelErrorKind::Internal)],
            ));
            let omitted = Arc::new(UncheckedScriptedProvider::new(
                omitted_descriptor,
                [ScriptedOutcome::Error(ModelErrorKind::Internal)],
            ));
            let remote = Arc::new(UncheckedScriptedProvider::new(
                remote_descriptor.clone(),
                [ScriptedOutcome::Error(ModelErrorKind::Internal)],
            ));
            let registry = ModelRegistry::try_from_providers([
                selected.clone() as Arc<dyn LanguageModelProvider>,
                available.clone(),
                unavailable.clone(),
                omitted.clone(),
                remote.clone(),
            ])
            .unwrap();
            let availability = ModelAvailabilitySnapshot::new(vec![
                ModelAvailabilityEntry {
                    provider_id: f.descriptor.provider_id,
                    model_id: f.descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
                ModelAvailabilityEntry {
                    provider_id: available_descriptor.provider_id,
                    model_id: available_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
                ModelAvailabilityEntry {
                    provider_id: unavailable_descriptor.provider_id,
                    model_id: unavailable_descriptor.model_id,
                    state: ModelAvailabilityState::Unavailable,
                },
                ModelAvailabilityEntry {
                    provider_id: remote_descriptor.provider_id,
                    model_id: remote_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
            ])
            .unwrap();
            let tokenizer =
                ScriptedModelInputTokenizer::new(f.descriptor.clone(), [token_outcome]).unwrap();
            let result = crate::generation::select_available_local_model_tokenize_invoke_validate_reported_usage_and_admit(
                &registry, f.request.invocation_id, &local_selection_requirements(), &availability,
                MODEL_INPUT_TOKENIZATION_V1, &tokenizer, &f.compilation, &f.authority,
                &f.context, &f.citations,
            );
            match expected {
                None => assert!(result.is_ok()),
                Some(error) => assert_eq!(
                    result,
                    Err(Outer::UsageValidatedTokenizedInvocationAdmission(error))
                ),
            }
            assert_eq!(tokenizer.remaining().unwrap(), usize::from(case == 1));
            assert_eq!(selected.remaining(), usize::from(matches!(case, 1 | 2)));
            assert_eq!(available.remaining(), 1);
            assert_eq!(unavailable.remaining(), 1);
            assert_eq!(omitted.remaining(), 1);
            assert_eq!(remote.remaining(), 1);
        }
    }

    #[test]
    fn available_local_usage_validated_tokenized_composition_diagnostics_are_content_free() {
        use crate::generation::{
            AvailableLocalUsageValidatedTokenizedInvocationAdmissionError as Outer,
            UsageValidatedTokenizedInvocationAdmissionError as Inner,
        };
        use crate::model::{
            LanguageModelProvider, ModelErrorKind, ModelUsage, RawModelOutput, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::selection::ModelSelectionError;
        use crate::tokenization::{
            ModelInputTokenizationError, ScriptedModelInputTokenizer, ScriptedTokenizationOutcome,
            TokenizeAndValidateModelRequestCapacityError as Capacity, MODEL_INPUT_TOKENIZATION_V1,
        };
        use crate::usage::ModelResponseReportedUsageValidationError as Usage;
        use nexa_domain::ProtocolVersion;
        use std::sync::Arc;

        struct DiagnosticProvider {
            inner: UncheckedScriptedProvider,
            endpoint: &'static str,
            credential: &'static str,
            private_diagnostic: &'static str,
            usage_adjacent: &'static str,
        }
        impl LanguageModelProvider for DiagnosticProvider {
            fn descriptor(&self) -> &crate::model::ModelDescriptor {
                self.inner.descriptor()
            }
            fn generate(
                &self,
                request: &crate::model::ModelRequest,
            ) -> Result<crate::model::ModelResponse, crate::model::ModelError> {
                assert!(!self.endpoint.is_empty());
                assert!(!self.credential.is_empty());
                assert!(!self.private_diagnostic.is_empty());
                assert!(!self.usage_adjacent.is_empty());
                self.inner.generate(request)
            }
        }

        let sentinels = [
            "prompt-private-sentinel",
            "learner-private-sentinel",
            "knowledge-private-sentinel",
            "response-private-sentinel",
            "usage-adjacent-sentinel",
            "tokenizer-private-sentinel",
            "provider-private-sentinel",
            "endpoint-private-sentinel",
            "credential-private-sentinel",
        ];
        let assert_closed = |error: Outer| {
            for diagnostic in [format!("{error:?}"), format!("{error}")] {
                for sentinel in sentinels {
                    assert!(!diagnostic.contains(sentinel), "leaked {sentinel}");
                }
            }
        };

        let mut base = admission_fixture();
        base.context.tokenizer_profile_id = sentinels[2].into();
        assert!(base.compilation.model_input.as_str().contains(sentinels[0]));
        assert!(base.compilation.model_input.as_str().contains(sentinels[1]));

        // Outer requirement and selection categories are produced by the wrapper itself.
        let provider = Arc::new(DiagnosticProvider {
            inner: UncheckedScriptedProvider::new(
                base.descriptor.clone(),
                [ScriptedOutcome::Response(base.response.clone())],
            ),
            endpoint: sentinels[7],
            credential: sentinels[8],
            private_diagnostic: sentinels[6],
            usage_adjacent: sentinels[4],
        });
        let registry =
            ModelRegistry::try_from_providers([provider.clone() as Arc<dyn LanguageModelProvider>])
                .unwrap();
        let tokenizer = SentinelTokenizer {
            inner: ScriptedModelInputTokenizer::new(
                base.descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(7)],
            )
            .unwrap(),
            private_diagnostic: sentinels[5].into(),
        };
        let mut invalid = local_selection_requirements();
        invalid.privacy_preference.clear();
        let error = available_local_usage_wrapper(
            &registry,
            base.request.invocation_id,
            &invalid,
            MODEL_INPUT_TOKENIZATION_V1,
            &tokenizer,
            &base.compilation,
            &base.authority,
            &base.context,
            &base.citations,
        )
        .unwrap_err();
        assert_eq!(error, Outer::InvalidLocalOnlyRequirements);
        assert_eq!(tokenizer.inner.remaining().unwrap(), 1);
        assert_eq!(provider.inner.remaining(), 1);
        assert_closed(error);

        let mut ineligible_descriptor = base.descriptor.clone();
        ineligible_descriptor.privacy_class = crate::model::PrivacyClass::ApprovedRemote;
        let ineligible = Arc::new(DiagnosticProvider {
            inner: UncheckedScriptedProvider::new(
                ineligible_descriptor,
                [ScriptedOutcome::Response(base.response.clone())],
            ),
            endpoint: sentinels[7],
            credential: sentinels[8],
            private_diagnostic: sentinels[6],
            usage_adjacent: sentinels[4],
        });
        let ineligible_registry = ModelRegistry::try_from_providers([
            ineligible.clone() as Arc<dyn LanguageModelProvider>
        ])
        .unwrap();
        let error = available_local_usage_wrapper(
            &ineligible_registry,
            base.request.invocation_id,
            &local_selection_requirements(),
            MODEL_INPUT_TOKENIZATION_V1,
            &tokenizer,
            &base.compilation,
            &base.authority,
            &base.context,
            &base.citations,
        )
        .unwrap_err();
        assert_eq!(
            error,
            Outer::AvailabilitySelection(crate::availability::ModelAvailabilityError::Selection(
                ModelSelectionError::NoEligibleModel,
            ),)
        );
        assert_eq!(tokenizer.inner.remaining().unwrap(), 1);
        assert_eq!(ineligible.inner.remaining(), 1);
        assert_closed(error);

        // The other availability leaves are also real wrapper failures with the same
        // sentinel-bearing prompt, context, tokenizer, and provider state supplied.
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilityError, ModelAvailabilitySnapshot,
            ModelAvailabilityState, MODEL_AVAILABILITY_V1,
        };
        use nexa_domain::{ModelId, ModelProviderId};
        let entry = ModelAvailabilityEntry {
            provider_id: base.descriptor.provider_id,
            model_id: base.descriptor.model_id,
            state: ModelAvailabilityState::Available,
        };
        let mut unsupported = ModelAvailabilitySnapshot::new(vec![entry]).unwrap();
        unsupported.contract_version = ProtocolVersion::new(2, 0);
        let cases = [
            (
                unsupported,
                ModelAvailabilityError::UnsupportedAvailabilityVersion,
            ),
            (
                ModelAvailabilitySnapshot {
                    contract_version: MODEL_AVAILABILITY_V1,
                    entries: vec![entry, entry],
                },
                ModelAvailabilityError::InvalidAvailability,
            ),
            (
                ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
                    provider_id: id(91_001, ModelProviderId::new),
                    model_id: id(91_001, ModelId::new),
                    state: ModelAvailabilityState::Available,
                }])
                .unwrap(),
                ModelAvailabilityError::RegistryInconsistency,
            ),
        ];
        for (availability, expected) in cases {
            let error = crate::generation::select_available_local_model_tokenize_invoke_validate_reported_usage_and_admit(
                &registry, base.request.invocation_id, &local_selection_requirements(),
                &availability, MODEL_INPUT_TOKENIZATION_V1, &tokenizer, &base.compilation,
                &base.authority, &base.context, &base.citations,
            ).unwrap_err();
            assert_eq!(error, Outer::AvailabilitySelection(expected));
            assert_eq!(tokenizer.inner.remaining().unwrap(), 1);
            assert_eq!(provider.inner.remaining(), 1);
            assert_closed(error);
        }

        // Every nested category below is reached through a fresh wrapper call carrying all
        // sentinel-bearing prompt/context/tokenizer/provider/endpoint/credential state.
        for mutation in 0..7 {
            let mut f = admission_fixture();
            f.context.tokenizer_profile_id = sentinels[2].into();
            f.response.output =
                RawModelOutput::new(format!("{} {} not json", sentinels[3], sentinels[4])).unwrap();
            let (version, token_outcome, provider_outcome, expected) = match mutation {
                0 => {
                    f.compilation.contract_version = ProtocolVersion::new(2, 0);
                    (
                        MODEL_INPUT_TOKENIZATION_V1,
                        ScriptedTokenizationOutcome::TokenCount(7),
                        ScriptedOutcome::Response(f.response.clone()),
                        Inner::Preflight(crate::admission::AdmissionError::UnsupportedVersion),
                    )
                }
                1 => (
                    ProtocolVersion::new(2, 0),
                    ScriptedTokenizationOutcome::TokenCount(7),
                    ScriptedOutcome::Response(f.response.clone()),
                    Inner::TokenizationCapacity(Capacity::Tokenization(
                        ModelInputTokenizationError::UnsupportedVersion,
                    )),
                ),
                2 => (
                    MODEL_INPUT_TOKENIZATION_V1,
                    ScriptedTokenizationOutcome::Error,
                    ScriptedOutcome::Response(f.response.clone()),
                    Inner::TokenizationCapacity(Capacity::Tokenization(
                        ModelInputTokenizationError::TokenizerFailure,
                    )),
                ),
                3 => (
                    MODEL_INPUT_TOKENIZATION_V1,
                    ScriptedTokenizationOutcome::TokenCount(7),
                    ScriptedOutcome::Error(ModelErrorKind::Internal),
                    Inner::Invocation(ModelErrorKind::Internal),
                ),
                4 => {
                    f.response.invocation_id = id(88_100, nexa_domain::ModelInvocationId::new);
                    (
                        MODEL_INPUT_TOKENIZATION_V1,
                        ScriptedTokenizationOutcome::TokenCount(7),
                        ScriptedOutcome::Response(f.response.clone()),
                        Inner::ReportedUsage(Usage::Response(ModelErrorKind::IdentityMismatch)),
                    )
                }
                5 => {
                    f.response.reported_usage = Some(ModelUsage {
                        input_tokens: 6,
                        output_tokens: 1,
                    });
                    (
                        MODEL_INPUT_TOKENIZATION_V1,
                        ScriptedTokenizationOutcome::TokenCount(7),
                        ScriptedOutcome::Response(f.response.clone()),
                        Inner::ReportedUsage(Usage::InputTokenCountMismatch),
                    )
                }
                _ => (
                    MODEL_INPUT_TOKENIZATION_V1,
                    ScriptedTokenizationOutcome::TokenCount(7),
                    ScriptedOutcome::Response(f.response.clone()),
                    Inner::Admission(crate::admission::AdmissionError::MalformedSyntax),
                ),
            };
            let provider = Arc::new(DiagnosticProvider {
                inner: UncheckedScriptedProvider::new(f.descriptor.clone(), [provider_outcome]),
                endpoint: sentinels[7],
                credential: sentinels[8],
                private_diagnostic: sentinels[6],
                usage_adjacent: sentinels[4],
            });
            let registry = ModelRegistry::try_from_providers([
                provider.clone() as Arc<dyn LanguageModelProvider>
            ])
            .unwrap();
            let tokenizer = SentinelTokenizer {
                inner: ScriptedModelInputTokenizer::new(f.descriptor.clone(), [token_outcome])
                    .unwrap(),
                private_diagnostic: sentinels[5].into(),
            };
            let error = available_local_usage_wrapper(
                &registry,
                f.request.invocation_id,
                &local_selection_requirements(),
                version,
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap_err();
            assert_eq!(
                error,
                Outer::UsageValidatedTokenizedInvocationAdmission(expected)
            );
            assert_eq!(
                tokenizer.inner.remaining().unwrap(),
                usize::from(mutation < 2)
            );
            assert_eq!(provider.inner.remaining(), usize::from(mutation < 3));
            assert_closed(error);
        }

        // Complete the remaining reachable leaves with the same real sentinel-bearing wrapper
        // call. These are kept explicit so diagnostics coverage cannot be inferred from a lower
        // level helper's tests.
        for mutation in 0..10 {
            let mut f = admission_fixture();
            f.context.tokenizer_profile_id = sentinels[2].into();
            f.response.output =
                RawModelOutput::new(format!("{} {} not json", sentinels[3], sentinels[4])).unwrap();
            let mut tokenizer_descriptor = f.descriptor.clone();
            let version = MODEL_INPUT_TOKENIZATION_V1;
            let mut outcomes = vec![ScriptedTokenizationOutcome::TokenCount(7)];
            let expected = match mutation {
                0 => {
                    f.compilation.compiled_bytes += 1;
                    Inner::Preflight(
                        crate::admission::AdmissionError::PromptAssociationReplayMismatch,
                    )
                }
                1 => {
                    f.authority.permitted_capabilities.clear();
                    Inner::Preflight(
                        crate::admission::AdmissionError::PolicyPedagogySafetyCapability,
                    )
                }
                2 => {
                    f.context.maximum_tokens = 0;
                    Inner::Preflight(crate::admission::AdmissionError::PlanningEvidenceProvenance)
                }
                3 => {
                    f.citations.maximum_citations = 0;
                    Inner::Preflight(crate::admission::AdmissionError::CitationGroundingReference)
                }
                4 => {
                    tokenizer_descriptor.provider_id =
                        id(88_401, nexa_domain::ModelProviderId::new);
                    Inner::TokenizationCapacity(Capacity::Tokenization(
                        ModelInputTokenizationError::InvalidDescriptor,
                    ))
                }
                5 => {
                    outcomes.clear();
                    Inner::TokenizationCapacity(Capacity::Tokenization(
                        ModelInputTokenizationError::ScriptExhausted,
                    ))
                }
                6 => {
                    outcomes = vec![ScriptedTokenizationOutcome::TokenCount(0)];
                    Inner::TokenizationCapacity(Capacity::Tokenization(
                        ModelInputTokenizationError::InvalidEvidence,
                    ))
                }
                7 => {
                    outcomes = vec![ScriptedTokenizationOutcome::TokenCount(u32::MAX)];
                    Inner::TokenizationCapacity(Capacity::TokenCapacity(
                        crate::tokenization::ModelRequestTokenCapacityError::ExactCapacity,
                    ))
                }
                8 => {
                    f.response.contract_version = ProtocolVersion::new(2, 0);
                    Inner::ReportedUsage(Usage::Response(ModelErrorKind::UnsupportedVersion))
                }
                _ => {
                    f.response.reported_usage = Some(ModelUsage {
                        input_tokens: 7,
                        output_tokens: local_selection_requirements().maximum_output_tokens + 1,
                    });
                    Inner::ReportedUsage(Usage::Response(ModelErrorKind::InvalidResponse))
                }
            };
            let provider = Arc::new(DiagnosticProvider {
                inner: UncheckedScriptedProvider::new(
                    f.descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                ),
                endpoint: sentinels[7],
                credential: sentinels[8],
                private_diagnostic: sentinels[6],
                usage_adjacent: sentinels[4],
            });
            let registry = ModelRegistry::try_from_providers([
                provider.clone() as Arc<dyn LanguageModelProvider>
            ])
            .unwrap();
            let tokenizer = SentinelTokenizer {
                inner: ScriptedModelInputTokenizer::new(tokenizer_descriptor, outcomes).unwrap(),
                private_diagnostic: sentinels[5].into(),
            };
            let error = available_local_usage_wrapper(
                &registry,
                f.request.invocation_id,
                &local_selection_requirements(),
                version,
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap_err();
            assert_eq!(
                error,
                Outer::UsageValidatedTokenizedInvocationAdmission(expected),
                "diagnostic mutation {mutation}"
            );
            assert_eq!(
                tokenizer.inner.remaining().unwrap(),
                usize::from(mutation <= 4)
            );
            assert_eq!(provider.inner.remaining(), usize::from(mutation < 8));
            assert_closed(error);
        }
    }

    #[test]
    fn authorized_remote_usage_validated_tokenized_composition_proves_every_selection_gate() {
        use crate::authorization::{
            RemoteAuthorizationError as SelectionError, RemoteModelAuthorization,
            RemoteModelAuthorizationEntry, REMOTE_AUTHORIZATION_POLICY_V1, REMOTE_AUTHORIZATION_V1,
        };
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilityError, ModelAvailabilitySnapshot,
            ModelAvailabilityState, MODEL_AVAILABILITY_V1,
        };
        use crate::generation::{
            select_authorized_available_remote_model_tokenize_invoke_validate_reported_usage_and_admit as call,
            AuthorizedAvailableRemoteUsageValidatedTokenizedInvocationAdmissionError as Outer,
        };
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::selection::ModelSelectionError;
        use crate::tokenization::{
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome, MODEL_INPUT_TOKENIZATION_V1,
        };
        use nexa_domain::{ModelId, ModelProviderId, ProtocolVersion};
        use std::sync::Arc;

        let mut f = admission_fixture();
        f.descriptor.privacy_class = PrivacyClass::ApprovedRemote;
        f.response.provider_id = f.descriptor.provider_id;
        f.response.model_id = f.descriptor.model_id;
        let provider = Arc::new(
            ScriptedModelProvider::new(
                f.descriptor.clone(),
                [ScriptedOutcome::Response(f.response.clone())],
            )
            .unwrap(),
        );
        let registry =
            ModelRegistry::try_from_providers([provider.clone() as Arc<dyn LanguageModelProvider>])
                .unwrap();
        let entry = RemoteModelAuthorizationEntry {
            provider_id: f.descriptor.provider_id,
            model_id: f.descriptor.model_id,
            privacy_class: f.descriptor.privacy_class,
        };
        let valid_authorization =
            RemoteModelAuthorization::new(f.compilation.replay_anchor.clone(), vec![entry])
                .unwrap();
        let valid_availability = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: entry.provider_id,
            model_id: entry.model_id,
            state: ModelAvailabilityState::Available,
        }])
        .unwrap();
        let tokenizer = ScriptedModelInputTokenizer::new(
            f.descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(7)],
        )
        .unwrap();
        let requirements = remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]);

        let mut cases = Vec::new();
        let mut invalid_requirements = requirements.clone();
        invalid_requirements.maximum_output_tokens = 0;
        cases.push((
            invalid_requirements,
            valid_availability.clone(),
            valid_authorization.clone(),
            f.compilation.clone(),
            SelectionError::InvalidRemoteRequirements,
        ));
        let mut unsupported_contract = valid_authorization.clone();
        unsupported_contract.contract_version = ProtocolVersion::new(2, 0);
        cases.push((
            requirements.clone(),
            valid_availability.clone(),
            unsupported_contract,
            f.compilation.clone(),
            SelectionError::UnsupportedAuthorizationVersion,
        ));
        let mut unsupported_policy = valid_authorization.clone();
        unsupported_policy.policy_version = ProtocolVersion::new(2, 0);
        cases.push((
            requirements.clone(),
            valid_availability.clone(),
            unsupported_policy,
            f.compilation.clone(),
            SelectionError::UnsupportedAuthorizationVersion,
        ));
        let mut invalid_authorization = valid_authorization.clone();
        invalid_authorization.prompt_compilation_replay_anchor =
            "authorization-private-sentinel".into();
        cases.push((
            requirements.clone(),
            valid_availability.clone(),
            invalid_authorization,
            f.compilation.clone(),
            SelectionError::InvalidAuthorizationEvidence,
        ));
        let mut invalid_compilation = f.compilation.clone();
        invalid_compilation.compiled_bytes += 1;
        cases.push((
            requirements.clone(),
            valid_availability.clone(),
            valid_authorization.clone(),
            invalid_compilation,
            SelectionError::PromptCompilationAssociation,
        ));
        let mismatched_authorization =
            RemoteModelAuthorization::new("a".repeat(64), vec![entry]).unwrap();
        cases.push((
            requirements.clone(),
            valid_availability.clone(),
            mismatched_authorization,
            f.compilation.clone(),
            SelectionError::PromptCompilationAssociation,
        ));
        let inconsistent = RemoteModelAuthorization::new(
            f.compilation.replay_anchor.clone(),
            vec![RemoteModelAuthorizationEntry {
                provider_id: id(90_001, ModelProviderId::new),
                model_id: id(90_001, ModelId::new),
                privacy_class: PrivacyClass::RestrictedRemote,
            }],
        )
        .unwrap();
        cases.push((
            requirements.clone(),
            valid_availability.clone(),
            inconsistent,
            f.compilation.clone(),
            SelectionError::AuthorizationRegistryInconsistency,
        ));
        let mut unsupported_availability = valid_availability.clone();
        unsupported_availability.contract_version = ProtocolVersion::new(2, 0);
        cases.push((
            requirements.clone(),
            unsupported_availability,
            valid_authorization.clone(),
            f.compilation.clone(),
            SelectionError::AvailabilitySelection(
                ModelAvailabilityError::UnsupportedAvailabilityVersion,
            ),
        ));
        let invalid_availability = ModelAvailabilitySnapshot {
            contract_version: MODEL_AVAILABILITY_V1,
            entries: vec![valid_availability.entries[0], valid_availability.entries[0]],
        };
        cases.push((
            requirements.clone(),
            invalid_availability,
            valid_authorization.clone(),
            f.compilation.clone(),
            SelectionError::AvailabilitySelection(ModelAvailabilityError::InvalidAvailability),
        ));
        let inconsistent_availability =
            ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
                provider_id: id(90_002, ModelProviderId::new),
                model_id: id(90_002, ModelId::new),
                state: ModelAvailabilityState::Available,
            }])
            .unwrap();
        cases.push((
            requirements.clone(),
            inconsistent_availability,
            valid_authorization.clone(),
            f.compilation.clone(),
            SelectionError::AvailabilitySelection(ModelAvailabilityError::RegistryInconsistency),
        ));

        for (requirements, availability, authorization, compilation, expected) in cases {
            let error = call(
                &registry,
                f.request.invocation_id,
                &requirements,
                &availability,
                &authorization,
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap_err();
            assert_eq!(error, Outer::AuthorizationAvailabilitySelection(expected));
            assert_eq!(tokenizer.remaining().unwrap(), 1);
            assert_eq!(provider.remaining(), 1);
            let diagnostic = format!("{error:?} {error}");
            for sentinel in [
                "authorization-private-sentinel",
                f.compilation.model_input.as_str(),
            ] {
                assert!(!diagnostic.contains(sentinel));
            }
        }

        // Authorization, availability, and static eligibility are independent intersections.
        for mode in 0..7 {
            let mut descriptor = f.descriptor.clone();
            let mut requirements = requirements.clone();
            let authorization = if mode == 0 {
                RemoteModelAuthorization::new(f.compilation.replay_anchor.clone(), vec![]).unwrap()
            } else {
                valid_authorization.clone()
            };
            let availability = match mode {
                1 => ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
                    state: ModelAvailabilityState::Unavailable,
                    ..valid_availability.entries[0]
                }])
                .unwrap(),
                2 => ModelAvailabilitySnapshot::new(vec![]).unwrap(),
                _ => valid_availability.clone(),
            };
            match mode {
                3 => {
                    descriptor.capabilities.context_window_tokens =
                        f.compilation.compiled_bytes + requirements.maximum_output_tokens - 1
                }
                4 => descriptor.capabilities.structured_output = false,
                5 => {
                    descriptor.capabilities.maximum_output_tokens =
                        requirements.maximum_output_tokens - 1
                }
                6 => requirements.privacy_preference = vec![PrivacyClass::RestrictedRemote],
                _ => {}
            }
            let candidate = Arc::new(
                ScriptedModelProvider::new(
                    descriptor,
                    [ScriptedOutcome::Error(
                        crate::model::ModelErrorKind::Internal,
                    )],
                )
                .unwrap(),
            );
            let candidate_registry = ModelRegistry::try_from_providers([
                candidate.clone() as Arc<dyn LanguageModelProvider>
            ])
            .unwrap();
            assert_eq!(
                call(
                    &candidate_registry,
                    f.request.invocation_id,
                    &requirements,
                    &availability,
                    &authorization,
                    MODEL_INPUT_TOKENIZATION_V1,
                    &tokenizer,
                    &f.compilation,
                    &f.authority,
                    &f.context,
                    &f.citations
                ),
                Err(Outer::AuthorizationAvailabilitySelection(
                    SelectionError::Selection(ModelSelectionError::NoEligibleModel)
                ))
            );
            assert_eq!(candidate.remaining(), 1);
            assert_eq!(tokenizer.remaining().unwrap(), 1);
        }
        let _ = (
            REMOTE_AUTHORIZATION_V1,
            REMOTE_AUTHORIZATION_POLICY_V1,
            ModelId::new,
        );
    }

    #[test]
    fn authorized_remote_usage_validated_tokenized_composition_preserves_selection_and_usage() {
        use crate::authorization::{
            RemoteAuthorizationError, RemoteModelAuthorization, RemoteModelAuthorizationEntry,
        };
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::{
            select_authorized_available_remote_model_tokenize_invoke_validate_reported_usage_and_admit,
            tokenize_invoke_validate_reported_usage_and_admit_model_output_with_token_capacity,
            AuthorizedAvailableRemoteUsageValidatedTokenizedInvocationAdmissionError as Outer,
            UsageValidatedTokenizedInvocationAdmissionError as Inner,
        };
        use crate::model::{
            LanguageModelProvider, ModelRequest, ModelUsage, PrivacyClass, ScriptedModelProvider,
            ScriptedOutcome, MODEL_INVOCATION_V1,
        };
        use crate::registry::ModelRegistry;
        use crate::tokenization::{
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome, MODEL_INPUT_TOKENIZATION_V1,
        };
        use crate::usage::ModelResponseReportedUsageValidationError;
        use nexa_domain::ProtocolVersion;
        use std::sync::Arc;

        for reported in [None, Some(7), Some(6), Some(8)] {
            let mut f = admission_fixture();
            let mut descriptor = f.descriptor.clone();
            descriptor.privacy_class = PrivacyClass::ApprovedRemote;
            f.response.provider_id = descriptor.provider_id;
            f.response.model_id = descriptor.model_id;
            f.response.reported_usage = reported.map(|input_tokens| ModelUsage {
                input_tokens,
                output_tokens: 1,
            });
            let provider = Arc::new(
                ScriptedModelProvider::new(
                    descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                provider.clone() as Arc<dyn LanguageModelProvider>
            ])
            .unwrap();
            let availability = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
                provider_id: descriptor.provider_id,
                model_id: descriptor.model_id,
                state: ModelAvailabilityState::Available,
            }])
            .unwrap();
            let authorization = RemoteModelAuthorization::new(
                f.compilation.replay_anchor.clone(),
                vec![RemoteModelAuthorizationEntry {
                    provider_id: descriptor.provider_id,
                    model_id: descriptor.model_id,
                    privacy_class: descriptor.privacy_class,
                }],
            )
            .unwrap();
            let tokenizer = ScriptedModelInputTokenizer::new(
                descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(7)],
            )
            .unwrap();
            let requirements = remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]);
            let result = select_authorized_available_remote_model_tokenize_invoke_validate_reported_usage_and_admit(
                &registry, f.request.invocation_id,
                &requirements,
                &availability, &authorization, MODEL_INPUT_TOKENIZATION_V1, &tokenizer,
                &f.compilation, &f.authority, &f.context, &f.citations,
            );
            if matches!(reported, None | Some(7)) {
                let result = result.unwrap();
                let request = ModelRequest {
                    invocation_id: f.request.invocation_id,
                    provider_id: descriptor.provider_id,
                    model_id: descriptor.model_id,
                    contract_version: MODEL_INVOCATION_V1,
                    input: f.compilation.model_input.clone(),
                    required_capabilities: requirements.required_capabilities.clone(),
                    maximum_output_tokens: requirements.maximum_output_tokens,
                };
                let direct_provider = ScriptedModelProvider::new(
                    descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap();
                let direct_tokenizer = ScriptedModelInputTokenizer::new(
                    descriptor.clone(),
                    [ScriptedTokenizationOutcome::TokenCount(7)],
                )
                .unwrap();
                let direct = tokenize_invoke_validate_reported_usage_and_admit_model_output_with_token_capacity(
                    MODEL_INPUT_TOKENIZATION_V1, &direct_tokenizer, &direct_provider, &request,
                    &f.compilation, &f.authority, &f.context, &f.citations,
                ).unwrap();
                assert_eq!(result, direct);
                assert_eq!(result.admission, f.admit().unwrap());
                assert_eq!(result.tokenization_evidence.input_token_count, 7);
                result
                    .tokenization_evidence
                    .validate_for(&descriptor, &f.compilation.model_input)
                    .unwrap();
                assert_eq!(direct_tokenizer.remaining().unwrap(), 0);
                assert_eq!(direct_provider.remaining(), 0);
            } else {
                assert_eq!(
                    result,
                    Err(Outer::UsageValidatedTokenizedInvocationAdmission(
                        Inner::ReportedUsage(
                            ModelResponseReportedUsageValidationError::InputTokenCountMismatch
                        )
                    ))
                );
            }
            assert_eq!(tokenizer.remaining().unwrap(), 0);
            assert_eq!(provider.remaining(), 0);
        }

        let f = admission_fixture();
        let registry =
            ModelRegistry::try_from_providers(Vec::<Arc<dyn LanguageModelProvider>>::new())
                .unwrap();
        let availability = ModelAvailabilitySnapshot::new(vec![]).unwrap();
        let authorization =
            RemoteModelAuthorization::new(f.compilation.replay_anchor.clone(), vec![]).unwrap();
        let tokenizer = ScriptedModelInputTokenizer::new(
            f.descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(7)],
        )
        .unwrap();
        let mut invalid = remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]);
        invalid.contract_version = ProtocolVersion::new(2, 0);
        let error = select_authorized_available_remote_model_tokenize_invoke_validate_reported_usage_and_admit(
            &registry, f.request.invocation_id, &invalid, &availability, &authorization,
            MODEL_INPUT_TOKENIZATION_V1, &tokenizer, &f.compilation, &f.authority, &f.context,
            &f.citations,
        ).unwrap_err();
        assert_eq!(
            error,
            Outer::AuthorizationAvailabilitySelection(
                RemoteAuthorizationError::InvalidRemoteRequirements
            )
        );
        assert_eq!(tokenizer.remaining().unwrap(), 1);
        for diagnostic in [format!("{error:?}"), format!("{error}")] {
            assert!(!diagnostic.contains(f.compilation.model_input.as_str()));
        }
    }

    #[test]
    fn authorized_remote_usage_validated_tokenized_selection_is_canonical_and_disjoint() {
        use crate::authorization::{RemoteModelAuthorization, RemoteModelAuthorizationEntry};
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::select_authorized_available_remote_model_tokenize_invoke_validate_reported_usage_and_admit;
        use crate::model::{
            LanguageModelProvider, ModelErrorKind, PrivacyClass, ScriptedModelProvider,
            ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::tokenization::{
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome, MODEL_INPUT_TOKENIZATION_V1,
        };
        use nexa_domain::{ModelId, ModelProviderId};
        use std::sync::Arc;

        for preferred in [PrivacyClass::ApprovedRemote, PrivacyClass::RestrictedRemote] {
            for reverse in [false, true] {
                let mut f = admission_fixture();
                let make_descriptor = |number, privacy| {
                    let mut descriptor = f.descriptor.clone();
                    descriptor.provider_id = id(number, ModelProviderId::new);
                    descriptor.model_id = id(number, ModelId::new);
                    descriptor.privacy_class = privacy;
                    descriptor
                };
                let approved_low = make_descriptor(92_001, PrivacyClass::ApprovedRemote);
                let approved_high = make_descriptor(92_002, PrivacyClass::ApprovedRemote);
                let restricted = make_descriptor(92_003, PrivacyClass::RestrictedRemote);
                let unauthorized = make_descriptor(92_004, preferred);
                let unavailable = make_descriptor(92_005, preferred);
                let omitted = make_descriptor(92_006, preferred);
                let unavailable_provider_id = unavailable.provider_id;
                let omitted_provider_id = omitted.provider_id;
                let local = make_descriptor(92_007, PrivacyClass::LocalOnly);
                let mut byte_ineligible = make_descriptor(92_000, preferred);
                let requirements = remote_selection_requirements(vec![
                    preferred,
                    if preferred == PrivacyClass::ApprovedRemote {
                        PrivacyClass::RestrictedRemote
                    } else {
                        PrivacyClass::ApprovedRemote
                    },
                ]);
                byte_ineligible.capabilities.context_window_tokens =
                    f.compilation.compiled_bytes + requirements.maximum_output_tokens - 1;
                let selected_descriptor = if preferred == PrivacyClass::ApprovedRemote {
                    approved_low.clone()
                } else {
                    restricted.clone()
                };
                f.response.provider_id = selected_descriptor.provider_id;
                f.response.model_id = selected_descriptor.model_id;
                let descriptors = [
                    approved_high,
                    restricted,
                    unauthorized,
                    unavailable,
                    omitted,
                    local,
                    byte_ineligible,
                    approved_low,
                ];
                let providers: Vec<_> = descriptors
                    .iter()
                    .map(|descriptor| {
                        Arc::new(
                            ScriptedModelProvider::new(
                                descriptor.clone(),
                                [if *descriptor == selected_descriptor {
                                    ScriptedOutcome::Response(f.response.clone())
                                } else {
                                    ScriptedOutcome::Error(ModelErrorKind::Internal)
                                }],
                            )
                            .unwrap(),
                        )
                    })
                    .collect();
                let mut handles: Vec<Arc<dyn LanguageModelProvider>> = providers
                    .iter()
                    .cloned()
                    .map(|provider| provider as Arc<dyn LanguageModelProvider>)
                    .collect();
                if reverse {
                    handles.reverse();
                }
                let registry = ModelRegistry::try_from_providers(handles).unwrap();
                let authorized = [
                    &descriptors[0],
                    &descriptors[1],
                    &descriptors[3],
                    &descriptors[6],
                    &descriptors[7],
                ];
                let mut authorization_entries: Vec<_> = authorized
                    .iter()
                    .map(|descriptor| RemoteModelAuthorizationEntry {
                        provider_id: descriptor.provider_id,
                        model_id: descriptor.model_id,
                        privacy_class: descriptor.privacy_class,
                    })
                    .collect();
                if reverse {
                    authorization_entries.reverse();
                }
                let authorization = RemoteModelAuthorization::new(
                    f.compilation.replay_anchor.clone(),
                    authorization_entries,
                )
                .unwrap();
                let mut availability_entries: Vec<_> = descriptors
                    .iter()
                    .filter(|descriptor| descriptor.provider_id != omitted_provider_id)
                    .map(|descriptor| ModelAvailabilityEntry {
                        provider_id: descriptor.provider_id,
                        model_id: descriptor.model_id,
                        state: if descriptor.provider_id == unavailable_provider_id {
                            ModelAvailabilityState::Unavailable
                        } else {
                            ModelAvailabilityState::Available
                        },
                    })
                    .collect();
                if reverse {
                    availability_entries.reverse();
                }
                let availability = ModelAvailabilitySnapshot::new(availability_entries).unwrap();
                let tokenizer = ScriptedModelInputTokenizer::new(
                    selected_descriptor.clone(),
                    [ScriptedTokenizationOutcome::TokenCount(7)],
                )
                .unwrap();
                let result = select_authorized_available_remote_model_tokenize_invoke_validate_reported_usage_and_admit(
                    &registry, f.request.invocation_id, &requirements, &availability,
                    &authorization, MODEL_INPUT_TOKENIZATION_V1, &tokenizer, &f.compilation,
                    &f.authority, &f.context, &f.citations,
                ).unwrap();
                result
                    .tokenization_evidence
                    .validate_for(&selected_descriptor, &f.compilation.model_input)
                    .unwrap();
                assert_eq!(tokenizer.remaining().unwrap(), 0);
                for (provider, descriptor) in providers.iter().zip(descriptors.iter()) {
                    assert_eq!(
                        provider.remaining(),
                        usize::from(*descriptor != selected_descriptor)
                    );
                }
            }
        }
    }

    #[test]
    fn authorized_remote_usage_validated_tokenized_composition_proves_nested_order_and_counts() {
        use crate::admission::AdmissionError;
        use crate::generation::{
            AuthorizedAvailableRemoteUsageValidatedTokenizedInvocationAdmissionError as Outer,
            UsageValidatedTokenizedInvocationAdmissionError as Inner,
        };
        use crate::model::{
            LanguageModelProvider, ModelErrorKind, ModelUsage, PrivacyClass, RawModelOutput,
            ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::tokenization::{
            ModelInputTokenizationError, ScriptedModelInputTokenizer, ScriptedTokenizationOutcome,
            TokenizeAndValidateModelRequestCapacityError as Capacity, MODEL_INPUT_TOKENIZATION_V1,
        };
        use crate::usage::ModelResponseReportedUsageValidationError as Usage;
        use nexa_domain::{ModelInvocationId, ModelProviderId, ProtocolVersion};
        use std::sync::Arc;

        // Every case calls only the ADR-0049 wrapper. Earlier failures preserve both queues;
        // tokenization/capacity consumes exactly its tokenizer outcome; invocation and all later
        // stages consume exactly one outcome from each selected dependency.
        for mode in 0..15 {
            let mut f = admission_fixture();
            f.descriptor.privacy_class = PrivacyClass::ApprovedRemote;
            f.response.provider_id = f.descriptor.provider_id;
            f.response.model_id = f.descriptor.model_id;
            // Keep later stages invalid by default so each earlier failure also proves
            // mandatory multi-invalid precedence through usage reconciliation and admission.
            f.response.output = RawModelOutput::new("response-private-sentinel not json").unwrap();
            f.response.reported_usage = Some(ModelUsage {
                input_tokens: 6,
                output_tokens: 1,
            });
            let mut tokenizer_descriptor = f.descriptor.clone();
            let mut version = MODEL_INPUT_TOKENIZATION_V1;
            let mut token_outcomes = vec![ScriptedTokenizationOutcome::TokenCount(7)];
            let mut provider_outcome = ScriptedOutcome::Response(f.response.clone());
            let expected = match mode {
                0 => {
                    f.authority.permitted_capabilities.clear();
                    Inner::Preflight(AdmissionError::PolicyPedagogySafetyCapability)
                }
                1 => {
                    version = ProtocolVersion::new(2, 0);
                    Inner::TokenizationCapacity(Capacity::Tokenization(
                        ModelInputTokenizationError::UnsupportedVersion,
                    ))
                }
                2 => {
                    tokenizer_descriptor.provider_id = id(91_100, ModelProviderId::new);
                    Inner::TokenizationCapacity(Capacity::Tokenization(
                        ModelInputTokenizationError::InvalidDescriptor,
                    ))
                }
                3 => {
                    token_outcomes = vec![ScriptedTokenizationOutcome::Error];
                    Inner::TokenizationCapacity(Capacity::Tokenization(
                        ModelInputTokenizationError::TokenizerFailure,
                    ))
                }
                4 => {
                    token_outcomes = vec![ScriptedTokenizationOutcome::TokenCount(0)];
                    Inner::TokenizationCapacity(Capacity::Tokenization(
                        ModelInputTokenizationError::InvalidEvidence,
                    ))
                }
                5 => {
                    token_outcomes = vec![ScriptedTokenizationOutcome::TokenCount(u32::MAX)];
                    Inner::TokenizationCapacity(Capacity::TokenCapacity(
                        crate::tokenization::ModelRequestTokenCapacityError::ExactCapacity,
                    ))
                }
                6 => {
                    provider_outcome = ScriptedOutcome::Error(ModelErrorKind::Unavailable);
                    Inner::Invocation(ModelErrorKind::Unavailable)
                }
                7 => {
                    f.response.invocation_id = id(91_101, ModelInvocationId::new);
                    provider_outcome = ScriptedOutcome::Response(f.response.clone());
                    Inner::ReportedUsage(Usage::Response(ModelErrorKind::IdentityMismatch))
                }
                8 => {
                    f.response.contract_version = ProtocolVersion::new(2, 0);
                    provider_outcome = ScriptedOutcome::Response(f.response.clone());
                    Inner::ReportedUsage(Usage::Response(ModelErrorKind::UnsupportedVersion))
                }
                9 => {
                    f.response.output =
                        RawModelOutput::new("response-private-sentinel not json").unwrap();
                    f.response.reported_usage = Some(ModelUsage {
                        input_tokens: 7,
                        output_tokens: remote_selection_requirements(vec![
                            PrivacyClass::ApprovedRemote,
                        ])
                        .maximum_output_tokens
                            + 1,
                    });
                    provider_outcome = ScriptedOutcome::Response(f.response.clone());
                    Inner::ReportedUsage(Usage::Response(ModelErrorKind::InvalidResponse))
                }
                10 => {
                    f.response.reported_usage = Some(ModelUsage {
                        input_tokens: 6,
                        output_tokens: 1,
                    });
                    provider_outcome = ScriptedOutcome::Response(f.response.clone());
                    Inner::ReportedUsage(Usage::InputTokenCountMismatch)
                }
                11 => {
                    f.response.reported_usage = Some(ModelUsage {
                        input_tokens: 8,
                        output_tokens: 1,
                    });
                    provider_outcome = ScriptedOutcome::Response(f.response.clone());
                    Inner::ReportedUsage(Usage::InputTokenCountMismatch)
                }
                _ => {
                    f.response.output =
                        RawModelOutput::new("response-private-sentinel not json").unwrap();
                    f.response.reported_usage = None;
                    provider_outcome = ScriptedOutcome::Response(f.response.clone());
                    Inner::Admission(AdmissionError::MalformedSyntax)
                }
            };
            let selected = Arc::new(UncheckedScriptedProvider::new(
                f.descriptor.clone(),
                [
                    provider_outcome,
                    ScriptedOutcome::Error(ModelErrorKind::Internal),
                ],
            ));
            let mut untouched_descriptor = f.descriptor.clone();
            untouched_descriptor.provider_id = id(91_200, ModelProviderId::new);
            let untouched = Arc::new(
                ScriptedModelProvider::new(
                    untouched_descriptor,
                    [ScriptedOutcome::Error(ModelErrorKind::Internal)],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                selected.clone() as Arc<dyn LanguageModelProvider>,
                untouched.clone() as Arc<dyn LanguageModelProvider>,
            ])
            .unwrap();
            let tokenizer =
                ScriptedModelInputTokenizer::new(tokenizer_descriptor, token_outcomes).unwrap();
            let error = authorized_remote_usage_wrapper(
                &registry,
                f.request.invocation_id,
                &remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]),
                version,
                &tokenizer,
                &f.compilation,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap_err();
            assert_eq!(
                error,
                Outer::UsageValidatedTokenizedInvocationAdmission(expected),
                "nested mode {mode}"
            );
            assert_eq!(tokenizer.remaining().unwrap(), usize::from(mode <= 2));
            assert_eq!(selected.remaining(), if mode < 6 { 2 } else { 1 });
            assert_eq!(untouched.remaining(), 1);
            let diagnostics = format!("{error:?} {error}");
            for sentinel in [
                "prompt-private-sentinel",
                "learner-private-sentinel",
                "response-private-sentinel",
                "provider-private-sentinel",
                "endpoint-private-sentinel",
                "credential-private-sentinel",
            ] {
                assert!(!diagnostics.contains(sentinel));
            }
        }
    }

    #[test]
    fn filtered_remote_usage_validated_tokenized_composition_gates_dependencies_at_adr_0034() {
        use crate::authorization::{RemoteModelAuthorization, RemoteModelAuthorizationEntry};
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::{
            select_filtered_authorized_available_remote_model_tokenize_invoke_validate_reported_usage_and_admit,
            FilteredAuthorizedAvailableRemoteUsageValidatedTokenizedInvocationAdmissionError as Outer,
        };
        use crate::model::{
            LanguageModelProvider, ModelErrorKind, PrivacyClass, ScriptedModelProvider,
            ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::remote_prompt::FilteredRemoteSelectionError;
        use crate::tokenization::{
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome, MODEL_INPUT_TOKENIZATION_V1,
        };
        use nexa_domain::ModelInvocationId;
        use std::sync::Arc;

        let f = admission_fixture();
        let mut descriptor = f.descriptor.clone();
        descriptor.privacy_class = PrivacyClass::ApprovedRemote;
        let provider = Arc::new(
            ScriptedModelProvider::new(
                descriptor.clone(),
                [ScriptedOutcome::Error(ModelErrorKind::Internal)],
            )
            .unwrap(),
        );
        let tokenizer = ScriptedModelInputTokenizer::new(
            descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(1)],
        )
        .unwrap();
        let registry =
            ModelRegistry::try_from_providers([provider.clone() as Arc<dyn LanguageModelProvider>])
                .unwrap();
        let availability = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: descriptor.provider_id,
            model_id: descriptor.model_id,
            state: ModelAvailabilityState::Available,
        }])
        .unwrap();
        let mut filtered = filtered_remote_fixture(PrivacyClass::ApprovedRemote);
        let authorization = RemoteModelAuthorization::new(
            filtered.filtered_compilation.replay_anchor.clone(),
            vec![RemoteModelAuthorizationEntry {
                provider_id: descriptor.provider_id,
                model_id: descriptor.model_id,
                privacy_class: descriptor.privacy_class,
            }],
        )
        .unwrap();
        filtered.evidence.filter_replay_anchor = "0".repeat(64);

        assert_eq!(
            select_filtered_authorized_available_remote_model_tokenize_invoke_validate_reported_usage_and_admit(
                &registry,
                id(1700, ModelInvocationId::new),
                &remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]),
                &availability,
                &authorization,
                MODEL_INPUT_TOKENIZATION_V1,
                &tokenizer,
                &filtered,
                &f.authority,
                &f.context,
                &f.citations,
            ),
            Err(Outer::FilteredSelection(
                FilteredRemoteSelectionError::FilterEvidence
            ))
        );
        assert_eq!(tokenizer.remaining().unwrap(), 1);
        assert_eq!(provider.remaining(), 1);
    }

    #[test]
    fn filtered_remote_usage_validated_tokenized_composition_returns_exact_filtered_evidence_and_admission(
    ) {
        use crate::authorization::{RemoteModelAuthorization, RemoteModelAuthorizationEntry};
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::{
            select_filtered_authorized_available_remote_model_tokenize_invoke_validate_reported_usage_and_admit,
            tokenize_invoke_validate_reported_usage_and_admit_model_output_with_token_capacity,
        };
        use crate::model::{
            LanguageModelProvider, ModelRequest, ModelUsage, PrivacyClass, ScriptedModelProvider,
            ScriptedOutcome, MODEL_INVOCATION_V1,
        };
        use crate::registry::ModelRegistry;
        use crate::tokenization::{
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome, MODEL_INPUT_TOKENIZATION_V1,
        };
        use std::sync::Arc;

        for (reverse, reported_usage) in [false, true]
            .into_iter()
            .flat_map(|reverse| [(reverse, None), (reverse, Some(7))])
        {
            let mut f = admission_fixture();
            let filtered = filtered_remote_fixture(PrivacyClass::ApprovedRemote);
            let mut descriptor = f.descriptor.clone();
            descriptor.privacy_class = PrivacyClass::ApprovedRemote;
            f.response.reported_usage = reported_usage.map(|input_tokens| ModelUsage {
                input_tokens,
                output_tokens: 1,
            });
            let selected = Arc::new(ObservingProvider {
                inner: ScriptedModelProvider::new(
                    descriptor.clone(),
                    [ScriptedOutcome::Response(f.response.clone())],
                )
                .unwrap(),
                observed: std::sync::Mutex::new(Vec::new()),
            });
            let mut other_descriptor = descriptor.clone();
            other_descriptor.provider_id = id(999, nexa_domain::ModelProviderId::new);
            other_descriptor.model_id = id(999, nexa_domain::ModelId::new);
            let other = Arc::new(
                ScriptedModelProvider::new(
                    other_descriptor.clone(),
                    [ScriptedOutcome::Error(
                        crate::model::ModelErrorKind::Internal,
                    )],
                )
                .unwrap(),
            );
            let providers: Vec<Arc<dyn LanguageModelProvider>> = if reverse {
                vec![other.clone(), selected.clone()]
            } else {
                vec![selected.clone(), other.clone()]
            };
            let registry = ModelRegistry::try_from_providers(providers).unwrap();
            let availability = ModelAvailabilitySnapshot::new(vec![
                ModelAvailabilityEntry {
                    provider_id: descriptor.provider_id,
                    model_id: descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
                ModelAvailabilityEntry {
                    provider_id: other_descriptor.provider_id,
                    model_id: other_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
            ])
            .unwrap();
            let authorization = RemoteModelAuthorization::new(
                filtered.filtered_compilation.replay_anchor.clone(),
                vec![
                    RemoteModelAuthorizationEntry {
                        provider_id: descriptor.provider_id,
                        model_id: descriptor.model_id,
                        privacy_class: descriptor.privacy_class,
                    },
                    RemoteModelAuthorizationEntry {
                        provider_id: other_descriptor.provider_id,
                        model_id: other_descriptor.model_id,
                        privacy_class: other_descriptor.privacy_class,
                    },
                ],
            )
            .unwrap();
            let tokenizer = ObservingTokenizer {
                inner: ScriptedModelInputTokenizer::new(
                    descriptor.clone(),
                    [ScriptedTokenizationOutcome::TokenCount(7)],
                )
                .unwrap(),
                observed: std::sync::Mutex::new(Vec::new()),
            };
            let result =
                select_filtered_authorized_available_remote_model_tokenize_invoke_validate_reported_usage_and_admit(
                    &registry,
                    f.request.invocation_id,
                    &remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]),
                    &availability,
                    &authorization,
                    MODEL_INPUT_TOKENIZATION_V1,
                    &tokenizer,
                    &filtered,
                    &f.authority,
                    &f.context,
                    &f.citations,
                )
                .unwrap();
            result
                .tokenization_evidence
                .validate_for(&descriptor, &filtered.filtered_compilation.model_input)
                .unwrap();
            let expected_request = ModelRequest {
                invocation_id: f.request.invocation_id,
                provider_id: descriptor.provider_id,
                model_id: descriptor.model_id,
                contract_version: MODEL_INVOCATION_V1,
                input: filtered.filtered_compilation.model_input.clone(),
                required_capabilities: remote_selection_requirements(vec![
                    PrivacyClass::ApprovedRemote,
                ])
                .required_capabilities,
                maximum_output_tokens: remote_selection_requirements(vec![
                    PrivacyClass::ApprovedRemote,
                ])
                .maximum_output_tokens,
            };
            assert_eq!(
                tokenizer.observed(),
                vec![filtered.filtered_compilation.model_input.clone()]
            );
            assert_eq!(selected.observed(), vec![expected_request.clone()]);
            assert_eq!(f.response.invocation_id, expected_request.invocation_id);
            assert_eq!(f.response.provider_id, expected_request.provider_id);
            assert_eq!(f.response.model_id, expected_request.model_id);
            let direct_provider = ScriptedModelProvider::new(
                descriptor.clone(),
                [ScriptedOutcome::Response(f.response.clone())],
            )
            .unwrap();
            let direct_tokenizer = ScriptedModelInputTokenizer::new(
                descriptor.clone(),
                [ScriptedTokenizationOutcome::TokenCount(7)],
            )
            .unwrap();
            let expected =
                tokenize_invoke_validate_reported_usage_and_admit_model_output_with_token_capacity(
                    MODEL_INPUT_TOKENIZATION_V1,
                    &direct_tokenizer,
                    &direct_provider,
                    &expected_request,
                    &filtered.filtered_compilation,
                    &f.authority,
                    &f.context,
                    &f.citations,
                )
                .unwrap();
            assert_eq!(result, expected);
            assert_eq!(result.tokenization_evidence.input_token_count, 7);
            let filtered_input = filtered.filtered_compilation.model_input.as_str();
            for included in [
                "prompt-private-sentinel",
                "identity-private-sentinel",
                "policy",
                "pedagogy",
                "learner-private-sentinel",
                "input",
                "output",
            ] {
                assert!(filtered_input.contains(included), "missing {included}");
            }
            for omitted in [
                "knowledge-private-sentinel",
                "conversation-private-sentinel",
                "tool-private-sentinel",
            ] {
                assert!(!filtered_input.contains(omitted), "retained {omitted}");
                assert!(!format!("{result:?}").contains(omitted), "leaked {omitted}");
            }
            assert_eq!(direct_tokenizer.remaining().unwrap(), 0);
            assert_eq!(direct_provider.remaining(), 0);
            assert_eq!(tokenizer.remaining(), 0);
            assert_eq!(selected.remaining(), 0);
            assert_eq!(other.remaining(), 1);
        }
    }

    #[test]
    fn filtered_remote_usage_validated_tokenized_composition_denials_preserve_exact_categories_and_dependencies(
    ) {
        use crate::authorization::{
            RemoteAuthorizationError, RemoteModelAuthorization, RemoteModelAuthorizationEntry,
        };
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilityError, ModelAvailabilitySnapshot,
            ModelAvailabilityState,
        };
        use crate::generation::{
            select_filtered_authorized_available_remote_model_tokenize_invoke_validate_reported_usage_and_admit,
            FilteredAuthorizedAvailableRemoteUsageValidatedTokenizedInvocationAdmissionError as Outer,
        };
        use crate::model::{
            LanguageModelProvider, PrivacyClass, ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::selection::ModelSelectionError;
        use crate::tokenization::{
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome, MODEL_INPUT_TOKENIZATION_V1,
        };
        use nexa_domain::{ModelId, ModelProviderId, ProtocolVersion};
        use std::sync::Arc;

        let f = admission_fixture();
        let filtered = filtered_remote_fixture(PrivacyClass::ApprovedRemote);
        let mut descriptor = f.descriptor.clone();
        descriptor.privacy_class = PrivacyClass::ApprovedRemote;
        let provider = Arc::new(
            ScriptedModelProvider::new(
                descriptor.clone(),
                [ScriptedOutcome::Response(f.response.clone())],
            )
            .unwrap(),
        );
        let registry =
            ModelRegistry::try_from_providers([provider.clone() as Arc<dyn LanguageModelProvider>])
                .unwrap();
        let entry = RemoteModelAuthorizationEntry {
            provider_id: descriptor.provider_id,
            model_id: descriptor.model_id,
            privacy_class: descriptor.privacy_class,
        };
        let valid_authorization = RemoteModelAuthorization::new(
            filtered.filtered_compilation.replay_anchor.clone(),
            vec![entry],
        )
        .unwrap();
        let valid_availability = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: descriptor.provider_id,
            model_id: descriptor.model_id,
            state: ModelAvailabilityState::Available,
        }])
        .unwrap();
        let tokenizer = ScriptedModelInputTokenizer::new(
            descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(1)],
        )
        .unwrap();
        let requirements = remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]);
        let call =
            |requirements: &crate::selection::ModelSelectionRequirements,
             availability: &ModelAvailabilitySnapshot,
             authorization: &RemoteModelAuthorization,
             filtered_result: &crate::remote_prompt::RemotePromptFilterResult| {
                select_filtered_authorized_available_remote_model_tokenize_invoke_validate_reported_usage_and_admit(
                    &registry,
                    f.request.invocation_id,
                    requirements,
                    availability,
                    authorization,
                    MODEL_INPUT_TOKENIZATION_V1,
                    &tokenizer,
                    filtered_result,
                    &f.authority,
                    &f.context,
                    &f.citations,
                )
            };

        // Every malformed, tampered, incomplete, duplicate, non-canonical, or reassociated
        // ADR-0033 result is rejected before either dependency is consumed.
        for mutation in 0..16 {
            let mut invalid = filtered.clone();
            match mutation {
                0 => invalid.policy.contract_version = ProtocolVersion::new(2, 0),
                1 => invalid.evidence.contract_version = ProtocolVersion::new(2, 0),
                2 => {
                    invalid.policy.rules.pop();
                }
                3 => invalid.policy.rules.push(invalid.policy.rules[0]),
                4 => invalid.policy.rules.swap(0, 1),
                5 => {
                    invalid.policy.rules[0].disposition =
                        crate::remote_prompt::RemotePromptLayerDisposition::Omit
                }
                6 => invalid.policy.target_privacy_class = PrivacyClass::RestrictedRemote,
                7 => invalid.evidence.target_privacy_class = PrivacyClass::RestrictedRemote,
                8 => {
                    invalid.evidence.source_present_layer_kinds.pop();
                }
                9 => invalid
                    .evidence
                    .source_present_layer_kinds
                    .push(invalid.evidence.source_present_layer_kinds[0]),
                10 => invalid.evidence.source_present_layer_kinds.swap(0, 1),
                11 => {
                    invalid.evidence.included_layer_kinds.pop();
                }
                12 => invalid
                    .evidence
                    .omitted_layer_kinds
                    .push(invalid.evidence.included_layer_kinds[0]),
                13 => invalid.evidence.policy_replay_anchor = "a".repeat(64),
                14 => invalid.evidence.source_compilation_replay_anchor = "b".repeat(64),
                _ => invalid.evidence.filtered_compilation_replay_anchor = "c".repeat(64),
            }
            assert_eq!(
                call(
                    &requirements,
                    &valid_availability,
                    &valid_authorization,
                    &invalid,
                ),
                Err(Outer::FilteredSelection(
                    crate::remote_prompt::FilteredRemoteSelectionError::FilterEvidence,
                )),
                "filter mutation {mutation}",
            );
            let error = Outer::FilteredSelection(
                crate::remote_prompt::FilteredRemoteSelectionError::FilterEvidence,
            );
            assert_content_free_diagnostics(
                &error,
                &[
                    "prompt-private-sentinel",
                    "learner-private-sentinel",
                    "knowledge-private-sentinel",
                    "authorization-private-sentinel",
                    "tokenizer-private-sentinel",
                    "provider-private-sentinel",
                    "endpoint-private-sentinel",
                    "credential-private-sentinel",
                    "response-private-sentinel",
                    "usage-private-sentinel",
                ],
            );
            assert_eq!(tokenizer.remaining().unwrap(), 1);
            assert_eq!(provider.remaining(), 1);
        }

        let mut unsupported_requirements = requirements.clone();
        unsupported_requirements.contract_version = ProtocolVersion::new(2, 0);
        let mut local_only_requirements = requirements.clone();
        local_only_requirements.privacy_preference = vec![PrivacyClass::LocalOnly];
        let mut mixed_requirements = requirements.clone();
        mixed_requirements.privacy_preference =
            vec![PrivacyClass::ApprovedRemote, PrivacyClass::LocalOnly];
        let mut empty_requirements = requirements.clone();
        empty_requirements.privacy_preference.clear();
        let mut duplicate_requirements = requirements.clone();
        duplicate_requirements.privacy_preference =
            vec![PrivacyClass::ApprovedRemote, PrivacyClass::ApprovedRemote];
        let mut unsupported_authorization = valid_authorization.clone();
        unsupported_authorization.contract_version = ProtocolVersion::new(2, 0);
        let mut malformed_authorization = valid_authorization.clone();
        malformed_authorization.prompt_compilation_replay_anchor =
            "authorization-private-sentinel".into();
        let wrong_anchor = RemoteModelAuthorization::new("a".repeat(64), vec![entry]).unwrap();
        let source_compilation_authorization = RemoteModelAuthorization::new(
            filtered.evidence.source_compilation_replay_anchor.clone(),
            vec![entry],
        )
        .unwrap();
        let bad_registry = RemoteModelAuthorization::new(
            filtered.filtered_compilation.replay_anchor.clone(),
            vec![RemoteModelAuthorizationEntry {
                model_id: id(99_001, ModelId::new),
                ..entry
            }],
        )
        .unwrap();
        let privacy_mismatch = RemoteModelAuthorization::new(
            filtered.filtered_compilation.replay_anchor.clone(),
            vec![RemoteModelAuthorizationEntry {
                privacy_class: PrivacyClass::RestrictedRemote,
                ..entry
            }],
        )
        .unwrap();
        let empty_authorization = RemoteModelAuthorization::new(
            filtered.filtered_compilation.replay_anchor.clone(),
            vec![],
        )
        .unwrap();
        let mut unsupported_availability = valid_availability.clone();
        unsupported_availability.contract_version = ProtocolVersion::new(2, 0);
        let duplicate_availability = ModelAvailabilitySnapshot {
            contract_version: crate::availability::MODEL_AVAILABILITY_V1,
            entries: vec![valid_availability.entries[0], valid_availability.entries[0]],
        };
        let unavailable = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            state: ModelAvailabilityState::Unavailable,
            ..valid_availability.entries[0]
        }])
        .unwrap();
        let missing = ModelAvailabilitySnapshot::new(vec![]).unwrap();
        let unknown = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: id(99_002, ModelProviderId::new),
            model_id: id(99_002, ModelId::new),
            state: ModelAvailabilityState::Available,
        }])
        .unwrap();

        for (r, a, auth, expected) in [
            (
                &unsupported_requirements,
                &valid_availability,
                &valid_authorization,
                RemoteAuthorizationError::InvalidRemoteRequirements,
            ),
            (
                &local_only_requirements,
                &valid_availability,
                &valid_authorization,
                RemoteAuthorizationError::InvalidRemoteRequirements,
            ),
            (
                &mixed_requirements,
                &valid_availability,
                &valid_authorization,
                RemoteAuthorizationError::InvalidRemoteRequirements,
            ),
            (
                &empty_requirements,
                &valid_availability,
                &valid_authorization,
                RemoteAuthorizationError::InvalidRemoteRequirements,
            ),
            (
                &duplicate_requirements,
                &valid_availability,
                &valid_authorization,
                RemoteAuthorizationError::InvalidRemoteRequirements,
            ),
            (
                &requirements,
                &valid_availability,
                &unsupported_authorization,
                RemoteAuthorizationError::UnsupportedAuthorizationVersion,
            ),
            (
                &requirements,
                &valid_availability,
                &malformed_authorization,
                RemoteAuthorizationError::InvalidAuthorizationEvidence,
            ),
            (
                &requirements,
                &valid_availability,
                &wrong_anchor,
                RemoteAuthorizationError::PromptCompilationAssociation,
            ),
            (
                &requirements,
                &valid_availability,
                &source_compilation_authorization,
                RemoteAuthorizationError::PromptCompilationAssociation,
            ),
            (
                &requirements,
                &valid_availability,
                &bad_registry,
                RemoteAuthorizationError::AuthorizationRegistryInconsistency,
            ),
            (
                &requirements,
                &valid_availability,
                &privacy_mismatch,
                RemoteAuthorizationError::AuthorizationRegistryInconsistency,
            ),
            (
                &requirements,
                &unsupported_availability,
                &valid_authorization,
                RemoteAuthorizationError::AvailabilitySelection(
                    ModelAvailabilityError::UnsupportedAvailabilityVersion,
                ),
            ),
            (
                &requirements,
                &duplicate_availability,
                &valid_authorization,
                RemoteAuthorizationError::AvailabilitySelection(
                    ModelAvailabilityError::InvalidAvailability,
                ),
            ),
            (
                &requirements,
                &unknown,
                &valid_authorization,
                RemoteAuthorizationError::AvailabilitySelection(
                    ModelAvailabilityError::RegistryInconsistency,
                ),
            ),
            (
                &requirements,
                &unavailable,
                &valid_authorization,
                RemoteAuthorizationError::Selection(ModelSelectionError::NoEligibleModel),
            ),
            (
                &requirements,
                &missing,
                &valid_authorization,
                RemoteAuthorizationError::Selection(ModelSelectionError::NoEligibleModel),
            ),
            (
                &requirements,
                &valid_availability,
                &empty_authorization,
                RemoteAuthorizationError::Selection(ModelSelectionError::NoEligibleModel),
            ),
        ] {
            let error = if expected == RemoteAuthorizationError::InvalidRemoteRequirements {
                Outer::FilteredSelection(
                    crate::remote_prompt::FilteredRemoteSelectionError::FilterPrivacyRequirements,
                )
            } else {
                Outer::FilteredSelection(
                    crate::remote_prompt::FilteredRemoteSelectionError::AuthorizedSelection(
                        expected,
                    ),
                )
            };
            assert_eq!(call(r, a, auth, &filtered), Err(error));
            let diagnostic = format!("{error:?} {error}");
            for sentinel in [
                "prompt-private-sentinel",
                "learner-private-sentinel",
                "knowledge-private-sentinel",
                "authorization-private-sentinel",
                "tokenizer-private-sentinel",
                "provider-private-sentinel",
                "endpoint-private-sentinel",
                "credential-private-sentinel",
                "response-private-sentinel",
                "usage-private-sentinel",
            ] {
                assert!(!diagnostic.contains(sentinel), "leaked {sentinel}");
            }
            assert_eq!(tokenizer.remaining().unwrap(), 1);
            assert_eq!(provider.remaining(), 1);
        }
        let mut tampered_compilation = filtered.clone();
        tampered_compilation.filtered_compilation.compiled_bytes += 1;
        assert_eq!(
            call(
                &requirements,
                &valid_availability,
                &valid_authorization,
                &tampered_compilation
            ),
            Err(Outer::FilteredSelection(
                crate::remote_prompt::FilteredRemoteSelectionError::FilterEvidence,
            ))
        );
        assert_eq!(tokenizer.remaining().unwrap(), 1);
        assert_eq!(provider.remaining(), 1);
    }

    #[test]
    fn filtered_remote_usage_validated_tokenized_selection_is_canonical_byte_gated_and_disjoint() {
        use crate::authorization::{RemoteModelAuthorization, RemoteModelAuthorizationEntry};
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::select_filtered_authorized_available_remote_model_tokenize_invoke_validate_reported_usage_and_admit;
        use crate::model::{
            LanguageModelProvider, ModelErrorKind, PrivacyClass, ScriptedModelProvider,
            ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::tokenization::{
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome, MODEL_INPUT_TOKENIZATION_V1,
        };
        use nexa_domain::{ModelId, ModelProviderId};
        use std::sync::Arc;

        for preferred in [PrivacyClass::ApprovedRemote, PrivacyClass::RestrictedRemote] {
            for reverse in [false, true] {
                let mut f = admission_fixture();
                let filtered = filtered_remote_fixture(preferred);
                let make_descriptor = |number, privacy| {
                    let mut descriptor = f.descriptor.clone();
                    descriptor.provider_id = id(number, ModelProviderId::new);
                    descriptor.model_id = id(number, ModelId::new);
                    descriptor.privacy_class = privacy;
                    descriptor
                };
                let approved_low = make_descriptor(92_001, PrivacyClass::ApprovedRemote);
                let approved_high = make_descriptor(92_002, PrivacyClass::ApprovedRemote);
                let restricted = make_descriptor(92_003, PrivacyClass::RestrictedRemote);
                let unauthorized = make_descriptor(92_004, preferred);
                let unavailable = make_descriptor(92_005, preferred);
                let omitted = make_descriptor(92_006, preferred);
                let unavailable_provider_id = unavailable.provider_id;
                let omitted_provider_id = omitted.provider_id;
                let local = make_descriptor(92_007, PrivacyClass::LocalOnly);
                let mut byte_ineligible = make_descriptor(92_000, preferred);
                let requirements = remote_selection_requirements(vec![preferred]);
                byte_ineligible.capabilities.context_window_tokens =
                    filtered.filtered_compilation.compiled_bytes
                        + requirements.maximum_output_tokens
                        - 1;
                let selected_descriptor = if preferred == PrivacyClass::ApprovedRemote {
                    approved_low.clone()
                } else {
                    restricted.clone()
                };
                f.response.provider_id = selected_descriptor.provider_id;
                f.response.model_id = selected_descriptor.model_id;
                let descriptors = [
                    approved_high,
                    restricted,
                    unauthorized,
                    unavailable,
                    omitted,
                    local,
                    byte_ineligible,
                    approved_low,
                ];
                let providers: Vec<_> = descriptors
                    .iter()
                    .map(|descriptor| {
                        Arc::new(
                            ScriptedModelProvider::new(
                                descriptor.clone(),
                                [if *descriptor == selected_descriptor {
                                    ScriptedOutcome::Response(f.response.clone())
                                } else {
                                    ScriptedOutcome::Error(ModelErrorKind::Internal)
                                }],
                            )
                            .unwrap(),
                        )
                    })
                    .collect();
                let mut handles: Vec<Arc<dyn LanguageModelProvider>> = providers
                    .iter()
                    .cloned()
                    .map(|provider| provider as Arc<dyn LanguageModelProvider>)
                    .collect();
                if reverse {
                    handles.reverse();
                }
                let registry = ModelRegistry::try_from_providers(handles).unwrap();
                let authorized = [
                    &descriptors[0],
                    &descriptors[1],
                    &descriptors[3],
                    &descriptors[6],
                    &descriptors[7],
                ];
                let mut authorization_entries: Vec<_> = authorized
                    .iter()
                    .map(|descriptor| RemoteModelAuthorizationEntry {
                        provider_id: descriptor.provider_id,
                        model_id: descriptor.model_id,
                        privacy_class: descriptor.privacy_class,
                    })
                    .collect();
                if reverse {
                    authorization_entries.reverse();
                }
                let authorization = RemoteModelAuthorization::new(
                    filtered.filtered_compilation.replay_anchor.clone(),
                    authorization_entries,
                )
                .unwrap();
                let mut availability_entries: Vec<_> = descriptors
                    .iter()
                    .filter(|descriptor| descriptor.provider_id != omitted_provider_id)
                    .map(|descriptor| ModelAvailabilityEntry {
                        provider_id: descriptor.provider_id,
                        model_id: descriptor.model_id,
                        state: if descriptor.provider_id == unavailable_provider_id {
                            ModelAvailabilityState::Unavailable
                        } else {
                            ModelAvailabilityState::Available
                        },
                    })
                    .collect();
                if reverse {
                    availability_entries.reverse();
                }
                let availability = ModelAvailabilitySnapshot::new(availability_entries).unwrap();
                let tokenizer = ScriptedModelInputTokenizer::new(
                    selected_descriptor.clone(),
                    [ScriptedTokenizationOutcome::TokenCount(7)],
                )
                .unwrap();
                let result = select_filtered_authorized_available_remote_model_tokenize_invoke_validate_reported_usage_and_admit(
                    &registry, f.request.invocation_id, &requirements, &availability,
                    &authorization, MODEL_INPUT_TOKENIZATION_V1, &tokenizer, &filtered,
                    &f.authority, &f.context, &f.citations,
                ).unwrap();
                result
                    .tokenization_evidence
                    .validate_for(
                        &selected_descriptor,
                        &filtered.filtered_compilation.model_input,
                    )
                    .unwrap();
                assert_eq!(tokenizer.remaining().unwrap(), 0);
                for (provider, descriptor) in providers.iter().zip(descriptors.iter()) {
                    assert_eq!(
                        provider.remaining(),
                        usize::from(*descriptor != selected_descriptor)
                    );
                }
            }
        }
    }

    #[test]
    fn filtered_remote_usage_validated_tokenized_composition_proves_complete_stage_precedence_and_counts(
    ) {
        use crate::admission::AdmissionError;
        use crate::generation::{
            FilteredAuthorizedAvailableRemoteUsageValidatedTokenizedInvocationAdmissionError as Outer,
            UsageValidatedTokenizedInvocationAdmissionError as Inner,
        };
        use crate::model::{
            LanguageModelProvider, ModelErrorKind, ModelUsage, PrivacyClass, RawModelOutput,
            ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::tokenization::{
            ModelInputTokenizationError, ScriptedModelInputTokenizer, ScriptedTokenizationOutcome,
            TokenizeAndValidateModelRequestCapacityError as Capacity, MODEL_INPUT_TOKENIZATION_V1,
        };
        use crate::usage::ModelResponseReportedUsageValidationError as Usage;
        use nexa_domain::{ModelInvocationId, ModelProviderId, ProtocolVersion};
        use std::sync::Arc;

        // Every case calls only the ADR-0050 wrapper. Earlier failures preserve both queues;
        // tokenization/capacity consumes exactly its tokenizer outcome; invocation and all later
        // stages consume exactly one outcome from each selected dependency.
        for mode in 0..13 {
            let mut f = admission_fixture();
            f.descriptor.privacy_class = PrivacyClass::ApprovedRemote;
            f.response.provider_id = f.descriptor.provider_id;
            f.response.model_id = f.descriptor.model_id;
            // Keep later stages invalid by default so each earlier failure also proves
            // mandatory multi-invalid precedence through usage reconciliation and admission.
            f.response.output = RawModelOutput::new("response-private-sentinel not json").unwrap();
            f.response.reported_usage = Some(ModelUsage {
                input_tokens: 6,
                output_tokens: 1,
            });
            let mut tokenizer_descriptor = f.descriptor.clone();
            let mut version = MODEL_INPUT_TOKENIZATION_V1;
            let mut token_outcomes = vec![ScriptedTokenizationOutcome::TokenCount(7)];
            let mut provider_outcome = ScriptedOutcome::Response(f.response.clone());
            let expected = match mode {
                0 => {
                    f.authority.permitted_capabilities.clear();
                    Inner::Preflight(AdmissionError::PolicyPedagogySafetyCapability)
                }
                1 => {
                    version = ProtocolVersion::new(2, 0);
                    Inner::TokenizationCapacity(Capacity::Tokenization(
                        ModelInputTokenizationError::UnsupportedVersion,
                    ))
                }
                2 => {
                    tokenizer_descriptor.provider_id = id(91_100, ModelProviderId::new);
                    Inner::TokenizationCapacity(Capacity::Tokenization(
                        ModelInputTokenizationError::InvalidDescriptor,
                    ))
                }
                3 => {
                    token_outcomes = vec![ScriptedTokenizationOutcome::Error];
                    Inner::TokenizationCapacity(Capacity::Tokenization(
                        ModelInputTokenizationError::TokenizerFailure,
                    ))
                }
                4 => {
                    token_outcomes = vec![ScriptedTokenizationOutcome::TokenCount(0)];
                    Inner::TokenizationCapacity(Capacity::Tokenization(
                        ModelInputTokenizationError::InvalidEvidence,
                    ))
                }
                5 => {
                    token_outcomes = vec![ScriptedTokenizationOutcome::TokenCount(u32::MAX)];
                    Inner::TokenizationCapacity(Capacity::TokenCapacity(
                        crate::tokenization::ModelRequestTokenCapacityError::ExactCapacity,
                    ))
                }
                6 => {
                    provider_outcome = ScriptedOutcome::Error(ModelErrorKind::Unavailable);
                    Inner::Invocation(ModelErrorKind::Unavailable)
                }
                7 => {
                    f.response.invocation_id = id(91_101, ModelInvocationId::new);
                    provider_outcome = ScriptedOutcome::Response(f.response.clone());
                    Inner::ReportedUsage(Usage::Response(ModelErrorKind::IdentityMismatch))
                }
                8 => {
                    f.response.contract_version = ProtocolVersion::new(2, 0);
                    provider_outcome = ScriptedOutcome::Response(f.response.clone());
                    Inner::ReportedUsage(Usage::Response(ModelErrorKind::UnsupportedVersion))
                }
                9 => {
                    // Restore valid admission syntax: excessive reported output usage is the
                    // only invalid response field in this case.
                    f.response.output = admission_fixture().response.output;
                    f.response.reported_usage = Some(ModelUsage {
                        input_tokens: 7,
                        output_tokens: remote_selection_requirements(vec![
                            PrivacyClass::ApprovedRemote,
                        ])
                        .maximum_output_tokens
                            + 1,
                    });
                    provider_outcome = ScriptedOutcome::Response(f.response.clone());
                    Inner::ReportedUsage(Usage::Response(ModelErrorKind::InvalidResponse))
                }
                10 => {
                    f.response.reported_usage = Some(ModelUsage {
                        input_tokens: 6,
                        output_tokens: 1,
                    });
                    provider_outcome = ScriptedOutcome::Response(f.response.clone());
                    Inner::ReportedUsage(Usage::InputTokenCountMismatch)
                }
                11 => {
                    f.response.reported_usage = Some(ModelUsage {
                        input_tokens: 8,
                        output_tokens: 1,
                    });
                    provider_outcome = ScriptedOutcome::Response(f.response.clone());
                    Inner::ReportedUsage(Usage::InputTokenCountMismatch)
                }
                _ => {
                    f.response.output =
                        RawModelOutput::new("response-private-sentinel not json").unwrap();
                    f.response.reported_usage = None;
                    provider_outcome = ScriptedOutcome::Response(f.response.clone());
                    Inner::Admission(AdmissionError::MalformedSyntax)
                }
            };
            let selected = Arc::new(SentinelUncheckedProvider {
                inner: UncheckedScriptedProvider::new(
                    f.descriptor.clone(),
                    [
                        provider_outcome,
                        ScriptedOutcome::Error(ModelErrorKind::Internal),
                    ],
                ),
                endpoint: "endpoint-private-sentinel".into(),
                credential: "credential-private-sentinel".into(),
                private_diagnostic: "provider-private-sentinel".into(),
            });
            let requirements = remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]);
            let filtered = filtered_remote_fixture(PrivacyClass::ApprovedRemote);
            let make_untouched =
                |number: u128, privacy_class: PrivacyClass, byte_ineligible: bool| {
                    let mut descriptor = f.descriptor.clone();
                    descriptor.provider_id = id(number, ModelProviderId::new);
                    descriptor.model_id = id(number, nexa_domain::ModelId::new);
                    descriptor.privacy_class = privacy_class;
                    if byte_ineligible {
                        descriptor.capabilities.context_window_tokens =
                            filtered.filtered_compilation.compiled_bytes
                                + requirements.maximum_output_tokens
                                - 1;
                    }
                    Arc::new(
                        ScriptedModelProvider::new(
                            descriptor,
                            [ScriptedOutcome::Error(ModelErrorKind::Internal)],
                        )
                        .unwrap(),
                    )
                };
            // Complete disjoint matrix: eligible non-selected, unauthorized, explicitly
            // unavailable, availability-omitted, conservative-byte-ineligible, and local.
            let other = make_untouched(91_200, PrivacyClass::ApprovedRemote, false);
            let unauthorized = make_untouched(91_201, PrivacyClass::ApprovedRemote, false);
            let unavailable = make_untouched(91_202, PrivacyClass::ApprovedRemote, false);
            let omitted = make_untouched(91_203, PrivacyClass::ApprovedRemote, false);
            let byte_ineligible = make_untouched(91_204, PrivacyClass::ApprovedRemote, true);
            let local = make_untouched(91_205, PrivacyClass::LocalOnly, false);
            let untouched = [
                &other,
                &unauthorized,
                &unavailable,
                &omitted,
                &byte_ineligible,
                &local,
            ];
            let registry = ModelRegistry::try_from_providers(
                std::iter::once(selected.clone() as Arc<dyn LanguageModelProvider>).chain(
                    untouched
                        .iter()
                        .map(|provider| Arc::clone(provider) as Arc<dyn LanguageModelProvider>),
                ),
            )
            .unwrap();
            let tokenizer = SentinelTokenizer {
                inner: ScriptedModelInputTokenizer::new(tokenizer_descriptor, token_outcomes)
                    .unwrap(),
                private_diagnostic: "tokenizer-private-sentinel".into(),
            };
            let authorized_descriptors = [
                selected.descriptor(),
                other.descriptor(),
                unavailable.descriptor(),
                omitted.descriptor(),
                byte_ineligible.descriptor(),
            ];
            let authorization = crate::authorization::RemoteModelAuthorization::new(
                filtered.filtered_compilation.replay_anchor.clone(),
                authorized_descriptors
                    .into_iter()
                    .map(
                        |descriptor| crate::authorization::RemoteModelAuthorizationEntry {
                            provider_id: descriptor.provider_id,
                            model_id: descriptor.model_id,
                            privacy_class: descriptor.privacy_class,
                        },
                    )
                    .collect(),
            )
            .unwrap();
            let availability = crate::availability::ModelAvailabilitySnapshot::new(
                [
                    selected.descriptor(),
                    other.descriptor(),
                    unauthorized.descriptor(),
                    unavailable.descriptor(),
                    byte_ineligible.descriptor(),
                    local.descriptor(),
                ]
                .into_iter()
                .map(|descriptor| crate::availability::ModelAvailabilityEntry {
                    provider_id: descriptor.provider_id,
                    model_id: descriptor.model_id,
                    state: if descriptor == unavailable.descriptor() {
                        crate::availability::ModelAvailabilityState::Unavailable
                    } else {
                        crate::availability::ModelAvailabilityState::Available
                    },
                })
                .collect(),
            )
            .unwrap();
            let error = crate::generation::select_filtered_authorized_available_remote_model_tokenize_invoke_validate_reported_usage_and_admit(
                &registry,
                f.request.invocation_id,
                &requirements,
                &availability,
                &authorization,
                version,
                &tokenizer,
                &filtered,
                &f.authority,
                &f.context,
                &f.citations,
            )
            .unwrap_err();
            assert_eq!(
                error,
                Outer::UsageValidatedTokenizedInvocationAdmission(expected),
                "nested mode {mode}"
            );
            assert_eq!(tokenizer.inner.remaining().unwrap(), usize::from(mode <= 2));
            assert_eq!(selected.remaining(), if mode < 6 { 2 } else { 1 });
            for provider in untouched {
                assert_eq!(
                    provider.remaining(),
                    1,
                    "mode {mode} touched a disjoint provider"
                );
            }
            assert_content_free_diagnostics(
                &error,
                &[
                    "prompt-private-sentinel",
                    "identity-private-sentinel",
                    "learner-private-sentinel",
                    "knowledge-private-sentinel",
                    "conversation-private-sentinel",
                    "tool-private-sentinel",
                    "authorization-private-sentinel",
                    "tokenizer-private-sentinel",
                    "provider-private-sentinel",
                    "endpoint-private-sentinel",
                    "credential-private-sentinel",
                    "response-private-sentinel",
                    "usage-private-sentinel",
                ],
            );
        }
    }

    #[test]
    fn filtered_remote_usage_validated_tokenized_composition_is_exact_single_attempt_and_content_free(
    ) {
        use crate::admission::AdmissionError;
        use crate::authorization::{RemoteModelAuthorization, RemoteModelAuthorizationEntry};
        use crate::availability::{
            ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState,
        };
        use crate::generation::{
            select_filtered_authorized_available_remote_model_tokenize_invoke_validate_reported_usage_and_admit,
            FilteredAuthorizedAvailableRemoteUsageValidatedTokenizedInvocationAdmissionError as Outer,
            UsageValidatedTokenizedInvocationAdmissionError as Inner,
        };
        use crate::model::{
            LanguageModelProvider, ModelErrorKind, PrivacyClass, RawModelOutput,
            ScriptedModelProvider, ScriptedOutcome,
        };
        use crate::registry::ModelRegistry;
        use crate::tokenization::{
            ModelInputTokenizationError, ModelRequestTokenCapacityError,
            ScriptedModelInputTokenizer, ScriptedTokenizationOutcome,
            TokenizeAndValidateModelRequestCapacityError as Capacity, MODEL_INPUT_TOKENIZATION_V1,
        };
        use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId, ProtocolVersion};
        use std::sync::Arc;

        for (mode, reverse_registry_order) in (0..7).flat_map(|mode| [(mode, false), (mode, true)])
        {
            let mut f = admission_fixture();
            let filtered = filtered_remote_fixture(PrivacyClass::ApprovedRemote);
            f.context.tokenizer_profile_id = "knowledge-private-sentinel".into();
            let invocation_id = id(
                99_100 + mode * 2 + u128::from(reverse_registry_order),
                ModelInvocationId::new,
            );
            let mut selected_descriptor = f.descriptor.clone();
            selected_descriptor.provider_id = id(99_110, ModelProviderId::new);
            selected_descriptor.model_id = id(99_110, ModelId::new);
            selected_descriptor.privacy_class = PrivacyClass::ApprovedRemote;
            f.response.invocation_id = invocation_id;
            f.response.provider_id = selected_descriptor.provider_id;
            f.response.model_id = selected_descriptor.model_id;
            if mode == 6 {
                f.response.output = RawModelOutput::new("model-output-private-sentinel").unwrap();
            }
            let outcome = if mode == 5 {
                ScriptedOutcome::Error(ModelErrorKind::Unavailable)
            } else {
                ScriptedOutcome::Response(f.response.clone())
            };
            let selected = Arc::new(SentinelProvider {
                inner: ScriptedModelProvider::new(
                    selected_descriptor.clone(),
                    [outcome, ScriptedOutcome::Error(ModelErrorKind::Internal)],
                )
                .unwrap(),
                endpoint: "endpoint-private-sentinel".into(),
                credential: "credential-private-sentinel".into(),
                private_diagnostic: "provider-private-sentinel".into(),
            });
            let mut other_descriptor = selected_descriptor.clone();
            other_descriptor.provider_id = id(99_120, ModelProviderId::new);
            other_descriptor.model_id = id(99_120, ModelId::new);
            let other = Arc::new(
                ScriptedModelProvider::new(
                    other_descriptor.clone(),
                    [ScriptedOutcome::Error(ModelErrorKind::Internal)],
                )
                .unwrap(),
            );
            let mut local_descriptor = selected_descriptor.clone();
            local_descriptor.provider_id = id(99_130, ModelProviderId::new);
            local_descriptor.model_id = id(99_130, ModelId::new);
            local_descriptor.privacy_class = PrivacyClass::LocalOnly;
            let local = Arc::new(
                ScriptedModelProvider::new(
                    local_descriptor,
                    [ScriptedOutcome::Error(ModelErrorKind::Internal)],
                )
                .unwrap(),
            );
            let providers: Vec<Arc<dyn LanguageModelProvider>> = if reverse_registry_order {
                vec![other.clone(), local.clone(), selected.clone()]
            } else {
                vec![selected.clone(), local.clone(), other.clone()]
            };
            let registry = ModelRegistry::try_from_providers(providers).unwrap();
            let availability = ModelAvailabilitySnapshot::new(vec![
                ModelAvailabilityEntry {
                    provider_id: selected_descriptor.provider_id,
                    model_id: selected_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
                ModelAvailabilityEntry {
                    provider_id: other_descriptor.provider_id,
                    model_id: other_descriptor.model_id,
                    state: ModelAvailabilityState::Available,
                },
            ])
            .unwrap();
            let authorization = RemoteModelAuthorization::new(
                filtered.filtered_compilation.replay_anchor.clone(),
                vec![
                    RemoteModelAuthorizationEntry {
                        provider_id: selected_descriptor.provider_id,
                        model_id: selected_descriptor.model_id,
                        privacy_class: selected_descriptor.privacy_class,
                    },
                    RemoteModelAuthorizationEntry {
                        provider_id: other_descriptor.provider_id,
                        model_id: other_descriptor.model_id,
                        privacy_class: other_descriptor.privacy_class,
                    },
                ],
            )
            .unwrap();
            let requirements = remote_selection_requirements(vec![PrivacyClass::ApprovedRemote]);
            let exact = selected_descriptor.capabilities.context_window_tokens
                - requirements.maximum_output_tokens;
            let tokenizer_descriptor = if mode == 1 {
                other_descriptor.clone()
            } else {
                selected_descriptor.clone()
            };
            let token_outcome = match mode {
                2 => ScriptedTokenizationOutcome::Error,
                3 => ScriptedTokenizationOutcome::TokenCount(exact + 1),
                _ => ScriptedTokenizationOutcome::TokenCount(exact),
            };
            let tokenizer = SentinelTokenizer {
                inner: ScriptedModelInputTokenizer::new(tokenizer_descriptor, [token_outcome])
                    .unwrap(),
                private_diagnostic: "tokenizer-private-sentinel".into(),
            };
            let version = if mode == 0 {
                ProtocolVersion::new(2, 0)
            } else {
                MODEL_INPUT_TOKENIZATION_V1
            };
            let result =
                select_filtered_authorized_available_remote_model_tokenize_invoke_validate_reported_usage_and_admit(
                    &registry,
                    invocation_id,
                    &requirements,
                    &availability,
                    &authorization,
                    version,
                    &tokenizer,
                    &filtered,
                    &f.authority,
                    &f.context,
                    &f.citations,
                );
            match mode {
                0 => assert_eq!(
                    result,
                    Err(Outer::UsageValidatedTokenizedInvocationAdmission(
                        Inner::TokenizationCapacity(Capacity::Tokenization(
                            ModelInputTokenizationError::UnsupportedVersion
                        ))
                    ))
                ),
                1 => assert_eq!(
                    result,
                    Err(Outer::UsageValidatedTokenizedInvocationAdmission(
                        Inner::TokenizationCapacity(Capacity::Tokenization(
                            ModelInputTokenizationError::InvalidDescriptor
                        ))
                    ))
                ),
                2 => assert_eq!(
                    result,
                    Err(Outer::UsageValidatedTokenizedInvocationAdmission(
                        Inner::TokenizationCapacity(Capacity::Tokenization(
                            ModelInputTokenizationError::TokenizerFailure
                        ))
                    ))
                ),
                3 => assert_eq!(
                    result,
                    Err(Outer::UsageValidatedTokenizedInvocationAdmission(
                        Inner::TokenizationCapacity(Capacity::TokenCapacity(
                            ModelRequestTokenCapacityError::ExactCapacity
                        ))
                    ))
                ),
                4 => {
                    let result = result.clone().unwrap();
                    assert_eq!(result.tokenization_evidence.input_token_count, exact);
                    let request = crate::model::ModelRequest {
                        invocation_id,
                        provider_id: selected_descriptor.provider_id,
                        model_id: selected_descriptor.model_id,
                        contract_version: crate::model::MODEL_INVOCATION_V1,
                        input: filtered.filtered_compilation.model_input.clone(),
                        required_capabilities: requirements.required_capabilities.clone(),
                        maximum_output_tokens: requirements.maximum_output_tokens,
                    };
                    assert_eq!(
                        result.admission,
                        crate::admission::admit_model_output(
                            &selected_descriptor,
                            &request,
                            &f.response,
                            &filtered.filtered_compilation,
                            &f.authority,
                            &f.context,
                            &f.citations
                        )
                        .unwrap()
                    );
                    result
                        .tokenization_evidence
                        .validate_for(
                            &selected_descriptor,
                            &filtered.filtered_compilation.model_input,
                        )
                        .unwrap();
                }
                5 => assert_eq!(
                    result,
                    Err(Outer::UsageValidatedTokenizedInvocationAdmission(
                        Inner::Invocation(ModelErrorKind::Unavailable)
                    ))
                ),
                _ => assert_eq!(
                    result,
                    Err(Outer::UsageValidatedTokenizedInvocationAdmission(
                        Inner::Admission(AdmissionError::MalformedSyntax)
                    ))
                ),
            }
            assert_eq!(tokenizer.inner.remaining().unwrap(), usize::from(mode < 2));
            assert_eq!(selected.inner.remaining(), if mode < 4 { 2 } else { 1 });
            assert_eq!(other.remaining(), 1);
            assert_eq!(local.remaining(), 1);
            if let Err(error) = result {
                let diagnostics = format!("{error:?} {error}");
                for sentinel in [
                    "prompt-private-sentinel",
                    "learner-private-sentinel",
                    "knowledge-private-sentinel",
                    "authorization-private-sentinel",
                    "tokenizer-private-sentinel",
                    "provider-private-sentinel",
                    "endpoint-private-sentinel",
                    "credential-private-sentinel",
                    "model-output-private-sentinel",
                ] {
                    assert!(!diagnostics.contains(sentinel), "leaked {sentinel}");
                }
            }
        }
    }
}
