//! Source-level guardrails for the experimental AttoLang public event binding.
//! These do not parse or execute AttoLang or prove JSON serialization parity.

use serde_json::Value;

const ATTO: &str = include_str!("../atto/domain_event.atto");
const EVENT_SCHEMA: &str = include_str!("../schemas/event.schema.json");

#[test]
fn atto_wire_keys_match_the_canonical_event_schema() {
    let schema: Value = serde_json::from_str(EVENT_SCHEMA).expect("valid event JSON Schema");
    let properties = schema["properties"].as_object().expect("event properties");
    let required = schema["required"].as_array().expect("required event fields");

    for (function, key) in [
        ("event_id_wire_key", "id"),
        ("event_type_wire_key", "type"),
        ("event_occurred_at_wire_key", "occurredAt"),
    ] {
        assert!(properties.contains_key(key), "schema missing {key}");
        assert!(
            required.iter().any(|value| value.as_str() == Some(key)),
            "field {key} must stay required"
        );
        let declaration = format!("pub fnc {function}(): string {{ return \"{key}\"; }}");
        assert!(ATTO.contains(&declaration), "Atto binding missing {declaration}");
    }
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["properties"]["occurredAt"]["format"], "date-time");
}

#[test]
fn atto_binding_does_not_claim_full_wire_validation() {
    assert!(ATTO.contains("define class DomainEvent as"));
    for method in ["get_id", "get_type", "get_occurred_at", "has_required_fields"] {
        assert!(ATTO.contains(&format!("pub {method}(")), "missing {method}");
    }
    assert!(ATTO.contains("Presence check only"));
    assert!(!ATTO.contains("json_encode("));
}
