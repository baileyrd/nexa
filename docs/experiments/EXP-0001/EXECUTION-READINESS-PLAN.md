# EXP-0001 Execution-Readiness and Staged-Implementation Plan

**Status:** Active staged plan; the locally decidable pure R20 mapper subset is implemented; R19 still blocks descriptive D1 capture pending a platform-interface decision
**Scope:** bridge from the completed EXP-0000 framework to an executable EXP-0001
**Evidence classification:** planning plus reviewed Slice A and Slice B correctness-validation status; no benchmark, persistence, durability, or performance evidence

## 1. Authority and current readiness

This plan is the authoritative ordering and gate map for preparing EXP-0001. The
[EXP-0000 contracts](../EXP-0000-measurement-and-semantics-readiness.md) supply its semantic and measurement inputs; the
[EXP-0001 definition](../EXP-0001-immutable-event-ingestion.md) remains the experiment authority. If either changes materially,
this plan must be reviewed and versioned before work continues.

EXP-0000 is complete **as measurement-readiness documentation**. No implementation, benchmark, correctness-run, recovery-run,
or performance evidence exists. [HYP-0001](../../hypotheses/HYP-0001-event-log-as-canonical-state.md) remains active and unproven,
and EXP-0001 remains proposed. Implementation authorization and confirmatory-execution authorization are separate gates.
A documentation-complete experiment framework is not an executable experiment.

| Readiness level | Present state | Exit condition |
|---|---|---|
| Framework readiness | Complete | EXP-0000's seven reviewed contracts remain linked and internally consistent. |
| Design-choice readiness | Incomplete | The physical and lifecycle choices required by the intended slice are resolved through R1–R8 as applicable. |
| Implementation readiness | Slices A, B/B0, A2, bounded Slice C/B1, and the R28 integration complete | R29 closes the semantic-to-physical integration blocker and freezes a capture/preflight implementation boundary; harness composition, capture, and every execution increment require separate authorization. |
| Correctness-validation readiness | Slices A, B/B0, A2, and bounded Slice C/B1 passed only | Stable fixtures, oracle, recovery rules, fault mechanisms, and validation procedures must exist for every applicable measured mode. |
| Descriptive execution readiness | Incomplete | A validated runnable cell, environment, stream, adapter, instrumentation, and result path pass the descriptive gate. |
| Confirmatory execution readiness | Incomplete | The stricter frozen-design gate in section 7 passes; descriptive readiness alone is insufficient. |

## 2. Remaining blocker registry

`Open` means no reviewed output resolves the blocker. Merely knowing the required fields does not resolve it. Dependencies name
other blocker IDs or the frozen semantic contracts (`SEM`). Unknowns refer to the repository [unknown registry](../../ASSUMPTIONS-AND-UNKNOWNS.md).

