# Nexa v1 System Architecture

Status: Proposed rebaseline architecture

## 1. Purpose

This document proposes the system architecture required to deliver the Nexa v1 product boundary. It is derived from the reconstructed `NEXA-ARCH-001`, the current implementation through PR #109, accepted ADR evidence, and the v1 release definition.

It does not silently replace `NEXA-ARCH-001`. Approval of this architecture must be accompanied by an explicit registry/status decision that promotes, revises, or supersedes the reconstructed authority.

## 2. Architectural objective

Nexa v1 is a local-first adaptive tutor application in which one learner can complete and resume a governed lesson using:

- a real learner-facing desktop application;
- durable learner and lesson state;
- governed curriculum, assessment, pedagogy, and knowledge;
- one concrete configured model-provider path;
- structured, admitted tutor output;
- observable and recoverable session orchestration.

The architecture must preserve the existing core rule: tutor intelligence produces semantic instructional/communicative intent; renderer/avatar implementation remains downstream and replaceable.

## 3. v1 system context

```text
                    ┌───────────────────────────┐
                    │          Learner          │
                    └─────────────┬─────────────┘
                                  │ text / UI
                                  ▼
                    ┌───────────────────────────┐
                    │   Nexa Desktop App (v1)   │
                    │   primary composition root │
                    └─────────────┬─────────────┘
                                  │
                                  ▼
                    ┌───────────────────────────┐
                    │    Session Orchestrator   │
                    │ lifecycle / workflow /    │
                    │ cancellation / recovery   │
                    └──────┬──────┬──────┬─────┘
                           │      │      │
              ┌────────────┘      │      └──────────────┐
              ▼                   ▼                     ▼
      ┌───────────────┐   ┌───────────────┐    ┌────────────────┐
      │ Learning Core │   │ Tutor Engine  │    │  Observability │
      │ lesson/student│   │ prompt/admit  │    │ logs/correlation│
      │ pedagogy/asmt │   └───────┬───────┘    └────────────────┘
      └───────┬───────┘           │
              │                   ├─────────────┐
              │                   ▼             ▼
              │          ┌──────────────┐ ┌───────────────┐
              │          │ Knowledge/RAG│ │ Model Adapter │
              │          └──────┬───────┘ │ concrete v1   │
              │                 │         │ provider      │
              │                 │         └──────┬────────┘
              │                 │                │
              └──────────┬──────┴────────────────┘
                         ▼
                ┌───────────────────┐
                │ Durable Data Layer│
                │ learner/progress  │
                │ content/provenance│
                │ migration/recovery│
                └───────────────────┘
```

Optional/conditional v1 adapters attach outside the core release path:

```text
Session/Tutor semantic outputs
        ├── Speech adapter (conditional)
        ├── Avatar/Behavior adapter (conditional)
        └── Lab/Tool adapter (conditional)
```

## 4. Primary composition root

### Decision

v1 requires one actual learner-facing application composition root. `apps/nexa-headless` remains a test/integration composition and MUST NOT be treated as the released product application.

The existing reserved `apps/nexa-desktop` boundary is the preferred conceptual owner for the v1 learner application unless a later explicit decision selects another app boundary.

### Responsibilities

The application composition root owns only assembly and user-facing adaptation. It must:

- construct/configure concrete adapters;
- start/resume a learner session;
- translate UI actions into orchestrator requests;
- present tutor/lesson/assessment outcomes;
- present recoverable/terminal failures;
- initiate graceful shutdown;
- avoid absorbing domain reasoning owned by subsystems.

## 5. Session orchestrator

### Existing foundation

`nexa-orchestrator` and `nexa-orchestrator-runtime` already provide substantial lifecycle, identity, cancellation-planning, task-ownership, and selective-cancellation foundations.

### v1 responsibility

The v1 orchestrator must compose the actual learner interaction, not merely cancellation controls.

For each interaction it owns the workflow sequence and dependency lifetime:

1. validate session/workflow identity;
2. load authoritative learner/lesson state references;
3. obtain the current lesson objective/activity;
4. obtain student mastery projection;
5. obtain pedagogy decision;
6. retrieve governed knowledge as required;
7. assemble tutor context and prompt;
8. invoke the concrete v1 model adapter;
9. admit/validate the structured response;
10. present output through the application;
11. process assessment/practice evidence when applicable;
12. atomically persist resulting learner/lesson state;
13. emit content-safe correlated operational evidence.

It owns coordination, not subsystem reasoning.

### Failure policy

The orchestrator specification must define:

- dependency timeouts;
- which failures are retryable;
- maximum retry attempts where allowed;
- cancellation/interruption semantics;
- state commit points;
- compensation/recovery where a workflow fails after external dependency use;
- startup recovery and incomplete-workflow disposition;
- graceful shutdown.

## 6. Learning core

The existing deterministic Phase 3 implementation remains the policy foundation.

