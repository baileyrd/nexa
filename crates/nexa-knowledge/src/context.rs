//! Provider-free deterministic assembly of governed hybrid retrieval material.
use crate::{ContentHash, HybridExclusionReason, HybridRetrievalResult, ProfileFingerprint, V1};
use nexa_domain::{
    ContextPackageId, HybridRetrievalResultId, KnowledgeArtifactId, KnowledgeChunkId,
    KnowledgeSourceId, ProtocolVersion, RetrievalQueryId,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};
use thiserror::Error;

pub const CONTEXT_ASSEMBLY_V1: ProtocolVersion = V1;
pub const TOKEN_ACCOUNTING_V1: ProtocolVersion = V1;
pub const MAX_CONTEXT_TOKENS: u64 = 16_000_000;
pub const MAX_CONTEXT_CHUNKS: usize = 100;
pub const MAX_TOKENIZER_ID_BYTES: usize = 255;

macro_rules! wire {($name:ident{$($field:ident:$ty:ty),*$(,)?})=>{impl<'de> Deserialize<'de> for $name {fn deserialize<D:serde::Deserializer<'de>>(d:D)->Result<Self,D::Error>{#[derive(Deserialize)]#[serde(deny_unknown_fields)]struct W{$($field:$ty),*}let w=W::deserialize(d)?;let x=Self{$($field:w.$field),*};x.validate().map_err(serde::de::Error::custom)?;Ok(x)}}};}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderingPolicy {
    HybridRank,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackingPolicy {
    RankedGreedyWholeChunk,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TokenAccountingPolicy {
    pub policy_version: ProtocolVersion,
    pub fixed_package_overhead: u64,
    pub per_chunk_overhead: u64,
    pub separator_tokens: u64,
    pub metadata_reference_overhead: u64,
}
impl TokenAccountingPolicy {
    fn validate(&self) -> Result<(), ContextError> {
        if self.policy_version != TOKEN_ACCOUNTING_V1 {
            return Err(ContextError::UnsupportedContract);
        }
        self.per_chunk_contribution(0)?;
        Ok(())
    }
    fn per_chunk_contribution(&self, content: u64) -> Result<u64, ContextError> {
        self.per_chunk_overhead
            .checked_add(self.separator_tokens)
            .and_then(|x| x.checked_add(self.metadata_reference_overhead))
            .and_then(|x| x.checked_add(content))
            .ok_or(ContextError::ArithmeticOverflow)
    }
}
wire!(TokenAccountingPolicy {
    policy_version: ProtocolVersion,
    fixed_package_overhead: u64,
    per_chunk_overhead: u64,
    separator_tokens: u64,
    metadata_reference_overhead: u64
});

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContextAssemblyRequest {
    pub contract_version: ProtocolVersion,
    pub context_package_id: ContextPackageId,
    pub hybrid_result_id: HybridRetrievalResultId,
    pub query_id: RetrievalQueryId,
    pub assembly_policy_version: ProtocolVersion,
    pub governance_policy_version: ProtocolVersion,
    pub tokenizer_profile_id: String,
    pub tokenizer_fingerprint: ProfileFingerprint,
    pub maximum_tokens: u64,
    pub maximum_chunk_tokens: Option<u64>,
    pub maximum_chunks: Option<usize>,
    pub ordering_policy: OrderingPolicy,
    pub packing_policy: PackingPolicy,
    pub accounting: TokenAccountingPolicy,
}
impl ContextAssemblyRequest {
    pub fn validate(&self) -> Result<(), ContextError> {
        if self.contract_version != V1
            || self.assembly_policy_version != CONTEXT_ASSEMBLY_V1
            || self.governance_policy_version != V1
        {
            return Err(ContextError::UnsupportedContract);
        }
        if self.tokenizer_profile_id.is_empty()
            || self.tokenizer_profile_id.len() > MAX_TOKENIZER_ID_BYTES
        {
            return Err(ContextError::TokenizerMismatch);
        }
        if self.maximum_tokens == 0
            || self.maximum_tokens > MAX_CONTEXT_TOKENS
            || self.maximum_chunk_tokens == Some(0)
            || self
                .maximum_chunk_tokens
                .is_some_and(|x| x > MAX_CONTEXT_TOKENS)
            || self.maximum_chunks == Some(0)
            || self.maximum_chunks.is_some_and(|x| x > MAX_CONTEXT_CHUNKS)
        {
            return Err(ContextError::InvalidLimit);
        }
        self.accounting.validate()?;
        if self.accounting.fixed_package_overhead > self.maximum_tokens {
            return Err(ContextError::InvalidLimit);
        }
        Ok(())
    }
}
wire!(ContextAssemblyRequest{contract_version:ProtocolVersion,context_package_id:ContextPackageId,hybrid_result_id:HybridRetrievalResultId,query_id:RetrievalQueryId,assembly_policy_version:ProtocolVersion,governance_policy_version:ProtocolVersion,tokenizer_profile_id:String,tokenizer_fingerprint:ProfileFingerprint,maximum_tokens:u64,maximum_chunk_tokens:Option<u64>,maximum_chunks:Option<usize>,ordering_policy:OrderingPolicy,packing_policy:PackingPolicy,accounting:TokenAccountingPolicy});

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GovernedChunkMaterial {
    pub chunk_id: KnowledgeChunkId,
    pub artifact_id: KnowledgeArtifactId,
    pub source_id: KnowledgeSourceId,
    pub source_version: u64,
    pub content_fingerprint: ContentHash,
    pub tokenizer_profile_id: String,
    pub tokenizer_fingerprint: ProfileFingerprint,
    pub exact_token_count: u64,
    pub content: String,
}
impl fmt::Debug for GovernedChunkMaterial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GovernedChunkMaterial")
            .field("chunk_id", &self.chunk_id)
            .field("content", &"[REDACTED]")
            .finish()
    }
}
impl GovernedChunkMaterial {
    fn validate(&self) -> Result<(), ContextError> {
        if self.source_version == 0
            || self.tokenizer_profile_id.is_empty()
            || self.tokenizer_profile_id.len() > MAX_TOKENIZER_ID_BYTES
            || self.exact_token_count > MAX_CONTEXT_TOKENS
        {
            return Err(ContextError::InvalidMaterial);
        }
        self.content_fingerprint
            .verify(self.content.as_bytes())
            .map_err(|_| ContextError::IntegrityMismatch)
    }
}
wire!(GovernedChunkMaterial {
    chunk_id: KnowledgeChunkId,
    artifact_id: KnowledgeArtifactId,
    source_id: KnowledgeSourceId,
    source_version: u64,
    content_fingerprint: ContentHash,
    tokenizer_profile_id: String,
    tokenizer_fingerprint: ProfileFingerprint,
    exact_token_count: u64,
    content: String
});

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextExclusionReason {
    MissingGovernedMaterial,
    PerChunkLimit,
    TokenBudget,
    ChunkCountLimit,
    UpstreamGovernance,
    UpstreamExclusion,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IncludedChunk {
    pub chunk_id: KnowledgeChunkId,
    pub artifact_id: KnowledgeArtifactId,
    pub source_id: KnowledgeSourceId,
    pub source_version: u64,
    pub hybrid_rank: u32,
    pub content_fingerprint: ContentHash,
    pub exact_token_count: u64,
    pub overhead_tokens: u64,
    pub total_contribution: u64,
    pub final_context_position: u32,
}
impl IncludedChunk {
    fn validate(&self) -> Result<(), ContextError> {
        if self.source_version == 0
            || self.hybrid_rank == 0
            || self.final_context_position == 0
            || self.overhead_tokens.checked_add(self.exact_token_count)
                != Some(self.total_contribution)
        {
            Err(ContextError::InvalidResult)
        } else {
            Ok(())
        }
    }
}
wire!(IncludedChunk {
    chunk_id: KnowledgeChunkId,
    artifact_id: KnowledgeArtifactId,
    source_id: KnowledgeSourceId,
    source_version: u64,
    hybrid_rank: u32,
    content_fingerprint: ContentHash,
    exact_token_count: u64,
    overhead_tokens: u64,
    total_contribution: u64,
    final_context_position: u32
});

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContextExclusion {
    pub chunk_id: KnowledgeChunkId,
    pub artifact_id: KnowledgeArtifactId,
    pub source_id: KnowledgeSourceId,
    pub source_version: u64,
    pub hybrid_rank: Option<u32>,
    pub reason: ContextExclusionReason,
    pub upstream_reason: Option<HybridExclusionReason>,
}
impl ContextExclusion {
    fn validate(&self) -> Result<(), ContextError> {
        let upstream = matches!(
            self.reason,
            ContextExclusionReason::UpstreamGovernance | ContextExclusionReason::UpstreamExclusion
        );
        if self.source_version == 0
            || upstream != self.upstream_reason.is_some()
            || upstream != self.hybrid_rank.is_none()
        {
            Err(ContextError::InvalidResult)
        } else {
            Ok(())
        }
    }
}
wire!(ContextExclusion{chunk_id:KnowledgeChunkId,artifact_id:KnowledgeArtifactId,source_id:KnowledgeSourceId,source_version:u64,hybrid_rank:Option<u32>,reason:ContextExclusionReason,upstream_reason:Option<HybridExclusionReason>});

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContextContent {
    pub chunk_id: KnowledgeChunkId,
    pub final_context_position: u32,
    pub content: String,
}
impl fmt::Debug for ContextContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContextContent")
            .field("chunk_id", &self.chunk_id)
            .field("final_context_position", &self.final_context_position)
            .field("content", &"[REDACTED]")
            .finish()
    }
}
impl ContextContent {
    fn validate(&self) -> Result<(), ContextError> {
        if self.final_context_position == 0 {
            Err(ContextError::InvalidResult)
        } else {
            Ok(())
        }
    }
}
wire!(ContextContent {
    chunk_id: KnowledgeChunkId,
    final_context_position: u32,
    content: String
});

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContextPackage {
    pub contract_version: ProtocolVersion,
    pub context_package_id: ContextPackageId,
    pub hybrid_result_id: HybridRetrievalResultId,
    pub query_id: RetrievalQueryId,
    pub assembly_policy_version: ProtocolVersion,
    pub governance_policy_version: ProtocolVersion,
    pub tokenizer_profile_id: String,
    pub tokenizer_fingerprint: ProfileFingerprint,
    pub maximum_tokens: u64,
    pub maximum_chunk_tokens: Option<u64>,
    pub maximum_chunks: Option<usize>,
    pub ordering_policy: OrderingPolicy,
    pub packing_policy: PackingPolicy,
    pub accounting: TokenAccountingPolicy,
    pub hybrid_candidate_count: usize,
    pub used_tokens: u64,
    pub remaining_tokens: u64,
    pub included: Vec<IncludedChunk>,
    pub exclusions: Vec<ContextExclusion>,
    pub content: Vec<ContextContent>,
}
impl fmt::Debug for ContextPackage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContextPackage")
            .field("context_package_id", &self.context_package_id)
            .field("included", &self.included)
            .field("exclusions", &self.exclusions)
            .field("content", &"[REDACTED]")
            .finish()
    }
}
impl ContextPackage {
    pub fn validate(&self) -> Result<(), ContextError> {
        if self.contract_version != V1
            || self.assembly_policy_version != V1
            || self.governance_policy_version != V1
            || self.accounting.policy_version != V1
            || self.maximum_tokens == 0
            || self.maximum_tokens > MAX_CONTEXT_TOKENS
            || self.tokenizer_profile_id.is_empty()
            || self.tokenizer_profile_id.len() > MAX_TOKENIZER_ID_BYTES
            || self.maximum_chunk_tokens == Some(0)
            || self
                .maximum_chunk_tokens
                .is_some_and(|x| x > MAX_CONTEXT_TOKENS)
            || self.maximum_chunks == Some(0)
            || self.maximum_chunks.is_some_and(|x| x > MAX_CONTEXT_CHUNKS)
            || self.hybrid_candidate_count > MAX_CONTEXT_CHUNKS
        {
            return Err(ContextError::UnsupportedContract);
        }
        self.accounting.validate()?;
        let mut ids = BTreeSet::new();
        let mut ranks = BTreeSet::new();
        let mut sum = self.accounting.fixed_package_overhead;
        for (i, x) in self.included.iter().enumerate() {
            x.validate()?;
            if x.final_context_position as usize != i + 1
                || !ids.insert(x.chunk_id)
                || !ranks.insert(x.hybrid_rank)
                || x.overhead_tokens != self.accounting.per_chunk_contribution(0)?
                || self
                    .maximum_chunk_tokens
                    .is_some_and(|m| x.exact_token_count > m)
            {
                return Err(ContextError::InvalidResult);
            }
            sum = sum
                .checked_add(x.total_contribution)
                .ok_or(ContextError::ArithmeticOverflow)?;
        }
        for x in &self.exclusions {
            x.validate()?;
            if !ids.insert(x.chunk_id) {
                return Err(ContextError::InvalidResult);
            }
            if let Some(r) = x.hybrid_rank {
                if !ranks.insert(r) {
                    return Err(ContextError::InvalidResult);
                }
            }
        }
        if ranks.len() != self.hybrid_candidate_count
            || ranks
                .iter()
                .copied()
                .ne(1..=u32::try_from(self.hybrid_candidate_count)
                    .map_err(|_| ContextError::InvalidResult)?)
            || sum != self.used_tokens
            || self.used_tokens.checked_add(self.remaining_tokens) != Some(self.maximum_tokens)
            || self.used_tokens > self.maximum_tokens
        {
            return Err(ContextError::InvalidResult);
        }
        if self.maximum_chunks.is_some_and(|m| self.included.len() > m)
            || self.content.len() != self.included.len()
        {
            return Err(ContextError::InvalidResult);
        }
        for (c, i) in self.content.iter().zip(&self.included) {
            c.validate()?;
            if c.chunk_id != i.chunk_id
                || c.final_context_position != i.final_context_position
                || i.content_fingerprint.verify(c.content.as_bytes()).is_err()
            {
                return Err(ContextError::InvalidResult);
            }
        }
        Ok(())
    }
}
wire!(ContextPackage{contract_version:ProtocolVersion,context_package_id:ContextPackageId,hybrid_result_id:HybridRetrievalResultId,query_id:RetrievalQueryId,assembly_policy_version:ProtocolVersion,governance_policy_version:ProtocolVersion,tokenizer_profile_id:String,tokenizer_fingerprint:ProfileFingerprint,maximum_tokens:u64,maximum_chunk_tokens:Option<u64>,maximum_chunks:Option<usize>,ordering_policy:OrderingPolicy,packing_policy:PackingPolicy,accounting:TokenAccountingPolicy,hybrid_candidate_count:usize,used_tokens:u64,remaining_tokens:u64,included:Vec<IncludedChunk>,exclusions:Vec<ContextExclusion>,content:Vec<ContextContent>});

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ContextError {
    #[error("unsupported context contract or policy")]
    UnsupportedContract,
    #[error("context limit violates bounds")]
    InvalidLimit,
    #[error("context identity or provenance mismatch")]
    ProvenanceMismatch,
    #[error("invalid governed chunk material")]
    InvalidMaterial,
    #[error("chunk integrity validation failed")]
    IntegrityMismatch,
    #[error("tokenizer provenance mismatch")]
    TokenizerMismatch,
    #[error("duplicate or conflicting chunk identity")]
    IdentityConflict,
    #[error("checked token arithmetic overflow")]
    ArithmeticOverflow,
    #[error("context package is invalid")]
    InvalidResult,
}

