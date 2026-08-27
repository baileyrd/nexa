# G1 evidence record

Date: 2026-08-27. Issue: #116. GitHub base: `0b29eca746090334825bdfae4e71b5eb3e0aa042`. Candidate versions are locked by committed Cargo/npm lockfiles. This is evidence-only disposable code, not production selection.

## Environment and method

The available executor was Ubuntu 24.04 x86_64, not Windows. Exact tool versions are in `environment.txt`. Linux frontend wall time was captured with the available shell clock and maximum RSS is explicitly unavailable in `linux-build-time.txt`; sizes are in `linux-artifact-sizes.txt`. These observations are diagnostic only. Automated local evidence covers frontend lint/typecheck, six lifecycle/component tests and production build plus runtime format/check and three end-to-end HTTP/WebSocket tests.

The repository-local `scripts/validate-windows.ps1` harness performs clean locked installs, the same frontend/runtime checks, a Tauri 2 Windows NSIS build, and package-size capture with machine-readable pass/fail JSON. It has not been executed on a representative Windows machine, so it is a reproduction mechanism and not Windows success evidence.

## Criteria

| # | Criterion | Result | Evidence / limitation |
|---|---|---|---|
| 1 | One build in same-PC browser and Tauri 2 Windows shell | **Blocked** | One Vite output is configured for browser and `frontendDist`; Windows harness is unexecuted and no two-client run exists. |
| 2 | Equivalent loading/success/cancel/reconnect/error behavior | **Blocked** | Automated component tests cover all transitions, including guarded cancellation while connecting; representative browser/WebView parity is unexecuted. |
| 3 | One versioned HTTP endpoint and WebSocket path | **Pass (fixture scope)** | `/v1/fixture` and `/v1/events`; both clients share the component and boundary. |
| 4 | Loopback/security/version/lifecycle/untrusted input | **Pass (automated fixture scope)** | End-to-end tests prove the Windows Tauri origin handshake, HTTP and WebSocket authorization/version rejection, origin rejection, cancellation acknowledgement, and normalized malformed/oversized input. The token is deliberately non-secret fixture material. |
| 5 | No second Tauri business API | **Pass (static fixture scope)** | Empty permissions and a command-free `Builder`. |
| 6 | Accessibility in both clients | **Blocked** | Semantic live status/alert, keyboard-native controls, focus, contrast, scaling, reduced motion and static text exist; representative browser/WebView accessibility validation is unexecuted. |
| 7 | Reproducible Windows build/run | **Blocked** | A clean-checkout harness covers build/package and emits JSON, but no representative Windows result has been executed or reviewed. |
| 8 | Required Windows measurements | **Blocked** | The harness captures package size only when run. Windows startup, idle/active CPU/RSS, package size, reconnect and cancellation timings remain unavailable. |
| 9 | Complete recommendation | **Pass** | Do not pass G1 or select candidates. Execute and review the Windows harness plus representative interaction, accessibility, resource and timing capture on the exact head. |

## Security, lifecycle, and accessibility findings

The fixture binds only IPv4 loopback. HTTP requires bearer authorization and protocol version. WebSockets require the allowlisted browser-preview or Tauri 2 Windows origin plus correct token/version; messages are bounded and malformed input is normalized. Frontend cancellation sends only on an open socket. Production token issuance/storage is intentionally excluded and cannot be inferred from this fixture.

## Maturity and exclusions

Before and after, the shared UI, shell, and loopback production boundary remain **Architecture Defined / Specification Approved candidates**. This spike does not establish Runtime Integrated, Concrete Adapter Implemented, System Verified, User Accepted, or Release Ready maturity. G2–G9, persistence, content, LM Studio, speech, avatar, labs/tools, LAN/remote, hosted, account, and multi-user work remain excluded and undispatched.

## Separately reviewable authority recommendation

Maintain **Tactical Pause** and G1 **blocked** until exact-head representative Windows browser/WebView parity and accessibility checks, startup/CPU/memory/package measurements, and reconnect/cancellation timing evidence are executed and reviewed. Only a later separate authority update may select or reject candidates and decide whether G2 receives Continue.
