use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let mut arguments = std::env::args().skip(1);
    let path = arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("usage: nexa-3d-viewer <Nexa.glb>"))?;
    let report = nexa_3d_runtime::asset::inspect(&path)?;
    println!("NEXA-3D asset report:\n{report:#?}");
    if let Err(error) = nexa_3d_runtime::asset::validate_first_model(&report) {
        eprintln!("Acceptance gate warning: {error}");
    }
    if let Some(manifest_path) = arguments.next() {
        let manifest = nexa_3d_runtime::manifest::NexaRuntimeManifest::load(manifest_path)?;
        manifest.validate(&report)?;
        println!("Runtime manifest accepted: {}", manifest.asset_version);
    }
    present(report)
}

#[cfg(feature = "viewer")]
fn present(report: nexa_3d_runtime::asset::AssetReport) -> anyhow::Result<()> {
    nexa_3d_runtime::viewer::run(report)
}

/// The inspection and manifest gates are the point of a headless build; opening
/// a window is not available there.
#[cfg(not(feature = "viewer"))]
fn present(_report: nexa_3d_runtime::asset::AssetReport) -> anyhow::Result<()> {
    Err(anyhow::anyhow!(
        "build with the `viewer` feature to open a window"
    ))
}
