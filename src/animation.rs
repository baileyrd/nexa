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
}

/// One glTF transform channel: a single scalar-per-component track bound to a node.
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

/// glTF cubic-spline tangents for a morph channel, laid out exactly like
/// [`MorphChannel::weights`] so a target index selects the same column in each.
#[derive(Debug, Clone, PartialEq)]
pub struct MorphTangents {
    pub in_tangents: Vec<f32>,
    pub out_tangents: Vec<f32>,
}

/// One glTF `weights` channel. A morph channel drives *every* morph target of the
/// node's mesh at once, so each keyframe carries `target_count` weights rather
/// than the single scalar a transform channel carries.
#[derive(Debug, Clone, PartialEq)]
pub struct MorphChannel {
    pub node_index: usize,
    /// Morph targets on the driven mesh. Every keyframe holds this many weights.
    pub target_count: usize,
    pub interpolation: Interpolation,
    pub times_seconds: Vec<f32>,
    /// Row-major `keyframe * target_count + target` weights.
    pub weights: Vec<f32>,
    pub cubic_tangents: Option<MorphTangents>,
}

/// Every sampled channel of one exported animation, split by what it drives.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AnimationClip {
    pub transforms: Vec<Channel>,
    pub morphs: Vec<MorphChannel>,
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

/// A renderer-neutral snapshot of one timeline instant. Morph weights stay keyed
/// by node and ordered by target index, so no per-target weight is collapsed.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Pose {
    pub nodes: BTreeMap<usize, NodeTransform>,
    pub morph_weights: BTreeMap<usize, Vec<f32>>,
}

impl Pose {
    /// Weight of one morph target of one node, or `None` when this pose does not
    /// drive it.
    pub fn morph_weight(&self, node_index: usize, target_index: usize) -> Option<f32> {
        self.morph_weights
            .get(&node_index)
            .and_then(|weights| weights.get(target_index))
            .copied()
    }

    /// Flattens per-node weights onto the shared morph-target slots a renderer
    /// packs its position deltas into; glTF indexes both by target index within
    /// a mesh. Nodes sharing a slot contribute their strongest weight, keeping
    /// the result independent of node ordering.
    pub fn morph_slot_weights(&self, slot_count: usize) -> Vec<f32> {
        let mut slots = vec![0.0_f32; slot_count];
        for weights in self.morph_weights.values() {
            for (slot, weight) in slots.iter_mut().zip(weights) {
                *slot = slot.max(*weight);
            }
        }
        slots
    }
}

/// Reads the selected GLB animation into the renderer-neutral sampler model.
pub fn load_animation_clip(
    path: impl AsRef<std::path::Path>,
    animation_index: usize,
) -> Result<AnimationClip, AssetError> {
    let path = path.as_ref().to_path_buf();
    let (document, buffers, _images) =
        gltf::import(&path).map_err(|source| AssetError::Import { path, source })?;
    let Some(animation) = document.animations().nth(animation_index) else {
        return Ok(AnimationClip::default());
    };
    let mut clip = AnimationClip::default();
    for source in animation.channels() {
        let interpolation = match source.sampler().interpolation() {
            gltf::animation::Interpolation::Linear => Interpolation::Linear,
            gltf::animation::Interpolation::Step => Interpolation::Step,
            gltf::animation::Interpolation::CubicSpline => Interpolation::CubicSpline,
        };
        let node_index = source.target().node().index();
        let reader = source.reader(|buffer| Some(&buffers[buffer.index()]));
        let Some(times_seconds) = reader.read_inputs().map(Iterator::collect::<Vec<f32>>) else {
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
            gltf::animation::util::ReadOutputs::MorphTargetWeights(values) => {
                clip.morphs.push(build_morph_channel(
                    node_index,
                    interpolation,
                    times_seconds,
                    values.into_f32().collect(),
                )?);
                continue;
            }
        };
        let (values, cubic_tangents) = if interpolation == Interpolation::CubicSpline {
            split_cubic_values(raw_values)?
        } else {
            (raw_values, None)
        };
        clip.transforms.push(Channel {
            node_index,
            interpolation,
            times_seconds,
            values,
            cubic_tangents,
        });
    }
    Ok(clip)
}

