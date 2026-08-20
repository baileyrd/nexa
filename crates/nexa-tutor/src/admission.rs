//! Fail-closed admission of untrusted structured model output into response planning.

use crate::model::{FinishReason, ModelDescriptor, ModelRequest, ModelResponse};
use crate::prompt::{PromptCompilationResult, PROMPT_COMPILATION_V1};
use crate::{
    plan_response, Capability, DecisionEvidence, PlanningRequest, ResponseLimits, Scope,
    SectionRequest, TutorError, TutorResponse, V1,
};
use nexa_domain::{
    CitationSetId, ContextPackageId, HybridRetrievalResultId, InteractionId, ModelId,
    ModelInvocationId, ModelProviderId, ProtocolVersion, RetrievalQueryId, TutorResponseId,
};
use nexa_knowledge::{CitationResult, ContextPackage};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fmt};
use thiserror::Error;

pub const MODEL_OUTPUT_ADMISSION_V1: ProtocolVersion = ProtocolVersion::new(1, 0);
pub const CANDIDATE_OUTPUT_V1: ProtocolVersion = ProtocolVersion::new(1, 0);

/// Caller-owned authority. Model bytes can supply none of these fields.
#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrustedPlanningAuthority {
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
}

impl TrustedPlanningAuthority {
    pub fn validate(&self) -> Result<(), AdmissionError> {
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
            return Err(AdmissionError::UnsupportedVersion);
        }
        self.limits.validate().map_err(map_planning_error)?;
        self.evidence.validate().map_err(map_planning_error)?;
        if self.scope != self.evidence.scope {
            return Err(AdmissionError::PlanningEvidenceProvenance);
        }
        if self.permitted_capabilities.is_empty() {
            return Err(AdmissionError::PolicyPedagogySafetyCapability);
        }
        Ok(())
    }

    fn with_sections(&self, sections: Vec<SectionRequest>) -> PlanningRequest {
        PlanningRequest {
            contract_version: self.contract_version,
            response_id: self.response_id,
            interaction_id: self.interaction_id,
            scope: self.scope.clone(),
            context_package_id: self.context_package_id,
            citation_set_id: self.citation_set_id,
            hybrid_result_id: self.hybrid_result_id,
            query_id: self.query_id,
            response_policy_version: self.response_policy_version,
            safety_policy_version: self.safety_policy_version,
            citation_policy_version: self.citation_policy_version,
            governance_policy_version: self.governance_policy_version,
            limits: self.limits,
            permitted_capabilities: self.permitted_capabilities.clone(),
            evidence: self.evidence.clone(),
            sections,
        }
    }

    fn validate_against(
        &self,
        context: &ContextPackage,
        citations: &CitationResult,
    ) -> Result<(), AdmissionError> {
        self.validate()?;
        context
            .validate()
            .map_err(|_| AdmissionError::PlanningEvidenceProvenance)?;
        citations
            .validate()
            .map_err(|_| AdmissionError::CitationGroundingReference)?;
        if (
            self.context_package_id,
            self.hybrid_result_id,
            self.query_id,
        ) != (
            context.context_package_id,
            context.hybrid_result_id,
            context.query_id,
        ) || (
            self.citation_set_id,
            self.context_package_id,
            self.hybrid_result_id,
            self.query_id,
        ) != (
            citations.citation_set_id,
            citations.context_package_id,
            citations.hybrid_result_id,
            citations.query_id,
        ) || self.governance_policy_version != context.governance_policy_version
            || self.citation_policy_version != citations.citation_policy_version
        {
            return Err(AdmissionError::PlanningEvidenceProvenance);
        }
        Ok(())
    }
}

impl fmt::Debug for TrustedPlanningAuthority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TrustedPlanningAuthority")
            .field("response_id", &self.response_id)
            .field("interaction_id", &self.interaction_id)
            .finish_non_exhaustive()
    }
}

