use regex::Regex;
use std::collections::BTreeMap;
use std::sync::LazyLock;
use teckel_model::{TeckelError, TeckelErrorCode};

static VAR_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$\{([^}:]+)(?::([^}]*))?\}").unwrap());

static ESCAPED_DOLLAR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\$\$").unwrap());

/// Perform single-pass variable substitution on raw YAML text (Section 12).
///
/// Resolution order:
/// 1. Explicit variables map
/// 2. Environment variables
/// 3. Default value (after `:` in `${VAR:default}`)
/// 4. Error E-VAR-001
pub fn substitute(
    input: &str,
    variables: &BTreeMap<String, String>,
) -> Result<String, TeckelError> {
    // First pass: replace $$ with a placeholder
    let placeholder = "\x00DOLLAR\x00";
    let escaped = ESCAPED_DOLLAR.replace_all(input, placeholder);

    // Second pass: resolve variables
    let mut errors = Vec::new();
    let result = VAR_PATTERN.replace_all(&escaped, |caps: &regex::Captures| {
        let var_name = caps.get(1).unwrap().as_str();
        let default_value = caps.get(2).map(|m| m.as_str());

        // 1. Check explicit variables
        if let Some(val) = variables.get(var_name) {
            return val.clone();
        }

        // 2. Check environment variables
        if let Ok(val) = std::env::var(var_name) {
            return val;
        }

        // 3. Use default value
        if let Some(default) = default_value {
            return default.to_string();
        }

        // 4. Error
        errors.push(TeckelError::spec(
            TeckelErrorCode::EVar001,
            format!("unresolved variable \"{var_name}\" with no default"),
        ));
        format!("${{{var_name}}}")
    });

    if !errors.is_empty() {
        return Err(errors.into_iter().next().unwrap());
    }

    // Restore escaped dollars
    Ok(result.replace(placeholder, "$"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitutes_from_map() {
        let vars = BTreeMap::from([("FOO".to_string(), "bar".to_string())]);
        let result = substitute("path: ${FOO}/data", &vars).unwrap();
        assert_eq!(result, "path: bar/data");
    }

    #[test]
    fn uses_default_value() {
        let vars = BTreeMap::new();
        let result = substitute("filter: ${COND:1=1}", &vars).unwrap();
        assert_eq!(result, "filter: 1=1");
    }

    #[test]
    fn escapes_double_dollar() {
        let vars = BTreeMap::new();
        let result = substitute("text: $${NOT_A_VAR}", &vars).unwrap();
        assert_eq!(result, "text: ${NOT_A_VAR}");
    }

    #[test]
    fn errors_on_unresolved() {
        let vars = BTreeMap::new();
        let result = substitute("path: ${MISSING}", &vars);
        assert!(result.is_err());
    }
}
