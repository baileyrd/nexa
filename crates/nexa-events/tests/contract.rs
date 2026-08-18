use nexa_domain::*;
use nexa_events::*;
use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};

fn fixture() -> Event<StudentTextSubmitted> {
    Event {
        event_version: ProtocolVersion::new(1, 0),
        event_id: EventId::from_str("0193f24a-9c41-7a4d-b64f-b6fa5b76f001").unwrap(),
        event_type: EventKind::StudentTextSubmitted,
        timestamp: "2026-08-17T23:45:00Z".parse().unwrap(),
        session_id: Some("0193f249-95c2-79e0-a221-628003c28501".parse().unwrap()),
        sequence: Some(Sequence::new(1842)),
        source: "nexa.training_ui".parse().unwrap(),
        subject: Some("student.current".parse().unwrap()),
        correlation_id: Some("0193f249-c239-7434-a223-c1af15431322".parse().unwrap()),
        causation_id: Some("0193f249-ba42-7210-8aca-f69323269d50".parse().unwrap()),
        trace_id: Some("0193f249-dc15-79e2-9879-b96452e04511".parse().unwrap()),
        payload: StudentTextSubmitted {
            text: "What is TCP?".into(),
        },
        metadata: Default::default(),
    }
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
    assert_eq!(error.failed_subscribers, 1);
    assert_eq!(*calls.lock().unwrap(), 1);
}
