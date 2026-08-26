# Nexa v1 Testing and System Acceptance Specification — Draft

Status: R1 proposal; non-authoritative until registered and approved

## 1. Purpose

Define the evidence required to claim each maturity state and to release Nexa v1. This specification exists to prevent unit/contract/conformance evidence from being reported as integrated product capability.

## 2. Capability maturity evidence

### 2.1 Concept Identified

Evidence:
- capability intent and owner identified;
- relationship to product/release boundary recorded.

This state authorizes research/specification work only.

### 2.2 Architecture Defined

Evidence:
- parent system/subsystem architecture reviewed;
- boundaries, dependencies, data/trust concerns, and quality attributes identified;
- unresolved architecture decisions explicit.

### 2.3 Specification Approved

Evidence:
- observable behavior and failure semantics approved;
- acceptance criteria defined;
- dependencies/versioning/security/privacy implications identified;
- registry status updated.

### 2.4 Contract Implemented

Evidence:
- public/domain contracts implemented;
- unit/contract tests pass;
- malformed/unsupported input behavior tested;
- boundary/dependency rules pass;
- implementation traceability exists.

This does not prove runtime integration or product usability.

### 2.5 Runtime Integrated

Evidence:
- capability composed through its actual runtime/application boundary;
- integration tests cover normal and failure paths;
- lifecycle/ownership/cancellation/commit interactions are exercised.

Scripted dependencies may still exist at this state.

### 2.6 Concrete Adapter Implemented

Evidence:
- release-intended or production-representative concrete dependency exists;
- adapter-specific configuration/failure/security behavior is tested;
- no required release path depends only on scripted/in-memory test doubles.

### 2.7 System Verified

Evidence:
- end-to-end tests traverse the complete release path;
- persistence/restart/recovery and relevant cross-cutting requirements pass;
- security/privacy/performance/accessibility evidence is integrated.

### 2.8 User Accepted

Evidence:
- representative user completes the defined acceptance journey;
- blocking usability/functional findings are resolved or explicitly accepted.

### 2.9 Release Ready

Evidence:
- every v1-required capability meets its required maturity;
- release/package/install/upgrade verification passes;
- no unresolved release-blocking architecture, security, privacy, integrity, or quality finding remains.

## 3. Test layers

Nexa maintains distinct test categories:

- **Unit** — one function/module/policy.
- **Contract** — public/wire/domain contract behavior and validation.
- **Conformance** — implementation against governed protocol/spec invariants.
- **Integration** — multiple real subsystem boundaries composed.
- **Adapter** — concrete storage/model/OS/network/device behavior.
- **System/E2E** — primary learner journey through release composition.
- **Failure injection** — controlled dependency/crash/conflict/error behavior.
- **Migration/recovery** — upgrade, restart, backup/corruption behavior.
- **Security/privacy** — trust/disclosure/redaction/credential/data lifecycle controls.
- **Performance** — measured approved budgets.
- **Accessibility** — supported learner UI baseline.
- **Evaluation/quality** — probabilistic tutor grounding/instructional quality where deterministic proof is not possible.
- **User acceptance** — observable learner outcome and usability.

A test may support more than one layer, but reports must identify what it actually proves.

## 4. Primary v1 end-to-end acceptance scenario

A release candidate must prove at minimum:

1. Install Nexa on a clean supported environment.
2. Configure the supported v1 model path without placing credentials in ordinary config/log/domain storage.
3. Launch successfully with an empty/new durable store.
4. Select/start the released course and lesson.
5. Submit learner text through the actual learner application.
6. Load learner/lesson state from the concrete store.
7. Produce learning/pedagogy context.
8. Retrieve governed knowledge using the concrete v1 knowledge persistence/retrieval path.
9. Invoke the concrete v1 model adapter.
10. Admit and quality-check the response.
11. Present the response in the real learner UI.
12. Complete an assessment/practice action.
13. Atomically persist evidence/mastery/progress.
14. Exit the application.
15. Relaunch.
16. Resume the exact accepted learner progress without duplicated evidence or state loss.
17. Complete the lesson acceptance outcome.

The release path must not substitute scripted providers or in-memory persistence for required concrete adapters except in separately classified non-release test suites.

## 5. Required failure-path system tests

At minimum exercise:

- invalid/corrupt persistent state;
- unsupported schema/migration failure;
- storage unavailable/commit failure;
- optimistic concurrency conflict;
- invalid/missing governed course content;
- retrieval failure/empty governed result where relevant;
- model configuration/authentication failure;
- model unavailable/dependency error;
- model timeout;
- malformed/structurally invalid model output;
- grounding/quality rejection;
- learner cancellation/interruption;
- shutdown during active but uncommitted interaction;
- restart after unclean termination if recoverable scenario is supported;
- observability sink failure;
- privacy/security rejection of unauthorized remote disclosure.

