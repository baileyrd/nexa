# ADR-0057: Headless Behavior cancellation binding

- Status: Accepted
- Date: 2026-08-24
- Scope: First concrete Phase 5 subsystem cancellation binding

## Context

ADR-0056 executes one exact canonical cancellation plan against workflow-owned Tokio tasks, but deliberately leaves concrete subsystem adapters and composition wiring unresolved. The repository owner approved one bounded binding for renderer-neutral Behavior cancellation without changing the orchestrator, runtime, or avatar contracts.

## Decision

Concrete subsystem wiring belongs in an application composition root. `apps/nexa-headless` owns the first such root and binds only `CancellationTarget::Behavior` to the existing `nexa-avatar::AvatarPort`. Tokio cancellation tokens, task ownership, joining, and abort-on-drop remain exclusively in the unchanged `nexa-orchestrator-runtime::WorkflowTaskGroup`.

The composition performs side-effect-free workflow/four-identity, capability, and exact cancellation-preview validation before planning or spawning. It uses the existing planner to obtain the single canonical `Behavior / Cancellable / RequestCancellation` directive, registers one target-aware task, waits cooperatively for its private token, invokes the exact avatar cancellation once, and returns only after exact-plan execution joins that task. Returned immutable evidence preserves the runtime result, avatar report, request identity, and bounded cancellation-versus-submit counts. Identical repeats are idempotent; conflicts fail closed.

## Consequences and deferrals

This proves the first concrete Behavior cancellation binding only. It adds no Behavior submission or general response execution, renderer/UI integration, new wire format, or change to existing orchestrator/avatar contracts.

Retrieval, tutor generation, speech, and tool/lab bindings remain deferred, as do real renderer/provider/network cancellation, interruption policy, timeouts, retries, fallback, compensation, recovery, observability, persistence, secrets/authentication/authorization changes, destructive actions, and migrations. Phase 5 remains active and broad cancellation-safe execution/propagation remains incomplete.
