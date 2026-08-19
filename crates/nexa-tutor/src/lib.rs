//! Provider-neutral, synchronous response planning over caller-supplied text.
#![forbid(unsafe_code)]

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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
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
impl fmt::Debug for InertText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("InertText([REDACTED])")
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CitationBinding {
    pub claim_id: ClaimId,
    pub citation_id: CitationId,
    pub claim_position: u32,
    pub citation_position: u32,
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
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
impl fmt::Debug for SectionRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SectionRequest")
            .field("section_id", &self.section_id)
            .field("kind", &self.kind)
            .field("content", &"[REDACTED]")
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
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
impl fmt::Debug for PlanningRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlanningRequest")
            .field("response_id", &self.response_id)
            .field("section_count", &self.sections.len())
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseLimits {
    pub maximum_sections: usize,
    pub maximum_section_bytes: usize,
    pub maximum_response_bytes: usize,
    pub maximum_references_per_section: usize,
}
impl ResponseLimits {
    fn validate(self) -> Result<(), TutorError> {
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

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlannedSection {
    pub section_id: TutorSectionId,
    pub position: u32,
    pub kind: SectionKind,
    pub content: InertText,
    pub claims: Vec<ClaimId>,
    pub citations: Vec<CitationBinding>,
    pub pedagogy_decision_evidence_id: EvidenceId,
    pub safety: SafetyClassification,
    pub capability: Capability,
    pub scaffolding: u8,
    pub assessment_restriction: AssessmentRestriction,
}
impl fmt::Debug for PlannedSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlannedSection")
            .field("section_id", &self.section_id)
            .field("position", &self.position)
            .field("content", &"[REDACTED]")
            .finish()
    }
}

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
            if protected
                && (s.assessment_restriction == AssessmentRestriction::None
                    || !matches!(
                        s.kind,
                        SectionKind::Hint
                            | SectionKind::CheckForUnderstanding
                            | SectionKind::ConstrainedResponse
                            | SectionKind::SafetyRefusal
                    ))
            {
                return Err(TutorError::InvalidEvidence);
            }
            match s.safety {
                SafetyClassification::RefusalRequired if s.kind != SectionKind::SafetyRefusal => {
                    return Err(TutorError::InvalidEvidence)
                }
                SafetyClassification::ConstrainedRequired
                | SafetyClassification::AssessmentProtected
                    if !matches!(
                        s.kind,
                        SectionKind::ConstrainedResponse | SectionKind::SafetyRefusal
                    ) =>
                {
                    return Err(TutorError::InvalidEvidence)
                }
                _ => {}
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
        let mut ids = BTreeSet::new();
        let mut total = 0;
        for (i, s) in self.sections.iter().enumerate() {
            if s.position as usize != i + 1
                || !ids.insert(s.section_id)
                || expected_capability(s.kind) != s.capability
                || !self.permitted_capabilities.contains(&s.capability)
            {
                return Err(TutorError::InvalidStructure);
            }
            total += s.content.as_str().len();
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
            status: ResponseStatus::Accepted,
            rationale: vec![Rationale::Validated],
            sections: vec![PlannedSection {
                section_id: id(13, TutorSectionId::new),
                position: 1,
                kind: SectionKind::Explanation,
                content: InertText::new("Caller supplied explanation.").unwrap(),
                claims: vec![],
                citations: vec![],
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
    fn implementation_has_no_generation_provider_or_async_surface() {
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