| ID | Blocker | Status | Dependencies | Affected unknowns | Required output | Controls |
|---|---|---|---|---|---|---|
| BLK-001 | Physical event encoding and record framing | Resolved by [R5 physical contract](R5-PHYSICAL-RECORD-INTEGRITY-AND-RECOVERABLE-COMMIT-CONTRACT.md) | SEM, BLK-002, BLK-013 | UNK-001, UNK-012 | Versioned physical contract and documentation vectors | Slice A/C prerequisite; implementation still gated |
| BLK-002 | Minimum integrity policy and supported integrity modes | Resolved by [R1](R1-PHYSICAL-RECORD-INTEGRITY-RECOVERY-REQUIREMENTS.md) | SEM | UNK-012 | Required coverage, detection, finalization, failure behavior, and mode contract | BLK-003 |
| BLK-003 | Integrity/checksum algorithm | Resolved by [R5 physical contract](R5-PHYSICAL-RECORD-INTEGRITY-AND-RECOVERABLE-COMMIT-CONTRACT.md) | BLK-002, BLK-001 | UNK-012 | CRC-32C profile, exact coverage, limits, and documentation vectors | Integrity-enabled implementation prerequisite; implementation still gated |
| BLK-004 | Request/event identity algorithms and assignment lifecycle | Resolved by [R3](R3-IDENTITY-TIME-SEQUENCING-RETRY-LIFECYCLE.md) | SEM, BLK-012 | UNK-002, UNK-009, UNK-016 | Separate algorithms, authorities, collision/error rules, and capture points | Slice A; comparable streams |
| BLK-005 | Timestamp representation, clocks, and capture points | Resolved by [R3](R3-IDENTITY-TIME-SEQUENCING-RETRY-LIFECYCLE.md) | SEM, BLK-012 | UNK-003, UNK-010 | Representation, precision, clock and lifecycle rules | Slice A; records |
| BLK-006 | Payload-content generator | Resolved as documentation design by [R12](R12-DETERMINISTIC-GENERATOR-SPECIFICATION-AND-VECTORS.md); dependency-free conformance implementation exists | SEM | UNK-018 | Platform-independent algorithm and test vectors | Comparable stream |
| BLK-007 | Identity/envelope/reference generator | Resolved as documentation design by [R12](R12-DETERMINISTIC-GENERATOR-SPECIFICATION-AND-VECTORS.md), consistent with [R2](R2-DETERMINISTIC-WORKLOAD-BYTES-IDENTITY-REFERENCES-DIGEST-REQUIREMENTS.md) and [R3](R3-IDENTITY-TIME-SEQUENCING-RETRY-LIFECYCLE.md); dependency-free conformance implementation exists | BLK-004, BLK-005, BLK-012 | UNK-013, UNK-018 | Deterministic generation specification and vectors | Slice A; comparable stream |
| BLK-008 | Stream and artifact digest algorithm | Resolved as documentation design by [R7](R7-BENCHMARK-RECORDS-ARTIFACTS-INSTRUMENTATION-FAULTS.md) and [R14](R14-CANONICAL-WORKLOAD-STREAM-DIGEST.md): SHA-256, domains, canonical workload bytes, and documentation vectors are frozen; dependency-free digest conformance implementation exists | BLK-006, BLK-007 | UNK-018, UNK-022 | Algorithm, canonical digest inputs, and vectors | Stream/artifact validation |
| BLK-009 | Workload-manifest physical serialization | Resolved as documentation design by [R16](R16-WORKLOAD-MANIFEST-SERIALIZATION-CONTRACT.md); dependency-free manifest and validator conformance implementation exists | BLK-006–BLK-008 | UNK-019 | Versioned serialization and fail-closed validator rules | Descriptive execution |
| BLK-010 | Benchmark-record physical serialization and validation | Resolved as documentation design by [R7](R7-BENCHMARK-RECORDS-ARTIFACTS-INSTRUMENTATION-FAULTS.md); executable schema/validator remains gated | BLK-008, BLK-025 | UNK-022 | Versioned serialization, validation and correction rules | Result admissibility |
| BLK-011 | Sequencing-gap treatment | Resolved by [R3](R3-IDENTITY-TIME-SEQUENCING-RETRY-LIFECYCLE.md) | SEM, BLK-013 | UNK-017 | Rules for failed/abandoned candidates and replay checking | Slice C correctness |
| BLK-012 | Retry and uncertain-outcome behavior | Resolved by [R3](R3-IDENTITY-TIME-SEQUENCING-RETRY-LIFECYCLE.md) | SEM, BLK-013 | UNK-016 | Request lifecycle, duplicate handling, retry and uncertain classification | Correctness gate |
| BLK-013 | Minimum append/replay recovery rules | Resolved by [R1](R1-PHYSICAL-RECORD-INTEGRITY-RECOVERY-REQUIREMENTS.md) | SEM, BLK-002 | UNK-001, UNK-015–UNK-017 | Physical invariants for boundaries, terminal damage, scanning and fail-closed replay | Slice C |
| BLK-014 | Target OS and hardware/environment | Closed for R4 conditional planning by the owner-approved [R4 evidence boundary](R4-FEDORA-44-BOSGAME-M5-TARGET-AND-PLATFORM-DURABILITY-CONTRACT.md); execution environment and final placement remain incomplete | — | UNK-014, UNK-020 | Reviewed target and completed environment record | Series freeze and execution, not R5 documentation design |
| BLK-015 | Filesystem/storage stack and platform durability contract | Open for dependent claims/execution; planning-unblocked for conditional R5 design by [R4](R4-FEDORA-44-BOSGAME-M5-TARGET-AND-PLATFORM-DURABILITY-CONTRACT.md); final placement, PLP/protection, and empirical survival remain unverified; R5 now fixes the B1 append/sync and post-boundary finalization/commit design; this does not supply platform-survival evidence | BLK-014 | UNK-014, UNK-021 | Stack-specific D2/D3 promise and limits | Canonical D2/D3 claims and execution, not conditional R5 design |
| BLK-016 | B0 implementation profile | Resolved as documentation design by [R5](R5-B0-B1-PHYSICAL-PROFILES-AND-ADAPTER-CONTRACTS.md); [R11](R11-SLICE-B-CLOSURE-AND-NEXT-GATE.md) closes the reviewed bounded process-local D0 implementation, while observations/evidence remain gated | BLK-001, BLK-006–BLK-007 | UNK-020 | Exact lower-bound operations and accounting | Slice B/baseline validation |
| BLK-017 | B1 append and synchronization APIs | Resolved as documentation design by [R5](R5-B0-B1-PHYSICAL-PROFILES-AND-ADAPTER-CONTRACTS.md) and its focused contract; BLK-015/implementation/evidence remain gated | BLK-001, BLK-003, BLK-013–BLK-015 | UNK-014, UNK-020 | Complete API/error/sync/grouping and finalized-commit mapping | Slices C–E |
| BLK-018 | SQLite/RocksDB versions, builds, bindings and configurations | Resolved as documentation design by [R6](R6-SQLITE-ROCKSDB-EXECUTION-PROFILES.md); toolchain, implementation, effective validation and evidence remain gated | BLK-014–BLK-015 | UNK-020, UNK-021 | Exact reproducible profiles with verified effective settings | Slices F/G; series freeze |
| BLK-019 | Adapter mappings for every baseline | Design complete: B0/B1 by R5 and B2/B3 by [R6](R6-SQLITE-ROCKSDB-EXECUTION-PROFILES.md); implementation and empirical validation open | BLK-001, BLK-004–BLK-005, BLK-011–BLK-012, BLK-016–BLK-018 | UNK-020, UNK-021 | Versioned semantic-to-physical mapping and equivalence classification per baseline | Baseline equivalence |
| BLK-020 | Benchmark harness architecture | Resolved by R9 only for Slice A and extended through the three reviewed correctness crates; R29 closes semantic mapping and freezes only the fourth-crate capture/preflight boundary, while harness composition and execution remain blocked | BLK-009–BLK-010, BLK-019 | UNK-020, UNK-022 | Experimental component and dependency-boundary design | Slice A authorization; later slices separately gated |
| BLK-021 | Instrumentation and overhead method | Resolved as documentation design by R7; implementation/effective capture remains gated | BLK-014, BLK-020 | UNK-022 | Named instruments, scope, calibration/bounding and loss rules | Descriptive/confirmatory execution |
| BLK-022 | Fault-injection mechanisms | Process termination and offline-condition design resolved by R7; kernel crash, physical power/reset and I/O-error apparatus remain owner/BLK-015 blocked | BLK-013–BLK-015, BLK-017–BLK-019 | UNK-015, UNK-021 | Injection-point mapping, apparatus validation and coverage | Recovery readiness |
| BLK-023 | Numeric threshold registry and rationale | Resolved for this documentation decision by the prospective owner-approved R8 `thresholds-v1`; implementation and admissibility remain gated | Primary-cell design, baseline profiles | UNK-008 | Reviewed versioned per-cell values with evidence/rationale | Confirmatory execution only |
| BLK-024 | Estimator, interval, repetitions, stopping and run order | Resolved as documentation design by [R8](R8-PRIMARY-MATRIX-THRESHOLDS-AND-STATISTICAL-PLAN.md); implementation/effective validation remain gated | BLK-014, primary-cell design | UNK-008 | Frozen analysis specification | Confirmatory execution only |
| BLK-025 | Artifact storage and retention | Resolved as documentation design by R7; external service/provisioning deliberately unselected | BLK-008, repository constraints | UNK-019, UNK-022 | Layout, durable references, retention/redaction and supersession rules | Execution/result admissibility |
| BLK-026 | Reproducible toolchain/build configuration | Resolved through the existing three-member Rust 1.89.0 correctness workspace; R19 does not authorize a fourth member and later/native/benchmark-series builds remain open | BLK-014, BLK-018 | UNK-020, UNK-022 | Rust 1.89.0, target, flags, lock/build identity and offline reproduction procedure | Slice A authorization; series separately gated |
| BLK-027 | Cargo/workspace and CI bootstrap authorization | Resolved only through the reviewed three-member correctness workspace; R19 does not authorize the candidate fourth member or executable capture expansion | BLK-020, BLK-026, section 6 | UNK-020, UNK-022 | Reviewed layout, empty dependency allowlist, exact CI plan, and approval record | First Slice A code creation after R9 merge |

