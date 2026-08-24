# ADR-0051: Deterministic session/workflow lifecycle and cancellation

- Status: Accepted
- Date: 2026-08-24
- Scope: Narrow Phase 5 synchronous lifecycle-contract foundation

## Context

Phase 4 has met its deterministic headless contract exit gate. NEXA-ORCH-001 sections 4–14 and 57–61 route the next phase toward session and interaction-workflow coordination, but its complete runtime design remains Baseline Draft. The first increment needs stable identity, lifecycle, correlation, and cancellation semantics without selecting execution infrastructure or integrating subsystems.

## Decision

Add canonical non-nil `WorkflowId` to `nexa-domain` and activate `nexa-orchestrator` with only a normal dependency on `nexa-domain` plus serialization/error support. The crate is synchronous, deterministic, and provider-neutral.

`RuntimeSessionState` uses the nine NEXA-ORCH-001 states. Its pure operation permits exactly the lifecycle diagram's forward, pause/resume, degrade/recover, ending, completion, and failure edges. Every nonterminal state may fail; terminal states cannot transition.

`InteractionWorkflow` contains exactly workflow, session, correlation, and trace identities plus current state. ADR-0051 selects this explicit graph from the specification's ordered interaction example: normalization, context, pedagogy, retrieval, generation, optional tool execution, planning, optional speaking, waiting, and completion. The optional paths are generation directly to planning and planning directly to waiting. All other advancement fails closed. Any nonterminal state may cancel; cancellation is idempotent once cancelled; completed and failed workflows reject cancellation. A separate pure failure operation permits every nonterminal, non-cancelled workflow to fail.

The V1 wire adds an explicit `1.0` protocol field, rejects unknown fields, versions, variants, nil identities, and malformed values, and preserves exact identities. A pure aggregate operation compares all four identities with trusted expected references and fails closed with one association-mismatch category when any identity was reassociated. Errors are closed and content-free. No payload or replay evidence is necessary for this reference-only aggregate.

## Consequences and deferrals

This establishes lifecycle cancellation only, not a cancellation token or propagation. It adds no initialization policy, dependency health, clocks, state store, service handles, input/output payloads, side effects, complete composition root, subsystem integration, async runtime, thread/task spawning, structured-concurrency implementation, compensation, non-cancellable action policy, timeouts, retries, recovery, providers, networking, persistence, tools, speech, rendering, telemetry, or platform capability.

NEXA-ORCH-001 remains Baseline Draft. The next Phase 5 increment should be separately approved; cancellation-safe execution is a recommendation rather than a decision.
