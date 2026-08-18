# ADR-0004: Serialization and wire-format compatibility

- **Status:** Accepted
- **Date:** 2026-08-18

## Decision

Canonical Phase 1 wire data is UTF-8 JSON. Field and variant names are `snake_case`; UUIDs use hyphenated lowercase text; times use UTC RFC 3339; versions use `MAJOR.MINOR`; optional absent values are omitted. Maps are not canonically ordered and serialized bytes are not suitable for signatures.

Envelopes and payload structs accept unknown fields to permit additive minor evolution. Unknown enum values and unknown tagged message types are rejected because they carry semantics that cannot safely be inferred. A minor version may add optional fields or variants only when a receiver is not required to act on them. Removing/renaming fields, changing meaning or representation, and adding required semantics require a major version.

Golden JSON fixtures lock representative logical representations. Serde round trips plus fixture assertions are the initial conformance mechanism; formal schema generation is deferred and recorded in the traceability matrix.

## Consequences

This chooses tolerant readers rather than `deny_unknown_fields`. It does not claim JSON canonicalization. Binary formats require a later ADR and must preserve the logical schema.
