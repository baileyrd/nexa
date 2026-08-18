# ADR-0005: Event identity, ordering, correlation, replay, and delivery

- **Status:** Accepted
- **Date:** 2026-08-18

## Decision

An `event_id` identifies one immutable fact forever and is the deduplication key. `sequence`, when present, is monotonically increasing within the tuple `(source, session_id)`; gaps are allowed, rollover is not. No global order is implied. Consumers must tolerate duplicates and out-of-order arrival.

`correlation_id` groups a logical operation. `causation_id` names the immediately causing event (command causation is deferred because command IDs are not yet shared in this increment). `trace_id` groups distributed diagnostic work and is not an authorization identity. These values propagate unchanged across derived work.

The target delivery contract is at-least-once. Publication acceptance is not durable success. Subscribers own idempotency and business decisions. Replay republishes original envelopes, including IDs and timestamps, in recorded stream order; replay is therefore distinguishable only by an out-of-band replay context, deferred with persistence/privacy policy.

The initial in-memory bus is a deterministic test/local adapter: synchronous fan-out in subscription order, isolated subscriber errors, no durability, and no delivery retry. It reports failures after attempting every matching subscriber. This weaker adapter does not redefine the target delivery contract.

## Consequences

Durable acknowledgements, retention/redaction, replay authorization, bounded asynchronous backpressure, commands, and dead-letter handling require later adapters/ADRs. The kernel does not pretend an in-process callback is durable delivery.
