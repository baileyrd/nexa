# NEXA-STU-001 — Student Model, Competency Engine & Knowledge Tracing Specification v1.0

**Specification ID:** NEXA-STU-001
**System:** Nexa AI Training Tutor
**Version:** 1.0
**Status:** Baseline Draft
**Depends On:** NEXA-DOM-001, NEXA-EVT-001, NEXA-PED-001, NEXA-ORCH-001
**Purpose:** Define the learner-state architecture that estimates what a student knows, how confidently the system knows it, how evidence changes that estimate, how competency relationships propagate, and how learning history supports adaptive instruction.

---

## 1. Purpose

The Student Model answers:

> **"What does the learner currently know, how certain are we, and what evidence supports that conclusion?"**

This is distinct from pedagogy.

The Student Model estimates learner state.

The Pedagogy Engine decides what to do about that state.

```text
Student Activity
      │
      ▼
   Evidence
      │
      ▼
┌─────────────────────┐
│   STUDENT MODEL     │
│                     │
│ mastery estimation  │
│ confidence          │
│ evidence ledger     │
│ knowledge tracing   │
└──────────┬──────────┘
           │
           ▼
 StudentLearningState
           │
           ▼
   Pedagogy Engine
```

---

# 2. Core Responsibilities

The Student Model SHALL own or coordinate:

* competency-state estimation;
* evidence accumulation;
* mastery estimation;
* model confidence;
* learner confidence calibration;
* retention tracking;
* forgetting/decay;
* hint-dependence effects;
* transfer evidence;
* prerequisite relationships;
* misconception evidence;
* competency history;
* mastery transitions;
* regression;
* learning velocity;
* student-state projections;
* competency update explanations.

---

# 3. Explicit Non-Responsibilities

The Student Model SHALL NOT decide:

* what Nexa should say;
* which avatar expression to use;
* which pedagogy strategy should be selected;
* lesson wording;
* question text;
* animation behavior;
* TTS behavior.

Those belong elsewhere.

---

# 4. Canonical Student Learning State

```rust
pub struct StudentLearningState {
    pub student_id: StudentId,

    pub competencies: Vec<StudentCompetencyState>,
    pub misconceptions: Vec<MisconceptionStateView>,
    pub reviews: Vec<ReviewState>,

    pub learning_velocity: LearningVelocity,
    pub confidence_profile: ConfidenceProfile,

    pub generated_at: Timestamp,
    pub projection_version: Revision,
}
```

This is a projection.

It is not the canonical history.

---

# 5. Evidence Ledger

The canonical source of learner-state change SHOULD be an append-only evidence ledger.

```text
Evidence 001
Evidence 002
Evidence 003
Evidence 004
       │
       ▼
Competency Projection
```

Competency state SHOULD be reconstructable from evidence history and policy version.

---

# 6. Evidence Record

```rust
pub struct LearningEvidence {
    pub id: EvidenceId,

    pub student_id: StudentId,
    pub competency_id: CompetencyId,

    pub evidence_type: EvidenceType,
    pub outcome: EvidenceOutcome,

    pub difficulty: EvidenceDifficulty,
    pub independence: IndependenceLevel,
    pub confidence: Option<Confidence>,

    pub source: EvidenceSource,
    pub observed_at: Timestamp,

    pub metadata: EvidenceMetadata,
}
```

---

# 7. Evidence Types

```rust
pub enum EvidenceType {
    Recognition,
    Recall,
    Explanation,
    Application,
    Demonstration,
    Debugging,
    Transfer,
    Retention,
    LabPerformance,
    Assessment,
    InstructorObservation,
}
```

Evidence types SHALL not be treated equally.

---

# 8. Evidence Outcome

```rust
pub enum EvidenceOutcome {
    Success,
    PartialSuccess,
    Failure,
    Ambiguous,
}
```

---

# 9. Evidence Difficulty

```rust
pub enum EvidenceDifficulty {
    VeryEasy,
    Easy,
    Moderate,
    Challenging,
    Advanced,
}
```

Correct performance on more difficult tasks SHOULD usually provide stronger evidence than trivial performance.

---

# 10. Independence Level

```rust
pub enum IndependenceLevel {
    Independent,
    MinorHint,
    ModerateHint,
    HeavyGuidance,
    SolutionExposed,
}
```

Independence SHALL influence evidence strength.

---

# 11. Evidence Strength Model

A baseline evidence-strength function MAY consider:

```text
base strength
× difficulty factor
× independence factor
× evidence-type factor
× recency factor
× transfer factor
```

Conceptually:

```text
strength = B × D × I × T × R × X
```

Exact coefficients SHALL be configurable.

---

# 12. Evidence-Type Weighting

A default relative ordering SHOULD generally resemble:

```text
Recognition
    <
Recall
    <
Explanation
    <
Application
    <
Demonstration
    <
Transfer
```

This is not absolute, but reflects increasing evidence of usable capability.

---

# 13. Recognition

Example:

```text
Select the correct answer.
```

Recognition is useful but weak evidence by itself.

---

# 14. Recall

Example:

```text
What are the three packets in the TCP handshake?
```

Recall is stronger because the learner produces the answer.

---

# 15. Explanation

Example:

```text
Explain why SYN-ACK is necessary.
```

Explanation can reveal conceptual structure.

---

# 16. Application

Example:

```text
Given this packet trace, determine where the handshake failed.
```

Application provides stronger evidence of practical understanding.

---

# 17. Transfer

Example:

The learner uses the same concept in a new, unfamiliar scenario.

Transfer SHOULD provide some of the strongest competency evidence.

---

# 18. Student Competency State

```rust
pub struct StudentCompetencyState {
    pub student_id: StudentId,
    pub competency_id: CompetencyId,

    pub mastery: MasteryScore,
    pub model_confidence: Confidence,

    pub status: CompetencyStatus,

    pub evidence_count: u32,
    pub independent_successes: u32,
    pub transfer_successes: u32,
    pub retention_successes: u32,

    pub last_evidence_at: Option<Timestamp>,
    pub last_mastered_at: Option<Timestamp>,

    pub trend: CompetencyTrend,
}
```

---

# 19. Competency Status

```rust
pub enum CompetencyStatus {
    Unestablished,
    Emerging,
    Developing,
    Functional,
    Proficient,
    Mastered,
    Regressed,
}
```

Status SHALL be derived from policy and state.

---

# 20. Mastery Score

```text
0.0 ───────────────────────────── 1.0
no evidence                  strong mastery
```

The score is probabilistic.

It SHALL NOT imply certainty.

---

# 21. Model Confidence

Mastery and confidence are separate.

Example:

```text
mastery = 0.82
confidence = 0.31
```

may mean:

> Existing evidence looks strong, but there is very little of it.

Whereas:

```text
mastery = 0.78
confidence = 0.95
```

may represent many consistent demonstrations.

---

# 22. Why Separate Mastery and Confidence

This prevents a common failure:

```text
one correct answer
      ↓
mastery = 0.95
```

without acknowledging uncertainty.

The system should instead represent both performance estimate and certainty in that estimate.

---

# 23. Baseline Mastery Update

The first implementation MAY use a bounded weighted update.

Conceptually:

```text
new_mastery =
    old_mastery
    + learning_rate × evidence_strength × prediction_error
```

where:

```text
prediction_error =
    observed_outcome - old_mastery
```

The exact algorithm may later be replaced by more sophisticated knowledge tracing.

---

# 24. Update Contract

```rust
pub trait CompetencyEstimator {
    fn update(
        &self,
        previous: &StudentCompetencyState,
        evidence: &LearningEvidence,
        policy: &CompetencyModelPolicy,
    ) -> StudentModelResult<CompetencyUpdate>;
}
```

---

# 25. Competency Update

```rust
pub struct CompetencyUpdate {
    pub competency_id: CompetencyId,

    pub previous_mastery: MasteryScore,
    pub new_mastery: MasteryScore,

    pub previous_confidence: Confidence,
    pub new_confidence: Confidence,

    pub status_before: CompetencyStatus,
    pub status_after: CompetencyStatus,

    pub explanation: CompetencyUpdateExplanation,
}
```

---

# 26. Explainable Updates

Every meaningful competency update SHOULD be explainable without hidden AI reasoning.

Example:

```text
Mastery:
0.61 → 0.69

Reason:
- successful application task
- moderate difficulty
- completed independently
- recent evidence

Confidence:
0.72 → 0.77
```

---

# 27. Update Explanation

```rust
pub struct CompetencyUpdateExplanation {
    pub factors: Vec<CompetencyUpdateFactor>,
    pub policy_version: String,
}
```

---

# 28. Update Factors

```rust
pub enum CompetencyUpdateFactor {
    SuccessfulEvidence,
    FailedEvidence,
    PartialEvidence,

    HighDifficulty,
    LowDifficulty,

    IndependentPerformance,
    HintDependence,

    TransferSuccess,
    TransferFailure,

    RetentionSuccess,
    RetentionFailure,

    EvidenceRecency,
    EvidenceQuantity,
    EvidenceConsistency,
}
```

---

# 29. Hint Dependence

Repeated success with substantial hints SHOULD not produce the same mastery gain as independent performance.

Conceptually:

```text
independent success     = 1.00 factor
minor hint              = 0.80
moderate hint           = 0.55
heavy guidance          = 0.30
solution exposed        = 0.05
```

These values SHALL be configurable.

---

# 30. Solution Exposure

If Nexa reveals the full answer:

```text
learner repeats answer correctly
```

the resulting evidence SHOULD be very weak.

The system must not mistake imitation for mastery.

---

# 31. Failure Evidence

Failure SHOULD usually reduce mastery less aggressively than equivalent success raises it when evidence is ambiguous.

Reasons include:

* attention slips;
* misreading;
* fatigue;
* unfamiliar wording.

