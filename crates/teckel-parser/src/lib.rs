pub mod expr;
pub mod resolve;
pub mod rewrite;
pub mod validate;
pub mod yaml;

use teckel_model::{Context, Pipeline, TeckelError};

/// Parse a raw YAML string into a fully validated `Pipeline`.
///
/// Processing pipeline (Section 4.1):
/// 1. Variable substitution
/// 2. Config merging (if overlay provided)
/// 3. YAML parsing
/// 4. Secret resolution
/// 5. Schema validation (via serde)
/// 6. Semantic validation (V-001..V-008)
/// 7. Rewrite to domain model
pub fn parse(
    yaml: &str,
    variables: &std::collections::BTreeMap<String, String>,
) -> Result<Pipeline, TeckelError> {
    // 1. Variable substitution
    let resolved = resolve::variables::substitute(yaml, variables)?;

    // 2. YAML parsing
    let document: yaml::Document =
        serde_yaml::from_str(&resolved).map_err(|e| TeckelError::Yaml(e.to_string()))?;

    // 3. Validate version
    validate::check_version(&document)?;

    // 4. Rewrite to full pipeline (includes context + all top-level sections)
    let pipeline = rewrite::to_pipeline(&document)?;

    // 5. Semantic validation
    validate::validate(&pipeline.context, &document)?;

    Ok(pipeline)
}

/// Parse a raw YAML string into just the asset context (without pipeline metadata).
/// Use `parse()` instead for full pipeline access.
pub fn parse_context(
    yaml: &str,
    variables: &std::collections::BTreeMap<String, String>,
) -> Result<Context, TeckelError> {
    let pipeline = parse(yaml, variables)?;
    Ok(pipeline.context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn parse_simple_pipeline() {
        let yaml = r#"
version: "2.0"
input:
  - name: employees
    format: csv
    path: "data/employees.csv"
    options:
      header: true

transformation:
  - name: filtered
    where:
      from: employees
      filter: "salary > 50000"

  - name: projected
    select:
      from: filtered
      columns:
        - id
        - name
        - salary

output:
  - name: projected
    format: parquet
    path: "output/employees"
    mode: overwrite
"#;
        let pipeline = parse(yaml, &BTreeMap::new()).unwrap();

        assert!(pipeline.context.contains_key("employees"));
        assert!(pipeline.context.contains_key("filtered"));
        assert!(pipeline.context.contains_key("projected"));
        assert!(pipeline.context.contains_key("output_projected"));
        assert_eq!(pipeline.context.len(), 4);
    }

    #[test]
    fn parse_rejects_invalid_version() {
        let yaml = r#"
version: "1.0"
input:
  - name: x
    format: csv
    path: "x.csv"
output:
  - name: x
    format: csv
    path: "out.csv"
"#;
        let err = parse(yaml, &BTreeMap::new()).unwrap_err();
        assert!(err.to_string().contains("E-VERSION-001"));
    }

    #[test]
    fn parse_with_variable_substitution() {
        let yaml = r#"
version: "2.0"
input:
  - name: data
    format: csv
    path: "${DATA_PATH}/input.csv"
output:
  - name: data
    format: csv
    path: "out.csv"
"#;
        let vars = BTreeMap::from([("DATA_PATH".to_string(), "/tmp/test".to_string())]);
        let pipeline = parse(yaml, &vars).unwrap();
        let asset = pipeline.context.get("data").unwrap();
        match &asset.source {
            teckel_model::Source::Input(input) => {
                assert_eq!(input.path, "/tmp/test/input.csv");
            }
            _ => panic!("expected Input source"),
        }
    }

    #[test]
    fn parse_join_pipeline() {
        let yaml = r#"
version: "2.0"
input:
  - name: orders
    format: csv
    path: "orders.csv"
  - name: customers
    format: csv
    path: "customers.csv"

transformation:
  - name: enriched
    join:
      left: orders
      right:
        - name: customers
          type: inner
          on:
            - "orders.customer_id = customers.id"

output:
  - name: enriched
    format: parquet
    path: "output/enriched"
    mode: overwrite
"#;
        let pipeline = parse(yaml, &BTreeMap::new()).unwrap();
        assert!(pipeline.context.contains_key("enriched"));
        let asset = pipeline.context.get("enriched").unwrap();
        assert_eq!(asset.source.dependencies().len(), 2);
    }

    #[test]
    fn parse_all_core_transformations() {
        let yaml = r#"
version: "2.0"
input:
  - name: a
    format: csv
    path: "a.csv"
  - name: b
    format: csv
    path: "b.csv"

transformation:
  - name: t_select
    select:
      from: a
      columns: [id, name]

  - name: t_where
    where:
      from: a
      filter: "x > 0"

  - name: t_group
    group:
      from: a
      by: [region]
      agg: ["sum(amount) as total"]

  - name: t_order
    orderBy:
      from: a
      columns:
        - column: id
          direction: desc

  - name: t_join
    join:
      left: a
      right:
        - name: b
          type: left
          on: ["a.id = b.id"]

  - name: t_union
    union:
      sources: [a, b]

  - name: t_intersect
    intersect:
      sources: [a, b]

  - name: t_except
    except:
      left: a
      right: b

  - name: t_distinct
    distinct:
      from: a

  - name: t_limit
    limit:
      from: a
      count: 10

output:
  - name: t_select
    format: csv
    path: "out.csv"
"#;
        let pipeline = parse(yaml, &BTreeMap::new()).unwrap();
        // 2 inputs + 10 transforms + 1 output = 13
        assert_eq!(pipeline.context.len(), 13);
    }
}
