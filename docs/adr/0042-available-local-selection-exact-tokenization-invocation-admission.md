# ADR-0042: Available local selection, exact tokenization, invocation, and admission

- Status: Accepted
- Date: 2026-08-23
- Scope: Narrow Phase 4 synchronous caller-availability-gated host composition

## Context

ADR-0030 composes caller-supplied availability-gated explicit local selection with one invocation and strict admission, but does not tokenize the selected input. ADR-0041 composes explicit local selection with exact tokenization, capacity validation, invocation, and admission, but deliberately does not consume availability evidence. Callers need one opt-in operation joining these existing boundaries without duplicating or weakening their rules.

## Decision

`nexa-tutor` provides `select_available_local_model_tokenize_invoke_and_admit`. The caller supplies the immutable registry, invocation identity, unchanged selection requirements, caller-supplied availability snapshot, tokenization contract version, exactly one model-bound tokenizer, exact compilation result, trusted planning authority, context package, and citation result.

The operation first enforces unchanged ADR-0030/ADR-0041 explicit-local requirements. It delegates the exact compiled `ModelInput`, unchanged requirements, and exact snapshot to unchanged ADR-0029 `select_available_model`. It constructs the existing ADR-0022 request from the selected descriptor, caller invocation identity, exact compilation input, and unchanged requirements, then delegates exact tokenization, request-capacity validation, one selected-provider invocation, and strict admission to unchanged ADR-0040.

Missing and caller-marked-unavailable models are excluded before tokenization. Choosing the next canonically ordered, already-available eligible local model is the initial deterministic availability-gated selection, not fallback. Remote availability never authorizes remote execution. ADR-0027 conservative UTF-8 byte eligibility remains authoritative, and exact token counts do not affect selection, availability, ordering, or authorization.

Success returns ADR-0040's exact `TokenizedInvocationAdmissionResult` unchanged. A closed, content-free error distinguishes invalid explicit-local requirements, nested ADR-0029 `ModelAvailabilityError`, and nested ADR-0040 `TokenizedInvocationAdmissionError`. There is no second selection, re-selection, tokenizer call, provider call, retry, fallback, repair, regeneration, or recovery.

## Consequences and deferrals

ADRs 0029, 0030, 0040, and 0041 remain unchanged and independently callable. The caller snapshot is evidence only; this composition adds no probing, refresh, freshness/authenticity proof, authentication, clocks, monitoring, token-aware routing, automatic local-first behavior, remote authorization/filtering, concrete tokenizer/provider, inference, transport/networking, credentials/endpoints, semantic output validation, async/streaming, telemetry/persistence, partial truncation, or new dependency. NEXA-TUTOR-001 remains Baseline Draft, the known ingestion/context checklist inconsistency is preserved, and Phase 4 remains in progress.
