# G1 evidence record

Date: 2026-08-30. Issues: #116 and #122. Original spike base: `0b29eca746090334825bdfae4e71b5eb3e0aa042`. Representative Windows evidence head: `9173ed152d7b1d5bb9831413862097076443be59` (exact current `main` after PR #121). Candidate versions are locked by committed Cargo/npm lockfiles. This is evidence-only disposable code, not production selection.

## Environment and method

The available executor was Ubuntu 24.04 x86_64, not Windows. Exact tool versions are in `environment.txt`. Linux frontend wall time was captured with the available shell clock and maximum RSS is explicitly unavailable in `linux-build-time.txt`; sizes are in `linux-artifact-sizes.txt`. These observations are diagnostic only. Automated local evidence covers frontend lint/typecheck, lifecycle/component tests and production build plus runtime format/check and end-to-end HTTP/WebSocket tests. Issue #120 adds a bounded, explicitly selected 60-second hold mode so a human can observe running and request cancellation; normal requests remain fast. Tests drive the runtime hold clock deterministically and drive the frontend acknowledgement directly rather than sleeping.

The repository-local `scripts/validate-windows.ps1` harness performs clean locked installs, the same frontend/runtime checks, a Tauri 2 Windows NSIS build, and package-size capture with machine-readable pass/fail JSON that records the exact repository head and tool versions. The earlier failed packaging run at `38bb0e71cda177c7ae9dc0afb81118ba4ee744c4` remains historical evidence of the icon prerequisite corrected by Issue #118; it is not counted as success evidence.

### Automated exact-head Windows harness evidence

The owner supplied the harness result from exact head `9173ed152d7b1d5bb9831413862097076443be59`: overall **PASS** on Microsoft Windows NT 10.0.26200.0 x64 with Node v22.22.3, npm 12.0.2, rustc 1.97.1, and cargo 1.97.1. The harness reported:

- frontend locked install, check, seven component tests, and build: **PASS**;
- runtime format, check, and five test groups: **PASS**;
- desktop locked install: **PASS**;
- Tauri Windows release build and NSIS packaging: **PASS**;
- installer: `desktop/src-tauri/target/release/bundle/nsis/Nexa G1 Spike_0.0.0_x64-setup.exe`;
- installer size: **1,848,941 bytes**.

These are machine-produced harness results as reported for the exact head. This repository record does not add a screenshot or claim an independently reproduced second run.

### Owner-observed representative interaction evidence

On the same current-main generation, the owner observed the same shared UI rendering equivalently in the browser preview and packaged Tauri shell. Both reached `ready / Event connection ready.` and completed the normal fixture identically at `success / Deterministic fixture complete.` Both showed `loading / Running held fixture. Press Cancel.` in the Issue #120 hold mode, and interactive cancellation reached `cancelled / Cancellation acknowledged.` in both.

Stopping the Rust loopback runtime moved both clients to `disconnected`; after restart, `Reconnect` returned both to `ready / Event connection ready.` Keyboard-only navigation visibly traversed the controls in both clients. `Accessible static details` opened and closed by keyboard, and visible focus indication was observed.

The following are approximate Task Manager observations, not instrumented samples or budgets:

| Condition | Desktop shell | Loopback runtime | WebView2 Manager |
|---|---:|---:|---:|
| Idle | ~4.1 MB / 0% CPU | ~1.6 MB / 0% CPU | ~117.2 MB / 0% CPU |
| Held active | ~4.2 MB / ~0.4% CPU | ~1.7 MB / 0% CPU | ~126.3 MB / ~1.3% CPU |

Interactive cancellation acknowledgement, reconnect after runtime restart, and desktop startup to usable `ready` were each owner-observed as effectively immediate, **<0.5 seconds**. These are human observations at the stated threshold, not instrumented distributions, repeated benchmarks, or evidence of a tighter precision.

## Criteria

