# ADR-0063: Provider-neutral composite Speech cancellation coordinator

- Status: Accepted
- Date: 2026-08-25
- Scope: Phase 5 Speech-owned cancellation coordination foundation

## Context

ADR-0062 establishes one provider-neutral asynchronous cancellation-control operation, while NEXA-SPCH-001 sections 74–94 require cancellation across synthesis, buffered audio, playback, future viseme timing, and related behavior. One acknowledgement cannot establish that complete surface. The repository owner approved one composite Speech coordinator and retained speech-dependent gesture ownership in Behavior.

## Decision

`nexa-speech` defines the closed canonical surface order synthesis, queued audio, playback, and viseme timeline. Each participant exposes one strict V1, content-free, side-effect-free capability declaration containing its exact `SpeechId`, surface, and cancellability. The coordinator accepts exactly one participant for every surface. It completely inspects and validates versions, association, uniqueness, coverage, and cancellability before requesting cancellation or creating any participant future.

After preflight, the coordinator invokes each exact participant once with the same ADR-0062 request and owns all four returned standard-library futures beneath the caller-owned coordinator future. It uses no runtime, task, thread, `async_trait`, or detached work. Success requires every future to terminalize with an exact associated acknowledgement. If execution fails, every remaining owned future is safely dropped before return; dropping the coordinator future likewise drops every active participant future.

Immutable per-surface evidence retains the exact supported version, `SpeechId`, canonical identity, and acknowledgement. Aggregate `Stopped` evidence is canonically ordered and valid only after all four exact acknowledgements and terminal futures. It proves only that the four speech-owned control dependencies accepted cancellation and their coordinator futures terminalized. It does not prove that any provider, device, process, or external request stopped and does not cover speech-dependent gestures.

Closed content-free errors cover unsupported versions, invalid sets, missing or duplicate or non-cancellable surfaces, association mismatch, dependency failure, acknowledgement mismatch, and aggregate failure. Deterministic per-surface scripts expose FIFO outcome consumption, received requests, active futures, exhaustion, failure, and pending/drop accounting.

## Compatibility and consequences

ADR-0062's single-service operation and acknowledgement meaning remain unchanged. No orchestrator vocabulary or runtime behavior changes. `CancellationTarget::Speech` remains unbound, and `apps/nexa-headless` is unchanged. Complete speech-interaction stop evidence would additionally require correlated Behavior cancellation evidence in a later approved binding.

This adds no speech/audio/provider/device implementation, Tokio, networking, storage, telemetry, retry, timeout, security, authentication, secrets, migrations, Tool Execution, or persistence. NEXA-SPCH-001 and NEXA-ORCH-001 remain Baseline Draft; Phase 5 and the five-subsystem gate remain incomplete.
