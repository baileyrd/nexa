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
| Context assembly, packing, and token budgeting | Explicitly deferred by ADR-0016 | Not implemented |
| Citation resolution and tutor behavior | Explicitly deferred by ADR-0016 | Not implemented |
| Model providers, networking, vector databases, and durable adapters | Explicitly deferred by ADR-0017 | Not implemented |

Phase 4 is **in progress**. Governed caller-supplied embeddings and deterministic vector retrieval are implemented. Learned/cross-encoder reranking, authority/freshness ranking, context assembly, context packing, token budgeting, citations, tutor intelligence and tutor-response generation, provider integration, networking, vector databases, and durable adapters remain unimplemented. The reconstructed knowledge and tutor specifications retain their registry authority; these increments do not promote them.
