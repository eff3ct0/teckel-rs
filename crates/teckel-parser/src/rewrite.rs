use crate::yaml::{self, TransformationOp};
use std::collections::BTreeMap;
use teckel_model::asset::{Asset, AssetMetadata, ColumnMetadata, Context};
use teckel_model::pipeline::*;
use teckel_model::quality as q;
use teckel_model::source::*;
use teckel_model::types::*;
use teckel_model::{Pipeline, TeckelError, TeckelErrorCode};

/// Convert a parsed YAML document into the full domain model pipeline.
pub fn to_pipeline(doc: &yaml::Document) -> Result<Pipeline, TeckelError> {
    let context = to_context(doc)?;

    Ok(Pipeline {
        context,
        metadata: rewrite_pipeline_metadata(doc),
        config: rewrite_config(doc),
        hooks: rewrite_hooks(doc),
        quality: rewrite_quality(doc),
        templates: rewrite_templates(doc),
        exposures: rewrite_exposures(doc),
        streaming_inputs: rewrite_streaming_inputs(doc),
        streaming_outputs: rewrite_streaming_outputs(doc),
        secrets: rewrite_secrets(doc),
    })
}

/// Convert a parsed YAML document into the domain model context (assets only).
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

// ── Pipeline-level rewrites ──────────────────────────────────

