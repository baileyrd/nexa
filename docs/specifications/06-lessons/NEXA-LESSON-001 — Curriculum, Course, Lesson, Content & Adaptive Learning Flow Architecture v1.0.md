# NEXA-LESSON-001 — Curriculum, Course, Lesson, Content & Adaptive Learning Flow Architecture v1.0

**Specification ID:** NEXA-LESSON-001
**System:** Nexa AI Training Tutor
**Version:** 1.0
**Status:** Baseline Draft
**Depends On:** NEXA-DOM-001, NEXA-EVT-001, NEXA-STU-001, NEXA-PED-001, NEXA-TUTOR-001, NEXA-KNOW-001, NEXA-ORCH-001, NEXA-NBP-001
**Purpose:** Define how structured learning content is represented, packaged, validated, delivered, adapted, resumed, versioned, and connected to competencies, knowledge sources, assessments, labs, tutor behavior, and student progress.

---

## 1. Purpose

The Lesson subsystem answers:

> **“What should be taught, in what structure, under what completion rules, and how may the experience adapt without losing curriculum intent?”**

The Lesson system SHALL transform authored training content into executable learning flows.

---

# 2. Curriculum Hierarchy

The baseline hierarchy is:

```text
Curriculum
   ↓
Program
   ↓
Course
   ↓
Module
   ↓
Lesson
   ↓
Lesson Step
   ↓
Learning Activity
```

Not every deployment must use every level.

---

# 3. Core Responsibilities

The Lesson subsystem SHALL own or coordinate:

* curriculum structure;
* course definitions;
* modules;
* lessons;
* lesson steps;
* learning objectives;
* competency mappings;
* prerequisites;
* branching;
* activity sequencing;
* completion rules;
* resume state;
* remediation branches;
* challenge branches;
* lesson metadata;
* course manifests;
* content versioning;
* lesson validation;
* course packaging;
* content publishing;
* adaptive routing contracts;
* assessment bindings;
* lab bindings;
* knowledge bindings;
* tutor behavior cues.

---

# 4. Explicit Non-Responsibilities

The Lesson system SHALL NOT own:

* competency estimation;
* pedagogy selection;
* final tutor wording;
* assessment scoring;
* lab execution;
* speech synthesis;
* avatar animation;
* knowledge retrieval implementation.

It defines authored instructional intent and execution structure.

---

# 5. Course Definition

```rust
pub struct Course {
    pub id: CourseId,
    pub key: CourseKey,
    pub title: String,
    pub description: String,
    pub version: CourseVersion,

    pub objectives: Vec<LearningObjectiveId>,
    pub modules: Vec<ModuleId>,
    pub prerequisites: Vec<CompetencyRequirement>,

    pub knowledge_policy: CourseKnowledgePolicy,
    pub completion_policy: CourseCompletionPolicy,
}
```

---

# 6. Course Key

A stable authored key SHOULD be human-readable.

Example:

```text
networking.fundamentals
rust.ownership
cybersecurity.linux.basics
```

UUID identity and authored key MAY coexist.

---

# 7. Module

```rust
pub struct Module {
    pub id: ModuleId,
    pub course_id: CourseId,
    pub key: String,
    pub title: String,
    pub description: String,

    pub lessons: Vec<LessonId>,
    pub objectives: Vec<LearningObjectiveId>,
}
```

Modules primarily organize related lessons.

---

# 8. Lesson

```rust
pub struct Lesson {
    pub id: LessonId,
    pub module_id: Option<ModuleId>,
    pub course_id: CourseId,

    pub key: String,
    pub title: String,
    pub description: String,

    pub objectives: Vec<LearningObjectiveId>,
    pub prerequisites: Vec<CompetencyRequirement>,

    pub steps: Vec<LessonStep>,
    pub branches: Vec<LessonBranch>,

    pub completion_policy: LessonCompletionPolicy,
    pub version: LessonVersion,
}
```

---

# 9. Lesson Step

```rust
pub struct LessonStep {
    pub id: LessonStepId,
    pub key: String,
    pub kind: LessonStepKind,

    pub objectives: Vec<LearningObjectiveId>,
    pub content: LessonStepContent,

    pub entry_condition: Option<LessonCondition>,
    pub completion_rule: CompletionRule,

    pub pedagogy_hint: Option<PedagogyHint>,
    pub behavior_cue: Option<BehaviorCue>,
}
```

---

# 10. Lesson Step Kinds

```rust
pub enum LessonStepKind {
    Introduction,
    Explanation,
    Demonstration,
    Example,
    Practice,
    Question,
    Reflection,
    Review,
    Remediation,
    Challenge,
    Lab,
    Assessment,
    Summary,
}
```

---

# 11. Learning Objective

Every lesson SHOULD tie activity to explicit learning objectives.

```rust
pub struct LearningObjective {
    pub id: LearningObjectiveId,
    pub key: String,
    pub statement: String,

    pub competency_ids: Vec<CompetencyId>,
    pub success_criteria: Vec<SuccessCriterion>,
}
```

---

# 12. Objective Example

```text
Objective:
Explain the purpose and ordering of the TCP three-way handshake.

Competencies:
networking.tcp.handshake.sequence
networking.tcp.handshake.purpose
```

