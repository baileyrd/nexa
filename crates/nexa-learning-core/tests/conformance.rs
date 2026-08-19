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
