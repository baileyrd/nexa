# Nexa R1 Implementation Baseline

Status: Approved; reconciled 2026-08-26 by ADR-0069
Parent architecture: `NEXA-ARCH-002`
Technology decision: ADR-0069 (ADR-0068 retained where non-conflicting)

## Approved scope

The registry-routed R1 supplements govern domain/events, learning, tutor/knowledge, orchestration, data, security/privacy, observability, UX, testing, performance, packaging, and content subject to this reconciliation. Older conditional speech/avatar, desktop-only, native-UI, and generic-provider statements are superseded where they conflict.

## Resolved v1 requirements

- One local learner, runtime, and authoritative SQLite store behind `crates/nexa-storage`; canonical IDs, migrations, atomic evidence/progress, backup/recovery, and restart/resume are mandatory.
- Identical desktop and same-machine browser clients use one shared candidate frontend and one versioned loopback HTTP/WebSocket business API. Loopback binding, origin/authorization, protocol compatibility, cancellation, reconnection, keyboard access, and equivalent text access require evidence.
- LM Studio is the only validated reference model server. It is separately installed and graphical; Nexa bundles no LLM runtime/weights. The adapter remains narrow and provider-neutral inward.
- Networking Fundamentals / TCP Connection Establishment supplies the first governed package and acceptance lesson.
- Speech input/output is required and bundled/managed by Nexa on the CPU-only Windows reference PC. Sherpa-ONNX remains a candidate subject to measured accuracy, latency, quality, memory, package size, interruption/cancellation, and lip-sync timing.
- Animated 2D embodiment is required with admitted semantic states and lip-sync. Rive remains a candidate subject to parity, accessibility, interruption, timing, and CPU evidence.
- Observability is content-safe; security/privacy preserve authority separation and local-only network posture. Tutor/model output cannot select renderer primitives, endpoints, IDs, policy, or tools.
- Windows build, clean-machine package, performance, accessibility, failure recovery, and both-client acceptance evidence are required before higher maturity claims.

## Acceptance and maturity

The primary E2E scenario launches either client, resumes durable state, completes the governed TCP lesson through LM Studio, commits assessment/evidence atomically, uses bundled speech, renders synchronized 2D behavior, restarts without duplication/loss, and passes system/user/package gates. Scripted providers, in-memory persistence, Linux/headless evidence, research, or spike code cannot close that gate.

Current foundations range from Contract Implemented to bounded Runtime Integrated as recorded by traceability. Candidate adapters remain Architecture Defined/Specification Approved. No System Verified, User Accepted, or Release Ready claim follows.

## Delivery control

General implementation remains paused. The first separately dispatched increment is the shared UI/loopback suitability spike. Speech and avatar spikes follow independently; each has explicit failure/authority-update consequences in ADR-0069 and the roadmap. Labs/tools, remote/LAN access, hosted deployment, cloud sync, multiple providers, dynamic routing/fallback, and 3D release integration are not v1 gates.
