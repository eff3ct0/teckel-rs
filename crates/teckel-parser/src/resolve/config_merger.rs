use serde_yaml::Value;

/// Deep-merge two YAML values (Section 21).
///
/// - Objects: deep merge (overlay fields override base fields recursively)
/// - Arrays: replaced entirely by the overlay
/// - Scalars: replaced by the overlay
pub fn deep_merge(base: Value, overlay: Value) -> Value {
    match (base, overlay) {
        (Value::Mapping(mut base_map), Value::Mapping(overlay_map)) => {
            for (key, overlay_val) in overlay_map {
                let merged = if let Some(base_val) = base_map.remove(&key) {
                    deep_merge(base_val, overlay_val)
                } else {
                    overlay_val
                };
                base_map.insert(key, merged);
            }
            Value::Mapping(base_map)
        }
        // Arrays and scalars: overlay wins
        (_, overlay) => overlay,
    }
}

/// Merge multiple YAML documents left to right.
/// The rightmost document has the highest precedence.
pub fn merge_documents(documents: Vec<Value>) -> Option<Value> {
    documents.into_iter().reduce(deep_merge)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deep_merges_objects() {
        let base: Value = serde_yaml::from_str("a: 1\nb:\n  c: 2\n  d: 3").unwrap();
        let overlay: Value = serde_yaml::from_str("b:\n  c: 99\n  e: 4").unwrap();
        let merged = deep_merge(base, overlay);
        let result: Value = serde_yaml::from_str("a: 1\nb:\n  c: 99\n  d: 3\n  e: 4").unwrap();
        assert_eq!(merged, result);
    }

    #[test]
    fn replaces_arrays() {
        let base: Value = serde_yaml::from_str("items:\n  - a\n  - b").unwrap();
        let overlay: Value = serde_yaml::from_str("items:\n  - x").unwrap();
        let merged = deep_merge(base, overlay);
        let result: Value = serde_yaml::from_str("items:\n  - x").unwrap();
        assert_eq!(merged, result);
    }
}
