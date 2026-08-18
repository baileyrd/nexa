//! Renderer-independent skinning inputs.
//!
//! A render backend needs two things beyond the mesh itself: where every node
//! sits under the current pose, and the joint matrix palette that maps
//! skin-space vertices into posed space. Both are resolved here without any
//! `wgpu` or `winit` dependency, so headless tests cover the same math the GPU
//! path will consume.

use std::path::Path;

use glam::{Mat4, Quat, Vec3};

use crate::{
    animation::{NodeTransform, Pose},
    asset::AssetError,
};

/// Scene node parents and rest transforms, captured once at import so a sampled
/// pose resolves to world space without reopening the glTF every frame.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NodeHierarchy {
    parents: Vec<Option<usize>>,
    rest: Vec<NodeTransform>,
    /// Node indices ordered so a parent is always visited before its children.
    order: Vec<usize>,
}

/// One glTF skin resolved into the values a joint palette is built from.
#[derive(Debug, Clone, PartialEq)]
pub struct SkinBinding {
    pub name: String,
    /// Node index of each joint, in the skin's own joint order. A vertex's
    /// `JOINTS_0` component indexes into this list, not into the node list.
    pub joint_nodes: Vec<usize>,
    /// `inverseBindMatrices`, one per joint. glTF makes them optional and
    /// defaults them to identity.
    pub inverse_bind_matrices: Vec<Mat4>,
    /// The node the skinned mesh hangs off. Skinned vertex positions are read in
    /// this node's local space, which is what [`SkinBinding::joint_matrices`]
    /// expects. When several nodes share a skin the first one wins; the Nexa
    /// export binds one.
    pub mesh_node: Option<usize>,
}

/// Everything needed to pose a skinned mesh, read in a single glTF import.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SkinRig {
    pub hierarchy: NodeHierarchy,
    pub skins: Vec<SkinBinding>,
}

pub fn load_skin_rig(path: impl AsRef<Path>) -> Result<SkinRig, AssetError> {
    let path = path.as_ref().to_path_buf();
    let (document, buffers, _images) =
        gltf::import(&path).map_err(|source| AssetError::Import { path, source })?;
    let mesh_nodes: Vec<Option<usize>> = document
        .skins()
        .map(|skin| {
            document
                .nodes()
                .find(|node| node.skin().map(|bound| bound.index()) == Some(skin.index()))
                .map(|node| node.index())
        })
        .collect();
    let skins = document
        .skins()
        .map(|skin| SkinBinding {
            name: skin.name().unwrap_or("<unnamed-skin>").to_owned(),
            joint_nodes: skin.joints().map(|joint| joint.index()).collect(),
            inverse_bind_matrices: skin
                .reader(|buffer| Some(&buffers[buffer.index()]))
                .read_inverse_bind_matrices()
                .map(|matrices| {
                    matrices
                        .map(|matrix| Mat4::from_cols_array_2d(&matrix))
                        .collect()
                })
                .unwrap_or_default(),
            mesh_node: mesh_nodes[skin.index()],
        })
        .collect();
    Ok(SkinRig {
        hierarchy: NodeHierarchy::from_document(&document),
        skins,
    })
}

impl NodeHierarchy {
    fn from_document(document: &gltf::Document) -> Self {
        let count = document.nodes().count();
        let mut parents = vec![None; count];
        let mut children = vec![Vec::new(); count];
        let mut rest = vec![NodeTransform::default(); count];
        for node in document.nodes() {
            let (translation, rotation, scale) = node.transform().decomposed();
            rest[node.index()] = NodeTransform {
                translation: Vec3::from(translation),
                rotation: Quat::from_array(rotation).normalize(),
                scale: Vec3::from(scale),
            };
            for child in node.children() {
                parents[child.index()] = Some(node.index());
                children[node.index()].push(child.index());
            }
        }
        // Depth-first from the roots. A malformed cyclic parent chain leaves its
        // nodes unordered rather than spinning here; they resolve to identity.
        let mut order = Vec::with_capacity(count);
        let mut visited = vec![false; count];
        let mut stack: Vec<usize> = (0..count)
            .filter(|index| parents[*index].is_none())
            .collect();
        while let Some(node) = stack.pop() {
            if std::mem::replace(&mut visited[node], true) {
                continue;
            }
            order.push(node);
            stack.extend_from_slice(&children[node]);
        }
        Self {
            parents,
            rest,
            order,
        }
    }

    pub fn node_count(&self) -> usize {
        self.rest.len()
    }

    pub fn parent(&self, node_index: usize) -> Option<usize> {
        self.parents.get(node_index).copied().flatten()
    }

    pub fn rest_transform(&self, node_index: usize) -> Option<NodeTransform> {
        self.rest.get(node_index).copied()
    }