## 3. Dependency ordering

The work is a directed sequence of independently reviewable freezes, not one resolve-everything gate.

| Order | Predecessor → dependent decision | Reason |
|---|---|---|
| 1 | Frozen semantics → BLK-002/013 → BLK-001/003 | Required integrity and recovery behavior precede framing; integrity policy precedes an algorithm. |
| 2 | Lifecycle semantics → BLK-004/005/011/012 | Physical identity, time, gaps and retries must preserve the semantic distinctions. |
| 3 | BLK-004–006/011/012 → BLK-007 → BLK-008/009 | Byte generation, identities and references precede reproducible streams, manifests and digests. |
| 4 | BLK-014 → BLK-015 → BLK-017/018/022 | Target stack determines meaningful sync and fault mechanisms; its durability contract precedes canonical D2/D3 claims. |
| 5 | BLK-001/004/005/011/012/015–018 → BLK-019 | Adapters can claim equivalence only after both semantic and physical sides are known. |
| 6 | BLK-008–010/019/025 → BLK-020/021 | The harness must preserve validated inputs, mappings, measurements and artifacts. |
| 7 | BLK-014/018/026 plus all effective configuration → series freeze | Environment and executable versions are frozen before any benchmark series. |
| 8 | Primary cells and BLK-023/024 → confirmatory freeze | Thresholds and analysis are fixed before observations, never after them. |
| 9 | Fixtures/oracles/adapters/BLK-022 → correctness and recovery pass → performance interpretation | Correctness gates speed; fault coverage must match the claimed contract. |

## 4. Recommended readiness sequence

Each increment ends in its own review; none silently authorizes a later increment.

