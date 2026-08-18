NEXA-ASMT-001 — Assessment, Question, Evaluation, Grading & Mastery Evidence Architecture v1.0

Specification ID: NEXA-ASMT-001
System: Nexa AI Training Tutor
Version: 1.0
Status: Baseline Draft
Depends On: NEXA-DOM-001, NEXA-EVT-001, NEXA-STU-001, NEXA-PED-001, NEXA-TUTOR-001, NEXA-KNOW-001, NEXA-LESSON-001, NEXA-ORCH-001
Purpose: Define how Nexa creates, presents, evaluates, secures, scores, analyzes, versions, and converts assessments into evidence of learner competency.

1. Purpose

The Assessment subsystem answers:

“What can the learner actually demonstrate, under what conditions, and how confidently should that performance affect competency state?”

It SHALL support far more than quizzes.

Nexa assessments may include:

selected-response questions;
constructed-response questions;
explanations;
code;
commands;
debugging tasks;
labs;
demonstrations;
procedural exercises;
scenario analysis;
transfer tasks.

The Assessment subsystem SHALL provide evidence to NEXA-STU-001.

It SHALL NOT directly set learner mastery.

2. Architectural Role
              Curriculum / Lesson
                      │
                      ▼
                 Assessment
                      │
          ┌───────────┼───────────┐
          ▼           ▼           ▼
      Questions      Labs       Tasks
          │           │           │
          └───────────┼───────────┘
                      ▼
                Student Response
                      │
                      ▼
                 Evaluation
                      │
                      ▼
                   Result
                      │
                      ▼
             Learning Evidence
                      │
                      ▼
                Student Model
3. Core Responsibilities

The subsystem SHALL own or coordinate:

assessment definitions;
question banks;
question selection;
attempts;
response capture;
answer-key protection;
scoring;
rubric evaluation;
partial credit;
item-level feedback;
assessment-level feedback;
time limits;
hint restrictions;
retries;
randomization;
code evaluation;
command evaluation;
lab evaluation;
AI-assisted semantic evaluation;
evaluator confidence;
grading;
passing rules;
regrading;
assessment analytics;
evidence generation;
audit trails;
versioning.
4. Explicit Non-Responsibilities

The subsystem SHALL NOT own:

long-term mastery estimation;
pedagogy outside assessment constraints;
final lesson navigation;
lab infrastructure internals;
tutor personality;
avatar behavior;
knowledge retrieval implementation.
5. Assessment Definition
pub struct Assessment {
    pub id: AssessmentId,
    pub key: String,
    pub title: String,
    pub description: String,


    pub version: AssessmentVersion,
    pub mode: AssessmentMode,


    pub objectives: Vec<LearningObjectiveId>,
    pub competency_ids: Vec<CompetencyId>,


    pub sections: Vec<AssessmentSection>,


    pub policy: AssessmentPolicy,
    pub passing_rule: PassingRule,
}
6. Assessment Modes
pub enum AssessmentMode {
    Diagnostic,
    Practice,
    Formative,
    Summative,
    Certification,
    Placement,
}

These modes SHALL affect feedback, hints, retries, and security.

7. Diagnostic Assessment

Purpose:

determine current learner state
before instruction

Characteristics MAY include:

broad competency sampling
little or no penalty
minimal teaching during attempt
adaptive item selection
8. Practice Assessment

Practice is primarily instructional.

It MAY permit:

hints;
immediate feedback;
retries;
worked explanations.

Practice scores SHOULD not necessarily carry the same evidence strength as controlled assessments.

9. Formative Assessment

Formative assessment measures progress during learning.

It SHOULD support:

feedback
targeted remediation
checkpoint decisions
10. Summative Assessment

Summative assessments measure achievement after instruction.

They SHOULD use stronger controls around:

hints
answer disclosure
retries
tool access
feedback timing
11. Certification Assessment

Certification mode MAY require:

strict attempt rules
no hints
protected answer keys
audit logs
time limits
identity controls
environment restrictions

The architecture SHALL support these requirements without forcing them on ordinary learning.

12. Assessment Section
pub struct AssessmentSection {
    pub id: AssessmentSectionId,
    pub title: Option<String>,


    pub item_source: AssessmentItemSource,
    pub selection_policy: ItemSelectionPolicy,


    pub completion_rule: SectionCompletionRule,
}
13. Assessment Item Sources
pub enum AssessmentItemSource {
    Fixed(Vec<QuestionId>),
    Pool(QuestionPoolId),
    Generated(GeneratedItemPolicy),
    Composite(Vec<AssessmentItemSource>),
}
14. Question Model
pub struct Question {
    pub id: QuestionId,
    pub key: String,
    pub version: QuestionVersion,


    pub prompt: QuestionPrompt,
    pub kind: QuestionKind,


    pub target_competencies: Vec<CompetencyId>,
    pub purpose: QuestionPurpose,


    pub difficulty: Difficulty,
    pub evaluation: EvaluationDefinition,


    pub hints: Vec<HintId>,
    pub metadata: QuestionMetadata,
}
15. Question Kinds
pub enum QuestionKind {
    MultipleChoice,
    MultipleSelect,
    TrueFalse,
    Matching,
    Ordering,
    FillBlank,
    ShortAnswer,
    LongAnswer,
    Numeric,
    Explanation,
    Code,
    Command,
    Debugging,
    Interactive,
    Lab,
    Demonstration,
}
16. Question Prompt
pub struct QuestionPrompt {
    pub display: String,
    pub spoken: Option<String>,
    pub artifacts: Vec<ArtifactId>,
}

