# NEXA-DOM-001 — Nexa Core Domain Model & Type System v1.0

**Specification ID:** NEXA-DOM-001
**System:** Nexa AI Training Tutor
**Version:** 1.0
**Status:** Baseline Draft
**Purpose:** Define the canonical domain concepts, identifiers, value objects, state models, Rust types, invariants, and subsystem contracts shared across the Nexa platform.

---

## 1. Purpose

The Nexa platform contains multiple independently evolving subsystems:

* tutor intelligence;
* pedagogy;
* student modeling;
* lessons;
* assessments;
* competencies;
* knowledge retrieval;
* memory;
* labs;
* tools;
* speech;
* avatar behavior;
* persistence;
* orchestration.

Those systems require a common vocabulary and a stable set of types.

This specification defines that shared domain model.

The primary architectural goal is to prevent concepts such as `Student`, `Lesson`, `Competency`, `Assessment`, or `TutorResponse` from being independently redefined by different components.

---

# 2. Design Principles

The core domain model SHALL favor:

1. strong typing;
2. explicit invariants;
3. semantic identifiers;
4. immutable value objects where practical;
5. minimal coupling;
6. domain-specific enums;
7. controlled extensibility;
8. serialization stability;
9. technology independence;
10. compile-time correctness where possible.

---

# 3. Domain Architecture

```text
NEXA DOMAIN MODEL
│
├── Identity
│   ├── IDs
│   ├── timestamps
│   └── versioning
│
├── Learning
│   ├── Student
│   ├── Course
│   ├── Lesson
│   ├── Objective
│   └── Competency
│
├── Pedagogy
│   ├── Strategy
│   ├── Hint
│   ├── Misconception
│   └── Evidence
│
├── Assessment
│   ├── Question
│   ├── Answer
│   ├── Attempt
│   └── Result
│
├── Tutor
│   ├── TutorRequest
│   ├── TutorResponse
│   ├── BehaviorIntent
│   └── ExplanationDepth
│
├── Knowledge
│   ├── KnowledgeSource
│   ├── Concept
│   └── RetrievalResult
│
├── Runtime
│   ├── Session
│   ├── Tool
│   ├── Lab
│   └── Artifact
│
└── Shared
    ├── Result
    ├── Error
    ├── metadata
    └── confidence
```

---

# 4. Newtype Rule

Primitive values SHOULD NOT be passed around the system when their semantic meaning is important.

Avoid:

```rust
fn load_lesson(id: String)
```

Prefer:

```rust
fn load_lesson(id: LessonId)
```

This prevents accidental substitution of unrelated identifiers.

---

# 5. Identifier Strategy

Identifiers SHOULD use strongly typed UUID-based newtypes.

Example:

```rust
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct StudentId(pub Uuid);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub Uuid);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct CourseId(pub Uuid);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct LessonId(pub Uuid);
```

UUIDv7 is recommended for newly generated identifiers where available.

---

# 6. Core Identifier Types

The baseline model SHOULD define:

```text
StudentId
SessionId
CourseId
LessonId
LessonStepId
LearningObjectiveId
ConceptId
CompetencyId
EvidenceId
QuestionId
AnswerId
AttemptId
AssessmentId
HintId
MisconceptionId
TutorRequestId
TutorResponseId
BehaviorId
ToolId
ToolExecutionId
LabId
ArtifactId
KnowledgeSourceId
MemoryEntryId
MessageId
EventId
CorrelationId
TraceId
```

---

# 7. Human-Readable Keys

UUIDs identify entities.

Human-readable keys MAY provide stable authored identifiers.

Example:

```rust
pub struct CompetencyKey(String);
```

Example value:

```text
networking.tcp.handshake
```

Both MAY coexist:

```rust
pub struct CompetencyIdentity {
    pub id: CompetencyId,
    pub key: CompetencyKey,
}
```

---

# 8. Version Type

Versioned domain entities SHOULD use a shared version representation.

```rust
pub struct Revision(pub u64);
```

or semantic versioning where schema compatibility matters:

```rust
pub struct SchemaVersion {
    pub major: u16,
    pub minor: u16,
}
```

---

# 9. Timestamp Types

All persistent timestamps SHALL be timezone-aware UTC values.

```rust
pub type Timestamp = DateTime<Utc>;
```

Domain-specific times MAY use wrapper types when semantics differ.

---

# 10. Confidence Type

Confidence values SHALL not be represented as unconstrained floats.

```rust
pub struct Confidence(f32);
```

Invariant:

```text
0.0 <= confidence <= 1.0
```

Construction SHALL validate the range.

---

# 11. Mastery Score

Competency mastery SHALL use a dedicated value type.

```rust
pub struct MasteryScore(f32);
```

Range:

```text
0.0 = no demonstrated mastery
1.0 = demonstrated mastery
```

This represents an estimate, not an absolute truth.

---

# 12. Student

A Student represents the learner within the Nexa platform.

```rust
pub struct Student {
    pub id: StudentId,
    pub profile: StudentProfile,
    pub preferences: LearningPreferences,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
```

---

# 13. Student Profile

```rust
pub struct StudentProfile {
    pub display_name: Option<String>,
    pub locale: Option<String>,
    pub timezone: Option<String>,
}
```

The core profile SHOULD remain intentionally small.

Extended personal data SHALL belong to separate optional profile modules.

---

# 14. Learning Preferences

```rust
pub struct LearningPreferences {
    pub explanation_depth: ExplanationDepth,
    pub preferred_modalities: Vec<LearningModality>,
    pub pacing: LearningPace,
    pub challenge_level: ChallengePreference,
}
```

---

# 15. Learning Modality

