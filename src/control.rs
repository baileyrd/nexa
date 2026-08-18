use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrbitCamera {
    pub yaw_radians: f32,
    pub pitch_radians: f32,
    pub distance_m: f32,
    pub target: Vec3,
}
impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            yaw_radians: 0.35,
            pitch_radians: 0.15,
            distance_m: 2.4,
            target: Vec3::new(0.0, 1.05, 0.0),
        }
    }
}
impl OrbitCamera {
    pub fn orbit(&mut self, yaw: f32, pitch: f32) {
        self.yaw_radians += yaw;
        self.pitch_radians = (self.pitch_radians + pitch).clamp(-1.45, 1.45);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorPanel {
    Skeleton,
    MorphTargets,
    Animation,
    Gaze,
    Viseme,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeControls {
    pub panel: InspectorPanel,
    pub camera: OrbitCamera,
    pub selected_morph: usize,
    pub selected_animation: usize,
    pub animation_playing: bool,
    pub animation_time_seconds: f32,
    pub gaze_enabled: bool,
    pub gaze_target: Vec3,
    pub pending_viseme: Option<String>,
}
impl Default for RuntimeControls {
    fn default() -> Self {
        Self {
            panel: InspectorPanel::Skeleton,
            camera: OrbitCamera::default(),
            selected_morph: 0,
            selected_animation: 0,
            animation_playing: false,
            animation_time_seconds: 0.0,
            gaze_enabled: true,
            gaze_target: Vec3::new(0.0, 1.55, -1.0),
            pending_viseme: None,
        }
    }
}
impl RuntimeControls {
    pub fn scrub(&mut self, seconds: f32) {
        self.animation_time_seconds = (self.animation_time_seconds + seconds).max(0.0);
    }
    pub fn trigger_viseme(&mut self, canonical_name: impl Into<String>) {
        self.pending_viseme = Some(canonical_name.into());
    }
    pub fn select_next_morph(&mut self, available_count: usize) {
        if available_count > 0 {
            self.selected_morph = (self.selected_morph + 1) % available_count;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrubbing_never_moves_before_animation_start() {
        let mut controls = RuntimeControls::default();
        controls.scrub(-5.0);
        assert_eq!(controls.animation_time_seconds, 0.0);
        controls.scrub(0.25);
        assert_eq!(controls.animation_time_seconds, 0.25);
    }

    #[test]
    fn orbit_pitch_is_constrained_for_debug_camera_safety() {
        let mut camera = OrbitCamera::default();
        camera.orbit(0.0, 99.0);
        assert_eq!(camera.pitch_radians, 1.45);
    }

    #[test]
    fn morph_selection_wraps_and_ignores_empty_assets() {
        let mut controls = RuntimeControls::default();
        controls.selected_morph = 2;
        controls.select_next_morph(3);
        assert_eq!(controls.selected_morph, 0);
        controls.select_next_morph(0);
        assert_eq!(controls.selected_morph, 0);
    }
}