This supports separate screen and speech presentation.

17. Question Purpose
pub enum QuestionPurpose {
    Diagnose,
    Recognize,
    Recall,
    Explain,
    Apply,
    Analyze,
    Debug,
    Transfer,
    RetentionCheck,
    VerifyMastery,
}

Purpose SHOULD affect evidence weighting later.

18. Selected-Response Questions

Selected-response types SHALL preserve:

choice ordering
correct answer
distractors
scoring rule
randomization policy
19. Multiple-Choice Definition
pub struct MultipleChoiceDefinition {
    pub choices: Vec<QuestionChoice>,
    pub correct_choice: ChoiceId,
    pub shuffle: bool,
}
20. Distractors

Distractors SHOULD ideally represent plausible misconceptions rather than random wrong answers.

This enables diagnostic value.

21. Distractor Mapping
pub struct QuestionChoice {
    pub id: ChoiceId,
    pub text: String,
    pub misconception_id: Option<MisconceptionId>,
}

Selecting a specific distractor MAY generate misconception evidence.

22. Multiple Select

Scoring SHOULD support policies such as:

all-or-nothing
partial credit
penalty for incorrect selections
23. Ordering Questions

Useful for:

protocols;
procedures;
troubleshooting steps;
lifecycle sequences.

Example:

SYN
SYN-ACK
ACK
24. Constructed Response

Constructed responses include:

short answer
long answer
explanation

These require evaluators rather than exact answer matching alone.

25. Evaluation Definition
pub enum EvaluationDefinition {
    Exact(ExactEvaluation),
    Choice(ChoiceEvaluation),
    Numeric(NumericEvaluation),
    Semantic(SemanticEvaluation),
    Rubric(RubricEvaluation),
    Code(CodeEvaluation),
    Command(CommandEvaluation),
    Lab(LabEvaluation),
    Composite(CompositeEvaluation),
    HumanReview,
}
26. Exact Evaluation

Appropriate for deterministic answers such as:

protocol name
small identifier
explicit term

Normalization MAY include case and whitespace rules.

27. Numeric Evaluation
pub struct NumericEvaluation {
    pub expected: f64,
    pub tolerance: NumericTolerance,
    pub unit: Option<UnitDefinition>,
}
28. Numeric Tolerance
pub enum NumericTolerance {
    Absolute(f64),
    Relative(f64),
    Range {
        minimum: f64,
        maximum: f64,
    },
}
29. Unit Awareness

Where relevant:

1024 KB
1 MB

may require unit normalization rather than naive number comparison.

30. Semantic Evaluation

Semantic evaluation MAY use an AI evaluator.

pub struct SemanticEvaluation {
    pub expected_concepts: Vec<ExpectedConcept>,
    pub prohibited_claims: Vec<String>,
    pub minimum_evaluator_confidence: Confidence,
}

The LLM SHALL not simply output a grade without structured justification.

31. Expected Concept
pub struct ExpectedConcept {
    pub concept_id: Option<ConceptId>,
    pub description: String,
    pub weight: f32,
}
32. Semantic Evaluation Pipeline
Student response
      ↓
Evaluator prompt/context
      ↓
Structured rubric output
      ↓
Validation
      ↓
Evaluation result
33. AI Evaluation Is Not Absolute

AI evaluation SHALL include:

score
outcome
confidence
criterion findings

Low-confidence results MAY require:

verification
second evaluator
human review
34. Evaluator Result
pub struct EvaluatorResult {
    pub outcome: EvaluationOutcome,
    pub score: Score,
    pub confidence: Confidence,
    pub criteria: Vec<CriterionResult>,
}
35. Rubric
pub struct Rubric {
    pub id: RubricId,
    pub criteria: Vec<RubricCriterion>,
    pub scoring_policy: RubricScoringPolicy,
}
36. Rubric Criterion
pub struct RubricCriterion {
    pub id: RubricCriterionId,
    pub description: String,
    pub weight: f32,
    pub levels: Vec<RubricLevel>,
}
37. Rubric Level
pub struct RubricLevel {
    pub score: f32,
    pub description: String,
}
38. Partial Credit

Partial credit SHALL be explicit.

pub struct Score(f32);

Recommended normalized range:

0.0 ───────────── 1.0
39. Evaluation Outcome
pub enum EvaluationOutcome {
    Correct,
    MostlyCorrect,
    Partial,
    Incorrect,
    Invalid,
    Uncertain,
    NotEvaluated,
}
40. Answer Submission
pub struct AssessmentAnswer {
    pub id: AnswerId,
    pub attempt_id: AttemptId,
    pub question_id: QuestionId,


    pub answer: StudentAnswer,
    pub self_confidence: Option<Confidence>,


    pub submitted_at: Timestamp,
}
41. Student Confidence

Assessment MAY capture:

How confident are you?

This SHOULD generally remain optional.

It provides useful evidence for misconception and calibration analysis.

42. Code Questions

Code evaluation SHALL use controlled execution where required.

pub struct CodeEvaluation {
    pub language: String,
    pub tests: TestSuiteId,
    pub resource_policy: ExecutionResourcePolicy,
}
43. Code Evaluation Pipeline
Student code
   ↓
isolated runtime
   ↓
compile
   ↓
tests
   ↓
static checks
   ↓
result
44. Correct Output Is Not Always Enough