```rust
pub enum LearningModality {
    Text,
    Voice,
    Diagram,
    Demonstration,
    InteractiveLab,
    Quiz,
    SocraticDialogue,
}
```

The model SHOULD permit multiple simultaneous modalities.

---

# 16. Learning Pace

```rust
pub enum LearningPace {
    Slow,
    Moderate,
    Fast,
    Adaptive,
}
```

---

# 17. Challenge Preference

```rust
pub enum ChallengePreference {
    Supportive,
    Balanced,
    Challenging,
    HighlyChallenging,
    Adaptive,
}
```

---

# 18. Session

A Session represents one contiguous tutoring interaction.

```rust
pub struct Session {
    pub id: SessionId,
    pub student_id: StudentId,
    pub course_id: Option<CourseId>,
    pub lesson_id: Option<LessonId>,
    pub mode: SessionMode,
    pub state: SessionState,
    pub started_at: Timestamp,
    pub ended_at: Option<Timestamp>,
}
```

---

# 19. Session Mode

```rust
pub enum SessionMode {
    Freeform,
    GuidedLesson,
    Assessment,
    Lab,
    Review,
    Practice,
    Debugging,
}
```

---

# 20. Session State

```rust
pub enum SessionState {
    Created,
    Starting,
    Active,
    Paused,
    Ending,
    Completed,
    Failed,
}
```

State transitions SHOULD be validated.

---

# 21. Course

```rust
pub struct Course {
    pub id: CourseId,
    pub key: String,
    pub title: String,
    pub description: String,
    pub version: Revision,
    pub objectives: Vec<LearningObjectiveId>,
    pub lessons: Vec<LessonId>,
}
```

A Course SHOULD describe instructional structure, not runtime state.

---

# 22. Lesson

```rust
pub struct Lesson {
    pub id: LessonId,
    pub course_id: CourseId,
    pub key: String,
    pub title: String,
    pub description: String,
    pub objectives: Vec<LearningObjectiveId>,
    pub prerequisites: Vec<CompetencyRequirement>,
    pub steps: Vec<LessonStep>,
    pub version: Revision,
}
```

---

# 23. Lesson Step

```rust
pub struct LessonStep {
    pub id: LessonStepId,
    pub kind: LessonStepKind,
    pub objective_ids: Vec<LearningObjectiveId>,
    pub content: LessonStepContent,
    pub completion: CompletionRule,
}
```

---

# 24. Lesson Step Kind

```rust
pub enum LessonStepKind {
    Introduction,
    Explanation,
    Demonstration,
    Question,
    Practice,
    Lab,
    Assessment,
    Reflection,
    Summary,
}
```

---

# 25. Lesson Step Content

A lesson step SHOULD use typed content.

```rust
pub enum LessonStepContent {
    Text(TextContent),
    Diagram(DiagramContent),
    Demonstration(DemonstrationContent),
    Question(QuestionId),
    Lab(LabId),
    Assessment(AssessmentId),
    Composite(Vec<LessonStepContent>),
}
```

---

# 26. Completion Rule

```rust
pub enum CompletionRule {
    Viewed,
    Acknowledged,
    CorrectAnswer,
    ObjectiveSatisfied,
    LabObjectiveSatisfied,
    AssessmentThreshold(MasteryScore),
    ExplicitInstructorDecision,
}
```

---

# 27. Learning Objective

A Learning Objective defines what the learner should be able to demonstrate.

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

# 28. Success Criterion

```rust
pub enum SuccessCriterion {
    ExplainConcept {
        concept_id: ConceptId,
    },
    AnswerCorrectly {
        question_ids: Vec<QuestionId>,
    },
    PerformProcedure {
        competency_id: CompetencyId,
    },
    CompleteLabObjective {
        lab_id: LabId,
        objective_key: String,
    },
    DemonstrateMastery {
        competency_id: CompetencyId,
        minimum: MasteryScore,
    },
}
```

---

# 29. Concept

A Concept represents a unit of knowledge.

```rust
pub struct Concept {
    pub id: ConceptId,
    pub key: String,
    pub name: String,
    pub description: String,
    pub relationships: Vec<ConceptRelationship>,
}
```

---

# 30. Concept Relationship

```rust
pub enum ConceptRelationshipType {
    Requires,
    Contains,
    PartOf,
    ContrastsWith,
    RelatedTo,
    Enables,
    Precedes,
}
```

```rust
pub struct ConceptRelationship {
    pub relationship: ConceptRelationshipType,
    pub target: ConceptId,
}
```

---

# 31. Competency

A Competency represents an ability the learner can demonstrate.

```rust
pub struct Competency {
    pub id: CompetencyId,
    pub key: CompetencyKey,
    pub title: String,
    pub description: String,
    pub concept_ids: Vec<ConceptId>,
    pub parent: Option<CompetencyId>,
}
```

---

# 32. Competency Requirement

```rust
pub struct CompetencyRequirement {
    pub competency_id: CompetencyId,
    pub minimum_mastery: MasteryScore,
}
```

Used for:

* prerequisites;
* certification;
* adaptive lesson selection.

---

# 33. Student Competency State

The competency definition and a student's current competency estimate SHALL remain separate.

```rust
pub struct StudentCompetency {
    pub student_id: StudentId,
    pub competency_id: CompetencyId,
    pub mastery: MasteryScore,
    pub confidence: Confidence,
    pub evidence_count: u32,
    pub last_evaluated_at: Timestamp,
}
```

---

# 34. Evidence

Competency changes SHALL be traceable to evidence.

```rust
pub struct Evidence {
    pub id: EvidenceId,
    pub student_id: StudentId,
    pub competency_id: CompetencyId,
    pub kind: EvidenceKind,
    pub strength: EvidenceStrength,
    pub source: EvidenceSource,
    pub observed_at: Timestamp,
}
```

