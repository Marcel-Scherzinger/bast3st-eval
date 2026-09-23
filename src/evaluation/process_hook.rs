use std::collections::BTreeMap;

use derive_getters::Getters;

use crate::{
    Features,
    evaluation::{Context, SelectableSource, SingleEvaluation, SingleEvaluationError},
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
    pub async fn from_hook_run<Fallback: Clone + SelectableSource>(
        context: &Context<'_, '_>,
        outer_fallback: &Fallback,
        crit: EntityId<CriterionEntity>,
        act: EntityId<ActionEntity>,
    ) -> Result<HookResult, HookFailure> {
        execute_hook(context, outer_fallback, crit, act).await
    }
}

pub(crate) async fn execute_hook<'p, 'e, 'f, Fallback: Clone + SelectableSource>(
    settings: &Context<'p, 'e>,
    outer_fallback: &'f Fallback,
    crit: EntityId<CriterionEntity>,
    act: EntityId<ActionEntity>,
) -> Result<HookResult, HookFailure> {
    let mut actions = vec![];

    let c_eval = SingleEvaluation::new(
        settings.entities(),
        crit.cast_id(),
        outer_fallback,
        Features::PermittedFEAT_PreTestHook,
    )
    .map_err(SingleEvaluationError::from)
    .map_err(HookFailure::Criterion)?;
    let c_eval = c_eval.run_to_end().await;

    let criterion: RuntimeCriterion = c_eval.one_specialized().map_err(HookFailure::Criterion)?;

    if criterion.is_fulfilled() {
        let a_eval = SingleEvaluation::new(
            settings.entities(),
            act.cast_id(),
            outer_fallback,
            Features::PermittedFEAT_PreTestHook,
        )
        .map_err(SingleEvaluationError::from)
        .map_err(HookFailure::Action)?;
        let a_eval = a_eval.run_to_end().await;

        let action: RuntimeAction = a_eval.one_specialized().map_err(HookFailure::Criterion)?;

        actions.extend(c_eval.into_actions());
        actions.extend(a_eval.into_actions());
        actions.push(action);
    }

    Ok(HookResult { actions })
}
