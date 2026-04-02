---
sidebar_position: 1
slug: /intro
title: Teckel Parser (teckel-rs)
---

# Teckel Parser (teckel-rs)

**teckel-rs** is the Rust implementation of the [Teckel Specification v3.0](https://teckel.rafaelfernandez.dev/docs/intro) parser. It takes YAML pipeline definitions and produces a fully validated, typed domain model ready for execution by a downstream engine.

The project is split into two crates:

| Crate | Purpose |
|---|---|
| **teckel-model** | Pure domain types — `Source`, `Asset`, `Pipeline`, `TeckelDataType`, error codes. Minimal dependencies (`serde`, `thiserror`). |
| **teckel-parser** | YAML deserialization, variable substitution, secret resolution, config merging, rewrite to domain model, and semantic validation (V-001 through V-008). |

## Quick start

Add the parser crate to your project:

```bash
cargo add teckel-parser
```

Parse a Teckel YAML document:

```rust
use std::collections::BTreeMap;
use teckel_parser::parse;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let yaml = r#"
version: "3.0"
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

output:
  - name: filtered
    format: parquet
    path: "output/employees"
    mode: overwrite
"#;

    let variables = BTreeMap::new();
    let pipeline = parse(yaml, &variables)?;

    println!("Parsed {} assets", pipeline.context.len());
    for (name, asset) in &pipeline.context {
        println!("  {} -> {:?} deps", name, asset.source.dependencies().len());
    }

    Ok(())
}
```

## What the parser does

The `parse()` function runs an 8-step pipeline:

1. **Variable substitution** -- resolves `${VAR:default}` placeholders
2. **Secret resolution** -- resolves `{{secrets.alias}}` via the `SecretsProvider` trait
3. **YAML parsing** -- deserializes into serde structs
4. **Version validation** -- ensures version is `"2.0"` or `"3.0"`
5. **Rewrite** -- converts the YAML model into the domain model (`Pipeline`)
6. **Semantic validation** -- runs V-001 through V-008 checks
7. **Expression validation** -- optionally parses all expression strings
8. **Tag propagation** -- inherits tags through the DAG

The result is a `Pipeline` containing a `Context` (a `BTreeMap<AssetRef, Asset>`) with every input, transformation, and output as a typed node in the pipeline DAG.

## Specification

This parser implements the [Teckel Specification v3.0](https://teckel.rafaelfernandez.dev/docs/intro). The spec defines the YAML schema, all 45 transformation types, the expression language, data quality suites, metadata, and error codes.

## Source code

The full source is available on [GitHub](https://github.com/eff3ct0/teckel-rs).
