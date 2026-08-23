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

Phase 4 is **in progress**. Governed caller-supplied embeddings and deterministic vector retrieval are implemented, as are deterministic prompt compilation, strict structural output admission, single-attempt invocation-to-admission composition, provider-neutral in-memory registry mechanics, static deterministic single-model selection, explicit local-only selection-to-single-attempt admission composition, deterministic caller-supplied availability-gated selection, ADR-0031 prompt-bound caller-supplied remote authorization, ADR-0032 provider-neutral authorized available remote selection-to-single-attempt invocation/admission, ADR-0033 caller-directed whole-layer structural disclosure filtering and filtered compilation, ADR-0034 filter-evidence-gated authorized available remote selection without ADR-0032 invocation, ADR-0035 filtered-evidence-gated authorized available remote single-attempt invocation/admission, ADR-0036's provider-neutral synchronous model-input counting boundary, standalone content-free replay evidence, and deterministic scripted tokenizer, ADR-0037's separate exact token-capacity gate for an existing request and evidence, ADR-0038's opt-in capacity-gated invocation/admission composition, ADR-0039's non-invoking tokenization-to-capacity composition, ADR-0040's exact-tokenization invocation/admission composition, ADR-0041's explicit local-only selection-to-exact-tokenization composition, ADR-0042's availability-gated local selection-to-exact-tokenization composition, ADR-0043's authorized available remote selection-to-exact-tokenization composition, and ADR-0044's filtered authorized available remote selection-to-exact-tokenization composition. Learned/cross-encoder reranking, authority/freshness ranking, partial truncation, dynamic health/latency/cost/task-complexity routing, automatic local-first policy, fallback/retry, general privacy policy, policy correctness, semantic sensitivity inference, semantic/content-level minimization, and field/sub-string redaction beyond ADR-0033, concrete remote adapters/providers, inference, transport/network execution, endpoints, credentials, concrete/provider tokenizer algorithms, token-count integration into selection, authorization, availability, routing, or provider execution, or into invocation/admission other than the opt-in ADR-0038, ADR-0040, ADR-0041, ADR-0042, ADR-0043, and ADR-0044 compositions, semantic citation fidelity/entailment, semantic safety, tutor inference and concrete providers, networking, vector databases, and durable adapters remain unimplemented. ADR-0025 still accepts an explicitly supplied provider; ADR-0028 adds a separate explicit local-only composition and does not implement automatic local-first routing. The reconstructed knowledge and tutor specifications retain their registry authority; NEXA-TUTOR-001 remains Baseline Draft and these increments do not promote it. The known inconsistency between implemented ingestion/context evidence here and unchecked corresponding roadmap bullets is intentionally not resolved by this increment.

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

## Caller-authorized available remote-model selection

| Requirement | Evidence | Status |
|---|---|---|
| Closed bounded prompt-bound authorization allowlist | `RemoteModelAuthorization`, standalone validating decode; ADR-0031 | Implemented slice |
| Exact registry identity and privacy agreement | fail-closed authorization/registry association checks | Implemented slice |
| Independent authorization and availability gates | intersection with ADR-0029 snapshot; omission denies/unavailable | Implemented slice |
| Unchanged deterministic static eligibility and ordering | shared ADR-0027 selection implementation | Implemented slice |
| Non-invoking original shared handle | `Arc::ptr_eq` and scripted FIFO preservation tests | Implemented slice |
| Filtering/minimization, authenticity/freshness, remote execution, routing/fallback/recovery, partial truncation | Explicitly deferred by ADR-0031 | Not implemented |

NEXA-TUTOR-001 remains Baseline Draft and Phase 4 remains in progress. The known ingestion/context roadmap inconsistency remains intentionally preserved.

## Authorized available remote selection, invocation, and admission

