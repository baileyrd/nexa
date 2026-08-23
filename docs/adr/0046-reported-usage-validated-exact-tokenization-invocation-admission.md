# ADR-0046: Reported-usage-validated exact-tokenization invocation and admission

- Status: Accepted
- Date: 2026-08-23
- Scope: Narrow Phase 4 synchronous host composition

## Context

ADR-0040 composes full admission preflight, exact tokenization and capacity validation, one provider invocation, and strict admission. ADR-0045 separately validates the exact request, response, and generated tokenization evidence before reconciling optional provider-reported input usage. Callers need one opt-in ordering that applies ADR-0045 to ADR-0040's exact artifacts before admission without changing either independent contract.

## Decision

`nexa-tutor` provides `tokenize_invoke_validate_reported_usage_and_admit_model_output_with_token_capacity`. It accepts exactly ADR-0040's caller-supplied inputs and returns the existing `TokenizedInvocationAdmissionResult`.

The operation runs unchanged shared admission preflight before dependency consumption; delegates exact evidence construction and capacity validation to unchanged ADR-0039; invokes the exact supplied provider once with the exact request; applies unchanged ADR-0045 validation to that response and generated evidence; and only then passes the same response to unchanged post-preflight ADR-0024 admission. Its closed, content-free error preserves preflight, tokenization/capacity, normalized invocation, reported-usage, and admission categories in that order. It never retries, falls back, or consumes either dependency twice.

Absent reported usage remains valid. Present reported input usage must equal the generated evidence count. Existing response validation remains authoritative for output-usage limits.

## Consequences and deferrals

ADR-0022 through ADR-0045 remain unchanged and independently callable. This composition enforces only structural association and equality for optional reported input usage. It grants no tokenizer correctness, provider truth, billing or cost accuracy, output-token evidence or exact verification, authenticity, freshness, telemetry authority, or semantic correctness.

No registry lookup, selection, filtering, authorization, availability check, routing, retry, fallback, repair, regeneration, I/O, clock, randomness, telemetry export, persistence, concrete tokenizer/provider, inference, networking, async/streaming, or new dependency is added. NEXA-TUTOR-001 remains Baseline Draft, Phase 4 remains in progress, and the known ingestion/context checklist inconsistency is preserved.
