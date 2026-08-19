//! Deterministic, headless assessment contracts and scoring.
#![forbid(unsafe_code)]

use nexa_domain::{
    AssessmentId, AssessmentItemInstanceId, AttemptId, CompetencyId, EvidenceId, ProtocolVersion,
    QuestionId, ResponseId, RubricCriterionId, RubricId, SemanticKey, StudentId, Timestamp,
};
use nexa_student::{
    EvidenceDifficulty, EvidenceOutcome, EvidenceSource, EvidenceType, IndependenceLevel,
    LearningEvidence,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const SCORING_POLICY_V1: ProtocolVersion = ProtocolVersion::new(1, 0);

#[derive(Clone, Debug, Error, PartialEq)]
pub enum AssessmentError {
    #[error("invalid assessment contract: {0}")]
    InvalidContract(&'static str),
    #[error("request does not belong to this student or assessment")]
    ScopeMismatch,
    #[error("scoring policy mismatch: expected {expected}, got {actual}")]
    PolicyMismatch {
        expected: ProtocolVersion,
        actual: ProtocolVersion,
    },
    #[error("attempt state does not permit this operation")]
    InvalidState,
    #[error("timestamp regressed")]
    TimestampRegression,
    #[error("response identifier was replayed with conflicting content")]
    ConflictingReplay,
    #[error("question is not part of the frozen attempt")]
    UnknownQuestion,
    #[error("response shape is incompatible with the question evaluator")]
    IncompatibleResponse,
    #[error("evidence identifiers do not exactly cover the question competencies")]
    InvalidEvidenceMapping,
}

fn text(value: &str) -> Result<(), AssessmentError> {
    if value.trim().is_empty() || value.chars().count() > 4_000 {
        Err(AssessmentError::InvalidContract(
            "text must contain 1..=4000 characters",
        ))
    } else {
        Ok(())
    }
}
fn sorted_unique<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|p| p[0] < p[1])
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Score(f64);
impl Score {
    pub fn new(value: f64) -> Result<Self, AssessmentError> {
        if value.is_finite() && (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(AssessmentError::InvalidContract(
                "score must be finite and within 0..=1",
            ))
        }
    }
    pub const fn get(self) -> f64 {
        self.0
    }
}
impl<'de> Deserialize<'de> for Score {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::new(f64::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentMode {
    Practice,
    Formative,
    Summative,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionPurpose {
    Recognize,
    Recall,
    Explain,
    Apply,
    Debug,
    Transfer,
    RetentionCheck,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ResponseValue {
    Choice {
        choice: SemanticKey,
    },
    Boolean {
        value: bool,
    },
    Text {
        value: String,
    },
    Ordering {
        values: Vec<SemanticKey>,
    },
    Rubric {
        levels: BTreeMap<RubricCriterionId, Score>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RubricCriterion {
    pub id: RubricCriterionId,
    pub description: String,
    pub weight: Score,
    pub competency_id: CompetencyId,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RubricWire")]
pub struct Rubric {
    pub id: RubricId,
    criteria: Vec<RubricCriterion>,
}
#[derive(Deserialize)]
struct RubricWire {
    id: RubricId,
    criteria: Vec<RubricCriterion>,
}
impl TryFrom<RubricWire> for Rubric {
    type Error = AssessmentError;
    fn try_from(mut w: RubricWire) -> Result<Self, Self::Error> {
        if w.criteria.is_empty() {
            return Err(AssessmentError::InvalidContract(
                "rubric criteria must be nonempty",
            ));
        }
        w.criteria.sort_by_key(|c| c.id);
        if !sorted_unique(&w.criteria.iter().map(|c| c.id).collect::<Vec<_>>()) {
            return Err(AssessmentError::InvalidContract(
                "duplicate rubric criterion",
            ));
        }
        for c in &w.criteria {
            text(&c.description)?;
            if c.weight.get() <= 0.0 {
                return Err(AssessmentError::InvalidContract(
                    "rubric weights must be positive",
                ));
            }
        }
        let sum: f64 = w.criteria.iter().map(|c| c.weight.get()).sum();
        if (sum - 1.0).abs() > 1e-9 {
            return Err(AssessmentError::InvalidContract(
                "rubric weights must sum to one",
            ));
        }
        Ok(Self {
            id: w.id,
            criteria: w.criteria,
        })
    }
}
impl Rubric {
    pub fn new(id: RubricId, criteria: Vec<RubricCriterion>) -> Result<Self, AssessmentError> {
        Self::try_from(RubricWire { id, criteria })
    }
    pub fn criteria(&self) -> &[RubricCriterion] {
        &self.criteria
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Evaluation {
    Choice {
        correct: SemanticKey,
    },
    Boolean {
        correct: bool,
    },
    Exact {
        expected: String,
        case_sensitive: bool,
    },
    Ordering {
        expected: Vec<SemanticKey>,
    },
    Rubric {
        rubric_id: RubricId,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Question {
    pub id: QuestionId,
    pub version: ProtocolVersion,
    pub prompt: String,
    pub purpose: QuestionPurpose,
    pub competency_ids: Vec<CompetencyId>,
    pub evaluation: Evaluation,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "AssessmentWire")]
pub struct Assessment {
    pub id: AssessmentId,
    pub version: ProtocolVersion,
    pub title: String,
    pub mode: AssessmentMode,
    pub scoring_policy_version: ProtocolVersion,
    pub passing_score: Score,
    questions: Vec<Question>,
    rubrics: Vec<Rubric>,
}
#[derive(Deserialize)]
struct AssessmentWire {
    id: AssessmentId,
    version: ProtocolVersion,
    title: String,
    mode: AssessmentMode,
    scoring_policy_version: ProtocolVersion,
    passing_score: Score,
    questions: Vec<Question>,
    rubrics: Vec<Rubric>,
}
impl TryFrom<AssessmentWire> for Assessment {
    type Error = AssessmentError;
    fn try_from(mut w: AssessmentWire) -> Result<Self, Self::Error> {
        text(&w.title)?;
        if w.questions.is_empty() {
            return Err(AssessmentError::InvalidContract(
                "assessment questions must be nonempty",
            ));
        }
        if w.scoring_policy_version != SCORING_POLICY_V1 {
            return Err(AssessmentError::PolicyMismatch {
                expected: SCORING_POLICY_V1,
                actual: w.scoring_policy_version,
            });
        }
        w.questions.sort_by_key(|q| q.id);
        w.rubrics.sort_by_key(|r| r.id);
        if !sorted_unique(&w.questions.iter().map(|q| q.id).collect::<Vec<_>>()) {
            return Err(AssessmentError::InvalidContract("duplicate question"));
        }
        if !sorted_unique(&w.rubrics.iter().map(|r| r.id).collect::<Vec<_>>()) {
            return Err(AssessmentError::InvalidContract("duplicate rubric"));
        }
        let rubrics: BTreeMap<_, _> = w.rubrics.iter().map(|r| (r.id, r)).collect();
        for q in &mut w.questions {
            text(&q.prompt)?;
            q.competency_ids.sort();
            if q.competency_ids.is_empty() || !sorted_unique(&q.competency_ids) {
                return Err(AssessmentError::InvalidContract(
                    "question competency mappings must be nonempty and unique",
                ));
            }
            if let Evaluation::Ordering { expected } = &q.evaluation {
                if expected.is_empty()
                    || expected.iter().collect::<BTreeSet<_>>().len() != expected.len()
                {
                    return Err(AssessmentError::InvalidContract(
                        "ordering key must be nonempty and unique",
                    ));
                }
            }
            if let Evaluation::Exact { expected, .. } = &q.evaluation {
                text(expected)?;
            }
            if let Evaluation::Rubric { rubric_id } = q.evaluation {
                let rubric = rubrics
                    .get(&rubric_id)
                    .ok_or(AssessmentError::InvalidContract(
                        "dangling rubric reference",
                    ))?;
                let rubric_competencies = rubric
                    .criteria
                    .iter()
                    .map(|criterion| criterion.competency_id)
                    .collect::<BTreeSet<_>>();
                let question_competencies =
                    q.competency_ids.iter().copied().collect::<BTreeSet<_>>();
                if rubric_competencies != question_competencies {
                    return Err(AssessmentError::InvalidContract(
                        "rubric criteria must exactly cover question competencies",
                    ));
                }
            }
        }
        Ok(Self {
            id: w.id,
            version: w.version,
            title: w.title,
            mode: w.mode,
            scoring_policy_version: w.scoring_policy_version,
            passing_score: w.passing_score,
            questions: w.questions,
            rubrics: w.rubrics,
        })
    }
}
impl Assessment {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: AssessmentId,
        version: ProtocolVersion,
        title: impl Into<String>,
        mode: AssessmentMode,
        scoring_policy_version: ProtocolVersion,
        passing_score: Score,
        questions: Vec<Question>,
        rubrics: Vec<Rubric>,
    ) -> Result<Self, AssessmentError> {
        Self::try_from(AssessmentWire {
            id,
            version,
            title: title.into(),
            mode,
            scoring_policy_version,
            passing_score,
            questions,
            rubrics,
        })
    }
    pub fn questions(&self) -> &[Question] {
        &self.questions
    }
    pub fn rubrics(&self) -> &[Rubric] {
        &self.rubrics
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationOutcome {
    Correct,
    Partial,
    Incorrect,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ItemResult {
    pub question_id: QuestionId,
    pub score: Score,
    pub outcome: EvaluationOutcome,
    pub rationale_code: SemanticKey,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssessmentResponse {
    pub id: ResponseId,
    pub student_id: StudentId,
    pub assessment_id: AssessmentId,
    pub question_id: QuestionId,
    pub value: ResponseValue,
    pub submitted_at: Timestamp,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptState {
    Created,
    Active,
    Paused,
    Submitted,
    Completed,
    Failed,
    Invalidated,
    Cancelled,
}
impl AttemptState {
    fn terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Invalidated | Self::Cancelled
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "AttemptWire")]
pub struct AssessmentAttempt {
    pub id: AttemptId,
    pub assessment_id: AssessmentId,
    pub assessment_version: ProtocolVersion,
    pub student_id: StudentId,
    pub policy_version: ProtocolVersion,
    pub state: AttemptState,
    pub started_at: Timestamp,
    pub updated_at: Timestamp,
    pub completed_at: Option<Timestamp>,
    items: Vec<AssessmentItemInstance>,
    responses: Vec<AssessmentResponse>,
    results: Vec<ItemResult>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssessmentItemInstance {
    pub id: AssessmentItemInstanceId,
    pub question_id: QuestionId,
    pub question_version: ProtocolVersion,
}
#[derive(Deserialize)]
struct AttemptWire {
    id: AttemptId,
    assessment_id: AssessmentId,
    assessment_version: ProtocolVersion,
    student_id: StudentId,
    policy_version: ProtocolVersion,
    state: AttemptState,
    started_at: Timestamp,
    updated_at: Timestamp,
    completed_at: Option<Timestamp>,
    items: Vec<AssessmentItemInstance>,
    responses: Vec<AssessmentResponse>,
    results: Vec<ItemResult>,
}
impl TryFrom<AttemptWire> for AssessmentAttempt {
    type Error = AssessmentError;
    fn try_from(w: AttemptWire) -> Result<Self, Self::Error> {
        if w.updated_at < w.started_at || w.completed_at.is_some_and(|t| t < w.updated_at) {
            return Err(AssessmentError::TimestampRegression);
        }
        if w.state.terminal() != w.completed_at.is_some() {
            return Err(AssessmentError::InvalidContract(
                "terminal state and completed_at disagree",
            ));
        }
        if !sorted_unique(&w.items.iter().map(|x| x.question_id).collect::<Vec<_>>())
            || !sorted_unique(
                &w.responses
                    .iter()
                    .map(|x| x.question_id)
                    .collect::<Vec<_>>(),
            )
            || w.responses
                .iter()
                .map(|x| x.id)
                .collect::<BTreeSet<_>>()
                .len()
                != w.responses.len()
            || w.items.iter().map(|x| x.id).collect::<BTreeSet<_>>().len() != w.items.len()
            || w.responses.len() != w.results.len()
            || w.responses
                .iter()
                .zip(&w.results)
                .any(|(a, r)| a.question_id != r.question_id)
            || w.responses
                .iter()
                .any(|r| r.student_id != w.student_id || r.assessment_id != w.assessment_id)
        {
            return Err(AssessmentError::InvalidContract(
                "attempt collections are inconsistent or unordered",
            ));
        }
        let frozen: BTreeSet<_> = w.items.iter().map(|x| x.question_id).collect();
        if w.responses.iter().any(|r| !frozen.contains(&r.question_id))
            || w.results.iter().any(|r| !frozen.contains(&r.question_id))
        {
            return Err(AssessmentError::InvalidContract(
                "responses and results must belong to frozen items",
            ));
        }
        Ok(Self {
            id: w.id,
            assessment_id: w.assessment_id,
            assessment_version: w.assessment_version,
            student_id: w.student_id,
            policy_version: w.policy_version,
            state: w.state,
            started_at: w.started_at,
            updated_at: w.updated_at,
            completed_at: w.completed_at,
            items: w.items,
            responses: w.responses,
            results: w.results,
        })
    }
}
impl AssessmentAttempt {
    pub fn responses(&self) -> &[AssessmentResponse] {
        &self.responses
    }
    pub fn results(&self) -> &[ItemResult] {
        &self.results
    }
    pub fn items(&self) -> &[AssessmentItemInstance] {
        &self.items
    }
}

#[derive(Clone, Debug)]
pub struct ScoringPolicyV1;
impl ScoringPolicyV1 {
    pub const VERSION: ProtocolVersion = SCORING_POLICY_V1;
    pub fn start(
        &self,
        assessment: &Assessment,
        attempt_id: AttemptId,
        student_id: StudentId,
        item_ids: BTreeMap<QuestionId, AssessmentItemInstanceId>,
        at: Timestamp,
    ) -> Result<AssessmentAttempt, AssessmentError> {
        if assessment.scoring_policy_version != Self::VERSION {
            return Err(AssessmentError::PolicyMismatch {
                expected: Self::VERSION,
                actual: assessment.scoring_policy_version,
            });
        }
        if item_ids.len() != assessment.questions.len()
            || assessment
                .questions
                .iter()
                .any(|q| !item_ids.contains_key(&q.id))
        {
            return Err(AssessmentError::InvalidContract(
                "item identifiers must exactly cover questions",
            ));
        }
        if item_ids.values().collect::<BTreeSet<_>>().len() != item_ids.len() {
            return Err(AssessmentError::InvalidContract(
                "item instance identifiers must be unique",
            ));
        }
        let items = assessment
            .questions
            .iter()
            .map(|q| AssessmentItemInstance {
                id: item_ids[&q.id],
                question_id: q.id,
                question_version: q.version,
            })
            .collect();
        Ok(AssessmentAttempt {
            id: attempt_id,
            assessment_id: assessment.id,
            assessment_version: assessment.version,
            student_id,
            policy_version: Self::VERSION,
            state: AttemptState::Created,
            started_at: at,
            updated_at: at,
            completed_at: None,
            items,
            responses: vec![],
            results: vec![],
        })
    }
    pub fn transition(
        &self,
        assessment: &Assessment,
        attempt: &AssessmentAttempt,
        to: AttemptState,
        at: Timestamp,
    ) -> Result<AssessmentAttempt, AssessmentError> {
        self.validate_frozen(assessment, attempt, false)?;
        if attempt.policy_version != Self::VERSION {
            return Err(AssessmentError::PolicyMismatch {
                expected: Self::VERSION,
                actual: attempt.policy_version,
            });
        }
        if at < attempt.updated_at {
            return Err(AssessmentError::TimestampRegression);
        }
        if attempt.state.terminal() {
            return Err(AssessmentError::InvalidState);
        }
        let allowed = matches!(
            (attempt.state, to),
            (AttemptState::Created, AttemptState::Active)
                | (AttemptState::Active, AttemptState::Paused)
                | (AttemptState::Paused, AttemptState::Active)
                | (AttemptState::Active, AttemptState::Submitted)
                | (AttemptState::Submitted, AttemptState::Completed)
                | (AttemptState::Submitted, AttemptState::Failed)
                | (_, AttemptState::Invalidated)
                | (_, AttemptState::Cancelled)
        );
        if !allowed {
            return Err(AssessmentError::InvalidState);
        }
        if matches!(to, AttemptState::Submitted | AttemptState::Completed)
            && (attempt.responses.len() != attempt.items.len()
                || attempt.results.len() != attempt.items.len())
        {
            return Err(AssessmentError::InvalidState);
        }
        let mut next = attempt.clone();
        next.state = to;
        next.updated_at = at;
        if to.terminal() {
            next.completed_at = Some(at)
        }
        Ok(next)
    }
    pub fn submit(
        &self,
        assessment: &Assessment,
        attempt: &AssessmentAttempt,
        response: AssessmentResponse,
        evidence_ids: BTreeMap<CompetencyId, EvidenceId>,
    ) -> Result<Submission, AssessmentError> {
        self.validate_frozen(assessment, attempt, false)?;
        if response.assessment_id != attempt.assessment_id
            || response.student_id != attempt.student_id
        {
            return Err(AssessmentError::ScopeMismatch);
        }
        if attempt.policy_version != Self::VERSION
            || assessment.scoring_policy_version != Self::VERSION
        {
            return Err(AssessmentError::PolicyMismatch {
                expected: Self::VERSION,
                actual: attempt.policy_version,
            });
        }
        if attempt.state != AttemptState::Active {
            return Err(AssessmentError::InvalidState);
        }
        if let Some(old) = attempt.responses.iter().find(|x| x.id == response.id) {
            if old == &response {
                let i = attempt
                    .responses
                    .iter()
                    .position(|x| x.id == response.id)
                    .expect("found");
                return Ok(Submission {
                    attempt: attempt.clone(),
                    evidence: vec![],
                    replayed: true,
                    result: attempt.results[i].clone(),
                });
            }
            return Err(AssessmentError::ConflictingReplay);
        }
        if response.submitted_at < attempt.updated_at {
            return Err(AssessmentError::TimestampRegression);
        }
        if attempt
            .responses
            .iter()
            .any(|x| x.question_id == response.question_id)
        {
            return Err(AssessmentError::ConflictingReplay);
        }
        let q = assessment
            .questions
            .iter()
            .find(|q| q.id == response.question_id)
            .ok_or(AssessmentError::UnknownQuestion)?;
        if !attempt.items.iter().any(|item| item.question_id == q.id) {
            return Err(AssessmentError::UnknownQuestion);
        }
        let expected: BTreeSet<_> = q.competency_ids.iter().copied().collect();
        if evidence_ids.keys().copied().collect::<BTreeSet<_>>() != expected
            || evidence_ids.values().collect::<BTreeSet<_>>().len() != evidence_ids.len()
        {
            return Err(AssessmentError::InvalidEvidenceMapping);
        }
        let score = evaluate(q, &assessment.rubrics, &response.value)?;
        let outcome = if score.get() == 1.0 {
            EvaluationOutcome::Correct
        } else if score.get() == 0.0 {
            EvaluationOutcome::Incorrect
        } else {
            EvaluationOutcome::Partial
        };
        let rationale_code: SemanticKey = match outcome {
            EvaluationOutcome::Correct => "assessment.correct",
            EvaluationOutcome::Partial => "assessment.partial",
            EvaluationOutcome::Incorrect => "assessment.incorrect",
        }
        .parse()
        .expect("static semantic key");
        let result = ItemResult {
            question_id: q.id,
            score,
            outcome,
            rationale_code,
        };
        let mut next = attempt.clone();
        next.updated_at = response.submitted_at;
        let position = next
            .responses
            .binary_search_by_key(&response.question_id, |r| r.question_id)
            .unwrap_err();
        next.responses.insert(position, response.clone());
        next.results.insert(position, result.clone());
        let evidence = q
            .competency_ids
            .iter()
            .map(|competency_id| LearningEvidence {
                id: evidence_ids[competency_id],
                student_id: attempt.student_id,
                competency_id: *competency_id,
                evidence_type: purpose_type(q.purpose),
                outcome: match competency_score(
                    q,
                    &assessment.rubrics,
                    &response.value,
                    *competency_id,
                )
                .unwrap_or(score)
                .get()
                {
                    1.0 => EvidenceOutcome::Success,
                    0.0 => EvidenceOutcome::Failure,
                    _ => EvidenceOutcome::PartialSuccess,
                },
                difficulty: EvidenceDifficulty::Unknown,
                independence: IndependenceLevel::Unknown,
                confidence: None,
                source: EvidenceSource::Assessment(attempt.id),
                observed_at: next.updated_at,
            })
            .collect();
        Ok(Submission {
            attempt: next,
            evidence,
            replayed: false,
            result,
        })
    }
    pub fn result(
        &self,
        assessment: &Assessment,
        attempt: &AssessmentAttempt,
    ) -> Result<AssessmentOutcome, AssessmentError> {
        self.validate_frozen(assessment, attempt, true)?;
        if attempt.state != AttemptState::Completed {
            return Err(AssessmentError::InvalidState);
        }
        let score = Score::new(
            attempt.results.iter().map(|x| x.score.get()).sum::<f64>() / attempt.items.len() as f64,
        )?;
        Ok(AssessmentOutcome {
            attempt_id: attempt.id,
            student_id: attempt.student_id,
            assessment_id: assessment.id,
            score,
            passed: score.get() >= assessment.passing_score.get(),
            completed_at: attempt.completed_at.expect("validated terminal"),
        })
    }

    fn validate_frozen(
        &self,
        assessment: &Assessment,
        attempt: &AssessmentAttempt,
        require_complete: bool,
    ) -> Result<(), AssessmentError> {
        if attempt.assessment_id != assessment.id
            || attempt.assessment_version != assessment.version
        {
            return Err(AssessmentError::ScopeMismatch);
        }
        if attempt.policy_version != Self::VERSION
            || assessment.scoring_policy_version != Self::VERSION
        {
            return Err(AssessmentError::PolicyMismatch {
                expected: Self::VERSION,
                actual: attempt.policy_version,
            });
        }
        let expected: Vec<_> = assessment
            .questions
            .iter()
            .map(|q| (q.id, q.version))
            .collect();
        let actual: Vec<_> = attempt
            .items
            .iter()
            .map(|i| (i.question_id, i.question_version))
            .collect();
        if actual != expected {
            return Err(AssessmentError::InvalidContract(
                "frozen items do not match assessment",
            ));
        }
        let item_questions: Vec<_> = attempt.items.iter().map(|i| i.question_id).collect();
        if attempt
            .responses
            .iter()
            .any(|r| !item_questions.contains(&r.question_id))
            || attempt
                .results
                .iter()
                .any(|r| !item_questions.contains(&r.question_id))
            || (require_complete
                && (attempt.responses.len() != item_questions.len()
                    || attempt.results.len() != item_questions.len()))
        {
            return Err(AssessmentError::InvalidContract(
                "attempt coverage does not match frozen items",
            ));
        }
        Ok(())
    }
}

fn competency_score(
    q: &Question,
    rubrics: &[Rubric],
    answer: &ResponseValue,
    competency: CompetencyId,
) -> Option<Score> {
    let (Evaluation::Rubric { rubric_id }, ResponseValue::Rubric { levels }) =
        (&q.evaluation, answer)
    else {
        return None;
    };
    let rubric = rubrics.iter().find(|r| r.id == *rubric_id)?;
    let criteria: Vec<_> = rubric
        .criteria
        .iter()
        .filter(|c| c.competency_id == competency)
        .collect();
    let total: f64 = criteria.iter().map(|c| c.weight.get()).sum();
    Score::new(
        criteria
            .iter()
            .map(|c| levels[&c.id].get() * c.weight.get())
            .sum::<f64>()
            / total,
    )
    .ok()
}
fn evaluate(
    q: &Question,
    rubrics: &[Rubric],
    answer: &ResponseValue,
) -> Result<Score, AssessmentError> {
    match answer {
        ResponseValue::Text { value } => {
            text(value).map_err(|_| AssessmentError::IncompatibleResponse)?
        }
        ResponseValue::Ordering { values }
            if values.is_empty()
                || values.iter().collect::<BTreeSet<_>>().len() != values.len() =>
        {
            return Err(AssessmentError::IncompatibleResponse);
        }
        _ => {}
    }
    let value = match (&q.evaluation, answer) {
        (Evaluation::Choice { correct }, ResponseValue::Choice { choice }) => {
            f64::from(correct == choice)
        }
        (Evaluation::Boolean { correct }, ResponseValue::Boolean { value }) => {
            f64::from(correct == value)
        }
        (
            Evaluation::Exact {
                expected,
                case_sensitive,
            },
            ResponseValue::Text { value },
        ) => {
            let (a, b) = (expected.trim(), value.trim());
            f64::from(if *case_sensitive {
                a == b
            } else {
                a.eq_ignore_ascii_case(b)
            })
        }
        (Evaluation::Ordering { expected }, ResponseValue::Ordering { values }) => {
            f64::from(expected == values)
        }
        (Evaluation::Rubric { rubric_id }, ResponseValue::Rubric { levels }) => {
            let r = rubrics
                .iter()
                .find(|r| r.id == *rubric_id)
                .ok_or(AssessmentError::InvalidContract("dangling rubric"))?;
            if levels.keys().copied().collect::<BTreeSet<_>>()
                != r.criteria.iter().map(|c| c.id).collect()
            {
                return Err(AssessmentError::IncompatibleResponse);
            }
            r.criteria
                .iter()
                .map(|c| c.weight.get() * levels[&c.id].get())
                .sum()
        }
        _ => return Err(AssessmentError::IncompatibleResponse),
    };
    Score::new(value)
}
fn purpose_type(p: QuestionPurpose) -> EvidenceType {
    match p {
        QuestionPurpose::Recognize => EvidenceType::Recognition,
        QuestionPurpose::Recall => EvidenceType::Recall,
        QuestionPurpose::Explain => EvidenceType::Explanation,
        QuestionPurpose::Apply => EvidenceType::Application,
        QuestionPurpose::Debug => EvidenceType::Debugging,
        QuestionPurpose::Transfer => EvidenceType::Transfer,
        QuestionPurpose::RetentionCheck => EvidenceType::Retention,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Submission {
    pub attempt: AssessmentAttempt,
    pub result: ItemResult,
    pub evidence: Vec<LearningEvidence>,
    pub replayed: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssessmentOutcome {
    pub attempt_id: AttemptId,
    pub student_id: StudentId,
    pub assessment_id: AssessmentId,
    pub score: Score,
    pub passed: bool,
    pub completed_at: Timestamp,
}