Repeated failures SHOULD have stronger effect.

---

# 32. Recency

Recent evidence SHOULD generally influence current competency more than very old evidence.

However, old strong evidence SHOULD not simply disappear.

---

# 33. Evidence Decay Versus Mastery Decay

The system SHOULD distinguish:

```text
uncertainty increasing over time
```

from:

```text
actual demonstrated regression
```

Silence alone is not proof of forgetting.

---

# 34. Passive Decay

A useful initial strategy is:

```text
mastery remains relatively stable
model confidence slowly decreases
review priority increases
```

until retention evidence is collected.

---

# 35. Retention Evidence

A delayed retrieval success provides evidence that learning survived over time.

Retention tests SHOULD be valued highly.

---

# 36. Forgetting Event

If a previously mastered learner later fails a fair retention check:

```text
competency.mastered
      ↓
retention failure
      ↓
mastery decreases
confidence changes
      ↓
competency.regressed
```

---

# 37. Regression

Regression SHALL be evidence-driven.

Elapsed time alone SHOULD not automatically mark a competency as lost.

---

# 38. Review Scheduling Boundary

The Student Model SHOULD produce retention state.

The Pedagogy Engine decides how to schedule and present review.

---

# 39. Review Record

```rust
pub struct CompetencyRetentionState {
    pub competency_id: CompetencyId,

    pub last_demonstrated_at: Option<Timestamp>,
    pub last_retention_check_at: Option<Timestamp>,

    pub retention_strength: Confidence,
    pub review_urgency: ReviewUrgency,
}
```

---

# 40. Review Urgency

```rust
pub enum ReviewUrgency {
    None,
    Low,
    Moderate,
    High,
    Overdue,
}
```

---

# 41. Transfer State

```rust
pub struct TransferState {
    pub competency_id: CompetencyId,

    pub transfer_attempts: u32,
    pub transfer_successes: u32,

    pub contexts_seen: Vec<TransferContext>,
}
```

---

# 42. Transfer Context

```rust
pub struct TransferContext {
    pub context_key: String,
    pub difficulty: Difficulty,
    pub success: bool,
}
```

A learner should not receive strong transfer credit for solving the same scenario repeatedly.

---

# 43. Evidence Diversity

Model confidence SHOULD increase when evidence comes from varied contexts.

Example:

```text
5 identical multiple-choice questions
```

should generally provide less confidence than:

```text
recall
explanation
lab
transfer task
retention check
```

---

# 44. Diversity Metric

```rust
pub struct EvidenceDiversity {
    pub unique_types: u32,
    pub unique_contexts: u32,
    pub unique_sources: u32,
}
```

---

# 45. Competency Graph

Competencies SHALL support relationships.

```text
Networking
│
├── IP Addressing
│
├── Ports
│
└── TCP
    ├── Handshake
    ├── Sequencing
    └── Congestion Control
```

---

# 46. Competency Relationship Types

```rust
pub enum CompetencyRelationship {
    ParentOf,
    ChildOf,
    Requires,
    Supports,
    Overlaps,
}
```

---

# 47. Parent Competency

Parent mastery SHOULD generally be derived from children rather than independently guessed.

Example:

```text
TCP mastery
   ↓ derived from
Handshake
Sequencing
Reliability
Congestion Control
```

---

# 48. Aggregation Policy

```rust
pub enum CompetencyAggregation {
    WeightedMean,
    Minimum,
    RequiredSubset,
    Custom,
}
```

---

# 49. Weighted Parent Example

```text
TCP:
  handshake          weight 0.20
  sequencing         weight 0.25
  reliability        weight 0.25
  congestion control weight 0.30
```

Weights SHALL be curriculum-defined.

---

# 50. Minimum Aggregation

Some competencies may require:

```text
parent mastery = minimum(child masteries)
```

when every child capability is mandatory.

---

# 51. Prerequisite Propagation

Weak prerequisite evidence MAY reduce readiness for an advanced competency.

It SHOULD NOT directly rewrite unrelated competency mastery.

Example:

```text
weak IP addressing
```

may influence readiness for subnet-routing exercises, but should not arbitrarily reduce TCP handshake mastery.

---

# 52. Prerequisite Readiness

```rust
pub struct PrerequisiteReadiness {
    pub competency_id: CompetencyId,
    pub ready: bool,
    pub weak_prerequisites: Vec<CompetencyId>,
    pub confidence: Confidence,
}
```

---

# 53. Misconception Evidence

Misconceptions SHOULD use a separate evidence model.

```rust
pub struct MisconceptionEvidence {
    pub misconception_id: MisconceptionId,
    pub supporting_evidence: Vec<EvidenceId>,
    pub contradicting_evidence: Vec<EvidenceId>,
    pub confidence: Confidence,
}
```

---

# 54. Suspected Misconception

One high-confidence incorrect answer MAY create:

```text
Suspected
```

but SHOULD not normally create:

```text
Confirmed
```

without corroboration.

---

