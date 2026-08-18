use crate::asset::Bounds;
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
    pub fn zoom(&mut self, delta_m: f32) {
        self.distance_m = (self.distance_m + delta_m).clamp(0.35, 10.0);
    }
    pub fn frame_bounds(&mut self, bounds: Bounds) {
        self.target = bounds.center();
        self.distance_m = (bounds.extent().length() * 1.35).clamp(0.75, 10.0);
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
    pub morph_weight: f32,
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
            morph_weight: 0.0,
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
    pub fn adjust_morph_weight(&mut self, delta: f32) {
        self.morph_weight = (self.morph_weight + delta).clamp(0.0, 1.0);
    }
    pub fn select_next_animation(&mut self, available_count: usize) {
        if available_count > 0 {
            self.selected_animation = (self.selected_animation + 1) % available_count;
            self.animation_time_seconds = 0.0;
        }
    }
    pub fn advance(&mut self, elapsed_seconds: f32) {
        if self.animation_playing {
            self.animation_time_seconds += elapsed_seconds.max(0.0);
        }
    }
    pub fn nudge_gaze_target(&mut self, offset: Vec3) {
        self.gaze_target += offset;
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
    fn zoom_stays_within_inspection_limits() {
        let mut camera = OrbitCamera::default();
        camera.zoom(-100.0);
        assert_eq!(camera.distance_m, 0.35);
        camera.zoom(100.0);
        assert_eq!(camera.distance_m, 10.0);
    }

    #[test]
    fn framing_uses_asset_center_and_reasonable_distance() {
        let mut camera = OrbitCamera::default();
        camera.frame_bounds(Bounds {
            minimum: Vec3::ZERO,
            maximum: Vec3::new(2.0, 4.0, 2.0),
        });
        assert_eq!(camera.target, Vec3::new(1.0, 2.0, 1.0));
        assert!(camera.distance_m > 5.0);
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

    #[test]
    fn morph_weight_stays_normalized() {
        let mut controls = RuntimeControls::default();
        controls.adjust_morph_weight(4.0);
        assert_eq!(controls.morph_weight, 1.0);
        controls.adjust_morph_weight(-4.0);
        assert_eq!(controls.morph_weight, 0.0);
    }

    #[test]
    fn playback_advances_only_while_enabled() {
        let mut controls = RuntimeControls::default();
        controls.advance(0.5);
        assert_eq!(controls.animation_time_seconds, 0.0);
        controls.animation_playing = true;
        controls.advance(0.5);
        controls.advance(-9.0);
        assert_eq!(controls.animation_time_seconds, 0.5);
    }

    #[test]
    fn gaze_target_can_be_nudged_without_changing_camera() {
        let mut controls = RuntimeControls::default();
        let original_camera = controls.camera;
        controls.nudge_gaze_target(Vec3::new(0.1, -0.2, 0.3));
        assert!(controls
            .gaze_target
            .abs_diff_eq(Vec3::new(0.1, 1.35, -0.7), 0.000_01));
        assert_eq!(controls.camera, original_camera);
    }
}
