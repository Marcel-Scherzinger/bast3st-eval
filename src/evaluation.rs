mod features;
mod selection;

pub use features::Features;
pub use selection::SelectableSource;

use std::{borrow::Cow, collections::BTreeMap};

use either::Either;

use crate::{
    catchable::cerr,
    spec::{
        Array, Entity, EntityId, FatalError, Machine, MissingValue, NetworkRequest,
        NetworkResponse, Numeric, RuntimeAction, RuntimeAny, RuntimeValue, Selector,
        UnfinishedMachine,
    },
};

pub struct SelectableData {
    output: RuntimeAny,
}

impl SelectableData {
    pub fn new(output: Array) -> Self {
        Self {
            output: RuntimeAny::Value(output.into()),
        }
    }
}

impl SelectableSource for SelectableData {
    async fn request<'a>(
        &'a self,
        selector: &Selector,
        feat: Features,
    ) -> Result<&'a RuntimeAny, FatalError> {
        log::info!("Requested:  {selector:?}");
        Ok(&self.output)
    }
}

pub struct SingleEvaluation<'e, 's, S> {
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
}

impl<'e, 's, S: SelectableSource> SingleEvaluation<'e, 's, S> {
    pub fn new(
        entities: &'e BTreeMap<EntityId, Entity>,
        entry_point: EntityId,
        selectable_data: &'s S,
        allowed_features: Features,
    ) -> Option<Self> {
        if entities.contains_key(&entry_point) {
            Some(Self {
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
            })
        } else {
            None
        }
    }
    pub fn all_values(&self) -> &BTreeMap<EntityId, RuntimeAny> {
        &self.values
    }
    pub fn value(&self) -> Result<Option<&RuntimeAny>, FatalError> {
        self.further_steps_and_no_fatal_err.clone()?;
        Ok(self.values.get(&self.entry_point))
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

    pub async fn run_to_end(&mut self) -> Result<(), FatalError> {
        while self.run_step().await? {}
        Ok(())
    }

    async fn get_selector_value(
        &mut self,
        selector: &Selector,
    ) -> Result<&'s RuntimeAny, FatalError> {
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
                            m.call(Ok(val))
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