    /// World matrix of every node under `pose`. Components the pose does not
    /// drive keep their exported rest value.
    pub fn world_transforms(&self, pose: &Pose) -> Vec<Mat4> {
        let mut world = vec![Mat4::IDENTITY; self.rest.len()];
        for &node in &self.order {
            let local = pose
                .nodes
                .get(&node)
                .copied()
                .unwrap_or_default()
                .apply_to(self.rest[node]);
            let local = Mat4::from_scale_rotation_translation(
                local.scale,
                local.rotation,
                local.translation,
            );
            world[node] = match self.parents[node] {
                Some(parent) => world[parent] * local,
                None => local,
            };
        }
        world
    }

    /// World matrices with no clip applied.
    pub fn rest_world_transforms(&self) -> Vec<Mat4> {
        self.world_transforms(&Pose::default())
    }
}

impl SkinBinding {
    pub fn joint_count(&self) -> usize {
        self.joint_nodes.len()
    }

    /// The joint matrix palette, `jointWorld * inverseBindMatrix` per joint.
    ///
    /// A vertex in the mesh node's local space is posed by the weighted sum of
    /// the matrices its `JOINTS_0`/`WEIGHTS_0` pairs select, landing directly in
    /// scene space. At the bind pose every entry is the identity, so an
    /// unanimated skin renders exactly as it was exported.
    ///
    /// glTF writes this as `inverse(meshNodeWorld) * jointWorld * inverseBind`
    /// and then has the renderer re-apply the mesh node's model matrix, which
    /// cancels the leading inverse. Emitting scene space skips both halves: the
    /// rendered result is identical, and the viewer needs no per-draw model
    /// matrix. This is why `mesh_node` does not appear below.
    pub fn joint_matrices(&self, world_transforms: &[Mat4]) -> Vec<Mat4> {
        self.joint_nodes
            .iter()
            .enumerate()
            .map(|(joint, node)| {
                let world = world_transforms
                    .get(*node)
                    .copied()
                    .unwrap_or(Mat4::IDENTITY);
                let inverse_bind = self
                    .inverse_bind_matrices
                    .get(joint)
                    .copied()
                    .unwrap_or(Mat4::IDENTITY);
                world * inverse_bind
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::PoseTransform;

    fn translation(x: f32, y: f32, z: f32) -> NodeTransform {
        NodeTransform {
            translation: Vec3::new(x, y, z),
            ..Default::default()
        }
    }

    /// Hips at the origin with a head one metre up, plus a separate mesh node.
    fn two_joint_hierarchy() -> NodeHierarchy {
        NodeHierarchy {
            parents: vec![None, None, Some(1)],
            rest: vec![
                NodeTransform::default(),
                NodeTransform::default(),
                translation(0.0, 1.0, 0.0),
            ],
            order: vec![0, 1, 2],
        }
    }

    fn head_skin() -> SkinBinding {
        SkinBinding {
            name: "Nexa_Rig".to_owned(),
            joint_nodes: vec![1, 2],
            inverse_bind_matrices: vec![
                Mat4::IDENTITY,
                Mat4::from_translation(Vec3::new(0.0, -1.0, 0.0)),
            ],
            mesh_node: Some(0),
        }
    }

    #[test]
    fn child_world_transforms_compose_through_their_parents() {
        let hierarchy = two_joint_hierarchy();
        let mut pose = Pose::default();
        pose.nodes.insert(
            1,
            PoseTransform {
                translation: Some(Vec3::new(3.0, 0.0, 0.0)),
                ..Default::default()
            },
        );
        let world = hierarchy.world_transforms(&pose);
        assert_eq!(
            world[2].transform_point3(Vec3::ZERO),
            Vec3::new(3.0, 1.0, 0.0)
        );
    }

    #[test]
    fn undriven_nodes_keep_their_rest_transform() {
        let world = two_joint_hierarchy().rest_world_transforms();
        assert_eq!(
            world[2].transform_point3(Vec3::ZERO),
            Vec3::new(0.0, 1.0, 0.0)
        );
    }

    #[test]
    fn the_bind_pose_palette_is_all_identity() {
        let world = two_joint_hierarchy().rest_world_transforms();
        for matrix in head_skin().joint_matrices(&world) {
            assert!(
                matrix.abs_diff_eq(Mat4::IDENTITY, 0.0001),
                "bind-pose joint matrix was {matrix:?}"
            );
        }
    }

    #[test]
    fn a_rotated_joint_swings_its_bound_vertex_about_the_joint() {
        let hierarchy = two_joint_hierarchy();
        let mut pose = Pose::default();
        pose.nodes.insert(
            2,
            PoseTransform {
                rotation: Some(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
                ..Default::default()
            },
        );
        let palette = head_skin().joint_matrices(&hierarchy.world_transforms(&pose));
        // A skin-space vertex half a metre above the head joint swings out to
        // the side, keeping its distance from the joint.
        let posed = palette[1].transform_point3(Vec3::new(0.0, 1.5, 0.0));
        assert!(
            posed.abs_diff_eq(Vec3::new(-0.5, 1.0, 0.0), 0.0001),
            "posed vertex was {posed:?}"
        );
    }

    #[test]
    fn the_palette_ignores_the_skinned_mesh_nodes_own_transform() {
        let mut hierarchy = two_joint_hierarchy();
        hierarchy.rest[0] = translation(10.0, 0.0, 0.0);
        let palette = head_skin().joint_matrices(&hierarchy.rest_world_transforms());
        // The joints drive the mesh, not the node it hangs off: displacing that
        // node must leave the bind pose exactly where the joints put it.
        assert!(palette[1].abs_diff_eq(Mat4::IDENTITY, 0.0001));
    }

    #[test]
    fn missing_inverse_bind_matrices_default_to_identity() {
        let skin = SkinBinding {
            inverse_bind_matrices: Vec::new(),
            ..head_skin()
        };
        let palette = skin.joint_matrices(&two_joint_hierarchy().rest_world_transforms());
        assert_eq!(palette.len(), 2);
        assert_eq!(palette[1], Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0)));
    }

    #[test]
    fn a_cyclic_parent_chain_resolves_to_identity_instead_of_hanging() {
        let hierarchy = NodeHierarchy {
            parents: vec![Some(1), Some(0)],
            rest: vec![translation(1.0, 0.0, 0.0), translation(0.0, 1.0, 0.0)],
            order: Vec::new(),
        };
        assert_eq!(hierarchy.rest_world_transforms(), vec![Mat4::IDENTITY; 2]);
    }

    #[test]
    fn a_skinned_gltf_crosses_the_import_boundary() {
        let directory = tempfile::tempdir().unwrap();
        let gltf_path = directory.path().join("rig.gltf");
        let bin_path = directory.path().join("rig.bin");
        let mut bytes = Vec::new();
        // POSITION: one vertex half a metre above the head joint.
        for value in [0.0_f32, 1.5, 0.0] {
            bytes.extend(value.to_le_bytes());
        }
        // JOINTS_0: fully bound to skin joint 1 (the head).
        for value in [1_u16, 0, 0, 0] {
            bytes.extend(value.to_le_bytes());
        }
        // WEIGHTS_0
        for value in [1.0_f32, 0.0, 0.0, 0.0] {
            bytes.extend(value.to_le_bytes());
        }
        // inverseBindMatrices, column-major: identity, then translate(0, -1, 0).
        for value in [
            1.0_f32, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 1.0,
        ] {
            bytes.extend(value.to_le_bytes());
        }
        assert_eq!(bytes.len(), 164);
        std::fs::write(bin_path, bytes).unwrap();
        std::fs::write(
            &gltf_path,
            r#"{
              "asset":{"version":"2.0"},
              "buffers":[{"uri":"rig.bin","byteLength":164}],
              "bufferViews":[
                {"buffer":0,"byteOffset":0,"byteLength":12},
                {"buffer":0,"byteOffset":12,"byteLength":8},
                {"buffer":0,"byteOffset":20,"byteLength":16},
                {"buffer":0,"byteOffset":36,"byteLength":128}
              ],
              "accessors":[
                {"bufferView":0,"componentType":5126,"count":1,"type":"VEC3","min":[0,1.5,0],"max":[0,1.5,0]},
                {"bufferView":1,"componentType":5123,"count":1,"type":"VEC4"},
                {"bufferView":2,"componentType":5126,"count":1,"type":"VEC4"},
                {"bufferView":3,"componentType":5126,"count":2,"type":"MAT4"}
              ],
              "meshes":[{"primitives":[{"attributes":{"POSITION":0,"JOINTS_0":1,"WEIGHTS_0":2}}]}],
              "nodes":[
                {"name":"Nexa_Body","mesh":0,"skin":0},
                {"name":"Hips","children":[2]},
                {"name":"Head","translation":[0,1,0]}
              ],
              "skins":[{"name":"Nexa_Rig","joints":[1,2],"inverseBindMatrices":3,"skeleton":1}],
              "scenes":[{"nodes":[0,1]}],"scene":0
            }"#,
        )
        .unwrap();

        let rig = load_skin_rig(&gltf_path).unwrap();
        assert_eq!(rig.hierarchy.node_count(), 3);
        assert_eq!(rig.hierarchy.parent(2), Some(1));
        assert_eq!(rig.skins.len(), 1);
        let skin = &rig.skins[0];
        assert_eq!(skin.name, "Nexa_Rig");
        assert_eq!(skin.joint_nodes, vec![1, 2]);
        assert_eq!(skin.mesh_node, Some(0));
        assert_eq!(skin.joint_count(), 2);
        assert_eq!(
            skin.inverse_bind_matrices[1],
            Mat4::from_translation(Vec3::new(0.0, -1.0, 0.0))
        );

        let mut pose = Pose::default();
        pose.nodes.insert(
            2,
            PoseTransform {
                rotation: Some(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
                ..Default::default()
            },
        );
        let palette = skin.joint_matrices(&rig.hierarchy.world_transforms(&pose));
        let posed = palette[1].transform_point3(Vec3::new(0.0, 1.5, 0.0));
        assert!(
            posed.abs_diff_eq(Vec3::new(-0.5, 1.0, 0.0), 0.0001),
            "posed vertex was {posed:?}"
        );
    }
}
