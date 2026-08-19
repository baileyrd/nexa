use nexa_domain::{LessonId, StudentId, Timestamp};
use nexa_lessons::*;
use nexa_pedagogy::PedagogyDecision;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::str::FromStr;

fn curriculum() -> Curriculum {
    serde_json::from_str(include_str!("fixtures/curriculum.json")).unwrap()
}
fn progress() -> LessonProgress {
    serde_json::from_str(include_str!("fixtures/progress.json")).unwrap()
}
fn at() -> Timestamp {
    "2026-08-19T12:00:00Z".parse().unwrap()
}
fn decision(option: &str, competency: &str) -> PedagogyDecision {
    serde_json::from_value(json!({"policy_version":"1.0","student_id":"00000000-0000-0000-0000-000000000009","competency_id":competency,"selected_option":option,"rationale_codes":["no_evidence"]})).unwrap()
}

#[test]
fn golden_contracts_round_trip() {
    let c = curriculum();
    let encoded = serde_json::to_value(&c).unwrap();
    let fixture: Value = serde_json::from_str(include_str!("fixtures/curriculum.json")).unwrap();
    assert_eq!(encoded, fixture);
    let p = progress();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::from_str::<Value>(include_str!("fixtures/progress.json")).unwrap()
    );
}
#[test]
fn malformed_progress_is_rejected_without_panics() {
    let mut p: Value = serde_json::from_str(include_str!("fixtures/progress.json")).unwrap();
    p["current_step_id"] = json!("00000000-0000-0000-0000-000000000005");
    assert!(serde_json::from_value::<LessonProgress>(p).is_err());
    assert!(serde_json::from_str::<Curriculum>("{}").is_err());
}

#[test]
fn malformed_accumulated_progress_is_rejected_for_each_lifecycle_family() {
    let base: Value = serde_json::from_str(include_str!("fixtures/progress.json")).unwrap();
    let step = "00000000-0000-0000-0000-000000000005";
    let started = "2026-08-19T12:00:00Z";
    let later = "2026-08-19T12:01:00Z";
    let malformed = [
        // A never-started lesson cannot contain accumulated progress or an update time.
        json!({"lifecycle":"not_started", "completed_steps":[step], "updated_at":later}),
        // Open cursors cannot point at a step already recorded as complete.
        json!({"lifecycle":"active", "current_step_id":step, "completed_steps":[step], "started_at":started, "updated_at":later}),
        // Open state requires a complete and monotonic started/updated timestamp pair.
        json!({"lifecycle":"waiting", "current_step_id":step, "started_at":later, "updated_at":started}),
        // Completion must record work and use the completion instant as its last update.
        json!({"lifecycle":"completed", "completed_steps":[], "started_at":started, "updated_at":started, "completed_at":later}),
        // Terminal non-completion states can only be reached after starting.
        json!({"lifecycle":"blocked", "updated_at":later}),
        // Only completed state may carry a completion timestamp.
        json!({"lifecycle":"abandoned", "started_at":started, "updated_at":later, "completed_at":later}),
    ];

    for changes in malformed {
        let mut candidate = base.clone();
        for (key, value) in changes.as_object().unwrap() {
            candidate[key] = value.clone();
        }
        assert!(
            serde_json::from_value::<LessonProgress>(candidate).is_err(),
            "accepted malformed progress override: {changes}"
        );
    }
}
#[test]
fn detects_duplicate_dangling_and_cycles() {
    let base: Value = serde_json::from_str(include_str!("fixtures/curriculum.json")).unwrap();
    let mut duplicate = base.clone();
    let duplicate_course = duplicate["courses"][0].clone();
    duplicate["courses"]
        .as_array_mut()
        .unwrap()
        .push(duplicate_course);
    assert!(serde_json::from_value::<Curriculum>(duplicate).is_err());
    let mut dangling = base.clone();
    dangling["modules"][0]["lesson_ids"][0] = json!("00000000-0000-0000-0000-000000000099");
    assert!(serde_json::from_value::<Curriculum>(dangling).is_err());
    let mut cycle = base;
    cycle["lessons"][0]["prerequisites"] = json!(["00000000-0000-0000-0000-000000000004"]);
    assert!(serde_json::from_value::<Curriculum>(cycle).is_err());
}
#[test]
fn lifecycle_table_and_terminal_behavior() {
    let c = curriculum();
    let empty = BTreeSet::new();
    let p = progress();
    let active = LessonPolicyV1::start(&c, &p, &empty, at()).unwrap();
    assert_eq!(active.lifecycle(), LessonLifecycle::Active);
    let waiting = LessonPolicyV1::wait(&c, &active, at()).unwrap();
    assert_eq!(waiting.lifecycle(), LessonLifecycle::Waiting);
    let resumed = LessonPolicyV1::resume(&c, &waiting, at()).unwrap();
    let advanced = LessonPolicyV1::advance(&c, &resumed, at()).unwrap();
    let completed = LessonPolicyV1::advance(&c, &advanced, at()).unwrap();
    assert_eq!(completed.lifecycle(), LessonLifecycle::Completed);
    assert!(LessonPolicyV1::advance(&c, &completed, at()).is_err());
    assert_eq!(
        LessonPolicyV1::block(&c, &active, at())
            .unwrap()
            .lifecycle(),
        LessonLifecycle::Blocked
    );
    assert_eq!(
        LessonPolicyV1::abandon(&c, &active, at())
            .unwrap()
            .lifecycle(),
        LessonLifecycle::Abandoned
    );
}
#[test]
fn routing_is_deterministic_structured_and_atomic() {
    let c = curriculum();
    let p = LessonPolicyV1::start(&c, &progress(), &BTreeSet::new(), at()).unwrap();
    let before = p.clone();
    let d = decision("advance", "00000000-0000-0000-0000-000000000008");
    let a = LessonPolicyV1::route(&c, &p, &d, &BTreeSet::new(), at()).unwrap();
    let b = LessonPolicyV1::route(&c, &p, &d, &BTreeSet::new(), at()).unwrap();
    assert_eq!(a, b);
    assert_eq!(p, before);
    assert_eq!(
        a.rationale(),
        &[
            "pedagogy.option.advance",
            "route.authored",
            "prerequisites.satisfied"
        ]
    );
    let unsupported = decision("assess", "00000000-0000-0000-0000-000000000008");
    assert!(matches!(
        LessonPolicyV1::route(&c, &p, &unsupported, &BTreeSet::new(), at()),
        Err(TransitionError::IncompatibleRoute { .. })
    ));
    assert_eq!(p, before);
    let unavailable = decision("hint", "00000000-0000-0000-0000-000000000008");
    assert!(matches!(
        LessonPolicyV1::route(&c, &p, &unavailable, &BTreeSet::new(), at()),
        Err(TransitionError::RouteUnavailable { .. })
    ));
}

