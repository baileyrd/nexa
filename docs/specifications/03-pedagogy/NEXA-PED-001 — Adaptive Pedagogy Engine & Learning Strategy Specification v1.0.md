NEXA-PED-001 — Adaptive Pedagogy Engine & Learning Strategy Specification v1.0

Specification ID: NEXA-PED-001
System: Nexa AI Training Tutor
Version: 1.0
Status: Baseline Draft
Depends On: NEXA-DOM-001, NEXA-EVT-001, NEXA-ORCH-001
Purpose: Define Nexa's adaptive instructional decision engine: how it interprets learner evidence, chooses teaching strategies, manages difficulty and hints, responds to misconceptions, schedules practice, and determines what the tutor should do next.

1. Architectural Role

The Pedagogy Engine answers a fundamentally different question from the Tutor Engine.

The Tutor Engine asks:

"What should Nexa say?"

The Pedagogy Engine asks:

"What should Nexa teach or ask next, and why?"

That separation is mandatory.

Student Interaction
        │
        ▼
 Student Model
        │
        ▼
┌──────────────────────┐
│   PEDAGOGY ENGINE    │
│                      │
│ What should happen   │
│ instructionally?     │
└──────────┬───────────┘
           │
           ▼
    PedagogyDecision
           │
           ▼
      Tutor Engine
           │
           ▼
   TutorResponse

The LLM SHALL NOT be the sole authority for instructional strategy.

2. Core Responsibilities

The Pedagogy Engine SHALL determine or influence:

instructional strategy;
explanation depth;
difficulty;
question selection;
hint level;
corrective feedback;
misconception intervention;
retrieval practice;
review timing;
challenge level;
prerequisite remediation;
lesson branching;
competency verification;
transfer testing;
progression readiness.

It SHOULD operate primarily on structured learning state rather than raw conversation history.

3. Pedagogical Control Loop

Nexa's fundamental learning loop is:

        ┌─────────────────────┐
        │      PRESENT        │
        │ concept / challenge │
        └──────────┬──────────┘
                   ▼
        ┌─────────────────────┐
        │       OBSERVE       │
        │ learner response    │
        └──────────┬──────────┘
                   ▼
        ┌─────────────────────┐
        │      EVALUATE       │
        │ evidence / mastery  │
        └──────────┬──────────┘
                   ▼
        ┌─────────────────────┐
        │       DECIDE        │
        │ pedagogy strategy   │
        └──────────┬──────────┘
                   ▼
        ┌─────────────────────┐
        │        ADAPT        │
        │ next instruction    │
        └──────────┬──────────┘
                   │
                   └──────────────► PRESENT

This loop continues throughout a learning session.

4. Pedagogy Request

The orchestrator SHALL invoke the Pedagogy Engine using a typed request.

pub struct PedagogyRequest {
    pub student_id: StudentId,
    pub session_id: SessionId,


    pub course_id: Option<CourseId>,
    pub lesson_id: Option<LessonId>,
    pub lesson_step_id: Option<LessonStepId>,


    pub target_objectives: Vec<LearningObjectiveId>,
    pub target_competencies: Vec<CompetencyId>,


    pub learning_state: StudentLearningState,
    pub recent_evidence: Vec<Evidence>,


    pub interaction: PedagogyInteraction,
    pub constraints: PedagogyConstraints,
}
5. Pedagogy Interaction
pub enum PedagogyInteraction {
    LessonStart,
    LessonContinue,


    ConceptIntroduction {
        concept_id: ConceptId,
    },


    StudentQuestion,


    AnswerEvaluation {
        evaluation: AnswerEvaluation,
    },


    HintRequested {
        question_id: QuestionId,
    },


    ExplanationRequested,


    PracticeRequested,


    AssessmentCompleted {
        assessment_id: AssessmentId,
    },


    LabObservation {
        observation: LabObservation,
    },


    ReviewDue {
        competency_id: CompetencyId,
    },
}
6. Pedagogy Decision

The primary engine output SHALL be structured.

pub struct PedagogyDecision {
    pub decision_id: PedagogyDecisionId,


    pub strategy: PedagogyStrategy,
    pub action: PedagogyAction,


    pub explanation_depth: ExplanationDepth,
    pub difficulty: DifficultyTarget,


    pub hint: Option<HintDecision>,
    pub feedback: FeedbackPolicy,


    pub target_competencies: Vec<CompetencyId>,
    pub target_concepts: Vec<ConceptId>,


    pub rationale: PedagogyRationale,
    pub confidence: Confidence,


    pub follow_up: Option<PedagogyFollowUp>,
}

This becomes part of TutorContext.

7. Structured Rationale

The system SHALL NOT require private LLM chain-of-thought to explain pedagogical decisions.

Instead:

pub struct PedagogyRationale {
    pub primary_reason: PedagogyReason,
    pub contributing_factors: Vec<PedagogyFactor>,
}

Example:

Primary:
    MisconceptionDetected


Factors:
    RepeatedIncorrectAnswer
    HighStudentConfidence
    StrongPriorMastery

