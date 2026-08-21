//! Provider-neutral, synchronous response planning over caller-supplied text.
#![forbid(unsafe_code)]

pub mod admission;
pub mod availability;
pub mod generation;
pub mod model;
pub mod prompt;
pub mod registry;
pub mod selection;

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
                    "distinctive private platform prompt",
                ),
                layer(PromptLayerKind::NexaIdentity, "identity"),
                layer(PromptLayerKind::Policy, "policy"),
                layer(PromptLayerKind::Pedagogy, "pedagogy"),
                layer(
                    PromptLayerKind::StudentInput,
                    "distinctive private learner prompt",
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
        use crate::generation::{invoke_and_admit_model_output, InvocationAdmissionError};
        use crate::model::ModelInput;
        use nexa_domain::{
            CitationSetId, ContextPackageId, HybridRetrievalResultId, ModelId, ModelProviderId,
            RetrievalQueryId, StudentId,
        };

        // Each index isolates one host-controlled preflight class. The counting provider is
        // intentionally non-validating so invalid descriptors can reach coordinator preflight.
        for mutation in 0..33 {
            let mut f = admission_fixture();
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
}
