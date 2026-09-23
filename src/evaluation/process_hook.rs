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

#[derive(Debug, thiserror::Error)]
pub enum HookFailure {
    #[error("criterion: {_0}")]
    Criterion(SingleEvaluationError),
    #[error("action: {_0}")]
    Action(SingleEvaluationError),
}

#[derive(Debug)]
pub struct HookResult {
    pub(crate) actions: Vec<RuntimeAction>,
}
impl HookResult {
    pub async fn from_pre_hook<Fallback: Clone + SelectableSource>(
        context: &Context<'_, '_>,
        outer_fallback: &Fallback,
        crit: EntityId<CriterionEntity>,
        act: EntityId<ActionEntity>,
    ) -> Result<HookResult, HookFailure> {
        execute_hook(
            context,
            outer_fallback,
            Features::PermittedFEAT_PreTestHook,
            crit,
            act,
        )
        .await
    }
    pub async fn from_post_hook<Fallback: Clone + SelectableSource>(
        context: &Context<'_, '_>,
        outer_fallback: &Fallback,
        crit: EntityId<CriterionEntity>,
        act: EntityId<ActionEntity>,
    ) -> Result<HookResult, HookFailure> {
        execute_hook(
            context,
            outer_fallback,
            Features::PermittedFEAT_PostTestHook,
            crit,
            act,
        )
        .await
    }
}

/// (Adds [`Features::END_THIS_TEST`] to given base features)
pub(crate) async fn execute_hook<'p, 'e, 'f, Fallback: Clone + SelectableSource>(
    settings: &Context<'p, 'e>,
    outer_fallback: &'f Fallback,
    base_features: Features,
    crit: EntityId<CriterionEntity>,
    act: EntityId<ActionEntity>,
) -> Result<HookResult, HookFailure> {
    let base_features = base_features | Features::END_THIS_TEST;
    let mut actions = vec![];

    let c_eval = SingleEvaluation::new(
        settings.entities(),
        crit.cast_id(),
        outer_fallback,
        base_features,
    )
    .map_err(SingleEvaluationError::from)
    .map_err(HookFailure::Criterion)?;
    let c_eval = c_eval.run_to_end_with_early_return().await;

    let criterion: Either<StopEval, RuntimeCriterion> =
        c_eval.one_specialized().map_err(HookFailure::Criterion)?;

    if criterion.is_right_and(|criterion| criterion.is_fulfilled()) {
        let a_eval = SingleEvaluation::new(
            settings.entities(),
            act.cast_id(),
            outer_fallback,
            base_features,
        )
        .map_err(SingleEvaluationError::from)
        .map_err(HookFailure::Action)?;
        let a_eval = a_eval.run_to_end_with_early_return().await;

        let action: Either<StopEval, RuntimeAction> =
            a_eval.one_specialized().map_err(HookFailure::Criterion)?;

        actions.extend(c_eval.into_actions());
        actions.extend(a_eval.into_actions());
        if let Either::Right(action) = action {
            actions.push(action);
        }
    }

    Ok(HookResult { actions })
}
