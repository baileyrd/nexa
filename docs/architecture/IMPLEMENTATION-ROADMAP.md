# Nexa Implementation Roadmap

Status date: 2026-08-26

The pre-tactical-pause Phase 0–6 roadmap is preserved in Git history and phase traceability, but it no longer selects new implementation work.

The current delivery authority is:

- [`NEXA-COMPLETION-ROADMAP.md`](NEXA-COMPLETION-ROADMAP.md)
- [`NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md`](NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md)
- [`NEXA-R1-IMPLEMENTATION-BASELINE.md`](NEXA-R1-IMPLEMENTATION-BASELINE.md)
- [`../adr/0068-v1-r2-walking-skeleton-baseline.md`](../adr/0068-v1-r2-walking-skeleton-baseline.md)

## Why the roadmap changed

The tactical-pause assessment found that qualified deterministic/headless contract gates had become program-progress signals while parent architecture/specification maturity and vertical product integration lagged. The new roadmap is organized around capability maturity and a finite release outcome.

Historical Phase 1–5 work remains valid evidence and reusable implementation. It is not discarded; it is reclassified according to what it actually proves.

## Current stage summary

| Stage | Purpose | Status |
|---|---|---|
| R0 | Rebaseline governance and architecture | PASS for R2, pending PR #110 merge |
| R1 | Critical specification/technology baseline | PASS for R2, pending PR #110 merge |
| R2 | Thin real production walking skeleton | Next implementation stage |
| R3 | Robust complete session orchestration | Not started |
| R4 | Grounding/tutor quality gate | Not started |
| R5 | Speech/avatar integration decision and optional capability hardening | Not started |
| R6 | Labs/tools only if promoted by release scope | Not started |
| R7 | Measured performance/operational hardening | Not started |
| R8 | Packaging/install/update/release engineering | Not started |
| R9 | System verification, user acceptance, release | Not started |

## R2 objective

Create the first real end-to-end Nexa learner path using concrete dependencies:

```text
learner text
 -> apps/nexa-desktop
 -> orchestrator
 -> SQLite durable state
 -> learning/pedagogy
 -> governed knowledge
 -> local llama.cpp model adapter
 -> admitted tutor response
 -> assessment/practice
 -> atomic evidence/mastery/progress commit
 -> restart/resume
```

R2 uses the bounded TCP Connection Establishment course as its first acceptance package.

## R2 exit gate

A learner can complete one primitive real lesson through one composition root using:

- actual learner desktop UI boundary;
- actual SQLite persistence;
- actual governed course/knowledge data;
- actual local model server adapter;
- existing domain/tutor/knowledge contracts where appropriate;
- atomic durable progress;
- restart/resume;
- content-safe correlated diagnostics.

Scripted provider outcomes, in-memory persistence, or headless contract tests alone do not close R2.

## Work-selection rule

Every implementation increment must identify:

1. the R2/R3 release blocker it addresses;
2. its governing parent architecture/specification/ADR;
3. the E2E step it makes more concrete;
4. the capability maturity transition it demonstrates;
5. the evidence required for that maturity transition.

Do not select work merely because another narrow ADR-sized contract can be added.

## Deferred from the R2 critical path

Unless a concrete R2 blocker proves otherwise:

- speech input/output;
- animated avatar embodiment;
- labs/tool execution;
- dynamic multi-provider routing/fallback;
- remote-provider privacy/credential support;
- dedicated vector database;
- durable event broker/outbox;
- final installer/signing/update mechanism;
- plugins/public API/analytics/authoring/server deployment.

These capabilities remain part of Nexa’s broader architecture where applicable, but are owned by later roadmap gates.

## Cross-stage quality gates

Every increment still requires proportionate:

- formatting/build/lint/test checks;
- contract/dependency-boundary verification;
- architecture/specification traceability;
- security/privacy review for the changed boundary;
- deterministic evidence where the behavior is deterministic;
- concrete-adapter/system evidence when claiming higher maturity;
- documentation/status updates that do not overstate capability maturity.

