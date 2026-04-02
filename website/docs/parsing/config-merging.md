---
sidebar_position: 3
title: Config Merging
---

# Config Merging

Config merging allows composing Teckel documents by deep-merging multiple YAML values. This is useful for layering environment-specific overrides on top of a base configuration.

**Module:** `crates/teckel-parser/src/resolve/config_merger.rs`

## Merge semantics

The merge follows these rules:

| Value type | Behavior |
|---|---|
| **Objects (mappings)** | Deep merge -- overlay fields override base fields recursively |
| **Arrays** | Replaced entirely by the overlay |
| **Scalars** | Replaced by the overlay |

## API

```rust
/// Deep-merge two YAML values.
pub fn deep_merge(base: Value, overlay: Value) -> Value;

/// Merge multiple YAML documents left to right.
/// The rightmost document has the highest precedence.
pub fn merge_documents(documents: Vec<Value>) -> Option<Value>;
```

## Example: Object deep merge

Given a base:

```yaml
a: 1
b:
  c: 2
  d: 3
```

And an overlay:

```yaml
b:
  c: 99
  e: 4
```

The result is:

```yaml
a: 1
b:
  c: 99
  d: 3
  e: 4
```

- `a` is preserved from the base (not present in overlay)
- `b.c` is overridden to `99`
- `b.d` is preserved from the base
- `b.e` is added from the overlay

## Example: Array replacement

Arrays are not merged element by element -- the overlay replaces the entire array:

```yaml
# Base
items:
  - a
  - b

# Overlay
items:
  - x

# Result
items:
  - x
```

## Multi-document merge

`merge_documents` accepts a vector of YAML values and merges them left to right. The rightmost document has the highest precedence:

```rust
use serde_yaml::Value;

let base: Value = serde_yaml::from_str("a: 1\nb: 2")?;
let env: Value = serde_yaml::from_str("b: 99\nc: 3")?;
let local: Value = serde_yaml::from_str("c: 100")?;

let merged = merge_documents(vec![base, env, local]);
// Result: { a: 1, b: 99, c: 100 }
```

## Use cases

- **Environment layering:** Base config + staging overrides + production overrides
- **Shared defaults:** Common input/output settings shared across pipelines
- **Template composition:** Merging template parameters with per-pipeline values
