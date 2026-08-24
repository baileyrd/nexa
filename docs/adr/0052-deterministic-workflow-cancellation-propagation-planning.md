# ADR-0052: Deterministic workflow cancellation propagation planning

- Status: Accepted
- Date: 2026-08-24
- Scope: Narrow Phase 5 synchronous cancellation-propagation planning foundation

## Context

ADR-0051 establishes lifecycle cancellation but deliberately does not propagate it. NEXA-ORCH-001 sections 57–61 name retrieval, tutor generation, speech, behavior, and tool execution as workflow-owned work that may need cancellation, while acknowledging that some actions cannot safely be cancelled. A bounded contract is needed before any runtime or subsystem integration is selected.

## Decision

`nexa-orchestrator` provides a pure `plan_workflow_cancellation` operation. It accepts an existing cancelled `InteractionWorkflow`, trusted workflow/session/correlation/trace references, and a bounded collection of active targets. The closed target vocabulary is retrieval, tutor generation, speech, behavior, and tool execution. Each target declares either cancellable or non-cancellable semantics.

The operation fails closed for a non-cancelled workflow, any identity reassociation, unsupported V1 input, or duplicate category. It permits no targets. Successful output preserves all four identities and contains exactly one directive per supplied target: request cancellation or report non-cancellable. Directives are sorted in the target vocabulary's declared order, independent of caller order.

All new forms are strict, versioned V1 wire contracts. Unknown fields, versions, variants, malformed values, and nil identities are rejected. Errors are closed and content-free. Planning performs no I/O or side effect and grants no authority to execute a directive.

## Consequences and deferrals

This establishes deterministic propagation planning only. It does not select an async runtime, cancellation token, thread, task, executor, queue, priority, timeout, retry, interruption, fallback, compensation, recovery, shutdown, or health policy. It performs no cancellation and adds no subsystem adapter, multiple-operation identity, composition root, networking, persistence, clock, telemetry, provider, secret, authentication, authorization-policy change, destructive action, or migration.

NEXA-ORCH-001 remains Baseline Draft. The broader roadmap item for cancellation-safe execution and propagation remains incomplete until runtime behavior is separately decided and proven.
