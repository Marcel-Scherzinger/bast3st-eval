use std::{borrow::Cow, collections::BTreeMap, fmt::Display};

use derive_more::{Display, From};
use either::Either;

use crate::{
    Features,
    catchable::cerr,
    evaluation::{RequiredFeatures, SelectableSource},
    spec::{
        Array, EndThisTestAction, EndThisTestMode, Entity, EntityId, FatalError, Machine,
        MissingValue, NetworkRequest, NetworkResponse, Numeric, RuntimeAction, RuntimeAny,
        RuntimeValue, Selector, SpecializeFrom, UnfinishedMachine,
    },
};

pub struct EvaluationRunning(());
pub struct EvaluationFinished(());
pub struct EvaluationFinishedOrCancelled(());

#[derive(Debug)]
pub struct SingleEvaluation<'e, 's, S, EvalStatus> {
    entities: &'e BTreeMap<EntityId, Entity>,
    selectable: &'s S,
    entry_point: EntityId,
    allowed_features: Features,
    values: BTreeMap<EntityId, RuntimeAny>,
    machines: BTreeMap<EntityId, Machine<RuntimeAny>>,
    id_stack: Vec<EntityId>,
    actions: Vec<RuntimeAction>,
    further_steps_and_no_early_termination: Result<bool, AnyEvalTermination>,
    used_features: Features,
    _phantom: std::marker::PhantomData<EvalStatus>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, thiserror::Error)]
#[error("entry point id {_0} not in entity set")]
pub struct EntryPointMissing(EntityId);

