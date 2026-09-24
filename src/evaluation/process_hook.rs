use std::collections::BTreeMap;

use derive_getters::Getters;
use either::Either;

use crate::{
    Features,
    evaluation::{
        Context, SelectableSource, SingleEvaluation, SingleEvaluationError,
        single_evaluation::StopEval,
    },
    spec::{
        ActionEntity, CriterionEntity, EntityId, RuntimeAction, RuntimeCriterion, RuntimeValue,
        Text,
    },
};

#[derive(Debug, thiserror::Error, PartialEq, PartialOrd, Clone)]
pub enum HookFailure {
    #[error("criterion: {_0}")]
    Criterion(SingleEvaluationError),
    #[error("action: {_0}")]
    Action(SingleEvaluationError),
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct HookResult {
    pub(crate) actions: Vec<RuntimeAction>,
    pub(crate) eval_signals: Option<StopEval>,
}
impl HookResult {
    pub async fn from_pre_hook<Source: SelectableSource>(
        context: &Context<'_, '_>,
        source: Source,
        crit: EntityId<CriterionEntity>,
        act: EntityId<ActionEntity>,
    ) -> Result<HookResult, HookFailure> {
        execute_hook(
            context,
            source,
            Features::PermittedFEAT_PreTestHook,
            crit,
            act,
        )
        .await
    }
    pub async fn from_post_hook<Source: SelectableSource>(
        context: &Context<'_, '_>,
        source: Source,
        crit: EntityId<CriterionEntity>,
        act: EntityId<ActionEntity>,
    ) -> Result<HookResult, HookFailure> {
        execute_hook(
            context,
            source,
            Features::PermittedFEAT_PostTestHook,
            crit,
            act,
        )
        .await
    }
    pub fn into_actions(self) -> Vec<RuntimeAction> {
        self.actions
    }
}

pub(crate) async fn execute_hook<'p, 'e, Source: SelectableSource>(
    settings: &Context<'p, 'e>,
    outer_fallback: Source,
    features: Features,
    crit: EntityId<CriterionEntity>,
    act: EntityId<ActionEntity>,
) -> Result<HookResult, HookFailure> {
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

    let criterion: Either<StopEval, RuntimeCriterion> = crit_eval
        .one_specialized()
        .map_err(HookFailure::Criterion)?;

    let eval_signals = match criterion.map_left(Some) {
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

                let action: Either<StopEval, RuntimeAction> =
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
    Ok(HookResult {
        actions,
        eval_signals,
    })
}
