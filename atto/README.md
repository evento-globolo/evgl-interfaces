# AttoLang event interface (preview)

`domain_event.atto` is an AttoLang source-level representation of the public
`DomainEvent` fields from [the canonical JSON Schema](../schemas/event.schema.json).
The Atto source uses snake_case methods, including `get_occurred_at`, while
the wire schema uses the exact JSON key `occurredAt`. The `*_wire_key`
functions keep this distinction explicit for prospective serializers.

The `has_required_fields` method checks only nonempty text values. It is
**not** equivalent to JSON Schema validation: the canonical schema also
requires a date-time string, rejects additional keys, and governs JSON
encoding. No Atto HTTP client, JSON codec, validator, or cross-backend
compilation support is implied by this source.

The Rust test `tests/atto_contract.rs` checks the source-level key
declarations against the JSON Schema and protects against drift. Compiler
parsing, execution, JSON round-trip, and wire compatibility still require
separate Atto compiler/runtime conformance tests before publication.
