use crate::pipeline::Owner;
use crate::source::Source;
use crate::types::AssetRef;
use std::collections::BTreeMap;

/// Metadata attached to any asset (§18).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AssetMetadata {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub remove_tags: Vec<String>,
    pub meta: BTreeMap<String, serde_yaml::Value>,
    pub owner: Option<Owner>,
    pub columns: Vec<ColumnMetadata>,
    /// Output-only: expected update frequency as ISO 8601 duration (§18.5).
    pub freshness: Option<String>,
    /// Output-only: dataset lifecycle stage (§18.6).
    pub maturity: Option<String>,
}

/// Column-level metadata declarations (§18.4).
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnMetadata {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub constraints: Vec<String>,
    pub meta: BTreeMap<String, serde_yaml::Value>,
    pub glossary_term: Option<String>,
}

/// A named node in the pipeline DAG.
/// Every input, transformation, and output produces an asset.
#[derive(Debug, Clone, PartialEq)]
pub struct Asset {
    pub asset_ref: AssetRef,
    pub source: Source,
    pub metadata: AssetMetadata,
}

/// A collection of assets keyed by their unique reference.
pub type Context = BTreeMap<AssetRef, Asset>;