impl<'de> Deserialize<'de> for TrustedPlanningAuthority {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
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
        }
        let w = Wire::deserialize(d)?;
        let value = Self {
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
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CandidateOutputV1 {
    candidate_schema_version: ProtocolVersion,
    sections: Vec<SectionRequest>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissionEvidence {
    pub contract_version: ProtocolVersion,
    pub provider_id: ModelProviderId,
    pub model_id: ModelId,
    pub invocation_id: ModelInvocationId,
    pub prompt_compilation_replay_anchor: String,
    pub prompt_package_version: ProtocolVersion,
    pub context_builder_version: ProtocolVersion,
    pub candidate_schema_version: ProtocolVersion,
    pub finish_reason: FinishReason,
    pub raw_output_sha256: String,
    pub tutor_response_replay_anchor: String,
    pub admission_replay_anchor: String,
}

impl<'de> Deserialize<'de> for AdmissionEvidence {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            contract_version: ProtocolVersion,
            provider_id: ModelProviderId,
            model_id: ModelId,
            invocation_id: ModelInvocationId,
            prompt_compilation_replay_anchor: String,
            prompt_package_version: ProtocolVersion,
            context_builder_version: ProtocolVersion,
            candidate_schema_version: ProtocolVersion,
            finish_reason: FinishReason,
            raw_output_sha256: String,
            tutor_response_replay_anchor: String,
            admission_replay_anchor: String,
        }

        let wire = Wire::deserialize(deserializer)?;
        let evidence = Self {
            contract_version: wire.contract_version,
            provider_id: wire.provider_id,
            model_id: wire.model_id,
            invocation_id: wire.invocation_id,
            prompt_compilation_replay_anchor: wire.prompt_compilation_replay_anchor,
            prompt_package_version: wire.prompt_package_version,
            context_builder_version: wire.context_builder_version,
            candidate_schema_version: wire.candidate_schema_version,
            finish_reason: wire.finish_reason,
            raw_output_sha256: wire.raw_output_sha256,
            tutor_response_replay_anchor: wire.tutor_response_replay_anchor,
            admission_replay_anchor: wire.admission_replay_anchor,
        };
        evidence.validate().map_err(serde::de::Error::custom)?;
        Ok(evidence)
    }
}

impl AdmissionEvidence {
    fn compute_anchor(&self) -> Result<String, AdmissionError> {
        #[derive(Serialize)]
        struct Bound<'a> {
            contract_version: ProtocolVersion,
            provider_id: ModelProviderId,
            model_id: ModelId,
            invocation_id: ModelInvocationId,
            prompt_compilation_replay_anchor: &'a str,
            prompt_package_version: ProtocolVersion,
            context_builder_version: ProtocolVersion,
            candidate_schema_version: ProtocolVersion,
            finish_reason: FinishReason,
            raw_output_sha256: &'a str,
            tutor_response_replay_anchor: &'a str,
        }
        let bytes = serde_json::to_vec(&Bound {
            contract_version: self.contract_version,
            provider_id: self.provider_id,
            model_id: self.model_id,
            invocation_id: self.invocation_id,
            prompt_compilation_replay_anchor: &self.prompt_compilation_replay_anchor,
            prompt_package_version: self.prompt_package_version,
            context_builder_version: self.context_builder_version,
            candidate_schema_version: self.candidate_schema_version,
            finish_reason: self.finish_reason,
            raw_output_sha256: &self.raw_output_sha256,
            tutor_response_replay_anchor: &self.tutor_response_replay_anchor,
        })
        .map_err(|_| AdmissionError::InternalFraming)?;
        Ok(hex_hash(&bytes))
    }

    fn validate(&self) -> Result<(), AdmissionError> {
        if self.contract_version != MODEL_OUTPUT_ADMISSION_V1
            || self.prompt_package_version != V1
            || self.context_builder_version != V1
            || self.candidate_schema_version != CANDIDATE_OUTPUT_V1
        {
            return Err(AdmissionError::UnsupportedVersion);
        }
        if self.finish_reason != FinishReason::Complete
            || !valid_hash(&self.raw_output_sha256)
            || !valid_hash(&self.prompt_compilation_replay_anchor)
            || !valid_hash(&self.tutor_response_replay_anchor)
            || !valid_hash(&self.admission_replay_anchor)
        {
            return Err(AdmissionError::PromptAssociationReplayMismatch);
        }
        if self.compute_anchor()? != self.admission_replay_anchor {
            return Err(AdmissionError::PromptAssociationReplayMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissionResult {
    pub response: TutorResponse,
    pub evidence: AdmissionEvidence,
}

impl fmt::Debug for AdmissionResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AdmissionResult")
            .field("response", &self.response)
            .field("evidence", &self.evidence)
            .finish()
    }
}

impl<'de> Deserialize<'de> for AdmissionResult {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            response: TutorResponse,
            evidence: AdmissionEvidence,
        }
        let w = Wire::deserialize(d)?;
        w.response.validate().map_err(serde::de::Error::custom)?;
        w.evidence.validate().map_err(serde::de::Error::custom)?;
        if w.response.replay_anchor != w.evidence.tutor_response_replay_anchor {
            return Err(serde::de::Error::custom("invalid admission evidence"));
        }
        Ok(Self {
            response: w.response,
            evidence: w.evidence,
        })
    }
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum AdmissionError {
    #[error("model-output admission uses an unsupported version")]
    UnsupportedVersion,
    #[error("model descriptor or request is invalid")]
    InvalidDescriptorRequest,
    #[error("prompt association or replay evidence is invalid")]
    PromptAssociationReplayMismatch,
    #[error("model response identity is invalid")]
    ModelResponseIdentityMismatch,
    #[error("structured output is unsupported or was not required")]
    UnsupportedStructuredOutput,
    #[error("model response is incomplete")]
    IncompleteOutput,
    #[error("candidate output syntax is malformed")]
    MalformedSyntax,
    #[error("candidate output schema is invalid")]
    InvalidCandidateSchema,
    #[error("planning evidence or provenance is invalid")]
    PlanningEvidenceProvenance,
    #[error("policy, pedagogy, safety, or capability validation failed")]
    PolicyPedagogySafetyCapability,
    #[error("citation or grounding-reference validation failed")]
    CitationGroundingReference,
    #[error("deterministic admission framing failed")]
    InternalFraming,
}

