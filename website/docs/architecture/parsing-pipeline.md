---
sidebar_position: 3
title: Parsing Pipeline
---

# Parsing Pipeline

The `teckel_parser::parse()` function runs an 8-step pipeline that transforms a raw YAML string into a fully validated `Pipeline`. Each step is implemented in a dedicated module.

![Parsing pipeline](/img/diagrams/parsing-pipeline.svg)

## Pipeline steps

### 1. Variable substitution

**Module:** `crates/teckel-parser/src/resolve/variables.rs`

Resolves `${VAR:default}` placeholders in the raw YAML text before any parsing occurs. Resolution order:

1. Explicit variables map (passed by the caller)
2. Environment variables
3. Default value (after `:` in the placeholder)
4. Error `E-VAR-001`

Escaped dollars (`$$`) are preserved as literal `$`.

### 2. Secret resolution

**Module:** `crates/teckel-parser/src/resolve/secrets.rs`

Resolves `{{secrets.alias}}` placeholders using the `SecretsProvider` trait. The default implementation (`EnvSecretsProvider`) maps alias `foo_bar` to environment variable `TECKEL_SECRET__FOO_BAR`.

### 3. YAML parsing

**Module:** `crates/teckel-parser/src/yaml/`

Deserializes the resolved YAML string into serde structs that mirror the YAML document shape. These are intermediate types -- not the final domain model.

```rust
let document: yaml::Document = serde_yaml::from_str(&resolved)
    .map_err(|e| TeckelError::Yaml(e.to_string()))?;
```

### 4. Version validation

**Module:** `crates/teckel-parser/src/validate.rs` (`check_version`)

Ensures the `version` field is `"2.0"` or `"3.0"`. Any other value produces error `E-VERSION-001`.

### 5. Rewrite to domain model

**Module:** `crates/teckel-parser/src/rewrite.rs`

Converts the intermediate serde model into the domain model. This step:

- Maps each YAML input to a `Source::Input`
- Maps each YAML transformation to its corresponding `Source` variant
- Maps each YAML output to a `Source::Output`
- Wraps everything in `Asset` structs and builds the `Context` (a `BTreeMap<AssetRef, Asset>`)
- Constructs the full `Pipeline` with metadata, config, hooks, quality suites, etc.

### 6. Semantic validation

**Module:** `crates/teckel-parser/src/validate.rs` (`validate`)

Runs V-001 through V-008 checks on the domain model:

| Rule | Check | Error code |
|---|---|---|
| V-001 | Reference integrity -- all `from`/`left`/`right`/`sources` point to existing assets | `E-REF-001` |
| V-002 | No cycles -- the dependency graph is a DAG (uses petgraph topological sort) | `E-CYCLE-001` |
| V-003 | Output references -- outputs must reference inputs or transformations, not other outputs | `E-REF-001` |
| V-005 | Non-empty lists -- `input` and `output` must have at least one element | `E-LIST-001` |
| V-007 | AssetRef format -- all names match `^[a-zA-Z][a-zA-Z0-9_-]{0,127}$` | `E-NAME-002` |
| V-008 | Version field -- must be `"2.0"` or `"3.0"` | `E-VERSION-001` |

If multiple errors are found, they are collected and returned as a `TeckelError::Validation` with all individual errors.

### 7. Expression validation (optional)

**Module:** `crates/teckel-parser/src/validate.rs` (`validate_expressions`)

When `ParseOptions::validate_expressions` is `true`, all expression and condition strings in the context are parsed using the expression parser in `crates/teckel-parser/src/expr/`. Invalid expressions produce error `E-EXPR-001`.

### 8. Tag propagation

**Module:** `crates/teckel-parser/src/dag.rs`

Inherits tags through the pipeline DAG so that downstream assets receive tags from their upstream dependencies.

## Entry points

```rust
/// Parse with default options (env secrets, no expression validation).
pub fn parse(
    yaml: &str,
    variables: &BTreeMap<String, String>,
) -> Result<Pipeline, TeckelError>;

/// Parse with full control over options.
pub fn parse_with_options(
    yaml: &str,
    options: &ParseOptions<'_>,
) -> Result<Pipeline, TeckelError>;

/// Parse into just the asset context (without pipeline metadata).
pub fn parse_context(
    yaml: &str,
    variables: &BTreeMap<String, String>,
) -> Result<Context, TeckelError>;
```

## Data flow diagram

```
   Raw YAML string
        │
        ▼
  ┌─────────────┐
  │  Variables   │  ${VAR:default} → resolved text
  └──────┬──────┘
         ▼
  ┌─────────────┐
  │   Secrets    │  {{secrets.alias}} → resolved text
  └──────┬──────┘
         ▼
  ┌─────────────┐
  │  serde_yaml  │  YAML text → yaml::Document
  └──────┬──────┘
         ▼
  ┌─────────────┐
  │   Version    │  check "2.0" or "3.0"
  └──────┬──────┘
         ▼
  ┌─────────────┐
  │   Rewrite    │  yaml::Document → Pipeline
  └──────┬──────┘
         ▼
  ┌─────────────┐
  │  Validation  │  V-001..V-008
  └──────┬──────┘
         ▼
  ┌─────────────┐
  │ Expressions  │  (optional) parse all expr strings
  └──────┬──────┘
         ▼
  ┌─────────────┐
  │    Tags      │  propagate through DAG
  └──────┬──────┘
         ▼
    Pipeline ✓
```