pub fn assemble_context(
    request: &ContextAssemblyRequest,
    hybrid: &HybridRetrievalResult,
    materials: Vec<GovernedChunkMaterial>,
) -> Result<ContextPackage, ContextError> {
    request.validate()?;
    hybrid.validate().map_err(|_| ContextError::InvalidResult)?;
    if hybrid.result_id != request.hybrid_result_id || hybrid.query_id != request.query_id {
        return Err(ContextError::ProvenanceMismatch);
    }
    let candidates: BTreeMap<_, _> = hybrid.candidates.iter().map(|x| (x.chunk_id, x)).collect();
    let excluded: BTreeSet<_> = hybrid.exclusions.iter().map(|x| x.chunk_id).collect();
    let mut supplied = BTreeMap::new();
    for m in materials {
        m.validate()?;
        if excluded.contains(&m.chunk_id) || !candidates.contains_key(&m.chunk_id) {
            return Err(ContextError::ProvenanceMismatch);
        }
        let c = candidates[&m.chunk_id];
        if (m.artifact_id, m.source_id, m.source_version)
            != (c.artifact_id, c.source_id, c.source_version)
        {
            return Err(ContextError::ProvenanceMismatch);
        }
        if m.tokenizer_profile_id != request.tokenizer_profile_id
            || m.tokenizer_fingerprint != request.tokenizer_fingerprint
        {
            return Err(ContextError::TokenizerMismatch);
        }
        if supplied.insert(m.chunk_id, m).is_some() {
            return Err(ContextError::IdentityConflict);
        }
    }
    let overhead = request.accounting.per_chunk_contribution(0)?;
    let mut used = request.accounting.fixed_package_overhead;
    let mut included = Vec::new();
    let mut exclusions = Vec::new();
    let mut content = Vec::new();
    for c in &hybrid.candidates {
        let rank = c.reranking.final_rank;
        let Some(m) = supplied.remove(&c.chunk_id) else {
            exclusions.push(ex(c, rank, ContextExclusionReason::MissingGovernedMaterial));
            continue;
        };
        let reason = if request
            .maximum_chunk_tokens
            .is_some_and(|x| m.exact_token_count > x)
        {
            Some(ContextExclusionReason::PerChunkLimit)
        } else if request.maximum_chunks.is_some_and(|x| included.len() >= x) {
            Some(ContextExclusionReason::ChunkCountLimit)
        } else {
            let contribution = request
                .accounting
                .per_chunk_contribution(m.exact_token_count)?;
            if used
                .checked_add(contribution)
                .ok_or(ContextError::ArithmeticOverflow)?
                > request.maximum_tokens
            {
                Some(ContextExclusionReason::TokenBudget)
            } else {
                let pos = u32::try_from(included.len() + 1)
                    .map_err(|_| ContextError::ArithmeticOverflow)?;
                used = used
                    .checked_add(contribution)
                    .ok_or(ContextError::ArithmeticOverflow)?;
                included.push(IncludedChunk {
                    chunk_id: c.chunk_id,
                    artifact_id: c.artifact_id,
                    source_id: c.source_id,
                    source_version: c.source_version,
                    hybrid_rank: rank,
                    content_fingerprint: m.content_fingerprint.clone(),
                    exact_token_count: m.exact_token_count,
                    overhead_tokens: overhead,
                    total_contribution: contribution,
                    final_context_position: pos,
                });
                content.push(ContextContent {
                    chunk_id: c.chunk_id,
                    final_context_position: pos,
                    content: m.content,
                });
                None
            }
        };
        if let Some(r) = reason {
            exclusions.push(ex(c, rank, r))
        }
    }
    for x in &hybrid.exclusions {
        exclusions.push(ContextExclusion {
            chunk_id: x.chunk_id,
            artifact_id: x.artifact_id,
            source_id: x.source_id,
            source_version: x.source_version,
            hybrid_rank: None,
            reason: if x.reason == HybridExclusionReason::Governance {
                ContextExclusionReason::UpstreamGovernance
            } else {
                ContextExclusionReason::UpstreamExclusion
            },
            upstream_reason: Some(x.reason),
        })
    }
    let out = ContextPackage {
        contract_version: V1,
        context_package_id: request.context_package_id,
        hybrid_result_id: request.hybrid_result_id,
        query_id: request.query_id,
        assembly_policy_version: request.assembly_policy_version,
        governance_policy_version: request.governance_policy_version,
        tokenizer_profile_id: request.tokenizer_profile_id.clone(),
        tokenizer_fingerprint: request.tokenizer_fingerprint.clone(),
        maximum_tokens: request.maximum_tokens,
        maximum_chunk_tokens: request.maximum_chunk_tokens,
        maximum_chunks: request.maximum_chunks,
        ordering_policy: request.ordering_policy,
        packing_policy: request.packing_policy,
        accounting: request.accounting.clone(),
        hybrid_candidate_count: hybrid.candidates.len(),
        used_tokens: used,
        remaining_tokens: request.maximum_tokens - used,
        included,
        exclusions,
        content,
    };
    out.validate()?;
    Ok(out)
}
fn ex(c: &crate::HybridCandidate, rank: u32, reason: ContextExclusionReason) -> ContextExclusion {
    ContextExclusion {
        chunk_id: c.chunk_id,
        artifact_id: c.artifact_id,
        source_id: c.source_id,
        source_version: c.source_version,
        hybrid_rank: Some(rank),
        reason,
        upstream_reason: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    const Q: &str = "00000000-0000-0000-0000-000000000001";
    const H: &str = "00000000-0000-0000-0000-000000000002";
    const P: &str = "00000000-0000-0000-0000-000000000003";
    const L: &str = "00000000-0000-0000-0000-000000000004";
    const V: &str = "00000000-0000-0000-0000-000000000005";
    const C: &str = "00000000-0000-0000-0000-000000000006";
    const A: &str = "00000000-0000-0000-0000-000000000007";
    const S: &str = "00000000-0000-0000-0000-000000000008";
    const F: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    fn id(n: u8) -> String {
        format!("00000000-0000-0000-0000-{n:012}")
    }
    fn hybrid(n: usize, upstream: bool) -> HybridRetrievalResult {
        let candidates=(0..n).map(|i|json!({"chunk_id":id(20+i as u8),"artifact_id":A,"source_id":S,"source_version":1,"participation":"lexical_only","lexical":{"result_id":L,"rank":i+1,"score":1.0},"vector":null,"score":{"numerator":1,"denominator":i+1},"reranking":{"policy_version":"1.0","final_rank":i+1,"rationale":"exact_fusion_then_canonical_chunk_identity"}})).collect::<Vec<_>>();
        let exclusions = if upstream {
            vec![
                json!({"chunk_id":id(99),"artifact_id":A,"source_id":S,"source_version":1,"reason":"governance","lexical_reason":"not_active","vector_reason":null}),
            ]
        } else {
            vec![]
        };
        serde_json::from_value(json!({"contract_version":"1.0","fusion_policy_version":"1.0","reranking_policy_version":"1.0","query_id":Q,"lexical_result_id":L,"vector_result_id":V,"lexical_policy_version":"1.0","vector_policy_version":"1.0","result_id":H,"profile_id":P,"profile_fingerprint":F,"dimension":1,"metric":"dot_product","maximum_results":100,"policy":{"contract_version":"1.0","fusion_policy_version":"1.0","reranking_policy_version":"1.0","channels":{"lexical_weight":1,"vector_weight":1,"rank_offset":0}},"candidates":candidates,"exclusions":exclusions})).unwrap()
    }
    fn request(budget: u64) -> ContextAssemblyRequest {
        serde_json::from_value(json!({"contract_version":"1.0","context_package_id":C,"hybrid_result_id":H,"query_id":Q,"assembly_policy_version":"1.0","governance_policy_version":"1.0","tokenizer_profile_id":"exact-test","tokenizer_fingerprint":F,"maximum_tokens":budget,"maximum_chunk_tokens":null,"maximum_chunks":null,"ordering_policy":"hybrid_rank","packing_policy":"ranked_greedy_whole_chunk","accounting":{"policy_version":"1.0","fixed_package_overhead":2,"per_chunk_overhead":1,"separator_tokens":1,"metadata_reference_overhead":1}})).unwrap()
    }
    fn material(i: u8, tokens: u64, text: &str) -> GovernedChunkMaterial {
        GovernedChunkMaterial {
            chunk_id: id(20 + i).parse().unwrap(),
            artifact_id: A.parse().unwrap(),
            source_id: S.parse().unwrap(),
            source_version: 1,
            content_fingerprint: ContentHash::sha256(text.as_bytes()),
            tokenizer_profile_id: "exact-test".into(),
            tokenizer_fingerprint: serde_json::from_value(json!(F)).unwrap(),
            exact_token_count: tokens,
            content: text.into(),
        }
    }
    #[test]
    fn exact_fit_skip_and_insertion_order() {
        let h = hybrid(3, false);
        let a = assemble_context(
            &request(12),
            &h,
            vec![
                material(2, 2, "c"),
                material(0, 8, "a"),
                material(1, 2, "b"),
            ],
        )
        .unwrap();
        assert_eq!((a.used_tokens, a.remaining_tokens), (12, 0));
        assert_eq!(a.included[0].hybrid_rank, 2);
        assert_eq!(
            a.exclusions
                .iter()
                .filter(|x| x.reason == ContextExclusionReason::TokenBudget)
                .count(),
            1
        );
        let b = assemble_context(
            &request(12),
            &h,
            vec![
                material(1, 2, "b"),
                material(0, 8, "a"),
                material(2, 2, "c"),
            ],
        )
        .unwrap();
        assert_eq!(a, b)
    }
    #[test]
    fn limits_missing_empty_and_upstream() {
        let h = hybrid(3, true);
        let mut r = request(100);
        r.maximum_chunk_tokens = Some(3);
        r.maximum_chunks = Some(1);
        let a = assemble_context(&r, &h, vec![material(0, 4, "a"), material(1, 2, "b")]).unwrap();
        assert_eq!(
            a.exclusions.iter().map(|x| x.reason).collect::<Vec<_>>(),
            vec![
                ContextExclusionReason::PerChunkLimit,
                ContextExclusionReason::MissingGovernedMaterial,
                ContextExclusionReason::UpstreamGovernance
            ]
        );
        assert_eq!(a.included[0].hybrid_rank, 2);
        let empty = assemble_context(&request(9), &hybrid(0, false), vec![]).unwrap();
        assert_eq!((empty.used_tokens, empty.remaining_tokens), (2, 7))
    }
    #[test]
    fn one_over_conflicts_and_redaction() {
        let h = hybrid(1, false);
        let a = assemble_context(&request(9), &h, vec![material(0, 5, "highly-secret")]).unwrap();
        assert!(a.included.is_empty());
        let m = material(0, 1, "highly-secret");
        assert_eq!(
            assemble_context(&request(20), &h, vec![m.clone(), m.clone()]).unwrap_err(),
            ContextError::IdentityConflict
        );
        let mut bad = m.clone();
        bad.content = "altered".into();
        assert_eq!(
            assemble_context(&request(20), &h, vec![bad]).unwrap_err(),
            ContextError::IntegrityMismatch
        );
        let mut profile = m.clone();
        profile.tokenizer_profile_id = "other".into();
        assert_eq!(
            assemble_context(&request(20), &h, vec![profile]).unwrap_err(),
            ContextError::TokenizerMismatch
        );
        let mut provenance = m;
        provenance.source_version = 2;
        assert_eq!(
            assemble_context(&request(20), &h, vec![provenance]).unwrap_err(),
            ContextError::ProvenanceMismatch
        );
        assert!(!format!("{a:?}").contains("highly-secret"))
    }
    #[test]
    fn replay_tamper_unknown_and_overflow() {
        let a = assemble_context(
            &request(20),
            &hybrid(1, false),
            vec![material(0, 1, "secret")],
        )
        .unwrap();
        let encoded = serde_json::to_string(&a).unwrap();
        assert_eq!(a, serde_json::from_str(&encoded).unwrap());
        for (path, value) in [("position", json!(2)), ("used", json!(999))] {
            let mut v: Value = serde_json::from_str(&encoded).unwrap();
            if path == "position" {
                v["included"][0]["final_context_position"] = value
            } else {
                v["used_tokens"] = value
            }
            assert!(serde_json::from_value::<ContextPackage>(v).is_err())
        }
        let mut v: Value = serde_json::from_str(&encoded).unwrap();
        v["unknown"] = json!(true);
        assert!(serde_json::from_value::<ContextPackage>(v).is_err());
        let mut r = request(20);
        r.accounting.per_chunk_overhead = u64::MAX;
        assert_eq!(r.validate().unwrap_err(), ContextError::ArithmeticOverflow)
    }
}
