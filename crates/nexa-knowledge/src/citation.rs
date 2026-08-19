//! Governed, provider-free citation resolution over a validated context package.
use crate::{ContentHash, ContextPackage, V1};
use nexa_domain::{
    CitationId, CitationSetId, ClaimId, ContextPackageId, HybridRetrievalResultId,
    KnowledgeArtifactId, KnowledgeChunkId, KnowledgeSourceId, ProtocolVersion, RetrievalQueryId,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use thiserror::Error;

pub const CITATION_POLICY_V1: ProtocolVersion = V1;
pub const SOURCE_LOCATION_POLICY_V1: ProtocolVersion = V1;
pub const MAX_CLAIMS: usize = 256;
pub const MAX_CITATIONS: usize = 1024;
pub const MAX_CITATIONS_PER_CLAIM: usize = 32;
pub const MAX_LOCATOR_COMPONENT_BYTES: usize = 255;
pub const MAX_SECTION_DEPTH: usize = 16;

macro_rules! wire {($name:ident{$($field:ident:$ty:ty),*$(,)?})=>{impl<'de> Deserialize<'de> for $name {fn deserialize<D:serde::Deserializer<'de>>(d:D)->Result<Self,D::Error>{#[derive(Deserialize)]#[serde(deny_unknown_fields)]struct W{$($field:$ty),*}let w=W::deserialize(d)?;let x=Self{$($field:w.$field),*};x.validate().map_err(serde::de::Error::custom)?;Ok(x)}}};}

/// A closed, non-executable source-location vocabulary. Ranges are half-open.
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceLocator {
    DocumentPage {
        page: u32,
    },
    SectionPath {
        path: Vec<String>,
    },
    Block {
        block_id: String,
    },
    LineRange {
        start: u64,
        end: u64,
    },
    ByteRange {
        start: u64,
        end: u64,
        content_length: u64,
    },
    CharacterRange {
        start: u64,
        end: u64,
        content_length: u64,
    },
}
impl fmt::Debug for SourceLocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SourceLocator([REDACTED])")
    }
}
impl SourceLocator {
    fn validate(&self) -> Result<(), CitationError> {
        let component = |s: &str| {
            !s.is_empty()
                && s.len() <= MAX_LOCATOR_COMPONENT_BYTES
                && !s.chars().any(char::is_control)
                && !s.contains("..")
                && !s.contains('/')
                && !s.contains('\\')
                && !s.contains("://")
        };
        match self {
            Self::DocumentPage { page } if *page > 0 => Ok(()),
            Self::SectionPath { path }
                if !path.is_empty()
                    && path.len() <= MAX_SECTION_DEPTH
                    && path.iter().all(|x| component(x) && x.trim() == x) =>
            {
                Ok(())
            }
            Self::Block { block_id } if component(block_id) && block_id.trim() == block_id => {
                Ok(())
            }
            Self::LineRange { start, end } if *start > 0 && start <= end => Ok(()),
            Self::ByteRange {
                start,
                end,
                content_length,
            }
            | Self::CharacterRange {
                start,
                end,
                content_length,
            } if start < end && end <= content_length => Ok(()),
            _ => Err(CitationError::MalformedLocator),
        }
    }

    fn validate_against_content(&self, content: &str) -> Result<(), CitationError> {
        match self {
            Self::ByteRange {
                end,
                content_length,
                ..
            } => {
                let actual =
                    u64::try_from(content.len()).map_err(|_| CitationError::InvalidEvidence)?;
                if *content_length != actual || *end > actual {
                    return Err(CitationError::InvalidEvidence);
                }
            }
            Self::CharacterRange {
                end,
                content_length,
                ..
            } => {
                let actual = u64::try_from(content.chars().count())
                    .map_err(|_| CitationError::InvalidEvidence)?;
                if *content_length != actual || *end > actual {
                    return Err(CitationError::InvalidEvidence);
                }
            }
            // Line ranges are one-based and inclusive. `str::lines` also gives the
            // intended result for a final line without a trailing newline.
            Self::LineRange { end, .. } => {
                let actual = u64::try_from(content.lines().count())
                    .map_err(|_| CitationError::InvalidEvidence)?;
                if *end > actual {
                    return Err(CitationError::InvalidEvidence);
                }
            }
            _ => {}
        }
        Ok(())
    }
}
impl<'de> Deserialize<'de> for SourceLocator {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
        enum Wire {
            DocumentPage {
                page: u32,
            },
            SectionPath {
                path: Vec<String>,
            },
            Block {
                block_id: String,
            },
            LineRange {
                start: u64,
                end: u64,
            },
            ByteRange {
                start: u64,
                end: u64,
                content_length: u64,
            },
            CharacterRange {
                start: u64,
                end: u64,
                content_length: u64,
            },
        }
        let locator = match Wire::deserialize(deserializer)? {
            Wire::DocumentPage { page } => Self::DocumentPage { page },
            Wire::SectionPath { path } => Self::SectionPath { path },
            Wire::Block { block_id } => Self::Block { block_id },
            Wire::LineRange { start, end } => Self::LineRange { start, end },
            Wire::ByteRange {
                start,
                end,
                content_length,
            } => Self::ByteRange {
                start,
                end,
                content_length,
            },
            Wire::CharacterRange {
                start,
                end,
                content_length,
            } => Self::CharacterRange {
                start,
                end,
                content_length,
            },
        };
        locator.validate().map_err(serde::de::Error::custom)?;
        Ok(locator)
    }
}

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceLocationEvidence {
    pub citation_id: CitationId,
    pub chunk_id: KnowledgeChunkId,
    pub artifact_id: KnowledgeArtifactId,
    pub source_id: KnowledgeSourceId,
    pub source_version: u64,
    pub content_fingerprint: ContentHash,
    pub context_position: u32,
    pub locator_policy_version: ProtocolVersion,
    pub locator: SourceLocator,
}
impl fmt::Debug for SourceLocationEvidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceLocationEvidence")
            .field("citation_id", &self.citation_id)
            .field("chunk_id", &self.chunk_id)
            .field("locator", &"[REDACTED]")
            .finish()
    }
}
impl SourceLocationEvidence {
    fn validate(&self) -> Result<(), CitationError> {
        if self.source_version == 0 || self.context_position == 0 {
            return Err(CitationError::InvalidEvidence);
        }
        if self.locator_policy_version != SOURCE_LOCATION_POLICY_V1 {
            return Err(CitationError::UnsupportedContract);
        }
        self.locator.validate()
    }
}
wire!(SourceLocationEvidence {
    citation_id: CitationId,
    chunk_id: KnowledgeChunkId,
    artifact_id: KnowledgeArtifactId,
    source_id: KnowledgeSourceId,
    source_version: u64,
    content_fingerprint: ContentHash,
    context_position: u32,
    locator_policy_version: ProtocolVersion,
    locator: SourceLocator
});

