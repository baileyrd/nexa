# Nexa R1 Implementation Baseline

Status: Approved v1 implementation baseline; reconciled by ADR-0069
Date: 2026-08-26
Parent architecture: `NEXA-ARCH-002`
Controlling owner decision: ADR-0069; ADR-0068 applies only in non-conflicting scope

## 1. Purpose

This document closes the R1 review to the level required to govern the G0–G9 v1 route. It intentionally does not attempt to finish every long-term Nexa specification before vertical integration begins.

The R1 drafts produced during the tactical pause are adopted as governing v1 supplements subject to the explicit decisions and scope limits below. Their `DRAFT` filenames and original proposal headers are historical working labels; this approval record is the status authority for their v1-applicable requirements until they are editorially relocated/renamed.

## 2. Approved R1 supplements

The following documents are approved as governing requirements where they apply to the owner-approved G0–G9 v1 path:

- `r1-drafts/NEXA-DATA-PERSISTENCE-V1-DRAFT.md`
- `r1-drafts/NEXA-SECURITY-V1-DRAFT.md`
- `r1-drafts/NEXA-PRIVACY-V1-DRAFT.md`
- `r1-drafts/NEXA-OBSERVABILITY-V1-DRAFT.md`
- `r1-drafts/NEXA-ORCH-V1-REBASELINE-DRAFT.md`
- `r1-drafts/NEXA-DOM-EVT-V1-REBASELINE-DRAFT.md`
- `r1-drafts/NEXA-LEARNING-V1-REBASELINE-DRAFT.md`
- `r1-drafts/NEXA-TUTOR-KNOWLEDGE-V1-REBASELINE-DRAFT.md`
- `r1-drafts/NEXA-UX-V1-DRAFT.md`
- `r1-drafts/NEXA-TESTING-ACCEPTANCE-V1-DRAFT.md`
- `r1-drafts/NEXA-PERFORMANCE-V1-DRAFT.md`
- `r1-drafts/NEXA-PACKAGING-DEPLOYMENT-V1-DRAFT.md`
- `r1-drafts/NEXA-CONTENT-RELEASE-V1-DRAFT.md`

They supplement, rather than erase, the original subsystem specifications and accepted ADRs. If an R1 supplement conflicts with an older Baseline Draft on v1 behavior, `NEXA-ARCH-002`, this baseline, ADR-0069, and any later accepted ADR control that behavior. ADR-0068 remains applicable only where it does not conflict. Any other direct conflict with an accepted ADR requires explicit resolution rather than silent reinterpretation.

## 3. R1 decisions resolved for v1

### 3.1 Supported process/concurrency model

v1 supports one learner and one authoritative local Rust runtime operating on one local data store at a time; both clients connect to that runtime.

The storage adapter must still use explicit optimistic version/concurrency checks on mutable aggregates so stale state cannot silently overwrite accepted state. Multi-process simultaneous writers are unsupported in the v1 route and must fail safely through store locking/open policy rather than being treated as supported concurrency.

### 3.2 Persistence technology

SQLite through `rusqlite` in `crates/nexa-storage` is approved by ADR-0069.

- Explicit transactions implement the learning-core atomic boundary.
- Canonical domain IDs remain authoritative.
- Ordered schema migrations are required before schema evolution enters the accepted path.
- Database clock defaults do not become domain time authority.
- The orchestrator/application supplies canonical timestamps at the defined operation boundaries.

### 3.3 Authoritative events/outbox

The v1 route has no required durable asynchronous event consumer. Therefore a durable event broker/outbox is not required for the v1 route.

Typed domain events may be emitted in process and used for integration/observability. If a later explicitly authorized gate introduces a durable asynchronous consumer whose fact loss would violate correctness, that decision reopens the outbox requirement.

### 3.4 Backup/recovery

For the v1 route, backup/recovery acceptance requires a consistent copy/export of the local SQLite database while the application is quiesced or through the selected SQLite-safe backup mechanism.

