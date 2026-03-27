use super::exposure::ExposureDef;
use super::input::InputDef;
use super::output::OutputDef;
use super::quality::QualitySuiteDef;
use super::streaming::{StreamingInputDef, StreamingOutputDef};
use super::transformation::RawTransformation;
use serde::Deserialize;
use std::collections::BTreeMap;
use teckel_model::types::Primitive;

/// Top-level Teckel v2.0 document (Section 4).
#[derive(Debug, Deserialize)]
pub struct Document {
    /// Teckel specification version. MUST be "2.0".
    pub version: String,

    /// Pipeline-level metadata.
    pub pipeline: Option<PipelineMetadataDef>,

    /// Pipeline-wide configuration.
    pub config: Option<ConfigDef>,

    /// Secret key declarations.
    pub secrets: Option<SecretsDef>,

    /// Lifecycle hooks.
    pub hooks: Option<HooksDef>,

    /// Data quality suites.
    pub quality: Option<Vec<QualitySuiteDef>>,

    /// Reusable templates.
    pub templates: Option<Vec<TemplateDef>>,

    /// Data source definitions. REQUIRED, at least one.
    pub input: Vec<InputDef>,

    /// Streaming source definitions.
    #[serde(rename = "streamingInput")]
    pub streaming_input: Option<Vec<StreamingInputDef>>,

    /// Transformation definitions.
    pub transformation: Option<Vec<RawTransformation>>,

    /// Data destination definitions. REQUIRED, at least one.
    pub output: Vec<OutputDef>,

    /// Streaming sink definitions.
    #[serde(rename = "streamingOutput")]
    pub streaming_output: Option<Vec<StreamingOutputDef>>,

    /// Downstream consumer declarations.
    pub exposures: Option<Vec<ExposureDef>>,
}

#[derive(Debug, Deserialize)]
pub struct PipelineMetadataDef {
    pub name: Option<String>,
    pub namespace: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub owner: Option<OwnerDef>,
    pub tags: Option<Vec<String>>,
    pub meta: Option<BTreeMap<String, serde_yaml::Value>>,
    pub schedule: Option<String>,
    pub freshness: Option<String>,
    pub links: Option<Vec<LinkDef>>,
    pub contacts: Option<Vec<ContactDef>>,
    pub catalog: Option<CatalogDef>,
}

#[derive(Debug, Deserialize)]
pub struct LinkDef {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct ContactDef {
    pub name: String,
    pub email: String,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CatalogDef {
    pub target: String,
    pub namespace: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OwnerDef {
    pub name: String,
    pub email: String,
    #[serde(rename = "type")]
    pub owner_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigDef {
    pub backend: Option<String>,
    pub cache: Option<CacheDef>,
    pub notifications: Option<NotificationsDef>,
}

#[derive(Debug, Deserialize)]
pub struct CacheDef {
    #[serde(rename = "autoCacheThreshold")]
    pub auto_cache_threshold: Option<u32>,
    #[serde(rename = "defaultStorageLevel")]
    pub default_storage_level: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NotificationsDef {
    #[serde(rename = "onSuccess")]
    pub on_success: Option<Vec<NotificationTargetDef>>,
    #[serde(rename = "onFailure")]
    pub on_failure: Option<Vec<NotificationTargetDef>>,
}

#[derive(Debug, Deserialize)]
pub struct NotificationTargetDef {
    pub channel: String,
    pub url: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SecretsDef {
    pub keys: Option<BTreeMap<String, SecretKeyDef>>,
}

#[derive(Debug, Deserialize)]
pub struct SecretKeyDef {
    pub scope: Option<String>,
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub struct HooksDef {
    #[serde(rename = "preExecution")]
    pub pre_execution: Option<Vec<HookDef>>,
    #[serde(rename = "postExecution")]
    pub post_execution: Option<Vec<HookDef>>,
}

#[derive(Debug, Deserialize)]
pub struct HookDef {
    pub name: String,
    pub command: String,
}

#[derive(Debug, Deserialize)]
pub struct TemplateDef {
    pub name: String,
    pub parameters: BTreeMap<String, Primitive>,
}
