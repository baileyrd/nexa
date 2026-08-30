# Nexa Project Status

Status date: 2026-08-30
Verified base `main` checkpoint: `9173ed152d7b1d5bb9831413862097076443be59` (PR #121 merge head)
Authority checkpoint: Issue #114 owner decisions recorded by ADR-0069; the G0 reconciliation is merged

This document is the concise current-state authority. Git history, accepted ADRs, and traceability preserve earlier checkpoints and evidence.

## Current program state

**Architecture outcome: Tactical Pause — G1 evidence gathering is recommended satisfied; candidate disposition is pending.**

Issue #114 supplied the explicit owner review previously missing. [ADR-0069](adr/0069-owner-approved-v1-delivery-baseline.md) records those decisions and supersedes ADR-0068 only where they conflict. G0 is merged, and Issue #122 records the completed G1 evidence set and recommends that evidence gathering be treated as satisfied. General product implementation remains paused: candidate disposition requires a separate Chief Systems Architect decision, no G2 work is dispatched, and implementation maturity is unchanged.

## Governing route

Read [`../CHATGPT_WORKFLOW.md`](../CHATGPT_WORKFLOW.md), [`../AGENTS.md`](../AGENTS.md), this file, [`BASELINE.md`](BASELINE.md), [`SPECIFICATION-REGISTRY.md`](SPECIFICATION-REGISTRY.md), [NEXA-ARCH-002](architecture/NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md), [NEXA-R1](architecture/NEXA-R1-IMPLEMENTATION-BASELINE.md), ADR-0069, applicable supplements, and the [completion roadmap](architecture/NEXA-COMPLETION-ROADMAP.md). ADR-0068 is preserved historical evidence and applies only where ADR-0069 does not conflict.

## Owner-approved v1 outcome

One local learner uses identical Windows desktop and same-machine browser clients against one local Rust runtime and authoritative SQLite state. The release path completes Networking Fundamentals / TCP Connection Establishment through the separately installed graphical LM Studio reference server, bundled CPU-capable speech, and a synchronized animated 2D tutor.

The shared frontend candidate is React/TypeScript/Vite packaged by a Tauri 2 candidate shell over one versioned loopback HTTP/WebSocket business API. Sherpa-ONNX and Rive remain evidence-gated candidates. Nexa bundles no LLM weights or inference runtime.

LAN/Internet-remote access, hosted deployment, cloud sync, accounts/multi-user administration, labs/tools, broad providers, dynamic routing/fallback, dedicated vector infrastructure unless demonstrated necessary, durable event brokerage, and 3D release integration remain deferred.

## Current work-selection and resume gate

G0 is merged. Issue #122 has completed the separately dispatched G1 shared UI/loopback evidence gathering and recommends that its evidence gate be treated as satisfied. That recommendation does not select the candidates or dispatch further work.

The current resume gate is a separate Chief Systems Architect disposition of the G1 candidates and an explicit Continue, Redirect, or Tactical Pause decision. Until that decision is recorded, React/TypeScript/Vite, Tauri 2, and the fixture API remain candidates, no G2 or later work is dispatched, and **Tactical Pause** remains in force. The superseded open-ended Phase 5 sequence must not resume.

## Capability maturity and preserved evidence

Use `Concept -> Architecture Defined -> Specification Approved -> Contract Implemented -> Runtime Integrated -> Concrete Adapter Implemented -> System Verified -> User Accepted -> Release Ready`.

Existing shared domain/event/NBP contracts, deterministic learning/tutor policies, orchestrator runtime/cancellation work, speech cancellation foundations, avatar semantics, 3D foundations, labs/tool contracts, and headless integration evidence retain only their demonstrated maturity. Scripted providers and in-memory persistence remain test tools. No shared release UI, versioned loopback production boundary, SQLite production path, LM Studio adapter, bundled speech adapter, synchronized 2D release renderer, complete system, user acceptance, or release package has been proven.

ADR-0069 and this reconciliation establish selected architecture/specification only. Every later increment identifies its blocker, governing authority, E2E step, maturity transition, and required evidence.

## G1 evidence chronology

On 2026-08-27, Issue #116 dispatched the authorized disposable shared UI/loopback suitability spike. Its evidence record is [`../spikes/g1-shared-ui/evidence/G1-EVIDENCE.md`](../spikes/g1-shared-ui/evidence/G1-EVIDENCE.md). Linux/headless automation covered frontend lint/typecheck, lifecycle component tests, production build, and end-to-end loopback HTTP/WebSocket security and cancellation tests. The first representative Windows harness run on merged `main` at `38bb0e71cda177c7ae9dc0afb81118ba4ee744c4` passed the frontend/runtime/desktop dependency steps and failed only when Tauri packaging found its default Windows icon absent. At that historical checkpoint, Issue #118 supplied an evidence-only repository icon, and G1 remained blocked pending a successful rerun and reviewed browser/WebView parity, accessibility, startup/CPU/memory/package, reconnect, and cancellation evidence.

Issue #120 then added only an opt-in, bounded hold mode to make the remaining interactive cancellation observation practical while preserving the normal fast fixture path. At that historical checkpoint, automated coverage did not replace the still-outstanding owner Windows execution and review. Candidate maturity remained unchanged, and Tactical Pause and all G2+ exclusions remained in force.

Issue #122 records the now-complete evidence set from exact `main` head `9173ed152d7b1d5bb9831413862097076443be59`. The Windows harness passed frontend, runtime, desktop dependency, Tauri release-build, and NSIS-package steps and produced a 1,848,941-byte installer. Separately, owner observation covered browser/Tauri parity, normal and held/cancelled flows, runtime disconnect/reconnect, keyboard/focus basics, approximate Task Manager idle/active CPU and memory, and human-observed startup/cancellation/reconnect thresholds of <0.5 seconds. The [G1 evidence record](../spikes/g1-shared-ui/evidence/G1-EVIDENCE.md) preserves the measurement precision and limitations and recommends that the G1 evidence gate be recorded as satisfied.

This is an evidence recommendation, not an architecture disposition. React/TypeScript/Vite, Tauri 2, and the fixture API remain candidates; maturity is unchanged. The Chief Systems Architect must separately select or reject the candidates and decide Continue, Redirect, or Tactical Pause. Until then, G2 and all later gates remain undispatched and Tactical Pause remains in force outside this evidence-recording increment.
