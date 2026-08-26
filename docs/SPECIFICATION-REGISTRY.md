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
| NEXA-ARCH-002 | [Nexa v1 Release Architecture](architecture/NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md) | Approved, reconciled by ADR-0069 | Governing v1 system architecture |
| NEXA-V1 | [Nexa v1 Product Definition and Release Boundary](architecture/NEXA-V1-DEFINITION.md) | Approved through NEXA-ARCH-002/R1 baseline | First releasable learner outcome and acceptance boundary |
| NEXA-R1 | [Nexa R1 Implementation Baseline](architecture/NEXA-R1-IMPLEMENTATION-BASELINE.md) | Approved | Cross-cutting v1 requirements |
| ADR-0069 | [Owner-approved v1 delivery baseline](adr/0069-owner-approved-v1-delivery-baseline.md) | Accepted | Current owner-approved clients, runtime, providers, embodiment, deferrals, and finite route |
| ADR-0068 | [Historical R2 walking-skeleton baseline](adr/0068-v1-r2-walking-skeleton-baseline.md) | Superseded only for conflicts | Preserved historical and non-conflicting decisions |
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

The following tactical-pause documents remain in the `architecture/r1-drafts/` directory for history, but `NEXA-R1-IMPLEMENTATION-BASELINE.md` approves their v1-applicable requirements. The filename/header word `DRAFT` records their origin and does not override the R1 approval record.

| Area | Approved supplement | v1 role |
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

If one of these supplements is insufficient during the v1 route, implementation stops at that boundary and the owning specification/ADR is corrected before continuing.

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
| 08 | NEXA-LAB-001 | Labs/tools | Contract foundation retained; deferred from v1 | `nexa-labs` |
| 09 | NEXA-SPCH-001 | Speech | Contract/cancellation foundations retained; required v1 integration, candidate unproven | `nexa-speech` |
| 10 | NEXA-AVTR-001 | Renderer-neutral avatar | Contract Implemented slice; required v1 2D integration, candidate unproven | `nexa-avatar` |
| 11 | NEXA-3D-001 | 3D character architecture | Contract/runtime implemented slices; 3D release integration deferred from v1 | `nexa-3d` |
| 11 | NEXA-3D-ART-001 | 3D production pipeline | Baseline Draft; retained for later embodiment gate | assets/tooling |
| 11 | NEXA-3D-REF-001 | Canonical 3D reference | Baseline Draft visual authority | assets |
| 11 | NEXA-3D-RUNTIME-001 | 3D validation runtime | Runtime Integrated in viewer/validator slice | `nexa-3d`, viewer, validator |
| 12 | NEXA-ORCH-001 | Session orchestration | Lifecycle/cancellation contract/runtime foundations + approved R1 complete-workflow supplement | `nexa-orchestrator`, `nexa-orchestrator-runtime`, apps |
| 13 | NEXA-UX-001 (R1 supplement) | Learner UX | Specification Approved for v1 via R1 baseline | `apps/nexa-desktop` |
| 15 | NEXA-DATA-001 (R1 supplement) | Data/persistence | Specification Approved for v1 via R1 baseline + ADR-0069 | `nexa-storage` |
| 16 | NEXA-SEC-001 (R1 supplement) | Security | Specification Approved for v1 via R1 baseline | cross-cutting/adapters/app |
| 17 | NEXA-PRIV-001 (R1 supplement) | Privacy | Specification Approved for v1 via R1 baseline | cross-cutting/adapters/app |
| 18 | NEXA-OBS-001 (R1 supplement) | Observability | Specification Approved for v1 via R1 baseline | runtime/app |
| 22/23 | NEXA-PKG/DEPLOY-001 (R1 supplement) | Packaging/deployment | Specification Approved to constrain v1; final R8 decisions deferred | release/app |
| 24 | NEXA-PERF-001 (R1 supplement) | Performance | Measurement specification approved; final release budgets measured later | system |
| 25 | NEXA-TEST-001 (R1 supplement) | Testing/acceptance | Specification Approved; maturity model and v1 E2E gate authoritative | system/CI |

Unlisted reserved namespaces remain future/post-v1 unless explicitly promoted by a later approved roadmap decision.

## Accepted ADR inventory

All ADR files under [`docs/adr/`](adr/) with status Accepted remain architecture authority for their documented scope unless explicitly superseded.

Current ranges:

- ADR-0001 through ADR-0009 — repository/contract/embodiment foundations;
- ADR-0010 through ADR-0014 — learning-core ownership and composition;
- ADR-0015 through ADR-0020 — governed knowledge/retrieval/context/citation;
- ADR-0021 through ADR-0050 — tutor/model/provider-neutral contracts and compositions;
- ADR-0051 through ADR-0067 — orchestration/cancellation/runtime/speech/tool foundations;
- ADR-0068 — preserved historical/non-conflicting walking-skeleton decisions;
- ADR-0069 — current owner-approved v1 delivery decisions.

Earlier provider-neutral/cancellation ADRs remain valid reusable foundations, but they do not define the post-rebaseline implementation sequence unless required by the current ADR-0069 finite release path.

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

The current delivery path is [`architecture/NEXA-COMPLETION-ROADMAP.md`](architecture/NEXA-COMPLETION-ROADMAP.md). General implementation is under Tactical Pause. G0 must merge before any product increment; afterward only a separately dispatched G1 shared UI/loopback suitability spike may receive Continue. Candidate success or failure requires evidence and a recorded authority update.

Every increment must trace to a finite-route blocker, cite governing authority, identify its concrete E2E step and maturity transition, and produce evidence appropriate to that transition. Existing contract/headless evidence is not system, user, or release evidence.

## Known later-release decisions and deferrals

Deferred from v1 unless later explicit authority changes scope: LAN/remote access, hosted deployment, cloud sync, accounts/multi-user administration, labs/tools, broad providers, dynamic routing/fallback, dedicated vector infrastructure unless proven necessary, durable event brokerage, and 3D release integration. Final packaging/signing/update mechanisms remain owned by the packaging gate. Required v1 speech and animated 2D integration are not deferrals; only Sherpa-ONNX, Rive, and their exact adapter designs remain evidence-gated.

## Governance rule

The Chief Systems Architect reviews the whole system independently of PR cadence at major stage boundaries or when material drift signals appear. The recorded result is Continue, Redirect, or Tactical Pause.

Local correctness never substitutes for system-level progress.

---