/// The keyframe span a sample time falls in, resolved once and reused by both
/// the transform and the morph sampler.
enum Segment {
    Hold(usize),
    Blend {
        left: usize,
        right: usize,
        amount: f32,
        delta_seconds: f32,
    },
}

fn locate_segment(
    times_seconds: &[f32],
    count: usize,
    time_seconds: f32,
    interpolation: Interpolation,
) -> Option<Segment> {
    if count == 0 {
        return None;
    }
    let time = time_seconds.max(0.0);
    let right = times_seconds[..count].partition_point(|key| *key <= time);
    if right == 0 {
        return Some(Segment::Hold(0));
    }
    if right == count || interpolation == Interpolation::Step {
        return Some(Segment::Hold(right - 1));
    }
    let left = right - 1;
    let start = times_seconds[left];
    let delta_seconds = times_seconds[right] - start;
    let amount = if delta_seconds > 0.0 {
        ((time - start) / delta_seconds).clamp(0.0, 1.0)
    } else {
        0.0
    };
    Some(Segment::Blend {
        left,
        right,
        amount,
        delta_seconds,
    })
}

impl Channel {
    /// Samples a single glTF-compatible channel. Malformed empty or mismatched
    /// channels are ignored so the debug viewer remains resilient to bad assets.
    pub fn sample(&self, time_seconds: f32) -> Option<ChannelValues> {
        let count = self.times_seconds.len().min(self.values.len());
        match locate_segment(&self.times_seconds, count, time_seconds, self.interpolation)? {
            Segment::Hold(index) => Some(self.values[index]),
            Segment::Blend {
                left,
                right,
                amount,
                delta_seconds,
            } => {
                if self.interpolation == Interpolation::CubicSpline {
                    let tangents = self.cubic_tangents.as_ref()?;
                    let (_left_in, left_out) = *tangents.get(left)?;
                    let (right_in, _right_out) = *tangents.get(right)?;
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
    }
}

impl MorphChannel {
    /// Keyframes actually backed by a complete row of weights. Truncated exports
    /// lose their trailing partial row rather than the whole channel.
    fn keyframe_count(&self) -> usize {
        if self.target_count == 0 {
            return 0;
        }
        self.times_seconds
            .len()
            .min(self.weights.len() / self.target_count)
    }

    fn keyframe(&self, index: usize) -> &[f32] {
        let start = index * self.target_count;
        &self.weights[start..start + self.target_count]
    }

    fn tangent_row(row: &[f32], index: usize, target_count: usize) -> Option<&[f32]> {
        let start = index * target_count;
        row.get(start..start + target_count)
    }

    /// Samples every target weight into `weights`, replacing its contents.
    /// Returns `false` (leaving `weights` untouched) for empty or malformed
    /// channels, matching [`Channel::sample`]'s tolerance of bad assets.
    pub fn sample_into(&self, time_seconds: f32, weights: &mut Vec<f32>) -> bool {
        let count = self.keyframe_count();
        let Some(segment) =
            locate_segment(&self.times_seconds, count, time_seconds, self.interpolation)
        else {
            return false;
        };
        match segment {
            Segment::Hold(index) => {
                weights.clear();
                weights.extend_from_slice(self.keyframe(index));
            }
            Segment::Blend {
                left,
                right,
                amount,
                delta_seconds,
            } => {
                if self.interpolation == Interpolation::CubicSpline {
                    let Some(tangents) = self.cubic_tangents.as_ref() else {
                        return false;
                    };
                    let (Some(left_out), Some(right_in)) = (
                        Self::tangent_row(&tangents.out_tangents, left, self.target_count),
                        Self::tangent_row(&tangents.in_tangents, right, self.target_count),
                    ) else {
                        return false;
                    };
                    let (start, end) = (self.keyframe(left), self.keyframe(right));
                    let basis = hermite_basis(amount, delta_seconds);
                    weights.clear();
                    weights.extend((0..self.target_count).map(|target| {
                        hermite_scalar(
                            start[target],
                            left_out[target],
                            end[target],
                            right_in[target],
                            basis,
                        )
                    }));
                } else {
                    let (start, end) = (self.keyframe(left), self.keyframe(right));
                    weights.clear();
                    weights.extend(
                        start
                            .iter()
                            .zip(end)
                            .map(|(start, end)| start + (end - start) * amount),
                    );
                }
            }
        }
        true
    }

    /// Convenience wrapper over [`MorphChannel::sample_into`] for callers that do
    /// not keep a reusable buffer.
    pub fn sample(&self, time_seconds: f32) -> Option<Vec<f32>> {
        let mut weights = Vec::with_capacity(self.target_count);
        self.sample_into(time_seconds, &mut weights)
            .then_some(weights)
    }
}

impl AnimationClip {
    pub fn is_empty(&self) -> bool {
        self.transforms.is_empty() && self.morphs.is_empty()
    }

    /// Last keyframe time across every channel of the clip.
    pub fn duration_seconds(&self) -> f32 {
        let transforms = self
            .transforms
            .iter()
            .filter_map(|channel| channel.times_seconds.last());
        let morphs = self
            .morphs
            .iter()
            .filter_map(|channel| channel.times_seconds.last());
        transforms
            .chain(morphs)
            .fold(0.0_f32, |end, time| end.max(*time))
    }

    /// Samples the whole clip, preserving each node's full per-target weight set.
    pub fn sample_pose(&self, time_seconds: f32) -> Pose {
        let mut pose = sample_pose(&self.transforms, time_seconds);
        let mut weights = Vec::new();
        for channel in &self.morphs {
            if channel.sample_into(time_seconds, &mut weights) {
                pose.morph_weights
                    .insert(channel.node_index, std::mem::take(&mut weights));
            }
        }
        pose
    }
}

fn build_morph_channel(
    node_index: usize,
    interpolation: Interpolation,
    times_seconds: Vec<f32>,
    raw_weights: Vec<f32>,
) -> Result<MorphChannel, AssetError> {
    let keyframes = times_seconds.len();
    let stride = if interpolation == Interpolation::CubicSpline {
        3
    } else {
        1
    };
    let row = keyframes * stride;
    if row == 0 || raw_weights.is_empty() {
        return Ok(MorphChannel {
            node_index,
            target_count: 0,
            interpolation,
            times_seconds,
            weights: Vec::new(),
            cubic_tangents: None,
        });
    }
    if !raw_weights.len().is_multiple_of(row) {
        return Err(AssetError::InvalidAnimationData {
            detail: format!(
                "morph weight output count {} does not divide into {keyframes} keyframes",
                raw_weights.len()
            ),
        });
    }
    let target_count = raw_weights.len() / row;
    let (weights, cubic_tangents) = if stride == 3 {
        let mut values = Vec::with_capacity(keyframes * target_count);
        let mut in_tangents = Vec::with_capacity(keyframes * target_count);
        let mut out_tangents = Vec::with_capacity(keyframes * target_count);
        for block in raw_weights.chunks_exact(3 * target_count) {
            in_tangents.extend_from_slice(&block[..target_count]);
            values.extend_from_slice(&block[target_count..target_count * 2]);
            out_tangents.extend_from_slice(&block[target_count * 2..]);
        }
        (
            values,
            Some(MorphTangents {
                in_tangents,
                out_tangents,
            }),
        )
    } else {
        (raw_weights, None)
    };
    Ok(MorphChannel {
        node_index,
        target_count,
        interpolation,
        times_seconds,
        weights,
        cubic_tangents,
    })
}

/// Keyframe values paired with their optional cubic `(in, out)` tangents.
type SplitCubicValues = (
    Vec<ChannelValues>,
    Option<Vec<(ChannelValues, ChannelValues)>>,
);

fn split_cubic_values(raw_values: Vec<ChannelValues>) -> Result<SplitCubicValues, AssetError> {
    if !raw_values.len().is_multiple_of(3) {
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

/// Samples transform channels only; morph weights arrive through
/// [`AnimationClip::sample_pose`].
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
        _ => None,
    }
}

/// Hermite basis weights `(h00, h10, h01, h11)` with the tangent terms already
/// scaled by the keyframe delta, as glTF exports tangents per second.
type HermiteBasis = (f32, f32, f32, f32);

fn hermite_basis(amount: f32, delta_seconds: f32) -> HermiteBasis {
    let t2 = amount * amount;
    let t3 = t2 * amount;
    (
        2.0 * t3 - 3.0 * t2 + 1.0,
        (t3 - 2.0 * t2 + amount) * delta_seconds,
        -2.0 * t3 + 3.0 * t2,
        (t3 - t2) * delta_seconds,
    )
}

fn hermite_scalar(
    start: f32,
    start_out_tangent: f32,
    end: f32,
    end_in_tangent: f32,
    (h00, h10, h01, h11): HermiteBasis,
) -> f32 {
    start * h00 + start_out_tangent * h10 + end * h01 + end_in_tangent * h11
}

fn hermite(
    start: ChannelValues,
    start_out_tangent: ChannelValues,
    end: ChannelValues,
    end_in_tangent: ChannelValues,
    amount: f32,
    delta_seconds: f32,
) -> Option<ChannelValues> {
    let basis = hermite_basis(amount, delta_seconds);
    let (h00, h10, h01, h11) = basis;
    match (start, start_out_tangent, end, end_in_tangent) {
        (
            ChannelValues::Translation(a),
            ChannelValues::Translation(a_tangent),
            ChannelValues::Translation(b),
            ChannelValues::Translation(b_tangent),
        ) => Some(ChannelValues::Translation(
            a * h00 + a_tangent * h10 + b * h01 + b_tangent * h11,
        )),
        (
            ChannelValues::Scale(a),
            ChannelValues::Scale(a_tangent),
            ChannelValues::Scale(b),
            ChannelValues::Scale(b_tangent),
        ) => Some(ChannelValues::Scale(
            a * h00 + a_tangent * h10 + b * h01 + b_tangent * h11,
        )),
        (
            ChannelValues::Rotation(a),
            ChannelValues::Rotation(a_tangent),
            ChannelValues::Rotation(b),
            ChannelValues::Rotation(b_tangent),
        ) => Some(ChannelValues::Rotation(
            Quat::from_xyzw(
                hermite_scalar(a.x, a_tangent.x, b.x, b_tangent.x, basis),
                hermite_scalar(a.y, a_tangent.y, b.y, b_tangent.y, basis),
                hermite_scalar(a.z, a_tangent.z, b.z, b_tangent.z, basis),
                hermite_scalar(a.w, a_tangent.w, b.w, b_tangent.w, basis),
            )
            .normalize(),
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn linear_morph_channel() -> MorphChannel {
        MorphChannel {
            node_index: 2,
            target_count: 3,
            interpolation: Interpolation::Linear,
            times_seconds: vec![0.0, 1.0],
            weights: vec![0.0, 0.5, 1.0, 1.0, 0.5, 0.0],
            cubic_tangents: None,
        }
    }

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
                ChannelValues::Translation(Vec3::ZERO),
                ChannelValues::Translation(Vec3::X),
            ],
            cubic_tangents: None,
        };
        let pose = sample_pose(&[channel], 0.9);
        assert_eq!(pose.nodes[&2].translation, Vec3::ZERO);
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
    fn every_morph_target_keeps_its_own_interpolated_weight() {
        let sampled = linear_morph_channel().sample(0.25).unwrap();
        assert_eq!(sampled, vec![0.25, 0.5, 0.75]);
    }

    #[test]
    fn morph_step_channels_hold_the_whole_previous_keyframe() {
        let mut channel = linear_morph_channel();
        channel.interpolation = Interpolation::Step;
        assert_eq!(channel.sample(0.9).unwrap(), vec![0.0, 0.5, 1.0]);
    }

    #[test]
    fn morph_sampling_clamps_to_the_first_and_last_keyframe() {
        let channel = linear_morph_channel();
        assert_eq!(channel.sample(-5.0).unwrap(), vec![0.0, 0.5, 1.0]);
        assert_eq!(channel.sample(9.0).unwrap(), vec![1.0, 0.5, 0.0]);
    }

    #[test]
    fn cubic_morph_channels_apply_per_target_tangents() {
        let channel = MorphChannel {
            node_index: 5,
            target_count: 2,
            interpolation: Interpolation::CubicSpline,
            times_seconds: vec![0.0, 1.0],
            weights: vec![0.0, 0.0, 1.0, 1.0],
            cubic_tangents: Some(MorphTangents {
                in_tangents: vec![0.0, 0.0, 0.0, 0.0],
                out_tangents: vec![2.0, 0.0, 0.0, 0.0],
            }),
        };
        let sampled = channel.sample(0.5).unwrap();
        // Target 0 carries an out-tangent, target 1 does not: the same keyframe
        // pair must still produce two different curves.
        assert!((sampled[0] - 0.75).abs() < 0.0001, "{sampled:?}");
        assert!((sampled[1] - 0.5).abs() < 0.0001, "{sampled:?}");
    }

    #[test]
    fn malformed_morph_channels_sample_to_nothing() {
        let empty = MorphChannel {
            node_index: 0,
            target_count: 0,
            interpolation: Interpolation::Linear,
            times_seconds: vec![0.0],
            weights: Vec::new(),
            cubic_tangents: None,
        };
        assert!(empty.sample(0.0).is_none());
        let missing_tangents = MorphChannel {
            interpolation: Interpolation::CubicSpline,
            cubic_tangents: None,
            ..linear_morph_channel()
        };
        assert!(missing_tangents.sample(0.5).is_none());
    }

    #[test]
    fn clip_poses_carry_transforms_and_per_target_weights_together() {
        let clip = AnimationClip {
            transforms: vec![Channel {
                node_index: 1,
                interpolation: Interpolation::Linear,
                times_seconds: vec![0.0, 4.0],
                values: vec![
                    ChannelValues::Translation(Vec3::ZERO),
                    ChannelValues::Translation(Vec3::new(4.0, 0.0, 0.0)),
                ],
                cubic_tangents: None,
            }],
            morphs: vec![linear_morph_channel()],
        };
        assert_eq!(clip.duration_seconds(), 4.0);
        let pose = clip.sample_pose(0.5);
        assert_eq!(pose.nodes[&1].translation, Vec3::new(0.5, 0.0, 0.0));
        assert_eq!(pose.morph_weights[&2], vec![0.5, 0.5, 0.5]);
        assert_eq!(pose.morph_weight(2, 1), Some(0.5));
        assert_eq!(pose.morph_weight(2, 3), None);
        assert_eq!(pose.morph_weight(9, 0), None);
    }

    #[test]
    fn morph_slots_take_the_strongest_weight_regardless_of_node_order() {
        let mut pose = Pose::default();
        pose.morph_weights.insert(4, vec![0.2, 0.9, 0.0]);
        pose.morph_weights.insert(1, vec![0.7, 0.1]);
        assert_eq!(pose.morph_slot_weights(3), vec![0.7, 0.9, 0.0]);
        // A renderer with fewer delta slots than the clip drives is truncated,
        // never panicked.
        assert_eq!(pose.morph_slot_weights(2), vec![0.7, 0.9]);
        assert!(pose.morph_slot_weights(0).is_empty());
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

        let clip = load_animation_clip(&gltf_path, 0).unwrap();
        assert_eq!(clip.transforms.len(), 1);
        assert!(clip.morphs.is_empty());
        let pose = clip.sample_pose(0.25);
        assert_eq!(pose.nodes[&0].translation, Vec3::new(0.5, 0.0, 0.0));
    }

    /// Two morph targets driven by one `weights` channel: the weights must stay
    /// separated by target index all the way from the glTF buffer to the pose.
    #[test]
    fn gltf_multi_target_weights_survive_the_import_boundary() {
        let directory = tempfile::tempdir().unwrap();
        let gltf_path = directory.path().join("face.gltf");
        let bin_path = directory.path().join("face.bin");
        let mut bytes = Vec::new();
        for value in [0.0_f32, 1.0] {
            bytes.extend(value.to_le_bytes());
        }
        // Keyframe-major weights: (target_0, target_1) per keyframe.
        for value in [0.0_f32, 0.0, 1.0, 0.5] {
            bytes.extend(value.to_le_bytes());
        }
        // Base position, then one POSITION delta per morph target.
        for value in [0.0_f32, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0] {
            bytes.extend(value.to_le_bytes());
        }
        assert_eq!(bytes.len(), 60);
        std::fs::write(bin_path, bytes).unwrap();
        std::fs::write(
            &gltf_path,
            r#"{
              "asset":{"version":"2.0"},
              "buffers":[{"uri":"face.bin","byteLength":60}],
              "bufferViews":[
                {"buffer":0,"byteOffset":0,"byteLength":8},
                {"buffer":0,"byteOffset":8,"byteLength":16},
                {"buffer":0,"byteOffset":24,"byteLength":12},
                {"buffer":0,"byteOffset":36,"byteLength":12},
                {"buffer":0,"byteOffset":48,"byteLength":12}
              ],
              "accessors":[
                {"bufferView":0,"componentType":5126,"count":2,"type":"SCALAR","min":[0],"max":[1]},
                {"bufferView":1,"componentType":5126,"count":4,"type":"SCALAR"},
                {"bufferView":2,"componentType":5126,"count":1,"type":"VEC3","min":[0,0,0],"max":[0,0,0]},
                {"bufferView":3,"componentType":5126,"count":1,"type":"VEC3","min":[0,1,0],"max":[0,1,0]},
                {"bufferView":4,"componentType":5126,"count":1,"type":"VEC3","min":[0,0,1],"max":[0,0,1]}
              ],
              "meshes":[{"primitives":[{"attributes":{"POSITION":2},"targets":[{"POSITION":3},{"POSITION":4}]}],"weights":[0,0]}],
              "nodes":[{"name":"Nexa_Face","mesh":0}],
              "scenes":[{"nodes":[0]}],"scene":0,
              "animations":[{"samplers":[{"input":0,"output":1,"interpolation":"LINEAR"}],"channels":[{"sampler":0,"target":{"node":0,"path":"weights"}}]}]
            }"#,
        )
        .unwrap();

        let clip = load_animation_clip(&gltf_path, 0).unwrap();
        assert!(clip.transforms.is_empty());
        assert_eq!(clip.morphs.len(), 1);
        assert_eq!(clip.morphs[0].target_count, 2);
        assert_eq!(clip.duration_seconds(), 1.0);
        let pose = clip.sample_pose(0.5);
        assert_eq!(pose.morph_weights[&0], vec![0.5, 0.25]);
    }

    #[test]
    fn ragged_morph_weight_output_is_rejected_at_import() {
        let error = build_morph_channel(
            0,
            Interpolation::Linear,
            vec![0.0, 1.0, 2.0],
            vec![0.0, 1.0, 0.0, 1.0],
        )
        .unwrap_err();
        assert!(matches!(error, AssetError::InvalidAnimationData { .. }));
    }
}
