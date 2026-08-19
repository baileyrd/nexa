# ADR-0016: Deterministic governed lexical retrieval

- **Status:** Accepted
- **Date:** 2026-08-19
- **Specification:** NEXA-KNOW-001

## Context

ADR-0015 established validated source, artifact, chunk, lifecycle, integrity, provenance, and visibility contracts while deliberately deferring retrieval. Phase 4 now needs a narrow retrieval capability that can be replayed and tested without selecting a model, vector store, network service, async runtime, or durable database. The reconstructed NEXA-KNOW-001 baseline requires lexical retrieval but does not prescribe a first scoring formula. NEXA-TUTOR-001 remains a Baseline Draft and does not authorize implementing tutor response generation in this increment.

Queries and source bytes are untrusted. Retrieval must not interpret markup, links, paths, commands, or embedded instructions, and it must not expose their text through results, errors, diagnostics, or `Debug` output.

## Decision

`nexa-knowledge` owns version `1.0` retrieval query, filter, candidate, result, score-evidence, exclusion, and error contracts. Query and result UUIDs are caller supplied canonical `nexa-domain` identities. Invariant-bearing JSON rejects unknown fields and unknown versions. Query text is limited to 4 KiB, 128 terms, and 256 UTF-8 bytes per normalized term. Result limits are nonzero and capped at 100.

The synchronous retrieval engine consumes an owned, repository-neutral read-port snapshot rather than public mutable repository maps. Snapshot construction sorts all records and validates identities, source/artifact/chunk relationships, artifact hashes, chunk hashes and structural provenance. Every represented source identity must have exactly one `Active` version. Only chunks belonging to that active version are indexed; other committed versions receive deterministic `not_active` exclusions. Missing, duplicate, conflicting, or corrupted records fail closed.

Governance and exact optional course and lesson filters run before document-frequency calculation and scoring. Student audiences cannot retrieve assessment-protected material. Results and exclusions carry only source, artifact, chunk, and version references; content remains in the validated snapshot.

Version `1.0` tokenization iterates Unicode scalar values, treats consecutive alphanumeric scalars as terms, applies Rust's Unicode lowercase mapping scalar by scalar, and splits on every other scalar. It performs no compatibility, accent, markup, URL, path, whitespace, or line-ending normalization. LF and CRLF therefore tokenize equivalently where their line endings are merely delimiters, while original artifact hashes and provenance remain distinct under ADR-0015.

The 256-byte normalized-term limit is a query validation boundary. During corpus indexing, an entire alphanumeric run whose normalized UTF-8 representation exceeds that limit is omitted; indexing resumes at the next delimiter. This deterministic omission prevents unusual source content from invalidating the snapshot while ensuring that an overlong run cannot partially match a query.

For each eligible document and normalized query term, V1 records query term frequency (`qtf`), chunk term frequency (`tf`), document frequency (`df`), and eligible document count (`N`). Its exact integer contribution is:

```text
qtf * tf * (N - df + 1)
```

A candidate score is the checked sum of its positive term contributions and is serialized as a finite positive integral `f64`. This numeric representation leaves room for future score families while preventing NaN, infinity, negative, fractional, or overflow-derived V1 scores. Evidence carries a SHA-256 reference to each normalized query term rather than copying untrusted text and is ordered by that term hash. Candidates sort by descending score, then ascending chunk UUID, artifact UUID, source UUID, and source version. Exclusions sort by the same stable identity references. Sorting and validation make results independent of record insertion and map iteration order.

## Consequences and deferred decisions

This is deliberately a small lexical policy, not a claim that the reconstructed hybrid architecture is complete. Its score is explainable and deterministic but is not BM25 and does not add authority or freshness weighting. A future policy version may introduce another lexical formula without silently changing V1 replay.

Embeddings, vector retrieval, hybrid fusion, reranking, authority/freshness ranking, context assembly, context packing, token budgeting, citation resolution, tutor intelligence, tutor-response generation, model/provider integration, networking, durable adapters, storage-engine selection, and multi-process snapshot semantics remain unresolved and unimplemented. This ADR does not promote NEXA-KNOW-001 or NEXA-TUTOR-001 beyond their registry authority.
