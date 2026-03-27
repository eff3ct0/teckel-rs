use serde::Deserialize;
use std::collections::BTreeMap;

use super::document::OwnerDef;

/// YAML representation of an exposure (Section 19).
#[derive(Debug, Deserialize)]
pub struct ExposureDef {
    pub name: String,
    #[serde(rename = "type")]
    pub exposure_type: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub maturity: Option<String>,
    pub owner: Option<OwnerDef>,
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub meta: BTreeMap<String, serde_yaml::Value>,
}
