# Nexa Specification Registry

Status date: 2026-08-26

This registry is the navigation and governance authority for Nexa specifications, architecture baselines, accepted ADRs, and active program controls.

Git history preserves prior registry detail. This version intentionally reports **current authority and maturity** instead of repeating the full chronological narrative of every increment.

## Status vocabulary

| Status | Meaning |
|---|---|
| Reconstructed | Recovered design/provenance; not sufficient by itself for new implementation authority |
| Baseline Draft | Working subsystem design; changes require traceability |
| Approved | Reviewed and accepted as governing design/specification |
| Accepted | Accepted architecture decision record |
| Contract Implemented | Governed contract/policy has verified implementation evidence |
| Runtime Integrated | Capability is composed through a runtime/application boundary |
| Concrete Adapter Implemented | A real provider/storage/OS/device adapter exists |
| System Verified | End-to-end release path has system evidence |
| User Accepted | Representative user acceptance evidence exists |
| Release Ready | Release/package/acceptance gates are satisfied |
| Superseded | Replaced for the stated scope by a later authority |
| Assessment | Factual review evidence; not product behavior authority |
| Active Control | Execution/governance control currently in force |

Do not use an unqualified `Complete` where one of these maturity states is more precise.

## Governing precedence

For v1 implementation selection, use this order unless a later accepted ADR explicitly changes it:

1. Approved v1 system architecture and approved v1/R1 specification supplements.
2. Accepted ADRs, with later directly conflicting ADRs superseding earlier decisions only for their documented scope.
3. Approved/Baseline Draft subsystem specifications where not superseded or supplemented for v1.
4. Character/behavior identity authority and canonical visual references for their owned scopes.
5. Verified implementation contracts and traceability as evidence of actual maturity.
6. Reconstructed architecture/provenance documents for historical intent where consistent with current authority.

Conflicts are reported and resolved explicitly; code does not silently redefine specifications.

## Current v1 authorities

| ID / artifact | Document | Status | Scope |
|---|---|---|---|
| NEXA-ARCH-002 | [Nexa v1 Release Architecture](architecture/NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md) | Approved | Governing v1 system architecture and R2 release path |
| NEXA-V1 | [Nexa v1 Product Definition and Release Boundary](architecture/NEXA-V1-DEFINITION.md) | Approved through NEXA-ARCH-002/R1 baseline | First releasable learner outcome and acceptance boundary |
| NEXA-R1 | [Nexa R1 Implementation Baseline](architecture/NEXA-R1-IMPLEMENTATION-BASELINE.md) | Approved | R1 cross-cutting/parent requirements sufficient to govern R2 |
| ADR-0068 | [v1 R2 walking-skeleton baseline](adr/0068-v1-r2-walking-skeleton-baseline.md) | Accepted | Text-first scope, SQLite, egui, llama.cpp, Windows x86_64, first course |
| R0-R9 roadmap | [Nexa Completion Roadmap](architecture/NEXA-COMPLETION-ROADMAP.md) | Approved for delivery control | Dependency-driven route from rebaseline to release |
| Rebaseline gates | [Architecture Rebaseline and Program-Integrity Gates](governance/ARCHITECTURE-REBASELINE-GATES.md) | Approved project governance | Periodic whole-system review and stop/redirect/continue control |
| Deferral register | [Architectural Deferral Register](governance/DEFERRAL-REGISTER.md) | Active Control | Cross-stage inherited deferrals and review/disposition routing |
| Project status | [PROJECT-STATUS.md](PROJECT-STATUS.md) | Active Control | Current factual status and resume route |

## Historical architecture and identity authority

| ID | Document | Status | Current role |
|---|---|---|---|
| NEXA-ARCH-001 | [Tutor System Architecture v0.1](Nexa%20Tutor%20System%20%E2%80%94%20Architecture%20v0.1.md) | Superseded for v1 implementation selection | Preserved reconstructed long-range architecture/provenance |
| NEXA-CBS-001 | [Character & Behavior Specification](Nexa%20Character%20&%20Behavior%20Specification%20v1.0.md) | Baseline Draft | Character identity, semantic behavior principles, pedagogy/embodiment context |

