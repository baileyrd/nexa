# Nexa v1 Governed Course and Content Release Specification — Draft

Status: R1 proposal; non-authoritative until registered and approved

## 1. Purpose

Define the minimum governed content package needed to prove and ship the Nexa v1 learner journey without requiring the full authoring application.

The current repository reserves `content/courses`, `content/assessments`, `content/knowledge`, and `content/labs`, but a reserved directory is not released instructional content. v1 requires at least one real governed course/lesson package.

## 2. v1 content objective

The release includes or installs one versioned course package sufficient for a representative learner to:

- start/resume a course;
- complete at least one governed lesson;
- interact with grounded knowledge;
- complete at least one released assessment/practice path;
- generate learning evidence/mastery/progress;
- satisfy the v1 E2E/user-acceptance scenario.

The first course is an acceptance vehicle and a real usable training artifact, not merely a unit-test fixture.

## 3. Course package contents

A v1 package must identify and version as applicable:

- curriculum/course/module/lesson definitions;
- lesson objectives and authored routes;
- instructional content used directly by the lesson;
- assessment/question/rubric definitions required by the released path;
- governed knowledge sources/artifacts used for tutor grounding;
- source provenance/license metadata;
- content/package version/fingerprint;
- compatibility requirements with Nexa application/spec/schema versions;
- optional media/assets actually required by the released lesson.

Labs are excluded unless explicitly promoted to v1.

## 4. Content authority and provenance

Every released knowledge/instructional artifact must have known provenance and distribution rights appropriate to release.

Required metadata includes enough to establish:

- source/author/publisher or internal authored origin;
- content version/date where relevant;
- license/permission classification;
- immutable content hash/fingerprint;
- exposure classification, especially assessment-protected material;
- course/lesson associations where required.

Unknown provenance blocks release packaging for that artifact.

## 5. Validation before release/use

A course package must be validated before it becomes selectable/active.

Validation covers:

- schema/contract version support;
- ID uniqueness and canonical associations;
- lesson graph/prerequisite validity;
- authored pedagogy route targets;
- assessment references and scoring-policy versions;
- knowledge source/artifact integrity;
- protected assessment exposure rules;
- referenced asset existence/integrity;
- package/application compatibility;
- provenance/license completeness required by release policy.

Partially valid packages must not be activated.

## 6. Authoring strategy for v1

A rich authoring application is not required.

Acceptable v1 approaches include:

- hand-authored governed JSON/Markdown/content files plus validation tooling;
- minimal focused compilers/validators needed to prevent invalid release content;
- deterministic build step producing an immutable content package.

The project should implement only the authoring tooling required to safely maintain/release the first course.

## 7. Knowledge corpus

The first course knowledge corpus must be deliberately bounded and suitable for grounding evaluation.

Requirements:

- every retrieval-evaluation question has known expected supporting source(s) or an expected no-answer outcome;
- corpus version is immutable for a release evaluation run;
- knowledge chunks retain source provenance after packaging/install/restart;
- assessment-protected sources are explicitly classified;
- corpus scale is recorded for retrieval/performance evidence.

## 8. Assessment content

The first course must use only assessment types supported by the approved v1 learning/assessment specification.

Required content evidence:

- question/assessment IDs and versions;
- correct scoring/rubric configuration;
- protected answer/solution classification;
- mapping to competency/objective evidence;
- expected learner feedback class;
- malformed/invalid content rejected by release validation.

## 9. Content versioning and learner progress

A learner's persisted progress must identify the governing content version/fingerprint.

When a course package changes, the content/lesson/data specs must determine whether:

- the change is compatible and progress remains valid;
- migration is required;
- the learner resumes the old installed version;
- the learner starts a new version;
- completion remains historical but not resumable.

Do not silently attach old progress/evidence to semantically different lesson content.

## 10. Packaging

The course may be:

- bundled in the application release; or
- distributed as a separately installable governed package.

Either path must preserve:

- immutable package identity;
- integrity verification;
- provenance/licenses;
- application compatibility;
- atomic activation/update behavior.

A generalized content marketplace is post-v1.

## 11. Content acceptance set

The v1 content package must include/refer to a versioned acceptance/evaluation set covering:

- lesson happy path;
- at least one incorrect/partial/correct pedagogical route where the lesson supports them;
- assessment evidence/mastery update;
- retrieval grounding questions;
- no-answer/insufficient-context case;
- citation fidelity cases;
- assessment-protection/answer-leak case;
- hostile instruction-like source text case if relevant to prompt-injection testing;
- restart/resume after durable progress.

## 12. Release gate

A v1 release is blocked if:

- no real course package exists;
- the primary learner E2E uses only synthetic test fixtures that are not the released content;
- provenance/license information is incomplete for distributed content;
- package validation fails;
- learner progress/content-version behavior is undefined;
- the tutor evaluation set is not tied to the exact released corpus version.

## 13. Decisions required for approval

- subject/domain of the first released course;
- exact lesson count/minimum acceptance scope;
- hand-authored format and validation tool path;
- bundled vs separately installed content;
- content/package ID/version format;
- content licensing/provenance policy;
- exact first-course assessment types;
- exact retrieval/evaluation corpus.