This is explainable without exposing hidden reasoning.

8. Pedagogy Factors
pub enum PedagogyFactor {
    NewConcept,
    LowMastery,
    ModerateMastery,
    HighMastery,


    CorrectAnswer,
    IncorrectAnswer,
    PartialAnswer,


    LowConfidence,
    HighConfidence,


    RepeatedError,
    RapidSuccess,


    MisconceptionSuspected,
    MisconceptionConfirmed,


    HintAlreadyUsed,
    MultipleHintsUsed,


    RetentionDue,
    TransferNeeded,


    PrerequisiteWeakness,


    StudentRequestedHelp,
    StudentRequestedChallenge,


    TimeConstraint,
    AssessmentConstraint,
}
9. Strategy Catalog

The baseline engine SHALL support:

pub enum PedagogyStrategy {
    DirectInstruction,
    GuidedInstruction,
    Socratic,
    GuidedDiscovery,
    Demonstration,
    WorkedExample,
    Scaffolding,
    RetrievalPractice,
    DeliberatePractice,
    Interleaving,
    Review,
    Remediation,
    Challenge,
    Transfer,
    Debugging,
    Reflection,
}

Strategies are instructional policies, not phrases for the LLM.

10. Direct Instruction

Use when:

a concept is genuinely new;
prerequisite knowledge is sufficient;
concise transmission is appropriate;
the learner explicitly asks for an explanation.

Typical sequence:

Explain
  ↓
Example
  ↓
Check understanding
  ↓
Practice

Direct instruction SHOULD NOT automatically become a long lecture.

11. Guided Instruction

Preferred when the learner has some relevant knowledge but requires structure.

small explanation
      ↓
question
      ↓
student response
      ↓
feedback
      ↓
next increment

This SHOULD be a major default mode for Nexa.

12. Socratic Strategy

Socratic instruction SHOULD guide the learner through questions rather than immediately providing conclusions.

Example:

Student:
"Why does TCP need SYN-ACK?"


Nexa:
"What does the client need to know before it can trust that
the server received its initial sequence information?"

It SHOULD NOT degenerate into endless questioning.

13. Guided Discovery

Guided discovery allows learners to infer a principle from examples or experimentation.

example
   ↓
observation
   ↓
second example
   ↓
pattern recognition
   ↓
concept articulation

Labs are especially suitable for this strategy.

14. Demonstration

Use when procedural knowledge benefits from observation.

Nexa demonstrates
       ↓
Nexa explains important decisions
       ↓
student performs similar task

Examples:

shell commands;
debugging;
programming;
configuration;
network analysis.
15. Worked Example

A worked example SHALL expose meaningful solution steps without relying on hidden model reasoning.

Problem
  ↓
Step 1 — observable operation + explanation
  ↓
Step 2 — observable operation + explanation
  ↓
Step 3
  ↓
Result
16. Scaffolding

Scaffolding temporarily reduces task difficulty.

Examples:

full task
  ↓
provide structure
  ↓
provide partial solution
  ↓
learner completes missing portion

Scaffolds SHOULD fade as competence increases.

17. Retrieval Practice

Previously learned information SHOULD periodically be recalled rather than merely reread.

learn
  ↓
delay
  ↓
retrieve
  ↓
evaluate
  ↓
strengthen

Nexa SHOULD prefer active recall over passive review when appropriate.

18. Deliberate Practice

Practice SHOULD target specific weaknesses.

competency weakness
       ↓
focused challenge
       ↓
immediate evidence
       ↓
targeted correction
       ↓
repeat with variation

Repeatedly asking identical questions is insufficient.

19. Interleaving

Once multiple related competencies exist, practice MAY mix them.

Instead of:

AAAA BBBB CCCC

Nexa may use:

A B A C B C A

This encourages discrimination between related concepts.

20. Remediation

Remediation occurs when the current learning path depends on a weak prerequisite.

current objective
      ↓
failure pattern
      ↓
prerequisite weakness detected
      ↓
temporary remediation
      ↓
prerequisite verification
      ↓
return to original objective

Nexa SHOULD avoid continuing to pile new material onto an unstable foundation.

21. Transfer

Mastery requires more than solving a memorized example.

Transfer asks whether knowledge can be applied in a different context.

TCP concept learned
       ↓
standard question solved
       ↓
different network scenario
       ↓
same concept must be applied

Transfer evidence SHOULD be stronger than simple recognition evidence.

22. Debugging Strategy

For technical instruction, debugging SHALL be treated as a distinct pedagogy.

The student SHOULD learn to:

observe
form hypothesis
select diagnostic action
execute
interpret result
revise hypothesis

Nexa SHOULD avoid immediately fixing every problem for the student.

23. Reflection

After significant exercises, Nexa MAY ask:

"What was the key clue that led you to the solution?"

Reflection can reveal whether success resulted from understanding or chance.

24. Mastery Is Probabilistic

Nexa SHALL NOT model mastery as:

knows / does not know

Instead:

MasteryScore ∈ [0.0, 1.0]

The score represents an estimate derived from evidence.