---

# 35. Evidence Kind

```rust
pub enum EvidenceKind {
    CorrectAnswer,
    IncorrectAnswer,
    Explanation,
    Demonstration,
    LabExecution,
    DebuggingPerformance,
    AssessmentResult,
    RetentionCheck,
    TransferTask,
    InstructorEvaluation,
}
```

---

# 36. Evidence Strength

```rust
pub enum EvidenceStrength {
    Weak,
    Moderate,
    Strong,
    Conclusive,
}
```

The exact scoring model is intentionally outside this specification.

---

# 37. Evidence Source

```rust
pub enum EvidenceSource {
    Question(QuestionId),
    Assessment(AssessmentId),
    Lab(LabId),
    Lesson(LessonId),
    TutorObservation(TutorResponseId),
    External(String),
}
```

---

# 38. Question

```rust
pub struct Question {
    pub id: QuestionId,
    pub prompt: String,
    pub kind: QuestionKind,
    pub competency_ids: Vec<CompetencyId>,
    pub difficulty: Difficulty,
    pub evaluation: EvaluationRule,
    pub hints: Vec<HintId>,
}
```

---

# 39. Question Kind

```rust
pub enum QuestionKind {
    MultipleChoice,
    MultipleSelect,
    TrueFalse,
    ShortAnswer,
    LongAnswer,
    Numeric,
    Code,
    Command,
    Interactive,
    Explanation,
}
```

---

# 40. Difficulty

```rust
pub enum Difficulty {
    Introductory,
    Basic,
    Intermediate,
    Advanced,
    Expert,
}
```

A future adaptive system MAY supplement this with a numerical difficulty model.

---

# 41. Student Answer

Answers SHOULD be typed.

```rust
pub enum StudentAnswer {
    Text(String),
    Boolean(bool),
    Number(f64),
    Choice(String),
    Choices(Vec<String>),
    Code(String),
    Command(String),
    Artifact(ArtifactId),
}
```

---

# 42. Answer Submission

```rust
pub struct AnswerSubmission {
    pub id: AnswerId,
    pub student_id: StudentId,
    pub question_id: QuestionId,
    pub answer: StudentAnswer,
    pub self_confidence: Option<Confidence>,
    pub submitted_at: Timestamp,
}
```

---

# 43. Evaluation Rule

```rust
pub enum EvaluationRule {
    ExactMatch,
    CaseInsensitiveMatch,
    NumericTolerance {
        expected: f64,
        tolerance: f64,
    },
    ChoiceSet,
    Semantic,
    CodeTests {
        test_suite: String,
    },
    CommandOutcome {
        expected_exit_code: Option<i32>,
    },
    HumanReview,
    Custom(String),
}
```

---

# 44. Answer Evaluation

```rust
pub struct AnswerEvaluation {
    pub answer_id: AnswerId,
    pub outcome: EvaluationOutcome,
    pub score: f32,
    pub confidence: Confidence,
    pub feedback: Option<String>,
    pub evidence: Vec<EvidenceId>,
}
```

`score` SHALL be validated to a documented range.

---

# 45. Evaluation Outcome

```rust
pub enum EvaluationOutcome {
    Correct,
    Incorrect,
    Partial,
    Uncertain,
    NotEvaluated,
}
```

---

# 46. Assessment

```rust
pub struct Assessment {
    pub id: AssessmentId,
    pub key: String,
    pub title: String,
    pub mode: AssessmentMode,
    pub question_ids: Vec<QuestionId>,
    pub policy: AssessmentPolicy,
    pub passing_rule: PassingRule,
}
```

---

# 47. Assessment Mode

```rust
pub enum AssessmentMode {
    Practice,
    Diagnostic,
    Formative,
    Summative,
    Certification,
}
```

---

# 48. Assessment Policy

```rust
pub struct AssessmentPolicy {
    pub hint_policy: HintPolicy,
    pub allow_retries: bool,
    pub max_attempts: Option<u32>,
    pub show_feedback_during_attempt: bool,
}
```

---

# 49. Hint Policy

```rust
pub enum HintPolicy {
    Open,
    Progressive,
    Limited(u8),
    Disabled,
}
```

---

# 50. Passing Rule

```rust
pub enum PassingRule {
    ScoreThreshold(f32),
    MasteryThreshold(MasteryScore),
    AllObjectives,
    Composite,
}
```

---

# 51. Assessment Attempt

```rust
pub struct AssessmentAttempt {
    pub id: AttemptId,
    pub assessment_id: AssessmentId,
    pub student_id: StudentId,
    pub started_at: Timestamp,
    pub completed_at: Option<Timestamp>,
    pub answers: Vec<AnswerId>,
    pub result: Option<AssessmentResult>,
}
```

---

# 52. Assessment Result

```rust
pub struct AssessmentResult {
    pub score: f32,
    pub passed: bool,
    pub competency_updates: Vec<CompetencyDelta>,
}
```

---

# 53. Competency Delta

```rust
pub struct CompetencyDelta {
    pub competency_id: CompetencyId,
    pub previous: MasteryScore,
    pub current: MasteryScore,
}
```

---

# 54. Hint

```rust
pub struct Hint {
    pub id: HintId,
    pub question_id: Option<QuestionId>,
    pub level: HintLevel,
    pub content: String,
    pub reveals_answer: bool,
}
```

---

# 55. Hint Level

```rust
pub enum HintLevel {
    Prompt,
    Concept,
    Narrowing,
    PartialProcedure,
    GuidedProcedure,
    Solution,
}
```

This maps to the Nexa progressive hint ladder.

