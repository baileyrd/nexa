# Nexa Development Divergence Analysis

Status: Tactical-pause forensic assessment

Baseline reviewed through PR #109 / merge commit `e2345a1bb8825451ea079ff5e350b7765075038a`.

## Purpose

This document identifies where Nexa development diverged from the intended systems-engineering path, which signals were available at the time, why the process continued, and which controls would have prevented the drift.

The purpose is not to retroactively classify every merged change as wrong. Most increments were locally correct, carefully scoped, and explicitly documented. The failure was that the program lacked a blocking system-level mechanism to determine whether continuing that class of work remained the right next action.

## Summary conclusion

The first material divergence occurred when implementation began before the Phase 0 baseline-governance exit criteria had actually been satisfied. Later phases preserved and compounded that condition. The clearest missed rebaseline opportunity occurred after the repository had already documented specification/status inconsistencies but still selected another narrow implementation slice. By late Phase 4 and Phase 5, the pattern had become systemic: accepted ADRs and well-tested provider-neutral/control contracts advanced rapidly while parent subsystem specifications remained Baseline Draft and the primary learner-facing vertical path still had no concrete model, durable persistence, production UX, or release boundary.

## Timeline

### Stage 1 — Reconstructed platform baseline: necessary but incomplete

**PR #2 — Establish the Nexa platform baseline**

This PR correctly reframed Nexa as a complete adaptive tutor platform, established the reconstructed documents as the working baseline, created the specification registry, recorded ADR-0001, and introduced a phased roadmap.

The important limitation was explicit: reconstructed specifications were preserved rather than fully reviewed or semantically normalized. The new roadmap defined Phase 0 as a governance/audit phase whose exit required active specifications to have authoritative status, dependencies, ownership/boundaries, and related acceptance evidence.

**Assessment:** correct foundation step, but not Phase 0 completion.

### Stage 2 — First divergence: implementation begins before Phase 0 exit

**PR #3 — Validation and implementation plan**

The plan surfaced specification conflicts, missing decisions, dependency risks, and an ADR queue. It then prepared the first implementation increments.

**PR #4 — Phase 1 shared contract kernel**

The PR described itself as recording "Gate 0 decisions before wider implementation" and began the contract kernel.

However, the repository's own Phase 0 exit criteria were broader than recording the immediate ADR queue. The specification registry still contained reconstructed/Baseline Draft authorities and known audit work.

**Divergence:** Phase 1 implementation began while Phase 0 remained materially incomplete.

**Why it was easy to miss:** the contract kernel was a sensible, low-risk next technical step and the immediate ADR decisions were strong. Local readiness was mistaken for program readiness.

**Preventive control:** G1/G2 parent-maturity gate. Phase 1 implementation could proceed only after an explicit Phase 0 exit review or an approved exception with a mandatory rebaseline date.

### Stage 3 — Useful vertical logic appears, but production deferrals begin accumulating

**PRs #8–#12 — Student, pedagogy, lessons, assessment, learning-core composition**

These increments created a valuable deterministic headless learning path. The Phase 3 end-to-end conformance is real evidence: assessment, evidence ingestion, mastery replay, pedagogy, authored routing, and atomic in-memory commit were composed.

At the same time, the PRs and ADRs explicitly deferred durable persistence, concurrency/isolation, authorization, retention, recovery, migration, outbox publication, and other production semantics.

**Assessment:** strong foundation and the closest early example of vertical product logic, but not a production walking skeleton because real persistence, application UX, concrete providers, and runtime integration were absent.

**Missed control:** inherited deferrals should have been entered into a program-visible register with mandatory review before later phases crossed into production/runtime concerns.

### Stage 4 — Phase 4 begins correctly, then becomes horizontally deep

**PRs #13–#19 — Knowledge ingestion through tutor response planning**

This sequence added governed ingestion, lexical/vector/hybrid retrieval, context assembly, citations, and structural tutor planning. These were coherent, valuable foundations.

