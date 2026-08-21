# Phase 4 traceability — governed ingestion and retrieval increments

| Requirement | Contract/evidence | Status |
|---|---|---|
| Canonical knowledge and retrieval identities | `nexa-domain` UUID newtypes | Implemented slice |
| Explicit SHA-256 integrity | `ContentHash`; known-vector tests | Implemented slice |
| Governed source/version and metadata provenance | `KnowledgeSource`, closed vocabularies | Implemented slice |
| Exact original artifact and bounded UTF-8 input | `KnowledgeArtifact` | Implemented slice |
| Deterministic structural provenance | `structural_ranges`, `KnowledgeChunk` validation | Implemented slice |
| Lifecycle, replay, and atomic persistence | `IngestionJob`, `KnowledgeUnitOfWork`, in-memory adapter | Implemented slice |
| Visibility and assessment protection | `exposure`; pre-score retrieval exclusions | Implemented slice |
| Versioned lexical query and result contracts | `RetrievalQuery`, `RetrievalFilters`, `RetrievalCandidate`, `RetrievalResult` | Implemented slice |
| Repository-neutral corpus reading | `KnowledgeRetrievalReader`, validated `InMemoryRetrievalSnapshot` | Implemented slice |
| Deterministic tokenization and scoring | ADR-0016; exact term evidence and stable tie-breaking tests | Implemented slice |
| Course/lesson retrieval scope | exact optional filter matching and exclusions | Implemented slice |
| Governed embedding identities, profiles, fingerprinting, and chunk binding | `EmbeddingProfile`, `ChunkEmbedding`; ADR-0017 | Implemented slice |
| Exact deterministic vector arithmetic and evidence | signed `i16` vectors, checked ordered `i64` dot product, stable UUID tie-breaking | Implemented slice |
| Repository-neutral vector corpus and immutable validation | `VectorRetrievalReader`, `InMemoryVectorSnapshot` | Implemented slice |
| Governance, scope, missing embedding, and result-limit exclusions | `VectorExclusionReason`; pre-ranking policy tests | Implemented slice |
| Reference-only vector query/result contracts and redaction | `VectorRetrievalQuery`, `VectorRetrievalResult`; wire/privacy tests | Implemented slice |
| Exact hybrid fusion and provider-free reranking | `HybridFusionRequest`, exact reduced `HybridScore`, governed reconciliation, reference-only evidence; ADR-0018 | Implemented slice |
| Governed context assembly, whole-chunk packing, and exact token budgeting | `ContextAssemblyRequest`, `ContextPackage`, standalone wire validation; ADR-0019 | Implemented slice |
| Governed deterministic citation resolution | `CitationRequest`, `SourceLocationEvidence`, `CitationResult`, standalone wire validation; ADR-0020 | Implemented slice |
| Semantic citation fidelity and tutor behavior | Explicitly deferred by ADR-0020 | Not implemented |
| Model providers, networking, vector databases, and durable adapters | Explicitly deferred by ADR-0017 | Not implemented |

Phase 4 is **in progress**. Governed caller-supplied embeddings and deterministic vector retrieval are implemented, as are deterministic prompt compilation, strict structural output admission, single-attempt invocation-to-admission composition, provider-neutral in-memory registry mechanics, static deterministic single-model selection, explicit local-only selection-to-single-attempt admission composition, and deterministic caller-supplied availability-gated selection. Learned/cross-encoder reranking, authority/freshness ranking, partial truncation, dynamic health/latency/cost/task-complexity routing, automatic local-first policy, fallback/retry, privacy filtering and remote authorization, provider tokenization, semantic citation fidelity/entailment, semantic safety, tutor inference and concrete providers, networking, vector databases, and durable adapters remain unimplemented. ADR-0025 still accepts an explicitly supplied provider; ADR-0028 adds a separate explicit local-only composition and does not implement automatic local-first routing. The reconstructed knowledge and tutor specifications retain their registry authority; NEXA-TUTOR-001 remains Baseline Draft and these increments do not promote it. The known inconsistency between implemented ingestion/context evidence here and unchecked corresponding roadmap bullets is intentionally not resolved by this increment.

