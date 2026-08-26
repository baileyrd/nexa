# Nexa Governing Baseline

Status date: 2026-08-26

## Purpose

This document defines how Nexa authorities are interpreted after the 2026 tactical-pause rebaseline.

The reconstructed design remains important provenance, but current v1 implementation is governed by the approved release architecture and R1 baseline rather than by reconstruction status alone.

## Current authority order

When sources disagree, use this order unless a later accepted ADR explicitly resolves the conflict:

1. Approved current system architecture and approved current specifications/supplements.
2. Accepted ADRs for their documented scope.
3. Baseline Draft subsystem specifications where not superseded/supplemented for v1.
4. NEXA-CBS-001 for character identity and semantic behavior principles within its owned scope.
5. Canonical visual references for visual identity.
6. Verified implementation contracts and traceability as evidence of what is actually implemented.
7. Reconstructed architecture/design narratives and conversation exports as provenance and long-range context.

Code does not silently amend a higher authority. A discovered conflict is recorded and resolved through architecture/specification/ADR review.

## Current v1 governing set

For v1/R2 work, begin with:

- [`PROJECT-STATUS.md`](PROJECT-STATUS.md)
- [`SPECIFICATION-REGISTRY.md`](SPECIFICATION-REGISTRY.md)
- [`architecture/NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md`](architecture/NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md)
- [`architecture/NEXA-R1-IMPLEMENTATION-BASELINE.md`](architecture/NEXA-R1-IMPLEMENTATION-BASELINE.md)
- [`adr/0068-v1-r2-walking-skeleton-baseline.md`](adr/0068-v1-r2-walking-skeleton-baseline.md)
- [`architecture/NEXA-COMPLETION-ROADMAP.md`](architecture/NEXA-COMPLETION-ROADMAP.md)
- the applicable subsystem specification and accepted ADRs for the selected increment.

`NEXA-ARCH-001` remains preserved as reconstructed architecture provenance and long-range design context but is superseded by `NEXA-ARCH-002` for v1 implementation selection.

## Immediate invariants

- Nexa is an adaptive tutor platform, not merely an avatar or chatbot.
- The first v1 path is text-first and local-first.
- Tutor/model output never directly controls animation primitives or host authority.
- Core domain identifiers/events remain shared canonical contracts rather than being redefined by adapters.
- Student mastery changes only from governed evidence and replayable policy.
- The orchestrator coordinates subsystem work but does not absorb subsystem reasoning.
- Storage, model provider, renderer, speech, and lab backends remain adapter concerns behind owned boundaries.
- Provider/renderer neutrality is an architectural property; R2 still prioritizes one concrete real path.
- Local correctness evidence and system/product maturity are separate.

## Preservation policy

Historical/reconstructed documents remain intact unless a deliberate reviewed change says otherwise. Git history preserves earlier registry/status/roadmap narratives.

Semantic changes require traceability through an approved architecture/specification, accepted ADR, or explicit governance/status decision.

## Capability maturity

Use:

`Concept -> Architecture Defined -> Specification Approved -> Contract Implemented -> Runtime Integrated -> Concrete Adapter Implemented -> System Verified -> User Accepted -> Release Ready`

Do not use an unqualified `Complete` where it would hide the maturity level actually proven.

## Architecture control

At major roadmap boundaries, and whenever parent documentation, deferrals, horizontal depth, or release convergence materially diverge, the Chief Systems Architect performs an independent whole-system review and records Continue, Redirect, or Tactical Pause.

A lower-level implementation gate cannot close a higher-level architecture maturity gap.

---

## 2026-08-26 owner-authority reconciliation (controlling addendum)

ADR-0069 records the explicit owner decisions from Issue #114. This addendum supersedes earlier text in this document only where it conflicts. Earlier `eframe`/`egui`, `llama.cpp`, text-first release, desktop-only, speech/avatar deferral, owner-delegation, or general R2-Continue/readiness language is preserved as historical evidence and is not active selection authority.

Status date: 2026-08-26

### Authority order

Use: (1) approved current architecture/specifications, (2) accepted ADRs, with later direct conflicts controlling only their scope, (3) approved/baseline subsystem specifications, (4) character identity authority, (5) verified implementation evidence, and (6) reconstructed provenance. Code never silently amends authority.

### Current v1 set

Begin with project status, the specification registry, [NEXA-ARCH-002](architecture/NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md), [NEXA-R1](architecture/NEXA-R1-IMPLEMENTATION-BASELINE.md), [ADR-0069](adr/0069-owner-approved-v1-delivery-baseline.md), the [completion roadmap](architecture/NEXA-COMPLETION-ROADMAP.md), and applicable supplements. [ADR-0068](adr/0068-v1-r2-walking-skeleton-baseline.md) is historical and remains effective only where ADR-0069 does not conflict. NEXA-ARCH-001 remains provenance.

### Invariants

Nexa is an adaptive tutor. v1 has identical Windows desktop and same-machine browser clients using one shared interface and one versioned loopback HTTP/WebSocket business boundary. It is local-first, single-learner, SQLite-durable, and provider-neutral at owned ports; LM Studio is the sole validated v1 reference model server. Speech and animated 2D embodiment are required, while Sherpa-ONNX and Rive remain unproven candidates. Model output never controls renderer primitives. Deferred capabilities do not become release gates without explicit authority.

### Maturity and control

Use `Concept -> Architecture Defined -> Specification Approved -> Contract Implemented -> Runtime Integrated -> Concrete Adapter Implemented -> System Verified -> User Accepted -> Release Ready`. The authority reconciliation establishes selected architecture/specification, not adapter or system proof. General product work remains paused; only roadmap-ordered bounded spikes may receive a Continue call.