Procedural or code assessments MAY evaluate:

correctness
robustness
required technique
forbidden technique
efficiency

where curriculum requires it.

45. Code Test Result
pub struct CodeTestResult {
    pub passed: u32,
    pub failed: u32,
    pub compilation_success: bool,
    pub diagnostics: Vec<TestDiagnostic>,
}
46. Hidden Tests

Assessments MAY include hidden tests.

Their definitions SHALL remain protected from learner-facing TutorContext.

47. Command Questions

A learner may be asked to:

run the correct command

Evaluation may inspect:

command;
arguments;
exit code;
output;
resulting environment state.
48. Command Evaluation
pub struct CommandEvaluation {
    pub accepted_commands: Vec<CommandPattern>,
    pub expected_outcome: CommandOutcomeRule,
}
49. Outcome-Based Scoring

Where possible, evaluate outcome rather than exact command spelling.

Example:

Several valid commands may achieve the objective.

50. Lab Assessment

Lab-based assessment may evaluate an entire process.

launch environment
      ↓
perform task
      ↓
observe actions
      ↓
inspect final state
      ↓
evaluate objectives
51. Lab Evaluation Definition
pub struct LabEvaluation {
    pub lab_id: LabId,
    pub objective_rules: Vec<LabObjectiveEvaluation>,
}
52. Process Versus Final State

A lab MAY score:

final state only

or:

process + final state

depending on competency.

53. Debugging Assessment

Debugging tasks SHOULD evaluate reasoning behavior through observable actions.

Potential evidence:

diagnostic commands chosen
hypothesis-testing order
unnecessary destructive actions
final root cause
fix verification
54. Avoid Private Reasoning Requirements

Nexa SHALL evaluate observable reasoning artifacts such as:

student explanation
chosen diagnostic action
recorded hypothesis

rather than requiring hidden internal thought.

55. Assessment Attempt
pub struct AssessmentAttempt {
    pub id: AttemptId,
    pub assessment_id: AssessmentId,
    pub assessment_version: AssessmentVersion,
    pub student_id: StudentId,


    pub state: AssessmentAttemptState,


    pub started_at: Timestamp,
    pub completed_at: Option<Timestamp>,


    pub selected_items: Vec<AssessmentItemInstance>,
    pub answers: Vec<AnswerId>,
}
56. Attempt States
pub enum AssessmentAttemptState {
    Created,
    Active,
    Paused,
    Submitted,
    Evaluating,
    Completed,
    Failed,
    Invalidated,
    Cancelled,
}
57. Item Instance

A selected question SHALL become an immutable attempt-specific item instance.

pub struct AssessmentItemInstance {
    pub instance_id: AssessmentItemInstanceId,
    pub question_id: QuestionId,
    pub question_version: QuestionVersion,
    pub presentation_seed: Option<u64>,
}
58. Why Freeze Item Versions

If an author edits a question during an active attempt, the learner SHALL still be graded against the exact version presented.

59. Attempt Policy
pub struct AssessmentPolicy {
    pub max_attempts: Option<u32>,
    pub allow_pause: bool,
    pub time_limit: Option<Duration>,


    pub hint_policy: AssessmentHintPolicy,
    pub feedback_policy: AssessmentFeedbackPolicy,


    pub answer_change_policy: AnswerChangePolicy,


    pub tool_policy: AssessmentToolPolicy,
    pub navigation_policy: AssessmentNavigationPolicy,


    pub randomization: RandomizationPolicy,
}
60. Hint Policy
pub enum AssessmentHintPolicy {
    Allowed,
    Limited(u8),
    Penalized,
    Disabled,
}
61. Hint Penalty

If hints are permitted but penalized:

pub struct HintPenalty {
    pub score_multiplier: f32,
    pub evidence_multiplier: f32,
}

A hint may reduce mastery evidence even when the assessment score remains high.

62. Feedback Policy
pub enum AssessmentFeedbackPolicy {
    ImmediateDetailed,
    ImmediateMinimal,
    AfterQuestion,
    AfterSection,
    AfterSubmission,
    AfterAttemptClosure,
    None,
}
63. Answer Change Policy
pub enum AnswerChangePolicy {
    Allowed,
    AllowedUntilSectionSubmit,
    LockedAfterSubmit,
}
64. Navigation Policy
pub enum AssessmentNavigationPolicy {
    Free,
    ForwardOnly,
    SectionRestricted,
    AdaptiveLocked,
}
65. Timing

The Assessment Engine SHALL own authoritative attempt timing.

Do not rely solely on client-side timers.

66. Timer State
pub struct AttemptTimer {
    pub started_at: Timestamp,
    pub paused_duration: Duration,
    pub limit: Duration,
}
67. Timeout Behavior

When time expires:

attempt timeout
      ↓
current response policy
      ↓
submit / invalidate / stop

SHALL be explicit.

68. Question Pools
pub struct QuestionPool {
    pub id: QuestionPoolId,
    pub questions: Vec<QuestionId>,
    pub competency_coverage: Vec<CompetencyCoverageRule>,
}
69. Item Selection
pub enum ItemSelectionPolicy {
    Fixed,
    Random,
    Stratified,
    CompetencyBalanced,
    DifficultyBalanced,
    Adaptive,
}
70. Randomization

Assessment randomization MAY include:

item ordering
choice ordering
parameterized variants
question pool sampling

All randomization SHALL use recorded seeds for reproducibility.

71. Randomization Seed
pub struct AssessmentRandomSeed(pub u64);

