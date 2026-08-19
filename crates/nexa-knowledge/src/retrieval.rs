//! Governed synchronous lexical retrieval. Artifact text and query text are inert data.

use crate::{
    exposure, Audience, ContentHash, KnowledgeArtifact, KnowledgeChunk, KnowledgeError,
    KnowledgeScope, KnowledgeSource, SourceStatus, V1,
};
use nexa_domain::{
    CourseId, KnowledgeArtifactId, KnowledgeChunkId, KnowledgeSourceId, LessonId, ProtocolVersion,
    RetrievalQueryId, RetrievalResultId,
};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    fmt,
};

/// Maximum UTF-8 byte length accepted for a query.
pub const MAX_RETRIEVAL_QUERY_BYTES: usize = 4 * 1024;
/// Maximum number of normalized query terms.
pub const MAX_RETRIEVAL_QUERY_TERMS: usize = 128;
/// Maximum UTF-8 byte length of one normalized term.
pub const MAX_RETRIEVAL_TERM_BYTES: usize = 256;
/// Hard platform result cap, applied in addition to the caller's limit.
pub const MAX_RETRIEVAL_RESULTS: usize = 100;
/// V1 lexical retrieval policy and wire contract.
pub const LEXICAL_RETRIEVAL_V1: ProtocolVersion = V1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetrievalFilters {
    pub audience: Audience,
    pub course_id: Option<CourseId>,
    pub lesson_id: Option<LessonId>,
}

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RetrievalQuery {
    pub contract_version: ProtocolVersion,
    pub retrieval_policy_version: ProtocolVersion,
    pub query_id: RetrievalQueryId,
    pub result_id: RetrievalResultId,
    pub text: String,
    pub filters: RetrievalFilters,
    pub maximum_results: usize,
}

impl fmt::Debug for RetrievalQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RetrievalQuery")
            .field("query_id", &self.query_id)
            .field("result_id", &self.result_id)
            .field("text", &"[REDACTED]")
            .field("filters", &self.filters)
            .field("maximum_results", &self.maximum_results)
            .finish()
    }
}

impl RetrievalQuery {
    pub fn validate(&self) -> Result<Vec<String>, RetrievalError> {
        if self.contract_version != V1 || self.retrieval_policy_version != LEXICAL_RETRIEVAL_V1 {
            return Err(RetrievalError::UnsupportedContract);
        }
        if self.maximum_results == 0 || self.maximum_results > MAX_RETRIEVAL_RESULTS {
            return Err(RetrievalError::InvalidResultLimit);
        }
        if self.text.is_empty() || self.text.len() > MAX_RETRIEVAL_QUERY_BYTES {
            return Err(RetrievalError::InvalidQuery);
        }
        let terms = tokenize(&self.text)?;
        if terms.is_empty() || terms.len() > MAX_RETRIEVAL_QUERY_TERMS {
            return Err(RetrievalError::InvalidQuery);
        }
        Ok(terms)
    }
}

impl<'de> Deserialize<'de> for RetrievalQuery {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            contract_version: ProtocolVersion,
            retrieval_policy_version: ProtocolVersion,
            query_id: RetrievalQueryId,
            result_id: RetrievalResultId,
            text: String,
            filters: RetrievalFilters,
            maximum_results: usize,
        }
        let w = Wire::deserialize(deserializer)?;
        let value = Self {
            contract_version: w.contract_version,
            retrieval_policy_version: w.retrieval_policy_version,
            query_id: w.query_id,
            result_id: w.result_id,
            text: w.text,
            filters: w.filters,
            maximum_results: w.maximum_results,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

/// Exact integer evidence for one query term. V1 contribution is
/// `query_frequency * term_frequency * (document_count - document_frequency + 1)`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TermScoreEvidence {
    /// SHA-256 of the normalized UTF-8 query term; the untrusted term is not returned.
    pub term_hash: ContentHash,
    pub query_frequency: u32,
    pub term_frequency: u32,
    pub document_frequency: u32,
    pub document_count: u32,
    pub contribution: u64,
}

