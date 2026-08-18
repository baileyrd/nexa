//! Semantic contract compatible with the `NEXA-3D-001` behavior layer.
use glam::Vec3;
use nexa_avatar::{AvatarCapabilities, AvatarCapability, AvatarPort, AvatarReport, AvatarRequest};
use nexa_domain::{MessageId, SemanticKey};
use nexa_nbp::{BehaviorCancel, BehaviorCommand, EmotionPreset, GazeTarget, GestureKind};

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
    fn set_expression(&mut self, command: ExpressionCommand);
    fn set_viseme(&mut self, command: VisemeCommand);
    fn set_gaze(&mut self, command: GazeCommand);
    fn play_gesture(&mut self, command: GestureCommand);
    fn set_animation_time(&mut self, seconds: f32);
}

/// Adapter translating orchestrator semantics into stable asset-facing commands.
pub struct NexaAvatarAdapter<R> {
    renderer: R,
}
impl<R: AvatarRenderer> NexaAvatarAdapter<R> {
    pub fn new(renderer: R) -> Self {
        Self { renderer }
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

    fn submit(&mut self, message_id: MessageId, command: BehaviorCommand) -> AvatarReport {
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
        if let Some(gaze) = &command.gaze {
            // World-space resolution remains in the presentation/manifest adapter.
            let target = match gaze.target_type {
                GazeTarget::Student | GazeTarget::Camera => Vec3::new(0.0, 1.55, -1.0),
                GazeTarget::CanvasObject => Vec3::new(0.45, 1.3, -1.0),
            };
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

        if command.speech.is_some() {
            AvatarReport::degraded(
                message_id,
                command.behavior_id,
                SemanticKey::new("avatar.capability.speech_unsupported")
                    .expect("static semantic key is valid"),
                "speech timing is not owned by the existing 3D runtime adapter".into(),
            )
        } else {
            AvatarReport::accepted(
                message_id,
                SemanticKey::new("nexa-3d-runtime").expect("static semantic key is valid"),
                &command,
            )
        }
    }

    fn cancel(&mut self, message_id: MessageId, cancellation: BehaviorCancel) -> AvatarReport {
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

    #[derive(Default)]
    struct Recorder {
        viseme: Option<VisemeCommand>,
        expression: Option<ExpressionCommand>,
        gesture: Option<GestureCommand>,
    }
    impl AvatarRenderer for Recorder {
        fn set_expression(&mut self, command: ExpressionCommand) {
            self.expression = Some(command);
        }
        fn set_viseme(&mut self, command: VisemeCommand) {
            self.viseme = Some(command);
        }
        fn set_gaze(&mut self, _: GazeCommand) {}
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
        assert_eq!(report.acknowledgement.status, RuntimeStatus::Accepted);
        let recorder = adapter.into_inner();
        assert_eq!(recorder.expression.unwrap().canonical_name, "Focused");
        assert_eq!(recorder.gesture.unwrap().canonical_name, "Point");
    }
}