---

# 13. Objective Granularity

Objectives SHOULD describe observable learner capability.

Prefer:

```text
Identify the sequence SYN → SYN-ACK → ACK.
```

Avoid:

```text
Understand TCP.
```

---

# 14. Content Types

Lesson steps MAY contain:

```rust
pub enum LessonStepContent {
    Text(TextContent),
    Diagram(DiagramContent),
    Code(CodeContent),
    Demonstration(DemonstrationContent),
    Question(QuestionId),
    Lab(LabId),
    Assessment(AssessmentId),
    Media(MediaContent),
    Composite(Vec<LessonStepContent>),
}
```

---

# 15. Text Content

```rust
pub struct TextContent {
    pub source: ContentSource,
    pub display_text: Option<String>,
    pub tutor_instruction: Option<String>,
}
```

`tutor_instruction` describes instructional intent, not exact phrasing.

---

# 16. Diagram Content

```rust
pub struct DiagramContent {
    pub artifact_id: ArtifactId,
    pub semantic_objects: Vec<DiagramObject>,
}
```

This permits Nexa to gaze at and point toward named elements.

---

# 17. Semantic Diagram Objects

Example:

```text
tcp.syn
tcp.syn_ack
tcp.ack
client
server
```

These IDs SHOULD be stable across lesson behavior cues.

---

# 18. Code Content

```rust
pub struct CodeContent {
    pub language: String,
    pub code: String,
    pub executable: bool,
    pub artifact_id: Option<ArtifactId>,
}
```

Executable content SHALL still require tool/lab authorization.

---

# 19. Demonstration Content

```rust
pub struct DemonstrationContent {
    pub steps: Vec<DemonstrationStep>,
}
```

---

# 20. Demonstration Step

```rust
pub struct DemonstrationStep {
    pub instruction: String,
    pub tool_action: Option<ToolRequestTemplate>,
    pub canvas_actions: Vec<CanvasAction>,
    pub behavior_cue: Option<BehaviorCue>,
}
```

---

# 21. Behavior Cue

Lesson authors MAY suggest semantic avatar behavior.

```rust
pub struct BehaviorCue {
    pub state: Option<BehaviorState>,
    pub gesture: Option<GestureIntent>,
    pub gaze_target: Option<AttentionTarget>,
}
```

These cues SHALL remain advisory unless the lesson explicitly requires synchronization.

---

# 22. Pedagogy Hint

Lesson authors MAY provide preferred instructional strategy.

```rust
pub struct PedagogyHint {
    pub preferred_strategy: Option<PedagogyStrategy>,
    pub avoid_strategies: Vec<PedagogyStrategy>,
}
```

The Pedagogy Engine remains authoritative within higher-level policy.

---

# 23. Lesson Conditions

```rust
pub enum LessonCondition {
    Always,
    ObjectiveCompleted(LearningObjectiveId),
    CompetencyAtLeast {
        competency_id: CompetencyId,
        threshold: MasteryScore,
    },
    CompetencyBelow {
        competency_id: CompetencyId,
        threshold: MasteryScore,
    },
    AssessmentPassed(AssessmentId),
    AssessmentFailed(AssessmentId),
    LabCompleted(LabId),
    HintUsageAtLeast(u8),
    Custom(String),
}
```

---

# 24. Branching

The Lesson Engine SHALL support conditional branches.

```text
Core lesson
   ↓
Check learner state
   ├── weak → remediation
   ├── normal → continue
   └── strong → challenge
```

---

# 25. Lesson Branch

```rust
pub struct LessonBranch {
    pub id: LessonBranchId,
    pub condition: LessonCondition,
    pub target_step_id: LessonStepId,
    pub priority: u16,
}
```

---

# 26. Branch Determinism

If multiple branches match, the system SHALL resolve them deterministically.

Recommended resolution:

```text
highest priority
then authored order
```

---

# 27. Adaptive Lesson Routing

The Lesson Engine SHOULD ask Pedagogy for routing advice where adaptation is permitted.

```text
Current step
   ↓
completion evidence
   ↓
Student Model
   ↓
Pedagogy Engine
   ↓
routing recommendation
   ↓
Lesson Engine validates against authored branches
```

---

# 28. Curriculum Authority

Adaptive systems SHALL NOT invent arbitrary progression outside curriculum constraints unless operating in freeform mode.

Authored lesson structure defines the permissible learning graph.

---

# 29. Mandatory Steps

Some lesson steps SHALL be non-skippable.

```rust
pub enum StepRequirement {
    Optional,
    Recommended,
    Required,
    MandatoryForCertification,
}
```

---

# 30. Skip Policy

A mastered learner MAY skip some content.

But the engine SHALL honor:

```text
required steps
mandatory assessments
safety-critical demonstrations
certification gates
```

---

# 31. Completion Rule

```rust
pub enum CompletionRule {
    Viewed,
    TutorDelivered,
    StudentAcknowledged,
    CorrectAnswer,
    ObjectiveSatisfied,
    AssessmentPassed,
    LabObjectiveSatisfied,
    MasteryThreshold {
        competency_id: CompetencyId,
        minimum: MasteryScore,
    },
    Composite(Vec<CompletionRule>),
}
```

