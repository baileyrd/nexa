# ADR-0026: Provider-neutral in-memory model registry

- **Status:** Accepted
- **Date:** 2026-08-20
- **Scope:** Narrow Phase 4 registry-mechanics prerequisite

## Context

ADR-0022 owns provider/model identities, immutable descriptors, capabilities, privacy classes, and
the synchronous `LanguageModelProvider` port. ADR-0025 accepts one explicitly supplied provider;
it deliberately does not locate or select one. NEXA-TUTOR-001 sections 46–54 and 127–137 provide
Baseline Draft guidance for provider abstraction and a registry, but their illustrative async,
routing, fallback, privacy, and provider APIs are not approved contracts.

A dependency-light catalog is needed before a composition root can implement separately governed
selection policy. Registry mechanics must not themselves become selection or execution policy.

## Decision

`nexa-tutor` owns an immutable, in-memory `ModelRegistry` of caller-supplied
`Arc<dyn LanguageModelProvider>` handles. The exact key is the existing
`(ModelProviderId, ModelId)` pair. Each provider's complete existing `ModelDescriptor` remains the
registration description; the registry creates no competing identity, capability, privacy, or
version type.

Construction is fallible and atomic. It builds private temporary state, validates every descriptor
with ADR-0022 validation, distinguishes unsupported descriptor versions from other invalid
descriptors, rejects duplicate exact keys, and returns no partially usable registry on failure.
An empty registry is valid and has an empty inventory; every lookup from it fails closed.

The registry uses canonical identifier ordering: `ModelProviderId` ascending and then `ModelId`
ascending. Its read-only inventory returns cloned descriptors in that order and exposes neither
mutable registry internals nor provider-private state. Exact lookup requires both identities,
returns a clone of the registered `Arc`, and therefore preserves the shared provider allocation.
A missing pair is a closed error; model-only or approximate lookup does not exist.

Construction, inventory, and lookup inspect only descriptors and never call `generate`. They do
not inspect, clone, log, or serialize provider-private state. Registry and error diagnostics are
content-free; registry `Debug` reports only registration count. The immutable design needs no lock
and therefore has no synchronization-failure category. It uses only standard-library ownership and
collections and remains provider-neutral, dependency-light, and safely shareable under the
existing `Send + Sync` provider boundary. Registry contents are runtime state, not a durable wire
format.

ADR-0025 is unchanged and continues to require an explicitly supplied provider. A caller may use
exact registry resolution before calling it, but this increment does not integrate those steps or
authorize any provider use.

## Consequences and deferrals

This decision establishes catalog mechanics only. It does not implement deterministic or dynamic
selection; task routing; local-first preference; ranking; fallback; retry; repair; provider health
or availability; cost or latency policy; privacy filtering or remote-provider authorization;
provider tokenization; concrete local/cloud providers; inference; endpoints, credentials, or
networking; async execution, streaming, cancellation, or timeout policy; prompt or output-admission
changes; semantic safety, prompt-injection detection, truth, entailment, or hallucination control;
telemetry export; filesystem configuration; persistence or durable registries; dynamic
registration, removal, or hot reload; tools or labs; Phase 5 orchestration; or unrelated cleanup.

Partial truncation remains deferred. Phase 4 remains in progress and its exit gate is not met.
NEXA-TUTOR-001 remains Baseline Draft and is not promoted. ADRs 0021–0025 retain their meanings and
historical deferrals.
