# NEXA-ARCH-002 — Nexa v1 Release Architecture

Status: Approved; reconciled 2026-08-26 by ADR-0069

## Objective and composition

Nexa v1 is a local-first adaptive tutor for one learner on one Windows PC. An identical compiled shared frontend runs in a normal same-machine browser and a Tauri 2 Windows shell. Both communicate with the local Rust runtime through one versioned loopback HTTP/WebSocket boundary. Tauri commands provide shell integration only and never a parallel business API.

```text
shared React/TypeScript/Vite candidate UI (browser | Tauri 2 candidate shell)
  -> versioned, loopback-only HTTP/WebSocket boundary
  -> Rust orchestrator/runtime
  -> SQLite durable learning/evidence state
  -> learning/pedagogy + governed TCP knowledge
  -> narrow LM Studio adapter -> separately installed local server
  -> admitted tutor response + assessment/practice
  -> bundled speech + semantic 2D tutor presentation
```

The UI stack and shell remain candidates pending the bounded first spike. The boundary must authenticate/authorize the local client as specified, restrict binding/origins, validate versioned messages, support cancellation/reconnect, and preserve identical features. Same-machine browser evidence does not prove remote or hosted deployment.

## Owned architecture

Domain-facing crates own stable IDs, learning/evidence rules, tutor admission, provider-neutral speech/model contracts, and semantic behavior. Runtime/adapters own HTTP/WebSocket, SQLite, LM Studio protocol mapping, audio devices, and rendering. The orchestrator coordinates lifetimes and atomic workflow without absorbing subsystem reasoning. Model output is untrusted semantic content and never chooses host authority or renderer primitives.

SQLite is isolated behind `crates/nexa-storage`; it preserves migrations, canonical identifiers, atomic accepted progress/evidence, backup/recovery, and restart/resume. One process/store writer and one learner are supported. No accounts or cloud sync are required.

LM Studio is the sole validated v1 reference model server. Nexa bundles neither LLM weights nor inference runtime. One narrow adapter records model/server compatibility and normalizes failures; provider-neutral contracts remain cloud-ready, but hosted deployment, multiple integrations, and dynamic routing/fallback are deferred.

Speech input/output and animated 2D embodiment are release requirements. Nexa bundles and manages CPU-capable speech models/runtime. Sherpa-ONNX remains evidence-gated, with `whisper.cpp` only the recognition fallback if disproved and TTS subject to separate selection. Rive remains evidence-gated for identical 2D rendering, lip-sync, semantic idle/listening/thinking/speaking/error states, interruption, accessibility fallback, and CPU use. Existing speech/avatar contracts are foundations, not adapter proof.

## Security, privacy, and deployment

The runtime binds only to loopback; learner/model content cannot select endpoints. Local data and diagnostics follow minimization, redaction, retention, deletion, and corruption-recovery policy. The separately operated model server is a local trust boundary whose configuration and availability are visible. Release packaging includes the local runtime, shared client assets, speech runtime/models, content, and migrations, but excludes LLM weights/server. Cloud-ready ports do not constitute hosted deployment.

## Delivery and evidence

ADR-0069 defines the ordered route: reconciliation; independently reviewable UI, speech, and avatar spikes; persistent text lesson; LM Studio; speech; 2D tutor; then system verification, packaging, and user acceptance. A spike cannot silently become production architecture. General product implementation stays paused; the first permissible follow-on is only the shared UI/loopback spike.

The real E2E gate launches either identical client, completes the governed TCP lesson through LM Studio, atomically persists and resumes progress, converses using bundled speech, displays synchronized semantic 2D behavior, and passes failure, security/privacy, performance, accessibility, Windows package, system, and user gates.

## Deferred scope and maturity

LAN/Internet-remote clients, hosted deployment, cloud sync, labs/tools, broad providers, dynamic routing/fallback, dedicated vector infrastructure unless proven necessary, durable event brokerage, and 3D release integration are post-v1. Existing contract and headless evidence remains factual; this reconciliation proves Architecture Defined/Specification Approved only, not a concrete adapter or System Verified maturity.