## Architecture rebaseline checkpoints

At R2 exit, before R3 broadening, and at each later major stage boundary, the Chief Systems Architect must independently evaluate the whole program and record Continue, Redirect, or Tactical Pause.

See [`../governance/ARCHITECTURE-REBASELINE-GATES.md`](../governance/ARCHITECTURE-REBASELINE-GATES.md).

---

## 2026-08-26 owner-authority reconciliation (controlling addendum)

ADR-0069 records the explicit owner decisions from Issue #114. This addendum supersedes earlier text in this document only where it conflicts. Earlier `eframe`/`egui`, `llama.cpp`, text-first release, desktop-only, speech/avatar deferral, owner-delegation, or general R2-Continue/readiness language is preserved as historical evidence and is not active selection authority.

Status: Active delivery control; reconciled 2026-08-26

### Finite dependency-ordered route

| Gate | Increment | Exit evidence | Failure consequence |
|---|---|---|---|
| G0 | Authority reconciliation (this increment) | ADR-0069 and all active parents/supplements/status/deferrals agree; links, stale-claim scan, scope check pass | Tactical Pause; correct authority only |
| G1 | Shared UI/loopback suitability spike | Same compiled React/TS/Vite candidate frontend in browser and Tauri 2 candidate shell; versioned HTTP request + WebSocket event; loopback bind/origin/auth; cancellation/reconnect/errors; keyboard/accessibility; Windows build; parity; startup/CPU/memory/package measurements | Reject disproved candidate/boundary; update ADR/architecture through owner route before production work |
| G2 | Bundled speech suitability spike | Sherpa-ONNX measurements for recognition accuracy, latency, synthesis quality, memory, package size, interruption/cancellation, lip-sync on CPU-only Windows reference PC | Reject Sherpa-ONNX; evaluate `whisper.cpp` for recognition only and govern TTS separately |
| G3 | Animated 2D suitability spike | Rive proves identical browser/desktop idle/listening/thinking/speaking/error states, interruption, lip-sync, accessible fallback, acceptable CPU | Reject Rive and govern a replacement before integration |
| G4 | Persistent text lesson | Production shared boundary, SQLite migrations/atomicity/recovery, governed TCP lesson, assessment, restart/resume in both clients | Fix owning UI/data/content authority; do not advance |
| G5 | LM Studio integration | Narrow adapter against documented separately installed reference server; admission, cancellation/error, compatibility and E2E text evidence | Reconcile provider adapter/compatibility authority |
| G6 | Speech integration | Bundled selected speech adapter/device lifecycle, accessible text equivalent, interruption/lip-sync, package evidence | Return to speech selection/integration gate |
| G7 | Animated tutor integration | Selected 2D runtime consumes admitted semantic states with synchronized speech and graceful fallback in both clients | Return to avatar selection/integration gate |
| G8 | Complete-system verification | Full E2E/failure/security/privacy/performance/accessibility evidence on Windows reference PC and both clients | Correct owning gate; no maturity overstatement |
| G9 | Packaging and user acceptance | Clean install/update/uninstall, package contents/licensing, representative acceptance; exact release head green | No Release Ready claim |

### Work selection

G1 is the precise first permitted follow-on and must be separately dispatched. Spike branches are disposable evidence, independently reviewable, and cannot silently become production architecture. No later gate starts merely because research looks favorable. A recorded authority update accepts or rejects each candidate.

The final outcome is completion of the governed TCP lesson from either identical client using LM Studio, durable restart/resume, bundled speech, synchronized 2D tutor behavior, and system/user/package evidence. Same-machine browser evidence is never labeled hosted-web or remote evidence.

### Deferrals and control

Labs/tools, LAN/Internet-remote access, hosted deployment, cloud sync, broad providers, dynamic routing/fallback, and 3D release integration remain post-v1. Existing Phase 1–5 evidence is retained, not repeated as substitute release proof. The Chief Systems Architect maintains Tactical Pause outside the next eligible gate and records Continue, Redirect, or Tactical Pause at every gate.
