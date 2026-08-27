# Nexa Domain and Event v1 Rebaseline Requirements — Draft

Status: R1 proposal; intended to mature `NEXA-DOM-001` and `NEXA-EVT-001`

## 1. Purpose

Define the bounded v1 parent-contract work required from the foundational Domain and Event specifications. The goal is not to transcribe every reconstructed type into code. It is to approve the canonical shared concepts actually needed by the v1 learner journey and settle the event/runtime semantics that later phases deferred.

## 2. Domain model v1 scope

The shared domain model must provide canonical value types for identities and values that cross subsystem boundaries in v1.

Already implemented examples include canonical session/message/event/behavior/correlation/trace identities, timestamps, duration, confidence, protocol version, learner/lesson/assessment/model/retrieval/workflow identities added across phases.

The rebaseline must:

1. inventory every public canonical ID/value currently implemented;
2. map each to an owning parent spec and v1 consumer;
3. remove no accepted type without compatibility review;
4. avoid implementing reconstructed identifiers/aggregates that have no v1 consumer merely for completeness;
5. define any missing v1 identities required by durable state, application/configuration, course/content packages, recovery, or release evidence.

## 3. Canonical identity requirements

A cross-boundary v1 identity must have:

- one canonical type owner;
- stable serialization where persisted/transmitted;
- non-nil/valid construction rules;
- no implicit authentication/authorization meaning unless explicitly specified;
- clear lifecycle/uniqueness scope;
- association validation at trust boundaries;
- migration/compatibility behavior if persisted.

Subsystems must not create competing string/UUID wrappers for the same concept.

## 4. Time requirements

Persistent event/evidence timestamps continue to use the canonical timestamp type.

R1 must distinguish:

- persisted observation/event time;
- monotonic elapsed/deadline measurement used for runtime timeout behavior;
- authored course/content time if any;
- provider-reported time if any.

Wall-clock timestamps must not be used as monotonic timeout clocks. The orchestrator/runtime specification must own deadline/elapsed behavior, while persisted timestamp capture points are defined by the owning domain operation.

## 5. Version requirements

v1 must distinguish at minimum:

- wire/protocol versions;
- persistent schema versions;
- authored course/content versions/fingerprints;
- policy/estimator/scoring versions;
- application release version;
- provider/model/tokenizer identity/version data as required by tutor evidence.

One generic version field must not be overloaded across these meanings.

## 6. Event model v1 decision

The reconstructed event architecture is broader than the current in-process event implementation. R1 must explicitly classify event uses into three categories.

### 6.1 Authoritative domain facts

Events whose loss would violate the v1 correctness/replay model.

Examples may include accepted learning evidence/progress facts if asynchronous consumers require them.

If v1 has authoritative asynchronously published events, durability/ordering/replay/outbox semantics must be specified and implemented.

### 6.2 Process-local notifications

Typed facts/events used to decouple in-process components but not required for durable correctness.

These may use the existing in-process bus if its delivery semantics satisfy the owning use case.

### 6.3 Operational telemetry

Logs/metrics/traces/outcome events used for diagnosis. These are governed by observability/privacy and do not become domain truth merely because they are serialized.

## 7. v1 event-bus scope

R1 should prefer the smallest event architecture that satisfies the primary learner journey.

Before adding durable bus infrastructure, document:

- which release-critical consumer needs asynchronous durable delivery;
- why direct in-process orchestration/transaction return values are insufficient;
- required ordering scope;
- at-least-once/at-most-once expectations;
- replay purpose;
- privacy/retention;
- failure/recovery behavior.

If no v1 consumer requires a durable bus, retain typed domain facts and in-process notifications while explicitly deferring broader event infrastructure.

## 8. Event envelope

For v1 persisted/transmitted events, retain the canonical envelope principles already implemented:

- stable event identity;
- event kind/type;
- schema/protocol version;
- source;
- session/workflow/correlation/trace association as applicable;
- timestamp;
- sequence/order evidence where required;
- typed validated payload.

Unknown required versions/types fail according to compatibility policy. Extensions remain namespaced/bounded as governed.

## 9. Ordering and delivery

Every v1 event stream/use must state its actual ordering scope rather than assuming global order.

Existing `(source, session)` sequencing may remain where appropriate.

Durable/asynchronous event use must specify:

- delivery semantics;
- duplicate handling;
- consumer idempotency expectations;
- replay boundaries;
- poison/invalid event behavior;
- retention.

## 10. Privacy

Event payloads inherit data classification from their content.

Requirements:

- do not log/rebroadcast full event payloads by default;
- learner/assessment/model content uses the privacy policy;
- event persistence/retention must be part of the data/privacy inventory;
- correlation IDs are not authentication credentials;
- operational telemetry should prefer content-free event metadata over domain payload copying.

## 11. Command vs event distinction

The event rebaseline must resolve the reconstructed command-envelope scope for v1.

Commands/requests express an instruction/request and may fail before a fact exists. Events record facts that occurred/accepted according to the owning domain semantics.

Do not label a request as an event merely to use the event bus.

## 12. Compatibility and schemas

Before v1 release, persisted/transmitted release-critical contracts require stronger compatibility evidence than early golden fixtures alone.

R1/R9 must decide and implement as applicable:

- formal JSON Schema or equivalent machine-readable schema for stable external/persisted JSON contracts;
- backward compatibility test strategy;
- migration rules for persisted envelopes/payloads;
- unknown-field/version behavior;
- content/hash fixtures tied to release versions.

In-process-only Rust types do not automatically require external schema artifacts.

## 13. Verification

Rebaseline acceptance includes:

- inventory test/document linking v1 cross-boundary identities to canonical owners;
- serialization/validation tests for persisted/wire canonical values;
- timestamp/version semantics tests;
- event-kind/envelope strictness tests;
- duplicate/order/replay tests for any durable v1 event use;
- privacy/redaction tests for event diagnostics;
- evidence that no v1 subsystem redefines a canonical shared identity;
- explicit test showing which event uses are authoritative versus telemetry/notification.

## 14. Explicit deferrals

Unless a v1 consumer proves the need:

- distributed event broker;
- cross-process/server event mesh;
- global total ordering;
- generalized event sourcing;
- full reconstructed event payload catalog;
- command bus infrastructure;
- cloud event retention/analytics.

## 15. Approval decisions

- final v1 canonical identity inventory;
- clock/deadline ownership boundary;
- which domain facts, if any, require durable asynchronous publication;
- outbox/durable event-store requirement;
- formal schema scope for release-critical persisted/wire contracts;
- event retention policy integration.

## 2026-08-26 ADR-0069 reconciliation

Canonical identities/events span both clients and the versioned loopback boundary without adapter-specific redefinition. Speech and semantic 2D behavior events are required v1 integration surfaces; renderer primitives and provider protocol details remain outside domain authority. Hosted/cloud brokerage remains deferred.
