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

Phase 4 is **in progress**. Governed caller-supplied embeddings and deterministic vector retrieval are implemented. Learned/cross-encoder reranking, authority/freshness ranking, partial truncation, provider tokenization, semantic citation fidelity, tutor intelligence and tutor-response generation, provider integration, networking, vector databases, and durable adapters remain unimplemented. The reconstructed knowledge and tutor specifications retain their registry authority; these increments do not promote them.

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
| Inference, provider routing/tokenization, output admission, semantic safety/grounding, repair, async/streaming, networking, persistence | Explicitly deferred by ADR-0023 | Not implemented |
