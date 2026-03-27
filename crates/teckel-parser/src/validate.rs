use crate::yaml;
use petgraph::algo::toposort;
use petgraph::graph::DiGraph;
use regex::Regex;
use std::collections::{BTreeMap, HashSet};
use std::sync::LazyLock;
use teckel_model::asset::Context;
use teckel_model::{TeckelError, TeckelErrorCode};

static ASSET_REF_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z][a-zA-Z0-9_-]{0,127}$").unwrap());

/// V-008: Check that the version field is present and equals "2.0".
pub fn check_version(doc: &yaml::Document) -> Result<(), TeckelError> {
    if doc.version != "2.0" {
        return Err(TeckelError::spec(
            TeckelErrorCode::EVersion001,
            format!("unsupported version \"{}\", expected \"2.0\"", doc.version),
        ));
    }
    Ok(())
}

/// Run all semantic validation rules (V-001 through V-008) on the domain context.
pub fn validate(context: &Context, doc: &yaml::Document) -> Result<(), TeckelError> {
    let mut errors = Vec::new();

    // V-007: AssetRef format validation
    validate_asset_ref_format(doc, &mut errors);

    // V-005: Non-empty lists
    validate_non_empty_lists(doc, &mut errors);

    // V-001: Reference integrity
    validate_references(context, &mut errors);

    // V-002: No cycles
    validate_no_cycles(context, &mut errors);

    // V-003: Output references must point to inputs or transformations
    validate_output_references(context, doc, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        let count = errors.len();
        Err(TeckelError::Validation { count, errors })
    }
}

/// V-007: All AssetRef values must match `^[a-zA-Z][a-zA-Z0-9_-]{0,127}$`.
fn validate_asset_ref_format(doc: &yaml::Document, errors: &mut Vec<TeckelError>) {
    for input in &doc.input {
        if !ASSET_REF_PATTERN.is_match(&input.name) {
            errors.push(TeckelError::spec(
                TeckelErrorCode::EName002,
                format!(
                    "invalid AssetRef \"{}\" -- must start with a letter",
                    input.name
                ),
            ));
        }
    }
    if let Some(transformations) = &doc.transformation {
        for t in transformations {
            if !ASSET_REF_PATTERN.is_match(&t.name) {
                errors.push(TeckelError::spec(
                    TeckelErrorCode::EName002,
                    format!(
                        "invalid AssetRef \"{}\" -- must start with a letter",
                        t.name
                    ),
                ));
            }
        }
    }
}

/// V-005: Fields typed as NonEmptyList must contain at least one element.
fn validate_non_empty_lists(doc: &yaml::Document, errors: &mut Vec<TeckelError>) {
    if doc.input.is_empty() {
        errors.push(TeckelError::spec(
            TeckelErrorCode::EList001,
            "\"input\" must contain at least one element",
        ));
    }
    if doc.output.is_empty() {
        errors.push(TeckelError::spec(
            TeckelErrorCode::EList001,
            "\"output\" must contain at least one element",
        ));
    }
}

/// V-001: For each AssetRef used in from/left/right/sources/views/output.name,
/// there must exist an asset with that name.
fn validate_references(context: &Context, errors: &mut Vec<TeckelError>) {
    let known_assets: HashSet<&str> = context.keys().map(|s| s.as_str()).collect();

    for (name, asset) in context {
        for dep in asset.source.dependencies() {
            // Output assets have "output_" prefix, their dependency is the referenced asset
            if !known_assets.contains(dep.as_str()) {
                errors.push(TeckelError::spec(
                    TeckelErrorCode::ERef001,
                    format!(
                        "asset \"{name}\" references undefined asset \"{dep}\"{}",
                        suggest_similar(dep, &known_assets)
                    ),
                ));
            }
        }
    }
}

/// V-002: The asset dependency graph must be a DAG (no cycles).
fn validate_no_cycles(context: &Context, errors: &mut Vec<TeckelError>) {
    let mut graph = DiGraph::<&str, ()>::new();
    let mut indices = BTreeMap::new();

    for name in context.keys() {
        let idx = graph.add_node(name.as_str());
        indices.insert(name.as_str(), idx);
    }

    for (name, asset) in context {
        if let Some(&to_idx) = indices.get(name.as_str()) {
            for dep in asset.source.dependencies() {
                if let Some(&from_idx) = indices.get(dep.as_str()) {
                    graph.add_edge(from_idx, to_idx, ());
                }
            }
        }
    }

    if toposort(&graph, None).is_err() {
        errors.push(TeckelError::spec(
            TeckelErrorCode::ECycle001,
            "circular dependency detected in the pipeline DAG",
        ));
    }
}

/// V-003: Output names must reference assets from input or transformation, not other outputs.
fn validate_output_references(
    _context: &Context,
    doc: &yaml::Document,
    errors: &mut Vec<TeckelError>,
) {
    let input_and_transform_names: HashSet<&str> = {
        let mut names: HashSet<&str> = doc.input.iter().map(|i| i.name.as_str()).collect();
        if let Some(ts) = &doc.transformation {
            for t in ts {
                names.insert(&t.name);
            }
        }
        names
    };

    for output in &doc.output {
        if !input_and_transform_names.contains(output.name.as_str()) {
            // Check if it's referencing another output
            let is_output_ref = doc
                .output
                .iter()
                .any(|o| !std::ptr::eq(o, output) && o.name == output.name);
            if !is_output_ref {
                errors.push(TeckelError::spec(
                    TeckelErrorCode::ERef001,
                    format!("output \"{}\" references undefined asset", output.name),
                ));
            }
        }
    }
}

/// Suggest a similar asset name using Levenshtein distance.
fn suggest_similar(name: &str, known: &HashSet<&str>) -> String {
    let mut best: Option<(&str, usize)> = None;
    for &candidate in known {
        let dist = levenshtein(name, candidate);
        if dist <= 3 && (best.is_none() || dist < best.unwrap().1) {
            best = Some((candidate, dist));
        }
    }
    match best {
        Some((suggestion, _)) => format!(" (did you mean \"{suggestion}\"?)"),
        None => String::new(),
    }
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let b_len = b_bytes.len();

    let mut prev = (0..=b_len).collect::<Vec<_>>();
    let mut curr = vec![0; b_len + 1];

    for (i, &a_byte) in a_bytes.iter().enumerate() {
        curr[0] = i + 1;
        for (j, &b_byte) in b_bytes.iter().enumerate() {
            let cost = if a_byte == b_byte { 0 } else { 1 };
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[b_len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_version() {
        let doc = yaml::Document {
            version: "1.0".to_string(),
            pipeline: None,
            config: None,
            secrets: None,
            hooks: None,
            quality: None,
            templates: None,
            input: vec![],
            streaming_input: None,
            transformation: None,
            output: vec![],
            streaming_output: None,
            exposures: None,
        };
        assert!(check_version(&doc).is_err());
    }

    #[test]
    fn levenshtein_distance() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("abc", "abc"), 0);
        assert_eq!(levenshtein("", "abc"), 3);
    }
}