v1 adds the production boundaries it intentionally deferred:

- durable unit of work;
- concurrency/isolation semantics;
- migration/version compatibility;
- learner authorization/ownership assumptions;
- retention/deletion behavior;
- restart/replay behavior;
- event/outbox semantics if events are part of the authoritative commit.

The learning core remains the authority for atomic composition of assessment, evidence, mastery, pedagogy, and lesson progression. The application and orchestrator must not duplicate those rules.

## 7. Tutor engine and model execution

### Existing foundation

`nexa-tutor` provides strong provider-neutral contracts for prompt compilation, model descriptors/requests/responses, selection, authorization/filtering, tokenization evidence, usage reconciliation, output admission, and structured planning.

### v1 simplification

The release path needs **one concrete provider path first**.

The architecture therefore separates:

- **provider-neutral domain contracts**, which remain reusable;
- **v1 provider selection policy**, which may be a simple explicit configured provider/model;
- **concrete adapter**, which owns provider SDK/API/network behavior, credentials, tokenizer integration, timeout mapping, and normalized failures.

Dynamic routing, cost/latency optimization, fallback chains, and multiple provider strategies are post-v1 unless explicitly promoted.

### Semantic quality boundary

Structural admission is necessary but not sufficient. v1 requires an evaluation/quality layer that establishes release-appropriate evidence for:

- grounding against supplied knowledge context;
- citation fidelity;
- instructional appropriateness;
- refusal/assessment-protection behavior;
- relevant prompt-injection/data-boundary defenses.

These may combine deterministic checks and model evaluation evidence, but the evidence types and acceptance thresholds must be specified.

## 8. Knowledge and retrieval

The existing `nexa-knowledge` deterministic ingestion/retrieval/context/citation logic remains reusable.

v1 requires a durable knowledge repository and a production ingestion workflow for the content formats used by the first released course.

The storage/retrieval implementation should be chosen from measured corpus and latency requirements. A separate vector database is not an architectural requirement if a simpler local durable implementation satisfies the v1 performance and quality gates.

Knowledge provenance, source/version identity, exposure policy, context-package identity, and citation evidence must survive restart and remain traceable.

## 9. Durable data architecture

### Authoritative state categories

The data specification must distinguish at minimum:

1. **Learner identity/profile state** required by v1.
2. **Immutable learning evidence** used to derive mastery.
3. **Derived mastery/projection state** and replay/version metadata.
4. **Course/lesson progress** and authored content version association.
5. **Assessment attempts/results** required for progression/evidence.
6. **Knowledge source/artifact/chunk provenance** required for retrieval/citation replay.
7. **Operational/session metadata** required for safe recovery and diagnostics.
8. **Configuration/secrets references**, stored separately according to security policy.

### Ownership rule

Subsystems own their domain state and persistence contracts. The data adapter may provide shared storage technology, transaction support, migrations, and backup mechanisms, but it must not become the owner of domain semantics.

### Commit rule

The primary learner journey must define exactly which state transitions are atomic. A learner must not observe mastery/progress that cannot be reproduced from committed evidence, and restart must not duplicate or lose accepted learner work.

## 10. Event architecture

The original architecture intends event-driven integration, and typed event contracts already exist.

For v1, event usage must be deliberately scoped rather than assumed.

Two categories must be separated:

- **authoritative domain facts**, whose durability/ordering/replay semantics are part of correctness;
- **operational notifications/telemetry**, which may be lossy according to the observability specification.

If authoritative events participate in state commits, durable event/outbox semantics are required. If v1 does not need an authoritative cross-process event bus, the architecture should explicitly narrow the scope instead of implementing one speculatively.

## 11. Security architecture

v1 security trust boundaries include:

```text
Learner
  |
Desktop process
  |-- local governed data
  |-- local configuration
  |-- credential store / secret reference
  |
  +---- optional remote model boundary ----> Model provider
```

If labs/tools are included, they introduce an additional untrusted execution boundary and require a separate sandbox/enforcement architecture before release.

Minimum v1 security decisions:

- credential storage and retrieval;
- network destination control;
- TLS/provider trust assumptions;
- least-privilege filesystem access;
- content/secret diagnostic redaction;
- update/package provenance;
- local user/learner ownership assumptions;
- threat model for remote model disclosure.

## 12. Privacy architecture

v1 must classify learner and instructional data by whether it may:

- remain local only;
- appear in model prompts;
- cross a remote provider boundary;
- appear in logs/diagnostics;
- be retained and for how long;
- be exported/deleted.

Remote inference is opt-in/configured behavior subject to explicit disclosure policy. Existing structural prompt filtering is reusable evidence but does not replace the v1 privacy policy.

## 13. Observability

Every primary interaction receives stable correlation through session/workflow/invocation identities already present in the domain model.

Observability must provide content-safe evidence for:

- application startup/configuration;
- session/workflow transitions;
- storage operations/failures;
- retrieval operations/failures;
- model invocation timing/outcome classification;
- admission/quality-gate outcome classification;
- cancellation/timeout/retry/recovery;
- shutdown/restart recovery.

Learner text, knowledge content, prompts, raw model output, and secrets must not appear in ordinary diagnostics unless an explicitly governed diagnostic mode permits it.

## 14. UX boundary

The v1 UX must make the system state understandable without exposing internal architecture.

Minimum states:

- first launch/configuration;
- course selection;
- lesson active;
- tutor thinking/working;
- tutor response;
- assessment/practice interaction;
- progress update;
- recoverable dependency failure;
- offline/unavailable provider state;
- resume after restart;
- terminal configuration/data failure.

The UX specification owns presentation and accessibility behavior; it must not own learning, tutor, or orchestration policy.

## 15. Conditional embodiment/speech/labs

### Speech

Speech is not required for the text-first walking skeleton. If promoted into v1, it must include real microphone/STT and/or TTS/audio adapters, privacy handling, interruption, and synchronization—not only provider-neutral contracts.

### Avatar

The existing 3D/viewer work is valuable and should remain intact. If promoted into v1, the released application must connect admitted semantic `BehaviorIntent` through NBP/avatar boundaries to a real asset/runtime and verify synchronization with tutor/speech output.

### Labs/tools

Contract-only admission/cancellation does not satisfy a v1 lab capability. Promotion requires actual sandbox/tool execution, enforcement, authorization, observation, and recovery.

## 16. Packaging and deployment

The first supported release target must be explicit. The packaging architecture must own:

- reproducible application build;
- installer/package format;
- application/data/config locations;
- provider configuration;
- secure credential integration;
- upgrade and data migration;
- rollback/recovery behavior where practical;
- uninstall and retained-data behavior;
- version/provenance reporting;
- third-party license/asset provenance.

## 17. Primary learner sequence

```text
Learner
  -> Desktop: submit text / begin activity
Desktop
  -> Orchestrator: start interaction workflow
Orchestrator
  -> Durable Data: load session/learner/lesson state
Orchestrator
  -> Learning Core: derive current learning context / pedagogy
Orchestrator
  -> Knowledge: retrieve and assemble governed context
Orchestrator
  -> Tutor: compile provider-neutral prompt/request
Tutor Adapter
  -> Concrete Model Provider: invoke one configured model
Concrete Model Provider
  -> Tutor: bounded response
Tutor
  -> Admission/Quality: validate structured + release quality gates
Orchestrator
  -> Desktop: present tutor response
Learner
  -> Desktop: answer / assessment action
Desktop
  -> Orchestrator
Orchestrator
  -> Learning Core: score/evidence/mastery/route
Learning Core
  -> Durable Data: atomic commit
Orchestrator
  -> Desktop: present progress / next step
```

Every arrow in the v1 release path must eventually have a concrete implementation or explicitly in-process composition, not only an interface contract.

## 18. Architecture verification strategy

The v1 architecture is considered exercised only when tests prove at least:

1. clean install/configuration on the supported target;
2. one real learner lesson through the complete sequence above;
3. real model adapter execution in a production-representative test mode;
4. durable commit and restart/resume;
5. retrieval/citation provenance after restart;
6. model/storage/retrieval failure behavior;
7. cancellation/timeout/recovery without state corruption;
8. content-safe operational correlation;
9. security/privacy controls at the actual remote/local boundaries;
10. user acceptance of the primary journey.

## 19. Architecture decisions still required before R0/R1 exit

This proposal deliberately does not silently choose:

- exact desktop UI framework;
- exact durable storage technology;
- first concrete model provider/model;
- local-only versus remote-capable default provider posture;
- exact secrets-store integration;
- exact packaging format/supported OS target;
- whether authoritative domain events require a durable outbox in v1;
- whether speech, avatar, or labs are included in v1;
- semantic-quality evaluation implementation/thresholds;
- exact performance budgets.

Those decisions must be made from the v1 product boundary and recorded in the owning specifications/ADRs.

## 20. Rebaseline effect on existing implementation

Existing Phase 1–5 code is not discarded. It is reclassified as reusable foundation evidence according to actual maturity.

No existing ADR is silently invalidated. During rebaseline each accepted ADR must be checked against this architecture and classified as:

- consistent and retained;
- retained but post-v1;
- requires parent-spec clarification;
- superseded by a reviewed later decision.

## 21. Proposed architecture exit statement

This architecture may become the v1 system authority only after review confirms that:

- it faithfully preserves the intended Nexa tutor mission;
- its v1 boundary is accepted;
- subsystem ownership is compatible with implemented crate boundaries;
- critical data/security/privacy/operations concerns have owning specifications;
- conditional capabilities are explicitly classified;
- the resulting completion roadmap is finite and testable.
