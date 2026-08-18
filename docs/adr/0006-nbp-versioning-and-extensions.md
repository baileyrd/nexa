# ADR-0006: NBP protocol versioning and extension policy

- **Status:** Accepted
- **Date:** 2026-08-18

## Decision

NBP protocol versions are independent of crate and event schema versions. Receivers support their own major version and any minor version whose additional data is optional and safely ignorable. A major mismatch is `UnsupportedVersion`.

The externally tagged `payload` determines `message_type`; the Rust model emits both from one enum and validates their equality when reading, preventing mismatched payloads. Core types reject unknown required message/enum semantics.

Extensions are an optional object keyed by a reverse-DNS or product namespace followed by a dot and a local name (for example `live2d.physics_hint`). Extension values are JSON objects. Core logic must not depend on extensions, receivers may ignore them, and extensions may not override core fields or weaken validation.

## Consequences

This increment implements only the minimum runtime messages and semantic channels from NEXA-NBP-001. Behavior update, capability negotiation details, arbitration, and transition policy remain explicit subsequent increments rather than guessed behavior.
