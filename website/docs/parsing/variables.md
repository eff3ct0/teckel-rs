---
sidebar_position: 1
title: Variable Substitution
---

# Variable Substitution

Variable substitution is the first step in the parsing pipeline. It resolves `${VAR:default}` placeholders in the raw YAML text before any YAML parsing occurs.

**Module:** `crates/teckel-parser/src/resolve/variables.rs`

## Syntax

```
${VARIABLE_NAME}
${VARIABLE_NAME:default_value}
```

- `VARIABLE_NAME` -- the name of the variable to resolve
- `default_value` -- optional fallback used when the variable is not found

To include a literal `$` in the output, escape it with `$$`:

```yaml
filter: "$${NOT_A_VAR}"   # produces: ${NOT_A_VAR}
```

## Resolution order

For each `${VAR:default}` placeholder, the resolver checks sources in this order:

1. **Explicit variables map** -- the `BTreeMap<String, String>` passed to `parse()`
2. **Environment variables** -- via `std::env::var()`
3. **Default value** -- the text after `:` in the placeholder
4. **Error** -- if no value is found, produces `E-VAR-001`

## Examples

### Basic substitution

```rust
use std::collections::BTreeMap;
use teckel_parser::parse;

let yaml = r#"
version: "3.0"
input:
  - name: data
    format: csv
    path: "${DATA_PATH}/input.csv"
output:
  - name: data
    format: csv
    path: "out.csv"
"#;

let vars = BTreeMap::from([
    ("DATA_PATH".to_string(), "/tmp/test".to_string()),
]);
let pipeline = parse(yaml, &vars)?;
// Input path is resolved to "/tmp/test/input.csv"
```

### Using default values

```yaml
transformation:
  - name: filtered
    where:
      from: data
      filter: "${FILTER_CONDITION:salary > 50000}"
```

If `FILTER_CONDITION` is not in the variables map or environment, the default `salary > 50000` is used.

### Environment variable fallback

If a variable is not in the explicit map, the resolver checks the process environment:

```bash
export DATA_PATH=/mnt/warehouse
```

```yaml
input:
  - name: data
    format: parquet
    path: "${DATA_PATH}/customers"
```

The input path resolves to `/mnt/warehouse/customers` even without passing `DATA_PATH` in the variables map.

## Error handling

When a variable has no match in the explicit map, no environment variable, and no default value, the parser returns:

```
[E-VAR-001] unresolved variable "MISSING_VAR" with no default
```

## Implementation details

The resolver uses a regex pattern `\$\{([^}:]+)(?::([^}]*))?\}` to match placeholders. It performs a single pass over the entire YAML string, replacing all matches simultaneously. The escaped dollar processing (`$$` to `$`) is done separately to avoid interference with variable patterns.
