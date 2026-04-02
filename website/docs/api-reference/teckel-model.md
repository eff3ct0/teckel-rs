---
sidebar_position: 1
title: teckel-model
---

# teckel-model

The `teckel-model` crate defines the public domain types for the Teckel pipeline model. It has minimal dependencies (`serde`, `serde_yaml`, `thiserror`) and is designed to be consumed by downstream crates without pulling in heavy parsing machinery.

**Crate path:** `crates/teckel-model/`

## Re-exports (lib.rs)

```rust
pub use asset::{Asset, AssetMetadata, ColumnMetadata, Context};
pub use error::{TeckelError, TeckelErrorCode};
pub use pipeline::Pipeline;
pub use source::Source;
pub use types::AssetRef;
```

## Source enum

The central type. Each variant corresponds to a Teckel asset type.

### I/O variants

| Variant | Struct | Key fields |
|---|---|---|
| `Input` | `InputSource` | `format`, `path`, `options` |
| `Output` | `OutputSource` | `asset_ref`, `format`, `path`, `mode`, `options` |

### Core transformations (8.1 -- 8.10)

| Variant | Struct | Key fields |
|---|---|---|
| `Select` | `SelectTransform` | `from`, `columns: Vec<Expression>` |
| `Where` | `WhereTransform` | `from`, `filter: Condition` |
| `GroupBy` | `GroupByTransform` | `from`, `by: Vec<Column>`, `agg: Vec<Expression>` |
| `OrderBy` | `OrderByTransform` | `from`, `columns: Vec<SortColumn>` |
| `Join` | `JoinTransform` | `left`, `right: Vec<JoinTarget>` |
| `Union` | `UnionTransform` | `sources: Vec<AssetRef>`, `all`, `by_name`, `allow_missing_columns` |
| `Intersect` | `IntersectTransform` | `sources: Vec<AssetRef>`, `all`, `by_name`, `allow_missing_columns` |
| `Except` | `ExceptTransform` | `left`, `right`, `all`, `by_name`, `allow_missing_columns` |
| `Distinct` | `DistinctTransform` | `from`, `columns: Option<Vec<Column>>` |
| `Limit` | `LimitTransform` | `from`, `count: u64` |

### Extended transformations (8.11 -- 8.31)

| Variant | Struct | Key fields |
|---|---|---|
| `AddColumns` | `AddColumnsTransform` | `from`, `columns: Vec<ColumnDef>` |
| `DropColumns` | `DropColumnsTransform` | `from`, `columns: Vec<Column>` |
| `RenameColumns` | `RenameColumnsTransform` | `from`, `mappings: BTreeMap<Column, String>` |
| `CastColumns` | `CastColumnsTransform` | `from`, `columns: Vec<CastDef>` |
| `Window` | `WindowTransform` | `from`, `partition_by`, `order_by`, `frame`, `functions` |
| `Pivot` | `PivotTransform` | `from`, `group_by`, `pivot_column`, `values`, `agg` |
| `Unpivot` | `UnpivotTransform` | `from`, `ids`, `values`, `variable_column`, `value_column` |
| `Flatten` | `FlattenTransform` | `from`, `separator`, `explode_arrays` |
| `Sample` | `SampleTransform` | `from`, `fraction`, `with_replacement`, `seed`, bounds |
| `Conditional` | `ConditionalTransform` | `from`, `output_column`, `branches`, `otherwise` |
| `Split` | `SplitTransform` | `from`, `condition`, `pass`, `fail` |
| `Sql` | `SqlTransform` | `query`, `views: Vec<AssetRef>` |
| `Rollup` | `RollupTransform` | `from`, `by`, `agg` |
| `Cube` | `CubeTransform` | `from`, `by`, `agg` |
| `Scd2` | `Scd2Transform` | `current`, `incoming`, `key_columns`, `track_columns`, date columns |
| `Enrich` | `EnrichTransform` | `from`, `url`, `method`, `key_column`, `response_column`, `headers` |
| `SchemaEnforce` | `SchemaEnforceTransform` | `from`, `mode`, `columns: Vec<SchemaColumn>` |
| `Assertion` | `AssertionTransform` | `from`, `checks: Vec<QualityCheck>`, `on_failure` |
| `Repartition` | `RepartitionTransform` | `from`, `num_partitions`, `columns` |
| `Coalesce` | `CoalesceTransform` | `from`, `num_partitions` |
| `Custom` | `CustomTransform` | `from`, `component`, `options` |

