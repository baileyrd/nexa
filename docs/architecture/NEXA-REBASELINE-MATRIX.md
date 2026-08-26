# Nexa Specification and Capability Rebaseline Matrix

Status: Tactical-pause working baseline

## Purpose

This matrix classifies the current governed document inventory against the proposed Nexa v1 release boundary. It identifies which authorities must be matured before implementation resumes, which are conditionally required, and which may remain post-v1.

## Disposition vocabulary

- **V1 Required — Mature now**: must be reviewed/promoted/revised before dependent implementation resumes.
- **V1 Required — Create now**: missing/reserved cross-cutting authority required for the v1 path.
- **Conditional v1**: only becomes release-critical if the capability is explicitly included in v1.
- **Post-v1**: preserve namespace/intent but do not allow it to distract from the first release.
- **Foundation evidence**: implementation exists and remains useful, but maturity/status must be reported precisely.

## Existing governing/system documents

| Document | Current registry status | v1 disposition | Rebaseline action |
|---|---|---|---|
| `NEXA-ARCH-001` Tutor System Architecture | Reconstructed | V1 Required — Mature now | review against current code and v1 scope; revise/promote or supersede; establish authoritative system context and interfaces |
| `NEXA-CBS-001` Character & Behavior Specification | Baseline Draft | Conditional v1 | review if avatar/behavior embodiment is included; otherwise preserve as post-v1/conditional authority |
| `BASELINE.md` | Governing policy | V1 Required — Mature now | revise Phase 0 interpretation so parent maturity gates implementation depth |
| `SPECIFICATION-REGISTRY.md` | Navigation/governance authority | V1 Required — Mature now | add explicit maturity/disposition and remove ambiguous generic completion language |
| `IMPLEMENTATION-ROADMAP.md` | Current delivery roadmap | Superseded by rebaseline after approval | reconcile into completion roadmap and preserve historical phase evidence |
| `PROJECT-STATUS.md` | Current checkpoint | V1 Required — Mature now | rewrite current status around capability maturity and tactical pause |

## Existing subsystem specifications

| Specification | Current status | v1 disposition | Rebaseline action |
|---|---|---|---|
| `NEXA-DOM-001` Core Domain Model | Baseline Draft | V1 Required — Mature now | audit implemented canonical types vs full v1 domain needs; approve bounded v1 authority |
| `NEXA-EVT-001` Event Model | Baseline Draft | V1 Required — Mature now | define v1 event/runtime scope, durable/replay requirements, retention and backpressure expectations |
| `NEXA-NBP-001` Behavior Protocol | Implemented Phase 2 slice | Conditional v1 / Foundation evidence | keep implemented slice; mature only as required by chosen v1 embodiment path |
| `NEXA-STU-001` Student Model | Implemented Phase 3 slice | V1 Required — Mature now | reconcile evidence/mastery contracts with durable learner-state requirements |
| `NEXA-PED-001` Adaptive Pedagogy | Implemented Phase 3 slice | V1 Required — Mature now | approve v1 policy behavior and explicit limits; integrate into primary journey |
| `NEXA-TUTOR-001` Tutor Intelligence | Baseline Draft | V1 Required — Mature now | consolidate ADR-0021–0050 decisions into coherent v1 tutor requirements; add concrete-provider and quality acceptance needs |
| `NEXA-KNOW-001` Knowledge/RAG | Implemented synchronous Phase 4 slices | V1 Required — Mature now | define durable store, production ingestion/retrieval, corpus limits and citation/grounding acceptance |
| `NEXA-LESSON-001` Curriculum/Lessons | Implemented Phase 3 slice | V1 Required — Mature now | reconcile authored content pipeline and persistent progress/resume requirements |
| `NEXA-ASMT-001` Assessment | Implemented Phase 3 slice | V1 Required — Mature now | define production content/persistence and learner-facing assessment acceptance |
| `NEXA-LAB-001` Labs/Sandbox | Baseline Draft with contract foundations | Conditional v1 | decide explicit v1/post-v1 disposition before further implementation |
| `NEXA-SPCH-001` Speech Interaction | Baseline Draft with foundations | Conditional v1 | decide text-first vs speech-required v1; stop speech depth until decision |
| `NEXA-AVTR-001` Avatar Runtime | Implemented Phase 2 slice | Conditional v1 | preserve working renderer-neutral foundation; integrate only if selected for v1 |
| `NEXA-3D-001` 3D Character | Implemented Phase 2 slice | Conditional v1 | preserve foundation/viewer; do not make release-critical unless explicitly selected |
| `NEXA-3D-ART-001` 3D Production | Baseline Draft | Post-v1/Conditional | defer unless v1 embodiment requires production asset pipeline |
| `NEXA-3D-REF-001` Canonical 3D Reference | Baseline Draft | Post-v1/Conditional | preserve canonical identity; not a core release blocker if text-first v1 |
| `NEXA-3D-RUNTIME-001` 3D Runtime Validation | Implemented slice | Foundation evidence | preserve and maintain |
| `NEXA-ORCH-001` Session Orchestration | Baseline Draft with extensive Phase 5 foundations | V1 Required — Mature now | consolidate lifecycle/cancellation ADRs and define complete primary-session orchestration, timeout, retry, recovery and concrete composition |

