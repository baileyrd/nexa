# Nexa Implementation Roadmap

This roadmap converts the reconstructed design baseline into verifiable increments. Phase boundaries are architectural gates, not calendar promises.

## Phase 0 — Govern the baseline

- Maintain the canonical specification registry.
- Audit dependency declarations, identifiers, duplicates, and reconstruction formatting.
- Define ownership, status transitions, and conformance evidence.
- Repair repository navigation and CI documentation.
- Preserve the working 3D runtime while the workspace is established.

Exit: every active specification has an ID, status, authority, dependencies, and intended implementation boundary.

## Phase 1 — Establish the contract kernel

Implement and test:

- `nexa-domain`
- `nexa-events`
- `nexa-nbp`
- shared error, identifier, time, version, and confidence types

Exit: downstream subsystems consume one canonical set of serialized contracts with schema compatibility tests.

## Phase 2 — Migrate embodiment

**Status:** Complete. The ownership migration and governed embodiment acceptance flow are verified.

- [x] Split the former root package into `crates/nexa-3d`, `apps/nexa-3d-viewer`, and `tools/nexa-3d-validate`.
- [x] Add `nexa-avatar` as the renderer-neutral embodiment port.
- [x] Preserve headless validation, animation, skinning, gaze, and viseme behavior.
- [x] Validate runtime assets and manifests through headless CI gates.
- [x] Complete NBP capability negotiation, governed outputs, typed lifecycle events, and deterministic headless conformance.

The ownership move is recorded in [ADR-0008](../adr/0008-controlled-3d-workspace-migration.md); acceptance and lifecycle semantics are recorded in [ADR-0009](../adr/0009-embodiment-acceptance-and-lifecycle.md).

Exit: the 3D implementation consumes NBP/avatar contracts and all existing tests pass from the workspace root.

## Phase 3 — Build the learning core

**Status:** Complete for the headless deterministic exit gate. Durable production adapters remain a
later infrastructure increment and are not implied by this status.

Implement:

- [x] governed student model, immutable evidence ledger, versioned replay policy, and persistence ports
- [x] pure, versioned, explainable pedagogy policy over read-only mastery projections
- [x] governed curriculum contracts and pure, versioned headless lesson transitions
- [x] dependency-light assessment contracts, deterministic scoring, lifecycle, and evidence creation
- [x] synchronous learning-core composition and explicit atomic unit-of-work port

Exit: **demonstrated** by the `nexa-learning-core` end-to-end conformance and failure-injection tests: a headless deterministic lesson adapts, assesses, appends evidence, replays mastery, routes governed progress, and commits atomically without an LLM or avatar.

See [ADR-0010](../adr/0010-learning-state-evidence-and-persistence.md),
[ADR-0011](../adr/0011-pedagogy-policy-ownership-and-versioning.md),
[ADR-0012](../adr/0012-governed-curriculum-and-lesson-transitions.md),
[ADR-0013](../adr/0013-assessment-contract-scoring-and-evidence.md),
[ADR-0014](../adr/0014-learning-core-composition-and-atomicity.md), and the
[Phase 3 traceability matrix](PHASE-3-TRACEABILITY.md). Durable adapter semantics remain unresolved and are recorded rather than silently selected.

## Phase 4 — Add knowledge and tutor intelligence