type SupportKey = (
    KnowledgeChunkId,
    KnowledgeArtifactId,
    KnowledgeSourceId,
    u64,
    ContentHash,
    u32,
    ProtocolVersion,
    SourceLocator,
);

fn support_key(e: &SourceLocationEvidence) -> SupportKey {
    (
        e.chunk_id,
        e.artifact_id,
        e.source_id,
        e.source_version,
        e.content_fingerprint.clone(),
        e.context_position,
        e.locator_policy_version,
        e.locator.clone(),
    )
}

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimCitationRequest {
    pub claim_id: ClaimId,
    /// Semantically unordered evidence. Resolution canonicalizes it by context position and locator.
    pub evidence: Vec<SourceLocationEvidence>,
}
impl fmt::Debug for ClaimCitationRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClaimCitationRequest")
            .field("claim_id", &self.claim_id)
            .field("evidence_count", &self.evidence.len())
            .finish()
    }
}
impl ClaimCitationRequest {
    fn validate(&self) -> Result<(), CitationError> {
        if self.evidence.len() > MAX_CITATIONS_PER_CLAIM {
            return Err(CitationError::InvalidLimit);
        }
        self.evidence
            .iter()
            .try_for_each(SourceLocationEvidence::validate)
    }
}
wire!(ClaimCitationRequest{claim_id:ClaimId,evidence:Vec<SourceLocationEvidence>});

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CitationRequest {
    pub contract_version: ProtocolVersion,
    pub citation_set_id: CitationSetId,
    pub context_package_id: ContextPackageId,
    pub hybrid_result_id: HybridRetrievalResultId,
    pub query_id: RetrievalQueryId,
    pub citation_policy_version: ProtocolVersion,
    pub locator_policy_version: ProtocolVersion,
    pub governance_policy_version: ProtocolVersion,
    pub integrity_profile_version: ProtocolVersion,
    pub maximum_citations: usize,
    pub maximum_citations_per_claim: usize,
    pub claims: Vec<ClaimCitationRequest>,
}
impl fmt::Debug for CitationRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CitationRequest")
            .field("citation_set_id", &self.citation_set_id)
            .field("claim_count", &self.claims.len())
            .finish()
    }
}
impl CitationRequest {
    pub fn validate(&self) -> Result<(), CitationError> {
        if self.contract_version != V1
            || self.citation_policy_version != CITATION_POLICY_V1
            || self.locator_policy_version != SOURCE_LOCATION_POLICY_V1
            || self.governance_policy_version != V1
            || self.integrity_profile_version != V1
        {
            return Err(CitationError::UnsupportedContract);
        }
        if self.claims.is_empty()
            || self.claims.len() > MAX_CLAIMS
            || self.maximum_citations == 0
            || self.maximum_citations > MAX_CITATIONS
            || self.maximum_citations_per_claim == 0
            || self.maximum_citations_per_claim > MAX_CITATIONS_PER_CLAIM
        {
            return Err(CitationError::InvalidLimit);
        }
        let mut claims = BTreeSet::new();
        let mut citations = BTreeMap::new();
        for (claim_index, claim) in self.claims.iter().enumerate() {
            claim.validate()?;
            if !claims.insert(claim.claim_id) {
                return Err(CitationError::IdentityConflict);
            }
            for evidence in &claim.evidence {
                let support = support_key(evidence);
                if let Some((prior_claim, prior_support)) =
                    citations.insert(evidence.citation_id, (claim_index, support.clone()))
                {
                    if prior_claim != claim_index || prior_support != support {
                        return Err(CitationError::IdentityConflict);
                    }
                }
                if evidence.locator_policy_version != self.locator_policy_version {
                    return Err(CitationError::ProvenanceMismatch);
                }
            }
        }
        Ok(())
    }
}
wire!(CitationRequest{contract_version:ProtocolVersion,citation_set_id:CitationSetId,context_package_id:ContextPackageId,hybrid_result_id:HybridRetrievalResultId,query_id:RetrievalQueryId,citation_policy_version:ProtocolVersion,locator_policy_version:ProtocolVersion,governance_policy_version:ProtocolVersion,integrity_profile_version:ProtocolVersion,maximum_citations:usize,maximum_citations_per_claim:usize,claims:Vec<ClaimCitationRequest>});

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CitationResolutionStatus {
    Resolved,
    Unresolved,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnresolvedCitationReason {
    NoSuppliedEvidence,
}

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedCitation {
    pub citation_id: CitationId,
    pub citation_position: u32,
    pub context_position: u32,
    pub chunk_id: KnowledgeChunkId,
    pub artifact_id: KnowledgeArtifactId,
    pub source_id: KnowledgeSourceId,
    pub source_version: u64,
    pub content_fingerprint: ContentHash,
    pub locator_policy_version: ProtocolVersion,
    pub locator: SourceLocator,
}
impl fmt::Debug for ResolvedCitation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResolvedCitation")
            .field("citation_id", &self.citation_id)
            .field("chunk_id", &self.chunk_id)
            .field("locator", &"[REDACTED]")
            .finish()
    }
}
impl ResolvedCitation {
    fn validate(&self) -> Result<(), CitationError> {
        if self.citation_position == 0
            || self.context_position == 0
            || self.source_version == 0
            || self.locator_policy_version != V1
        {
            return Err(CitationError::InvalidResult);
        }
        self.locator
            .validate()
            .map_err(|_| CitationError::InvalidResult)
    }
}
wire!(ResolvedCitation {
    citation_id: CitationId,
    citation_position: u32,
    context_position: u32,
    chunk_id: KnowledgeChunkId,
    artifact_id: KnowledgeArtifactId,
    source_id: KnowledgeSourceId,
    source_version: u64,
    content_fingerprint: ContentHash,
    locator_policy_version: ProtocolVersion,
    locator: SourceLocator
});

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimCitationResult {
    pub claim_id: ClaimId,
    pub claim_position: u32,
    pub status: CitationResolutionStatus,
    pub unresolved_reason: Option<UnresolvedCitationReason>,
    pub citations: Vec<ResolvedCitation>,
}
impl ClaimCitationResult {
    fn validate(&self) -> Result<(), CitationError> {
        let resolved = !self.citations.is_empty();
        if self.claim_position == 0
            || resolved != (self.status == CitationResolutionStatus::Resolved)
            || resolved == self.unresolved_reason.is_some()
        {
            return Err(CitationError::InvalidResult);
        }
        let mut ids = BTreeSet::new();
        let mut previous = None;
        for (i, c) in self.citations.iter().enumerate() {
            c.validate()?;
            if c.citation_position as usize != i + 1 || !ids.insert(c.citation_id) {
                return Err(CitationError::InvalidResult);
            }
            let key = (c.context_position, c.locator.clone(), c.citation_id);
            if previous.as_ref().is_some_and(|p| p >= &key) {
                return Err(CitationError::InvalidResult);
            }
            previous = Some(key);
        }
        Ok(())
    }
}
wire!(ClaimCitationResult{claim_id:ClaimId,claim_position:u32,status:CitationResolutionStatus,unresolved_reason:Option<UnresolvedCitationReason>,citations:Vec<ResolvedCitation>});

