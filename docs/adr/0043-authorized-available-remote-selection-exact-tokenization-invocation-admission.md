# ADR-0043: Authorized available remote selection, exact tokenization, invocation, and admission

- Status: Accepted
- Date: 2026-08-23
- Scope: Narrow Phase 4 synchronous caller-authorized host composition

## Context

ADR-0031 is the sole permission boundary for deterministic caller-authorized, availability-gated remote selection. ADR-0032 composes that selection with one invocation and strict admission, while ADR-0040 composes exact tokenization and request-capacity validation with one explicitly supplied provider invocation and admission. A narrow opt-in composition is needed without duplicating or changing authorization.

## Decision

`nexa-tutor` provides `select_authorized_available_remote_model_tokenize_invoke_and_admit`. The caller supplies an immutable registry, invocation identity, unchanged selection requirements, exact availability snapshot, unchanged remote authorization, tokenization contract version, exactly one model-bound tokenizer, exact compilation, trusted planning authority, context, and citations.

The operation first delegates the exact registry, unchanged requirements, availability, authorization, and compilation to unchanged ADR-0031. ADR-0031 remains the sole permission and selection boundary, including remote-only validation, exact compilation replay-anchor association, registry identity/privacy agreement, independent authorization and availability gates, conservative byte eligibility, privacy preference, and canonical ordering. It then constructs the existing ADR-0022 request from the selected descriptor, caller invocation identity, exact compilation input, and unchanged requirements, and delegates exact tokenization, capacity validation, exactly one selected-provider invocation, and strict admission to unchanged ADR-0040.

No tokenizer or provider is consumed before selection and host preflight permit it. Exact token counts do not affect authorization, availability, selection, privacy preference, or ordering. Success returns ADR-0040's `TokenizedInvocationAdmissionResult` unchanged. A closed, content-free error distinguishes nested `RemoteAuthorizationError` from nested `TokenizedInvocationAdmissionError`. There is no second authorization, selection, tokenizer/provider call, retry, fallback, repair, regeneration, recovery, degradation, or alternate-provider attempt.

## Permission, privacy, and deferrals

Authorization remains bound to the exact ADR-0023 replay anchor and exact authorized provider/model privacy identity. Tokenization proves no authorization authenticity/freshness, disclosure filtering, privacy correctness, minimization, redaction, or semantic safety. Local, mixed, empty, duplicate, unsupported, mismatched, and otherwise invalid evidence fails closed through ADR-0031; a local provider cannot be selected.

This increment reuses existing caller authorization and adds no authentication, authorization-policy change, secret handling, filtering/minimization proof, concrete tokenizer/provider, remote execution implementation, transport, or networking. All existing inference, endpoint/credential, routing, fallback/recovery, semantic-validation, async/streaming, telemetry, persistence, and partial-truncation deferrals remain. NEXA-TUTOR-001 remains Baseline Draft, the known ingestion/context checklist inconsistency is preserved, and Phase 4 remains in progress.
