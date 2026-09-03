# Traceability Registry

This registry links intent to planned validation without inventing evidence.

| Source | Traces to | Status |
|---|---|---|
| Vision: represent once, materialize many | P-001, P-002, P-003, P-006, P-010; REQ-001; RQ-001 | Research direction |
| Vision: evidence-driven success | P-011, P-014, P-015; ADR-0001 | Accepted governance |
| REQ-001 through REQ-014 | ADR-0002; HYP-0001 where empirical feasibility is involved | Approved constraints; feasibility unproven |
| HYP-0001 | RQ-001; EXP-0000; EXP-0001; later representation experiments | Active / unproven |
| EXP-0000 | RQ-002; EXP-0001 prerequisites; [single-event semantic envelope](experiments/EXP-0000/SEMANTIC-EVENT-ENVELOPE.md); [reproducible workload contract](experiments/EXP-0000/WORKLOADS.md); [EXP-0001 baseline contract](benchmarks/BASELINES.md); [acknowledgement, visibility, fault, and durability contract](experiments/EXP-0000/ACKNOWLEDGEMENT-VISIBILITY-DURABILITY.md); [crash/recovery correctness contract](experiments/EXP-0000/CRASH-RECOVERY-CORRECTNESS.md); [environment record](benchmarks/ENVIRONMENT-TEMPLATE.md); [raw-result record](benchmarks/RAW-RESULT-TEMPLATE.md); [interpretation and decision contract](benchmarks/INTERPRETATION-CRITERIA.md) | Complete as readiness documentation; no experimental evidence and no EXP-0001 implementation authorization |
| EXP-0000 semantic envelope fields and observation-side lifecycle metadata | REQ-002 through REQ-010, REQ-013, REQ-014; ADR-0002 | Approved-semantics elaboration; the original envelope is immutable after commit, provenance and integrity are conditional where applicable, and physical choices and listed semantic questions remain unresolved |
| EXP-0000 acknowledgement, visibility, fault, and durability contract | REQ-001, REQ-002, REQ-004, REQ-013, REQ-014; ADR-0002; EXP-0001 | Measurement-readiness semantics; D0/D1 remain provisional, D2/D3 require an explicit platform durability contract, and no experimental survival evidence is claimed |
| EXP-0000 crash/recovery correctness contract | REQ-001 through REQ-006, REQ-009, REQ-012 through REQ-014; ADR-0002; EXP-0001 correctness gate | Measurement-readiness procedure; defines oracle, fault placement/matrix, deterministic fail-closed recovery, D3 treatment, and result classifications without selecting physical mechanisms or claiming evidence |
| EXP-0000 reproducible workload contract | REQ-003 through REQ-010, REQ-013, REQ-014; RQ-002; EXP-0001 workload and baseline fairness | Measurement-readiness input; freezes deterministic sizes, distributions and semantic class order, content/envelope/temporal profiles, single-producer assigned-sequence order, concurrent producer-local order, matrix rules, and logical manifest fields; assignment is not canonical commit, and the contract does not select sequencing-gap policy, payload-byte generation, uncontrolled cross-producer interleaving, encoding, time, or serialization mechanisms |
| EXP-0000 EXP-0001 baseline contract | REQ-003 through REQ-010, REQ-013, REQ-014; RQ-002; EXP-0001 comparison and correctness gates | Measurement-readiness input; selects B0–B3 semantic profiles, classifies D0–D3 equivalence, freezes adapter/version/configuration fairness and exclusions, and cites official semantics without selecting executable versions, adapters, platform sync primitives, or claiming evidence |
| EXP-0000 environment and raw-result contracts | REQ-001 through REQ-014; RQ-002; benchmark methodology; EXP-0001 reproducibility and correctness gates | Measurement-readiness records; freeze series identity, immutable evidence, units/missing states, correctness gating, D0–D3 labels, and provenance without selecting serialization, identity/digest, timestamp/clock, or event-integrity mechanisms |
| EXP-0000 interpretation and decision contract | HYP-0001 (partial future evidence only); RQ-002; EXP-0001 interpretation and completion gates; benchmark methodology | Measurement-readiness framework; freezes admissibility, analysis populations, reporting, uncertainty, threshold-registry fields, outcomes, trade-space, complexity, and immutable interpretation records without supplying results or numeric thresholds |
| EXP-0001 | RQ-003; HYP-0001 (partial future evidence only); [execution-readiness plan](experiments/EXP-0001/EXECUTION-READINESS-PLAN.md); R1–R29 authorities | Proposed; R29 closes the merged R28 test-only D1 integration as bounded correctness evidence and freezes one capture-boundary implementation authorization; capture, BLK-015, D2/D3, apparatus, execution, benchmarks, production, and later slices remain gated |
| EXP-0001 readiness plan | REQ-001 through REQ-014; UNK-001 through UNK-003 and UNK-008 through UNK-022; EXP-0000 contracts; benchmark methodology | Planning/traceability and staged authorization; Slices A, B/B0, and A2 passed bounded correctness gates; R19 closes raw D1 append/reopen/replay correctness while BLK-015, D2/D3, execution, benchmarks, fault work, production, and later slices remain gated |
| EXP-0001 R1 physical-record, integrity, and recovery requirements | REQ-001 through REQ-010, REQ-012 through REQ-014; BLK-002; BLK-013; constraints on BLK-001/003/011/012; UNK-012/015 policy portions; EXP-0000 semantic/durability/recovery contracts | Reviewed experiment-local requirements; no encoding, algorithm, implementation, correctness result, or benchmark evidence |
| EXP-0001 R2 deterministic workload, identity/reference-input, digest, and manifest requirements | REQ-003 through REQ-010, REQ-013, REQ-014; BLK-006–009 constraints; UNK-018/019/022; EXP-0000 workload and raw-result contracts | Complete experiment-local requirements/reference-vector plan; BLK-006–009 remain open; no algorithm, serialization, fixture, implementation, correctness result, or benchmark evidence |
| EXP-0001 R3 identity, time, sequencing, retry, and uncertain-outcome lifecycle | REQ-003/004/006/009/013/014; BLK-004/005/011/012 resolutions; BLK-007 constraint; UNK-002/003/009/010/016/017; EXP-0000 lifecycle and R1/R2 contracts | Complete experiment-local lifecycle requirements; no physical API, platform contract, implementation, correctness result, or benchmark evidence |
| EXP-0001 R4 Fedora 44 Bosgame M5 target and platform durability contract | REQ-001/002/004/013/014; BLK-014/015 evidence gates; UNK-014/020/021; EXP-0000 environment, lifecycle and recovery contracts | Complete for conditional planning at the owner-approved evidence boundary; final placement, PLP/controller protection, and empirical survival remain unverified and block D2/D3 claims and execution |
| EXP-0001 R5 B0/B1 physical profiles, framing, integrity, and recoverable commit | REQ-001–010/013/014; RQ-003; BLK-001/003/016/017 and B0/B1 BLK-019; UNK-001/012/015/020/021; EXP-0000 contracts; R1–R4; ADR-0002 | Complete documentation design with stable vectors and exact lifecycle mapping; BLK-015, B2/B3, implementation, empirical equivalence, execution, and evidence remain open |
| EXP-0001 R6 SQLite/RocksDB execution profiles | REQ-001–010/013/014; RQ-003; BLK-015/018/019; UNK-014/020/021/022; EXP-0000 contracts; R1–R5; ADR-0002 | Complete documentation design with exact source/build/API profiles, mappings, effective-setting obligations and D0–D3 classifications; BLK-015, implementation, empirical equivalence, execution, and evidence remain open |
| EXP-0001 R7 benchmark records, artifacts, instrumentation and faults | REQ-001–010/012–014; RQ-003; BLK-008/010/015/020/021/022/025; UNK-014/015/018–022; EXP-0000 contracts; R1–R6; ADR-0002 | Complete documentation design for JCS records, SHA-256 domains, artifact provenance/retention, instrumentation/overhead, process termination and fail-closed fault classification; owner-dependent destructive apparatus, implementation, effective validation, execution and evidence remain open |
| EXP-0001 R8 primary matrix, thresholds and statistical plan | REQ-001–010/012–014; RQ-002/003; HYP-0001; BLK-015/020/022–024/026/027; UNK-008/014/020–022; EXP-0000 contracts; benchmark methodology/baselines; R1–R7; ADR-0002 | Forty-cell candidate-primary registry, BLK-024 analysis design, and prospective owner-approved `thresholds-v1` frozen; BLK-023/UNK-008 are resolved for this decision, R8 remains complete as documentation design; R9 subsequently authorized Slice A; its reviewed implementation is correctness-validation evidence only and no execution exists |
| [EXP-0001 R9 workspace, harness, CI, and Slice A authorization](experiments/EXP-0001/R9-WORKSPACE-HARNESS-CI-AND-SLICE-A-AUTHORIZATION.md) | REQ-003–010/013/014; RQ-003; BLK-001/003/020/026/027; UNK-001/002/003/012/020/022; R1–R8; R5/R7 stable vectors; ADR-0002 | Complete documentation design; freezes the Slice A-only workspace, harness subset, Rust 1.89.0 build, empty dependency allowlist, correctness tests, and exact CI; became effective on merge and authorized the now-reviewed Slice A correctness implementation only |
| [EXP-0001 R10 Slice A closure and Slice B authorization](experiments/EXP-0001/R10-SLICE-A-CLOSURE-AND-SLICE-B-AUTHORIZATION.md) | RQ-003; BLK-006–009/015/016/020/026/027; UNK-018–020/022; R3/R5/R7/R9 | Historical prospective authority that closed Slice A and bounded Slice B; R11 records the later merged Slice B evidence without retroactively broadening R10 |
| [EXP-0001 R11 Slice B closure and next gate](experiments/EXP-0001/R11-SLICE-B-CLOSURE-AND-NEXT-GATE.md) | RQ-003; BLK-006–009/015/016/017/020/026/027; UNK-018–022; R1/R2/R3/R5/R7/R9/R10 | Closes Slice B/B0 as bounded implementation/correctness-validation evidence and prospectively authorizes only a documentation/research freeze for BLK-006/007; Slice C, workload generation, execution, persistence, durability, benchmarks, faults, and later work remain unauthorized |
| [EXP-0001 R12 deterministic generator specification and vectors](experiments/EXP-0001/R12-DETERMINISTIC-GENERATOR-SPECIFICATION-AND-VECTORS.md) | RQ-003; BLK-006/007; UNK-018/019/022; R2/R3/R7/R11; EXP-0000 workload/envelope contracts | Freezes SHA-256 counter/motif/zero payload profiles, deterministic UUIDv4-shaped typed identities, explicit envelope/reference/logical-time inputs, failure semantics, and documentation vectors; no generator, fixture, stream, BLK-008/009 resolution, Slice C, or execution |
| [EXP-0001 R13 R12 closure and next gate](experiments/EXP-0001/R13-R12-CLOSURE-AND-NEXT-GATE.md) | RQ-003; BLK-006–009/015/017/020/026/027; UNK-018–022; R1/R2/R3/R5/R7/R9/R10/R11/R12 | Closes exact reviewed R12 as documentation/governance evidence and prospectively authorizes only the smallest prerequisite, a BLK-008 documentation freeze; generator implementation, BLK-009, Slice C/B1, workload generation, persistence, faults, benchmarks, and execution remain unauthorized |
| [EXP-0001 R14 canonical workload-stream digest freeze](experiments/EXP-0001/R14-CANONICAL-WORKLOAD-STREAM-DIGEST.md) | RQ-003; BLK-008/009; UNK-018/019/022; R2/R7/R12/R13 | Freezes canonical semantic-operation and ordered workload-stream preimages, R7-domain SHA-256 documentation vectors, and later BLK-009 metadata obligations; documentation design only, with no generator, manifest, execution, or later work |
| [EXP-0001 R15 R14 closure and BLK-009 next gate](experiments/EXP-0001/R15-R14-CLOSURE-AND-BLK-009-NEXT-GATE.md) | RQ-003; BLK-008/009/010/015/020/025/026/027; UNK-018/019/022; R2/R7/R12/R14 | Closes exact reviewed R14 as documentation/governance evidence and prospectively authorizes only a focused BLK-009 workload-manifest serialization documentation freeze; all implementation, artifacts, execution, persistence, faults, benchmarks, and later slices remain unauthorized |
| [EXP-0001 R16 workload-manifest serialization contract](experiments/EXP-0001/R16-WORKLOAD-MANIFEST-SERIALIZATION-CONTRACT.md) | RQ-003; BLK-009; UNK-019/022; R2/R7/R12/R14/R15 | Freezes `EXP-0001-WORKLOAD-MANIFEST-JCS-v1`, its closed ledger, JCS bytes, bindings, fail-closed validation, immutable supersession, and documentation vectors; resolves BLK-009/UNK-019 only as documentation design and authorizes no implementation or execution |
| ADR-0001 | Research lifecycle and benchmark methodology | Accepted governance decision |
| ADR-0002 | REQ-001 through REQ-014 and terminology | Accepted research constraints; no empirical validation claim |