The important boundary was repeatedly explicit: no concrete model provider, networking, durable storage, semantic entailment/truth, or production integration.

**Assessment:** still defensible contract-first work, but by PR #19 the next systems question should have been whether to establish the first concrete tutor path rather than continue deepening provider-neutral abstraction.

### Stage 5 — Clearest missed rebaseline opportunity

**PR #20 — Repository continuity foundation**

This PR is a key forensic checkpoint. It created `AGENTS.md` and `PROJECT-STATUS.md`, explicitly recorded a documentation inconsistency, and created a durable workflow for resuming from repository truth.

Its own follow-up recommendation nevertheless selected another narrow Phase 4 capability: model-provider abstraction/safety gates, while explicitly advising not to bundle provider selection, generation, networking, or persistence.

This was a high-value moment to stop and ask:

- Why is the architecture still reconstructed?
- Why are active parent specifications still Baseline Draft?
- Which Phase 3/4 deferrals are now blocking a usable tutor?
- What is the first releasable Nexa experience?
- Should the next milestone be a concrete vertical walking skeleton rather than another contract boundary?

**Divergence checkpoint:** the repository had enough governance visibility to detect the issue, but the process still optimized for the next bounded PR.

**Preventive control:** mandatory periodic architecture rebaseline independent of the `next` implementation trigger.

### Stage 6 — Horizontal abstraction becomes systemic

**PRs #21–#43 and #48–#72 — Model invocation, selection, authorization, filtering, tokenization, capacity, usage reconciliation, and composition variants**

This sequence created a large, internally rigorous provider-neutral tutor surface:

- model invocation contracts;
- prompt compilation;
- strict model-output admission;
- invocation/admission composition;
- registry mechanics;
- static selection;
- local-only and availability-gated paths;
- remote authorization;
- disclosure filtering;
- exact tokenization and capacity validation;
- reported usage reconciliation;
- multiple local/remote/filtered composition variants.

The PR descriptions repeatedly stated that `NEXA-TUTOR-001` remained Baseline Draft and that concrete provider integration, inference, networking, durable persistence, semantic correctness/safety, dynamic routing, and other production capabilities remained absent.

**Assessment:** the individual changes were often high quality. The system-level error was allowing many successive horizontal variants while the concrete learner path remained unchanged.

**Strong signal:** the ratio of abstraction/composition PRs to new learner-visible capability became very high.

**Preventive control:** after a defined number or threshold of horizontal increments, G4 rebaseline must require evidence of vertical capability advancement before approving more horizontal depth.

### Stage 7 — Contract gate formally labeled Phase 4 closure

**PR #74 — Close Phase 4 and establish Phase 5 lifecycle contracts**

This PR explicitly closed Phase 4 at a qualified deterministic headless contract gate and began Phase 5 lifecycle work. It also explicitly stated that `NEXA-ORCH-001` remained Baseline Draft and that async runtime, subsystem integration, provider, networking, persistence, tool, speech, renderer, clock, and side-effect capabilities were not introduced by that increment.

The qualification was technically honest, but the program-level phrase "Close Phase 4" created a stronger maturity signal than the product state justified.

**Architecture-gate failure:** a contract/conformance gate was allowed to stand in for a broader phase-completion signal while critical parent and product maturity remained incomplete.

**Preventive control:** capability maturity terminology must be explicit; `Contract Gate Satisfied` cannot be represented simply as `Complete` or used to advance program maturity without a separate architecture/product gate.

### Stage 8 — Phase 5 repeats the pattern around cancellation/control

**PRs #76–#109**

The Phase 5 sequence added:

- cancellation propagation planning;
- propagation port;
- Tokio task ownership;
- target-aware ownership;
- exact-plan runtime execution;
- Behavior binding;
- Tutor Generation cancellation control and binding;
- async Retrieval service and binding;
- Speech cancellation control/coordinator/binding;
- Tool Execution security/cancellation contracts and binding;
- asynchronous Speech input foundation.