## Reserved namespaces — v1 classification

| Namespace | Current state | v1 disposition | Required action |
|---|---|---|---|
| 13 UX | Reserved | V1 Required — Create now | specify primary learner journey, application states, errors/recovery and accessibility |
| 14 Authoring | Reserved | Post-v1 by default | use hand-authored governed fixtures/content for v1 unless evidence requires authoring UI |
| 15 Data | Reserved | V1 Required — Create now | define persistent state, transactions, migration, retention, backup/recovery |
| 16 Security | Reserved | V1 Required — Create now | define threat/trust/credential/privilege model |
| 17 Privacy | Reserved | V1 Required — Create now | define learner data, disclosure, retention/deletion and diagnostics policy |
| 18 Observability | Reserved | V1 Required — Create now | define logs/events/metrics/correlation and content-safe operations evidence |
| 19 Analytics | Reserved | Post-v1 by default | do not block first release |
| 20 Plugins | Reserved | Post-v1 | do not implement before v1 |
| 21 API | Reserved | Post-v1 by default | internal interfaces suffice for first release unless packaging architecture proves otherwise |
| 22 Packaging | Reserved | V1 Required — Create now | define build/install/configure/update/uninstall and provenance |
| 23 Deployment | Reserved | V1 Required — Create now | define first supported desktop deployment target and release mechanics |
| 24 Performance | Reserved | V1 Required — Create now | define measurable release budgets rather than optimize speculatively |
| 25 Testing | Reserved | V1 Required — Create now | define integration/system/E2E/user-acceptance maturity evidence |
| 26 Engineering | Reserved | V1 Required — Create/bind now | codify maturity model, rebaseline gates and release-oriented work selection; reusable rules move to Atlas |
| 27 Governance | Reserved | V1 Required — Create/bind now | codify document maturity/status transitions and authority responsibilities or reference governing project controls |

## Immediate Phase 0 rebaseline order

1. Approve the v1 product boundary.
2. Mature `NEXA-ARCH-001` into the authoritative v1 system architecture.
3. Mature the required parent subsystem specifications in dependency order: domain/events -> student/lesson/assessment/pedagogy -> knowledge/tutor -> orchestrator.
4. Create cross-cutting v1 specifications for data, security, privacy, observability, testing, UX, packaging/deployment and performance.
5. Reconcile every inherited ADR against those matured parent authorities; do not rewrite accepted decisions silently.
6. Classify every existing deferral into a mandatory review stage.
7. Update registry/status/README/roadmap terminology to the explicit capability maturity model.

## Restart criterion

Normal implementation may resume only when the next proposed code increment can point to:

- an authoritative v1 capability;
- a sufficiently mature parent specification;
- a named completion-roadmap stage;
- an acceptance outcome;
- the exact inherited deferral or release blocker it closes;
- evidence that the increment is higher leverage than remaining alternatives for the primary learner journey.
