# Phase 2 embodiment traceability

| Governed requirement | Implementation | Conformance evidence | Status |
|---|---|---|---|
| NBP capability declaration/negotiation | `nexa_nbp::RuntimeCapabilities`, `AvatarCapability` | NBP and avatar contract tests | Implemented |
| Acceptance through terminal outcome | `nexa_avatar::AvatarReport` ordered lifecycle | fake-adapter and 3D conformance tests | Implemented |
| Origin message/behavior correlation | NBP ack/state/error payload fields | stable success output fixture | Implemented |
| Governed output envelopes | `AvatarReport::to_nbp_messages` | 3D end-to-end conformance test | Implemented |
| Typed avatar lifecycle facts | six payloads in `nexa-events` | 3D event assertions | Implemented |
| Semantic target resolution | `NexaAvatarAdapter` canvas target registry | unresolved-target degradation test | Implemented |
| Renderer-neutral upstream boundary | avatar port plus boundary script | `check-contract-boundaries.sh` | Implemented |
| Synchronous headless integration | `nexa_3d_runtime::integration::execute` | workspace/no-default-features tests | Implemented |

ADR-0009 resolves capability wire ownership and acknowledgement/completion ambiguity. Async transport scheduling, networking, persistence, speech/audio processing, and rendering features are deferred rather than inferred.