---

# 56. Misconception

```rust
pub struct Misconception {
    pub id: MisconceptionId,
    pub student_id: StudentId,
    pub concept_id: ConceptId,
    pub description: String,
    pub confidence: Confidence,
    pub evidence_ids: Vec<EvidenceId>,
    pub state: MisconceptionState,
}
```

---

# 57. Misconception State

```rust
pub enum MisconceptionState {
    Suspected,
    Confirmed,
    Addressing,
    Resolved,
}
```

---

# 58. Pedagogy Strategy

```rust
pub enum PedagogyStrategy {
    DirectInstruction,
    GuidedInstruction,
    Socratic,
    Coaching,
    Demonstration,
    RetrievalPractice,
    DeliberatePractice,
    Review,
    Challenge,
    Debugging,
}
```

---

# 59. Pedagogy Decision

```rust
pub struct PedagogyDecision {
    pub strategy: PedagogyStrategy,
    pub rationale_code: PedagogyReason,
    pub difficulty_adjustment: DifficultyAdjustment,
    pub hint_level: Option<HintLevel>,
}
```

The rationale SHOULD be structured rather than storing private model reasoning.

---

# 60. Pedagogy Reason

```rust
pub enum PedagogyReason {
    NewConcept,
    LowMastery,
    HighMastery,
    RepeatedError,
    MisconceptionDetected,
    RetentionCheck,
    StudentRequestedExplanation,
    StudentRequestedChallenge,
    AssessmentPolicy,
}
```

---

# 61. Difficulty Adjustment

```rust
pub enum DifficultyAdjustment {
    Increase,
    Maintain,
    Decrease,
}
```

---

# 62. Explanation Depth

```rust
pub enum ExplanationDepth {
    Minimal,
    Concise,
    Standard,
    Detailed,
    Deep,
    Expert,
    Exhaustive,
}
```

---

# 63. Tutor Request

```rust
pub struct TutorRequest {
    pub id: TutorRequestId,
    pub session_id: SessionId,
    pub student_input: StudentInput,
    pub context: TutorContext,
    pub requested_at: Timestamp,
}
```

---

# 64. Student Input

```rust
pub enum StudentInput {
    Text(String),
    SpeechTranscript(String),
    Answer(AnswerSubmission),
    ToolAction(ToolExecutionId),
    LabAction(LabObservation),
    Control(StudentControl),
}
```

---

# 65. Student Control

```rust
pub enum StudentControl {
    Repeat,
    ExplainMore,
    ExplainLess,
    GiveHint,
    Skip,
    Pause,
    Resume,
    Stop,
}
```

---

# 66. Tutor Context

```rust
pub struct TutorContext {
    pub student_id: StudentId,
    pub course_id: Option<CourseId>,
    pub lesson_id: Option<LessonId>,
    pub lesson_step_id: Option<LessonStepId>,
    pub competencies: Vec<StudentCompetency>,
    pub active_misconceptions: Vec<MisconceptionId>,
    pub pedagogy: PedagogyContext,
    pub available_tools: Vec<ToolId>,
}
```

The full domain state SHOULD not be sent blindly to an LLM.

A context builder SHALL select what is relevant.

---

# 67. Tutor Response

```rust
pub struct TutorResponse {
    pub id: TutorResponseId,
    pub request_id: TutorRequestId,
    pub speech: Option<String>,
    pub display_text: Option<String>,
    pub intent: TutorIntent,
    pub behavior: BehaviorIntent,
    pub pedagogy_action: Option<PedagogyAction>,
    pub tool_requests: Vec<ToolRequest>,
    pub canvas_actions: Vec<CanvasAction>,
}
```

---

# 68. Tutor Intent

```rust
pub enum TutorIntent {
    Explain,
    Ask,
    Answer,
    Hint,
    Correct,
    Encourage,
    Demonstrate,
    Summarize,
    Review,
    Warn,
    Challenge,
    Observe,
}
```

---

# 69. Behavior Intent

BehaviorIntent is higher level than NBP.

```rust
pub struct BehaviorIntent {
    pub state: BehaviorState,
    pub emotion: EmotionIntent,
    pub gaze: Option<GazeIntent>,
    pub gesture: Option<GestureIntent>,
    pub speech_style: Option<SpeechStyle>,
}
```

This domain object is translated into an NBP message later.

---

# 70. Behavior State

```rust
pub enum BehaviorState {
    Idle,
    Attentive,
    Listening,
    Processing,
    Thinking,
    Speaking,
    Explaining,
    Demonstrating,
    Questioning,
    Waiting,
    Observing,
    Evaluating,
    Hinting,
    Correcting,
    Encouraging,
    Celebrating,
    Warning,
    Summarizing,
    Debugging,
}
```

---

# 71. Emotion Intent

```rust
pub struct EmotionIntent {
    pub preset: EmotionPreset,
    pub intensity: Confidence,
}
```

---

# 72. Emotion Preset

```rust
pub enum EmotionPreset {
    Neutral,
    Curious,
    Focused,
    Thinking,
    Encouraging,
    Concerned,
    Skeptical,
    Serious,
    Excited,
    Celebrating,
    Corrective,
    Surprised,
    Confused,
}
```

---

# 73. Gaze Intent

```rust
pub struct GazeIntent {
    pub target: AttentionTarget,
    pub intensity: Confidence,
}
```

---

# 74. Attention Target

```rust
pub enum AttentionTarget {
    Student,
    Camera,
    Terminal,
    CodeEditor,
    Canvas,
    CanvasObject(String),
    Diagram(String),
    Quiz,
    EnvironmentObject(String),
}
```

---

# 75. Gesture Intent