fn rewrite_pipeline_metadata(doc: &yaml::Document) -> PipelineMetadata {
    match &doc.pipeline {
        None => PipelineMetadata::default(),
        Some(p) => PipelineMetadata {
            name: p.name.clone(),
            namespace: p.namespace.clone(),
            version: p.version.clone(),
            description: p.description.clone(),
            owner: p.owner.as_ref().map(rewrite_owner),
            tags: p.tags.clone().unwrap_or_default(),
            meta: p.meta.clone().unwrap_or_default(),
            schedule: p.schedule.clone(),
            freshness: p.freshness.clone(),
            links: p
                .links
                .as_ref()
                .map(|ls| {
                    ls.iter()
                        .map(|l| Link {
                            label: l.label.clone(),
                            url: l.url.clone(),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            contacts: p
                .contacts
                .as_ref()
                .map(|cs| {
                    cs.iter()
                        .map(|c| Contact {
                            name: c.name.clone(),
                            email: c.email.clone(),
                            role: c.role.clone(),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            catalog: p.catalog.as_ref().map(|c| CatalogConfig {
                target: c.target.clone(),
                namespace: c.namespace.clone(),
            }),
        },
    }
}

fn rewrite_config(doc: &yaml::Document) -> PipelineConfig {
    match &doc.config {
        None => PipelineConfig::default(),
        Some(c) => PipelineConfig {
            backend: c.backend.clone(),
            cache: c.cache.as_ref().map(|cc| CacheConfig {
                auto_cache_threshold: cc.auto_cache_threshold,
                default_storage_level: cc.default_storage_level.clone(),
            }),
            notifications: c.notifications.as_ref().map(|n| NotificationConfig {
                on_success: n
                    .on_success
                    .as_ref()
                    .map(|ts| ts.iter().map(rewrite_notification_target).collect())
                    .unwrap_or_default(),
                on_failure: n
                    .on_failure
                    .as_ref()
                    .map(|ts| ts.iter().map(rewrite_notification_target).collect())
                    .unwrap_or_default(),
            }),
        },
    }
}

fn rewrite_notification_target(t: &crate::yaml::document::NotificationTargetDef) -> NotificationTarget {
    NotificationTarget {
        channel: t.channel.clone(),
        url: t.url.clone(),
        path: t.path.clone(),
    }
}

fn rewrite_hooks(doc: &yaml::Document) -> Hooks {
    match &doc.hooks {
        None => Hooks::default(),
        Some(h) => Hooks {
            pre_execution: h
                .pre_execution
                .as_ref()
                .map(|hs| {
                    hs.iter()
                        .map(|h| Hook {
                            name: h.name.clone(),
                            command: h.command.clone(),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            post_execution: h
                .post_execution
                .as_ref()
                .map(|hs| {
                    hs.iter()
                        .map(|h| Hook {
                            name: h.name.clone(),
                            command: h.command.clone(),
                        })
                        .collect()
                })
                .unwrap_or_default(),
        },
    }
}

fn rewrite_templates(doc: &yaml::Document) -> Vec<Template> {
    doc.templates
        .as_ref()
        .map(|ts| {
            ts.iter()
                .map(|t| Template {
                    name: t.name.clone(),
                    parameters: t.parameters.clone(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn rewrite_secrets(doc: &yaml::Document) -> BTreeMap<String, SecretKey> {
    doc.secrets
        .as_ref()
        .and_then(|s| s.keys.as_ref())
        .map(|keys| {
            keys.iter()
                .map(|(alias, def)| {
                    (
                        alias.clone(),
                        SecretKey {
                            scope: def.scope.clone(),
                            key: def.key.clone(),
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn rewrite_exposures(doc: &yaml::Document) -> Vec<Exposure> {
    doc.exposures
        .as_ref()
        .map(|es| {
            es.iter()
                .map(|e| Exposure {
                    name: e.name.clone(),
                    exposure_type: e.exposure_type.clone(),
                    description: e.description.clone(),
                    url: e.url.clone(),
                    maturity: e.maturity.clone(),
                    owner: e.owner.as_ref().map(rewrite_owner),
                    depends_on: e.depends_on.clone(),
                    tags: e.tags.clone(),
                    meta: e.meta.clone(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn rewrite_streaming_inputs(doc: &yaml::Document) -> Vec<StreamingInput> {
    doc.streaming_input
        .as_ref()
        .map(|sis| {
            sis.iter()
                .map(|si| StreamingInput {
                    name: si.name.clone(),
                    format: si.format.clone(),
                    path: si.path.clone(),
                    options: si.options.clone(),
                    trigger: si.trigger.clone(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn rewrite_streaming_outputs(doc: &yaml::Document) -> Vec<StreamingOutput> {
    doc.streaming_output
        .as_ref()
        .map(|sos| {
            sos.iter()
                .map(|so| StreamingOutput {
                    name: so.name.clone(),
                    format: so.format.clone(),
                    path: so.path.clone(),
                    options: so.options.clone(),
                    output_mode: so.output_mode.as_deref().map(|m| match m {
                        "update" => OutputMode::Update,
                        "complete" => OutputMode::Complete,
                        _ => OutputMode::Append,
                    }),
                    checkpoint_location: so.checkpoint_location.clone(),
                    trigger: so.trigger.clone(),
                })
                .collect()
        })
        .unwrap_or_default()
}

// ── Quality rewrite ──────────────────────────────────────────

fn rewrite_quality(doc: &yaml::Document) -> Vec<q::QualitySuite> {
    doc.quality
        .as_ref()
        .map(|suites| suites.iter().map(rewrite_quality_suite).collect())
        .unwrap_or_default()
}

fn rewrite_quality_suite(s: &crate::yaml::quality::QualitySuiteDef) -> q::QualitySuite {
    q::QualitySuite {
        suite: s.suite.clone(),
        description: s.description.clone(),
        target: s.target.clone(),
        filter: s.filter.clone(),
        severity: parse_severity(&s.severity),
        checks: s.checks.iter().map(rewrite_quality_check).collect(),
    }
}

fn rewrite_quality_check(c: &crate::yaml::quality::QualityCheckDef) -> q::Check {
    match c.check_type.as_str() {
        "schema" => {
            let (required, forbidden) = match &c.columns {
                Some(crate::yaml::quality::QualityCheckColumns::SchemaColumns {
                    required,
                    forbidden,
                }) => (
                    required.clone().unwrap_or_default(),
                    forbidden.clone().unwrap_or_default(),
                ),
                _ => (vec![], vec![]),
            };
            q::Check::Schema(q::SchemaCheck {
                required_columns: required,
                forbidden_columns: forbidden,
                types: c.types.clone().unwrap_or_default(),
            })
        }
        "completeness" => q::Check::Completeness(q::CompletenessCheck {
            column: c.column.clone().unwrap_or_default(),
            threshold: c.threshold.unwrap_or(1.0),
            severity: c.severity.as_deref().map(parse_severity),
            escalate: c.escalate.as_ref().map(|e| q::EscalateRule {
                threshold: e.threshold,
                severity: parse_severity(&e.severity),
            }),
        }),
        "uniqueness" => {
            let columns = match &c.columns {
                Some(crate::yaml::quality::QualityCheckColumns::List(cols)) => cols.clone(),
                _ => c.column.clone().map(|c| vec![c]).unwrap_or_default(),
            };
            q::Check::Uniqueness(q::UniquenessCheck {
                columns,
                threshold: c.threshold.unwrap_or(1.0),
            })
        }
        "validity" => q::Check::Validity(q::ValidityCheck {
            column: c.column.clone().unwrap_or_default(),
            accepted_values: c.accepted_values.clone(),
            range: c.range.as_ref().map(|r| q::RangeSpec {
                min: r.min,
                max: r.max,
                strict_min: r.strict_min,
                strict_max: r.strict_max,
            }),
            pattern: c.pattern.clone(),
            format: c.format.clone(),
            length_between: c.length_between,
            threshold: c.threshold.unwrap_or(1.0),
        }),
        "statistical" => q::Check::Statistical(q::StatisticalCheck {
            column: c.column.clone().unwrap_or_default(),
            mean: c.mean.as_ref().map(rewrite_bound),
            min: c.min.as_ref().map(rewrite_bound),
            max: c.max.as_ref().map(rewrite_bound),
            sum: c.sum.as_ref().map(rewrite_bound),
            stdev: c.stdev.as_ref().map(rewrite_bound),
            quantiles: c
                .quantiles
                .as_ref()
                .map(|qs| {
                    qs.iter()
                        .map(|(k, v)| (k.clone(), rewrite_bound(v)))
                        .collect()
                })
                .unwrap_or_default(),
        }),
        "volume" => q::Check::Volume(q::VolumeCheck {
            row_count: c.row_count.as_ref().map(rewrite_bound),
            column_count: c.column_count.as_ref().map(rewrite_bound),
        }),
        "freshness" => q::Check::Freshness(q::FreshnessCheck {
            column: c.column.clone().unwrap_or_default(),
            max_age: c.max_age.clone().unwrap_or_default(),
        }),
        "referential" => {
            let (asset, col) = match &c.reference {
                Some(r) => (r.asset.clone(), r.column.clone()),
                None => (String::new(), String::new()),
            };
            q::Check::Referential(q::ReferentialCheck {
                column: c.column.clone().unwrap_or_default(),
                reference_asset: asset,
                reference_column: col,
                threshold: c.threshold.unwrap_or(1.0),
            })
        }
        "crossColumn" => q::Check::CrossColumn(q::CrossColumnCheck {
            condition: c.condition.clone().unwrap_or_default(),
            description: c.description.clone(),
            threshold: c.threshold.unwrap_or(1.0),
        }),
        _ => q::Check::Custom(q::CustomCheck {
            condition: c.condition.clone().unwrap_or_default(),
            description: c.description.clone(),
            threshold: c.threshold.unwrap_or(1.0),
        }),
    }
}

fn rewrite_bound(b: &crate::yaml::quality::BoundSpecDef) -> q::BoundSpec {
    q::BoundSpec {
        min: b.min,
        max: b.max,
        between: b.between,
    }
}

fn parse_severity(s: &str) -> q::Severity {
    match s {
        "warn" => q::Severity::Warn,
        "info" => q::Severity::Info,
        _ => q::Severity::Error,
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
