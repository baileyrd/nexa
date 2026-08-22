# ADR-0040: Exact tokenization, single-attempt invocation, and admission composition

- Status: Accepted
- Date: 2026-08-22
- Scope: Narrow Phase 4 synchronous host composition

## Context

ADR-0039 composes exact tokenization evidence creation with request-capacity validation but does not invoke a provider. ADR-0038 accepts existing evidence before one provider invocation and strict admission but does not create or return that evidence. Callers need one opt-in operation with a single governed ordering across both compositions, without reproducing their validation, hashing, replay, capacity, or admission rules.

## Decision

`nexa-tutor` provides `tokenize_invoke_and_admit_model_output_with_token_capacity`. The caller supplies the tokenization contract version, a `ModelInputTokenizer`, a `LanguageModelProvider`, the exact existing `ModelRequest`, compilation evidence, trusted planning authority, context package, and citation result.

The operation completes unchanged shared ADR-0025/ADR-0024 admission preflight before consuming either supplied dependency. It then delegates to unchanged ADR-0039 `tokenize_and_validate_model_request_capacity` using the supplied provider's exact descriptor. Only successful tokenization and request-capacity validation permits exactly one provider invocation. The exact response enters unchanged post-preflight ADR-0024 admission.

Success returns both the exact generated `ModelInputTokenizationEvidence` and the unchanged `AdmissionResult`. A closed, content-free error distinguishes shared preflight, tokenization/request-capacity composition, normalized provider invocation, and post-invocation admission failures. There is no retry or second tokenizer/provider call.

## Consequences and deferrals

The ADR-0025 and ADR-0036 through ADR-0039 APIs and behavior remain unchanged and independently callable. This composition adds no concrete tokenizer algorithm or provider; selection, authorization, availability, routing, automatic local-first policy, fallback, retry, repair, regeneration, truncation, rewriting, summarization, provider-usage reconciliation, inference, transport, networking, endpoint, credential, filesystem, configuration, semantic validation, general privacy policy, async/streaming, telemetry, persistence, or new dependency. NEXA-TUTOR-001 remains Baseline Draft, the known ingestion/context-assembly checklist inconsistency is preserved, and Phase 4 remains in progress.
