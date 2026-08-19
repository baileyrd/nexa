//! Synchronous Phase 3 composition of governed learning-core policies.
#![forbid(unsafe_code)]

use nexa_assessment::{
    Assessment, AssessmentAttempt, AssessmentError, AssessmentResponse, AttemptState,
    EvaluationOutcome, ScoringPolicyV1,
};
use nexa_domain::{
    AssessmentItemInstanceId, CompetencyId, EvidenceId, LessonId, LessonTransitionId, MasteryScore,
    ProtocolVersion, QuestionId, SemanticKey, StudentId, Timestamp,
};
use nexa_events::{
    AssessmentResponseEvaluated, CompetencyEvidenceAdded, CompetencyUpdated, PedagogyDecisionMade,
};
use nexa_lessons::{Curriculum, LessonLifecycle, LessonPolicyV1, LessonProgress, TransitionError};
use nexa_pedagogy::{
    InstructionalOption, PedagogyDecision, PedagogyError, PedagogyInput, PedagogyPolicy,
    PedagogyPolicyV1, RecentOutcome,
};
use nexa_student::{
    replay, AppendOutcome, BoundedWeightedV1, LearningEvidence, MasteryState, StudentModelError,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const COMPOSITION_VERSION: ProtocolVersion = ProtocolVersion::new(1, 0);

/// The complete atomic state visible to the composition boundary. Durable adapters may
/// represent it differently, but must preserve the load/commit semantics of this port.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LearningState {
    pub lesson_progress: Vec<LessonProgress>,
    pub assessment_attempts: Vec<AssessmentAttempt>,
    pub evidence: Vec<LearningEvidence>,
    pub mastery: Vec<MasteryState>,
    pub receipts: BTreeMap<LessonTransitionId, OperationReceipt>,
}

/// A committed request fingerprint and result. Keeping the result makes identical retry a no-op.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OperationReceipt {
    pub request: LearningOperation,
    pub result: LearningResult,
}

