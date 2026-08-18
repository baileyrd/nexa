use nexa_domain::*;
use nexa_nbp::*;

fn fixture() -> NbpMessage {
    NbpMessage::new(
        ProtocolVersion::new(1, 0),
        "018f5e91-16bd-7e67-83df-9d11f18444a1".parse().unwrap(),
        "2026-08-17T23:30:00Z".parse().unwrap(),
        "018f5e90-acde-7a45-b369-102662c8112a".parse().unwrap(),
        Sequence::new(1042),
        "nexa.orchestrator".parse().unwrap(),
        Some("nexa.avatar.primary".parse().unwrap()),
        Some("018f5e90-fc92-75a0-9899-8890489270ff".parse().unwrap()),
        Payload::BehaviorCommand(BehaviorCommand {
            behavior_id: "018f5e91-16bd-7e67-83df-9d11f18444a2".parse().unwrap(),
            state: BehaviorState::Explaining,
            priority: Priority::new(50).unwrap(),
            interruptibility: Interruptibility::PhraseBoundary,
            emotion: Some(Emotion {
                preset: EmotionPreset::Encouraging,
                confidence: Some(Confidence::new(0.88).unwrap()),
                intensity: Some(Confidence::new(0.65).unwrap()),
            }),
            gaze: Some(Gaze {
                target_type: GazeTarget::CanvasObject,
                target_id: Some("tcp.syn_ack".parse().unwrap()),
                intensity: Confidence::new(0.85).unwrap(),
                duration_ms: Some(DurationMs::new(2400)),
                lead_time_ms: Some(DurationMs::new(200)),
            }),
            gesture: Some(Gesture {
                kind: GestureKind::Point,
                target_id: Some("tcp.syn_ack".parse().unwrap()),
                intensity: Confidence::new(0.6).unwrap(),
                duration_ms: Some(DurationMs::new(1800)),
            }),
            speech: Some(Speech {
                text: "The server responds with SYN-ACK.".into(),
                style: SpeechStyle::Instructional,
                allow_interruption: true,
                emit_visemes: true,
            }),
        }),
        Default::default(),
    )
    .unwrap()
}
#[test]
fn golden_behavior_command_round_trip() {
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/behavior-command.json")).unwrap();
    let json = serde_json::to_string_pretty(&fixture()).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&json).unwrap(),
        expected
    );
    assert_eq!(
        serde_json::from_str::<NbpMessage>(&json).unwrap(),
        fixture()
    );
}
#[test]
fn mismatched_payload_is_rejected() {
    let json =
        include_str!("fixtures/behavior-command.json").replace("behavior.command", "runtime.ack");
    assert!(serde_json::from_str::<NbpMessage>(&json).is_err());
}
#[test]
fn major_version_and_priority_are_validated() {
    assert!(Priority::new(101).is_err());
    assert!(serde_json::from_str::<Priority>("101").is_err());
    let mut value = serde_json::to_value(fixture()).unwrap();
    value["nbp_version"] = "2.0".into();
    assert!(serde_json::from_value::<NbpMessage>(value).is_err());
}

#[test]
fn extension_keys_follow_the_wire_grammar() {
    for key in ["live2d.physics_hint", "com.vendor.feature-name", "x.0"] {
        let mut extensions = Extensions::new();
        extensions.insert(key.into(), Default::default());
        assert!(NbpMessage::new(
            fixture().nbp_version,
            fixture().message_id,
            fixture().timestamp,
            fixture().session_id,
            fixture().sequence,
            fixture().source,
            fixture().target,
            fixture().correlation_id,
            fixture().payload,
            extensions,
        )
        .is_ok());
    }

    for key in ["a. b", "single", ".local", "ns.", "UPPER.local", "a.!bad"] {
        let mut extensions = Extensions::new();
        extensions.insert(key.into(), Default::default());
        let mut value = serde_json::to_value(fixture()).unwrap();
        value["extensions"] = serde_json::to_value(extensions).unwrap();
        assert!(
            serde_json::from_value::<NbpMessage>(value).is_err(),
            "{key}"
        );
    }
}
