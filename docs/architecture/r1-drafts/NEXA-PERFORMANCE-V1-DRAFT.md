# Nexa v1 Performance Specification — Draft

Status: R1 proposal; non-authoritative until registered and approved

## 1. Purpose

Define measurable performance budgets for the first release so architecture and implementation choices are driven by evidence rather than speculative optimization.

## 2. Principles

- Budgets apply to the primary learner journey first.
- Measure before optimizing.
- Separate Nexa-controlled latency from LM Studio latency; retain remote-provider separation as a post-v1 safeguard.
- Use representative release hardware, corpus, data size, and configuration.
- Report warm/cold behavior separately where meaningful.
- Performance must not compromise correctness, security, privacy, durability, or accessibility.

## 3. Required budget categories

Before R2 implementation is considered release-oriented, approve numerical budgets or bounded target classes for:

- application cold start to usable course selection;
- warm start/resume;
- learner/course state load;
- durable learning commit;
- knowledge retrieval and context assembly;
- local prompt compilation/admission overhead;
- model-provider request preflight;
- end-to-end text interaction excluding provider generation;
- end-to-end text interaction including the selected provider, reported separately;
- cancellation/interruption responsiveness;
- clean shutdown;
- migration/startup recovery for supported data sizes;
- memory footprint at idle and during a representative lesson;
- persistent data growth for a representative learner/course;
- released knowledge corpus size and retrieval scale;
- installer/application footprint where relevant.

## 4. Provider-dependent latency

Same-machine LM Studio generation latency is dependency/configuration dependent. Any remote-provider latency branch is a post-v1 safeguard requiring new owner and architecture authority.

Report at least:

- Nexa pre-provider overhead;
- provider time-to-complete for the released mode;
- Nexa post-provider admission/quality/presentation overhead;
- total learner-perceived response time.

Nexa must not hide a slow local implementation behind provider latency. Likewise, provider latency should not be reported as if entirely under Nexa control.

## 5. Representative workload

The release benchmark set must include:

- clean/new learner;
- resumed learner with representative evidence/progress history;
- smallest and representative released course/knowledge corpus;
- representative retrieval query/result/context sizes;
- representative assessment commit;
- representative tutor prompt/output sizes;
- restart after accumulated progress;
- at least one degraded dependency/error path.

## 6. Resource budgets

Measure:

- process resident memory;
- CPU during idle and active interaction;
- storage footprint/growth;
- observed GPU usage, if any, for the required UI/embodiment path and its CPU fallback;
- loopback network bytes per representative LM Studio tutor interaction where measurable; if remote inference is separately authorized after v1, measure its network use for privacy/cost awareness.

Record CPU and any observed GPU use for both identical clients, required bundled speech, and required animated 2D rendering. The reference path must remain usable on the CPU-only Windows reference PC; candidate evidence, not a blanket no-GPU rule, governs acceleration and fallback decisions.

## 7. Persistence performance

Data architecture must remain correct under the selected durability settings.

Measure:

- state load;
- atomic learning commit;
- evidence replay/mastery rebuild at representative history size;
- knowledge ingestion/reopen;
- migration on representative predecessor data once applicable.

Do not weaken transaction/durability semantics merely to hit a latency target without an explicit architecture decision.

## 8. Retrieval performance

Benchmark the chosen concrete v1 retrieval implementation using the released corpus scale.

Record:

- corpus/chunk count;
- storage/index mode;
- query latency;
- context assembly latency;
- result limits;
- memory footprint;
- cold/warm distinction.

An external vector database is justified only if the simpler local architecture fails approved quality/performance requirements or other v1 constraints.

## 9. UI responsiveness

Long-running storage/retrieval/model operations must not make the learner application appear permanently frozen.

The UX/runtime design must provide:

- visible active/working state;
- cancellation/interruption where supported;
- responsive window/input handling according to the selected framework;
- bounded synchronous work on the UI thread.

Concrete responsiveness thresholds are defined after selecting the UI framework/platform.

## 10. Measurement methodology

Every release performance result records:

- build/release version and commit;
- OS/hardware;
- debug/release build mode;
- provider/model/configuration;
- corpus/data sizes;
- benchmark/test revision;
- run count;
- warm/cold state;
- median and a tail percentile appropriate to the metric;
- failures/timeouts separately from successful latency samples.

## 11. Regression gates

Once v1 budgets are established, release CI/validation must detect material regressions for deterministic/local measurements where feasible.

LM Studio latency must be monitored/reported with controlled release evidence. If remote inference is explicitly authorized after v1, its latency must not become an unstable hard CI gate unless a controlled test service exists.

## 12. Verification

R7 passes only when:

- all required budgets have numeric approved values;
- measurements use production/release builds and representative data;
- no release-critical deterministic/local budget is exceeded without explicit accepted disposition;
- memory/storage growth is bounded for the supported learner/course scope;
- performance fixes preserve all correctness/security/privacy tests.

## 13. Decisions required for approval

- first supported hardware/OS baseline;
- first concrete model-provider mode;
- released corpus/course scale;
- concrete numeric latency/resource budgets;
- acceptable provider-latency reporting target versus hard requirement;
- UI responsiveness thresholds after framework selection.

## 2026-08-26 ADR-0069 reconciliation

Budgets must cover both identical clients, the loopback HTTP/WebSocket boundary, LM Studio, bundled speech, and animated 2D rendering on the CPU-only Windows reference PC. The UI, Sherpa-ONNX, and Rive spikes record startup, CPU, memory, latency/timing, and package impact as applicable. The earlier text-only/no-GPU framing does not remove required speech/avatar measurements.