Expected learner-visible and diagnostic outcomes must be asserted separately.

## 6. Persistence acceptance

Verify with the concrete v1 store:

- atomic learning commit;
- failure injection at each state-update stage;
- idempotent duplicate replay;
- conflicting replay rejection;
- mastery rebuild/replay;
- lesson resume;
- knowledge provenance after restart;
- migration success/failure behavior;
- backup/recovery mechanism selected for v1;
- retention/deletion mechanism required by privacy policy.

## 7. Model/tutor acceptance

### Deterministic evidence

Continue to verify:

- request/response identity and version binding;
- prompt compilation integrity;
- capacity/token accounting where applicable;
- provider selection/configuration policy;
- structural admission;
- citation/reference association;
- redacted errors/evidence.

### Evaluation evidence

v1 must also maintain a governed evaluation set for the first released course covering:

- factual grounding against provided source material;
- citation support/fidelity;
- instructional correctness;
- assessment-answer protection;
- refusal/restriction behavior;
- prompt-injection or hostile-source cases relevant to the corpus;
- response usefulness/readability appropriate to target learners.

Evaluation method, model/version/configuration, sample set/version, thresholds, and known nondeterminism must be recorded. One passing model sample is not sufficient evidence.

## 8. Security/privacy acceptance

Verify at system level:

- secret storage/config path;
- no secret leakage in logs/domain files/errors;
- remote disclosure cannot bypass configured/privacy policy;
- protected assessment material cannot leak through retrieval/prompt path;
- sensitive raw content absent from ordinary diagnostics;
- deletion/reset behavior matches privacy claims;
- corrupt/reassociated identity/evidence inputs fail closed on critical boundaries;
- package/update provenance mechanism works.

## 9. Performance acceptance

Use representative release hardware/configuration/corpus and the approved performance specification.

Record:

- environment;
- model/provider mode;
- data/corpus size;
- test version;
- warm/cold distinction;
- median and tail metrics where relevant;
- resource usage;
- pass/fail against budgets.

Performance claims without reproducible measurement do not pass the gate.

## 10. Accessibility acceptance

The released learner UI must be reviewed against the UX/accessibility requirements selected for the supported platform, including at minimum keyboard navigation where applicable, readable focus/status/error states, text scaling/layout resilience where practical, and alternatives for information not conveyed solely by animation/audio.

Required speech and animated 2D behavior must preserve equivalent access to core instructional content through text, captions/transcripts, reduced-motion/static presentation, and keyboard controls.

## 11. User acceptance

Before Release Ready, a representative user must be able to complete the v1 primary learner journey without developer intervention beyond documented installation/configuration.

Record:

- release-candidate version;
- environment;
- acceptance script/outcomes;
- blocking findings;
- accepted limitations;
- final disposition.

## 12. CI and release gates

PR CI remains necessary for local correctness but is not the release gate.

Recommended separation:

- **PR gate:** format/check/lint/unit/contract/conformance/boundary tests proportional to change.
- **integration gate:** concrete adapter and composed runtime tests.
- **release-candidate gate:** E2E, migration/recovery, security/privacy, performance, packaging, accessibility.
- **user acceptance gate:** primary learner journey on the release candidate.

## 13. Status reporting rule

Project/roadmap/registry/status artifacts must report the highest demonstrated maturity state and its scope.

Examples:

- `Contract Implemented — deterministic headless adapter`
- `Runtime Integrated — scripted model provider`
- `Concrete Adapter Implemented — remote provider X`
- `System Verified — primary v1 text journey`

Avoid unqualified `Complete` where multiple maturity meanings are possible.

## 14. Release blocking rule

A release is blocked when:

- a v1-required capability is below its required maturity;
- a required system acceptance test is missing or failing;
- required evidence is attached to a different code/config/model/content version;
- critical security/privacy/data-integrity findings remain unresolved;
- supported installation/upgrade path is unverified;
- product acceptance criteria cannot be reproduced.

## 15. Post-v1 testing scope

Testing for optional/post-v1 capabilities should be added when those capabilities are promoted. Their foundation tests may remain green without making them part of the v1 release acceptance surface.

## 2026-08-26 ADR-0069 reconciliation

The release E2E must run through both identical clients on the same Windows PC, LM Studio, SQLite restart/resume, bundled speech, and synchronized semantic 2D behavior. It distinguishes browser-on-loopback evidence from hosted/remote evidence. Speech/avatar accessibility, interruption, timing, package/resource, and graceful-failure checks are required. Candidate spike evidence is suitability evidence only and cannot establish Concrete Adapter Implemented, System Verified, User Accepted, or Release Ready.
