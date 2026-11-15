//! Dependency-free repository guard primitives shared by Rust CI and tests.
//! Parsing/formatting of JSON Schema is kept distinct from wire-value validation.

use serde_json::Value;

/// Match only full Git conflict-marker lines, never code quoting a marker regex.
pub fn has_conflict_markers(text: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim_end_matches('\r');
        for prefix in ["<<<<<<<", ">>>>>>>"] {
            if let Some(rest) = line.strip_prefix(prefix) {
                if rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t') {
                    return true;
                }
            }
        }
        line == "======="
    })
}

/// Reject credential-shaped source. Does not echo the value into CI logs.
pub fn has_credential_markers(text: &str) -> bool {
    for prefix in ["ghp_", "gho_", "ghu_", "ghs_", "ghr_", "lin_api_"] {
        for (at, _) in text.match_indices(prefix) {
            let bytes = &text.as_bytes()[at + prefix.len()..];
            if bytes
                .iter()
                .take_while(|byte| byte.is_ascii_alphanumeric())
                .count() >= 20
            {
                return true;
            }
        }
    }
    text.lines().any(|line| {
        let Some((_, tail)) = line.split_once("BEGIN ") else {
            return false;
        };
        let Some((kind, _)) = tail.split_once("PRIVATE KEY") else {
            return false;
        };
        kind.chars().all(|ch| ch.is_ascii_uppercase() || ch == ' ')
    })
}

/// Structural JSON Schema checks. Descend only through subschema applicators,
/// not through arbitrary maps containing properties named "type"/"required".
pub fn schema_keyword_errors(document: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    let mut stack = vec![document];
    const MAPS: &[&str] = &[
        "properties", "patternProperties", "$defs", "definitions", "dependentSchemas",
    ];
    const SINGLES: &[&str] = &[
        "additionalProperties", "unevaluatedProperties", "items", "contains",
        "propertyNames", "not", "if", "then", "else", "contentSchema",
    ];
    const LISTS: &[&str] = &["allOf", "anyOf", "oneOf", "prefixItems"];

    while let Some(node) = stack.pop() {
        let Some(fields) = node.as_object() else {
            continue;
        };
        for key in MAPS {
            if let Some(value) = fields.get(*key) {
                if let Some(children) = value.as_object() {
                    for child in children.values() {
                        stack.push(child);
                    }
                } else {
                    errors.push(format!("'{key}' must be a dict"));
                }
            }
        }
        for key in ["required", "enum", "allOf", "anyOf", "oneOf", "prefixItems"] {
            if fields.get(key).is_some_and(|value| !value.is_array()) {
                errors.push(format!("'{key}' must be a list"));
            }
        }
        if fields.get("type").is_some_and(|value| {
            !value.is_string() && !value.is_array()
        }) {
            errors.push("'type' must be a string or array".to_owned());
        }
        for key in SINGLES {
            if let Some(child) = fields.get(*key) {
                if child.is_object() {
                    stack.push(child);
                }
            }
        }
        for key in LISTS {
            if let Some(children) = fields.get(*key).and_then(Value::as_array) {
                for child in children {
                    stack.push(child);
                }
            }
        }
    }
    errors
}