The system must document the data location and must not overwrite a corrupt authoritative database with a fresh empty database without an explicit learner-visible recovery choice.

### 3.5 Privacy posture

The v1 route is local-inference-first:

- learner state and governed course/knowledge content remain local;
- no remote model disclosure is part of the v1 path;
- ordinary logs contain no raw learner text, prompt, knowledge source content, raw model output, assessment response, or secret;
- learner data persists until explicit reset/removal or application uninstall behavior defined by packaging policy;
- a learner-data reset must remove/reinitialize learner-specific state without silently altering shipped governed course assets.

Remote disclosure requirements remain approved requirements for future remote-provider work but do not block the v1 route.

### 3.6 Security posture

The v1 route uses a local model server and therefore does not require provider credentials.

- the Nexa business API binds to loopback; LM Studio remains a separately configured local trust boundary.
- Endpoint/model paths are trusted configuration.
- The application runs as an ordinary user after installation.
- Normal runtime does not require administrator/root privilege.
- No arbitrary tool/lab process execution is part of the v1 route.

If a remote provider is later added on Windows, secrets must use an approved platform credential mechanism or separately accepted secure storage decision; plaintext secret configuration is not the default.

### 3.7 Model invocation retry policy

The v1 route performs no automatic retry of a model generation after a request has been consumed by the concrete model adapter. The learner may explicitly retry through a new interaction/workflow where policy permits.

This avoids accidental duplicate model side effects and simplifies evidence association.

Pre-invocation connection/configuration failures may be surfaced immediately; they do not consume a tutor generation attempt.

### 3.8 Timeouts

All external/long-running v1 operations must have finite cancellation-aware timeouts. Exact defaults are implementation configuration and must be tested on the v1 reference environment before System Verified maturity.

The timeout policy must distinguish:

- UI responsiveness timeout/state indication;
- local model operation deadline;
- graceful cancellation/join window;
- storage operation failure.

No infinite wait is allowed on the primary learner path.

### 3.9 Shared clients and accessibility

Both release clients use one React/TypeScript/Vite shared frontend over one versioned same-machine loopback HTTP/WebSocket business API, with Tauri 2 as the Windows shell. G1 selected this architecture; its disposable fixture remains evidence rather than production implementation. Tauri commands never become a second business API.

Both clients must provide keyboard-only operation, visible focus, usable scaling/high contrast, semantic status/error communication, text equivalents and captions/transcripts, reduced-motion handling, interruption controls, and equivalent recovery behavior. Required speech and animated 2D embodiment supplement rather than displace the accessible text path.

### 3.10 Observability

Use Rust structured tracing/logging in the application/runtime boundary with canonical IDs and bounded classifications.

The v1 route must capture at least:

- application startup/configuration outcome;
- session/workflow lifecycle;
- storage operation outcome;
- retrieval operation outcome;
- model invocation start/end timing and normalized outcome;
- admission/quality outcome classification;
- timeout/cancellation/recovery classification;
- shutdown/restart recovery outcome.

Raw content remains excluded from normal diagnostics.

### 3.11 Tutor/knowledge scope

The v1 route uses one explicit configured local model and one governed first-course corpus.

The existing provider-neutral selection/routing/filtering contracts are retained but dynamic multi-provider routing is not used to select v1 work.

V1 retrieval may use the simplest existing deterministic retrieval configuration that meets first-course grounding and performance requirements. A vector database is not required.

The response must retain governed citation/source association. Release-quality evaluation must test factual grounding, citation fidelity, assessment protection, and instructional usefulness for the first course.

### 3.12 First governed content package

The first v1 content package is Networking Fundamentals / TCP Connection Establishment.

It must include:

- immutable course/module/lesson identities and version/fingerprint;
- governed source material with provenance;
- objectives for TCP establishment purpose, SYN/SYN-ACK/ACK ordering, and basic TCP-vs-UDP contrast;
- deterministic assessment/practice items supported by the existing assessment engine;
- knowledge chunks sufficient for grounded tutor answers;
- expected acceptance outcomes and fixtures.