# 55. Misconception Confirmation

Confirmation MAY require:

```text
repeated error pattern
+
diagnostic response
+
high inference confidence
```

Policy SHALL be configurable.

---

# 56. Misconception Resolution

A misconception SHOULD not disappear simply because Nexa explained the correct answer.

Resolution SHOULD require corrective evidence.

Example:

```text
diagnostic failure
      ↓
intervention
      ↓
correct explanation
      ↓
near transfer
      ↓
delayed check
      ↓
resolved
```

---

# 57. Confidence Calibration

The system SHOULD track whether student self-confidence is well calibrated.

---

# 58. Confidence Profile

```rust
pub struct ConfidenceProfile {
    pub calibration: ConfidenceCalibration,
    pub observations: u32,
}
```

---

# 59. Calibration Categories

```rust
pub enum ConfidenceCalibration {
    Unknown,
    UnderConfident,
    WellCalibrated,
    OverConfident,
    Mixed,
}
```

---

# 60. Confidence Calibration Example

If a learner repeatedly reports:

```text
95% confidence
```

on incorrect answers, the system may infer overconfidence.

Pedagogy can then use additional verification.

---

# 61. Calibration Shall Not Become Judgmental

Nexa should not say:

> "You're overconfident."

unless explicitly useful and pedagogically appropriate.

The state primarily informs instructional strategy.

---

# 62. Learning Velocity

Learning velocity estimates how rapidly evidence is improving.

```rust
pub struct LearningVelocity {
    pub short_term: VelocityEstimate,
    pub medium_term: VelocityEstimate,
}
```

---

# 63. Velocity Estimate

```rust
pub struct VelocityEstimate {
    pub direction: TrendDirection,
    pub magnitude: f32,
    pub confidence: Confidence,
}
```

---

# 64. Trend Direction

```rust
pub enum TrendDirection {
    Improving,
    Stable,
    Declining,
    Unknown,
}
```

---

# 65. Why Learning Velocity Matters

Two students may both have:

```text
mastery = 0.55
```

but:

Student A:

```text
0.20 → 0.35 → 0.45 → 0.55
```

Student B:

```text
0.78 → 0.70 → 0.62 → 0.55
```

Those situations require different pedagogy.

---

# 66. Mastery Gate

Competency mastery SHOULD require policy satisfaction.

```text
mastery threshold
      +
minimum evidence
      +
minimum model confidence
      +
independence requirement
      +
transfer requirement
      +
retention requirement
      =
MASTERED
```

---

# 67. Competency Policy

```rust
pub struct CompetencyModelPolicy {
    pub mastery_threshold: MasteryScore,
    pub minimum_evidence: u32,
    pub minimum_model_confidence: Confidence,

    pub require_independent_success: bool,
    pub minimum_independent_successes: u32,

    pub require_transfer: bool,
    pub minimum_transfer_successes: u32,

    pub require_retention: bool,
    pub minimum_retention_successes: u32,
}
```

---

# 68. Status Transition

Example:

```text
Unestablished
   ↓
Emerging
   ↓
Developing
   ↓
Functional
   ↓
Proficient
   ↓
Mastered
```

Regression path:

```text
Mastered
   ↓
Regressed
```

---

# 69. Mastery Event

When policy conditions become satisfied:

```text
competency.mastered
```

SHALL be emitted.

---

# 70. Regression Event

When previously mastered competency no longer satisfies required evidence:

```text
competency.regressed
```

MAY be emitted.

---

# 71. Guess Detection

The Student Model MAY maintain a guess-likelihood estimate.

Indicators may include:

```text
correct answer
+
very low confidence
+
very short response time
+
weak historical competency
```

This SHALL remain probabilistic.

---

# 72. Guess Probability

```rust
pub struct GuessEstimate {
    pub probability: Confidence,
    pub factors: Vec<GuessFactor>,
}
```

---

# 73. Guess Factors

```rust
pub enum GuessFactor {
    LowSelfConfidence,
    VeryFastResponse,
    WeakPriorMastery,
    RecognitionOnly,
    InconsistentHistory,
}
```

A guessed correct answer SHOULD produce weaker evidence.

---

# 74. Slip Detection

Conversely:

```text
strong mastery
+
high historical consistency
+
single minor failure
```

may indicate a slip rather than knowledge loss.

---

# 75. Slip Estimate

```rust
pub struct SlipEstimate {
    pub probability: Confidence,
}
```

This can prevent overreacting to isolated mistakes.

---

# 76. Bayesian Knowledge Tracing

Future implementations MAY support BKT-style models with parameters such as:

```text
P(L0) initial knowledge
P(T)  learning transition
P(S)  slip
P(G)  guess
```

The architecture SHALL not hardwire one algorithm.

---

# 77. Item Response Models

Future versions MAY incorporate:

* item difficulty;
* learner ability;
* discrimination;
* question calibration.

This may improve assessment quality.

---

# 78. Deep Knowledge Tracing

Neural knowledge-tracing approaches MAY be explored later.

