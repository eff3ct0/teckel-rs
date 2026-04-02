---
sidebar_position: 2
title: Domain Model
---

# Domain Model

The domain model lives in the `teckel-model` crate. It defines every type needed to represent a parsed Teckel pipeline.

## Asset and Context

An `Asset` is a named node in the pipeline DAG. Every input, transformation, and output produces an asset.

```rust
pub struct Asset {
    pub asset_ref: AssetRef,
    pub source: Source,
    pub metadata: AssetMetadata,
}

/// A collection of assets keyed by their unique reference.
pub type Context = BTreeMap<AssetRef, Asset>;
```

`AssetRef` is a type alias for `String`. It must match the pattern `^[a-zA-Z][a-zA-Z0-9_-]{0,127}$`.

## Source enum

The `Source` enum is the heart of the domain model. It has 47 variants: 2 I/O types and 45 transformations.

```rust
pub enum Source {
    // I/O
    Input(InputSource),
    Output(OutputSource),

    // Core transformations (8.1 - 8.10)
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

    // Extended transformations (8.11 - 8.31)
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

    // v3 transformations (8.32 - 8.45)
    Offset(OffsetTransform),
    Tail(TailTransform),
    FillNa(FillNaTransform),
    DropNa(DropNaTransform),
    Replace(ReplaceTransform),
    Merge(MergeTransform),
    Parse(ParseTransform),
    AsOfJoin(AsOfJoinTransform),
    LateralJoin(LateralJoinTransform),
    Transpose(TransposeTransform),
    GroupingSets(GroupingSetsTransform),
    Describe(DescribeTransform),
    Crosstab(CrosstabTransform),
    Hint(HintTransform),
}
```

Each variant wraps a dedicated struct. For example:

```rust
pub struct SelectTransform {
    pub from: AssetRef,
    pub columns: Vec<Expression>,
}

pub struct JoinTransform {
    pub left: AssetRef,
    pub right: Vec<JoinTarget>,
}

pub struct JoinTarget {
    pub name: AssetRef,
    pub join_type: JoinType,
    pub on: Vec<Condition>,
}
```

The `Source` enum also provides a `dependencies()` method that returns all upstream `AssetRef` values for any variant:

```rust
impl Source {
    pub fn dependencies(&self) -> Vec<&AssetRef> { /* ... */ }
}
```

## TeckelDataType

The type system covers all Teckel v3.0 data types:

```rust
pub enum TeckelDataType {
    String,
    Integer,    // alias: int
    Long,       // alias: bigint
    Float,
    Double,
    Boolean,
    Date,
    Timestamp,
    Binary,
    Decimal { precision: u8, scale: u8 },
    Array(Box<TeckelDataType>),
    Map(Box<TeckelDataType>, Box<TeckelDataType>),
    Struct(Vec<StructField>),
    // v3 types
    Byte,           // alias: tinyint
    Short,          // alias: smallint
    Char(u32),
    VarChar(u32),
    TimestampNtz,
    Time(Option<u8>),
    YearMonthInterval,
    DayTimeInterval,
    Variant,
}
```

## Supporting types

The `types.rs` module defines several newtypes and enums:

| Type | Definition |
|---|---|
| `AssetRef` | `String` -- unique asset identifier |
| `Column` | `String` -- column name, optionally qualified as `asset.column` |
| `Expression` | `String` -- SQL-like expression |
| `Condition` | `String` -- boolean expression |
| `Options` | `BTreeMap<String, Primitive>` -- format-specific key-value options |
| `Primitive` | Enum: `Bool`, `Int`, `Float`, `String` |
| `Format` | Enum: `Csv`, `Json`, `Parquet`, `Delta`, `Orc`, `Avro`, `Jdbc`, `Custom(String)` |
| `WriteMode` | Enum: `Error`, `Overwrite`, `Append`, `Ignore` |
| `JoinType` | Enum: `Inner`, `Left`, `Right`, `Outer`, `Cross`, `LeftSemi`, `LeftAnti` |
| `SortColumn` | Enum: `Simple(Column)` or `Explicit { column, direction, nulls }` |
| `FrameType` | Enum: `Rows`, `Range` |
| `AsOfDirection` | Enum: `Backward`, `Forward`, `Nearest` |
| `MergeActionType` | Enum: `Update`, `Insert`, `Delete` |
| `ParseFormat` | Enum: `Json`, `Csv` |
| `DropNaHow` | Enum: `Any`, `All` |

## Pipeline struct

The top-level `Pipeline` struct contains everything from a parsed Teckel document:

```rust
pub struct Pipeline {
    pub context: Context,
    pub metadata: PipelineMetadata,
    pub config: PipelineConfig,
    pub hooks: Hooks,
    pub quality: Vec<QualitySuite>,
    pub templates: Vec<Template>,
    pub exposures: Vec<Exposure>,
    pub streaming_inputs: Vec<StreamingInput>,
    pub streaming_outputs: Vec<StreamingOutput>,
    pub secrets: BTreeMap<String, SecretKey>,
}
```

## Quality suites

Data quality is modeled through `QualitySuite` with 10 check types:

```rust
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
```

Each check targets a specific asset and runs with a configurable severity (`Error`, `Warn`, `Info`).

## Asset metadata

Every asset can carry metadata:

```rust
pub struct AssetMetadata {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub remove_tags: Vec<String>,
    pub meta: BTreeMap<String, serde_yaml::Value>,
    pub owner: Option<Owner>,
    pub columns: Vec<ColumnMetadata>,
    pub freshness: Option<String>,
    pub maturity: Option<String>,
}
```

Column-level metadata is also supported:

```rust
pub struct ColumnMetadata {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub constraints: Vec<String>,
    pub meta: BTreeMap<String, serde_yaml::Value>,
    pub glossary_term: Option<String>,
}
```