### v3 transformations (8.32 -- 8.45)

| Variant | Struct | Key fields |
|---|---|---|
| `Offset` | `OffsetTransform` | `from`, `count: u64` |
| `Tail` | `TailTransform` | `from`, `count: u64` |
| `FillNa` | `FillNaTransform` | `from`, `columns`, `value`, `values` |
| `DropNa` | `DropNaTransform` | `from`, `columns`, `how`, `min_non_nulls` |
| `Replace` | `ReplaceTransform` | `from`, `columns`, `mappings: Vec<Replacement>` |
| `Merge` | `MergeTransform` | `target`, `source`, `on`, `when_matched`, `when_not_matched` |
| `Parse` | `ParseTransform` | `from`, `column`, `format`, `schema`, `options` |
| `AsOfJoin` | `AsOfJoinTransform` | `left`, `right`, `left_as_of`, `right_as_of`, `direction`, `tolerance` |
| `LateralJoin` | `LateralJoinTransform` | `left`, `right`, `join_type`, `on` |
| `Transpose` | `TransposeTransform` | `from`, `index_columns` |
| `GroupingSets` | `GroupingSetsTransform` | `from`, `sets`, `agg` |
| `Describe` | `DescribeTransform` | `from`, `columns`, `statistics` |
| `Crosstab` | `CrosstabTransform` | `from`, `col1`, `col2` |
| `Hint` | `HintTransform` | `from`, `hints: Vec<HintSpec>` |

## TeckelDataType

All supported data types:

| Type | Rust variant | Notes |
|---|---|---|
| string | `String` | |
| integer | `Integer` | alias: `int` |
| long | `Long` | alias: `bigint` |
| float | `Float` | |
| double | `Double` | |
| boolean | `Boolean` | |
| date | `Date` | |
| timestamp | `Timestamp` | |
| binary | `Binary` | |
| decimal | `Decimal { precision, scale }` | |
| array | `Array(Box<TeckelDataType>)` | |
| map | `Map(Box<TeckelDataType>, Box<TeckelDataType>)` | |
| struct | `Struct(Vec<StructField>)` | |
| byte | `Byte` | alias: `tinyint` (v3) |
| short | `Short` | alias: `smallint` (v3) |
| char | `Char(u32)` | v3 |
| varchar | `VarChar(u32)` | v3 |
| timestamp_ntz | `TimestampNtz` | v3 |
| time | `Time(Option<u8>)` | v3, optional precision |
| year_month_interval | `YearMonthInterval` | v3 |
| day_time_interval | `DayTimeInterval` | v3 |
| variant | `Variant` | v3 |

## Pipeline

The top-level output type containing all parsed pipeline data:

| Field | Type | Description |
|---|---|---|
| `context` | `Context` | All assets keyed by name |
| `metadata` | `PipelineMetadata` | Name, namespace, version, owner, tags, schedule, freshness |
| `config` | `PipelineConfig` | Backend, cache, notifications |
| `hooks` | `Hooks` | Pre/post execution lifecycle hooks |
| `quality` | `Vec<QualitySuite>` | Data quality suites |
| `templates` | `Vec<Template>` | Reusable templates |
| `exposures` | `Vec<Exposure>` | Downstream consumer declarations |
| `streaming_inputs` | `Vec<StreamingInput>` | Streaming input definitions |
| `streaming_outputs` | `Vec<StreamingOutput>` | Streaming output definitions |
| `secrets` | `BTreeMap<String, SecretKey>` | Secret key declarations |

## Quality types

Ten check types within `QualitySuite`:

| Check | Key fields |
|---|---|
| `Schema` | `required_columns`, `forbidden_columns`, `types` |
| `Completeness` | `column`, `threshold`, `severity`, `escalate` |
| `Uniqueness` | `columns`, `threshold` |
| `Validity` | `column`, `accepted_values`, `range`, `pattern`, `format`, `length_between`, `threshold` |
| `Statistical` | `column`, `mean`, `min`, `max`, `sum`, `stdev`, `quantiles` |
| `Volume` | `row_count`, `column_count` |
| `Freshness` | `column`, `max_age` |
| `Referential` | `column`, `reference_asset`, `reference_column`, `threshold` |
| `CrossColumn` | `condition`, `description`, `threshold` |
| `Custom` | `condition`, `description`, `threshold` |