No listed link represents completed experiment evidence.

## R17 / Slice A2 trace

| Approval | Implementation | Authorities exercised | Evidence | Retained boundary |
|---|---|---|---|---|
| Issue #63, `EXP-0001-A2/workload-conformance-v1` | `experiments/exp-0001/crates/exp1-workload-conformance` | R12 / BLK-006–007; R14 / BLK-008; R16 / BLK-009 | literal-vector implementation/correctness validation | BLK-015; Slice C/B1; execution; benchmark/capture; persistence; faults; durability; adapters; production |

BLK-020/026/027 apply to A2 only through the existing external-dependency-free workspace with reviewed workspace path dependencies, Rust 1.89.0,
and unchanged exact-head CI gate. R17 grants no later-slice authority.

## R18 / A2 closure and Slice C/B1 readiness trace

| Authority | Inputs | Disposition | Retained boundary |
|---|---|---|---|
| [R18](experiments/EXP-0001/R18-A2-CLOSURE-AND-SLICE-C-B1-READINESS.md) | PR #64 historical head/merge; corrective PR #68 reviewed head/merge and workflows; R1/R3/R5/R16/R17; BLK-015/017/020/026/027 | Closes A2 as correctness evidence; preserves corrected M01 as the canonical positively valid R7-backed fixture; authorizes one external-dependency-free raw D1 append/reopen/replay correctness crate with exactly one reviewed workspace path dependency on `exp1-record-format` | BLK-015 and all D2/D3, execution, benchmark, fault, adapter, production, and later-slice work |
| R18-authorized Slice C/B1 implementation | `experiments/exp-0001/crates/exp1-raw-append-replay`; R1/R3/R5/R18 | Bounded implementation/correctness-validation evidence for validated raw D1 submission and deterministic physical accepted-prefix reopen/replay | No stable-storage, namespace, canonical recovery, D2/D3, workload, benchmark, fault, adapter, production, performance, or later-tranche claim |


