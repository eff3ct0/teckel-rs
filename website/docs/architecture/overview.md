---
sidebar_position: 1
title: Overview
---

# Architecture Overview

teckel-rs follows a strict two-crate architecture that separates domain types from parsing logic.

## Crate dependency graph

```
teckel-model   -->   teckel-parser
(domain types)       (YAML deser + validation)
```

`teckel-parser` depends on `teckel-model`. Nothing depends on `teckel-parser` at the library level -- it is the top-level entry point for consumers.

## teckel-model

Pure domain types with minimal dependencies:

- **serde** / **serde_yaml** -- serialization support for data types
- **thiserror** -- ergonomic error definitions

Key modules:

| Module | Contents |
|---|---|
| `source.rs` | `Source` enum with `Input`, `Output`, and all 45 transformation variants |
| `asset.rs` | `Asset` struct (wraps `AssetRef` + `Source`), `Context` type alias |
| `types.rs` | Newtypes (`AssetRef`, `Column`, `Expression`, `Condition`), enums (`Format`, `WriteMode`, `JoinType`, `TeckelDataType`) |
| `error.rs` | `TeckelError` and `TeckelErrorCode` covering all spec error codes |
| `pipeline.rs` | `Pipeline`, `PipelineMetadata`, `Hooks`, `PipelineConfig`, `Template`, `Exposure`, streaming types |
| `quality.rs` | `QualitySuite` with 10 check types (schema, completeness, uniqueness, validity, statistical, volume, freshness, referential, cross-column, custom) |

## teckel-parser

Depends on `teckel-model` and adds heavier dependencies:

- **regex** -- variable and secret pattern matching
- **petgraph** -- DAG cycle detection
- **serde_yaml** -- YAML deserialization

Key modules:

| Module | Contents |
|---|---|
| `resolve/variables.rs` | `${VAR:default}` substitution |
| `resolve/secrets.rs` | `{{secrets.alias}}` resolution via `SecretsProvider` trait |
| `resolve/config_merger.rs` | Deep merge of multiple YAML documents |
| `yaml/` | Serde `#[derive(Deserialize)]` structs matching the YAML shape |
| `rewrite.rs` | Converts serde model to domain model |
| `validate.rs` | V-001 through V-008 semantic validation |
| `expr/` | Expression parser for optional expression validation |
| `dag.rs` | Tag propagation through the pipeline DAG |

## Design principles

1. **Minimal dependencies in the model crate.** Any code that needs to work with Teckel types (e.g., an execution engine) can depend on `teckel-model` alone without pulling in regex, petgraph, or the full YAML parser.

2. **Heavy lifting in the parser crate.** All I/O, string processing, and graph algorithms live in `teckel-parser`. This keeps the model crate fast to compile and easy to reason about.

3. **Validation is separate from parsing.** The YAML deserialization step produces an intermediate representation. The rewrite step converts it to domain types. Validation runs on the final domain model, ensuring that all checks operate on the canonical representation.

4. **Errors carry codes.** Every error includes a `TeckelErrorCode` from the spec (e.g., `E-REF-001`, `E-CYCLE-001`), making it straightforward to map parser errors back to spec requirements.
