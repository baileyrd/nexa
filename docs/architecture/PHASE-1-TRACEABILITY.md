# Phase 1 contract-kernel traceability

This matrix defines the normative surface of the first implementation increment. Specification examples outside this table are informative and remain deferred. `Public` means non-secret contract data; payload-specific privacy classification remains the producer's responsibility.

| Specification source | Requirement/type | Canonical code owner | Wire name/representation | Validation | Privacy | Initial consumer/status |
|---|---|---|---|---|---|---|
| DOM §§4–6 | Session/Event/Message/Behavior/Correlation/Trace IDs | `nexa-domain::ids` | transparent UUID string | non-nil UUID | Public identifiers; not authentication | Events/NBP, implemented |
| DOM §8 | protocol/schema version | `nexa-domain::ProtocolVersion` | `MAJOR.MINOR` | two `u16` components | Public | Events/NBP, implemented |
| DOM §9 | persistent timestamp | `nexa-domain::Timestamp` | RFC 3339 UTC | valid instant; normalize UTC | Operational | Events/NBP, implemented |
| DOM §10 | confidence | `nexa-domain::Confidence` | JSON number | finite `[0,1]` | Payload-dependent | NBP emotion, implemented |
| NBP §§15,18,29 | durations | `nexa-domain::DurationMs` | integer milliseconds | `u64` | Public | NBP, implemented |
| EVT §§7–8,82 | typed event envelope | `nexa-events::Event<T>` | named JSON object | validated value objects | Payload-dependent | Local integration, implemented |
| EVT §§9,106 | MVP event identity/catalog | `nexa-events::EventKind` | dotted semantic string | closed MVP catalog | Public | Subscribers, implemented |
| EVT §§43–48 | sequence and delivery | domain/events + ADR-0005 | integer/at-least-once target | `(source,session)` scope | Operational | Consumers, documented/tested |
| EVT §§55–56 | replay | event envelope + ADR-0005 | original envelope | preserve identity/order | Governed later | Persistence deferred |
| EVT §§85–86 | event bus | `nexa-events::InProcessEventBus` | not a wire type | subscriber isolation | In-process | Tests/local use, implemented adapter |
| NBP §§5–6 | envelope | `nexa-nbp::NbpMessage` | named JSON object | type/payload agreement; v1 major | Payload-dependent | Avatar port, implemented |
| NBP §§7–30,74 | behavior command/cancel | `nexa-nbp` behavior types | tagged payload | priority/ranges; MVP vocabulary | May contain learner text | Fake/headless adapter, implemented |
| NBP §§35–40,74 | ack/state/error | `nexa-nbp::Payload` | tagged payload | typed statuses/severity | Operational | Orchestrator, implemented |
| AVTR §§19–20,83–86; 3D §§3,104,212 | avatar command/cancel port and adapter direction | `nexa-avatar::AvatarPort`, root `avatar::NexaAvatarAdapter` | renderer-neutral request/report | semantic input only; no renderer types | May contain learner speech text | Fake and existing 3D adapters, implemented |
| AVTR §§12,87,172; 3D §§17,165 | capability discovery and graceful degradation | `nexa-avatar::AvatarCapabilities`, `AvatarReport` | ordered semantic capability set; NBP status/error | unsupported optional facilities report recoverable degradation | Operational | Deterministic fake adapter, implemented locally; wire negotiation completed by ADR-0009 / Phase 2 |
| NBP §§61–66 | JSON/version/extensions | `nexa-nbp` + ADR-0004/0006 | JSON/extensions object | namespaced object values | Payload-dependent | All peers, implemented |

## Deferred or unresolved

- NEXA-DOM-001's remaining aggregate and identifier inventory has no first-increment consumer and is intentionally not transcribed.
- NEXA-EVT-001's command envelope, durable store/replay context, privacy retention, async backpressure, and full event payload catalog require owners and adapters.
- NEXA-NBP-001 alternates between graceful handling of future states and strongly typed enum validation. ADR-0004 rejects unknown required states for safety; a future minor-version capability design may introduce an explicit extension state.
- NBP update merge/race rules, arbitration, and canvas messages remain deferred. Phase 2 implements synchronous capability negotiation and acceptance; asynchronous transport scheduling remains deferred by ADR-0009.
- Formal JSON Schemas and automated backward-compatibility comparison are required before the contracts are promoted from Baseline Draft; Phase 1 uses reviewed golden fixtures.
