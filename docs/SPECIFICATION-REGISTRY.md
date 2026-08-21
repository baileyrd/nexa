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
| ADR-0017 | [Governed embeddings and deterministic vector retrieval](adr/0017-governed-vector-retrieval.md) | Accepted | Phase 4 embedding profiles, exact integer vectors, validated snapshots, governance filtering, and stable vector ranking |
| ADR-0018 | [Exact hybrid fusion and provider-free reranking](adr/0018-deterministic-hybrid-fusion.md) | Accepted | Phase 4 exact rank fusion, governance reconciliation, evidence, and stable reranking |
| ADR-0019 | [Governed deterministic context assembly](adr/0019-governed-context-assembly.md) | Accepted | Phase 4 governed material reconciliation, exact token accounting, whole-chunk packing, and replay evidence |
| ADR-0020 | [Governed deterministic citation resolution](adr/0020-governed-deterministic-citation-resolution.md) | Accepted | Phase 4 claim-to-context evidence, closed source locators, deterministic resolution, and standalone replay |
| ADR-0021 | [Provider-neutral tutor response planning](adr/0021-provider-neutral-tutor-response-planning.md) | Accepted | Phase 4 structured caller content, citation binding, structural safety, and replay |
| ADR-0022 | [Provider-neutral model invocation](adr/0022-provider-neutral-model-invocation.md) | Accepted | Phase 4 dependency-light invocation contracts, capability validation, redaction, and deterministic mock |
| ADR-0023 | [Deterministic provider-neutral prompt compilation](adr/0023-deterministic-provider-neutral-prompt-compilation.md) | Accepted | Phase 4 closed prompt layers, canonical compilation, redaction, and replay evidence |
| ADR-0024 | [Provider-neutral model-output admission](adr/0024-provider-neutral-model-output-admission.md) | Accepted | Phase 4 strict candidate decoding, trusted planning authority, planner admission, and redacted replay evidence |
| ADR-0025 | [Single-attempt provider-neutral invocation-to-admission composition](adr/0025-single-attempt-provider-neutral-invocation-admission.md) | Accepted | Phase 4 preflight, exactly-once synchronous provider invocation, and reuse of strict output admission |
| ADR-0026 | [Provider-neutral in-memory model registry](adr/0026-provider-neutral-in-memory-model-registry.md) | Accepted | Phase 4 validated exact provider/model registration, deterministic inventory, and exact resolution without invocation |
| ADR-0027 | [Deterministic provider-neutral model selection](adr/0027-deterministic-provider-neutral-model-selection.md) | Accepted | Phase 4 static eligibility and deterministic single-model selection without invocation |
| ADR-0028 | [Deterministic local-only selection-to-invocation-to-admission composition](adr/0028-deterministic-local-only-selection-invocation-admission.md) | Accepted | Phase 4 explicit local-only single-selection and single-attempt admission composition |
| ADR-0029 | [Deterministic provider-neutral availability-gated selection](adr/0029-deterministic-provider-neutral-availability-gated-selection.md) | Accepted | Phase 4 caller-supplied availability eligibility for deterministic non-invoking selection |
| ADR-0030 | [Deterministic available local selection-to-invocation-to-admission composition](adr/0030-deterministic-available-local-selection-invocation-admission.md) | Accepted | Phase 4 caller-availability-gated explicit local-only single-attempt admission composition |
| ADR-0031 | [Deterministic caller-authorized available remote-model selection](adr/0031-deterministic-caller-authorized-remote-model-selection.md) | Accepted | Phase 4 caller-authorized, availability-gated, non-invoking remote selection |
| ADR-0032 | [Deterministic authorized available remote selection-to-invocation-to-admission composition](adr/0032-deterministic-authorized-available-remote-selection-invocation-admission.md) | Accepted | Phase 4 prompt-authorized available remote selection and single-attempt strict admission |
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
| 05 | NEXA-KNOW-001 | Knowledge and RAG | Implemented ingestion/provenance, retrieval, hybrid fusion, deterministic context assembly, and citation-resolution slices; Phase 4 in progress | `nexa-knowledge` |
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

