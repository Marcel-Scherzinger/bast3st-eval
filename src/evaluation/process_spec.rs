use itertools::Itertools;
use scratch_test_model::ProjectDoc;
use tokio::task::JoinError;

use crate::{
    Features, Messages,
    evaluation::{
        Context, Effects, FatalRunError, PCategory, PSpec, ParamData, SelectableSource,
        WithEffects, single_evaluation::EvalSignal,
    },
    spec::{Bast3StSpec, MapKey, Numeric, PrimitiveValue, RealMapping, RuntimeValue, SpecHooks},
};

#[derive(Debug, thiserror::Error)]
pub enum SpecRunError {
    #[error("join: {_0}")]
    Join(#[from] JoinError),
    #[error("run: {_0}")]
    FatalRun(#[from] FatalRunError),
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
        .collect();

    let blockcount = RealMapping::from_iter(vec![
        ("total".into(), total.clone()),
        ("opcode".into(), RuntimeValue::from(by_opcode.clone())),
    ]);
    (blockcount, (total, by_opcode))
}

impl PSpec {
    pub async fn new<Fallback: SelectableSource + Clone + 'static>(
        ctx: &Context,
        spec: &Bast3StSpec,
        param_my: impl Into<RealMapping>,
        fallback: Fallback,
    ) -> Result<WithEffects<PSpec>, SpecRunError> {
        let (blockcount, _) = get_blockcount_map(ctx.doc());
        let doc = RealMapping::from_iter(vec![("blockcount".into(), blockcount.into())]);
        let param_my = param_my.into();
        let all = RealMapping::from_iter(vec![
            ("doc".into(), doc.into()),
            ("my".into(), param_my.clone().into()),
        ]);

        let stats = ParamData::from(all);
        Self::new_with_effects(ctx, spec, (stats, fallback)).await
    }

    pub async fn new_with_effects<Fallback: SelectableSource + Clone + 'static>(
        ctx: &Context,
        spec: &Bast3StSpec,
        fallback: Fallback,
    ) -> Result<WithEffects<PSpec>, SpecRunError> {
        let mut categories = vec![];
        let mut eft = Effects::default();

        let (before_all_categories, sig) = spec
            .hooks()
            .before_all_categories()
            .run_all(ctx, &mut eft, Features::PermittedFEAT_SpecHook, &fallback)
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
                for cat in spec.categories() {
                    let cat = cat.clone();
                    let source = (frozen_effects.clone(), fallback.clone());
                    let ctx = ctx.clone();
                    cat_futures.push(tokio::spawn(async move {
                        PCategory::new(&ctx, &cat, source).await
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
            .run_all(ctx, &mut eft, Features::PermittedFEAT_SpecHook, fallback)
            .await;
        match sig {
            Some(EvalSignal::EndTest(_)) | None => {} // not relevant
        }
        messages.extend(eft.take_messages());

        Ok(WithEffects::new(
            PSpec {
                messages,
                categories,
                hooks: SpecHooks {
                    before_all_categories,
                    after_all_categories,
                },
            },
            eft,
        ))
    }
}
