# ADR-0009: Embodiment acceptance, capability wire ownership, and lifecycle

- **Status:** Accepted
- **Date:** 2026-08-18

## Context

NEXA-NBP-001 describes capability discovery and acknowledgement/state/error output, while NEXA-AVTR-001 owns the renderer-neutral port. The reconstructed material does not settle which layer owns the capability wire shape, and it sometimes uses acknowledgement as shorthand for both receipt and eventual completion. Inventing transport concurrency would also conflict with the current synchronous, dependency-light core.

## Decision

`nexa-nbp` owns the renderer-neutral `runtime.capabilities` wire message and its closed v1 capability vocabulary. `nexa-avatar` owns adapter capability evaluation and converts its ordered report into governed NBP acknowledgement, state, and error messages. Capability lists are sorted and de-duplicated. Unknown major versions are rejected; compatible v1 minor messages retain ADR-0006's optional-field rule.

An `accepted` acknowledgement means only that the adapter took responsibility for a behavior. It is not completion. Success is the ordered lifecycle `accepted`, `started`, `completed`. Cancellation emits `cancelled`; optional unsupported facilities and unresolved semantic targets emit recoverable `degraded`; refusal emits `rejected`; execution faults emit `accepted`, `started`, `failed`. Each output payload carries the originating message ID and behavior ID where applicable, while its envelope preserves session and correlation and uses caller-supplied output identity and sequence.

Typed avatar lifecycle events are emitted for accepted, started, completed, cancelled, degraded, and failed facts. Rejection remains an NBP acknowledgement/error outcome, not a lifecycle event, because no behavior lifecycle began. Event payloads preserve originating message and behavior identity; envelopes preserve session, correlation, source-scoped sequence, and caller-supplied event identity.

The core flow remains synchronous and deterministic. Composition roots provide message/event IDs, timestamps through the input, and starting sequences; the core does not read clocks or generate identity. Future asynchronous or network transports may schedule this flow and publish its outputs, but may not change acknowledgement semantics or introduce an async runtime into contract crates.

## Consequences

Capability negotiation is portable across renderers without leaking clips, bones, blendshapes, shaders, glTF nodes, or renderer objects. The headless fake has deterministic completion, degradation, rejection, failure, and cancellation behavior. Speech/audio processing, persistence, networking, scheduling, retries, and transport delivery remain out of scope.
