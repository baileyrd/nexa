use nexa_assessment::*;
use nexa_domain::*;
use nexa_learning_core::*;
use nexa_lessons::Curriculum;
use nexa_pedagogy::InstructionalOption;
use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

fn id<T: FromStr>(n: u128) -> T
where
    T::Err: std::fmt::Debug,
{
    T::from_str(&format!("00000000-0000-0000-0000-{n:012x}")).unwrap()
}
fn at() -> Timestamp {
    "2026-08-19T12:00:00Z".parse().unwrap()
}
fn curriculum() -> Curriculum {
    serde_json::from_str(include_str!("fixtures/curriculum.json")).unwrap()
}
fn assessment() -> Assessment {
    Assessment::new(
        id(20),
        ProtocolVersion::new(1, 0),
        "Checkpoint",
        AssessmentMode::Formative,
        SCORING_POLICY_V1,
        Score::new(0.6).unwrap(),
        vec![Question {
            id: id(21),
            version: ProtocolVersion::new(1, 0),
            prompt: "Is this governed?".into(),
            purpose: QuestionPurpose::Recall,
            competency_ids: vec![id(8)],
            evaluation: Evaluation::Boolean { correct: true },
        }],
        vec![],
    )
    .unwrap()
}
fn multi_competency_assessment() -> Assessment {
    let rubric_id = id(40);
    Assessment::new(
        id(20),
        ProtocolVersion::new(1, 0),
        "Multi-competency checkpoint",
        AssessmentMode::Formative,
        SCORING_POLICY_V1,
        Score::new(0.6).unwrap(),
        vec![Question {
            id: id(21),
            version: ProtocolVersion::new(1, 0),
            prompt: "Apply both competencies".into(),
            purpose: QuestionPurpose::Apply,
            competency_ids: vec![id(8), id(10)],
            evaluation: Evaluation::Rubric { rubric_id },
        }],
        vec![Rubric::new(
            rubric_id,
            vec![
                RubricCriterion {
                    id: id(41),
                    description: "First competency".into(),
                    weight: Score::new(0.5).unwrap(),
                    competency_id: id(8),
                },
                RubricCriterion {
                    id: id(42),
                    description: "Second competency".into(),
                    weight: Score::new(0.5).unwrap(),
                    competency_id: id(10),
                },
            ],
        )
        .unwrap()],
    )
    .unwrap()
}
fn request() -> LearningOperation {
    LearningOperation {
        version: COMPOSITION_VERSION,
        operation_id: id(30),
        student_id: id(9),
        lesson_id: id(4),
        attempt_id: id(31),
        competency_id: id(8),
        response: AssessmentResponse {
            id: id(32),
            student_id: id(9),
            assessment_id: id(20),
            question_id: id(21),
            value: ResponseValue::Boolean { value: true },
            submitted_at: at(),
        },
        evidence_ids: BTreeMap::from([(id(8), id(33))]),
        item_ids: BTreeMap::from([(id(21), id(34))]),
        completed_lessons: BTreeSet::new(),
        available_options: BTreeSet::from([InstructionalOption::Practice]),
        at: at(),
    }
}

#[test]
fn phase_3_headless_conformance_and_round_trips() {
    let mut uow = InMemoryUnitOfWork::default();
    let result = LearningCore::apply(&mut uow, &curriculum(), &assessment(), request()).unwrap();
    assert_eq!(result.assessment_attempt.state, AttemptState::Completed);
    assert_eq!(
        result.pedagogy_decision.selected_option(),
        InstructionalOption::Practice
    );
    assert_eq!(result.lesson_progress.current_step_id(), Some(id(6)));
    assert_eq!(result.mastery.evidence_count(), 1);
    assert_eq!(uow.state().evidence.len(), 1);
    assert_eq!(uow.state().receipts.len(), 1);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_eq!(
        serde_json::from_str::<LearningResult>(&json).unwrap(),
        result
    );
    let state_json = serde_json::to_string(uow.state()).unwrap();
    assert_eq!(
        serde_json::from_str::<LearningState>(&state_json).unwrap(),
        *uow.state()
    );
    assert!(!json.contains("Is this governed?") && !json.contains("\"value\":true"));
}

#[test]
fn identical_operation_is_idempotent_and_conflicting_reuse_is_atomic() {
    let mut uow = InMemoryUnitOfWork::default();
    LearningCore::apply(&mut uow, &curriculum(), &assessment(), request()).unwrap();
    let committed = uow.state().clone();
    let replay = LearningCore::apply(&mut uow, &curriculum(), &assessment(), request()).unwrap();
    assert!(replay.replayed);
    assert_eq!(uow.state(), &committed);
    let mut conflict = request();
    conflict.response.value = ResponseValue::Boolean { value: false };
    assert!(matches!(
        LearningCore::apply(&mut uow, &curriculum(), &assessment(), conflict),
        Err(CompositionError::ConflictingReplay)
    ));
    assert_eq!(uow.state(), &committed);
}

