# ADR-0067: Provider-neutral asynchronous Speech input port foundation

- Status: Accepted
- Date: 2026-08-26
- Scope: First bounded Phase 5 Speech input contract

## Context

NEXA-SPCH-001 describes a broad real-time speech pipeline, while ADRs 0062–0064 establish only cancellation-control surfaces and their bounded application binding. The next approved increment needs one input operation without selecting microphone, audio, recognition, provider, network, or application integration policy. Input content is private and therefore must not enter general diagnostics.

## Decision

`nexa-domain` owns canonical non-nil `SpeechInputOperationId`. `nexa-speech` owns a strict V1 request associated with exactly one existing `SpeechId` and input-operation identity, bounded successful transcript evidence, exact cancellation evidence, a closed normalized failure vocabulary, an object-safe asynchronous service port using erased standard-library futures, and a caller/owner-supplied cooperative cancellation signal.

The host performs side-effect-free version and exact-association preflight before calling the dependency. Successful preflight permits exactly one service call and one FIFO scripted outcome consumption, except that already-requested cancellation returns exact cancellation evidence before consuming an outcome. Returned success or cancellation evidence is accepted only with the exact request identities and supported version; the host never reassociates dependency evidence.

Successful evidence contains a nonempty UTF-8 transcript bounded to 16 KiB. Serialization preserves that exact evidence for an explicitly authorized consumer, while `Debug`, `Display`, public errors, failures, and cancellation evidence never expose transcript text, audio, authorization material, or dependency reasons. This is structural content safety, not authentication, authorization, moderation, semantic transcript validation, privacy-policy correctness, retention policy, or permission to log or persist transcript content.

The deterministic FIFO adapter records exact requests and active futures. It supports success, both normalized failures, exhaustion, already-requested cancellation, and waiting for the single supplied signal. It creates no task, thread, or detached work; returning or dropping the caller future drops all adapter-owned operation work.

## Threat, privacy, and ownership consequences

Transcript evidence is intentionally accessible only through an explicit accessor and wire serialization because it is the result of this input contract; callers remain responsible for its private treatment. All ambient diagnostics are redacted. Identity evidence proves association only, not user identity, authenticity, freshness, consent, authorization, transcript truth, speaker identity, or safe retention. Cooperative cancellation proves only that this service future observed the signal and terminalized, not that external work stopped.

No microphone/device or OS audio API, VAD, wake word, codec, buffering, streaming transport, networking, STT provider/model inference, transcript normalization or semantic validation, moderation, authentication, authorization policy, retention, persistence, telemetry, synthesis, playback, queued audio, viseme, gesture, Behavior change, retry, timeout, interruption, recovery, or concrete external-work stop is added. `apps/nexa-headless` remains unchanged. The existing cancellation contracts remain unchanged.

Phase 5 and the combined speech input/output roadmap item remain incomplete: this ADR establishes only the input port foundation and no output port or end-to-end binding.