They SHALL remain behind the same estimator interface.

The core system SHOULD preserve explainable projections regardless of internal estimator sophistication.

---

# 79. Pluggable Estimator Architecture

```text
Evidence
   │
   ▼
CompetencyEstimator
   │
   ├── WeightedEstimator
   ├── BayesianEstimator
   ├── IRTBasedEstimator
   └── ExperimentalEstimator
```

---

# 80. Estimator Versioning

Each update SHOULD record:

```text
estimator type
estimator version
policy version
```

This supports reproducibility.

---

# 81. Rebuildability

Given:

```text
evidence ledger
+
competency definitions
+
policy version
+
estimator version
```

the system SHOULD be able to rebuild student competency state.

---

# 82. Projection Rebuild

```text
Evidence Store
    ↓
Replay
    ↓
Estimator
    ↓
Competency Projection
```

This is critical for future model improvements.

---

# 83. Snapshotting

For performance, projections MAY be snapshotted.

```text
Evidence 1–50,000
       ↓
snapshot
       ↓
Evidence 50,001–50,120
```

Rebuilding can start from the snapshot.

---

# 84. Snapshot Contents

```rust
pub struct StudentModelSnapshot {
    pub student_id: StudentId,
    pub competencies: Vec<StudentCompetencyState>,
    pub misconceptions: Vec<MisconceptionStateView>,
    pub review_states: Vec<CompetencyRetentionState>,

    pub evidence_sequence: u64,
    pub estimator_version: String,
    pub policy_version: String,

    pub created_at: Timestamp,
}
```

---

# 85. Canonical Versus Derived State

Canonical:

```text
Evidence Ledger
```

Derived:

```text
Competency State
Misconception State
Retention State
Learning Velocity
Confidence Calibration
```

Derived state SHOULD be rebuildable.

---

# 86. Idempotent Evidence Ingestion

Evidence IDs SHALL be unique.

Processing the same evidence twice SHALL NOT double-update mastery.

---

# 87. Evidence Ingestion Pipeline

```text
Evidence arrives
      ↓
schema validation
      ↓
duplicate check
      ↓
competency lookup
      ↓
estimator update
      ↓
projection update
      ↓
events emitted
```

---

# 88. Evidence Service Contract

```rust
#[async_trait]
pub trait StudentModelService: Send + Sync {
    async fn ingest(
        &self,
        evidence: LearningEvidence,
    ) -> StudentModelResult<CompetencyUpdate>;

    async fn state(
        &self,
        student_id: StudentId,
    ) -> StudentModelResult<StudentLearningState>;

    async fn competency(
        &self,
        student_id: StudentId,
        competency_id: CompetencyId,
    ) -> StudentModelResult<StudentCompetencyState>;
}
```

---

# 89. Batch Evidence

The service SHOULD support transactional batches.

Example:

```text
assessment completed
      ↓
12 evidence records
      ↓
atomic update
```

This avoids partially applied assessment results.

---

# 90. Batch Contract

```rust
async fn ingest_batch(
    &self,
    evidence: Vec<LearningEvidence>,
) -> StudentModelResult<Vec<CompetencyUpdate>>;
```

---

# 91. Competency Events

The Student Model SHOULD emit:

```text
competency.evidence.added
competency.updated
competency.status.changed
competency.mastered
competency.regressed

student_model.snapshot.created
student_model.projection.rebuilt

retention.review_due
retention.success
retention.failure

misconception.suspected
misconception.confirmed
misconception.resolved
```

---

# 92. Evidence Event

```json
{
  "event_type": "competency.evidence.added",
  "payload": {
    "evidence_id": "ev-3812",
    "competency_id": "networking.tcp.handshake",
    "type": "application",
    "outcome": "success",
    "independence": "independent"
  }
}
```

---

# 93. Competency Update Event

```json
{
  "event_type": "competency.updated",
  "payload": {
    "competency_id": "networking.tcp.handshake",
    "previous_mastery": 0.64,
    "new_mastery": 0.71,
    "previous_confidence": 0.74,
    "new_confidence": 0.79
  }
}
```

---

# 94. Event Causation

A competency update SHOULD reference the evidence event that caused it.

```text
answer evaluated
      ↓
evidence created
      ↓
competency updated
```

This preserves traceability.

---

# 95. Multi-Competency Evidence

A single task MAY provide evidence for multiple competencies.

Example:

A debugging lab may demonstrate:

```text
Linux shell usage
network diagnostics
TCP knowledge
systematic debugging
```

Evidence SHOULD be emitted separately per competency where practical.

---

# 96. Evidence Attribution

Automatic multi-competency attribution SHOULD be conservative.

The system SHOULD avoid claiming mastery of every concept merely touched by an exercise.

---

# 97. Negative Evidence

Failure on one composite task SHOULD not automatically penalize every related competency.

Diagnostic attribution is required where possible.

---

# 98. Example

A lab fails because the learner mistypes a command.

Do not reduce:

```text
TCP conceptual mastery
```

if the error was clearly:

```text
shell syntax
```

---

# 99. Evidence Attribution Service

A future dedicated component MAY classify evidence attribution.

```rust
pub trait EvidenceAttributor {
    fn attribute(
        &self,
        observation: LearningObservation,
    ) -> StudentModelResult<Vec<LearningEvidence>>;
}
```

---

# 100. Student Model and Assessments

The Assessment Engine produces evaluated outcomes.

The Student Model converts those outcomes into learning evidence and state updates.

Assessment scores SHALL NOT directly become competency mastery percentages.

---

# 101. Student Model and Labs

Labs may generate richer evidence than quizzes.

Potential evidence includes:

```text
command choice
sequence of actions
error recovery
independence
tool usage
final result
explanation
```

---

# 102. Process Evidence

For procedural competencies, the process may matter as much as the final result.

Example:

A learner reaches the correct answer through random trial-and-error.

That SHOULD not necessarily equal a systematic successful procedure.

---

# 103. Process Quality

```rust
pub struct ProcessQuality {
    pub systematic: Confidence,
    pub efficient: Confidence,
    pub independent: Confidence,
}
```

This MAY contribute to procedural evidence.

---

# 104. Student Model and Tutor Conversation

Conversational statements MAY generate evidence when sufficiently reliable.

Example:

Student explains:

> "The server sends SYN-ACK because it acknowledges the client's sequence number while providing its own."

This can become explanation evidence.

---

# 105. Conversation Evidence Boundary

Raw LLM interpretation SHALL not automatically mutate mastery.

Recommended flow:

```text
student explanation
      ↓
evaluation/classifier
      ↓
structured EvidenceCandidate
      ↓
confidence threshold/policy
      ↓
LearningEvidence
```

---

# 106. Evidence Candidate

```rust
pub struct EvidenceCandidate {
    pub competency_id: CompetencyId,
    pub evidence_type: EvidenceType,
    pub outcome: EvidenceOutcome,
    pub evaluator_confidence: Confidence,
}
```

---

# 107. Low-Confidence Candidate

If evaluator confidence is low:

```text
do not update strongly
```

Possible action:

```text
ask verification question
```

---

# 108. Minimum Evaluator Confidence

Policies MAY define:

```text
minimum confidence to create evidence
```

or evidence weight scaling based on evaluator confidence.

---

# 109. Source Reliability

Evidence MAY include reliability of the source/evaluator.

```rust
pub struct SourceReliability(pub Confidence);
```

For example:

```text
deterministic code test
```

may be more reliable than:

```text
semantic free-text classifier
```

for certain competencies.

---

# 110. Competency History

The system SHOULD retain history rather than only current mastery.

```rust
pub struct CompetencyHistoryPoint {
    pub timestamp: Timestamp,
    pub mastery: MasteryScore,
    pub model_confidence: Confidence,
    pub evidence_id: EvidenceId,
}
```

---

# 111. Learning Curve

History enables visualization:

```text
Mastery
1.0 |                         ●
    |                    ●
    |               ●
    |          ●
    |     ●
0.0 +-----------------------------
       time
```

---

# 112. Plateau Detection

A learner may stop improving.

```text
0.48
0.49
0.48
0.50
0.49
```

The system MAY detect a plateau and inform pedagogy.

---

# 113. Plateau State

```rust
pub enum ProgressPattern {
    Improving,
    Stable,
    Plateau,
    Regressing,
    Volatile,
    InsufficientData,
}
```

---

# 114. Volatility

Large swings may indicate:

* inconsistent understanding;
* variable task difficulty;
* guessing;
* poor evidence calibration.

Pedagogy may respond with additional diagnostic evidence.

---

# 115. Competency Dependencies

The Student Model SHOULD support prerequisite queries.

```rust
pub trait CompetencyGraphService {
    fn prerequisites(
        &self,
        competency_id: CompetencyId,
    ) -> Vec<CompetencyId>;

    fn descendants(
        &self,
        competency_id: CompetencyId,
    ) -> Vec<CompetencyId>;
}
```

---

# 116. Graph Cycles

The competency prerequisite graph SHOULD reject invalid cycles where prerequisites imply impossible circular dependency.

---

# 117. Parent Aggregation

A parent competency update MAY occur when child competency state changes.

```text
child updated
      ↓
aggregate parent
      ↓
parent updated
```

These updates SHOULD preserve causation references.

---

# 118. Readiness Projection

The Student Model MAY provide:

```rust
pub struct LearningReadiness {
    pub target: CompetencyId,
    pub readiness: ReadinessLevel,
    pub weak_prerequisites: Vec<CompetencyId>,
    pub confidence: Confidence,
}
```

---

# 119. Readiness Level

```rust
pub enum ReadinessLevel {
    Ready,
    MostlyReady,
    NeedsReview,
    NeedsRemediation,
    Unknown,
}
```

---

# 120. Privacy Boundary

The Student Model SHOULD store only information required for learning.