Given the same:

assessment version
seed

the system SHOULD reconstruct the same presentation.

72. Parameterized Questions

A question may use authored parameter ranges.

Example:

Given network 10.0.X.0/24...

Generated parameter values SHALL be recorded in the item instance.

73. AI-Generated Questions

Generated questions MAY be allowed only when policy explicitly permits.

They SHALL undergo:

grounding validation
answerability validation
competency alignment
difficulty check
assessment-policy check
74. AI-Generated Certification Items

Certification assessments SHOULD default to:

authored or formally approved items

unless the deployment has a rigorous generated-item validation process.

75. Adaptive Testing

Future assessments MAY select the next item based on current performance.

question
  ↓
response
  ↓
ability estimate
  ↓
next difficulty

This SHALL remain distinct from normal lesson pedagogy.

76. Adaptive Test State
pub struct AdaptiveAssessmentState {
    pub estimate: f32,
    pub uncertainty: f32,
    pub administered_items: Vec<QuestionId>,
}
77. Termination Rules

Adaptive assessments MAY end when:

target precision achieved
maximum items reached
minimum items satisfied
competency decision sufficiently certain
78. Passing Rules
pub enum PassingRule {
    ScoreThreshold(Score),
    CompetencyThresholds(Vec<CompetencyRequirement>),
    AllRequiredSections,
    AllCriticalItems,
    Composite(Vec<PassingCriterion>),
}
79. Critical Items

Some assessments MAY contain items that must be passed regardless of aggregate score.

Example:

safety procedure

A learner SHALL not pass by compensating with unrelated questions.

80. Assessment Result
pub struct AssessmentResult {
    pub attempt_id: AttemptId,


    pub score: Score,
    pub passed: bool,


    pub section_results: Vec<SectionResult>,
    pub item_results: Vec<ItemResult>,


    pub competency_evidence: Vec<EvidenceId>,


    pub completed_at: Timestamp,
}
81. Item Result
pub struct ItemResult {
    pub item_instance_id: AssessmentItemInstanceId,
    pub outcome: EvaluationOutcome,
    pub score: Score,
    pub evaluator_confidence: Confidence,
    pub feedback: Option<FeedbackReference>,
}
82. Score Versus Competency Evidence

This distinction is critical:

Assessment Score
      ≠
Mastery Score

An assessment score measures performance on an attempt.

The Student Model integrates resulting evidence with prior history.

83. Evidence Generation

Each evaluable item SHOULD produce structured learning evidence.

pub struct AssessmentEvidenceFactory;

Conceptually:

item result
+
question purpose
+
difficulty
+
hint usage
+
assessment mode
+
evaluator confidence
      ↓
LearningEvidence
84. Evidence Strength

A correct independent transfer item in a certification assessment SHOULD generally carry stronger evidence than:

correct practice question
after three hints
85. Assessment Evidence Metadata
pub struct AssessmentEvidenceMetadata {
    pub assessment_id: AssessmentId,
    pub attempt_id: AttemptId,
    pub item_instance_id: AssessmentItemInstanceId,


    pub mode: AssessmentMode,
    pub hints_used: u8,


    pub evaluator_confidence: Confidence,
}
86. Multiple Competencies

A question MAY target several competencies.

Evidence attribution SHALL be specific.

A partially correct response MAY produce:

positive evidence for competency A
negative evidence for competency B

rather than one blanket result.

87. Rubric-to-Competency Mapping

Rubric criteria SHOULD be independently mappable.

pub struct CriterionCompetencyLink {
    pub criterion_id: RubricCriterionId,
    pub competency_id: CompetencyId,
}
88. Misconception Evidence

Responses MAY produce misconception evidence.

Example:

student repeatedly chooses:
"Client sends SYN-ACK"

Question metadata may map that distractor to a known misconception.

89. Answer-Key Protection

Correct answers, expected responses, hidden tests, and grading rubrics SHALL be treated as protected assessment data.

They SHALL not enter ordinary learner-facing TutorContext.

90. Protected Assessment Data
pub enum AssessmentDataVisibility {
    PublicPrompt,
    EvaluatorOnly,
    InstructorOnly,
    Protected,
}
91. Tutor Boundary During Assessment

The Tutor Engine receives only data allowed by AssessmentPolicy.

Example:

Student-visible question
Allowed hint policy
Feedback constraints

It SHALL not receive hidden answer data unless acting inside an isolated evaluator role.

92. Evaluator Isolation

AI evaluation SHOULD use a separate evaluator context from learner-facing tutoring.

Learner-facing Tutor
        ≠
Assessment Evaluator
93. Evaluator Context

The evaluator MAY receive:

question
rubric
expected concepts
student answer

but SHOULD not have authority to speak directly to the learner.

94. Evaluation Provider Abstraction
#[async_trait]
pub trait AssessmentEvaluator {
    async fn evaluate(
        &self,
        request: EvaluationRequest,
    ) -> AssessmentResult<EvaluatorResult>;
}
95. Deterministic Evaluators

Prefer deterministic evaluation for:

choice questions
numeric questions
code tests
command outcomes
lab state checks

Use AI semantic evaluators where deterministic evaluation is insufficient.

96. Evaluation Confidence

All non-deterministic evaluators SHOULD produce confidence.

Low-confidence decisions SHALL not be silently treated as definitive in high-stakes modes.

97. Double Evaluation

High-stakes free-text responses MAY use:

evaluator A
+
evaluator B