```rust
pub enum GestureIntent {
    None,
    Nod,
    SmallNod,
    ShakeHead,
    HeadTilt,
    Point {
        target: Option<String>,
    },
    OpenHand,
    TwoHandExplain,
    ThinkingChin,
    AdjustGlasses,
    Shrug,
    LeanForward,
    LeanBack,
    ThumbsUp,
    Celebrate,
    Attention,
    Typing,
}
```

---

# 76. Speech Style

```rust
pub enum SpeechStyle {
    Neutral,
    Instructional,
    Conversational,
    Encouraging,
    Questioning,
    Serious,
    Warning,
    Excited,
    Reflective,
    Concise,
}
```

---

# 77. Pedagogy Action

```rust
pub enum PedagogyAction {
    ContinueLesson,
    AskQuestion(QuestionId),
    ProvideHint(HintLevel),
    RetryQuestion(QuestionId),
    ReviewConcept(ConceptId),
    IncreaseDifficulty,
    DecreaseDifficulty,
    MarkObjectiveComplete(LearningObjectiveId),
}
```

---

# 78. Tool

A Tool represents a capability Nexa may invoke.

```rust
pub struct Tool {
    pub id: ToolId,
    pub key: String,
    pub name: String,
    pub description: String,
    pub capabilities: Vec<ToolCapability>,
    pub risk: ToolRisk,
}
```

---

# 79. Tool Capability

```rust
pub enum ToolCapability {
    ReadFile,
    WriteFile,
    ExecuteCommand,
    ExecuteCode,
    SearchKnowledge,
    RenderDiagram,
    InspectEnvironment,
    Custom(String),
}
```

---

# 80. Tool Risk

```rust
pub enum ToolRisk {
    ReadOnly,
    Low,
    Moderate,
    High,
    Restricted,
}
```

Tool authorization policies SHALL be handled separately.

---

# 81. Tool Request

```rust
pub struct ToolRequest {
    pub tool_id: ToolId,
    pub operation: String,
    pub arguments: serde_json::Value,
}
```

Provider-specific arguments MAY remain structured JSON at the integration boundary.

---

# 82. Tool Execution

```rust
pub struct ToolExecution {
    pub id: ToolExecutionId,
    pub tool_id: ToolId,
    pub session_id: SessionId,
    pub status: ToolExecutionStatus,
    pub requested_at: Timestamp,
    pub started_at: Option<Timestamp>,
    pub completed_at: Option<Timestamp>,
    pub result: Option<ToolResult>,
}
```

---

# 83. Tool Execution Status

```rust
pub enum ToolExecutionStatus {
    Requested,
    Authorized,
    Denied,
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}
```

---

# 84. Tool Result

```rust
pub struct ToolResult {
    pub success: bool,
    pub output: Option<ArtifactId>,
    pub summary: Option<String>,
    pub exit_code: Option<i32>,
}
```

Large outputs SHOULD be represented as Artifacts.

---

# 85. Lab

```rust
pub struct Lab {
    pub id: LabId,
    pub key: String,
    pub title: String,
    pub description: String,
    pub objectives: Vec<LabObjective>,
    pub environment: LabEnvironment,
    pub policy: LabPolicy,
}
```

---

# 86. Lab Objective

```rust
pub struct LabObjective {
    pub key: String,
    pub description: String,
    pub competency_ids: Vec<CompetencyId>,
    pub completion_rule: LabCompletionRule,
}
```

---

# 87. Lab Environment

```rust
pub enum LabEnvironment {
    Terminal,
    Container,
    VirtualMachine,
    BrowserSandbox,
    NetworkSimulation,
    CodeSandbox,
    Custom(String),
}
```

---

# 88. Lab Policy

```rust
pub struct LabPolicy {
    pub isolated: bool,
    pub network_access: NetworkAccessPolicy,
    pub destructive_actions: DestructiveActionPolicy,
    pub resettable: bool,
}
```

---

# 89. Network Access Policy

```rust
pub enum NetworkAccessPolicy {
    None,
    LabOnly,
    AllowList,
    Internet,
}
```

---

# 90. Destructive Action Policy

```rust
pub enum DestructiveActionPolicy {
    Deny,
    Confirm,
    AllowInSandbox,
}
```

---

# 91. Lab Observation

```rust
pub enum LabObservation {
    CommandExecuted {
        execution_id: ToolExecutionId,
    },
    FileChanged {
        artifact_id: ArtifactId,
    },
    ObjectiveCompleted {
        objective_key: String,
    },
    ErrorDetected {
        message: String,
    },
}
```

---

# 92. Artifact

Artifacts represent externally stored or generated data.

```rust
pub struct Artifact {
    pub id: ArtifactId,
    pub kind: ArtifactKind,
    pub locator: ArtifactLocator,
    pub media_type: Option<String>,
    pub created_at: Timestamp,
}
```

---

# 93. Artifact Kind

```rust
pub enum ArtifactKind {
    Text,
    Audio,
    Image,
    Video,
    SourceCode,
    TerminalOutput,
    Document,
    Dataset,
    Model,
    Other,
}
```

---

# 94. Artifact Locator

```rust
pub enum ArtifactLocator {
    LocalPath(PathBuf),
    ObjectStore(String),
    Uri(String),
    InMemory,
}
```

Secrets SHALL not be embedded in locators.

---

# 95. Knowledge Source

```rust
pub struct KnowledgeSource {
    pub id: KnowledgeSourceId,
    pub title: String,
    pub source_type: KnowledgeSourceType,
    pub authority: SourceAuthority,
    pub version: Option<String>,
    pub published_at: Option<Timestamp>,
    pub locator: ArtifactId,
}
```

---

# 96. Knowledge Source Type

