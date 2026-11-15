//! Rust replacement for the repository identity / conflict / secret audit.
//! Intentionally avoids new dependencies or executing untrusted source files.

use evgl_interfaces::validation::{has_conflict_markers, has_credential_markers};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const EXPECTED_PACKAGE: &str = "evento-globolo/evgl-interfaces";
const EXPECTED_REPOSITORY: &str = "https://github.com/evento-globolo/evgl-interfaces";

fn read_text(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))
}

fn tables(input: &str) -> Result<BTreeMap<String, BTreeMap<String, String>>, String> {
    let mut tables = BTreeMap::<String, BTreeMap<String, String>>::new();
    let mut current = String::new();
    for (index, raw) in input.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(section) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            if section.is_empty() || tables.contains_key(section) {
                return Err(format!("duplicate or empty manifest table on line {}", index + 1));
            }
            current = section.to_owned();
            tables.insert(current.clone(), BTreeMap::new());
            continue;
        }
        if current.is_empty() {
            return Err(format!("manifest key outside a table on line {}", index + 1));
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("malformed manifest assignment on line {}", index + 1))?;
        let key = key.trim().trim_matches('"');
        if key.is_empty() {
            return Err(format!("empty manifest key on line {}", index + 1));
        }
        let values = tables.get_mut(&current).expect("current table exists");
        if values.insert(key.to_owned(), value.trim().to_owned()).is_some() {
            return Err(format!("duplicate manifest key on line {}", index + 1));
        }
    }
    Ok(tables)
}

fn value<'a>(
    tables: &'a BTreeMap<String, BTreeMap<String, String>>,
    section: &str,
    key: &str,
) -> Option<&'a str> {
    tables.get(section)?.get(key).map(|s| s.trim_matches('"'))
}

fn verify_manifest(root: &Path) -> Result<(), String> {
    let source = read_text(&root.join(".zpkg.toml"))?;
    let manifest = tables(&source)?;
    let coordinate = format!(
        "{}/{}",
        value(&manifest, "package", "org").unwrap_or(""),
        value(&manifest, "package", "name").unwrap_or("")
    );
    if coordinate != EXPECTED_PACKAGE {
        return Err(format!("unexpected Zed package identity: {coordinate}"));
    }
    for (table, key, expected) in [
        ("package", "version", "0.1.0"),
        ("package", "language", "universal"),
        ("package.repository", "vcs", "git"),
        ("package.repository", "url", EXPECTED_REPOSITORY),
        ("publish", "tag_format", "v{version}"),
    ] {
        if value(&manifest, table, key) != Some(expected) {
            return Err(format!("invalid manifest key {table}.{key}"));
        }
    }
    if manifest.get("dependencies").is_some_and(|items| !items.is_empty()) {
        return Err("interface package must remain a dependency root".to_owned());
    }
    if manifest.get("targets.repository").is_some()
        && value(&manifest, "targets.repository", "dir") != Some(".")
    {
        return Err("repository target must publish the repository root".to_owned());
    }
    Ok(())
}

fn verify_tree(root: &Path) -> Result<(), String> {
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let entries = fs::read_dir(&directory)
            .map_err(|e| format!("cannot inspect {}: {e}", directory.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("directory read: {e}"))?;
            let path = entry.path();
            let ty = entry.file_type().map_err(|e| format!("file type: {e}"))?;
            if ty.is_symlink() {
                continue;
            }
            if ty.is_dir() {
                let name = entry.file_name();
                if ![".git", "target", "node_modules", ".vendor"]
                    .iter()
                    .any(|ignored| name == *ignored)
                {
                    stack.push(path);
                }
            } else if ty.is_file() {
                let metadata = entry.metadata().map_err(|e| format!("metadata: {e}"))?;
                if metadata.len() > 1_000_000 {
                    continue;
                }
                let bytes = fs::read(&path)
                    .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
                let Ok(text) = String::from_utf8(bytes) else {
                    continue;
                };
                if has_conflict_markers(&text) {
                    return Err(format!("conflict marker in {}", path.display()));
                }
                if has_credential_markers(&text) {
                    return Err(format!("credential-shaped content in {}", path.display()));
                }
            }
        }
    }
    Ok(())
}

fn verify_repository(root: &Path) -> Result<(), String> {
    let meta: Value = serde_json::from_str(&read_text(&root.join("project.json"))?)
        .map_err(|e| format!("project.json: {e}"))?;
    let required_paths = meta["required_paths"]
        .as_array()
        .ok_or("project.json required_paths must be an array")?;
    for required in [
        "README.md", "AGENTS.md", "project.json", "docs/architecture.md", ".zpkg.toml",
    ] {
        if !root.join(required).exists() {
            return Err(format!("missing required path: {required}"));
        }
    }
    for item in required_paths {
        let path = item.as_str().ok_or("required path must be a string")?;
        if !root.join(path).exists() {
            return Err(format!("missing required path: {path}"));
        }
    }
    verify_tree(root)?;
    verify_manifest(root)?;
    if meta["organization"] != "evento-globolo" || meta["repository"] != "evgl-interfaces" {
        return Err("project.json repository identity does not match".to_owned());
    }
    Ok(())
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    if let Err(message) = verify_repository(root) {
        eprintln!("repository verification failed: {message}");
        std::process::exit(1);
    }
    println!("validated {EXPECTED_PACKAGE} (Rust verifier)");
}
