# ADR-0058: Provider-neutral Tutor Generation cancellation control

- Status: Accepted
- Date: 2026-08-25
- Scope: Phase 5 Tutor Generation cancellation-control contract foundation

## Context

ADR-0057 leaves Tutor Generation unbound. The repository owner approved a two-step direction: first establish a dependency-light provider-neutral control contract owned by `nexa-tutor`, then separately review any `apps/nexa-headless` runtime binding. The existing ADR-0022 `LanguageModelProvider::generate` operation is synchronous and must not be wrapped or represented as cancellable.

## Decision

`nexa-tutor` owns a synchronous V1 cancellation-control request, acknowledgement, closed dependency error, closed public operation error, caller-supplied port, host operation, and deterministic FIFO scripted adapter. The request identifies exactly one existing non-nil `ModelInvocationId` and preserves its exact `ModelProviderId` and `ModelId`; it carries no prompt, output, learner content, token, task/runtime handle, endpoint, credential, or provider payload.

The host validates the supported version and exact caller association without side effects, calls the supplied port exactly once after successful preflight, and accepts only an acknowledgement with the same supported version and exact identity tuple. Dependency failure and script exhaustion normalize to one content-free public dependency failure. The scripted adapter exposes exact received requests and outcome consumption/remaining counts for deterministic tests.

An acknowledgement means only that the control request was accepted. It does not prove generation stopped, joined, or emitted no later output. `ModelErrorKind::Cancelled` remains outcome vocabulary only and does not prove this control operation ran.

## Compatibility and consequences

`LanguageModelProvider::generate`, `ModelRequest`, `ModelResponse`, and all existing generation and admission operations remain unchanged. This decision adds no Tokio, async task, thread, cancellation token, `spawn_blocking`, provider/network integration, inference, endpoint, credential, retry, persistence, or `apps/nexa-headless` composition.

Behavior remains the sole concrete subsystem binding. A Tutor Generation runtime binding is a separately reviewed possible next step. Retrieval, speech, and tool/lab bindings and all prior deferrals remain unchanged. NEXA-TUTOR-001 and NEXA-ORCH-001 remain Baseline Draft; Phase 5 and the broad cancellation-safe execution/propagation checklist remain incomplete.
