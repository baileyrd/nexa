# ADR-0019: Governed deterministic context assembly

- **Status:** Accepted
- **Date:** 2026-08-19
- **Specification:** NEXA-KNOW-001

## Context

ADRs 0015–0018 establish governed provenance and a validated, reference-only hybrid rank. The next Phase 4 slice must turn caller-supplied chunk bytes and exact tokenizer counts into a replayable context without choosing a tokenizer/provider, weakening upstream exclusions, or inventing source authority. The reconstructed specification requires permissions before tutor context and reproducible retrieval, but does not prescribe accounting constants or truncation.

## Decision

`nexa-knowledge` owns a synchronous, provider-free assembly boundary. `ContextAssemblyRequest` binds package, hybrid-result and query identities; V1 assembly and governance versions; tokenizer identity and SHA-256-shaped profile fingerprint; exact limits; rank ordering; whole-chunk packing; and an explicit accounting policy. `GovernedChunkMaterial` is the separate content-bearing input. It binds canonical chunk/artifact/source/source-version provenance, content SHA-256, tokenizer provenance, caller-attested exact token count, and inert content. Debug output redacts content. Errors expose classifications only.

Accounting uses checked `u64` arithmetic. Used tokens equal fixed package overhead plus, for every included chunk, content tokens + per-chunk overhead + separator tokens + metadata/reference overhead. Remaining tokens equal maximum minus used. No floating-point estimate or truncation exists. V1 scans validated hybrid candidates in rank order and greedily includes a whole chunk when it passes the optional per-chunk and chunk-count limits and its full contribution fits. A size exclusion does not stop scanning: a lower-ranked fitting chunk may be included. Missing material is recorded. The exclusion precedence for ranked candidates is missing material, per-chunk limit, chunk-count limit, then token budget; upstream exclusions remain excluded and are represented after ranked decisions with their upstream classification.

Material outside the candidate set, reassociated provenance, duplicate identities, altered content, tokenizer mismatch, and supplying an upstream-excluded chunk fail the operation. The output carries content separately from reference/accounting evidence. Its wire validator denies unknown fields, revalidates every content hash, requires consecutive hybrid ranks across all ranked decisions, consecutive final positions, unique identities, exact contributions and totals, persisted limits and policies, and preserved upstream classifications. Thus a serialized package is independently tamper-detecting and replayable under the caller-attested tokenizer contract. Exact counts cannot prove tokenizer execution; the immutable tokenizer profile fingerprint is the audit boundary.

## Consequences and deferred decisions

Stable hybrid order is authoritative; assembly never consults lexical/vector scores, authority, freshness, or a model. Empty packages are valid when fixed overhead fits. Content remains untrusted and must not enter logs, errors, credentials, paths, endpoints, vectors, or provider metadata.

Partial truncation, provider tokenization, tokenizer SDKs, citation resolution, tutor behavior/responses, LLM calls, networking, async execution, databases, vector stores, and durable adapters remain deferred. NEXA-KNOW-001 retains its registry authority and Phase 4 remains in progress.
