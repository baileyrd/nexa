# ADR-0017: Governed embeddings and deterministic vector retrieval

**Status:** Accepted

## Context

ADR-0015 established validated knowledge provenance and ADR-0016 established lexical policy V1. Phase 4 also requires semantic retrieval where caller-supplied embeddings are available, without prematurely choosing a provider, SDK, vector database, network, async runtime, or durable store.

## Decision

`nexa-domain` owns canonical embedding-record and embedding-profile UUID identities. `nexa-knowledge` owns immutable, validated profile, vector, chunk-binding, query, evidence, exclusion, result, read-port, and snapshot contracts. Embedding contract V1 and vector policy V1 are explicit and independent of lexical policy V1.

A profile contains a stable caller identity, bounded provider-neutral model-family identifier and inert metadata, dimension, signed-16-bit scalar representation, dot-product metric, no-normalization requirement, and contract version. It cannot contain credentials, endpoints, SDK objects, or hooks. Its lowercase SHA-256 fingerprint covers the contract version, model family, dimension, scalar representation, metric, normalization, and length-prefixed ordered metadata. The profile identity itself is not fingerprinted: it names the governed descriptor, while the fingerprint detects conflicting behavior under that identity.

V1 vectors are non-empty arrays of exactly `dimension` JSON integers in the inclusive `i16` range, with a maximum dimension of 4096. This avoids floating-point encoding and NaN, infinity, signed-zero, rounding, and platform-library behavior. Similarity is the exact signed dot product. Components are multiplied as `i64` and accumulated with checked `i64` addition from dimension index zero upward. Greater signed values rank first; equal scores break by canonical chunk UUID ascending. The evidence returns the exact `i64` value and policy version. These bounds make V1 accumulation safe, while checked arithmetic remains mandatory. This is deterministic integer replay, not a broader claim about provider output reproducibility.

Every embedding binds its own canonical identity to chunk, artifact, source, source version, original artifact hash, exact chunk hash, profile identity and fingerprint, dimension, contract version, vector, and governed creation timestamp. Snapshot loading validates the complete ingestion corpus, exactly one Active version for every represented source, all bindings and hashes, unique canonical identities, one artifact per source version, and at most one embedding per chunk/profile. Orphans, conflicts, corruption, unknown profiles, and duplicate identities fail closed. Record order has no semantic effect.

Retrieval applies lifecycle, audience, assessment protection, course, and lesson rules before scoring. Only the selected identical profile and dimension can be scored. Active eligible chunks lacking that profile receive an explicit `missing_embedding` exclusion; this is the V1 fail-visible missing-record policy rather than silently changing corpus membership. Non-selected profile records remain validated but do not participate. Results contain canonical references, integer evidence, and exclusions only—never text, query vectors, or embedding payloads. A nonzero caller limit capped at 100 is applied after stable ranking, with `result_limit` exclusions.

## Consequences and deferred decisions

The synchronous read port returns an owned point-in-time record set, and the in-memory snapshot is an adapter suitable for deterministic validation and tests. Provider output quality, quantization production, alternate representations, cosine or distance metrics, normalization, durable snapshot semantics, and re-embedding coordination require new versioned decisions.

Hybrid fusion, lexical-policy changes, reranking, context assembly, context packing, token budgeting, citations, tutor intelligence and tutor-response generation, embedding/model provider integration, networking, vector databases, and durable adapters remain unimplemented. NEXA-KNOW-001 and NEXA-TUTOR-001 retain their current registry authority and are not promoted by this ADR.
