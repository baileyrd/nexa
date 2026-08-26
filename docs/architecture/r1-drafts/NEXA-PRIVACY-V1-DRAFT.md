# Nexa v1 Privacy and Data Handling Specification — Draft

Status: R1 proposal; non-authoritative until registered and approved

## 1. Purpose

Define how learner, instructional, model, diagnostic, and operational data may be collected, retained, disclosed, logged, exported, and deleted in Nexa v1.

This draft complements the data and security specifications. Structural redaction/filtering contracts already implemented in Nexa are reusable mechanisms but do not themselves define complete product privacy policy.

## 2. Principles

1. Local-first means learner-critical data remains local by default unless a release capability explicitly requires and authorizes remote disclosure.
2. Collect and persist only data required for the accepted learner experience, recovery, security, or governed product operation.
3. Remote provider disclosure is explicit, bounded, and purpose-specific.
4. Learner data and Nexa knowledge/content remain separate ownership domains.
5. Diagnostics are content-safe by default.
6. Retention and deletion behavior must be implementable and testable, not aspirational.
7. Privacy classifications affect allowed flows, not merely labels in documentation.

## 3. v1 data classes

### 3.1 Learner identity/profile

Examples:

- canonical learner identifier;
- display/profile preferences required by v1;
- course enrollment/selection state.

Default posture: Local.

### 3.2 Learning progress and evidence

Examples:

- lesson progress;
- assessment attempts/outcomes;
- competency evidence;
- mastery projections;
- pedagogy decisions/references.

Default posture: Local. Only the minimum subset necessary for a remote tutor interaction may enter an approved remote prompt.

### 3.3 Learner-provided interaction content

Examples:

- typed questions;
- assessment answers;
- speech transcripts if speech is enabled;
- learner-supplied files if a future feature permits them.

Default posture: Sensitive learner content. Remote disclosure only when required for the configured tutor capability and allowed by policy.

### 3.4 Governed instructional/knowledge content

Examples:

- course/lesson text;
- knowledge chunks;
- assessment material;
- source metadata/citations.

Classification depends on authored governance/exposure rules. Assessment-protected material must remain subject to existing fail-closed exposure controls.

### 3.5 Model input/output

Compiled prompts and raw model output may contain learner and instructional content and inherit the strongest applicable classification of their constituent material.

They are not ordinary operational telemetry.

### 3.6 Operational metadata

Examples:

- canonical IDs;
- versions;
- hashes/replay anchors;
- timing/duration;
- counts;
- bounded error/outcome classifications;
- dependency/provider/model identity where non-secret.

Default posture: suitable for content-safe diagnostics subject to retention policy.

### 3.7 Secrets

Credentials, API keys, authentication tokens, and secret configuration are security data and must never be treated as learner/profile/model telemetry.

## 4. Allowed data flows

### 4.1 Local processing

All v1 data classes may be processed locally by the owning component when required for the learner journey and allowed by their governance policy.

### 4.2 Remote model provider

A remote provider may receive only the exact model input produced after:

- trusted prompt-layer construction;
- assessment/governance restrictions;
- approved privacy/disclosure filtering;
- explicit configured provider authorization;
- size/version/security preflight.

The remote provider must not receive local storage records or unrestricted learner history by default. Context should be assembled specifically for the current interaction.

### 4.3 Logs/telemetry

Normal logs/telemetry may receive operational metadata but not raw learner, prompt, knowledge, assessment, raw model output, or secret content.

### 4.4 Export

Learner-data export, if included in v1, should provide understandable learner-associated state without including secrets or unrelated governed knowledge content. The exact format is a UX/data decision.

## 5. Remote disclosure policy

The v1 architecture must declare whether the default configured tutor path is:

- local-only;
- remote-capable with explicit configuration;
- remote-first with explicit user disclosure/consent expectations.

No code path may infer permission solely from provider availability.

For every remote prompt layer, the product policy must define whether the layer is:

- required and remotely allowed;
- optional and remotely allowed;
- local-only and therefore omitted/refused for remote invocation;
- prohibited because disclosure would violate assessment/security policy.

Existing ADR-0033 whole-layer filtering may implement part of this mechanism. Semantic/field-level minimization needs separate evidence where required.

## 6. Data minimization

Tutor context should contain only information reasonably necessary for the current instructional action.

At minimum:

- do not send the complete learner record when bounded mastery/progress references suffice;
- do not send unrelated course/knowledge chunks;
- do not send hidden assessment answers/solutions unless the approved tutor policy requires a protected evaluator boundary that permits them;
- do not include secrets or local filesystem metadata unnecessarily;
- prefer reference/summary data where existing contracts permit it without harming the instructional requirement.

The quality/grounding design must account for minimization rather than assuming more context is always better.

## 7. Retention

R1 approval must define concrete v1 retention behavior for:

- learner profile;
- lesson/course progress;
- immutable learning evidence;
- assessment responses vs assessment outcome/evidence;
- conversation/turn content if persisted at all;
- raw model input/output if persisted at all;
- operational logs;
- knowledge/content provenance;
- optional diagnostic captures.

Default rule: raw conversational/model content should not be persisted indefinitely merely because it is available.

## 8. Deletion and reset

The learner must have a documented way to reset/delete locally persisted learner data appropriate to v1.

Deletion behavior must specify:

- which learner-associated records are removed;
- how immutable evidence is handled under a deletion request;
- whether backups/exports remain and for how long;
- whether shared course/knowledge content is unaffected;
- how logs are handled according to their retention period;
- how provider-side data is outside local deletion authority and must be described according to the provider relationship/configuration.

Do not claim deletion of data outside Nexa's control.

## 9. Conversation and memory

The reconstructed architecture describes multiple memory scopes. For v1:

- turn/session context may exist in memory for active interaction;
- persistent learner memory must be limited to approved learner/profile/progress/evidence fields;
- persistence of free-form conversation history is not required unless explicitly approved;
- if free-form conversation history is persisted, it becomes a first-class privacy-governed data type with retention, export, deletion, and remote-disclosure rules.

This avoids accidentally turning transient model context into permanent learner memory.

## 10. Assessment privacy

Assessment-protected material and learner answers require stricter handling.

Requirements:

- existing exposure restrictions remain authoritative;
- logs/diagnostics must not contain raw answers by default;
- remote prompt construction must prevent protected answer keys/solutions from leaking into learner-facing generation unless a governing assessment/tutor flow explicitly requires and contains them;
- retained answer content must be minimized according to assessment replay/review requirements;
- mastery evidence should remain privacy-minimal where possible.

## 11. Speech privacy — conditional v1

If speech is promoted to v1:

- microphone capture state must be visible to the learner;
- raw audio retention defaults to off unless explicitly required;
- transcript classification is learner interaction content;
- remote STT/TTS disclosure requires the same configured-provider transparency as remote model use;
- voice/provider identifiers and any generated audio retention must be specified;
- cancellation must stop Nexa-owned capture/playback and define what cannot be revoked after remote transmission.

## 12. Labs/tools privacy — conditional v1

If labs/tools are promoted:

- command text, stdout/stderr, filenames, environment data, and captured artifacts may contain sensitive learner/system content;
- the lab specification must classify what is retained and what may be shown to the tutor/model;
- secrets/environment variables must be filtered from observation paths;
- diagnostics must remain content-safe by default.

## 13. Observability/privacy contract

Allowed default diagnostic evidence includes:

- canonical session/workflow/request IDs;
- provider/model IDs where not secret;
- contract/schema/policy versions;
- hashes/replay anchors;
- sizes/counts;
- durations/timestamps;
- bounded lifecycle/outcome/error classifications.

Disallowed by default:

- prompt text;
- learner text;
- knowledge/source text;
- assessment answers;
- raw model output;
- raw audio/transcripts if speech is enabled;
- secrets/credentials.

## 14. Provider transparency

For every supported remote provider configuration, the product must make clear enough for the learner/operator to understand:

- that data may leave the local machine;
- which capability causes the disclosure;
- the configured provider identity;
- which Nexa data classes may be included;
- that provider-side handling/retention is governed by the provider relationship and cannot be erased by Nexa beyond available provider mechanisms.

Exact UI language belongs to the UX specification.

## 15. Privacy verification

Before release verify:

- remote model invocation cannot occur through a LocalOnly policy path;
- each approved prompt layer follows the disclosure policy;
- assessment-protected material is not exposed through retrieval/prompt bypass;
- ordinary logs/errors/debug output contain no raw sensitive classes;
- persisted records match the retention inventory;
- reset/deletion removes the classes promised by v1;
- restart does not unexpectedly persist transient conversation/model data;
- exported learner data excludes secrets and unrelated knowledge content;
- conditional speech/lab paths satisfy their additional privacy tests if included.

## 16. Decisions required for approval

- v1 model provider local/remote default posture;
- whether free-form conversation history is persisted;
- exact retention periods or retention triggers for each data class;
- learner reset/export UX requirements;
- handling of immutable evidence under deletion;
- whether diagnostic content-capture mode exists in v1;
- speech/labs v1 disposition.

## 17. Explicit post-v1 scope unless promoted

- cloud account/profile sync;
- organizational learner analytics;
- cross-device memory;
- shared instructor dashboards;
- federated/enterprise privacy administration;
- plugin data-sharing policy;
- server-side multi-tenant retention controls.

## 2026-08-26 ADR-0069 reconciliation

Speech is required and locally bundled/managed, so audio/transcript disclosure, retention, deletion, diagnostics, and consent/indicator behavior must be specified and tested rather than treated as conditional. The v1 client/API is loopback-only and LM Studio is local; remote-provider and cloud-sync provisions remain future safeguards, not v1 flows.
