# Nexa v1 Observability Specification — Draft

Status: Approved R1 supplement through NEXA-R1; reconciled by ADR-0069

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

- local runtime and both identical client startup/readiness/shutdown;
- client kind and shared-interface/release version, without learner content;
- versioned loopback HTTP request and WebSocket connection/reconnection lifecycle;
- configuration validation;
- session lifecycle;
- interaction workflow lifecycle;
- durable state load/commit/migration/recovery;
- knowledge retrieval/context assembly;
- LM Studio configuration, health, compatibility, and model invocation;
- output admission and tutor-quality gate;
- assessment/learning commit;
- timeout/retry/cancellation/recovery;
- user-visible error classification;
- bundled speech recognition/synthesis/device lifecycle and interruption;
- admitted semantic 2D state, speech/animation synchronization, and accessible fallback;

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
- speech audio/transcript;
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

- local runtime and browser/desktop client startup/readiness duration;
- loopback HTTP/WebSocket request, event, reconnect, cancellation, and failure timing/counts;
- storage load/commit duration and failure count;
- retrieval duration/result count;
- LM Studio health/invocation duration, compatibility, and outcome classification;
- admission/quality rejection count;
- end-to-end learner interaction latency;
- timeout/retry/cancellation count;
- restart/recovery outcomes;
- speech recognition/synthesis latency, interruption outcome, and CPU/memory use;
- semantic-state-to-animation and speech/lip-sync timing, fallback outcome, and 2D CPU use;

Exact metric backend/export is an implementation decision. A local structured log may satisfy part of v1 if it meets diagnostics and performance-verification needs.

## Health/status

The application must be able to classify the availability/configuration state of required v1 dependencies sufficiently for UX:

- durable data ready / migration required / failed;
- course/content ready / invalid;
- LM Studio configured / incompatible / unavailable / configuration failure;
- bundled speech model/runtime/device ready / degraded / unavailable;
- synchronized 2D runtime ready / degraded to accessible text / unavailable;
- loopback business API and WebSocket ready / reconnecting / incompatible / failed.

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

- the primary learner journey from either identical client can be traced across the loopback HTTP/WebSocket boundary by correlation IDs;
- storage/retrieval/model/admission failures are distinguishable;
- retry/timeout/cancellation evidence is coherent;
- restart/recovery is diagnosable;
- prohibited raw content/secrets do not appear in normal diagnostics;
- logging/telemetry failure does not cause a false successful learning commit or corrupt state;
- retention/rotation behavior matches policy;
- LM Studio, bundled speech, and synchronized 2D health/failure paths are distinguishable without content capture;
- resource and timing evidence covers both clients, loopback transport, model, speech, and 2D behavior on the Windows reference environment.

## Explicit deferrals

Unless required by deployment evidence:

- distributed tracing backend;
- cloud telemetry service;
- centralized fleet dashboards;
- analytics/product-event pipeline;
- long-term data warehouse.
