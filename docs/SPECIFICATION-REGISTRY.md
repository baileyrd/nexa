# Nexa Specification Registry

This registry is the navigation and governance authority for Nexa system specifications.

## Status vocabulary

| Status | Meaning |
|---|---|
| Reconstructed | Recovered from the design conversation; pending structured review |
| Baseline Draft | Accepted as the working design baseline; changes require traceability |
| Approved | Reviewed and accepted for implementation |
| Implemented | The governed contract has a verified implementation |
| Superseded | Replaced by an identified later specification |

## Governing documents

| ID | Document | Status | Scope |
|---|---|---|---|
| NEXA-CBS-001 | [Character & Behavior Specification](Nexa%20Character%20&%20Behavior%20Specification%20v1.0.md) | Baseline Draft | Character identity, behavior, pedagogy, and runtime contract |
| NEXA-ARCH-001 | [Tutor System Architecture](Nexa%20Tutor%20System%20%E2%80%94%20Architecture%20v0.1.md) | Reconstructed | Platform context and subsystem architecture |
| ADR-0001 | [Monorepo and contract-first architecture](adr/0001-monorepo-and-contract-first-architecture.md) | Accepted | Repository and implementation organization |
| ADR-0002 | [Contract-kernel dependency boundaries](adr/0002-contract-kernel-dependency-boundaries.md) | Accepted | Crate ownership and dependency DAG |
| ADR-0003 | [Canonical values](adr/0003-canonical-values.md) | Accepted | Identifiers, time, duration, version, confidence, and sequence |
| ADR-0004 | [JSON wire compatibility](adr/0004-json-wire-compatibility.md) | Accepted | Serialization and compatibility policy |
| ADR-0005 | [Event envelope semantics](adr/0005-event-envelope-semantics.md) | Accepted | Identity, ordering, correlation, delivery, and replay |
| ADR-0006 | [NBP versioning and extensions](adr/0006-nbp-versioning-and-extensions.md) | Accepted | Protocol evolution and extension governance |
| ADR-0007 | [Avatar port ownership and adapter direction](adr/0007-avatar-port-ownership-and-adapter-direction.md) | Accepted | Renderer-neutral port ownership and adapter boundaries |
| ADR-0009 | [Embodiment acceptance and lifecycle](adr/0009-embodiment-acceptance-and-lifecycle.md) | Accepted | Capability wire ownership, acknowledgement, lifecycle events, and synchronous core |
| ADR-0010 | [Learning state, immutable evidence, and persistence](adr/0010-learning-state-evidence-and-persistence.md) | Accepted | Phase 3 ownership, replay, policy versioning, and persistence ports |
| ADR-0011 | [Pedagogy policy ownership and versioning](adr/0011-pedagogy-policy-ownership-and-versioning.md) | Accepted | Pedagogy ownership, thresholds, explanations, and lesson separation |
| ADR-0012 | [Governed curriculum and lesson transitions](adr/0012-governed-curriculum-and-lesson-transitions.md) | Accepted | Authored curriculum, progress, prerequisites, and versioned transitions |
| ADR-0013 | [Assessment contract, scoring, and evidence](adr/0013-assessment-contract-scoring-and-evidence.md) | Accepted | Assessment ownership, scoring version, lifecycle, and evidence boundary |
| ADR-0015 | [Governed knowledge ingestion and provenance](adr/0015-governed-knowledge-ingestion.md) | Accepted | Phase 4 artifact integrity, provenance, visibility, lifecycle, and atomic staging |
| ADR-0016 | [Deterministic governed lexical retrieval](adr/0016-deterministic-lexical-retrieval.md) | Accepted | Phase 4 lexical query contracts, corpus validation, governance filtering, scoring, and stable ordering |
| ADR-0014 | [Learning-core composition and atomicity](adr/0014-learning-core-composition-and-atomicity.md) | Accepted | Phase 3 policy composition, unit of work, idempotency, and deferred durable adapters |
| ADR-0008 | [Controlled 3D workspace migration and ownership](adr/0008-controlled-3d-workspace-migration.md) | Accepted | 3D library, viewer, and validator ownership after migration |

## Subsystem specifications

| Order | ID | Area | Status | Intended crate |
|---:|---|---|---|---|
| 00 | NEXA-DOM-001 | Core domain model | Baseline Draft | `nexa-domain` |
| 00 | NEXA-EVT-001 | Events and runtime bus | Baseline Draft | `nexa-events` |
| 01 | NEXA-NBP-001 | Behavior protocol | Implemented Phase 2 slice | `nexa-nbp` |
| 02 | NEXA-STU-001 | Student model | Implemented Phase 3 slice | `nexa-student` |
| 03 | NEXA-PED-001 | Adaptive pedagogy | Implemented Phase 3 slice | `nexa-pedagogy` |
| 04 | NEXA-TUTOR-001 | Tutor intelligence | Baseline Draft | `nexa-tutor` |
| 05 | NEXA-KNOW-001 | Knowledge and RAG | Implemented ingestion/provenance and lexical retrieval slices; Phase 4 in progress | `nexa-knowledge` |
| 06 | NEXA-LESSON-001 | Curriculum and lessons | Implemented Phase 3 slice | `nexa-lessons` |
| 07 | NEXA-ASMT-001 | Assessment | Implemented Phase 3 slice | `nexa-assessment` |
| 08 | NEXA-LAB-001 | Labs and sandboxes | Baseline Draft | `nexa-labs` |
| 09 | NEXA-SPCH-001 | Speech interaction | Baseline Draft | `nexa-speech` |
| 10 | NEXA-AVTR-001 | Renderer-neutral avatar | Implemented Phase 2 slice | `nexa-avatar` |
| 11 | NEXA-3D-001 | 3D character architecture | Implemented Phase 2 slice | `nexa-3d` |
| 11 | NEXA-3D-ART-001 | 3D production pipeline | Baseline Draft | Assets/tooling |
| 11 | NEXA-3D-REF-001 | Canonical 3D reference | Baseline Draft | Assets |
| 11 | NEXA-3D-RUNTIME-001 | 3D validation runtime | Implemented slice | `crates/nexa-3d`; `apps/nexa-3d-viewer`; `tools/nexa-3d-validate` |
| 12 | NEXA-ORCH-001 | Session orchestration | Baseline Draft | `nexa-orchestrator` |

Directories 13 through 27 reserve future specification namespaces. A reserved directory is not an approved subsystem contract.

## Authority rules

1. The registry identifies the current working authority; filenames alone do not.
2. Reconstructed text is preserved and reviewed before semantic rewriting.
3. Specifications govern behavior; ADRs record cross-cutting implementation decisions.
4. Code that intentionally diverges from a baseline specification must cite an ADR or tracked issue.
5. NEXA-CBS-001 owns character identity. NEXA-NBP-001 owns semantic behavior. Renderer specifications own physical realization.
6. The tutor/LLM never directly selects animation clips, bones, blendshapes, or renderer operations.

## Known baseline work

- Normalize Markdown lost during reconstruction without changing meaning.
- Reconcile duplicate and relocated 3D specification material.
- Validate every dependency declaration against this registry.
- Add ownership, acceptance criteria, conformance tests, and implementation links.
- Promote specifications from Baseline Draft only through explicit review.