impl TermScoreEvidence {
    pub fn new(
        term_hash: ContentHash,
        query_frequency: u32,
        term_frequency: u32,
        document_frequency: u32,
        document_count: u32,
    ) -> Result<Self, RetrievalError> {
        let contribution = u64::from(query_frequency)
            .checked_mul(u64::from(term_frequency))
            .and_then(|n| {
                n.checked_mul(u64::from(
                    document_count
                        .checked_sub(document_frequency)?
                        .checked_add(1)?,
                ))
            })
            .ok_or(RetrievalError::InvalidScore)?;
        let value = Self {
            term_hash,
            query_frequency,
            term_frequency,
            document_frequency,
            document_count,
            contribution,
        };
        value.validate()?;
        Ok(value)
    }

    fn validate(&self) -> Result<(), RetrievalError> {
        let expected = u64::from(self.query_frequency)
            .checked_mul(u64::from(self.term_frequency))
            .and_then(|n| {
                self.document_count
                    .checked_sub(self.document_frequency)
                    .and_then(|d| d.checked_add(1))
                    .and_then(|d| n.checked_mul(u64::from(d)))
            });
        if self.query_frequency == 0
            || self.term_frequency == 0
            || self.document_frequency == 0
            || expected != Some(self.contribution)
        {
            return Err(RetrievalError::InvalidScore);
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for TermScoreEvidence {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            term_hash: ContentHash,
            query_frequency: u32,
            term_frequency: u32,
            document_frequency: u32,
            document_count: u32,
            contribution: u64,
        }
        let w = Wire::deserialize(d)?;
        let value = Self {
            term_hash: w.term_hash,
            query_frequency: w.query_frequency,
            term_frequency: w.term_frequency,
            document_frequency: w.document_frequency,
            document_count: w.document_count,
            contribution: w.contribution,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

/// A finite score validated at every construction and wire boundary.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct RetrievalScore(f64);

impl RetrievalScore {
    pub fn new(value: u64) -> Result<Self, RetrievalError> {
        let value = value as f64;
        if value.is_finite() && value > 0.0 {
            Ok(Self(value))
        } else {
            Err(RetrievalError::InvalidScore)
        }
    }
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl<'de> Deserialize<'de> for RetrievalScore {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = f64::deserialize(deserializer)?;
        if value.is_finite() && value > 0.0 && value.fract() == 0.0 {
            Ok(Self(value))
        } else {
            Err(serde::de::Error::custom(RetrievalError::InvalidScore))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RetrievalCandidate {
    pub chunk_id: KnowledgeChunkId,
    pub artifact_id: KnowledgeArtifactId,
    pub source_id: KnowledgeSourceId,
    pub source_version: u64,
    pub score: RetrievalScore,
    pub score_evidence: Vec<TermScoreEvidence>,
}

impl RetrievalCandidate {
    pub fn new(
        chunk_id: KnowledgeChunkId,
        artifact_id: KnowledgeArtifactId,
        source_id: KnowledgeSourceId,
        source_version: u64,
        score: RetrievalScore,
        score_evidence: Vec<TermScoreEvidence>,
    ) -> Result<Self, RetrievalError> {
        let value = Self {
            chunk_id,
            artifact_id,
            source_id,
            source_version,
            score,
            score_evidence,
        };
        value.validate()?;
        Ok(value)
    }

    fn validate(&self) -> Result<(), RetrievalError> {
        let total = self.score_evidence.iter().try_fold(0u64, |n, e| {
            e.validate()?;
            n.checked_add(e.contribution)
                .ok_or(RetrievalError::InvalidScore)
        })?;
        if self.source_version == 0
            || self.score_evidence.is_empty()
            || self
                .score_evidence
                .windows(2)
                .any(|e| e[0].term_hash >= e[1].term_hash)
            || total as f64 != self.score.get()
        {
            return Err(RetrievalError::InvalidScore);
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for RetrievalCandidate {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            chunk_id: KnowledgeChunkId,
            artifact_id: KnowledgeArtifactId,
            source_id: KnowledgeSourceId,
            source_version: u64,
            score: RetrievalScore,
            score_evidence: Vec<TermScoreEvidence>,
        }
        let w = Wire::deserialize(d)?;
        let value = Self {
            chunk_id: w.chunk_id,
            artifact_id: w.artifact_id,
            source_id: w.source_id,
            source_version: w.source_version,
            score: w.score,
            score_evidence: w.score_evidence,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalExclusionReason {
    NotActive,
    AssessmentProtected,
    AudienceRestricted,
    CourseScopeMismatch,
    LessonScopeMismatch,
    NoMatchingTerms,
    ResultLimit,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RetrievalExclusion {
    pub chunk_id: KnowledgeChunkId,
    pub artifact_id: KnowledgeArtifactId,
    pub source_id: KnowledgeSourceId,
    pub source_version: u64,
    pub reason: RetrievalExclusionReason,
}

impl RetrievalExclusion {
    pub fn new(
        chunk_id: KnowledgeChunkId,
        artifact_id: KnowledgeArtifactId,
        source_id: KnowledgeSourceId,
        source_version: u64,
        reason: RetrievalExclusionReason,
    ) -> Result<Self, RetrievalError> {
        let value = Self {
            chunk_id,
            artifact_id,
            source_id,
            source_version,
            reason,
        };
        value.validate()?;
        Ok(value)
    }

    fn validate(&self) -> Result<(), RetrievalError> {
        if self.source_version == 0 {
            Err(RetrievalError::InvalidCorpus)
        } else {
            Ok(())
        }
    }
}

impl<'de> Deserialize<'de> for RetrievalExclusion {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            chunk_id: KnowledgeChunkId,
            artifact_id: KnowledgeArtifactId,
            source_id: KnowledgeSourceId,
            source_version: u64,
            reason: RetrievalExclusionReason,
        }
        let w = Wire::deserialize(d)?;
        let value = Self {
            chunk_id: w.chunk_id,
            artifact_id: w.artifact_id,
            source_id: w.source_id,
            source_version: w.source_version,
            reason: w.reason,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RetrievalResult {
    pub contract_version: ProtocolVersion,
    pub retrieval_policy_version: ProtocolVersion,
    pub query_id: RetrievalQueryId,
    pub result_id: RetrievalResultId,
    pub candidates: Vec<RetrievalCandidate>,
    pub exclusions: Vec<RetrievalExclusion>,
}

impl RetrievalResult {
    fn validate(&self) -> Result<(), RetrievalError> {
        if self.contract_version != V1
            || self.retrieval_policy_version != LEXICAL_RETRIEVAL_V1
            || self.candidates.len() > MAX_RETRIEVAL_RESULTS
        {
            return Err(RetrievalError::UnsupportedContract);
        }
        let mut previous = None;
        if self.candidates.iter().any(|candidate| {
            let order = (candidate.score.get(), candidate.chunk_id);
            let incorrectly_ordered = previous
                .map(|(score, chunk)| score < order.0 || (score == order.0 && chunk >= order.1))
                .unwrap_or(false);
            previous = Some(order);
            incorrectly_ordered || candidate.validate().is_err()
        }) {
            return Err(RetrievalError::InvalidScore);
        }
        if self
            .exclusions
            .iter()
            .any(|exclusion| exclusion.source_version == 0)
            || self
                .exclusions
                .windows(2)
                .any(|pair| exclusion_order(&pair[0]) >= exclusion_order(&pair[1]))
        {
            return Err(RetrievalError::InvalidCorpus);
        }
        let candidate_ids: BTreeSet<_> = self.candidates.iter().map(reference_identity).collect();
        let exclusion_ids: BTreeSet<_> = self.exclusions.iter().map(exclusion_identity).collect();
        let candidate_chunk_ids: BTreeSet<_> = self
            .candidates
            .iter()
            .map(|candidate| candidate.chunk_id)
            .collect();
        let exclusion_chunk_ids: BTreeSet<_> = self
            .exclusions
            .iter()
            .map(|exclusion| exclusion.chunk_id)
            .collect();
        if candidate_ids.len() != self.candidates.len()
            || exclusion_ids.len() != self.exclusions.len()
            || !candidate_ids.is_disjoint(&exclusion_ids)
            || candidate_chunk_ids.len() != self.candidates.len()
            || exclusion_chunk_ids.len() != self.exclusions.len()
            || !candidate_chunk_ids.is_disjoint(&exclusion_chunk_ids)
        {
            return Err(RetrievalError::InvalidCorpus);
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for RetrievalResult {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            contract_version: ProtocolVersion,
            retrieval_policy_version: ProtocolVersion,
            query_id: RetrievalQueryId,
            result_id: RetrievalResultId,
            candidates: Vec<RetrievalCandidate>,
            exclusions: Vec<RetrievalExclusion>,
        }
        let w = Wire::deserialize(deserializer)?;
        let result = Self {
            contract_version: w.contract_version,
            retrieval_policy_version: w.retrieval_policy_version,
            query_id: w.query_id,
            result_id: w.result_id,
            candidates: w.candidates,
            exclusions: w.exclusions,
        };
        result.validate().map_err(serde::de::Error::custom)?;
        Ok(result)
    }
}

#[derive(Clone, Debug, thiserror::Error, Eq, PartialEq)]
pub enum RetrievalError {
    #[error("unsupported retrieval contract or policy")]
    UnsupportedContract,
    #[error("query violates lexical retrieval bounds")]
    InvalidQuery,
    #[error("result limit violates lexical retrieval bounds")]
    InvalidResultLimit,
    #[error("retrieval corpus is incomplete or conflicting")]
    InvalidCorpus,
    #[error("retrieval score is invalid")]
    InvalidScore,
    #[error("knowledge integrity validation failed")]
    IntegrityFailure,
}

/// Repository-neutral immutable records supplied to the retrieval boundary.
#[derive(Clone)]
pub struct RetrievalCorpusRecords {
    pub sources: Vec<KnowledgeSource>,
    pub artifacts: Vec<KnowledgeArtifact>,
    pub chunks: Vec<KnowledgeChunk>,
}

/// Read port. Implementations return an owned point-in-time view; retrieval never reads mutable maps.
pub trait KnowledgeRetrievalReader {
    fn load_retrieval_corpus(&self) -> Result<RetrievalCorpusRecords, RetrievalError>;
}

#[derive(Clone)]
struct CorpusDocument {
    source: KnowledgeSource,
    artifact: KnowledgeArtifact,
    chunk: KnowledgeChunk,
    terms: BTreeMap<String, u32>,
}

/// Validated, immutable, insertion-order-independent retrieval snapshot.
#[derive(Clone)]
pub struct InMemoryRetrievalSnapshot {
    documents: Vec<CorpusDocument>,
    exclusions: Vec<RetrievalExclusion>,
}

impl InMemoryRetrievalSnapshot {
    pub fn load(reader: &impl KnowledgeRetrievalReader) -> Result<Self, RetrievalError> {
        Self::from_records(reader.load_retrieval_corpus()?)
    }

    pub fn from_records(mut records: RetrievalCorpusRecords) -> Result<Self, RetrievalError> {
        records
            .sources
            .sort_by_key(|s| (s.source_id, s.source_version));
        records.artifacts.sort_by_key(|a| a.artifact_id);
        records.chunks.sort_by_key(|c| c.chunk_id);
        if adjacent_duplicate(&records.sources, |s| (s.source_id, s.source_version))
            || adjacent_duplicate(&records.artifacts, |a| a.artifact_id)
            || adjacent_duplicate(&records.chunks, |c| c.chunk_id)
        {
            return Err(RetrievalError::InvalidCorpus);
        }

        let mut active_counts = BTreeMap::new();
        for source in &records.sources {
            source.validate().map_err(corpus_error)?;
            active_counts.entry(source.source_id).or_insert(0usize);
            if source.status == SourceStatus::Active {
                *active_counts.entry(source.source_id).or_insert(0usize) += 1;
            }
        }
        if active_counts.values().any(|count| *count != 1) {
            return Err(RetrievalError::InvalidCorpus);
        }
        let sources: BTreeMap<_, _> = records
            .sources
            .iter()
            .map(|s| ((s.source_id, s.source_version), s))
            .collect();
        let mut chunks_by_artifact: BTreeMap<_, Vec<&KnowledgeChunk>> = BTreeMap::new();
        let mut artifacts_by_source = BTreeMap::new();
        for artifact in &records.artifacts {
            artifact.validate().map_err(corpus_error)?;
            if !sources.contains_key(&(artifact.source_id, artifact.source_version)) {
                return Err(RetrievalError::InvalidCorpus);
            }
            *artifacts_by_source
                .entry((artifact.source_id, artifact.source_version))
                .or_insert(0usize) += 1;
        }
        if records.sources.iter().any(|source| {
            artifacts_by_source.get(&(source.source_id, source.source_version)) != Some(&1)
        }) {
            return Err(RetrievalError::InvalidCorpus);
        }
        for chunk in &records.chunks {
            chunks_by_artifact
                .entry(chunk.artifact_id)
                .or_default()
                .push(chunk);
        }
        let mut documents = Vec::new();
        let mut exclusions = Vec::new();
        for artifact in &records.artifacts {
            let source = sources[&(artifact.source_id, artifact.source_version)];
            let chunks = chunks_by_artifact
                .remove(&artifact.artifact_id)
                .ok_or(RetrievalError::InvalidCorpus)?;
            let mut ordered: Vec<_> = chunks.into_iter().cloned().collect();
            ordered.sort_by_key(|c| c.ordinal);
            crate::validate_chunks(&ordered, source, artifact).map_err(corpus_error)?;
            for chunk in ordered {
                if source.status == SourceStatus::Active {
                    let text = std::str::from_utf8(chunk.content(artifact).map_err(corpus_error)?)
                        .map_err(|_| RetrievalError::IntegrityFailure)?;
                    documents.push(CorpusDocument {
                        source: source.clone(),
                        artifact: artifact.clone(),
                        terms: frequencies(tokenize_source(text)),
                        chunk,
                    });
                } else {
                    exclusions.push(exclusion(&chunk, RetrievalExclusionReason::NotActive));
                }
            }
        }
        if !chunks_by_artifact.is_empty() || documents.is_empty() {
            return Err(RetrievalError::InvalidCorpus);
        }
        documents.sort_by_key(|d| d.chunk.chunk_id);
        exclusions.sort_by_key(exclusion_order);
        Ok(Self {
            documents,
            exclusions,
        })
    }

    pub fn retrieve(&self, query: &RetrievalQuery) -> Result<RetrievalResult, RetrievalError> {
        let query_terms = frequencies(query.validate()?);
        let mut eligible = Vec::new();
        let mut exclusions = self.exclusions.clone();
        for document in &self.documents {
            let reason = exposure(&document.source, query.filters.audience)
                .err()
                .map(|reason| match reason {
                    crate::ExclusionReason::NotActive => RetrievalExclusionReason::NotActive,
                    crate::ExclusionReason::AssessmentProtected => {
                        RetrievalExclusionReason::AssessmentProtected
                    }
                    crate::ExclusionReason::AudienceRestricted => {
                        RetrievalExclusionReason::AudienceRestricted
                    }
                })
                .or_else(|| scope_mismatch(&document.source.scope, &query.filters));
            if let Some(reason) = reason {
                exclusions.push(exclusion(&document.chunk, reason));
            } else {
                eligible.push(document);
            }
        }

        let document_count =
            u32::try_from(eligible.len()).map_err(|_| RetrievalError::InvalidScore)?;
        let mut document_frequency = BTreeMap::new();
        for term in query_terms.keys() {
            let count = eligible
                .iter()
                .filter(|d| d.terms.contains_key(term))
                .count();
            document_frequency.insert(
                term.clone(),
                u32::try_from(count).map_err(|_| RetrievalError::InvalidScore)?,
            );
        }
        let mut candidates = Vec::new();
        for document in eligible {
            let mut evidence = Vec::new();
            let mut total = 0u64;
            for (term, query_frequency) in &query_terms {
                let Some(term_frequency) = document.terms.get(term) else {
                    continue;
                };
                let df = document_frequency[term];
                let inverse_frequency = document_count
                    .checked_sub(df)
                    .and_then(|n| n.checked_add(1))
                    .ok_or(RetrievalError::InvalidScore)?;
                let contribution = u64::from(*query_frequency)
                    .checked_mul(u64::from(*term_frequency))
                    .and_then(|n| n.checked_mul(u64::from(inverse_frequency)))
                    .ok_or(RetrievalError::InvalidScore)?;
                total = total
                    .checked_add(contribution)
                    .ok_or(RetrievalError::InvalidScore)?;
                evidence.push(TermScoreEvidence {
                    term_hash: ContentHash::sha256(term.as_bytes()),
                    query_frequency: *query_frequency,
                    term_frequency: *term_frequency,
                    document_frequency: df,
                    document_count,
                    contribution,
                });
            }
            evidence.sort_by(|left, right| left.term_hash.cmp(&right.term_hash));
            if evidence.is_empty() {
                exclusions.push(exclusion(
                    &document.chunk,
                    RetrievalExclusionReason::NoMatchingTerms,
                ));
            } else {
                candidates.push(RetrievalCandidate {
                    chunk_id: document.chunk.chunk_id,
                    artifact_id: document.artifact.artifact_id,
                    source_id: document.source.source_id,
                    source_version: document.source.source_version,
                    score: RetrievalScore::new(total)?,
                    score_evidence: evidence,
                });
            }
        }
        candidates.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| left.chunk_id.cmp(&right.chunk_id))
                .then_with(|| left.artifact_id.cmp(&right.artifact_id))
                .then_with(|| left.source_id.cmp(&right.source_id))
                .then_with(|| left.source_version.cmp(&right.source_version))
        });
        if candidates.len() > query.maximum_results {
            for candidate in candidates.drain(query.maximum_results..) {
                exclusions.push(RetrievalExclusion {
                    chunk_id: candidate.chunk_id,
                    artifact_id: candidate.artifact_id,
                    source_id: candidate.source_id,
                    source_version: candidate.source_version,
                    reason: RetrievalExclusionReason::ResultLimit,
                });
            }
        }
        exclusions.sort_by_key(exclusion_order);
        let result = RetrievalResult {
            contract_version: V1,
            retrieval_policy_version: LEXICAL_RETRIEVAL_V1,
            query_id: query.query_id,
            result_id: query.result_id,
            candidates,
            exclusions,
        };
        result.validate()?;
        Ok(result)
    }
}

/// V1 tokenization splits on every non-alphanumeric Unicode scalar and lowercases each
/// alphanumeric scalar with Rust's Unicode lowercase mapping. No compatibility, accent,
/// markup, path, URL, whitespace, or line-ending normalization is performed.
pub fn tokenize(text: &str) -> Result<Vec<String>, RetrievalError> {
    let mut terms = Vec::new();
    let mut term = String::new();
    for character in text.chars() {
        if character.is_alphanumeric() {
            for lowercase in character.to_lowercase() {
                term.push(lowercase);
            }
            if term.len() > MAX_RETRIEVAL_TERM_BYTES {
                return Err(RetrievalError::InvalidQuery);
            }
        } else if !term.is_empty() {
            terms.push(std::mem::take(&mut term));
        }
    }
    if !term.is_empty() {
        terms.push(term);
    }
    Ok(terms)
}

/// Tokenizes corpus text using the query policy while deterministically omitting an
/// entire alphanumeric run if its normalized representation exceeds the term bound.
fn tokenize_source(text: &str) -> Vec<String> {
    let mut terms = Vec::new();
    let mut term = String::new();
    let mut oversized = false;
    for character in text.chars() {
        if character.is_alphanumeric() {
            if !oversized {
                for lowercase in character.to_lowercase() {
                    term.push(lowercase);
                }
                if term.len() > MAX_RETRIEVAL_TERM_BYTES {
                    term.clear();
                    oversized = true;
                }
            }
        } else {
            if !oversized && !term.is_empty() {
                terms.push(std::mem::take(&mut term));
            }
            term.clear();
            oversized = false;
        }
    }
    if !oversized && !term.is_empty() {
        terms.push(term);
    }
    terms
}

fn frequencies(terms: Vec<String>) -> BTreeMap<String, u32> {
    let mut frequencies = BTreeMap::new();
    for term in terms {
        *frequencies.entry(term).or_insert(0) += 1;
    }
    frequencies
}

fn adjacent_duplicate<T, K: Eq>(values: &[T], key: impl Fn(&T) -> K) -> bool {
    values.windows(2).any(|pair| key(&pair[0]) == key(&pair[1]))
}

fn corpus_error(error: KnowledgeError) -> RetrievalError {
    match error {
        KnowledgeError::InvalidHash
        | KnowledgeError::IntegrityMismatch
        | KnowledgeError::InvalidChunk => RetrievalError::IntegrityFailure,
        _ => RetrievalError::InvalidCorpus,
    }
}

fn scope_mismatch(
    scope: &KnowledgeScope,
    filters: &RetrievalFilters,
) -> Option<RetrievalExclusionReason> {
    if filters.course_id.is_some() && scope.course_id != filters.course_id {
        Some(RetrievalExclusionReason::CourseScopeMismatch)
    } else if filters.lesson_id.is_some() && scope.lesson_id != filters.lesson_id {
        Some(RetrievalExclusionReason::LessonScopeMismatch)
    } else {
        None
    }
}

fn exclusion(chunk: &KnowledgeChunk, reason: RetrievalExclusionReason) -> RetrievalExclusion {
    RetrievalExclusion {
        chunk_id: chunk.chunk_id,
        artifact_id: chunk.artifact_id,
        source_id: chunk.source_id,
        source_version: chunk.source_version,
        reason,
    }
}

fn exclusion_order(
    value: &RetrievalExclusion,
) -> (
    KnowledgeChunkId,
    KnowledgeArtifactId,
    KnowledgeSourceId,
    u64,
) {
    (
        value.chunk_id,
        value.artifact_id,
        value.source_id,
        value.source_version,
    )
}

fn reference_identity(
    value: &RetrievalCandidate,
) -> (
    KnowledgeChunkId,
    KnowledgeArtifactId,
    KnowledgeSourceId,
    u64,
) {
    (
        value.chunk_id,
        value.artifact_id,
        value.source_id,
        value.source_version,
    )
}

fn exclusion_identity(
    value: &RetrievalExclusion,
) -> (
    KnowledgeChunkId,
    KnowledgeArtifactId,
    KnowledgeSourceId,
    u64,
) {
    (
        value.chunk_id,
        value.artifact_id,
        value.source_id,
        value.source_version,
    )
}
