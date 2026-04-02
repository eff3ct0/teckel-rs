---
sidebar_position: 3
title: Error Catalog
---

# Error Catalog

All error codes defined in `TeckelErrorCode`. Each error follows the format `E-{CATEGORY}-{NUMBER}` and corresponds to a specific validation rule or failure condition from the Teckel Specification.

**Module:** `crates/teckel-model/src/error.rs`

## Error types

The `TeckelError` enum has five variants:

```rust
pub enum TeckelError {
    /// A spec-defined error with a code and message.
    Spec { code: TeckelErrorCode, message: String },

    /// YAML parsing failed.
    Yaml(String),

    /// I/O error.
    Io(std::io::Error),

    /// Multiple validation errors collected in a single pass.
    Validation { count: usize, errors: Vec<TeckelError> },

    /// Runtime execution error.
    Execution(String),
}
```

## Error codes

### Schema

| Code | Variant | Description |
|---|---|---|
| `E-REQ-001` | `EReq001` | Missing required field |

### Naming

| Code | Variant | Description |
|---|---|---|
| `E-NAME-001` | `EName001` | Duplicate AssetRef |
| `E-NAME-002` | `EName002` | Invalid AssetRef format (must match `^[a-zA-Z][a-zA-Z0-9_-]{0,127}$`) |
| `E-NAME-003` | `EName003` | Column name collision after rename |

### Reference

| Code | Variant | Description |
|---|---|---|
| `E-REF-001` | `ERef001` | Undefined asset reference |
| `E-REF-002` | `ERef002` | Invalid output reference (output references output) |
| `E-CYCLE-001` | `ECycle001` | Circular dependency detected in the pipeline DAG |

### Format

| Code | Variant | Description |
|---|---|---|
| `E-FMT-001` | `EFmt001` | Unknown data format |
| `E-MODE-001` | `EMode001` | Unknown write mode |

### Transformation

| Code | Variant | Description |
|---|---|---|
| `E-OP-001` | `EOp001` | Zero or multiple operation keys in transformation |
| `E-OP-002` | `EOp002` | Unknown operation key |

### Schema validation

| Code | Variant | Description |
|---|---|---|
| `E-LIST-001` | `EList001` | Empty list where NonEmptyList required |
| `E-ENUM-001` | `EEnum001` | Invalid enum value |

### Column

| Code | Variant | Description |
|---|---|---|
| `E-COL-001` | `ECol001` | Column not found in dataset |

### Join

| Code | Variant | Description |
|---|---|---|
| `E-JOIN-001` | `EJoin001` | Ambiguous column reference in join condition |

### Aggregation

| Code | Variant | Description |
|---|---|---|
| `E-AGG-001` | `EAgg001` | Non-aggregate expression in group-by output |

### Expression

| Code | Variant | Description |
|---|---|---|
| `E-EXPR-001` | `EExpr001` | Expression type mismatch or parse error |

### Schema compatibility

| Code | Variant | Description |
|---|---|---|
| `E-SCHEMA-001` | `ESchema001` | Incompatible schemas in set operation |
| `E-SCHEMA-002` | `ESchema002` | Operation would produce empty schema |
| `E-SCHEMA-003` | `ESchema003` | Unexpected extra columns in strict mode |
| `E-SCHEMA-004` | `ESchema004` | Missing expected columns in strict mode |

### Type

| Code | Variant | Description |
|---|---|---|
| `E-TYPE-001` | `EType001` | Incompatible types, cannot widen |

### I/O

| Code | Variant | Description |
|---|---|---|
| `E-IO-001` | `EIo001` | Input path not found or unreadable |
| `E-IO-002` | `EIo002` | Output destination already exists (error mode) |

### Substitution

| Code | Variant | Description |
|---|---|---|
| `E-VAR-001` | `EVar001` | Unresolved variable with no default |

### Secrets

| Code | Variant | Description |
|---|---|---|
| `E-SECRET-001` | `ESecret001` | Unresolved secret reference |

### Hooks

| Code | Variant | Description |
|---|---|---|
| `E-HOOK-001` | `EHook001` | Pre-execution hook failed |

### Custom components

| Code | Variant | Description |
|---|---|---|
| `E-COMP-001` | `EComp001` | Unregistered custom component |

### Quality

| Code | Variant | Description |
|---|---|---|
| `E-QUALITY-001` | `EQuality001` | Assertion or quality check failed |
| `E-QUALITY-002` | `EQuality002` | Unknown quality check type |
| `E-QUALITY-003` | `EQuality003` | Invalid threshold value |
| `E-QUALITY-004` | `EQuality004` | Freshness check failed |
| `E-QUALITY-005` | `EQuality005` | Referential integrity check failed |

### Metadata

| Code | Variant | Description |
|---|---|---|
| `E-META-001` | `EMeta001` | Invalid owner type |
| `E-META-002` | `EMeta002` | Invalid maturity value |
| `E-META-003` | `EMeta003` | Invalid freshness duration |
| `E-META-004` | `EMeta004` | Column metadata references non-existent column |

### Exposures

| Code | Variant | Description |
|---|---|---|
| `E-EXPOSE-001` | `EExpose001` | Exposure depends_on references undefined asset |
| `E-EXPOSE-002` | `EExpose002` | Unknown exposure type |

### Version

| Code | Variant | Description |
|---|---|---|
| `E-VERSION-001` | `EVersion001` | Missing or unsupported version field |

## Total: 34 error codes

## Usage

Create spec errors using the convenience constructor:

```rust
use teckel_model::{TeckelError, TeckelErrorCode};

let err = TeckelError::spec(
    TeckelErrorCode::ERef001,
    "asset \"filtered\" references undefined asset \"emplyees\"",
);

// Display format: [E-REF-001] asset "filtered" references undefined asset "emplyees"
println!("{err}");
```

## Handling validation errors

When multiple errors are found, they are wrapped in a `Validation` variant:

```rust
match result {
    Err(TeckelError::Validation { count, errors }) => {
        eprintln!("Found {count} validation error(s):");
        for err in &errors {
            eprintln!("  {err}");
        }
    }
    Err(err) => eprintln!("Error: {err}"),
    Ok(pipeline) => { /* success */ }
}
```
