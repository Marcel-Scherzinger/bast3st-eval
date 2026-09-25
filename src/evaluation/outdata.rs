use derive_more::From;

use crate::{
    Messages,
    evaluation::{
        JustFailTestRunError, Rundata, SingleEvaluationError, process_hook::FallibleHookResults,
    },
    spec::{
        AlternativeTest, AlternativeTestHooks, Bast3StSpec, Category, CategoryHooks,
        EndThisTestAction, MainTest, MainTestHooks, RuntimeCriterion, SpecHooks,
    },
};

pub struct PSpec {
    pub(crate) messages: Messages<Bast3StSpec>,
    pub(crate) categories: Vec<PCategory>,
    pub(crate) hooks: SpecHooks<FallibleHookResults>,
}

pub struct PCategory {
    pub(crate) messages: Messages<Category>,
    pub(crate) tests: Vec<PMainTest>,
    pub(crate) hooks: CategoryHooks<FallibleHookResults>,
}
pub struct PMainTest {
    pub(crate) messages: Messages<MainTest>,
    pub(crate) general: PGeneralTest<MainTestHooks<FallibleHookResults>>,
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
    pub(crate) messages: Messages<AlternativeTest>,
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
