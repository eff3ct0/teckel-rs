---
sidebar_position: 2
title: teckel-parser
---

# teckel-parser

The `teckel-parser` crate is the entry point for parsing Teckel YAML documents. It depends on `teckel-model` and provides the full parsing pipeline from raw YAML to a validated `Pipeline`.

**Crate path:** `crates/teckel-parser/`

## Public API

### parse

```rust
pub fn parse(
    yaml: &str,
    variables: &BTreeMap<String, String>,
) -> Result<Pipeline, TeckelError>
```

Parse a raw YAML string into a fully validated `Pipeline`. Uses the default `EnvSecretsProvider` for secret resolution and does not validate expressions.

**Parameters:**

| Parameter | Type | Description |
|---|---|---|
| `yaml` | `&str` | Raw YAML document string |
| `variables` | `&BTreeMap<String, String>` | Variable substitution map for `${VAR:default}` placeholders |

**Returns:** `Result<Pipeline, TeckelError>` -- the fully validated pipeline or an error.

**Example:**

```rust
use std::collections::BTreeMap;
use teckel_parser::parse;

let yaml = r#"
version: "3.0"
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

let pipeline = parse(yaml, &BTreeMap::new())?;
assert_eq!(pipeline.context.len(), 4); // 2 inputs + 1 transform + 1 output
```

### parse_with_options

```rust
pub fn parse_with_options(
    yaml: &str,
    options: &ParseOptions<'_>,
) -> Result<Pipeline, TeckelError>
```

Parse with full control over parsing behavior.

### parse_context

```rust
pub fn parse_context(
    yaml: &str,
    variables: &BTreeMap<String, String>,
) -> Result<Context, TeckelError>
```

Parse into just the asset context (`BTreeMap<AssetRef, Asset>`) without pipeline metadata. Useful when you only need the DAG structure.

## ParseOptions

```rust
pub struct ParseOptions<'a> {
    pub variables: &'a BTreeMap<String, String>,
    pub secrets_provider: &'a dyn SecretsProvider,
    pub validate_expressions: bool,
}
```

| Field | Type | Default | Description |
|---|---|---|---|
| `variables` | `&BTreeMap<String, String>` | -- | Variable substitution map |
| `secrets_provider` | `&dyn SecretsProvider` | `EnvSecretsProvider` | Secret resolution backend |
| `validate_expressions` | `bool` | `false` | Whether to parse and validate all expression strings |

**Constructor:**

```rust
impl<'a> ParseOptions<'a> {
    pub fn with_variables(
        variables: &'a BTreeMap<String, String>,
    ) -> Self;
}
```

Creates options with the given variables, default `EnvSecretsProvider`, and expression validation disabled.

## Modules

### resolve::variables

```rust
pub fn substitute(
    input: &str,
    variables: &BTreeMap<String, String>,
) -> Result<String, TeckelError>
```

Single-pass `${VAR:default}` substitution on raw text.

### resolve::secrets

```rust
pub trait SecretsProvider: Send + Sync {
    fn resolve(&self, alias: &str) -> Option<String>;
}

pub struct EnvSecretsProvider;

pub fn resolve_secrets(
    input: &str,
    provider: &dyn SecretsProvider,
) -> Result<String, TeckelError>
```

### resolve::config_merger

```rust
pub fn deep_merge(base: Value, overlay: Value) -> Value;

pub fn merge_documents(documents: Vec<Value>) -> Option<Value>;
```

### validate

```rust
pub fn check_version(doc: &yaml::Document) -> Result<(), TeckelError>;

pub fn validate(
    context: &Context,
    doc: &yaml::Document,
) -> Result<(), TeckelError>;

pub fn validate_expressions(context: &Context) -> Result<(), TeckelError>;
```

## Processing pipeline

The `parse` and `parse_with_options` functions execute these steps in order:

1. Variable substitution (`resolve::variables::substitute`)
2. Secret resolution (`resolve::secrets::resolve_secrets`)
3. YAML parsing (`serde_yaml::from_str`)
4. Version validation (`validate::check_version`)
5. Rewrite to domain model (`rewrite::to_pipeline`)
6. Semantic validation (`validate::validate`) -- V-001 through V-008
7. Expression validation (`validate::validate_expressions`) -- optional
8. Tag propagation (`dag::propagate_tags`)
