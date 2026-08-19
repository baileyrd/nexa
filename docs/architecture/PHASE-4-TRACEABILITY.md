# Phase 4 traceability — governed ingestion and lexical retrieval increments

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
| Vector/hybrid retrieval and reranking | Explicitly deferred by ADR-0016 | Not implemented |
| Context assembly, packing, and token budgeting | Explicitly deferred by ADR-0016 | Not implemented |
| Citation resolution and tutor behavior | Explicitly deferred by ADR-0016 | Not implemented |
| Model providers, networking, and durable adapters | Explicitly deferred by ADR-0016 | Not implemented |

Phase 4 is **in progress**. Embeddings, vector/hybrid retrieval, reranking, authority/freshness ranking, context assembly, context packing, token budgeting, citation resolution, tutor intelligence, model-provider integration, networking, and durable adapters remain unimplemented. The reconstructed knowledge and tutor specifications retain their registry authority; these increments do not promote them.
