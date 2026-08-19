use nexa_domain::ProtocolVersion;
use nexa_pedagogy::*;
use nexa_student::MasteryState;
use serde_json::json;

const STUDENT: &str = "018f6f3e-8c5d-7a20-8000-000000000001";
const COMPETENCY: &str = "018f6f3e-8c5d-7a20-8000-000000000002";

fn state(score: f64, confidence: f64, status: &str, count: u32, version: &str) -> MasteryState {
    serde_json::from_value(json!({
        "student_id": STUDENT, "competency_id": COMPETENCY, "mastery": score,
        "model_confidence": confidence, "status": status, "evidence_count": count,
        "last_evidence_at": if count == 0 { serde_json::Value::Null } else { json!("2026-08-19T00:00:00Z") },
        "policy_version": version
    })).unwrap()
}

fn decide(
    s: MasteryState,
    outcome: Option<RecentOutcome>,
    attempts: u32,
    failures: u32,
    options: &[InstructionalOption],
) -> Result<PedagogyDecision, PedagogyError> {
    let input = PedagogyInput::new(
        ProtocolVersion::new(1, 0),
        s,
        outcome,
        attempts,
        failures,
        options.iter().copied(),
    )?;
    PedagogyPolicyV1.decide(&input)
}

#[test]
fn golden_round_trip_and_deterministic_replay() {
    let input = PedagogyInput::new(
        ProtocolVersion::new(1, 0),
        state(0.5, 0.8, "functional", 3, "1.0"),
        Some(RecentOutcome::PartialSuccess),
        2,
        0,
        [InstructionalOption::Clarify, InstructionalOption::Practice],
    )
    .unwrap();
    let golden = include_str!("fixtures/input.json").trim();
    assert_eq!(serde_json::to_string_pretty(&input).unwrap(), golden);
    let decoded: PedagogyInput = serde_json::from_str(golden).unwrap();
    assert_eq!(decoded, input);
    let before = serde_json::to_value(&input).unwrap();
    let first = PedagogyPolicyV1.decide(&input).unwrap();
    for _ in 0..20 {
        assert_eq!(PedagogyPolicyV1.decide(&input).unwrap(), first);
    }
    assert_eq!(
        serde_json::to_value(&input).unwrap(),
        before,
        "policy mutated its input"
    );
    assert!(!first.rationale_codes.is_empty());
}

#[test]
fn decision_table_and_boundaries() {
    use InstructionalOption::*;
    let all = [
        Introduce,
        Explain,
        Demonstrate,
        Practice,
        Hint,
        Clarify,
        Reinforce,
        Review,
        Challenge,
        Assess,
        Retry,
        Advance,
    ];
    let cases = [
        (
            state(0.0, 0.0, "unestablished", 0, "1.0"),
            None,
            0,
            0,
            Introduce,
            RationaleCode::NoEvidence,
        ),
        (
            state(0.2, 0.2, "emerging", 1, "1.0"),
            Some(RecentOutcome::Success),
            1,
            0,
            Assess,
            RationaleCode::InsufficientEvidence,
        ),
        (
            state(0.4, 0.599, "developing", 2, "1.0"),
            Some(RecentOutcome::Success),
            1,
            0,
            Assess,
            RationaleCode::LowModelConfidence,
        ),
        (
            state(0.4, 0.60, "developing", 2, "1.0"),
            Some(RecentOutcome::Failure),
            1,
            1,
            Hint,
            RationaleCode::RecentFailure,
        ),
        (
            state(0.4, 0.8, "developing", 3, "1.0"),
            Some(RecentOutcome::Failure),
            2,
            2,
            Clarify,
            RationaleCode::RepeatedFailure,
        ),
        (
            state(0.4, 0.8, "developing", 3, "1.0"),
            Some(RecentOutcome::Failure),
            3,
            3,
            Reinforce,
            RationaleCode::RetryLimitReached,
        ),
        (
            state(0.5, 0.8, "functional", 3, "1.0"),
            Some(RecentOutcome::PartialSuccess),
            2,
            0,
            Practice,
            RationaleCode::RecentPartialSuccess,
        ),
        (
            state(0.849, 0.8, "proficient", 5, "1.0"),
            Some(RecentOutcome::Success),
            2,
            0,
            Challenge,
            RationaleCode::RecentSuccess,
        ),
        (
            state(0.85, 0.8, "proficient", 5, "1.0"),
            Some(RecentOutcome::Success),
            2,
            0,
            Assess,
            RationaleCode::MasteryThresholdMet,
        ),
        (
            state(0.9, 1.0, "mastered", 5, "1.0"),
            Some(RecentOutcome::Success),
            2,
            0,
            Advance,
            RationaleCode::CompetencyMastered,
        ),
        (
            state(0.4, 0.8, "developing", 2, "1.0"),
            None,
            0,
            0,
            Review,
            RationaleCode::InsufficientEvidence,
        ),
    ];
    for (s, o, a, f, expected, reason) in cases {
        let d = decide(s, o, a, f, &all).unwrap();
        assert_eq!(d.selected_option, expected);
        assert!(d.rationale_codes.contains(&reason));
    }
}

#[test]
fn unavailable_preference_uses_only_available_stable_fallback() {
    let d = decide(
        state(0.9, 1.0, "mastered", 5, "1.0"),
        Some(RecentOutcome::Success),
        1,
        0,
        &[InstructionalOption::Review],
    )
    .unwrap();
    assert_eq!(d.selected_option, InstructionalOption::Review);
    assert!(d
        .rationale_codes
        .contains(&RationaleCode::PreferredOptionUnavailable));
}

#[test]
fn versions_and_malformed_inputs_are_structured_errors() {
    let s = state(0.4, 0.8, "developing", 2, "1.0");
    let input = PedagogyInput::new(
        ProtocolVersion::new(2, 0),
        s,
        None,
        0,
        0,
        [InstructionalOption::Review],
    )
    .unwrap();
    assert!(matches!(
        PedagogyPolicyV1.decide(&input),
        Err(PedagogyError::PolicyVersionMismatch { .. })
    ));
    assert!(matches!(
        decide(
            state(0.4, 0.8, "developing", 2, "2.0"),
            None,
            0,
            0,
            &[InstructionalOption::Review]
        ),
        Err(PedagogyError::ProjectionVersionMismatch { .. })
    ));
    for malformed in [
        json!({"policy_version":"1.0","mastery":state(0.0,0.0,"unestablished",0,"1.0"),"recent_outcome":null,"attempt_count":0,"consecutive_failures":0,"available_options":[]}),
        json!({"policy_version":"1.0","mastery":state(0.0,0.0,"unestablished",0,"1.0"),"recent_outcome":"failure","attempt_count":0,"consecutive_failures":1,"available_options":["retry"]}),
        json!({"policy_version":"1.0","mastery":state(0.4,0.8,"developing",2,"1.0"),"recent_outcome":"success","attempt_count":1,"consecutive_failures":1,"available_options":["practice"]}),
        json!({"policy_version":"1.0","mastery":state(0.4,0.8,"mastered",2,"1.0"),"recent_outcome":null,"attempt_count":0,"consecutive_failures":0,"available_options":["advance"]}),
    ] {
        assert!(serde_json::from_value::<PedagogyInput>(malformed).is_err());
    }
    assert!(serde_json::from_str::<PedagogyInput>("not json").is_err());
}