**Status:** In progress. The knowledge slices through deterministic citation resolution (ADRs 0015–0020), provider-neutral structured response planning (ADR-0021), provider-neutral model invocation contracts with a deterministic mock (ADR-0022), deterministic provider-neutral prompt compilation (ADR-0023), strict structural model-output admission (ADR-0024), single-attempt invocation-to-admission composition (ADR-0025), provider-neutral in-memory registry mechanics (ADR-0026), static deterministic single-model selection (ADR-0027), explicit local-only selection-to-single-attempt admission composition (ADR-0028), deterministic caller-supplied availability-gated selection (ADR-0029), and availability-gated explicit local-only single-attempt admission composition (ADR-0030), deterministic caller-authorized available remote-model selection (ADR-0031), provider-neutral authorized available remote selection-to-single-attempt invocation/admission (ADR-0032), deterministic caller-directed whole-layer disclosure filtering with filtered compilation (ADR-0033), non-invoking filtered authorized available remote selection (ADR-0034), and filtered-evidence-gated authorized available remote single-attempt invocation/admission (ADR-0035) are implemented. Learned reranking, partial truncation, provider tokenization, semantic safety and prompt-injection detection, factual correctness, grounding entailment and hallucination control, generative inference, concrete provider integration, dynamic health/latency/cost/task-complexity routing, automatic local-first routing, fallback, general privacy policy and semantic/content-level minimization beyond ADR-0033, concrete remote adapters/providers, endpoints, credentials, transport/network execution, response repair/regeneration, tool execution, async/streaming, networking, vector databases, persistence, and durable adapters remain unimplemented.

- Source ingestion and provenance
- [x] deterministic governed lexical retrieval
- [x] governed embedding contracts and deterministic vector retrieval
- [x] exact hybrid fusion and provider-free policy reranking
- context assembly and token budgeting
- [x] provider-neutral structured tutor response planning contracts
- provider-neutral model invocation contracts implemented; concrete provider integration and safety gates remain
- [x] deterministic provider-neutral prompt compilation
- [x] strict provider-neutral model-output decoding and structural admission; inference and semantic validation remain deferred
- [x] deterministic synchronous single-attempt invocation-to-admission composition; concrete providers, selection/routing/fallback, retry, repair, and semantic validation remain deferred
- [x] immutable provider-neutral model registry with validated exact lookup and deterministic inventory; ADR-0025 still requires an explicitly supplied provider
- [x] static provider-neutral eligibility and deterministic single-model selection with explicit caller privacy ordering
- [x] explicit local-only selection, exact request construction, and single-attempt invocation/admission; ADR-0025 remains explicitly supplied and automatic local-first routing remains deferred
- [x] deterministic caller-supplied availability-gated selection without provider probing or invocation
- [x] deterministic availability-gated explicit local-only selection, exactly one invocation, and strict admission
- [x] deterministic caller-authorized available remote selection without provider invocation
- [x] deterministic authorized available remote selection and single-attempt strict admission
- [x] deterministic caller-directed whole-layer disclosure filtering and filtered prompt compilation
- [x] deterministic ADR-0033-evidence-gated authorized available remote selection without invocation
- [x] deterministic filtered-evidence-gated authorized available remote single-attempt invocation and strict admission, without fallback

Exit: grounded responses carry citations, confidence, and machine-validated tutor/behavior output.

## Phase 5 — Orchestrate a complete session

- cancellation-safe session workflow
- speech input/output ports
- behavior synchronization
- tool/lab execution
- interruption, retry, timeout, and recovery policies
- event-driven observability

Exit: a user completes an end-to-end lesson through one composition root.

## Phase 6 — Authoring, packaging, and operations

- course, assessment, lab, and asset compilers
- authoring application
- plugin SDK and capability permissions
- local-first packaging and update strategy
- analytics, privacy, security, and performance gates

Exit: a signed release can be authored, validated, installed, upgraded, observed, and recovered.

## Cross-phase quality gates

Every increment includes:

- unit, contract, integration, and conformance tests
- threat/privacy review proportional to scope
- accessibility checks for user-facing work
- deterministic fixtures where practical
- schema and content versioning
- documentation and traceability updates

### Phase 4 narrow increment: tutor response planning

ADR-0021 adds provider-neutral caller-supplied structured responses, deterministic citation/pedagogy/safety validation, and standalone replay evidence. It does **not** complete Phase 4 or tutor intelligence; semantic safety, semantic entailment, generation, providers, networking, persistence, and durable adapters remain deferred.

### Phase 4 narrow increment: model invocation

ADR-0022 adds a synchronous provider-neutral invocation port, bounded untrusted request/response contracts, capability validation, normalized errors, and a deterministic scripted adapter. It does **not** call a model or connect raw output to response planning; concrete providers, routing, async/streaming execution, and semantic safety remain deferred. Its prompt-compilation deferral is addressed only by the separate ADR-0023 increment below.