## Tutor-response planning increment

| Requirement | Contract/evidence | Status |
|---|---|---|
| Provider-neutral structured tutor boundary | `nexa-tutor` request, closed sections/capabilities, and response package; ADR-0021 | Implemented slice |
| Deterministic knowledge citation binding | Exact context/citation/query/hybrid identities and ordered claim/citation positions | Implemented slice |
| Reference-only learner and pedagogy integration | Versioned `DecisionEvidence` with exact learner/course/lesson/session scope | Implemented slice |
| Structural safety and assessment precedence | Closed classifications, fail-closed restrictions, constrained/refusal status | Implemented slice |
| Standalone replay and privacy | SHA-256 replay anchor, validating decode, redacted content Debug | Implemented slice |
| Semantic safety, entailment, prose/model generation, providers and durable adapters | Explicitly deferred by ADR-0021 | Not implemented |

## Model-invocation increment

| Requirement | Contract/evidence | Status |
|---|---|---|
| Canonical provider, model, and invocation identities | `ModelProviderId`, `ModelId`, `ModelInvocationId`; ADR-0022 | Implemented slice |
| Provider-neutral bounded invocation contracts | `ModelDescriptor`, `ModelRequest`, `ModelResponse`, validating wire decode | Implemented slice |
| Capability, limit, identity, and version enforcement | Closed capabilities and fail-closed validation before adapter consumption | Implemented slice |
| Untrusted content and privacy boundary | Opaque bounded input/output and redacted `Debug`/errors | Implemented slice |
| Deterministic provider-independent testing | Synchronous port and FIFO `ScriptedModelProvider` | Implemented slice |
| Concrete inference, provider integration, routing, async/streaming, and semantic safety | Explicitly deferred by ADR-0022 | Not implemented |

## Prompt-compilation increment

| Requirement | Contract/evidence | Status |
|---|---|---|
| Closed canonical layers and derived authority/trust | `PromptLayerKind`, `LayerClassification`, canonical order; ADR-0023 | Implemented slice |
| Version and byte-bound compilation into ADR-0022 input | `PromptCompilationRequest`, `PromptLimits`, `compile_prompt` | Implemented slice |
| Unambiguous deterministic framing and content preservation | fixed canonical JSON envelope, explicit position/length metadata | Implemented slice |
| Content-free audit and standalone replay integrity | manifest, exact byte count, SHA-256 anchor, validating decode | Implemented slice |
| Final provider-neutral context validation | integration with `ModelRequest::validate_for`; ADR-0022 remains authoritative | Implemented slice |
| Inference, provider routing/tokenization, semantic safety/grounding, repair, async/streaming, networking, persistence | Still deferred after ADR-0023; output admission is addressed separately by ADR-0024 | Not implemented |

## Model-output admission increment

| Requirement | Evidence | Status |
|---|---|---|
| Caller-owned identities, policies, limits, capability permissions, and decision evidence | `TrustedPlanningAuthority`; ADR-0024 | Implemented slice |
| Closed model-owned V1 candidate sections and strict JSON decoding | `CandidateOutputV1`, `serde(deny_unknown_fields)`, bounded `RawModelOutput` | Implemented slice |
| Descriptor/request/response identities and exact compiled-input association | `admit_model_output`, intrinsic `PromptCompilationResult::validate` | Implemented slice |
| Output-limit rejection with no repair or partial success | `FinishReason::Complete` gate; admission tests | Implemented slice |
| Existing policy, pedagogy, safety, capability, provenance, and citation-reference validation | Delegation to unchanged `plan_response` with exact governed inputs | Implemented slice |
| Content-free raw-output, prompt, response, and admission replay binding | `AdmissionEvidence`, `AdmissionResult` standalone validation | Implemented slice |
| Truth, semantic safety, prompt-injection detection, entailment, hallucination control, instructional quality | Explicitly not established by ADR-0024 | Not implemented |
| Inference/providers, routing/tokenization, repair/regeneration, tool execution, async/streaming, networking/persistence | Explicitly deferred by ADR-0024 | Not implemented |

