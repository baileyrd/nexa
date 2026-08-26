# ADR-0068 — Nexa v1 R2 walking-skeleton baseline

Status: Accepted
Date: 2026-08-26

## Context

The tactical-pause assessment found that Nexa had strong deterministic contract and conformance foundations but no sufficiently concrete release path. Parent architecture/specification maturity trailed implementation, and qualified headless/contract gates were allowed to act as program-progress signals.

The owner subsequently delegated completion of the R0/R1 review and convergence work, including bounded decisions necessary to remove R2 blockers, without waiting for further interaction.

This ADR establishes the minimum concrete baseline for the R2 thin production walking skeleton. It is intentionally narrower than the full long-term Nexa architecture.

## Decision

### 1. R2 is text-first

R2 MUST prove one real learner journey through text input/output. Speech input/output and animated avatar embodiment are retained architectural capabilities but are not R2 exit criteria. They are reconsidered after the walking skeleton at the later embodiment/speech integration gate.

Labs/tool execution and dynamic multi-provider routing are post-R2 and do not block R2.

### 2. First learner application

The first learner-facing composition root is `apps/nexa-desktop`.

The initial UI technology is `eframe`/`egui`, subject to a short implementation spike proving:

- asynchronous tutor work does not block the UI;
- the required course, lesson, tutor, assessment, progress, error, and resume states can be represented;
- keyboard/accessibility basics are available on the supported platform;
- release packaging is feasible.

If the spike disproves suitability, a replacement requires a new ADR; implementation must not silently switch UI architecture.

### 3. Durable state

The first concrete durable store is SQLite through a dedicated `nexa-storage` infrastructure adapter using `rusqlite`.

Domain crates MUST NOT depend on `rusqlite`.

The adapter must preserve canonical Nexa identifiers and provide transaction semantics compatible with the existing learning-core unit-of-work contract. Schema migrations, optimistic concurrency, restart/resume, backup/recovery, and corruption behavior are governed by the R1 data baseline.

SQLite row identifiers MUST NOT replace canonical domain IDs.

### 4. First concrete model path

The first concrete model execution path is a local `llama.cpp` server accessed through a bounded Nexa HTTP adapter implementing existing provider-neutral tutor/model contracts.

R2 uses one explicitly configured provider/model. Dynamic routing, fallback chains, latency/cost optimization, and multi-provider policy are not R2 requirements.

`llama.cpp` compatibility is adapter-specific; OpenAI-compatible surface similarity MUST NOT be treated as proof of provider equivalence.

The exact release GGUF model is selected later through the tutor-quality/performance gate. A development model may be used to build R2, but release evidence must identify the exact model and configuration tested.

### 5. First supported release environment

Windows x86_64 is the first release acceptance target. Linux remains useful for headless CI and portability evidence but does not by itself establish Windows release readiness.

Release-critical UI/storage/model paths require Windows CI or equivalent reproducible Windows validation before System Verified maturity.

### 6. First course

The first governed walking-skeleton course is a bounded Networking Fundamentals package centered on TCP connection establishment.

Minimum lesson objectives:

- purpose of TCP connection establishment;
- SYN → SYN/ACK → ACK ordering;
- basic contrast with UDP;
- deterministic assessment of those objectives;
- persistent learner evidence/progress;
- governed source/citation material.

This course is an acceptance vehicle, not a limitation on Nexa's eventual curriculum scope.

### 7. Event architecture for R2

R2 does not introduce a durable event broker merely because typed events exist.

Authoritative learner state is committed through the durable transaction boundary. Existing typed events may be emitted as in-process domain facts and observability inputs. A durable outbox is introduced only if an R2/R3 correctness requirement establishes an asynchronous durable consumer whose loss would violate accepted behavior.

### 8. Packaging posture

R2 may treat `llama-server` and the development model as explicitly configured local dependencies while exercising the application path. Final bundling/download/runtime distribution is an R8 release decision.

The project may evaluate `cargo-dist` for release artifact orchestration. No MSI-specific tool is required to clear R2.

## Architecture consequences

- Provider neutrality remains an architectural property, but R2 optimizes for one real provider path rather than additional abstract routing.
- Existing Phase 1–5 code is reused where it directly supports the learner journey; it does not gain product maturity merely by being retained.
- `apps/nexa-headless` remains test/integration infrastructure and is not the v1 learner application.
- `nexa-storage` becomes an actual infrastructure boundary instead of a reserved directory.
- Conditional speech/avatar/lab foundations are not deleted and continue to be tested, but they cannot consume R2 critical-path capacity unless a release blocker requires them.

## Security/privacy constraints

- The local model server binds to loopback by default.
- Model/server paths and endpoints come from trusted configuration, never learner/model content.
- No provider credential or secret belongs in ordinary domain persistence or normal logs.
- Remote inference, if later added, requires explicit security/privacy authorization; R2 local inference does not silently authorize remote disclosure.
- Raw learner text, prompts, source content, raw model output, and secrets remain excluded from normal diagnostics.

## Verification required before R2 closure

R2 is not complete until a test demonstrates:

1. launch the learner desktop app on the supported environment;
2. initialize/open the concrete SQLite store;
3. start the governed TCP lesson;
4. submit learner text through the real UI boundary;
5. load/derive learner, lesson, pedagogy, and knowledge context;
6. invoke the concrete local llama.cpp adapter;
7. admit and present the tutor response;
8. complete one governed assessment/practice action;
9. atomically persist evidence/mastery/progress;
10. exit and restart;
11. resume without duplicated or lost accepted state;
12. correlate the interaction using content-safe operational evidence.

Scripted model providers and in-memory persistence do not satisfy this R2 exit gate.

## Deferred decisions

The following remain later gates unless evidence promotes them:

- release model family/quantization;
- final llama.cpp distribution/bundling policy;
- remote provider support;
- speech and avatar inclusion in the final v1 release;
- lab/tool execution;
- vector database/extension adoption;
- advanced routing/fallback;
- final installer format and signing mechanism.

## Supersession

This ADR does not invalidate ADR-0001 through ADR-0067. Where an earlier ADR describes a broader possible capability, this ADR controls only the R2 release path and maturity priority.