| Increment | Question and inputs | Required artifact | Deliberately retained | Gate / implementation authorization | Validation | Revisit condition |
|---|---|---|---|---|---|---|
| R1 — minimum physical record, integrity-policy, and replay/recovery requirements (**complete**) | What must one record guarantee? Inputs: semantic envelope, durability and crash contracts. | [Requirements/research record](R1-PHYSICAL-RECORD-INTEGRITY-RECOVERY-REQUIREMENTS.md) resolving BLK-002 and BLK-013 and constraining BLK-001/003. | Encoding and checksum algorithm were retained for later R5 resolution; no code is authorized by R1. | Review semantic coverage and fail-closed rules; **no code**. | Examples reviewed against every recovery invariant. | A requirement is unimplementable or misses a declared fault. |
| R2 — deterministic workload bytes, identity/reference inputs, and digests (**complete**) | How can an equivalent operation stream be regenerated and verified? Inputs: workload contract plus R1 boundaries. | [Generator/digest requirements and reference-vector plan](R2-DETERMINISTIC-WORKLOAD-BYTES-IDENTITY-REFERENCES-DIGEST-REQUIREMENTS.md) constraining open BLK-006–009; algorithm selections require separate recorded rationale. Physical-record vectors are not stable until BLK-001 and every applicable BLK-003 algorithm/profile are separately resolved and versioned. | Runtime harness and event encoding beyond digest-input boundaries. | Byte-for-byte cross-implementation review; **no code**. | Hand/reference cases and platform-independence review. | Ambiguous canonical input or substitution cannot be detected. |
| R3 — identity, time, gaps, retry, and uncertain-outcome lifecycle (**complete**) | When and by whom are values assigned and failures classified? Inputs: semantic/lifecycle contracts and R1. | [Lifecycle contract](R3-IDENTITY-TIME-SEQUENCING-RETRY-LIFECYCLE.md) resolving BLK-004/005/011/012 and constraining BLK-007. | Concrete storage APIs. | No semantic ambiguity for Slice A/C; **no code**. | State-transition and failure-scenario review. | Contradiction with REQ-003/004/006/009/013. |
| R4 — target environment and platform durability (**complete for conditional planning**) | What exact stack is studied and what can D2/D3 claim? Inputs: environment template and platform primary sources/research. | [Owner-approved evidence boundary, selected-target profile, conditional platform contract, sources, and fault matrix](R4-FEDORA-44-BOSGAME-M5-TARGET-AND-PLATFORM-DURABILITY-CONTRACT.md). BLK-014 is closed for R4 planning; BLK-015 remains open for dependent claims and execution. | Repository-retained provenance, final path placement, PLP/protection, empirical survival and performance claims; R5 now freezes post-boundary finalization/commit as design only. | Conditional planning boundary reviewed; no D2/D3 survival promise and **no code**. | Source/cross-reference and contract-to-fault matrix review. | Later evidence contradicts the profile, final paths are created/remounted, or stack/configuration changes. |
| R5 — B0/B1 physical profiles and adapter contracts (**complete**) | What conditional lower bounds and OS operations preserve the declared cells? Inputs: R1–R4 and baseline contract. | [R5 authority](R5-B0-B1-PHYSICAL-PROFILES-AND-ADAPTER-CONTRACTS.md) plus [focused physical contract](R5-PHYSICAL-RECORD-INTEGRITY-AND-RECOVERABLE-COMMIT-CONTRACT.md), resolving BLK-001/003/016/017 and B0/B1 BLK-019. | SQLite/RocksDB details; implementation, execution, final placement, PLP/protection verification, and empirical survival. | Complete design mapping; **documentation only, no code or execution**. | Byte/vector, scan-classification, lifecycle/recovery, and error-path review. | A selected mechanism cannot meet the intended D-mode or later evidence contradicts an assumed physical fact. |
| R6 — SQLite/RocksDB execution-profile freeze (**complete**) | Which exact reproducible builds and mappings are eligible? Inputs: R3/R4 and official version semantics. | [R6 authority](R6-SQLITE-ROCKSDB-EXECUTION-PROFILES.md), resolving BLK-018 and B2/B3 design portions of BLK-019. | Toolchain, implementation, effective validation and empirical equivalence. | Conditional/diagnostic status preserved; **no code or execution**. | Source-to-decision, complete mapping, configuration/error/recovery and exclusion review. | Version, build, binding, default or stack changes. |
| R7 — benchmark records, artifacts, instrumentation and faults (**complete at the owner-dependent apparatus boundary**) | How are runs captured, validated, retained and faulted without corrupting inference? Inputs: R1–R6 and record contracts. | [R7 authority](R7-BENCHMARK-RECORDS-ARTIFACTS-INSTRUMENTATION-FAULTS.md): BLK-010/021/025 design resolution, partial BLK-008/022 resolution, and BLK-020 constraints. | Owner-dependent destructive apparatus, numeric thresholds and executable harness. | Descriptive design review; **no code by itself**. | Schema examples, provenance graph and apparatus-validation plan review. | Missing state/provenance or overhead cannot be bounded. |
| R8 — primary matrix, thresholds and statistical plan (**complete as documentation design**) | What observations can decide predeclared claims? Inputs: interpretation contract, frozen profiles/environment. | [R8 authority](R8-PRIMARY-MATRIX-THRESHOLDS-AND-STATISTICAL-PLAN.md): exact 40-cell candidate matrix, prospective owner-approved threshold registry, decision table, and BLK-024 statistical freeze. | Exploratory expansions. | Confirmatory-design approval; **no code by itself**. | Completeness and independent statistical review. | Unsupported rationale, excessive uncertainty, or material design change. |
| R9 — workspace, harness, CI and first-slice authorization (**complete as prospective authorization**) | Can exactly one approved slice be implemented reproducibly? Inputs: R1–R8, resolved/versioned R5 BLK-001/003 profiles, and stable R5/R7 vectors. | [R9 authority](R9-WORKSPACE-HARNESS-CI-AND-SLICE-A-AUTHORIZATION.md): BLK-020/026 Slice A disposition, prospective BLK-027 resolution, section 6 checklist, and approval record. | Every later slice and all execution not expressly approved. | **Authorizes Slice A only after R9 is reviewed and merged.** | Documentation/link checks; frozen future format, lint, unit/vector and CI checks. | Dependency/layout/toolchain change or newly exposed semantic ambiguity. |
| R10 — Slice A closure and Slice B authorization (**complete as prospective authorization**) | Did merged Slice A satisfy R9, and can the minimum B0 mechanism be implemented without unlocking execution? | [R10 authority](R10-SLICE-A-CLOSURE-AND-SLICE-B-AUTHORIZATION.md): exact-head closure audit, generator-blocker disposition, and prospective D0-only Slice B boundary. | Workload generation, observations, executable harness/capture, persistence, and later slices. | **Authorizes bounded Slice B code only after R10 is reviewed and merged.** | Documentation/link checks; existing R9 Rust CI applies to the later implementation. | Any accounting ambiguity or workspace/dependency/toolchain/layout expansion. |
| R11 — Slice B closure and next gate (**complete as prospective authorization**) | Did merged Slice B satisfy R10, and what is the smallest remaining prerequisite? | [R11 authority](R11-SLICE-B-CLOSURE-AND-NEXT-GATE.md): exact-head B0 audit and BLK-006/007 prerequisite analysis. | Generator implementation, BLK-008/009 completion, Slice C, execution, persistence, and later work. | **Authorizes only the BLK-006/007 documentation/research freeze after R11 is reviewed and merged.** | Documentation/link checks plus unchanged R9 Rust validation. | Generator ambiguity, missing independent vectors, dependency need, or changed staged authority. |
| R12 — deterministic generator specification and vectors (**complete as documentation design**) | Can BLK-006/007 be frozen without implementation or workload generation? | [R12 authority](R12-DETERMINISTIC-GENERATOR-SPECIFICATION-AND-VECTORS.md): versioned payload, typed identity, envelope/reference, logical-time, failure, and documentation-vector rules. | Generator implementation, BLK-008/009, Slice C, workload generation, and execution. | **Resolves BLK-006/007 as documentation design only; authorizes no code or execution.** | Documentation/link checks plus unchanged R9 Rust validation. | Vector disagreement, missing canonical input, or implementation-only oracle requirement. |
| R13 — R12 closure and next gate (**complete as prospective authorization**) | Did exact reviewed R12 satisfy R11, and which remaining dependency is smallest? | [R13 authority](R13-R12-CLOSURE-AND-NEXT-GATE.md): exact-head R12 audit and generator/BLK-008/BLK-009/Slice C prerequisite analysis. | Generator implementation, BLK-009, Slice C/B1, workloads, execution, persistence, benchmarks, and later work. | **Authorizes only a focused BLK-008 documentation freeze after R13 is reviewed and merged.** | Documentation/link checks, unchanged R9 Rust validation, dependency and exclusion audits. | BLK-008 cannot be independent of BLK-009, documentation vectors disagree, an executable oracle is needed, or staged authority changes. |
| R14 — canonical semantic operation and workload-stream digest freeze (**complete as documentation design**) | Can BLK-008 be completed from R7/R12 without executable or manifest work? | [R14 authority](R14-CANONICAL-WORKLOAD-STREAM-DIGEST.md): `EXP-0001-SEMANTIC-OP-v1`, ordered stream framing, literal SHA-256 vectors, and BLK-009 metadata obligations. | Generator implementation, BLK-009, Slice C/B1, workloads, execution, persistence, benchmarks, and later work. | **Resolves BLK-008 as documentation design only and grants no prospective authorization.** | Documentation/link checks, unchanged R9 Rust validation, vector/canonicalization and exclusion audits. | Vector disagreement, semantic ambiguity, executable-oracle need, or staged authority change. |
| R15 — R14 closure and BLK-009 next gate (**complete as prospective authorization**) | Did exact reviewed R14 satisfy R13, and can BLK-009 now be frozen independently? | [R15 authority](R15-R14-CLOSURE-AND-BLK-009-NEXT-GATE.md): exact-head closure and manifest identity, profile, digest, count, provenance, JCS, correction, and fail-closed prerequisite audit. | Manifest/validator or generator implementation, generated artifacts, Slice C/B1, execution, persistence, faults, benchmarks, and later work. | **Authorizes only a focused BLK-009 workload-manifest serialization documentation freeze after R15 is reviewed and merged.** | Documentation/link checks, unchanged R9 Rust validation, dependency and changed-path exclusion audits. | Authority conflict, missing canonical field/vector, executable-oracle need, or staged authority change. |
| R16 — workload-manifest serialization contract (**complete as documentation design**) | Can BLK-009 be uniquely frozen without executable work? | [R16 authority](R16-WORKLOAD-MANIFEST-SERIALIZATION-CONTRACT.md): `EXP-0001-WORKLOAD-MANIFEST-JCS-v1`, closed bindings, fail-closed validation, immutable supersession, and literal vectors. | Manifest/generator implementation, artifacts, Slice C/B1, execution, persistence, faults, benchmarks, and later work. | **Resolves BLK-009/UNK-019 only as documentation design and grants no prospective authorization.** | Documentation/link checks, unchanged R9 Rust validation, binding/canonicalization/exclusion audits. | Any executable implementation or authority conflict. |

