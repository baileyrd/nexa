# PR #2 validation and implementation plan

**Review date:** 2026-08-18  
**Reviewed head:** `5b425af` (`Declare the transitional Nexa workspace`)  
**Scope:** validation and planning only; no runtime migration or contract implementation

## Validation result

The draft PR head passes every requested workspace gate without a source fix:

| Check | Exit | Exact result |
|---|---:|---|
| `cargo fmt --all --check` | 0 | Passed with no formatting differences. |
| `cargo check --workspace --all-targets` | 0 | Finished the `dev` profile successfully; all workspace targets checked. |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | Finished the `dev` profile successfully with no warnings. |
| `cargo test --workspace` | 0 | 54 library tests passed; 0 failed, ignored, measured, or filtered out. Both binary targets and doc-tests ran 0 tests and passed. |

No failure was attributable to PR #2, so no validation fix was appropriate. The existing root package, runtime behavior, tests, assets, reconstructed specifications, and canonical reference images remain unchanged.

## Baseline and registry review

The specification registry is the navigation and governance authority. It classifies the domain, event, NBP, avatar, and principal 3D specifications as **Baseline Draft**, while the current 3D runtime is only an **Implemented slice**. Consequently, implementation may encode the baseline but must not silently settle unresolved semantics or promote a document's status.

The reconstructed baseline establishes this effective authority order: approved ADRs/specifications, registered Baseline Draft specifications, character/behavior identity, canonical visual sheets, verified runtime contracts, architecture narratives, then provenance. It also requires conflicts between design and verified behavior to be recorded rather than silently resolved.

The immediate architecture is coherent at its highest level:

- `nexa-domain` owns canonical identifiers and shared value/domain types.
- `nexa-events` owns generic event/command envelopes, event transport abstractions, ordering, correlation, replay, and typed integration events.
- `nexa-nbp` owns transport- and renderer-independent semantic behavior messages.
- The avatar boundary consumes semantic commands; it does not expose clips, bones, blendshapes, or renderer objects to tutor intelligence.
- The present root package is deliberately transitional and must remain intact until the contract kernel provides stable dependencies for migration.

The registry is not yet sufficient as a compile-ready contract. The specifications are broad baseline inventories and include overlaps, illustrative Rust, undefined extension rules, and reconstruction artifacts. The first implementation should therefore begin with a narrow, explicitly versioned MVP surface and conformance fixtures, not a wholesale transcription of every example type.

## Shared contract kernel plan

### Gate 0 — decisions before code

1. Inventory every MVP type named by NEXA-DOM-001, NEXA-EVT-001, and NEXA-NBP-001 in a traceability table: specification section, canonical owner, wire name, validation rule, privacy classification, and initial consumer.
2. Mark examples as either normative, informative, or unresolved. Do not infer normative Rust layout from prose that says types “should resemble” an example.
3. Approve ADRs for ownership/dependency direction, serialization compatibility, identifiers/time, asynchronous bus semantics, and NBP/event separation.
4. Check in representative golden JSON fixtures and JSON Schemas before downstream consumers depend on the crates.
5. Define MSRV support, feature policy, public API stability, and semver/versioning gates for the workspace.

### `nexa-domain`

**Boundary**

- Create a dependency-light leaf crate containing shared identifiers, validated scalar value objects, protocol/schema versions, timestamps, confidence/mastery values, endpoint/subject references, and the smallest cross-subsystem domain payloads required by the first vertical slice.
- Keep repositories, clocks, randomness, persistence adapters, orchestration, event-bus implementations, renderer types, and provider SDKs outside the crate.
- Split broad domain entities into modules without splitting their ownership across crates. Feature flags must not change wire representations.

**First increment**

1. Implement a macro or consistent hand-written pattern for opaque UUID newtypes (`SessionId`, `EventId`, `MessageId`, `BehaviorId`, `CorrelationId`, and `TraceId` first).
2. Implement validated `Version`, `Timestamp`, `Confidence`, sequence, endpoint, and semantic-key types with smart constructors and structured errors.
3. Add the minimum behavior intent and session references required by events/NBP only after ownership is decided; avoid importing the complete student/lesson/assessment model in phase 1.
4. Stabilize Serde conventions: transparent newtypes, UUID text form, RFC 3339 timestamps, enum tagging/casing, omitted-option behavior, unknown-field policy, and map ordering expectations.
5. Test constructor invariants, JSON golden round trips, invalid boundary values, UUID/version parsing, and forward/backward compatibility fixtures.

**Exit evidence**

- One canonical definition exists for every shared primitive used by events and NBP.
- Public types compile without GUI, GPU, network, database, or async runtime dependencies.
- Golden fixtures validate against schemas and round-trip without semantic loss.

### `nexa-events`

**Boundary**

