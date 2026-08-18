//! Semantic contract compatible with the `NEXA-3D-001` behavior layer.
use glam::Vec3;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Recorder {
        viseme: Option<VisemeCommand>,
    }
    impl AvatarRenderer for Recorder {
        fn set_expression(&mut self, _: ExpressionCommand) {}
        fn set_viseme(&mut self, command: VisemeCommand) {
            self.viseme = Some(command);
        }
        fn set_gaze(&mut self, _: GazeCommand) {}
        fn play_gesture(&mut self, _: GestureCommand) {}
        fn set_animation_time(&mut self, _: f32) {}
    }

    #[test]
    fn adapter_preserves_canonical_viseme_name() {
        let mut adapter = NexaAvatarAdapter::new(Recorder::default());
        adapter.apply_viseme("MBP", 0.8, 0.12);
        assert_eq!(adapter.into_inner().viseme.unwrap().canonical_name, "MBP");
    }
}