R9 and R10 authorized the now-merged Slice A and Slice B implementations. [R11](R11-SLICE-B-CLOSURE-AND-NEXT-GATE.md) records that Slice B passed its continuation gate as correctness-validation evidence only; R12 completes that documentation freeze without authorizing implementation. R16 resolves BLK-009 only as documentation design and grants no prospective authorization. Slice C, execution, benchmark, persistence, fault, and durability work remains unauthorized.

## 5. Conditional staged-implementation proposal

All eventual code remains under `/experiments/`; nothing graduates to `/crates/` without evidence, an ADR when warranted, and a
specification. The ordering is conditional: A is the candidate first slice; B–E earn continuation in order because each isolates a
new mechanism. F and G may follow C once common adapter validation is stable and may be ordered independently; they do not have to
wait for D3. H is introduced incrementally for the first persistence slice, not postponed as a monolith. A failed gate can stop the
sequence, and no slice is authorized by this plan.

| Slice | Research question / D-mode | Prerequisites; inputs → outputs | Correctness tests | Initially enabled cells | Exclusions | Evidence and continuation gate |
|---|---|---|---|---|---|---|
| A — deterministic physical-record fixtures and validators | Can semantic operations be encoded as physical records and can those records be generated and rejected deterministically? No benchmark D-mode. | Reviewed R1–R3 outputs **plus resolved and versioned BLK-001 and every BLK-003 algorithm/profile applicable to the slice, each backed by stable test vectors** → physical-record fixtures/validators and oracle outputs. R1's constraints and R2's vector plan alone are insufficient. | Golden-vector and round-trip tests; malformed/truncated/corrupt physical-record tests; identity/reference/order checks. | None; validation only. | Semantic-only fixture work, append, sync, performance claims, production schema. | Reproducibility/correctness evidence; approve B only if the frozen physical vectors and independent oracle pass. |
| B — B0 in-memory lower bound | What is construction/sequencing/accounting overhead without persistence? D0 only. | A, B0 profile, existing validation workspace → process-local mechanism and correctness tests. Comparable observations additionally require frozen BLK-006/007 inputs, dependent digest/manifest outputs, and a separate execution authorization. | Unique monotonic assignment, no invention/duplication, D0 never labeled canonical. | Reference single-producer P1–P3 minimal-envelope D0, descriptive first. | Recovery/durability and database adapters. | B0 mechanism evidence; approve C only if accounting and D0 semantics validate. |
| C — B1 raw append/replay | What does OS-buffer append and deterministic replay cost? D1 only. | A/B, R4/R5, fault/result subset → append artifact and replay/oracle record. | Boundaries, short writes/errors, terminal damage, gaps, replay order; no D1 canonical claim. | Reference P1–P3 minimal-envelope D1, descriptive first. | Stable-media claims and grouping. | D1/replay evidence; approve D only after B1 mapping, fault apparatus and recovery oracle pass. |
| D — B1 per-event sync | What is per-event declared synchronization behavior/cost? D2 only under the platform contract. | C plus R4 sync contract → synchronized records and fault results. | Ack/error ordering and every claimed D2 fault; recover every canonical ack. | D2 subset matching validated D1 cells; confirmatory only after section 7. | D3 and claims beyond the target stack. | D2 correctness/cost evidence; approve E only if D2 is valid and D3 remains a useful separate question. |
| E — controlled B1 grouping | What trade space follows from observable controlled grouping? D3 only. | D plus frozen membership/window/count policy → group outcomes and membership evidence. | Shared sync/outcome, no partial group ack, per-event latency includes formation wait, faults at group boundaries. | Small predeclared D3 subset; expand only after validation. | Atomic multi-event transactions and opaque grouping. | D3 trade-space evidence; continue only if group semantics and measurement are valid. |
| F — SQLite adapter | Which baseline profiles are semantically comparable and at what cost? D1; D2 only conditionally validated. | A/C, R4/R6 mapping → adapter/config/equivalence and results. | Stream mapping, effective settings, error/retry, replay and claimed-fault oracle. | Cells equivalent to validated subject cells. | Strict D3 via transactions; product conclusions. | Conditional baseline evidence; next adapter requires stable shared validation, not a win. |
| G — RocksDB adapter | Same question for the selected RocksDB WAL profile. D1; D2 only conditionally validated. | Same class as F with its independently frozen profile. | Same obligations plus verified WAL/write options and recovery mapping. | Equivalent validated cells only. | `WriteBatch` as strict D3 and opaque grouping claims. | Conditional baseline evidence; continuation requires valid equivalence and reproducibility. |
| H — fault/recovery and result-capture integration | Can each persistence slice produce admissible, retained correctness evidence? Applicable D1–D3, never a new mode. | Begins with C; R7 artifacts/faults/instrumentation → validated raw records and provenance. | Apparatus self-tests, full applicable fault matrix, schema/provenance/digest validation, missing-data behavior. | Diagnostic apparatus cells first; later the exact approved matrix. | New storage mechanisms or post-hoc analysis. | Admissibility evidence; performance interpretation stays blocked until correctness and the applicable execution gate pass. |

## 6. First implementation authorization gate

Creating any Cargo file or Rust code requires one repository-recorded reviewer approval that names exactly one slice and confirms:

- the first slice is approved and bounded;
- every physical contract required by that slice is resolved, versioned, and linked;
- for any physical-record fixture, validator, generation, round-trip, truncation, or corruption work, BLK-001 and every applicable
  integrity-algorithm/profile portion of BLK-003 are resolved and versioned, and their stable test vectors are linked;
- stable fixture/test-vector inputs and a defined independent correctness oracle exist;
- repository layout and dependency boundaries keep experimental code under `/experiments/`;
- the toolchain, target and build configuration are frozen reproducibly;
- the CI validation plan states format, static, unit, property/vector, and documentation checks applicable to the slice;
- an explicit allowed direct-dependency list records purpose, version policy, alternatives, license/build implications, and why standard facilities are insufficient;
- no unresolved semantic ambiguity could invalidate the slice;
- rollback/revisit conditions and prohibited later slices remain explicit; and
- approval identity, date, reviewer, addressed blocker IDs and exact repository state are recorded.

The gate fails closed. In particular, R1 requirements, R2 vector planning, or R3 lifecycle resolution cannot substitute for the
BLK-001 and applicable BLK-003 freezes required by physical-record work. R9 satisfied this gate for Slice A and R10 for Slice B. R11 closes Slice B, R12 completes its generator documentation freeze, R14 completes the digest documentation freeze, and R16 completes the BLK-009 documentation freeze without authorizing later work. None satisfies a later implementation or execution gate.

## 7. Execution gates

### Descriptive execution

Descriptive execution may record mechanism behavior only after the selected runnable cell has validated workload identity, subject or
adapter mapping, environment/series identity, applicable correctness oracle, instrumentation/overhead assessment, and validated raw
record/artifact path. It must be labeled descriptive, diagnostic, or exploratory. If numeric thresholds or analysis rules remain open,
it cannot support threshold-based claims.

### Confirmatory execution

Confirmatory execution additionally requires completed versioned threshold-registry entries; a frozen estimator, interval,
repetition, stopping, multiplicity and run-order method; a complete predeclared primary-cell matrix; exact environment and series
identity; a validated workload stream; validated subject and every applicable baseline adapter; passed correctness and recovery gates;
exact fault coverage matching each claim; and validated artifact retention and benchmark records. Deviations start a new series or
downgrade evidence as the interpretation contract requires.

A descriptive result can never be retroactively relabeled confirmatory. A later confirmatory design requires new observations gathered
under that already-frozen design.

## 8. Decision records and traceability

Every R increment must record: blocker IDs addressed; affected unknowns; linked requirements and experiment sections; decision or
research-record identity; alternatives considered; evidence actually available; assumptions; unresolved questions; revisit conditions;
status; and immutable supersession links. A freeze is not `resolved` until its required output is reviewed and repository-recorded.

An ADR is required when a choice materially constrains durable architecture, public interfaces, cross-experiment correctness semantics,
or a format proposed for graduation. Experiment-local encodings, algorithms, versions, bindings, flags, target environments, harness
mechanics, and statistical freezes belong in versioned experiment research/configuration records unless promoted. An ADR must not turn
an untested performance candidate into validated architecture; empirical promotion cites admissible evidence.

## 9. Phase reconciliation and next increment

Phase 0's documented exit criteria are now satisfied: EXP-0000 is complete, EXP-0001 has a reproducible semantic benchmark plan and
correctness criteria, baseline families are identified, and target environments can be recorded consistently. The project therefore
enters **Phase 1 planning/readiness**, not experimental execution. Slices A, B/B0, A2, and raw D1 append/replay provide bounded implementation/correctness-validation evidence only. R19 closes that correctness tranche; EXP-0001 remains non-executable as a workload or benchmark.

R1 is complete through the [physical-record, integrity, and recovery requirements](R1-PHYSICAL-RECORD-INTEGRITY-RECOVERY-REQUIREMENTS.md). It resolved BLK-002 and BLK-013 while leaving later decisions; R3 subsequently resolved BLK-011/012 and R5 resolved BLK-001/003 as documentation design. Platform evidence and concrete fault mechanisms remain open. [R2](R2-DETERMINISTIC-WORKLOAD-BYTES-IDENTITY-REFERENCES-DIGEST-REQUIREMENTS.md) is complete as requirements/reference-vector planning and constrains BLK-006–009; BLK-006/007 are resolved by R12 and BLK-008 by R14 as documentation design; BLK-009 is resolved as documentation design by R16; external-dependency-free generator/manifest conformance implementation exists with reviewed workspace path dependencies, while generated workload and benchmark execution remain absent. [R3](R3-IDENTITY-TIME-SEQUENCING-RETRY-LIFECYCLE.md) is complete: it resolves BLK-004/005/011/012 and further constrains open BLK-007.

R4 is complete for conditional planning through the owner-approved [target and platform durability evidence boundary](R4-FEDORA-44-BOSGAME-M5-TARGET-AND-PLATFORM-DURABILITY-CONTRACT.md). BLK-014 is closed for that planning purpose; BLK-015 remains open for dependent claims and execution. R5 completes B0/B1 design and resolves BLK-001/003/016/017. [R6](R6-SQLITE-ROCKSDB-EXECUTION-PROFILES.md) completes B2/B3 design, resolving BLK-018 and the remaining design portion of BLK-019 without empirical proof. R8 records the completed accountable-owner threshold decision. [R9](R9-WORKSPACE-HARNESS-CI-AND-SLICE-A-AUTHORIZATION.md) authorized the reviewed Slice A implementation. [R10](R10-SLICE-A-CLOSURE-AND-SLICE-B-AUTHORIZATION.md) authorized the now-reviewed bounded Slice B implementation. R11 then authorized R12’s documentation freeze. R16 resolves the focused BLK-009 documentation freeze. R18 then closes A2 and authorizes only bounded raw D1 append/replay correctness. No generated workload, capture, adapter, descriptive or confirmatory execution, benchmark, fault action, machine change, D2/D3 claim, or later slice is authorized.

### Slice A2 closure (R17)

GitHub issue #63 is the prospective approval for `EXP-0001-A2/workload-conformance-v1` against
base `27e5141d48991ecce2770bfa1720d03ac4ae67da`. R17 records the bounded dependency-free
implementation and correctness-vector closure. This extends BLK-020/026/027 only to the second
conformance workspace member and implements BLK-006–009 only at the R12/R14/R16 contract boundary.
It does not authorize Slice C/B1, execution, benchmark/capture, persistence, faults, durability,
adapters, production code, or a later tranche.