- Depend on `nexa-domain`; do not make `nexa-domain` depend on events.
- Own `Event<T>`, `Command<T>`, event type/schema identity, metadata, subscriptions, delivery errors, and the bus abstraction.
- Keep NBP payload mapping in an adapter/integration module or higher-level crate so the generic bus does not depend on `nexa-nbp` unless an ADR explicitly approves that edge.
- Separate durable facts from transient high-frequency media/control streams.

**First increment**

1. Define generic typed envelopes with explicit schema and envelope versions, source, optional subject/session, sequence, correlation, causation, trace, timestamp, and metadata.
2. Define a sealed or trait-based `DomainEvent` contract and a small MVP catalog: session started/ended, student text or answer submitted, tutor response completed, speech synthesis completed, avatar behavior started/completed/failed.
3. Specify sequence scope and enforce monotonic publication per scope. Consumers must deduplicate by event ID and tolerate documented out-of-order delivery.
4. Implement an in-process bus behind a runtime-neutral or deliberately selected async interface, with bounded queues, cancellation-safe subscriptions, backpressure behavior, and deterministic shutdown.
5. Provide an optional in-memory event store/test harness for replay, without conflating persistence with the bus.
6. Test correlation/causation propagation through the first vertical slice, duplicate delivery, slow subscribers, subscriber failure isolation, ordering, cancellation, replay, and schema compatibility.

**Exit evidence**

- The first integration sequence preserves `session_id`, `correlation_id`, and `trace_id` end to end.
- Bus business logic is absent; subscribers own domain decisions and idempotency.
- Media bytes and secrets are rejected or represented by governed references.

### `nexa-nbp`

**Boundary**

- Depend only on `nexa-domain` plus minimal serialization/schema dependencies.
- Own NBP envelopes, semantic behavior commands/updates/cancellation, channel values, runtime acknowledgments/state/capabilities, protocol errors, and validation.
- Exclude renderers, GLB identifiers, animation clip names, speech-provider objects, LLM SDKs, course logic, and transport implementations.

**First increment**

1. Define protocol/message versions independently from crate versions and event schema versions.
2. Implement typed payload variants for behavior command, update, cancel, acknowledgment, runtime state/capabilities, and error. Resolve the envelope `message_type`/payload relationship so invalid combinations are unrepresentable or rejected.
3. Implement validated emotion, gaze, gesture, speech, presentation, timing, priority, interruptibility, and completion policy values, beginning only with the minimum runtime acceptance surface.
4. Specify a capability vocabulary and deterministic degradation result for unsupported optional channels.
5. Implement protocol state validation and arbitration as separate pure policy modules; do not hide state transitions inside serialization or renderer adapters.
6. Add canonical JSON/schema fixtures for the questioning, correction, celebration, interruption, cancellation, and capability-negotiation flows.
7. Add property/transition tests for valid state graphs, sequence behavior, update/cancel races, channel ownership, unknown optional fields, unsupported capabilities, and deterministic replay.

**Exit evidence**

- A headless fake avatar accepts the MVP NBP sequence and produces deterministic acknowledgments/state events.
- The existing 3D runtime can be adapted from NBP semantic identifiers without NBP learning about its internal commands.
- Invalid state transitions and unknown required semantics fail with stable protocol errors.

### Kernel integration order

1. Land workspace lint/MSRV/public-API policy and `nexa-domain` primitives.
2. Land `nexa-events` envelopes plus a deterministic in-process test bus.
3. Land `nexa-nbp` envelopes and pure protocol validation.
4. Add an integration-test crate or workspace test that maps the typed tutor result to an event, produces an NBP command, exercises a fake runtime, and emits correlated completion.
5. Only after fixture and compatibility review, expose the kernel to the existing runtime migration.

## Controlled 3D runtime migration plan

### Migration invariants

- Move with history-preserving Git operations; do not rewrite runtime behavior while relocating it.
- Preserve public behavior, default features, binary names/arguments, manifest format, shader behavior, asset fixtures, canonical images, and all 54 current tests.
- Preserve a GPU-free path for validation and most tests.
- Separate relocation commits from contract-adapter commits so regressions are bisectable.
- Do not make the 3D crate the owner of NBP, shared domain, tutor, or event types.

### Stage 1 — freeze and characterize

1. Record current public modules, features, binaries, CLI behavior, accepted manifest/GLB semantics, test inventory, and sample commands.
2. Add no new behavior; only introduce missing characterization tests if separately approved as migration safeguards.
3. Capture checksums/paths for the runtime example, WGSL, generated test fixtures, canonical visual references, and eventual approved GLB fixture.
4. Establish the exact baseline gates: the four workspace checks, headless validator fixture, manifest validation, shader validation, and an opt-in viewer smoke test for supported display/GPU CI.