/// Reference-only anchor for the caller's ordered claims.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimOrderAnchor {
    pub claim_id: ClaimId,
    pub claim_position: u32,
}
impl ClaimOrderAnchor {
    fn validate(&self) -> Result<(), CitationError> {
        (self.claim_position > 0)
            .then_some(())
            .ok_or(CitationError::InvalidResult)
    }
}
wire!(ClaimOrderAnchor {
    claim_id: ClaimId,
    claim_position: u32
});

/// Immutable provenance mapping copied from the governed context package.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContextAnchor {
    pub context_position: u32,
    pub chunk_id: KnowledgeChunkId,
    pub artifact_id: KnowledgeArtifactId,
    pub source_id: KnowledgeSourceId,
    pub source_version: u64,
    pub content_fingerprint: ContentHash,
}
impl ContextAnchor {
    fn validate(&self) -> Result<(), CitationError> {
        if self.context_position == 0 || self.source_version == 0 {
            Err(CitationError::InvalidResult)
        } else {
            Ok(())
        }
    }
}
wire!(ContextAnchor {
    context_position: u32,
    chunk_id: KnowledgeChunkId,
    artifact_id: KnowledgeArtifactId,
    source_id: KnowledgeSourceId,
    source_version: u64,
    content_fingerprint: ContentHash
});

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CitationResult {
    pub contract_version: ProtocolVersion,
    pub citation_set_id: CitationSetId,
    pub context_package_id: ContextPackageId,
    pub hybrid_result_id: HybridRetrievalResultId,
    pub query_id: RetrievalQueryId,
    pub citation_policy_version: ProtocolVersion,
    pub locator_policy_version: ProtocolVersion,
    pub governance_policy_version: ProtocolVersion,
    pub integrity_profile_version: ProtocolVersion,
    pub maximum_citations: usize,
    pub maximum_citations_per_claim: usize,
    pub claim_order_anchor: Vec<ClaimOrderAnchor>,
    pub context_anchor: Vec<ContextAnchor>,
    pub claims: Vec<ClaimCitationResult>,
}
impl CitationResult {
    pub fn validate(&self) -> Result<(), CitationError> {
        if self.contract_version != V1
            || self.citation_policy_version != V1
            || self.locator_policy_version != V1
            || self.governance_policy_version != V1
            || self.integrity_profile_version != V1
        {
            return Err(CitationError::UnsupportedContract);
        }
        if self.claims.is_empty()
            || self.claims.len() > MAX_CLAIMS
            || self.maximum_citations == 0
            || self.maximum_citations > MAX_CITATIONS
            || self.maximum_citations_per_claim == 0
            || self.maximum_citations_per_claim > MAX_CITATIONS_PER_CLAIM
        {
            return Err(CitationError::InvalidResult);
        }
        if self.claim_order_anchor.len() != self.claims.len() || self.context_anchor.is_empty() {
            return Err(CitationError::InvalidResult);
        }
        let mut anchor_claim_ids = BTreeSet::new();
        for (i, anchor) in self.claim_order_anchor.iter().enumerate() {
            anchor.validate()?;
            if anchor.claim_position as usize != i + 1 || !anchor_claim_ids.insert(anchor.claim_id)
            {
                return Err(CitationError::InvalidResult);
            }
        }
        let mut context_by_position = BTreeMap::new();
        let mut context_chunk_ids = BTreeSet::new();
        for (i, anchor) in self.context_anchor.iter().enumerate() {
            anchor.validate()?;
            if anchor.context_position as usize != i + 1
                || context_by_position
                    .insert(anchor.context_position, anchor)
                    .is_some()
                || !context_chunk_ids.insert(anchor.chunk_id)
            {
                return Err(CitationError::InvalidResult);
            }
        }
        let mut claim_ids = BTreeSet::new();
        let mut citation_ids = BTreeSet::new();
        let mut total = 0;
        for (i, claim) in self.claims.iter().enumerate() {
            claim.validate()?;
            if claim.claim_position as usize != i + 1
                || !claim_ids.insert(claim.claim_id)
                || self.claim_order_anchor[i].claim_id != claim.claim_id
                || claim.citations.len() > self.maximum_citations_per_claim
            {
                return Err(CitationError::InvalidResult);
            }
            total += claim.citations.len();
            for c in &claim.citations {
                if !citation_ids.insert(c.citation_id) {
                    return Err(CitationError::InvalidResult);
                }
                let Some(anchor) = context_by_position.get(&c.context_position) else {
                    return Err(CitationError::InvalidResult);
                };
                if (
                    c.chunk_id,
                    c.artifact_id,
                    c.source_id,
                    c.source_version,
                    &c.content_fingerprint,
                ) != (
                    anchor.chunk_id,
                    anchor.artifact_id,
                    anchor.source_id,
                    anchor.source_version,
                    &anchor.content_fingerprint,
                ) {
                    return Err(CitationError::InvalidResult);
                }
            }
        }
        if total > self.maximum_citations {
            return Err(CitationError::InvalidResult);
        }
        Ok(())
    }
}
wire!(CitationResult{contract_version:ProtocolVersion,citation_set_id:CitationSetId,context_package_id:ContextPackageId,hybrid_result_id:HybridRetrievalResultId,query_id:RetrievalQueryId,citation_policy_version:ProtocolVersion,locator_policy_version:ProtocolVersion,governance_policy_version:ProtocolVersion,integrity_profile_version:ProtocolVersion,maximum_citations:usize,maximum_citations_per_claim:usize,claim_order_anchor:Vec<ClaimOrderAnchor>,context_anchor:Vec<ContextAnchor>,claims:Vec<ClaimCitationResult>});

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum CitationError {
    #[error("unsupported citation contract or policy")]
    UnsupportedContract,
    #[error("citation limits violate bounds")]
    InvalidLimit,
    #[error("citation identity is duplicated or conflicting")]
    IdentityConflict,
    #[error("citation provenance does not match governed context")]
    ProvenanceMismatch,
    #[error("source locator is malformed or unsafe")]
    MalformedLocator,
    #[error("source-location evidence is invalid")]
    InvalidEvidence,
    #[error("citation result is invalid")]
    InvalidResult,
}

