# Nexa R1 Critical Specification Plan

Status: Proposed rebaseline execution plan

## Purpose

This plan defines the minimum documentation work required to close Completion Roadmap Stage R1 without recreating the earlier failure mode of implementing from incomplete parent authorities.

The objective is not to fill every reserved namespace. It is to create only the cross-cutting specifications required by the accepted Nexa v1 path, in dependency order, with clear decisions and acceptance evidence.

## Authoring rule

Each new specification must state:

- purpose and v1 scope;
- owners and affected subsystem boundaries;
- externally observable/systemically significant behavior;
- required inputs/outputs/state and ownership;
- failure behavior;
- security/privacy implications;
- compatibility/version/migration expectations where applicable;
- dependencies on other Nexa authorities;
- acceptance and conformance evidence;
- explicit v1 exclusions/post-v1 deferrals;
- document maturity/status.

No specification is considered ready merely because a namespace/file exists.

## Sequence

### R1.1 — Data and Persistence

**Why first:** student, lesson, assessment, knowledge, orchestration, restart/resume, packaging upgrades, privacy retention, and recovery all depend on a coherent state model.

Must decide/define:

- authoritative vs derived state;
- immutable evidence/replay rules;
- transaction and atomicity boundaries;
- supported concurrency model for v1;
- persistent identities and foreign associations;
- migration/schema versioning;
- backup/recovery and corruption behavior;
- retention/deletion hooks;
- durable knowledge provenance;
- whether authoritative events/outbox participate in commits;
- storage-adapter ownership versus domain ownership;
- persistence test strategy.

Must **not** select a database before these behavioral requirements are clear. Technology selection follows as an ADR based on the requirements and v1 constraints.

### R1.2 — Security Architecture

**Why next:** concrete persistence and model-provider integration introduce credential, local filesystem, network, and trust boundaries.

Must define:

- v1 threat model;
- trusted/untrusted boundaries;
- local learner ownership/authentication assumptions;
- provider credential lifecycle/storage;
- allowed network destinations and transport assumptions;
- least-privilege filesystem/process behavior;
- dependency/update/supply-chain expectations;
- sensitive diagnostic handling;
- security consequences if labs/tools are promoted to v1;
- security verification evidence.

### R1.3 — Privacy and Data Handling

**Dependency:** uses the data inventory and security boundaries.

Must define:

- learner-data classes;
- instructional/content classes;
- local-only vs remotely disclosable data;
- prompt disclosure policy;
- retention/deletion/export expectations;
- diagnostic/telemetry redaction;
- privacy behavior when remote inference is disabled/unavailable;
- provider data-handling assumptions that must be visible to configuration/user experience;
- privacy verification evidence.

Existing ADR-0033 structural filtering is reusable evidence but is not itself the privacy policy.

### R1.4 — Session Orchestration v1 Parent Specification

**Why here:** after state/security/privacy boundaries exist, the orchestrator can define the complete learner workflow without inventing those concerns locally.

Must consolidate existing ADR-0051+ foundations and define:

- primary interaction sequence;
- dependency ownership/lifetime;
- durable load/commit points;
- cancellation/interruption;
- timeout policy;
- retry/recovery classification;
- failure-to-UX mapping contract;
- shutdown/restart/incomplete-workflow handling;
- operational correlation;
- conditional speech/avatar/lab attachment points;
- system integration acceptance.

### R1.5 — Observability

Must define:

- logs, metrics, traces/events needed for v1 operations;
- session/workflow/invocation correlation;
- content-safe diagnostic fields;
- startup/configuration/storage/retrieval/model/admission/recovery evidence;
- retention and privacy constraints;
- user-visible vs operator-visible failure information;
- observability acceptance tests.

Do not treat every domain event as telemetry or every telemetry event as an authoritative domain fact.

### R1.6 — Tutor/Knowledge v1 Maturity Reconciliation

This is primarily maturation of existing parent specifications, not a new parallel namespace.

Must consolidate existing Phase 4 ADRs into coherent v1 requirements for:

