# Phase 4 traceability — governed ingestion increment

| Requirement | Contract/evidence | Status |
|---|---|---|
| Canonical knowledge identities | `nexa-domain` UUID newtypes | Implemented slice |
| Explicit SHA-256 integrity | `ContentHash`; known-vector tests | Implemented slice |
| Governed source/version and metadata provenance | `KnowledgeSource`, closed vocabularies | Implemented slice |
| Exact original artifact and bounded UTF-8 input | `KnowledgeArtifact` | Implemented slice |
| Deterministic structural provenance | `structural_ranges`, `KnowledgeChunk` validation | Implemented slice |
| Lifecycle, replay, and atomic persistence | `IngestionJob`, `KnowledgeUnitOfWork`, in-memory adapter | Implemented slice |
| Visibility enforcement | `exposure` and deterministic exclusion reasons | Implemented slice |
| Retrieval and tutor behavior | Explicitly deferred by ADR-0015 | Not implemented |

Phase 4 is **in progress**. Lexical retrieval, vector embeddings, hybrid ranking, reranking, context packing, citation resolution, tutor intelligence, provider integration, and durable adapters remain unimplemented.