25. Mastery Bands

A default policy MAY interpret mastery as:

Range	Interpretation
0.00–0.19	Unestablished
0.20–0.39	Emerging
0.40–0.59	Developing
0.60–0.74	Functional
0.75–0.89	Proficient
0.90–1.00	Strong

These thresholds SHALL be configurable.

26. Evidence Matters More Than Raw Score

Two learners with:

mastery = 0.75

may have very different evidence.

Student A:

20 recent varied demonstrations

Student B:

2 multiple-choice answers

The pedagogy engine SHALL consider evidence quality and quantity.

27. Evidence Dimensions

Evidence SHOULD eventually be characterized by:

correctness
difficulty
independence
recency
repetition
transfer
confidence
hint usage
source reliability
28. Independence

A correct answer after substantial help is not equivalent to independent success.

Conceptually:

independent success     → stronger evidence
minor hint              → moderate evidence
guided procedure        → weaker evidence
solution revealed       → minimal mastery evidence
29. Student Confidence

Self-reported confidence SHOULD modify interpretation.

The four important cases are:

Correct + Confident
Correct + Uncertain
Incorrect + Confident
Incorrect + Uncertain

They SHALL NOT be treated identically.

30. Correct + Confident

Likely interpretation:

strong evidence

Possible action:

increase challenge
move forward
test transfer
31. Correct + Uncertain

Possible interpretation:

knowledge may exist but is unstable

Action:

brief confirmation
another retrieval opportunity
avoid unnecessary reteaching
32. Incorrect + Uncertain

Likely interpretation:

knowledge gap

Action:

hint
guided explanation
scaffold
33. Incorrect + Confident

This is especially important.

Likely interpretation:

possible misconception

Action:

diagnose before simply correcting
34. Misconception Detection

A misconception SHOULD require evidence.

incorrect answer
       ↓
high confidence
       ↓
related error repeated
       ↓
misconception suspected
       ↓
diagnostic question
       ↓
confirmed / rejected

One wrong answer SHALL NOT automatically establish a misconception.

35. Misconception Intervention

Once confirmed:

surface learner model
      ↓
create cognitive contrast
      ↓
explain correct model
      ↓
guided practice
      ↓
near-transfer check
      ↓
delayed retention check
36. Avoiding the "Just Tell Them" Failure

For a misconception, Nexa SHOULD often first expose the contradiction.

Example:

Student believes SYN-ACK occurs after ACK.

Nexa might ask:

"If the client sends its ACK before receiving the server's sequence number, what sequence number would it be acknowledging?"

This creates a meaningful conceptual conflict.

37. Hint Ladder

Nexa SHALL use progressive hints.

Recommended ladder:

0  No hint
1  Prompt
2  Concept cue
3  Narrowing cue
4  Partial procedure
5  Guided procedure
6  Solution
38. Hint Escalation
attempt
   ↓ incorrect
Hint 1
   ↓ incorrect
Hint 2
   ↓ incorrect
Hint 3
   ↓
...

Escalation SHOULD consider more than attempt count.

39. Hint Decision
pub struct HintDecision {
    pub level: HintLevel,
    pub reason: HintReason,
    pub previous_hints: u8,
    pub reveals_answer: bool,
}
40. Hint Reasons
pub enum HintReason {
    StudentRequested,
    FirstFailure,
    RepeatedFailure,
    MisconceptionIntervention,
    TimeConstraint,
    ExcessiveStruggle,
    AssessmentPolicy,
}
41. Productive Struggle

Nexa SHOULD permit meaningful struggle.

It SHALL NOT immediately intervene whenever the learner hesitates.

The goal is:

challenge without abandonment
42. Struggle Detection

Potential indicators include:

repeated errors
long response delay
repeated hint requests
rapid guessing
low confidence
abandoned attempts
repeated command failures

The engine SHOULD avoid interpreting any single signal deterministically.

43. Excessive Struggle

When struggle ceases to be productive:

difficulty ↓
scaffolding ↑
hint specificity ↑
task size ↓
44. Difficulty Model

Difficulty SHOULD be represented separately from mastery.

pub struct DifficultyTarget {
    pub nominal: Difficulty,
    pub adjustment: DifficultyAdjustment,
}
45. Challenge Zone

Nexa SHOULD seek tasks slightly beyond demonstrated capability.

Conceptually:

too easy
   ↓
low learning value


appropriate challenge
   ↓
high learning value


too difficult
   ↓
unproductive failure
46. Adaptive Difficulty Inputs

Difficulty selection MAY consider:

mastery
recent success rate
hint usage
response confidence
response time
transfer performance
task complexity
prerequisite strength
47. Difficulty Increase

Increase difficulty when evidence shows:

repeated independent success
high confidence
fast accurate responses
successful transfer
low hint usage
48. Difficulty Decrease

Decrease difficulty when:

repeated failures
heavy hint dependence
prerequisite weakness
clear overload
misconception blocks progress
49. Difficulty Should Not Oscillate

The engine SHALL include hysteresis.

Avoid:

medium
hard
medium
hard
medium
hard

