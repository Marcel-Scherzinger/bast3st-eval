use crate::{
    Features, Messages,
    evaluation::{
        Context, Effects, PCategory, PMainTest, SelectableSource, WithEffects,
        process_spec::SpecRunError, single_evaluation::EvalSignal,
    },
    spec::{Category, CategoryHooks},
};

impl PCategory {
    pub async fn new<Fallback: SelectableSource + Clone + 'static>(
        ctx: &Context,
        category: &Category,
        fallback: Fallback,
    ) -> Result<WithEffects<PCategory>, SpecRunError> {
        let mut eft = Effects::default();
        let mut p_tests = vec![];

        let (before_all_tests, sig) = category
            .hooks()
            .before_all_tests()
            .run_all(
                ctx,
                &mut eft,
                Features::PermittedFEAT_CategoryHook,
                &fallback,
            )
            .await;
        match sig {
            Some(EvalSignal::EndTest(_)) | None => {} // not relevant
        }
        // take "current"-level messages for category, don't move this line
        let mut messages: Messages<Category> = eft.take_messages();

        {
            let mut test_futures = vec![];
            {
                let frozen_effects = std::sync::Arc::new(eft.clone());
                for test in category.tests() {
                    let source = (frozen_effects.clone(), fallback.clone());
                    let ctx = ctx.clone();
                    let test = test.clone();
                    test_futures.push(tokio::spawn(async move {
                        PMainTest::new(&ctx, &test, source).await
                    }));
                }
            }
            for fut in test_futures {
                let (test, test_effects) = fut.await??.into_parts();
                p_tests.push(test);
                log::trace!("effects of finished test: {test_effects:?}");
                eft.append(test_effects);
            }
        }
        log::trace!("category effects after test runs: {eft:?}");

        let (after_all_tests, sig) = category
            .hooks()
            .after_all_tests()
            .run_all(
                ctx,
                &mut eft,
                Features::PermittedFEAT_CategoryHook,
                &fallback,
            )
            .await;
        match sig {
            Some(EvalSignal::EndTest(_)) | None => {} // not relevant
        }
        messages.extend(eft.take_messages());

        Ok(WithEffects::new(
            PCategory {
                messages,
                tests: p_tests,
                hooks: CategoryHooks {
                    before_all_tests,
                    after_all_tests,
                },
            },
            eft,
        ))
    }
}
