# ADR-0061: Headless cooperative Retrieval cancellation binding

- Status: Accepted
- Date: 2026-08-25
- Scope: Third concrete Phase 5 subsystem cancellation binding

## Context

ADR-0060 establishes an asynchronous cooperative Retrieval service contract but deliberately leaves composition unresolved. ADR-0056 already owns target-aware tasks and executes canonical cancellation plans. The repository owner approved the smallest separate headless binding without changing either contract or the synchronous knowledge engine.

## Decision

`apps/nexa-headless` owns a Retrieval cancellation composition separate from Behavior and Tutor Generation. Construction preflights an already-cancelled workflow, its exact workflow/session/correlation/trace identities, and one exact valid existing `RetrievalQuery` before planning, spawning, polling, recording, or consuming service work. It obtains exactly one canonical `Retrieval / Cancellable / RequestCancellation` directive from the unchanged planner.

The composition terminalizes before spawning one Retrieval-target task in the unchanged `WorkflowTaskGroup`. The task passes its private cancellation token directly to the unchanged ADR-0060 `retrieve` host boundary. Exact-plan execution cancels and joins that task. Success requires an exact associated `RetrievalServiceOutcome::Cancelled`; success results, dependency failures or exhaustion, association mismatch, missing outcomes, and runtime/join divergence fail closed. Immutable evidence contains the exact runtime execution and content-free query/result cancellation association. Read-only service inspection exposes no lock, token, or task handle. Identical successful repeats are idempotent; conflict, failure, and caller-future drop cannot restart service work.

## Consequences and deferrals

`Stopped` and cancellation evidence prove only that the owned service future observed cancellation, terminated, and joined. They do not prove an external provider, database, vector engine, or network request stopped. The synchronous `nexa-knowledge` engine and ADR-0060 contracts remain unchanged. No `spawn_blocking`, concrete dependency, networking, persistence, retry, timeout, recovery, telemetry, speech, or tool/lab integration is introduced.

Behavior and Tutor Generation contracts remain unchanged. Retrieval is the third bounded headless binding; Speech and Tool Execution remain absent. Phase 5 and the five-subsystem cancellation gate remain incomplete. NEXA-KNOW-001 and NEXA-ORCH-001 retain their registry status.
