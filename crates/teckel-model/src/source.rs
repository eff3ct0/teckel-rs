use crate::types::*;
use std::collections::BTreeMap;

/// The source definition of an asset -- either an input, an output, or one of 31 transformations.
#[derive(Debug, Clone, PartialEq)]
pub enum Source {
    // ── I/O ──────────────────────────────────────────────────
    Input(InputSource),
    Output(OutputSource),

    // ── Core transformations (8.1 - 8.10) ────────────────────
    Select(SelectTransform),
    Where(WhereTransform),
    GroupBy(GroupByTransform),
    OrderBy(OrderByTransform),
    Join(JoinTransform),
    Union(UnionTransform),
    Intersect(IntersectTransform),
    Except(ExceptTransform),
    Distinct(DistinctTransform),
    Limit(LimitTransform),

    // ── Extended transformations (8.11 - 8.31) ───────────────
    AddColumns(AddColumnsTransform),
    DropColumns(DropColumnsTransform),
    RenameColumns(RenameColumnsTransform),
    CastColumns(CastColumnsTransform),
    Window(WindowTransform),
    Pivot(PivotTransform),
    Unpivot(UnpivotTransform),
    Flatten(FlattenTransform),
    Sample(SampleTransform),
    Conditional(ConditionalTransform),
    Split(SplitTransform),
    Sql(SqlTransform),
    Rollup(RollupTransform),
    Cube(CubeTransform),
    Scd2(Scd2Transform),
    Enrich(EnrichTransform),
    SchemaEnforce(SchemaEnforceTransform),
    Assertion(AssertionTransform),
    Repartition(RepartitionTransform),
    Coalesce(CoalesceTransform),
    Custom(CustomTransform),
}

impl Source {
    /// Returns all upstream asset references this source depends on.
    pub fn dependencies(&self) -> Vec<&AssetRef> {
        match self {
            Source::Input(_) => vec![],
            Source::Output(o) => vec![&o.asset_ref],
            Source::Select(t) => vec![&t.from],
            Source::Where(t) => vec![&t.from],
            Source::GroupBy(t) => vec![&t.from],
            Source::OrderBy(t) => vec![&t.from],
            Source::Join(t) => {
                let mut deps = vec![&t.left];
                for target in &t.right {
                    deps.push(&target.name);
                }
                deps
            }
            Source::Union(t) => t.sources.iter().collect(),
            Source::Intersect(t) => t.sources.iter().collect(),
            Source::Except(t) => vec![&t.left, &t.right],
            Source::Distinct(t) => vec![&t.from],
            Source::Limit(t) => vec![&t.from],
            Source::AddColumns(t) => vec![&t.from],
            Source::DropColumns(t) => vec![&t.from],
            Source::RenameColumns(t) => vec![&t.from],
            Source::CastColumns(t) => vec![&t.from],
            Source::Window(t) => vec![&t.from],
            Source::Pivot(t) => vec![&t.from],
            Source::Unpivot(t) => vec![&t.from],
            Source::Flatten(t) => vec![&t.from],
            Source::Sample(t) => vec![&t.from],
            Source::Conditional(t) => vec![&t.from],
            Source::Split(t) => vec![&t.from],
            Source::Sql(t) => t.views.iter().collect(),
            Source::Rollup(t) => vec![&t.from],
            Source::Cube(t) => vec![&t.from],
            Source::Scd2(t) => vec![&t.current, &t.incoming],
            Source::Enrich(t) => vec![&t.from],
            Source::SchemaEnforce(t) => vec![&t.from],
            Source::Assertion(t) => vec![&t.from],
            Source::Repartition(t) => vec![&t.from],
            Source::Coalesce(t) => vec![&t.from],
            Source::Custom(t) => vec![&t.from],
        }
    }
}

// ── I/O types ────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct InputSource {
    pub format: String,
    pub path: String,
    pub options: Options,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutputSource {
    pub asset_ref: AssetRef,
    pub format: String,
    pub path: String,
    pub mode: WriteMode,
    pub options: Options,
}

