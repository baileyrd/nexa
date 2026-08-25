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
| ADR-0032 | [Deterministic authorized available remote selection-to-invocation-to-admission composition](adr/0032-deterministic-authorized-available-remote-selection-invocation-admission.md) | Accepted | Phase 4 provider-neutral authorized available remote selection-to-single-attempt invocation/admission |
| ADR-0033 | [Deterministic remote prompt layer disclosure filtering](adr/0033-deterministic-remote-prompt-layer-disclosure-filtering.md) | Accepted | Phase 4 caller-directed whole-layer remote disclosure filtering and filtered prompt compilation |
| ADR-0034 | [Deterministic filtered authorized available remote-model selection](adr/0034-deterministic-filtered-authorized-available-remote-selection.md) | Accepted | Phase 4 non-invoking ADR-0033-evidence-gated ADR-0031 remote selection |
| ADR-0035 | [Deterministic filtered authorized available remote invocation and admission](adr/0035-deterministic-filtered-authorized-available-remote-invocation-admission.md) | Accepted | Phase 4 ADR-0034 selection followed by one ADR-0025 invocation and strict admission |
| ADR-0036 | [Provider-neutral model-input tokenization](adr/0036-provider-neutral-model-input-tokenization.md) | Accepted | Phase 4 synchronous exact-model token counting, content-free replay evidence, and deterministic scripted tokenizer |
| ADR-0037 | [Provider-neutral exact model-request token-capacity validation](adr/0037-provider-neutral-exact-model-request-token-capacity-validation.md) | Accepted | Phase 4 non-invoking validation of an existing request and ADR-0036 evidence, preserving mandatory ADR-0022 byte capacity |
| ADR-0038 | [Token-capacity-gated provider invocation and admission](adr/0038-token-capacity-gated-provider-invocation-admission.md) | Accepted | Phase 4 opt-in composition of ADR-0025 single-attempt admission with ADR-0037 exact token-capacity validation |
| ADR-0039 | [Exact tokenization and request-capacity composition](adr/0039-exact-tokenization-request-capacity-composition.md) | Accepted | Phase 4 non-invoking composition of ADR-0036 evidence creation with ADR-0037 request-capacity validation |
| ADR-0040 | [Exact tokenization, single-attempt invocation, and admission composition](adr/0040-exact-tokenization-single-attempt-invocation-admission.md) | Accepted | Phase 4 full-preflight composition of ADR-0039, one supplied-provider invocation, and strict admission |
| ADR-0041 | [Local-only selection, exact tokenization, invocation, and admission](adr/0041-local-only-selection-exact-tokenization-invocation-admission.md) | Accepted | Phase 4 explicit local-only selection followed by exact tokenization, one selected-provider invocation, and strict admission |
| ADR-0042 | [Available local selection, exact tokenization, invocation, and admission](adr/0042-available-local-selection-exact-tokenization-invocation-admission.md) | Accepted | Phase 4 availability-gated explicit local selection followed by exact tokenization, one selected-provider invocation, and strict admission |
| ADR-0043 | [Authorized available remote selection, exact tokenization, invocation, and admission](adr/0043-authorized-available-remote-selection-exact-tokenization-invocation-admission.md) | Accepted | Phase 4 authorized available remote selection followed by exact tokenization, one selected-provider invocation, and strict admission |
| ADR-0044 | [Filtered authorized available remote selection, exact tokenization, invocation, and admission](adr/0044-filtered-authorized-available-remote-selection-exact-tokenization-invocation-admission.md) | Accepted | Phase 4 filter-evidence-gated authorized remote selection followed by exact tokenization, one selected-provider invocation, and strict admission |
| ADR-0045 | [Model-response reported-usage reconciliation](adr/0045-model-response-reported-usage-reconciliation.md) | Accepted | Phase 4 non-invoking equality validation of optional reported input usage against exact tokenization evidence |
| ADR-0046 | [Reported-usage-validated exact-tokenization invocation and admission](adr/0046-reported-usage-validated-exact-tokenization-invocation-admission.md) | Accepted | Phase 4 opt-in exact-tokenization invocation with reported-usage validation before admission |
| ADR-0047 | [Local-only selection with reported-usage-validated exact-tokenization invocation and admission](adr/0047-local-only-selection-reported-usage-validated-exact-tokenization-invocation-admission.md) | Accepted | Phase 4 explicit local-only selection followed by reported-usage-validated exact-tokenization invocation/admission |
| ADR-0048 | [Available local selection with reported-usage-validated exact-tokenization invocation and admission](adr/0048-available-local-selection-reported-usage-validated-exact-tokenization-invocation-admission.md) | Accepted | Phase 4 caller-availability-gated explicit local selection followed by reported-usage-validated exact-tokenization invocation/admission |
| ADR-0049 | [Authorized available remote selection with reported-usage-validated exact-tokenization invocation and admission](adr/0049-authorized-available-remote-selection-reported-usage-validated-exact-tokenization-invocation-admission.md) | Accepted | Phase 4 caller-authorized available remote selection followed by reported-usage-validated exact-tokenization invocation/admission |
| ADR-0050 | [Filtered authorized available remote selection with reported-usage-validated exact-tokenization invocation and admission](adr/0050-filtered-authorized-available-remote-selection-reported-usage-validated-exact-tokenization-invocation-admission.md) | Accepted | Phase 4 filtered caller-authorized available remote selection followed by reported-usage-validated exact-tokenization invocation/admission |
| ADR-0051 | [Deterministic session/workflow lifecycle and cancellation](adr/0051-deterministic-session-workflow-lifecycle-cancellation.md) | Accepted | Phase 5 synchronous session/workflow lifecycle, identity association, and lifecycle cancellation foundation |
| ADR-0052 | [Deterministic workflow cancellation propagation planning](adr/0052-deterministic-workflow-cancellation-propagation-planning.md) | Accepted | Phase 5 synchronous canonical cancellation-propagation planning foundation |
| ADR-0053 | [Provider-neutral workflow cancellation propagation port](adr/0053-provider-neutral-workflow-cancellation-propagation-port.md) | Accepted | Phase 5 synchronous exact-plan propagation-port and scripted-adapter foundation |
| ADR-0054 | [Tokio owned workflow task and cancellation foundation](adr/0054-tokio-owned-workflow-task-cancellation-foundation.md) | Accepted | Phase 5 Tokio owned-task, cancellation-token, join/drain, and abort-on-drop runtime foundation |
| ADR-0055 | [Target-aware workflow task ownership foundation](adr/0055-target-aware-workflow-task-ownership-foundation.md) | Accepted | Phase 5 closed-target task association and hierarchical token ownership foundation |
| ADR-0056 | [Atomic exact-plan runtime cancellation execution](adr/0056-atomic-exact-plan-runtime-execution.md) | Accepted | Phase 5 atomic exact-plan selective cancellation, joining, and non-cancellable evidence foundation |
| ADR-0057 | [Headless Behavior cancellation binding](adr/0057-headless-behavior-cancellation-binding.md) | Accepted | First concrete Phase 5 Behavior cancellation binding in an application composition root |
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
| 12 | NEXA-ORCH-001 | Session orchestration | Baseline Draft; ADR-0051 through ADR-0057 foundations and first Behavior binding implemented | `nexa-orchestrator`; `nexa-orchestrator-runtime`; `apps/nexa-headless` |

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

