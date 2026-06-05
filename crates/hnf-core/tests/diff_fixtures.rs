//! Integration tests for `hnf_core::diff` using fixtures.

use hnf_core::{
    diff_domain, diff_schematic, parse_schematic, serialize_schematic, uses_object_graph_diff,
    DiffChangeType,
};

fn load_fixture(name: &str) -> serde_json::Value {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse {path}: {e}"))
}

#[test]
fn diff_schematic_fixture_no_changes_when_identical() {
    let value = load_fixture("schematic_valid.json");
    let domain = parse_schematic(&value).expect("parse");
    let changes = diff_schematic(&serialize_schematic(&domain), &serialize_schematic(&domain));
    assert!(changes.is_empty());
}

#[test]
fn diff_domain_bom_fixture_detects_quantity_change() {
    let left = load_fixture("bom_valid.json");
    let mut right = left.clone();
    right["properties"]["lines"][0]["quantity"] = 2.into();
    let changes = diff_domain(&left, &right);
    assert!(changes.iter().any(|c| c.change_type == DiffChangeType::Modified));
}

#[test]
fn diff_domain_layout_uses_path_fallback() {
    let left = load_fixture("layout_valid.json");
    let mut right = left.clone();
    right["properties"]["tracks"][0]["net"] = "VCC".into();
    assert!(!uses_object_graph_diff("layout"));
    let changes = diff_domain(&left, &right);
    assert!(changes.iter().any(|c| c.path.contains("net")));
}

#[test]
fn diff_schematic_value_change_via_domain_entry() {
    let left = load_fixture("schematic_valid.json");
    let mut right = left.clone();
    right["properties"]["symbols"][0]["value"] = "22k".into();
    let changes = diff_domain(&left, &right);
    assert!(changes.iter().any(|c| c.change_type == DiffChangeType::Modified));
}