It SHOULD avoid unnecessary profiling unrelated to instruction.

---

# 121. User Inspectability

Learners SHOULD eventually be able to inspect:

```text
competency estimates
evidence history
completed lessons
identified misconceptions
review schedule
```

where appropriate.

---

# 122. Correctability

The architecture SHOULD support correction of erroneous evidence.

Because evidence is append-only, corrections SHOULD be represented by new records.

---

# 123. Evidence Retraction

```rust
pub struct EvidenceRetraction {
    pub evidence_id: EvidenceId,
    pub reason: RetractionReason,
    pub retracted_at: Timestamp,
}
```

Projection rebuild then ignores or counteracts the retracted evidence.

---

# 124. Retraction Reasons

```rust
pub enum RetractionReason {
    EvaluationError,
    CorruptedSource,
    Duplicate,
    AdministrativeCorrection,
}
```

---

# 125. Manual Instructor Evidence

Future instructor workflows MAY add:

```text
InstructorObservation
```

with explicit provenance.

Instructor evidence SHOULD not silently overwrite automated history.

---

# 126. Model Policy Versioning

Every student-state projection SHOULD identify:

```text
estimator version
competency policy version
graph version
```

This enables auditability.

---

# 127. Migration

When estimator logic changes significantly:

```text
old projection
      ↓
rebuild from evidence
      ↓
new projection
```

The evidence ledger allows this without losing history.

---

# 128. Experimental Estimators

Research variants MAY coexist:

```text
weighted-v1
bkt-v1
irt-v1
experimental-neural-v1
```

Only one SHOULD be authoritative for a given production projection unless explicit ensemble logic is defined.

---

# 129. Ensemble Estimation

Future implementations MAY combine estimators.

Example:

```text
weighted estimator
+
BKT
+
assessment model
      ↓
ensemble mastery estimate
```

But explainability requirements remain.

---

# 130. Performance

Student-state queries SHOULD be fast enough for every interaction.

The orchestrator should not need to replay the full evidence ledger before each tutor response.

Therefore:

```text
persistent projection + incremental update
```

is the normal runtime path.

---

# 131. Storage Architecture

Conceptually:

```text
┌─────────────────────────────┐
│ Evidence Store              │
│ append-only                 │
└─────────────┬───────────────┘
              │
              ▼
┌─────────────────────────────┐
│ Competency Projection Store │
│ current learner state       │
└─────────────┬───────────────┘
              │
              ▼
       Pedagogy / Tutor
```

---

# 132. Repository Interfaces

```rust
#[async_trait]
pub trait EvidenceRepository {
    async fn append(
        &self,
        evidence: &LearningEvidence,
    ) -> StudentModelResult<()>;

    async fn by_student(
        &self,
        student_id: StudentId,
    ) -> StudentModelResult<Vec<LearningEvidence>>;
}
```

---

# 133. Competency State Repository

```rust
#[async_trait]
pub trait CompetencyStateRepository {
    async fn get(
        &self,
        student_id: StudentId,
        competency_id: CompetencyId,
    ) -> StudentModelResult<Option<StudentCompetencyState>>;

    async fn save(
        &self,
        state: &StudentCompetencyState,
    ) -> StudentModelResult<()>;
}
```

---

# 134. Transaction Requirement

Evidence persistence and competency projection update SHOULD normally occur within one logical transaction.

```text
append evidence
      +
update projection
      +
emit outbox event
      =
commit
```

---

# 135. Transactional Outbox

For reliable event publication:

```text
database transaction
      ├── evidence
      ├── projection
      └── outbox event
             ↓
        event publisher
```

This prevents state changes without matching events.

---

# 136. Recommended Crate Structure

```text
crates/
└── nexa-student/
    ├── src/
    │   ├── lib.rs
    │   ├── service.rs
    │   ├── evidence.rs
    │   ├── estimator.rs
    │   ├── policy.rs
    │   ├── competency_state.rs
    │   ├── mastery.rs
    │   ├── confidence.rs
    │   ├── retention.rs
    │   ├── transfer.rs
    │   ├── misconception.rs
    │   ├── calibration.rs
    │   ├── velocity.rs
    │   ├── readiness.rs
    │   ├── graph.rs
    │   ├── projection.rs
    │   ├── snapshot.rs
    │   ├── errors.rs
    │   └── estimators/
    │       ├── weighted.rs
    │       ├── bayesian.rs
    │       └── experimental.rs
    └── tests/
        ├── evidence.rs
        ├── mastery.rs
        ├── hints.rs
        ├── retention.rs
        ├── transfer.rs
        ├── regression.rs
        ├── graph.rs
        └── rebuild.rs
```

---

# 137. Dependency Direction

```text
              nexa-domain
                  │
                  ▼
             nexa-student
                  │
        ┌─────────┴────────┐
        ▼                  ▼
   nexa-pedagogy      nexa-events
        │
        ▼
   orchestrator
        │
        ▼
      tutor
```

