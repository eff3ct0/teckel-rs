# teckel-rs

Rust implementation of the [Teckel Specification v2.0](https://github.com/eff3ct0/teckel-spec) -- a declarative YAML-based language for defining data transformation pipelines.

This crate provides the **parser and model layer**: YAML parsing, variable/secret resolution, validation, and conversion to a typed domain model. The execution backend is implemented in a separate repository.

## Crates

| Crate | Description |
|-------|-------------|
| `teckel-model` | Core domain types: assets, sources, 31 transformation types, error catalog, quality, metadata |
| `teckel-parser` | YAML deserialization, variable substitution, secret resolution, config merging, validation (V-001..V-008), rewrite to domain model |

## Quick Start

```rust
use teckel_parser::parse;
use std::collections::BTreeMap;

let yaml = r#"
version: "2.0"
input:
  - name: employees
    format: csv
    path: "data/employees.csv"
    options:
      header: true
transformation:
  - name: active
    where:
      from: employees
      filter: "status = 'active'"
output:
  - name: active
    format: parquet
    path: "output/active_employees"
    mode: overwrite
"#;

let context = parse(yaml, &BTreeMap::new()).unwrap();
// context is a BTreeMap<AssetRef, Asset> representing the pipeline DAG
```

## Conformance

This parser supports parsing all constructs defined in the Teckel v2.0 specification:

- **Core** (Sections 1-12): Document structure, inputs, outputs, 10 basic transformations, expression language (as raw strings), data types, null semantics, variable substitution, path resolution, validation rules, execution model, error catalog
- **Extended** (Sections 8.11-8.31, 13-21): All 31 transformations, secrets, configuration, hooks, data quality, metadata, exposures, templates, config merging

## Build

```bash
cargo build
cargo test
cargo clippy
```

## License

Apache License 2.0
