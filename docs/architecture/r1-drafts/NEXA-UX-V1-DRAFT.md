# Nexa v1 Learner UX Specification — Draft

Status: R1 proposal; non-authoritative until registered and approved

## 1. Purpose

Define the minimum learner-facing experience required to make the Nexa v1 system usable and testable. This specification owns presentation, interaction, accessibility, and user-visible recovery behavior. It does not own learning, tutor, model, persistence, or orchestration policy.

## 2. v1 UX objective

A learner can install/launch Nexa, configure the required tutor model path, select or resume a course, complete one adaptive lesson through text interaction and assessment, understand progress and errors, exit, restart, and resume correctly.

The core v1 experience is text-capable. Speech/avatar/labs are additive unless explicitly promoted into the required release path.

## 3. Required application states

The learner application must represent at minimum:

- First Launch / Setup Required
- Ready / Course Selection
- Loading Course/State
- Lesson Active
- Waiting for Learner Input
- Tutor Working
- Tutor Response Available
- Assessment/Practice Active
- Progress Committing
- Lesson Progress Updated
- Recoverable Dependency Failure
- Configuration Required/Invalid
- Provider Unavailable
- Data Migration/Recovery Required
- Cancelled/Interrupted
- Terminal Application/Data Failure
- Lesson Completed

The UI may combine visual layouts, but these semantic states must be distinguishable where behavior differs.

## 4. First launch and setup

The application must:

- identify whether required local data/schema initialization is needed;
- identify whether a v1 model provider/configuration is ready;
- guide the user to required configuration without exposing credentials after entry;
- distinguish local-only and remote-provider posture clearly enough to support privacy expectations;
- report unsupported/missing configuration with actionable next steps;
- not require knowledge of internal crate/ADR/provider-neutral architecture.

## 5. Course and lesson selection

v1 must present:

- available released course(s);
- current/resumable learner progress;
- lesson identity/title/objective sufficient for orientation;
- completed/current/available state according to governed lesson rules;
- an explicit way to start or resume the supported lesson.

The UX must not invent lesson availability or progress rules; it renders the owning lesson/learning state.

## 6. Tutor interaction

Minimum text interaction requirements:

- clear learner input control;
- submit/send action;
- visible indication that Nexa is processing;
- ability to request cancellation/interruption while an interaction is active where the orchestrator supports it;
- tutor response display preserving structured sections as required by the tutor contract;
- citation/source affordance for grounded responses sufficient to identify/review the governed source evidence included in v1;
- constrained/refusal/error responses rendered distinctly from normal instructional content;
- no raw model/provider protocol/error payload displayed to the learner.

## 7. Progress and pedagogy presentation

The learner must be able to understand:

- current lesson/activity;
- whether an assessment/practice response was accepted/evaluated;
- whether progress was durably saved;
- current progression/next action at a level appropriate to the lesson design.

The UX may show mastery/competency information if the product decision includes it, but it must not fabricate precision or expose internal estimator details as pedagogical truth.

## 8. Assessment/practice UX

The application must:

- render the released assessment/question type(s);
- collect a response without leaking protected answer/solution content;
- distinguish submission-in-progress from committed/evaluated outcome;
- present feedback allowed by the assessment/pedagogy policy;
- prevent accidental duplicate submissions where the workflow is already committed or in-flight;
- handle retry/recovery according to orchestrator/assessment rules.

## 9. Save/commit semantics

Learner-facing progress indicators must reflect durable state accurately.

Requirements:

- do not show `Saved`/completed progress until the durable commit succeeds;
- if presentation occurs before a required learning-state commit, communicate pending state where necessary;
- on commit failure, preserve the learner's understanding that progress may not have been saved and provide the approved recovery action;
- after restart, show only durable accepted state.

## 10. Error and recovery UX

Errors must map from bounded application/orchestrator classifications into useful learner messages.

At minimum distinguish:

- provider not configured;
- provider authentication/configuration failure;
- provider temporarily unavailable/timeout;
- course/content unavailable or invalid;
- local data migration/recovery required;
- local data corruption/unrecoverable state;
- response rejected by safety/quality/admission policy;
- concurrent/conflicting operation requiring retry/reload;
- learner cancellation;
- unexpected internal failure.

Messages should explain what the learner can do next without revealing secrets or raw sensitive content.

## 11. Offline/local behavior

The UI must accurately represent the configured capability:

- if the v1 tutor path is local, remote-connectivity language must not be shown unnecessarily;
- if the provider is remote and network is unavailable, distinguish that from corrupted local learner/course state;
- course/progress state that is safely usable offline should remain accessible according to product policy;
- do not claim full offline tutoring if the configured required model path cannot operate offline.

## 12. Restart and resume UX

After normal or supported unclean restart:

- load/validate durable learner/course progress;
- identify the resumable lesson/activity;
- do not resurrect transient in-flight model/task state as if committed;
- surface any recovery/migration issue before allowing conflicting new work;
- allow continuation from the last accepted durable boundary.

## 13. Accessibility baseline

For the required learner UI, v1 must provide at minimum:

- keyboard operation for primary controls where the desktop framework supports it;
- visible focus and actionable-state indication;
- readable text contrast and scaling according to the selected platform/framework guidelines;
- error/status information not conveyed solely by color, animation, or sound;
- textual access to instructional content even if avatar/speech is enabled;
- labels/names for interactive controls sufficient for platform accessibility APIs where supported;
- predictable navigation order;
- learner control to stop/cancel long-running interactions where supported.

A later UX technology decision should map these requirements to concrete platform accessibility mechanisms.

## 14. Privacy transparency

When remote provider use is configured, the setup/UX must communicate at a product-appropriate level:

- that tutor interaction data may be sent to the configured provider;
- provider identity;
- whether local-only operation is available/selected;
- where learner-data reset/delete controls are located if provided in v1.

Do not expose every internal prompt layer or implementation detail; transparency should be accurate and understandable.

## 15. Diagnostics/support UX

The application should provide a safe way to identify:

- application version;
- configured provider/model identity without secret;
- current course/content version;
- diagnostic/log location or export action if included;
- correlation/reference ID for a failed interaction where useful.

Do not present raw prompt/model/learner content as default diagnostic details.

## 16. Conditional speech

If speech is promoted to v1:

- microphone-active state is visible;
- learner can stop capture;
- transcript/recognition state is distinguishable from committed learner input;
- speech failure falls back to text where product policy allows;
- TTS/audio can be stopped/interrupted;
- essential content is available in text.

## 17. Conditional avatar

If avatar embodiment is promoted:

- animation may enhance but not be the sole carrier of required instructional/status information;
- the avatar reflects admitted semantic behavior rather than direct model renderer control;
- avatar/runtime failure must degrade according to explicit product policy without corrupting the learner workflow.

## 18. Conditional labs/tools

If labs/tools are promoted:

- execution/destructive actions require the confirmation/security UX defined by the lab/security specs;
- running/stopped/failed state is visible;
- learner can distinguish tool output from tutor text;
- timeouts/cancellation/non-cancellable state are represented accurately.

## 19. User acceptance criteria

A representative learner must be able to, without developer intervention beyond documented setup:

1. launch/setup Nexa;
2. select/resume the released course;
3. understand the current lesson objective;
4. ask/respond through text tutor interaction;
5. inspect a grounded/cited response where citations are provided;
6. complete the released assessment/practice interaction;
7. understand whether progress was saved;
8. recover from at least one simulated provider failure using the provided UX;
9. exit/relaunch and resume the correct durable state;
10. complete the lesson.

Blocking confusion, inaccessible primary actions, false save/completion state, or inability to recover from documented recoverable failures blocks User Accepted maturity.

## 20. Decisions required for approval

- concrete desktop UI framework/application shell;
- first supported OS/platform;
- exact v1 provider setup/config flow;
- amount of mastery/progress detail exposed;
- citation/source presentation design;
- reset/delete/export controls included in v1;
- speech/avatar/labs v1 disposition;
- concrete accessibility standard/platform mapping.
