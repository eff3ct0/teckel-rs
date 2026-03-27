use serde::Deserialize;
use std::collections::BTreeMap;
use teckel_model::types::Primitive;

/// YAML representation of an input asset (Section 6).
#[derive(Debug, Deserialize)]
pub struct InputDef {
    pub name: String,
    pub format: String,
    pub path: String,
    #[serde(default)]
    pub options: BTreeMap<String, Primitive>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub meta: Option<BTreeMap<String, serde_yaml::Value>>,
    pub owner: Option<super::document::OwnerDef>,
    pub columns: Option<Vec<ColumnMetadataDef>>,
}

#[derive(Debug, Deserialize)]
pub struct ColumnMetadataDef {
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub constraints: Option<Vec<String>>,
    pub meta: Option<BTreeMap<String, serde_yaml::Value>>,
}
