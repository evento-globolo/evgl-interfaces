//! Rust replacements for the legacy Python validation regression suite.
use evgl_interfaces::validation::{
    has_conflict_markers, has_credential_markers, schema_keyword_errors,
};
use serde_json::json;

#[test]
fn json_property_names_are_not_mistaken_for_schema_keywords() {
    let doc = json!({
        "type": "object",
        "properties": {
            "type": {"$ref": "#/$defs/scalarType"},
            "required": {"type": "boolean"}
        },
        "$defs": {"scalarType": {"anyOf": [{"type": "string"}]}},
        "required": ["type"]
    });
    assert!(schema_keyword_errors(&doc).is_empty());
}

#[test]
fn invalid_nested_schema_type_still_fails() {
    let doc = json!({"type": "object", "properties": {"value": {"type": {"bad": true}}}});
    assert!(
        schema_keyword_errors(&doc)
            .iter()
            .any(|msg| msg.contains("'type' must be a string or array"))
    );
}

#[test]
fn invalid_required_keyword_still_fails() {
    let doc = json!({"type": "object", "required": {"bad": "not a list"}});
    assert!(schema_keyword_errors(&doc).contains(&"'required' must be a list".to_owned()));
}

#[test]
fn only_whole_lines_count_as_merge_conflicts() {
    let text = format!(
        "{}{} HEAD\ncode\n{}{}\ncode\n{}{} main\n",
        "<<<", "<<<<", "===", "====", ">>>", ">>>>"
    );
    assert!(has_conflict_markers(&text));
    let workflow = format!("git grep -E '^(<{0} .+|={0}|>{0} .+)$'\n", 7);
    assert!(!has_conflict_markers(&workflow));
    assert!(!has_conflict_markers("x ======= y\n"));
}

#[test]
fn suspicious_source_secrets_are_rejected_without_logging_them() {
    let token = format!("{}{}", "ghp_", "a".repeat(20));
    assert!(has_credential_markers(&token));
    let key_header = format!("{}{}{}", "BEGIN ", "RSA ", "PRIVATE KEY");
    assert!(has_credential_markers(&key_header));
    assert!(!has_credential_markers("the sample token was redacted"));
}

#[test]
fn existing_event_schema_passes_structural_checks() {
    let document: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/event.schema.json"))
            .expect("canonical JSON Schema parses");
    assert!(schema_keyword_errors(&document).is_empty());
}