after every individual response.

Adaptation SHOULD use accumulated evidence.

50. Feedback Policy
pub enum FeedbackPolicy {
    Immediate,
    Delayed,
    Minimal,
    Elaborated,
    Reflective,
    AssessmentControlled,
}
51. Immediate Feedback

Preferred during:

early skill acquisition
practice
procedural training
misconception correction
52. Delayed Feedback

May be preferred during:

summative assessment
complex problem solving
multi-step tasks

when immediate correction would invalidate later reasoning.

53. Elaborated Feedback

Feedback SHOULD often explain:

what was correct
what was incorrect
why
what principle applies
what to try next

It SHOULD NOT simply say:

Wrong.
54. Feedback Economy

Nexa SHOULD avoid overwhelming the student with unnecessary explanation after every correct answer.

Example:

Correct + strong evidence
       ↓
"Exactly."
       ↓
next challenge

not a five-minute explanation of material already demonstrated.

55. Question Selection

Questions SHOULD be selected for instructional purpose.

pub enum QuestionPurpose {
    Diagnose,
    Recall,
    Understand,
    Apply,
    Analyze,
    Debug,
    Transfer,
    VerifyMastery,
    RetentionCheck,
}
56. Diagnostic Questions

Diagnostic questions determine what the student understands before instruction.

They SHOULD maximize information gained, not simply produce a grade.

57. Verification Questions

After an explanation, Nexa SHOULD often verify understanding.

Avoid:

"Do you understand?"

Prefer:

"Which side sends the SYN-ACK, and what is it acknowledging?"

Demonstration is stronger evidence than self-report.

58. Question Variation

Repeated practice SHOULD vary:

surface wording
context
numbers
examples
failure modes
ordering
tool environment

while preserving the target competency.

59. Recognition Versus Recall

Nexa SHOULD distinguish:

recognition:
    selecting correct answer


recall:
    producing answer independently

Recall generally provides stronger evidence.

60. Recall Versus Application

Application provides stronger evidence still.

recognize
   ↓
recall
   ↓
explain
   ↓
apply
   ↓
transfer

This SHOULD influence competency evidence strength.

61. Review Scheduling

The Pedagogy Engine SHOULD support spaced review.

A competency SHALL not be considered permanently learned after one successful session.

62. Review State
pub struct ReviewState {
    pub competency_id: CompetencyId,
    pub last_success: Timestamp,
    pub next_review: Timestamp,
    pub successful_retrievals: u32,
    pub failed_retrievals: u32,
    pub interval: Duration,
}
63. Review Interval

Successful retrieval SHOULD generally increase the interval.

Failure SHOULD shorten it.

Conceptually:

10 minutes
   ↓
1 day
   ↓
3 days
   ↓
1 week
   ↓
several weeks

Exact scheduling SHALL be configurable and empirically tunable.

64. Forgetting Evidence

A previously strong competency may regress.

mastery high
    ↓
long delay
    ↓
retention failure
    ↓
confidence reduced
    ↓
review scheduled

The historical evidence remains intact.

The current estimate changes.

65. Interleaved Review

Review SHOULD be inserted naturally into later lessons where appropriate.

Example:

current lesson: HTTP


review question:
"Which transport protocol is normally carrying this HTTP connection?"

This tests retention in context.

66. Prerequisite Evaluation

Before beginning advanced instruction:

target competency
      ↓
prerequisite graph
      ↓
student competency state
      ↓
ready?
67. Prerequisite Outcomes
pub enum Readiness {
    Ready,
    ReadyWithReview,
    RemediationRecommended,
    Blocked,
}
68. Ready With Review

The learner may have enough mastery to continue but benefit from a quick refresher.

This SHOULD not force unnecessary full remediation.

69. Adaptive Lesson Branching

Pedagogy MAY influence lesson navigation.

lesson step
    ↓
evidence
    ↓
PedagogyDecision
    ├── continue
    ├── extra example
    ├── practice
    ├── remediation
    ├── skip mastered content
    └── challenge branch
70. Mastered Content

When strong evidence already exists, Nexa SHOULD avoid requiring redundant beginner instruction.

Possible action:

brief verification
      ↓
skip
71. Overlearning

Some competencies may require continued practice even after initial mastery.

Examples:

command syntax
safety procedures
critical troubleshooting steps

Competency definitions MAY declare overlearning requirements.

72. Competency Learning Policy
pub struct CompetencyLearningPolicy {
    pub mastery_threshold: MasteryScore,
    pub minimum_evidence: u32,
    pub require_transfer: bool,
    pub require_retention: bool,
    pub overlearning: Option<OverlearningPolicy>,
}
73. Evidence Sufficiency

A mastery threshold alone SHALL NOT necessarily satisfy a competency.

Example:

mastery = 0.92
evidence_count = 1

may still be insufficient.

74. Mastery Gate

Conceptually:

score threshold met
        +
minimum evidence met
        +
required transfer met
        +
retention requirement met
        =
competency mastered
75. Assessment Boundary

The Pedagogy Engine MAY recommend assessment behavior.

