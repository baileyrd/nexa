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
    #[cfg(feature = "viewer")]
    nexa_3d_runtime::viewer::run(report)?;
    #[cfg(not(feature = "viewer"))]
    {
        let _ = report;
        return Err(anyhow::anyhow!(
            "build with the `viewer` feature to open a window"
        ));
    }
    Ok(())
}