`NEXA-ARCH-002` preserves the core rule that tutor intelligence produces semantic intent and does not directly select animation clips, bones, blendshapes, or renderer operations.

## R1 approved supplements

The following tactical-pause documents remain in the `architecture/r1-drafts/` directory for history, but `NEXA-R1-IMPLEMENTATION-BASELINE.md` approves their R2-applicable requirements. The filename/header word `DRAFT` records their origin and does not override the R1 approval record.

| Area | Approved supplement | R2 role |
|---|---|---|
| Domain / Events | [Domain and Events v1 rebaseline](architecture/r1-drafts/NEXA-DOM-EVT-V1-REBASELINE-DRAFT.md) | Canonical identity/event scope and v1 runtime facts |
| Learning | [Learning subsystems v1 rebaseline](architecture/r1-drafts/NEXA-LEARNING-V1-REBASELINE-DRAFT.md) | Durable student/pedagogy/lesson/assessment behavior |
| Tutor / Knowledge | [Tutor and Knowledge v1 rebaseline](architecture/r1-drafts/NEXA-TUTOR-KNOWLEDGE-V1-REBASELINE-DRAFT.md) | One real model path, durable knowledge, grounding/citation quality |
| Orchestration | [Orchestrator v1 rebaseline](architecture/r1-drafts/NEXA-ORCH-V1-REBASELINE-DRAFT.md) | Complete learner workflow, timeout/retry/recovery/shutdown |
| Data / Persistence | [Data and Persistence v1](architecture/r1-drafts/NEXA-DATA-PERSISTENCE-V1-DRAFT.md) | SQLite-backed authoritative state, atomicity, migration/recovery |
| Security | [Security Architecture v1](architecture/r1-drafts/NEXA-SECURITY-V1-DRAFT.md) | Local trust boundaries, authority separation, secrets/network rules |
| Privacy | [Privacy and Data Handling v1](architecture/r1-drafts/NEXA-PRIVACY-V1-DRAFT.md) | Learner/content disclosure, retention/deletion, diagnostics |
| Observability | [Observability v1](architecture/r1-drafts/NEXA-OBSERVABILITY-V1-DRAFT.md) | Content-safe correlation and operational evidence |
| Learner UX | [Learner UX v1](architecture/r1-drafts/NEXA-UX-V1-DRAFT.md) | Desktop learner flow and accessibility baseline |
| Testing / Acceptance | [Testing and System Acceptance v1](architecture/r1-drafts/NEXA-TESTING-ACCEPTANCE-V1-DRAFT.md) | Maturity evidence and primary E2E acceptance scenario |
| Performance | [Performance v1](architecture/r1-drafts/NEXA-PERFORMANCE-V1-DRAFT.md) | Measurement-first release budgets/evidence |
| Packaging / Deployment | [Packaging and Deployment v1](architecture/r1-drafts/NEXA-PACKAGING-DEPLOYMENT-V1-DRAFT.md) | Windows release/install/update constraints; final mechanism later |
| Governed Content | [Content and Release v1](architecture/r1-drafts/NEXA-CONTENT-RELEASE-V1-DRAFT.md) | First governed TCP course package and provenance/quality |

If one of these supplements is insufficient during R2, implementation stops at that boundary and the owning specification/ADR is corrected before continuing.

## Subsystem specification inventory

