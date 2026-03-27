use crate::yaml::{self, TransformationOp};
use teckel_model::asset::{Asset, AssetMetadata, ColumnMetadata, Context};
use teckel_model::pipeline::Owner;
use teckel_model::source::*;
use teckel_model::types::*;
use teckel_model::{TeckelError, TeckelErrorCode};

/// Convert a parsed YAML document into the domain model context.
pub fn to_context(doc: &yaml::Document) -> Result<Context, TeckelError> {
    let mut context = Context::new();

    // Rewrite inputs
    for input in &doc.input {
        let metadata = AssetMetadata {
            description: input.description.clone(),
            tags: input.tags.clone().unwrap_or_default(),
            meta: input.meta.clone().unwrap_or_default(),
            owner: input.owner.as_ref().map(rewrite_owner),
            columns: input
                .columns
                .as_ref()
                .map(|cols| cols.iter().map(rewrite_column_metadata).collect())
                .unwrap_or_default(),
            ..Default::default()
        };
        let asset = Asset {
            asset_ref: input.name.clone(),
            source: Source::Input(InputSource {
                format: input.format.clone(),
                path: input.path.clone(),
                options: input.options.clone(),
            }),
            metadata,
        };
        if context.insert(input.name.clone(), asset).is_some() {
            return Err(TeckelError::spec(
                TeckelErrorCode::EName001,
                format!("duplicate AssetRef \"{}\"", input.name),
            ));
        }
    }

    // Rewrite transformations
    if let Some(transformations) = &doc.transformation {
        for t in transformations {
            let source = rewrite_transformation(t)?;

            let metadata = rewrite_transformation_metadata(t);

            // Split produces two assets (pass/fail), not the named one
            if let Source::Split(ref split) = source {
                let pass_asset = Asset {
                    asset_ref: split.pass.clone(),
                    source: Source::Where(WhereTransform {
                        from: split.from.clone(),
                        filter: split.condition.clone(),
                    }),
                    metadata: metadata.clone(),
                };
                let fail_asset = Asset {
                    asset_ref: split.fail.clone(),
                    source: Source::Where(WhereTransform {
                        from: split.from.clone(),
                        filter: format!("NOT({})", split.condition),
                    }),
                    metadata,
                };
                check_dup(&context, &split.pass)?;
                context.insert(split.pass.clone(), pass_asset);
                check_dup(&context, &split.fail)?;
                context.insert(split.fail.clone(), fail_asset);
            } else {
                check_dup(&context, &t.name)?;
                let asset = Asset {
                    asset_ref: t.name.clone(),
                    source,
                    metadata,
                };
                context.insert(t.name.clone(), asset);
            }
        }
    }

    // Rewrite outputs
    for output in &doc.output {
        let mode = match output.mode.as_str() {
            "overwrite" => WriteMode::Overwrite,
            "append" => WriteMode::Append,
            "ignore" => WriteMode::Ignore,
            "error" => WriteMode::Error,
            other => {
                return Err(TeckelError::spec(
                    TeckelErrorCode::EMode001,
                    format!(
                        "unknown write mode \"{other}\", expected: error|overwrite|append|ignore"
                    ),
                ))
            }
        };

        let output_key = format!("output_{}", output.name);
        let metadata = AssetMetadata {
            description: output.description.clone(),
            tags: output.tags.clone().unwrap_or_default(),
            meta: output.meta.clone().unwrap_or_default(),
            freshness: output.freshness.clone(),
            maturity: output.maturity.clone(),
            ..Default::default()
        };
        let asset = Asset {
            asset_ref: output_key.clone(),
            source: Source::Output(OutputSource {
                asset_ref: output.name.clone(),
                format: output.format.clone(),
                path: output.path.clone(),
                mode,
                options: output.options.clone(),
            }),
            metadata,
        };
        context.insert(output_key, asset);
    }

    Ok(context)
}

fn check_dup(context: &Context, name: &str) -> Result<(), TeckelError> {
    if context.contains_key(name) {
        return Err(TeckelError::spec(
            TeckelErrorCode::EName001,
            format!("duplicate AssetRef \"{name}\""),
        ));
    }
    Ok(())
}