with disagreement detection.

98. Evaluator Disagreement
pub struct EvaluationDisagreement {
    pub evaluator_results: Vec<EvaluatorResult>,
    pub severity: DisagreementSeverity,
}

Possible actions:

third evaluator
human review
conservative score
attempt held for review
99. Human Review

The architecture SHALL support manual review for:

ambiguous answers
appeals
high-stakes responses
evaluator disagreement
100. Manual Grade Adjustment

Manual changes SHALL be auditable.

Do not overwrite original evaluation.

Record:

original result
review result
reviewer
reason
timestamp
101. Regrading

If a question or rubric is later found defective:

assessment result
      ↓
regrade process
      ↓
revised result
      ↓
revised evidence

The original result SHALL remain historically traceable.

102. Regrade Record
pub struct Regrade {
    pub id: RegradeId,
    pub attempt_id: AttemptId,
    pub reason: RegradeReason,
    pub previous_result: AssessmentResultId,
    pub new_result: AssessmentResultId,
}
103. Faulty Question Handling

A defective item MAY be:

removed from score
rescored
given full credit
re-evaluated

according to explicit policy.

104. Assessment Versioning

Assessment definitions SHALL be versioned.

Question versions SHALL also be independently versioned.

105. Reproducibility

An attempt SHALL identify:

assessment version
question versions
random seed
evaluator version
rubric version
policy version
106. Evaluation Model Version

If AI is used:

pub struct EvaluatorVersion {
    pub provider: String,
    pub model: String,
    pub prompt_version: String,
    pub schema_version: String,
}
107. Assessment Manifest
assessment:
  id: tcp-checkpoint
  version: 1.0.0
  mode: formative


policy:
  hints: limited
  max_attempts: 3
  feedback: after_question


sections:
  - id: handshake
    pool: tcp-handshake-pool
    select: 5
108. Question Package
assessments/
├── manifest.yaml
├── assessments/
├── questions/
├── rubrics/
├── test-suites/
└── protected/
109. Protected Directory

Runtime packaging SHOULD logically separate:

learner-visible assets
evaluator-only assets

Physical implementation may vary, but permissions SHALL remain explicit.

110. Assessment Compiler
pub trait AssessmentCompiler {
    fn compile(
        &self,
        source: AssessmentSource,
    ) -> AssessmentResult<CompiledAssessment>;
}

The runtime SHOULD consume validated compiled definitions.

111. Structural Validation

Before publication verify:

question IDs unique
all referenced competencies exist
rubrics valid
answer keys present
pools non-empty
selection rules satisfiable
passing rules valid
112. Coverage Validation

An assessment SHOULD report competency coverage.

Example:

TCP handshake sequence     3 items
TCP handshake purpose      2 items
TCP troubleshooting        0 items
113. Blueprint

Formal assessments SHOULD support assessment blueprints.

pub struct AssessmentBlueprint {
    pub competency_targets: Vec<BlueprintCompetencyTarget>,
    pub difficulty_distribution: DifficultyDistribution,
    pub item_type_distribution: ItemTypeDistribution,
}
114. Blueprint Example
40% recall
40% application
20% transfer

This prevents an assessment from accidentally measuring only memorization.

115. Content Validity

Assessment validation SHOULD ask:

Does the assessment actually measure the intended objectives?

This cannot be determined from raw score statistics alone.

116. Reliability

The architecture SHOULD support statistical reliability analysis over time.

Potential metrics include:

internal consistency
item discrimination
test-retest behavior
inter-rater agreement
117. Item Difficulty Analytics

Observed item difficulty:

proportion of learners answering correctly

SHOULD be tracked separately from authored intended difficulty.

118. Item Discrimination

A useful item SHOULD distinguish stronger from weaker learners on the targeted competency.

Poor discrimination MAY indicate:

ambiguous wording
trivial question
broken item
misaligned content
119. Distractor Analytics

For multiple choice:

which wrong answers are selected?

This can reveal:

ineffective distractors;
common misconceptions;
confusing wording.
120. Question Exposure

High-stakes item banks MAY track how frequently questions are used.

Excessively exposed items MAY be retired or rotated.

121. Item Status
pub enum QuestionStatus {
    Draft,
    Review,
    Active,
    Suspended,
    Retired,
}
122. Question Lifecycle
draft
  ↓
review
  ↓
pilot
  ↓
active
  ↓
monitor
  ↓
retire
123. Pilot Questions

New questions MAY be inserted experimentally without affecting scores.

Their responses can provide calibration data.

124. Score Exclusion
pub struct ScoringPolicy {
    pub scored: bool,
    pub evidence_only: bool,
}

This allows diagnostic or pilot items.

125. Assessment Security

Assessment security SHALL be layered.

Controls MAY include:

answer-key isolation;
restricted tools;
protected knowledge;
randomization;
time limits;
attempt limits;
audit events;
environment controls.
126. Tool Restrictions

AssessmentPolicy MAY specify:

pub enum AssessmentToolPolicy {
    None,
    ReadOnlyApproved,
    Approved(Vec<ToolId>),
    CoursePolicy,
}
127. Knowledge Restrictions

Assessment mode MAY constrain RAG to:

no knowledge
approved references only
open-book course sources
128. Open-Book Assessment

Open-book is a valid explicit mode.

Assessment integrity does not always require banning retrieval.

The rules simply need to be clear.

129. Security Events

Emit where appropriate:

assessment.policy.violation
assessment.protected_content.requested
assessment.tool.denied
assessment.attempt.invalidated
130. Audit Trail

Certification attempts SHOULD be auditable.

Record:

attempt lifecycle
item instances
answers
timing
evaluations
policy events
grade changes
131. Assessment Events

Canonical events include:

assessment.started
assessment.paused
assessment.resumed
assessment.submitted
assessment.completed
assessment.failed


assessment.question.presented
assessment.answer.submitted
assessment.answer.changed
assessment.answer.evaluated


assessment.hint.requested
assessment.hint.used


assessment.section.started
assessment.section.completed


assessment.timeout
assessment.regraded
assessment.invalidated
132. Answer Submitted Event
{
  "event_type": "assessment.answer.submitted",
  "payload": {
    "attempt_id": "att-1004",
    "item_instance_id": "item-22",
    "question_id": "tcp-q-4"
  }
}

Do not unnecessarily put protected answer data in general event streams.

133. Evaluated Event
{
  "event_type": "assessment.answer.evaluated",
  "payload": {
    "attempt_id": "att-1004",
    "item_instance_id": "item-22",
    "outcome": "partial",
    "score": 0.65,
    "evaluator_confidence": 0.91
  }
}
134. Assessment Completion Event
{
  "event_type": "assessment.completed",
  "payload": {
    "attempt_id": "att-1004",
    "assessment_id": "tcp-checkpoint",
    "score": 0.84,
    "passed": true
  }
}
135. Event Privacy

General event consumers SHOULD receive identifiers and outcomes, not hidden answer keys or evaluator rubrics.

136. Idempotency

Repeated processing of:

assessment.answer.submitted

SHALL not duplicate:

scores
evidence
attempt answers
137. Transaction Boundary

A completed item evaluation SHOULD logically commit:

answer
+
evaluation
+
score update
+
learning evidence
+
outbox events

atomically where possible.

138. Assessment Repository
#[async_trait]
pub trait AssessmentRepository {
    async fn get(
        &self,
        id: AssessmentId,
        version: Option<AssessmentVersion>,
    ) -> AssessmentResult<Option<Assessment>>;
}
139. Attempt Repository
#[async_trait]
pub trait AssessmentAttemptRepository {
    async fn save(
        &self,
        attempt: &AssessmentAttempt,
    ) -> AssessmentResult<()>;


    async fn get(
        &self,
        attempt_id: AttemptId,
    ) -> AssessmentResult<Option<AssessmentAttempt>>;
}
140. Assessment Engine Contract
#[async_trait]
pub trait AssessmentEngine: Send + Sync {
    async fn start(
        &self,
        request: StartAssessmentRequest,
    ) -> AssessmentResult<AssessmentAttempt>;


    async fn submit_answer(
        &self,
        request: SubmitAnswerRequest,
    ) -> AssessmentResult<ItemResult>;


    async fn submit_attempt(
        &self,
        attempt_id: AttemptId,
    ) -> AssessmentResult<AssessmentResult>;


    async fn resume(
        &self,
        attempt_id: AttemptId,
    ) -> AssessmentResult<AssessmentAttempt>;
}
141. Assessment Runtime Flow
Start assessment
      ↓
freeze assessment version
      ↓
select item set
      ↓
present item
      ↓
student responds
      ↓
evaluate
      ↓
record result
      ↓
next item
      ↓
final scoring
      ↓
generate evidence
      ↓
Student Model
142. Resume

Assessments MAY support resume depending on policy.

A certification test might prohibit it.

A practice assessment may allow it.

143. Resume Integrity

On resume verify:

assessment version
attempt policy
timer state
selected items
answers already submitted
144. Feedback Generation

Assessment evaluation SHOULD produce structured feedback intent.

pub struct AssessmentFeedback {
    pub type_: FeedbackType,
    pub content: FeedbackContent,
}
145. Feedback Types
pub enum FeedbackType {
    CorrectnessOnly,
    BriefExplanation,
    DetailedExplanation,
    Hint,
    RemediationRecommendation,
    NoFeedback,
}
146. Tutor Feedback Boundary

The Assessment Engine decides:

what feedback may be revealed

The Tutor Engine decides:

how Nexa phrases it

within that constraint.

147. Example

Assessment Engine:

Outcome:
Incorrect


Allowed feedback:
Identify concept only


Forbidden:
Correct answer

Tutor:

"Your answer is using the right transport-layer idea, but reconsider which side initiates the acknowledgment."

148. No Unauthorized Reveal

The Tutor Engine SHALL not decide to reveal more because it believes that would be educationally helpful.

Assessment policy wins.

149. Assessment and Pedagogy

Formative assessments MAY emit:

remediation recommendation
review recommendation
challenge recommendation

Pedagogy determines the instructional response after the assessment.

150. Post-Assessment Flow
Assessment Result
      ↓
Student Model update
      ↓
Pedagogy decision
      ↓
Lesson routing
151. Retention Assessment

A delayed assessment MAY specifically generate:

EvidenceType::Retention

which SHOULD carry different meaning from immediate post-lesson success.

152. Transfer Assessment

Tasks intentionally using new contexts SHOULD generate:

EvidenceType::Transfer

rather than ordinary application evidence.

153. Evidence Classification

Question metadata SHALL explicitly identify evidence type where possible.

Do not rely on the Student Model to infer everything from question text.

154. Assessment Confidence

Assessment-level confidence MAY be derived from:

item count
evaluator confidence
competency coverage
difficulty distribution
response consistency
155. Measurement Confidence
pub struct AssessmentMeasurement {
    pub score: Score,
    pub confidence: Confidence,
}

A score of 90% from two weak items is not equivalent to 90% from a robust assessment.

156. Invalid Attempts

An attempt MAY be invalidated for reasons such as:

technical corruption
policy violation
faulty item set
administrative action

Invalidation SHALL preserve history.

157. Assessment Error Types
pub enum AssessmentError {
    AssessmentNotFound,
    AttemptNotFound,
    InvalidState,
    QuestionUnavailable,
    EvaluationFailed,
    EvaluatorUnavailable,
    PolicyViolation,
    TimeExpired,
    ProtectedDataAccess,
    LabUnavailable,
    ToolUnavailable,
    ValidationFailed,
}
158. Evaluator Failure

If a semantic evaluator fails:

retry
fallback evaluator
hold response
human review

depending on mode.

Do not invent a score.

159. High-Stakes Failure

Certification assessment evaluation failure SHOULD normally prefer:

pending review

over uncertain automatic grading.

160. Assessment Analytics

Track:

completion rate
average score
pass rate
time per item
hint usage
item difficulty
item discrimination
distractor selection
evaluator disagreement
regrade frequency
competency coverage
161. Learning Gain

For formative use, analytics SHOULD compare:

pre-assessment learner state
      ↓
instruction
      ↓
post-assessment evidence

rather than only reporting test score.

162. Bias and Fairness Review

Assessments SHOULD support review for systematic item-performance disparities and content issues.

Statistical differences alone SHALL not automatically imply item bias, but they should enable investigation.

163. Accessibility

Assessment content SHOULD support:

screen readers
captions
keyboard navigation
alternative media
extended-time policy
display scaling

Accommodations SHOULD be policy-driven and auditable where required.

164. Accommodation Model
pub struct AssessmentAccommodation {
    pub additional_time: Option<Duration>,
    pub alternate_presentation: Option<String>,
    pub assistive_tool_permissions: Vec<ToolId>,
}
165. Accommodation Shall Not Alter Target Competency

An accessibility accommodation SHOULD preserve the intended competency unless the assessment specifically measures the affected modality.

166. Assessment Authoring

Human-readable authoring SHOULD be supported.

Example:

question:
  id: tcp-handshake-001
  kind: ordering


  prompt: >
    Put the TCP handshake packets in order.


  competency:
    - networking.tcp.handshake.sequence


  difficulty: basic


  answer:
    - SYN
    - SYN-ACK
    - ACK
167. Free-Text Authoring Example
question:
  id: tcp-handshake-purpose
  kind: explanation


  prompt: >
    Explain why SYN-ACK is required.


  rubric:
    criteria:
      - mentions acknowledgement of client SYN
      - mentions server synchronization information
168. Protected Compilation

Source authoring files MAY contain answer keys.

Compiled learner-facing artifacts SHOULD separate protected evaluator data.

169. Assessment Linter

Future CLI:

nexa assessment validate ./assessments

Checks SHOULD include:

broken references
invalid rubrics
impossible pool selection
missing answer key
competency gaps
protected-data leaks
170. Question Bank Linter

It SHOULD also detect:

duplicate prompts
near-duplicate questions
imbalanced difficulty
overused competencies
unmapped questions
171. Assessment Testing

Tests SHOULD include:

fixed question scoring
partial credit
timing
hint restrictions
randomization reproducibility
code evaluation
semantic evaluation
low-confidence evaluator
answer-key isolation
regrade
evidence generation
172. Golden Semantic Evaluation Set

For AI evaluation, maintain examples of:

fully correct
partially correct
incorrect
misconception
off-topic
ambiguous

with expected rubric outcomes.

173. Evaluator Regression

A new evaluator model SHALL be tested against historical labeled responses before promotion for important assessments.

174. Scoring Regression

Changing rubric logic SHOULD support replay against prior responses.

This allows analysis of how scores would change.

175. Headless Assessment Testing

Assessment logic SHALL be testable without UI, Tutor, Speech, or Avatar.

176. Mock Evaluator
pub struct MockAssessmentEvaluator {
    pub results: VecDeque<EvaluatorResult>,
}

This supports deterministic orchestration tests.

177. MVP Scope

The first implementation SHOULD support:

Assessment modes:
  practice
  formative
  summative


Question types:
  multiple choice
  true/false
  ordering
  short answer


Evaluation:
  deterministic exact/choice
  simple rubric semantic evaluator


Policies:
  hints allowed/disabled
  retries
  immediate/delayed feedback


Results:
  normalized score
  pass/fail
  competency evidence
178. MVP TCP Assessment

Example:

Question 1:
Put SYN, SYN-ACK, ACK in order.


Question 2:
Which side sends SYN-ACK?


Question 3:
Explain what SYN-ACK accomplishes.


Question 4:
Given a packet trace, identify where the handshake failed.

This samples:

recall
recognition
explanation
application

rather than only one skill.

179. MVP Evidence Example

Question:

Identify which side sends SYN-ACK.

Student:

"The server."

No hint.

High confidence.

Result:

Correct
score = 1.0

Evidence:

type = Recall
outcome = Success
independence = Independent
student confidence = High
180. Hint-Assisted Example

Same question.

Student requires two hints.

Result:

Correct
score = 1.0

but evidence strength MAY be lower because:

independence = ModerateHint

Score and mastery evidence remain separate.

181. Misconception Example

