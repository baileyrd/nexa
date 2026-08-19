use nexa_domain::*;
use nexa_events::*;
use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};

fn fixture() -> Event<StudentTextSubmitted> {
    Event::new(
        ProtocolVersion::new(1, 0),
        EventId::from_str("0193f24a-9c41-7a4d-b64f-b6fa5b76f001").unwrap(),
        "2026-08-17T23:45:00Z".parse().unwrap(),
        Some("0193f249-95c2-79e0-a221-628003c28501".parse().unwrap()),
        Some(Sequence::new(1842)),
        "nexa.training_ui".parse().unwrap(),
        Some("student.current".parse().unwrap()),
        Some("0193f249-c239-7434-a223-c1af15431322".parse().unwrap()),
        Some("0193f249-ba42-7210-8aca-f69323269d50".parse().unwrap()),
        Some("0193f249-dc15-79e2-9879-b96452e04511".parse().unwrap()),
        StudentTextSubmitted {
            text: "What is TCP?".into(),
        },
        Default::default(),
    )
}
#[test]
fn golden_event_round_trip() {
    let expected = include_str!("fixtures/student-text-submitted.json").trim();
    let json = serde_json::to_string_pretty(&fixture()).unwrap();
    assert_eq!(json, expected);
    assert_eq!(
        serde_json::from_str::<Event<StudentTextSubmitted>>(&json).unwrap(),
        fixture()
    );
}
#[test]
fn bus_filters_and_isolates_failures() {
    let bus = InProcessEventBus::new();
    let calls = Arc::new(Mutex::new(0));
    let seen = calls.clone();
    bus.subscribe(Some(EventKind::StudentTextSubmitted), move |_| {
        *seen.lock().unwrap() += 1;
        Ok(())
    });
    bus.subscribe(None, |_| {
        Err(SubscriberError {
            message: "expected".into(),
        })
    });
    let error = bus.publish(&fixture()).unwrap_err();
    assert!(matches!(
        error,
        PublishError::Subscribers {
            failed_subscribers: 1
        }
    ));
    assert_eq!(*calls.lock().unwrap(), 1);
}

#[test]
fn payload_kind_mismatch_is_rejected() {
    let json = serde_json::to_string(&fixture())
        .unwrap()
        .replace("student.text.submitted", "session.started");
    assert!(serde_json::from_str::<Event<StudentTextSubmitted>>(&json).is_err());
}

#[derive(Clone, serde::Deserialize)]
struct FailingPayload;

impl serde::Serialize for FailingPayload {
    fn serialize<S: serde::Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom("deliberate encoding failure"))
    }
}

impl DomainEvent for FailingPayload {
    const KIND: EventKind = EventKind::SystemError;
}

#[test]
fn fallible_payload_serializer_returns_encoding_error() {
    let original = fixture();
    let event = Event::new(
        original.event_version,
        original.event_id,
        original.timestamp,
        original.session_id,
        original.sequence,
        original.source,
        original.subject,
        original.correlation_id,
        original.causation_id,
        original.trace_id,
        FailingPayload,
        Default::default(),
    );
    assert!(matches!(
        InProcessEventBus::new().publish(&event),
        Err(PublishError::Encoding(_))
    ));
}

fn pedagogy_fixture() -> Event<PedagogyDecisionMade> {
    let payload = PedagogyDecisionMade::new(
        "018f6f3e-8c5d-7a20-8000-000000000001".parse().unwrap(),
        "018f6f3e-8c5d-7a20-8000-000000000002".parse().unwrap(),
        "review".parse().unwrap(),
        vec![
            "no_recent_outcome".parse().unwrap(),
            "preferred_option_unavailable".parse().unwrap(),
        ],
        ProtocolVersion::new(1, 0),
    )
    .unwrap();
    let original = fixture();
    Event::new(
        original.event_version,
        original.event_id,
        original.timestamp,
        original.session_id,
        original.sequence,
        original.source,
        original.subject,
        original.correlation_id,
        original.causation_id,
        original.trace_id,
        payload,
        Default::default(),
    )
}

