# Nexa Architectural Deferral Register

Status: Proposed tactical-pause register; non-authoritative consolidation

## Purpose

Nexa has historically recorded deferrals inside ADRs and phase traceability. This register makes inherited deferrals program-visible and assigns each a mandatory completion-roadmap review point. It does not replace the originating ADR/specification.

## Disposition vocabulary

- **Required for v1** — must be resolved before the applicable release gate.
- **Conditional v1** — required only if the associated optional capability is included in v1.
- **Post-v1** — explicitly outside the first release unless promoted by architecture review.
- **Supersede/clarify** — documentation ambiguity must be reconciled during rebaseline.

## Initial inherited deferrals

| Deferral | Origin/evidence family | Current risk | Disposition | Mandatory review stage |
|---|---|---|---|---|
| Durable event store/replay | Phase 1 / NEXA-EVT-001 | event-driven architecture cannot be production-grade without explicit persistence/replay scope | Required for v1 if durable events are authoritative; otherwise explicitly narrow v1 event model | R1 Data/Event spec |
| Event privacy retention | Phase 1 | learner/event data lifecycle undefined | Required for v1 | R1 Privacy/Data |
| Async event backpressure | Phase 1 | runtime behavior under load/failure undefined | Required only for event paths used by v1 | R1/R3 |
| NBP arbitration/update races/canvas | Phase 1-2 | behavior protocol incomplete for richer interactive embodiment | Conditional v1 | R5 embodiment decision |
| Async avatar transport scheduling | ADR-0009 / Phase 2 | real synchronized embodiment not proven | Conditional v1 | R5 |
| Durable learning persistence | Phase 3 / ADR-0010/0014 | learner progress cannot survive restart safely | Required for v1 | R1 Data then R2 |
| Learning concurrency semantics | Phase 3 | duplicate/concurrent sessions may corrupt or conflict | Required for v1 supported usage model | R1 Data |
| Learning authorization | Phase 3 | state mutation trust boundary undefined | Required for v1 | R1 Security/Data |
| Learning retention/deletion | Phase 3 | learner data lifecycle undefined | Required for v1 | R1 Privacy/Data |
| Learning migration/versioning | Phase 3 | upgrades may invalidate persisted state | Required for v1 | R1 Data/R8 |
| Durable outbox/event publication | Phase 3 | atomic state/event publication unresolved | Required only if v1 event architecture depends on it | R1 Data/Event |
| Rich lesson branch conditions/freeform routing | Phase 3 | advanced adaptive curriculum incomplete | Post-v1 unless needed by first course | R0 course-scope decision |
| Assessment weighting/timing/selection/evaluator richness | Phase 3 | advanced assessment behaviors incomplete | Post-v1 unless first course requires them | R0/R5 |
| Assessment security/review semantics | Phase 3 | release assessment trust could be underspecified | Required for v1 scope actually exposed | R1 Security/Testing |
| Concrete model provider/inference | Phase 4 ADR-0022+ | no real tutor generation path | Required for v1 | R2 |
| Concrete tokenizer/provider token accounting | Phase 4 ADR-0036+ | exact provider capacity path not production connected | Required for selected v1 provider as needed | R2 |
| Dynamic model routing | Phase 4 | horizontal sophistication not needed for one-provider release | Post-v1 | after v1 |
| Automatic local-first routing/fallback | Phase 4 | architectural goal not productionized | Post-v1 by default | after v1 unless v1 provider policy requires it |
| Provider health/latency/cost routing | Phase 4 | no release need for single provider | Post-v1 | after v1 |
| Retry/repair/regeneration | Phase 4 | model failure behavior incomplete | Bounded retry/recovery required; advanced repair post-v1 | R3/R4 |
| General privacy policy/semantic minimization | Phase 4 | structural filtering alone is insufficient for real remote inference | Required for remote-provider v1 | R1 Privacy/R4 |
| Semantic citation fidelity/entailment | Phase 4 | structurally valid citations may not support claims | Required for v1 grounded tutor acceptance | R4 |
| Semantic safety/instructional quality | Phase 4 | structural admission does not prove educational quality | Required for v1 | R4 |
| Vector database integration | Phase 4 | current deterministic in-memory/vector contracts may not meet production corpus needs | Conditional on measured v1 corpus/performance | R1 Data/Performance then R2 |
| Durable knowledge store | Phase 4 | content/provenance not restart-safe in production | Required for v1 | R1 Data/R2 |
| Networking/remote transport/credentials | Phase 4 | concrete remote inference unavailable and secrets policy undefined | Required for remote-provider v1 | R1 Security then R2 |
| Async/streaming model execution | Phase 4 | responsiveness may suffer but not necessarily first-release blocker | Post-v1 by default | R7 performance may promote |
| Complete cancellation-safe execution | Phase 5 | many controls exist but full learner workflow not composed | Required for v1 primary journey | R3 |
| Concrete tutor generation cancellation | Phase 5 | control acknowledgement does not prove provider work stops | Required only to extent selected provider supports/needs cancellation semantics | R3 |
| Concrete retrieval dependency cancellation | Phase 5 | service-future stop does not prove external work stop | Required only for concrete v1 dependency behavior | R3 |
| Concrete speech provider/device cancellation | Phase 5 | current evidence covers control futures, not external stop | Conditional v1 | R5 |
| Actual tool/lab execution/sandbox enforcement | Phase 5 | contracts exist without execution | Conditional v1 / otherwise post-v1 | R5 |
| Speech microphone/STT adapter | ADR-0067 | speech input contract exists without real input | Conditional v1 | R5 |
| Speech output/TTS/audio path | Phase 5 roadmap | learner speech output unavailable | Conditional v1 | R5 |
| Behavior synchronization | Phase 5 roadmap | tutor/speech/avatar timing incomplete | Conditional v1 | R5 |
| Interruption/timeout/retry/recovery policies | Phase 5 roadmap | primary session failure behavior incomplete | Required for v1 | R1 Orchestrator spec then R3 |
| Event-driven observability | Phase 5 roadmap | runtime cannot be operated/diagnosed as intended | Required for v1 at release-appropriate level | R1 Observability/R3/R7 |
| Clock abstraction/time ownership | Phase 5 traceability | runtime time semantics incomplete | Required where deterministic recovery/timeouts/persistence need it | R1 Orchestrator/Data |
| UX/application definition | reserved namespace | no complete learner-facing product surface | Required for v1 | R1 UX then R2/R5 |
| Packaging/update strategy | Phase 6 / reserved | no distributable product | Required for v1 | R1 Packaging then R8 |
| Deployment target | Phase 6 / reserved | supported environment undefined | Required for v1 | R1 Deployment |
| Performance budgets | reserved | cannot know if real path is acceptable | Required for v1 | R1 Performance/R7 |
| System/E2E/user acceptance standard | reserved | contract tests cannot prove product acceptance | Required for v1 | R1 Testing/R9 |
| Repository-wide licensing/third-party asset provenance | README | distribution cannot be confidently released | Required for v1 | R8 |

## Review rule

At each completion-roadmap stage boundary:

1. Review every deferral whose mandatory stage has arrived.
2. Resolve it, narrow the release requirement explicitly, supersede it through approved authority, or classify it post-v1.
3. Do not silently carry a required deferral forward.
4. Update this register and the originating authority/traceability when disposition changes.

## New deferrals

No new deferral is accepted without:

- owning subsystem or cross-cutting authority;
- reason it is safe to defer;
- effect on the primary learner journey;
- mandatory review stage;
- test/evidence impact;
- explicit v1/post-v1 classification.
