# ADR-0020: Governed deterministic citation resolution

- **Status:** Accepted
- **Date:** 2026-08-19
- **Specifications:** NEXA-KNOW-001; NEXA-TUTOR-001 (citation constraints only)

## Context

NEXA-KNOW-001 assigns citation resolution to the knowledge subsystem and prohibits model-invented citation metadata. NEXA-TUTOR-001 requires citation identifiers from context and anticipates externally significant claim identities, but its reconstructed text does not define a wire protocol, locator normalization, or deterministic failure rules. ADR-0019 supplies the validated governed context boundary. This ADR records the narrow V1 interpretation rather than silently treating retrieval relevance as entailment.

## Decision

`nexa-knowledge` owns a synchronous, provider-free resolver. Canonical `CitationId` identifies each caller-supplied citation. New `CitationSetId` and `ClaimId` UUID values are canonical because result sets and claims are externally addressed and replayed; the resolver never manufactures them. A request binds its set, context-package, hybrid-result, and query identities, citation/locator/governance/integrity versions, ordered claims, and explicit limits.

Each evidence record binds an included chunk's exact context position, artifact, source, source version, content fingerprint, locator policy, and a closed locator. V1 supports one-based document pages; trimmed bounded section paths; bounded block identifiers; inclusive one-based line ranges; and half-open byte or character ranges with explicit content bounds. Section/block values reject controls, traversal/path separators, and URI syntax. External URLs, filesystem paths, timestamps/media locators, executable schemes, metadata, quotes, and spans are not authorized. Locator values are untrusted and redacted from `Debug` and errors.

Claim order is caller order. Within a claim evidence is semantically unordered and canonicalized by context position, normalized locator value, then canonical citation identity. Citation positions are consecutive. Citation identities are globally unique, provenance disagreement fails closed, and all cited chunks must be in `ContextPackage.included`; context and upstream exclusions can never be reintroduced or relabeled. The same included chunk may support multiple claims, and distinct normalized locators may address one chunk. The resolver does not infer support, authority, or entailment. A valid claim with no evidence is retained as `unresolved/no_supplied_evidence`; absent or contradictory evidence is an error. Limits are checked before resolution.

Every invariant-bearing struct uses validating deserialization and denies unknown fields. Results retain identities, versions, limits, positions, provenance, fingerprints, normalized locators, status, and unresolved reason. Standalone validation replays ordering, uniqueness, limits, and status classification without request text or context content. Results contain references only, not source/context text.

## Consequences and deferred decisions

The result establishes that a caller explicitly associated a claim with governed context material; it does not establish semantic truth. The baseline's statement that a citation “shall support” a claim cannot be mechanically satisfied without a future authorized entailment policy, so semantic citation fidelity remains an explicitly unresolved specification boundary.

Quote extraction and quoted spans, semantic entailment, authority adjudication, stable external URLs, media timestamps, tutor response generation, LLM/provider integration, networking, async infrastructure, databases, vector stores, persistence, and durable adapters remain deferred. Phase 4 remains in progress.