## R19 / Slice C/B1 closure and descriptive D1 readiness trace

| Source | Inputs and blockers | Output | Still excluded or blocked |
|---|---|---|---|
| [R19](experiments/EXP-0001/R19-SLICE-C-B1-CLOSURE-AND-DESCRIPTIVE-D1-HARNESS-READINESS.md) | PR #71 reviewed head/merge; PR #74 reviewed head/merge and exact-head workflows; R7/R12/R14/R16/R18; merged three-crate workspace; BLK-020/021/026/027; UNK-022 | Closes Slice C/B1 as bounded implementation/correctness-validation evidence; freezes the maximum candidate descriptive D1 boundary; records the exact semantic-to-physical mapping and live Linux capture blockers | Fourth crate, executable R7 capture/validation, workload materialization/execution, benchmark evidence, BLK-015, D2/D3, faults, adapters, production, performance interpretation, and later tranches |


## R20 semantic mapping trace

| Authority | Inputs | Decision | Still excluded/open |
|---|---|---|---|
| [R20](experiments/EXP-0001/R20-SEMANTIC-OPERATION-TO-PHYSICAL-RECORD-MAPPING.md) | R1/R2/R3/R5/R7/R12/R14/R16/R18/R19; M01; three-crate implementation | Freezes one validated SOP1 to one `STRUCTURAL-0` type-3 RF1 provisional record, exact field preservation, runtime sequence/physical-ordinal inputs, vectors, failure policy, and a pure public mapper owned by `exp1-raw-append-replay` with direct path dependencies on both unchanged authority crates | Append integration; live Linux capture decision; fourth crate; workload/harness execution; canonical commit/recovery; D2/D3; `fsync`; faults; benchmarks; performance/durability evidence; production |
| R20 mapper implementation | `exp1-raw-append-replay::mapping`; R20 V20-01–V20-05 | Bounded implementation/correctness-validation evidence for the locally decidable pure SOP1-to-RF1 subset, checked unforgeable caller state, byte-identical stable cores, and validator-decidable failures; R21 freezes the locally decidable prior-event/stream context only; complete R20 closure remains blocked on the closed-scope governance decision and later authorization/implementation/review | Append/reopen integration and every R20 exclusion remain outside the evidence |

