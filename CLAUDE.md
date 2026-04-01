# CLAUDE.md

## Project Overview

**teckel-rs** is the Rust parser and model for the [Teckel Specification v3.0](https://github.com/eff3ct0/teckel-spec). It parses YAML pipeline definitions into a typed domain model. The execution backend lives in a separate repository.

## Build & Test

```bash
cargo build            # Build all crates
cargo test             # Run all tests
cargo clippy           # Lint
cargo fmt --check      # Check formatting
```

## Architecture

```
teckel-model   →   teckel-parser
(domain types)     (YAML deser + validation)
```

### teckel-model

Pure domain types with no heavy dependencies (only `serde`, `serde_yaml`, `thiserror`).

- `source.rs`: `Source` enum with `Input`, `Output`, and all 45 transformation variants
- `asset.rs`: `Asset` wraps an `AssetRef` + `Source`. `Context = BTreeMap<AssetRef, Asset>`
- `types.rs`: Newtypes (`AssetRef`, `Column`, `Expression`, `Condition`), enums (`Format`, `WriteMode`, `JoinType`, `SortColumn`, `TeckelDataType`)
- `error.rs`: `TeckelError` and `TeckelErrorCode` covering all ~30 spec error codes
- `pipeline.rs`: Pipeline metadata, hooks, config, templates, exposures, streaming I/O
- `quality.rs`: Data quality suites with 10 check types

### teckel-parser

Depends on `teckel-model`. Handles the full parsing pipeline:

1. `resolve/variables.rs` — `${VAR:default}` substitution
2. `resolve/secrets.rs` — `{{secrets.alias}}` resolution via `SecretsProvider` trait
3. `resolve/config_merger.rs` — Deep merge of YAML documents
4. `yaml/` — Serde `#[derive(Deserialize)]` structs matching the YAML shape
5. `rewrite.rs` — Converts serde model to domain model (yaml → `Context<Asset>`)
6. `validate.rs` — V-001..V-008 semantic validation (references, cycles, uniqueness, etc.)

Entry point: `teckel_parser::parse(yaml, variables) -> Result<Context, TeckelError>`

## Spec Reference

- [Teckel Spec v3.0](https://github.com/eff3ct0/teckel-spec/blob/main/spec/v3.0/teckel-spec.md)
- [JSON Schema v3.0](https://github.com/eff3ct0/teckel-spec/blob/main/spec/v3.0/teckel-schema.json)
