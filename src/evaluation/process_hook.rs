use either::Either;

use crate::{
    Features,
    evaluation::{
        Context, Effects, SelectableSource, SingleEvaluation, SingleEvaluationError,
        single_evaluation::EvalSignal,
    },
    spec::{ActionEntity, CriterionEntity, EntityId, HookList, RuntimeAction, RuntimeCriterion},
};

#[derive(Debug, thiserror::Error, PartialEq, PartialOrd, Clone)]
pub enum HookFailure {
    #[error("criterion: {_0}")]
    Criterion(SingleEvaluationError),
    #[error("action: {_0}")]
    Action(SingleEvaluationError),
}

pub type FallibleHookResults = Vec<Result<HookResult, HookFailure>>;

// ##########################################################
// ##########################################################

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct HookResult {
    pub(crate) criterion: Option<RuntimeCriterion>,
    // pub(crate) eval_signals: Option<StopEval>,
}

impl HookList {
    #[allow(unused)]
    pub(crate) async fn run_all<Source: SelectableSource>(
        &self,
        ctx: &Context,
        eft: &mut Effects,
        feat: Features,
        fallback: Source,
    ) -> (FallibleHookResults, Option<EvalSignal>) {
        let mut tried = vec![];
        for (crit, act) in self.iter() {
            match execute_hook(ctx, (&eft, &fallback), feat, crit, act).await {
                Err(failure) => tried.push(Err(failure)),
                Ok((actions, criterion, signal)) => {
                    tried.push(Ok(HookResult { criterion }));
                    eft.extend(actions);

                    if let Some(signal) = signal {
                        match &signal {
                            EvalSignal::EndTest(_) => return (tried, Some(signal)),
                        }
                    }
                }
            }
        }
        (tried, None)
    }
}

async fn execute_hook<Source: SelectableSource>(
    settings: &Context,
    outer_fallback: Source,
    features: Features,
    crit: EntityId<CriterionEntity>,
    act: EntityId<ActionEntity>,
) -> Result<
    (
        Vec<RuntimeAction>,
        Option<RuntimeCriterion>,
        Option<EvalSignal>,
    ),
    HookFailure,
> {
    let mut actions = vec![];

    let crit_eval = SingleEvaluation::new(
        settings.entities(),
        crit.cast_id(),
        &outer_fallback,
        features,
    )
    .map_err(SingleEvaluationError::from)
    .map_err(HookFailure::Criterion)?;
    let crit_eval = crit_eval.run_to_end_with_early_return().await;

    let criterion: Either<EvalSignal, RuntimeCriterion> = crit_eval
        .one_specialized()
        .map_err(HookFailure::Criterion)?;

    let eval_signals = match criterion.clone().map_left(Some) {
        Either::Left(eval_signals) => eval_signals,
        Either::Right(criterion) => {
            if criterion.is_fulfilled() {
                let act_eval = SingleEvaluation::new(
                    settings.entities(),
                    act.cast_id(),
                    &outer_fallback,
                    features,
                )
                .map_err(SingleEvaluationError::from)
                .map_err(HookFailure::Action)?;
                let act_eval = act_eval.run_to_end_with_early_return().await;

                let action: Either<EvalSignal, RuntimeAction> =
                    act_eval.one_specialized().map_err(HookFailure::Criterion)?;

                actions.extend(crit_eval.into_actions());
                actions.extend(act_eval.into_actions());
                match action.map_left(Some) {
                    Either::Left(eval_signals) => eval_signals,
                    Either::Right(action) => {
                        actions.push(action);
                        None
                    }
                }
            } else {
                None
            }
        }
    };
    Ok((actions, criterion.right(), eval_signals))
}