pub fn admit_model_output(
    descriptor: &ModelDescriptor,
    request: &ModelRequest,
    response: &ModelResponse,
    compilation: &PromptCompilationResult,
    authority: &TrustedPlanningAuthority,
    context: &ContextPackage,
    citations: &CitationResult,
) -> Result<AdmissionResult, AdmissionError> {
    validate_admission_preflight(
        descriptor,
        request,
        compilation,
        authority,
        context,
        citations,
    )?;
    admit_model_output_after_preflight(
        request,
        response,
        compilation,
        authority,
        context,
        citations,
    )
}

pub(crate) fn validate_admission_preflight(
    descriptor: &ModelDescriptor,
    request: &ModelRequest,
    compilation: &PromptCompilationResult,
    authority: &TrustedPlanningAuthority,
    context: &ContextPackage,
    citations: &CitationResult,
) -> Result<(), AdmissionError> {
    descriptor
        .validate()
        .map_err(|_| AdmissionError::InvalidDescriptorRequest)?;
    if !request.required_capabilities.structured_output
        || !descriptor.capabilities.structured_output
    {
        return Err(AdmissionError::UnsupportedStructuredOutput);
    }
    request
        .validate_for(descriptor)
        .map_err(|_| AdmissionError::InvalidDescriptorRequest)?;
    if compilation.contract_version != PROMPT_COMPILATION_V1
        || compilation.prompt_package_version != V1
        || compilation.context_builder_version != V1
        || compilation.output_schema_version != CANDIDATE_OUTPUT_V1
    {
        return Err(AdmissionError::UnsupportedVersion);
    }
    compilation
        .validate()
        .map_err(|_| AdmissionError::PromptAssociationReplayMismatch)?;
    if request.input != compilation.model_input {
        return Err(AdmissionError::PromptAssociationReplayMismatch);
    }
    authority.validate_against(context, citations)?;
    Ok(())
}