pub fn resolve_citations(
    request: &CitationRequest,
    package: &ContextPackage,
) -> Result<CitationResult, CitationError> {
    request.validate()?;
    package
        .validate()
        .map_err(|_| CitationError::InvalidResult)?;
    if (
        request.context_package_id,
        request.hybrid_result_id,
        request.query_id,
    ) != (
        package.context_package_id,
        package.hybrid_result_id,
        package.query_id,
    ) || request.governance_policy_version != package.governance_policy_version
    {
        return Err(CitationError::ProvenanceMismatch);
    }
    let mut claims = Vec::with_capacity(request.claims.len());
    let mut resolved_total = 0usize;
    for (claim_index, claim) in request.claims.iter().enumerate() {
        let mut unique_support = BTreeMap::<SupportKey, SourceLocationEvidence>::new();
        for e in &claim.evidence {
            let Some(chunk) = package.included.iter().find(|x| x.chunk_id == e.chunk_id) else {
                return Err(CitationError::ProvenanceMismatch);
            };
            let Some(content) = package.content.iter().find(|x| {
                x.chunk_id == e.chunk_id && x.final_context_position == e.context_position
            }) else {
                return Err(CitationError::ProvenanceMismatch);
            };
            if (
                e.context_position,
                e.artifact_id,
                e.source_id,
                e.source_version,
                &e.content_fingerprint,
            ) != (
                chunk.final_context_position,
                chunk.artifact_id,
                chunk.source_id,
                chunk.source_version,
                &chunk.content_fingerprint,
            ) {
                return Err(CitationError::ProvenanceMismatch);
            }
            e.locator.validate_against_content(&content.content)?;
            let key = support_key(e);
            unique_support
                .entry(key)
                .and_modify(|survivor| {
                    if e.citation_id < survivor.citation_id {
                        survivor.citation_id = e.citation_id;
                    }
                })
                .or_insert_with(|| e.clone());
        }
        if unique_support.len() > request.maximum_citations_per_claim {
            return Err(CitationError::InvalidLimit);
        }
        resolved_total = resolved_total
            .checked_add(unique_support.len())
            .ok_or(CitationError::InvalidLimit)?;
        if resolved_total > request.maximum_citations {
            return Err(CitationError::InvalidLimit);
        }
        let mut evidence = unique_support.into_values().collect::<Vec<_>>();
        evidence.sort_by_key(|e| (e.context_position, e.locator.clone(), e.citation_id));
        let citations = evidence
            .into_iter()
            .enumerate()
            .map(|(i, e)| ResolvedCitation {
                citation_id: e.citation_id,
                citation_position: (i + 1) as u32,
                context_position: e.context_position,
                chunk_id: e.chunk_id,
                artifact_id: e.artifact_id,
                source_id: e.source_id,
                source_version: e.source_version,
                content_fingerprint: e.content_fingerprint,
                locator_policy_version: e.locator_policy_version,
                locator: e.locator,
            })
            .collect::<Vec<_>>();
        let resolved = !citations.is_empty();
        claims.push(ClaimCitationResult {
            claim_id: claim.claim_id,
            claim_position: (claim_index + 1) as u32,
            status: if resolved {
                CitationResolutionStatus::Resolved
            } else {
                CitationResolutionStatus::Unresolved
            },
            unresolved_reason: if resolved {
                None
            } else {
                Some(UnresolvedCitationReason::NoSuppliedEvidence)
            },
            citations,
        });
    }
    let result = CitationResult {
        contract_version: request.contract_version,
        citation_set_id: request.citation_set_id,
        context_package_id: request.context_package_id,
        hybrid_result_id: request.hybrid_result_id,
        query_id: request.query_id,
        citation_policy_version: request.citation_policy_version,
        locator_policy_version: request.locator_policy_version,
        governance_policy_version: request.governance_policy_version,
        integrity_profile_version: request.integrity_profile_version,
        maximum_citations: request.maximum_citations,
        maximum_citations_per_claim: request.maximum_citations_per_claim,
        claim_order_anchor: request
            .claims
            .iter()
            .enumerate()
            .map(|(i, claim)| ClaimOrderAnchor {
                claim_id: claim.claim_id,
                claim_position: (i + 1) as u32,
            })
            .collect(),
        context_anchor: package
            .included
            .iter()
            .map(|chunk| ContextAnchor {
                context_position: chunk.final_context_position,
                chunk_id: chunk.chunk_id,
                artifact_id: chunk.artifact_id,
                source_id: chunk.source_id,
                source_version: chunk.source_version,
                content_fingerprint: chunk.content_fingerprint.clone(),
            })
            .collect(),
        claims,
    };
    result.validate()?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    const SET: &str = "00000000-0000-0000-0000-000000000071";
    const CLAIM1: &str = "00000000-0000-0000-0000-000000000072";
    const CLAIM2: &str = "00000000-0000-0000-0000-000000000073";
    const CIT1: &str = "00000000-0000-0000-0000-000000000074";
    const CIT2: &str = "00000000-0000-0000-0000-000000000075";
    const PACKAGE: &str = "00000000-0000-0000-0000-000000000051";
    const HYBRID: &str = "00000000-0000-0000-0000-000000000041";
    const QUERY: &str = "00000000-0000-0000-0000-000000000011";
    const CHUNK1: &str = "00000000-0000-0000-0000-000000000031";
    const CHUNK2: &str = "00000000-0000-0000-0000-000000000032";
    const ART1: &str = "00000000-0000-0000-0000-000000000021";
    const ART2: &str = "00000000-0000-0000-0000-000000000022";
    const SOURCE: &str = "00000000-0000-0000-0000-000000000001";
    fn hash(s: &str) -> Value {
        serde_json::to_value(ContentHash::sha256(s.as_bytes())).unwrap()
    }
    fn package() -> ContextPackage {
        serde_json::from_value(json!({"contract_version":"1.0","context_package_id":PACKAGE,"hybrid_result_id":HYBRID,"query_id":QUERY,"assembly_policy_version":"1.0","governance_policy_version":"1.0","tokenizer_profile_id":"test","tokenizer_fingerprint":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","maximum_tokens":20,"maximum_chunk_tokens":null,"maximum_chunks":null,"ordering_policy":"hybrid_rank","packing_policy":"ranked_greedy_whole_chunk","accounting":{"policy_version":"1.0","fixed_package_overhead":0,"per_chunk_overhead":0,"separator_tokens":0,"metadata_reference_overhead":0},"hybrid_candidate_count":2,"used_tokens":2,"remaining_tokens":18,"included":[{"chunk_id":CHUNK1,"artifact_id":ART1,"source_id":SOURCE,"source_version":1,"hybrid_rank":1,"content_fingerprint":hash("a"),"exact_token_count":1,"overhead_tokens":0,"total_contribution":1,"final_context_position":1},{"chunk_id":CHUNK2,"artifact_id":ART2,"source_id":SOURCE,"source_version":1,"hybrid_rank":2,"content_fingerprint":hash("b"),"exact_token_count":1,"overhead_tokens":0,"total_contribution":1,"final_context_position":2}],"exclusions":[],"content":[{"chunk_id":CHUNK1,"final_context_position":1,"content":"a"},{"chunk_id":CHUNK2,"final_context_position":2,"content":"b"}]})).unwrap()
    }
    fn evidence(id: &str, chunk: &str, artifact: &str, position: u32, locator: Value) -> Value {
        json!({"citation_id":id,"chunk_id":chunk,"artifact_id":artifact,"source_id":SOURCE,"source_version":1,"content_fingerprint":if chunk==CHUNK1{hash("a")}else{hash("b")},"context_position":position,"locator_policy_version":"1.0","locator":locator})
    }
    fn request(claims: Value) -> CitationRequest {
        serde_json::from_value(json!({"contract_version":"1.0","citation_set_id":SET,"context_package_id":PACKAGE,"hybrid_result_id":HYBRID,"query_id":QUERY,"citation_policy_version":"1.0","locator_policy_version":"1.0","governance_policy_version":"1.0","integrity_profile_version":"1.0","maximum_citations":10,"maximum_citations_per_claim":5,"claims":claims})).unwrap()
    }

    #[test]
    fn resolves_orders_and_round_trips_without_inference() {
        let r = request(json!([
            {"claim_id":CLAIM1,"evidence":[evidence(CIT2,CHUNK2,ART2,2,json!({"kind":"section_path","path":["API","Types"]})),evidence(CIT1,CHUNK1,ART1,1,json!({"kind":"document_page","page":2}))]},
            {"claim_id":CLAIM2,"evidence":[]}
        ]));
        let out = resolve_citations(&r, &package()).unwrap();
        assert_eq!(out.claims[0].citations[0].context_position, 1);
        assert_eq!(out.claims[1].status, CitationResolutionStatus::Unresolved);
        assert_eq!(
            out.claims[1].unresolved_reason,
            Some(UnresolvedCitationReason::NoSuppliedEvidence)
        );
        let wire = serde_json::to_string(&out).unwrap();
        assert_eq!(serde_json::from_str::<CitationResult>(&wire).unwrap(), out);
    }
    #[test]
    fn supports_authorized_closed_locator_forms() {
        for locator in [
            json!({"kind":"document_page","page":1}),
            json!({"kind":"section_path","path":["Intro"]}),
            json!({"kind":"block","block_id":"p-1"}),
            json!({"kind":"line_range","start":1,"end":1}),
            json!({"kind":"byte_range","start":0,"end":1,"content_length":1}),
            json!({"kind":"character_range","start":0,"end":1,"content_length":1}),
        ] {
            let r = request(
                json!([{"claim_id":CLAIM1,"evidence":[evidence(CIT1,CHUNK1,ART1,1,locator)]}]),
            );
            assert!(resolve_citations(&r, &package()).is_ok());
        }
    }
    #[test]
    fn rejects_malformed_locators_provenance_and_limits() {
        for locator in [
            json!({"kind":"line_range","start":3,"end":2}),
            json!({"kind":"byte_range","start":1,"end":1,"content_length":1}),
            json!({"kind":"block","block_id":"https://private"}),
        ] {
            let mut v =
                serde_json::to_value(request(json!([{"claim_id":CLAIM1,"evidence":[]}]))).unwrap();
            v["claims"][0]["evidence"] = json!([evidence(CIT1, CHUNK1, ART1, 1, locator)]);
            assert!(serde_json::from_value::<CitationRequest>(v).is_err());
        }
        let mut v=serde_json::to_value(request(json!([{"claim_id":CLAIM1,"evidence":[evidence(CIT1,CHUNK1,ART1,1,json!({"kind":"document_page","page":1}))]}]))).unwrap();
        v["claims"][0]["evidence"][0]["context_position"] = json!(2);
        let bad: CitationRequest = serde_json::from_value(v).unwrap();
        assert_eq!(
            resolve_citations(&bad, &package()),
            Err(CitationError::ProvenanceMismatch)
        );
        let mut v =
            serde_json::to_value(request(json!([{"claim_id":CLAIM1,"evidence":[]}]))).unwrap();
        v["maximum_citations"] = json!(0);
        assert!(serde_json::from_value::<CitationRequest>(v).is_err());
    }
    #[test]
    fn standalone_validation_rejects_tampering_and_unknown_fields() {
        let r = request(
            json!([{"claim_id":CLAIM1,"evidence":[evidence(CIT1,CHUNK1,ART1,1,json!({"kind":"document_page","page":1}))]},{"claim_id":CLAIM2,"evidence":[evidence(CIT2,CHUNK2,ART2,2,json!({"kind":"block","block_id":"b"}))]}]),
        );
        let out = resolve_citations(&r, &package()).unwrap();
        let base = serde_json::to_value(out).unwrap();
        let mut v = base.clone();
        v["extra"] = json!(true);
        assert!(serde_json::from_value::<CitationResult>(v).is_err());
        let mut v = base.clone();
        v["claims"].as_array_mut().unwrap().swap(0, 1);
        assert!(serde_json::from_value::<CitationResult>(v).is_err());
        let mut v = base.clone();
        v["claims"][0]["status"] = json!("unresolved");
        assert!(serde_json::from_value::<CitationResult>(v).is_err());
        let mut v = base;
        v["governance_policy_version"] = json!("2.0");
        assert!(serde_json::from_value::<CitationResult>(v).is_err());
    }
    #[test]
    fn rejects_reassociation_exclusions_duplicates_and_redacts() {
        let ev = evidence(
            CIT1,
            CHUNK1,
            ART1,
            1,
            json!({"kind":"section_path","path":["secret-heading"]}),
        );
        let r = request(json!([{"claim_id":CLAIM1,"evidence":[ev.clone()]}]));
        assert!(!format!("{:?}", r).contains("secret-heading"));
        let mut absent = serde_json::to_value(&r).unwrap();
        absent["claims"][0]["evidence"][0]["chunk_id"] =
            json!("00000000-0000-0000-0000-000000000099");
        let absent: CitationRequest = serde_json::from_value(absent).unwrap();
        assert_eq!(
            resolve_citations(&absent, &package()),
            Err(CitationError::ProvenanceMismatch)
        );
        let mut duplicate = r.clone();
        duplicate.claims.push(ClaimCitationRequest {
            claim_id: CLAIM2.parse().unwrap(),
            evidence: duplicate.claims[0].evidence.clone(),
        });
        assert_eq!(duplicate.validate(), Err(CitationError::IdentityConflict));
    }

    #[test]
    fn binds_ranges_to_short_multibyte_context_content() {
        let mut package = package();
        let content = "é\n猫";
        let fingerprint = ContentHash::sha256(content.as_bytes());
        package.content[0].content = content.into();
        package.included[0].content_fingerprint = fingerprint.clone();

        for locator in [
            json!({"kind":"byte_range","start":0,"end":7,"content_length":100}),
            json!({"kind":"byte_range","start":0,"end":6,"content_length":7}),
            json!({"kind":"character_range","start":0,"end":4,"content_length":100}),
            json!({"kind":"character_range","start":0,"end":3,"content_length":4}),
            json!({"kind":"line_range","start":1,"end":3}),
        ] {
            let mut ev = evidence(CIT1, CHUNK1, ART1, 1, locator);
            ev["content_fingerprint"] = serde_json::to_value(&fingerprint).unwrap();
            let request = request(json!([{"claim_id":CLAIM1,"evidence":[ev]}]));
            assert_eq!(
                resolve_citations(&request, &package),
                Err(CitationError::InvalidEvidence)
            );
        }
        for locator in [
            json!({"kind":"byte_range","start":0,"end":6,"content_length":6}),
            json!({"kind":"character_range","start":0,"end":3,"content_length":3}),
            json!({"kind":"line_range","start":1,"end":2}),
        ] {
            let mut ev = evidence(CIT1, CHUNK1, ART1, 1, locator);
            ev["content_fingerprint"] = serde_json::to_value(&fingerprint).unwrap();
            assert!(resolve_citations(
                &request(json!([{"claim_id":CLAIM1,"evidence":[ev]}])),
                &package
            )
            .is_ok());
        }
    }

    #[test]
    fn deduplicates_support_deterministically_and_rejects_conflicting_ids() {
        let locator = json!({"kind":"document_page","page":1});
        let first = evidence(CIT1, CHUNK1, ART1, 1, locator.clone());
        let second = evidence(CIT2, CHUNK1, ART1, 1, locator);
        let forward = request(
            json!([{"claim_id":CLAIM1,"evidence":[first.clone(),second.clone(),first.clone()]}]),
        );
        let reverse = request(json!([{"claim_id":CLAIM1,"evidence":[second,first]}]));
        let a = resolve_citations(&forward, &package()).unwrap();
        let b = resolve_citations(&reverse, &package()).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.claims[0].citations.len(), 1);
        assert_eq!(a.claims[0].citations[0].citation_id, CIT1.parse().unwrap());

        let mut conflicting = reverse.clone();
        let mut reused = conflicting.claims[0].evidence[0].clone();
        reused.chunk_id = CHUNK2.parse().unwrap();
        reused.artifact_id = ART2.parse().unwrap();
        reused.context_position = 2;
        reused.content_fingerprint = ContentHash::sha256(b"b");
        conflicting.claims[0].evidence.push(reused);
        assert_eq!(conflicting.validate(), Err(CitationError::IdentityConflict));
    }

    #[test]
    fn standalone_anchors_reject_coordinated_claim_and_provenance_tampering() {
        let request = request(json!([
            {"claim_id":CLAIM1,"evidence":[evidence(CIT1,CHUNK1,ART1,1,json!({"kind":"document_page","page":1}))]},
            {"claim_id":CLAIM2,"evidence":[evidence(CIT2,CHUNK2,ART2,2,json!({"kind":"document_page","page":1}))]}
        ]));
        let base = serde_json::to_value(resolve_citations(&request, &package()).unwrap()).unwrap();

        let mut swapped = base.clone();
        swapped["claims"].as_array_mut().unwrap().swap(0, 1);
        swapped["claims"][0]["claim_position"] = json!(1);
        swapped["claims"][1]["claim_position"] = json!(2);
        assert!(serde_json::from_value::<CitationResult>(swapped).is_err());

        for (field, replacement) in [
            ("context_position", json!(2)),
            ("chunk_id", json!(CHUNK2)),
            ("artifact_id", json!(ART2)),
            ("source_id", json!("00000000-0000-0000-0000-000000000099")),
            ("source_version", json!(2)),
            ("content_fingerprint", hash("b")),
        ] {
            let mut tampered = base.clone();
            tampered["claims"][0]["citations"][0][field] = replacement;
            assert!(
                serde_json::from_value::<CitationResult>(tampered).is_err(),
                "{field}"
            );
        }
    }
}
