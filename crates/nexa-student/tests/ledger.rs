use nexa_domain::*;
use nexa_student::*;

fn evidence(id: &str, at: &str, outcome: EvidenceOutcome) -> LearningEvidence {
    LearningEvidence {
        id: id.parse().unwrap(),
        student_id: "0193f249-95c2-79e0-a221-628003c28501".parse().unwrap(),
        competency_id: "0193f249-95c2-79e0-a221-628003c28502".parse().unwrap(),
        evidence_type: EvidenceType::Application,
        outcome,
        difficulty: EvidenceDifficulty::Moderate,
        independence: IndependenceLevel::Independent,
        confidence: Some(Confidence::new(0.9).unwrap()),
        source: EvidenceSource::Assessment("0193f249-95c2-79e0-a221-628003c28503".parse().unwrap()),
        observed_at: at.parse().unwrap(),
    }
}

#[test]
fn golden_evidence_round_trip() {
    let value = evidence(
        "0193f249-95c2-79e0-a221-628003c28504",
        "2026-08-19T10:00:00Z",
        EvidenceOutcome::Success,
    );
    let json = serde_json::to_string_pretty(&value).unwrap();
    assert_eq!(json, include_str!("fixtures/evidence.json").trim());
    assert_eq!(
        serde_json::from_str::<LearningEvidence>(&json).unwrap(),
        value
    );
}

#[test]
fn ledger_is_append_only_and_duplicates_are_idempotent() {
    let mut ledger = InMemoryEvidenceRepository::default();
    let item = evidence(
        "0193f249-95c2-79e0-a221-628003c28504",
        "2026-08-19T10:00:00Z",
        EvidenceOutcome::Success,
    );
    assert_eq!(
        ledger.append(item.clone()).unwrap(),
        AppendOutcome::Appended
    );
    assert_eq!(
        ledger.append(item.clone()).unwrap(),
        AppendOutcome::Duplicate
    );
    assert_eq!(ledger.all().unwrap(), vec![item.clone()]);
    let mut conflict = item;
    conflict.outcome = EvidenceOutcome::Failure;
    assert!(matches!(
        ledger.append(conflict),
        Err(StudentModelError::ConflictingDuplicate(_))
    ));
}

#[test]
fn replay_and_tie_order_are_deterministic() {
    let a = evidence(
        "0193f249-95c2-79e0-a221-628003c28504",
        "2026-08-19T10:00:00Z",
        EvidenceOutcome::Success,
    );
    let b = evidence(
        "0193f249-95c2-79e0-a221-628003c28505",
        "2026-08-19T10:00:00Z",
        EvidenceOutcome::Failure,
    );
    let expected = replay(&[a.clone(), b.clone()], &BoundedWeightedV1).unwrap();
    assert_eq!(
        replay(&[b, a.clone()], &BoundedWeightedV1).unwrap(),
        expected
    );
    assert_eq!(
        replay(&[a.clone(), a], &BoundedWeightedV1).unwrap()[0].evidence_count(),
        1
    );
    assert_eq!(expected[0].policy_version(), ProtocolVersion::new(1, 0));
}

#[test]
fn replay_rejects_conflicting_duplicates_regardless_of_input_order() {
    let original = evidence(
        "0193f249-95c2-79e0-a221-628003c28504",
        "2026-08-19T10:00:00Z",
        EvidenceOutcome::Success,
    );
    let mut conflict = original.clone();
    conflict.outcome = EvidenceOutcome::Failure;

    for input in [
        vec![original.clone(), conflict.clone()],
        vec![conflict.clone(), original.clone()],
    ] {
        assert_eq!(
            replay(&input, &BoundedWeightedV1),
            Err(StudentModelError::ConflictingDuplicate(original.id))
        );
    }
    assert_eq!(
        replay(&[original.clone(), original], &BoundedWeightedV1).unwrap()[0].evidence_count(),
        1
    );
}