### Phase 4 narrow increment: prompt compilation

ADR-0023 adds closed classified prompt layers, canonical version-bound compilation into ADR-0022 `ModelInput`, byte accounting, redaction, and standalone replay evidence. It does **not** invoke a model, route providers, tokenize, decode or admit output, connect raw output to ADR-0021, establish semantic safety/grounding/entailment, repair responses, stream, network, or persist data.

### Phase 4 narrow increment: model-output admission

ADR-0024 adds a caller-owned planning-authority envelope, a closed candidate-section schema, exact descriptor/request/response/prompt binding, fail-closed output-limit handling, and redacted deterministic admission evidence before delegating to ADR-0021. It validates syntax, schema, identity, provenance, policy references, capabilities, citation references, and existing planner structure only. Inference and concrete providers; routing and tokenization; semantic safety and prompt-injection detection; factual correctness, entailment, and hallucination control; repair/regeneration; tool execution; async/streaming; networking; and persistence remain deferred.

### Phase 4 narrow increment: invocation-to-admission composition

ADR-0025 adds shared host-input preflight, exactly one synchronous call to a caller-supplied provider, and reuse of ADR-0024 admission with closed preflight, invocation, and admission failures. It does not complete the Phase 4 exit gate. Actual inference and concrete providers; selection, routing, fallback, provider tokenization, and privacy filtering/authorization; semantic safety, correctness, entailment, and prompt-injection resistance; repair/regeneration; async/streaming; networking; and persistence remain deferred.

### Phase 4 narrow increment: model registry mechanics

ADR-0026 adds atomic validated construction of an immutable in-memory registry, canonical provider-then-model inventory, and exact shared-provider resolution with no invocation. ADR-0025 still accepts an explicitly supplied provider. Static selection is addressed separately by ADR-0027. Dynamic routing, automatic local-first policy, fallback, privacy authorization, concrete provider integration, inference, partial truncation, and the other Phase 4 deferrals remain unimplemented; NEXA-TUTOR-001 remains Baseline Draft.

### Phase 4 narrow increment: deterministic model selection

ADR-0027 adds static descriptor eligibility and deterministic single-choice selection over ADR-0026. Caller-supplied privacy order precedes canonical provider/model identity tie-breaking, and the original registered `Arc` is returned without invocation. ADR-0025 remains unchanged and explicitly supplied. Dynamic availability, latency/cost/task-complexity routing, automatic local-first policy, fallback/retry, concrete providers/inference, provider tokenization, privacy filtering/remote authorization, semantic validation, tools, streaming, networking, persistence, and partial truncation remain deferred; NEXA-TUTOR-001 remains Baseline Draft.

### Phase 4 narrow increment: explicit local-only selection and admission

ADR-0028 adds deterministic explicit `LocalOnly` selection, exact ADR-0022 request construction, and reuse of ADR-0025's single-attempt invocation/admission operation. ADR-0025's original API remains explicitly supplied and ADR-0027 remains independently non-invoking. This is not automatic local-first routing. Remote authorization and privacy filtering; dynamic health/availability, latency, cost, and task-complexity routing; fallback/capability degradation; retry/repair/regeneration; concrete providers/inference; provider tokenization; semantic validation/safety; tools; async/streaming; networking; telemetry export; persistence/durable adapters; and partial truncation remain deferred. NEXA-TUTOR-001 remains Baseline Draft.

### Phase 4 narrow increment: availability-gated model selection

ADR-0029 adds a bounded caller-supplied availability snapshot that gates ADR-0027 eligibility while preserving its privacy and canonical ordering. Missing and explicitly unavailable models are excluded, unknown registry identities fail closed, and selection remains non-invoking. It does not establish health probing, freshness/authenticity, monitoring, recovery, general routing, fallback/retry, automatic local-first policy, remote authorization/privacy filtering, or any previously deferred provider, inference, networking, async/streaming, telemetry, persistence, tokenization, semantic-validation, or partial-truncation capability. ADR-0028 remains unchanged and does not consume the snapshot; NEXA-TUTOR-001 remains Baseline Draft and Phase 4 remains in progress.