### Stage 2 — mechanical library relocation

1. Create `crates/nexa-3d/Cargo.toml` using the existing library package metadata and feature behavior.
2. Move root `src` library modules, WGSL, runtime test support, and library-owned assets with `git mv`; keep module names and APIs unchanged.
3. Keep `nexa_3d_runtime` as the Rust library name initially if required for compatibility; decide package/lib rename separately by ADR and deprecation policy.
4. Repoint paths only. Do not introduce `nexa-avatar`, events, or NBP adaptations in the same commit.
5. Run and compare the complete test inventory and headless validator outputs.

### Stage 3 — viewer application extraction

1. Create `apps/nexa-3d-viewer` as the composition/binary package and move `src/main.rs` with `git mv`.
2. Make the application depend on `nexa-3d` with its viewer feature. Move windowing, `wgpu` surface lifecycle, OS input, and presentation-only composition into the app only where this can be done behavior-preservingly and in reviewable steps.
3. Preserve the `nexa-3d-viewer` binary name and controls. Keep `nexa-3d-validate` either as a second app binary or a thin dedicated app package based on the CLI packaging ADR; preserve its arguments and exit semantics.
4. Ensure `nexa-3d --no-default-features` remains headless and free of optional window/surface initialization.

### Stage 4 — semantic adapter insertion

1. Introduce a renderer-neutral avatar port only after its contract is approved. Implement it around the existing `AvatarRenderer`/adapter behavior rather than replacing working runtime internals.
2. Add a boundary adapter from `nexa-nbp` types to existing `ExpressionCommand`, `VisemeCommand`, `GazeCommand`, and `GestureCommand` semantics.
3. Retain current canonical names and manifest resolution. Unsupported names/capabilities must produce diagnosable errors or explicit degradation results, never unrelated fallback animation.
4. Add correlated NBP acknowledgment/state output without coupling renderer code to the event bus.
5. Prove parity with adapter tests that replay current semantic behavior and compare command ordering, gaze limits, viseme envelopes, animation sampling, morph mixing, and skin palettes.

### Stage 5 — migration acceptance

1. Run all four workspace gates from the root, the headless GLB/manifest acceptance gate, schema fixtures, and the NBP fake-runtime integration test.
2. Confirm the 54 existing tests still exist and pass; any renamed test must have an explicit mapping in the migration PR.
3. Validate canonical reference images and specifications are byte-identical unless a separately reviewed governance change intentionally updates them.
4. Exercise the viewer on a supported GPU/display and capture visual evidence only when the move produces a perceptible change; a pure relocation should not.
5. Remove the transitional root package only in the final mechanical cleanup commit after downstream paths and documentation resolve to the new packages.

## Conflicts and missing decisions

### Reconstructed-document conflicts/artifacts

1. **Concatenated specifications:** NEXA-DOM-001 ends by beginning NEXA-ORCH-001, and NEXA-EVT-001 ends by beginning NEXA-DOM-001. These embedded starts conflict with the registry's one-file-per-authority navigation and make scope/version review ambiguous. Preserve the originals; resolve via a traceable normalization change or reviewed clean editions.
2. **3D document links:** NEXA-3D-RUNTIME-001 links simplified filenames that do not match the registered Unicode filenames. The registry remains authoritative, but relative navigation from the runtime document is currently unreliable.
3. **3D status language:** the registry says “Implemented slice,” while the runtime document says “implementation scaffold” and its increment checklist mixes completed and partially complete statements. Define a single evidence-based status model.
4. **Avatar naming:** NEXA-CBS/NBP use semantic intent, NEXA-AVTR is the renderer-neutral boundary, and the runtime already exposes similarly purposed adapter/command types. Ownership and conversion boundaries must be fixed before crates are created to prevent duplicate public models.
5. **Events versus NBP:** both define envelopes with message/event ID, version, timestamp, session, sequence, source, correlation, status/error, and replay concepts. They explicitly must remain distinct, but shared field ownership and the adapter's dependency direction are not settled.
6. **Normative force:** the documents mix SHALL, SHOULD, prose examples, “should resemble,” and recommended layouts. A conformance profile is needed before examples become stable public Rust APIs.

### Contract decisions still missing

