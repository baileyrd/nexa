# Nexa Governing Baseline

Status date: 2026-08-26

## Authority order

Use: (1) approved current architecture/specifications, (2) accepted ADRs, with later direct conflicts controlling only their scope, (3) approved/baseline subsystem specifications, (4) character identity authority, (5) verified implementation evidence, and (6) reconstructed provenance. Code never silently amends authority.

## Current v1 set

Begin with project status, the specification registry, [NEXA-ARCH-002](architecture/NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md), [NEXA-R1](architecture/NEXA-R1-IMPLEMENTATION-BASELINE.md), [ADR-0069](adr/0069-owner-approved-v1-delivery-baseline.md), the [completion roadmap](architecture/NEXA-COMPLETION-ROADMAP.md), and applicable supplements. [ADR-0068](adr/0068-v1-r2-walking-skeleton-baseline.md) is historical and remains effective only where ADR-0069 does not conflict. NEXA-ARCH-001 remains provenance.

## Invariants

Nexa is an adaptive tutor. v1 has identical Windows desktop and same-machine browser clients using one shared interface and one versioned loopback HTTP/WebSocket business boundary. It is local-first, single-learner, SQLite-durable, and provider-neutral at owned ports; LM Studio is the sole validated v1 reference model server. Speech and animated 2D embodiment are required, while Sherpa-ONNX and Rive remain unproven candidates. Model output never controls renderer primitives. Deferred capabilities do not become release gates without explicit authority.

## Maturity and control

Use `Concept -> Architecture Defined -> Specification Approved -> Contract Implemented -> Runtime Integrated -> Concrete Adapter Implemented -> System Verified -> User Accepted -> Release Ready`. The authority reconciliation establishes selected architecture/specification, not adapter or system proof. General product work remains paused; only roadmap-ordered bounded spikes may receive a Continue call.
