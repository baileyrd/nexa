# ADR-0008: Controlled 3D workspace migration and ownership

- Status: Accepted
- Date: 2026-08-18
- Governing decisions: ADR-0001, ADR-0007
- Governing specifications: NEXA-3D-001, NEXA-3D-RUNTIME-001

## Context

The original root package combined a reusable, renderer-independent runtime, two command-line entry
points, and the `wgpu`/`winit` viewer. ADR-0001 reserved monorepo boundaries and required a dedicated,
behavior-preserving move. ADR-0007 established that `nexa-avatar`, not a concrete renderer, owns the
semantic avatar port.

## Decision

Ownership after the move is:

- `crates/nexa-3d` owns reusable asset inspection, animation, skinning, runtime manifests, headless
  validation, renderer-independent controls, and the `nexa-avatar` adapter. Its package is `nexa-3d`;
  its library target remains `nexa_3d_runtime` so existing Rust imports remain source-compatible.
- `apps/nexa-3d-viewer` owns the `nexa-3d-viewer` binary, `wgpu`/`winit` composition, OS input,
  surface lifecycle, and WGSL presentation shader.
- `tools/nexa-3d-validate` owns the `nexa-3d-validate` binary. It is a production/CI asset validation
  tool rather than an end-user application or reusable library API. Its thin entry point depends only
  on `nexa-3d`, `anyhow`, and JSON serialization and cannot initialize a GPU, window, audio, or input.

The library has no viewer feature and no renderer dependencies. This is stricter than the former
optional `viewer` feature: `cargo check -p nexa-3d --no-default-features` is always headless, while
viewer construction is possible only through the application package. Binary names, arguments,
validation semantics, public modules, runtime assets, and semantic adapter behavior are unchanged.

## Migration map

| Old path | New path | Owner |
|---|---|---|
| `Cargo.toml` package sections | `crates/nexa-3d/Cargo.toml`, `apps/nexa-3d-viewer/Cargo.toml`, `tools/nexa-3d-validate/Cargo.toml` | workspace/library/app/tool |
| `src/lib.rs` | `crates/nexa-3d/src/lib.rs` | library |
| `src/animation.rs` | `crates/nexa-3d/src/animation.rs` | library |
| `src/asset.rs` | `crates/nexa-3d/src/asset.rs` | library |
| `src/avatar.rs` | `crates/nexa-3d/src/avatar.rs` | library adapter |
| `src/behavior.rs` | `crates/nexa-3d/src/behavior.rs` | library |
| `src/control.rs` | `crates/nexa-3d/src/control.rs` | renderer-neutral library controls |
| `src/gaze.rs` | `crates/nexa-3d/src/gaze.rs` | library |
| `src/headless.rs` | `crates/nexa-3d/src/headless.rs` | library |
| `src/manifest.rs` | `crates/nexa-3d/src/manifest.rs` | library |
| `src/runtime.rs` | `crates/nexa-3d/src/runtime.rs` | library |
| `src/skin.rs` | `crates/nexa-3d/src/skin.rs` | library |
| `src/viseme.rs` | `crates/nexa-3d/src/viseme.rs` | library |
| `src/test_fixtures.rs` | `crates/nexa-3d/src/test_fixtures.rs` | library tests |
| `src/main.rs` | `apps/nexa-3d-viewer/src/main.rs` | viewer app |
| `src/viewer.rs` | `apps/nexa-3d-viewer/src/viewer.rs` | viewer app |
| `src/static_mesh.wgsl` | `apps/nexa-3d-viewer/src/static_mesh.wgsl` | viewer app |
| `src/bin/nexa-3d-validate.rs` | `tools/nexa-3d-validate/src/main.rs` | validation tool |

Repository-level `assets/`, specifications, canonical PNG/JPEG references, and runtime example
manifests do not move. Their paths and bytes remain canonical.

## Compatibility

`cargo run --bin nexa-3d-viewer -- ...` and `cargo run --bin nexa-3d-validate -- ...` continue to
resolve from the workspace root. Downstream Cargo dependencies must change their package path from the
repository root to `crates/nexa-3d` and the package name from `nexa-3d-runtime` to `nexa-3d`; Rust code
may continue importing `nexa_3d_runtime`. The old `viewer` feature is removed because renderer code is
now an application boundary; consumers that embedded `nexa_3d_runtime::viewer` must compose or invoke
the viewer application instead.

## Consequences

Feature unification can no longer pull windowing or GPU crates into the headless library or validator.
The shader validation tests move with the viewer. No renderer feature, rendering behavior, canonical
asset, or reconstructed specification meaning is changed. Future renderers belong in application or
adapter packages and must continue implementing contracts owned by `nexa-avatar`.
