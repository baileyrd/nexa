# ADR-0060: Asynchronous cooperative Retrieval service foundation

- Status: Accepted
- Date: 2026-08-25
- Scope: Phase 5 Retrieval service and cancellation foundation

## Context

The Phase 4 `nexa-knowledge` engine is deterministic, synchronous, and runtime-independent. Future database, vector-store, or network integrations inherently need asynchronous execution and in-flight cancellation, but placing those concerns in the governed core would reverse the dependency boundary. Aborting a `spawn_blocking` wrapper would only drop the waiting task and would not prove its underlying synchronous operation stopped.

## Decision

The additive `nexa-knowledge-runtime` crate depends inward on `nexa-domain`, `nexa-knowledge`, Tokio, and `tokio-util`; `nexa-knowledge` remains unchanged and cannot depend outward on this crate or an executor. An object-safe `RetrievalService` accepts one owned, validated existing `RetrievalQuery`, a runtime cancellation token, and returns an erased asynchronous future. The host preserves the exact `RetrievalQueryId` / `RetrievalResultId` association and rejects reassociation.

The closed result distinguishes exact success, cooperative cancellation, and normalized dependency/runtime failure. Cancellation evidence states only that the service future observed cancellation and terminated before returning. It does not claim an external dependency stopped unless a future concrete adapter supplies separate proof. The boundary exposes no task handle and permits no detached work.

A deterministic FIFO scripted adapter records requests, consumes zero or one outcomes, supports a deliberately waiting operation, and tracks active futures. It spawns no task, so completion or caller-future drop leaves no adapter work running. Errors and cancellation evidence are content-free, and query `Debug` remains redacted.

## Consequences and deferrals

The existing `InMemoryRetrievalSnapshot::retrieve`, retrieval semantics, and `nexa-knowledge` dependency surface remain synchronous and unchanged. `spawn_blocking` is neither used nor accepted as cancellation implementation or proof.

Concrete storage, vector databases, networking, providers, persistence, retries, timeouts, recovery, telemetry, and the later `apps/nexa-headless` Retrieval cancellation-plan binding are deferred. Behavior and Tutor Generation bindings are unchanged. Phase 5 remains active and the five-subsystem cancellation gate is not complete.