---

# 32. Completion Is Evidence-Based

A lesson SHALL NOT necessarily complete merely because all screens were viewed.

Completion SHOULD reflect objective satisfaction where the course requires competency.

---

# 33. Lesson Progress

```rust
pub struct LessonProgress {
    pub student_id: StudentId,
    pub lesson_id: LessonId,
    pub version: LessonVersion,

    pub state: LessonProgressState,
    pub current_step_id: Option<LessonStepId>,

    pub completed_steps: Vec<LessonStepId>,
    pub completed_objectives: Vec<LearningObjectiveId>,

    pub started_at: Option<Timestamp>,
    pub last_activity_at: Option<Timestamp>,
    pub completed_at: Option<Timestamp>,
}
```

---

# 34. Progress State

```rust
pub enum LessonProgressState {
    NotStarted,
    InProgress,
    Paused,
    Completed,
    Abandoned,
    Invalidated,
}
```

---

# 35. Resume

Lessons SHALL support resuming.

```text
Session ends
   ↓
LessonProgress persisted
   ↓
new session
   ↓
resume current step
```

---

# 36. Resume Validation

When resuming, the engine SHOULD verify:

```text
same lesson version?
dependencies still valid?
required source versions available?
assessment policy unchanged?
```

---

# 37. Version Change During Progress

If a learner began lesson v1.2 and the course is now v1.3, policy MAY choose:

```text
continue old version
migrate progress
restart changed section
require instructor decision
```

---

# 38. Lesson Migration

```rust
pub struct LessonMigration {
    pub from_version: LessonVersion,
    pub to_version: LessonVersion,
    pub step_mappings: Vec<StepMigration>,
}
```

---

# 39. Step Migration

```rust
pub struct StepMigration {
    pub old_step: LessonStepId,
    pub new_step: Option<LessonStepId>,
    pub preserve_completion: bool,
}
```

---

# 40. Course Progress

```rust
pub struct CourseProgress {
    pub student_id: StudentId,
    pub course_id: CourseId,

    pub completed_lessons: Vec<LessonId>,
    pub active_lesson: Option<LessonId>,

    pub completed_objectives: Vec<LearningObjectiveId>,
    pub state: CourseProgressState,
}
```

---

# 41. Course Progress States

```rust
pub enum CourseProgressState {
    NotStarted,
    InProgress,
    Completed,
    Suspended,
}
```

---

# 42. Course Completion Policy

```rust
pub enum CourseCompletionPolicy {
    AllRequiredLessons,
    AllObjectives,
    RequiredCompetenciesMastered,
    FinalAssessmentPassed,
    Composite(Vec<CourseCompletionCriterion>),
}
```

---

# 43. Completion Gate Example

```text
Course completion requires:

all required lessons
+
final assessment passed
+
competency mastery >= defined thresholds
```

---

# 44. Course Manifest

Every packaged course SHALL have a manifest.

```yaml
course:
  id: networking-fundamentals
  version: 1.0.0
  title: Networking Fundamentals

requires:
  nexa_runtime: ">=0.1.0"

modules:
  - fundamentals
  - transport
  - troubleshooting

knowledge:
  manifest: knowledge/manifest.yaml

assessments:
  manifest: assessments/manifest.yaml

labs:
  manifest: labs/manifest.yaml
```

---

# 45. Course Package

Recommended structure:

```text
course/
├── course.yaml
├── modules/
├── lessons/
├── knowledge/
├── assessments/
├── labs/
├── media/
├── diagrams/
├── code/
└── tests/
```

---

# 46. Module Package

```text
modules/
└── transport/
    ├── module.yaml
    └── lessons/
        ├── tcp-handshake.yaml
        ├── tcp-sequencing.yaml
        └── udp.yaml
```

---

# 47. Lesson File

Example:

```yaml
lesson:
  id: tcp-handshake
  title: TCP Three-Way Handshake
  version: 1.0.0

objectives:
  - tcp.handshake.sequence
  - tcp.handshake.purpose

steps:
  - id: intro
    kind: introduction

  - id: diagram
    kind: explanation

  - id: verify
    kind: question

  - id: practice
    kind: practice
```

---

# 48. Content Authoring Format

Human-authored YAML or Markdown SHOULD be supported initially.

The source format SHOULD remain readable in Git.

---

# 49. Markdown Lesson Authoring

A lesson MAY use frontmatter:

```markdown
---
lesson: tcp-handshake
version: 1.0.0
objectives:
  - tcp.handshake.sequence
---

# TCP Three-Way Handshake

## Introduction

...

## Practice

...
```

A compiler can convert this into the canonical lesson model.

---

# 50. Lesson Compiler

```rust
pub trait LessonCompiler {
    fn compile(
        &self,
        source: LessonSource,
    ) -> LessonResult<CompiledLesson>;
}
```

---

# 51. Compiled Lesson

Compiled lessons SHOULD be normalized, validated runtime artifacts.

The runtime SHOULD not need to parse complex authoring syntax during delivery.

---

# 52. Authoring Versus Runtime Format

