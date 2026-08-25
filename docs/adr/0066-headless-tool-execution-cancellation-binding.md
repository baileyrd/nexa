# ADR-0066: Headless Tool Execution cancellation binding

- Status: Accepted
- Date: 2026-08-25
- Scope: Bounded Tool Execution application-control composition

## Context

ADR-0065 defines admitted Tool Execution cancellation contracts but deliberately leaves the
`CancellationTarget::ToolExecution` application binding absent. ADR-0056 already defines exact-plan
runtime behavior for cancellable and reported-non-cancellable targets. The final target needs a
composition without implying actual tool execution, sandbox enforcement, policy evaluation, or
external-work cancellation.

## Decision

`apps/nexa-headless` depends normally on `nexa-labs` and exports one Tool Execution cancellation
composition. Construction verifies an already-cancelled workflow's four trusted identities, passes
the complete request unchanged through ADR-0065 admission, verifies the exact V1 capability and full
association, maps its declared semantics, and constructs one canonical Tool Execution plan before
retaining the control dependency or creating runtime work.

For cancellable declarations, one target-aware owned task waits for its private target token and then
invokes unchanged ADR-0065 cancellation once. Success requires exact `Accepted` evidence and the
ADR-0056 `Stopped` outcome after join. For non-cancellable declarations, unchanged ADR-0065 produces
`DeclaredNonCancellable` without dependency invocation. One private bounded application-control
placeholder establishes live target ownership; its token is not cancelled, ADR-0056 reports exactly
one owned task, and a deterministic private release/completion handshake terminalizes it before
return. The placeholder does not represent or simulate tool execution.

The composition terminalizes before awaiting. Success is immutable and idempotent; conflicting
repeat, failure, and caller-future drop cannot consume more work. Dropping the future drops its local
task group, so no task or cancellation future detaches. Errors are closed and content-free. Evidence
contains only exact runtime execution, ADR-0065 cancellation evidence, association, and risk; digest
debugging remains redacted.

Structural admission does not establish authentic or fresh policy, real confirmation identity, or
sandbox enforcement. `Accepted` establishes only dependency acceptance and control-future
terminalization. `DeclaredNonCancellable` establishes only declared semantics. Neither proves a tool
ran or any process, command, provider, environment, hardware operation, database commit, destructive
action, external side effect, or Tool Execution stopped.

## Compatibility and consequences

ADR-0056 and ADR-0065 public semantics are unchanged. The dependency remains composition-root-only:
no contract crate depends on the headless application or Tokio. This completes the narrow five-target
application-control binding foundation, including explicit Tool Execution non-cancellable reporting,
but does not complete Phase 5 or the broad end-to-end lesson exit gate.

Actual Tool Execution, registries, terminals, processes, filesystem/network/database access,
containers/VMs, providers, sandbox provisioning or enforcement, authentication, identity proof,
policy evaluation or authenticity, confirmation UI, secrets, persistence, events, telemetry, timeout,
retry, recovery, and external-work cancellation remain deferred to separately approved increments.
