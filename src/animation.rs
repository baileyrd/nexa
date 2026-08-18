//! Renderer-neutral animation pose sampling.
//!
//! The GLB importer adapts exported channels into these values; render backends
//! consume the sampled pose without taking a dependency on `wgpu` or `winit`.

use std::collections::BTreeMap;

use glam::{Quat, Vec3};

use crate::asset::AssetError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interpolation {
    Linear,
    Step,
    CubicSpline,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelValues {
    Translation(Vec3),
    Rotation(Quat),
    Scale(Vec3),
    MorphWeight(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Channel {
    pub node_index: usize,
    pub interpolation: Interpolation,
    pub times_seconds: Vec<f32>,
    pub values: Vec<ChannelValues>,
    /// glTF cubic-spline `(in_tangent, out_tangent)` pairs, one per keyframe.
    /// Tangents are expressed per second, exactly as exported by glTF.
    pub cubic_tangents: Option<Vec<(ChannelValues, ChannelValues)>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeTransform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for NodeTransform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Pose {
    pub nodes: BTreeMap<usize, NodeTransform>,
    pub morph_weights: BTreeMap<usize, f32>,
}

/// Reads the selected GLB animation into the renderer-neutral sampler model.
///
/// Multi-target weight channels deliberately do not enter this first adapter:
/// they need target-index preservation rather than a lossy scalar conversion.
pub fn load_supported_channels(
    path: impl AsRef<std::path::Path>,
    animation_index: usize,
) -> Result<Vec<Channel>, AssetError> {
    let path = path.as_ref().to_path_buf();
    let (document, buffers, _images) =
        gltf::import(&path).map_err(|source| AssetError::Import { path, source })?;
    let Some(animation) = document.animations().nth(animation_index) else {
        return Ok(Vec::new());
    };
    let mut channels = Vec::new();
    for source in animation.channels() {
        let interpolation = match source.sampler().interpolation() {
            gltf::animation::Interpolation::Linear => Interpolation::Linear,
            gltf::animation::Interpolation::Step => Interpolation::Step,
            gltf::animation::Interpolation::CubicSpline => Interpolation::CubicSpline,
        };
        let reader = source.reader(|buffer| Some(&buffers[buffer.index()]));
        let Some(times_seconds) = reader.read_inputs().map(Iterator::collect) else {
            continue;
        };
        let Some(outputs) = reader.read_outputs() else {
            continue;
        };
        let raw_values: Vec<_> = match outputs {
            gltf::animation::util::ReadOutputs::Translations(values) => values
                .map(|value| ChannelValues::Translation(Vec3::from(value)))
                .collect(),
            gltf::animation::util::ReadOutputs::Rotations(values) => values
                .into_f32()
                .map(|value| ChannelValues::Rotation(Quat::from_array(value).normalize()))
                .collect(),
            gltf::animation::util::ReadOutputs::Scales(values) => values
                .map(|value| ChannelValues::Scale(Vec3::from(value)))
                .collect(),
            gltf::animation::util::ReadOutputs::MorphTargetWeights(_) => continue,
        };
        let (values, cubic_tangents) = if interpolation == Interpolation::CubicSpline {
            split_cubic_values(raw_values)?
        } else {
            (raw_values, None)
        };
        channels.push(Channel {
            node_index: source.target().node().index(),
            interpolation,
            times_seconds,
            values,
            cubic_tangents,
        });
    }
    Ok(channels)
}

impl Channel {
    /// Samples a single glTF-compatible channel. Malformed empty or mismatched
    /// channels are ignored so the debug viewer remains resilient to bad assets.
    pub fn sample(&self, time_seconds: f32) -> Option<ChannelValues> {
        let count = self.times_seconds.len().min(self.values.len());
        if count == 0 {
            return None;
        }
        let time = time_seconds.max(0.0);
        let right = self.times_seconds[..count].partition_point(|key| *key <= time);
        if right == 0 {
            return Some(self.values[0]);
        }
        if right == count || self.interpolation == Interpolation::Step {
            return Some(self.values[right - 1]);
        }
        let left = right - 1;
        let start = self.times_seconds[left];
        let end = self.times_seconds[right];
        let amount = if end > start {
            ((time - start) / (end - start)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        if self.interpolation == Interpolation::CubicSpline {
            let tangents = self.cubic_tangents.as_ref()?;
            let (_left_in, left_out) = *tangents.get(left)?;
            let (right_in, _right_out) = *tangents.get(right)?;
            let delta_seconds = end - start;
            hermite(
                self.values[left],
                left_out,
                self.values[right],
                right_in,
                amount,
                delta_seconds,
            )
        } else {
            interpolate(self.values[left], self.values[right], amount)
        }
    }
}

fn split_cubic_values(
    raw_values: Vec<ChannelValues>,
) -> Result<
    (
        Vec<ChannelValues>,
        Option<Vec<(ChannelValues, ChannelValues)>>,
    ),
    AssetError,
> {
    if raw_values.len() % 3 != 0 {
        return Err(AssetError::InvalidAnimationData {
            detail: "cubic-spline output count is not three values per keyframe".into(),
        });
    }
    let mut values = Vec::with_capacity(raw_values.len() / 3);
    let mut tangents = Vec::with_capacity(raw_values.len() / 3);
    for triple in raw_values.chunks_exact(3) {
        tangents.push((triple[0], triple[2]));
        values.push(triple[1]);
    }
    Ok((values, Some(tangents)))
}

pub fn sample_pose(channels: &[Channel], time_seconds: f32) -> Pose {
    let mut pose = Pose::default();
    for channel in channels {
        let Some(value) = channel.sample(time_seconds) else {
            continue;
        };
        match value {
            ChannelValues::Translation(translation) => {
                pose.nodes
                    .entry(channel.node_index)
                    .or_default()
                    .translation = translation;
            }
            ChannelValues::Rotation(rotation) => {
                pose.nodes.entry(channel.node_index).or_default().rotation = rotation;
            }
            ChannelValues::Scale(scale) => {
                pose.nodes.entry(channel.node_index).or_default().scale = scale;
            }
            ChannelValues::MorphWeight(weight) => {
                pose.morph_weights.insert(channel.node_index, weight);
            }
        }
    }
    pose
}

fn interpolate(left: ChannelValues, right: ChannelValues, amount: f32) -> Option<ChannelValues> {
    match (left, right) {
        (ChannelValues::Translation(a), ChannelValues::Translation(b)) => {
            Some(ChannelValues::Translation(a.lerp(b, amount)))
        }
        (ChannelValues::Rotation(a), ChannelValues::Rotation(b)) => {
            Some(ChannelValues::Rotation(a.slerp(b, amount)))
        }
        (ChannelValues::Scale(a), ChannelValues::Scale(b)) => {
            Some(ChannelValues::Scale(a.lerp(b, amount)))
        }
        (ChannelValues::MorphWeight(a), ChannelValues::MorphWeight(b)) => {
            Some(ChannelValues::MorphWeight(a + (b - a) * amount))
        }
        _ => None,
    }
}

fn hermite(
    start: ChannelValues,
    start_out_tangent: ChannelValues,
    end: ChannelValues,
    end_in_tangent: ChannelValues,
    amount: f32,
    delta_seconds: f32,
) -> Option<ChannelValues> {
    let t2 = amount * amount;
    let t3 = t2 * amount;
    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + amount;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;
    match (start, start_out_tangent, end, end_in_tangent) {
        (
            ChannelValues::Translation(a),
            ChannelValues::Translation(a_tangent),
            ChannelValues::Translation(b),
            ChannelValues::Translation(b_tangent),
        ) => Some(ChannelValues::Translation(
            a * h00
                + a_tangent * (h10 * delta_seconds)
                + b * h01
                + b_tangent * (h11 * delta_seconds),
        )),
        (
            ChannelValues::Scale(a),
            ChannelValues::Scale(a_tangent),
            ChannelValues::Scale(b),
            ChannelValues::Scale(b_tangent),
        ) => Some(ChannelValues::Scale(
            a * h00
                + a_tangent * (h10 * delta_seconds)
                + b * h01
                + b_tangent * (h11 * delta_seconds),
        )),
        (
            ChannelValues::Rotation(a),
            ChannelValues::Rotation(a_tangent),
            ChannelValues::Rotation(b),
            ChannelValues::Rotation(b_tangent),
        ) => Some(ChannelValues::Rotation(
            Quat::from_xyzw(
                a.x * h00
                    + a_tangent.x * (h10 * delta_seconds)
                    + b.x * h01
                    + b_tangent.x * (h11 * delta_seconds),
                a.y * h00
                    + a_tangent.y * (h10 * delta_seconds)
                    + b.y * h01
                    + b_tangent.y * (h11 * delta_seconds),
                a.z * h00
                    + a_tangent.z * (h10 * delta_seconds)
                    + b.z * h01
                    + b_tangent.z * (h11 * delta_seconds),
                a.w * h00
                    + a_tangent.w * (h10 * delta_seconds)
                    + b.w * h01
                    + b_tangent.w * (h11 * delta_seconds),
            )
            .normalize(),
        )),
        (
            ChannelValues::MorphWeight(a),
            ChannelValues::MorphWeight(a_tangent),
            ChannelValues::MorphWeight(b),
            ChannelValues::MorphWeight(b_tangent),
        ) => Some(ChannelValues::MorphWeight(
            a * h00
                + a_tangent * (h10 * delta_seconds)
                + b * h01
                + b_tangent * (h11 * delta_seconds),
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_translation_interpolates_between_keys() {
        let channel = Channel {
            node_index: 7,
            interpolation: Interpolation::Linear,
            times_seconds: vec![0.0, 2.0],
            values: vec![
                ChannelValues::Translation(Vec3::ZERO),
                ChannelValues::Translation(Vec3::new(4.0, 2.0, 0.0)),
            ],
            cubic_tangents: None,
        };
        let pose = sample_pose(&[channel], 0.5);
        assert_eq!(pose.nodes[&7].translation, Vec3::new(1.0, 0.5, 0.0));
    }

    #[test]
    fn step_channels_hold_the_previous_value() {
        let channel = Channel {
            node_index: 2,
            interpolation: Interpolation::Step,
            times_seconds: vec![0.0, 1.0],
            values: vec![
                ChannelValues::MorphWeight(0.2),
                ChannelValues::MorphWeight(0.9),
            ],
            cubic_tangents: None,
        };
        let pose = sample_pose(&[channel], 0.9);
        assert_eq!(pose.morph_weights[&2], 0.2);
    }

    #[test]
    fn rotation_sampling_uses_a_normalized_slerp() {
        let channel = Channel {
            node_index: 4,
            interpolation: Interpolation::Linear,
            times_seconds: vec![0.0, 1.0],
            values: vec![
                ChannelValues::Rotation(Quat::IDENTITY),
                ChannelValues::Rotation(Quat::from_rotation_y(std::f32::consts::PI)),
            ],
            cubic_tangents: None,
        };
        let pose = sample_pose(&[channel], 0.5);
        assert!(pose.nodes[&4].rotation.is_normalized());
        let facing = pose.nodes[&4].rotation.mul_vec3(Vec3::Z);
        assert!(facing.x.abs() > 0.9999, "facing was {facing:?}");
        assert!(facing.y.abs() < 0.0001 && facing.z.abs() < 0.0001);
    }

    #[test]
    fn cubic_translation_preserves_exported_tangents() {
        let channel = Channel {
            node_index: 3,
            interpolation: Interpolation::CubicSpline,
            times_seconds: vec![0.0, 1.0],
            values: vec![
                ChannelValues::Translation(Vec3::ZERO),
                ChannelValues::Translation(Vec3::new(1.0, 0.0, 0.0)),
            ],
            cubic_tangents: Some(vec![
                (
                    ChannelValues::Translation(Vec3::ZERO),
                    ChannelValues::Translation(Vec3::new(2.0, 0.0, 0.0)),
                ),
                (
                    ChannelValues::Translation(Vec3::ZERO),
                    ChannelValues::Translation(Vec3::ZERO),
                ),
            ]),
        };
        let pose = sample_pose(&[channel], 0.5);
        assert!(pose.nodes[&3]
            .translation
            .abs_diff_eq(Vec3::new(0.75, 0.0, 0.0), 0.0001));
    }

    #[test]
    fn gltf_translation_channel_crosses_the_import_boundary() {
        let directory = tempfile::tempdir().unwrap();
        let gltf_path = directory.path().join("animated.gltf");
        let bin_path = directory.path().join("animated.bin");
        let mut bytes = Vec::new();
        for value in [0.0_f32, 1.0] {
            bytes.extend(value.to_le_bytes());
        }
        for value in [0.0_f32, 0.0, 0.0, 2.0, 0.0, 0.0] {
            bytes.extend(value.to_le_bytes());
        }
        std::fs::write(bin_path, bytes).unwrap();
        std::fs::write(
            &gltf_path,
            r#"{
              "asset":{"version":"2.0"},
              "buffers":[{"uri":"animated.bin","byteLength":32}],
              "bufferViews":[
                {"buffer":0,"byteOffset":0,"byteLength":8},
                {"buffer":0,"byteOffset":8,"byteLength":24}
              ],
              "accessors":[
                {"bufferView":0,"componentType":5126,"count":2,"type":"SCALAR","min":[0],"max":[1]},
                {"bufferView":1,"componentType":5126,"count":2,"type":"VEC3"}
              ],
              "nodes":[{"name":"Nexa_Head"}],
              "animations":[{"samplers":[{"input":0,"output":1,"interpolation":"LINEAR"}],"channels":[{"sampler":0,"target":{"node":0,"path":"translation"}}]}]
            }"#,
        )
        .unwrap();

        let channels = load_supported_channels(&gltf_path, 0).unwrap();
        assert_eq!(channels.len(), 1);
        let pose = sample_pose(&channels, 0.25);
        assert_eq!(pose.nodes[&0].translation, Vec3::new(0.5, 0.0, 0.0));
    }
}
