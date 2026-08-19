//! Provider-neutral governed vector retrieval using exact signed-integer arithmetic.

use crate::{
    exposure, validate_chunks, Audience, ContentHash, KnowledgeArtifact, KnowledgeChunk,
    KnowledgeSource, SourceStatus, V1,
};
use nexa_domain::{
    CourseId, EmbeddingProfileId, EmbeddingRecordId, KnowledgeArtifactId, KnowledgeChunkId,
    KnowledgeSourceId, LessonId, ProtocolVersion, RetrievalQueryId, RetrievalResultId, Timestamp,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};
use thiserror::Error;

pub const EMBEDDING_CONTRACT_V1: ProtocolVersion = V1;
pub const VECTOR_RETRIEVAL_V1: ProtocolVersion = V1;
pub const MAX_VECTOR_DIMENSION: usize = 4096;
pub const MAX_VECTOR_RESULTS: usize = 100;
pub const MAX_PROFILE_METADATA_ENTRIES: usize = 32;
pub const MAX_PROFILE_FIELD_BYTES: usize = 255;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScalarRepresentation {
    SignedI16,
}
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VectorMetric {
    DotProduct,
}
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VectorNormalization {
    None,
}

/// Immutable SHA-256 fingerprint of every behavior-affecting profile field.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ProfileFingerprint(String);
impl ProfileFingerprint {
    fn calculate(p: &EmbeddingProfile) -> Self {
        let mut h = Sha256::new();
        // Length-prefix variable fields and use fixed-width big-endian integers.
        hash_field(&mut h, p.model_family.as_bytes());
        h.update((p.dimension as u32).to_be_bytes());
        h.update([0]); // signed_i16
        h.update([0]); // dot_product
        h.update([0]); // none
        h.update(p.contract_version.major().to_be_bytes());
        h.update(p.contract_version.minor().to_be_bytes());
        h.update((p.metadata.len() as u32).to_be_bytes());
        for (key, value) in &p.metadata {
            hash_field(&mut h, key.as_bytes());
            hash_field(&mut h, value.as_bytes());
        }
        Self(format!("{:x}", h.finalize()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
fn hash_field(h: &mut Sha256, value: &[u8]) {
    h.update((value.len() as u32).to_be_bytes());
    h.update(value);
}
impl<'de> Deserialize<'de> for ProfileFingerprint {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        if s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            Ok(Self(s))
        } else {
            Err(serde::de::Error::custom(VectorError::InvalidProfile))
        }
    }
}

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmbeddingProfile {
    pub contract_version: ProtocolVersion,
    pub profile_id: EmbeddingProfileId,
    pub model_family: String,
    pub dimension: usize,
    pub scalar_representation: ScalarRepresentation,
    pub metric: VectorMetric,
    pub normalization: VectorNormalization,
    pub metadata: BTreeMap<String, String>,
    pub fingerprint: ProfileFingerprint,
}
impl fmt::Debug for EmbeddingProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EmbeddingProfile")
            .field("profile_id", &self.profile_id)
            .field("model_family", &"[REDACTED]")
            .field("dimension", &self.dimension)
            .field("scalar_representation", &self.scalar_representation)
            .field("metric", &self.metric)
            .field("normalization", &self.normalization)
            .field("metadata", &"[REDACTED]")
            .field("fingerprint", &self.fingerprint)
            .finish()
    }
}