## R21 reference-context trace

| Authority | Inputs | Frozen result | Still excluded/open |
|---|---|---|---|
| [R21](experiments/EXP-0001/R21-REFERENCE-VALIDATION-CONTEXT.md) | R12 section 5.3 and A01/R01–R04; R14 WS1; R20 mapper and gate; UNK-013 | Supplied validated semantic streams form an immutable typed identity catalog; separate accepted-prefix state, bounds, collisions, locally decidable precedence, and vectors are frozen, R22 supersedes the cross-segment gap and R23 freezes closed-scope proof; no mapper extension is authorized | Complete R20 implementation gate; reconstruction; append/reopen; live Linux capture/harness; execution; Cargo/authority-crate changes; fourth crate; durability/performance evidence; all later work |

## R22 cross-segment reference trace

| Authority | Inputs | Decision/evidence boundary | Still excluded or blocked |
|---|---|---|---|
| [R22](experiments/EXP-0001/R22-CROSS-SEGMENT-REFERENCE-RULE.md) | R12 section 5.3/vectors; R14 WS1 ordering; R16; R20; R21 catalog/context | Strictly segment-local reference domain, `[0,i)` interpretation, `E-REFERENCE-CROSS-SEGMENT`, precedence, transactional vectors, and prospective R21 effect; documentation design only | R23 subsequently closes stream-scope governance; R21 implementation; complete R20 gate; Rust/Cargo changes; capture/harness/execution; append/reopen; fourth crate; D2/D3/`fsync`; faults/adapters/production; durability/performance evidence |

