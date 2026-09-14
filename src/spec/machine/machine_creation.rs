use std::borrow::Cow;

use crate::spec::{
    Entity, EntityId, PrimitiveValue, ValueReference,
    machine::{Machine, MissingValue, Param, cast_param},
    runtime::{
        CheapBorrowFromAny, ClarifiedCerrMerging, PossibleRuntimeValue, RuntimeAny, RuntimeValue,
        SelectableTaskRequest, Selector, SpecializeFrom,
    },
};

pub trait MachineConstruction<E, RuntimeT>: PossibleRuntimeValue<RuntimeT> {
    fn query_cow<R: ClarifiedCerrMerging + 'static>(
        &self,
        closure: impl for<'a> FnOnce(Cow<'a, RuntimeT>) -> Machine<R> + 'static,
    ) -> Machine<R>;

    fn query<R: ClarifiedCerrMerging + 'static>(
        &self,
        closure: impl for<'a> FnOnce(RuntimeT) -> Machine<R> + 'static,
    ) -> Machine<R>
    where
        RuntimeT: Clone,
    {
        self.query_cow(|cow| closure(cow.into_owned()))
    }

    fn query_ref<R: ClarifiedCerrMerging + 'static>(
        &self,
        closure: impl for<'a> FnOnce(&'a RuntimeT) -> Machine<R> + 'static,
    ) -> Machine<R>
    where
        RuntimeT: Clone + CheapBorrowFromAny,
    {
        self.query_cow(|cow| closure(&cow))
    }
}

impl<E, RuntimeT> MachineConstruction<E, RuntimeT> for EntityId<E>
where
    EntityId<E>: PossibleRuntimeValue<RuntimeT>,
    RuntimeT: Clone + SpecializeFrom,
    Entity: From<E>,
{
    fn query_cow<R: ClarifiedCerrMerging + 'static>(
        &self,
        closure: impl for<'a> FnOnce(Cow<'a, RuntimeT>) -> Machine<R> + 'static,
    ) -> Machine<R> {
        let id: EntityId<Entity> = self.cast_id();
        let on_push =
            move |delivered: Param<'_>| Machine::from_res(cast_param(delivered).map(closure));
        Machine::from_pushable(id, Box::new(on_push))
    }
}

impl<RuntimeT> MachineConstruction<ValueReference, RuntimeT> for ValueReference
where
    ValueReference: PossibleRuntimeValue<RuntimeT>,
    RuntimeT: Clone + SpecializeFrom + /*SpecializeFrom<Text> +*/ 'static,
    EntityId<Entity>: PossibleRuntimeValue<RuntimeT>,
{
    fn query_cow<R: ClarifiedCerrMerging + 'static>(
        &self,
        closure: impl for<'a> FnOnce(Cow<'a, RuntimeT>) -> Machine<R> + 'static,
    ) -> Machine<R> {
        match self {
            Self::EntityId(id) => id.cast_id::<Entity>().query_cow(closure),
            Self::LitString(text) => {
                let any = RuntimeAny::Value(RuntimeValue::Prim(PrimitiveValue::Str(text.clone())));
                let run = RuntimeT::specialize_from(&any).map(|c| c.into_owned());
                let on_push = move |_: Param<'_>| {
                    let converted: Cow<'_, RuntimeT> = match run {
                        Ok(o) => Cow::Owned(o),
                        Err(e) => return e.into(),
                    };
                    closure(converted)
                };
                Machine::from_pushable(MissingValue::NoValueNeeded, Box::new(on_push))
            }
        }
    }
}

impl<RuntimeT> MachineConstruction<Selector, RuntimeT> for Selector
where
    Selector: PossibleRuntimeValue<RuntimeT>,
    RuntimeT: SpecializeFrom,
{
    fn query_cow<R: ClarifiedCerrMerging + 'static>(
        &self,
        closure: impl for<'a> FnOnce(Cow<'a, RuntimeT>) -> Machine<R> + 'static,
    ) -> Machine<R> {
        let id = self.clone();
        let on_push = move |param: Param<'_>| Machine::from_res(cast_param(param).map(closure));
        Machine::from_pushable(self.clone(), Box::new(on_push))
    }
}
impl<RuntimeT, Task: Into<SelectableTaskRequest>> MachineConstruction<Task, RuntimeT> for Task
where
    Task: PossibleRuntimeValue<RuntimeT> + Clone,
    RuntimeT: SpecializeFrom,
{
    fn query_cow<R: ClarifiedCerrMerging + 'static>(
        &self,
        closure: impl for<'a> FnOnce(Cow<'a, RuntimeT>) -> Machine<R> + 'static,
    ) -> Machine<R> {
        Machine::from_task(
            self.clone(),
            Box::new(move |param: Param<'_>| Machine::from_res(cast_param(param).map(closure))),
        )
    }
}

pub trait MachineConstructionN<Closure, Extra, Ret> {
    fn query_n(self, closure: Closure) -> Machine<Ret>;
}

impl<C, E1, E2, R1, R2, Ret: ClarifiedCerrMerging + 'static, T1: ToOwned, T2: ToOwned>
    MachineConstructionN<C, (E1, E2, R1, R2), Ret> for (&T1, &T2)
where
    C: FnOnce(R1, R2) -> Machine<Ret> + 'static,
    R1: Clone + 'static,
    R2: Clone + 'static,
    T1::Owned: MachineConstruction<E1, R1> + 'static,
    T2::Owned: MachineConstruction<E2, R2> + 'static,
{
    fn query_n(self, closure: C) -> Machine<Ret> {
        let t1 = self.0.to_owned();
        let t2 = self.1.to_owned();
        t1.query::<Ret>(move |r1: R1| t2.query::<Ret>(move |r2: R2| closure(r1, r2)))
    }
}
