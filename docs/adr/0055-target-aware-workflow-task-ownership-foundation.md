# ADR-0055: Target-aware workflow task ownership foundation

- Status: Accepted
- Date: 2026-08-24
- Scope: Narrow Phase 5 target-association foundation

## Context

ADR-0054 establishes one Tokio task owner, one private workflow-root cancellation token, and one private join set for each workflow. ADRs 0052 and 0053 separately define a closed five-target cancellation vocabulary and a synchronous propagation handoff, but neither is bound to runtime work. The next bounded prerequisite must identify the target owning a task without claiming selective cancellation or live propagation.

## Decision

`nexa_orchestrator::CancellationTarget` remains the sole closed target vocabulary: retrieval, tutor generation, speech, behavior, and tool execution. `WorkflowTaskGroup` retains exactly one private root `CancellationToken` and one private `JoinSet` per workflow. It adds a target-aware spawn operation that accepts one existing target, returns exactly `Result<(), WorkflowTaskGroupError>`, and exposes no handle.

Each target has a private hierarchical token beneath the workflow root. Each target-aware task receives a private child token beneath its target token and records exactly that target association in the single owned join set. The root, target, and task tokens remain private, and no `JoinHandle`, `AbortHandle`, mutable ownership collection, task output, or panic text escapes. Bounded inspection reports only the current owned-task count for a supplied closed target.

The existing unclassified `spawn` operation and its workflow-root child-token behavior remain unchanged; unclassified tasks are not reinterpreted as subsystem work. Both spawn paths reject work once cancellation or draining begins using the existing closed error. Workflow-wide `cancel_and_wait` remains authoritative: root cancellation reaches classified and unclassified work and completion joins all of it. Natural drain, normalized content-free join failure, completion evidence, repeat completion behavior, and abort-on-drop ownership remain unchanged.

Target association alone neither authorizes selective target cancellation nor proves execution of an ADR-0052 directive or ADR-0053 runtime-port binding. No runtime or wire format is added for this private ownership state.

## Compatibility

ADRs 0051 through 0054 and all their APIs and semantics are preserved, including ADR-0054's existing workflow-wide `spawn`. The broader cancellation-safe execution and propagation roadmap item remains incomplete.

## Consequences and deferrals

The runtime can now prove how many currently owned tasks are associated with each closed target while preserving one workflow owner and one join path. This is a foundation, not full cancellation propagation.

Explicitly deferred are execution of ADR-0052 directives; ADR-0053 runtime-port binding; selective target cancellation; concrete retrieval, tutor, speech, behavior, or tool adapters; non-cancellable-operation result reporting; timeouts, retry, fallback, repair, recovery, and shutdown orchestration; persistence, networking, providers, secrets, authentication or authorization changes, destructive actions, and migrations. NEXA-ORCH-001 remains Baseline Draft.