Again, the PRs generally described their limitations accurately. They repeatedly stated that acknowledgements or task joins did not prove real provider/device/external work stopped, and that concrete provider/networking/persistence/retry/recovery/observability or actual tool execution remained deferred.

Meanwhile `NEXA-ORCH-001`, `NEXA-TUTOR-001`, `NEXA-SPCH-001`, and `NEXA-LAB-001` remained Baseline Draft or otherwise below full product maturity.

**Assessment:** Phase 5 demonstrates the same systemic pattern that appeared in late Phase 4: strong local correctness and increasingly sophisticated control surfaces without equivalent progress on the complete learner journey.

### Stage 9 — Tactical pause after PR #109

PR #109 added a provider-neutral asynchronous Speech input port foundation and explicitly did not add microphone, VAD, codec, provider/model, networking, persistence, output/synthesis, playback, Behavior, or headless composition capability.

At this point the mismatch became difficult to ignore: significant engineering depth existed across many subsystems, yet a learner still could not install Nexa, use one real model, complete a persistent lesson, restart, and resume through one product application.

**Decision:** tactical pause and comprehensive rebaseline.

## Where the program actually diverged

There was not one bad PR. The divergence was a governance sequence:

1. Phase 0 was treated as sufficiently complete for implementation without satisfying its own full exit criteria.
2. Narrow ADRs became the preferred mechanism for resolving ambiguity incrementally.
3. Every PR was optimized for bounded reviewability and local correctness.
4. Deferrals were documented but not centrally gated.
5. Periodic whole-system architecture reviews were not mandatory.
6. No finite v1 release definition constrained work selection.
7. Contract/conformance gates became phase-progress signals.
8. Horizontal depth continued because each next increment was locally justifiable.

## Why the existing workflow did not stop it

The ChatGPT/Codex workflow was optimized for:

- one bounded increment at a time;
- exact-scope implementation;
- PR diff review;
- tests and exact-head CI;
- documentation/traceability consistency with current authorities;
- correction and merge safety.

Those are valuable controls, but they answer the question:

> Is this change correct under the current local authorities?

They did not independently force the question:

> Are the current authorities, roadmap, and class of work still the correct system-level path to the intended release?

Without the second question, the workflow could reliably produce locally correct drift.

## Chief Systems Architect responsibility

The Chief Systems Architect should have recognized and acted on these signals earlier:

- parent architecture/specification maturity remained behind implementation;
- the number of horizontal contract/composition increments increased without learner-visible progress;
- recurring deferrals crossed phase boundaries;
- project status required increasingly qualified descriptions of "complete";
- the repository itself recorded documentation inconsistencies;
- the system had no finite release definition;
- concrete adapters and production composition repeatedly remained deferred.

The role failure was not approving obviously incorrect designs. It was failing to elevate these cumulative signals into a blocking program-integrity decision.

## Corrective controls mapped to divergence points

| Divergence | Required control |
|---|---|
| Phase 1 starts before Phase 0 exit | Parent-maturity / explicit baseline-exit gate |
| Production deferrals accumulate in Phase 3 | Central deferral register with mandatory review milestones |
| PR #20 records inconsistency but continues narrow work | Independent periodic architecture rebaseline |
| Long Phase 4 wrapper/composition sequence | Horizontal-depth threshold requiring vertical progress evidence |
| Phase 4 contract gate called complete | Explicit capability maturity vocabulary |
| Phase 5 control surfaces deepen without product path | Primary-journey blocker priority and walking-skeleton gate |
| Parent specs remain Baseline Draft | Child implementation maturity cannot outrun parent authority |
| No installable persistent learner journey | Finite v1 definition and release acceptance criteria |

## Final lesson

The Nexa failure mode was not uncontrolled coding. It was **well-controlled incremental engineering without sufficient whole-system program control**.

That distinction matters because the remedy is not to abandon small PRs, ADRs, deterministic tests, or contract-first design. Those practices should remain. The remedy is to place them inside stronger architecture maturity, vertical integration, deferral, and release-convergence gates.