```rust
pub enum KnowledgeSourceType {
    Documentation,
    Standard,
    Rfc,
    Textbook,
    CourseMaterial,
    SourceCode,
    LabManual,
    Article,
    InstructorNote,
    Transcript,
}
```

---

# 97. Source Authority

```rust
pub enum SourceAuthority {
    Primary,
    Authoritative,
    Secondary,
    Informal,
    Unknown,
}
```

Retrieval ranking MAY use this information.

---

# 98. Retrieval Query

```rust
pub struct RetrievalQuery {
    pub text: String,
    pub concept_ids: Vec<ConceptId>,
    pub source_scope: Vec<KnowledgeSourceId>,
    pub maximum_results: usize,
}
```

---

# 99. Retrieval Result

```rust
pub struct RetrievalResult {
    pub source_id: KnowledgeSourceId,
    pub artifact_id: ArtifactId,
    pub relevance: Confidence,
    pub excerpt: String,
    pub metadata: RetrievalMetadata,
}
```

---

# 100. Memory Entry

Memory SHALL use explicit scope.

```rust
pub struct MemoryEntry {
    pub id: MemoryEntryId,
    pub scope: MemoryScope,
    pub subject: MemorySubject,
    pub content: MemoryContent,
    pub created_at: Timestamp,
    pub expires_at: Option<Timestamp>,
}
```

---

# 101. Memory Scope

```rust
pub enum MemoryScope {
    Turn,
    Conversation,
    Lesson,
    Course,
    Learner,
    Knowledge,
}
```

---

# 102. Memory Subject

```rust
pub enum MemorySubject {
    Student(StudentId),
    Session(SessionId),
    Lesson(LessonId),
    Course(CourseId),
    Concept(ConceptId),
    System,
}
```

---

# 103. Memory Content

Core memory SHOULD prefer typed content.

```rust
pub enum MemoryContent {
    Preference {
        key: String,
        value: String,
    },
    Misconception(MisconceptionId),
    CompetencyObservation(EvidenceId),
    SessionSummary(String),
    LessonSummary(String),
    Fact(String),
}
```

---

# 104. Canvas Action

```rust
pub enum CanvasAction {
    Show {
        object_id: String,
    },
    Hide {
        object_id: String,
    },
    Highlight {
        object_id: String,
    },
    Focus {
        object_id: String,
    },
    Annotate {
        object_id: String,
        text: String,
    },
    ClearAnnotation {
        object_id: String,
    },
    Reset,
}
```

---

# 105. Message

Conversation messages SHOULD remain distinct from domain events.

```rust
pub struct Message {
    pub id: MessageId,
    pub session_id: SessionId,
    pub role: MessageRole,
    pub content: MessageContent,
    pub timestamp: Timestamp,
}
```

---

# 106. Message Role

```rust
pub enum MessageRole {
    Student,
    Tutor,
    System,
    Tool,
}
```

---

# 107. Message Content

```rust
pub enum MessageContent {
    Text(String),
    SpeechTranscript(String),
    Artifact(ArtifactId),
    Structured(serde_json::Value),
}
```

---

# 108. Domain Error

The core domain SHALL expose typed errors.

```rust
pub enum DomainError {
    InvalidIdentifier,
    InvalidStateTransition,
    ValidationFailed(ValidationError),
    NotFound(EntityReference),
    Conflict(String),
    Unauthorized(String),
    Unsupported(String),
}
```

---

# 109. Validation Error

```rust
pub struct ValidationError {
    pub field: Option<String>,
    pub code: String,
    pub message: String,
}
```

Machine-readable error codes are preferred over relying on message text.

---

# 110. Entity Reference

```rust
pub struct EntityReference {
    pub entity_type: EntityType,
    pub id: String,
}
```

---

# 111. Domain Result

Core modules SHOULD use:

```rust
pub type DomainResult<T> = Result<T, DomainError>;
```

Infrastructure errors SHOULD remain distinct from domain errors.

---

# 112. State Transition Rule

Entities with explicit lifecycle states SHALL own their state-transition rules.

Avoid:

```rust
session.state = SessionState::Completed;
```

Prefer:

```rust
session.complete()?;
```

This enables invariant enforcement.

---

# 113. Session Transition Example

Valid:

```text
Created
   ↓
Starting
   ↓
Active
   ↓
Paused
   ↓
Active
   ↓
Ending
   ↓
Completed
```

Invalid transitions SHOULD produce `DomainError::InvalidStateTransition`.

---

# 114. Immutability Rule

Stable identity and authored definitions SHOULD be immutable where practical.

Examples:

* IDs;
* event IDs;
* evidence records;
* submitted answers;
* completed assessment attempts.

Changes SHOULD generate new records or revisions where auditability matters.

---

# 115. Aggregate Boundaries

Initial aggregate candidates include:

```text
Student
Course
Lesson
Assessment
Session
Lab
```

Competency observations SHOULD generally be managed through dedicated services rather than mutating Course or Lesson definitions.

---

# 116. Definition Versus Runtime State

The model SHALL distinguish definitions from runtime instances.

Examples:

```text
Assessment       → authored definition
AssessmentAttempt → student runtime instance

Competency        → domain definition
StudentCompetency → learner state

Lesson            → authored definition
LessonProgress    → runtime state
```

This distinction is mandatory.

---

# 117. Lesson Progress

```rust
pub struct LessonProgress {
    pub student_id: StudentId,
    pub lesson_id: LessonId,
    pub state: LessonProgressState,
    pub current_step: Option<LessonStepId>,
    pub completed_steps: Vec<LessonStepId>,
    pub completed_objectives: Vec<LearningObjectiveId>,
}
```

---

