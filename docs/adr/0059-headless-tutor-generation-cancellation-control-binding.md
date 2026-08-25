# ADR-0059: Headless Tutor Generation cancellation-control binding

- Status: Accepted
- Date: 2026-08-25
- Scope: Second concrete Phase 5 subsystem cancellation-control binding

## Context

ADR-0058 established a provider-neutral Tutor Generation cancellation-control port while deliberately leaving runtime composition unresolved. The repository owner approved the second step: a bounded `apps/nexa-headless` binding through ADR-0056's unchanged exact-plan runtime. The synchronous ADR-0022 `LanguageModelProvider::generate` contract remains separate and cancellation-unaware.

## Decision

`apps/nexa-headless` owns a separate Tutor Generation cancellation composition rather than merging it with the Behavior API. Construction preflights the already-cancelled workflow and its exact four identities, the supported V1 request, and its exact invocation/provider/model association before planning, task spawn, port recording, or outcome consumption. The composition obtains one canonical `TutorGeneration / Cancellable / RequestCancellation` plan from the existing planner.

The supplied port is private behind a synchronization boundary. The composition terminalizes before spawning exactly one target-aware Tutor Generation task. That task waits for its private token and then invokes unchanged `request_tutor_generation_cancellation` exactly once with the approved request. Exact-plan execution must join the required task, and the acknowledgement must preserve the V1 invocation/provider/model tuple. Immutable success evidence contains the unchanged runtime execution, exact request, and exact acknowledgement. Identical success repeats are idempotent; conflicts, dependency or acknowledgement failures, runtime/join failures, and caller-future drop cannot restart or duplicate work. Application errors are closed and content-free.

`Stopped` proves only that the owned cancellation-control task completed and joined after submitting its request. The acknowledgement proves only that the dependency accepted that exact control request. Neither proves that `LanguageModelProvider::generate` or underlying provider work stopped, joined, or emitted no later output.

## Consequences and deferrals

Behavior and Tutor Generation control now have headless composition bindings. There is still no concrete provider cancellation or provider-generation stop evidence. Retrieval, speech, and tool/lab bindings remain absent. No orchestrator, runtime, Tutor contract, generation API, provider integration, networking, retry, timeout, persistence, or policy changes are introduced.

NEXA-TUTOR-001 and NEXA-ORCH-001 remain Baseline Draft. Phase 5 remains active, and the broad cancellation-safe execution and propagation checklist remains incomplete.