- durable knowledge ingestion/retrieval;
- first released corpus/content formats;
- one concrete configured provider path;
- provider/tokenizer association;
- grounding/citation semantics;
- semantic/instructional quality acceptance;
- remote disclosure/privacy integration;
- failure and retry behavior delegated appropriately to orchestration;
- post-v1 routing/fallback exclusions.

### R1.7 — UX / Learner Application

Must define:

- first-launch/configuration flow;
- course selection/resume;
- active lesson/tutor interaction;
- assessment/practice presentation;
- progress feedback;
- model/provider unavailable states;
- recoverable/terminal errors;
- restart/resume user behavior;
- accessibility baseline;
- conditional speech/avatar presence if selected;
- UX acceptance evidence.

The UX specification presents domain state but does not own tutor, learning, or orchestration policy.

### R1.8 — Testing and System Acceptance

Must define evidence by maturity layer:

- unit;
- contract;
- conformance;
- integration;
- concrete-adapter;
- end-to-end system;
- failure injection;
- security/privacy;
- performance;
- accessibility;
- user acceptance;
- release acceptance.

Must prohibit generic `Complete` status when only a lower maturity evidence layer has passed.

### R1.9 — Performance Budgets

Must define measurable v1 budgets for at least:

- application startup;
- local state load/commit;
- retrieval/context assembly;
- tutor/model response latency boundaries (with provider-dependent classification);
- interaction responsiveness;
- memory/resource usage;
- course/knowledge corpus scale;
- packaging/install footprint where relevant.

No optimization work is authorized solely from intuition; budgets and representative measurement come first.

### R1.10 — Packaging and Deployment

Must define:

- first supported OS/platform target;
- application packaging/installer format decision boundary;
- install/config/data/cache/log locations;
- provider configuration and secret integration;
- upgrade/data migration;
- rollback/recovery expectations;
- uninstall/retained-data behavior;
- release version/provenance;
- third-party license and asset provenance;
- clean-install/upgrade/uninstall acceptance matrix.

## Conditional specification decisions

Before R1 closes, the architecture review must decide whether these are **v1 release capabilities** or explicitly **post-v1**:

- Speech input;
- Speech output/TTS;
- animated avatar/behavior synchronization;
- labs/tool execution.

If post-v1, their existing specifications/contracts are preserved but no further implementation is selected during the v1 critical path except maintenance required to keep the repository healthy.

If promoted to v1, their parent specifications must be matured and concrete adapter/system acceptance work added to R1/R2+.

## Documentation maturity gates

A cross-cutting specification passes R1 only when:

1. its parent/system architecture relationship is explicit;
2. v1 scope and exclusions are explicit;
3. unresolved decisions are either blocking or assigned to named ADR decisions;
4. downstream subsystem specs can reference it without inventing conflicting policy;
5. acceptance evidence is defined;
6. registry/status reflects the actual document maturity;
7. no implementation task is needed to discover fundamental requirements that should have been decided here.

## R1 exit matrix

| Authority | Required maturity before R2 |
|---|---|
| v1 system architecture | Reviewed/approved working authority |
| Data/Persistence | Approved v1 requirements + technology ADR as needed |
| Security | Approved v1 requirements/threat boundaries |
| Privacy | Approved v1 data/disclosure/retention policy |
| Session Orchestration | Approved complete v1 workflow requirements |
| Observability | Approved minimum operational evidence |
| Tutor/Knowledge parents | Approved v1 slices including concrete-path and quality requirements |
| UX | Approved primary learner journey and error states |
| Testing/System Acceptance | Approved evidence/maturity model |
| Performance | Approved measurable budgets |
| Packaging/Deployment | Approved first-target release behavior |

## R2 handoff criterion

The first walking-skeleton implementation increment may be selected only when the team can draw a direct trace from:

`v1 learner acceptance -> v1 system architecture -> owning parent specifications -> concrete R2 capability -> test/acceptance evidence`.

If that chain is incomplete, R1 is not complete and implementation remains paused.
