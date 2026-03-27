use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Unique identifier for an asset within a Teckel document.
/// Must match: `^[a-zA-Z][a-zA-Z0-9_-]{0,127}$`
pub type AssetRef = String;

/// A column name, optionally qualified as `asset.column`.
pub type Column = String;

/// A boolean expression string.
pub type Condition = String;

/// A Teckel expression string (SQL-like).
pub type Expression = String;

/// Format-specific key-value options.
pub type Options = BTreeMap<String, Primitive>;

/// Primitive value types supported in options.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Primitive {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

/// Data format for inputs and outputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Csv,
    Json,
    Parquet,
    Delta,
    Orc,
    Avro,
    Jdbc,
    #[serde(untagged)]
    Custom(String),
}

/// Write mode for outputs.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WriteMode {
    #[default]
    Error,
    Overwrite,
    Append,
    Ignore,
}

/// Sort direction.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

/// Null placement in sorting.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NullOrdering {
    First,
    #[default]
    Last,
}

/// Join type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Outer,
    Cross,
    LeftSemi,
    LeftAnti,
}

/// Window frame type.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FrameType {
    Rows,
    #[default]
    Range,
}

/// Schema enforcement mode.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SchemaEnforceMode {
    #[default]
    Strict,
    Evolve,
}

/// Assertion failure handling mode.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OnFailure {
    #[default]
    Fail,
    Warn,
    Drop,
}

/// Enrich HTTP error handling mode.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OnError {
    #[default]
    Null,
    Fail,
    Skip,
}

/// Streaming output mode.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    #[default]
    Append,
    Update,
    Complete,
}

/// Teckel data type system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TeckelDataType {
    String,
    #[serde(alias = "int")]
    Integer,
    Long,
    Float,
    Double,
    Boolean,
    Date,
    Timestamp,
    Binary,
    Decimal {
        precision: u8,
        scale: u8,
    },
    Array(Box<TeckelDataType>),
    Map(Box<TeckelDataType>, Box<TeckelDataType>),
    Struct(Vec<StructField>),
}

/// A field within a struct data type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub data_type: TeckelDataType,
    #[serde(default = "default_true")]
    pub nullable: bool,
}

fn default_true() -> bool {
    true
}

/// Sort column specification (simple string or explicit object).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SortColumn {
    Simple(Column),
    Explicit {
        column: Column,
        #[serde(default)]
        direction: SortDirection,
        #[serde(default)]
        nulls: NullOrdering,
    },
}

impl SortColumn {
    pub fn column_name(&self) -> &str {
        match self {
            SortColumn::Simple(c) => c,
            SortColumn::Explicit { column, .. } => column,
        }
    }
}
