use crate::types::*;
use std::collections::BTreeMap;

/// Data quality suite (Section 17).
#[derive(Debug, Clone, PartialEq)]
pub struct QualitySuite {
    pub suite: String,
    pub description: Option<String>,
    pub target: AssetRef,
    pub filter: Option<Condition>,
    pub severity: Severity,
    pub checks: Vec<Check>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Severity {
    #[default]
    Error,
    Warn,
    Info,
}

/// Individual quality check within a suite.
#[derive(Debug, Clone, PartialEq)]
pub enum Check {
    Schema(SchemaCheck),
    Completeness(CompletenessCheck),
    Uniqueness(UniquenessCheck),
    Validity(ValidityCheck),
    Statistical(StatisticalCheck),
    Volume(VolumeCheck),
    Freshness(FreshnessCheck),
    Referential(ReferentialCheck),
    CrossColumn(CrossColumnCheck),
    Custom(CustomCheck),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaCheck {
    pub required_columns: Vec<String>,
    pub forbidden_columns: Vec<String>,
    pub types: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompletenessCheck {
    pub column: Column,
    pub threshold: f64,
    pub severity: Option<Severity>,
    pub escalate: Option<EscalateRule>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalateRule {
    pub threshold: f64,
    pub severity: Severity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UniquenessCheck {
    pub columns: Vec<Column>,
    pub threshold: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidityCheck {
    pub column: Column,
    pub accepted_values: Option<Vec<String>>,
    pub range: Option<RangeSpec>,
    pub pattern: Option<String>,
    pub format: Option<String>,
    pub length_between: Option<(u64, u64)>,
    pub threshold: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RangeSpec {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub strict_min: bool,
    pub strict_max: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoundSpec {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub between: Option<(f64, f64)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatisticalCheck {
    pub column: Column,
    pub mean: Option<BoundSpec>,
    pub min: Option<BoundSpec>,
    pub max: Option<BoundSpec>,
    pub sum: Option<BoundSpec>,
    pub stdev: Option<BoundSpec>,
    pub quantiles: BTreeMap<String, BoundSpec>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VolumeCheck {
    pub row_count: Option<BoundSpec>,
    pub column_count: Option<BoundSpec>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreshnessCheck {
    pub column: Column,
    pub max_age: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReferentialCheck {
    pub column: Column,
    pub reference_asset: AssetRef,
    pub reference_column: Column,
    pub threshold: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CrossColumnCheck {
    pub condition: Condition,
    pub description: Option<String>,
    pub threshold: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomCheck {
    pub condition: Condition,
    pub description: Option<String>,
    pub threshold: f64,
}