## Invocation-to-admission composition evidence

| Requirement | Evidence | Status |
|---|---|---|
| Host validation before provider consumption | Shared admission preflight plus scripted-provider `remaining()` tests | Implemented slice |
| Exactly one synchronous invocation | `invoke_and_admit_model_output`; success, invocation-error, and admission-error FIFO tests | Implemented slice |
| Reuse of ADR-0024 admission and existing result | Coordinator delegates to the factored post-preflight admission path and returns `AdmissionResult`; equality test against direct admission | Implemented slice |
| Closed, content-free failure separation | `InvocationAdmissionError::{Preflight, Invocation, Admission}` and redaction tests | Implemented slice |
| Deterministic mock evidence without provider selection | Caller-supplied `ScriptedModelProvider` tests retain the second outcome | Implemented slice |
| Inference/providers, selection/routing/fallback, tokenization, privacy authorization, semantic safety/correctness, retry/repair/regeneration, async/streaming, networking/persistence | Explicitly deferred by ADR-0025; partial truncation remains deferred | Not implemented |

## Model registry mechanics

| Requirement | Evidence | Status |
|---|---|---|
| Atomic validation and duplicate rejection | `ModelRegistry::try_from_providers`; ADR-0022 descriptor validation; focused invalid/version/duplicate tests | Implemented slice |
| Deterministic read-only inventory | Canonical `ModelProviderId`, then `ModelId`, ordering independent of insertion | Implemented slice |
| Exact shared-provider resolution | Exact-pair lookup, missing-pair errors, and `Arc::ptr_eq` identity evidence | Implemented slice |
| No provider consumption and content-free diagnostics | Scripted FIFO preservation, counting-provider boundary, and redaction tests; ADR-0026 | Implemented slice |
| Static selection | Deferred by ADR-0026 and implemented separately by ADR-0027 | Implemented slice |
| Dynamic routing, automatic local-first policy, fallback, privacy authorization, concrete providers, and inference | Still deferred; ADR-0025 continues to require an explicitly supplied provider | Not implemented |


## Deterministic model selection

| Requirement | Evidence | Status |
|---|---|---|
| Closed, standalone-validating, content-free caller requirements | `ModelSelectionRequirements`, strict V1 wire decode; ADR-0027 | Implemented slice |
| Shared ADR-0022 capability and conservative capacity eligibility | Factored descriptor check used by selection and unchanged `ModelRequest::validate_for` behavior | Implemented slice |
| Explicit privacy eligibility and deterministic total ordering | Caller privacy position, canonical provider identity, then canonical model identity | Implemented slice |
| Exactly one original registered handle without provider consumption | `SelectedModel`, `Arc::ptr_eq`, scripted FIFO preservation, insertion-order tests | Implemented slice |
| Dynamic routing, automatic local-first policy, fallback/retry, privacy filtering/authorization, provider integration/inference | Explicitly deferred by ADR-0027; ADR-0025 remains explicitly supplied | Not implemented |

## Explicit local-only selection-to-admission composition evidence

