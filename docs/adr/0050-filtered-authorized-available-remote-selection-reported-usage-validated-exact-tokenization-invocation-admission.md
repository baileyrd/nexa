# ADR-0050: Filtered authorized available remote selection with reported-usage-validated exact-tokenization invocation and admission

- Status: Accepted
- Date: 2026-08-23
- Scope: Narrow Phase 4 synchronous filtered, caller-authorized, caller-availability-gated host composition

## Context

ADR-0044 composes ADR-0034 filtered authorized available remote selection and exact filtered request construction with ADR-0040. ADR-0046 separately validates optional reported input usage before admission. Callers need one opt-in operation joining those unchanged boundaries without reproducing or changing their ordering.

## Decision

`nexa-tutor` provides `select_filtered_authorized_available_remote_model_tokenize_invoke_validate_reported_usage_and_admit`. It accepts exactly ADR-0044's caller-supplied inputs and returns the existing `TokenizedInvocationAdmissionResult`.

The operation first delegates the unchanged requirements, exact caller availability and authorization, and complete filter result to ADR-0034. ADR-0034 remains the filtered-selection boundary and ADR-0031 remains the sole permission boundary; no authentication or authorization-policy change is introduced. After selection it constructs ADR-0044's exact request from only the filtered compilation, then delegates the original selected shared provider, exact request, caller tokenizer, exact filtered compilation, and governed inputs to unchanged ADR-0046. Its closed, content-free error preserves the complete ADR-0034 failure or complete nested ADR-0046 failure.

The exact filtered compilation binds authorization, request input, tokenization evidence, response association, reported-usage validation, and admission. ADR-0033 proves deterministic byte-preserving whole-layer inclusion and omission with replay evidence only. It grants no general privacy-policy correctness, semantic sensitivity inference, content minimization, field/sub-string redaction, or anonymization.

Authorization and availability are caller-supplied evidence without authenticity or freshness; availability grants no monitoring or recovery authority. Conservative UTF-8 byte selection precedes exact tokenization. The composition is deterministic and single-attempt, and only the selected filtered-authorized available remote provider can consume one outcome.

## Consequences and deferrals

ADRs 0022 through 0049 remain unchanged and independently callable. Absent reported usage remains valid; present reported input usage must equal generated evidence. This establishes structural association and equality only and grants no tokenizer/provider truth, billing/cost, output-token evidence, telemetry, or semantic authority.

No authentication, authorization or filter policy change, secrets, endpoint, credential, concrete tokenizer/provider, inference, transport/networking, availability acquisition/refresh, token-aware selection, routing, fallback, retry/recovery/repair/regeneration, partial truncation, usage accumulation, pricing/billing/quota/cost, telemetry export, persistence, tools, async/streaming, semantic validation, new dependency, or unrelated refactor is added. NEXA-TUTOR-001 remains Baseline Draft, Phase 4 remains in progress, and the known ingestion/context checklist inconsistency is preserved.
