use std::{borrow::Cow, collections::BTreeMap};

use scratch_test_interpreter::{
    Limits, RunError,
    default_state::{DefaultState, DefaultStateError},
};
use scratch_test_model::{Id, ProjectDoc, attrs::DataId};

use crate::{
    evaluation::{Rundata, SelFal, SelectableSource, TestRundataSource},
    spec::{Array, GeneralTest, MapKey, PrimitiveValue, RuntimeValue, Text},
};

fn setup_initial_lists_and_vars<Hooks>(
    doc: &ProjectDoc,
    state: &mut DefaultState,
    general: &GeneralTest<Hooks>,
) {
    let mut new_id = 0;
    let mut new_id_str = String::new();

    if let Some(initial_vars) = general.initial_variables().as_ref() {
        let doc_vars: BTreeMap<&str, &scratch_test_model::Id> = doc
            .targets()
            .iter()
            .flat_map(|target| target.variables().values())
            .map(|x| (x.0.name().as_str(), x.0.id()))
            .collect();

        for (name, value) in initial_vars.iter() {
            let id: scratch_test_model::Id = if let Some(id) = doc_vars.get(name.as_str()).cloned()
            {
                id.clone()
            } else {
                loop {
                    new_id += 1;
                    new_id_str = format!("id{new_id}");
                    if (doc_vars
                        .values()
                        .find(|x| ****x == *new_id_str.as_str())
                        .is_none())
                    {
                        break;
                    }
                }
                new_id_str.into()
            };
            state
                .variables_mut()
                .insert(DataId::new(name.clone(), id), value.clone().into_svalue());
        }
    }

    if let Some(initial_lists) = general.initial_lists().as_ref() {
        let doc_lists: BTreeMap<&str, &scratch_test_model::Id> = doc
            .targets()
            .iter()
            .flat_map(|target| target.lists().values())
            .map(|x| (x.0.name().as_str(), x.0.id()))
            .collect();
        for (name, value) in initial_lists.iter() {
            let id: scratch_test_model::Id = if let Some(id) = doc_lists.get(name.as_str()).cloned()
            {
                id.clone()
            } else {
                loop {
                    new_id += 1;
                    new_id_str = format!("id{new_id}");
                    if (doc_lists
                        .values()
                        .find(|x| ****x == *new_id_str.as_str())
                        .is_none())
                    {
                        break;
                    }
                }
                new_id_str.into()
            };
            state.lists_mut().insert(
                DataId::new(name.clone(), id),
                value.iter().map(|x| x.clone().into_svalue()).collect(),
            );
        }
    }
}
const MAX_LIST_LENGTH: i64 = 100;

// TODO: Deal with Limits and RunError

fn run_single_test<'f, Hooks, Fallback: SelectableSource + Clone>(
    doc: &ProjectDoc,
    initial_block: &Id,
    general: &GeneralTest<Hooks>,
    fallback: Cow<'f, Fallback>,
) -> (
    TestRundataSource<'f, Fallback>,
    DefaultState,
    Option<RunError<DefaultStateError>>,
    Limits,
) {
    let input: Option<&Vec<PrimitiveValue>> = general.input().as_ref();
    let predefined_randoms = general.predefined_randoms().as_ref();
    let random_generation = general.random_generation();

    let interp = scratch_test_interpreter::Interpreter::new_restrictive();

    let mut state = DefaultState::from_doc(doc, MAX_LIST_LENGTH);
    if let Some(input) = input {
        let answers: Vec<std::sync::Arc<str>> = input
            .iter()
            .map(|x| x.to_owned().into_text().into())
            .collect();
        state.set_answers(answers);
    }

    setup_initial_lists_and_vars(doc, &mut state, general);

    if let Some(predefined_randoms) = predefined_randoms {
        state.set_predefined_randoms(predefined_randoms.iter().cloned().collect());
    }
    let report = interp.run(doc, state, initial_block);
    let (state, error, limits) = report.take_parts();

    let rundata: TestRundataSource<'f, Fallback> = TestRundataSource::new(
        state
            .answer_inputs()
            .iter()
            .map(|x| RuntimeValue::from(Text::from(x.to_string())))
            .collect::<Vec<_>>(),
        state
            .output_lines()
            .map(|x| RuntimeValue::from(Text::from(x.to_string())))
            .collect::<Vec<_>>(),
        state
            .used_randoms()
            .map(|x| RuntimeValue::from(*x))
            .collect::<Vec<_>>(),
        state
            .lists()
            .iter()
            .map(|x| {
                (
                    MapKey::Str(x.0.name().to_string().into()),
                    RuntimeValue::from(Array::from(x.1.clone())),
                )
            })
            .collect::<BTreeMap<MapKey, RuntimeValue>>(),
        state
            .variables()
            .iter()
            .map(|x| {
                (
                    MapKey::Str(x.0.name().to_string().into()),
                    RuntimeValue::from(x.1.clone()),
                )
            })
            .collect::<BTreeMap<MapKey, RuntimeValue>>(),
        fallback,
    );
    (rundata, state, error, limits)
}
