# ADR-0069 — Owner-approved v1 delivery baseline

Status: Accepted  
Date: 2026-08-26  
Decision owner: repository owner

## G1 disposition amendment — 2026-08-30

After reviewing the completed Issue #122 evidence set, the owner/Chief Systems Architect accepts the G1 evidence gate as satisfied and selects React + TypeScript + Vite as the shared v1 frontend, Tauri 2 as the Windows desktop shell, and one versioned same-machine loopback HTTP/WebSocket business boundary between the identical shared frontend and local Rust runtime. Tauri commands MUST NOT form a second business API. Same-machine browser/desktop parity, loopback-only access, and the G1 security, accessibility, and lifecycle constraints carry into production architecture.

The G1 fixture remains disposable evidence and is not promoted to production implementation. This selection does not establish Runtime Integrated, Concrete Adapter Implemented, System Verified, User Accepted, or Release Ready maturity.

The recorded program decision is **Continue to G2** for the bounded CPU-only speech evidence gate below. Sherpa-ONNX remains a candidate until that gate passes. Tactical Pause remains in force outside G2, and G3 and all later gates remain undispatched. This authority-recording amendment implements no G2 work.

## Context and authority

The tactical pause exposed that ADR-0068 and related authorities had been merged without the explicit owner review they claimed or implied. On 2026-08-26 the owner supplied fifteen explicit decisions in Issue #114. This ADR records those decisions without treating prior CI, merge history, or technical research as owner approval.

This ADR supersedes ADR-0068 **only where the decisions below conflict**. ADR-0068 remains historical evidence. Existing Phase 1–5 contracts, deterministic tests, headless compositions, and traceability retain only the maturity their evidence demonstrates; this documentation change implements no adapter and proves no system, user, or release maturity.

## Decision

1. **Clients.** Windows desktop and a same-machine browser client ship together in v1 with exact feature parity and one identical shared interface.
2. **Access.** The browser connects only to the Nexa runtime on the same Windows PC. LAN and Internet-remote access are deferred.
3. **UI boundary candidate.** React, TypeScript, and Vite are the evidence-gated shared-interface candidate, packaged with Tauri 2 for Windows. Both clients use the same compiled frontend. One versioned loopback HTTP/WebSocket boundary connects it to the Rust runtime; Tauri commands MUST NOT become a second business API.
4. **Runtime/deployment.** The Rust runtime is local-first. v1 supplies the local runtime and cloud-ready ports/contracts, not a working hosted deployment.
5. **Learner.** v1 supports one local learner, without login, accounts, multi-user administration, or cloud sync.
6. **Persistence.** SQLite remains behind `crates/nexa-storage`, preserving canonical IDs, migrations, atomic progress/evidence updates, backup/recovery, and restart/resume.
7. **Model delivery.** Nexa bundles neither model weights nor an inference runtime. It connects to one separately installed, graphically operated local model server.
8. **Reference model server.** LM Studio is the single validated v1 reference server through a narrow provider-neutral Rust adapter. Other providers remain architectural possibilities, not guaranteed v1 compatibility.
9. **Content.** Networking Fundamentals remains first; TCP Connection Establishment is the first complete lesson and acceptance vehicle.
10. **Speech.** Input and output are required in v1. Nexa bundles and manages their models/runtime, which must work on the CPU-only Windows reference PC behind provider-neutral speech boundaries.
11. **Speech candidate.** Sherpa-ONNX is not selected: a bounded spike must measure recognition accuracy, latency, synthesis quality, memory, package size, interruption/cancellation, and lip-sync timing. If disproved, `whisper.cpp` is the recognition fallback; TTS then requires a separate governed selection.
12. **Avatar.** v1 requires an animated 2D tutor with lip-sync and basic semantic idle, listening, thinking, speaking, and error behavior. Model output never selects renderer primitives.
13. **Avatar candidate.** Rive is not selected: a bounded spike must prove those states, interruption, lip-sync timing, accessibility fallback, identical browser/desktop rendering, and acceptable CPU use.
14. **Deferrals.** Labs/tools, LAN/remote access, hosted deployment, cloud sync, broad model-server support, 3D release integration, and dynamic provider routing/fallback are post-v1.
15. **Order.** Delivery proceeds through authority reconciliation; bounded UI, speech, and avatar spikes; persistent text lesson path; LM Studio integration; speech integration; animated tutor integration; then complete-system verification, packaging, and user acceptance.

## Evidence gates and failure consequences

The first permitted follow-on increment is the independently reviewable shared UI/loopback suitability spike. It must demonstrate one compiled frontend rendered identically in a normal browser and Tauri 2 shell; keyboard/accessibility basics; a versioned loopback HTTP request and WebSocket event; loopback-only binding and origin/authorization protection; cancellation/reconnect/error behavior; Windows build viability; and measured idle/interaction CPU, memory, startup, and package impact. Spike code is disposable evidence and does not become production architecture without a later authority update. Failure returns React/TypeScript/Vite, Tauri 2, or the boundary design to owner-governed selection before implementation proceeds.

The subsequent speech spike must produce the measurements in decision 11 on the CPU-only Windows reference PC. Failure removes Sherpa-ONNX as candidate, promotes only `whisper.cpp` recognition to the next evidence gate, and requires a separate TTS decision. The avatar spike must produce the evidence in decision 13 on both clients. Failure removes Rive as candidate and requires a separately governed renderer selection. Research alone cannot pass any gate.

## Maturity and program control

Authority reconciliation changes the selected product architecture from disputed to **Architecture Defined** and reconciles approved specifications; it does not advance concrete adapters. Product implementation remains under **Tactical Pause** except for the ordered bounded spikes. The Chief Systems Architect may issue **Continue** only for a spike whose gate is defined here and in the roadmap; general R2 readiness is not declared.

## Consequences

- The release E2E outcome is: launch either identical client, complete the governed TCP lesson with LM Studio, persist and resume progress, converse through bundled speech, receive synchronized 2D tutor behavior, and pass system, user, and packaging gates.
- Same-machine browser verification is distinct from hosted-web or remote-access evidence.
- Provider-neutral, cloud-ready, speech, avatar, and 3D foundations remain reusable without proving their concrete v1 candidates.
- No implementation is authorized by this documentation increment beyond the separately dispatched bounded spike.
