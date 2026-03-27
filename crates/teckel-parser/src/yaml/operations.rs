use serde::Deserialize;
use std::collections::BTreeMap;

// ── Core operations (8.1 - 8.10) ────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SelectOp {
    pub from: String,
    pub columns: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct WhereOp {
    pub from: String,
    pub filter: String,
}

#[derive(Debug, Deserialize)]
pub struct GroupOp {
    pub from: String,
    pub by: Vec<String>,
    pub agg: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct OrderByOp {
    pub from: String,
    pub columns: Vec<SortColumnDef>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum SortColumnDef {
    Simple(String),
    Explicit {
        column: String,
        #[serde(default = "default_asc")]
        direction: String,
        #[serde(default = "default_last")]
        nulls: String,
    },
}

fn default_asc() -> String {
    "asc".to_string()
}
fn default_last() -> String {
    "last".to_string()
}

#[derive(Debug, Deserialize)]
pub struct JoinOp {
    pub left: String,
    pub right: Vec<JoinTargetDef>,
}

#[derive(Debug, Deserialize)]
pub struct JoinTargetDef {
    pub name: String,
    #[serde(rename = "type")]
    pub join_type: String,
    pub on: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UnionOp {
    pub sources: Vec<String>,
    #[serde(default = "default_true")]
    pub all: bool,
}

#[derive(Debug, Deserialize)]
pub struct IntersectOp {
    pub sources: Vec<String>,
    #[serde(default)]
    pub all: bool,
}

#[derive(Debug, Deserialize)]
pub struct ExceptOp {
    pub left: String,
    pub right: String,
    #[serde(default)]
    pub all: bool,
}

#[derive(Debug, Deserialize)]
pub struct DistinctOp {
    pub from: String,
    pub columns: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct LimitOp {
    pub from: String,
    pub count: u64,
}

// ── Extended operations (8.11 - 8.31) ────────────────────────

#[derive(Debug, Deserialize)]
pub struct ColumnDefOp {
    pub name: String,
    pub expression: String,
}

#[derive(Debug, Deserialize)]
pub struct AddColumnsOp {
    pub from: String,
    pub columns: Vec<ColumnDefOp>,
}

#[derive(Debug, Deserialize)]
pub struct DropColumnsOp {
    pub from: String,
    pub columns: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RenameColumnsOp {
    pub from: String,
    pub mappings: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct CastDefOp {
    pub name: String,
    #[serde(rename = "targetType")]
    pub target_type: String,
}

#[derive(Debug, Deserialize)]
pub struct CastColumnsOp {
    pub from: String,
    pub columns: Vec<CastDefOp>,
}

#[derive(Debug, Deserialize)]
pub struct WindowFuncDefOp {
    pub expression: String,
    pub alias: String,
}

#[derive(Debug, Deserialize)]
pub struct FrameSpecOp {
    #[serde(rename = "type", default = "default_range")]
    pub frame_type: String,
    #[serde(default = "default_unbounded_preceding")]
    pub start: String,
    #[serde(default = "default_current_row")]
    pub end: String,
}

fn default_range() -> String {
    "range".to_string()
}
fn default_unbounded_preceding() -> String {
    "unbounded preceding".to_string()
}
fn default_current_row() -> String {
    "current row".to_string()
}

#[derive(Debug, Deserialize)]
pub struct WindowOp {
    pub from: String,
    #[serde(rename = "partitionBy")]
    pub partition_by: Vec<String>,
    #[serde(rename = "orderBy")]
    pub order_by: Option<Vec<SortColumnDef>>,
    pub frame: Option<FrameSpecOp>,
    pub functions: Vec<WindowFuncDefOp>,
}

#[derive(Debug, Deserialize)]
pub struct PivotOp {
    pub from: String,
    #[serde(rename = "groupBy")]
    pub group_by: Vec<String>,
    #[serde(rename = "pivotColumn")]
    pub pivot_column: String,
    pub values: Option<Vec<String>>,
    pub agg: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UnpivotOp {
    pub from: String,
    pub ids: Vec<String>,
    pub values: Vec<String>,
    #[serde(rename = "variableColumn")]
    pub variable_column: String,
    #[serde(rename = "valueColumn")]
    pub value_column: String,
}

#[derive(Debug, Deserialize)]
pub struct FlattenOp {
    pub from: String,
    #[serde(default = "default_separator")]
    pub separator: String,
    #[serde(rename = "explodeArrays", default)]
    pub explode_arrays: bool,
}

fn default_separator() -> String {
    "_".to_string()
}

#[derive(Debug, Deserialize)]
pub struct SampleOp {
    pub from: String,
    pub fraction: f64,
    #[serde(rename = "withReplacement", default)]
    pub with_replacement: bool,
    pub seed: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct BranchDef {
    pub condition: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct ConditionalOp {
    pub from: String,
    #[serde(rename = "outputColumn")]
    pub output_column: String,
    pub branches: Vec<BranchDef>,
    pub otherwise: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SplitOp {
    pub from: String,
    pub condition: String,
    pub pass: String,
    pub fail: String,
}

#[derive(Debug, Deserialize)]
pub struct SqlOp {
    pub query: String,
    pub views: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RollupOp {
    pub from: String,
    pub by: Vec<String>,
    pub agg: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CubeOp {
    pub from: String,
    pub by: Vec<String>,
    pub agg: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Scd2Op {
    pub current: String,
    pub incoming: String,
    #[serde(rename = "keyColumns")]
    pub key_columns: Vec<String>,
    #[serde(rename = "trackColumns")]
    pub track_columns: Vec<String>,
    #[serde(rename = "startDateColumn")]
    pub start_date_column: String,
    #[serde(rename = "endDateColumn")]
    pub end_date_column: String,
    #[serde(rename = "currentFlagColumn")]
    pub current_flag_column: String,
}

#[derive(Debug, Deserialize)]
pub struct EnrichOp {
    pub from: String,
    pub url: String,
    #[serde(default = "default_get")]
    pub method: String,
    #[serde(rename = "keyColumn")]
    pub key_column: String,
    #[serde(rename = "responseColumn")]
    pub response_column: String,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(rename = "onError", default = "default_null_str")]
    pub on_error: String,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(rename = "maxRetries", default = "default_retries")]
    pub max_retries: u32,
}

fn default_get() -> String {
    "GET".to_string()
}
fn default_null_str() -> String {
    "null".to_string()
}
fn default_timeout() -> u64 {
    30000
}
fn default_retries() -> u32 {
    3
}

#[derive(Debug, Deserialize)]
pub struct SchemaColumnDef {
    pub name: String,
    #[serde(rename = "dataType")]
    pub data_type: String,
    #[serde(default = "default_true")]
    pub nullable: bool,
    pub default: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SchemaEnforceOp {
    pub from: String,
    #[serde(default = "default_strict")]
    pub mode: String,
    pub columns: Vec<SchemaColumnDef>,
}

fn default_strict() -> String {
    "strict".to_string()
}

#[derive(Debug, Deserialize)]
pub struct QualityCheckDef {
    pub column: Option<String>,
    pub rule: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssertionOp {
    pub from: String,
    pub checks: Vec<QualityCheckDef>,
    #[serde(rename = "onFailure", default = "default_fail")]
    pub on_failure: String,
}

fn default_fail() -> String {
    "fail".to_string()
}

#[derive(Debug, Deserialize)]
pub struct RepartitionOp {
    pub from: String,
    #[serde(rename = "numPartitions")]
    pub num_partitions: u32,
    #[serde(default)]
    pub columns: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CoalesceOp {
    pub from: String,
    #[serde(rename = "numPartitions")]
    pub num_partitions: u32,
}

#[derive(Debug, Deserialize)]
pub struct CustomOp {
    pub from: String,
    pub component: String,
    #[serde(default)]
    pub options: BTreeMap<String, String>,
}

fn default_true() -> bool {
    true
}