| Requirement | Evidence | Status |
| --- | --- | --- |
| Provider-neutral authorized available remote selection-to-single-attempt invocation/admission | `select_authorized_available_remote_model_invoke_and_admit`; focused `authorized_available_remote_invocation` order, insertion, denial, and non-consumption tests; ADR-0032 | Implemented slice |
| Exact ADR-0022 request and unchanged ADR-0025 admission | Shared `request_for_selected` and `invoke_and_admit_model_output`; success/evidence and nested-error focused tests | Implemented slice |
| At-most-once invocation with no fallback | Scripted second-outcome retention and non-selected provider retention across preflight, invocation, and admission failures | Implemented slice |
| Content-free diagnostics | Closed nested error and supplied prompt/response sentinel assertions | Implemented slice |
| Filtering/minimization/sensitivity inference, authenticity/freshness, automatic/general routing, fallback/recovery, concrete providers/inference/networking, partial truncation | Explicitly deferred by ADR-0032 | Not implemented |

NEXA-TUTOR-001 remains Baseline Draft and Phase 4 remains in progress. The known ingestion/context roadmap inconsistency remains intentionally preserved.

## Deterministic remote prompt whole-layer disclosure filtering

| Requirement | Evidence | Status |
| --- | --- | --- |
| Explicit decision for every ADR-0023 layer and remote privacy target | `RemotePromptDisclosurePolicy`; ADR-0033 | Implemented slice |
| Mandatory-layer fail-closed behavior and whole optional-layer omission | `filter_and_compile_remote_prompt`; focused `remote_prompt_filter` tests | Implemented slice |
| Exact source-present partition, policy, filtered compilation, and content-free replay binding | `RemotePromptFilterEvidence` source-present inventory and standalone validation | Implemented slice |
| Provider-neutral non-invoking boundary | No authorization, selection, availability, registry, or generation dependency | Preserved |
| Semantic sensitivity inference, content/field redaction, privacy-policy correctness, automatic ADR-0031/0032 integration, partial truncation | Explicitly deferred by ADR-0033 | Not implemented |

NEXA-TUTOR-001 remains Baseline Draft; the reserved privacy specification namespace remains unimplemented. The known ingestion/context roadmap inconsistency and all existing provider, inference, routing, fallback, semantic-validation, networking, async/streaming, telemetry, and persistence deferrals remain preserved.

## Filter-evidence-gated authorized available remote selection

| Requirement | Evidence | Status |
| --- | --- | --- |
| Complete ADR-0033 evidence and exact singleton target privacy gate | `select_filtered_authorized_available_remote_model`; focused `filtered_authorized_remote_selection` tests; ADR-0034 | Implemented slice |
| Exact filtered compilation delegated to unchanged ADR-0031 | Prompt-anchor association and nested failure tests | Implemented slice |
| Non-invoking original registered shared handle | `Arc::ptr_eq` and scripted FIFO preservation | Implemented slice |
| ADR-0032 invocation, general privacy policy/correctness, semantic filtering/redaction, partial truncation, and existing provider/routing/networking/persistence scope | Explicitly deferred by ADR-0034 | Not implemented |

NEXA-TUTOR-001 remains Baseline Draft; Phase 4 remains in progress; the privacy namespace remains reserved. The known ingestion/context roadmap inconsistency is preserved.

## Filtered authorized available remote invocation and admission

| Requirement | Evidence | Status |
| --- | --- | --- |
| Mandatory complete ADR-0033 evidence and only ADR-0034 selection | `select_filtered_authorized_available_remote_model_invoke_and_admit`; focused `filtered_authorized_available_remote_invocation` tests; ADR-0035 | Implemented slice |
| Exact filtered ADR-0022 request and replay binding | Shared `request_for_selected`; filtered input and `AdmissionResult` anchor assertions | Implemented slice |
| Unchanged ADR-0025 preflight, one invocation, and strict admission | Direct `invoke_and_admit_model_output` delegation; selected second-outcome and non-consumption evidence | Implemented slice |
| Closed content-free nested failures | `FilteredAuthorizedAvailableRemoteInvocationAdmissionError` | Implemented slice |
| Existing API boundaries | ADR-0034 remains non-invoking; ADR-0032 remains callable without ADR-0033 evidence | Preserved |
| General privacy correctness, semantic minimization/redaction/anonymization, authorization authenticity/freshness, partial truncation, routing/fallback/retry/repair/recovery, concrete providers/inference/networking, tools, async/streaming, telemetry, persistence | Explicitly deferred by ADR-0035 | Not implemented |

