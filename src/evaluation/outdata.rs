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

fn is_default<T: Default + PartialEq>(val: &T) -> bool {
    val == &T::default()
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Getters, Serialize, Deserialize)]
pub struct PSpec {
    pub(crate) title: Text,
    #[serde(skip_serializing_if = "is_default")]
    pub(crate) description: Option<Text>,
    #[serde(skip_serializing_if = "is_default")]
    pub(crate) messages: Messages<Bast3StSpec>,
    #[serde(skip_serializing_if = "is_default")]
    pub(crate) categories: Vec<PCategory>,
    #[serde(skip_serializing_if = "is_default")]
    pub(crate) hooks: SpecHooks<FallibleHookResults>,
    #[serde(skip_serializing_if = "is_default")]
    pub(crate) flags: FlagData,
    #[serde(skip_serializing_if = "is_default")]
    pub(crate) notice: Vec<NoticeAction>,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Getters, Serialize, Deserialize)]
pub struct PCategory {
    pub(crate) title: Text,
    #[serde(skip_serializing_if = "is_default")]
    pub(crate) messages: Messages<Category>,
    #[serde(skip_serializing_if = "is_default")]
    pub(crate) tests: Vec<PMainTest>,
    #[serde(skip_serializing_if = "is_default")]
    pub(crate) hooks: CategoryHooks<FallibleHookResults>,
}
#[derive(Debug, PartialEq, PartialOrd, Clone, Getters, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PMainTest {
    #[serde(skip_serializing_if = "is_default")]
    pub(crate) messages: Messages<MainTest>,
    pub(crate) general:
        PGeneralTest<MainTestProcessedTestStatus, MainTestHooks<FallibleHookResults>>,
    #[serde(skip_serializing_if = "is_default")]
    pub(crate) tried_alternatives: Vec<PAlternativeTest>,
}

impl PMainTest {
    pub fn final_status(&self) -> &MainTestProcessedTestStatus {
        &self.general.status
        //self.tried_alternatives
        //.last()
        //.map_or(&self.general.status, |alt| &alt.general.status)
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Getters, Serialize, Deserialize)]
pub struct PAlternativeTest {
    #[serde(skip_serializing_if = "is_default")]
    pub(crate) messages: Messages<AlternativeTest>,
    pub(crate) general:
        PGeneralTest<ProcessedTestStatus, AlternativeTestHooks<FallibleHookResults>>,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Getters, Serialize, Deserialize)]
pub struct PGeneralTest<Status, Hooks: Default + PartialEq> {
    pub(crate) title: Text,
    #[serde(flatten)]
    pub(crate) status: Status,
    #[serde(skip_serializing_if = "is_default")]
    pub(crate) hooks: Hooks,
    #[serde(skip_serializing_if = "is_default")]
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
#[derive(Debug, PartialEq, PartialOrd, Clone, From, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct MainTestProcessedTestStatus {
    pub(crate) is_successful: bool,
    pub(crate) main_status: ProcessedTestStatus,
    #[serde(skip_serializing_if = "is_default")]
    pub(crate) alternative_status: Option<ProcessedTestStatus>,
}

impl PSpec {
    pub fn take_notice(&mut self) -> Vec<NoticeAction> {
        std::mem::take(&mut self.notice)
    }
}
