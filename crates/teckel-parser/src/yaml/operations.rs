use serde::Deserialize;
use std::collections::BTreeMap;
use teckel_model::types::Primitive;

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
    #[serde(rename = "byName", default)]
    pub by_name: bool,
    #[serde(rename = "allowMissingColumns", default)]
    pub allow_missing_columns: bool,
}

#[derive(Debug, Deserialize)]
pub struct IntersectOp {
    pub sources: Vec<String>,
    #[serde(default)]
    pub all: bool,
    #[serde(rename = "byName", default)]
    pub by_name: bool,
    #[serde(rename = "allowMissingColumns", default)]
    pub allow_missing_columns: bool,
}

#[derive(Debug, Deserialize)]
pub struct ExceptOp {
    pub left: String,
    pub right: String,
    #[serde(default)]
    pub all: bool,
    #[serde(rename = "byName", default)]
    pub by_name: bool,
    #[serde(rename = "allowMissingColumns", default)]
    pub allow_missing_columns: bool,
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
    #[serde(rename = "lowerBound")]
    pub lower_bound: Option<f64>,
    #[serde(rename = "upperBound")]
    pub upper_bound: Option<f64>,
    #[serde(rename = "deterministicOrder", default)]
    pub deterministic_order: bool,
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

// ── v3 operations ────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct OffsetOp {
    pub from: String,
    pub count: u64,
}

#[derive(Debug, Deserialize)]
pub struct TailOp {
    pub from: String,
    pub count: u64,
}

#[derive(Debug, Deserialize)]
pub struct FillNaOp {
    pub from: String,
    pub columns: Option<Vec<String>>,
    pub value: Option<Primitive>,
    pub values: Option<BTreeMap<String, Primitive>>,
}

#[derive(Debug, Deserialize)]
pub struct DropNaOp {
    pub from: String,
    pub columns: Option<Vec<String>>,
    #[serde(default)]
    pub how: Option<String>,
    #[serde(rename = "minNonNulls")]
    pub min_non_nulls: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ReplacementDef {
    pub old: Primitive,
    pub new: Primitive,
}

#[derive(Debug, Deserialize)]
pub struct ReplaceOp {
    pub from: String,
    pub columns: Option<Vec<String>>,
    pub mappings: Vec<ReplacementDef>,
}

#[derive(Debug, Deserialize)]
pub struct MergeActionDef {
    pub action: String,
    pub condition: Option<String>,
    pub set: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub star: bool,
}

#[derive(Debug, Deserialize)]
pub struct MergeOp {
    pub target: String,
    pub source: String,
    pub on: Vec<String>,
    #[serde(rename = "whenMatched", default)]
    pub when_matched: Vec<MergeActionDef>,
    #[serde(rename = "whenNotMatched", default)]
    pub when_not_matched: Vec<MergeActionDef>,
    #[serde(rename = "whenNotMatchedBySource", default)]
    pub when_not_matched_by_source: Vec<MergeActionDef>,
}

#[derive(Debug, Deserialize)]
pub struct ParseOp {
    pub from: String,
    pub column: String,
    pub format: String,
    pub schema: Option<Vec<SchemaColumnDef>>,
    #[serde(default)]
    pub options: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct AsOfJoinOp {
    pub left: String,
    pub right: String,
    #[serde(rename = "leftAsOf")]
    pub left_as_of: String,
    #[serde(rename = "rightAsOf")]
    pub right_as_of: String,
    #[serde(default)]
    pub on: Vec<String>,
    #[serde(rename = "type", default = "default_left")]
    pub join_type: Option<String>,
    #[serde(default)]
    pub direction: Option<String>,
    pub tolerance: Option<String>,
    #[serde(rename = "allowExactMatches", default = "default_true")]
    pub allow_exact_matches: bool,
}

fn default_left() -> Option<String> {
    Some("left".to_string())
}

#[derive(Debug, Deserialize)]
pub struct LateralJoinOp {
    pub left: String,
    pub right: String,
    #[serde(rename = "type", default)]
    pub join_type: Option<String>,
    #[serde(default)]
    pub on: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TransposeOp {
    pub from: String,
    #[serde(rename = "indexColumns", default)]
    pub index_columns: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GroupingSetsOp {
    pub from: String,
    pub sets: Vec<Vec<String>>,
    pub agg: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DescribeOp {
    pub from: String,
    pub columns: Option<Vec<String>>,
    pub statistics: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CrosstabOp {
    pub from: String,
    pub col1: String,
    pub col2: String,
}

#[derive(Debug, Deserialize)]
pub struct HintSpecDef {
    pub name: String,
    #[serde(default)]
    pub parameters: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct HintOp {
    pub from: String,
    pub hints: Vec<HintSpecDef>,
}