```text
Human-readable source
      ↓
Lesson compiler
      ↓
Validated canonical format
      ↓
Runtime
```

This separation enables better validation and tooling.

---

# 53. Content IDs

Content elements SHOULD have stable semantic IDs.

Example:

```text
lesson.tcp-handshake.diagram.main
lesson.tcp-handshake.question.verify-order
```

Stable IDs help preserve progress across minor edits.

---

# 54. Knowledge Bindings

Lessons SHOULD bind to approved knowledge scopes.

```rust
pub struct LessonKnowledgeBinding {
    pub source_ids: Vec<KnowledgeSourceId>,
    pub concept_ids: Vec<ConceptId>,
    pub preferred_roles: Vec<KnowledgeRole>,
}
```

---

# 55. Why Bind Knowledge

A lesson on TCP SHOULD not accidentally retrieve unrelated but semantically similar content from an unrelated course.

Bindings provide retrieval guardrails.

---

# 56. Knowledge Role Bindings

Examples:

```text
Explanation step
   → Definition, Explanation

Practice step
   → Example, Procedure

Remediation step
   → Remediation, Contrast
```

---

# 57. Assessment Binding

```rust
pub struct AssessmentBinding {
    pub assessment_id: AssessmentId,
    pub role: AssessmentRole,
}
```

---

# 58. Assessment Roles

```rust
pub enum AssessmentRole {
    Diagnostic,
    Checkpoint,
    Practice,
    Summative,
    Certification,
}
```

---

# 59. Lab Binding

```rust
pub struct LabBinding {
    pub lab_id: LabId,
    pub objectives: Vec<LearningObjectiveId>,
}
```

---

# 60. Lab Integration

A lesson step MAY launch a lab.

```text
Lesson
   ↓
Lab step
   ↓
Orchestrator starts lab
   ↓
student activity
   ↓
lab evidence
   ↓
Student Model
   ↓
completion decision
```

---

# 61. Practice Activity

```rust
pub struct PracticeActivity {
    pub target_competencies: Vec<CompetencyId>,
    pub question_pool: Option<QuestionPoolId>,
    pub target_attempts: Option<u32>,
    pub completion_rule: CompletionRule,
}
```

---

# 62. Question Pools

Questions SHOULD support pools instead of always using a fixed sequence.

This enables variation.

```rust
pub struct QuestionPool {
    pub id: QuestionPoolId,
    pub questions: Vec<QuestionId>,
    pub selection_policy: QuestionSelectionPolicy,
}
```

---

# 63. Question Selection Policy

```rust
pub enum QuestionSelectionPolicy {
    Sequential,
    Random,
    Adaptive,
    UnseenFirst,
    WeaknessTargeted,
}
```

---

# 64. Adaptive Question Selection

The Pedagogy Engine MAY recommend:

```text
target competency
difficulty
purpose
```

The Lesson Engine selects an allowed question from the authored pool.

---

# 65. Dynamic Question Generation

Courses MAY permit AI-generated variants.

```rust
pub enum QuestionSourcePolicy {
    AuthoredOnly,
    AuthoredPreferred,
    GeneratedAllowed,
}
```

---

# 66. Generated Question Constraints

AI-generated questions SHALL remain bound by:

```text
competency
difficulty
course scope
knowledge sources
assessment policy
```

---

# 67. Generated Content Validation

Generated instructional content SHOULD be validated before presentation when it can affect grading or critical technical correctness.

---

# 68. Lesson Modes

```rust
pub enum LessonDeliveryMode {
    Guided,
    SelfPaced,
    TutorLed,
    LabHeavy,
    AssessmentFocused,
    Adaptive,
}
```

---

# 69. Guided Mode

Nexa controls progression closely.

---

# 70. Self-Paced Mode

Students can navigate more freely, subject to required-step constraints.

---

# 71. Adaptive Mode

The system may alter pacing and branch selection based on learner state.

---

# 72. Lesson State Machine

```text
NOT_STARTED
    ↓
INITIALIZING
    ↓
ACTIVE
    ├── PAUSED
    │    ↓
    │  ACTIVE
    │
    ├── REMEDIATION
    ├── CHALLENGE
    └── ASSESSMENT
          ↓
      COMPLETING
          ↓
       COMPLETED
```

---

# 73. Lesson Runtime

```rust
pub struct LessonRuntime {
    pub lesson: CompiledLesson,
    pub progress: LessonProgress,
    pub current_step: Option<LessonStepId>,
    pub active_branch: Option<LessonBranchId>,
}
```

---

# 74. Lesson Engine Contract

```rust
#[async_trait]
pub trait LessonEngine: Send + Sync {
    async fn start(
        &self,
        request: StartLessonRequest,
    ) -> LessonResult<LessonRuntime>;

    async fn next(
        &self,
        request: LessonAdvanceRequest,
    ) -> LessonResult<LessonNavigationDecision>;

    async fn resume(
        &self,
        request: ResumeLessonRequest,
    ) -> LessonResult<LessonRuntime>;
}
```

---

# 75. Navigation Decision

```rust
pub struct LessonNavigationDecision {
    pub current_step: LessonStepId,
    pub next_step: Option<LessonStepId>,
    pub branch: Option<LessonBranchId>,
    pub reason: NavigationReason,
}
```