Question:

"Which system sends SYN-ACK?"

Student confidently answers:

"The client."

Distractor maps to:

tcp.handshake.direction misconception

Result may produce:

incorrect evidence
+
misconception candidate
182. Lab Assessment Example

Task:

"Use the packet trace to determine why the TCP connection failed."

Evaluation:

correct failure point identified
+
correct explanation
+
appropriate diagnostic process

may generate evidence for:

TCP handshake
packet analysis
debugging methodology
183. Certification Example

Policy:

hints disabled
feedback after attempt
tools restricted
60 minute limit
one attempt
passing score 85%
critical safety items mandatory

The Tutor Engine acts only as presentation and permitted instruction interface.

184. Assessment Repository Layout
crates/
└── nexa-assessment/
    ├── src/
    │   ├── lib.rs
    │   ├── engine.rs
    │   ├── assessment.rs
    │   ├── section.rs
    │   ├── question.rs
    │   ├── pool.rs
    │   ├── attempt.rs
    │   ├── answer.rs
    │   ├── evaluation.rs
    │   ├── rubric.rs
    │   ├── scoring.rs
    │   ├── feedback.rs
    │   ├── policy.rs
    │   ├── timing.rs
    │   ├── selection.rs
    │   ├── randomization.rs
    │   ├── evidence.rs
    │   ├── regrade.rs
    │   ├── analytics.rs
    │   ├── compiler.rs
    │   ├── validation.rs
    │   ├── errors.rs
    │   └── evaluators/
    │       ├── exact.rs
    │       ├── choice.rs
    │       ├── numeric.rs
    │       ├── rubric.rs
    │       ├── semantic.rs
    │       ├── code.rs
    │       └── lab.rs
    └── tests/
        ├── scoring.rs
        ├── policy.rs
        ├── timing.rs
        ├── randomization.rs
        ├── semantic.rs
        ├── evidence.rs
        ├── security.rs
        └── regrade.rs
185. Dependency Direction
               nexa-domain
                   │
                   ▼
            nexa-assessment
             /      |       \
            ▼       ▼        ▼
      nexa-events student   labs/tools
            │
            ▼
       nexa-lessons
            │
            ▼
      nexa-orchestrator
186. Assessment Invariants

NEXA-ASMT-001 establishes these invariants:

Assessment score SHALL remain distinct from learner mastery.
Assessments SHALL produce evidence for the Student Model.
Assessment definitions SHALL be versioned.
Question definitions SHALL be versioned.
Attempts SHALL freeze the exact item versions presented.
Randomized attempts SHALL be reproducible through recorded seeds.
Protected answer data SHALL not enter ordinary TutorContext.
Learner-facing tutoring and assessment evaluation SHALL remain distinct roles.
Deterministic evaluators SHOULD be preferred where sufficient.
Non-deterministic evaluators SHALL produce confidence.
Low-confidence high-stakes grading SHALL support escalation or review.
Partial credit SHALL be explicit.
Rubric criteria SHOULD map independently to competencies where appropriate.
Hint use MAY affect evidence strength independently of score.
Assessment mode SHALL control hints, retries, tools, and feedback.
Assessment policy SHALL override ordinary Tutor preferences.
Assessment timing SHALL be authoritative outside the client UI.
Assessment results SHALL be auditable.
Manual regrades SHALL not erase original results.
Faulty items SHALL support retrospective regrading.
Certification attempts SHOULD support stronger audit requirements.
AI-generated questions SHALL require explicit permission and validation.
Generated assessment content SHALL remain constrained by competency and source scope.
Hidden tests and grading keys SHALL remain protected.
One item MAY generate evidence for multiple competencies, but attribution SHALL be specific.
Composite failure SHALL not automatically penalize every mapped competency.
Question analytics SHOULD support continuous quality improvement.
Accessibility accommodations SHOULD preserve target competency where possible.
Assessment logic SHALL be testable headlessly.
The Assessment Engine SHALL measure demonstrated performance, while the Student Model determines what that performance means over time.
187. Architecture Status

Nexa now has the complete authored learning cycle:

                 Course / Lesson
                       │
                       ▼
                 Instruction
                       │
                       ▼
                  Practice
                       │
                       ▼
                Assessment
                       │
                       ▼
                    Evidence
                       │
                       ▼
                 Student Model
                       │
                       ▼
                   Pedagogy
                       │
             ┌─────────┴─────────┐
             ▼                   ▼
         Remediation          Challenge
             │                   │
             └─────────┬─────────┘
                       ▼
                   Next Lesson

This gives the platform a formal answer to:

“How do we know the learner can actually do what Nexa has taught?”

188. Next Specification

The next specification should be:

NEXA-LAB-001 — Interactive Lab, Sandbox, Tool Execution & Observable Practice Architecture v1.0

That document should define the environment where learners actually do things rather than merely answer questions, including:

terminal sandboxes
code execution
containers
virtual machines
network simulations
lab lifecycle
tool registry
capability model
filesystem isolation
network isolation
resource quotas
commands
stdout/stderr
artifact capture
environment snapshots
reset/restore
objective detection
observation streams
student-action history
destructive-action policy
authorization
tool schemas
timeouts
cancellation
lab evidence
assessment integration
debugging workflows
security boundaries
local-first labs
remote lab providers
reproducibility
lab manifests
lab packaging
lab validation

That is the next major architectural step because it gives Nexa the ability to move from “Tell me the answer” to “Show me that you can actually do it.”