## R23 closed stream-scope trace

| Authority | Inputs | Decision/evidence boundary | Still excluded or blocked |
|---|---|---|---|
| [R23](experiments/EXP-0001/R23-CLOSED-STREAM-SCOPE-PROOF.md) | R12 reference dispositions; R14 WS1/digest; R16 manifests/artifacts; R20 mapper gate; R21 catalog; R22 segment locality; UNK-013 | Canonical cell-scoped descriptor, namespace-ordered exact membership, R16/R14/digest bindings, exact supplied-set equality, bounds and fail-closed construction; governance design only | Reference-context implementation and complete R20 gate; Rust/Cargo/authority changes; append/reopen; capture/harness/execution; fourth crate; D2/D3/`fsync`; faults/adapters/production; durability/performance evidence |

## R24 reference-context implementation-authorization trace (superseded by R25)

| Authority | Inputs | Authorized output | Still excluded/open |
|---|---|---|---|
| [R24](experiments/EXP-0001/R24-REFERENCE-CONTEXT-IMPLEMENTATION-AUTHORIZATION.md) | R20 mapper/API/tests; R21 catalog, prefix, bounds and vectors; R22 segment locality; R23 closed-scope descriptor, bindings, bounds and vectors | Historical prospective authorization, now superseded by R25, for exactly one pending pure extension in `exp1-raw-append-replay`; frozen constructor/catalog/state/contextual-map APIs, deterministic errors/precedence, exact source/test paths, transactional and resource test gate; no manifest/dependency changes | Implementation and complete R20 closure until exact-head gate; authority crates; append/reopen; capture/harness/execution; fourth crate/dependency/unsafe; D2/D3/`fsync`; benchmarks, durability, recovery, faults, adapters, production, server/network/query/distributed work |


