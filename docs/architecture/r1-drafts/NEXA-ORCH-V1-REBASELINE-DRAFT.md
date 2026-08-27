# NEXA-ORCH-001 v1 Rebaseline Requirements — Draft

Status: R1 proposal; intended to mature/reconcile the existing Baseline Draft, not create a competing parent specification

## 1. Purpose

The existing `NEXA-ORCH-001` and ADR-0051 through ADR-0067 provide substantial lifecycle, identity, cancellation, task-ownership, and bounded application-control foundations. This draft identifies the additional/reconciled requirements needed for the orchestrator to govern the actual Nexa v1 learner journey.

The final approved work should update or supersede the existing parent specification through the registry. These requirements are not a new parallel orchestrator authority.

## 2. v1 orchestrator mission

The orchestrator coordinates one learner session and its interaction workflows across the owning subsystems while preserving their reasoning boundaries.

It must make the primary learner journey executable and recoverable:

`learner input -> state load -> learning context -> governed retrieval -> tutor/model -> admitted response -> learner action/assessment -> learning-core commit -> progress output`

It owns sequencing, dependency lifetime, cancellation, timeout, retry/recovery coordination, and workflow-level observability. It does not own student mastery, pedagogy rules, lesson transitions, assessment scoring, retrieval ranking, tutor semantic planning, storage semantics, renderer behavior, or provider-specific inference logic.

## 3. Session lifecycle

The existing closed lifecycle contracts remain useful. v1 must additionally define application-visible semantics for:

- create/start session;
- resume session from durable learner/course/lesson state;
- idle/active interaction boundaries;
- orderly session completion;
- user-requested cancellation/interruption;
- dependency-caused failure;
- recoverable vs terminal session failure;
- graceful application shutdown;
- restart after unclean shutdown.

The orchestrator must not persist task/runtime implementation objects as domain state.

## 4. Interaction workflow

Every tutor/assessment interaction must have an exact canonical workflow identity associated with the current session and relevant learner/course/lesson identities.

A v1 interaction workflow has explicit stages:

1. **Accepted** — learner/application input passed host validation.
2. **StateLoaded** — required durable state was loaded/validated.
3. **LearningContextReady** — lesson/student/pedagogy state required for the action is available.
4. **KnowledgeReady** — governed retrieval/context is complete when required.
5. **TutorRequestReady** — trusted prompt/model request is built.
6. **ModelCompleted** — configured provider returned a bounded outcome or normalized failure.
7. **TutorAdmitted** — structural and required v1 quality gates passed.
8. **Presented** — response/action was delivered to the application boundary.
9. **LearningCommitReady** — assessment/practice outcome and resulting learning state are staged when applicable.
10. **Committed** — authoritative local state commit completed.
11. **Completed** — workflow is terminal and no required owned work remains.

The final state model may compress stages, but the specification must preserve the relevant ordering/commit invariants and observable failure boundaries.

## 5. State-load boundary

Before invoking domain policies that depend on persisted state, the orchestrator must load and validate the exact required state through owning repositories/UoW services.

Requirements:

- reject unsupported/corrupt schema/domain state;
- preserve exact learner/course/lesson association;
- distinguish not-found/new state from corrupt/unreadable state;
- carry version/concurrency tokens needed by the learning-core commit;
- do not invoke remote dependencies merely to discover that local required state is invalid.

## 6. Learning context composition

The orchestrator calls existing owning boundaries rather than reproducing policy:

- lesson/curriculum for current activity and authored routes;
- student for evidence/mastery projection;
- pedagogy for instructional decision;
- assessment for governed attempt/scoring behavior;
- learning-core for atomic composition when a learner action changes governed learning state.

Any cached/derived context must be associated with the exact state/version from which it was built.

## 7. Knowledge/tutor sequence

For tutor generation the orchestrator must enforce this conceptual ordering:

1. validate local host/interaction state;
2. construct retrieval request from approved lesson/tutor needs;
3. retrieve governed material;
4. assemble context/citations or required evidence;
5. construct trusted tutor planning/prompt authority;
6. apply privacy/disclosure policy appropriate to configured provider;
7. perform model/token-capacity preflight;
8. invoke exactly the selected/configured provider according to v1 policy;
9. apply structural admission;
10. apply required v1 grounding/quality/safety acceptance;
11. return accepted semantic tutor output to the application and optional downstream embodiment adapters.

A failure before provider invocation must not consume provider work where existing contracts establish this guarantee.

## 8. Concrete dependency ownership

The primary application composition root constructs concrete adapters, but the orchestrator owns their lifecycle/use during an interaction.

v1 release-critical dependencies include at least:

- durable data/UoW service;
- governed knowledge service/repository;
- one concrete model-provider adapter;
- required tokenizer/capacity service for that provider path;
- observability sink/context;
- learner-facing application output boundary.

Required v1 dependencies:

- bundled speech input/output through the owned speech boundary;
- synchronized semantic 2D behavior through the NBP/avatar boundary.

Labs/tools are conditional in the general architecture and omitted from v1. The specification must state the accessible text, caption, reduced-motion/static, and failure degradation available when a required speech or embodiment dependency is temporarily unavailable; degradation does not make that dependency optional for release acceptance.

## 9. Timeout policy

Every external or potentially blocking dependency used in the primary journey must have a bounded timeout policy owned by an explicit layer.

The orchestrator owns workflow-level deadlines and maps dependency timeout outcomes into workflow state. Provider adapters may enforce lower-level request timeouts but may not silently choose workflow retry/recovery policy.

R1 approval must define timeout categories for at least:

- durable storage operations;
- retrieval if asynchronous/external;
- model invocation;
- required speech dependencies and their cancellation ownership; tool dependencies remain absent.

Timeout diagnostics must be content-safe.

## 10. Retry policy

Retry is not a generic default.

The specification must classify operations by retry safety:

- **Pure/local deterministic** — may be recomputed freely when inputs are unchanged.
- **Idempotent durable read** — retry allowed according to bounded policy.
- **Idempotent/identity-keyed durable write** — retry only when the owning data contract proves duplicate safety.
- **Remote model invocation** — no automatic retry unless the v1 policy explicitly defines identity, duplicate cost/side effect, and response-selection semantics.
- **Assessment/evidence commit** — retry only through existing operation/evidence idempotency/concurrency contracts.

Every retry has a bounded attempt count and reason classification. Infinite/background retry is prohibited.

## 11. Recovery policy

Recovery distinguishes:

1. failure before any irreversible external action;
2. remote provider action completed but no local authoritative state change occurred;
3. learner-visible response was presented but subsequent learning commit failed;
4. local commit succeeded but presentation/telemetry failed;
5. application/process terminated during active workflow.

The specification must define the safe disposition for each v1-relevant class.

General principles:

- never fabricate a successful commit;
- never duplicate accepted learning evidence under a new identity to hide uncertainty;
- preserve enough correlation to diagnose uncertain outcomes;
- derived/replayable state may be recomputed;
- remote model generation may be discarded and regenerated only under explicit policy;
- startup recovery must prefer known durable commit boundaries over transient runtime state.

## 12. Cancellation and interruption

ADR-0051 through ADR-0067 remain foundation evidence.

v1 must distinguish:

- cancellation of Nexa-owned task/future;
- dependency acceptance of a cancellation request;
- proof that an external provider/device/process stopped;
- operations declared non-cancellable.

The application must not tell the learner that external work stopped unless the concrete adapter evidence supports that claim.

Cancellation must never leave a partial learning-core commit visible.

## 13. Commit boundary and learner-visible output

The orchestrator must define when a learner-facing action becomes authoritative.

For a tutor explanation that changes no learning state, presentation may complete without a learning-state commit.

For assessment/practice actions that change evidence/mastery/progress:

- compute/stage the governed learning result;
- commit according to the data/UoW contract;
- report progress as committed only after durable success;
- if response/presentation occurs before commit, the UX must not imply durable progress until commit succeeds.