NEXA-TUTOR-001 remains Baseline Draft; Phase 4 remains in progress; the privacy namespace remains reserved and unimplemented. The known ingestion/context roadmap inconsistency is preserved.

## ADR-0036 provider-neutral model-input tokenization boundary

| Requirement | Evidence | Status |
|---|---|---|
| Exact-model synchronous counting boundary | `ModelInputTokenizer`, existing ADR-0022 `ModelDescriptor` and `ModelInput`, checked `u32` count | Implemented slice |
| Standalone content-free association and replay evidence | `ModelInputTokenizationEvidence`, strict validating decode, UTF-8 byte count, SHA-256 input hash and replay anchor | Implemented slice |
| Deterministic provider-independent testing | FIFO `ScriptedModelInputTokenizer`, preflight non-consumption and exactly-once outcome tests | Implemented slice |
| Conservative capacity behavior preserved | ADR-0036 introduced no API that consumes tokenization evidence, and ADR-0022 validation and ADR-0027 selection remain unchanged and do not consume it; ADR-0037 later added the separate existing-request capacity gate and ADR-0038 opt-in composes it with one invocation and strict admission | Preserved |
| Concrete tokenizers and token-count integration | ADR-0036 itself deferred integration; ADR-0037 later implements the separate existing-request capacity gate and ADR-0038 opt-in composes that gate with one invocation and strict admission. Concrete/provider tokenizer algorithms and integration into selection, authorization, availability, routing, or provider execution, or invocation/admission beyond the opt-in ADR-0038, ADR-0040, ADR-0041, ADR-0042, ADR-0043, and ADR-0044 compositions remain deferred. | Partially implemented |

NEXA-TUTOR-001 remains Baseline Draft; Phase 4 remains in progress; the privacy namespace remains reserved. Partial truncation and the known ingestion/context roadmap inconsistency remain preserved, along with all existing privacy, provider, inference, routing, networking, semantic-validation, async/streaming, telemetry, and persistence deferrals.

## ADR-0037 exact model-request token-capacity validation

| Requirement | Evidence | Status |
|---|---|---|
| Mandatory unchanged ADR-0022 request validation | `validate_model_request_token_capacity` delegates first to `ModelRequest::validate_for`; focused `model_request_token_capacity` tests | Implemented slice |
| Exact association and checked capacity | Existing ADR-0036 `validate_for`; checked `u32` sum; boundary, excess, overflow, and reassociation tests | Implemented slice |
| Non-invoking and non-consuming | Scripted tokenizer and provider FIFO preservation tests | Implemented slice |
| Closed content-free failures without duplicate evidence | `ModelRequestTokenCapacityError`; sentinel diagnostics tests; success is `()` | Implemented slice |
| Existing API behavior | ADR-0027 selection, ADR-0025 invocation/admission, and ADR-0037 capacity validation remain unchanged and independently callable; only the new opt-in ADR-0038 composition requires evidence | Preserved |

NEXA-TUTOR-001 remains Baseline Draft; Phase 4 remains in progress; the privacy namespace remains reserved and unimplemented. Partial truncation, the known ingestion/context roadmap inconsistency, and all existing provider, inference, selection/invocation token integration, routing, privacy, networking, semantic-validation, async/streaming, telemetry, and persistence deferrals remain preserved.

## ADR-0038 token-capacity-gated invocation and admission

| Requirement | Evidence | Status |
|---|---|---|
| Existing shared preflight before capacity validation | `invoke_and_admit_model_output_with_token_capacity`; ordering and non-consumption tests | Implemented slice |
| Exact existing evidence and mandatory conservative validation | Direct delegation to unchanged `validate_model_request_token_capacity`; mismatch, overflow, excess, and equality tests | Implemented slice |
| Exactly one invocation followed by unchanged strict admission | Direct provider call and `admit_model_output_after_preflight`; success, provider-error, and admission-error tests | Implemented slice |
| Closed content-free failure separation | `TokenCapacityInvocationAdmissionError::{Preflight, TokenCapacity, Invocation, Admission}` and focused prompt, model-output, and private-provider-diagnostic sentinel assertions for both `Debug` and `Display` | Implemented slice |
| Existing APIs and tokenizer boundary | ADR-0025 and ADR-0037 APIs remain unchanged; the composition accepts no tokenizer and produces no evidence | Preserved |

