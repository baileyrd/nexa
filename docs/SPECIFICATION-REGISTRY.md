# Nexa Specification Registry

Status date: 2026-08-26

This is the navigation and governance authority. Status vocabulary is: Reconstructed, Baseline Draft, Approved, Accepted, Contract Implemented, Runtime Integrated, Concrete Adapter Implemented, System Verified, User Accepted, Release Ready, Superseded, Assessment, and Active Control.

## Precedence and current authorities

Approved architecture/specifications precede accepted ADRs; later ADRs control direct conflicts only. Verified implementation is maturity evidence, not authority. Reconstructed material is provenance.

| Artifact | Status | Scope |
|---|---|---|
| [NEXA-ARCH-002](architecture/NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md) | Approved, reconciled by ADR-0069 | v1 system architecture |
| [NEXA v1 definition](architecture/NEXA-V1-DEFINITION.md) | Approved | release boundary |
| [NEXA-R1](architecture/NEXA-R1-IMPLEMENTATION-BASELINE.md) | Approved | cross-cutting implementation requirements |
| [ADR-0069](adr/0069-owner-approved-v1-delivery-baseline.md) | Accepted | explicit owner-approved clients, runtime, provider, embodiment, deferrals, sequence |
| [ADR-0068](adr/0068-v1-r2-walking-skeleton-baseline.md) | Superseded only for conflicts | preserved historical/non-conflicting decisions |
| [Completion roadmap](architecture/NEXA-COMPLETION-ROADMAP.md) | Approved delivery control | finite v1 route and evidence gates |
| [Deferral register](governance/DEFERRAL-REGISTER.md) | Active Control | deferred scope |
| [Project status](PROJECT-STATUS.md) | Active Control | current pause/resume gate |

NEXA-ARCH-001 remains reconstructed provenance. NEXA-CBS-001 remains Baseline Draft character/semantic behavior authority.

## Approved R1 supplements

The following `architecture/r1-drafts/` files retain historical filenames but their R1-applicable requirements are approved through NEXA-R1: domain/events, learning, tutor/knowledge, orchestration, data/persistence, security, privacy, observability, learner UX, testing/acceptance, performance, packaging/deployment, and content/release. Their 2026-08-26 reconciliation notes apply ADR-0069 where older conditional or desktop-only language conflicts.

| Area | Document |
|---|---|
| Domain/events | [Supplement](architecture/r1-drafts/NEXA-DOM-EVT-V1-REBASELINE-DRAFT.md) |
| Learning | [Supplement](architecture/r1-drafts/NEXA-LEARNING-V1-REBASELINE-DRAFT.md) |
| Tutor/knowledge | [Supplement](architecture/r1-drafts/NEXA-TUTOR-KNOWLEDGE-V1-REBASELINE-DRAFT.md) |
| Orchestration | [Supplement](architecture/r1-drafts/NEXA-ORCH-V1-REBASELINE-DRAFT.md) |
| Data/persistence | [Supplement](architecture/r1-drafts/NEXA-DATA-PERSISTENCE-V1-DRAFT.md) |
| Security/privacy | [Security](architecture/r1-drafts/NEXA-SECURITY-V1-DRAFT.md), [privacy](architecture/r1-drafts/NEXA-PRIVACY-V1-DRAFT.md) |
| UX/accessibility | [Supplement](architecture/r1-drafts/NEXA-UX-V1-DRAFT.md) |
| Testing/acceptance | [Supplement](architecture/r1-drafts/NEXA-TESTING-ACCEPTANCE-V1-DRAFT.md) |
| Performance | [Supplement](architecture/r1-drafts/NEXA-PERFORMANCE-V1-DRAFT.md) |
| Packaging/deployment | [Supplement](architecture/r1-drafts/NEXA-PACKAGING-DEPLOYMENT-V1-DRAFT.md) |
| Content | [Supplement](architecture/r1-drafts/NEXA-CONTENT-RELEASE-V1-DRAFT.md) |

## Implemented maturity inventory

Shared domain/event/NBP and owned learning/tutor/orchestration contracts have Contract Implemented slices; selected headless/runtime compositions have Runtime Integrated evidence. Speech cancellation and avatar semantic foundations remain reusable. Scripted providers, in-memory stores, `.gitkeep` boundaries, and research do not prove concrete v1 adapters. No v1 release path is System Verified, User Accepted, or Release Ready.

## Work route

General implementation is paused. After reconciliation merges, only the bounded shared UI/loopback suitability spike may be dispatched first. Candidate success requires recorded evidence and an authority update; failure requires candidate removal and owner-governed reselection. Speech and avatar spikes follow; no spike silently establishes production architecture.
