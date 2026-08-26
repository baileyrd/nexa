# Nexa Project Status

Status date: 2026-08-26
Verified base `main` checkpoint: `b49b15081f5afcb397b09186036d9ab6636f4a76` (PR #112 merge head)
Authority checkpoint: Issue #114 owner decisions recorded by ADR-0069; this reconciliation PR is not recorded as merged

This document is the concise current-state authority. Git history, accepted ADRs, and traceability preserve earlier checkpoints and evidence.

## Current program state

**Architecture outcome: Tactical Pause — bounded evidence work only after reconciliation merges.**

Issue #114 supplied the explicit owner review previously missing. [ADR-0069](adr/0069-owner-approved-v1-delivery-baseline.md) records those decisions and supersedes ADR-0068 only where they conflict. General product implementation remains paused. This documentation correction neither begins G1 nor changes implementation maturity.

Existing contract, deterministic, runtime, cancellation, speech, and tool evidence remains at its demonstrated maturity. No shared UI/loopback production boundary, SQLite production path, LM Studio adapter, bundled speech adapter, synchronized 2D release renderer, complete system, user acceptance, or release package has been proven.

## Governing route

Read [`../CHATGPT_WORKFLOW.md`](../CHATGPT_WORKFLOW.md), [`../AGENTS.md`](../AGENTS.md), this file, [`BASELINE.md`](BASELINE.md), [`SPECIFICATION-REGISTRY.md`](SPECIFICATION-REGISTRY.md), [NEXA-ARCH-002](architecture/NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md), [NEXA-R1](architecture/NEXA-R1-IMPLEMENTATION-BASELINE.md), ADR-0069, and the applicable supplement and [roadmap](architecture/NEXA-COMPLETION-ROADMAP.md). ADR-0068 is preserved historical evidence and applies only where ADR-0069 does not conflict.

## Owner-approved v1 outcome

One local learner uses identical Windows desktop and same-machine browser clients against one local Rust runtime and authoritative SQLite state. The release path completes Networking Fundamentals / TCP Connection Establishment through the separately installed graphical LM Studio reference server, bundled CPU-capable speech, and a synchronized animated 2D tutor.

The shared frontend candidate is React/TypeScript/Vite packaged by a Tauri 2 candidate shell over one versioned loopback HTTP/WebSocket business API. Sherpa-ONNX and Rive remain evidence-gated candidates. Nexa bundles no LLM weights or inference runtime.

LAN/Internet-remote access, hosted deployment, cloud sync, accounts/multi-user administration, labs/tools, broad providers, dynamic routing/fallback, dedicated vector infrastructure unless demonstrated necessary, durable event brokerage, and 3D release integration remain deferred.

## Current work-selection and resume gate

G0 is this authority reconciliation. It must be reviewed, green, and merged before any product increment resumes. It does not itself dispatch a spike.

After G0 merges, the first and only selectable follow-on is a separately dispatched G1 shared UI/loopback suitability spike. Its evidence must cover identical browser/desktop behavior, the versioned HTTP/WebSocket boundary, loopback security, cancellation/reconnect, accessibility, Windows build, and resource/package measurements. Candidate success or failure requires a recorded authority update; spike code cannot silently become production architecture. G2 speech and G3 avatar spikes follow only through the roadmap gates.

The Chief Systems Architect call is **Tactical Pause** outside the next eligible gate. The superseded open-ended Phase 5 sequence must not resume.

## Capability maturity

Use:

`Concept -> Architecture Defined -> Specification Approved -> Contract Implemented -> Runtime Integrated -> Concrete Adapter Implemented -> System Verified -> User Accepted -> Release Ready`

ADR-0069 and this reconciliation establish selected architecture/specification only. Existing slices retain their evidenced Contract Implemented or bounded Runtime Integrated state. Do not infer Concrete Adapter Implemented, System Verified, User Accepted, or Release Ready.

Every later increment identifies its blocker, governing authority, E2E step, maturity transition, and required evidence.