NEXA-TUTOR-001 remains Baseline Draft and Phase 4 remains in progress. All concrete tokenizer/provider, selection/routing/fallback, inference, networking, semantic-validation, privacy-policy, async/streaming, telemetry, and persistence deferrals remain preserved.

## ADR-0039 exact tokenization and request-capacity composition

| Requirement | Evidence | Status |
|---|---|---|
| ADR-0022 request preflight before tokenizer consumption | `tokenize_and_validate_model_request_capacity`; focused identity, version, capability, output-limit, and byte-capacity non-consumption tests | Implemented slice |
| Unchanged exact evidence construction and exactly one tokenizer outcome | Direct `tokenize_model_input` delegation; version, descriptor, FIFO, zero-count, and closed-failure tests | Implemented slice |
| Unchanged checked exact-capacity validation | Direct `validate_model_request_token_capacity` delegation; fit, equality, excess, and overflow tests | Implemented slice |
| Exact content-free replay evidence returned | Association, serialization/replay, and redacted `Debug`/`Display` tests | Implemented slice |
| Non-invoking and existing APIs preserved | Provider FIFO preservation; ADR-0036, ADR-0037, and ADR-0038 signatures unchanged | Preserved |

NEXA-TUTOR-001 remains Baseline Draft and Phase 4 remains in progress. The known ingestion/context checklist inconsistency and all concrete tokenizer/provider, routing, inference, networking, semantic-validation, privacy-policy, recovery, async/streaming, telemetry, and persistence deferrals remain preserved.

## ADR-0040 exact tokenization, single-attempt invocation, and admission composition

| Requirement | Evidence | Status |
|---|---|---|
| Complete shared admission preflight before dependency consumption | `tokenize_invoke_and_admit_model_output_with_token_capacity`; all 33 shared-preflight mutations preserve tokenizer and provider outcomes | Implemented slice |
| Unchanged exact tokenization and capacity composition | Direct delegation to ADR-0039 `tokenize_and_validate_model_request_capacity`; version, descriptor, tokenizer failure, zero, exhaustion, internal, excess, overflow, fit, and equality tests | Implemented slice |
| Exactly one invocation and unchanged strict admission | Direct provider call followed by `admit_model_output_after_preflight`; success equality, provider failure, and admission failure tests | Implemented slice |
| Dual exact success evidence and replay | `TokenizedInvocationAdmissionResult`; generated evidence association and serialization/replay validation plus direct-admission equality | Implemented slice |
| Closed content-free failure separation | `TokenizedInvocationAdmissionError::{Preflight, TokenizationCapacity, Invocation, Admission}` and sentinel diagnostics tests | Implemented slice |
| Existing APIs and explicit deferrals | ADR-0025 and ADR-0036 through ADR-0039 signatures unchanged; no selection, routing, concrete dependency, retry, or networking | Preserved |

NEXA-TUTOR-001 remains Baseline Draft and Phase 4 remains in progress. The known ingestion/context checklist inconsistency and all concrete tokenizer/provider, routing, inference, networking, semantic-validation, privacy-policy, recovery, async/streaming, telemetry, and persistence deferrals remain preserved.

## ADR-0041 explicit local-only selection and exact-tokenization composition

