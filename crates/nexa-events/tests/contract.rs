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