### R18 — A2 closure and bounded Slice C/B1 readiness

R18 preserves PR #64 historical implementation head
`d2ee72aa4ff047d4cfcaa1df82d83f13566568f2` and merge
`9b5d89a36ed71d38420e9ae19f59d441a9d927aa`, while binding final A2 closure to corrective PR #68
exact reviewed head `fcaf7f14c94df5a6cda1aeeb283b6726551d1844`. Both required exact-head workflows
passed, and PR #68 merged as `79cbd64a436b104835a4279c07ba2777fb06cddb`; Slice A2 is closed as
bounded correctness evidence. Corrected M01 remains the canonical positively valid R7-backed
vector with its 1,274-byte stream and independent 1,152-byte workload-manifest artifact fixtures.

R18 historically authorized the now-closed Slice C raw D1 append plus reopen/replay correctness subset in a
third external-dependency-free workspace member with exactly one reviewed workspace path dependency on `exp1-record-format` under the unchanged R9 toolchain and CI. This limited reuse
extends BLK-017 implementation and BLK-020/026/027 only to that crate. It intentionally stays below
BLK-015: no synchronization, canonical recovery, D2/D3, survival, fault, workload/benchmark
execution, capture, performance, adapter, or production claim is authorized. The exact API,
framing/integrity, accepted-prefix, fail-closed, test, and exclusion boundary is normative in R18.

The authorized crate now implements that bounded boundary and its deterministic correctness gate.
This closes only the raw D1 append/physical replay implementation subset of BLK-017 and extends the
existing workspace validation boundary to the third member. BLK-015 and all synchronization,
stable-storage, canonical recovery, D2/D3, generated workload, execution, benchmark/capture, fault,
adapter, production, performance, and later-tranche work remain open or unauthorized.


### R19 — Slice C/B1 closure and blocked descriptive D1 harness

[R19](R19-SLICE-C-B1-CLOSURE-AND-DESCRIPTIVE-D1-HARNESS-READINESS.md) binds R18 to PR #71
reviewed head/merge and closes PR #74 reviewed head
`5c448695f4e460cab57eaadd7f7a83bfce1559ab` at merge
`ef29804347faa812502f855e5cc3ffee6f4901c2`; both exact-head workflows passed. This is only raw
D1 append and deterministic physical replay implementation/correctness evidence.

R19 finds the maximum candidate fourth binary and its three reviewed path dependencies, M01/D1
sequence, replay oracle, R7 artifact graph, lifecycle, tests, and exclusions sufficiently bounded.
It does not authorize implementation because R7's named Linux capture interfaces have no uniquely
selected safe realization under the empty external-dependency allowlist and unsafe-code prohibition,
and the semantic-operation-to-physical-record mapping was not frozen when R19 was decided; R20 now resolves that mapping only. Caller/authority identity
assignment is selected and requires a complete validated manifest. BLK-020/021/026/027 and UNK-022
remain open for the exact owner-reviewed capture decision; R20 separately resolves the mapping as documentation design. No workload materialization, capture, execution, or later tranche is
authorized.

## R20 semantic mapping gate update

[R20](R20-SEMANTIC-OPERATION-TO-PHYSICAL-RECORD-MAPPING.md) resolves R19's semantic-operation-to-physical-record blocker as documentation design. It freezes exactly one validated SOP1 to one `STRUCTURAL-0` RF1 type-3 provisional record, with complete SOP1 stable-core preservation and later-ingestion sequence/physical-ordinal inputs. A later pure public `exp1-raw-append-replay::mapping` implementation is prospectively authorized with direct path dependencies on `exp1-record-format` and `exp1-workload-conformance`; only its manifest and the matching lock entry may change, the other crates remain unchanged, and append integration is excluded. That mapper must pass R20's independent correctness gate. R29 subsequently freezes the live Linux capture-interface and bounded-unsafe decision; descriptive D1 harness composition and all execution remain blocked. No fourth crate, external dependency, D2/D3, `fsync`, canonical recovery, fault work, workload/benchmark execution, or evidence claim is authorized.

The pure mapper and its deterministic tests implement only the locally decidable subset of that gate. Duplicate reference bytes reject through SOP1 validation, but future, self, and cross-stream checks require prior-event membership and stream-locality context that R20 did not freeze. R21 freezes the locally decidable subset of that context, R22 resolves cross-segment eligibility, and R23 freezes closed-scope proof. R25 records that R24’s prospective implementation authorization is superseded because the unchanged v1 authorities cannot form its required valid bootstrap-to-reference stream; R26 freezes and prospectively authorizes only the prerequisite v2 conformance/validator increment, and the complete R20 gate remains open. This is bounded implementation/correctness-validation evidence only; it does not alter the still-open live Linux capture decision or authorize harness construction or execution.

## R21 reference-context gate update

[R21](R21-REFERENCE-VALIDATION-CONTEXT.md) freezes an immutable typed identity catalog constructed from supplied conformance-validated semantic streams and a separate mutable accepted-prefix state for one selected stream. It fixes bounds, collision rules, transactional state advance, and locally decidable precedence. R22 supersedes the cross-segment ambiguity, and R23 now proves complete closed stream scope. R25 supersedes R24’s bounded implementation authorization after its v1 bootstrap premise failed; the full R20 gate remains open. Live Linux capture remains an independent blocker and no descriptive D1 harness is authorized.

## R22 cross-segment gate update

[R22](R22-CROSS-SEGMENT-REFERENCE-RULE.md) fully resolves only the cross-segment governance question:
references are same-stream and strictly segment-local, and known same-stream targets in the other
segment receive the new `E-REFERENCE-CROSS-SEGMENT` disposition. Total WS1 position remains the
accepted-prefix order and is not reference eligibility. At the R22 checkpoint the complete closed stream-scope proof was unresolved. R23 resolves it, but R25 supersedes R24’s still-pending bounded implementation authorization; complete R20 closure, live Linux capture, descriptive D1 harness, workload or benchmark execution, and later tranches remain gated or unauthorized.

## R23 closed stream-scope gate update

[R23](R23-CLOSED-STREAM-SCOPE-PROOF.md) freezes the canonical cell-scoped descriptor, ordered
manifest/stream membership, digest and artifact bindings, exact supplied-set equality, bounds,
fail-closed construction, and `CrossStream`/`Missing` classification boundary. The governance
blocker is fully closed. A later pure reference-context implementation remains subject to separate
authorization, exact-head review, and CI; no harness, capture, execution, or later work follows.

