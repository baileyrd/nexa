# G1 shared UI/loopback suitability spike

Disposable evidence code for Issue #116. It is not a production application or an architecture selection. General implementation remains under **Tactical Pause**, G1 remains **blocked**, and G2+ are not dispatched.

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

The script uses both committed npm lockfiles and Cargo lockfile. It performs frontend `npm ci`, lint/typecheck, component tests and production build; runtime format/check/tests; desktop `npm ci`; a Tauri 2 Windows build/NSIS package; and package-size capture. It writes deterministic machine-readable pass/fail JSON to `evidence/windows-validation.json` (or `-OutputPath <path>`) and exits nonzero when any step fails.

The harness is reproducible infrastructure, **not an executed Windows result**. It does not prove interactive same-machine browser/WebView parity, accessibility, startup, idle/active CPU or memory, reconnect timing, or cancellation timing. G1 stays blocked until those representative Windows observations and the script output are captured and reviewed on an exact PR head.

## Runtime and trust boundary

The runtime listens on `127.0.0.1:43116` only. The deterministic development token is supplied explicitly to both clients; no real learner data or secrets are used. The allowlist includes the browser preview at `http://127.0.0.1:4173` and Tauri 2's packaged Windows origin at `http://tauri.localhost`. The desktop shell loads the exact frontend build and defines no Tauri commands or desktop business logic.