# 118. Lesson Progress State

```rust
pub enum LessonProgressState {
    NotStarted,
    InProgress,
    Paused,
    Completed,
    Abandoned,
}
```

---

# 119. Dependency Rule

`nexa-domain` SHALL NOT depend on:

* GUI frameworks;
* database drivers;
* LLM SDKs;
* TTS providers;
* Live2D;
* Unity;
* Unreal;
* web frameworks;
* transport implementations.

The domain crate represents business meaning only.

---

# 120. Permitted Dependencies

The domain crate MAY depend on small foundational libraries for:

```text
serialization
time
UUIDs
error derivation
collections
```

Dependency count SHOULD remain intentionally low.

---

# 121. Recommended Crate Structure

```text
crates/
└── nexa-domain/
    ├── src/
    │   ├── lib.rs
    │   ├── ids.rs
    │   ├── time.rs
    │   ├── confidence.rs
    │   ├── errors.rs
    │   │
    │   ├── student/
    │   │   ├── mod.rs
    │   │   ├── profile.rs
    │   │   └── preferences.rs
    │   │
    │   ├── session/
    │   │   ├── mod.rs
    │   │   └── state.rs
    │   │
    │   ├── course/
    │   │   ├── mod.rs
    │   │   ├── lesson.rs
    │   │   └── objective.rs
    │   │
    │   ├── competency/
    │   │   ├── mod.rs
    │   │   ├── evidence.rs
    │   │   └── misconception.rs
    │   │
    │   ├── assessment/
    │   │   ├── mod.rs
    │   │   ├── question.rs
    │   │   ├── answer.rs
    │   │   └── attempt.rs
    │   │
    │   ├── tutor/
    │   │   ├── mod.rs
    │   │   ├── request.rs
    │   │   ├── response.rs
    │   │   └── behavior.rs
    │   │
    │   ├── knowledge/
    │   │   ├── mod.rs
    │   │   └── retrieval.rs
    │   │
    │   ├── tools/
    │   │   ├── mod.rs
    │   │   └── execution.rs
    │   │
    │   ├── labs/
    │   │   └── mod.rs
    │   │
    │   ├── memory/
    │   │   └── mod.rs
    │   │
    │   └── artifact/
    │       └── mod.rs
    │
    └── tests/
        ├── validation.rs
        ├── serialization.rs
        └── state_transitions.rs
```

---

# 122. Crate Relationship

The foundational dependency graph should become:

```text
                 nexa-domain
                 /         \
                ▼           ▼
         nexa-events     nexa-nbp
                \           /
                 \         /
                  ▼       ▼
               orchestrator
                    │
        ┌───────────┼────────────┐
        ▼           ▼            ▼
     tutor       pedagogy      avatar
```

`nexa-domain` should sit near the bottom of the dependency graph.

---

# 123. Serialization

Externally shared domain structures SHOULD support deterministic serialization where practical.

Initial canonical interchange format:

```text
JSON
```

Internal Rust types SHOULD remain strongly typed even where JSON is used at boundaries.

---

# 124. Unknown Enum Values

Public protocol boundaries SHOULD plan for future values.

Internal exhaustive enums are useful for compile-time safety, but compatibility adapters MAY need:

```rust
Unknown(String)
```

for externally versioned enums.

---

# 125. Sensitive Data Boundary

The core learning model SHOULD avoid embedding unnecessary sensitive or personally identifying data.

Student identity SHOULD generally be represented using:

```text
StudentId
```

rather than duplicating profile attributes through every object.

---

# 126. Secret Boundary

Domain objects SHALL NOT contain:

```text
API keys
passwords
access tokens
private keys
database credentials
```

Use secure configuration or secret-reference types outside the core domain.

---

# 127. Domain Validation

Validation SHALL occur at construction boundaries.

Examples:

```text
Confidence ∈ [0,1]
MasteryScore ∈ [0,1]
non-empty authored keys
valid state transitions
valid assessment thresholds
valid hint levels
valid prerequisite references
```

Invalid objects SHOULD be difficult to construct.

---

# 128. Smart Constructors

Example:

```rust
impl Confidence {
    pub fn new(value: f32) -> DomainResult<Self> {
        if (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(DomainError::ValidationFailed(
                ValidationError {
                    field: Some("confidence".into()),
                    code: "OUT_OF_RANGE".into(),
                    message: "Confidence must be between 0.0 and 1.0.".into(),
                }
            ))
        }
    }
}
```

---

# 129. Domain Services

Business logic spanning multiple entities SHOULD live in explicit domain services.

Examples:

```text
CompetencyEvaluator
AssessmentEvaluator
LessonNavigator
PedagogySelector
PrerequisiteEvaluator
```

These should remain separate from persistence implementations.

---

# 130. Competency Evaluator Contract

Conceptually:

```rust
pub trait CompetencyEvaluator {
    fn evaluate(
        &self,
        current: &StudentCompetency,
        evidence: &[Evidence],
    ) -> DomainResult<StudentCompetency>;
}
```

The scoring algorithm can evolve independently.

---

# 131. Lesson Navigator Contract

```rust
pub trait LessonNavigator {
    fn next_step(
        &self,
        lesson: &Lesson,
        progress: &LessonProgress,
        student_state: &StudentLearningState,
    ) -> DomainResult<Option<LessonStepId>>;
}
```

This supports future adaptive lesson branching.

---

# 132. Student Learning State

A convenient read model MAY combine:

```rust
pub struct StudentLearningState {
    pub student_id: StudentId,
    pub competencies: Vec<StudentCompetency>,
    pub active_misconceptions: Vec<Misconception>,
    pub current_courses: Vec<CourseId>,
}
```

