use std::fmt::Debug;

use itertools::Itertools;
use scratch_test_model::{Id, ProjectDoc};
use tokio::task::JoinError;

use crate::{
    Features, LogPfx, Messages,
    evaluation::{
        Context, Effects, FatalRunError, PCategory, PSpec, ParamData, SelectableSource, WithNotice,
        initial_block::{InitialBlockAmbiguity, find_initial_block},
        report::{ReportBuilder, WithSpec},
        single_evaluation::EvalSignal,
    },
    spec::{Bast3StSpec, MapKey, Numeric, PrimitiveValue, RealMapping, RuntimeValue, SpecHooks},
};

#[derive(Debug, thiserror::Error)]
pub enum SpecRunError {
    #[error("join: {_0}")]
    Join(#[from] JoinError),
    #[error("run: {_0}")]
    FatalRun(#[from] WithNotice<FatalRunError>),
    #[error("initial-block: {_0}")]
    InitialBlock(#[from] InitialBlockAmbiguity),
}

fn get_blockcount_map(doc: &ProjectDoc) -> (RealMapping, (RuntimeValue, RealMapping)) {
    let base_counts = doc.ids_with_opcodes().map(|(_, unit)| unit).counts();
    let total = RuntimeValue::from(Numeric::Int(
        base_counts
            .values()
            .sum::<usize>()
            .try_into()
            .unwrap_or(i64::MAX),
    ));
    let by_opcode: RealMapping = base_counts
        .iter()
        .map(|(key, count)| {
            (
                MapKey::Str(key.to_string().into()),
                RuntimeValue::Prim(PrimitiveValue::Number(Numeric::Int(
                    (*count).try_into().unwrap_or(i64::MAX),
                ))),
            )
        })
        .collect::<RealMapping>()
        .with_default(Ok(0.into()));

    let blockcount = RealMapping::from_iter(vec![
        ("total".into(), total.clone()),
        ("opcode".into(), RuntimeValue::from(by_opcode.clone())),
    ]);
    (blockcount, (total, by_opcode))
}

impl PSpec {
    pub async fn new<'a, Fallback: SelectableSource + Clone + 'static>(
        report_builder: ReportBuilder<'a, Fallback, WithSpec>,
        doc: ProjectDoc,
        initial_block: Option<Id>,
    ) -> Result<PSpec, SpecRunError> {
        let initial_block = initial_block
            .map(Ok)
            .unwrap_or_else(|| find_initial_block(&doc).cloned())?;
        let (ctx, (log_pfx, param_my, fallback, spec)) = report_builder.build(doc, initial_block);
        Self::inner_new(&ctx, log_pfx, spec, param_my, fallback).await
    }

    async fn inner_new<Fallback: SelectableSource + Clone + 'static>(
        ctx: &Context,
        log_pfx: impl Into<LogPfx>,
        spec: &Bast3StSpec,
        param_my: Option<impl Into<RealMapping>>,
        fallback: Fallback,
    ) -> Result<PSpec, SpecRunError> {
        let log_pfx = log_pfx.into();
        let (blockcount, _) = get_blockcount_map(ctx.doc());
        let doc =
            RealMapping::from_iter(vec![("blockcount".into(), RuntimeValue::from(blockcount))]);
        let all =
            RealMapping::from_iter(vec![("doc".into(), doc.into())].into_iter().chain(
                param_my.map(|param_my| ("my".into(), RuntimeValue::from(param_my.into()))),
            ));

        let stats = ParamData::from(all);
        Self::new_with_effects(ctx, log_pfx, spec, (stats, fallback)).await
    }

    async fn new_with_effects<Fallback: SelectableSource + Clone + 'static>(
        ctx: &Context,
        log_pfx: LogPfx,
        spec: &Bast3StSpec,
        fallback: Fallback,
    ) -> Result<PSpec, SpecRunError> {
        let mut categories = vec![];
        let mut eft = Effects::default();

        let (before_all_categories, sig) = spec
            .hooks()
            .before_all_categories()
            .run_all(
                ctx,
                log_pfx.join("before-all-cat"),
                &mut eft,
                Features::PermittedFEAT_SpecHook,
                &fallback,
            )
            .await;
        match sig {
            Some(EvalSignal::EndTest(_)) | None => {} // not relevant
        }

        // take "current"-level messages for spec, don't move this line
        let mut messages: Messages<Bast3StSpec> = eft.take_messages();
        {
            let mut cat_futures = vec![];
            {
                let frozen_effects = std::sync::Arc::new(eft.clone());
                let log_pfx = log_pfx.join("cat");
                for (index, cat) in spec.categories().iter().enumerate() {
                    let cat = cat.clone();
                    let source = (frozen_effects.clone(), fallback.clone());
                    let ctx = ctx.clone();
                    let log_pfx = log_pfx.join(index);
                    cat_futures.push(tokio::spawn(async move {
                        PCategory::new(&ctx, log_pfx, &cat, source).await
                    }));
                }
            }
            for cat_future in cat_futures {
                let (cat, cat_effects) = cat_future.await??.into_parts();
                eft.append(cat_effects);
                categories.push(cat);
            }
        }

        let (after_all_categories, sig) = spec
            .hooks()
            .after_all_categories()
            .run_all(
                ctx,
                log_pfx.join("after-all-cat"),
                &mut eft,
                Features::PermittedFEAT_SpecHook,
                fallback,
            )
            .await;
        match sig {
            Some(EvalSignal::EndTest(_)) | None => {} // not relevant
        }
        messages.extend(eft.take_messages());

        let (flags, notice) = eft.take_rest();
        Ok(PSpec {
            title: spec.title().clone().into(),
            description: spec.description().clone().map(|x| x.into()),
            messages,
            categories,
            hooks: SpecHooks {
                before_all_categories,
                after_all_categories,
            },
            flags,
            notice,
        })
    }
}
