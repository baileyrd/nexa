# Nexa v1 Data and Persistence Specification — Draft

Status: R1 proposal; non-authoritative until registered and approved

## 1. Purpose

This draft defines v1 data ownership, persistence behavior, atomicity, recovery, migration, and retention requirements needed by the primary learner journey. It intentionally defines behavior before selecting storage technology.

## 2. Scope

v1 persistence covers only data required to install, run, complete, persist, restart, resume, diagnose, migrate, and safely remove the primary Nexa learner experience.

In scope:

- learner identity/profile fields required by v1;
- curriculum/course/lesson version association;
- lesson progress;
- assessment attempts/results required for progression;
- immutable learning evidence;
- mastery projection/replay metadata;
- pedagogy-relevant persisted inputs where required;
- governed knowledge source/artifact/chunk provenance;
- operational/session recovery metadata required for safe restart;
- schema/version/migration metadata;
- backup/recovery behavior appropriate to local-first v1.

Out of scope by default:

- organization/multi-tenant administration;
- cloud synchronization;
- cross-device replication;
- analytics warehouse;
- plugin storage APIs;
- server/fleet data architecture;
- speculative generalized event sourcing beyond v1 correctness needs.

## 3. Data ownership principles

1. Domain subsystems own data semantics.
2. Storage adapters own persistence mechanics, not domain policy.
3. Immutable evidence is authoritative for mastery changes; derived mastery must remain reproducible from its governed evidence/policy inputs.
4. Authored content/version identity must remain explicit so persisted progress cannot silently bind to a different course/lesson definition.
5. Knowledge provenance required for citation/replay must survive restart.
6. Operational telemetry is not automatically authoritative domain state.
7. Secrets are not normal domain persistence and are governed by the security specification.

## 4. v1 authoritative state inventory

### 4.1 Learner record

Minimum persisted association:

- canonical `StudentId`;
- application-level display/profile fields strictly required by v1 UX;
- created/updated schema metadata as needed;
- privacy/retention metadata required by policy.

Authentication/account systems are not implied by the presence of `StudentId`.

### 4.2 Authored curriculum identity

Persist enough identity to prove which authored definitions governed learner progress:

- curriculum/course/module/lesson identifiers as applicable;
- content/specification version or immutable content fingerprint;
- policy versions required to replay transitions.

A stored progress record must not silently migrate to semantically different authored content.

### 4.3 Lesson progress

Persist the canonical progress state needed to resume the lesson exactly enough to satisfy the v1 UX and lesson-policy invariants.

Progress must identify:

- learner;
- course/lesson;
- current lifecycle/cursor state;
- governing lesson policy/content version;
- version/concurrency token;
- last committed transition evidence/reference where needed.

### 4.4 Assessment state

Persist assessment attempts required to:

- prevent duplicate/conflicting scoring;
- resume supported in-progress assessment state;
- preserve frozen authored/policy association;
- reproduce accepted evidence.

Raw learner responses may be retained only as required by the privacy/assessment policy; evidence should remain privacy-minimal where possible.

### 4.5 Learning evidence

Accepted `LearningEvidence` is append-only and immutable after commit.

Required properties:

- globally/canonically unique evidence identity;
- exact learner/competency/assessment/attempt associations required by the domain contract;
- observation time and policy/version data already governed by the learning model;
- duplicate-identical replay is idempotent;
- conflicting reuse of an identity fails closed;
- deletion/erasure, if legally/product-required, must be an explicit governed lifecycle operation and not an ordinary update.

### 4.6 Mastery projection

Mastery is derived state.

A persisted mastery projection must include enough information to determine:

- learner/competency;
- exact estimator/policy version;
- evidence horizon or replay anchor from which it was derived;
- projection value;
- concurrency/version token.

If a projection cannot be validated against its evidence horizon/policy, the system must replay or fail safely rather than trusting stale derived state.

### 4.7 Knowledge provenance

Persist v1 knowledge records needed by the released corpus:

- source identity and governance metadata;
- artifact identity/version/hash;
- structural chunk identity/ranges/hash;
- lifecycle/visibility state;
- embedding/profile data only if the selected v1 retrieval architecture requires it;
- provenance needed by context/citation replay.

The original governed artifact or an immutable recoverable equivalent must remain available for any citation path that requires source reconstruction.

### 4.8 Recovery metadata

Persist only the operational state required to make restart safe. Do not persist runtime task handles/tokens.

Examples may include:

- last durably committed workflow/interaction boundary;
- incomplete transaction/recovery marker if the selected storage architecture requires one;
- schema migration state;
- application shutdown cleanliness marker if useful for recovery diagnostics.

The exact set depends on the storage technology ADR and orchestrator recovery model.

## 5. Transaction boundaries

### 5.1 Learning operation atomicity

The existing Phase 3 atomic learning-core contract remains authoritative in intent: one accepted learning operation must not partially persist lesson progress, assessment state, evidence, mastery projection, or operation receipt.

The concrete v1 persistence adapter must provide an atomic commit boundary compatible with that contract.

### 5.2 Knowledge ingestion atomicity

A knowledge ingestion/promotion operation must not expose a partially validated source/artifact/chunk set as active retrievable content.

### 5.3 Cross-domain transaction rule

Do not create distributed/global transactions simply for conceptual neatness. Cross-domain atomicity is required only where the v1 correctness model proves partial commit would create an invalid externally observable state.

The v1 LM Studio invocation cannot be rolled back. The orchestrator must treat provider use as an external side effect and commit local state only according to explicit workflow semantics. The same rule applies to any remote provider separately authorized after v1.

