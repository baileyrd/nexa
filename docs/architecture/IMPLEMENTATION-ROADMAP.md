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
