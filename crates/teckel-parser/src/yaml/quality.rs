use serde::Deserialize;
use std::collections::BTreeMap;

/// YAML representation of a data quality suite (Section 17).
#[derive(Debug, Deserialize)]
pub struct QualitySuiteDef {
    pub suite: String,
    pub description: Option<String>,
    pub target: String,
    pub filter: Option<String>,
    #[serde(default = "default_error")]
    pub severity: String,
    pub checks: Vec<QualityCheckDef>,
}

/// A single quality check within a suite.
/// Fields are a union — which ones are relevant depends on `type`.
#[derive(Debug, Deserialize)]
pub struct QualityCheckDef {
    #[serde(rename = "type")]
    pub check_type: String,
    pub column: Option<String>,
    pub columns: Option<QualityCheckColumns>,
    pub threshold: Option<f64>,
    pub severity: Option<String>,
    pub escalate: Option<EscalateDef>,
    pub description: Option<String>,
    // Validity
    #[serde(rename = "acceptedValues")]
    pub accepted_values: Option<Vec<String>>,
    pub range: Option<RangeSpecDef>,
    pub pattern: Option<String>,
    pub format: Option<String>,
    #[serde(rename = "lengthBetween")]
    pub length_between: Option<(u64, u64)>,
    // Statistical
    pub mean: Option<BoundSpecDef>,
    pub min: Option<BoundSpecDef>,
    pub max: Option<BoundSpecDef>,
    pub sum: Option<BoundSpecDef>,
    pub stdev: Option<BoundSpecDef>,
    pub quantiles: Option<BTreeMap<String, BoundSpecDef>>,
    // Volume
    #[serde(rename = "rowCount")]
    pub row_count: Option<BoundSpecDef>,
    #[serde(rename = "columnCount")]
    pub column_count: Option<BoundSpecDef>,
    // Freshness
    #[serde(rename = "maxAge")]
    pub max_age: Option<String>,
    // Referential
    pub reference: Option<ReferenceDef>,
    // Cross-column / Custom
    pub condition: Option<String>,
    // Schema
    pub types: Option<BTreeMap<String, String>>,
}

/// The `columns` field in quality checks can be either a list (uniqueness)
/// or an object with required/forbidden (schema checks).
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum QualityCheckColumns {
    List(Vec<String>),
    SchemaColumns {
        required: Option<Vec<String>>,
        forbidden: Option<Vec<String>>,
    },
}

#[derive(Debug, Deserialize)]
pub struct BoundSpecDef {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub between: Option<(f64, f64)>,
}

#[derive(Debug, Deserialize)]
pub struct RangeSpecDef {
    pub min: Option<f64>,
    pub max: Option<f64>,
    #[serde(rename = "strictMin", default)]
    pub strict_min: bool,
    #[serde(rename = "strictMax", default)]
    pub strict_max: bool,
}

#[derive(Debug, Deserialize)]
pub struct EscalateDef {
    pub threshold: f64,
    pub severity: String,
}

#[derive(Debug, Deserialize)]
pub struct ReferenceDef {
    pub asset: String,
    pub column: String,
}

fn default_error() -> String {
    "error".to_string()
}
