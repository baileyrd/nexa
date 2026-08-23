# ADR-0048: Available local selection with reported-usage-validated exact-tokenization invocation and admission

- Status: Accepted
- Date: 2026-08-23
- Scope: Narrow Phase 4 synchronous caller-availability-gated host composition

## Context

ADR-0042 composes caller-supplied availability-gated explicit local-only selection and exact request construction with ADR-0040. ADR-0046 separately adds optional reported-usage reconciliation before admission to that exact-tokenization path. Callers need one opt-in operation joining those unchanged boundaries without reproducing their ordering or changing either contract.

## Decision

`nexa-tutor` provides `select_available_local_model_tokenize_invoke_validate_reported_usage_and_admit`. It accepts exactly ADR-0042's caller-supplied inputs and returns the existing `TokenizedInvocationAdmissionResult`.

The operation first enforces unchanged explicit-local requirements. It delegates the exact compiled input, unchanged requirements, and exact caller snapshot to unchanged ADR-0029 selection, constructs the ADR-0022 request exactly as ADR-0042 does, and delegates the selected provider, exact request, tokenizer, and governed inputs to unchanged ADR-0046. Its closed, content-free error preserves invalid-local-requirement, complete availability-selection, and complete nested ADR-0046 failure categories.

Caller-supplied availability is eligibility evidence only and grants no freshness or authenticity, health-monitoring, recovery, or authorization authority. Omitted and caller-marked-unavailable models are excluded; remote availability does not authorize remote execution. Conservative UTF-8 byte eligibility and deterministic selection precede exact tokenization. The operation is single-attempt and consumes only one tokenizer outcome and one selected-provider outcome after their respective gates.

## Consequences and deferrals

ADRs 0022 through 0047 remain unchanged and independently callable. This composition establishes only structural association and equality for optional reported input usage. It grants no tokenizer or provider truth, billing or cost accuracy, output-token correctness or evidence, telemetry authority, or semantic correctness.

No availability acquisition, probing, refresh, monitoring, clock, mutable health service, remote authorization/filtering, token-aware selection, routing, fallback, retry, repair, regeneration, concrete tokenizer/provider, inference, transport/networking, endpoint/credential/configuration work, authentication change, general privacy policy, usage accumulation, pricing, quota, latency, telemetry export, persistence, tools, async/streaming, semantic validation, or new dependency is added. NEXA-TUTOR-001 remains Baseline Draft, Phase 4 remains in progress, and the known ingestion/context checklist inconsistency is preserved.
