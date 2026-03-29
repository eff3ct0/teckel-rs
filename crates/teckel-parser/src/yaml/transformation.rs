use super::operations::*;
use serde::Deserialize;
use std::collections::BTreeMap;

/// YAML representation of a transformation entry (Section 8).
///
/// Each transformation has a `name` and exactly one operation key.
/// We use `#[serde(flatten)]` to dispatch to the correct operation variant.
#[derive(Debug, Deserialize)]
pub struct RawTransformation {
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    #[serde(rename = "removeTags")]
    pub remove_tags: Option<Vec<String>>,
    pub meta: Option<BTreeMap<String, serde_yaml::Value>>,
    #[serde(flatten)]
    pub operation: TransformationOp,
}

/// Discriminated union of all 31 transformation operation types.
/// Serde dispatches based on which YAML key is present.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformationOp {
    // Core (8.1 - 8.10)
    Select(SelectOp),
    Where(WhereOp),
    Group(GroupOp),
    OrderBy(OrderByOp),
    Join(JoinOp),
    Union(UnionOp),
    Intersect(IntersectOp),
    Except(ExceptOp),
    Distinct(DistinctOp),
    Limit(LimitOp),

    // Extended (8.11 - 8.31)
    AddColumns(AddColumnsOp),
    DropColumns(DropColumnsOp),
    RenameColumns(RenameColumnsOp),
    CastColumns(CastColumnsOp),
    Window(WindowOp),
    Pivot(PivotOp),
    Unpivot(UnpivotOp),
    Flatten(FlattenOp),
    Sample(SampleOp),
    Conditional(ConditionalOp),
    Split(SplitOp),
    Sql(SqlOp),
    Rollup(RollupOp),
    Cube(CubeOp),
    Scd2(Scd2Op),
    Enrich(EnrichOp),
    SchemaEnforce(SchemaEnforceOp),
    Assertion(AssertionOp),
    Repartition(RepartitionOp),
    Coalesce(CoalesceOp),
    Custom(CustomOp),

    // v3 (8.32 - 8.45)
    Offset(OffsetOp),
    Tail(TailOp),
    FillNa(FillNaOp),
    DropNa(DropNaOp),
    Replace(ReplaceOp),
    Merge(MergeOp),
    Parse(ParseOp),
    AsOfJoin(AsOfJoinOp),
    LateralJoin(LateralJoinOp),
    Transpose(TransposeOp),
    GroupingSets(GroupingSetsOp),
    Describe(DescribeOp),
    Crosstab(CrosstabOp),
    Hint(HintOp),
}
