use crate::asset::Context;
use crate::quality::QualitySuite;
use crate::types::*;
use std::collections::BTreeMap;

/// The complete parsed pipeline — everything from a Teckel v2.0 document.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Pipeline {
    /// All assets (inputs, transformations, outputs) keyed by name.
    pub context: Context,
    /// Pipeline-level metadata (§18).
    pub metadata: PipelineMetadata,
    /// Pipeline-wide configuration (§14).
    pub config: PipelineConfig,
    /// Lifecycle hooks (§16).
    pub hooks: Hooks,
    /// Data quality suites (§17).
    pub quality: Vec<QualitySuite>,
    /// Reusable templates (§20).
    pub templates: Vec<Template>,
    /// Downstream consumer declarations (§19).
    pub exposures: Vec<Exposure>,
    /// Streaming inputs (§15).
    pub streaming_inputs: Vec<StreamingInput>,
    /// Streaming outputs (§15).
    pub streaming_outputs: Vec<StreamingOutput>,
    /// Secret key declarations (§13).
    pub secrets: BTreeMap<String, SecretKey>,
}

/// A secret key declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct SecretKey {
    pub scope: Option<String>,
    pub key: String,
}

/// Pipeline-level metadata (Section 18.2).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PipelineMetadata {
    pub name: Option<String>,
    pub namespace: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub owner: Option<Owner>,
    pub tags: Vec<String>,
    pub meta: BTreeMap<String, serde_yaml::Value>,
    pub schedule: Option<String>,
    pub freshness: Option<String>,
    pub links: Vec<Link>,
    pub contacts: Vec<Contact>,
    pub catalog: Option<CatalogConfig>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Owner {
    pub name: String,
    pub email: String,
    pub owner_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Contact {
    pub name: String,
    pub email: String,
    pub role: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogConfig {
    pub target: String,
    pub namespace: Option<String>,
}

/// Lifecycle hooks (Section 16).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Hooks {
    pub pre_execution: Vec<Hook>,
    pub post_execution: Vec<Hook>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Hook {
    pub name: String,
    pub command: String,
}

/// Pipeline-wide configuration (Section 14).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PipelineConfig {
    pub backend: Option<String>,
    pub cache: Option<CacheConfig>,
    pub notifications: Option<NotificationConfig>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CacheConfig {
    pub auto_cache_threshold: Option<u32>,
    pub default_storage_level: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NotificationConfig {
    pub on_success: Vec<NotificationTarget>,
    pub on_failure: Vec<NotificationTarget>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NotificationTarget {
    pub channel: String,
    pub url: Option<String>,
    pub path: Option<String>,
}

/// Template definition (Section 20).
#[derive(Debug, Clone, PartialEq)]
pub struct Template {
    pub name: String,
    pub parameters: BTreeMap<String, Primitive>,
}

/// Exposure declaration (Section 19).
#[derive(Debug, Clone, PartialEq)]
pub struct Exposure {
    pub name: String,
    pub exposure_type: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub maturity: Option<String>,
    pub owner: Option<Owner>,
    pub depends_on: Vec<AssetRef>,
    pub tags: Vec<String>,
    pub meta: BTreeMap<String, serde_yaml::Value>,
}

/// Streaming input definition (Section 15.1).
#[derive(Debug, Clone, PartialEq)]
pub struct StreamingInput {
    pub name: AssetRef,
    pub format: String,
    pub path: Option<String>,
    pub options: Options,
    pub trigger: Option<String>,
}

/// Streaming output definition (Section 15.2).
#[derive(Debug, Clone, PartialEq)]
pub struct StreamingOutput {
    pub name: AssetRef,
    pub format: String,
    pub path: Option<String>,
    pub options: Options,
    pub output_mode: Option<OutputMode>,
    pub checkpoint_location: Option<String>,
    pub trigger: Option<String>,
}
