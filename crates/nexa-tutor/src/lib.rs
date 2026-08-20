//! Provider-neutral, synchronous response planning over caller-supplied text.
#![forbid(unsafe_code)]

pub mod model;

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
    fn validate(&self) -> Result<(), TutorError> {
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
    fn validate(&self) -> Result<(), TutorError> {
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
