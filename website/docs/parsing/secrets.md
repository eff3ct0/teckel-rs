---
sidebar_position: 2
title: Secret Resolution
---

# Secret Resolution

Secret resolution is the second step in the parsing pipeline. It resolves `{{secrets.alias}}` placeholders after variable substitution but before YAML parsing.

**Module:** `crates/teckel-parser/src/resolve/secrets.rs`

## Syntax

```
{{secrets.alias_name}}
```

The alias must match `[a-zA-Z][a-zA-Z0-9_-]*`.

## SecretsProvider trait

Secret resolution is abstracted through the `SecretsProvider` trait:

```rust
pub trait SecretsProvider: Send + Sync {
    fn resolve(&self, alias: &str) -> Option<String>;
}
```

Implement this trait to integrate with your secrets backend (e.g., HashiCorp Vault, AWS Secrets Manager, Azure Key Vault).

## Default: EnvSecretsProvider

The built-in `EnvSecretsProvider` maps secret aliases to environment variables using the convention:

```
alias "foo_bar" → TECKEL_SECRET__FOO_BAR
```

The mapping rules:
- Prefix with `TECKEL_SECRET__` (double underscore)
- Convert the alias to uppercase
- Replace hyphens with underscores

```rust
pub struct EnvSecretsProvider;

impl SecretsProvider for EnvSecretsProvider {
    fn resolve(&self, alias: &str) -> Option<String> {
        let env_key = format!(
            "TECKEL_SECRET__{}",
            alias.to_uppercase().replace('-', "_")
        );
        std::env::var(&env_key).ok()
    }
}
```

## Using a custom provider

Use `parse_with_options` to supply a custom secrets provider:

```rust
use teckel_parser::{parse_with_options, ParseOptions};
use teckel_parser::resolve::secrets::SecretsProvider;

struct VaultProvider { /* ... */ }

impl SecretsProvider for VaultProvider {
    fn resolve(&self, alias: &str) -> Option<String> {
        // Fetch from Vault, return None if not found
        todo!()
    }
}

let options = ParseOptions {
    variables: &variables,
    secrets_provider: &VaultProvider { /* ... */ },
    validate_expressions: false,
};

let pipeline = parse_with_options(yaml, &options)?;
```

## Example

```yaml
version: "3.0"
secrets:
  db_password:
    scope: production
    key: database/password

input:
  - name: customers
    format: jdbc
    path: "jdbc:postgresql://db.example.com/main"
    options:
      user: app_user
      password: "{{secrets.db_password}}"
```

With the default `EnvSecretsProvider`, set the environment variable:

```bash
export TECKEL_SECRET__DB_PASSWORD="s3cur3_p4ss"
```

## Error handling

When a secret alias cannot be resolved by the provider, the parser returns:

```
[E-SECRET-001] unresolved secret "db_password"
```
