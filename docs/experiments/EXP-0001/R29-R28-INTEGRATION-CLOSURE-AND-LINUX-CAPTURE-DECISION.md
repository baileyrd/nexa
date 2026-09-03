# R29 — R28 Integration Closure and Live Linux Capture Decision

**Status:** Complete as governance and bounded integration closure; exactly one capture-boundary implementation PR is prospectively authorized, while all execution remains unauthorized
**Target:** Fedora Linux 44 on the bare-metal Bosgame M5, x86_64
**Evidence classification:** documentation design plus bounded deterministic integration/correctness evidence; no experiment, durability, recovery, benchmark, accuracy, or performance evidence
**Decision date:** 2026-09-03

## 1. Scope and exact integration closure

R29 closes the one integration authorized by R28. PR #101 reviewed head
`b88908cb9cbba39774437e582308bab25a88482b` passed the exact-head **Documentation validation**
and **EXP-0001 Slice A** workflows and merged as
`2168839a70baebdea1773fc56e7b8aa0dc9a89e4`. The merged test connects the frozen four-operation
literal SOP2 stream to the contextual mapper, byte-exact RF1 records, one `RawAppender`, and
deterministic physical reopen/replay. It also checks transactional pre-append rejection and exact
state, receipt, prefix, record, sequence, ordinal, byte, and order accounting.

This is only bounded deterministic integration/correctness evidence. It is not workload execution,
an R7 observation, canonical recovery, durability evidence, or a performance result. The full R20
semantic-operation-to-physical-record mapping blocker named by R19 is now closed. The sole remaining
prerequisite decision for construction of a descriptive D1 harness was the live Linux capture
implementation decision; sections 2–6 freeze that decision. Construction beyond the capture
boundary, and every execution gate, remain separately unauthorized.

## 2. Selected capture implementation boundary

The future capture boundary is target-specific and external-dependency-free:

- every existing crate retains `#![forbid(unsafe_code)]` and remains unchanged;
- future unsafe code is permitted only in one narrowly isolated Linux capture module in the future
  fourth experiment crate;
- each unsafe operation must be minimal, locally documented with its safety invariants, and hidden
  behind a reviewed safe wrapper; application, workload, record, and orchestration logic is
  prohibited inside unsafe blocks;
- the wrappers use direct Linux/glibc interfaces for `clock_gettime`, `clock_getres`, `getrusage`,
  `statx`/`fstat`, and `perf_event_open`;
- procfs and tracefs are read through safe `std` filesystem APIs wherever possible; and
- generic unsafe utilities, privilege escalation, shell-command substitution for required
  measurements, and unsupported architectures are prohibited.

The implementation is compile-time gated to Linux on x86_64. Any other OS or architecture must fail
the package's target gate rather than compile a stub that could be mistaken for supported capture.
This selection is experiment-local and is not a production FFI or portability decision.

## 3. Fail-closed preflight and capture result model

Before creating any publishable evidence, preflight must exercise every required source and wrapper
for the selected observation. Preflight and capture must retain, as applicable, interface identity,
scope, units, permissions, availability, enabled and running time, multiplexing state, counter
width, and source-specific loss diagnostics. Permission denial is a typed recorded result and must
never cause a retry with elevated privilege or a machine configuration change.

Safe wrappers return a typed result that distinguishes **success**, **unavailable**, **permission
denied**, **loss**, and **error**. An unavailable diagnostic field carries an explicit typed
disposition such as unsupported, not present, not applicable, permission denied, or read error; it
is never omitted, inferred, or represented as zero. Required R8-primary metric unavailability makes
the observation invalid or inconclusive under the existing R7/R8 authority.

Absence or loss on a correctness or lifecycle channel invalidates the run. Trace/perf loss, recorder
queue overflow, sequence gaps, read/write errors, failed sentinel verification, or an incomplete
final drain follows R7's existing failure rules: correctness/lifecycle loss invalidates the run;
diagnostic loss makes that metric unavailable and also invalidates the run when the metric is
R8-primary. Start, sentinel, warm-up, measured interval, stop, and complete drain ordering remains
mandatory. Captured values are observations only, never conclusions about accuracy, durability,
recovery, or performance.

