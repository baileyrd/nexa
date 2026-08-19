# ADR-0018: Exact hybrid fusion and provider-free reranking

- **Status:** Accepted
- **Date:** 2026-08-19
- **Specification:** NEXA-KNOW-001

## Context

ADRs 0016 and 0017 produce independently governed, reference-only lexical and vector results. Phase 4 needs a narrow composition policy without rerunning retrieval, comparing incompatible raw scores, or weakening either channel's governance.

## Decision

`nexa-knowledge` owns a synchronous V1 hybrid boundary above both retrieval policies. A caller supplies the shared query identity, both channel result identities, a distinct hybrid-result identity, all policy versions, selected embedding profile identity/fingerprint/dimension/metric, and final limit. Deserialization validates every invariant-bearing contract and rejects unknown fields and versions. Fusion first revalidates both complete channel results and all provenance. Canonical chunk identity is the join key; reassociation of a chunk with another artifact, source, or source version fails closed. Vector participation retains embedding-record and profile provenance.

V1 uses exact weighted reciprocal rank fusion. Ranks are one-origin and limited to the validated channel depth of 100. For each present channel `c`, its term is `weight_c / (rank_offset + rank_c)`; missing channels contribute exactly zero. Both weights are independently nonzero and at most 1,000; the offset is 0 through 1,000. Two terms are added using checked `u64` numerator/denominator arithmetic and reduced by greatest common divisor. Rational comparisons use checked cross multiplication. There is no rounding and raw lexical or vector scores never enter fusion arithmetic. Overflow rejects the operation. The policy treats lexical and vector channels symmetrically except for their explicit caller weights.

Lifecycle, assessment protection, audience, course, and lesson exclusions are eligibility gates. A governance exclusion in either channel is never overridden. A candidate opposed by a governance exclusion is contradictory and rejected rather than emitted. `no_matching_terms`, `missing_embedding`, and `profile_mismatch` record channel absence and do not veto a candidate from the other channel. Existing channel `result_limit` is also channel absence, not final truncation. Final hybrid limit exclusions are added only after reranking.

Reranking V1 is pure and provider-free: descending exact fusion rational, then ascending canonical chunk UUID. Authority and trust are not present in retrieval-result contracts and are therefore neither reconstructed nor used to alter relevance. The evidence records the policy, final one-origin rank, exact fraction, channel ranks and scores, participation, and the machine-readable rationale `exact_fusion_then_canonical_chunk_identity`. Input/map order has no effect.

Results contain references and numeric evidence only. They contain no query/source/chunk text, vectors, profile metadata, provider data, paths, endpoints, credentials, or executable content. Errors and custom `Debug` omit untrusted content.

## Consequences and deferred decisions

This implements deterministic hybrid fusion and policy reranking only. Learned or cross-encoder reranking, context assembly and packing, token budgeting, citations, tutor intelligence and tutor-response generation, model/embedding-provider integration, networking, vector databases, durable adapters, and authority/freshness relevance policies remain unimplemented. NEXA-KNOW-001 and NEXA-TUTOR-001 retain their current registry authority.
