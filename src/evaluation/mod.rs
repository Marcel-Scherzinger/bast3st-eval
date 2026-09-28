mod context;
mod features;
pub mod initial_block;
mod outdata;
mod process_cat;
mod process_hook;
mod process_spec;
mod process_test;
mod report;
mod selectable;
mod single_evaluation;
mod testrun2selectable;

pub use context::Context;
pub use features::{Features, RequiredFeatures};
pub use outdata::*;
pub use report::{EvalLimitations, ReportBuilder};
pub use selectable::*;
pub use single_evaluation::{
    AllowNetData, EntryPointMissing, SingleEvaluation, SingleEvaluationError,
};

pub use testrun2selectable::{
    ActualTestResult, ActualTestResultEval, FatalRunError, JustFailTestRunError,
};

pub use process_hook::{HookFailure, HookResult};
