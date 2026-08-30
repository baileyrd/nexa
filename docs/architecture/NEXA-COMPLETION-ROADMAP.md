# Nexa Completion Roadmap

Status: Approved delivery control
Date: 2026-08-30

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

## G0–G2 — Authority, shared client, and speech evidence

G0 was the Issue #114 reconciliation and did not change implementation maturity. It is merged.

G1 is the only first separately dispatchable follow-on: prove identical shared frontend behavior in the same-machine browser and Tauri 2 Windows shell candidate over one versioned loopback HTTP/WebSocket API. Evidence covers loopback security, cancellation/reconnect, accessibility, Windows build, startup/CPU/memory/package impact, and parity. Failure removes the candidate and returns selection to owner authority.

G2 separately proves the bundled CPU speech candidate, including microphone/output devices, recognition and synthesis quality, interruption/cancellation, latency/resources, packaging, licensing, privacy, and accessible text/caption fallback. Sherpa-ONNX remains a candidate; failure invokes the ADR-0069 fallback/reselection path.

### G1 evidence disposition recommendation (Issue #122)

The [criterion-level G1 record](../../spikes/g1-shared-ui/evidence/G1-EVIDENCE.md) now includes the successful representative Windows harness at exact `main` head `9173ed152d7b1d5bb9831413862097076443be59` and separately identified owner-observed parity, cancellation/reconnect, accessibility-basics, resource, package-size, and timing evidence. It recommends **G1 evidence satisfied; candidate disposition pending separate authority review**.

This recommendation neither selects React/TypeScript/Vite or Tauri 2 nor promotes the fixture API into production architecture. It does not dispatch G2 or change maturity. Tactical Pause remains in force outside this evidence-recording increment, and G2+ remain undispatched until the Chief Systems Architect separately records candidate disposition and Continue, Redirect, or Tactical Pause.

## G3–G7 — 2D evidence and concrete vertical integration

G3 separately proves the Rive candidate renders identical synchronized animated 2D behavior in both clients, including semantic idle/listening/thinking/speaking/error states, lip-sync, interruption, reduced-motion/static fallback, accessibility, and CPU/resource evidence. Failure removes Rive and requires owner-governed reselection.

G4 integrates shared clients, the local Rust runtime, authoritative SQLite state/migrations, governed TCP content, atomic learning commits, and restart/resume. G5 integrates the narrow LM Studio adapter and admitted tutor output. G6 integrates required bundled speech. G7 integrates required synchronized animated 2D embodiment. Each gate requires a recorded Continue, Redirect, or Tactical Pause and evidence at the claimed maturity.

## G4–G7 — Robust complete session orchestration

Status: Not started.

Objective: compose the G4–G7 vertical route into a resilient learner session.

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

## G5/G8 — Grounding and tutor quality

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

## Preserved speech/avatar hardening requirements

Status: Required by G2/G3/G6/G7; not yet proven.

Objective: prove and integrate the required bundled speech and synchronized animated 2D capabilities through the finite release route.

G2 and G3 must establish candidate suitability evidence; G6 and G7 must advance the capabilities beyond existing contracts to concrete adapters and integrated release-path evidence; G8 must provide system verification:

- real microphone/STT and/or TTS/audio for speech;
- semantic behavior synchronization;
- actual avatar asset/runtime integration through NBP/avatar boundaries;
- interruption/synchronization/privacy/accessibility evidence.

Candidate failure triggers the ADR-0069 fallback/reselection path and an explicit authority update; it does not remove either required capability from v1.

## Deferred labs/tools

Status: Post-v1.

Labs/tools are not part of the owner-approved v1 route and cannot be promoted into v1 without new explicit owner and architecture authority.

Promotion requires:

- real tool execution;
- actual sandbox/enforcement;
- authorization/confirmation UX;
- resource/network/filesystem restrictions;
- observation/evidence;
- timeout/cancellation/orphan cleanup;
- security testing.

Contract declarations/cancellation controls alone are insufficient.

## G8 — Performance and operational hardening

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

## G8 — Packaging, install, update, and distribution

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

## G8/G9 — System verification, user acceptance, release

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

- after each candidate evidence gate (G1–G3) and before its production selection;
- before G4 vertical integration and before each material G5–G7 expansion;
- before G8 system verification and release engineering;
- before any proposal to change required speech/avatar scope or promote deferred labs/tools;
- whenever material drift signals appear.

The outcome is explicitly Continue, Redirect, or Tactical Pause.

## Current next action

Review and merge the Issue #122 G1 evidence record, then conduct the separate G1 candidate disposition and record Continue, Redirect, or Tactical Pause. Do not dispatch G2 from this evidence PR; Tactical Pause remains in force until the separate authority decision.