## 4. ABI, arithmetic, and deterministic safeguards

The implementation must transcribe and review the exact selected glibc/Linux UAPI constants,
layouts, alignments, integer widths, syscall and libc return-value conventions, and errno mapping.
It must check every narrowing conversion, signed/unsigned boundary, duration sum/product, counter
delta, multiplex scaling operation, byte-length conversion, and overflow. Clock conversion must
validate seconds and subsecond ranges before checked conversion to nanoseconds. Counter conversion
must retain raw values, counter width, enabled time, running time, and multiplexing disposition; a
zero running time cannot produce a scaled value.

Compile-time assertions or deterministic tests must cover constants and ABI layout wherever the
toolchain permits. Synthetic deterministic tests may exercise parsing, conversion, overflow,
return-value/errno classification, unavailable and permission paths, loss classification, and
preflight aggregation. CI must require none of perf privileges, tracefs mounting/access, special
capabilities, or target-machine configuration. CI capture, including an opportunistic successful
call, is validation of code paths only and must never be retained or described as experimental
evidence.

## 5. Exactly one prospective implementation authorization

After R29 review and merge, exactly one next PR may create the fourth experiment workspace member,
provisionally named `exp1-descriptive-d1-harness`, and implement only the Linux capture/preflight
boundary and deterministic tests frozen above.

The only permitted project-configuration changes are:

1. add `crates/exp1-descriptive-d1-harness` to the existing
   `experiments/exp-0001/Cargo.toml` workspace members;
2. add that package's `Cargo.toml` and the minimum Rust source/test files required for the isolated
   capture/preflight module; and
3. add only the corresponding package entry to `experiments/exp-0001/Cargo.lock`.

The new package has no crates.io, Git, build, dev, or workspace path dependencies in this tranche.
It uses only `std` plus its locally declared, reviewed Linux/glibc FFI. Existing package manifests,
sources, fixtures, toolchain files, and workflows do not change. No existing crate may depend on the
new crate, and the new crate may not depend on `exp1-record-format`,
`exp1-workload-conformance`, or `exp1-raw-append-replay` until a later governance record authorizes
the harness composition. Thus dependency direction is empty at this boundary and cannot smuggle
M01, mapping, append, replay, or R7 production into the capture tranche.

Completion requires the unchanged R9 validation sequence, `git diff --check`, exact-head review,
and both exact-head workflows successful. Passing that gate will be bounded capture-boundary
implementation/correctness evidence only and will not authorize capture or execution.

## 6. Explicit exclusions

R29 authorizes no experiment execution, generated-workload run, benchmark, publication, or
performance evidence. The next PR must not implement M01 materialization, semantic mapping,
appending, reopen/replay execution, R7 record or artifact production, workload orchestration, or
measurement publication.

Also excluded are D2/D3, `fsync`, durability, canonical or crash recovery, fault injection,
SQLite/RocksDB execution, adapters, production code or crates, networking, servers, query
languages, distributed behavior, destructive apparatus, privilege escalation, and any machine,
kernel, filesystem, mount, perf, tracefs, security-policy, or service configuration change.

## 7. Traceability and revisit conditions

R29 addresses the live-capture portions of BLK-020, BLK-021, BLK-026, BLK-027, and UNK-022 as a
prospective implementation design. It does not close their execution/evidence portions or BLK-015.
It depends on R4's Fedora 44/Bosgame M5 boundary, R7's source and loss rules, R19's candidate D1
harness analysis, and R20–R28 plus PR #101 for the closed semantic-to-physical correctness path.

Revisit is mandatory if the target OS/architecture changes, a required interface cannot be
represented without broader unsafe code, reviewed constants/layouts disagree with the target
headers, preflight cannot distinguish permission/unavailability/loss, a required primary metric
cannot be captured, or implementation needs any dependency or configuration change not explicitly
listed above. Such a finding blocks the tranche; it does not silently widen this authorization.