---

# 76. Navigation Reason

```rust
pub enum NavigationReason {
    NormalProgression,
    ObjectiveCompleted,
    RemediationRequired,
    ChallengeAvailable,
    AssessmentOutcome,
    PrerequisiteFailure,
    StudentChoice,
}
```

---

# 77. Lesson Execution Flow

```text
Load lesson
   ↓
validate prerequisites
   ↓
start step
   ↓
Nexa delivers activity
   ↓
collect learner evidence
   ↓
evaluate completion
   ↓
Pedagogy recommendation
   ↓
select valid next step
   ↓
continue
```

---

# 78. Prerequisite Check

Before starting a lesson:

```text
lesson prerequisites
      ↓
Student Model
      ↓
ready?
```

Possible results:

```text
ready
review first
remediate
blocked
```

---

# 79. Review Before Start

A student with borderline prerequisite mastery MAY receive a brief review branch rather than being denied entry.

---

# 80. Remediation Branch

Remediation branches SHOULD be authored or generated within explicit constraints.

Example:

```text
main step
   ↓ failure pattern
remediation A
   ↓
verification
   ↓
return to main flow
```

---

# 81. Challenge Branch

High-performing learners MAY receive:

```text
advanced example
transfer problem
optional lab
deeper explanation
```

without delaying required progression.

---

# 82. Branch Reentry

Every branch SHOULD declare where it returns.

Avoid ambiguous branch endings.

```rust
pub struct BranchExit {
    pub return_step: Option<LessonStepId>,
    pub completes_lesson: bool,
}
```

---

# 83. Infinite Loop Protection

The Lesson Engine SHALL detect cycles such as:

```text
remediation A
   ↓
main question
   ↓
remediation A
```

repeating indefinitely.

---

# 84. Branch Attempt Limits

Policies MAY specify:

```rust
pub struct BranchExecutionPolicy {
    pub maximum_entries: Option<u32>,
}
```

---

# 85. Escalation After Repeated Failure

After repeated unsuccessful remediation:

```text
alternate representation
deeper prerequisite remediation
instructor escalation
pause lesson
```

may be preferable to infinite repetition.

---

# 86. Lesson Events

Canonical events include:

```text
course.started
course.completed

module.started
module.completed

lesson.loaded
lesson.started
lesson.paused
lesson.resumed
lesson.completed
lesson.abandoned

lesson.step.started
lesson.step.completed
lesson.step.skipped

lesson.objective.started
lesson.objective.completed

lesson.branch.entered
lesson.branch.exited

lesson.remediation.started
lesson.remediation.completed

lesson.challenge.started
lesson.challenge.completed
```

---

# 87. Lesson Started Example

```json
{
  "event_type": "lesson.started",
  "payload": {
    "lesson_id": "tcp-handshake",
    "version": "1.0.0"
  }
}
```

---

# 88. Branch Event

```json
{
  "event_type": "lesson.branch.entered",
  "payload": {
    "lesson_id": "tcp-handshake",
    "branch_id": "remediate-sequence",
    "reason": "low_mastery"
  }
}
```

---

# 89. Objective Completion Event

```json
{
  "event_type": "lesson.objective.completed",
  "payload": {
    "objective_id": "tcp.handshake.sequence",
    "evidence_ids": [
      "ev-101",
      "ev-104"
    ]
  }
}
```

---

# 90. Progress Persistence

Important progression state SHALL be persisted transactionally.

At minimum:

```text
current lesson
current step
completed steps
completed objectives
branch state
lesson version
```

---

# 91. Lesson Event Sourcing

Lesson progress MAY be rebuilt from events.

```text
lesson.started
step.started
step.completed
branch.entered
step.completed
...
```

A projection SHOULD still exist for fast runtime use.

---

# 92. Progress Snapshotting

Long course histories MAY use snapshots.

---

# 93. Course Versioning

Course versions SHOULD use semantic versioning where appropriate.

```text
1.0.0
1.1.0
2.0.0
```

---

# 94. Major Version Change

A major course version MAY indicate:

```text
learning objectives changed
lesson graph changed significantly
assessment requirements changed
```

---

# 95. Minor Version Change

May indicate:

```text
new example
clarification
additional optional content
```

while preserving progression compatibility.

---

# 96. Patch Change

Usually:

```text
typos
minor metadata fixes
broken links
```

---

# 97. Content Dependency Versioning

A course manifest SHOULD track:

```text
knowledge pack
assessment pack
lab pack
media pack
```

versions independently.

---

# 98. Reproducibility

A historical session SHOULD be able to identify:

```text
course version
lesson version
knowledge version
assessment version
lab version
```

used during delivery.

---

# 99. Course Validation

Before publication, a course SHALL pass validation.

---

# 100. Structural Validation

Check:

```text
unique IDs
valid references
reachable steps
branch targets exist
no invalid cycles
objectives exist
competencies exist
```

---

# 101. Pedagogical Validation

Check for:

```text
objective coverage
required practice
assessment alignment
missing remediation
missing prerequisite mapping
```

where policies require them.

---

# 102. Knowledge Validation

Check:

```text
lesson knowledge bindings exist
sources are active
protected knowledge isn't exposed
required citations resolvable
```

---

# 103. Lab Validation

Check:

```text
bound lab exists
required capabilities available
lab objective mappings valid
```

---

# 104. Assessment Validation

Check:

```text
assessment exists
objective mapping valid
passing rule valid
policy compatible with lesson
```

---

# 105. Behavior Cue Validation

Check:

```text
gaze target exists
canvas object exists
gesture semantically valid
```

Missing optional animation capability MAY generate warnings rather than failure.

---

# 106. Course Linter

A CLI tool SHOULD eventually support:

```text
nexa course validate ./course
```

---

# 107. Validation Severity

```rust
pub enum CourseValidationSeverity {
    Info,
    Warning,
    Error,
    Fatal,
}
```

---

# 108. Publishing Pipeline

```text
author
  ↓
compile
  ↓
validate
  ↓
run tests
  ↓
build package
  ↓
sign
  ↓
publish
```

---

# 109. Course Package Manifest

A package SHOULD identify cryptographic hashes for important files.

This supports integrity checking.

---

# 110. Signed Course Packages

Future enterprise deployments SHOULD support signed packages.

```text
publisher signature
   ↓
runtime verification
```

---

# 111. Package Trust

Untrusted course packages SHALL not automatically gain:

```text
tool access
lab privileges
filesystem access
network access
```

Course content remains data.

---

# 112. Lesson Scripts SHALL NOT Execute Arbitrary Code

Authoring syntax SHALL not allow unrestricted embedded runtime scripting.

Any dynamic actions SHALL map to approved typed capabilities.

---

# 113. Tool Action Templates

A lesson MAY request a tool capability semantically.

```rust
pub struct ToolRequestTemplate {
    pub tool_id: ToolId,
    pub operation: String,
    pub arguments_template: JsonValue,
}
```

The Orchestrator still enforces authorization.

---

# 114. Content Variables

Lesson content MAY support safe substitution.

Example:

```text
{{student.current_lab_ip}}
```

Variable resolution SHALL use whitelisted values.

---

# 115. No Arbitrary Template Evaluation

The template engine SHALL not execute arbitrary code.

---

# 116. Lesson Localization

Course content SHOULD be localizable.

```rust
pub struct LocalizedString {
    pub key: String,
    pub fallback: String,
}
```

---

# 117. Technical Terms

Localization SHOULD preserve:

```text
code
commands
API names
protocol names
identifiers
```

unless explicit localized equivalents exist.

---

# 118. Accessibility Metadata

Lesson content SHOULD support:

```text
alt text
captions
transcripts
keyboard navigation
screen-reader labels
reduced-motion alternatives
```

---

# 119. Media Content

```rust
pub struct MediaContent {
    pub artifact_id: ArtifactId,
    pub media_type: MediaType,
    pub transcript_id: Option<ArtifactId>,
    pub captions_id: Option<ArtifactId>,
}
```

---

# 120. Media Types

```rust
pub enum MediaType {
    Image,
    Audio,
    Video,
    Animation,
}
```

---

# 121. Adaptive Representation

Pedagogy MAY choose among multiple equivalent representations.

Example:

```text
concept explanation:
  text
  diagram
  worked example
```

The lesson can author all three and let pedagogy select.

---

# 122. Content Variant

```rust
pub struct ContentVariant {
    pub id: ContentVariantId,
    pub representation: InstructionRepresentation,
    pub difficulty: Difficulty,
    pub content: LessonStepContent,
}
```

---

# 123. Variant Selection

Selection MAY depend on:

```text
student preference
prior failure
mastery
lesson policy
device capability
```

---

# 124. Device Capability

A lesson requiring 3D visualization SHOULD provide fallback if the runtime lacks 3D support.

---

# 125. Offline Courses

Course packages SHOULD be capable of containing everything necessary for local delivery:

```text
lesson content
knowledge
media
assessments
labs where feasible
```

---

# 126. Course Dependency Manifest

Example:

```yaml
dependencies:
  knowledge:
    - networking-core@1.3.0

  labs:
    - tcp-sandbox@2.0.0

  assessments:
    - tcp-assessments@1.1.0
```

---

# 127. Dependency Resolution

The runtime SHALL verify dependencies before starting the course.

---

# 128. Missing Dependency

Possible behavior:

```text
required → block course
optional → degrade
```

---

# 129. Course Catalog

The platform SHOULD eventually expose:

```text
course title
version
description
objectives
prerequisites
estimated scope
installed status
update status
```

---

# 130. Course Enrollment

Enrollment is optional for local single-user deployments but SHOULD exist as a logical concept.

```rust
pub struct CourseEnrollment {
    pub student_id: StudentId,
    pub course_id: CourseId,
    pub enrolled_at: Timestamp,
}
```

---

# 131. Curriculum Graph

Courses MAY have prerequisite relationships.

```text
Networking Fundamentals
      ↓
Network Troubleshooting
      ↓
Advanced Packet Analysis
```

---

# 132. Curriculum Prerequisite

