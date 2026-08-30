# G1 shared UI/loopback suitability spike

Disposable evidence code for Issue #116. It is not a production application or an architecture selection. General implementation remains under **Tactical Pause**, G1 remains **blocked**, and G2+ are not dispatched.

The desktop bundle uses the repository-owned placeholder at `desktop/src-tauri/icons/icon.ico` solely to satisfy Tauri's Windows resource generation. It is spike infrastructure, not production branding. A representative Windows harness run against merged `main` at `38bb0e71cda177c7ae9dc0afb81118ba4ee744c4` passed every preceding dependency and validation step, then exposed the missing default icon during Tauri packaging. The placeholder corrects that packaging prerequisite; it is not Windows success evidence, and the owner must rerun the harness after this correction merges.

## Linux/headless validation available now

Run these commands from a clean checkout at the repository root:

```bash
npm --prefix spikes/g1-shared-ui/frontend ci
npm --prefix spikes/g1-shared-ui/frontend run check
npm --prefix spikes/g1-shared-ui/frontend test
npm --prefix spikes/g1-shared-ui/frontend run build
cargo fmt --manifest-path spikes/g1-shared-ui/runtime/Cargo.toml --check
cargo check --locked --manifest-path spikes/g1-shared-ui/runtime/Cargo.toml
cargo test --locked --manifest-path spikes/g1-shared-ui/runtime/Cargo.toml
npm --prefix spikes/g1-shared-ui/desktop ci
npm --prefix spikes/g1-shared-ui/desktop run tauri -- build
```

The current Linux executor automates the frontend lint/typecheck, component lifecycle tests, production build, and runtime format/check/end-to-end HTTP/WebSocket tests. On Linux, the final Tauri package command verifies the corrected sibling-frontend build command before stopping when host WebKit/GLib packaging prerequisites are unavailable; that is not Windows evidence.

## Clean Windows validation harness

On a representative CPU-only Windows PC with supported Node.js, Rust, Microsoft C++ Build Tools, and WebView2 prerequisites installed, run from any directory:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File spikes/g1-shared-ui/scripts/validate-windows.ps1
```

The script uses both committed npm lockfiles and Cargo lockfile. It performs frontend `npm ci`, lint/typecheck, component tests and production build; runtime format/check/tests; desktop `npm ci`; a Tauri 2 Windows build/NSIS package; and package-size capture. It writes the exact repository head, tool versions, step outcomes and durations, package sizes, and overall pass/fail status as machine-readable JSON to `evidence/windows-validation.json` (or `-OutputPath <path>`) and exits nonzero when any step fails.

The harness is reproducible infrastructure, **not a successful Windows result**. Its first representative execution exposed the now-corrected missing-icon packaging prerequisite, and it must be rerun after this correction merges. It does not prove interactive same-machine browser/WebView parity, accessibility, startup, idle/active CPU or memory, reconnect timing, or cancellation timing. G1 stays blocked until a successful rerun and those representative Windows observations are captured and reviewed on an exact head.

## Runtime and trust boundary

The runtime listens on `127.0.0.1:43116` only. The deterministic development token is supplied explicitly to both clients; no real learner data or secrets are used. The allowlist includes the browser preview at `http://127.0.0.1:4173` and Tauri 2's packaged Windows origin at `http://tauri.localhost`. The desktop shell loads the exact frontend build and defines no Tauri commands or desktop business logic.