### Phase 4 narrow increment: available explicit local-only selection and admission

ADR-0030 composes caller-supplied ADR-0029 availability gating with exact ADR-0022 request construction and ADR-0025's single-attempt invocation/admission, while requiring exactly `LocalOnly`. Initial exclusion of unavailable models is deterministic eligibility, not fallback; after one selection there is no recovery chain. ADR-0028 remains unchanged without implicit availability, ADR-0029 remains non-invoking, and ADR-0025 remains explicitly supplied. Health probing, remote authorization, general/automatic routing, fallback/retry/repair, and all prior deferrals including partial truncation remain unimplemented. NEXA-TUTOR-001 remains Baseline Draft and Phase 4 remains in progress.

### Phase 4 narrow increment: caller-authorized available remote selection

ADR-0031 adds bounded caller-supplied remote authorization bound to exact ADR-0023 replay evidence, then intersects it with ADR-0029 availability and unchanged ADR-0027 deterministic eligibility and ordering. Selection is non-invoking; omission denies authorization. It adds no filtering, minimization, remote execution, authenticity/freshness proof, local-first routing, fallback/recovery, or other previously deferred capability. `partial truncation` remains deferred and NEXA-TUTOR-001 remains Baseline Draft.

### Phase 4 narrow increment: authorized available remote selection and admission

ADR-0032 composes unchanged ADR-0031 with exact ADR-0022 request construction and ADR-0025's single-attempt invocation and strict admission. Caller authorization of the exact compiled prompt is the permission boundary. There is no second selection, fallback, retry, filtering/minimization proof, concrete provider, inference, networking, endpoint, credential, or authenticity/freshness proof. Existing deferrals including `partial truncation` remain; NEXA-TUTOR-001 remains Baseline Draft.

### Phase 4 narrow increment: remote prompt whole-layer disclosure filtering

ADR-0033 adds deterministic caller-directed whole-layer disclosure filtering and filtered ADR-0023 compilation after ADR-0032. Mandatory ADR-0023 layers fail closed; optional layers are included byte-for-byte or omitted in full. This is non-invoking and does not change ADR-0031 or ADR-0032. General privacy policy/enforcement, semantic minimization or sensitivity inference, content redaction, and `partial truncation` remain unimplemented. NEXA-TUTOR-001 remains Baseline Draft, the privacy specification namespace remains reserved, and the known ingestion/context roadmap inconsistency is preserved.

### Phase 4 narrow increment: filtered authorized available remote selection

ADR-0034 validates complete ADR-0033 evidence, requires exact singleton target-privacy agreement, and delegates the exact filtered compilation to unchanged ADR-0031 selection. It does not invoke ADR-0032. General privacy policy and correctness, sensitivity inference, semantic/content minimization, field/sub-string redaction, anonymization, partial truncation, and all concrete provider, routing, recovery, semantic-validation, tool, async/networking, telemetry, and persistence capabilities remain deferred. NEXA-TUTOR-001 remains Baseline Draft; the privacy namespace remains reserved; the known ingestion/context checklist inconsistency is preserved.


### Phase 4 narrow increment: filtered authorized available remote invocation and admission

ADR-0035 composes unchanged ADR-0034 as the only selection with the existing exact ADR-0022 request construction and unchanged ADR-0025 preflight, one invocation, and strict ADR-0024 admission. The request input and successful admission replay binding are the exact filtered ADR-0033 compilation. ADR-0034 remains independently non-invoking and ADR-0032 remains independently callable without ADR-0033 evidence. This is not general or automatic routing and adds no fallback, retry, repair, recovery, concrete provider/inference, networking, semantic privacy proof, content redaction, anonymization, partial truncation, tools, async/streaming, telemetry, or persistence. NEXA-TUTOR-001 remains Baseline Draft, the privacy namespace remains reserved, and Phase 4 remains in progress.