The Assessment Engine remains authoritative for assessment rules.

Pedagogy
   ↓
recommendation


Assessment Policy
   ↓
permission
76. Assessment Hint Restriction

If assessment policy states:

hints = disabled

the Pedagogy Engine SHALL NOT override it.

77. Student Choice

Learners SHOULD retain agency where pedagogically acceptable.

They may request:

more detail
less detail
another example
hint
challenge
skip
review
practice

The engine SHOULD honor these unless constrained by assessment, safety, or required learning policy.

78. Explanation Depth Adaptation

Depth MAY adapt using:

student preference
mastery
question complexity
prior explanations
student requests
available time
79. Avoiding Explanation Loops

If a student repeatedly fails after explanation, Nexa SHOULD NOT simply produce the same explanation with more words.

Instead change representation:

verbal explanation
      ↓ failure
diagram
      ↓ failure
worked example
      ↓
guided practice
80. Representation Switching

The engine SHOULD support:

pub enum InstructionRepresentation {
    Verbal,
    Textual,
    Diagram,
    Analogy,
    WorkedExample,
    Demonstration,
    InteractiveLab,
    SocraticQuestioning,
}
81. Modality Selection

Selection SHOULD consider both:

student preference
        +
instructional suitability

Preference SHALL NOT automatically override effectiveness.

82. Analogy Policy

Analogies MAY be useful but can create misconceptions.

Nexa SHOULD:

introduce analogy
      ↓
explain mapping
      ↓
explicitly identify where analogy breaks down
83. Error Classification

Incorrect responses SHOULD be classified where possible.

pub enum LearningErrorType {
    KnowledgeGap,
    Misconception,
    ProceduralError,
    RecallFailure,
    CalculationError,
    AttentionError,
    MisreadQuestion,
    ToolUsageError,
    Unknown,
}

Different error types require different interventions.

84. Knowledge Gap Response

Typical response:

teach missing information
      ↓
guided practice
      ↓
verify
85. Procedural Error Response

Typical response:

identify failed step
      ↓
demonstrate or cue
      ↓
student retries

Do not necessarily reteach the entire concept.

86. Attention Error

If the learner clearly understands the concept but makes an isolated mistake:

brief correction

may be better than remediation.

87. Error Pattern Detection

Patterns SHOULD be evaluated across evidence.

error A
error A
error A
     ↓
pattern


error A
error B
error C
     ↓
possible broader weakness
88. Pedagogy State

The engine MAY maintain per-competency instructional state.

pub struct PedagogyState {
    pub competency_id: CompetencyId,
    pub current_strategy: PedagogyStrategy,
    pub current_difficulty: Difficulty,
    pub hint_level: HintLevel,
    pub consecutive_successes: u32,
    pub consecutive_failures: u32,
    pub representation_history: Vec<InstructionRepresentation>,
}
89. State Persistence

Not all pedagogy state requires permanent storage.

Persist:

meaningful long-term learning state

Avoid persisting excessive runtime minutiae unless useful for research or diagnostics.

90. Decision Determinism

Given the same:

policy
learning state
evidence
interaction

the core rule-based Pedagogy Engine SHOULD produce deterministic decisions where practical.

This greatly improves testing.

91. AI-Assisted Pedagogy

LLMs MAY assist with:

classifying student explanations
detecting likely misconceptions
generating examples
generating question variants
rewriting explanations

But structured policy SHALL remain authoritative.

92. Hybrid Engine

The preferred architecture is:

              Pedagogy Request
                     │
            ┌────────┴────────┐
            ▼                 ▼
       Rule Engine       AI Classifier
            │                 │
            └────────┬────────┘
                     ▼
               Policy Engine
                     │
                     ▼
             PedagogyDecision

This combines predictability with flexibility.

93. AI Recommendation Boundary

An AI classifier MAY return:

pub struct PedagogyRecommendation {
    pub suspected_error: Option<LearningErrorType>,
    pub suspected_misconception: Option<String>,
    pub suggested_strategy: Option<PedagogyStrategy>,
    pub confidence: Confidence,
}

The policy engine decides whether to accept it.

94. Low-Confidence AI Recommendation

If AI confidence is low:

do not make strong learner-state mutation

Instead:

ask diagnostic question
95. Evidence Before Mutation

A model saying:

"The student appears confused about sequence numbers."

SHALL NOT itself establish a confirmed misconception.

It may create:

misconception.suspected

followed by diagnostic evidence.

96. Pedagogical Constraints
pub struct PedagogyConstraints {
    pub assessment_mode: Option<AssessmentMode>,
    pub hints_allowed: bool,
    pub solutions_allowed: bool,
    pub tools_allowed: bool,
    pub maximum_time: Option<Duration>,
    pub required_objectives: Vec<LearningObjectiveId>,
}
97. Time-Constrained Learning

If limited time remains, strategy MAY change.

Example:

20 minutes available
      ↓
focus on critical objectives
      ↓
reduce optional examples
      ↓
preserve verification

Time pressure SHOULD NOT automatically eliminate competency verification.

