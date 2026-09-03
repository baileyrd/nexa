# AGENTS.md — Rusty Data OS Working Agreement

This file defines how human contributors and AI coding agents must work in this repository.

## 1. Authority order

Before proposing or implementing work, read these files in order:

1. `/AGENTS.md`
2. `/CHATGPT_WORKFLOW.md`
3. `/docs/PROJECT-STATUS.md`
4. `/docs/VISION.md`
5. `/docs/PRINCIPLES.md`
6. `/docs/ARCHITECTURE.md`
7. `/docs/RESEARCH-ROADMAP.md`
8. Applicable hypotheses, experiment definitions, benchmark methodology, ADRs, and specifications.

Repository state on `main` is authoritative over chat history or unstaged ideas.

## 2. Research before architecture

Do not promote an architectural idea because it sounds elegant or familiar.

Every significant technical claim should move through:

```text
idea -> hypothesis -> experiment -> evidence -> decision
```

A hypothesis must be falsifiable. An experiment must state its measurement method before implementation. A decision must cite the evidence that justifies it.

## 3. No premature production code

Experimental implementations belong under `/experiments/`.

Reusable production-oriented code belongs under `/crates/` only after the relevant experiment has produced evidence and an ADR or specification has authorized graduation.

Do not add server/networking layers, SQL interfaces, generalized plugin systems, distributed coordination, or unrelated abstractions unless the research roadmap and current experiment require them.

## 4. Benchmark integrity

Performance claims must follow `/docs/benchmarks/METHODOLOGY.md`.

At minimum:

- capture hardware, OS, filesystem, compiler, toolchain, and build configuration;
- distinguish warm-up from measured samples;
- report distributions, not only averages;
- separate throughput from latency;
- identify durability semantics for every write benchmark;
- preserve raw benchmark results when practical;
- compare against explicit baselines;
- avoid changing multiple independent variables in the same experiment unless the design requires it.

## 5. Correctness precedes speed

A faster implementation that violates declared durability, ordering, replay, or recovery semantics is not a successful optimization.

Each experiment must define correctness invariants and tests independently of performance targets.

## 6. Failed experiments are valuable

Do not delete or obscure negative results. Mark the experiment conclusion clearly and preserve enough evidence to explain why an approach was rejected, deferred, or constrained.

## 7. Documentation synchronization

When an experiment changes project knowledge, update all relevant artifacts in the same change:

- experiment result/conclusion;
- associated hypothesis status;
- ADR if a decision is made;
- `PROJECT-STATUS.md`;
- architecture/specification only if the evidence warrants promotion.

## 8. Smallest useful increment

Prefer bounded experiments that isolate one research question. Avoid "build the database" tasks. The smallest experiment that can falsify a hypothesis is preferred.

## 9. Rust implementation expectations

When Rust code begins:

- favor explicit ownership and concurrency semantics;
- make unsafe code exceptional and justified;
- benchmark release builds;
- keep experimental dependencies isolated;
- document platform-specific behavior;
- prefer deterministic replay and testability over hidden background behavior;
- expose durability boundaries explicitly in APIs rather than implying them.

## 10. Current constraint

The project remains planning/readiness-first; no engine architecture is final. Slice A/A1 and Slice
B/B0 are closed only as bounded implementation/correctness-validation evidence. Slice A2 is closed only as bounded conformance/correctness evidence after its corrective gate; R17 implements the frozen R12/R14/R16 contracts only in the external-dependency-free conformance subset, which contains reviewed workspace path dependencies. R19 closes the R18-authorized Slice C/B1 raw D1 append and deterministic reopen/replay implementation only as bounded correctness evidence. The locally decidable R20 semantic-operation-to-physical-record mapping subset is implemented as a pure correctness component in `exp1-raw-append-replay`, with direct path dependencies on the existing record-format and workload-conformance crates and no append integration; this is bounded implementation/correctness-validation evidence only. R21 freezes the locally decidable immutable-catalog/accepted-prefix design, R22 selects strictly segment-local references, and R23 freezes a canonical closed-scope descriptor whose exact manifest-bound membership and domain-separated digest prove the complete stream set for one authorized cell. R23 closes the closed-scope governance blocker. R25 records that closed, unmerged PR #91 falsified R24’s assumption that the unchanged v1 authorities can produce a valid bootstrap-to-reference causal stream; R25 supersedes R24 for implementation authorization only, preserves every R12/R14/R16 v1 vector and byte, and freezes prospective v2 governance without authorizing code. R26 freezes the complete v2 conformance contract; PR #95 reviewed head `35f9a0f245ac488828df4f639263edb3fb50be86`, merged as `f4ed0c310fa46c6de209ea0f776c4749e31cdd34` with exact-head successful CI, implements it side by side with v1 as bounded conformance/correctness evidence. R27 closes that tranche, freezes the minimum v2 extension of R23 closed-scope proof while preserving v1 and prohibiting mixed membership, and authorizes exactly one pure v2 reference-context mapper implementation in the existing `exp1-raw-append-replay` crate. R28 closes that implementation at PR #98 reviewed head `67715b3efc4732542152ea9d935d92ebdb2ca0d6`, merged as `f5cde575cbd82bb788b9519c4efc56e4d1186131` with both exact-head workflows successful, as bounded correctness evidence. The full R20 reference-context correctness gate is closed. R29 closes the R28-authorized test-only literal SOP2-to-physical-reopen D1 correctness path at PR #101 reviewed head `b88908cb9cbba39774437e582308bab25a88482b`, merged as `2168839a70baebdea1773fc56e7b8aa0dc9a89e4` after successful exact-head Documentation validation and EXP-0001 Slice A workflows, only as bounded deterministic integration/correctness evidence. The R19 semantic-to-physical mapping blocker is closed. R29 freezes the remaining live Linux capture implementation decision and authorizes exactly one next PR to create the external-dependency-free fourth experiment member `exp1-descriptive-d1-harness` for only an isolated Linux/x86_64 capture/preflight boundary and deterministic tests; capture and all execution remain unauthorized. Generated workloads, workload or benchmark execution, D2/D3, `fsync` durability, faults, adapters, production crates, and later increments remain unauthorized. Run the unchanged R9 validation
sequence and `git diff --check` from the repository root.