This is a projection, not the canonical source of truth.

---

# 133. Behavior Mapping Boundary

The domain model SHALL stop at `BehaviorIntent`.

```text
TutorResponse
      │
      ▼
BehaviorIntent
      │
      ▼
NBP Adapter
      │
      ▼
Nexa Behavior Protocol
      │
      ▼
Avatar Runtime
```

Animation-specific types SHALL not enter `nexa-domain`.

---

# 134. Event Mapping Boundary

Domain actions and state changes may produce domain events.

Example:

```text
AnswerEvaluation
      ↓
Evidence
      ↓
StudentCompetency change
      ↓
competency.updated event
```

The domain crate SHOULD define the meaning of the change.

`nexa-events` SHOULD define transport/envelope concerns.

---

# 135. Persistence Boundary

Domain objects SHALL not expose SQL concerns.

Avoid:

```rust
pub struct Student {
    pub db_row_id: i64,
    ...
}
```

Persistence adapters map storage schemas to domain entities.

---

# 136. Repository Interfaces

Storage-dependent systems SHOULD work through repository traits.

Example:

```rust
#[async_trait]
pub trait StudentRepository {
    async fn get(&self, id: StudentId) -> DomainResult<Option<Student>>;
    async fn save(&self, student: &Student) -> DomainResult<()>;
}
```

Concrete implementations may use:

```text
SQLite
DuckDB
PostgreSQL
file-backed storage
in-memory stores
```

---

# 137. Clock Abstraction

Testing time-dependent behavior becomes easier if services depend on a clock abstraction.

```rust
pub trait Clock {
    fn now(&self) -> Timestamp;
}
```

Production:

```text
SystemClock
```

Tests:

```text
FixedClock
```

---

# 138. Randomness Abstraction

Adaptive exercises or randomized assessments SHOULD permit deterministic testing.

```rust
pub trait RandomSource {
    fn next_u64(&mut self) -> u64;
}
```

Production may use secure or standard randomness as appropriate.

Tests may use seeded generators.

---

# 139. Domain Invariants

NEXA-DOM-001 establishes the following invariants:

1. Domain identifiers SHALL be strongly typed.
2. Definitions SHALL remain separate from runtime state.
3. Student competency SHALL remain separate from competency definitions.
4. Competency updates SHOULD be traceable to evidence.
5. Confidence and mastery values SHALL be range validated.
6. Lifecycle transitions SHALL be explicit and validated.
7. Domain objects SHALL not depend on infrastructure frameworks.
8. Secrets SHALL remain outside the domain model.
9. Large outputs SHALL be represented using Artifacts.
10. Tutor responses SHALL use structured intent.
11. BehaviorIntent SHALL remain independent of rendering technology.
12. Tool executions SHALL have explicit lifecycle states.
13. Assessment commands SHALL not imply assessment results.
14. Memory SHALL declare scope.
15. Knowledge sources SHALL preserve provenance.
16. Domain errors SHALL be typed.
17. Important values SHOULD use smart constructors.
18. Repository concerns SHALL remain behind interfaces.
19. Domain logic SHALL be independently testable.
20. The type system SHOULD make invalid states difficult to represent.

---

# 140. MVP Domain Subset

The first executable vertical slice does not require every domain type.

Minimum implementation:

```text
IDs
Timestamp
Confidence
Student
Session
StudentInput
TutorRequest
TutorResponse
TutorIntent
BehaviorIntent
BehaviorState
EmotionIntent
GestureIntent
SpeechStyle
Message
Artifact
DomainError
```

Then add:

```text
Course
Lesson
Competency
Evidence
Assessment
Question
Answer
```

after the basic Nexa conversation loop works.

---

# 141. First Compile-Time Milestone

The initial workspace should compile with:

```text
nexa-domain
nexa-events
nexa-nbp
```

and no runtime services.

This produces the first stable architectural foundation.

---

# 142. First Cross-Crate Contract

A complete compile-time flow should be possible:

```rust
let response: TutorResponse = ...;

let behavior: BehaviorIntent = response.behavior;

let nbp_command = NbpBehaviorBuilder::from_intent(behavior)
    .build()?;

event_bus.publish(
    AvatarBehaviorRequested::new(nbp_command)
).await?;
```

The exact APIs may evolve, but the architectural boundaries should remain.

---

# 143. First Domain Test Suite

The initial tests SHOULD cover:

```text
ID serialization
confidence validation
mastery validation
session transitions
assessment transitions
answer serialization
TutorResponse serialization
BehaviorIntent → NBP mapping
event correlation preservation
domain error behavior
```

---

# 144. Architectural Foundation After NEXA-DOM-001

At this stage, Nexa has three foundational specifications:

```text
NEXA-CBS-001
Character & Behavior Specification
            │
            ▼
NEXA-DOM-001
Core Domain Model
        │           │
        ▼           ▼
NEXA-EVT-001     NEXA-NBP-001
Event Model      Behavior Protocol
        │           │
        └─────┬─────┘
              ▼
        Runtime Services
```

This is enough architecture to begin creating actual Rust crates without inventing the system ad hoc during implementation.

---

# 145. Next Specification

The next specification should be:

# NEXA-ORCH-001 — Session Orchestrator & Runtime Coordination Specification v1.0

It will define the component that actually makes Nexa operate as a coherent system, including:

```text
session lifecycle
command routing
event routing
TutorRequest construction
context assembly
pedagogy invocation
knowledge retrieval
tool invocation
TutorResponse execution
NBP generation
speech/avatar synchronization
interruptions
timeouts
cancellation
failure recovery
workflow state
concurrency
backpressure
observability
```

This is the architectural layer where the specifications completed so far begin operating together as an actual application.