| Requirement | Evidence | Status |
|---|---|---|
| Exact ADR-0028 local-only gate before dependency consumption | `select_local_model_tokenize_invoke_and_admit`; malformed privacy-list tests preserve tokenizer and provider outcomes | Implemented slice |
| Unchanged conservative ADR-0027 selection before exact tokenization | Direct `select_model` delegation over the exact compiled input; empty-registry and canonical selection tests | Implemented slice |
| Exact selected request followed by unchanged ADR-0040 | Shared request construction and direct `tokenize_invoke_and_admit_model_output_with_token_capacity` delegation | Implemented slice |
| Exactly one selected tokenizer/provider outcome and exact dual success evidence | Exact-fit/equality and failure-path tests; returned `ModelInputTokenizationEvidence` plus direct `AdmissionResult` equality | Implemented slice |
| Closed, content-free failure separation | `SelectedTokenizedInvocationAdmissionError::{InvalidLocalOnlyRequirements, Selection, TokenizedInvocationAdmission}` and sentinel diagnostics | Implemented slice |
| Existing APIs and deferrals | ADR-0028 and ADR-0040 signatures unchanged; no token-aware selection, fallback, retry, concrete dependency, routing, or networking | Preserved |

NEXA-TUTOR-001 remains Baseline Draft and Phase 4 remains in progress. The known ingestion/context checklist inconsistency and all existing deferrals remain preserved.

## ADR-0042 availability-gated local selection and exact-tokenization composition

| Requirement | Evidence | Status |
|---|---|---|
| Explicit-local validation before dependency consumption | `select_available_local_model_tokenize_invoke_and_admit`; malformed requirement tests preserve tokenizer/provider outcomes | Implemented slice |
| Exact caller snapshot and unchanged deterministic selection | Direct `select_available_model` delegation; invalid, unsupported, inconsistent, missing, unavailable, and next-available tests | Implemented slice |
| Existing request followed by unchanged ADR-0040 | Shared request construction and direct tokenized invocation/admission delegation | Implemented slice |
| One tokenizer and one selected provider with exact success evidence | Equality and failure-path outcome-count tests; exact tokenization evidence and direct admission result | Implemented slice |
| Closed content-free failures | `AvailableLocalTokenizedInvocationAdmissionError` and sentinel diagnostics | Implemented slice |
| Existing APIs and deferrals | ADRs 0029, 0030, 0040, and 0041 unchanged; no probing, token-aware selection, fallback, retry, or concrete dependency | Preserved |

NEXA-TUTOR-001 remains Baseline Draft and Phase 4 remains in progress. The known ingestion/context checklist inconsistency and all existing deferrals remain preserved.

## ADR-0043 authorized available remote selection and exact-tokenization composition

| Requirement | Evidence | Status |
|---|---|---|
| ADR-0031 remains the sole permission and selection boundary | Direct test `authorized_remote_tokenized_composition_denials_preserve_exact_categories_and_dependencies` covers every malformed remote privacy preference, unsupported inputs, replay and registry/privacy association, authorization and availability gates, exact nested errors, and zero dependency consumption | Implemented slice |
| Existing request followed by unchanged ADR-0040 | Direct test `authorized_remote_tokenized_composition_is_exact_single_attempt_and_content_free` covers pre-tokenization failures, exact-capacity validation, invocation, and strict admission | Implemented slice |
| Exact success result and single selected dependency use | Direct test `authorized_remote_tokenized_composition_is_exact_single_attempt_and_content_free` checks replayable tokenization evidence, equality with direct admission, both insertion orders for two eligible authorized remote providers, exact selected shared-handle consumption, and untouched non-selected remote/local providers | Implemented slice |
| Closed content-free failures | Both direct focused tests format the closed errors and cover content-free selection, tokenizer, provider, and admission failure categories | Implemented slice |
| Permission and deferrals preserved | No authentication, authorization-policy change, secrets, filtering/minimization proof, concrete remote execution, networking, retry, fallback, or token-aware selection | Preserved |

NEXA-TUTOR-001 remains Baseline Draft and Phase 4 remains in progress. The known ingestion/context checklist inconsistency and all existing deferrals remain preserved.

## ADR-0044 filtered authorized remote selection and exact-tokenization composition

