//! Object-graph diff for schematic and BOM domains (keyed collections).

use serde_json::{Map, Value};

use super::path::diff_values;
use super::{DiffChange, DiffChangeType};

pub fn diff_schematic(left: &Value, right: &Value) -> Vec<DiffChange> {
    let mut changes = diff_envelope_fields(left, right);
    changes.extend(diff_keyed_array(
        left,
        right,
        "properties.symbols",
        "id",
        &["refdes", "lib_id", "value"],
    ));
    changes.extend(diff_keyed_array(
        left,
        right,
        "properties.nets",
        "id",
        &["name", "net_class"],
    ));
    changes.extend(diff_keyed_array(
        left,
        right,
        "properties.power_domains",
        "id",
        &["name", "voltage"],
    ));
    changes.extend(diff_pins(left, right));
    changes
}

pub fn diff_bom(left: &Value, right: &Value) -> Vec<DiffChange> {
    let mut changes = diff_envelope_fields(left, right);
    changes.extend(diff_keyed_array(
        left,
        right,
        "properties.lines",
        "line_id",
        &[
            "quantity",
            "refdes",
            "mpn",
            "manufacturer",
            "supplier_ref",
            "lifecycle",
            "description",
        ],
    ));
    changes
}

fn diff_envelope_fields(left: &Value, right: &Value) -> Vec<DiffChange> {
    let fields = ["refs", "content_hash"];
    let mut changes = Vec::new();
    for field in fields {
        let path = field;
        let lv = left.get(field);
        let rv = right.get(field);
        if lv == rv {
            continue;
        }
        changes.push(DiffChange {
            path: path.into(),
            change_type: if lv.is_none() {
                DiffChangeType::Added
            } else if rv.is_none() {
                DiffChangeType::Removed
            } else {
                DiffChangeType::Modified
            },
            old_value: lv.cloned(),
            new_value: rv.cloned(),
        });
    }
    changes
}

fn diff_keyed_array(
    left: &Value,
    right: &Value,
    path: &str,
    key_field: &str,
    compare_fields: &[&str],
) -> Vec<DiffChange> {
    let left_items = array_at_path(left, path);
    let right_items = array_at_path(right, path);

    let left_map = index_by_key(&left_items, key_field);
    let right_map = index_by_key(&right_items, key_field);

    let keys: std::collections::BTreeSet<String> =
        left_map.keys().chain(right_map.keys()).cloned().collect();
    let mut changes = Vec::new();

    for key in keys {
        let item_path = format!("{path}[{key}]");
        match (left_map.get(&key), right_map.get(&key)) {
            (None, Some(rv)) => changes.push(DiffChange {
                path: item_path,
                change_type: DiffChangeType::Added,
                old_value: None,
                new_value: Some(rv.clone()),
            }),
            (Some(lv), None) => changes.push(DiffChange {
                path: item_path,
                change_type: DiffChangeType::Removed,
                old_value: Some(lv.clone()),
                new_value: None,
            }),
            (Some(lv), Some(rv)) => {
                for field in compare_fields {
                    let fp = format!("{item_path}.{field}");
                    let lfv = lv.get(*field);
                    let rfv = rv.get(*field);
                    if lfv != rfv {
                        changes.push(DiffChange {
                            path: fp,
                            change_type: DiffChangeType::Modified,
                            old_value: lfv.cloned(),
                            new_value: rfv.cloned(),
                        });
                    }
                }
            }
            (None, None) => {}
        }
    }

    changes
}

fn diff_pins(left: &Value, right: &Value) -> Vec<DiffChange> {
    let left_items = array_at_path(left, "properties.pins");
    let right_items = array_at_path(right, "properties.pins");

    let key = |item: &Value| -> String {
        format!(
            "{}#{}",
            item.get("symbol_id").and_then(|v| v.as_str()).unwrap_or(""),
            item.get("pin_number").and_then(|v| v.as_str()).unwrap_or("")
        )
    };

    let left_map: Map<String, Value> = left_items
        .iter()
        .map(|v| (key(v), v.clone()))
        .collect();
    let right_map: Map<String, Value> = right_items
        .iter()
        .map(|v| (key(v), v.clone()))
        .collect();

    let mut changes = Vec::new();
    let keys: std::collections::BTreeSet<String> =
        left_map.keys().chain(right_map.keys()).cloned().collect();

    for k in keys {
        let path = format!("properties.pins[{k}]");
        match (left_map.get(&k), right_map.get(&k)) {
            (None, Some(rv)) => changes.push(DiffChange {
                path,
                change_type: DiffChangeType::Added,
                old_value: None,
                new_value: Some(rv.clone()),
            }),
            (Some(lv), None) => changes.push(DiffChange {
                path,
                change_type: DiffChangeType::Removed,
                old_value: Some(lv.clone()),
                new_value: None,
            }),
            (Some(lv), Some(rv)) => {
                if lv.get("net_id") != rv.get("net_id") {
                    changes.push(DiffChange {
                        path: format!("{path}.net_id"),
                        change_type: DiffChangeType::Modified,
                        old_value: lv.get("net_id").cloned(),
                        new_value: rv.get("net_id").cloned(),
                    });
                }
            }
            (None, None) => {}
        }
    }

    changes
}

fn array_at_path(value: &Value, path: &str) -> Vec<Value> {
    let mut current = value;
    for segment in path.split('.') {
        current = current.get(segment).unwrap_or(&Value::Null);
    }
    current.as_array().cloned().unwrap_or_default()
}

fn index_by_key(items: &[Value], key_field: &str) -> Map<String, Value> {
    items
        .iter()
        .filter_map(|item| {
            item.get(key_field)
                .and_then(|v| v.as_str())
                .map(|k| (k.to_string(), item.clone()))
        })
        .collect()
}

/// Diff unknown domain payloads using path-level fallback.
pub fn diff_path_fallback(left: &Value, right: &Value) -> Vec<DiffChange> {
    diff_values(left, right, "")
}