## R24 reference-context implementation gate update (superseded authorization)

[R24](R24-REFERENCE-CONTEXT-IMPLEMENTATION-AUTHORIZATION.md) confirms that R21–R23 close all
reference-context governance prerequisites and prospectively authorizes exactly one bounded pure
extension in the existing `exp1-raw-append-replay` mapper. Only its frozen source/test paths may
change; manifests, lockfile, authority crates, dependencies, append/reopen, Linux capture, harnesses,
and execution remain unchanged or excluded. Completion requires the implementation, every frozen
synthetic correctness class, synchronized documentation, exact-head review, and both existing CI
workflows green. R25 supersedes this prospective authorization after closed PR #91 proved its valid-reference gate impossible under v1; R24 is not complete or merged evidence, and the complete R20 correctness gate remains open.


## R25 bootstrap causal-reference governance correction

[R25](R25-BOOTSTRAP-CAUSAL-REFERENCE-GOVERNANCE.md) records the failed R24 assumption exposed by
closed, unmerged PR #91: R16's uniform envelope and positive scalar causal cardinality, R12's
positive-target rule, R21's first-operation accepted prefix, and R22's segment locality make a valid
bootstrap-to-reference v1 stream impossible; R23 forbids repairing that stream outside its exact
manifest binding. All v1 vectors and bytes remain unchanged. R25 freezes only prospective v2
semantics and a per-segment bootstrap/subsequent cardinality-policy representation. It supersedes
R24 for implementation authorization only, authorizes no code. R26 now supplies the separate v2
conformance/validator-and-vector authorization; a bounded reference-context implementation still
requires another later authorization. The complete R20 gate, capture, harness, and execution remain open.


## R26 v2 conformance/validator gate update

[R26](R26-V2-CAUSAL-REFERENCE-CONFORMANCE-AND-VALIDATOR-AUTHORIZATION.md) freezes the exact v2
profile tuple, binary and JCS encodings, digest domains, manifest policy and bindings, validator
precedence, and literal V25/R21–R23 oracle coverage. It prospectively authorizes one later change
only inside the existing `exp1-workload-conformance` crate with no manifest, dependency, lockfile,
workspace, toolchain, or workflow change. Passing the unchanged R9 gate would provide bounded
conformance correctness evidence only. Reference-context implementation, complete R20 closure, live
Linux capture, harness construction, workload or benchmark execution, and every later tranche remain
separately gated or unauthorized.

## R27 R26 closure and v2 reference-context gate update

[R27](R27-R26-CLOSURE-AND-V2-REFERENCE-CONTEXT-AUTHORIZATION.md) closes PR #95 reviewed head
`35f9a0f245ac488828df4f639263edb3fb50be86` (merge
`f4ed0c310fa46c6de209ea0f776c4749e31cdd34`) with exact-head successful CI as bounded R26
conformance/correctness evidence. It freezes a side-by-side v2 R23 scope profile accepting only
exact v2 manifest/WS2/artifact/digest bindings, with v1 unchanged and mixed membership rejected.
Exactly one later pure implementation in `exp1-raw-append-replay` may build the immutable v2
catalog, caller-owned accepted prefix, and contextual mapper against the merged literal oracle.
Complete R20 closure requires that implementation, review, and exact-head CI. Capture, harness,
append integration, execution, benchmarks, durability, faults, and later work remain excluded.

### R28 R27 closure and end-to-end D1 integration gate

[R28](R28-R27-CLOSURE-AND-END-TO-END-D1-INTEGRATION-AUTHORIZATION.md) closes PR #98 reviewed head
`67715b3efc4732542152ea9d935d92ebdb2ca0d6`, merge
`f5cde575cbd82bb788b9519c4efc56e4d1186131`, and both successful exact-head workflows as bounded
R27 correctness evidence. The complete R20 reference-context correctness gate is closed.

Exactly one follow-on PR may update only
`experiments/exp-0001/crates/exp1-raw-append-replay/tests/reference_context.rs` plus synchronized
closure documentation. It must prove the frozen four-operation literal SOP2 → contextual mapper →
byte-exact RF1 → `RawAppender` → physical reopen/replay path, including transactional pre-append
failure, exact receipt/prefix/record/order accounting, and unwind-safe test-owned `std` file cleanup.
This is deterministic integration/correctness testing only. The independent live Linux capture
freeze, descriptive D1 harness, fourth crate, R7 production, real execution, `fsync`, D2/D3, faults,
adapters, baselines, production code, durability, benchmarks, and performance claims remain blocked
or unauthorized.

R29 closes the authorized test-only path at PR #101 reviewed head `b88908cb9cbba39774437e582308bab25a88482b`, merged as `2168839a70baebdea1773fc56e7b8aa0dc9a89e4` after both exact-head workflows succeeded. It connects the four
literal SOP2 operations to the existing contextual mapper, byte-exact RF1 oracle, one
`RawAppender`, and physical reopen/replay, with a pre-append discontinuity check and exact
state/receipt/prefix/record/order accounting. This is bounded deterministic integration/correctness evidence only. R29 freezes the independent live-capture implementation decision but does not authorize capture or execution.

### R29 R28 integration closure and Linux capture gate

[R29](R29-R28-INTEGRATION-CLOSURE-AND-LINUX-CAPTURE-DECISION.md) closes PR #101 reviewed head
`b88908cb9cbba39774437e582308bab25a88482b`, merge
`2168839a70baebdea1773fc56e7b8aa0dc9a89e4`, and its successful exact-head workflows as bounded
deterministic integration/correctness evidence. The semantic-to-physical mapping blocker is closed.

The remaining capture implementation decision is frozen for Fedora 44/Bosgame M5 x86_64: one
external-dependency-free fourth crate may implement only direct Linux/glibc capture and fail-closed
preflight behind typed safe wrappers, with unsafe isolated from all existing crates. Its prospective
PR may add only the workspace member, package/source/tests, and matching lock entry, with no
external or workspace path dependencies. It authorizes neither capture nor M01/harness composition,
R7 production, workload/benchmark execution, publication, or any durability, recovery, fault,
baseline, production, or performance claim.
