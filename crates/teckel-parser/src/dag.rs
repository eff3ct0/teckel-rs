//! DAG utilities: topological sort for execution plans and tag propagation (§18.7).

use petgraph::algo::toposort;
use petgraph::graph::DiGraph;
use std::collections::BTreeMap;
use teckel_model::asset::Context;
use teckel_model::types::AssetRef;
use teckel_model::{Pipeline, TeckelError, TeckelErrorCode};

/// Produce a topologically-sorted execution plan from the asset context.
///
/// Returns asset refs in dependency order: inputs first, then transformations,
/// then outputs. Each asset only appears after all of its dependencies.
pub fn execution_plan(context: &Context) -> Result<Vec<AssetRef>, TeckelError> {
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

    match toposort(&graph, None) {
        Ok(order) => Ok(order
            .into_iter()
            .map(|idx| graph[idx].to_string())
            .collect()),
        Err(_) => Err(TeckelError::spec(
            TeckelErrorCode::ECycle001,
            "circular dependency detected — cannot produce execution plan",
        )),
    }
}

/// Propagate tags downstream through the DAG (§18.7).
///
/// For each asset in topological order:
/// - Inherit tags from all upstream assets
/// - Apply `remove_tags` to strip inherited tags
/// - Merge with the asset's own declared tags
pub fn propagate_tags(pipeline: &mut Pipeline) -> Result<(), TeckelError> {
    let order = execution_plan(&pipeline.context)?;

    // Build a snapshot of tags per asset after propagation
    let mut resolved_tags: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for name in &order {
        let asset = match pipeline.context.get(name) {
            Some(a) => a,
            None => continue,
        };

        // Collect inherited tags from all dependencies
        let mut inherited: Vec<String> = Vec::new();
        for dep in asset.source.dependencies() {
            if let Some(dep_tags) = resolved_tags.get(dep.as_str()) {
                for tag in dep_tags {
                    if !inherited.contains(tag) {
                        inherited.push(tag.clone());
                    }
                }
            }
        }

        // Remove tags specified in remove_tags
        let remove_set = &asset.metadata.remove_tags;
        inherited.retain(|t| !remove_set.contains(t));

        // Merge with own declared tags (own tags take priority / appear first)
        let own_tags = &asset.metadata.tags;
        let mut final_tags = own_tags.clone();
        for tag in inherited {
            if !final_tags.contains(&tag) {
                final_tags.push(tag);
            }
        }

        resolved_tags.insert(name.clone(), final_tags);
    }

    // Write back resolved tags
    for (name, tags) in resolved_tags {
        if let Some(asset) = pipeline.context.get_mut(&name) {
            asset.metadata.tags = tags;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use teckel_model::asset::{Asset, AssetMetadata};
    use teckel_model::source::*;

    fn make_input(name: &str, tags: &[&str]) -> (String, Asset) {
        (
            name.to_string(),
            Asset {
                asset_ref: name.to_string(),
                source: Source::Input(InputSource {
                    format: "csv".to_string(),
                    path: "test.csv".to_string(),
                    options: Default::default(),
                }),
                metadata: AssetMetadata {
                    tags: tags.iter().map(|s| s.to_string()).collect(),
                    ..Default::default()
                },
            },
        )
    }

    fn make_where(name: &str, from: &str, tags: &[&str], remove: &[&str]) -> (String, Asset) {
        (
            name.to_string(),
            Asset {
                asset_ref: name.to_string(),
                source: Source::Where(WhereTransform {
                    from: from.to_string(),
                    filter: "x > 0".to_string(),
                }),
                metadata: AssetMetadata {
                    tags: tags.iter().map(|s| s.to_string()).collect(),
                    remove_tags: remove.iter().map(|s| s.to_string()).collect(),
                    ..Default::default()
                },
            },
        )
    }

    #[test]
    fn execution_plan_sorts_topologically() {
        let mut context = Context::new();
        let (k, v) = make_input("src", &[]);
        context.insert(k, v);
        let (k, v) = make_where("filtered", "src", &[], &[]);
        context.insert(k, v);

        let plan = execution_plan(&context).unwrap();
        let src_idx = plan.iter().position(|s| s == "src").unwrap();
        let filt_idx = plan.iter().position(|s| s == "filtered").unwrap();
        assert!(src_idx < filt_idx);
    }

    #[test]
    fn tag_propagation_inherits() {
        let mut pipeline = Pipeline::default();
        let (k, v) = make_input("src", &["pii", "sensitive"]);
        pipeline.context.insert(k, v);
        let (k, v) = make_where("filtered", "src", &["processed"], &[]);
        pipeline.context.insert(k, v);

        propagate_tags(&mut pipeline).unwrap();

        let tags = &pipeline.context.get("filtered").unwrap().metadata.tags;
        assert!(tags.contains(&"processed".to_string()));
        assert!(tags.contains(&"pii".to_string()));
        assert!(tags.contains(&"sensitive".to_string()));
    }

    #[test]
    fn tag_propagation_removes() {
        let mut pipeline = Pipeline::default();
        let (k, v) = make_input("src", &["pii", "sensitive", "raw"]);
        pipeline.context.insert(k, v);
        let (k, v) = make_where("cleaned", "src", &["clean"], &["pii", "raw"]);
        pipeline.context.insert(k, v);

        propagate_tags(&mut pipeline).unwrap();

        let tags = &pipeline.context.get("cleaned").unwrap().metadata.tags;
        assert!(tags.contains(&"clean".to_string()));
        assert!(tags.contains(&"sensitive".to_string()));
        assert!(!tags.contains(&"pii".to_string()));
        assert!(!tags.contains(&"raw".to_string()));
    }

    #[test]
    fn tag_propagation_multi_hop() {
        let mut pipeline = Pipeline::default();
        let (k, v) = make_input("a", &["origin"]);
        pipeline.context.insert(k, v);
        let (k, v) = make_where("b", "a", &["mid"], &[]);
        pipeline.context.insert(k, v);
        let (k, v) = make_where("c", "b", &["final"], &[]);
        pipeline.context.insert(k, v);

        propagate_tags(&mut pipeline).unwrap();

        let tags = &pipeline.context.get("c").unwrap().metadata.tags;
        assert!(tags.contains(&"origin".to_string()));
        assert!(tags.contains(&"mid".to_string()));
        assert!(tags.contains(&"final".to_string()));
    }
}