| Order | ID | Area | Governing maturity / v1 interpretation | Intended implementation |
|---:|---|---|---|---|
| 00 | NEXA-DOM-001 | Core domain model | Baseline Draft + approved R1 supplement; contract implemented slices | `nexa-domain` |
| 00 | NEXA-EVT-001 | Events/runtime facts | Baseline Draft + approved R1 supplement; contract implemented slices | `nexa-events` |
| 01 | NEXA-NBP-001 | Behavior protocol | Contract Implemented for existing semantic embodiment slices | `nexa-nbp` |
| 02 | NEXA-STU-001 | Student model | Contract Implemented deterministic slice + approved R1 durable supplement | `nexa-student` |
| 03 | NEXA-PED-001 | Adaptive pedagogy | Contract Implemented deterministic slice + approved R1 supplement | `nexa-pedagogy` |
| 04 | NEXA-TUTOR-001 | Tutor intelligence | Extensive contract implementation + approved R1 v1 execution supplement; no concrete model adapter yet | `nexa-tutor` + concrete adapter |
| 05 | NEXA-KNOW-001 | Knowledge/RAG | Contract Implemented deterministic slices + approved R1 durable/quality supplement; no concrete durable adapter yet | `nexa-knowledge`, `nexa-knowledge-runtime` |
| 06 | NEXA-LESSON-001 | Curriculum/lessons | Contract Implemented deterministic slice + approved R1 durable/content supplement | `nexa-lessons` |
| 07 | NEXA-ASMT-001 | Assessment | Contract Implemented deterministic slice + approved R1 durable/privacy supplement | `nexa-assessment` |
| 08 | NEXA-LAB-001 | Labs/tools | Contract foundation retained; post-R2 by default | `nexa-labs` |
| 09 | NEXA-SPCH-001 | Speech | Contract/cancellation foundations retained; required v1 integration, candidate unproven | `nexa-speech` |
| 10 | NEXA-AVTR-001 | Renderer-neutral avatar | Contract Implemented slice; required v1 2D integration, candidate unproven | `nexa-avatar` |
| 11 | NEXA-3D-001 | 3D character architecture | Contract/runtime implemented slices; embodiment integration deferred from R2 | `nexa-3d` |
| 11 | NEXA-3D-ART-001 | 3D production pipeline | Baseline Draft; retained for later embodiment gate | assets/tooling |
| 11 | NEXA-3D-REF-001 | Canonical 3D reference | Baseline Draft visual authority | assets |
| 11 | NEXA-3D-RUNTIME-001 | 3D validation runtime | Runtime Integrated in viewer/validator slice | `nexa-3d`, viewer, validator |
| 12 | NEXA-ORCH-001 | Session orchestration | Lifecycle/cancellation contract/runtime foundations + approved R1 complete-workflow supplement | `nexa-orchestrator`, `nexa-orchestrator-runtime`, apps |
| 13 | NEXA-UX-001 (R1 supplement) | Learner UX | Specification Approved for R2 via R1 baseline | `apps/nexa-desktop` |
| 15 | NEXA-DATA-001 (R1 supplement) | Data/persistence | Specification Approved for R2 via R1 baseline + ADR-0068 | `nexa-storage` |
| 16 | NEXA-SEC-001 (R1 supplement) | Security | Specification Approved for R2 via R1 baseline | cross-cutting/adapters/app |
| 17 | NEXA-PRIV-001 (R1 supplement) | Privacy | Specification Approved for R2 via R1 baseline | cross-cutting/adapters/app |
| 18 | NEXA-OBS-001 (R1 supplement) | Observability | Specification Approved for R2 via R1 baseline | runtime/app |
| 22/23 | NEXA-PKG/DEPLOY-001 (R1 supplement) | Packaging/deployment | Specification Approved to constrain R2; final R8 decisions deferred | release/app |
| 24 | NEXA-PERF-001 (R1 supplement) | Performance | Measurement specification approved; final release budgets measured later | system |
| 25 | NEXA-TEST-001 (R1 supplement) | Testing/acceptance | Specification Approved; maturity model and R2 E2E gate authoritative | system/CI |

Unlisted reserved namespaces remain future/post-v1 unless explicitly promoted by a later approved roadmap decision.

## Accepted ADR inventory

All ADR files under [`docs/adr/`](adr/) with status Accepted remain architecture authority for their documented scope unless explicitly superseded.

Current ranges:

- ADR-0001 through ADR-0009 — repository/contract/embodiment foundations;
- ADR-0010 through ADR-0014 — learning-core ownership and composition;
- ADR-0015 through ADR-0020 — governed knowledge/retrieval/context/citation;
- ADR-0021 through ADR-0050 — tutor/model/provider-neutral contracts and compositions;
- ADR-0051 through ADR-0067 — orchestration/cancellation/runtime/speech/tool foundations;
- ADR-0068 — approved R2 walking-skeleton technology and scope baseline.

Earlier provider-neutral/cancellation ADRs remain valid reusable foundations, but they do not define the post-rebaseline implementation sequence unless required by the current R2/R3 release path.

## Evidence and traceability

Historical technical-gate evidence remains in:

- [`architecture/PHASE-1-TRACEABILITY.md`](architecture/PHASE-1-TRACEABILITY.md)
- [`architecture/PHASE-2-TRACEABILITY.md`](architecture/PHASE-2-TRACEABILITY.md)
- [`architecture/PHASE-3-TRACEABILITY.md`](architecture/PHASE-3-TRACEABILITY.md)
- [`architecture/PHASE-4-TRACEABILITY.md`](architecture/PHASE-4-TRACEABILITY.md)
- [`architecture/PHASE-5-TRACEABILITY.md`](architecture/PHASE-5-TRACEABILITY.md)

These matrices prove their stated contract/headless/runtime slices. They do not override current product maturity in `PROJECT-STATUS.md`.

## Assessment and lessons evidence

| Artifact | Status | Purpose |
|---|---|---|
| [Current-State Assessment](architecture/NEXA-CURRENT-STATE-ASSESSMENT.md) | Assessment | Factual gap/maturity baseline at PR #109 |
| [Development Divergence Analysis](governance/DEVELOPMENT-DIVERGENCE-ANALYSIS.md) | Assessment | Where/why the process diverged |
| [Development Lessons Learned](governance/DEVELOPMENT-LESSONS-LEARNED.md) | Assessment | Nexa-specific lessons and process changes |
| [Rebaseline Matrix](architecture/NEXA-REBASELINE-MATRIX.md) | Assessment/support | Capability/spec maturity mapping used during convergence |

Reusable lessons are also preserved in the Atlas Engineering Standards Library; Atlas issue #20 tracks normative program-integrity/rebaseline requirements.

## Active delivery control

The current delivery path is [`architecture/NEXA-COMPLETION-ROADMAP.md`](architecture/NEXA-COMPLETION-ROADMAP.md).

R0 and R1 are complete to the level required to begin R2 once the documentation PR containing this registry is green and merged.

Every new implementation increment must:

1. trace to a current R2/R3 release blocker;
2. cite its governing parent architecture/specification and applicable ADRs;
3. identify the maturity state it advances;
4. include evidence appropriate to that state;
5. update project status/traceability without overstating maturity.

## Known later-release decisions

Not R2 blockers unless explicitly promoted:

- release GGUF model/quantization;
- final local-model runtime distribution;
- remote providers/credentials;
- speech/avatar final release inclusion;
- lab/tool sandbox implementation;
- durable event/outbox;
- advanced vector infrastructure;
- final installer/signing/update mechanism;
- plugins/public API/analytics/authoring/server deployment.

## Governance rule

The Chief Systems Architect reviews the whole system independently of PR cadence at major stage boundaries or when material drift signals appear. The recorded result is Continue, Redirect, or Tactical Pause.

Local correctness never substitutes for system-level progress.

---

## 2026-08-26 owner-authority reconciliation (controlling addendum)

ADR-0069 records the explicit owner decisions from Issue #114. This addendum supersedes earlier text in this document only where it conflicts. Earlier `eframe`/`egui`, `llama.cpp`, text-first release, desktop-only, speech/avatar deferral, owner-delegation, or general R2-Continue/readiness language is preserved as historical evidence and is not active selection authority.

Status date: 2026-08-26

This is the navigation and governance authority. Status vocabulary is: Reconstructed, Baseline Draft, Approved, Accepted, Contract Implemented, Runtime Integrated, Concrete Adapter Implemented, System Verified, User Accepted, Release Ready, Superseded, Assessment, and Active Control.

### Precedence and current authorities

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

### Approved R1 supplements

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

### Implemented maturity inventory

Shared domain/event/NBP and owned learning/tutor/orchestration contracts have Contract Implemented slices; selected headless/runtime compositions have Runtime Integrated evidence. Speech cancellation and avatar semantic foundations remain reusable. Scripted providers, in-memory stores, `.gitkeep` boundaries, and research do not prove concrete v1 adapters. No v1 release path is System Verified, User Accepted, or Release Ready.

### Work route

General implementation is paused. After reconciliation merges, only the bounded shared UI/loopback suitability spike may be dispatched first. Candidate success requires recorded evidence and an authority update; failure requires candidate removal and owner-governed reselection. Speech and avatar spikes follow; no spike silently establishes production architecture.
