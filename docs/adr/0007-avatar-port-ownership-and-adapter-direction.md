# ADR-0007: Avatar port ownership, capability negotiation, and adapter direction

- Status: Accepted
- Date: 2026-08-18
- Governing specifications: NEXA-NBP-001, NEXA-AVTR-001, NEXA-3D-001, NEXA-3D-RUNTIME-001

## Context

NBP defines semantic behavior input and runtime output, while the existing root 3D runtime owns
asset-facing names and renderer controls. Directly exposing its clips, nodes, blendshapes, shader
types, or `wgpu` objects would couple tutor intelligence to one embodiment. The avatar specifications
also require capability reporting and explicit degradation when an adapter cannot realize an optional
semantic behavior.

The source specifications show asynchronous signatures in conceptual examples, but Phase 1 has not
selected an async runtime, transport, queue, or callback ownership model. Introducing one in the
contract kernel would violate the governed dependency baseline.

## Decision

`nexa-avatar` owns the renderer-neutral inbound `AvatarPort`, its semantic request/report values,
capability vocabulary, NBP conversion, and deterministic fake adapter. It depends only on
`nexa-domain`, `nexa-nbp`, serialization, and dependency-light error support.

Direction is strictly:

```text
tutor/orchestrator -> NBP -> nexa-avatar port <- renderer/runtime adapter -> manifest -> renderer
```

Runtime adapters implement the port. The contract crate never imports a runtime or renderer. The
existing root `nexa-3d-runtime` implements the port on its semantic adapter and retains all concrete
asset-name and renderer mappings.

Capabilities are an ordered set of semantic facilities. A command requiring an unavailable optional
facility produces an NBP `Degraded` acknowledgement and recoverable warning rather than being
silently accepted. Cancellation is a first-class request and capability. Reports use NBP
acknowledgement, state, and error types rather than duplicating wire vocabulary.

Phase 1 uses a synchronous port because it is deterministic, object-safe in ordinary hosts, and does
not choose an async executor. Transports may invoke it from an asynchronous adapter later.

## Consequences

- Tutor code cannot address clips, bones, blendshapes, shaders, glTF nodes, or renderer APIs.
- 2D, 3D, headless, remote, and future engine adapters share one semantic boundary.
- Capability fallback is observable and testable without a GPU or OS event loop.
- The root runtime remains in place; moving it to `crates/nexa-3d` or an application is explicitly out
  of scope for this increment.

## Unresolved decisions

- Completion and intermediate-state delivery may eventually need a separate outbound stream; its
  async runtime, ordering, backpressure, and event-bus ownership remain undecided.
- NBP does not yet define a capability-negotiation wire message. This increment exposes local
  discovery and does not invent a wire extension.
- Cancellation acknowledgement does not yet define whether renderer completion or cancellation
  acceptance is authoritative. Phase 1 acknowledges adapter acceptance.
- Speech timing and viseme ownership remain with the speech/audio integration; the existing 3D
  adapter reports semantic speech as degraded rather than claiming playback support.