/// One synchronous atomic unit. Implementations must not expose partially staged state and must
/// reject a stale expected snapshot rather than merging it.
pub trait LearningUnitOfWork {
    type Error: std::error::Error + Send + Sync + 'static;
    fn load(&self) -> Result<LearningState, Self::Error>;
    fn commit(
        &mut self,
        expected: &LearningState,
        replacement: LearningState,
    ) -> Result<(), Self::Error>;
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LearningOperation {
    pub version: ProtocolVersion,
    pub operation_id: LessonTransitionId,
    pub student_id: StudentId,
    pub lesson_id: LessonId,
    pub attempt_id: nexa_domain::AttemptId,
    pub competency_id: CompetencyId,
    pub response: AssessmentResponse,
    pub evidence_ids: BTreeMap<CompetencyId, EvidenceId>,
    pub item_ids: BTreeMap<QuestionId, AssessmentItemInstanceId>,
    pub completed_lessons: BTreeSet<LessonId>,
    pub available_options: BTreeSet<InstructionalOption>,
    pub at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LearningEventFacts {
    pub response_evaluated: Option<AssessmentResponseEvaluated>,
    pub evidence_added: Vec<CompetencyEvidenceAdded>,
    pub competency_updated: Vec<CompetencyUpdated>,
    pub pedagogy_decision: PedagogyDecisionMade,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LearningResult {
    pub lesson_progress: LessonProgress,
    pub assessment_attempt: AssessmentAttempt,
    pub mastery: MasteryState,
    pub pedagogy_decision: PedagogyDecision,
    pub evidence_outcomes: Vec<AppendOutcomeContract>,
    pub event_facts: LearningEventFacts,
    pub replayed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppendOutcomeContract {
    Appended,
    Duplicate,
}

#[derive(Debug, Error)]
pub enum CompositionError {
    #[error("composition request is invalid: {0}")]
    Invalid(&'static str),
    #[error("operation identifier was replayed with conflicting content")]
    ConflictingReplay,
    #[error("assessment policy failed: {0}")]
    Assessment(#[from] AssessmentError),
    #[error("student policy failed: {0}")]
    Student(#[from] StudentModelError),
    #[error("pedagogy policy failed: {0}")]
    Pedagogy(#[from] PedagogyError),
    #[error("lesson policy failed: {0}")]
    Lesson(#[from] TransitionError),
    #[error("persistence operation failed: {0}")]
    Persistence(String),
}

pub struct LearningCore;
impl LearningCore {
    pub fn apply<U: LearningUnitOfWork>(
        uow: &mut U,
        curriculum: &Curriculum,
        assessment: &Assessment,
        request: LearningOperation,
    ) -> Result<LearningResult, CompositionError> {
        validate_request(assessment, &request)?;
        let original = uow
            .load()
            .map_err(|e| CompositionError::Persistence(e.to_string()))?;
        if let Some(receipt) = original.receipts.get(&request.operation_id) {
            return if receipt.request == request {
                let mut result = receipt.result.clone();
                result.replayed = true;
                Ok(result)
            } else {
                Err(CompositionError::ConflictingReplay)
            };
        }
        let mut staged = original.clone();
        let lesson_index = scoped_lesson(&staged, request.student_id, request.lesson_id)?;
        let mut progress = lesson_index.map_or_else(
            || LessonProgress::not_started(request.student_id, request.lesson_id),
            |i| staged.lesson_progress[i].clone(),
        );
        if progress.lifecycle() == LessonLifecycle::NotStarted {
            progress = LessonPolicyV1::start(
                curriculum,
                &progress,
                &request.completed_lessons,
                request.at,
            )?;
        }

        let scoring = ScoringPolicyV1;
        let attempt_index = scoped_attempt(&staged, &request)?;
        let mut attempt = match attempt_index {
            Some(i) => staged.assessment_attempts[i].clone(),
            None => scoring.start(
                assessment,
                request.attempt_id,
                request.student_id,
                request.item_ids.clone(),
                request.at,
            )?,
        };
        if attempt.state == AttemptState::Created {
            attempt = scoring.transition(assessment, &attempt, AttemptState::Active, request.at)?;
        }
        let before_response_count = attempt.responses().len();
        let submission = scoring.submit(
            assessment,
            &attempt,
            request.response.clone(),
            request.evidence_ids.clone(),
        )?;
        attempt = submission.attempt;

        let previous_mastery = replay(&staged.evidence, &BoundedWeightedV1)?
            .into_iter()
            .find(|m| {
                m.student_id() == request.student_id && m.competency_id() == request.competency_id
            })
            .map_or_else(
                || MasteryScore::new(0.0).expect("constant"),
                |m| m.mastery(),
            );

        let mut outcomes = Vec::new();
        let mut added = Vec::new();
        for evidence in submission.evidence {
            let outcome = append_evidence(&mut staged.evidence, evidence.clone())?;
            outcomes.push(match outcome {
                AppendOutcome::Appended => AppendOutcomeContract::Appended,
                AppendOutcome::Duplicate => AppendOutcomeContract::Duplicate,
            });
            if outcome == AppendOutcome::Appended {
                added.push(evidence);
            }
        }
        if attempt.responses().len() == attempt.items().len()
            && attempt.state == AttemptState::Active
        {
            attempt =
                scoring.transition(assessment, &attempt, AttemptState::Submitted, request.at)?;
            attempt =
                scoring.transition(assessment, &attempt, AttemptState::Completed, request.at)?;
        }

        let projections = replay(&staged.evidence, &BoundedWeightedV1)?;
        let mastery = projections
            .into_iter()
            .find(|m| {
                m.student_id() == request.student_id && m.competency_id() == request.competency_id
            })
            .ok_or(CompositionError::Invalid(
                "no evidence projection for requested competency",
            ))?;
        let recent = match submission.result.outcome {
            EvaluationOutcome::Correct => RecentOutcome::Success,
            EvaluationOutcome::Partial => RecentOutcome::PartialSuccess,
            EvaluationOutcome::Incorrect => RecentOutcome::Failure,
        };
        let failures = if recent == RecentOutcome::Failure {
            1
        } else {
            0
        };
        let input = PedagogyInput::new(
            COMPOSITION_VERSION,
            mastery.clone(),
            Some(recent),
            attempt.responses().len() as u32,
            failures,
            request.available_options.iter().copied(),
        )?;
        let decision = PedagogyPolicyV1.decide(&input)?;
        progress = LessonPolicyV1::route(
            curriculum,
            &progress,
            &decision,
            &request.completed_lessons,
            request.at,
        )?;

        put(&mut staged.lesson_progress, lesson_index, progress.clone());
        put(
            &mut staged.assessment_attempts,
            attempt_index,
            attempt.clone(),
        );
        staged.mastery.retain(|m| {
            !(m.student_id() == request.student_id && m.competency_id() == request.competency_id)
        });
        staged.mastery.push(mastery.clone());
        staged
            .mastery
            .sort_by_key(|m| (m.student_id(), m.competency_id()));

        let option: SemanticKey = format!("{:?}", decision.selected_option())
            .to_ascii_lowercase()
            .parse()
            .expect("enum semantic key");
        let rationale = decision
            .rationale_codes()
            .iter()
            .map(|r| {
                format!("{:?}", r)
                    .to_ascii_lowercase()
                    .parse()
                    .expect("enum semantic key")
            })
            .collect();
        let pedagogy_fact = PedagogyDecisionMade::new(
            request.student_id,
            request.competency_id,
            option,
            rationale,
            COMPOSITION_VERSION,
        )
        .map_err(|_| CompositionError::Invalid("invalid pedagogy event fact"))?;
        let item = attempt
            .items()
            .iter()
            .find(|i| i.question_id == request.response.question_id)
            .ok_or(CompositionError::Invalid("response question is not frozen"))?;
        let response_fact = (attempt.responses().len() > before_response_count).then(|| {
            AssessmentResponseEvaluated::new(
                attempt.id,
                assessment.id,
                item.id,
                request.response.id,
                MasteryScore::new(submission.result.score.get()).expect("score domain"),
                ScoringPolicyV1::VERSION,
            )
        });
        let evidence_facts = added.iter().map(evidence_fact).collect();
        let updated_facts = added
            .iter()
            .map(|e| CompetencyUpdated {
                evidence_id: e.id,
                student_id: e.student_id,
                competency_id: e.competency_id,
                previous_mastery,
                new_mastery: mastery.mastery(),
                policy_version: mastery.policy_version(),
            })
            .collect();
        let result = LearningResult {
            lesson_progress: progress,
            assessment_attempt: attempt,
            mastery,
            pedagogy_decision: decision,
            evidence_outcomes: outcomes,
            event_facts: LearningEventFacts {
                response_evaluated: response_fact,
                evidence_added: evidence_facts,
                competency_updated: updated_facts,
                pedagogy_decision: pedagogy_fact,
            },
            replayed: false,
        };
        staged.receipts.insert(
            request.operation_id,
            OperationReceipt {
                request,
                result: result.clone(),
            },
        );
        uow.commit(&original, staged)
            .map_err(|e| CompositionError::Persistence(e.to_string()))?;
        Ok(result)
    }
}

fn validate_request(a: &Assessment, r: &LearningOperation) -> Result<(), CompositionError> {
    if r.version != COMPOSITION_VERSION {
        return Err(CompositionError::Invalid(
            "composition policy version mismatch",
        ));
    }
    if r.response.student_id != r.student_id || r.response.assessment_id != a.id {
        return Err(CompositionError::Invalid("request scope mismatch"));
    }
    if r.evidence_ids.len() != 1 || !r.evidence_ids.contains_key(&r.competency_id) {
        return Err(CompositionError::Invalid(
            "operation must target exactly one competency",
        ));
    }
    if r.response.submitted_at != r.at {
        return Err(CompositionError::Invalid(
            "operation and response timestamps differ",
        ));
    }
    Ok(())
}
fn scoped_lesson(
    s: &LearningState,
    student: StudentId,
    lesson: LessonId,
) -> Result<Option<usize>, CompositionError> {
    Ok(s.lesson_progress
        .iter()
        .position(|p| p.student_id() == student && p.lesson_id() == lesson))
}
fn scoped_attempt(
    s: &LearningState,
    r: &LearningOperation,
) -> Result<Option<usize>, CompositionError> {
    if let Some((i, a)) = s
        .assessment_attempts
        .iter()
        .enumerate()
        .find(|(_, a)| a.id == r.attempt_id)
    {
        if a.student_id != r.student_id || a.assessment_id != r.response.assessment_id {
            return Err(CompositionError::Invalid("cross-scope assessment attempt"));
        }
        Ok(Some(i))
    } else {
        Ok(None)
    }
}
fn put<T>(values: &mut Vec<T>, index: Option<usize>, value: T) {
    if let Some(i) = index {
        values[i] = value
    } else {
        values.push(value)
    }
}
fn append_evidence(
    values: &mut Vec<LearningEvidence>,
    e: LearningEvidence,
) -> Result<AppendOutcome, StudentModelError> {
    if let Some(old) = values.iter().find(|x| x.id == e.id) {
        return if old == &e {
            Ok(AppendOutcome::Duplicate)
        } else {
            Err(StudentModelError::ConflictingDuplicate(e.id))
        };
    }
    if values
        .iter()
        .filter(|x| x.student_id == e.student_id && x.competency_id == e.competency_id)
        .any(|x| x.observed_at > e.observed_at)
    {
        return Err(StudentModelError::Repository {
            message: "evidence timestamp regressed within student competency scope".into(),
        });
    }
    values.push(e);
    Ok(AppendOutcome::Appended)
}
fn evidence_fact(e: &LearningEvidence) -> CompetencyEvidenceAdded {
    CompetencyEvidenceAdded {
        evidence_id: e.id,
        student_id: e.student_id,
        competency_id: e.competency_id,
        evidence_type: format!("{:?}", e.evidence_type)
            .to_ascii_lowercase()
            .parse()
            .expect("enum key"),
        outcome: format!("{:?}", e.outcome)
            .to_ascii_lowercase()
            .parse()
            .expect("enum key"),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommitStage {
    Lesson,
    Assessment,
    Evidence,
    Mastery,
    Receipt,
    Finalize,
}
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum InMemoryError {
    #[error("simulated failure at {0:?}")]
    Injected(CommitStage),
    #[error("stale unit of work")]
    Stale,
}
#[derive(Clone, Debug, Default)]
pub struct InMemoryUnitOfWork {
    state: LearningState,
    fail_at: Option<CommitStage>,
}
impl InMemoryUnitOfWork {
    pub fn state(&self) -> &LearningState {
        &self.state
    }
    pub fn fail_next_at(&mut self, stage: CommitStage) {
        self.fail_at = Some(stage);
    }
}
impl LearningUnitOfWork for InMemoryUnitOfWork {
    type Error = InMemoryError;
    fn load(&self) -> Result<LearningState, Self::Error> {
        Ok(self.state.clone())
    }
    fn commit(
        &mut self,
        expected: &LearningState,
        replacement: LearningState,
    ) -> Result<(), Self::Error> {
        if &self.state != expected {
            return Err(InMemoryError::Stale);
        }
        // Stages deliberately validate the all-or-nothing adapter contract; publication occurs only at finalize.
        for stage in [
            CommitStage::Lesson,
            CommitStage::Assessment,
            CommitStage::Evidence,
            CommitStage::Mastery,
            CommitStage::Receipt,
            CommitStage::Finalize,
        ] {
            if self.fail_at == Some(stage) {
                self.fail_at = None;
                return Err(InMemoryError::Injected(stage));
            }
        }
        self.state = replacement;
        Ok(())
    }
}
