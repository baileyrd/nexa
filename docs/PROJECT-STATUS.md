# Nexa Project Status

Status date: 2026-08-30
Verified base `main` checkpoint: `b49b15081f5afcb397b09186036d9ab6636f4a76` (PR #112 merge head)
Authority checkpoint: Issue #114 owner decisions recorded by ADR-0069; this reconciliation PR is not recorded as merged

This document is the concise current-state authority. Git history, accepted ADRs, and traceability preserve earlier checkpoints and evidence.

## Current program state

**Architecture outcome: Tactical Pause — bounded evidence work only after reconciliation merges.**

Issue #114 supplied the explicit owner review previously missing. [ADR-0069](adr/0069-owner-approved-v1-delivery-baseline.md) records those decisions and supersedes ADR-0068 only where they conflict. General product implementation remains paused. This documentation correction neither begins G1 nor changes implementation maturity.

## Governing route

Read [`../CHATGPT_WORKFLOW.md`](../CHATGPT_WORKFLOW.md), [`../AGENTS.md`](../AGENTS.md), this file, [`BASELINE.md`](BASELINE.md), [`SPECIFICATION-REGISTRY.md`](SPECIFICATION-REGISTRY.md), [NEXA-ARCH-002](architecture/NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md), [NEXA-R1](architecture/NEXA-R1-IMPLEMENTATION-BASELINE.md), ADR-0069, applicable supplements, and the [completion roadmap](architecture/NEXA-COMPLETION-ROADMAP.md). ADR-0068 is preserved historical evidence and applies only where ADR-0069 does not conflict.

## Owner-approved v1 outcome

One local learner uses identical Windows desktop and same-machine browser clients against one local Rust runtime and authoritative SQLite state. The release path completes Networking Fundamentals / TCP Connection Establishment through the separately installed graphical LM Studio reference server, bundled CPU-capable speech, and a synchronized animated 2D tutor.

The shared frontend candidate is React/TypeScript/Vite packaged by a Tauri 2 candidate shell over one versioned loopback HTTP/WebSocket business API. Sherpa-ONNX and Rive remain evidence-gated candidates. Nexa bundles no LLM weights or inference runtime.

LAN/Internet-remote access, hosted deployment, cloud sync, accounts/multi-user administration, labs/tools, broad providers, dynamic routing/fallback, dedicated vector infrastructure unless demonstrated necessary, durable event brokerage, and 3D release integration remain deferred.

## Current work-selection and resume gate

G0 is this authority reconciliation. It must be reviewed, green, and merged before any product increment resumes. It does not itself dispatch a spike.

After G0 merges, the first and only selectable follow-on is a separately dispatched G1 shared UI/loopback suitability spike. Its evidence must cover identical browser/desktop behavior, the versioned HTTP/WebSocket boundary, loopback security, cancellation/reconnect, accessibility, Windows build, and resource/package measurements. Candidate success or failure requires a recorded authority update; spike code cannot silently become production architecture. G2 speech and G3 avatar spikes follow only through the roadmap gates.

The Chief Systems Architect call is **Tactical Pause** outside the next eligible gate. The superseded open-ended Phase 5 sequence must not resume.

## Capability maturity and preserved evidence

Use `Concept -> Architecture Defined -> Specification Approved -> Contract Implemented -> Runtime Integrated -> Concrete Adapter Implemented -> System Verified -> User Accepted -> Release Ready`.

Existing shared domain/event/NBP contracts, deterministic learning/tutor policies, orchestrator runtime/cancellation work, speech cancellation foundations, avatar semantics, 3D foundations, labs/tool contracts, and headless integration evidence retain only their demonstrated maturity. Scripted providers and in-memory persistence remain test tools. No shared release UI, versioned loopback production boundary, SQLite production path, LM Studio adapter, bundled speech adapter, synchronized 2D release renderer, complete system, user acceptance, or release package has been proven.

ADR-0069 and this reconciliation establish selected architecture/specification only. Every later increment identifies its blocker, governing authority, E2E step, maturity transition, and required evidence.

## G1 evidence dispatch (2026-08-27)

Issue #116 dispatched the authorized disposable shared UI/loopback suitability spike. Its evidence record is [`../spikes/g1-shared-ui/evidence/G1-EVIDENCE.md`](../spikes/g1-shared-ui/evidence/G1-EVIDENCE.md). Linux/headless automation covers frontend lint/typecheck, lifecycle component tests, production build, and end-to-end loopback HTTP/WebSocket security and cancellation tests. The first representative Windows harness run on merged `main` at `38bb0e71cda177c7ae9dc0afb81118ba4ee744c4` passed the frontend/runtime/desktop dependency steps and failed only when Tauri packaging found its default Windows icon absent. Issue #118 supplies an evidence-only repository icon; Windows evidence remains incomplete until the owner reruns the harness after that correction merges. G1 remains **blocked** pending the successful rerun and reviewed browser/WebView parity, accessibility, startup/CPU/memory/package, reconnect, and cancellation evidence. Candidate maturity and authority are unchanged, Tactical Pause remains in force, and G2 and later gates remain undispatched pending a separate authority decision.

Issue #120 adds only an opt-in, bounded hold mode to make the remaining interactive cancellation observation practical while preserving the normal fast fixture path. Automated coverage does not replace the owner's representative Windows execution and review. G1 remains blocked, candidate maturity is unchanged, and Tactical Pause and all G2+ exclusions remain in force.

Issue #122 records the now-complete evidence set from exact `main` head `9173ed152d7b1d5bb9831413862097076443be59`. The Windows harness passed frontend, runtime, desktop dependency, Tauri release-build, and NSIS-package steps and produced a 1,848,941-byte installer. Separately, owner observation covered browser/Tauri parity, normal and held/cancelled flows, runtime disconnect/reconnect, keyboard/focus basics, approximate Task Manager idle/active CPU and memory, and human-observed startup/cancellation/reconnect thresholds of <0.5 seconds. The [G1 evidence record](../spikes/g1-shared-ui/evidence/G1-EVIDENCE.md) preserves the measurement precision and limitations and recommends that the G1 evidence gate be recorded as satisfied.

This is an evidence recommendation, not an architecture disposition. React/TypeScript/Vite, Tauri 2, and the fixture API remain candidates; maturity is unchanged. The Chief Systems Architect must separately select or reject the candidates and decide Continue, Redirect, or Tactical Pause. Until then, G2 and all later gates remain undispatched and Tactical Pause remains in force outside this evidence-recording increment.
