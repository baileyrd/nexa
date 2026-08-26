# Nexa Architectural Deferral Register

Status: Active Control
Status date: 2026-08-26

## Purpose

This register makes inherited and new architectural deferrals visible at program level and prevents required work from silently rolling across release gates.

Originating specifications/ADRs remain authoritative for detailed technical decisions. This register owns only program disposition and mandatory review stage.

## Disposition vocabulary

- **R2 required** — blocks the walking-skeleton exit and must become concrete in R2.
- **Later v1 required** — required before Release Ready, owned by a later R3–R9 gate.
- **Conditional v1** — required only if the associated capability is promoted into the first release.
- **Post-R2/Post-v1** — explicitly outside the current critical path unless promoted by evidence.
- **Resolved/Narrowed** — the tactical pause made the blocking decision; remaining detail is owned elsewhere.

## Current deferrals

| Deferral | Origin | Current disposition | Mandatory review |
|---|---|---|---|
| Durable learner/lesson/evidence/mastery persistence | Phase 3 / ADR-0010/0014 | **R2 required** — SQLite adapter selected by ADR-0068 | R2 |
| Learning atomic transaction | Phase 3 | **R2 required** — implement existing UoW intent with SQLite transaction | R2 |
| Learning concurrency | Phase 3 | **Resolved/Narrowed** — one desktop process/store writer supported; stale versions still fail closed | R2 tests |
| Learning retention/reset | Phase 3 | **Later v1 required** — local learner reset mechanism required; fuller packaging lifecycle at R8 | R2/R8 |
| Learning/content schema migration | Phase 3 | **R2 required foundation**, release upgrade matrix later | R2 then R8 |
| Durable event broker/store | Phase 1 | **Post-R2** — not required without a durable asynchronous correctness consumer | R3 review |
| Durable outbox | Phase 3 | **Post-R2** under same rule; promote only with authoritative async consumer | R3 review |
| Async event backpressure | Phase 1 | Required only for event path actually adopted | R3/R7 |
| NBP arbitration/race/canvas richness | Phase 1–2 | Conditional v1 | R5 |
| Async avatar transport/synchronization | Phase 2 | Conditional v1 | R5 |
| Rich curriculum branching/freeform routing | Phase 3 | Post-R2/Post-v1 unless first course proves need | R4/course review |
| Advanced assessment weighting/timing/selection | Phase 3 | Post-R2 unless first course proves need | R4 |
| Assessment protection/security | Phase 3 | Later v1 required for released scope | R4/R9 |
| Durable knowledge store/provenance | Phase 4 | **R2 required** — SQLite-backed concrete path | R2 |
| Concrete model provider/inference | Phase 4 | **R2 required** — local llama.cpp adapter selected | R2 |
| Concrete tokenizer/capacity behavior | Phase 4 | R2 required only to extent selected adapter/model contract needs it | R2 |
| Dynamic model routing | Phase 4 | Post-R2/Post-v1 | after R4 |
| Automatic local-first fallback chains | Phase 4 | Post-R2; local path is explicit in R2 | after R4 |
| Provider health/latency/cost routing | Phase 4 | Post-v1 by default | post-v1 |
| Automatic model retry/regeneration | Phase 4 | **Resolved/Narrowed** — no automatic post-consumption retry in R2 | R3/R4 |
| Remote-provider privacy/credentials | Phase 4 | Post-R2; R2 has no remote disclosure | R5/R8 if promoted |
| Semantic citation fidelity/grounding quality | Phase 4 | Later v1 required | R4 |
| Semantic safety/instructional quality | Phase 4 | Later v1 required | R4 |
| Prompt-injection/hostile-source quality testing | Phase 4 | Later v1 required for released corpus | R4/R9 |
| Dedicated vector database | Phase 4 | Conditional on measured corpus/performance evidence | R4/R7 |
| Async/streaming model generation | Phase 4 | Post-R2 by default | R7 may promote |
| Complete session cancellation/recovery | Phase 5 | Later v1 required; compose real learner workflow | R3 |
| Concrete provider cancellation | Phase 5 | Required only to extent selected provider can/should support it | R3 |
| Concrete retrieval dependency cancellation | Phase 5 | Required only for actual R2/R3 dependency behavior | R3 |
| Speech microphone/STT | ADR-0067 | Conditional v1; not R2 | R5 |
| Speech output/TTS/audio | Phase 5 roadmap | Conditional v1; not R2 | R5 |
| Speech/provider/device cancellation | Phase 5 | Conditional v1 | R5 |
| Avatar/behavior synchronization | Phase 5 | Conditional v1 | R5 |
| Tool/lab execution and sandbox enforcement | Phase 5 | Post-R2; conditional first-release promotion only | R6 |
| Interruption/timeout/recovery policy | Phase 5 | Later v1 required; R1 baseline supplies governing rules | R3 |
| Clock ownership | Phase 5 | **Resolved/Narrowed** — app/orchestrator supplies canonical operation timestamps; DB defaults are not domain authority | R2/R3 |
| Learner UX implementation | reserved namespace | **R2 required** — apps/nexa-desktop + egui path | R2 |
| Content-safe observability | reserved/Phase 5 | **R2 required foundation**, broader operations later | R2/R7 |
| Performance budgets | reserved | Measurement required in R2; final release thresholds later | R2/R7 |
| Windows build/acceptance coverage | rebaseline | R2 required for release-critical surfaces before System Verified | R2 |
| Final installer/signing/update | Phase 6/reserved | Later v1 required | R8 |
| Local model runtime/model distribution | rebaseline | Configured dependency allowed in R2; final distribution later | R8 |
| Release GGUF model/quantization | R2/R4 | Development model allowed in R2; release selection later | R4 |
| User acceptance | reserved | Later v1 required | R9 |
| Repository/model/runtime/license/asset provenance | release | Later v1 required | R8/R9 |
| Plugins/public API/analytics/authoring/server/fleet | reserved | Post-v1 by default | post-v1 |

## R2 blocking set

Before R2 can close, the following must be concrete and integrated:

1. learner desktop application boundary;
2. SQLite durable state and migrations;
3. durable learning state atomicity/restart;
4. governed first-course/knowledge persistence/retrieval;
5. local llama.cpp adapter;
6. admitted tutor interaction;
7. assessment/evidence/mastery/progress commit;
8. restart/resume;
9. content-safe operational correlation;
10. Windows release-critical validation proportional to implemented surfaces.

No speech/avatar/lab/remote-routing item is in the R2 blocking set.

## Review rule

At each R0–R9 stage boundary:

1. review every deferral whose mandatory stage has arrived;
2. implement it, narrow/supersede the requirement explicitly, classify it later, or stop the stage;
3. never silently carry a required deferral forward;
4. update both this register and the originating status/specification/ADR when program disposition changes.

## New deferral rule

A new deferral is accepted only when it records:

- owning authority/boundary;
- why deferral is safe now;
- effect on the primary learner/release path;
- earliest required stage;
- consequence if still unresolved at that stage;
- explicit disposition/review owner.

Repeated deferrals in the same horizontal area trigger the whole-system architecture rebaseline gate.
