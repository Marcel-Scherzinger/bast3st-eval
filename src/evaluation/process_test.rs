use either::Either;

use crate::{
    Features,
    evaluation::{
        Context, Effects, PAlternativeTest, PGeneralTest, PMainTest, ProcessedTestStatus,
        SelectableSource, WithEffects,
        single_evaluation::EvalSignal,
        testrun2selectable::{ActualTestResultStatus, FatalRunError, run_actual_test},
    },
    spec::{AlternativeTest, AlternativeTestHooks, EndThisTestMode, MainTest, MainTestHooks},
};

impl ProcessedTestStatus {
    pub fn is_successful(&self) -> bool {
        match self {
            Self::JustFailTestRun(_) | Self::Eval(_) => false,
            Self::Criterion(c) => c.is_fulfilled(),
            Self::EndedByAction(e) => match e.mode() {
                EndThisTestMode::Pass => true,
                EndThisTestMode::Fail => false,
            },
        }
    }
    pub fn maybe_overwrite_with_signal(&mut self, signal: Option<EvalSignal>) -> &mut Self {
        #[allow(clippy::collapsible_match)]
        match signal {
            Some(EvalSignal::EndTest(end)) => {
                if self.proceedable() {
                    *self = end.into();
                }
            }
            None => {}
        }

        self
    }
    /// If the test can be proceeded in other ways by e. g. running alternative tests.
    /// If this is false, the test has an explicit result and shouldn't be processed further.
    pub fn proceedable(&self) -> bool {
        !matches!(self, ProcessedTestStatus::EndedByAction(_))
    }
}

pub type MainTestFailure = FatalRunError;

fn process_test_status(
    status: ActualTestResultStatus,
    effects: &mut Effects,
) -> Result<ProcessedTestStatus, FatalRunError> {
    Ok(match status {
        Either::Left((run_actions, decisison)) => {
            effects.extend(run_actions);
            match decisison {
                Ok(Either::Left(EvalSignal::EndTest(end_test_action))) => {
                    ProcessedTestStatus::EndedByAction(end_test_action)
                }
                Ok(Either::Right(criterion)) => ProcessedTestStatus::Criterion(criterion),
                Err(eval_error) => ProcessedTestStatus::Eval(eval_error),
            }
        }
        Either::Right(run_failed) => ProcessedTestStatus::JustFailTestRun(run_failed?),
    })
}

impl PMainTest {
    #[allow(unused)]
    pub async fn new<'p, 'e, Fallback: SelectableSource>(
        ctx: &Context<'p, 'e>,
        test: &MainTest,
        fallback: Fallback,
    ) -> Result<WithEffects<PMainTest>, MainTestFailure> {
        process_main_test(ctx, test, fallback).await
    }
}

async fn process_main_test<'p, 'e, Fallback: SelectableSource>(
    ctx: &Context<'p, 'e>,
    test: &MainTest,
    fallback: Fallback,
) -> Result<WithEffects<PMainTest>, MainTestFailure> {
    let hooks = test.general().hooks();

    let mut eft = Effects::default();

    // #########################################
    // ### Hooks: before_main
    // #########################################
    let (before_main, sig) = hooks
        .before_main()
        .run_all(
            ctx,
            &mut eft,
            Features::PermittedFEAT_PreTestHook,
            &fallback,
        )
        .await;

    // #########################################
    // ### Actual test if hooks send no signal
    // #########################################
    let (mut main_status, main_rundata) = if let Some(sig) = sig {
        match sig {
            EvalSignal::EndTest(end_this_test) => (end_this_test.into(), None),
        }
    } else {
        let main_result = run_actual_test(
            ctx,
            test.as_ref(),
            (&eft, &fallback),
            Features::PermittedFEAT_PassTestCrit,
        )
        .await;
        let status = process_test_status(main_result.status, &mut eft)?;
        (status, Some(main_result.testdata))
    };
    // #############################################
    // ### Initiate alternatives if test proceedable
    // #############################################
    let mut before_alternatives = Default::default();
    let tried_alternatives = if main_status.proceedable() && !main_status.is_successful() {
        // TODO: adjust documentation about this hook
        let sig;
        (before_alternatives, sig) = hooks
            .before_alternatives()
            .run_all(
                ctx,
                &mut eft,
                Features::PermittedFEAT_PostTestHook,
                (&main_rundata, &fallback),
            )
            .await;
        // ###############################################
        // ### Actual alternatives if hooks send no signal
        // ###############################################
        if main_status.maybe_overwrite_with_signal(sig).proceedable() {
            try_alternative_tests_of_main(ctx, &mut eft, test.alternative_tests(), &fallback)
                .await?
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    // #########################################
    // ### Hooks: after_alternatives
    // #########################################
    let mut after_alternatives = Default::default();
    if main_status.proceedable() {
        let sig;
        (after_alternatives, sig) = hooks
            .after_alternatives()
            .run_all(
                ctx,
                &mut eft,
                Features::PermittedFEAT_PostTestHook,
                (&main_rundata, &fallback),
            )
            .await;
        main_status.maybe_overwrite_with_signal(sig);
    }

    let out = PMainTest {
        general: PGeneralTest {
            status: main_status,
            data: main_rundata,
            hooks: MainTestHooks {
                before_main,
                before_alternatives,
                after_alternatives,
            },
        },
        tried_alternatives,
    };

    Ok(WithEffects::new(out, eft))
}

async fn try_alternative_tests_of_main<'p, 'e, Source: SelectableSource>(
    ctx: &Context<'p, 'e>,
    eft: &mut Effects,
    alternatives: impl IntoIterator<Item = &AlternativeTest>,
    fallback: Source,
) -> Result<Vec<PAlternativeTest>, MainTestFailure> {
    let mut tried_alternatives = vec![];
    for alternative in alternatives {
        let alt_hooks = alternative.general().hooks();

        // #########################################
        // ### Hooks: before_alt
        // #########################################
        let (before_alt, sig) = alt_hooks
            .before_alt()
            .run_all(ctx, eft, Features::PermittedFEAT_PreTestHook, &fallback)
            .await;

        // #########################################
        // ### Actual test if hooks send no signal
        // #########################################
        let (mut alt_status, alt_rundata) = if let Some(sig) = sig {
            match sig {
                EvalSignal::EndTest(end) => (end.into(), None),
            }
        } else {
            let alternative_result = run_actual_test(
                ctx,
                alternative.as_ref(),
                (&eft, &fallback),
                Features::PermittedFEAT_PassTestCrit,
            )
            .await;
            let alternative_status = process_test_status(alternative_result.status, eft)?;
            (alternative_status, Some(alternative_result.testdata))
        };

        // #########################################
        // ### Hooks: after_alt if test proceedable
        // #########################################
        let mut after_alt = Default::default();
        if alt_status.proceedable() {
            let sig;
            (after_alt, sig) = alt_hooks
                .after_alt()
                .run_all(
                    ctx,
                    eft,
                    Features::PermittedFEAT_PostTestHook,
                    (&alt_rundata, &fallback),
                )
                .await;
            alt_status.maybe_overwrite_with_signal(sig);
        }

        let no_other_alternatives_needed = alt_status.is_successful();
        tried_alternatives.push(PAlternativeTest {
            general: PGeneralTest {
                status: alt_status,
                data: alt_rundata,
                hooks: AlternativeTestHooks {
                    before_alt,
                    after_alt,
                },
            },
        });
        if no_other_alternatives_needed {
            break;
        }
    }
    Ok(tried_alternatives)
}