#[derive(Debug, PartialEq, PartialOrd, Clone, thiserror::Error)]
pub enum SingleEvaluationError<Extra: Display = std::convert::Infallible> {
    #[error("missing entity/final value: {_0}")]
    EntryPoint(#[from] EntryPointMissing),
    #[error("catchable: {_0:?}")]
    Cerr(cerr),
    #[error("fatal: {_0:?}")]
    Fatal(#[from] FatalError),
    #[error("other: {_0}")]
    Extra(Extra),
}

impl<'e, 's, S: SelectableSource> SingleEvaluation<'e, 's, S, EvaluationFinished> {
    pub fn one_specialized<T: SpecializeFrom>(&self) -> Result<T, SingleEvaluationError<StopEval>> {
        match self.maybe_specialized_value::<T>() {
            Ok(Some(normal)) => Ok(normal.into_owned()),
            // This situation should never occur as `run_to_end` *should* compute
            // the value (or create an error), so no extra variant is used.
            Ok(None) => Err(SingleEvaluationError::EntryPoint(EntryPointMissing(
                self.entry_point,
            ))),
            Err(Either::Left(catchable)) => Err(SingleEvaluationError::Cerr(catchable)),
            Err(Either::Right(AnyEvalTermination::Fatal(fatal))) => {
                Err(SingleEvaluationError::Fatal(fatal))
            }
            Err(Either::Right(AnyEvalTermination::End(end))) => {
                Err(SingleEvaluationError::Extra(end))
            }
        }
    }
}

impl<'e, 's, S: SelectableSource> SingleEvaluation<'e, 's, S, EvaluationFinishedOrCancelled> {
    pub fn one_specialized<T: SpecializeFrom>(
        &self,
    ) -> Result<Either<StopEval, T>, SingleEvaluationError> {
        match self.maybe_specialized_value::<T>() {
            Ok(Some(normal)) => Ok(Either::Right(normal.into_owned())),
            // This situation should never occur as `run_to_end` *should* compute
            // the value (or create an error), so no extra variant is used.
            Ok(None) => Err(SingleEvaluationError::EntryPoint(EntryPointMissing(
                self.entry_point,
            ))),
            Err(Either::Left(catchable)) => Err(SingleEvaluationError::Cerr(catchable)),
            Err(Either::Right(AnyEvalTermination::Fatal(fatal))) => {
                Err(SingleEvaluationError::Fatal(fatal))
            }
            Err(Either::Right(AnyEvalTermination::End(end))) => Ok(Either::Left(end)),
        }
    }
}

impl<'e, 's, S: SelectableSource, X> SingleEvaluation<'e, 's, S, X> {
    pub fn maybe_specialized_value<T: SpecializeFrom>(
        &self,
    ) -> Result<Option<Cow<'_, T>>, Either<cerr, AnyEvalTermination>> {
        if let Some(val) = self.value().map_err(Either::Right)? {
            let specialized = T::specialize_from(Cow::Borrowed(val));
            specialized.map_err(Either::Left).map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn value(&self) -> Result<Option<&RuntimeAny>, AnyEvalTermination> {
        self.further_steps_and_no_early_termination.clone()?;
        Ok(self.values.get(&self.entry_point))
    }
    pub fn into_actions(self) -> Vec<RuntimeAction> {
        self.actions
    }

    pub fn all_values(&self) -> &BTreeMap<EntityId, RuntimeAny> {
        &self.values
    }
}

impl<'e, 's, S: SelectableSource> SingleEvaluation<'e, 's, S, EvaluationRunning> {
    pub fn new(
        entities: &'e BTreeMap<EntityId, Entity>,
        entry_point: EntityId,
        selectable_data: &'s S,
        allowed_features: Features,
    ) -> Result<Self, EntryPointMissing> {
        if entities.contains_key(&entry_point) {
            Ok(Self {
                entities,
                entry_point,
                selectable: selectable_data,
                values: Default::default(),
                machines: Default::default(),
                id_stack: vec![entry_point],
                further_steps_and_no_early_termination: Ok(true),
                actions: Default::default(),
                used_features: Features::empty(),
                allowed_features,
                _phantom: Default::default(),
            })
        } else {
            Err(EntryPointMissing(entry_point))
        }
    }

    /// Run the next step if there is one and return (as bool) if
    /// there was a step to execute.
    ///
    /// - `Ok(true)`: There was something to do
    /// - `Ok(false)`: Computation finished
    /// - `Err(fatal)`: Something really bad happened
    pub async fn run_step(&mut self) -> Result<bool, AnyEvalTermination> {
        self.further_steps_and_no_early_termination.clone()?;
        self.further_steps_and_no_early_termination = self._run_step().await;
        self.further_steps_and_no_early_termination.clone()
    }

    // This function doesn't mutate the state so the produced error
    // should be saved by another part of the program
    fn check_features(&self) -> Result<(), FatalError> {
        if !self.allowed_features.contains(self.used_features)
            && self.further_steps_and_no_early_termination.is_ok()
        {
            Err(FatalError::FeatureMissmatch {
                required: self.used_features,
                provided: self.allowed_features,
            })
        } else {
            Ok(())
        }
    }

    pub async fn run_to_end_without_early_return(
        mut self,
    ) -> SingleEvaluation<'e, 's, S, EvaluationFinished> {
        while self.run_step().await.is_ok_and(|x| x) {}
        SingleEvaluation {
            entities: self.entities,
            selectable: self.selectable,
            entry_point: self.entry_point,
            allowed_features: self.allowed_features,
            values: self.values,
            machines: self.machines,
            id_stack: self.id_stack,
            actions: self.actions,
            further_steps_and_no_early_termination: self.further_steps_and_no_early_termination,
            used_features: self.used_features,
            _phantom: Default::default(),
        }
    }

    pub async fn run_to_end_with_early_return(
        mut self,
    ) -> SingleEvaluation<'e, 's, S, EvaluationFinishedOrCancelled> {
        while self.run_step().await.is_ok_and(|x| x) {}
        SingleEvaluation {
            entities: self.entities,
            selectable: self.selectable,
            entry_point: self.entry_point,
            allowed_features: self.allowed_features,
            values: self.values,
            machines: self.machines,
            id_stack: self.id_stack,
            actions: self.actions,
            further_steps_and_no_early_termination: self.further_steps_and_no_early_termination,
            used_features: self.used_features,
            _phantom: Default::default(),
        }
    }
    async fn get_selector_value(
        &mut self,
        selector: &Selector,
    ) -> Result<Cow<'s, RuntimeAny>, FatalError> {
        self.selectable
            .request(selector, self.allowed_features)
            .await
    }
    async fn run_network_task(
        &mut self,
        task: NetworkRequest,
    ) -> Result<Result<NetworkResponse, cerr>, FatalError> {
        todo!()
    }

    async fn _run_step(&mut self) -> Result<bool, AnyEvalTermination> {
        if let Some(current) = self.id_stack.last().cloned() {
            if self.values.contains_key(&current) {
                self.id_stack.pop();
                return Ok(true);
            }

            let entity = self
                .entities
                .get(&current)
                .ok_or(FatalError::EntityNotFound(current))?;
            log::trace!("Run step for entity {current}: {entity:?}");

            let machine: Machine<RuntimeAny> = self
                .machines
                .remove(&current)
                .unwrap_or_else(|| entity.machine());
            match machine.big_red_and_btn() {
                Either::Left(finished) => {
                    let value = match finished? {
                        Ok((finished, meta)) => {
                            self.used_features |= meta.features();

                            let mut return_early = Ok(());
                            for action in meta.into_actions() {
                                if let RuntimeAction::EndThisTest {
                                    mode,
                                    ref explaination,
                                } = action
                                    && return_early.is_ok()
                                {
                                    return_early = Err(StopEval::EndTest(
                                        mode.action_with(explaination.clone()),
                                    ));
                                }
                                self.actions.push(action);
                            }
                            self.check_features()?;
                            return_early?;
                            finished
                        }
                        Err(err) => RuntimeAny::Catchable(err),
                    };
                    self.values.insert(current, value);
                }
                Either::Right(UnfinishedMachine::Missing(m)) => {
                    let missing_value = m.missing_value();
                    let new_machine = match missing_value {
                        MissingValue::NoValueNeeded => m.call(Ok(&RuntimeAny::Value(0.into()))),
                        MissingValue::EntityId(id) => {
                            if let Some(delivered) = self.values.get(id) {
                                m.call(Ok(delivered))
                            } else {
                                if self.id_stack.contains(id) {
                                    Err(FatalError::CyclicIdReferences(*id))?;
                                }

                                self.id_stack.push(*id);
                                m.delay()
                            }
                        }
                        MissingValue::Selector(selector) => {
                            self.used_features |= selector.required_features();
                            self.check_features()?;
                            let val = self.get_selector_value(selector).await?;
                            m.call(Ok(val.as_ref()))
                        }
                    };
                    self.machines.insert(current, new_machine);
                }
                Either::Right(UnfinishedMachine::RegexTask(m)) => {
                    let pattern = regex::Regex::new(m.request.requested_pattern())
                        .map_err(|_| cerr::regex_syntax);
                    let new_machine = (m.closure)(Cow::Owned(pattern));
                    self.machines.insert(current, new_machine);
                }
                Either::Right(UnfinishedMachine::NetworkTask(m)) => {
                    let response = self.run_network_task(m.request).await?;
                    let new_machine = (m.closure)(Cow::Owned(response));
                    self.machines.insert(current, new_machine);
                }
            }

            return Ok(true);
        }
        Ok(false)
    }
}

#[derive(Debug, PartialEq, From, Clone)]
pub enum AnyEvalTermination {
    Fatal(FatalError),
    End(StopEval),
}

#[derive(Debug, Display, PartialEq, From, Clone, PartialOrd)]
pub enum StopEval {
    #[display("end-test: early return using {_0:?}")]
    EndTest(EndThisTestAction),
}
