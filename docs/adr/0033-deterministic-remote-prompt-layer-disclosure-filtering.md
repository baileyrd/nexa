# ADR-0033: Deterministic remote prompt layer disclosure filtering

- **Status:** Accepted
- **Date:** 2026-08-21
- **Scope:** Narrow Phase 4 caller-directed structural disclosure boundary

## Context

ADR-0023 defines the authoritative prompt-layer inventory, classifications, mandatory layers, canonical order, compilation, and replay evidence. ADR-0031 authorizes an exact compiled prompt for remote selection, and ADR-0032 may invoke that authorized prompt, but neither filters it.

The `docs/specifications/17-privacy` namespace is reserved and contains no approved privacy specification. NEXA-TUTOR-001 remains Baseline Draft. Those sources do not authorize a complete privacy subsystem, semantic sensitivity inference, or content-level redaction. A narrow accepted decision is therefore required for deterministic structural filtering only.

## Decision

`nexa-tutor` adds a provider-neutral, non-invoking V1 contract in which a trusted caller explicitly supplies exactly one include-or-omit rule for every ADR-0023 `PromptLayerKind`, targeting exactly `ApprovedRemote` or `RestrictedRemote`. Rules are canonicalized by the ordinary constructor; standalone wire input must already be complete, unique, and in ADR-0023 canonical order.

ADR-0023 remains the sole layer inventory and mandatory-layer authority. Its six mandatory layers—platform contract, Nexa identity, policy, pedagogy, student input, and output contract—cannot be omitted. A policy denying one fails closed. An optional layer is either retained byte-for-byte with its exact kind and classification or omitted in full. There is no partial truncation, rewriting, normalization, summarization, semantic inspection, anonymization, field-level filtering, or substring redaction.

Filtering first compiles the complete source through unchanged ADR-0023 behavior to validate it and establish its local replay anchor. It then preserves source versions and limits, removes only explicitly omitted source-present optional layers, and delegates the result to unchanged `compile_prompt`. It performs no selection, authorization, availability check, provider lookup or consumption, invocation, networking, filesystem or clock access, randomness, or persistence.

The result contains the exact filtered compilation, the applied content-free policy, and content-free evidence binding the supported versions, target privacy class, canonical source-present included and omitted kinds, policy anchor, source compilation anchor, filtered compilation anchor, and a final SHA-256 replay anchor over those fields. The source anchor binds independently retained source compilation evidence; it cannot reconstruct omitted content bytes by itself.

## Consequences and limits

This establishes caller-directed whole-layer disclosure filtering and deterministic filtered prompt compilation only. It does not prove that a caller's decision is correct, infer sensitivity, enforce general privacy policy, redact content, establish provider authenticity/freshness, or transmit a prompt.

ADR-0031 and ADR-0032 APIs and behavior remain unchanged. There is no automatic integration: a later reviewed composition may require ADR-0033 evidence before remote execution. NEXA-TUTOR-001 remains Baseline Draft; the privacy namespace remains reserved and unimplemented; `partial truncation` and all existing provider, inference, routing, fallback, semantic-validation, networking, async/streaming, telemetry, and persistence work remain deferred.