pub(crate) fn admit_model_output_after_preflight(
    request: &ModelRequest,
    response: &ModelResponse,
    compilation: &PromptCompilationResult,
    authority: &TrustedPlanningAuthority,
    context: &ContextPackage,
    citations: &CitationResult,
) -> Result<AdmissionResult, AdmissionError> {
    response
        .validate_for(request)
        .map_err(|error| match error.kind {
            crate::model::ModelErrorKind::IdentityMismatch => {
                AdmissionError::ModelResponseIdentityMismatch
            }
            crate::model::ModelErrorKind::UnsupportedVersion => AdmissionError::UnsupportedVersion,
            _ => AdmissionError::InvalidDescriptorRequest,
        })?;
    if response.finish_reason != FinishReason::Complete {
        return Err(AdmissionError::IncompleteOutput);
    }
    let mut deserializer = serde_json::Deserializer::from_str(response.output.as_str());
    let candidate =
        CandidateOutputV1::deserialize(&mut deserializer).map_err(|error| {
            match error.classify() {
                serde_json::error::Category::Syntax | serde_json::error::Category::Eof => {
                    AdmissionError::MalformedSyntax
                }
                _ => AdmissionError::InvalidCandidateSchema,
            }
        })?;
    deserializer
        .end()
        .map_err(|_| AdmissionError::MalformedSyntax)?;
    if candidate.candidate_schema_version != CANDIDATE_OUTPUT_V1 {
        return Err(AdmissionError::UnsupportedVersion);
    }
    if candidate.sections.is_empty() {
        return Err(AdmissionError::InvalidCandidateSchema);
    }
    let planning_request = authority.with_sections(candidate.sections);
    let tutor_response =
        plan_response(&planning_request, context, citations).map_err(map_planning_error)?;
    let mut evidence = AdmissionEvidence {
        contract_version: MODEL_OUTPUT_ADMISSION_V1,
        provider_id: response.provider_id,
        model_id: response.model_id,
        invocation_id: response.invocation_id,
        prompt_compilation_replay_anchor: compilation.replay_anchor.clone(),
        prompt_package_version: compilation.prompt_package_version,
        context_builder_version: compilation.context_builder_version,
        candidate_schema_version: candidate.candidate_schema_version,
        finish_reason: response.finish_reason,
        raw_output_sha256: hex_hash(response.output.as_str().as_bytes()),
        tutor_response_replay_anchor: tutor_response.replay_anchor.clone(),
        admission_replay_anchor: String::new(),
    };
    evidence.admission_replay_anchor = evidence.compute_anchor()?;
    Ok(AdmissionResult {
        response: tutor_response,
        evidence,
    })
}

fn map_planning_error(error: TutorError) -> AdmissionError {
    match error {
        TutorError::UnsupportedVersion => AdmissionError::UnsupportedVersion,
        TutorError::ProvenanceMismatch | TutorError::InvalidLimit | TutorError::ReplayMismatch => {
            AdmissionError::PlanningEvidenceProvenance
        }
        TutorError::CitationMismatch => AdmissionError::CitationGroundingReference,
        TutorError::InvalidEvidence
        | TutorError::UnsupportedCapability
        | TutorError::UnsafeContent => AdmissionError::PolicyPedagogySafetyCapability,
        TutorError::IdentityConflict | TutorError::InvalidStructure => {
            AdmissionError::InvalidCandidateSchema
        }
    }
}
fn hex_hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[cfg(test)]
mod evidence_tests {
    use super::*;
    use serde_json::{json, Value};
    use uuid::Uuid;

    fn id<T>(
        value: u128,
        constructor: impl FnOnce(Uuid) -> Result<T, nexa_domain::ValueError>,
    ) -> T {
        constructor(Uuid::from_u128(value)).unwrap()
    }

    fn evidence() -> AdmissionEvidence {
        let mut evidence = AdmissionEvidence {
            contract_version: MODEL_OUTPUT_ADMISSION_V1,
            provider_id: id(1, ModelProviderId::new),
            model_id: id(2, ModelId::new),
            invocation_id: id(3, ModelInvocationId::new),
            prompt_compilation_replay_anchor: "1".repeat(64),
            prompt_package_version: V1,
            context_builder_version: V1,
            candidate_schema_version: CANDIDATE_OUTPUT_V1,
            finish_reason: FinishReason::Complete,
            raw_output_sha256: "2".repeat(64),
            tutor_response_replay_anchor: "3".repeat(64),
            admission_replay_anchor: String::new(),
        };
        evidence.admission_replay_anchor = evidence.compute_anchor().unwrap();
        evidence
    }

