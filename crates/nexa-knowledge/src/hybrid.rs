//! Exact, provider-free fusion of already governed lexical and vector results.
use crate::{
    ProfileFingerprint, RetrievalExclusionReason, RetrievalResult, RetrievalScore,
    VectorExclusionReason, VectorMetric, VectorRetrievalResult, LEXICAL_RETRIEVAL_V1,
    MAX_RETRIEVAL_RESULTS, MAX_VECTOR_DIMENSION, V1, VECTOR_RETRIEVAL_V1,
};
use nexa_domain::{
    EmbeddingProfileId, EmbeddingRecordId, HybridRetrievalResultId, KnowledgeArtifactId,
    KnowledgeChunkId, KnowledgeSourceId, ProtocolVersion, RetrievalQueryId, RetrievalResultId,
};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    fmt,
};
use thiserror::Error;

pub const HYBRID_FUSION_V1: ProtocolVersion = V1;
pub const HYBRID_RERANK_V1: ProtocolVersion = V1;
pub const MAX_CHANNEL_WEIGHT: u32 = 1_000;
pub const MAX_RANK_OFFSET: u32 = 1_000;
pub const MAX_HYBRID_RESULTS: usize = 100;

macro_rules! wire {($name:ident{$($field:ident:$ty:ty),*$(,)?})=>{impl<'de> Deserialize<'de> for $name {fn deserialize<D:serde::Deserializer<'de>>(d:D)->Result<Self,D::Error>{#[derive(Deserialize)]#[serde(deny_unknown_fields)]struct W{$($field:$ty),*}let w=W::deserialize(d)?;let x=Self{$($field:w.$field),*};x.validate().map_err(serde::de::Error::custom)?;Ok(x)}}};}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChannelConfiguration {
    pub lexical_weight: u32,
    pub vector_weight: u32,
    pub rank_offset: u32,
}
impl ChannelConfiguration {
    pub fn new(
        lexical_weight: u32,
        vector_weight: u32,
        rank_offset: u32,
    ) -> Result<Self, HybridError> {
        let x = Self {
            lexical_weight,
            vector_weight,
            rank_offset,
        };
        x.validate()?;
        Ok(x)
    }
    fn validate(&self) -> Result<(), HybridError> {
        if self.lexical_weight == 0
            || self.vector_weight == 0
            || self.lexical_weight > MAX_CHANNEL_WEIGHT
            || self.vector_weight > MAX_CHANNEL_WEIGHT
            || self.rank_offset > MAX_RANK_OFFSET
        {
            Err(HybridError::InvalidPolicy)
        } else {
            Ok(())
        }
    }
}
wire!(ChannelConfiguration {
    lexical_weight: u32,
    vector_weight: u32,
    rank_offset: u32
});

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HybridFusionPolicy {
    pub contract_version: ProtocolVersion,
    pub fusion_policy_version: ProtocolVersion,
    pub reranking_policy_version: ProtocolVersion,
    pub channels: ChannelConfiguration,
}
impl HybridFusionPolicy {
    pub fn v1(channels: ChannelConfiguration) -> Self {
        Self {
            contract_version: V1,
            fusion_policy_version: HYBRID_FUSION_V1,
            reranking_policy_version: HYBRID_RERANK_V1,
            channels,
        }
    }
    fn validate(&self) -> Result<(), HybridError> {
        if self.contract_version != V1
            || self.fusion_policy_version != HYBRID_FUSION_V1
            || self.reranking_policy_version != HYBRID_RERANK_V1
        {
            return Err(HybridError::UnsupportedContract);
        }
        self.channels.validate()
    }
}
wire!(HybridFusionPolicy {
    contract_version: ProtocolVersion,
    fusion_policy_version: ProtocolVersion,
    reranking_policy_version: ProtocolVersion,
    channels: ChannelConfiguration
});

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HybridFusionRequest {
    pub contract_version: ProtocolVersion,
    pub query_id: RetrievalQueryId,
    pub lexical_result_id: RetrievalResultId,
    pub vector_result_id: RetrievalResultId,
    pub hybrid_result_id: HybridRetrievalResultId,
    pub lexical_policy_version: ProtocolVersion,
    pub vector_policy_version: ProtocolVersion,
    pub profile_id: EmbeddingProfileId,
    pub profile_fingerprint: ProfileFingerprint,
    pub dimension: usize,
    pub metric: VectorMetric,
    pub maximum_results: usize,
    pub policy: HybridFusionPolicy,
}
impl HybridFusionRequest {
    pub fn validate(&self) -> Result<(), HybridError> {
        self.policy.validate()?;
        if self.contract_version != V1
            || self.lexical_policy_version != LEXICAL_RETRIEVAL_V1
            || self.vector_policy_version != VECTOR_RETRIEVAL_V1
        {
            return Err(HybridError::UnsupportedContract);
        }
        if self.dimension == 0
            || self.dimension > MAX_VECTOR_DIMENSION
            || self.metric != VectorMetric::DotProduct
        {
            return Err(HybridError::ProfileMismatch);
        }
        if self.maximum_results == 0 || self.maximum_results > MAX_HYBRID_RESULTS {
            return Err(HybridError::InvalidResultLimit);
        }
        Ok(())
    }
}
wire!(HybridFusionRequest {
    contract_version: ProtocolVersion,
    query_id: RetrievalQueryId,
    lexical_result_id: RetrievalResultId,
    vector_result_id: RetrievalResultId,
    hybrid_result_id: HybridRetrievalResultId,
    lexical_policy_version: ProtocolVersion,
    vector_policy_version: ProtocolVersion,
    profile_id: EmbeddingProfileId,
    profile_fingerprint: ProfileFingerprint,
    dimension: usize,
    metric: VectorMetric,
    maximum_results: usize,
    policy: HybridFusionPolicy
});

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelParticipation {
    LexicalOnly,
    VectorOnly,
    Both,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LexicalChannelEvidence {
    pub result_id: RetrievalResultId,
    pub rank: u32,
    pub score: RetrievalScore,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VectorChannelEvidence {
    pub result_id: RetrievalResultId,
    pub rank: u32,
    pub exact_dot_product: i64,
    pub embedding_id: EmbeddingRecordId,
    pub profile_id: EmbeddingProfileId,
    pub profile_fingerprint: ProfileFingerprint,
    pub dimension: usize,
    pub metric: VectorMetric,
}

impl LexicalChannelEvidence {
    fn validate(&self) -> Result<(), HybridError> {
        if self.rank == 0 || self.rank as usize > MAX_RETRIEVAL_RESULTS {
            Err(HybridError::InvalidResult)
        } else {
            Ok(())
        }
    }
}
wire!(LexicalChannelEvidence {
    result_id: RetrievalResultId,
    rank: u32,
    score: RetrievalScore
});
impl VectorChannelEvidence {
    fn validate(&self) -> Result<(), HybridError> {
        if self.rank == 0
            || self.rank as usize > MAX_RETRIEVAL_RESULTS
            || self.dimension == 0
            || self.dimension > MAX_VECTOR_DIMENSION
            || self.metric != VectorMetric::DotProduct
        {
            Err(HybridError::InvalidResult)
        } else {
            Ok(())
        }
    }
}
wire!(VectorChannelEvidence {
    result_id: RetrievalResultId,
    rank: u32,
    exact_dot_product: i64,
    embedding_id: EmbeddingRecordId,
    profile_id: EmbeddingProfileId,
    profile_fingerprint: ProfileFingerprint,
    dimension: usize,
    metric: VectorMetric
});

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HybridScore {
    numerator: u64,
    denominator: u64,
}
impl HybridScore {
    pub fn numerator(self) -> u64 {
        self.numerator
    }
    pub fn denominator(self) -> u64 {
        self.denominator
    }
    fn new(n: u64, d: u64) -> Result<Self, HybridError> {
        if n == 0 || d == 0 {
            return Err(HybridError::InvalidArithmetic);
        }
        let g = gcd(n, d);
        Ok(Self {
            numerator: n / g,
            denominator: d / g,
        })
    }
    fn compare(self, o: Self) -> Result<Ordering, HybridError> {
        Ok(self
            .numerator
            .checked_mul(o.denominator)
            .ok_or(HybridError::ArithmeticOverflow)?
            .cmp(
                &o.numerator
                    .checked_mul(self.denominator)
                    .ok_or(HybridError::ArithmeticOverflow)?,
            ))
    }
}
impl<'de> Deserialize<'de> for HybridScore {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct W {
            numerator: u64,
            denominator: u64,
        }
        let w = W::deserialize(d)?;
        let x = Self::new(w.numerator, w.denominator).map_err(serde::de::Error::custom)?;
        if (x.numerator, x.denominator) != (w.numerator, w.denominator) {
            return Err(serde::de::Error::custom(HybridError::InvalidArithmetic));
        }
        Ok(x)
    }
}
fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r
    }
    a
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RerankingRationale {
    ExactFusionThenCanonicalChunkIdentity,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RerankingEvidence {
    pub policy_version: ProtocolVersion,
    pub final_rank: u32,
    pub rationale: RerankingRationale,
}

impl RerankingEvidence {
    fn validate(&self) -> Result<(), HybridError> {
        if self.policy_version != HYBRID_RERANK_V1 || self.final_rank == 0 {
            Err(HybridError::InvalidResult)
        } else {
            Ok(())
        }
    }
}
wire!(RerankingEvidence {
    policy_version: ProtocolVersion,
    final_rank: u32,
    rationale: RerankingRationale
});
#[derive(Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HybridCandidate {
    pub chunk_id: KnowledgeChunkId,
    pub artifact_id: KnowledgeArtifactId,
    pub source_id: KnowledgeSourceId,
    pub source_version: u64,
    pub participation: ChannelParticipation,
    pub lexical: Option<LexicalChannelEvidence>,
    pub vector: Option<VectorChannelEvidence>,
    pub score: HybridScore,
    pub reranking: RerankingEvidence,
}
impl fmt::Debug for HybridCandidate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HybridCandidate")
            .field("chunk_id", &self.chunk_id)
            .field("score", &self.score)
            .field("reranking", &self.reranking)
            .finish()
    }
}
impl HybridCandidate {
    fn validate(&self) -> Result<(), HybridError> {
        if self.source_version == 0 {
            return Err(HybridError::InvalidResult);
        }
        if let Some(lexical) = &self.lexical {
            lexical.validate()?;
        }
        if let Some(vector) = &self.vector {
            vector.validate()?;
        }
        self.reranking.validate()?;
        let p = match (self.lexical.is_some(), self.vector.is_some()) {
            (true, true) => ChannelParticipation::Both,
            (true, false) => ChannelParticipation::LexicalOnly,
            (false, true) => ChannelParticipation::VectorOnly,
            _ => return Err(HybridError::InvalidResult),
        };
        if p != self.participation {
            return Err(HybridError::InvalidResult);
        }
        Ok(())
    }
}
wire!(HybridCandidate{chunk_id:KnowledgeChunkId,artifact_id:KnowledgeArtifactId,source_id:KnowledgeSourceId,source_version:u64,participation:ChannelParticipation,lexical:Option<LexicalChannelEvidence>,vector:Option<VectorChannelEvidence>,score:HybridScore,reranking:RerankingEvidence});

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HybridExclusionReason {
    Governance,
    ChannelAbsence,
    ResultLimit,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HybridExclusion {
    pub chunk_id: KnowledgeChunkId,
    pub artifact_id: KnowledgeArtifactId,
    pub source_id: KnowledgeSourceId,
    pub source_version: u64,
    pub reason: HybridExclusionReason,
    pub lexical_reason: Option<RetrievalExclusionReason>,
    pub vector_reason: Option<VectorExclusionReason>,
}

impl HybridExclusion {
    fn validate(&self) -> Result<(), HybridError> {
        if self.source_version == 0 {
            return Err(HybridError::InvalidResult);
        }
        match self.reason {
            HybridExclusionReason::Governance
                if (self.lexical_reason.is_none() && self.vector_reason.is_none())
                    || self.lexical_reason.is_some_and(|reason| !gov_l(reason))
                    || self.vector_reason.is_some_and(|reason| !gov_v(reason)) =>
            {
                Err(HybridError::InvalidResult)
            }
            HybridExclusionReason::ChannelAbsence
                if (self.lexical_reason.is_none() && self.vector_reason.is_none())
                    || self.lexical_reason.is_some_and(gov_l)
                    || self.vector_reason.is_some_and(gov_v) =>
            {
                Err(HybridError::InvalidResult)
            }
            HybridExclusionReason::ResultLimit
                if self.lexical_reason.is_some() || self.vector_reason.is_some() =>
            {
                Err(HybridError::InvalidResult)
            }
            _ => Ok(()),
        }
    }
}
wire!(HybridExclusion{chunk_id:KnowledgeChunkId,artifact_id:KnowledgeArtifactId,source_id:KnowledgeSourceId,source_version:u64,reason:HybridExclusionReason,lexical_reason:Option<RetrievalExclusionReason>,vector_reason:Option<VectorExclusionReason>});

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HybridRetrievalResult {
    pub contract_version: ProtocolVersion,
    pub fusion_policy_version: ProtocolVersion,
    pub reranking_policy_version: ProtocolVersion,
    pub query_id: RetrievalQueryId,
    pub lexical_result_id: RetrievalResultId,
    pub vector_result_id: RetrievalResultId,
    pub lexical_policy_version: ProtocolVersion,
    pub vector_policy_version: ProtocolVersion,
    pub result_id: HybridRetrievalResultId,
    pub profile_id: EmbeddingProfileId,
    pub profile_fingerprint: ProfileFingerprint,
    pub dimension: usize,
    pub metric: VectorMetric,
    pub maximum_results: usize,
    pub policy: HybridFusionPolicy,
    pub candidates: Vec<HybridCandidate>,
    pub exclusions: Vec<HybridExclusion>,
}
impl HybridRetrievalResult {
    fn validate(&self) -> Result<(), HybridError> {
        if self.contract_version != V1
            || self.fusion_policy_version != HYBRID_FUSION_V1
            || self.reranking_policy_version != HYBRID_RERANK_V1
            || self.lexical_policy_version != LEXICAL_RETRIEVAL_V1
            || self.vector_policy_version != VECTOR_RETRIEVAL_V1
            || self.dimension == 0
            || self.dimension > MAX_VECTOR_DIMENSION
            || self.metric != VectorMetric::DotProduct
            || self.policy.validate().is_err()
            || self.fusion_policy_version != self.policy.fusion_policy_version
            || self.reranking_policy_version != self.policy.reranking_policy_version
            || self.maximum_results == 0
            || self.maximum_results > MAX_HYBRID_RESULTS
            || self.candidates.len() > self.maximum_results
        {
            return Err(HybridError::UnsupportedContract);
        }
        let mut ids = BTreeSet::new();
        for (i, c) in self.candidates.iter().enumerate() {
            c.validate()?;
            if c.lexical
                .as_ref()
                .is_some_and(|x| x.result_id != self.lexical_result_id)
                || c.vector.as_ref().is_some_and(|x| {
                    x.result_id != self.vector_result_id
                        || x.profile_id != self.profile_id
                        || x.profile_fingerprint != self.profile_fingerprint
                        || x.dimension != self.dimension
                        || x.metric != self.metric
                })
                || fusion_score(
                    &self.policy.channels,
                    c.lexical.as_ref().map(|x| x.rank),
                    c.vector.as_ref().map(|x| x.rank),
                )? != c.score
            {
                return Err(HybridError::InvalidResult);
            }
            if !ids.insert(c.chunk_id)
                || c.reranking.final_rank
                    != u32::try_from(i + 1).map_err(|_| HybridError::InvalidResult)?
            {
                return Err(HybridError::InvalidResult);
            }
            if i > 0 {
                let p = &self.candidates[i - 1];
                let o = p.score.compare(c.score)?;
                if o == Ordering::Less || (o == Ordering::Equal && p.chunk_id >= c.chunk_id) {
                    return Err(HybridError::InvalidResult);
                }
            }
        }
        let mut ex = BTreeSet::new();
        for e in &self.exclusions {
            if e.validate().is_err() || !ex.insert(e.chunk_id) || ids.contains(&e.chunk_id) {
                return Err(HybridError::InvalidResult);
            }
        }
        Ok(())
    }
}
wire!(HybridRetrievalResult{contract_version:ProtocolVersion,fusion_policy_version:ProtocolVersion,reranking_policy_version:ProtocolVersion,query_id:RetrievalQueryId,lexical_result_id:RetrievalResultId,vector_result_id:RetrievalResultId,lexical_policy_version:ProtocolVersion,vector_policy_version:ProtocolVersion,result_id:HybridRetrievalResultId,profile_id:EmbeddingProfileId,profile_fingerprint:ProfileFingerprint,dimension:usize,metric:VectorMetric,maximum_results:usize,policy:HybridFusionPolicy,candidates:Vec<HybridCandidate>,exclusions:Vec<HybridExclusion>});

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum HybridError {
    #[error("unsupported hybrid contract or policy")]
    UnsupportedContract,
    #[error("hybrid policy violates bounds")]
    InvalidPolicy,
    #[error("hybrid result limit violates bounds")]
    InvalidResultLimit,
    #[error("channel identity or provenance mismatch")]
    ProvenanceMismatch,
    #[error("vector profile provenance mismatch")]
    ProfileMismatch,
    #[error("channel records conflict")]
    ChannelConflict,
    #[error("invalid exact fusion arithmetic")]
    InvalidArithmetic,
    #[error("checked hybrid arithmetic overflow")]
    ArithmeticOverflow,
    #[error("hybrid result is invalid")]
    InvalidResult,
}

type Reference = (KnowledgeArtifactId, KnowledgeSourceId, u64);
fn gov_l(x: RetrievalExclusionReason) -> bool {
    matches!(
        x,
        RetrievalExclusionReason::NotActive
            | RetrievalExclusionReason::AssessmentProtected
            | RetrievalExclusionReason::AudienceRestricted
            | RetrievalExclusionReason::CourseScopeMismatch
            | RetrievalExclusionReason::LessonScopeMismatch
    )
}
fn gov_v(x: VectorExclusionReason) -> bool {
    matches!(
        x,
        VectorExclusionReason::NotActive
            | VectorExclusionReason::AssessmentProtected
            | VectorExclusionReason::AudienceRestricted
            | VectorExclusionReason::CourseScopeMismatch
            | VectorExclusionReason::LessonScopeMismatch
    )
}

pub fn fuse(
    request: &HybridFusionRequest,
    lexical: &RetrievalResult,
    vector: &VectorRetrievalResult,
) -> Result<HybridRetrievalResult, HybridError> {
    request.validate()?;
    lexical
        .validate()
        .map_err(|_| HybridError::ChannelConflict)?;
    vector
        .validate()
        .map_err(|_| HybridError::ChannelConflict)?;
    if lexical.query_id != request.query_id
        || vector.query_id != request.query_id
        || lexical.result_id != request.lexical_result_id
        || vector.result_id != request.vector_result_id
        || lexical.retrieval_policy_version != request.lexical_policy_version
        || vector.vector_policy_version != request.vector_policy_version
    {
        return Err(HybridError::ProvenanceMismatch);
    }
    if vector.profile_id != request.profile_id
        || vector.profile_fingerprint != request.profile_fingerprint
        || vector.dimension != request.dimension
        || vector.metric != request.metric
    {
        return Err(HybridError::ProfileMismatch);
    }
    let mut refs: BTreeMap<KnowledgeChunkId, Reference> = BTreeMap::new();
    let mut lc = BTreeMap::new();
    let mut vc = BTreeMap::new();
    let mut le = BTreeMap::new();
    let mut ve = BTreeMap::new();
    macro_rules! bind {
        ($x:expr) => {{
            let r = ($x.artifact_id, $x.source_id, $x.source_version);
            if refs.insert($x.chunk_id, r).is_some_and(|old| old != r) {
                return Err(HybridError::ChannelConflict);
            }
        }};
    }
    for (i, x) in lexical.candidates.iter().enumerate() {
        bind!(x);
        lc.insert(x.chunk_id, (i, x));
    }
    for x in &lexical.exclusions {
        bind!(x);
        le.insert(x.chunk_id, x.reason);
    }
    for (i, x) in vector.candidates.iter().enumerate() {
        bind!(x);
        vc.insert(x.chunk_id, (i, x));
    }
    for x in &vector.exclusions {
        bind!(x);
        ve.insert(x.chunk_id, x.reason);
    }
    let mut candidates = Vec::new();
    let mut exclusions = Vec::new();
    for (id, r) in refs {
        let l = lc.get(&id);
        let v = vc.get(&id);
        let lr = le.get(&id).copied();
        let vr = ve.get(&id).copied();
        if (l.is_some() && lr.is_some())
            || (v.is_some() && vr.is_some())
            || (l.is_some() && vr.is_some_and(gov_v))
            || (v.is_some() && lr.is_some_and(gov_l))
        {
            return Err(HybridError::ChannelConflict);
        }
        if lr.is_some_and(gov_l) || vr.is_some_and(gov_v) {
            exclusions.push(HybridExclusion {
                chunk_id: id,
                artifact_id: r.0,
                source_id: r.1,
                source_version: r.2,
                reason: HybridExclusionReason::Governance,
                lexical_reason: lr,
                vector_reason: vr,
            });
            continue;
        }
        if l.is_none() && v.is_none() {
            exclusions.push(HybridExclusion {
                chunk_id: id,
                artifact_id: r.0,
                source_id: r.1,
                source_version: r.2,
                reason: HybridExclusionReason::ChannelAbsence,
                lexical_reason: lr,
                vector_reason: vr,
            });
            continue;
        }
        let lex = l.map(|(i, x)| LexicalChannelEvidence {
            result_id: lexical.result_id,
            rank: (i + 1) as u32,
            score: x.score,
        });
        let vec = v.map(|(i, x)| VectorChannelEvidence {
            result_id: vector.result_id,
            rank: (i + 1) as u32,
            exact_dot_product: x.evidence.exact_dot_product,
            embedding_id: x.embedding_id,
            profile_id: x.profile_id,
            profile_fingerprint: x.profile_fingerprint.clone(),
            dimension: x.dimension,
            metric: x.evidence.metric,
        });
        let score = fusion_score(
            &request.policy.channels,
            lex.as_ref().map(|x| x.rank),
            vec.as_ref().map(|x| x.rank),
        )?;
        let participation = match (lex.is_some(), vec.is_some()) {
            (true, true) => ChannelParticipation::Both,
            (true, false) => ChannelParticipation::LexicalOnly,
            (false, true) => ChannelParticipation::VectorOnly,
            _ => unreachable!(),
        };
        candidates.push(HybridCandidate {
            chunk_id: id,
            artifact_id: r.0,
            source_id: r.1,
            source_version: r.2,
            participation,
            lexical: lex,
            vector: vec,
            score,
            reranking: RerankingEvidence {
                policy_version: HYBRID_RERANK_V1,
                final_rank: 1,
                rationale: RerankingRationale::ExactFusionThenCanonicalChunkIdentity,
            },
        })
    }
    candidates.sort_by(|a, b| {
        b.score
            .compare(a.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.chunk_id.cmp(&b.chunk_id))
    });
    let dropped = candidates.split_off(request.maximum_results.min(candidates.len()));
    for x in dropped {
        exclusions.push(HybridExclusion {
            chunk_id: x.chunk_id,
            artifact_id: x.artifact_id,
            source_id: x.source_id,
            source_version: x.source_version,
            reason: HybridExclusionReason::ResultLimit,
            lexical_reason: None,
            vector_reason: None,
        })
    }
    for (i, x) in candidates.iter_mut().enumerate() {
        x.reranking.final_rank = u32::try_from(i + 1).map_err(|_| HybridError::InvalidResult)?
    }
    exclusions.sort_by_key(|x| x.chunk_id);
    let out = HybridRetrievalResult {
        contract_version: V1,
        fusion_policy_version: request.policy.fusion_policy_version,
        reranking_policy_version: request.policy.reranking_policy_version,
        query_id: request.query_id,
        lexical_result_id: lexical.result_id,
        vector_result_id: vector.result_id,
        lexical_policy_version: lexical.retrieval_policy_version,
        vector_policy_version: vector.vector_policy_version,
        result_id: request.hybrid_result_id,
        profile_id: request.profile_id,
        profile_fingerprint: request.profile_fingerprint.clone(),
        dimension: request.dimension,
        metric: request.metric,
        maximum_results: request.maximum_results,
        policy: request.policy.clone(),
        candidates,
        exclusions,
    };
    out.validate()?;
    Ok(out)
}
fn fusion_score(
    p: &ChannelConfiguration,
    l: Option<u32>,
    v: Option<u32>,
) -> Result<HybridScore, HybridError> {
    let term = |w: u32, r: u32| -> Result<(u64, u64), HybridError> {
        if r == 0 || r as usize > MAX_RETRIEVAL_RESULTS {
            return Err(HybridError::InvalidArithmetic);
        }
        Ok((
            u64::from(w),
            u64::from(p.rank_offset)
                .checked_add(u64::from(r))
                .ok_or(HybridError::ArithmeticOverflow)?,
        ))
    };
    match (l, v) {
        (Some(a), Some(b)) => {
            let (x, xd) = term(p.lexical_weight, a)?;
            let (y, yd) = term(p.vector_weight, b)?;
            HybridScore::new(
                x.checked_mul(yd)
                    .and_then(|n| y.checked_mul(xd).and_then(|m| n.checked_add(m)))
                    .ok_or(HybridError::ArithmeticOverflow)?,
                xd.checked_mul(yd).ok_or(HybridError::ArithmeticOverflow)?,
            )
        }
        (Some(a), None) => {
            let (n, d) = term(p.lexical_weight, a)?;
            HybridScore::new(n, d)
        }
        (None, Some(b)) => {
            let (n, d) = term(p.vector_weight, b)?;
            HybridScore::new(n, d)
        }
        _ => Err(HybridError::InvalidArithmetic),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ContentHash, EmbeddingProfile, SimilarityEvidence, TermScoreEvidence,
        VectorRetrievalCandidate,
    };
    use std::{collections::BTreeMap, str::FromStr};
    fn id<T: FromStr>(s: &str) -> T
    where
        T::Err: fmt::Debug,
    {
        s.parse().unwrap()
    }
    fn profile() -> EmbeddingProfile {
        EmbeddingProfile::new(
            id("018f0000-0000-7000-b000-000000000001"),
            "inert.family",
            3,
            BTreeMap::new(),
        )
        .unwrap()
    }
    fn lexical(q: RetrievalQueryId, r: RetrievalResultId, chunks: &[u64]) -> RetrievalResult {
        RetrievalResult {
            contract_version: V1,
            retrieval_policy_version: V1,
            query_id: q,
            result_id: r,
            candidates: chunks
                .iter()
                .map(|n| {
                    let e =
                        TermScoreEvidence::new(ContentHash::sha256(b"term"), 1, 1, 1, 1).unwrap();
                    crate::RetrievalCandidate::new(
                        id(&format!("018f0000-0000-7000-a000-{n:012x}")),
                        id(&format!("018f0000-0000-7000-9000-{n:012x}")),
                        id(&format!("018f0000-0000-7000-8000-{n:012x}")),
                        1,
                        RetrievalScore::new(1).unwrap(),
                        vec![e],
                    )
                    .unwrap()
                })
                .collect(),
            exclusions: vec![],
        }
    }
    fn vector(
        q: RetrievalQueryId,
        r: RetrievalResultId,
        p: &EmbeddingProfile,
        chunks: &[u64],
    ) -> VectorRetrievalResult {
        VectorRetrievalResult {
            contract_version: V1,
            vector_policy_version: V1,
            query_id: q,
            result_id: r,
            profile_id: p.profile_id,
            profile_fingerprint: p.fingerprint.clone(),
            dimension: 3,
            metric: VectorMetric::DotProduct,
            candidates: chunks
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    let eid = id(&format!("018f0000-0000-7000-c000-{n:012x}"));
                    VectorRetrievalCandidate {
                        embedding_id: eid,
                        profile_id: p.profile_id,
                        profile_fingerprint: p.fingerprint.clone(),
                        dimension: 3,
                        chunk_id: id(&format!("018f0000-0000-7000-a000-{n:012x}")),
                        artifact_id: id(&format!("018f0000-0000-7000-9000-{n:012x}")),
                        source_id: id(&format!("018f0000-0000-7000-8000-{n:012x}")),
                        source_version: 1,
                        evidence: SimilarityEvidence {
                            embedding_id: eid,
                            metric: VectorMetric::DotProduct,
                            exact_dot_product: 100 - i as i64,
                            accumulation_order: V1,
                        },
                    }
                })
                .collect(),
            exclusions: vec![],
        }
    }
    fn request(
        q: RetrievalQueryId,
        l: RetrievalResultId,
        v: RetrievalResultId,
        p: &EmbeddingProfile,
        max: usize,
    ) -> HybridFusionRequest {
        HybridFusionRequest {
            contract_version: V1,
            query_id: q,
            lexical_result_id: l,
            vector_result_id: v,
            hybrid_result_id: id("018f0000-0000-7000-d000-000000000001"),
            lexical_policy_version: V1,
            vector_policy_version: V1,
            profile_id: p.profile_id,
            profile_fingerprint: p.fingerprint.clone(),
            dimension: 3,
            metric: VectorMetric::DotProduct,
            maximum_results: max,
            policy: HybridFusionPolicy::v1(ChannelConfiguration::new(1, 1, 60).unwrap()),
        }
    }
    #[test]
    fn exact_fusion_and_participation_round_trip() {
        let q = id("018f0000-0000-7000-e000-000000000001");
        let l = id("018f0000-0000-7000-e000-000000000002");
        let v = id("018f0000-0000-7000-e000-000000000003");
        let p = profile();
        let out = fuse(
            &request(q, l, v, &p, 3),
            &lexical(q, l, &[1, 2]),
            &vector(q, v, &p, &[2, 3]),
        )
        .unwrap();
        assert_eq!(
            out.candidates
                .iter()
                .map(|x| x.participation)
                .collect::<Vec<_>>(),
            vec![
                ChannelParticipation::Both,
                ChannelParticipation::LexicalOnly,
                ChannelParticipation::VectorOnly
            ]
        );
        assert_eq!(
            (
                out.candidates[0].score.numerator(),
                out.candidates[0].score.denominator()
            ),
            (123, 3782)
        );
        let json = serde_json::to_string(&out).unwrap();
        assert!(!json.contains("inert.family") && !json.contains("query_text"));
        assert_eq!(
            serde_json::from_str::<HybridRetrievalResult>(&json).unwrap(),
            out
        )
    }
    #[test]
    fn validates_policy_provenance_limits_and_ties() {
        assert!(ChannelConfiguration::new(0, 1, 0).is_err());
        assert!(ChannelConfiguration::new(1, 1001, 0).is_err());
        let q = id("018f0000-0000-7000-e000-000000000001");
        let l = id("018f0000-0000-7000-e000-000000000002");
        let v = id("018f0000-0000-7000-e000-000000000003");
        let p = profile();
        let mut req = request(q, l, v, &p, 1);
        let out = fuse(&req, &lexical(q, l, &[1, 2]), &vector(q, v, &p, &[])).unwrap();
        assert_eq!(
            out.candidates[0].chunk_id,
            id("018f0000-0000-7000-a000-000000000001")
        );
        assert_eq!(out.exclusions[0].reason, HybridExclusionReason::ResultLimit);
        req.vector_result_id = l;
        assert_eq!(
            fuse(&req, &lexical(q, l, &[1]), &vector(q, v, &p, &[])),
            Err(HybridError::ProvenanceMismatch)
        );
    }
    #[test]
    fn malformed_wire_and_redaction() {
        let p = HybridFusionPolicy::v1(ChannelConfiguration::new(1, 1, 60).unwrap());
        let mut value = serde_json::to_value(p).unwrap();
        value["channels"]["lexical_weight"] = 0.into();
        assert!(serde_json::from_value::<HybridFusionPolicy>(value).is_err());
        let error = HybridError::ChannelConflict;
        assert!(!format!("{error:?} {error}").contains("018f"));
    }

    #[test]
    fn standalone_candidate_rejects_invalid_nested_evidence() {
        let q = id("018f0000-0000-7000-e000-000000000001");
        let l = id("018f0000-0000-7000-e000-000000000002");
        let v = id("018f0000-0000-7000-e000-000000000003");
        let p = profile();
        let out = fuse(
            &request(q, l, v, &p, 1),
            &lexical(q, l, &[1]),
            &vector(q, v, &p, &[1]),
        )
        .unwrap();
        let candidate = serde_json::to_value(&out.candidates[0]).unwrap();

        let mut invalid_dimension = candidate.clone();
        invalid_dimension["vector"]["dimension"] = 0.into();
        assert!(serde_json::from_value::<HybridCandidate>(invalid_dimension).is_err());

        let mut invalid_metric = candidate.clone();
        invalid_metric["vector"]["metric"] = serde_json::json!("cosine");
        assert!(serde_json::from_value::<HybridCandidate>(invalid_metric).is_err());

        let mut unsupported_reranking = candidate;
        unsupported_reranking["reranking"]["policy_version"] = 2.into();
        assert!(serde_json::from_value::<HybridCandidate>(unsupported_reranking).is_err());
    }

    #[test]
    fn result_rejects_limit_and_source_policy_tampering() {
        let q = id("018f0000-0000-7000-e000-000000000001");
        let l = id("018f0000-0000-7000-e000-000000000002");
        let v = id("018f0000-0000-7000-e000-000000000003");
        let p = profile();
        let channels = lexical(q, l, &[1, 2]);
        let vectors = vector(q, v, &p, &[]);
        let limited = fuse(&request(q, l, v, &p, 1), &channels, &vectors).unwrap();
        let complete = fuse(&request(q, l, v, &p, 2), &channels, &vectors).unwrap();

        let mut moved_exclusion = serde_json::to_value(&limited).unwrap();
        moved_exclusion["candidates"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::to_value(&complete.candidates[1]).unwrap());
        moved_exclusion["exclusions"] = serde_json::json!([]);
        assert!(serde_json::from_value::<HybridRetrievalResult>(moved_exclusion).is_err());

        for field in ["lexical_policy_version", "vector_policy_version"] {
            let mut tampered = serde_json::to_value(&limited).unwrap();
            tampered[field] = 2.into();
            assert!(serde_json::from_value::<HybridRetrievalResult>(tampered).is_err());
        }
    }

    #[test]
    fn exclusions_reject_governance_downgrades_and_mixed_classification() {
        let base = serde_json::json!({
            "chunk_id": "018f0000-0000-7000-a000-000000000001",
            "artifact_id": "018f0000-0000-7000-9000-000000000001",
            "source_id": "018f0000-0000-7000-8000-000000000001",
            "source_version": 1,
            "reason": "channel_absence",
            "lexical_reason": null,
            "vector_reason": null
        });
        let mut lexical_downgrade = base.clone();
        lexical_downgrade["lexical_reason"] = serde_json::json!("not_active");
        assert!(serde_json::from_value::<HybridExclusion>(lexical_downgrade).is_err());

        let mut vector_downgrade = base.clone();
        vector_downgrade["vector_reason"] = serde_json::json!("not_active");
        assert!(serde_json::from_value::<HybridExclusion>(vector_downgrade).is_err());

        let mut mixed_governance = base;
        mixed_governance["reason"] = serde_json::json!("governance");
        mixed_governance["lexical_reason"] = serde_json::json!("not_active");
        mixed_governance["vector_reason"] = serde_json::json!("missing_embedding");
        assert!(serde_json::from_value::<HybridExclusion>(mixed_governance).is_err());
    }
}
