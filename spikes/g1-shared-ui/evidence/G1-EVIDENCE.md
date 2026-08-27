# G1 evidence record

Date: 2026-08-27. Issue: #116. Base: `0b29eca`. Candidate versions are locked by the committed Cargo/npm lockfiles. This is evidence-only disposable code, not production selection.

## Environment and method

The available executor was Ubuntu 24.04 x86_64, not Windows. Exact tool versions are in `environment.txt`. Frontend build wall time and maximum RSS are captured by `/usr/bin/time -v` in `linux-build-time.txt`; sizes are captured by `du -sh` in `linux-artifact-sizes.txt`. These Linux observations are diagnostic only and do not satisfy Windows criteria. The documented README commands reproduce the exact frontend and NSIS Tauri package, but no completed Windows run or interactive accessibility/resource capture was available at authoring time.

## Criteria

| # | Criterion | Result | Evidence / limitation |
|---|---|---|---|
| 1 | One build in same-PC browser and Tauri 2 Windows shell | **Blocked** | One Vite output is configured for browser and `frontendDist`; no Windows run evidence available. |
| 2 | Equivalent loading/success/cancel/reconnect/error behavior | **Blocked** | One component implements the states and component tests cover the static baseline; two-client Windows observation unavailable. |
| 3 | One versioned HTTP endpoint and WebSocket path | **Pass (fixture scope)** | `/v1/fixture` and `/v1/events`; both clients share the same code. |
| 4 | Loopback/security/version/lifecycle/untrusted input | **Pass (automated fixture scope)** | IPv4 loopback bind, origin allowlist, bearer/version rejection, bounded messages and normalized malformed-event response; focused Rust tests. The token is deliberately non-secret fixture material. |
| 5 | No second Tauri business API | **Pass (static fixture scope)** | Empty permissions and a command-free `Builder`. |
| 6 | Accessibility in both clients | **Blocked** | Semantic live status/alert, keyboard-native controls, focus, contrast, scaling, reduced motion and static text are implemented; interactive Windows browser/WebView accessibility validation unavailable. |
| 7 | Reproducible Windows build/run | **Blocked** | README commands define build reproduction; no representative Windows run result yet. |
| 8 | Required Windows measurements | **Blocked** | Linux frontend build/size observations are retained; Windows startup, idle/active CPU/RSS, package size, reconnect/cancel timings were unavailable. |
| 9 | Complete recommendation | **Pass** | Do not pass G1 or select candidates. Run Windows workflow plus scripted representative interaction and accessibility/resource capture, then review the exact-head evidence. |

## Security, lifecycle, and accessibility findings

The runtime has no wildcard/LAN bind, rejects missing authorization and mismatched protocol versions, requires an allowlisted WebSocket origin, bounds inbound text, and handles disconnect/cancel without durable state. CORS permits only the two candidate origins. The UI preserves visible and announced states and an animation-independent path. Production-grade token issuance/storage is intentionally excluded and cannot be inferred from this fixture.

## Maturity and exclusions

Before and after, the shared UI, shell, and loopback production boundary remain **Architecture Defined / Specification Approved candidates**. This spike adds local fixture evidence only; it does not establish Runtime Integrated, Concrete Adapter Implemented, System Verified, User Accepted, or Release Ready maturity. G2–G9, persistence, content, LM Studio, speech, avatar, labs/tools, LAN/remote, hosted, account, and multi-user work remain excluded and undispatched.

## Separately reviewable authority recommendation

Maintain **Tactical Pause** and record G1 as **blocked**, not failed on technical suitability, until exact-head Windows build/run, two-client parity/accessibility observation, and required Windows resource/timing measurements exist. Only after that evidence should a separate authority change select or reject the candidates and decide whether G2 may receive Continue.