#[test]
fn pedagogy_event_golden_round_trip_and_malformed_rationales() {
    let expected = include_str!("fixtures/pedagogy-decision-made.json").trim();
    let encoded = serde_json::to_string_pretty(&pedagogy_fixture()).unwrap();
    assert_eq!(encoded, expected);
    assert_eq!(
        serde_json::from_str::<Event<PedagogyDecisionMade>>(&encoded).unwrap(),
        pedagogy_fixture()
    );

    let value = serde_json::to_value(pedagogy_fixture()).unwrap();
    for rationales in [
        serde_json::json!([]),
        serde_json::json!(["no_recent_outcome", "no_recent_outcome"]),
        serde_json::json!(["preferred_option_unavailable", "no_recent_outcome"]),
    ] {
        let mut malformed = value.clone();
        malformed["payload"]["rationale_codes"] = rationales;
        assert!(serde_json::from_value::<Event<PedagogyDecisionMade>>(malformed).is_err());
    }
}

#[test]
fn assessment_payloads_have_closed_round_trip_contracts() {
    let evaluated = AssessmentResponseEvaluated::new(
        "018f6f3e-8c5d-7a20-8000-000000000010".parse().unwrap(),
        "018f6f3e-8c5d-7a20-8000-000000000011".parse().unwrap(),
        "018f6f3e-8c5d-7a20-8000-000000000012".parse().unwrap(),
        "018f6f3e-8c5d-7a20-8000-000000000013".parse().unwrap(),
        MasteryScore::new(0.5).unwrap(),
        ProtocolVersion::new(1, 0),
    );
    let golden = r#"{
  "attempt_id": "018f6f3e-8c5d-7a20-8000-000000000010",
  "assessment_id": "018f6f3e-8c5d-7a20-8000-000000000011",
  "item_instance_id": "018f6f3e-8c5d-7a20-8000-000000000012",
  "response_id": "018f6f3e-8c5d-7a20-8000-000000000013",
  "score": 0.5,
  "outcome": "partial",
  "policy_version": "1.0"
}"#;
    assert_eq!(serde_json::to_string_pretty(&evaluated).unwrap(), golden);
    let encoded = serde_json::to_value(&evaluated).unwrap();
    assert_eq!(encoded["outcome"], "partial");
    assert_eq!(
        serde_json::from_value::<AssessmentResponseEvaluated>(encoded.clone()).unwrap(),
        evaluated
    );
    let mut malformed = encoded.clone();
    malformed["outcome"] = "mostly_right".into();
    assert!(serde_json::from_value::<AssessmentResponseEvaluated>(malformed).is_err());
    for (score, outcome) in [(0.0, "correct"), (0.2, "incorrect"), (1.0, "partial")] {
        let mut inconsistent = encoded.clone();
        inconsistent["score"] = score.into();
        inconsistent["outcome"] = outcome.into();
        assert!(serde_json::from_value::<AssessmentResponseEvaluated>(inconsistent).is_err());
    }

    let completed = AssessmentCompleted::new(
        evaluated.attempt_id,
        evaluated.assessment_id,
        MasteryScore::new(0.8).unwrap(),
        MasteryScore::new(0.6).unwrap(),
        ProtocolVersion::new(1, 0),
    );
    assert!(completed.passed());
    let encoded = serde_json::to_value(&completed).unwrap();
    assert_eq!(
        serde_json::to_string_pretty(&completed).unwrap(),
        r#"{
  "attempt_id": "018f6f3e-8c5d-7a20-8000-000000000010",
  "assessment_id": "018f6f3e-8c5d-7a20-8000-000000000011",
  "score": 0.8,
  "passing_score": 0.6,
  "passed": true,
  "policy_version": "1.0"
}"#
    );
    assert_eq!(
        serde_json::from_value::<AssessmentCompleted>(encoded.clone()).unwrap(),
        completed
    );
    let mut contradictory = encoded;
    contradictory["passed"] = false.into();
    assert!(serde_json::from_value::<AssessmentCompleted>(contradictory).is_err());
}
