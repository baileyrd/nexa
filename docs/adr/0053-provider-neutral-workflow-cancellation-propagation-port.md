# ADR-0053: Provider-neutral workflow cancellation propagation port

- Status: Accepted
- Date: 2026-08-24
- Scope: Narrow Phase 5 synchronous propagation-port foundation

## Context

ADR-0051 defines lifecycle cancellation and ADR-0052 produces a complete, canonical `WorkflowCancellationPlan`, but neither provides a dependency boundary through which a host can hand off that plan. NEXA-ORCH-001 sections 57–61 remain Baseline Draft and do not authorize selecting runtime infrastructure or integrating a subsystem.

## Decision

`nexa-orchestrator` defines a synchronous `WorkflowCancellationPropagationPort` whose one operation accepts the whole existing validated plan. Before invoking it, `propagate_workflow_cancellation` validates the plan version, canonical directive set, and exact workflow, session, correlation, and trace association against trusted caller references. Host preflight failure consumes no dependency outcome; successful preflight makes exactly one port call.

The port returns `WorkflowCancellationAcknowledgement`, a strict content-free V1 value containing the exact four identities and canonical directives it accepted. Success means only that the port accepted this exact plan. The host validates version, identities, target order, directive values, cardinality, and exact directive equality before returning the acknowledgement. `ReportNonCancellable` remains an unchanged directive and is never translated into a cancellation request.

New wire input rejects unknown fields, unsupported versions, malformed values, nil identities, noncanonical order, duplicates, and collections above the five-target bound. Validation-critical fields remain private and construction is validated. Dependency failures and acknowledgement mismatches become closed content-free operation errors.

A deterministic scripted adapter supplies FIFO acknowledgement or dependency-failure outcomes, explicit content-free exhaustion, received-plan evidence, and exact outcome-consumption accounting. Every successfully preflighted call invokes the adapter once; an available scripted outcome is consumed whether it succeeds or fails.

## Consequences and deferrals

This decision adds a handoff contract and deterministic evidence only. It performs no cancellation and does not prove that a task, operation, or subsystem stopped. It adds no async runtime, thread, task, executor, cancellation token, channel, queue, scheduler, background work, structured concurrency, subsystem integration, multiple-operation identity, retry, fallback, repair, timeout, interruption, compensation, recovery, shutdown, health policy, networking, persistence, clock, telemetry, provider, secret, authorization change, destructive action, migration, or composition root.

ADR-0051 and ADR-0052 APIs and invariants remain unchanged. NEXA-ORCH-001 remains Baseline Draft, and the broader cancellation-safe execution and propagation roadmap item remains incomplete.
