//! Structural diff for HNF domain payloads (v0.1).

mod object_graph;
mod path;

use serde_json::Value;

pub use object_graph::{diff_bom, diff_path_fallback, diff_schematic};
pub use path::diff_values;

/// Change classification for diff entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffChangeType {
    Added,
    Removed,
    Modified,
}

/// Single diff entry with JSON path and optional old/new values.
#[derive(Debug, Clone, PartialEq)]
pub struct DiffChange {
    pub path: String,
    pub change_type: DiffChangeType,
    pub old_value: Option<Value>,
    pub new_value: Option<Value>,
}

/// Diff two domain JSON values. Schematic and BOM use object-graph rules; others use path diff.
pub fn diff_domain(left: &Value, right: &Value) -> Vec<DiffChange> {
    let domain = left
        .get("domain")
        .and_then(|v| v.as_str())
        .or_else(|| right.get("domain").and_then(|v| v.as_str()));

    match domain {
        Some("schematic") => object_graph::diff_schematic(left, right),
        Some("bom") => object_graph::diff_bom(left, right),
        _ => object_graph::diff_path_fallback(left, right),
    }
}

/// Whether a domain uses object-graph diff (vs path fallback).
pub fn uses_object_graph_diff(domain: &str) -> bool {
    matches!(domain, "schematic" | "bom")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse_bom, parse_schematic, serialize_bom, serialize_schematic};

    #[test]
    fn schematic_symbol_value_change_is_modified() {
        let mut right = parse_schematic(&fixture("schematic_valid.json")).expect("parse");
        right.properties.symbols[0].value = Some("4k7".into());
        let left = parse_schematic(&fixture("schematic_valid.json")).expect("parse");

        let changes = diff_domain(&serialize_schematic(&left), &serialize_schematic(&right));
        assert!(changes.iter().any(|c| {
            c.change_type == DiffChangeType::Modified
                && c.path.contains("sym-r1")
                && c.path.ends_with("value")
        }));
    }

    #[test]
    fn schematic_added_net_is_added() {
        let left = parse_schematic(&fixture("schematic_valid.json")).expect("parse");
        let mut right = left.clone();
        right.properties.nets.push(crate::SchematicNet {
            id: "net-vcc".into(),
            name: "VCC".into(),
            net_class: None,
        });

        let changes = diff_domain(&serialize_schematic(&left), &serialize_schematic(&right));
        assert!(changes.iter().any(|c| {
            c.change_type == DiffChangeType::Added && c.path.contains("net-vcc")
        }));
    }

    #[test]
    fn bom_quantity_change_is_modified() {
        let left = parse_bom(&fixture("bom_valid.json")).expect("parse");
        let mut right = left.clone();
        right.properties.lines[0].quantity = 5;

        let changes = diff_domain(&serialize_bom(&left), &serialize_bom(&right));
        assert!(changes.iter().any(|c| {
            c.change_type == DiffChangeType::Modified && c.path.contains("quantity")
        }));
    }

    #[test]
    fn layout_domain_uses_path_fallback() {
        let left = fixture_value("layout_valid.json");
        let mut right = left.clone();
        right["properties"]["footprints"][0]["refdes"] = "U2".into();

        assert!(!uses_object_graph_diff("layout"));
        let changes = diff_domain(&left, &right);
        assert!(changes.iter().any(|c| c.path.contains("refdes")));
    }

    #[test]
    fn identical_domains_produce_no_changes() {
        let value = fixture_value("bom_valid.json");
        assert!(diff_domain(&value, &value).is_empty());
    }

    fn fixture(name: &str) -> Value {
        fixture_value(name)
    }

    fn fixture_value(name: &str) -> Value {
        let path = format!(
            "{}/tests/fixtures/{name}",
            env!("CARGO_MANIFEST_DIR")
        );
        let raw = std::fs::read_to_string(&path).expect("read fixture");
        serde_json::from_str(&raw).expect("parse fixture")
    }
}