## 6. Concurrency and isolation

v1 must declare a supported concurrency model.

Minimum required behavior:

- concurrent writes to the same learner/lesson/assessment aggregate must not silently overwrite accepted state;
- stale expected versions/snapshots fail closed with a typed conflict;
- safe retry behavior must distinguish duplicate-identical operations from conflicting operations;
- read-modify-write operations use optimistic concurrency or stronger semantics appropriate to the selected store;
- multi-process access is either supported and tested or explicitly unsupported by v1 packaging/runtime rules.

The simplest acceptable v1 model may be single-user/single-process with durable storage, but the limitation must be explicit and mechanically protected where practical.

## 7. Identifiers and time

Canonical domain identifiers remain owned by `nexa-domain`.

Persistence must not substitute storage-generated identities for caller/domain-owned identities where the existing contracts require canonical IDs.

Persistent timestamps use the canonical domain representation. Clock sourcing and capture points used for workflow/persistence semantics must be explicitly defined by the owning orchestrator/data decisions; database wall-clock defaults must not silently become domain authority.

## 8. Schema and migration

Every persistent schema must have an explicit version/evolution strategy.

v1 requirements:

- application startup detects unsupported/newer schema versions and fails safely;
- upgrades apply ordered, idempotent migrations or an equivalently verified replacement strategy;
- migrations preserve canonical IDs and governed replay/provenance semantics;
- migration failure does not leave a silently partially upgraded store;
- downgrade support is not required unless explicitly specified, but backup/restore behavior before irreversible migration must be defined;
- release acceptance tests cover upgrade from every supported predecessor version once predecessors exist.

## 9. Backup, recovery, and corruption

For local-first v1:

- data location must be documented;
- backup/export mechanism must be defined at least for learner-progress-critical data;
- the application must detect unreadable/corrupt stores where practical and fail with diagnostic classification rather than overwrite them;
- recovery must never silently fabricate mastery/progress;
- derived state may be rebuilt from authoritative evidence/content when possible;
- unrecoverable authoritative-state loss must be surfaced explicitly.

## 10. Retention and deletion hooks

The privacy specification owns policy; this data specification must provide mechanisms capable of enforcing it.

The storage design must support:

- discovery of learner-associated records required for export/deletion;
- deletion/retention lifecycle for non-authoritative operational data;
- explicit handling of immutable learning evidence if deletion is required by policy;
- knowledge/content retention independent of learner records;
- log/telemetry retention outside the primary domain store unless intentionally unified.

## 11. Events and outbox decision boundary

The existing event architecture includes typed domain facts but does not yet establish v1 durable publication.

Before selecting an outbox/event-store design, R1 must decide:

1. Which v1 events are authoritative facts whose loss would violate correctness?
2. Which are process-local notifications?
3. Which are operational telemetry?
4. Are any consumers asynchronous/durable in v1?

If authoritative asynchronous publication is required, the local state commit and event publication intent must be atomic through an outbox/equivalent pattern. If not required, do not build durable event infrastructure speculatively.

## 12. Storage technology selection criteria

A later ADR may select the v1 store only after evaluating:

- Rust ecosystem maturity/support;
- transactional atomicity needed by the learning core;
- local-first embedded operation;
- migration support;
- crash consistency;
- concurrency model;
- backup/export/recovery;
- query needs for learner state and governed knowledge;
- vector/search requirements only if justified by measured v1 corpus needs;
- packaging footprint;
- supported OS target;
- operational complexity;
- license/distribution constraints.

One technology may serve multiple state categories, but shared technology does not merge domain ownership.

## 13. Failure vocabulary requirements

Concrete persistence adapters must normalize at least:

- unavailable/open failure;
- unsupported schema version;
- migration failure;
- serialization/validation corruption;
- optimistic-concurrency conflict;
- constraint/integrity conflict;
- storage-capacity/resource failure where observable;
- transaction/commit failure;
- recovery-required/unrecoverable corruption classification.

Errors exposed beyond the data boundary must be content-safe.

## 14. Verification requirements

Before R2/R9 closure, tests must prove:

- clean store creation;
- atomic successful learning commit;
- failure injection at each staged learning write with no partial visible state;
- duplicate-identical replay and conflicting replay behavior;
- optimistic-concurrency conflict handling;
- restart and exact lesson/progress resume;
- mastery replay/rebuild from persisted evidence;
- knowledge provenance/citation reconstruction after restart;
- ingestion atomicity;
- migration success/failure atomicity;
- backup/export and recovery behavior selected for v1;
- retention/deletion mechanisms required by privacy policy;
- corruption detection/fail-safe behavior;
- no secret leakage into normal persisted domain records.

## 15. Decisions required to approve this specification

- v1 supported concurrency/process model;
- exact authoritative event/outbox scope;
- persistence technology ADR;
- backup/export mechanism;
- clock/capture-point ownership relevant to persistent operations;
- privacy-driven retention/deletion requirements;
- whether vector embeddings are persisted for the first released corpus.

## 16. Explicit deferrals

Unless promoted by v1 evidence:

- cloud sync;
- distributed database operation;
- multi-tenant server storage;
- cross-device conflict resolution;
- generalized event sourcing;
- data warehouse/analytics pipeline;
- plugin persistence API;
- external vector database.

## 2026-08-26 ADR-0069 reconciliation

SQLite behind `crates/nexa-storage` is the selected v1 store. One local learner, canonical IDs, migrations, atomic progress/evidence, backup/recovery, and restart/resume remain mandatory. No login, accounts, multi-user administration, or cloud sync is in v1. Both identical clients reach the same authoritative store only through the local runtime API.