| Requirement | Evidence | Status |
|---|---|---|
| ADR-0034 remains the sole filter-evidence-gated selection boundary | `select_filtered_authorized_available_remote_model_tokenize_invoke_and_admit`; direct tampered-evidence dependency-preservation test | Implemented slice |
| Exact filtered request followed by unchanged ADR-0040 | Shared request construction from `filtered_result.filtered_compilation` and direct tokenized invocation/admission delegation | Implemented slice |
| Exact success evidence and deterministic original-handle use | Focused filtered tokenized success test covers both registry insertion orders, exact filtered tokenization association, direct admission equality, and untouched non-selected provider | Implemented slice |
| Closed content-free failures | Direct `filtered_remote_tokenized_composition_denials_preserve_exact_categories_and_dependencies` and `filtered_remote_tokenized_composition_is_exact_single_attempt_and_content_free` sentinel tests format `Debug` and `Display` across the nested ADR-0034 and ADR-0040 categories | Implemented slice |
| Privacy, permission, and deferrals preserved | Existing structural whole-layer filtering and caller authorization are reused; no general privacy correctness, semantic minimization, authentication, authorization-policy change, secrets, concrete remote execution, or networking | Preserved |

NEXA-TUTOR-001 remains Baseline Draft and Phase 4 remains in progress. The known ingestion/context checklist inconsistency and all existing deferrals remain preserved.

## ADR-0045 model-response reported-usage reconciliation

| Requirement | Evidence | Status |
|---|---|---|
| Ordered exact association validation | Direct `usage` tests preserve request, response, and tokenization-evidence categories and mandated failure precedence | Implemented slice |
| Optional usage and exact input-count equality | Direct tests cover absent usage, equality, lower/higher mismatch, and unchanged maximum-output validation | Implemented slice |
| Pure, non-consuming, content-free boundary | Direct tests retain scripted tokenizer/provider outcomes and verify every error category excludes private sentinels | Implemented slice |
| Limited claim and unchanged authorities | ADR-0045 establishes equality only; provider usage remains optional evidence without truth, billing, output-token, or telemetry authority; NEXA-TUTOR-001 remains Baseline Draft | Preserved |

## ADR-0046 reported-usage-validated exact-tokenization invocation and admission

| Requirement | Evidence | Status |
|---|---|---|
| Mandatory preflight-to-admission ordering | `tokenize_invoke_validate_reported_usage_and_admit_model_output_with_token_capacity`; direct preflight and ordered failure tests | Implemented slice |
| Exact evidence, single attempt, and dependency accounting | Deterministic scripted tokenizer/provider tests assert exact evidence and remaining outcomes | Implemented slice |
| Optional usage equality before unchanged admission | Direct absent/equal/lower/higher, structural-response, and admission-failure tests | Implemented slice |
| Closed content-free failures and limited authority | Five-category error; diagnostics tests and ADR-0046 exclusions preserve no truth, billing/cost, output-token, authenticity/freshness, or telemetry claims | Preserved |

NEXA-TUTOR-001 remains Baseline Draft and Phase 4 remains in progress. The known ingestion/context checklist inconsistency and all existing deferrals remain preserved.

## ADR-0047 local-only selection with reported-usage-validated exact-tokenization invocation and admission

| Requirement | Evidence | Status |
|---|---|---|
| Explicit-local validation and conservative selection precede dependency consumption | `select_local_model_tokenize_invoke_validate_reported_usage_and_admit`; direct malformed-requirement and empty-registry dependency-preservation tests | Implemented slice |
| Exact selected request followed by unchanged ADR-0046 | Shared ADR-0041 request construction and direct ADR-0046 delegation | Implemented slice |
| Optional usage equality, exact evidence, and single attempt | Direct absent/equal/lower/higher tests assert exact results and dependency counts; ADR-0046 focused tests preserve all nested ordering | Implemented slice |
| Closed content-free failures and limited authority | Three-category outer error preserves complete nested ADR-0046 errors; existing sentinel-bearing diagnostics tests cover those unchanged nested categories | Preserved |
| Existing APIs and explicit deferrals | ADRs 0022 through 0046 unchanged; no availability, authorization, routing, retry, concrete dependency, networking, telemetry, or semantic authority | Preserved |

NEXA-TUTOR-001 remains Baseline Draft and Phase 4 remains in progress. The known ingestion/context checklist inconsistency and all existing deferrals remain preserved.