98. Curriculum Authority

Pedagogy MAY adapt within curriculum constraints.

It SHALL NOT silently redefine required objectives.

Curriculum:
    what must be learned


Pedagogy:
    how best to get there
99. Safety-Critical Training

Certain competencies MAY require stricter pedagogy.

Policies may require:

no skipping
minimum practice count
mandatory assessment
retention check
independent demonstration

The engine SHALL support such constraints.

100. Pedagogy Events

The engine SHOULD emit:

pedagogy.decision.requested
pedagogy.strategy.selected
pedagogy.strategy.changed


pedagogy.difficulty.increased
pedagogy.difficulty.decreased


pedagogy.hint.selected
pedagogy.hint.escalated


pedagogy.remediation.started
pedagogy.remediation.completed


pedagogy.review.scheduled
pedagogy.review.completed


pedagogy.misconception.suspected
pedagogy.misconception.confirmed
pedagogy.misconception.resolved


pedagogy.mastery.detected
101. Strategy Event
{
  "event_type": "pedagogy.strategy.selected",
  "payload": {
    "competency_id": "networking.tcp.handshake",
    "strategy": "guided_instruction",
    "reason": "low_mastery",
    "confidence": 0.91
  }
}
102. Decision Trace

Every important decision SHOULD be inspectable.

Example:

Decision #P8421


Target:
networking.tcp.handshake


Strategy:
GuidedInstruction


Reason:
MisconceptionSuspected


Evidence:
answer-118
answer-124


Difficulty:
Maintain


Hint:
Concept


Confidence:
0.88

This will be essential for debugging adaptive behavior.

103. Pedagogy Timeline

Developer tooling SHOULD eventually show:

10:21:04 concept introduced
10:22:17 answer incorrect
10:22:18 hint level 1
10:23:01 answer incorrect
10:23:02 misconception suspected
10:23:14 diagnostic question
10:24:02 misconception confirmed
10:24:04 strategy → remediation
10:26:51 near-transfer success
10:26:52 misconception resolved
104. Pedagogy Metrics

Useful system-level metrics include:

time_to_mastery
attempts_to_mastery
hint_usage
remediation_frequency
misconception_resolution_rate
retention_success
transfer_success
difficulty_adjustment_frequency
strategy effectiveness
student correction rate
105. Strategy Effectiveness

Nexa SHOULD eventually evaluate whether instructional strategies work.

Conceptually:

strategy
   +
student state
   +
competency
   ↓
subsequent evidence improvement

This permits evidence-based tuning.

106. No Naive Optimization

The system SHALL NOT optimize solely for:

answer correctness

That could encourage over-hinting and answer revelation.

Better outcomes include:

independent success
retention
transfer
reduced hint dependency
mastery stability
107. Student Frustration

Future versions MAY estimate frustration or disengagement.

Such estimates SHALL be treated as uncertain signals rather than facts.

They SHOULD influence intervention cautiously.

108. Emotional Adaptation

Avatar emotion SHOULD support pedagogy without becoming patronizing.

Examples:

student succeeds after struggle
    → warm satisfaction


student makes ordinary mistake
    → neutral/encouraging


student is thinking
    → attentive silence

Avoid exaggerated celebration for trivial successes.

109. Praise Policy

Praise SHOULD emphasize meaningful achievement, strategy, or persistence rather than generic praise after every action.

Preferred:

"Good catch—you used the sequence number to rule that out."

Less useful:

"Amazing! You're a genius!"

110. Correction Tone

Correction SHOULD be clear without becoming unnecessarily harsh.

Nexa SHOULD distinguish:

conceptual correction
procedural correction
minor slip
safety-critical correction

Avatar behavior and speech style can reflect the difference.

111. Pedagogy → Behavior Intent

The engine MAY provide behavior guidance.

Example:

pub struct PedagogyBehaviorGuidance {
    pub tone: InstructionalTone,
    pub emphasis: EmphasisLevel,
    pub learner_space: LearnerSpace,
}

But it SHALL NOT generate animation commands.

112. Learner Space
pub enum LearnerSpace {
    Normal,
    EncourageResponse,
    AllowThinkingTime,
    MinimizeInterruption,
    IncreaseGuidance,
}

This lets pedagogy influence conversational timing.

113. Wait-Time Policy

After asking a meaningful question, Nexa SHOULD allow appropriate thinking time.

It SHOULD NOT immediately fill silence with hints.

Wait time MAY depend on:

question difficulty
student history
task type
response modality
114. Hint Delay

The engine MAY recommend:

pub struct HintTiming {
    pub minimum_wait: Duration,
    pub offer_after: Duration,
    pub auto_hint_after: Option<Duration>,
}
115. Mastery Detection

A competency SHOULD be declared mastered only when its learning policy is satisfied.

mastery estimate
      +
evidence sufficiency
      +
independence
      +
transfer requirement
      +
retention requirement
      ↓
MASTERED
116. Mastery Is Reversible

Mastery SHALL NOT necessarily be permanent.

mastered
   ↓
retention failure
   ↓
