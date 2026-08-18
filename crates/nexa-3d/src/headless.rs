use crate::asset::{
    inspect, load_static_geometry, validate_first_model, validate_render_geometry, AssetError,
    AssetReport,
};
use std::path::Path;

/// The CI entry point: no GPU, window, audio device, or event loop is created.
pub fn validate_glb(path: impl AsRef<Path>) -> Result<AssetReport, AssetError> {
    let report = inspect(path)?;
    validate_first_model(&report)?;
    let geometry = load_static_geometry(&report.source)?;
    validate_render_geometry(&geometry)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_minimal_asset_passes_without_a_gpu() {
        let directory = tempfile::tempdir().unwrap();
        let gltf_path = directory.path().join("fixture.gltf");
        let bin_path = directory.path().join("fixture.bin");

        let mut bytes = Vec::new();
        for value in [0.0_f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0] {
            bytes.extend(value.to_le_bytes());
        }
        for index in [0_u16, 1, 2] {
            bytes.extend(index.to_le_bytes());
        }
        // A valid, intentionally neutral POSITION morph target.
        for _ in 0..9 {
            bytes.extend(0.0_f32.to_le_bytes());
        }
        std::fs::write(bin_path, bytes).unwrap();
        std::fs::write(
            &gltf_path,
            r#"{
              "asset":{"version":"2.0"},
              "buffers":[{"uri":"fixture.bin","byteLength":78}],
              "bufferViews":[
                {"buffer":0,"byteOffset":0,"byteLength":36},
                {"buffer":0,"byteOffset":36,"byteLength":6},
                {"buffer":0,"byteOffset":42,"byteLength":36}
              ],
              "accessors":[
                {"bufferView":0,"componentType":5126,"count":3,"type":"VEC3","min":[0,0,0],"max":[1,1,0]},
                {"bufferView":1,"componentType":5123,"count":3,"type":"SCALAR"},
                {"bufferView":2,"componentType":5126,"count":3,"type":"VEC3"}
              ],
              "meshes":[{"name":"Nexa_Body","primitives":[{"attributes":{"POSITION":0},"indices":1,"targets":[{"POSITION":2}]}]}],
              "nodes":[{"name":"Nexa_Rig"},{"name":"Nexa_Body","mesh":0,"skin":0}],
              "skins":[{"name":"Nexa_Rig","joints":[0],"skeleton":0}],
              "scenes":[{"nodes":[1]}],"scene":0
            }"#,
        ).unwrap();

        let report = validate_glb(&gltf_path).unwrap();
        assert_eq!(report.skins[0].joint_names, ["Nexa_Rig"]);
        assert_eq!(report.morph_target_count, 1);
    }
}