## R25 bootstrap causal-reference governance trace

| Authority | Contradiction inputs | Corrective decision | Still excluded/open |
|---|---|---|---|
| [R25](experiments/EXP-0001/R25-BOOTSTRAP-CAUSAL-REFERENCE-GOVERNANCE.md) | R12 positive causal target/cardinality; R16 one stream envelope and scalar positive causal cardinality; R21 mapping from the first operation; R22 strict segment locality; R23 exact manifest-bound closed scope; R24 valid-reference implementation gate; closed unmerged PR #91 | Records the failed governance assumption; preserves every v1 vector/byte; freezes prospective v2 zero-target ordinal-0 causal bootstraps plus positive same-segment subsequent references and an explicit per-segment cardinality policy; supersedes only R24 implementation authorization | V2 conformance/validator code and literal vectors require separate authorization, followed by a separate bounded reference-context implementation authorization; complete R20 gate, Rust/Cargo/workflow/test changes, capture/harness/execution, D2/D3/`fsync`, benchmarks, durability/recovery, faults/adapters/production remain open or excluded |


## R26 v2 conformance and validator-authorization trace

| Authority | Inputs | Frozen/authorized output | Still excluded/open |
|---|---|---|---|
| [R26](experiments/EXP-0001/R26-V2-CAUSAL-REFERENCE-CONFORMANCE-AND-VALIDATOR-AUTHORIZATION.md) | R12/R14/R16 v1 immutable contracts; R20 mapper gate; R21 catalog/precedence; R22 segment locality; R23 scope proof; R25 v2 bootstrap correction | Complete v2 profile tuple, binary/JCS encoding, digest domains, manifest cardinality policy, immutable bindings, validation precedence, and literal V25/R21–R23 oracle requirements; prospectively authorizes only a later side-by-side v2 extension in `exp1-workload-conformance` with unchanged manifests/dependencies/workflow | Implementation and correctness evidence; R21–R23 reference-context implementation; complete R20 closure; other crates/Cargo/workflows; append/reopen; capture/harness/execution; D2/D3/`fsync`; benchmarks, durability/recovery, faults/adapters/production |

## R27 R26-closure and v2 reference-context trace

