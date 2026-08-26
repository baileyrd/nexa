# Nexa Completion Roadmap

Status: Approved delivery control
Date: 2026-08-26

## Purpose

This roadmap is the dependency-driven path from the tactical-pause baseline to Nexa v1 Release Ready. It replaces open-ended selection of the next narrow technical/ADR increment with product-maturity gates tied to the v1 acceptance statement.

Existing Phase 1–5 implementation is retained as reusable foundation evidence. New work is selected only when it advances a current release-path blocker.

## Governing rule

Every implementation increment must trace to:

1. a v1-required capability or explicit release risk;
2. a governing parent architecture/specification/ADR;
3. one or more concrete E2E steps;
4. a capability maturity transition;
5. evidence sufficient for that maturity claim.

No implementation enters the queue merely because another narrow contract/ADR can be added.

## R0 — Governance and architecture rebaseline

Status: **PASS for R2 after the rebaseline PR is green and merged.**

Completed:

- current-state/documentation-gap assessment;
- divergence analysis and lessons learned;
- architecture rebaseline/program-integrity gates;
- finite v1 product definition;
- approved `NEXA-ARCH-002` v1 release architecture;
- consolidated deferral review;
- current registry/status/README/roadmap authority reconciliation;
- R1 specification plan and implementation baseline.

Exit: parent architecture and v1 release boundary are sufficiently authoritative to govern R2.

## R1 — Critical specification and technology baseline

Status: **PASS for R2 after the rebaseline PR is green and merged.**

Approved R2-governing areas:

- data/persistence;
- security;
- privacy;
- observability;
- complete session orchestration;
- domain/events;
- learning subsystems;
- tutor/knowledge;
- learner UX;
- testing/system acceptance;
- performance measurement;
- packaging/deployment constraints;
- governed first-course content.

ADR-0068 resolves the R2 technology/scope decisions.

R1 intentionally does not finish all post-v1 specifications. The exit criterion is enough mature authority to implement the R2 walking skeleton safely.

## R2 — Thin production walking skeleton

Status: **Next implementation stage.**

Objective: prove one real learner lesson through one composition root and concrete dependencies.

Required path:

```text
learner text
 -> apps/nexa-desktop
 -> session orchestrator
 -> SQLite durable state
 -> learning/pedagogy
 -> governed TCP course knowledge
 -> local llama.cpp model adapter
 -> admitted/quality-checked tutor response
 -> assessment/practice
 -> atomic evidence/mastery/progress commit
 -> exit/restart/resume
```

Required concrete elements:

- `apps/nexa-desktop` learner application shell;
- `eframe`/`egui` suitability proven by bounded spike;
- active `crates/nexa-storage` SQLite/`rusqlite` adapter;
- governed Networking Fundamentals/TCP content package;
- concrete local llama.cpp adapter;
- existing learning/tutor/knowledge contracts composed where applicable;
- content-safe operational correlation.

R2 exit:

A learner can complete the bounded TCP lesson through the actual desktop boundary with the real local model and durable state, then restart/resume without lost or duplicated accepted progress.

Scripted provider outcomes and in-memory persistence do not close R2.

### R2 implementation sequence guidance

Prefer small vertical increments:

1. Desktop + storage architecture activation.
2. Durable learning state and restart/resume foundation.
3. Governed first-course/content persistence/retrieval path.
4. Concrete local model adapter.
5. Compose tutor/knowledge/model into desktop interaction.
6. Compose assessment/evidence/progress commit.
7. Close the R2 E2E acceptance scenario.

The exact PR boundaries may change with evidence, but each must make the vertical path more concrete.

## R3 — Robust complete session orchestration

Status: Not started.

Objective: turn the R2 skeleton into a resilient learner session.

Required maturity:

- actual lifecycle/cancellation foundations composed into the learner workflow;
- finite timeout policy;
- bounded retry/recovery policy;
- learner-visible dependency failures;
- cancellation/shutdown without state corruption;
- startup/recovery behavior;
- workflow-level observability.

Reuse ADR-0051 through ADR-0067 only where they directly support the actual session path.

