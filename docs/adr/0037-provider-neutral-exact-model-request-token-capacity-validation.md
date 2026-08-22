# ADR-0037: Provider-neutral exact model-request token-capacity validation

- Status: Accepted
- Date: 2026-08-22
- Scope: Narrow Phase 4 non-invoking capacity-validation gate

## Context

ADR-0022 conservatively treats every UTF-8 input byte as one provider-neutral context unit. ADR-0036 separately records an exact, model-associated input-token count. A caller with an already-constructed request and existing evidence needs an additional fail-closed capacity gate without weakening ADR-0022 or calling a tokenizer or provider.

## Decision

`nexa-tutor` provides `validate_model_request_token_capacity`. It first delegates to unchanged `ModelRequest::validate_for`, so exact evidence can never admit a request rejected by ADR-0022 identity, version, capability, output-limit, or conservative byte-capacity validation. It then validates the existing ADR-0036 evidence against the exact descriptor and request input, preserving provider, model, descriptor-contract-version, byte-count, input-hash, intrinsic-version, count, and replay-anchor checks.

The operation uses checked `u32` addition of the evidence's non-zero exact input-token count and the request's requested maximum output-token count. Overflow and a total above the descriptor context window fail closed. Equality succeeds. Success returns only `()`; ADR-0036 evidence remains the sole tokenization evidence.

A closed content-free error distinguishes nested ADR-0022 request failure, nested ADR-0036 evidence failure, and exact-capacity overflow or excess. The synchronous operation mutates nothing and neither accepts nor inspects a tokenizer or provider.

## Consequences and deferrals

ADR-0022's conservative UTF-8 byte rule remains mandatory, authoritative, and unchanged. Existing selection, invocation, admission, authorization, availability, remote-filtering, and generation APIs do not implicitly consume tokenization evidence. `ModelUsage` remains adapter-reported evidence and is not reinterpreted.

No concrete tokenizer or provider is implemented. Token-aware selection and integration into invocation, admission, authorization, availability, routing, or provider execution remain deferred, as do provider-usage reconciliation, partial truncation, rewriting, summarization, compaction, inference, networking, fallback/retry/repair/recovery, semantic validation, general privacy policy, async/streaming, telemetry, and persistence. NEXA-TUTOR-001 remains Baseline Draft, the privacy specification namespace remains reserved and unimplemented, and Phase 4 remains in progress.