- [ADR-0033](adr/0033-deterministic-remote-prompt-layer-disclosure-filtering.md) adds deterministic caller-directed whole-layer disclosure filtering and filtered ADR-0023 compilation. It does not establish general privacy filtering, semantic minimization, sensitivity inference, redaction, transmission, or automatic ADR-0031/0032 integration.

- [ADR-0034](adr/0034-deterministic-filtered-authorized-available-remote-selection.md) gates unchanged ADR-0031 selection on valid ADR-0033 evidence and exact singleton target privacy agreement. It is non-invoking and does not invoke ADR-0032 or add general privacy policy.

- [ADR-0035](adr/0035-deterministic-filtered-authorized-available-remote-invocation-admission.md) composes unchanged ADR-0034 selection, shared exact ADR-0022 request construction from the filtered compilation, and unchanged ADR-0025 single-attempt strict admission. It is not general or automatic routing and adds no fallback or duplicate success evidence; ADR-0032 remains independently callable without ADR-0033 evidence.
- [ADR-0036](adr/0036-provider-neutral-model-input-tokenization.md) records the separate synchronous provider-neutral model-input token-counting boundary and deterministic content-free replay evidence. ADR-0036 itself did not integrate that evidence; ADR-0037 later implements a separate existing-request capacity gate, ADR-0038 opt-in composes that gate with one invocation and strict admission, and ADR-0040 opt-in composes evidence creation and that gate with one invocation and strict admission. Concrete/provider tokenizer algorithms and token-count integration into selection, authorization, availability, routing, or provider execution, or into invocation/admission other than the opt-in ADR-0038, ADR-0040, ADR-0041, and ADR-0042 compositions remain unimplemented; ADR-0022 and ADR-0027 conservative byte accounting is unchanged.
- [ADR-0037](adr/0037-provider-neutral-exact-model-request-token-capacity-validation.md) records the separate, independently non-invoking gate over an existing ADR-0022 request and existing ADR-0036 evidence. Conservative byte validation remains mandatory and unchanged; pre-existing selection and invocation APIs remain unchanged, while only the opt-in ADR-0038 composition consumes existing evidence before invocation and admission. No concrete tokenizer or provider is added.
- [ADR-0039](adr/0039-exact-tokenization-request-capacity-composition.md) records the opt-in non-invoking host composition that performs ADR-0022 request preflight before delegating exact evidence creation to ADR-0036 and immediate capacity validation to ADR-0037. It returns the generated replay evidence and adds no tokenizer algorithm or provider invocation.
- [ADR-0040](adr/0040-exact-tokenization-single-attempt-invocation-admission.md) records the opt-in composition that completes shared admission preflight before unchanged ADR-0039, invokes one supplied provider only after exact capacity succeeds, applies unchanged strict admission, and returns both exact tokenization evidence and the existing admission result. It adds no selection, routing, retry, concrete tokenizer, provider, inference, or networking capability.
- [ADR-0041](adr/0041-local-only-selection-exact-tokenization-invocation-admission.md) records the opt-in composition of unchanged explicit local-only ADR-0027 selection with unchanged ADR-0040 exact tokenization, one selected-provider invocation, and strict admission. Conservative byte eligibility remains authoritative for selection; no fallback, retry, token-aware routing, or concrete dependency is added.

