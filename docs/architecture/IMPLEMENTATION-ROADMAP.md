# Nexa Implementation Roadmap

Status date: 2026-08-30

The pre-tactical-pause Phase 0–6 roadmap is preserved in Git history and phase traceability, but it no longer selects new implementation work.

The current delivery authority is:

- [`NEXA-COMPLETION-ROADMAP.md`](NEXA-COMPLETION-ROADMAP.md)
- [`NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md`](NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md)
- [`NEXA-R1-IMPLEMENTATION-BASELINE.md`](NEXA-R1-IMPLEMENTATION-BASELINE.md)
- [`../adr/0069-owner-approved-v1-delivery-baseline.md`](../adr/0069-owner-approved-v1-delivery-baseline.md)
- [`../adr/0068-v1-r2-walking-skeleton-baseline.md`](../adr/0068-v1-r2-walking-skeleton-baseline.md) (historical/non-conflicting scope)

## Why the roadmap changed

The tactical-pause assessment found that qualified deterministic/headless contract gates had become program-progress signals while parent architecture/specification maturity and vertical product integration lagged. The new roadmap is organized around capability maturity and a finite release outcome.

Historical Phase 1–5 work remains valid evidence and reusable implementation. It is not discarded; it is reclassified according to what it actually proves.

## Finite dependency-ordered route

| Gate | Outcome | Current disposition |
|---|---|---|
| G0 | Reconcile Issue #114 authority without changing implementation maturity | Merged; authority reconciled |
| G1 | Prove shared React/TypeScript/Vite + Tauri 2 candidate and versioned loopback HTTP/WebSocket suitability | Evidence complete; disposition pending separate authority review |
| G2 | Prove bundled CPU speech candidate, including cancellation, devices, packaging, latency, and licensing | Evidence-gated after G1 |
| G3 | Prove synchronized Rive 2D candidate in both clients, including lip-sync, semantic states, interruption, accessibility, and CPU use | Evidence-gated after G2 |
| G4 | Integrate the shared clients, local Rust runtime, SQLite, governed TCP lesson, and atomic restart/resume | Pending prior gates |
| G5 | Integrate the narrow LM Studio adapter and admitted tutor response | Pending G4 |
| G6 | Integrate required bundled speech | Pending G2 and G5 |
| G7 | Integrate required synchronized animated 2D tutor | Pending G3 and G6 |
| G8 | System verification and Windows packaging/recovery/security/privacy/accessibility/performance evidence | Pending integrated route |
| G9 | Owner user acceptance and release decision | Pending G8 |

Candidate spike success does not silently select production architecture; failure removes that candidate and requires owner-governed reselection plus an authority update. General product implementation remains paused outside the next eligible, separately dispatched gate.

Every increment states its finite-route blocker, governing authority, concrete E2E step, maturity before/after, and required evidence. Deferred scope is LAN/remote access, hosted deployment, cloud sync, accounts/multi-user administration, labs/tools, broad providers, dynamic routing/fallback, dedicated vector infrastructure unless proven necessary, durable event brokerage, and 3D release integration.

## Cross-stage quality gates

Every increment still requires proportionate:

- formatting/build/lint/test checks;
- contract/dependency-boundary verification;
- architecture/specification traceability;
- security/privacy review for the changed boundary;
- deterministic evidence where the behavior is deterministic;
- concrete-adapter/system evidence when claiming higher maturity;
- documentation/status updates that do not overstate capability maturity.

## Architecture rebaseline checkpoints

After each candidate evidence gate (G1–G3), before G4 integration, before each material G5–G7 expansion, and before G8/G9, the Chief Systems Architect must independently evaluate the whole program and record Continue, Redirect, or Tactical Pause.

See [`../governance/ARCHITECTURE-REBASELINE-GATES.md`](../governance/ARCHITECTURE-REBASELINE-GATES.md).

---

## G1 evidence status (2026-08-30)

Issue #116 produced a bounded disposable fixture and [criterion-level evidence](../../spikes/g1-shared-ui/evidence/G1-EVIDENCE.md). Linux/headless automation covers the shared frontend checks/lifecycle tests/build and end-to-end loopback security/cancellation tests. The first representative Windows run later exposed the missing packaging icon corrected through Issue #118; Issue #120 added bounded cancellation-observation support. Those earlier results remain historical evidence rather than a successful G1 disposition.

Issue #122 records the harness **PASS** from exact `main` head `9173ed152d7b1d5bb9831413862097076443be59`, including the 1,848,941-byte NSIS installer, and separately records owner-observed two-client parity, cancellation/reconnect, keyboard/focus basics, approximate Task Manager CPU/memory, and <0.5-second human-observed startup/cancellation/reconnect thresholds. All criteria now have bounded spike evidence, so the separately reviewable recommendation is **G1 evidence satisfied; candidate disposition pending**. This is not production architecture selection or a G2 dispatch. Candidate maturity is unchanged; G2+ remain undispatched and Tactical Pause remains in force pending the separate Chief Systems Architect authority decision.