| Authority / evidence | Inputs | Establishes | Does not establish / remains excluded |
|---|---|---|---|
| [R27](experiments/EXP-0001/R27-R26-CLOSURE-AND-V2-REFERENCE-CONTEXT-AUTHORIZATION.md) | R21–R23 context/scope; R25 bootstrap governance; R26 contract; PR #95 reviewed head `35f9a0f245ac488828df4f639263edb3fb50be86`, merge `f4ed0c310fa46c6de209ea0f776c4749e31cdd34`, exact-head successful CI | Closes R26 implementation as bounded conformance/correctness evidence; freezes homogeneous exact v2 R23 scope bindings with v1 unchanged; authorizes one pure v2 catalog/state/contextual mapper in `exp1-raw-append-replay` using merged literals as oracle | R27-authorized implementation and complete R20 closure; append/reopen; capture/harness/execution; fourth crate/new dependencies; D2/D3/`fsync`; benchmarks, durability/recovery, faults/adapters/production |
| R27-authorized v2 reference-context implementation | Exact R27 APIs/errors/precedence/bounds/paths/tests; PR #98 reviewed head `67715b3efc4732542152ea9d935d92ebdb2ca0d6`, merge `f5cde575cbd82bb788b9519c4efc56e4d1186131`, both exact-head workflows successful | R28 closes homogeneous v2 scope, immutable typed catalog, private caller-owned prefix, zero-target bootstraps, same-segment prior references, and transactional RF1 mapping as bounded correctness evidence; complete R20 reference-context gate closed | Append/reopen integration evidence and every R27 exclusion remain outside this closure |


## R28 R27-closure and end-to-end D1 integration-authorization trace

| Authority / evidence | Inputs | Establishes / authorizes | Does not establish / remains excluded |
|---|---|---|---|
| [R28](experiments/EXP-0001/R28-R27-CLOSURE-AND-END-TO-END-D1-INTEGRATION-AUTHORIZATION.md) | R19 raw append/replay; R20–R27 mapping/context authorities; merged R26 literals; PR #98 reviewed head `67715b3efc4732542152ea9d935d92ebdb2ca0d6`, merge `f5cde575cbd82bb788b9519c4efc56e4d1186131`, both exact-head workflows successful | Closes R27 implementation and complete R20 reference-context correctness gate as bounded evidence; authorizes one test-only four-operation SOP2 → contextual mapping → RF1 → `RawAppender` → physical reopen/replay correctness path, transactional pre-append failure, exact accounting/order, and `std` cleanup | Production/configuration changes; fourth crate/harness; live Linux capture; R7 production; workload or benchmark execution; `fsync`, D2/D3, durability/recovery evidence; faults, adapters, SQLite/RocksDB execution, performance, production |
| R28-authorized test-only D1 integration | Existing R26 literals and RF1 expectations; merged R27 context mapper; R19 `RawAppender` and physical replayer; PR #101 reviewed head `b88908cb9cbba39774437e582308bab25a88482b`, merge `2168839a70baebdea1773fc56e7b8aa0dc9a89e4`, exact-head workflows successful | R29 closes the deterministic four-record integration with transactional rejection and exact state/receipt/replay identity as bounded correctness evidence | Harness/capture/execution; synchronization, durability, recovery, performance; every other R28 exclusion |

## R29 integration-closure and Linux-capture-decision trace

| Authority / evidence | Inputs | Establishes / authorizes | Does not establish / remains excluded |
|---|---|---|---|
| [R29](experiments/EXP-0001/R29-R28-INTEGRATION-CLOSURE-AND-LINUX-CAPTURE-DECISION.md) | R4; R7; R19–R28; PR #101 reviewed head `b88908cb9cbba39774437e582308bab25a88482b`, merge `2168839a70baebdea1773fc56e7b8aa0dc9a89e4`; successful exact-head Documentation validation and EXP-0001 Slice A workflows | Closes R28 integration and R19 semantic-to-physical blocker as bounded deterministic correctness evidence; freezes Linux/x86_64 direct-interface, isolated-unsafe, typed fail-closed capture design; authorizes one dependency-free fourth-crate capture/preflight implementation and deterministic tests | Capture or execution; M01 materialization; append/replay orchestration; R7 production/publication; workloads/benchmarks/performance; D2/D3, `fsync`, durability/recovery/faults; baselines/adapters/production/network/server/query/distributed work; destructive apparatus or machine changes |