| # | Criterion | Result | Evidence / limitation |
|---|---|---|---|
| 1 | One build in same-PC browser and Tauri 2 Windows shell | **Pass (spike scope)** | The exact-head harness built the shared Vite output and packaged Tauri shell; the owner observed the same UI rendered equivalently in browser preview and the packaged shell. |
| 2 | Equivalent loading/success/cancel/reconnect/error behavior | **Pass (spike scope)** | Owner observation covers equivalent ready, loading, success, interactive cancellation, disconnect, and reconnect behavior in both clients. Seven shared component tests cover the error states and lifecycle paths in the single component; error parity is shared-code/automated evidence, not a separately timed human error exercise. |
| 3 | One versioned HTTP endpoint and WebSocket path | **Pass (fixture scope)** | `/v1/fixture` (including its evidence-only `mode=hold`) and `/v1/events`; both clients share the component and boundary. |
| 4 | Loopback/security/version/lifecycle/untrusted input | **Pass (automated fixture scope)** | End-to-end tests prove accepted Windows-origin HTTP and WebSocket requests, HTTP and WebSocket authorization/version rejection, HTTP and WebSocket origin rejection, cancellation acknowledgement, and normalized malformed/oversized input. The token is deliberately non-secret fixture material. |
| 5 | No second Tauri business API | **Pass (static fixture scope)** | Empty permissions and a command-free `Builder`. |
| 6 | Accessibility in both clients | **Pass (G1 basics)** | In both clients, the owner observed keyboard-only traversal, keyboard open/close of the static accessible details, and visible focus. Automated/shared-code evidence covers semantic status/alert, contrast, scaling, reduced motion, and static text. This is the bounded G1 basics check, not a full accessibility audit or release acceptance. |
| 7 | Reproducible Windows build/run | **Pass (one exact-head representative run)** | The deterministic repository harness passed clean validation, Tauri release build, and NSIS packaging at the recorded exact head and versions. Only one reported representative Windows environment is evidenced; broader reproducibility is not claimed. |
| 8 | Required Windows measurements | **Pass (G1 measurement scope)** | Harness: exact 1,848,941-byte installer. Owner: approximate Task Manager idle/held-active CPU and memory observations and human-threshold startup, reconnect, and cancellation observations (<0.5 s each). No profiler, repeated benchmark, distribution, or tighter precision is claimed. |
| 9 | Complete recommendation | **Pass** | The evidence supports closing the G1 evidence-gathering gate. Candidate selection or rejection and any G2 Continue decision remain a separately reviewed authority action. |

## Security, lifecycle, and accessibility findings

The fixture binds only IPv4 loopback. HTTP requires bearer authorization and protocol version. WebSockets require the allowlisted browser-preview or Tauri 2 Windows origin plus correct token/version; messages are bounded and malformed input is normalized. Frontend cancellation sends only on an open socket. Production token issuance/storage is intentionally excluded and cannot be inferred from this fixture.

The hold mode is testability infrastructure, not a product timeout or architecture decision. It is bounded, opt-in, and cancelled at the frontend transport while the existing WebSocket supplies the deterministic acknowledgement; it adds no Tauri command API or client-specific behavior.

## Maturity and exclusions

Before and after, the shared UI, shell, and loopback production boundary remain **Architecture Defined / Specification Approved candidates**. The G1 evidence is now available, but this recording increment does not itself select production architecture or establish Runtime Integrated, Concrete Adapter Implemented, System Verified, User Accepted, or Release Ready maturity. G2–G9, persistence, content, LM Studio, speech, avatar, labs/tools, LAN/remote, hosted, account, and multi-user work remain excluded and undispatched.

## Separately reviewable authority recommendation

**Recommendation: record the G1 evidence gate as satisfied and send candidate disposition to a separate authority review.** All nine G1 criteria have evidence at the bounded spike scope, with the human-observed and automated limitations above. This recommendation does **not** select React/TypeScript/Vite, Tauri 2, or the fixture API as production architecture and does not dispatch G2. Maintain **Tactical Pause** outside this completed evidence-recording increment until the Chief Systems Architect separately records candidate selection or rejection and an explicit Continue, Redirect, or Tactical Pause decision. G2 and all later gates remain undispatched.
