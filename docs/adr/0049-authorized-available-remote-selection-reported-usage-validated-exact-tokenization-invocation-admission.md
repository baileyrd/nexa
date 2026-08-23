# ADR-0049: Authorized available remote selection with reported-usage-validated exact-tokenization invocation and admission

- Status: Accepted
- Date: 2026-08-23
- Scope: Narrow Phase 4 synchronous caller-authorized, caller-availability-gated host composition

## Context

ADR-0043 composes ADR-0031 authorized available remote selection and exact request construction with ADR-0040. ADR-0046 separately adds optional reported-usage reconciliation before admission. Callers need one opt-in operation joining those unchanged boundaries without reproducing their ordering or changing either contract.

## Decision

`nexa-tutor` provides `select_authorized_available_remote_model_tokenize_invoke_validate_reported_usage_and_admit`. It accepts exactly ADR-0043's caller-supplied inputs and returns the existing `TokenizedInvocationAdmissionResult`.

The operation delegates the exact registry, requirements, availability snapshot, authorization, and compilation to unchanged ADR-0031. ADR-0031 remains the sole permission boundary; this decision introduces no authentication or authorization-policy change. It constructs the existing ADR-0022 request exactly as ADR-0043 does, then delegates the selected original shared provider, exact request, caller tokenizer, and governed inputs to unchanged ADR-0046. Its closed, content-free error preserves the complete ADR-0031 failure or complete nested ADR-0046 failure.

Authorization is caller-supplied policy evidence only and grants no authenticity or freshness. Availability is caller-supplied eligibility evidence only and grants no authenticity or freshness, monitoring, or recovery authority. Conservative UTF-8 byte eligibility and deterministic privacy and identity ordering precede exact tokenization. Exact token counts and reported usage cannot affect authorization, availability, selection, privacy preference, or ordering. The operation is deterministic, single-attempt, and consumes only the selected provider.

## Consequences and deferrals

ADRs 0022 through 0048 remain unchanged and independently callable. Optional reported input usage establishes structural association and equality only. It grants no tokenizer/provider truth, billing/cost, output-token evidence or correctness, telemetry, or semantic authority.

No authentication, authorization-policy, secret, endpoint, credential, configuration, transport, networking, filtering, availability acquisition or refresh, token-aware selection, routing, fallback, retry, repair, regeneration, concrete tokenizer/provider, inference, usage accumulation, pricing, quota, latency, telemetry export, persistence, tools, async/streaming, semantic validation, new dependency, or unrelated refactor is added. NEXA-TUTOR-001 remains Baseline Draft, Phase 4 remains in progress, and the known ingestion/context checklist inconsistency is preserved.
