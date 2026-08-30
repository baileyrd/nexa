# G1 evidence record

Date: 2026-08-27. Issue: #116. GitHub base: `0b29eca746090334825bdfae4e71b5eb3e0aa042`. Candidate versions are locked by committed Cargo/npm lockfiles. This is evidence-only disposable code, not production selection.

## Environment and method

The available executor was Ubuntu 24.04 x86_64, not Windows. Exact tool versions are in `environment.txt`. Linux frontend wall time was captured with the available shell clock and maximum RSS is explicitly unavailable in `linux-build-time.txt`; sizes are in `linux-artifact-sizes.txt`. These observations are diagnostic only. Automated local evidence covers frontend lint/typecheck, lifecycle/component tests and production build plus runtime format/check and end-to-end HTTP/WebSocket tests. Issue #120 adds a bounded, explicitly selected 60-second hold mode so a human can observe running and request cancellation; normal requests remain fast. Tests drive the runtime hold clock deterministically and drive the frontend acknowledgement directly rather than sleeping.

The repository-local `scripts/validate-windows.ps1` harness performs clean locked installs, the same frontend/runtime checks, a Tauri 2 Windows NSIS build, and package-size capture with machine-readable pass/fail JSON that records the exact repository head and tool versions. A representative Windows execution on merged `main` at `38bb0e71cda177c7ae9dc0afb81118ba4ee744c4` passed every frontend, runtime, and desktop dependency step, then failed during Tauri packaging because the default `desktop/src-tauri/icons/icon.ico` resource was absent. Issue #118 adds a repository-owned evidence-only placeholder at that conventional location. Windows evidence remains incomplete until the owner reruns the harness after the correction merges; the prior failed run is not Windows success evidence.

## Criteria

| # | Criterion | Result | Evidence / limitation |
|---|---|---|---|
| 1 | One build in same-PC browser and Tauri 2 Windows shell | **Blocked** | One Vite output is configured for browser and `frontendDist`; the first Windows package attempt exposed a missing icon prerequisite, the corrected harness has not been rerun, and no two-client run exists. |
| 2 | Equivalent loading/success/cancel/reconnect/error behavior | **Blocked** | Automated component tests cover normal success, explicit held-running state, acknowledged cancellation, guarded cancellation while connecting, reconnect, and errors in the one shared component; representative browser/WebView parity and interactive cancellation remain unexecuted. |
| 3 | One versioned HTTP endpoint and WebSocket path | **Pass (fixture scope)** | `/v1/fixture` (including its evidence-only `mode=hold`) and `/v1/events`; both clients share the component and boundary. |
| 4 | Loopback/security/version/lifecycle/untrusted input | **Pass (automated fixture scope)** | End-to-end tests prove accepted Windows-origin HTTP and WebSocket requests, HTTP and WebSocket authorization/version rejection, HTTP and WebSocket origin rejection, cancellation acknowledgement, and normalized malformed/oversized input. The token is deliberately non-secret fixture material. |
| 5 | No second Tauri business API | **Pass (static fixture scope)** | Empty permissions and a command-free `Builder`. |
| 6 | Accessibility in both clients | **Blocked** | Semantic live status/alert, keyboard-native controls, focus, contrast, scaling, reduced motion and static text exist; representative browser/WebView accessibility validation is unexecuted. |
| 7 | Reproducible Windows build/run | **Blocked** | The first representative harness run reached Tauri packaging and failed on the missing default icon. The repository now supplies an evidence-only icon, but a post-merge rerun has not been executed or reviewed. |
| 8 | Required Windows measurements | **Blocked** | The harness captures package size only when run. Windows startup, idle/active CPU/RSS, package size, reconnect and cancellation timings remain unavailable. |
| 9 | Complete recommendation | **Pass** | Do not pass G1 or select candidates. Execute and review the Windows harness plus representative interaction, accessibility, resource and timing capture on the exact head. |

## Security, lifecycle, and accessibility findings

The fixture binds only IPv4 loopback. HTTP requires bearer authorization and protocol version. WebSockets require the allowlisted browser-preview or Tauri 2 Windows origin plus correct token/version; messages are bounded and malformed input is normalized. Frontend cancellation sends only on an open socket. Production token issuance/storage is intentionally excluded and cannot be inferred from this fixture.

The hold mode is testability infrastructure, not a product timeout or architecture decision. It is bounded, opt-in, and cancelled at the frontend transport while the existing WebSocket supplies the deterministic acknowledgement; it adds no Tauri command API or client-specific behavior.

## Maturity and exclusions

Before and after, the shared UI, shell, and loopback production boundary remain **Architecture Defined / Specification Approved candidates**. This spike does not establish Runtime Integrated, Concrete Adapter Implemented, System Verified, User Accepted, or Release Ready maturity. G2–G9, persistence, content, LM Studio, speech, avatar, labs/tools, LAN/remote, hosted, account, and multi-user work remain excluded and undispatched.

## Separately reviewable authority recommendation

Maintain **Tactical Pause** and G1 **blocked** until exact-head representative Windows browser/WebView parity and accessibility checks, startup/CPU/memory/package measurements, and reconnect/cancellation timing evidence are executed and reviewed. Only a later separate authority update may select or reject candidates and decide whether G2 receives Continue.
