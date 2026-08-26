# Nexa Completion Roadmap

Status: Active delivery control; reconciled 2026-08-26

## Finite dependency-ordered route

| Gate | Increment | Exit evidence | Failure consequence |
|---|---|---|---|
| G0 | Authority reconciliation (this increment) | ADR-0069 and all active parents/supplements/status/deferrals agree; links, stale-claim scan, scope check pass | Tactical Pause; correct authority only |
| G1 | Shared UI/loopback suitability spike | Same compiled React/TS/Vite candidate frontend in browser and Tauri 2 candidate shell; versioned HTTP request + WebSocket event; loopback bind/origin/auth; cancellation/reconnect/errors; keyboard/accessibility; Windows build; parity; startup/CPU/memory/package measurements | Reject disproved candidate/boundary; update ADR/architecture through owner route before production work |
| G2 | Bundled speech suitability spike | Sherpa-ONNX measurements for recognition accuracy, latency, synthesis quality, memory, package size, interruption/cancellation, lip-sync on CPU-only Windows reference PC | Reject Sherpa-ONNX; evaluate `whisper.cpp` for recognition only and govern TTS separately |
| G3 | Animated 2D suitability spike | Rive proves identical browser/desktop idle/listening/thinking/speaking/error states, interruption, lip-sync, accessible fallback, acceptable CPU | Reject Rive and govern a replacement before integration |
| G4 | Persistent text lesson | Production shared boundary, SQLite migrations/atomicity/recovery, governed TCP lesson, assessment, restart/resume in both clients | Fix owning UI/data/content authority; do not advance |
| G5 | LM Studio integration | Narrow adapter against documented separately installed reference server; admission, cancellation/error, compatibility and E2E text evidence | Reconcile provider adapter/compatibility authority |
| G6 | Speech integration | Bundled selected speech adapter/device lifecycle, accessible text equivalent, interruption/lip-sync, package evidence | Return to speech selection/integration gate |
| G7 | Animated tutor integration | Selected 2D runtime consumes admitted semantic states with synchronized speech and graceful fallback in both clients | Return to avatar selection/integration gate |
| G8 | Complete-system verification | Full E2E/failure/security/privacy/performance/accessibility evidence on Windows reference PC and both clients | Correct owning gate; no maturity overstatement |
| G9 | Packaging and user acceptance | Clean install/update/uninstall, package contents/licensing, representative acceptance; exact release head green | No Release Ready claim |

## Work selection

G1 is the precise first permitted follow-on and must be separately dispatched. Spike branches are disposable evidence, independently reviewable, and cannot silently become production architecture. No later gate starts merely because research looks favorable. A recorded authority update accepts or rejects each candidate.

The final outcome is completion of the governed TCP lesson from either identical client using LM Studio, durable restart/resume, bundled speech, synchronized 2D tutor behavior, and system/user/package evidence. Same-machine browser evidence is never labeled hosted-web or remote evidence.

## Deferrals and control

Labs/tools, LAN/Internet-remote access, hosted deployment, cloud sync, broad providers, dynamic routing/fallback, and 3D release integration remain post-v1. Existing Phase 1–5 evidence is retained, not repeated as substitute release proof. The Chief Systems Architect maintains Tactical Pause outside the next eligible gate and records Continue, Redirect, or Tactical Pause at every gate.
