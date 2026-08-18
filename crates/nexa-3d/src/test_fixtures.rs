//! Synthetic glTF fixtures shared by the module tests.
//!
//! These are pipeline-contract fixtures only. The approved Nexa turnaround,
//! expression, viseme, hand, and gesture sheets remain the visual authority.

use std::path::{Path, PathBuf};

/// A one-vertex mesh bound to a two-joint rig, written next to its buffer.
///
/// The mesh node carries its own translation so callers can check that skinned
/// positions are imported in mesh-node-local space rather than baked to world
/// space. The head joint sits one metre above the hips, the single vertex sits
/// half a metre above the head joint, and the inverse bind matrices put the
/// whole rig at its bind pose.
pub fn skinned_rig_gltf(directory: &Path) -> PathBuf {
    let gltf_path = directory.join("rig.gltf");
    let mut bytes = Vec::new();
    // POSITION
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
        1.0_f32, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0,
        0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 1.0,
    ] {
        bytes.extend(value.to_le_bytes());
    }
    assert_eq!(bytes.len(), 164);
    std::fs::write(directory.join("rig.bin"), bytes).unwrap();
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
            {"name":"Nexa_Body","mesh":0,"skin":0,"translation":[10,0,0]},
            {"name":"Hips","children":[2]},
            {"name":"Head","translation":[0,1,0]}
          ],
          "skins":[{"name":"Nexa_Rig","joints":[1,2],"inverseBindMatrices":3,"skeleton":1}],
          "scenes":[{"nodes":[0,1]}],"scene":0
        }"#,
    )
    .unwrap();
    gltf_path
}