### 3.13 Platform and packaging

Windows x86_64 is the G4–G9 first-release acceptance target.

The v1 route itself may be exercised from a reproducible development/release-candidate build and does not require the final signed installer. G8 owns final installer/update/uninstall/provenance decisions.

Windows CI or equivalent reproducible Windows validation is required before claiming System Verified maturity for release-critical UI/storage/model paths.

## 4. Maturity evidence model adopted

The testing draft's maturity model is adopted for Nexa status reporting:

`Concept → Architecture Defined → Specification Approved → Contract Implemented → Runtime Integrated → Concrete Adapter Implemented → System Verified → User Accepted → Release Ready`

Unqualified `Complete` must not substitute for these states.

A green unit/contract/conformance suite proves only the maturity associated with the exercised boundary.

## 5. v1 acceptance scenario adopted

The primary E2E scenario launches either identical client, resumes durable SQLite state, completes Networking Fundamentals / TCP Connection Establishment through the local runtime and LM Studio, admits tutor output, commits assessment/evidence/mastery atomically, uses bundled speech, renders synchronized semantic 2D behavior, handles cancellation/reconnect/restart without duplication or loss, and demonstrates equivalent behavior in both clients. It then passes security, privacy, accessibility, performance, Windows packaging/recovery, system-verification, and owner user-acceptance gates.

Scripted providers, in-memory stores, Linux/headless tests, research, and spike code do not close this scenario.

## 6. Performance posture for v1

v1 performance work is measurement-first across both clients, the loopback boundary, LM Studio, bundled speech, and synchronized 2D rendering on the CPU-only Windows reference environment.

Before System Verified maturity, establish a documented Windows reference environment and record at minimum:

- application startup time;
- UI responsiveness while model work is active;
- durable-state load/commit latency;
- retrieval/context latency for the first corpus;
- model generation latency/time to learner-visible response;
- memory/disk usage;
- restart/resume latency.

No optimization project is authorized solely from an unmeasured assumption. A measured failure against a user/release requirement may select the next performance increment.

## 7. Items explicitly deferred from v1

The following remain valid future requirements but do not block v1:

- final remote-provider security/privacy design;
- release model selection;
- final installer/signing/update mechanism;
- tool/lab execution and sandbox enforcement;
- durable asynchronous event/outbox architecture;
- multi-process/multi-user concurrency;
- cloud sync/server deployment;
- plugin/public API/analytics/authoring systems;
- advanced routing/fallback;
- dedicated vector database.

Bundled speech input/output and synchronized animated 2D embodiment are required v1 capabilities, not deferrals. Their candidate proof remains gated at G2/G3, and their concrete integration and system evidence remain gated at G6/G7/G8.

## 8. R1 disposition and current delivery gate

R1 is approved as a governing v1 baseline. Current work selection is controlled by ADR-0069 and the G0–G9 roadmap:

1. this baseline and `NEXA-ARCH-002` are registered as current v1 authorities;
2. ADR-0069 is registered and ADR-0068 is preserved only for non-conflicting scope;
3. G0 is reviewed, green, and merged on its exact final documentation head;
4. G1 evidence is accepted, and React/TypeScript/Vite, Tauri 2, and one versioned same-machine loopback HTTP/WebSocket business boundary are selected at Architecture Defined / Specification Approved maturity;
5. the disposable G1 fixture remains evidence rather than production implementation and does not advance Runtime Integrated, Concrete Adapter Implemented, or system maturity;
6. **Continue to G2** authorizes only a separately dispatched bounded CPU-only speech evidence gate; Sherpa-ONNX remains a candidate until that gate passes;
7. **Tactical Pause** remains in force outside G2, and G3+ remain undispatched.

Those are convergence/recording tasks, not new product-specification design tasks.

## 9. Rebaseline rule

If implementation at any G gate reveals that an approved R1 requirement is insufficient or contradictory, implementation stops at that boundary and the parent specification/ADR is corrected before proceeding. The finite route is not permission to resume the old open-ended ADR loop.

---