regressed
   ↓
review
   ↓
restored

This aligns competency state with demonstrated capability.

117. Knowledge Tracing Boundary

The Pedagogy Engine consumes mastery estimates.

The Student/Competency Engine SHALL own the mathematical knowledge-tracing model.

That separation becomes important for NEXA-STU-001.

118. Pedagogy Engine Contract
#[async_trait]
pub trait PedagogyEngine: Send + Sync {
    async fn decide(
        &self,
        request: PedagogyRequest,
    ) -> PedagogyResult<PedagogyDecision>;


    async fn readiness(
        &self,
        request: ReadinessRequest,
    ) -> PedagogyResult<ReadinessDecision>;


    async fn schedule_review(
        &self,
        request: ReviewRequest,
    ) -> PedagogyResult<ReviewDecision>;
}
119. Policy Engine

Policies SHOULD be composable.

Global Policy
     │
     ├── Curriculum Policy
     ├── Competency Policy
     ├── Assessment Policy
     ├── Student Preference Policy
     └── Session Policy
              │
              ▼
        Effective Policy

Higher-authority constraints override lower-level preferences.

120. Policy Precedence

Recommended precedence:

Safety / Security
       ↓
Assessment / Certification
       ↓
Curriculum Requirement
       ↓
Competency Policy
       ↓
Pedagogical Optimization
       ↓
Student Preference
       ↓
Presentation Preference
121. Configurable Policy

Policy SHOULD be data-driven where practical.

Example:

competency:
  networking.tcp.handshake:


    mastery_threshold: 0.85
    minimum_evidence: 5


    require:
      transfer: true
      retention: true


    hints:
      maximum_level_before_remediation: 4


    difficulty:
      increase_after_independent_successes: 3
      decrease_after_failures: 2

Hardcoding all pedagogical rules into Rust SHOULD be avoided.

122. Rule Engine

The initial implementation does not require a complex expert-system framework.

A deterministic rules layer is sufficient:

facts
  ↓
ordered rules
  ↓
candidate actions
  ↓
policy resolution
  ↓
PedagogyDecision
123. Rule Example

Conceptually:

if evidence.repeated_failure()
    && learner.mastery(target) < MasteryScore::new(0.50)?
{
    decision.strategy = PedagogyStrategy::Remediation;
    decision.difficulty = DifficultyAdjustment::Decrease;
}
124. Rule Conflict

Multiple rules may apply.

Example:

student requested challenge
+
recent failures

The engine SHALL resolve conflicts deterministically.

125. Rule Priority

Rules SHOULD carry explicit priority or policy authority.

pub struct PedagogyRule {
    pub id: RuleId,
    pub priority: RulePriority,
    pub condition: RuleCondition,
    pub action: RuleAction,
}
126. Decision Explanation

Developer tools SHOULD be able to answer:

Why did Nexa choose remediation?

Example:

Rule PED-REM-004 matched.


Conditions:
- consecutive_failures >= 2
- target mastery < 0.50
- prerequisite weakness detected


Selected:
Remediation


Suppressed:
StudentRequestedChallenge

This is far more useful than an opaque AI decision.

127. Simulation Mode

The Pedagogy Engine SHOULD eventually support offline simulation.

synthetic student
      ↓
pedagogy engine
      ↓
simulated evidence
      ↓
thousands of learning paths

This enables policy testing before exposing changes to students.

128. Replay Evaluation

Historical sessions MAY be replayed against new pedagogy versions.

historical evidence
       ↓
old policy → decision A
new policy → decision B
       ↓
comparison

This supports regression analysis.

129. Policy Versioning

Every decision SHOULD identify the pedagogy policy version.

pub struct PedagogyPolicyVersion {
    pub version: String,
    pub hash: String,
}

This is important for reproducibility.

130. Research Boundary

Nexa SHOULD allow experimental pedagogy policies without destabilizing the production baseline.

stable policy
experimental policy A
experimental policy B

Experiments SHOULD be explicitly versioned and auditable.

131. Recommended Crate Structure
crates/
└── nexa-pedagogy/
    ├── src/
    │   ├── lib.rs
    │   ├── engine.rs
    │   ├── request.rs
    │   ├── decision.rs
    │   ├── policy.rs
    │   ├── rule.rs
    │   ├── readiness.rs
    │   ├── difficulty.rs
    │   ├── hints.rs
    │   ├── feedback.rs
    │   ├── misconception.rs
    │   ├── review.rs
    │   ├── transfer.rs
    │   ├── remediation.rs
    │   ├── representation.rs
    │   ├── metrics.rs
    │   ├── errors.rs
    │   └── strategies/
    │       ├── direct.rs
    │       ├── guided.rs
    │       ├── socratic.rs
    │       ├── discovery.rs
    │       ├── retrieval.rs
    │       ├── deliberate.rs
    │       └── debugging.rs
    └── tests/
        ├── rules.rs
        ├── hints.rs
        ├── difficulty.rs
        ├── misconceptions.rs
        ├── readiness.rs
        ├── review.rs
        └── regression.rs
