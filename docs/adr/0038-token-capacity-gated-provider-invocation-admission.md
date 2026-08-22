# ADR-0038: Token-capacity-gated provider invocation and admission

- Status: Accepted
- Date: 2026-08-22
- Scope: Narrow Phase 4 opt-in synchronous composition

## Context

ADR-0025 composes shared host preflight, one explicitly supplied provider invocation, and strict ADR-0024 admission. ADR-0037 independently validates an existing request against existing exact-model ADR-0036 tokenization evidence without invocation. Callers need an opt-in composition that requires both gates before consuming the provider while preserving both existing APIs and ADR-0022's mandatory conservative byte validation.

## Decision

`nexa-tutor` provides `invoke_and_admit_model_output_with_token_capacity`. The caller supplies the provider, exact existing request, existing ADR-0036 evidence, compilation evidence, trusted planning authority, context package, and citation result.

The operation first completes the unchanged shared ADR-0025 admission preflight. It then delegates to unchanged ADR-0037 validation using the supplied provider descriptor, exact request, and exact existing evidence. Consequently descriptor/request/input/evidence mismatches, invalid evidence, checked-add overflow, exact capacity excess, and every existing preflight failure occur before provider consumption. ADR-0037 still runs mandatory ADR-0022 conservative byte validation, and equality with the exact context-window capacity succeeds.

Only successful host validation permits exactly one provider call. The exact response then enters the unchanged post-preflight ADR-0024 admission path. A closed, content-free error distinguishes shared preflight, token-capacity, normalized provider, and post-invocation admission failures. Success returns the existing `AdmissionResult`.

The operation accepts no tokenizer, creates no tokenization evidence, and does not alter ADR-0025 or ADR-0037. It performs no retry, fallback, repair, truncation, rewriting, summarization, or provider-usage reconciliation.

## Consequences and deferrals

This opt-in boundary adds no concrete tokenizer or provider, inference, transport, networking, endpoints, credentials, selection, token-aware routing, automatic local-first policy, fallback, semantic validation, general privacy policy, async/streaming, telemetry, or persistence. Existing selection, invocation/admission, and capacity-validation APIs remain independently callable. NEXA-TUTOR-001 remains Baseline Draft and Phase 4 remains in progress.
