# ADR-0036: Provider-neutral model-input tokenization

- Status: Accepted
- Date: 2026-08-21

## Context

ADR-0022 deliberately uses one UTF-8 input byte per provider-neutral context unit, and ADR-0027 reuses that conservative rule for eligibility. Exact tokenizer results are model-specific and require a separate boundary that grants no inference, provider invocation, selection, routing, or network capability and does not weaken established capacity validation.

## Decision

`nexa-tutor` owns a synchronous `Send + Sync` model-input token-counting port, closed errors, standalone-validating evidence, host composition, and deterministic FIFO scripted test infrastructure. `nexa-domain` continues to own canonical provider/model identities and protocol versions. Concrete tokenizer implementations belong in future composition/adapters outside the dependency-light tutor core.

The port is immutably bound to an existing validated ADR-0022 `ModelDescriptor` and accepts the existing bounded `ModelInput`. Host preflight validates the requested tokenization version, supplied and tokenizer descriptors, and exact equality before exactly one counting call. Counts are checked `u32` values and zero fails closed. There is no retry or fallback.

V1 tokenizer semantics are governed by the exact `ModelProviderId` plus `ModelId`; no tokenizer identifier is introduced. A materially different tokenizer requires a distinct governed model identity or a future reviewed contract revision.

Evidence binds the tokenization contract version, exact provider/model identities, ADR-0022 descriptor contract version, checked UTF-8 byte count, lowercase SHA-256 input hash, non-zero input-token count, and lowercase SHA-256 replay anchor over every preceding governed field. Strict standalone deserialization validates versions, bounds, hashes, count, and replay. Association validation additionally proves the exact descriptor, byte count, and input hash. Evidence is content-free and proves deterministic association with a reported result, not tokenizer correctness, provider authenticity, freshness, or compatibility with later provider-reported usage.

The scripted tokenizer is deterministic testing infrastructure, not a real or concrete tokenizer. It consumes one FIFO outcome only after host preflight reaches the counting boundary and normalizes exhaustion, scripted failure, and synchronization failure without private diagnostics.

## Consequences

ADR-0022's one-input-byte-per-context-unit validation and ADR-0027 selection remain unchanged and authoritative. No selection, request-validation, invocation, admission, or routing API consumes tokenization evidence in this increment, and provider-reported `ModelUsage` semantics are unchanged. Future use of token counts requires a separate reviewed increment.

This decision adds no concrete tokenization algorithm, model files, inference, provider integration, networking, endpoint, credential, configuration, filesystem, time, randomness, telemetry, persistence, async/streaming, routing, fallback, retry, repair, semantic validation, or privacy-policy capability. Partial truncation remains deferred.