The Pedagogy Engine consumes learner state.

It SHOULD NOT recalculate mastery itself.

---

# 138. MVP Estimator

The first implementation SHOULD use a simple, transparent weighted estimator.

It SHOULD support:

```text
success / failure / partial
difficulty factor
independence factor
evidence-type factor
recency factor
student confidence
model confidence
minimum evidence
mastery gates
```

Do not begin with a neural model.

The simple estimator gives us a testable baseline.

---

# 139. MVP Example — Independent Success

Current:

```text
mastery = 0.56
model confidence = 0.62
```

Evidence:

```text
type = Application
difficulty = Challenging
outcome = Success
independence = Independent
student confidence = 0.88
```

Possible result:

```text
mastery = 0.66
model confidence = 0.69
```

with explanation:

```text
+ successful application
+ challenging task
+ independent performance
+ consistent with prior evidence
```

---

# 140. MVP Example — Hint-Assisted Success

Same current state.

Evidence:

```text
type = Application
difficulty = Challenging
outcome = Success
independence = HeavyGuidance
```

Possible result:

```text
mastery = 0.59
model confidence = 0.64
```

Success still matters, but far less.

---

# 141. MVP Example — Isolated Failure

Current:

```text
mastery = 0.86
confidence = 0.91
```

Evidence:

```text
one moderate task
failure
historically strong competency
```

Possible result:

```text
mastery = 0.82
confidence = 0.89
```

Do not collapse mastery due to one mistake.

---

# 142. MVP Example — Confident Repeated Failure

Current:

```text
mastery = 0.63
```

Evidence:

```text
failure
failure
failure

student confidence:
0.91
0.88
0.94
```

Result may include:

```text
mastery ↓
misconception suspected
diagnostic evidence requested
```

---

# 143. MVP Mastery Gate Example

Policy:

```text
mastery >= 0.85
minimum evidence = 5
independent successes >= 2
transfer successes >= 1
```

Learner:

```text
mastery = 0.91
evidence = 7
independent successes = 3
transfer successes = 0
```

Result:

```text
Proficient
not yet Mastered
```

Nexa should test transfer rather than simply marking completion.

---

# 144. Student Model Invariants

NEXA-STU-001 establishes these invariants:

1. Learner state SHALL be evidence-based.
2. Evidence history SHOULD be append-only and auditable.
3. Competency projections SHOULD be rebuildable.
4. Mastery and model confidence SHALL remain distinct.
5. A single correct answer SHALL not automatically establish mastery.
6. Recognition evidence SHALL generally be weaker than application or transfer evidence.
7. Hint-assisted success SHALL provide less mastery evidence than independent success.
8. Full answer exposure SHALL provide minimal mastery evidence.
9. Failure attribution SHOULD be competency-specific where possible.
10. One composite failure SHALL not automatically penalize every related competency.
11. Mastery gates MAY require evidence count, independence, transfer, and retention.
12. Mastery MAY regress when later evidence supports regression.
13. Time alone SHOULD primarily increase uncertainty and review urgency rather than prove forgetting.
14. Misconceptions SHALL require corroborating evidence.
15. Misconception resolution SHALL require corrective evidence.
16. Student confidence and correctness SHOULD be modeled jointly.
17. Evidence-source reliability SHOULD affect update confidence.
18. Student-state algorithms SHALL be versioned.
19. State projections SHOULD support snapshots.
20. Evidence ingestion SHALL be idempotent.
21. Important state mutations SHOULD emit events.
22. Assessment scores SHALL not be equated directly with competency mastery.
23. The student model SHALL remain separate from pedagogy.
24. The initial estimator SHOULD prioritize transparency and testability over complexity.

---

# 145. Architecture Status

We now have the adaptive-learning loop:

```text
Student
   │
   ▼
Evidence
   │
   ▼
NEXA-STU-001
"What does the learner know?"
   │
   ▼
NEXA-PED-001
"What should happen next?"
   │
   ▼
NEXA-TUTOR-001
"What should Nexa say/do?"
   │
   ▼
NEXA-ORCH-001
"Coordinate execution"
   │
   ├── NBP
   ├── Speech
   ├── Canvas
   └── Avatar
```

---

# 146. Next Specification

The next document should now be:

**NEXA-TUTOR-001 — Tutor Intelligence, Context Assembly & Response Contract Specification v1.0**

This should define:

```text
Tutor Engine interface
TutorContext construction
system/persona contract
pedagogy integration
knowledge grounding
structured output schema
tool-use planning
conversation context
response generation
streaming
confidence
uncertainty handling
citations/provenance
hallucination controls
lesson-aware responses
assessment restrictions
behavior intent generation
follow-up planning
model abstraction
local/cloud model support
fallback models
prompt architecture
context budgets
response validation
```

That is the next major piece because it defines **Nexa's actual AI intelligence contract** without allowing the model to take over responsibilities that already belong to the Student Model, Pedagogy Engine, Orchestrator, or Behavior Engine.
