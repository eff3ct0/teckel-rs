use crate::source::Source;
use crate::types::AssetRef;
use std::collections::BTreeMap;

/// A named node in the pipeline DAG.
/// Every input, transformation, and output produces an asset.
#[derive(Debug, Clone, PartialEq)]
pub struct Asset {
    pub asset_ref: AssetRef,
    pub source: Source,
}

/// A collection of assets keyed by their unique reference.
pub type Context = BTreeMap<AssetRef, Asset>;
