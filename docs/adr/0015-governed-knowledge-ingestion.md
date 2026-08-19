# ADR-0015: Governed knowledge ingestion and provenance

- **Status:** Accepted
- **Date:** 2026-08-19
- **Specification:** NEXA-KNOW-001

## Context

Phase 4 needs verifiable knowledge before retrieval or tutor behavior can be safe. The reconstructed specification leaves durable storage, remote acquisition, ranking, embedding, and promotion coordination choices unresolved.

## Decision

`nexa-knowledge` owns dependency-light source, artifact, ingestion, chunk provenance, visibility, and synchronous persistence contracts. Cross-boundary UUID identities remain in `nexa-domain`. Sources have immutable identity/version and closed authority, trust, origin, status, and visibility values; relevance is deliberately absent. Authored and inferred metadata are separate and timestamped.

V1 accepts only caller-provided UTF-8 Markdown or plain-text bytes, bounded at 8 MiB. It preserves exact original bytes. SHA-256 uses an explicit `1.0` hash contract and lowercase 64-character hexadecimal encoding. Media type, encoding, length, digest, and scope are verified. Content is inert untrusted data: no paths, networking, link following, HTML/macro/code execution, or content-bearing errors/events exist.

Structural chunking returns exact half-open original byte ranges and inclusive one-based line ranges. It retains source order and heading paths, treats fenced code and HTML as data, and bounds chunks at 64 KiB on line boundaries. CRLF and LF are distinct original artifacts: neither is normalized. Each chunk binds source/artifact/version, both hashes, range, ordinal, and policy `1.0`.

The lifecycle is closed and transition-validated. Activation is explicit; active versions are never silently replaced. Supersession and rollback preserve old records. Student assessment filtering excludes assessment-protected content in deterministic code. Non-active content is visible only to explicit point-in-time administration.

Persistence is a synchronous unit-of-work: source, artifact, job, and chunks commit atomically. Identical replay is a no-op; conflicting identifier reuse fails before mutation. The in-memory adapter is deterministic and supports stage failure injection.

## Consequences and deferred decisions

This increment provides no retrieval. Lexical retrieval, vector embeddings, hybrid ranking, reranking, context packing, citation resolution, tutor intelligence, provider integration, durable adapters, remote sources, storage engines, update discovery, and multi-process transaction coordination remain unresolved and unimplemented. Phase 4 remains in progress.
