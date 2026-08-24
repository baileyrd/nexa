# ADR-0054: Tokio owned workflow task and cancellation foundation

- Status: Accepted
- Date: 2026-08-24
- Scope: Narrow Phase 5 runtime owned-task/cancellation foundation

## Context

ADRs 0051–0053 establish synchronous lifecycle cancellation, canonical propagation planning, and an exact-plan propagation port. They deliberately defer runtime infrastructure and do not prove that work stopped. NEXA-ORCH-001 sections 57–61 recommend structured concurrency, workflow ownership, cancellation tokens, propagation to five subsystem categories, and explicit treatment of non-cancellable actions. The repository owner approved Tokio and a bounded first runtime increment without approving those subsystem integrations.

## Decision

Tokio is the Phase 5 async runtime implementation. A new `nexa-orchestrator-runtime` adapter depends inward on unchanged `nexa-orchestrator`, while the synchronous contract crate remains free of async-runtime dependencies. `tokio_util::sync::CancellationToken` provides hierarchical workflow cancellation and `tokio::task::JoinSet` provides exclusive task ownership.

One `WorkflowTaskGroup` is constructed with one existing `InteractionWorkflow`, thereby preserving its exact workflow, session, correlation, and trace association. It owns one root token and one join set. Each spawn receives a child token and returns no task handle, so callers cannot detach owned work. Spawning fails after cancellation or draining begins. Bounded inspection exposes only the associated workflow, closed state, task count, and whether cancellation was requested; it exposes neither the root token nor raw task handles.

`cancel_and_wait` requests root cancellation idempotently, drains all owned tasks, and returns only when the join set is empty. Cancellation completion means all owned tasks have joined, failed with the normalized content-free `TaskJoinFailure`, or, in a later subsystem increment, have been explicitly represented as non-cancellable. This increment implements the first two cases only. Repeated cancellation after successful cancellation returns the same identity-preserving evidence without creating work. `drain` separately waits for natural completion and returns evidence whose closed kind does not claim cancellation. Panic and Tokio join failure are normalized only after every remaining task is joined. Dropping the owner invokes abort-on-drop for outstanding work; `JoinSet` retains ownership while cancellation is delivered, so workflow work cannot detach.

No value added here is persisted or transmitted, so no wire format is necessary. Validation-critical state remains private.

## Compatibility and partially resolved deferrals

ADR-0051, ADR-0052, and ADR-0053 APIs and semantics remain unchanged. This decision partially resolves their deferrals of an async runtime, executor/task spawning, cancellation token, background-work ownership, and structured concurrency. It does not bind an ADR-0052 directive or ADR-0053 accepted plan to a live task group and does not turn propagation-port acknowledgement into proof that work stopped.

## Consequences and deferrals

Detached workflow tasks are prohibited by construction, and a workflow owner can now prove that its directly owned Tokio tasks have stopped. This is not full cancellation propagation and does not complete the roadmap checkbox for cancellation-safe execution and propagation.

Explicitly deferred are retrieval, tutor-generation, speech, behavior, and tool-execution adapters; binding ADR-0052 directives to live task groups; non-cancellable-operation result reporting; timeouts, retries, fallback, repair, recovery, shutdown orchestration, persistence, networking, providers, secrets, authentication or authorization changes, destructive actions, and migrations. NEXA-ORCH-001 remains Baseline Draft.