```rust
pub struct CoursePrerequisite {
    pub course_id: CourseId,
    pub minimum_completion: Option<bool>,
    pub competency_requirements: Vec<CompetencyRequirement>,
}
```

---

# 133. Recommended Course

Future systems MAY recommend a next course based on:

```text
goals
competency gaps
completed courses
prerequisites
```

Recommendation logic SHOULD remain outside core lesson execution.

---

# 134. Lesson Duration

Authored lessons MAY include estimated duration.

It SHALL remain advisory because adaptive paths vary.

---

# 135. No Rigid Time Assumption

A struggling learner may require remediation.

A proficient learner may skip mastered content.

Therefore lesson duration is not deterministic.

---

# 136. Course Analytics

Useful metrics include:

```text
lesson completion rate
objective completion rate
time per lesson
remediation frequency
challenge branch usage
drop-off step
assessment outcomes
mastery gain
```

---

# 137. Content Effectiveness

A course author should eventually be able to ask:

```text
Which lesson steps produce repeated misconceptions?
Which remediation branch actually works?
Which question is too easy?
Where do learners stall?
```

---

# 138. Step Effectiveness

A step MAY be evaluated against subsequent evidence.

This helps improve authored curriculum.

---

# 139. Content Revision Feedback Loop

```text
learner evidence
      ↓
analytics
      ↓
course author review
      ↓
lesson revision
      ↓
new version
```

---

# 140. AI-Assisted Authoring

Future tools MAY help authors generate:

```text
lesson drafts
examples
questions
remediation variants
summaries
```

but generated content SHALL pass course validation.

---

# 141. Course Authoring Assistant

Nexa itself may eventually assist instructors in course construction under a separate authoring role.

Learner-facing and authoring permissions SHALL remain separate.

---

# 142. Course Test Scenarios

Each lesson SHOULD support scripted tests.

Example:

```text
student mastery low
   → remediation expected

student mastery high
   → challenge expected

assessment pass
   → continue

assessment fail
   → branch
```

---

# 143. Synthetic Learner Testing

Course packages SHOULD eventually be testable against synthetic learner profiles.

---

# 144. Path Coverage

A lesson validator SHOULD report which branches have automated test coverage.

Example:

```text
main path            covered
remediation A        covered
remediation B        missing
challenge branch     covered
```

---

# 145. Unreachable Content

Any step unreachable under all valid conditions SHOULD produce a validation error or warning.

---

# 146. Orphan Objective

An objective with no teaching or assessment coverage SHOULD be flagged.

---

# 147. Over-Assessed Objective

A course may also detect objectives with excessive assessment relative to instruction.

This is a warning rather than a hard structural error.

---

# 148. Lesson Runtime Events and Tutor Flow

Typical sequence:

```text
lesson.step.started
      ↓
TutorContext includes step
      ↓
Pedagogy decision
      ↓
TutorResponse
      ↓
student interaction
      ↓
evidence
      ↓
completion rule evaluated
      ↓
lesson.step.completed
      ↓
navigation decision
```

---

# 149. Lesson-to-Tutor Contract

The Tutor Engine receives:

```text
current objective
current activity
lesson constraints
knowledge bindings
authored intent
```

It SHALL not receive uncontrolled authoring markup.

---

# 150. Lesson-to-Pedagogy Contract

Pedagogy receives:

```text
current objective
allowed branches
student state
lesson strategy hints
required completion rules
```

---

# 151. Lesson-to-Student Model Contract

The Lesson subsystem does not update mastery directly.

It emits or triggers learning evidence through the appropriate evaluator.

---

# 152. Lesson-to-Assessment Contract

Assessment steps delegate scoring and grading to the Assessment Engine.

---

# 153. Lesson-to-Lab Contract

Lab steps delegate execution and observation to the Lab subsystem.

---

# 154. Lesson-to-Knowledge Contract

Knowledge retrieval SHALL be constrained using lesson/course bindings where appropriate.

---

# 155. Lesson-to-Avatar Contract

Lesson steps MAY provide semantic presentation cues.

They SHALL not contain rig-specific animation code.

---

# 156. MVP Course Scope

The first implementation SHOULD support:

```text
one course
one module
several lessons

step kinds:
  introduction
  explanation
  question
  practice
  summary

simple branches:
  normal
  remediation
  challenge

completion:
  viewed
  correct answer
  objective satisfied

resume
versioned YAML
```

---

# 157. MVP Example Course

```text
Networking Fundamentals
│
└── Transport Module
    │
    ├── TCP Handshake
    ├── TCP Sequencing
    └── UDP Basics
```

---

# 158. MVP TCP Lesson

```text
1. Introduction
2. Diagram explanation
3. Verification question
4. Practice question
5. Remediation if needed
6. Challenge if mastery high
7. Summary
```

---

# 159. MVP Branch Example

```text
Verification question
    │
    ├── correct
    │      ↓
    │   practice
    │
    └── incorrect
           ↓
       remediation
           ↓
       retry
```

---

# 160. MVP Resume Example

Student completes steps 1–3 and exits.

Next session:

```text
load LessonProgress
      ↓
step 4
```

unless lesson-version migration requires otherwise.

---

# 161. First End-to-End Course Scenario