impl EmbeddingProfile {
    pub fn new(
        profile_id: EmbeddingProfileId,
        model_family: impl Into<String>,
        dimension: usize,
        metadata: BTreeMap<String, String>,
    ) -> Result<Self, VectorError> {
        let mut p = Self {
            contract_version: V1,
            profile_id,
            model_family: model_family.into(),
            dimension,
            scalar_representation: ScalarRepresentation::SignedI16,
            metric: VectorMetric::DotProduct,
            normalization: VectorNormalization::None,
            metadata,
            fingerprint: ProfileFingerprint(String::new()),
        };
        p.validate_fields()?;
        p.fingerprint = ProfileFingerprint::calculate(&p);
        Ok(p)
    }
    fn validate_fields(&self) -> Result<(), VectorError> {
        let identifier = |s: &str| {
            !s.is_empty()
                && s.len() <= MAX_PROFILE_FIELD_BYTES
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
        };
        if self.contract_version != EMBEDDING_CONTRACT_V1
            || self.dimension == 0
            || self.dimension > MAX_VECTOR_DIMENSION
            || !identifier(&self.model_family)
            || self.metadata.len() > MAX_PROFILE_METADATA_ENTRIES
            || self.metadata.iter().any(|(k, v)| {
                !identifier(k) || v.is_empty() || v.len() > MAX_PROFILE_FIELD_BYTES || !v.is_ascii()
            })
        {
            return Err(VectorError::InvalidProfile);
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), VectorError> {
        self.validate_fields()?;
        if self.fingerprint != ProfileFingerprint::calculate(self) {
            Err(VectorError::ProfileFingerprintMismatch)
        } else {
            Ok(())
        }
    }
}
impl<'de> Deserialize<'de> for EmbeddingProfile {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct W {
            contract_version: ProtocolVersion,
            profile_id: EmbeddingProfileId,
            model_family: String,
            dimension: usize,
            scalar_representation: ScalarRepresentation,
            metric: VectorMetric,
            normalization: VectorNormalization,
            metadata: BTreeMap<String, String>,
            fingerprint: ProfileFingerprint,
        }
        let w = W::deserialize(d)?;
        let p = Self {
            contract_version: w.contract_version,
            profile_id: w.profile_id,
            model_family: w.model_family,
            dimension: w.dimension,
            scalar_representation: w.scalar_representation,
            metric: w.metric,
            normalization: w.normalization,
            metadata: w.metadata,
            fingerprint: w.fingerprint,
        };
        p.validate().map_err(serde::de::Error::custom)?;
        Ok(p)
    }
}

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct EmbeddingVector(Vec<i16>);
impl EmbeddingVector {
    pub fn new(values: Vec<i16>, dimension: usize) -> Result<Self, VectorError> {
        if dimension == 0 || dimension > MAX_VECTOR_DIMENSION || values.len() != dimension {
            Err(VectorError::InvalidVector)
        } else {
            Ok(Self(values))
        }
    }
    fn values(&self) -> &[i16] {
        &self.0
    }
    pub fn dimension(&self) -> usize {
        self.0.len()
    }
}
impl fmt::Debug for EmbeddingVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("EmbeddingVector([REDACTED])")
    }
}
// Standalone decoding enforces global bounds; profile-specific equality is enforced by owners.
impl<'de> Deserialize<'de> for EmbeddingVector {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = Vec::<i16>::deserialize(d)?;
        if v.is_empty() || v.len() > MAX_VECTOR_DIMENSION {
            Err(serde::de::Error::custom(VectorError::InvalidVector))
        } else {
            Ok(Self(v))
        }
    }
}

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChunkEmbedding {
    pub contract_version: ProtocolVersion,
    pub embedding_id: EmbeddingRecordId,
    pub chunk_id: KnowledgeChunkId,
    pub artifact_id: KnowledgeArtifactId,
    pub source_id: KnowledgeSourceId,
    pub source_version: u64,
    pub original_artifact_hash: ContentHash,
    pub chunk_content_hash: ContentHash,
    pub profile_id: EmbeddingProfileId,
    pub profile_fingerprint: ProfileFingerprint,
    pub dimension: usize,
    vector: EmbeddingVector,
    pub created_at: Timestamp,
}
impl fmt::Debug for ChunkEmbedding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChunkEmbedding")
            .field("embedding_id", &self.embedding_id)
            .field("chunk_id", &self.chunk_id)
            .field("vector", &"[REDACTED]")
            .finish()
    }
}
impl ChunkEmbedding {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        embedding_id: EmbeddingRecordId,
        chunk: &KnowledgeChunk,
        profile: &EmbeddingProfile,
        vector: EmbeddingVector,
        created_at: Timestamp,
    ) -> Result<Self, VectorError> {
        profile.validate()?;
        if vector.dimension() != profile.dimension {
            return Err(VectorError::InvalidVector);
        };
        Ok(Self {
            contract_version: V1,
            embedding_id,
            chunk_id: chunk.chunk_id,
            artifact_id: chunk.artifact_id,
            source_id: chunk.source_id,
            source_version: chunk.source_version,
            original_artifact_hash: chunk.original_content_hash.clone(),
            chunk_content_hash: chunk.chunk_content_hash.clone(),
            profile_id: profile.profile_id,
            profile_fingerprint: profile.fingerprint.clone(),
            dimension: profile.dimension,
            vector,
            created_at,
        })
    }
    fn validate(&self) -> Result<(), VectorError> {
        if self.contract_version != V1
            || self.source_version == 0
            || self.dimension == 0
            || self.dimension > MAX_VECTOR_DIMENSION
            || self.vector.dimension() != self.dimension
        {
            Err(VectorError::InvalidEmbedding)
        } else {
            Ok(())
        }
    }
}
impl<'de> Deserialize<'de> for ChunkEmbedding {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct W {
            contract_version: ProtocolVersion,
            embedding_id: EmbeddingRecordId,
            chunk_id: KnowledgeChunkId,
            artifact_id: KnowledgeArtifactId,
            source_id: KnowledgeSourceId,
            source_version: u64,
            original_artifact_hash: ContentHash,
            chunk_content_hash: ContentHash,
            profile_id: EmbeddingProfileId,
            profile_fingerprint: ProfileFingerprint,
            dimension: usize,
            vector: EmbeddingVector,
            created_at: Timestamp,
        }
        let w = W::deserialize(d)?;
        let x = Self {
            contract_version: w.contract_version,
            embedding_id: w.embedding_id,
            chunk_id: w.chunk_id,
            artifact_id: w.artifact_id,
            source_id: w.source_id,
            source_version: w.source_version,
            original_artifact_hash: w.original_artifact_hash,
            chunk_content_hash: w.chunk_content_hash,
            profile_id: w.profile_id,
            profile_fingerprint: w.profile_fingerprint,
            dimension: w.dimension,
            vector: w.vector,
            created_at: w.created_at,
        };
        x.validate().map_err(serde::de::Error::custom)?;
        Ok(x)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VectorRetrievalFilters {
    pub audience: Audience,
    pub course_id: Option<CourseId>,
    pub lesson_id: Option<LessonId>,
}
#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VectorRetrievalQuery {
    pub contract_version: ProtocolVersion,
    pub vector_policy_version: ProtocolVersion,
    pub query_id: RetrievalQueryId,
    pub result_id: RetrievalResultId,
    pub profile_id: EmbeddingProfileId,
    pub profile_fingerprint: ProfileFingerprint,
    pub dimension: usize,
    vector: EmbeddingVector,
    pub filters: VectorRetrievalFilters,
    pub maximum_results: usize,
}
impl fmt::Debug for VectorRetrievalQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VectorRetrievalQuery")
            .field("query_id", &self.query_id)
            .field("vector", &"[REDACTED]")
            .finish()
    }
}
impl VectorRetrievalQuery {
    pub fn new(
        query_id: RetrievalQueryId,
        result_id: RetrievalResultId,
        profile: &EmbeddingProfile,
        vector: EmbeddingVector,
        filters: VectorRetrievalFilters,
        maximum_results: usize,
    ) -> Result<Self, VectorError> {
        let q = Self {
            contract_version: V1,
            vector_policy_version: VECTOR_RETRIEVAL_V1,
            query_id,
            result_id,
            profile_id: profile.profile_id,
            profile_fingerprint: profile.fingerprint.clone(),
            dimension: profile.dimension,
            vector,
            filters,
            maximum_results,
        };
        q.validate(profile)?;
        Ok(q)
    }
    fn validate(&self, p: &EmbeddingProfile) -> Result<(), VectorError> {
        p.validate()?;
        if self.contract_version != V1 || self.vector_policy_version != VECTOR_RETRIEVAL_V1 {
            return Err(VectorError::UnsupportedContract);
        }
        if self.maximum_results == 0 || self.maximum_results > MAX_VECTOR_RESULTS {
            return Err(VectorError::InvalidResultLimit);
        }
        if self.profile_id != p.profile_id || self.profile_fingerprint != p.fingerprint {
            return Err(VectorError::ProfileMismatch);
        }
        if self.dimension != p.dimension || self.vector.dimension() != p.dimension {
            return Err(VectorError::InvalidVector);
        }
        Ok(())
    }
}
impl<'de> Deserialize<'de> for VectorRetrievalQuery {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct W {
            contract_version: ProtocolVersion,
            vector_policy_version: ProtocolVersion,
            query_id: RetrievalQueryId,
            result_id: RetrievalResultId,
            profile_id: EmbeddingProfileId,
            profile_fingerprint: ProfileFingerprint,
            dimension: usize,
            vector: EmbeddingVector,
            filters: VectorRetrievalFilters,
            maximum_results: usize,
        }
        let w = W::deserialize(d)?;
        if w.contract_version != V1 || w.vector_policy_version != VECTOR_RETRIEVAL_V1 {
            return Err(serde::de::Error::custom(VectorError::UnsupportedContract));
        }
        if w.maximum_results == 0
            || w.maximum_results > MAX_VECTOR_RESULTS
            || w.dimension != w.vector.dimension()
        {
            return Err(serde::de::Error::custom(VectorError::InvalidVector));
        }
        Ok(Self {
            contract_version: w.contract_version,
            vector_policy_version: w.vector_policy_version,
            query_id: w.query_id,
            result_id: w.result_id,
            profile_id: w.profile_id,
            profile_fingerprint: w.profile_fingerprint,
            dimension: w.dimension,
            vector: w.vector,
            filters: w.filters,
            maximum_results: w.maximum_results,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SimilarityEvidence {
    pub embedding_id: EmbeddingRecordId,
    pub metric: VectorMetric,
    pub exact_dot_product: i64,
    pub accumulation_order: ProtocolVersion,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VectorRetrievalCandidate {
    pub embedding_id: EmbeddingRecordId,
    pub profile_id: EmbeddingProfileId,
    pub profile_fingerprint: ProfileFingerprint,
    pub dimension: usize,
    pub chunk_id: KnowledgeChunkId,
    pub artifact_id: KnowledgeArtifactId,
    pub source_id: KnowledgeSourceId,
    pub source_version: u64,
    pub evidence: SimilarityEvidence,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VectorExclusionReason {
    NotActive,
    AssessmentProtected,
    AudienceRestricted,
    CourseScopeMismatch,
    LessonScopeMismatch,
    MissingEmbedding,
    ProfileMismatch,
    ResultLimit,
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VectorRetrievalExclusion {
    pub chunk_id: KnowledgeChunkId,
    pub artifact_id: KnowledgeArtifactId,
    pub source_id: KnowledgeSourceId,
    pub source_version: u64,
    pub reason: VectorExclusionReason,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VectorRetrievalResult {
    pub contract_version: ProtocolVersion,
    pub vector_policy_version: ProtocolVersion,
    pub query_id: RetrievalQueryId,
    pub result_id: RetrievalResultId,
    pub profile_id: EmbeddingProfileId,
    pub profile_fingerprint: ProfileFingerprint,
    pub dimension: usize,
    pub metric: VectorMetric,
    pub candidates: Vec<VectorRetrievalCandidate>,
    pub exclusions: Vec<VectorRetrievalExclusion>,
}

impl<'de> Deserialize<'de> for VectorRetrievalCandidate {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct W {
            embedding_id: EmbeddingRecordId,
            profile_id: EmbeddingProfileId,
            profile_fingerprint: ProfileFingerprint,
            dimension: usize,
            chunk_id: KnowledgeChunkId,
            artifact_id: KnowledgeArtifactId,
            source_id: KnowledgeSourceId,
            source_version: u64,
            evidence: SimilarityEvidence,
        }
        let w = W::deserialize(d)?;
        if w.source_version == 0
            || w.dimension == 0
            || w.dimension > MAX_VECTOR_DIMENSION
            || w.embedding_id != w.evidence.embedding_id
            || w.evidence.metric != VectorMetric::DotProduct
            || w.evidence.accumulation_order != VECTOR_RETRIEVAL_V1
        {
            return Err(serde::de::Error::custom(VectorError::InvalidCorpus));
        }
        Ok(Self {
            embedding_id: w.embedding_id,
            profile_id: w.profile_id,
            profile_fingerprint: w.profile_fingerprint,
            dimension: w.dimension,
            chunk_id: w.chunk_id,
            artifact_id: w.artifact_id,
            source_id: w.source_id,
            source_version: w.source_version,
            evidence: w.evidence,
        })
    }
}
impl<'de> Deserialize<'de> for VectorRetrievalExclusion {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct W {
            chunk_id: KnowledgeChunkId,
            artifact_id: KnowledgeArtifactId,
            source_id: KnowledgeSourceId,
            source_version: u64,
            reason: VectorExclusionReason,
        }
        let w = W::deserialize(d)?;
        if w.source_version == 0 {
            return Err(serde::de::Error::custom(VectorError::InvalidCorpus));
        }
        Ok(Self {
            chunk_id: w.chunk_id,
            artifact_id: w.artifact_id,
            source_id: w.source_id,
            source_version: w.source_version,
            reason: w.reason,
        })
    }
}
impl VectorRetrievalResult {
    pub(crate) fn validate(&self) -> Result<(), VectorError> {
        if self.contract_version != V1
            || self.vector_policy_version != VECTOR_RETRIEVAL_V1
            || self.dimension == 0
            || self.dimension > MAX_VECTOR_DIMENSION
            || self.metric != VectorMetric::DotProduct
            || self.candidates.len() > MAX_VECTOR_RESULTS
        {
            return Err(VectorError::UnsupportedContract);
        }
        if self.candidates.windows(2).any(|w| {
            w[0].evidence.exact_dot_product < w[1].evidence.exact_dot_product
                || (w[0].evidence.exact_dot_product == w[1].evidence.exact_dot_product
                    && w[0].chunk_id >= w[1].chunk_id)
        }) || self.exclusions.windows(2).any(|w| w[0] >= w[1])
        {
            return Err(VectorError::InvalidCorpus);
        }
        let candidates: BTreeSet<_> = self.candidates.iter().map(|x| x.chunk_id).collect();
        let embedding_records: BTreeSet<_> =
            self.candidates.iter().map(|x| x.embedding_id).collect();
        let exclusions: BTreeSet<_> = self.exclusions.iter().map(|x| x.chunk_id).collect();
        if candidates.len() != self.candidates.len()
            || embedding_records.len() != self.candidates.len()
            || exclusions.len() != self.exclusions.len()
            || !candidates.is_disjoint(&exclusions)
            || self.candidates.iter().any(|x| {
                x.profile_id != self.profile_id
                    || x.profile_fingerprint != self.profile_fingerprint
                    || x.dimension != self.dimension
                    || x.evidence.metric != self.metric
            })
        {
            return Err(VectorError::InvalidCorpus);
        }
        Ok(())
    }
}
impl<'de> Deserialize<'de> for VectorRetrievalResult {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct W {
            contract_version: ProtocolVersion,
            vector_policy_version: ProtocolVersion,
            query_id: RetrievalQueryId,
            result_id: RetrievalResultId,
            profile_id: EmbeddingProfileId,
            profile_fingerprint: ProfileFingerprint,
            dimension: usize,
            metric: VectorMetric,
            candidates: Vec<VectorRetrievalCandidate>,
            exclusions: Vec<VectorRetrievalExclusion>,
        }
        let w = W::deserialize(d)?;
        let x = Self {
            contract_version: w.contract_version,
            vector_policy_version: w.vector_policy_version,
            query_id: w.query_id,
            result_id: w.result_id,
            profile_id: w.profile_id,
            profile_fingerprint: w.profile_fingerprint,
            dimension: w.dimension,
            metric: w.metric,
            candidates: w.candidates,
            exclusions: w.exclusions,
        };
        x.validate().map_err(serde::de::Error::custom)?;
        Ok(x)
    }
}

#[derive(Clone)]
pub struct VectorCorpusRecords {
    pub sources: Vec<KnowledgeSource>,
    pub artifacts: Vec<KnowledgeArtifact>,
    pub chunks: Vec<KnowledgeChunk>,
    pub profiles: Vec<EmbeddingProfile>,
    pub embeddings: Vec<ChunkEmbedding>,
}
pub trait VectorRetrievalReader {
    fn load_vector_corpus(&self) -> Result<VectorCorpusRecords, VectorError>;
}
#[derive(Clone)]
struct Doc {
    source: KnowledgeSource,
    artifact_id: KnowledgeArtifactId,
    chunk: KnowledgeChunk,
    embedding: Option<ChunkEmbedding>,
}
#[derive(Clone)]
pub struct InMemoryVectorSnapshot {
    profile: EmbeddingProfile,
    documents: Vec<Doc>,
}
impl InMemoryVectorSnapshot {
    pub fn load(
        reader: &impl VectorRetrievalReader,
        profile_id: EmbeddingProfileId,
    ) -> Result<Self, VectorError> {
        Self::from_records(reader.load_vector_corpus()?, profile_id)
    }
    pub fn from_records(
        mut r: VectorCorpusRecords,
        profile_id: EmbeddingProfileId,
    ) -> Result<Self, VectorError> {
        r.sources.sort_by_key(|x| (x.source_id, x.source_version));
        r.artifacts.sort_by_key(|x| x.artifact_id);
        r.chunks.sort_by_key(|x| x.chunk_id);
        r.profiles.sort_by_key(|x| x.profile_id);
        r.embeddings.sort_by_key(|x| x.embedding_id);
        if dup(&r.sources, |x| (x.source_id, x.source_version))
            || dup(&r.artifacts, |x| x.artifact_id)
            || dup(&r.chunks, |x| x.chunk_id)
            || dup(&r.profiles, |x| x.profile_id)
            || dup(&r.embeddings, |x| x.embedding_id)
        {
            return Err(VectorError::InvalidCorpus);
        }
        for p in &r.profiles {
            p.validate()?
        }
        let profiles_by_id: BTreeMap<_, _> = r.profiles.iter().map(|p| (p.profile_id, p)).collect();
        let profile = r
            .profiles
            .iter()
            .find(|p| p.profile_id == profile_id)
            .cloned()
            .ok_or(VectorError::ProfileMismatch)?;
        let mut active = BTreeMap::new();
        for s in &r.sources {
            s.validate().map_err(|_| VectorError::InvalidCorpus)?;
            *active.entry(s.source_id).or_insert(0) += usize::from(s.status == SourceStatus::Active)
        }
        if active.values().any(|n| *n != 1) {
            return Err(VectorError::InvalidCorpus);
        }
        let sources: BTreeMap<_, _> = r
            .sources
            .iter()
            .map(|s| ((s.source_id, s.source_version), s))
            .collect();
        let mut chunks_by_art: BTreeMap<_, Vec<KnowledgeChunk>> = BTreeMap::new();
        for c in r.chunks {
            chunks_by_art.entry(c.artifact_id).or_default().push(c)
        }
        let mut artifact_counts = BTreeMap::new();
        for a in &r.artifacts {
            *artifact_counts
                .entry((a.source_id, a.source_version))
                .or_insert(0usize) += 1;
        }
        if r.sources
            .iter()
            .any(|s| artifact_counts.get(&(s.source_id, s.source_version)) != Some(&1))
        {
            return Err(VectorError::InvalidCorpus);
        }
        let mut docs = Vec::new();
        let mut known_chunks = BTreeSet::new();
        for a in &r.artifacts {
            a.validate().map_err(|_| VectorError::InvalidCorpus)?;
            let s = sources
                .get(&(a.source_id, a.source_version))
                .ok_or(VectorError::InvalidCorpus)?;
            let mut cs = chunks_by_art
                .remove(&a.artifact_id)
                .ok_or(VectorError::InvalidCorpus)?;
            cs.sort_by_key(|c| c.ordinal);
            validate_chunks(&cs, s, a).map_err(|_| VectorError::InvalidCorpus)?;
            for c in cs {
                known_chunks.insert(c.chunk_id);
                docs.push(Doc {
                    source: (*s).clone(),
                    artifact_id: a.artifact_id,
                    chunk: c,
                    embedding: None,
                })
            }
        }
        if !chunks_by_art.is_empty() {
            return Err(VectorError::InvalidCorpus);
        }
        let mut slot = BTreeSet::new();
        for e in r.embeddings {
            e.validate()?;
            let embedding_profile = profiles_by_id
                .get(&e.profile_id)
                .ok_or(VectorError::InvalidCorpus)?;
            if e.profile_fingerprint != embedding_profile.fingerprint
                || e.dimension != embedding_profile.dimension
            {
                return Err(VectorError::ProfileFingerprintMismatch);
            }
            if !known_chunks.contains(&e.chunk_id) {
                return Err(VectorError::InvalidCorpus);
            }
            if !slot.insert((e.chunk_id, e.profile_id)) {
                return Err(VectorError::InvalidCorpus);
            }
            let d = docs
                .iter_mut()
                .find(|d| d.chunk.chunk_id == e.chunk_id)
                .ok_or(VectorError::InvalidCorpus)?;
            if (
                e.artifact_id,
                e.source_id,
                e.source_version,
                &e.original_artifact_hash,
                &e.chunk_content_hash,
            ) != (
                d.chunk.artifact_id,
                d.chunk.source_id,
                d.chunk.source_version,
                &d.chunk.original_content_hash,
                &d.chunk.chunk_content_hash,
            ) {
                return Err(VectorError::InvalidCorpus);
            }
            if e.profile_id == profile.profile_id {
                if e.profile_fingerprint != profile.fingerprint || e.dimension != profile.dimension
                {
                    return Err(VectorError::ProfileFingerprintMismatch);
                }
                d.embedding = Some(e)
            }
        }
        docs.sort_by_key(|d| d.chunk.chunk_id);
        Ok(Self {
            profile,
            documents: docs,
        })
    }
    pub fn retrieve(&self, q: &VectorRetrievalQuery) -> Result<VectorRetrievalResult, VectorError> {
        q.validate(&self.profile)?;
        let mut candidates = Vec::new();
        let mut exclusions = Vec::new();
        for d in &self.documents {
            let reason = if d.source.status != SourceStatus::Active {
                Some(VectorExclusionReason::NotActive)
            } else if let Err(e) = exposure(&d.source, q.filters.audience) {
                Some(match e {
                    crate::ExclusionReason::NotActive => VectorExclusionReason::NotActive,
                    crate::ExclusionReason::AssessmentProtected => {
                        VectorExclusionReason::AssessmentProtected
                    }
                    crate::ExclusionReason::AudienceRestricted => {
                        VectorExclusionReason::AudienceRestricted
                    }
                })
            } else if q.filters.course_id.is_some()
                && q.filters.course_id != d.source.scope.course_id
            {
                Some(VectorExclusionReason::CourseScopeMismatch)
            } else if q.filters.lesson_id.is_some()
                && q.filters.lesson_id != d.source.scope.lesson_id
            {
                Some(VectorExclusionReason::LessonScopeMismatch)
            } else if d.embedding.is_none() {
                Some(VectorExclusionReason::MissingEmbedding)
            } else {
                None
            };
            if let Some(reason) = reason {
                exclusions.push(exclusion(d, reason));
                continue;
            }
            let e = d.embedding.as_ref().expect("checked");
            let score = dot(q.vector.values(), e.vector.values())?;
            candidates.push(VectorRetrievalCandidate {
                embedding_id: e.embedding_id,
                profile_id: e.profile_id,
                profile_fingerprint: e.profile_fingerprint.clone(),
                dimension: e.dimension,
                chunk_id: d.chunk.chunk_id,
                artifact_id: d.artifact_id,
                source_id: d.source.source_id,
                source_version: d.source.source_version,
                evidence: SimilarityEvidence {
                    embedding_id: e.embedding_id,
                    metric: VectorMetric::DotProduct,
                    exact_dot_product: score,
                    accumulation_order: VECTOR_RETRIEVAL_V1,
                },
            })
        }
        candidates.sort_by(|a, b| {
            b.evidence
                .exact_dot_product
                .cmp(&a.evidence.exact_dot_product)
                .then(a.chunk_id.cmp(&b.chunk_id))
        });
        let limited = if candidates.len() > q.maximum_results {
            candidates.split_off(q.maximum_results)
        } else {
            Vec::new()
        };
        for x in limited {
            let d = self
                .documents
                .iter()
                .find(|d| d.chunk.chunk_id == x.chunk_id)
                .expect("snapshot");
            exclusions.push(exclusion(d, VectorExclusionReason::ResultLimit))
        }
        exclusions.sort();
        Ok(VectorRetrievalResult {
            contract_version: V1,
            vector_policy_version: VECTOR_RETRIEVAL_V1,
            query_id: q.query_id,
            result_id: q.result_id,
            profile_id: self.profile.profile_id,
            profile_fingerprint: self.profile.fingerprint.clone(),
            dimension: self.profile.dimension,
            metric: self.profile.metric,
            candidates,
            exclusions,
        })
    }
}
fn dot(a: &[i16], b: &[i16]) -> Result<i64, VectorError> {
    a.iter().zip(b).try_fold(0i64, |sum, (&x, &y)| {
        sum.checked_add(i64::from(x) * i64::from(y))
            .ok_or(VectorError::NumericOverflow)
    })
}
fn exclusion(d: &Doc, reason: VectorExclusionReason) -> VectorRetrievalExclusion {
    VectorRetrievalExclusion {
        chunk_id: d.chunk.chunk_id,
        artifact_id: d.artifact_id,
        source_id: d.source.source_id,
        source_version: d.source.source_version,
        reason,
    }
}
fn dup<T, K: Eq>(v: &[T], f: impl Fn(&T) -> K) -> bool {
    v.windows(2).any(|w| f(&w[0]) == f(&w[1]))
}
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum VectorError {
    #[error("unsupported embedding contract or vector policy")]
    UnsupportedContract,
    #[error("invalid embedding profile")]
    InvalidProfile,
    #[error("embedding profile fingerprint mismatch")]
    ProfileFingerprintMismatch,
    #[error("embedding profile mismatch")]
    ProfileMismatch,
    #[error("invalid vector dimension or payload")]
    InvalidVector,
    #[error("invalid chunk embedding")]
    InvalidEmbedding,
    #[error("invalid result limit")]
    InvalidResultLimit,
    #[error("vector arithmetic overflow")]
    NumericOverflow,
    #[error("vector corpus is incomplete, conflicting, orphaned, or corrupted")]
    InvalidCorpus,
}