    fn rejects(mut value: Value, field: &str, replacement: Value) {
        value[field] = replacement;
        assert!(serde_json::from_value::<AdmissionEvidence>(value).is_err());
    }

    #[test]
    fn standalone_evidence_round_trips_and_rejects_unknown_fields() {
        let evidence = evidence();
        let wire = serde_json::to_string(&evidence).unwrap();
        assert_eq!(
            serde_json::from_str::<AdmissionEvidence>(&wire).unwrap(),
            evidence
        );

        let mut value = serde_json::to_value(evidence).unwrap();
        value["unknown"] = json!(true);
        assert!(serde_json::from_value::<AdmissionEvidence>(value).is_err());
    }

    #[test]
    fn standalone_evidence_rejects_every_unsupported_version_and_finish_reason() {
        let value = serde_json::to_value(evidence()).unwrap();
        for field in [
            "contract_version",
            "prompt_package_version",
            "context_builder_version",
            "candidate_schema_version",
        ] {
            rejects(value.clone(), field, json!("2.0"));
        }
        rejects(value, "finish_reason", json!("output_limit"));
    }

    #[test]
    fn standalone_evidence_rejects_malformed_and_noncanonical_hashes() {
        let value = serde_json::to_value(evidence()).unwrap();
        for field in [
            "prompt_compilation_replay_anchor",
            "raw_output_sha256",
            "tutor_response_replay_anchor",
            "admission_replay_anchor",
        ] {
            rejects(value.clone(), field, json!("a".repeat(63)));
            rejects(value.clone(), field, json!("A".repeat(64)));
            rejects(value.clone(), field, json!("g".repeat(64)));
        }
    }

    #[test]
    fn standalone_evidence_anchor_binds_every_material_field() {
        let value = serde_json::to_value(evidence()).unwrap();
        let changes = [
            ("contract_version", json!("1.1")),
            ("provider_id", json!(Uuid::from_u128(11))),
            ("model_id", json!(Uuid::from_u128(12))),
            ("invocation_id", json!(Uuid::from_u128(13))),
            ("prompt_compilation_replay_anchor", json!("4".repeat(64))),
            ("prompt_package_version", json!("1.1")),
            ("context_builder_version", json!("1.1")),
            ("candidate_schema_version", json!("1.1")),
            ("finish_reason", json!("output_limit")),
            ("raw_output_sha256", json!("5".repeat(64))),
            ("tutor_response_replay_anchor", json!("6".repeat(64))),
        ];
        let original = evidence();
        let original_anchor = original.compute_anchor().unwrap();
        for (field, replacement) in changes {
            let mut changed = original.clone();
            match field {
                "contract_version" => changed.contract_version = ProtocolVersion::new(1, 1),
                "provider_id" => changed.provider_id = id(11, ModelProviderId::new),
                "model_id" => changed.model_id = id(12, ModelId::new),
                "invocation_id" => changed.invocation_id = id(13, ModelInvocationId::new),
                "prompt_compilation_replay_anchor" => {
                    changed.prompt_compilation_replay_anchor = "4".repeat(64)
                }
                "prompt_package_version" => {
                    changed.prompt_package_version = ProtocolVersion::new(1, 1)
                }
                "context_builder_version" => {
                    changed.context_builder_version = ProtocolVersion::new(1, 1)
                }
                "candidate_schema_version" => {
                    changed.candidate_schema_version = ProtocolVersion::new(1, 1)
                }
                "finish_reason" => changed.finish_reason = FinishReason::OutputLimit,
                "raw_output_sha256" => changed.raw_output_sha256 = "5".repeat(64),
                "tutor_response_replay_anchor" => {
                    changed.tutor_response_replay_anchor = "6".repeat(64)
                }
                _ => unreachable!(),
            }
            assert_ne!(
                changed.compute_anchor().unwrap(),
                original_anchor,
                "{field}"
            );
            rejects(value.clone(), field, replacement);
        }
    }
}