```text
Student starts TCP lesson
      ↓
Lesson Engine loads step 1
      ↓
Nexa introduces objective
      ↓
Step 2 shows handshake diagram
      ↓
Nexa points to SYN / SYN-ACK / ACK
      ↓
Step 3 asks verification question
      ↓
Student answers incorrectly
      ↓
Student Model records evidence
      ↓
Pedagogy selects remediation
      ↓
Lesson Engine enters remediation branch
      ↓
Nexa explains with alternate representation
      ↓
Student retries successfully
      ↓
Lesson continues
      ↓
Objective completed
      ↓
Lesson completed
```

This is the first truly adaptive authored Nexa lesson.

---

# 162. Recommended Crate Structure

```text
crates/
└── nexa-lessons/
    ├── src/
    │   ├── lib.rs
    │   ├── course.rs
    │   ├── module.rs
    │   ├── lesson.rs
    │   ├── step.rs
    │   ├── objective.rs
    │   ├── content.rs
    │   ├── branch.rs
    │   ├── condition.rs
    │   ├── completion.rs
    │   ├── progress.rs
    │   ├── navigation.rs
    │   ├── compiler.rs
    │   ├── manifest.rs
    │   ├── validation.rs
    │   ├── migration.rs
    │   ├── package.rs
    │   ├── errors.rs
    │   └── authoring/
    │       ├── markdown.rs
    │       └── yaml.rs
    └── tests/
        ├── compile.rs
        ├── validation.rs
        ├── progression.rs
        ├── branching.rs
        ├── resume.rs
        └── migration.rs
```

---

# 163. Dependency Direction

```text
             nexa-domain
                 │
                 ▼
            nexa-lessons
           /      |       \
          ▼       ▼        ▼
   nexa-knowledge pedagogy assessments
          \       |        /
           \      |       /
              orchestrator
```

The Lesson Engine remains a curriculum/runtime structure component.

---

# 164. Lesson System Invariants

`NEXA-LESSON-001` establishes these invariants:

1. Courses SHALL define explicit learning objectives.
2. Learning objectives SHOULD map to competencies.
3. Lesson definitions SHALL remain distinct from learner progress.
4. Lesson content SHALL be versioned.
5. Course packages SHALL be versioned.
6. Lesson progression SHALL be deterministic for a given state and policy.
7. Adaptive routing SHALL remain within authored curriculum constraints.
8. Mandatory steps SHALL not be bypassed by ordinary adaptation.
9. Completion SHALL support evidence-based rules.
10. Viewing content alone SHALL not imply mastery.
11. Lessons SHALL support resume.
12. Progress SHALL record the lesson version.
13. Version migration SHALL be explicit.
14. Knowledge bindings SHALL constrain retrieval where appropriate.
15. Assessment bindings SHALL delegate grading to the Assessment Engine.
16. Lab bindings SHALL delegate execution to the Lab Engine.
17. Tool execution SHALL remain authorization-controlled.
18. Avatar behavior cues SHALL remain semantic.
19. Course authoring formats SHOULD remain human-readable and Git-friendly.
20. Runtime lessons SHOULD use compiled validated artifacts.
21. Course packages SHALL treat content as data, not unrestricted executable code.
22. Lesson branches SHALL avoid unbounded loops.
23. Branch targets SHALL be valid and testable.
24. Objectives without instructional or assessment coverage SHOULD be flagged.
25. Stable semantic IDs SHOULD support progress preservation.
26. Offline course packaging SHOULD be first-class.
27. Accessibility metadata SHOULD be part of content authoring.
28. Adaptive representation MAY vary modality without changing facts.
29. Course analytics SHOULD support continuous curriculum improvement.
30. Authored curriculum SHALL define what must be learned; Pedagogy SHALL decide how best to get there.

---

# 165. Architecture Status

Nexa can now support both freeform and authored tutoring:

```text
                    STUDENT
                       │
                       ▼
                NEXA-LESSON-001
                 Learning Flow
                       │
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
 Student Model      Pedagogy       Knowledge
        │              │              │
        └──────────────┼──────────────┘
                       ▼
                 Tutor Engine
                       │
                       ▼
                  Orchestrator
                       │
            ┌──────────┼──────────┐
            ▼          ▼          ▼
          Speech      Avatar     Canvas
```

Nexa is no longer just capable of answering questions. She now has an architecture for **delivering structured courses, adapting the path, remediating weaknesses, challenging advanced learners, and resuming progress across sessions**.

---

# 166. Next Specification

The next specification should be:

# **NEXA-ASMT-001 — Assessment, Question, Evaluation, Grading & Mastery Evidence Architecture v1.0**

It should define:

```text
diagnostic assessments
formative assessments
summative assessments
certification assessments
question models
question banks
item variants
free-text evaluation
code evaluation
command evaluation
lab-based assessment
rubrics
partial credit
attempts
timers
hint restrictions
answer-key protection
grading
feedback policy
assessment security
randomization
item selection
difficulty calibration
evidence generation
mastery integration
question analytics
assessment validity
reliability
versioning
regrade support
auditability
```

That is the next major step because the lesson system now tells Nexa **what to teach**, and the assessment system needs to determine **what the learner can actually demonstrate**.