Exit: the primary learner journey survives expected model/retrieval/storage failures, cancellation, shutdown, and restart without corrupting accepted state.

## R4 — Grounding and tutor quality

Status: Not started.

Objective: prove the tutor is not merely structurally valid but acceptable for released instruction.

Required evidence:

- governed first-course evaluation set;
- factual grounding against supplied sources;
- citation fidelity/support;
- instructional correctness/usefulness;
- assessment-answer protection;
- relevant prompt-injection/hostile-source cases;
- exact model/configuration/version evidence;
- repeatable quality thresholds and known nondeterminism.

This stage also selects the release model/quantization from evidence rather than architecture preference.

Exit: the exact release-intended model/configuration meets the approved first-course quality gate.

## R5 — Speech/avatar product decision and optional integration

Status: Not started; not R2 blocking.

Objective: decide from the functioning text tutor whether speech and/or animated embodiment are required for the first public v1 release or should ship later.

If promoted, a capability must advance beyond existing contracts to concrete adapters and system verification:

- real microphone/STT and/or TTS/audio for speech;
- semantic behavior synchronization;
- actual avatar asset/runtime integration through NBP/avatar boundaries;
- interruption/synchronization/privacy/accessibility evidence.

If not promoted, retain the existing foundations for post-v1 without blocking release.

## R6 — Labs/tools if promoted

Status: Post-R2 by default.

Only enter this stage before v1 if the accepted course/release outcome requires actual tool/lab practice.

Promotion requires:

- real tool execution;
- actual sandbox/enforcement;
- authorization/confirmation UX;
- resource/network/filesystem restrictions;
- observation/evidence;
- timeout/cancellation/orphan cleanup;
- security testing.

Contract declarations/cancellation controls alone are insufficient.

## R7 — Performance and operational hardening

Status: Not started.

Objective: measure and harden the exact release path.

Measure on an approved Windows reference environment:

- startup/readiness;
- UI responsiveness under model work;
- storage load/commit;
- retrieval/context assembly;
- model learner-visible latency;
- memory/disk/resource use;
- restart/resume;
- failure/recovery behavior.

Optimization is selected from measured failures against accepted budgets, not assumptions.

Exit: no unresolved release-blocking performance/operational finding.

## R8 — Packaging, install, update, and distribution

Status: Not started.

Required outcomes:

- reproducible Windows x86_64 release build;
- final package/installer decision;
- model/runtime distribution/configuration decision;
- application/data/config/log locations;
- upgrade and schema migration;
- rollback/recovery where applicable;
- uninstall/data-retention behavior;
- artifact provenance/signing/attestation policy;
- third-party dependency/model/runtime/license/asset provenance.

`cargo-dist` may be evaluated as release-artifact orchestration; final packaging technology is evidence-driven.

Exit: a release candidate can be installed/upgraded/uninstalled according to the supported v1 policy.

## R9 — System verification, user acceptance, release

Status: Not started.

Required gates:

- complete primary learner E2E test with release-equivalent adapters;
- migration/restart/recovery tests;
- security/privacy review of actual flows;
- performance budgets passed;
- package/install/upgrade acceptance;
- accessibility acceptance for the supported UI;
- representative user acceptance;
- no unresolved critical architecture, security, privacy, data-integrity, or system-quality finding.

Exit: Nexa v1 satisfies the approved v1 acceptance statement and may be declared Release Ready.

## Capability maturity vocabulary

Use:

`Concept -> Architecture Defined -> Specification Approved -> Contract Implemented -> Runtime Integrated -> Concrete Adapter Implemented -> System Verified -> User Accepted -> Release Ready`

Each status claim must name its scope and evidence.

## Rebaseline checkpoints

A whole-system architecture/program-integrity review occurs:

- at R2 exit;
- before material R3 broadening;
- before promoting speech/avatar/labs into the release critical path;
- before R8 release engineering;
- whenever material drift signals appear.

The outcome is explicitly Continue, Redirect, or Tactical Pause.

## Current next action

After the rebaseline PR is green on the exact final head and merged, begin R2. Do not resume the superseded horizontal Phase 5 sequence.