### Phase 4 implementation decisions

- [ADR-0021](adr/0021-provider-neutral-tutor-response-planning.md) records the narrow provider-neutral tutor-response planning boundary. It implements structural validation only and does not supersede NEXA-TUTOR-001, NEXA-KNOW-001, NEXA-STU-001, NEXA-PED-001, or NEXA-ASMT-001.
- [ADR-0022](adr/0022-provider-neutral-model-invocation.md) records the synchronous provider-neutral invocation port and deterministic scripted adapter. It does not implement inference, provider integration, generation, or semantic safety.
- [ADR-0023](adr/0023-deterministic-provider-neutral-prompt-compilation.md) records deterministic compilation of classified, bounded prompt layers into ADR-0022 `ModelInput`. It does not invoke a provider, accept model output, or provide semantic safety.
- [ADR-0024](adr/0024-provider-neutral-model-output-admission.md) records strict admission of model-owned candidate sections into the existing ADR-0021 planner under caller-owned authority. It provides structural validation, not inference, truth, entailment, instructional quality, or semantic safety.
- [ADR-0025](adr/0025-single-attempt-provider-neutral-invocation-admission.md) records the single-attempt synchronous composition of host-input preflight, one caller-supplied provider invocation, and ADR-0024 admission. It adds no provider selection, inference, retry, fallback, repair, or semantic-safety capability.
- [ADR-0026](adr/0026-provider-neutral-in-memory-model-registry.md) records immutable validated registration, canonical provider/model inventory, and exact shared-provider resolution without invocation. ADR-0025 still requires an explicitly supplied provider; static selection is addressed by ADR-0027 while dynamic routing, automatic local-first policy, fallback, privacy authorization, concrete providers, and inference remain unimplemented.
- [ADR-0027](adr/0027-deterministic-provider-neutral-model-selection.md) records static descriptor eligibility and deterministic single-model selection over ADR-0026, with explicit caller privacy ordering and no provider invocation. Dynamic routing, automatic local-first policy, fallback, privacy filtering/authorization, concrete providers, and inference remain unimplemented.

- [ADR-0028](adr/0028-deterministic-local-only-selection-invocation-admission.md) records explicit local-only selection, exact request construction, and reuse of ADR-0025 single-attempt admission. It is not automatic local-first routing and adds no remote authorization, fallback, retry, concrete provider, or inference. ADR-0025 remains explicitly supplied and ADR-0027 remains independently non-invoking.
- [ADR-0029](adr/0029-deterministic-provider-neutral-availability-gated-selection.md) records deterministic caller-supplied availability gating of non-invoking ADR-0027 selection. Missing models are unavailable; probing, freshness, recovery, and remote authorization remain unimplemented.
- [ADR-0030](adr/0030-deterministic-available-local-selection-invocation-admission.md) records explicit `LocalOnly` composition of ADR-0029 selection with exact ADR-0022 request construction and ADR-0025's one invocation and strict admission. Initial availability exclusion is not fallback; no remote authorization or recovery chain is added, and ADRs 0025, 0028, and 0029 retain their existing APIs.
- [ADR-0031](adr/0031-deterministic-caller-authorized-remote-model-selection.md) records caller-supplied prompt-bound remote authorization and its intersection with ADR-0029 availability and unchanged ADR-0027 selection. It is non-invoking and adds no filtering, remote execution, fallback, or authenticity/freshness proof.
- [ADR-0032](adr/0032-deterministic-authorized-available-remote-selection-invocation-admission.md) composes ADR-0031 with exact ADR-0022 request construction and unchanged ADR-0025 single-attempt strict admission. Exact prompt-bound caller authorization is the permission boundary; no filtering/minimization proof, fallback, concrete provider, transport, credential, or inference is added.
