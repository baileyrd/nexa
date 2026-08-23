# ADR-0047: Local-only selection with reported-usage-validated exact-tokenization invocation and admission

- Status: Accepted
- Date: 2026-08-23
- Scope: Narrow Phase 4 synchronous host composition

## Context

ADR-0041 composes explicit local-only ADR-0027 selection and exact ADR-0022 request construction with ADR-0040 exact-tokenization invocation/admission. ADR-0046 separately inserts ADR-0045 optional reported-usage reconciliation into that exact-tokenization path before admission. Callers need one opt-in operation that joins the unchanged local-only selection boundary to the unchanged usage-validated path without reproducing their ordering or changing either contract.

## Decision

`nexa-tutor` provides `select_local_model_tokenize_invoke_validate_reported_usage_and_admit`. It accepts exactly ADR-0041's caller-supplied inputs and returns the existing `TokenizedInvocationAdmissionResult`.

The operation first enforces unchanged ADR-0028/ADR-0041 explicit-local requirements before selection or dependency consumption. It delegates unchanged ADR-0027 deterministic selection over the exact compiled `ModelInput`, constructs the existing ADR-0022 request exactly as ADR-0041 does, and delegates the selected provider, exact request, tokenizer, and governed inputs to unchanged ADR-0046. Its closed, content-free error preserves invalid-local-requirement, exact selection, and complete nested ADR-0046 failure categories.

Selection remains governed by conservative UTF-8 byte eligibility; exact tokenization begins only after selection and cannot influence it. The operation is deterministic and single-attempt. Only the selected local provider can consume one outcome; non-selected and remote providers remain untouched.

## Consequences and deferrals

ADRs 0022 through 0046 remain unchanged and independently callable. This composition reuses explicit local-only selection and establishes only structural association and equality for optional reported input usage. It grants no tokenizer correctness, provider truth, billing or cost accuracy, output-token correctness or evidence, authenticity, freshness, telemetry authority, or semantic correctness.

No availability lookup, authorization, filtering, routing, automatic local-first policy, retry, fallback, repair, regeneration, I/O, clock, randomness, telemetry export, persistence, concrete tokenizer/provider, inference, transport/networking, endpoint/credential work, tools, async/streaming, semantic validation, or new dependency is added. NEXA-TUTOR-001 remains Baseline Draft, Phase 4 remains in progress, and the known ingestion/context checklist inconsistency is preserved.