## 14. Event and observability semantics

The orchestrator attaches stable correlation to primary operations using existing canonical IDs.

It must emit or make available content-safe operational evidence for:

- session/workflow lifecycle;
- state load/commit;
- retrieval;
- model invocation classification/timing;
- tutor admission/quality outcome;
- retry/timeout/cancellation/recovery;
- application presentation outcome;
- shutdown/startup recovery.

Authoritative domain events remain owned by their domain/event specifications; observability is not permitted to redefine domain truth.

## 15. Application error contract

The orchestrator/application boundary must expose a bounded error model sufficient for UX decisions, such as:

- configuration required/invalid;
- local data unavailable/corrupt/migration required;
- course/lesson unavailable/invalid;
- model provider unavailable/authentication/configuration failure;
- retrieval/content unavailable;
- tutor response rejected/quality gate failed;
- timeout;
- cancelled/interrupted;
- concurrency/conflict/retry required;
- unrecoverable internal failure.

Internal dependency details may remain diagnostic-only. Raw sensitive content must not appear in error messages.

## 16. Graceful shutdown

On normal application shutdown the composition must:

- stop accepting new learner interactions;
- cancel or complete active owned work according to bounded policy;
- await required task termination within a defined shutdown budget;
- commit no partial learning operation;
- flush/close durable state according to the storage adapter contract;
- flush required observability without making telemetry a shutdown blocker beyond its configured budget;
- exit with any unresolved recovery condition recorded safely.

## 17. Restart/resume

At startup:

- validate configuration and persistent schema;
- recover/migrate according to data policy;
- detect any recovery marker/unclean shutdown if the storage design uses one;
- reconstruct session capability from durable learner/course/lesson state rather than prior runtime task state;
- offer/resume the exact supported learner activity according to the UX/lesson rules.

## 18. Required speech/avatar composition and deferred labs

Labs/tools remain omitted from v1. Speech and synchronized semantic 2D behavior are required parts of the primary journey, while accessible text and graceful degradation remain available.

Required speech and animated 2D behavior attach through their existing semantic/control boundaries. The orchestrator owns interaction sequencing/cancellation but not provider or renderer semantics. Each requires concrete-adapter and system-level acceptance, not only control-contract tests. Labs/tools remain absent.

## 19. Verification requirements

The complete v1 orchestrator path must be proven by system/integration tests covering:

- happy-path learner interaction with real production-equivalent storage and model adapters;
- assessment/progress commit and exact restart/resume;
- state-load corruption/version failure before remote invocation;
- retrieval failure;
- model provider authentication/unavailable/timeout/error;
- structural admission rejection;
- quality/grounding rejection;
- storage commit failure with no false learner progress;
- concurrency conflict and safe retry behavior;
- user cancellation before/during remote work;
- process/task failure normalization;
- clean shutdown and unclean restart recovery;
- content-safe correlated diagnostics.

## 20. Existing ADR reconciliation

During parent-spec maturity review, ADR-0051 through ADR-0067 must each be classified as:

- retained and required by v1;
- retained foundation but post-v1/conditional;
- requires wording/parent clarification;
- superseded only through an explicit accepted later decision.

No accepted ADR is silently rewritten by this draft.

## 21. Approval blockers

Before the orchestrator parent can be approved for v1, R1 must decide:

- data/UoW concrete behavioral contract;
- configured model-provider policy and adapter boundary;
- timeout values/categories or configuration method;
- retry classification and maxima;
- user-visible recovery behavior;
- authoritative event/outbox scope;
- speech/avatar/lab v1 disposition;
- system-level test/acceptance contract.

## 2026-08-26 ADR-0069 reconciliation

The v1 workflow composes required speech and semantic 2D behavior, not optional branches, while preserving text accessibility and graceful degradation. It serves both identical clients through one versioned loopback API, owns cancellation/reconnect coordination, persists atomic learning state, and invokes the narrow LM Studio adapter. Labs/tools remain absent.
