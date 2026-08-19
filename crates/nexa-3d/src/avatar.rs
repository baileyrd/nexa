//! Semantic contract compatible with the `NEXA-3D-001` behavior layer.
use glam::Vec3;
use nexa_avatar::{
    missing_required_capabilities, AvatarCapabilities, AvatarCapability, AvatarPort, AvatarReport,
    AvatarRequest,
};
use nexa_domain::{MessageId, SemanticKey};
use nexa_nbp::{
    BehaviorCancel, BehaviorCommand, BehaviorState, EmotionPreset, GazeTarget, GestureKind,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct GazeCommand {
    pub eye_target: Vec3,
    pub head_weight: f32,
    pub eye_weight: f32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionCommand {
    pub canonical_name: String,
    pub weight: f32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct VisemeCommand {
    pub canonical_name: String,
    pub weight: f32,
    pub duration_seconds: f32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GestureCommand {
    pub canonical_name: String,
    pub intensity: f32,
}

/// Port owned by the host renderer. No `wgpu` type crosses this boundary.
pub trait AvatarRenderer {
    fn set_behavior_state(&mut self, state: BehaviorState);
    fn cancel_behavior(&mut self, cancellation: &BehaviorCancel);
    fn set_expression(&mut self, command: ExpressionCommand);
    fn set_viseme(&mut self, command: VisemeCommand);
    fn set_gaze(&mut self, command: GazeCommand);
    fn play_gesture(&mut self, command: GestureCommand);
    fn set_animation_time(&mut self, seconds: f32);
}

/// Adapter translating orchestrator semantics into stable asset-facing commands.
pub struct NexaAvatarAdapter<R> {
    renderer: R,
    canvas_targets: BTreeMap<SemanticKey, Vec3>,
}
impl<R: AvatarRenderer> NexaAvatarAdapter<R> {
    pub fn new(renderer: R) -> Self {
        Self {
            renderer,
            canvas_targets: BTreeMap::new(),
        }
    }
    /// Registers a presentation-owned position for a semantic canvas object.
    pub fn register_canvas_target(&mut self, target_id: SemanticKey, position: Vec3) {
        self.canvas_targets.insert(target_id, position);
    }
    pub fn apply_gaze(&mut self, target: Vec3) {
        self.renderer.set_gaze(GazeCommand {
            eye_target: target,
            head_weight: 0.55,
            eye_weight: 1.0,
        });
    }
    pub fn apply_expression(&mut self, name: &str, weight: f32) {
        self.renderer.set_expression(ExpressionCommand {
            canonical_name: name.to_owned(),
            weight,
        });
    }
    pub fn apply_viseme(&mut self, name: &str, weight: f32, duration_seconds: f32) {
        self.renderer.set_viseme(VisemeCommand {
            canonical_name: name.to_owned(),
            weight,
            duration_seconds,
        });
    }
    pub fn apply_gesture(&mut self, name: &str, intensity: f32) {
        self.renderer.play_gesture(GestureCommand {
            canonical_name: name.to_owned(),
            intensity,
        });
    }
    pub fn into_inner(self) -> R {
        self.renderer
    }
}

impl<R: AvatarRenderer> AvatarPort for NexaAvatarAdapter<R> {
    fn capabilities(&self) -> AvatarCapabilities {
        AvatarCapabilities::new([
            AvatarCapability::BehaviorState,
            AvatarCapability::Emotion,
            AvatarCapability::Gaze,
            AvatarCapability::Gesture,
            AvatarCapability::Cancellation,
        ])
    }

    fn preview(&self, request: &AvatarRequest) -> AvatarReport {
        match request {
            AvatarRequest::Cancel {
                message_id,
                cancellation,
            } => AvatarReport::cancelled(*message_id, cancellation.behavior_id),
            AvatarRequest::Submit {
                message_id,
                command,
            } => {
                let missing = missing_required_capabilities(command, &self.capabilities());
                if let Some(capability) = missing.iter().next() {
                    return AvatarReport::degraded(
                        *message_id,
                        command.behavior_id,
                        SemanticKey::new("avatar.capability.unsupported")
                            .expect("static semantic key is valid"),
                        format!("{capability:?} is unsupported; semantic command was not applied"),
                    );
                }
                let unresolved_gaze = command.gaze.as_ref().is_some_and(|gaze| {
                    gaze.target_type == GazeTarget::CanvasObject
                        && gaze
                            .target_id
                            .as_ref()
                            .is_none_or(|target| !self.canvas_targets.contains_key(target))
                });
                if unresolved_gaze {
                    AvatarReport::degraded(
                        *message_id,
                        command.behavior_id,
                        SemanticKey::new("avatar.gaze.target_unresolved")
                            .expect("static semantic key is valid"),
                        "the semantic gaze target is not registered; command was not applied"
                            .into(),
                    )
                } else {
                    AvatarReport::completed(
                        *message_id,
                        SemanticKey::new("nexa-3d-runtime").expect("static semantic key is valid"),
                        command,
                    )
                }
            }
        }
    }

    fn submit(&mut self, message_id: MessageId, command: BehaviorCommand) -> AvatarReport {
        let missing = missing_required_capabilities(&command, &self.capabilities());
        if let Some(capability) = missing.iter().next() {
            return AvatarReport::degraded(
                message_id,
                command.behavior_id,
                SemanticKey::new("avatar.capability.unsupported")
                    .expect("static semantic key is valid"),
                format!("{capability:?} is unsupported; semantic command was not applied"),
            );
        }

        let gaze_target = command
            .gaze
            .as_ref()
            .and_then(|gaze| match gaze.target_type {
                GazeTarget::Student | GazeTarget::Camera => Some(Vec3::new(0.0, 1.55, -1.0)),
                GazeTarget::CanvasObject => gaze
                    .target_id
                    .as_ref()
                    .and_then(|target_id| self.canvas_targets.get(target_id).copied()),
            });
        if command.gaze.is_some() && gaze_target.is_none() {
            return AvatarReport::degraded(
                message_id,
                command.behavior_id,
                SemanticKey::new("avatar.gaze.target_unresolved")
                    .expect("static semantic key is valid"),
                "the semantic gaze target is not registered; command was not applied".into(),
            );
        }

        self.renderer.set_behavior_state(command.state);
        if let Some(emotion) = &command.emotion {
            let name = match emotion.preset {
                EmotionPreset::Neutral => "Neutral",
                EmotionPreset::Focused => "Focused",
                EmotionPreset::Encouraging => "Encouraging",
            };
            self.apply_expression(
                name,
                emotion.intensity.map_or(1.0, |value| value.get()) as f32,
            );
        }
        if let Some(target) = gaze_target {
            self.apply_gaze(target);
        }
        if let Some(gesture) = &command.gesture {
            let name = match gesture.kind {
                GestureKind::Idle => "Idle",
                GestureKind::Nod => "Nod",
                GestureKind::Point => "Point",
            };
            self.apply_gesture(name, gesture.intensity.get() as f32);
        }

        AvatarReport::completed(
            message_id,
            SemanticKey::new("nexa-3d-runtime").expect("static semantic key is valid"),
            &command,
        )
    }

    fn cancel(&mut self, message_id: MessageId, cancellation: BehaviorCancel) -> AvatarReport {
        self.renderer.cancel_behavior(&cancellation);
        AvatarReport::cancelled(message_id, cancellation.behavior_id)
    }
}

/// Converts and dispatches a complete NBP input without exposing renderer controls upstream.
pub fn dispatch_nbp<R: AvatarRenderer>(
    adapter: &mut NexaAvatarAdapter<R>,
    message: &nexa_nbp::NbpMessage,
) -> Result<AvatarReport, nexa_avatar::RequestConversionError> {
    let request = AvatarRequest::try_from(message)?;
    Ok(adapter.handle(request))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexa_domain::BehaviorId;

    #[derive(Default)]
    struct Recorder {
        state: Option<BehaviorState>,
        cancelled: Option<BehaviorId>,
        gaze: Option<GazeCommand>,
        viseme: Option<VisemeCommand>,
        expression: Option<ExpressionCommand>,
        gesture: Option<GestureCommand>,
    }
    impl AvatarRenderer for Recorder {
        fn set_behavior_state(&mut self, state: BehaviorState) {
            self.state = Some(state);
        }
        fn cancel_behavior(&mut self, cancellation: &BehaviorCancel) {
            self.cancelled = Some(cancellation.behavior_id);
        }
        fn set_expression(&mut self, command: ExpressionCommand) {
            self.expression = Some(command);
        }
        fn set_viseme(&mut self, command: VisemeCommand) {
            self.viseme = Some(command);
        }
        fn set_gaze(&mut self, command: GazeCommand) {
            self.gaze = Some(command);
        }
        fn play_gesture(&mut self, command: GestureCommand) {
            self.gesture = Some(command);
        }
        fn set_animation_time(&mut self, _: f32) {}
    }

    #[test]
    fn adapter_preserves_canonical_viseme_name() {
        let mut adapter = NexaAvatarAdapter::new(Recorder::default());
        adapter.apply_viseme("MBP", 0.8, 0.12);
        assert_eq!(adapter.into_inner().viseme.unwrap().canonical_name, "MBP");
    }

    #[test]
    fn avatar_port_maps_nbp_semantics_without_exposing_renderer_names_upstream() {
        use nexa_domain::{BehaviorId, Confidence};
        use nexa_nbp::{
            BehaviorState, Emotion, Gesture, Interruptibility, Priority, RuntimeStatus,
        };
        use std::str::FromStr;

        let command = BehaviorCommand {
            behavior_id: BehaviorId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249001").unwrap(),
            state: BehaviorState::Explaining,
            priority: Priority::default(),
            interruptibility: Interruptibility::Immediate,
            emotion: Some(Emotion {
                preset: EmotionPreset::Focused,
                confidence: None,
                intensity: Some(Confidence::new(0.75).unwrap()),
            }),
            gaze: None,
            gesture: Some(Gesture {
                kind: GestureKind::Point,
                target_id: None,
                intensity: Confidence::new(0.5).unwrap(),
                duration_ms: None,
            }),
            speech: None,
        };
        let mut adapter = NexaAvatarAdapter::new(Recorder::default());
        let report = adapter.submit(
            MessageId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249002").unwrap(),
            command,
        );
        assert_eq!(report.terminal_status(), RuntimeStatus::Completed);
        let recorder = adapter.into_inner();
        assert_eq!(recorder.expression.unwrap().canonical_name, "Focused");
        assert_eq!(recorder.gesture.unwrap().canonical_name, "Point");
        assert_eq!(recorder.state, Some(BehaviorState::Explaining));
    }

    #[test]
    fn unresolved_canvas_gaze_is_degraded_without_applying_the_command() {
        use nexa_domain::{BehaviorId, Confidence};
        use nexa_nbp::{BehaviorState, Gaze, Interruptibility, Priority, RuntimeStatus};
        use std::str::FromStr;

        let target_id = SemanticKey::new("lesson.diagram.axis").unwrap();
        let command = BehaviorCommand {
            behavior_id: BehaviorId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249001").unwrap(),
            state: BehaviorState::Attentive,
            priority: Priority::default(),
            interruptibility: Interruptibility::Immediate,
            emotion: None,
            gaze: Some(Gaze {
                target_type: GazeTarget::CanvasObject,
                target_id: Some(target_id.clone()),
                intensity: Confidence::new(1.0).unwrap(),
                duration_ms: None,
                lead_time_ms: None,
            }),
            gesture: None,
            speech: None,
        };
        let message_id = MessageId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249002").unwrap();

        let mut unresolved = NexaAvatarAdapter::new(Recorder::default());
        let report = unresolved.submit(message_id, command.clone());
        assert_eq!(report.terminal_status(), RuntimeStatus::Degraded);
        assert_eq!(
            report.error().unwrap().code.as_str(),
            "avatar.gaze.target_unresolved"
        );
        let recorder = unresolved.into_inner();
        assert!(recorder.state.is_none());
        assert!(recorder.gaze.is_none());

        let expected = Vec3::new(0.25, 1.1, -0.8);
        let mut resolved = NexaAvatarAdapter::new(Recorder::default());
        resolved.register_canvas_target(target_id, expected);
        let report = resolved.submit(message_id, command);
        assert_eq!(report.terminal_status(), RuntimeStatus::Completed);
        assert_eq!(resolved.into_inner().gaze.unwrap().eye_target, expected);
    }

    #[test]
    fn cancellation_acknowledges_only_after_forwarding_to_renderer() {
        use nexa_nbp::{CancellationMode, RuntimeStatus};
        use std::str::FromStr;

        let behavior_id = BehaviorId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249001").unwrap();
        let message_id = MessageId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249002").unwrap();
        let mut adapter = NexaAvatarAdapter::new(Recorder::default());
        let report = adapter.cancel(
            message_id,
            BehaviorCancel {
                behavior_id,
                reason: "new turn".into(),
                transition: CancellationMode::Immediate,
            },
        );
        assert_eq!(report.terminal_status(), RuntimeStatus::Cancelled);
        assert_eq!(adapter.into_inner().cancelled, Some(behavior_id));
    }
}
