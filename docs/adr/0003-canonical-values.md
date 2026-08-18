# ADR-0003: Canonical identifiers, time, duration, version, and confidence

- **Status:** Accepted
- **Date:** 2026-08-18

## Decision

- IDs are transparent, non-nil UUID newtypes. Parsing rejects nil UUIDs. Creation is outside the contract crates; UUIDv7 is recommended by the baseline but generation requires an injected clock/random source.
- Persistent time is `Timestamp`, a UTC instant serialized as RFC 3339 with second or subsecond precision. Offset inputs are normalized to UTC. Monotonic elapsed time is never represented by `Timestamp`.
- Wire durations are `DurationMs(u64)`. They are semantic durations, not deadlines and not wall-clock instants.
- `ProtocolVersion { major: u16, minor: u16 }` serializes as `MAJOR.MINOR`. It is used as a representation by protocols and schemas without coupling their release cadence.
- `Confidence` is a finite inclusive `[0, 1]` value. JSON numbers are accepted; non-finite and out-of-range values are errors.
- Ordered stream positions are `Sequence(u64)`; scope and progression are defined by the owning envelope ADR.

All constructors and deserializers enforce the same invariants and return structured errors.

## Consequences

UUID generation, test clocks, and arithmetic belong in adapters. Milliseconds resolve the specifications' duration examples without silently treating them as timestamps. The broad NEXA-DOM-001 identifier inventory remains deferred until a first consumer requires each type.