- [ADR-0042](adr/0042-available-local-selection-exact-tokenization-invocation-admission.md) records the opt-in composition of unchanged caller-supplied availability-gated local selection with unchanged ADR-0040 exact tokenization, one selected-provider invocation, and strict admission. Availability exclusion precedes tokenization; no fallback, probing, retry, token-aware selection, or concrete dependency is added.

- [ADR-0043](adr/0043-authorized-available-remote-selection-exact-tokenization-invocation-admission.md) records the opt-in composition of unchanged ADR-0031 caller-authorized available remote selection with unchanged ADR-0040 exact tokenization, one selected-provider invocation, and strict admission. ADR-0031 remains the sole permission boundary; no authentication, authorization-policy change, secrets, filtering/minimization proof, concrete remote execution, or networking is added.

- [ADR-0044](adr/0044-filtered-authorized-available-remote-selection-exact-tokenization-invocation-admission.md) records the opt-in composition of unchanged ADR-0034 filter-evidence-gated selection with unchanged ADR-0040 exact tokenization, one selected-provider invocation, and strict admission over the exact filtered compilation. It reuses structural whole-layer filtering and caller authorization and adds no general privacy correctness, semantic minimization, authentication, authorization-policy change, secrets, concrete remote execution, or networking.
- [ADR-0045](adr/0045-model-response-reported-usage-reconciliation.md) records a standalone non-invoking validator for unchanged request, response, and exact tokenization-evidence associations followed by equality of optional reported input usage. Equality establishes agreement only, not tokenizer or provider truth, billing accuracy, output-token correctness, or telemetry authority.

- [ADR-0046](adr/0046-reported-usage-validated-exact-tokenization-invocation-admission.md) records the opt-in composition of unchanged full preflight and exact tokenization/capacity construction, one supplied-provider invocation, unchanged reported-usage validation, and unchanged strict admission. It establishes only structural association and optional reported input-count equality, not provider truth, billing/cost, output-token, authenticity/freshness, or telemetry authority.

- [ADR-0047](adr/0047-local-only-selection-reported-usage-validated-exact-tokenization-invocation-admission.md) records the opt-in composition of unchanged explicit local-only selection and request construction with unchanged reported-usage-validated exact-tokenization invocation/admission. It establishes only structural association and optional reported input-count equality, not tokenizer/provider truth, billing/cost, output-token, authenticity/freshness, telemetry, or semantic authority.
- [ADR-0048](adr/0048-available-local-selection-reported-usage-validated-exact-tokenization-invocation-admission.md) records the corresponding caller-availability-gated explicit local composition. Caller availability remains eligibility evidence without freshness/authenticity, monitoring, recovery, or authorization authority; reported usage establishes only structural association and optional input-count equality without tokenizer/provider truth, billing/cost, output-token, telemetry, or semantic authority.
- [ADR-0049](adr/0049-authorized-available-remote-selection-reported-usage-validated-exact-tokenization-invocation-admission.md) records the caller-authorized available remote composition. ADR-0031 remains the sole permission boundary; caller authorization and availability add no authenticity/freshness or monitoring/recovery authority.
- [ADR-0050](adr/0050-filtered-authorized-available-remote-selection-reported-usage-validated-exact-tokenization-invocation-admission.md) records the filtered caller-authorized available remote composition. ADR-0034 remains the filtered-selection boundary, ADR-0031 remains the sole permission boundary, ADR-0033 remains structural whole-layer evidence only, and reported usage establishes structural association and optional input-count equality only.
