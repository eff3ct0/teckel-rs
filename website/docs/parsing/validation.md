---
sidebar_position: 4
title: Validation
---

# Validation

The parser runs semantic validation after converting the YAML document to the domain model. Validation checks are named V-001 through V-008, following the Teckel Specification.

![Validation rules](/img/diagrams/validation-rules.svg)

**Module:** `crates/teckel-parser/src/validate.rs`

## Validation rules

### V-001: Reference integrity

Every asset reference used in `from`, `left`, `right`, `sources`, or `views` fields must point to a defined asset in the context.

```
[E-REF-001] asset "filtered" references undefined asset "emplyees" (did you mean "employees"?)
```

The validator includes a Levenshtein distance-based suggestion when a similar asset name exists (distance <= 3).

### V-002: No cycles

The asset dependency graph must be a directed acyclic graph (DAG). The validator builds a directed graph using petgraph and attempts a topological sort. If the sort fails, a cycle exists.

```
[E-CYCLE-001] circular dependency detected in the pipeline DAG
```

### V-003: Output reference validation

Output names must reference assets from the `input` or `transformation` sections. An output cannot reference another output.

```
[E-REF-001] output "result" references undefined asset
```

### V-005: Non-empty lists

Fields typed as `NonEmptyList` in the spec must contain at least one element. The parser checks that `input` and `output` arrays are non-empty.

```
[E-LIST-001] "input" must contain at least one element
[E-LIST-001] "output" must contain at least one element
```

### V-007: AssetRef format

All asset names must match the pattern `^[a-zA-Z][a-zA-Z0-9_-]{0,127}$`. Names must start with a letter and contain only alphanumeric characters, underscores, and hyphens.

```
[E-NAME-002] invalid AssetRef "123_bad" -- must start with a letter
```

### V-008: Version field

The `version` field must be present and set to `"2.0"` or `"3.0"`. This check runs before the full validation pass.

```
[E-VERSION-001] unsupported version "1.0", expected "2.0" or "3.0"
```

## Expression validation (optional)

When `ParseOptions::validate_expressions` is set to `true`, the parser additionally validates all expression and condition strings by parsing them with the built-in expression parser.

```rust
let options = ParseOptions {
    variables: &vars,
    secrets_provider: &EnvSecretsProvider,
    validate_expressions: true,  // Enable expression validation
};

let pipeline = parse_with_options(yaml, &options)?;
```

Invalid expressions produce error `E-EXPR-001`:

```
[E-EXPR-001] invalid expression in "computed": unexpected token ">>" (expression: "col1 >> 2")
```

## Error collection

The validator collects all errors rather than stopping at the first one. If multiple validation rules fail, the result is a `TeckelError::Validation` containing all individual errors:

```rust
pub enum TeckelError {
    // ...
    Validation {
        count: usize,
        errors: Vec<TeckelError>,
    },
}
```

This lets callers report all issues in a single pass rather than requiring iterative fix-and-retry cycles.

## Validation order

The checks run in this order:

1. V-008 -- Version (runs before the main validation pass)
2. V-007 -- AssetRef format
3. V-005 -- Non-empty lists
4. V-001 -- Reference integrity
5. V-002 -- No cycles
6. V-003 -- Output references