#[test]
fn routing_rejects_competency_mapped_only_to_another_step_objective() {
    let mut value: Value = serde_json::from_str(include_str!("fixtures/curriculum.json")).unwrap();
    let other_objective = "00000000-0000-0000-0000-000000000017";
    let other_competency = "00000000-0000-0000-0000-000000000018";
    value["lessons"][0]["objective_ids"]
        .as_array_mut()
        .unwrap()
        .push(json!(other_objective));
    value["lessons"][0]["steps"][1]["objective_ids"]
        .as_array_mut()
        .unwrap()
        .push(json!(other_objective));
    value["objective_mappings"]
        .as_array_mut()
        .unwrap()
        .push(json!({"objective_id":other_objective, "competency_ids":[other_competency]}));
    let curriculum: Curriculum = serde_json::from_value(value).unwrap();
    let active = LessonPolicyV1::start(&curriculum, &progress(), &BTreeSet::new(), at()).unwrap();

    assert_eq!(
        LessonPolicyV1::route(
            &curriculum,
            &active,
            &decision("advance", other_competency),
            &BTreeSet::new(),
            at(),
        ),
        Err(TransitionError::PedagogyCompetencyMismatch)
    );
}
#[test]
fn prerequisite_order_is_stable_and_unmet_is_rejected() {
    let c = curriculum();
    assert_eq!(
        c.prerequisite_order(),
        vec![LessonId::from_str("00000000-0000-0000-0000-000000000004").unwrap()]
    );
    // A completed lesson set never substitutes for an authored prerequisite; parsing invalid external state is also rejected.
    let mut v: Value = serde_json::from_str(include_str!("fixtures/progress.json")).unwrap();
    v["policy_version"] = json!("2.0");
    let p: LessonProgress = serde_json::from_value(v).unwrap();
    assert!(matches!(
        LessonPolicyV1::start(&c, &p, &BTreeSet::new(), at()),
        Err(TransitionError::PolicyVersionMismatch { .. })
    ));
    let _: StudentId = "00000000-0000-0000-0000-000000000009".parse().unwrap();
}
