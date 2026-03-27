use serde::Deserialize;
use std::collections::BTreeMap;
use teckel_model::types::Primitive;

/// YAML representation of a streaming input (Section 15.1).
#[derive(Debug, Deserialize)]
pub struct StreamingInputDef {
    pub name: String,
    pub format: String,
    pub path: Option<String>,
    #[serde(default)]
    pub options: BTreeMap<String, Primitive>,
    pub trigger: Option<String>,
}

/// YAML representation of a streaming output (Section 15.2).
#[derive(Debug, Deserialize)]
pub struct StreamingOutputDef {
    pub name: String,
    pub format: String,
    pub path: Option<String>,
    #[serde(default)]
    pub options: BTreeMap<String, Primitive>,
    #[serde(rename = "outputMode")]
    pub output_mode: Option<String>,
    #[serde(rename = "checkpointLocation")]
    pub checkpoint_location: Option<String>,
    pub trigger: Option<String>,
}