#[test]
fn response_identity_is_idempotent_across_operations_and_conflicts_atomically() {
    let mut uow = InMemoryUnitOfWork::default();
    let original = LearningCore::apply(&mut uow, &curriculum(), &assessment(), request()).unwrap();
    let committed = uow.state().clone();
    let mut retry = request();
    retry.operation_id = id(35);
    let replay = LearningCore::apply(&mut uow, &curriculum(), &assessment(), retry).unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.mastery, original.mastery);
    assert_eq!(uow.state(), &committed);

    let mut conflict = request();
    conflict.operation_id = id(36);
    conflict.response.value = ResponseValue::Boolean { value: false };
    assert!(matches!(
        LearningCore::apply(&mut uow, &curriculum(), &assessment(), conflict),
        Err(CompositionError::ConflictingReplay)
    ));
    assert_eq!(uow.state(), &committed);
}

#[test]
fn response_replay_is_bound_to_the_complete_authored_context() {
    let mut uow = InMemoryUnitOfWork::default();
    LearningCore::apply(&mut uow, &curriculum(), &assessment(), request()).unwrap();
    let committed = uow.state().clone();

    let mut changed_lesson = request();
    changed_lesson.operation_id = id(50);
    changed_lesson.lesson_id = id(5);
    let mut changed_options = request();
    changed_options.operation_id = id(51);
    changed_options.available_options = BTreeSet::from([InstructionalOption::Review]);
    for changed in [changed_lesson, changed_options] {
        assert!(matches!(
            LearningCore::apply(&mut uow, &curriculum(), &assessment(), changed),
            Err(CompositionError::ConflictingReplay)
        ));
        assert_eq!(uow.state(), &committed);
    }

    let mut changed_assessment = assessment();
    changed_assessment.version = ProtocolVersion::new(1, 1);
    let mut retry = request();
    retry.operation_id = id(52);
    assert!(matches!(
        LearningCore::apply(&mut uow, &curriculum(), &changed_assessment, retry),
        Err(CompositionError::ConflictingReplay)
    ));
    assert_eq!(uow.state(), &committed);

    let changed_curriculum: Curriculum =
        serde_json::from_str(&include_str!("fixtures/curriculum.json").replace(
            "00000000-0000-0000-0000-000000000001",
            "00000000-0000-0000-0000-000000000099",
        ))
        .unwrap();
    let mut retry = request();
    retry.operation_id = id(53);
    assert!(matches!(
        LearningCore::apply(&mut uow, &changed_curriculum, &assessment(), retry),
        Err(CompositionError::ConflictingReplay)
    ));
    assert_eq!(uow.state(), &committed);
}

#[test]
fn response_replay_rejects_a_changed_selected_competency() {
    let mut original = request();
    original.response.value = ResponseValue::Rubric {
        levels: BTreeMap::from([
            (id(41), Score::new(1.0).unwrap()),
            (id(42), Score::new(0.0).unwrap()),
        ]),
    };
    original.evidence_ids = BTreeMap::from([(id(8), id(33)), (id(10), id(37))]);
    let mut uow = InMemoryUnitOfWork::default();
    LearningCore::apply(
        &mut uow,
        &curriculum(),
        &multi_competency_assessment(),
        original.clone(),
    )
    .unwrap();
    let committed = uow.state().clone();
    original.operation_id = id(54);
    original.competency_id = id(10);
    assert!(matches!(
        LearningCore::apply(
            &mut uow,
            &curriculum(),
            &multi_competency_assessment(),
            original
        ),
        Err(CompositionError::ConflictingReplay)
    ));
    assert_eq!(uow.state(), &committed);
}

#[test]
fn multi_competency_rubric_updates_each_projection_and_selects_pedagogy_scope() {
    let mut operation = request();
    operation.response.value = ResponseValue::Rubric {
        levels: BTreeMap::from([
            (id(41), Score::new(1.0).unwrap()),
            (id(42), Score::new(0.0).unwrap()),
        ]),
    };
    operation.evidence_ids = BTreeMap::from([(id(8), id(33)), (id(10), id(37))]);
    // The authored lesson owns competency 8, so it is the explicitly selected routing scope.
    operation.competency_id = id(8);
    let mut uow = InMemoryUnitOfWork::default();
    let result = LearningCore::apply(
        &mut uow,
        &curriculum(),
        &multi_competency_assessment(),
        operation,
    )
    .unwrap();

    assert_eq!(result.pedagogy_decision.competency_id(), id(8));
    assert_eq!(result.event_facts.evidence_added.len(), 2);
    assert_eq!(result.event_facts.competency_updated.len(), 2);
    assert_eq!(uow.state().evidence.len(), 2);
    assert_eq!(uow.state().mastery.len(), 2);
    let first = uow
        .state()
        .mastery
        .iter()
        .find(|m| m.competency_id() == id(8))
        .unwrap();
    let second = uow
        .state()
        .mastery
        .iter()
        .find(|m| m.competency_id() == id(10))
        .unwrap();
    assert!(first.mastery().get() > second.mastery().get());
}

