# Nexa v1 Observability Specification — Draft

Status: R1 proposal; non-authoritative until registered and approved

## Purpose

Define the minimum operational evidence required to diagnose and recover the Nexa v1 primary learner journey without leaking learner, knowledge, prompt, model-output, or secret content.

## Observability principles

- Operational evidence is not automatically authoritative domain state.
- Correlation uses canonical identifiers already owned by Nexa domain contracts.
- Content-safe metadata is the default.
- Observability failure should not corrupt learning state.
- Release-critical failures must be diagnosable without requiring raw prompt or learner content.
- Telemetry retention follows the privacy specification.

## Required correlated operations

At minimum correlate:

- application startup/shutdown;
- configuration validation;
- session lifecycle;
- interaction workflow lifecycle;
- durable state load/commit/migration/recovery;
- knowledge retrieval/context assembly;
- model invocation;
- output admission and tutor-quality gate;
- assessment/learning commit;
- timeout/retry/cancellation/recovery;
- user-visible error classification.

## Required safe fields

Where applicable, records may contain:

- session/workflow/invocation/request/result IDs;
- provider/model identifiers when non-secret;
- subsystem/operation name;
- contract/schema/policy versions;
- outcome/error classification;
- timestamps/durations;
- bounded counts/sizes;
- hashes/replay anchors;
- retry attempt number/max;
- cancellation/timeout classification;
- storage schema/migration version;
- application/release version.

## Prohibited default fields

Normal logs/telemetry must not contain raw:

- learner text;
- assessment answers;
- course/knowledge text;
- source documents;
- prompts/model input;
- raw model output;
- speech audio/transcript if enabled;
- tool stdout/stderr if tools are enabled;
- credentials/tokens/secrets.

## Severity and outcome model

The observability implementation must distinguish at least:

- informational lifecycle transition;
- recoverable dependency failure;
- retryable failure;
- user-action/configuration required;
- integrity/conflict failure;
- privacy/security rejection;
- terminal application/session failure.

Do not encode raw dependency error strings as the only machine-readable evidence.

## Metrics / release measurements

v1 should make measurable at minimum:

- startup duration;
- storage load/commit duration and failure count;
- retrieval duration/result count;
- model invocation duration/outcome;
- admission/quality rejection count;
- end-to-end learner interaction latency;
- timeout/retry/cancellation count;
- restart/recovery outcomes.

Exact metric backend/export is an implementation decision. A local structured log may satisfy part of v1 if it meets diagnostics and performance-verification needs.

## Health/status

The application must be able to classify the availability/configuration state of required v1 dependencies sufficiently for UX:

- durable data ready / migration required / failed;
- course/content ready / invalid;
- model provider configured / unavailable / auth/config failure;
- optional capability available/unavailable.

Do not infer external health beyond the evidence available from the concrete adapter.

## Startup and recovery diagnostics

Startup evidence must identify safely:

- application version;
- persistent schema version;
- migration attempted/outcome;
- configured provider/model identity without secrets;
- course/content version loaded;
- recovery/unclean-shutdown outcome if relevant.

## Retention and rotation

The packaging/privacy specifications must define:

- log location;
- maximum retained size/time;
- rotation/deletion behavior;
- whether user export of diagnostics is supported;
- whether an opt-in content-capture diagnostic mode exists.

Unlimited log growth is not acceptable.

## Verification

Release tests must prove:

- primary learner journey can be traced by correlation IDs;
- storage/retrieval/model/admission failures are distinguishable;
- retry/timeout/cancellation evidence is coherent;
- restart/recovery is diagnosable;
- prohibited raw content/secrets do not appear in normal diagnostics;
- logging/telemetry failure does not cause a false successful learning commit or corrupt state;
- retention/rotation behavior matches policy.

## Explicit deferrals

Unless required by deployment evidence:

- distributed tracing backend;
- cloud telemetry service;
- centralized fleet dashboards;
- analytics/product-event pipeline;
- long-term data warehouse.