132. Dependency Direction
                  nexa-domain
                      │
                      ▼
                nexa-pedagogy
                 /         \
                ▼           ▼
        student model    curriculum
                \           /
                 └────┬─────┘
                      ▼
               orchestrator
                      │
                      ▼
                 tutor engine

The Tutor Engine consumes the pedagogy decision.

It does not own it.

133. MVP Pedagogy Scope

The first implementation SHOULD support:

DirectInstruction
GuidedInstruction
Socratic
Remediation
RetrievalPractice


Difficulty:
    increase
    maintain
    decrease


Hints:
    levels 0–6


Evidence:
    correct
    incorrect
    partial
    self-confidence


Mastery:
    simple threshold input


Misconceptions:
    suspected / confirmed


Feedback:
    immediate / elaborated

Advanced adaptive algorithms can follow.

134. MVP Decision Example

Input:

Competency:
networking.tcp.handshake


Mastery:
0.44


Recent evidence:
incorrect
incorrect


Confidence:
0.82


Current hint level:
1

Decision:

Strategy:
Remediation


Reason:
RepeatedError + HighConfidence


Action:
DiagnosticQuestion


Difficulty:
Decrease


Hint:
Concept


Misconception:
Suspected


Follow-up:
VerifyConceptualModel
135. MVP Success Example

Input:

Mastery:
0.78


Recent evidence:
correct
correct
correct


Hints:
none


Confidence:
0.91

Decision:

Strategy:
Challenge


Difficulty:
Increase


Action:
TransferQuestion


Explanation depth:
Minimal


Feedback:
Minimal

Nexa should stop reteaching and test whether the learner can generalize.

136. Pedagogy Invariants

NEXA-PED-001 establishes these invariants:

The Pedagogy Engine SHALL remain distinct from the Tutor Engine.
Instructional decisions SHOULD be represented structurally.
LLM hidden reasoning SHALL NOT be required for decision explainability.
Mastery SHALL be treated as probabilistic.
Evidence quality SHALL matter in addition to quantity.
Correctness and student confidence SHALL be interpreted jointly.
A single incorrect answer SHALL NOT automatically establish a misconception.
Hints SHOULD escalate progressively.
Nexa SHOULD permit productive struggle.
Difficulty SHALL adapt without excessive oscillation.
Repeated failure SHOULD trigger strategy change, not merely longer explanations.
Mastery SHOULD require sufficient evidence.
Transfer and retention MAY be mandatory competency requirements.
Mastery MAY regress.
Assessment policy SHALL override normal pedagogical freedom.
Curriculum requirements SHALL remain authoritative.
Student preferences SHOULD be honored where higher-level constraints permit.
AI-generated pedagogy recommendations SHALL remain subordinate to policy.
Important decisions SHALL be auditable.
Pedagogy policies SHALL be versionable.
Historical sessions SHOULD support policy replay.
The system SHOULD optimize for durable independent capability rather than immediate answer correctness.
137. The Architecture Now Has a Learning Brain

We now have:

NEXA-CBS-001
Character & Behavior
        │
        ▼
NEXA-DOM-001
Core Domain Model
        │
   ┌────┴────┐
   ▼         ▼
EVT-001    NBP-001
   │         │
   └────┬────┘
        ▼
NEXA-ORCH-001
Runtime Orchestrator
        │
        ▼
NEXA-PED-001
Adaptive Pedagogy
        │
        ▼
"What should Nexa teach next?"

There is an important architectural dependency we should define before building NEXA-TUTOR-001.

138. Next Specification — NEXA-STU-001

The next document should be:

NEXA-STU-001 — Student Model, Competency Engine & Knowledge Tracing Specification v1.0

This is the engine that gives NEXA-PED-001 the learner state it needs.

It should define:

StudentLearningState
CompetencyGraph
StudentCompetency
Evidence Ledger
Evidence weighting
Knowledge tracing
Mastery estimation
Confidence estimation
Decay / forgetting
Retention
Transfer evidence
Hint-dependence penalties
Guess detection
Misconception evidence
Prerequisite propagation
Competency history
Learning velocity
Mastery gates
Projection rebuilding
event sourcing
snapshots
explainable competency updates

The critical pipeline will then become:

               STUDENT
                  │
                  ▼
              Evidence
                  │
                  ▼
         ┌─────────────────┐
         │ NEXA-STU-001    │
         │ Student Model   │
         └────────┬────────┘
                  │
          "What does the
           learner know?"
                  │
                  ▼
         ┌─────────────────┐
         │ NEXA-PED-001    │
         │ Pedagogy Engine │
         └────────┬────────┘
                  │
          "What should we
             do next?"
                  │
                  ▼
         ┌─────────────────┐
         │ NEXA-TUTOR-001  │
         │ Tutor Engine    │
         └────────┬────────┘
                  │
          "How should Nexa
          communicate it?"
                  │
                  ▼
           Speech + Avatar

That's the architecture I'd build next: NEXA-STU-001. It gives us the missing half of adaptive instruction—the mathematical and evidence-based model of what the learner actually knows.
