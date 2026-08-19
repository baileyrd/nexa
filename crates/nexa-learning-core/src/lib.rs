//! Synchronous Phase 3 composition of governed learning-core policies.
#![forbid(unsafe_code)]

use nexa_assessment::{
    Assessment, AssessmentAttempt, AssessmentError, AssessmentResponse, AttemptState,
    ScoringPolicyV1,
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
    PedagogyPolicyV1, RationaleCode, RecentOutcome,
};
use nexa_student::{
    replay, AppendOutcome, BoundedWeightedV1, EvidenceOutcome, EvidenceType, LearningEvidence,
    MasteryState, StudentModelError,
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
    /// Exact authored inputs are retained as the v1 semantic fingerprint. This deliberately
    /// favors an auditable comparison over a lossy or implementation-dependent hash.
    pub assessment: Assessment,
    pub curriculum: Curriculum,
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

/// Reject adapter-provided snapshots that are ambiguous or non-canonical before any policy runs.
pub fn validate_loaded_state(state: &LearningState) -> Result<(), CompositionError> {
    ensure_sorted_unique(
        state
            .lesson_progress
            .iter()
            .map(|p| (p.student_id(), p.lesson_id())),
        "lesson progress scopes are duplicated or not canonically ordered",
    )?;
    let mut evidence_ids = BTreeSet::new();
    if state.evidence.iter().any(|e| !evidence_ids.insert(e.id)) {
        return Err(CompositionError::Invalid("duplicate evidence identifier"));
    }
    ensure_sorted_unique(
        state.assessment_attempts.iter().map(|a| a.id),
        "assessment attempt identifiers are duplicated or not canonically ordered",
    )?;
    ensure_sorted_unique(
        state
            .evidence
            .iter()
            .map(|e| (e.student_id, e.competency_id, e.observed_at, e.id)),
        "evidence is duplicated or not canonically ordered",
    )?;
    ensure_sorted_unique(
        state
            .mastery
            .iter()
            .map(|m| (m.student_id(), m.competency_id())),
        "mastery scopes are duplicated or not canonically ordered",
    )?;
    for (key, receipt) in &state.receipts {
        let request = &receipt.request;
        let result = &receipt.result;
        if *key != request.operation_id
            || receipt.assessment.id != request.response.assessment_id
            || result.lesson_progress.student_id() != request.student_id
            || result.lesson_progress.lesson_id() != request.lesson_id
            || result.assessment_attempt.id != request.attempt_id
            || result.assessment_attempt.student_id != request.student_id
            || result.assessment_attempt.assessment_id != request.response.assessment_id
            || result.mastery.student_id() != request.student_id
            || result.mastery.competency_id() != request.competency_id
        {
            return Err(CompositionError::Invalid(
                "inconsistent operation receipt scope",
            ));
        }
    }
    Ok(())
}

fn ensure_sorted_unique<T: Ord>(
    values: impl Iterator<Item = T>,
    message: &'static str,
) -> Result<(), CompositionError> {
    let mut previous = None;
    for value in values {
        if previous.as_ref().is_some_and(|old| old >= &value) {
            return Err(CompositionError::Invalid(message));
        }
        previous = Some(value);
    }
    Ok(())
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
        validate_loaded_state(&original)?;
        if let Some(receipt) = original.receipts.get(&request.operation_id) {
            return if same_semantic_request(receipt, &request, assessment, curriculum, false) {
                let mut result = receipt.result.clone();
                result.replayed = true;
                Ok(result)
            } else {
                Err(CompositionError::ConflictingReplay)
            };
        }
        if let Some(receipt) = response_receipt(&original, &request, assessment, curriculum)? {
            let mut result = receipt.result.clone();
            result.replayed = true;
            return Ok(result);
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

        let previous_projections = replay(&staged.evidence, &BoundedWeightedV1)?;

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
            .iter()
            .find(|m| {
                m.student_id() == request.student_id && m.competency_id() == request.competency_id
            })
            .cloned()
            .ok_or(CompositionError::Invalid(
                "no evidence projection for requested competency",
            ))?;
        let (recent, attempt_count, failures) =
            pedagogy_history(&staged.evidence, request.student_id, request.competency_id)?;
        let input = PedagogyInput::new(
            COMPOSITION_VERSION,
            mastery.clone(),
            recent,
            attempt_count,
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
        staged
            .lesson_progress
            .sort_by_key(|p| (p.student_id(), p.lesson_id()));
        staged.assessment_attempts.sort_by_key(|a| a.id);
        staged
            .evidence
            .sort_by_key(|e| (e.student_id, e.competency_id, e.observed_at, e.id));
        let affected: BTreeSet<_> = request.evidence_ids.keys().copied().collect();
        staged.mastery.retain(|m| {
            m.student_id() != request.student_id || !affected.contains(&m.competency_id())
        });
        staged.mastery.extend(
            projections
                .iter()
                .filter(|m| {
                    m.student_id() == request.student_id && affected.contains(&m.competency_id())
                })
                .cloned(),
        );
        staged
            .mastery
            .sort_by_key(|m| (m.student_id(), m.competency_id()));

        let option = option_key(decision.selected_option());
        let rationale = decision
            .rationale_codes()
            .iter()
            .map(|r| rationale_key(*r))
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
            .map(|e| {
                let old = previous_projections.iter().find(|m| {
                    m.student_id() == e.student_id && m.competency_id() == e.competency_id
                });
                let new = projections
                    .iter()
                    .find(|m| {
                        m.student_id() == e.student_id && m.competency_id() == e.competency_id
                    })
                    .expect("appended evidence has projection");
                CompetencyUpdated {
                    evidence_id: e.id,
                    student_id: e.student_id,
                    competency_id: e.competency_id,
                    previous_mastery: old.map_or_else(
                        || MasteryScore::new(0.0).expect("constant"),
                        |m| m.mastery(),
                    ),
                    new_mastery: new.mastery(),
                    policy_version: new.policy_version(),
                }
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
                assessment: assessment.clone(),
                curriculum: curriculum.clone(),
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
    let question = a
        .questions()
        .iter()
        .find(|q| q.id == r.response.question_id)
        .ok_or(CompositionError::Invalid(
            "response question is not in assessment",
        ))?;
    let expected: BTreeSet<_> = question.competency_ids.iter().copied().collect();
    if r.evidence_ids.keys().copied().collect::<BTreeSet<_>>() != expected
        || !expected.contains(&r.competency_id)
    {
        return Err(CompositionError::Invalid(
            "evidence mapping must cover the question and include the pedagogy competency",
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

fn response_receipt<'a>(
    state: &'a LearningState,
    request: &LearningOperation,
    assessment: &Assessment,
    curriculum: &Curriculum,
) -> Result<Option<&'a OperationReceipt>, CompositionError> {
    let mut matching = None;
    for receipt in state
        .receipts
        .values()
        .filter(|receipt| receipt.request.response.id == request.response.id)
    {
        if !same_semantic_request(receipt, request, assessment, curriculum, true) {
            return Err(CompositionError::ConflictingReplay);
        }
        matching = Some(receipt);
    }
    // Attempts are also checked so a corrupt/incomplete adapter cannot bypass response identity.
    for response in state
        .assessment_attempts
        .iter()
        .flat_map(|attempt| attempt.responses())
        .filter(|response| response.id == request.response.id)
    {
        if response != &request.response {
            return Err(CompositionError::ConflictingReplay);
        }
        if matching.is_none() {
            return Err(CompositionError::Invalid(
                "stored response has no composition receipt",
            ));
        }
    }
    Ok(matching)
}

fn same_semantic_request(
    receipt: &OperationReceipt,
    request: &LearningOperation,
    assessment: &Assessment,
    curriculum: &Curriculum,
    ignore_operation_id: bool,
) -> bool {
    let mut expected = receipt.request.clone();
    if ignore_operation_id {
        expected.operation_id = request.operation_id;
    }
    expected == *request && receipt.assessment == *assessment && receipt.curriculum == *curriculum
}

/// V1 history is the timestamp/evidence-id ordered evidence stream in one student/competency
/// projection. Every evidence item is one attempt; failures are the trailing failure run only.
fn pedagogy_history(
    evidence: &[LearningEvidence],
    student_id: StudentId,
    competency_id: CompetencyId,
) -> Result<(Option<RecentOutcome>, u32, u32), CompositionError> {
    let mut scoped: Vec<_> = evidence
        .iter()
        .filter(|e| e.student_id == student_id && e.competency_id == competency_id)
        .collect();
    scoped.sort_by_key(|e| (e.observed_at, e.id));
    let attempt_count = u32::try_from(scoped.len())
        .map_err(|_| CompositionError::Invalid("pedagogy history exceeds v1 count"))?;
    let recent = scoped.last().map(|e| recent_outcome(e.outcome));
    let consecutive_failures = scoped
        .iter()
        .rev()
        .take_while(|e| e.outcome == EvidenceOutcome::Failure)
        .count()
        .try_into()
        .map_err(|_| CompositionError::Invalid("pedagogy failure history exceeds v1 count"))?;
    Ok((recent, attempt_count, consecutive_failures))
}

fn recent_outcome(outcome: EvidenceOutcome) -> RecentOutcome {
    match outcome {
        EvidenceOutcome::Success => RecentOutcome::Success,
        EvidenceOutcome::PartialSuccess | EvidenceOutcome::Ambiguous => {
            RecentOutcome::PartialSuccess
        }
        EvidenceOutcome::Failure => RecentOutcome::Failure,
    }
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
        evidence_type: evidence_type_key(e.evidence_type),
        outcome: evidence_outcome_key(e.outcome),
    }
}

fn key(value: &'static str) -> SemanticKey {
    value.parse().expect("static semantic key")
}

fn option_key(value: InstructionalOption) -> SemanticKey {
    key(match value {
        InstructionalOption::Introduce => "introduce",
        InstructionalOption::Explain => "explain",
        InstructionalOption::Demonstrate => "demonstrate",
        InstructionalOption::Practice => "practice",
        InstructionalOption::Hint => "hint",
        InstructionalOption::Clarify => "clarify",
        InstructionalOption::Reinforce => "reinforce",
        InstructionalOption::Review => "review",
        InstructionalOption::Challenge => "challenge",
        InstructionalOption::Assess => "assess",
        InstructionalOption::Retry => "retry",
        InstructionalOption::Advance => "advance",
    })
}

fn rationale_key(value: RationaleCode) -> SemanticKey {
    key(match value {
        RationaleCode::NoEvidence => "no_evidence",
        RationaleCode::InsufficientEvidence => "insufficient_evidence",
        RationaleCode::LowModelConfidence => "low_model_confidence",
        RationaleCode::RecentSuccess => "recent_success",
        RationaleCode::RecentPartialSuccess => "recent_partial_success",
        RationaleCode::RecentFailure => "recent_failure",
        RationaleCode::RepeatedFailure => "repeated_failure",
        RationaleCode::RetryLimitReached => "retry_limit_reached",
        RationaleCode::MasteryThresholdMet => "mastery_threshold_met",
        RationaleCode::CompetencyMastered => "competency_mastered",
        RationaleCode::PreferredOptionUnavailable => "preferred_option_unavailable",
        RationaleCode::ProjectionStatusControls => "projection_status_controls",
        RationaleCode::NoRecentOutcome => "no_recent_outcome",
    })
}

fn evidence_type_key(value: EvidenceType) -> SemanticKey {
    key(match value {
        EvidenceType::Recognition => "recognition",
        EvidenceType::Recall => "recall",
        EvidenceType::Explanation => "explanation",
        EvidenceType::Application => "application",
        EvidenceType::Demonstration => "demonstration",
        EvidenceType::Debugging => "debugging",
        EvidenceType::Transfer => "transfer",
        EvidenceType::Retention => "retention",
        EvidenceType::LabPerformance => "lab_performance",
        EvidenceType::Assessment => "assessment",
        EvidenceType::InstructorObservation => "instructor_observation",
    })
}

fn evidence_outcome_key(value: EvidenceOutcome) -> SemanticKey {
    key(match value {
        EvidenceOutcome::Success => "success",
        EvidenceOutcome::PartialSuccess => "partial_success",
        EvidenceOutcome::Failure => "failure",
        EvidenceOutcome::Ambiguous => "ambiguous",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexa_student::{EvidenceDifficulty, EvidenceSource, IndependenceLevel};
    use std::str::FromStr;

    fn id<T: FromStr>(n: u128) -> T
    where
        T::Err: std::fmt::Debug,
    {
        T::from_str(&format!("00000000-0000-0000-0000-{n:012x}")).unwrap()
    }

    fn evidence(n: u128, competency: u128, outcome: EvidenceOutcome, at: &str) -> LearningEvidence {
        LearningEvidence {
            id: id(n),
            student_id: id(1),
            competency_id: id(competency),
            evidence_type: EvidenceType::Assessment,
            outcome,
            difficulty: EvidenceDifficulty::Unknown,
            independence: IndependenceLevel::Unknown,
            confidence: None,
            source: EvidenceSource::Assessment(id(n + 100)),
            observed_at: at.parse().unwrap(),
        }
    }

    #[test]
    fn v1_history_is_scoped_and_counts_only_trailing_failures() {
        let history = vec![
            evidence(1, 8, EvidenceOutcome::Failure, "2026-08-19T10:00:00Z"),
            evidence(2, 9, EvidenceOutcome::Failure, "2026-08-19T10:30:00Z"),
            evidence(3, 8, EvidenceOutcome::Success, "2026-08-19T11:00:00Z"),
            evidence(4, 8, EvidenceOutcome::Failure, "2026-08-19T12:00:00Z"),
        ];
        assert_eq!(
            pedagogy_history(&history, id(1), id(8)).unwrap(),
            (Some(RecentOutcome::Failure), 3, 1)
        );
        assert_eq!(
            pedagogy_history(&history, id(1), id(9)).unwrap(),
            (Some(RecentOutcome::Failure), 1, 1)
        );
    }

    #[test]
    fn event_fact_keys_match_governed_golden_vocabulary() {
        assert_eq!(
            rationale_key(RationaleCode::PreferredOptionUnavailable).as_str(),
            "preferred_option_unavailable"
        );
        assert_eq!(
            evidence_type_key(EvidenceType::LabPerformance).as_str(),
            "lab_performance"
        );
        assert_eq!(
            evidence_outcome_key(EvidenceOutcome::PartialSuccess).as_str(),
            "partial_success"
        );
        assert_eq!(option_key(InstructionalOption::Review).as_str(), "review");
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