fn rewrite_transformation(t: &yaml::RawTransformation) -> Result<Source, TeckelError> {
    match &t.operation {
        TransformationOp::Select(op) => Ok(Source::Select(SelectTransform {
            from: op.from.clone(),
            columns: op.columns.clone(),
        })),
        TransformationOp::Where(op) => Ok(Source::Where(WhereTransform {
            from: op.from.clone(),
            filter: op.filter.clone(),
        })),
        TransformationOp::Group(op) => Ok(Source::GroupBy(GroupByTransform {
            from: op.from.clone(),
            by: op.by.clone(),
            agg: op.agg.clone(),
        })),
        TransformationOp::OrderBy(op) => Ok(Source::OrderBy(OrderByTransform {
            from: op.from.clone(),
            columns: op.columns.iter().map(rewrite_sort_column).collect(),
        })),
        TransformationOp::Join(op) => Ok(Source::Join(JoinTransform {
            left: op.left.clone(),
            right: op
                .right
                .iter()
                .map(|jt| {
                    Ok(JoinTarget {
                        name: jt.name.clone(),
                        join_type: parse_join_type(&jt.join_type)?,
                        on: jt.on.clone(),
                    })
                })
                .collect::<Result<Vec<_>, TeckelError>>()?,
        })),
        TransformationOp::Union(op) => Ok(Source::Union(UnionTransform {
            sources: op.sources.clone(),
            all: op.all,
        })),
        TransformationOp::Intersect(op) => Ok(Source::Intersect(IntersectTransform {
            sources: op.sources.clone(),
            all: op.all,
        })),
        TransformationOp::Except(op) => Ok(Source::Except(ExceptTransform {
            left: op.left.clone(),
            right: op.right.clone(),
            all: op.all,
        })),
        TransformationOp::Distinct(op) => Ok(Source::Distinct(DistinctTransform {
            from: op.from.clone(),
            columns: op.columns.clone(),
        })),
        TransformationOp::Limit(op) => Ok(Source::Limit(LimitTransform {
            from: op.from.clone(),
            count: op.count,
        })),
        TransformationOp::AddColumns(op) => Ok(Source::AddColumns(AddColumnsTransform {
            from: op.from.clone(),
            columns: op
                .columns
                .iter()
                .map(|c| ColumnDef {
                    name: c.name.clone(),
                    expression: c.expression.clone(),
                })
                .collect(),
        })),
        TransformationOp::DropColumns(op) => Ok(Source::DropColumns(DropColumnsTransform {
            from: op.from.clone(),
            columns: op.columns.clone(),
        })),
        TransformationOp::RenameColumns(op) => Ok(Source::RenameColumns(RenameColumnsTransform {
            from: op.from.clone(),
            mappings: op.mappings.clone(),
        })),
        TransformationOp::CastColumns(op) => Ok(Source::CastColumns(CastColumnsTransform {
            from: op.from.clone(),
            columns: op
                .columns
                .iter()
                .map(|c| CastDef {
                    name: c.name.clone(),
                    target_type: c.target_type.clone(),
                })
                .collect(),
        })),
        TransformationOp::Window(op) => Ok(Source::Window(WindowTransform {
            from: op.from.clone(),
            partition_by: op.partition_by.clone(),
            order_by: op
                .order_by
                .as_ref()
                .map(|cols| cols.iter().map(rewrite_sort_column).collect())
                .unwrap_or_default(),
            frame: op
                .frame
                .as_ref()
                .map(|f| FrameSpec {
                    frame_type: if f.frame_type == "rows" {
                        FrameType::Rows
                    } else {
                        FrameType::Range
                    },
                    start: f.start.clone(),
                    end: f.end.clone(),
                })
                .unwrap_or_default(),
            functions: op
                .functions
                .iter()
                .map(|f| WindowFuncDef {
                    expression: f.expression.clone(),
                    alias: f.alias.clone(),
                })
                .collect(),
        })),
        TransformationOp::Pivot(op) => Ok(Source::Pivot(PivotTransform {
            from: op.from.clone(),
            group_by: op.group_by.clone(),
            pivot_column: op.pivot_column.clone(),
            values: op.values.clone(),
            agg: op.agg.clone(),
        })),
        TransformationOp::Unpivot(op) => Ok(Source::Unpivot(UnpivotTransform {
            from: op.from.clone(),
            ids: op.ids.clone(),
            values: op.values.clone(),
            variable_column: op.variable_column.clone(),
            value_column: op.value_column.clone(),
        })),
        TransformationOp::Flatten(op) => Ok(Source::Flatten(FlattenTransform {
            from: op.from.clone(),
            separator: op.separator.clone(),
            explode_arrays: op.explode_arrays,
        })),
        TransformationOp::Sample(op) => Ok(Source::Sample(SampleTransform {
            from: op.from.clone(),
            fraction: op.fraction,
            with_replacement: op.with_replacement,
            seed: op.seed,
        })),
        TransformationOp::Conditional(op) => Ok(Source::Conditional(ConditionalTransform {
            from: op.from.clone(),
            output_column: op.output_column.clone(),
            branches: op
                .branches
                .iter()
                .map(|b| Branch {
                    condition: b.condition.clone(),
                    value: b.value.clone(),
                })
                .collect(),
            otherwise: op.otherwise.clone(),
        })),
        TransformationOp::Split(op) => Ok(Source::Split(SplitTransform {
            from: op.from.clone(),
            condition: op.condition.clone(),
            pass: op.pass.clone(),
            fail: op.fail.clone(),
        })),
        TransformationOp::Sql(op) => Ok(Source::Sql(SqlTransform {
            query: op.query.clone(),
            views: op.views.clone(),
        })),
        TransformationOp::Rollup(op) => Ok(Source::Rollup(RollupTransform {
            from: op.from.clone(),
            by: op.by.clone(),
            agg: op.agg.clone(),
        })),
        TransformationOp::Cube(op) => Ok(Source::Cube(CubeTransform {
            from: op.from.clone(),
            by: op.by.clone(),
            agg: op.agg.clone(),
        })),
        TransformationOp::Scd2(op) => Ok(Source::Scd2(Scd2Transform {
            current: op.current.clone(),
            incoming: op.incoming.clone(),
            key_columns: op.key_columns.clone(),
            track_columns: op.track_columns.clone(),
            start_date_column: op.start_date_column.clone(),
            end_date_column: op.end_date_column.clone(),
            current_flag_column: op.current_flag_column.clone(),
        })),
        TransformationOp::Enrich(op) => Ok(Source::Enrich(EnrichTransform {
            from: op.from.clone(),
            url: op.url.clone(),
            method: op.method.clone(),
            key_column: op.key_column.clone(),
            response_column: op.response_column.clone(),
            headers: op.headers.clone(),
            on_error: match op.on_error.as_str() {
                "fail" => OnError::Fail,
                "skip" => OnError::Skip,
                _ => OnError::Null,
            },
            timeout: op.timeout,
            max_retries: op.max_retries,
        })),
        TransformationOp::SchemaEnforce(op) => Ok(Source::SchemaEnforce(SchemaEnforceTransform {
            from: op.from.clone(),
            mode: match op.mode.as_str() {
                "evolve" => SchemaEnforceMode::Evolve,
                _ => SchemaEnforceMode::Strict,
            },
            columns: op
                .columns
                .iter()
                .map(|c| SchemaColumn {
                    name: c.name.clone(),
                    data_type: c.data_type.clone(),
                    nullable: c.nullable,
                    default: c.default.clone(),
                })
                .collect(),
        })),
        TransformationOp::Assertion(op) => Ok(Source::Assertion(AssertionTransform {
            from: op.from.clone(),
            checks: op
                .checks
                .iter()
                .map(|c| teckel_model::source::QualityCheck {
                    column: c.column.clone(),
                    rule: c.rule.clone(),
                    description: c.description.clone(),
                })
                .collect(),
            on_failure: match op.on_failure.as_str() {
                "warn" => OnFailure::Warn,
                "drop" => OnFailure::Drop,
                _ => OnFailure::Fail,
            },
        })),
        TransformationOp::Repartition(op) => Ok(Source::Repartition(RepartitionTransform {
            from: op.from.clone(),
            num_partitions: op.num_partitions,
            columns: op.columns.clone(),
        })),
        TransformationOp::Coalesce(op) => Ok(Source::Coalesce(CoalesceTransform {
            from: op.from.clone(),
            num_partitions: op.num_partitions,
        })),
        TransformationOp::Custom(op) => Ok(Source::Custom(CustomTransform {
            from: op.from.clone(),
            component: op.component.clone(),
            options: op.options.clone(),
        })),
    }
}

