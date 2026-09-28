use derive_getters::Getters;
use derive_more::From;
use serde::{Deserialize, Serialize};

use crate::{
    Messages,
    evaluation::{
        FlagData, JustFailTestRunError, Rundata, SingleEvaluationError,
        process_hook::FallibleHookResults,
    },
    spec::{
        AlternativeTest, AlternativeTestHooks, Bast3StSpec, Category, CategoryHooks,
        EndThisTestAction, MainTest, MainTestHooks, NoticeAction, RuntimeCriterion, SpecHooks,
        Text,
    },
};

#[derive(Debug, PartialEq, PartialOrd, Clone, Getters, Serialize, Deserialize)]
pub struct PSpec {
    pub(crate) title: Text,
    pub(crate) description: Option<Text>,
    pub(crate) messages: Messages<Bast3StSpec>,
    pub(crate) categories: Vec<PCategory>,
    pub(crate) hooks: SpecHooks<FallibleHookResults>,
    pub(crate) flags: FlagData,
    pub(crate) notice: Vec<NoticeAction>,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Getters, Serialize, Deserialize)]
pub struct PCategory {
    pub(crate) title: Text,
    pub(crate) messages: Messages<Category>,
    pub(crate) tests: Vec<PMainTest>,
    pub(crate) hooks: CategoryHooks<FallibleHookResults>,
}
#[derive(Debug, PartialEq, PartialOrd, Clone, Getters, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, PartialOrd, Clone, Getters, Serialize, Deserialize)]
pub struct PAlternativeTest {
    pub(crate) messages: Messages<AlternativeTest>,
    pub(crate) general: PGeneralTest<AlternativeTestHooks<FallibleHookResults>>,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Getters, Serialize, Deserialize)]
pub struct PGeneralTest<Hooks> {
    pub(crate) title: Text,
    pub(crate) status: ProcessedTestStatus,
    pub(crate) hooks: Hooks,
    pub(crate) data: Option<Rundata>,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, From, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProcessedTestStatus {
    EndedByAction(EndThisTestAction),
    Criterion(RuntimeCriterion),
    Eval(SingleEvaluationError),
    JustFailTestRun(JustFailTestRunError),
}