#[test]
fn every_persistence_stage_rolls_back_and_commit_retry_is_safe() {
    for stage in [
        CommitStage::Lesson,
        CommitStage::Assessment,
        CommitStage::Evidence,
        CommitStage::Mastery,
        CommitStage::Receipt,
        CommitStage::Finalize,
    ] {
        let mut uow = InMemoryUnitOfWork::default();
        uow.fail_next_at(stage);
        assert!(matches!(
            LearningCore::apply(&mut uow, &curriculum(), &assessment(), request()),
            Err(CompositionError::Persistence(_))
        ));
        assert_eq!(uow.state(), &LearningState::default());
        assert!(LearningCore::apply(&mut uow, &curriculum(), &assessment(), request()).is_ok());
    }
}

#[test]
fn malformed_and_cross_scope_requests_are_rejected_without_changes() {
    let cases = [
        {
            let mut r = request();
            r.version = ProtocolVersion::new(2, 0);
            r
        },
        {
            let mut r = request();
            r.response.student_id = id(99);
            r
        },
        {
            let mut r = request();
            r.response.assessment_id = id(99);
            r
        },
        {
            let mut r = request();
            r.competency_id = id(99);
            r
        },
        {
            let mut r = request();
            r.response.submitted_at = "2026-08-19T11:59:59Z".parse().unwrap();
            r
        },
    ];
    for r in cases {
        let mut uow = InMemoryUnitOfWork::default();
        assert!(LearningCore::apply(&mut uow, &curriculum(), &assessment(), r).is_err());
        assert_eq!(uow.state(), &LearningState::default());
    }
}

#[derive(Clone)]
struct UntrustedUnitOfWork {
    state: LearningState,
    commit_called: bool,
}

impl LearningUnitOfWork for UntrustedUnitOfWork {
    type Error = std::io::Error;

    fn load(&self) -> Result<LearningState, Self::Error> {
        Ok(self.state.clone())
    }

    fn commit(
        &mut self,
        _expected: &LearningState,
        _replacement: LearningState,
    ) -> Result<(), Self::Error> {
        self.commit_called = true;
        Ok(())
    }
}

#[test]
fn malformed_loaded_states_are_rejected_without_commit() {
    let mut source = InMemoryUnitOfWork::default();
    LearningCore::apply(&mut source, &curriculum(), &assessment(), request()).unwrap();
    let valid = source.state().clone();
    let mut cases = Vec::new();

    let mut duplicate_lesson = valid.clone();
    duplicate_lesson
        .lesson_progress
        .push(duplicate_lesson.lesson_progress[0].clone());
    cases.push(duplicate_lesson);

    let mut duplicate_attempt = valid.clone();
    duplicate_attempt
        .assessment_attempts
        .push(duplicate_attempt.assessment_attempts[0].clone());
    cases.push(duplicate_attempt);

    let mut duplicate_evidence = valid.clone();
    duplicate_evidence
        .evidence
        .push(duplicate_evidence.evidence[0].clone());
    cases.push(duplicate_evidence);

    let mut bad_receipt_key = valid.clone();
    let (_, receipt) = bad_receipt_key.receipts.pop_first().unwrap();
    bad_receipt_key.receipts.insert(id(99), receipt);
    cases.push(bad_receipt_key);

    let mut bad_receipt_scope = valid;
    bad_receipt_scope
        .receipts
        .first_entry()
        .unwrap()
        .get_mut()
        .result
        .assessment_attempt
        .student_id = id(99);
    cases.push(bad_receipt_scope);

    for state in cases {
        let original = state.clone();
        let mut uow = UntrustedUnitOfWork {
            state,
            commit_called: false,
        };
        assert!(matches!(
            LearningCore::apply(&mut uow, &curriculum(), &assessment(), request()),
            Err(CompositionError::Invalid(_))
        ));
        assert!(!uow.commit_called);
        assert_eq!(uow.state, original);
    }
}

#[test]
fn operation_golden_json_is_stable_and_malformed_wire_is_rejected() {
    let json = serde_json::to_string_pretty(&request()).unwrap() + "\n";
    assert_eq!(json, include_str!("fixtures/operation.json"));
    assert_eq!(
        serde_json::from_str::<LearningOperation>(&json).unwrap(),
        request()
    );
    for bad in [
        json.replace("\"version\": \"1.0\"", "\"version\": \"x\""),
        json.replace(
            "00000000-0000-0000-0000-000000000009",
            "00000000-0000-0000-0000-000000000000",
        ),
    ] {
        assert!(serde_json::from_str::<LearningOperation>(&bad).is_err());
    }
}
