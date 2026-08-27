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

For current v1 work, begin with:

- [`PROJECT-STATUS.md`](PROJECT-STATUS.md)
- [`SPECIFICATION-REGISTRY.md`](SPECIFICATION-REGISTRY.md)
- [`architecture/NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md`](architecture/NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md)
- [`architecture/NEXA-R1-IMPLEMENTATION-BASELINE.md`](architecture/NEXA-R1-IMPLEMENTATION-BASELINE.md)
- [`adr/0069-owner-approved-v1-delivery-baseline.md`](adr/0069-owner-approved-v1-delivery-baseline.md)
- [`adr/0068-v1-r2-walking-skeleton-baseline.md`](adr/0068-v1-r2-walking-skeleton-baseline.md) (historical/non-conflicting scope only)
- [`architecture/NEXA-COMPLETION-ROADMAP.md`](architecture/NEXA-COMPLETION-ROADMAP.md)
- the applicable subsystem specification and accepted ADRs for the selected increment.

`NEXA-ARCH-001` remains preserved as reconstructed architecture provenance and long-range design context but is superseded by `NEXA-ARCH-002` for v1 implementation selection.

## Immediate invariants

- Nexa is an adaptive tutor platform, not merely an avatar or chatbot.
- The v1 path is local-first and includes accessible text, bundled speech, and synchronized animated 2D embodiment in both identical clients.
- Tutor/model output never directly controls animation primitives or host authority.
- Core domain identifiers/events remain shared canonical contracts rather than being redefined by adapters.
- Student mastery changes only from governed evidence and replayable policy.
- The orchestrator coordinates subsystem work but does not absorb subsystem reasoning.
- Storage, model provider, renderer, speech, and lab backends remain adapter concerns behind owned boundaries.
- Provider/renderer neutrality is architectural; LM Studio is the sole v1 reference model server, while the bundled speech and 2D candidates remain evidence-gated.
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
