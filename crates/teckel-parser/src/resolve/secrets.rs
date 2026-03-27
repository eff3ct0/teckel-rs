use regex::Regex;
use std::sync::LazyLock;
use teckel_model::{TeckelError, TeckelErrorCode};

static SECRET_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{secrets\.([a-zA-Z][a-zA-Z0-9_-]*)\}\}").unwrap());

/// Trait for resolving secret values at runtime.
pub trait SecretsProvider: Send + Sync {
    fn resolve(&self, alias: &str) -> Option<String>;
}

/// Default secrets provider that reads from environment variables.
/// Maps alias `foo_bar` to `TECKEL_SECRET__FOO_BAR`.
pub struct EnvSecretsProvider;

impl SecretsProvider for EnvSecretsProvider {
    fn resolve(&self, alias: &str) -> Option<String> {
        let env_key = format!("TECKEL_SECRET__{}", alias.to_uppercase().replace('-', "_"));
        std::env::var(&env_key).ok()
    }
}

/// Resolve `{{secrets.alias}}` placeholders in a string (Section 13).
pub fn resolve_secrets(input: &str, provider: &dyn SecretsProvider) -> Result<String, TeckelError> {
    let mut errors = Vec::new();
    let result = SECRET_PATTERN.replace_all(input, |caps: &regex::Captures| {
        let alias = caps.get(1).unwrap().as_str();
        match provider.resolve(alias) {
            Some(val) => val,
            None => {
                errors.push(TeckelError::spec(
                    TeckelErrorCode::ESecret001,
                    format!("unresolved secret \"{alias}\""),
                ));
                format!("{{{{secrets.{alias}}}}}")
            }
        }
    });

    if !errors.is_empty() {
        return Err(errors.into_iter().next().unwrap());
    }

    Ok(result.into_owned())
}