- UUID crate and UUIDv7 generation location; parsing/display and nil-ID policy.
- Timestamp representation, precision, UTC normalization, monotonic versus wall-clock use, and test clock behavior.
- Protocol, envelope, schema, content, crate, and asset version relationships.
- JSON enum/tag/casing conventions, numeric representation, unknown variants/fields, and canonical serialization for signing/golden tests.
- Whether sequence numbers are scoped per session, source, stream, subject, or connection; gap and rollover handling.
- Exact command/event/NBP correlation and causation mapping.
- Event-bus async runtime, object safety, `Send`/`Sync` guarantees, queue bounds, fan-out semantics, subscriber error policy, and shutdown contract.
- Delivery guarantee boundary (at-most-once, at-least-once, durable acknowledgement) and idempotency ownership.
- Event persistence/redaction/retention, privacy classification, and replay authorization.
- Domain type breadth: a minimal kernel versus the entire platform aggregate inventory.
- NBP state graph, update merge semantics, cancellation race behavior, arbitration ownership, priority range, time units, and clock reference.
- Extension governance and behavior for unknown required versus optional message/channel values.
- Capability negotiation direction, cache lifetime, and deterministic degradation outcomes.
- Canonical viseme vocabulary conflict handling across NBP, speech, avatar, manifest, and runtime.
- Workspace package names versus Rust library names, compatibility aliases, feature unification, and binary packaging.
- Location/ownership of example assets, schemas, cross-crate fixtures, integration tests, and canonical GLB when available.
- Supported platforms/backends and whether GPU/display smoke checks are required or informational.

## Dependency risks

1. **Cycles:** putting behavior intent in `nexa-domain`, NBP mapping in `nexa-events`, and event emission in `nexa-nbp` can create a three-crate cycle. Enforce a leaf-domain DAG and place orchestration adapters above both protocols.
2. **Dependency inflation:** UUID/time/schema/async choices can pull substantial feature graphs into every subsystem. Use workspace-pinned dependencies, disable unnecessary defaults, and measure feature trees for headless builds.
3. **Async lock-in:** an event trait designed directly around one executor makes local-first, WASM, deterministic tests, and future distributed adapters harder. Decide runtime policy explicitly rather than accidentally through the first channel crate.
4. **Schema drift:** Serde derives alone do not establish wire compatibility. Golden fixtures, generated schemas, compatibility checks, and explicit version ownership are required.
5. **Type leakage:** `wgpu`, `winit`, glTF indices, animation clips, and provider SDK types must not cross shared or avatar contracts.
6. **Feature unification:** viewer defaults can accidentally enable GPU/window dependencies for validators and downstream headless services. Test `nexa-3d` both with and without viewer features.
7. **Version skew:** events may persist longer than a process while NBP can be negotiated per runtime. Treat compatibility and upgrade policies separately.
8. **High-frequency streams:** visemes, gaze, telemetry, and media can overwhelm a general reliable event bus. Classify transient streams and use references for media payloads.
9. **Platform/GPU nondeterminism:** visual and surface tests are environment-sensitive. Keep pure math, shader parsing, manifests, and import acceptance headless; isolate opt-in GPU evidence.
10. **Migration churn:** combining moves with API redesign obscures history and makes regressions hard to bisect. Require mechanical moves and semantic changes in separate commits/PRs.

## Recommended ADR queue

1. **ADR-0002: Contract-kernel ownership and dependency DAG** — canonical type ownership, permitted crate edges, and adapter placement.
2. **ADR-0003: Wire format, schema registry, and compatibility policy** — JSON representation, schema generation, golden fixtures, unknown fields/variants, and breaking-change rules.
3. **ADR-0004: Identity, time, ordering, and correlation semantics** — UUIDv7, clocks, timestamp precision, sequence scope, causation, trace propagation, and deterministic test sources.
4. **ADR-0005: Event delivery, backpressure, persistence, and replay** — async runtime, queueing, guarantees, idempotency, retention, redaction, dead letters, and shutdown.
5. **ADR-0006: NBP lifecycle, arbitration, cancellation, and capability negotiation** — state machine, update semantics, channel ownership, unsupported capability degradation, and protocol errors.
6. **ADR-0007: Semantic vocabulary and extension governance** — expressions, gestures, gaze targets, speech styles, visemes, namespace ownership, and unknown-value behavior.
7. **ADR-0008: Contract conformance and version release gates** — normative profiles, fixture/schema testing, semver relationships, MSRV, and promotion from Baseline Draft.
8. **ADR-0009: Controlled 3D package migration and compatibility policy** — package/library/binary names, feature boundaries, `git mv` sequencing, assets/tests, and deprecation strategy.
9. **ADR-0010: Avatar port and 3D adapter boundary** — responsibility split among `nexa-avatar`, `nexa-nbp`, `nexa-events`, `nexa-3d`, and the viewer composition root.
10. **ADR-0011: 3D acceptance evidence and platform matrix** — headless versus GPU gates, canonical GLB/reference ownership, image regression tolerances, and supported backends.

## Recommended next action

Approve ADR-0002 through ADR-0004 and a narrow MVP conformance profile before adding contract crates. Then implement `nexa-domain` primitives and golden fixtures as the first independently reviewable change. Keep the current runtime at the repository root until the kernel passes its contract tests; begin the 3D migration only as a mechanical, behavior-preserving series after that gate.
