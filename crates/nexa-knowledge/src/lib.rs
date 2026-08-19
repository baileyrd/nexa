//! Dependency-light governed knowledge ingestion. Content is untrusted inert data.
#![forbid(unsafe_code)]
use nexa_domain::{
    CourseId, IngestionJobId, KnowledgeArtifactId, KnowledgeChunkId, KnowledgeSourceId, LessonId,
    ProtocolVersion, Timestamp,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
pub const MAX_ARTIFACT_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_METADATA_ENTRIES: usize = 64;
pub const MAX_FIELD_BYTES: usize = 1024;
pub const MAX_CHUNK_BYTES: usize = 64 * 1024;
pub const V1: ProtocolVersion = ProtocolVersion::new(1, 0);
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum KnowledgeError {
    #[error("invalid content hash")]
    InvalidHash,
    #[error("artifact integrity validation failed")]
    IntegrityMismatch,
    #[error("unsupported source contract")]
    UnsupportedSource,
    #[error("artifact violates v1 bounds, media, or encoding policy")]
    InvalidArtifact,
    #[error("metadata violates v1 bounds or provenance policy")]
    InvalidMetadata,
    #[error("invalid ingestion transition")]
    InvalidTransition,
    #[error("identifier reuse conflicts with immutable knowledge")]
    IdentifierConflict,
    #[error("chunk provenance is invalid")]
    InvalidChunk,
    #[error("active version requires explicit supersession")]
    ActiveVersionConflict,
    #[error("persistence transaction failed")]
    PersistenceFailure,
}
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HashAlgorithm {
    Sha256,
}
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ContentHash {
    pub algorithm: HashAlgorithm,
    pub contract_version: ProtocolVersion,
    pub digest: String,
}
impl ContentHash {
    pub fn sha256(b: &[u8]) -> Self {
        Self {
            algorithm: HashAlgorithm::Sha256,
            contract_version: V1,
            digest: format!("{:x}", Sha256::digest(b)),
        }
    }
    pub fn new(a: HashAlgorithm, v: ProtocolVersion, d: String) -> Result<Self, KnowledgeError> {
        if v != V1
            || d.len() != 64
            || !d
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            Err(KnowledgeError::InvalidHash)
        } else {
            Ok(Self {
                algorithm: a,
                contract_version: v,
                digest: d,
            })
        }
    }
    pub fn verify(&self, b: &[u8]) -> Result<(), KnowledgeError> {
        if *self == Self::sha256(b) {
            Ok(())
        } else {
            Err(KnowledgeError::IntegrityMismatch)
        }
    }
}
#[derive(Deserialize)]
struct HashWire {
    algorithm: HashAlgorithm,
    contract_version: ProtocolVersion,
    digest: String,
}
impl<'de> Deserialize<'de> for ContentHash {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let w = HashWire::deserialize(d)?;
        Self::new(w.algorithm, w.contract_version, w.digest).map_err(serde::de::Error::custom)
    }
}
macro_rules! closed{($n:ident{$($v:ident),+})=>{#[derive(Clone,Copy,Debug,Eq,PartialEq,Serialize,Deserialize)]#[serde(rename_all="snake_case")]pub enum $n{$($v),+}}}
closed!(SourceType {
    Markdown,
    PlainText
});
closed!(SourceAuthority {
    Authoritative,
    Approved,
    Supplemental
});
closed!(SourceTrust {
    Verified,
    Reviewed,
    Unverified
});
closed!(SourceOrigin { Authored, Imported });
closed!(SourceStatus {
    Registered,
    Staged,
    Active,
    Failed,
    Stale,
    Superseded,
    RolledBack
});
closed!(KnowledgeVisibility {
    Public,
    Student,
    Instructor,
    Administrative,
    AssessmentProtected
});
closed!(MetadataProvenance {
    SourceAuthored,
    SystemInferred
});
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeScope {
    pub course_id: Option<CourseId>,
    pub lesson_id: Option<LessonId>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetadataEntry {
    pub value: String,
    pub provenance: MetadataProvenance,
    pub recorded_at: Timestamp,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeSource {
    pub contract_version: ProtocolVersion,
    pub source_id: KnowledgeSourceId,
    pub source_version: u64,
    pub source_type: SourceType,
    pub authority: SourceAuthority,
    pub trust: SourceTrust,
    pub origin: SourceOrigin,
    pub status: SourceStatus,
    pub visibility: KnowledgeVisibility,
    pub scope: KnowledgeScope,
    pub source_metadata: BTreeMap<String, MetadataEntry>,
    pub inferred_metadata: BTreeMap<String, MetadataEntry>,
    pub registered_at: Timestamp,
}
impl KnowledgeSource {
    pub fn validate(&self) -> Result<(), KnowledgeError> {
        if self.contract_version != V1 || self.source_version == 0 {
            return Err(KnowledgeError::UnsupportedSource);
        };
        metadata(&self.source_metadata, MetadataProvenance::SourceAuthored)?;
        metadata(&self.inferred_metadata, MetadataProvenance::SystemInferred)
    }
}
fn metadata(
    m: &BTreeMap<String, MetadataEntry>,
    p: MetadataProvenance,
) -> Result<(), KnowledgeError> {
    if m.len() > MAX_METADATA_ENTRIES
        || m.iter().any(|(k, v)| {
            k.is_empty()
                || k.len() > MAX_FIELD_BYTES
                || v.value.len() > MAX_FIELD_BYTES
                || v.provenance != p
        })
    {
        Err(KnowledgeError::InvalidMetadata)
    } else {
        Ok(())
    }
}
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeArtifact {
    pub contract_version: ProtocolVersion,
    pub artifact_id: KnowledgeArtifactId,
    pub source_id: KnowledgeSourceId,
    pub source_version: u64,
    pub media_type: String,
    pub encoding: String,
    pub byte_length: u64,
    pub content_hash: ContentHash,
    bytes: Vec<u8>,
    pub created_at: Timestamp,
}
impl std::fmt::Debug for KnowledgeArtifact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KnowledgeArtifact")
            .field("artifact_id", &self.artifact_id)
            .field("content_hash", &self.content_hash)
            .field("bytes", &"[REDACTED]")
            .finish()
    }
}
impl KnowledgeArtifact {
    pub fn new(
        id: KnowledgeArtifactId,
        s: &KnowledgeSource,
        media: &str,
        bytes: Vec<u8>,
        at: Timestamp,
    ) -> Result<Self, KnowledgeError> {
        s.validate()?;
        let expected = match s.source_type {
            SourceType::Markdown => "text/markdown",
            SourceType::PlainText => "text/plain",
        };
        if media != expected
            || bytes.is_empty()
            || bytes.len() > MAX_ARTIFACT_BYTES
            || std::str::from_utf8(&bytes).is_err()
        {
            return Err(KnowledgeError::InvalidArtifact);
        }
        Ok(Self {
            contract_version: V1,
            artifact_id: id,
            source_id: s.source_id,
            source_version: s.source_version,
            media_type: media.into(),
            encoding: "utf-8".into(),
            byte_length: bytes.len() as u64,
            content_hash: ContentHash::sha256(&bytes),
            bytes,
            created_at: at,
        })
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn text(&self) -> &str {
        std::str::from_utf8(&self.bytes).expect("validated UTF-8")
    }
    pub fn validate(&self) -> Result<(), KnowledgeError> {
        if self.contract_version != V1
            || self.encoding != "utf-8"
            || self.source_version == 0
            || self.bytes.is_empty()
            || self.bytes.len() > MAX_ARTIFACT_BYTES
            || self.byte_length != self.bytes.len() as u64
            || std::str::from_utf8(&self.bytes).is_err()
            || !matches!(self.media_type.as_str(), "text/markdown" | "text/plain")
        {
            return Err(KnowledgeError::InvalidArtifact);
        }
        self.content_hash.verify(&self.bytes)
    }
}
closed!(IngestionState {
    Registered,
    Parsing,
    Chunking,
    Staged,
    Active,
    Failed,
    Stale,
    Superseded,
    RolledBack
});
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IngestionJob {
    pub contract_version: ProtocolVersion,
    pub job_id: IngestionJobId,
    pub source_id: KnowledgeSourceId,
    pub source_version: u64,
    pub artifact_id: KnowledgeArtifactId,
    pub artifact_hash: ContentHash,
    pub state: IngestionState,
    pub revision: u64,
    pub updated_at: Timestamp,
}
impl IngestionJob {
    pub fn transition(&self, n: IngestionState, at: Timestamp) -> Result<Self, KnowledgeError> {
        if n == self.state
            || at < self.updated_at
            || matches!(
                self.state,
                IngestionState::Failed | IngestionState::Superseded | IngestionState::RolledBack
            )
        {
            return Err(KnowledgeError::InvalidTransition);
        }
        let ok = matches!(
            (self.state, n),
            (IngestionState::Registered, IngestionState::Parsing)
                | (IngestionState::Parsing, IngestionState::Chunking)
                | (IngestionState::Chunking, IngestionState::Staged)
                | (IngestionState::Staged, IngestionState::Active)
                | (
                    IngestionState::Active,
                    IngestionState::Stale | IngestionState::Superseded
                )
                | (
                    IngestionState::Stale,
                    IngestionState::Active | IngestionState::RolledBack
                )
                | (_, IngestionState::Failed)
        );
        if !ok {
            return Err(KnowledgeError::InvalidTransition);
        }
        let mut x = self.clone();
        x.state = n;
        x.updated_at = at;
        x.revision += 1;
        Ok(x)
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ByteRange {
    pub start: u64,
    pub end: u64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineRange {
    pub start: u64,
    pub end: u64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeChunk {
    pub contract_version: ProtocolVersion,
    pub chunk_id: KnowledgeChunkId,
    pub source_id: KnowledgeSourceId,
    pub source_version: u64,
    pub artifact_id: KnowledgeArtifactId,
    pub ordinal: u32,
    pub heading_path: Vec<String>,
    pub byte_range: ByteRange,
    pub line_range: LineRange,
    pub original_content_hash: ContentHash,
    pub chunk_content_hash: ContentHash,
    pub chunking_policy_version: ProtocolVersion,
}
impl KnowledgeChunk {
    pub fn content<'a>(&self, a: &'a KnowledgeArtifact) -> Result<&'a [u8], KnowledgeError> {
        check_chunk(self, a)?;
        Ok(&a.bytes[self.byte_range.start as usize..self.byte_range.end as usize])
    }
}
/// Returns exact original byte ranges; CRLF is retained and therefore has distinct hashes/offsets from LF.
pub fn structural_ranges(
    s: &KnowledgeSource,
    a: &KnowledgeArtifact,
) -> Result<Vec<(Vec<String>, ByteRange, LineRange)>, KnowledgeError> {
    s.validate()?;
    a.validate()?;
    if (s.source_id, s.source_version) != (a.source_id, a.source_version) {
        return Err(KnowledgeError::InvalidArtifact);
    }
    let t = a.text();
    let mut lines = vec![];
    let mut p = 0;
    for x in t.split_inclusive('\n') {
        lines.push((p, p + x.len(), x));
        p += x.len()
    }
    if p < t.len() {
        lines.push((p, t.len(), &t[p..]))
    }
    let mut starts = vec![0];
    let mut paths: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    let mut path = vec![];
    paths.insert(0, path.clone());
    if s.source_type == SourceType::Markdown {
        let mut fence = false;
        for (i, (_, _, line)) in lines.iter().enumerate() {
            let q = line.trim_end_matches(['\r', '\n']);
            if q.trim_start().starts_with("```") {
                fence = !fence;
                continue;
            }
            let n = q.bytes().take_while(|b| *b == b'#').count();
            if !fence && (1..=6).contains(&n) && q.as_bytes().get(n) == Some(&b' ') {
                if i > 0 {
                    starts.push(i)
                }
                path.truncate(n - 1);
                path.push(q[n + 1..].trim().into());
                paths.insert(i, path.clone());
            }
        }
    }
    let mut out = vec![];
    for (k, start) in starts.iter().enumerate() {
        let end = *starts.get(k + 1).unwrap_or(&lines.len());
        let mut i = *start;
        while i < end {
            let begin = lines[i].0;
            let mut j = i + 1;
            while j < end && lines[j].1 - begin <= MAX_CHUNK_BYTES {
                j += 1
            }
            let finish = lines[j - 1].1;
            if !t[begin..finish].trim().is_empty() {
                out.push((
                    paths
                        .range(..=i)
                        .next_back()
                        .map(|x| x.1.clone())
                        .unwrap_or_default(),
                    ByteRange {
                        start: begin as u64,
                        end: finish as u64,
                    },
                    LineRange {
                        start: (i + 1) as u64,
                        end: j as u64,
                    },
                ))
            }
            i = j
        }
    }
    Ok(out)
}
pub fn validate_chunks(cs: &[KnowledgeChunk], a: &KnowledgeArtifact) -> Result<(), KnowledgeError> {
    let (mut last, mut ids) = (0, BTreeSet::new());
    for (i, c) in cs.iter().enumerate() {
        if c.ordinal != i as u32 || c.byte_range.start < last || !ids.insert(c.chunk_id) {
            return Err(KnowledgeError::InvalidChunk);
        }
        check_chunk(c, a)?;
        last = c.byte_range.end
    }
    Ok(())
}
fn check_chunk(c: &KnowledgeChunk, a: &KnowledgeArtifact) -> Result<(), KnowledgeError> {
    let (s, e) = (c.byte_range.start as usize, c.byte_range.end as usize);
    if c.contract_version != V1
        || c.chunking_policy_version != V1
        || (c.source_id, c.source_version, c.artifact_id)
            != (a.source_id, a.source_version, a.artifact_id)
        || c.original_content_hash != a.content_hash
        || s >= e
        || e > a.bytes.len()
        || c.heading_path.len() > 6
        || c.chunk_content_hash != ContentHash::sha256(&a.bytes[s..e])
    {
        Err(KnowledgeError::InvalidChunk)
    } else {
        Ok(())
    }
}
closed!(Audience {
    StudentLearning,
    StudentAssessment,
    Instructor,
    Administrative,
    PointInTimeAdministrative
});
closed!(ExclusionReason {
    NotActive,
    AssessmentProtected,
    AudienceRestricted
});
pub fn exposure(s: &KnowledgeSource, a: Audience) -> Result<(), ExclusionReason> {
    if a != Audience::PointInTimeAdministrative && s.status != SourceStatus::Active {
        return Err(ExclusionReason::NotActive);
    }
    match (s.visibility, a) {
        (
            KnowledgeVisibility::AssessmentProtected,
            Audience::StudentLearning | Audience::StudentAssessment,
        ) => Err(ExclusionReason::AssessmentProtected),
        (
            KnowledgeVisibility::Instructor,
            Audience::StudentLearning | Audience::StudentAssessment,
        )
        | (
            KnowledgeVisibility::Administrative,
            Audience::StudentLearning | Audience::StudentAssessment | Audience::Instructor,
        ) => Err(ExclusionReason::AudienceRestricted),
        _ => Ok(()),
    }
}
pub trait KnowledgeUnitOfWork {
    fn commit(
        &mut self,
        s: KnowledgeSource,
        a: KnowledgeArtifact,
        j: IngestionJob,
        c: Vec<KnowledgeChunk>,
    ) -> Result<CommitOutcome, KnowledgeError>;
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommitOutcome {
    Inserted,
    NoOp,
}
#[derive(Clone, Default)]
pub struct InMemoryKnowledgeRepository {
    pub sources: BTreeMap<(KnowledgeSourceId, u64), KnowledgeSource>,
    pub artifacts: BTreeMap<KnowledgeArtifactId, KnowledgeArtifact>,
    pub jobs: BTreeMap<IngestionJobId, IngestionJob>,
    pub chunks: BTreeMap<KnowledgeChunkId, KnowledgeChunk>,
    pub fail_after: Option<usize>,
}
impl KnowledgeUnitOfWork for InMemoryKnowledgeRepository {
    fn commit(
        &mut self,
        s: KnowledgeSource,
        a: KnowledgeArtifact,
        j: IngestionJob,
        cs: Vec<KnowledgeChunk>,
    ) -> Result<CommitOutcome, KnowledgeError> {
        s.validate()?;
        a.validate()?;
        validate_chunks(&cs, &a)?;
        if (
            s.source_id,
            s.source_version,
            a.artifact_id,
            a.content_hash.clone(),
        ) != (
            a.source_id,
            a.source_version,
            j.artifact_id,
            j.artifact_hash.clone(),
        ) || j.source_id != s.source_id
            || j.source_version != s.source_version
        {
            return Err(KnowledgeError::IdentifierConflict);
        }
        let same = self.sources.get(&(s.source_id, s.source_version)) == Some(&s)
            && self.artifacts.get(&a.artifact_id) == Some(&a)
            && self.jobs.get(&j.job_id) == Some(&j)
            && cs.iter().all(|c| self.chunks.get(&c.chunk_id) == Some(c));
        if same {
            return Ok(CommitOutcome::NoOp);
        }
        if self.sources.contains_key(&(s.source_id, s.source_version))
            || self.artifacts.contains_key(&a.artifact_id)
            || self.jobs.contains_key(&j.job_id)
            || cs.iter().any(|c| self.chunks.contains_key(&c.chunk_id))
        {
            return Err(KnowledgeError::IdentifierConflict);
        }
        let mut n = self.clone();
        let mut stage = 0;
        macro_rules! step {
            () => {{
                stage += 1;
                if self.fail_after == Some(stage) {
                    return Err(KnowledgeError::PersistenceFailure);
                }
            }};
        }
        n.sources.insert((s.source_id, s.source_version), s);
        step!();
        n.artifacts.insert(a.artifact_id, a);
        step!();
        n.jobs.insert(j.job_id, j);
        step!();
        for c in cs {
            n.chunks.insert(c.chunk_id, c);
        }
        step!();
        *self = n;
        Ok(CommitOutcome::Inserted)
    }
}