// ── Core transformations ─────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct SelectTransform {
    pub from: AssetRef,
    pub columns: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhereTransform {
    pub from: AssetRef,
    pub filter: Condition,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroupByTransform {
    pub from: AssetRef,
    pub by: Vec<Column>,
    pub agg: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderByTransform {
    pub from: AssetRef,
    pub columns: Vec<SortColumn>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JoinTransform {
    pub left: AssetRef,
    pub right: Vec<JoinTarget>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JoinTarget {
    pub name: AssetRef,
    pub join_type: JoinType,
    pub on: Vec<Condition>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnionTransform {
    pub sources: Vec<AssetRef>,
    pub all: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntersectTransform {
    pub sources: Vec<AssetRef>,
    pub all: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExceptTransform {
    pub left: AssetRef,
    pub right: AssetRef,
    pub all: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DistinctTransform {
    pub from: AssetRef,
    pub columns: Option<Vec<Column>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitTransform {
    pub from: AssetRef,
    pub count: u64,
}

// ── Extended transformations ─────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDef {
    pub name: String,
    pub expression: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AddColumnsTransform {
    pub from: AssetRef,
    pub columns: Vec<ColumnDef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DropColumnsTransform {
    pub from: AssetRef,
    pub columns: Vec<Column>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenameColumnsTransform {
    pub from: AssetRef,
    pub mappings: BTreeMap<Column, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CastDef {
    pub name: Column,
    pub target_type: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CastColumnsTransform {
    pub from: AssetRef,
    pub columns: Vec<CastDef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WindowFuncDef {
    pub expression: Expression,
    pub alias: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FrameSpec {
    pub frame_type: FrameType,
    pub start: String,
    pub end: String,
}

impl Default for FrameSpec {
    fn default() -> Self {
        Self {
            frame_type: FrameType::Range,
            start: "unbounded preceding".to_string(),
            end: "current row".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WindowTransform {
    pub from: AssetRef,
    pub partition_by: Vec<Column>,
    pub order_by: Vec<SortColumn>,
    pub frame: FrameSpec,
    pub functions: Vec<WindowFuncDef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PivotTransform {
    pub from: AssetRef,
    pub group_by: Vec<Column>,
    pub pivot_column: Column,
    pub values: Option<Vec<String>>,
    pub agg: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnpivotTransform {
    pub from: AssetRef,
    pub ids: Vec<Column>,
    pub values: Vec<Column>,
    pub variable_column: String,
    pub value_column: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlattenTransform {
    pub from: AssetRef,
    pub separator: String,
    pub explode_arrays: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SampleTransform {
    pub from: AssetRef,
    pub fraction: f64,
    pub with_replacement: bool,
    pub seed: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Branch {
    pub condition: Condition,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalTransform {
    pub from: AssetRef,
    pub output_column: String,
    pub branches: Vec<Branch>,
    pub otherwise: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SplitTransform {
    pub from: AssetRef,
    pub condition: Condition,
    pub pass: AssetRef,
    pub fail: AssetRef,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SqlTransform {
    pub query: String,
    pub views: Vec<AssetRef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollupTransform {
    pub from: AssetRef,
    pub by: Vec<Column>,
    pub agg: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CubeTransform {
    pub from: AssetRef,
    pub by: Vec<Column>,
    pub agg: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scd2Transform {
    pub current: AssetRef,
    pub incoming: AssetRef,
    pub key_columns: Vec<Column>,
    pub track_columns: Vec<Column>,
    pub start_date_column: String,
    pub end_date_column: String,
    pub current_flag_column: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnrichTransform {
    pub from: AssetRef,
    pub url: String,
    pub method: String,
    pub key_column: Column,
    pub response_column: String,
    pub headers: BTreeMap<String, String>,
    pub on_error: OnError,
    pub timeout: u64,
    pub max_retries: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaColumn {
    pub name: Column,
    pub data_type: String,
    pub nullable: bool,
    pub default: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaEnforceTransform {
    pub from: AssetRef,
    pub mode: SchemaEnforceMode,
    pub columns: Vec<SchemaColumn>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QualityCheck {
    pub column: Option<Column>,
    pub rule: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssertionTransform {
    pub from: AssetRef,
    pub checks: Vec<QualityCheck>,
    pub on_failure: OnFailure,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepartitionTransform {
    pub from: AssetRef,
    pub num_partitions: u32,
    pub columns: Vec<Column>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoalesceTransform {
    pub from: AssetRef,
    pub num_partitions: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomTransform {
    pub from: AssetRef,
    pub component: String,
    pub options: BTreeMap<String, String>,
}
