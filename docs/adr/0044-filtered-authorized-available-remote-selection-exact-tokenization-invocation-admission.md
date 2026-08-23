# ADR-0044: Filtered authorized available remote selection, exact tokenization, invocation, and admission

- Status: Accepted
- Date: 2026-08-23
- Scope: Narrow Phase 4 synchronous filtered caller-authorized host composition

## Context

ADR-0034 is the sole boundary for selecting an authorized available remote model from intrinsically valid ADR-0033 whole-layer filter evidence. ADR-0035 composes that selection with ADR-0025, while ADR-0040 separately composes exact tokenization, request-capacity validation, one invocation, and strict admission. The tokenized counterpart to ADR-0035 is needed without changing either boundary.

## Decision

`nexa-tutor` provides `select_filtered_authorized_available_remote_model_tokenize_invoke_and_admit`. It accepts the immutable registry, caller invocation identity, unchanged requirements, exact availability snapshot, unchanged authorization, tokenization version, one model-bound tokenizer, complete filter result, trusted authority, context, and citations.

The operation first delegates the exact registry, requirements, snapshot, authorization, and complete filter result to unchanged ADR-0034. ADR-0034 remains solely responsible for intrinsic ADR-0033 evidence validation, singleton target-privacy agreement, authorization bound to the filtered compilation replay anchor, and unchanged ADR-0031 authorization, availability, registry, eligibility, ordering, and original shared-handle selection. The operation then constructs the existing ADR-0022 request from the caller invocation identity, selected descriptor, unchanged requirements, and `filtered_result.filtered_compilation`, and delegates the exact request and filtered compilation to unchanged ADR-0040.

Success returns ADR-0040's exact `TokenizedInvocationAdmissionResult` without replacement evidence. The closed, content-free `FilteredAuthorizedAvailableRemoteTokenizedInvocationAdmissionError` distinguishes only nested ADR-0034 selection failure from nested ADR-0040 failure. No tokenizer or provider is consumed before those boundaries permit it, and dependency consumption is followed by no second filter, authorization, selection, call, retry, fallback, repair, regeneration, recovery, degradation, or alternate-provider attempt.

## Privacy, permission, and deferrals

The request input, tokenization evidence, admission evidence, and returned result bind to the exact filtered compilation. ADR-0033 continues to prove deterministic, byte-preserving whole-layer disclosure and replay only. This increment reuses that structural filtering and caller authorization; it establishes no general privacy-policy correctness, semantic sensitivity inference, content-level minimization, redaction, anonymization, authentication, authorization authenticity/freshness, authorization-policy change, or semantic safety.

No secret handling, credential or endpoint configuration, concrete tokenizer/provider, remote execution, inference, transport/networking, token-aware selection, automatic routing, fallback/recovery, tools, async/streaming, telemetry, or persistence is added. NEXA-TUTOR-001 remains Baseline Draft, Phase 4 remains in progress, and the known ingestion/context checklist inconsistency is preserved.
