# Nexa v1 Product Definition and Release Boundary

Status: Proposed rebaseline baseline

## Purpose

This document gives Nexa a finite first-release target. It intentionally separates what must work in v1 from capabilities that may follow later.

Nexa v1 is not defined by how many subsystem contracts exist. It is defined by whether a learner can complete a real adaptive lesson through one supported application and receive persistent, grounded, observable results.

## v1 mission

A learner can launch Nexa locally, enter or resume a course, interact with the tutor, receive grounded adaptive instruction, complete an assessment, have competency progress updated and persisted, and complete the lesson through one supported composition root.

The experience must use real runtime dependencies for every release-critical boundary rather than scripted test doubles.

## Primary v1 learner journey

1. Install and launch Nexa on a supported desktop platform.
2. Select or resume a locally available course.
3. Nexa loads learner state, lesson state, and governed course/knowledge content.
4. The learner provides text input. Speech input may be included if it reaches release maturity without blocking the core release.
5. The orchestrator establishes one interaction workflow.
6. Tutor context is assembled from lesson, learner, pedagogy, and governed knowledge.
7. One configured model provider generates a structured tutor response.
8. Nexa validates/adapts the response and presents it through the user interface.
9. The learner completes an assessment or governed practice step.
10. Nexa records evidence, updates/replays mastery, and chooses the next authored/adaptive route.
11. Progress is committed durably and can be resumed after restart.
12. Operational failures are surfaced clearly and do not corrupt learner state.

## Required v1 capabilities

### Application and UX

- One supported learner-facing desktop application/composition root.
- Course/lesson selection and resume.
- Text conversation/input and tutor response presentation.
- Visible lesson/progress state sufficient to understand current activity.
- Clear recoverable and terminal error presentation.
- Accessibility baseline proportional to the supported UI.

### Session orchestration

- Complete session and interaction workflow composition.
- Deterministic ownership of subsystem work.
- Cancellation/interruption for active learner interaction.
- Bounded timeout and retry/recovery behavior where dependencies can fail.
- Clean shutdown without state corruption.

### Learning core

- Authored curriculum/lesson progression.
- Student evidence ledger and mastery replay.
- Adaptive pedagogy decision.
- Assessment scoring/evidence.
- Atomic durable persistence of learning progress.

### Tutor and model execution

- One concrete supported model-provider adapter.
- One concrete tokenizer/capacity integration compatible with that provider where required.
- Governed prompt/context compilation.
- Strict structured output admission.
- Grounded citation-bearing responses over governed Nexa knowledge.
- Bounded failure behavior with no hidden fallback.

v1 does not require multi-provider dynamic routing. Provider neutrality remains an architectural property; release success needs one reliable concrete path first.

### Knowledge

- Durable governed content store.
- Production-capable ingestion for the v1 content format(s).
- Retrieval sufficient for the released course corpus.
- Context assembly and citation resolution integrated with tutor execution.
- Content/version provenance retained across restart.

A dedicated external vector database is not mandatory if the released corpus and performance requirements are satisfied by a simpler durable design.

### Persistence and data

Durably persist at minimum:

- learner/profile identity required by the application;
- course/lesson progress;
- competency evidence/mastery state or authoritative replay inputs;
- authored content/version identity;
- knowledge provenance required for citation/replay;
- operational state required for safe restart/recovery.

Persistence must define transaction, concurrency, migration, backup/recovery, retention, and corruption/failure behavior for v1.

### Security and privacy

- Local data boundaries and sensitive-data classification.
- Secrets/credential handling for any configured model provider.
- Explicit remote-disclosure boundary when remote inference is enabled.
- No sensitive content in normal diagnostics.
- Least-privilege file/network access appropriate to the application.
- Defined retention/deletion behavior for learner data.

### Observability and recovery

- Structured operational logging with content-safe diagnostics.
- Correlation across a learner interaction/workflow.
- Dependency failure visibility.
- Startup/recovery diagnostics.
- Sufficient evidence to troubleshoot a failed session without exposing learner/model content unnecessarily.

### Packaging and release

- Reproducible release build.
- Installer/package for at least one explicitly supported desktop target.
- Configuration path for local and/or remote model provider chosen for v1.
- Upgrade/migration strategy for persisted data.
- Version reporting and release notes.
- License/third-party asset provenance sufficient for distribution.

### System verification

- Automated end-to-end test covering the primary learner journey with production-equivalent adapters wherever practical.
- Persistence restart/resume verification.
- failure-injection tests for model, storage, and cancellation/recovery paths.
- security/privacy review against the v1 data flows.
- measured performance against explicit v1 budgets.
- user acceptance of the primary learner journey.

## Conditionally required for v1

These capabilities are valuable but must not block the first release unless subsequent evidence shows they are essential to the accepted learner experience:

- speech input;
- speech output/TTS;
- full animated avatar embodiment in the primary learner workflow;
- advanced vector retrieval;
- interactive labs/tool execution.

If included, each must meet the same concrete-adapter, integration, verification, and user-acceptance standards as other v1-required capabilities. A contract-only implementation is not enough.

## Post-v1 candidates

Unless explicitly promoted by the rebaseline:

- multi-provider dynamic routing;
- automatic local-first fallback chains;
- advanced cost/latency/task-complexity model routing;
- streaming multimodal orchestration beyond the v1 interaction path;
- advanced speech/VAD pipeline;
- fully synchronized avatar/speech/gesture behavior;
- generalized secure sandbox lab infrastructure;
- rich authoring application;
- plugin SDK/ecosystem;
- external public API;
- advanced analytics;
- fleet/server deployment;
- broad multi-platform distribution beyond the first supported target.

## Explicit non-definition

Nexa v1 is not complete merely because:

- all current unit tests pass;
- a phase traceability matrix is green;
- every provider-neutral contract exists;
- all planned crates/directories exist;
- all ADRs are accepted;
- the headless test composition proves deterministic behavior.

Those are evidence inputs, not product acceptance.

## v1 acceptance statement

Nexa v1 is release ready only when a new user can install the supported build, complete the primary learner journey with real configured dependencies, exit, restart, resume with correct durable state, and complete the release acceptance suite with no unresolved critical security, privacy, data-integrity, or architecture findings.
