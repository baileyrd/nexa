# ADR-0022: Provider-neutral model invocation

- **Status:** Accepted
- **Date:** 2026-08-20
- **Specification:** NEXA-TUTOR-001

## Context

NEXA-TUTOR-001 requires replaceable local and remote model providers, explicit capabilities,
normalized failures, and deterministic mock output. ADR-0021 deliberately stops before model
invocation and treats caller-supplied response text as inert. The reconstructed specification
illustrates asynchronous generation and streaming, but the current contract crates exclude async
runtimes, networking, and provider implementations. Provider identity ownership, raw-output
validation, and the boundary between invocation success and tutor-response acceptance were not
otherwise decided.

## Decision

`nexa-tutor` owns a synchronous V1 `LanguageModelProvider` port, bounded provider-neutral request
and response contracts, closed capability and privacy classifications, normalized redacted errors,
and a deterministic FIFO scripted adapter. `ModelProviderId`, `ModelId`, and `ModelInvocationId`
are canonical non-nil UUID values in `nexa-domain` because provider-independent audit and
correlation cross adapter and tutor boundaries.

A model descriptor declares immutable identity, privacy class, context/output limits, and
capabilities. A request binds the target identities, invocation identity, V1 contract, bounded
opaque input, required capabilities, and output limit. Unsupported capabilities, excessive limits,
versions, and identity mismatches fail before an adapter consumes work. A response repeats the
bound identities and contains bounded opaque output, a closed finish reason, and optional
adapter-reported usage. Usage is evidence, not independently verified tokenizer truth.

Input and output remain untrusted and are redacted from `Debug` and errors. Successful invocation
does not create or validate a `TutorResponse`; no implicit conversion joins these contracts. The
scripted adapter exists for deterministic testing without inference, clocks, randomness,
filesystem access, or networking. Invalid requests do not consume its next scripted outcome.

The synchronous port is the dependency-light semantic boundary, not a production execution
strategy. Concrete adapters may wrap it from composition layers, and a later decision may add an
asynchronous execution port without changing V1 wire representations.

## Consequences and deferrals

This increment establishes provider abstraction but performs no inference. Concrete local/cloud
adapters, endpoints, credentials, networking, async execution, cancellation, streaming, provider
selection, registries, routing, fallback, retry, prompt compilation, context privacy filtering,
provider tokenization, repair/regeneration, telemetry export, and persistence remain deferred.

Raw output is not proof of structural validity, pedagogy compliance, grounding, citation
entailment, truth, prompt-injection resistance, or semantic safety. Connecting provider output to
ADR-0021 response planning requires a separate reviewed increment. NEXA-TUTOR-001 remains a
Baseline Draft and Phase 4 remains in progress.