| Requirement | Evidence | Status |
|---|---|---|
| Fail-closed explicit local-only requirements | `select_local_model_invoke_and_admit`; version, output, structured-output, and exact singleton-privacy tests; ADR-0028 | Implemented slice |
| Exact ADR-0027 selection and ADR-0022 request construction | Exact compiled `ModelInput`, unchanged requirements, selected descriptor identities, and caller invocation identity tests | Implemented slice |
| Reuse of ADR-0025 single-attempt preflight/invocation/admission | Nested `SelectedInvocationAdmissionError`; wrapper-specific `local_selection_preflight_association_matrix_consumes_no_provider`, `local_selection_post_invocation_admission_failures_are_single_attempt`, `local_selection_response_identity_mismatch_reaches_admission_once`, and invocation-failure tests | Implemented slice |
| Canonical selection independent of registry insertion order | Multiple eligible local-provider order tests | Implemented slice |
| Local-only privacy, eligibility, and untouched non-selected state | Wrapper-specific remote-only and independently ineligible capability/output/context tests; scripted outcome retention for selected preflight failures and non-selected local and remote providers on success and post-invocation failure | Implemented slice |
| Original API boundaries | ADR-0025 remains explicitly supplied; ADR-0027 remains standalone and non-invoking | Preserved |
| Automatic local-first routing, remote authorization/privacy filtering, dynamic routing, fallback/retry/repair, inference, tokenization, semantic safety, tools, async/networking/telemetry/persistence, partial truncation | Explicitly deferred by ADR-0028 | Not implemented |

NEXA-TUTOR-001 remains Baseline Draft. The documented inconsistency between implemented ingestion/context assembly evidence and their unchecked roadmap bullets remains intentionally preserved.

## Deterministic availability-gated selection

| Requirement | Evidence | Status |
|---|---|---|
| Closed, bounded, canonical caller-supplied availability evidence | `ModelAvailabilitySnapshot`, strict validating decode, provider/model ordering; ADR-0029 | Implemented slice |
| Exact registry association and missing-is-unavailable behavior | `select_available_model`; unknown-identity, omission, and explicit-unavailable tests | Implemented slice |
| Reuse of ADR-0027 eligibility and ordering | Shared selection implementation; privacy, capability, output, context, and insertion-order tests | Implemented slice |
| Non-invoking original shared handle | `Arc::ptr_eq`, scripted FIFO preservation, descriptor-inconsistency tests | Implemented slice |
| Freshness/authenticity, probing/monitoring, recovery, general routing, fallback/retry, and remote authorization | Explicitly deferred by ADR-0029; ADR-0028 remains unchanged | Not implemented |

NEXA-TUTOR-001 remains Baseline Draft and Phase 4 remains in progress. The known ingestion/context roadmap inconsistency remains intentionally preserved.

## Available explicit local-only selection-to-admission composition

| Requirement | Evidence | Status |
|---|---|---|
| Fail-closed exact local-only requirements | `select_available_local_model_invoke_and_admit`; focused `available_local_selection` malformed/version/output/structured/privacy tests; ADR-0030 | Implemented slice |
| Exact caller availability and deterministic initial selection | Direct ADR-0029 delegation; unavailable-first, available-next, insertion-order, omission, unknown, version, duplicate, and non-canonical tests | Implemented slice |
| Exact request and existing admission evidence | Shared request construction; provider/model/invocation/prompt replay identity assertions; unchanged ADR-0024 `AdmissionResult` | Implemented slice |
| Exactly one selected invocation and no fallback | Invocation/admission failure tests preserve the second selected outcome and all non-selected outcomes | Implemented slice |
| Remote exclusion and content-free diagnostics | Exact `LocalOnly` gate, available-remote untouched evidence, and sentinel diagnostic assertions | Implemented slice |
| Existing API boundaries | ADR-0028 remains availability-free; ADR-0029 remains non-invoking; ADR-0025 remains explicitly supplied | Preserved |
| Probing/monitoring, remote authorization, automatic/general routing, recovery/retry/repair, inference, semantic validation, persistence, partial truncation | Explicitly deferred by ADR-0030 | Not implemented |

NEXA-TUTOR-001 remains Baseline Draft and Phase 4 remains in progress. The known ingestion/context roadmap inconsistency remains intentionally preserved.
