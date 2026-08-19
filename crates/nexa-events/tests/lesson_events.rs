use nexa_domain::{
    LessonId, LessonStepId, LessonTransitionId, ProtocolVersion, SemanticKey, StudentId,
};
use nexa_events::{LessonEventLifecycle, LessonLifecycleChanged, LessonTransitionApplied};

#[test]
fn lesson_lifecycle_payload_golden_round_trip_and_malformed_rejection() {
    let transition_id = "00000000-0000-0000-0000-000000000010"
        .parse::<LessonTransitionId>()
        .unwrap();
    let student_id = "00000000-0000-0000-0000-000000000009"
        .parse::<StudentId>()
        .unwrap();
    let lesson_id = "00000000-0000-0000-0000-000000000004"
        .parse::<LessonId>()
        .unwrap();
    let payload = LessonLifecycleChanged::new(
        transition_id,
        student_id,
        lesson_id,
        LessonEventLifecycle::NotStarted,
        LessonEventLifecycle::Active,
        ProtocolVersion::new(1, 0),
    )
    .unwrap();
    let golden = r#"{"transition_id":"00000000-0000-0000-0000-000000000010","student_id":"00000000-0000-0000-0000-000000000009","lesson_id":"00000000-0000-0000-0000-000000000004","from":"not_started","to":"active","policy_version":"1.0"}"#;
    assert_eq!(serde_json::to_string(&payload).unwrap(), golden);
    assert_eq!(
        serde_json::from_str::<LessonLifecycleChanged>(golden).unwrap(),
        payload
    );
    assert!(serde_json::from_str::<LessonLifecycleChanged>(
        &golden.replace("not_started", "banana")
    )
    .is_err());
    assert!(serde_json::from_str::<LessonLifecycleChanged>(
        &golden.replace("not_started", "active")
    )
    .is_err());
}

#[test]
fn lesson_transition_payload_golden_round_trip_and_malformed_rejection() {
    let payload = LessonTransitionApplied::new(
        "00000000-0000-0000-0000-000000000010"
            .parse::<LessonTransitionId>()
            .unwrap(),
        "00000000-0000-0000-0000-000000000009"
            .parse::<StudentId>()
            .unwrap(),
        "00000000-0000-0000-0000-000000000004"
            .parse::<LessonId>()
            .unwrap(),
        Some(
            "00000000-0000-0000-0000-000000000005"
                .parse::<LessonStepId>()
                .unwrap(),
        ),
        Some(
            "00000000-0000-0000-0000-000000000006"
                .parse::<LessonStepId>()
                .unwrap(),
        ),
        vec![
            "authored.order".parse::<SemanticKey>().unwrap(),
            "lesson.advanced".parse().unwrap(),
        ],
        ProtocolVersion::new(1, 0),
    )
    .unwrap();
    let golden = r#"{"transition_id":"00000000-0000-0000-0000-000000000010","student_id":"00000000-0000-0000-0000-000000000009","lesson_id":"00000000-0000-0000-0000-000000000004","from_step_id":"00000000-0000-0000-0000-000000000005","to_step_id":"00000000-0000-0000-0000-000000000006","rationale_codes":["authored.order","lesson.advanced"],"policy_version":"1.0"}"#;
    assert_eq!(serde_json::to_string(&payload).unwrap(), golden);
    assert_eq!(
        serde_json::from_str::<LessonTransitionApplied>(golden).unwrap(),
        payload
    );
    let malformed = golden.replace("[\"authored.order\",\"lesson.advanced\"]", "[]");
    assert!(serde_json::from_str::<LessonTransitionApplied>(&malformed).is_err());
}