fn rewrite_sort_column(sc: &yaml::operations::SortColumnDef) -> SortColumn {
    match sc {
        yaml::operations::SortColumnDef::Simple(name) => SortColumn::Simple(name.clone()),
        yaml::operations::SortColumnDef::Explicit {
            column,
            direction,
            nulls,
        } => SortColumn::Explicit {
            column: column.clone(),
            direction: if direction == "desc" {
                SortDirection::Desc
            } else {
                SortDirection::Asc
            },
            nulls: if nulls == "first" {
                NullOrdering::First
            } else {
                NullOrdering::Last
            },
        },
    }
}

fn rewrite_transformation_metadata(t: &yaml::RawTransformation) -> AssetMetadata {
    AssetMetadata {
        description: t.description.clone(),
        tags: t.tags.clone().unwrap_or_default(),
        remove_tags: t.remove_tags.clone().unwrap_or_default(),
        meta: t.meta.clone().unwrap_or_default(),
        ..Default::default()
    }
}

fn rewrite_owner(o: &crate::yaml::document::OwnerDef) -> Owner {
    Owner {
        name: o.name.clone(),
        email: o.email.clone(),
        owner_type: o.owner_type.clone(),
    }
}

fn rewrite_column_metadata(c: &crate::yaml::input::ColumnMetadataDef) -> ColumnMetadata {
    ColumnMetadata {
        name: c.name.clone(),
        description: c.description.clone(),
        tags: c.tags.clone().unwrap_or_default(),
        constraints: c.constraints.clone().unwrap_or_default(),
        meta: c.meta.clone().unwrap_or_default(),
        glossary_term: None,
    }
}

fn parse_join_type(s: &str) -> Result<JoinType, TeckelError> {
    match s {
        "inner" => Ok(JoinType::Inner),
        "left" => Ok(JoinType::Left),
        "right" => Ok(JoinType::Right),
        "outer" => Ok(JoinType::Outer),
        "cross" => Ok(JoinType::Cross),
        "left_semi" => Ok(JoinType::LeftSemi),
        "left_anti" => Ok(JoinType::LeftAnti),
        other => Err(TeckelError::spec(
            TeckelErrorCode::EEnum001,
            format!(
                "invalid join type \"{other}\", expected: inner|left|right|outer|cross|left_semi|left_anti"
            ),
        )),
    }
}
