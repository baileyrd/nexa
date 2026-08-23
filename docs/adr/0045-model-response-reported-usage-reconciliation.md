# ADR-0045: Model-response reported-usage reconciliation

- **Status:** Accepted
- **Date:** 2026-08-23
- **Specification:** NEXA-TUTOR-001

## Context

ADR-0022 permits a model response to carry optional provider-reported usage and treats that usage as evidence rather than verified tokenizer truth. ADR-0036 independently records exact-model input-tokenization evidence, while deliberately deferring any association with later provider-reported usage. A narrow boundary is needed to compare already-supplied contracts without invoking either dependency or changing their semantics.

## Decision

`nexa-tutor` owns a synchronous, provider-neutral `validate_model_response_reported_usage` operation. It accepts the exact existing `ModelDescriptor`, `ModelRequest`, `ModelResponse`, and `ModelInputTokenizationEvidence` and returns only success or a closed, content-free error.

Validation is ordered and fail-closed: unchanged request validation against the descriptor; unchanged response validation against the request; unchanged tokenization-evidence association with the descriptor and exact request input; then, only when reported usage is present, equality between its input-token count and the validated evidence count. Request, response, and evidence errors retain distinct categories. A count inequality has one category.

Absent usage succeeds because ADR-0022 intentionally makes it optional. Existing response validation remains authoritative for the maximum reported output-token bound. No output-token evidence exists, so this operation does not verify exact output usage.

The operation is pure and non-consuming. It performs no tokenization, provider invocation, mutation, registry lookup, selection, filtering, authorization, availability or capacity decision, retry, fallback, I/O, clock, randomness, telemetry export, or persistence.

## Consequences

Success establishes only structural association and equality between two supplied input-count reports. It does not prove tokenizer correctness, provider honesty, authenticity or freshness, billing or cost accuracy, output-token correctness, or semantic correctness. Provider-reported usage remains optional evidence and gains no telemetry or billing authority.

No existing model, usage, request, response, tokenization, provider, invocation, admission, selection, authorization, filtering, or capacity API changes. No concrete dependency, output-token evidence, accumulation, quota, pricing, networking, inference, retry, or unrelated orchestration is introduced. NEXA-TUTOR-001 remains Baseline Draft and Phase 4 remains in progress.
