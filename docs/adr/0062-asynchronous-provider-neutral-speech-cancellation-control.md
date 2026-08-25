# ADR-0062: Asynchronous provider-neutral Speech cancellation control

- Status: Accepted
- Date: 2026-08-25
- Scope: Phase 5 Speech cancellation-control contract foundation

## Context

ADR-0061 leaves Speech unbound. NEXA-SPCH-001 requires cancellation to cover a broader surface than any single provider operation, including synthesis, queued audio, playback, future viseme timing, and speech-dependent behavior. The repository owner therefore approved a dependency-light asynchronous control contract before any application binding. A narrow acknowledgement must not be misrepresented as subsystem-level stop evidence.

## Decision

`nexa-domain` owns canonical non-nil `SpeechId`. `nexa-speech` owns a strict V1, content-free request and immutable acknowledgement associated with exactly one `SpeechId`; an object-safe asynchronous service returning an erased standard-library future; closed service outcomes and host errors; one full-preflight host operation; and a deterministic FIFO scripted service with request, outcome, and active-future accounting.

The host validates version and exact caller-supplied association before invoking the service, invokes it exactly once after successful preflight, awaits it directly, and accepts only the exact version and `SpeechId` acknowledgement. Dependency failure and exhausted scripts fail closed as dependency failure. A mismatched acknowledgement fails closed and is not acceptance evidence. The service and host create no detached work, and dropping a pending host future drops the service future.

Acknowledgement means only that the supplied dependency accepted the cancellation-control request. It does not prove TTS generation, queued audio, playback, visemes, speech-dependent gestures, a provider, device, thread, process, or external request stopped.

## Compatibility and consequences

This decision converts the reserved Speech directory into a dependency-light contract crate. It adds no Tokio, `async_trait`, task, thread, cancellation token, `spawn_blocking`, STT/TTS implementation, audio or viseme model, device API, provider, network, storage, persistence, retry, timeout, telemetry, endpoint, credential, or application composition.

`CancellationTarget::Speech` remains unbound in `apps/nexa-headless`. A later binding must separately account for the complete NEXA-SPCH-001 cancellation surface before reporting subsystem-level cancellation. Phase 5, cancellation-safe execution and propagation, Speech binding, Tool Execution binding, and the five-subsystem gate remain incomplete. NEXA-SPCH-001 and NEXA-ORCH-001 remain Baseline Draft.
