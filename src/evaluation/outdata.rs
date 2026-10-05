use std::borrow::Cow;

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
    pub(crate) general: PGeneralTest<MainTestHooks<FallibleHookResults>>,
    #[serde(skip_serializing_if = "is_default")]
    pub(crate) tried_alternatives: Vec<PAlternativeTest>,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Getters, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FinalMainTestStatus<'a> {
    pub(crate) main_status: Cow<'a, ProcessedTestStatus>,
    pub(crate) alternative_status: Option<Cow<'a, ProcessedTestStatus>>,
}

impl PMainTest {
    pub fn final_status<'a>(&'a self) -> FinalMainTestStatus<'a> {
        let mstatus = self.general().status();
        let astatus = self.tried_alternatives.last().map(|x| x.general().status());
        FinalMainTestStatus {
            main_status: Cow::Borrowed(mstatus),
            alternative_status: astatus.map(Cow::Borrowed),
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Getters, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PAlternativeTest {
    #[serde(skip_serializing_if = "is_default")]
    pub(crate) messages: Messages<AlternativeTest>,
    pub(crate) general: PGeneralTest<AlternativeTestHooks<FallibleHookResults>>,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Getters, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PGeneralTest<Hooks: Default + PartialEq> {
    pub(crate) title: Text,
    pub(crate) is_passed: bool,
    pub(crate) status: ProcessedTestStatus,
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

impl PSpec {
    pub fn take_notice(&mut self) -> Vec<NoticeAction> {
        std::mem::take(&mut self.notice)
    }
}
