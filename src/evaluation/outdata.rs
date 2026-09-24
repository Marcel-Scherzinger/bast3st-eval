use derive_more::From;

use crate::{
    evaluation::{
        JustFailTestRunError, Rundata, SingleEvaluationError, process_hook::FallibleHookResults,
    },
    spec::{
        AlternativeTestHooks, EndThisTestAction, MainTestHooks, ProcessedAction, RuntimeCriterion,
    },
};

pub struct PSpec {
    pub(crate) categories: Vec<PCategory>,
}

pub struct PCategory {
    pub(crate) tests: Vec<PMainTest>,
}
pub struct PMainTest {
    pub(crate) general: PGeneralTest<MainTestHooks<FallibleHookResults>>,
    pub(crate) actions: Vec<ProcessedAction>,
    pub(crate) tried_alternatives: Vec<PAlternativeTest>,
}

impl PMainTest {
    pub fn final_status(&self) -> &ProcessedTestStatus {
        self.tried_alternatives
            .last()
            .map_or(&self.general.status, |alt| &alt.general.status)
    }
}

pub struct PAlternativeTest {
    pub(crate) general: PGeneralTest<AlternativeTestHooks<FallibleHookResults>>,
}

pub struct PGeneralTest<Hooks> {
    pub(crate) status: ProcessedTestStatus,
    pub(crate) hooks: Hooks,
    pub(crate) data: Option<Rundata>,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, From)]
pub enum ProcessedTestStatus {
    EndedByAction(EndThisTestAction),
    Criterion(RuntimeCriterion),
    Eval(SingleEvaluationError),
    JustFailTestRun(JustFailTestRunError),
}
