use tokio::task::JoinError;

use crate::{
    Features, Messages,
    evaluation::{
        Context, Effects, FatalRunError, PCategory, PSpec, SelectableSource, WithEffects,
        single_evaluation::EvalSignal,
    },
    spec::{Bast3StSpec, SpecHooks},
};

#[derive(Debug, thiserror::Error)]
pub enum SpecRunError {
    #[error("join: {_0}")]
    Join(#[from] JoinError),
    #[error("run: {_0}")]
    FatalRun(#[from] FatalRunError),
}

impl PSpec {
    async fn new_with_effects<Fallback: SelectableSource + Clone + 'static>(
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
