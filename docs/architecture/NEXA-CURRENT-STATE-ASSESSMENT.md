# Nexa Current-State and Documentation-Gap Assessment

Status: Tactical-pause assessment

Baseline commit: `e2345a1bb8825451ea079ff5e350b7765075038a` (PR #109)

## Purpose

This assessment establishes the factual current state of Nexa before further product implementation. It distinguishes architecture intent, specification maturity, contract implementation, runtime integration, concrete adapters, and learner-visible capability.

## Executive finding

Nexa has a substantial tested contract-first foundation, but implementation maturity has exceeded system/specification maturity. The repository proves many deterministic headless slices while several parent specifications remain Reconstructed or Baseline Draft, and multiple production-critical cross-cutting specifications remain only reserved namespaces.

The project therefore has strong local correctness evidence without an equivalent level of system-level completion evidence.

## Current control state

- `main` is the authoritative baseline at the commit above.
- There were no open pull requests or issues at the start of this tactical pause.
- Phase 0 documentation is explicitly not claimed complete by `PROJECT-STATUS.md`.
- `NEXA-ARCH-001` remains Reconstructed.
- Several active subsystem specifications remain Baseline Draft despite substantial downstream ADR and code implementation.
- Phase 5 is in progress and does not provide a complete learner-facing session runtime.

## Capability maturity model used by this assessment

Every capability is evaluated separately across these maturity states:

1. Concept identified
2. Architecture defined
3. Specification approved
4. Contract implemented
5. Runtime integrated
6. Concrete adapter implemented
7. System verified
8. Learner/user accepted
9. Release ready

A lower maturity state must not be reported as a higher one merely because tests are green.

## Capability gap matrix

| Capability | Current evidence | Highest defensible maturity | Critical remaining gap |
|---|---|---|---|
| Canonical domain identities/types | `nexa-domain`, Phase 1 tests | Contract implemented | broader production integration and lifecycle use |
| Events | typed events and in-process bus | Contract implemented | durable event infrastructure, replay, retention, backpressure, production composition |
| Student model/mastery | deterministic ledger/replay | Contract implemented | durable learner persistence, authorization, retention, migration, recovery |
| Pedagogy | deterministic policy | Contract implemented | composition into real learner session and product UX |
| Curriculum/lessons | authored graph contracts and transitions | Contract implemented | authoring/compiler/content pipeline and production persistence |
| Assessment | contracts, scoring, evidence | Contract implemented | authored content pipeline, persistence, product integration |
| Learning core | deterministic headless end-to-end slice | Runtime integrated in test composition | durable UoW/persistence and learner-facing composition |
| Knowledge ingestion | governed in-memory/deterministic slice | Contract implemented | durable repository and production ingestion workflow |
| Retrieval | lexical/vector/hybrid deterministic algorithms | Contract implemented | concrete retrieval/vector infrastructure and operational integration |
| Tutor planning | structural planning/admission contracts | Contract implemented | real inference, semantic validation, end-to-end tutor behavior |
| Model selection/invocation | extensive provider-neutral contracts | Contract implemented | concrete provider/tokenizer/network adapters and operational routing |
| Semantic truth/entailment/safety | deliberately deferred | Concept/spec gap | governed semantic policy and verifiable implementation |
| Speech input | async provider-neutral foundation | Contract implemented | microphone/audio/STT adapter and headless/session composition |
| Speech output | cancellation/control foundations | Partial contract implementation | TTS, audio output, streaming, viseme timing, device integration |
| Avatar/3D | validated runtime plus viewer | Concrete adapter partially implemented | production avatar assets, full behavior/session composition |
| Behavior synchronization | partial control/runtime evidence | Partial runtime integration | complete speech/tutor/avatar synchronization |
| Labs/tools | admission and cancellation contracts | Contract implemented | actual sandbox/tool execution and enforcement |
| Session orchestrator | lifecycle/cancellation foundations | Partial runtime integration | complete workflow composition, timeout/retry/recovery and production dependencies |
| Persistent memory | present in original architecture | Concept only | data model, storage, privacy/retention, integration |
| Event-driven runtime | original architecture plus typed facts | Contract-only/partial | production bus and subsystem composition |
| UX/Desktop | reserved namespace/application shell | Concept only | product UX specification and implementation |
| Authoring | reserved namespace | Concept only | authoring specification, compilers, application |
| Security | isolated bounded decisions | Partial | system security architecture/specification and enforcement |
| Privacy | isolated structural safeguards | Partial | system privacy architecture/specification and semantic minimization policy |
| Observability | reserved namespace | Concept only | specification, telemetry model, runtime instrumentation |
| Analytics | reserved namespace | Concept only | specification and implementation |
| Packaging/update | reserved namespace | Concept only | release/install/update strategy and implementation |
| Deployment | reserved namespace | Concept only | supported deployment models and release pipeline |
| Performance | reserved namespace | Concept only | budgets, benchmarks, gates |
| System testing/acceptance | no unified release acceptance specification | Concept gap | product-level verification and acceptance definition |

## Architecture/documentation gaps

### 1. System architecture maturity trails implementation

The governing Tutor System Architecture remains Reconstructed. Lower-level implementation decisions have continued through many accepted ADRs, meaning increasingly specific implementation behavior is governed by documents that matured faster than the parent system architecture.

### 2. Phase 0 did not actually close

The roadmap requires every active specification to have an ID, status, authority, dependencies, and intended implementation boundary before Phase 0 exits. The registry still records baseline work including structured review, dependency validation, acceptance criteria, conformance links, and explicit promotion from Baseline Draft.

### 3. Subsystem specification maturity is inconsistent with implementation maturity

Several subsystems with extensive code remain governed by Baseline Draft specifications. This creates a risk that ADRs become substitutes for completing parent specifications rather than records of genuine architectural decisions.

### 4. Cross-cutting product specifications are missing

UX, authoring, data, security, privacy, observability, analytics, plugins, API, packaging, deployment, performance, testing, engineering, and governance namespaces are reserved but not approved contracts. Not all are required for an initial release, but several are now on the critical path to a releasable system.

### 5. Documentation status language is inconsistent

The README and later project/roadmap material do not consistently describe the same phase state. Qualified deterministic contract-gate completion has at times been summarized using language that can be mistaken for complete product capability.

## Program-control findings

### Finding A — implementation maturity exceeded specification maturity

The project continued to create accepted lower-level ADRs and implementation slices while higher-level architecture/specification review remained incomplete.

### Finding B — contract/conformance gates were treated as architecture or phase completion

The gates demonstrated deterministic behavior within bounded headless scopes. They did not prove production adapters, integration, user experience, persistence, operations, or release readiness.

### Finding C — horizontal technical depth displaced vertical product integration

Development repeatedly deepened provider-neutral contracts, validation, selection, evidence, cancellation, and propagation before proving a thin complete learner journey with concrete dependencies.

### Finding D — deferred work accumulated without a mandatory debt gate

Deferrals were generally documented accurately, but there was no program rule requiring old deferrals to be retired, promoted, or explicitly re-approved before later phases advanced.

### Finding E — system-level progress was not a mandatory review dimension

PR review was effective at local correctness, scope, CI, and traceability. It did not enforce periodic proof that the program as a whole remained on the shortest viable path to the intended Nexa product.

## Immediate conclusion

Normal feature development must remain paused until Nexa has:

1. an explicit v1 product definition and release boundary;
2. a completed Phase 0 rebaseline for active v1 authorities;
3. a capability maturity matrix tied to v1 acceptance;
4. approved critical cross-cutting specifications;
5. a dependency-driven completion roadmap emphasizing vertical integration;
6. architecture revalidation gates that cannot be satisfied by local CI alone.

## Assessment exit criteria

This tactical-pause assessment is complete when every v1-required capability has an identified authority, current maturity, evidence, gap, owner boundary, and completion gate, and when the resulting finite roadmap can be used to determine whether any proposed implementation increment materially advances Nexa toward release.
