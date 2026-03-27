use serde::Deserialize;
use std::collections::BTreeMap;
use teckel_model::types::Primitive;

/// YAML representation of an output asset (Section 7).
#[derive(Debug, Deserialize)]
pub struct OutputDef {
    pub name: String,
    pub format: String,
    pub path: String,
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default)]
    pub options: BTreeMap<String, Primitive>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub meta: Option<BTreeMap<String, serde_yaml::Value>>,
    pub freshness: Option<String>,
    pub maturity: Option<String>,
}

fn default_mode() -> String {
    "error".to_string()
}