#[test]
fn malformed_external_values_are_rejected() {
    assert!(serde_json::from_str::<Student>(r#"{"id":"0193f249-95c2-79e0-a221-628003c28501","display_name":"   ","created_at":"2026-08-19T10:00:00Z"}"#).is_err());
    assert!(serde_json::from_str::<Competency>(
        r#"{"id":"0193f249-95c2-79e0-a221-628003c28502","title":"TCP","objective_ids":[]}"#
    )
    .is_err());
    assert!(serde_json::from_str::<AssessmentAttempt>(r#"{"id":"0193f249-95c2-79e0-a221-628003c28503","assessment_id":"0193f249-95c2-79e0-a221-628003c28504","student_id":"0193f249-95c2-79e0-a221-628003c28501","started_at":"2026-08-20T10:00:00Z","completed_at":"2026-08-19T10:00:00Z"}"#).is_err());
    assert!(serde_json::from_str::<MasteryScore>("1.01").is_err());
}

#[test]
fn policy_rejects_cross_projection_evidence() {
    let item = evidence(
        "0193f249-95c2-79e0-a221-628003c28504",
        "2026-08-19T10:00:00Z",
        EvidenceOutcome::Success,
    );
    let wrong = "0193f249-95c2-79e0-a221-628003c28509".parse().unwrap();
    let state = MasteryState::empty(item.student_id, wrong, ProtocolVersion::new(1, 0));
    assert_eq!(
        BoundedWeightedV1.update(&state, &item),
        Err(StudentModelError::ProjectionMismatch)
    );
}

#[test]
fn policy_rejects_mismatched_versions_directly_and_from_repository() {
    let item = evidence(
        "0193f249-95c2-79e0-a221-628003c28504",
        "2026-08-19T10:00:00Z",
        EvidenceOutcome::Success,
    );
    let state = MasteryState::empty(
        item.student_id,
        item.competency_id,
        ProtocolVersion::new(2, 0),
    );
    let expected = Err(StudentModelError::PolicyVersionMismatch {
        expected: ProtocolVersion::new(1, 0),
        actual: ProtocolVersion::new(2, 0),
    });
    assert_eq!(BoundedWeightedV1.update(&state, &item), expected);

    let mut repository = InMemoryMasteryRepository::default();
    repository.put(state).unwrap();
    let loaded = repository
        .get(item.student_id, item.competency_id)
        .unwrap()
        .unwrap();
    assert_eq!(BoundedWeightedV1.update(&loaded, &item), expected);
}

#[test]
fn malformed_mastery_state_invariants_are_rejected() {
    let item = evidence(
        "0193f249-95c2-79e0-a221-628003c28504",
        "2026-08-19T10:00:00Z",
        EvidenceOutcome::Success,
    );
    let empty = MasteryState::empty(
        item.student_id,
        item.competency_id,
        ProtocolVersion::new(1, 0),
    );
    let mut value = serde_json::to_value(empty).unwrap();

    value["last_evidence_at"] = serde_json::json!("2026-08-19T10:00:00Z");
    assert!(serde_json::from_value::<MasteryState>(value.clone()).is_err());
    value["last_evidence_at"] = serde_json::Value::Null;
    value["mastery"] = serde_json::json!(0.5);
    assert!(serde_json::from_value::<MasteryState>(value.clone()).is_err());
    value["mastery"] = serde_json::json!(0.0);
    value["model_confidence"] = serde_json::json!(0.5);
    assert!(serde_json::from_value::<MasteryState>(value.clone()).is_err());
    value["model_confidence"] = serde_json::json!(0.0);
    value["status"] = serde_json::json!("mastered");
    assert!(serde_json::from_value::<MasteryState>(value).is_err());
}

#[test]
fn policy_rejects_evidence_count_overflow() {
    let item = evidence(
        "0193f249-95c2-79e0-a221-628003c28504",
        "2026-08-19T10:00:00Z",
        EvidenceOutcome::Success,
    );
    let state = serde_json::from_value::<MasteryState>(serde_json::json!({
        "student_id": item.student_id,
        "competency_id": item.competency_id,
        "mastery": 0.5,
        "model_confidence": 1.0,
        "status": "functional",
        "evidence_count": u32::MAX,
        "last_evidence_at": "2026-08-18T10:00:00Z",
        "policy_version": "1.0"
    }))
    .unwrap();
    assert_eq!(
        BoundedWeightedV1.update(&state, &item),
        Err(StudentModelError::EvidenceCountOverflow)
    );
}
