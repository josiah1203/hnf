//! Path-level JSON tree diff (fallback for domains without object-graph rules).

use serde_json::Value;

use super::{DiffChange, DiffChangeType};

pub fn diff_values(left: &Value, right: &Value, prefix: &str) -> Vec<DiffChange> {
    match (left, right) {
        (Value::Object(l), Value::Object(r)) => {
            let mut changes = Vec::new();
            let keys: std::collections::BTreeSet<&str> =
                l.keys().map(|k| k.as_str()).chain(r.keys().map(|k| k.as_str())).collect();
            for key in keys {
                let path = join_path(prefix, key);
                match (l.get(key), r.get(key)) {
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
                    (Some(lv), Some(rv)) => changes.extend(diff_values(lv, rv, &path)),
                    (None, None) => {}
                }
            }
            changes
        }
        (Value::Array(l), Value::Array(r)) => {
            let mut changes = Vec::new();
            let max_len = l.len().max(r.len());
            for i in 0..max_len {
                let path = format!("{prefix}[{i}]");
                match (l.get(i), r.get(i)) {
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
                    (Some(lv), Some(rv)) => changes.extend(diff_values(lv, rv, &path)),
                    (None, None) => {}
                }
            }
            changes
        }
        _ if left == right => Vec::new(),
        _ => vec![DiffChange {
            path: prefix.to_string(),
            change_type: DiffChangeType::Modified,
            old_value: Some(left.clone()),
            new_value: Some(right.clone()),
        }],
    }
}

fn join_path(prefix: &str, key: &str) -> String {
    if prefix.is_empty() {
        key.to_string()
    } else {
        format!("{prefix}.{key}")
    }
}
