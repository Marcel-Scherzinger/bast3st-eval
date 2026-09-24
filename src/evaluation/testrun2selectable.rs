use std::{borrow::Cow, collections::BTreeMap};

use either::Either;
use scratch_test_interpreter::{
    Limits, RunError,
    default_state::{DefaultState, DefaultStateError},
    error::UserError,
};
use scratch_test_model::{Id, ProjectDoc, attrs::DataId};

use crate::{
    Features,
    evaluation::{
        Context, Rundata, SelectableSource, SingleEvaluation, SingleEvaluationError,
        single_evaluation::EvalSignal,
    },
    spec::{
        Array, EndThisTestAction, Entity, EntityId, GeneralTest, MapKey, PrimitiveValue,
        RuntimeAction, RuntimeCriterion, RuntimeValue, Text,
    },
};

#[derive(Debug, thiserror::Error, Clone, PartialEq, PartialOrd)]
pub enum FatalRunError {
    #[error("runerror-file: {_0}")]
    File(#[from] scratch_test_interpreter::error::InvalidFileError),
    #[error("runerror-internal: {_0}")]
    Internal(#[from] scratch_test_interpreter::error::InternalError),
    #[error("runerror-unsupported: {_0}")]
    Unsupported(#[from] scratch_test_interpreter::error::UnsupportedError),
}

/// An error that indicates that something didn't work out as expected with the
/// users submission, but it is not as severe as a [`FatalRunError`] and it will
/// be counted as a failed test without checking the criterion at all.
#[derive(Debug, thiserror::Error, PartialEq, PartialOrd, Clone)]
pub enum JustFailTestRunError {
    #[error("runerror-user: {_0}")]
    User(#[from] UserError),
    #[error("runerror-limit: {_0}")]
    Limit(#[from] scratch_test_interpreter::error::LimitError),
    #[error("runerror-other: {_0}")]
    State(#[from] DefaultStateError),
}

pub type ActualTestResultEval = (
    Vec<RuntimeAction>,
    Result<Either<EvalSignal, RuntimeCriterion>, SingleEvaluationError>,
);
pub type ActualTestResultStatus =
    Either<ActualTestResultEval, Result<JustFailTestRunError, FatalRunError>>;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct ActualTestResult {
    pub(crate) status: ActualTestResultStatus,
    pub(crate) testdata: Rundata,
}

impl ActualTestResult {
    pub async fn from_run<Hooks, Source: SelectableSource>(
        settings: &Context<'_, '_>,
        test: &GeneralTest<Hooks>,
        source: Source,
        features: Features,
    ) -> Self {
        run_actual_test(settings, test, source, features).await
    }
}

pub(crate) async fn run_actual_test<Hooks, Source: SelectableSource>(
    settings: &Context<'_, '_>,
    test: &GeneralTest<Hooks>,
    fallback: Source,
    features: Features,
) -> ActualTestResult {
    let (testdata, _state, error, _limits) =
        run_single_test_for_selectable(settings, settings.doc(), settings.initial_block(), test);

    if let Some(error) = error {
        // some errors like infinite loops can be counted as immediate test failure,
        // but this won't cause an exception
        let maybe_error: Result<Option<JustFailTestRunError>, FatalRunError> = match error {
            RunError::TerminatedByControlStop => Ok(None),
            RunError::User(
                err @ (UserError::InfiniteLoopWithoutBodyNeverStops
                | UserError::ConditionLoopWithoutBodyNeverStops
                | UserError::WaitUntilFalseNeverTerminatesInSingleThreadded),
            ) => Ok(Some(err.into())),
            RunError::File(err) => Err(err.into()),
            RunError::Internal(err) => Err(err.into()),
            RunError::Unsupported(err) => Err(err.into()),
            RunError::Limit(err) => Ok(Some(err.into())),
            RunError::State(err) => Ok(Some(err.into())),
        };
        if let Some(error) = maybe_error.transpose() {
            return ActualTestResult {
                status: Either::Right(error),
                testdata,
            };
        }
    }

    let criterion = test.criterion();
    match SingleEvaluation::new(
        settings.entities(),
        criterion.cast_id(),
        &(&testdata, &fallback),
        features,
    ) {
        Ok(eval) => {
            let eval = eval.run_to_end_with_early_return().await;
            let value: Result<Either<EvalSignal, RuntimeCriterion>, SingleEvaluationError> =
                eval.one_specialized();
            ActualTestResult {
                status: Either::Left((eval.into_actions(), value)),
                testdata,
            }
        }
        Err(err) => ActualTestResult {
            status: Either::Left((Default::default(), Err(err.into()))),
            testdata,
        },
    }
}

fn run_single_test_for_selectable<Hooks>(
    settings: &Context,
    doc: &ProjectDoc,
    initial_block: &Id,
    general: &GeneralTest<Hooks>,
) -> (
    Rundata,
    DefaultState,
    Option<RunError<DefaultStateError>>,
    Limits,
) {
    let input: Option<&Vec<PrimitiveValue>> = general.input().as_ref();
    let predefined_randoms = general.predefined_randoms().as_ref();
    let random_generation = general.random_generation();

    let interp = scratch_test_interpreter::Interpreter::new_restrictive();

    let mut state = DefaultState::from_doc(doc, settings.max_list_length().into());
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

    let rundata = Rundata::new(
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
    );
    (rundata, state, error, limits)
}

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
