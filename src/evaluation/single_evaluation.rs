use std::{borrow::Cow, collections::BTreeMap};

use either::Either;

use crate::{
    Features,
    catchable::cerr,
    evaluation::{RequiredFeatures, SelectableSource},
    spec::{
        Array, Entity, EntityId, FatalError, Machine, MissingValue, NetworkRequest,
        NetworkResponse, Numeric, RuntimeAction, RuntimeAny, RuntimeValue, Selector,
        SpecializeFrom, UnfinishedMachine,
    },
};

pub struct EvaluationRunning(());
pub struct EvaluationFinished(());

pub struct SingleEvaluation<'e, 's, S, EvalStatus> {
    entities: &'e BTreeMap<EntityId, Entity>,
    selectable: &'s S,
    entry_point: EntityId,
    allowed_features: Features,
    values: BTreeMap<EntityId, RuntimeAny>,
    machines: BTreeMap<EntityId, Machine<RuntimeAny>>,
    id_stack: Vec<EntityId>,
    actions: Vec<RuntimeAction>,
    further_steps_and_no_fatal_err: Result<bool, FatalError>,
    used_features: Features,
    _phantom: std::marker::PhantomData<EvalStatus>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, thiserror::Error)]
#[error("entry point id {_0} not in entity set")]
pub struct EntryPointMissing(EntityId);

#[derive(Debug, PartialEq, PartialOrd, Clone, thiserror::Error)]
pub enum SingleEvaluationError {
    #[error("missing entity/final value: {_0}")]
    EntryPoint(#[from] EntryPointMissing),
    #[error("catchable: {_0:?}")]
    Cerr(cerr),
    #[error("fatal: {_0:?}")]
    Fatal(#[from] FatalError),
}

impl<'e, 's, S: SelectableSource> SingleEvaluation<'e, 's, S, EvaluationFinished> {
    pub fn one_specialized<T: SpecializeFrom>(&self) -> Result<T, SingleEvaluationError> {
        match self.maybe_specialized_value::<T>() {
            Ok(Some(normal)) => Ok(normal.into_owned()),
            // This situation should never occur as `run_to_end` *should* compute
            // the value (or create an error), so no extra variant is used.
            Ok(None) => Err(SingleEvaluationError::EntryPoint(EntryPointMissing(
                self.entry_point,
            ))),
            Err(Either::Left(catchable)) => Err(SingleEvaluationError::Cerr(catchable)),
            Err(Either::Right(fatal)) => Err(SingleEvaluationError::Fatal(fatal)),
        }
    }
}

impl<'e, 's, S: SelectableSource, X> SingleEvaluation<'e, 's, S, X> {
    pub fn maybe_specialized_value<T: SpecializeFrom>(
        &self,
    ) -> Result<Option<Cow<'_, T>>, Either<cerr, FatalError>> {
        if let Some(val) = self.value().map_err(Either::Right)? {
            let specialized = T::specialize_from(Cow::Borrowed(val));
            specialized.map_err(Either::Left).map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn value(&self) -> Result<Option<&RuntimeAny>, FatalError> {
        self.further_steps_and_no_fatal_err.clone()?;
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
                further_steps_and_no_fatal_err: Ok(true),
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
    pub async fn run_step(&mut self) -> Result<bool, FatalError> {
        self.further_steps_and_no_fatal_err.clone()?;
        self.further_steps_and_no_fatal_err = self._run_step().await;
        self.further_steps_and_no_fatal_err.clone()
    }

    // This function doesn't mutate the state so the produced error
    // should be saved by another part of the program
    fn check_features(&self) -> Result<(), FatalError> {
        if !self.allowed_features.contains(self.used_features)
            && self.further_steps_and_no_fatal_err.is_ok()
        {
            Err(FatalError::FeatureMissmatch {
                required: self.used_features,
                provided: self.allowed_features,
            })
        } else {
            Ok(())
        }
    }

    pub async fn run_to_end(mut self) -> SingleEvaluation<'e, 's, S, EvaluationFinished> {
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
            further_steps_and_no_fatal_err: self.further_steps_and_no_fatal_err,
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

    async fn _run_step(&mut self) -> Result<bool, FatalError> {
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
                            self.actions.extend(meta.into_actions());
                            self.check_features()?;
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
                                    return Err(FatalError::CyclicIdReferences(*id));
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
