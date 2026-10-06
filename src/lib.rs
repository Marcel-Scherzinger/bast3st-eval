#![doc = include_str!("../README.md")]
//!
//! ## [`ReportBuilder`](evaluation::ReportBuilder)
//!
//! ## Input format
//! - [`Bast3StSpec`](spec::Bast3StSpec)
//!     - [`SpecHooks`](spec::SpecHooks)
//! - [`Category`](spec::Category)
//!     - [`CategoryHooks`](spec::CategoryHooks)
//! - [`MainTest`](spec::MainTest)
//!     - [`GeneralTest`](spec::GeneralTest)
//!     - [`MainTestHooks`](spec::MainTestHooks)
//! - [`AlternativeTest`](spec::AlternativeTest)
//!     - [`GeneralTest`](spec::GeneralTest)
//!     - [`AlternativeTestHooks`](spec::AlternativeTestHooks)
//!
//! ### Entities
//! - [`EntityId`](spec::EntityId)
//! - [`Entity`](spec::Entity)
//! - [`CriterionEntity`](spec::CriterionEntity)
//! - [`ActionEntity`](spec::ActionEntity)
//! - [`ValueEntity`](spec::ValueEntity)
//! - [`ValueReference`](spec::ValueReference)
//!
//! ## Runtime(-selectable) types
//! - Runtime types
//!     - [`RuntimeAny`](spec::RuntimeAny)
//!     - [`RuntimeValue`](spec::RuntimeValue)
//!     - [`RuntimeAction`](spec::RuntimeAction)
//!     - [`PrimitiveValue`](spec::PrimitiveValue)
//!     - [`Text`](spec::Text)
//!     - [`MappingOrArray`](spec::MappingOrArray)
//!     - [`RealMapping`](spec::RealMapping)
//!     - [`Array`](spec::Array)
//! - Selectable types
//!     - [`MapKey`](spec::MapKey)
//!     - [`PrimitiveIntoText`](spec::PrimitiveIntoText)
//! - Errors
//!     - [`cerr`]
//!     - [`FatalError`](spec::FatalError)
//!
//! ## General types
//! - [`RandomGeneration`](spec::RandomGeneration)
//! - [`HookList`](spec::HookList)
//!
//!
//! ## Actions
//! - [`ProcessedAction`](spec::ProcessedAction)
//! - [`EndThisTestAction`](spec::EndThisTestAction)
//!     - [`EndThisTestMode`](spec::EndThisTestMode)
//! - [`SetFlagAction`](spec::SetFlagAction)
//!     - [`SetFlagMode`](spec::SetFlagMode)
//! - [`SendMsgAction`](spec::SendMsgAction)
//! - [`NoticeAction`](spec::NoticeAction)
//!
//! ## Criteria
//! - [`RuntimeCriterion`](spec::RuntimeCriterion)
//!     - [`InnerRuntimeCriterion`](spec::InnerRuntimeCriterion)
//!     - [`SpecialCritVariant`](spec::SpecialCritVariant)
//!
//! ## Task-request & -response types
//! - [`NetworkRequest`](spec::NetworkRequest)
//!     - [`InnerNetworkRequest`](spec::InnerNetworkRequest)
//!     - [`NetworkResponse`](spec::NetworkResponse)
//!     - [`NetworkMethod`](spec::NetworkMethod)
//! - [`CompiledRegex`](spec::CompiledRegex)
//!
//! [`PropertyPerspective`](spec::PropertyPerspective)
//!
//! ## Messages
//! - [`Message`]
//! - [`Messages`]
//! - [`MessageSendingLevel`](spec::MessageSendingLevel)
//! - [`MessageSeverity`]
//!
//! ## Evaluation
//! - [`Machine`](spec::Machine)
//!     - [`UnfinishedMachine`](spec::UnfinishedMachine)
//!     - [`MaybeEval`](spec::MaybeEval)
//!     - [`MissingValue`](spec::MissingValue)
//! - [`Selector`](spec::Selector)
//!     - [`Coremapping`](spec::Coremapping)
//! - [`Context`](evaluation::Context)
//! - [`Features`]
//! - [`EvalLimitations`](evaluation::EvalLimitations)
//! - [`ActualTestResult`](evaluation::ActualTestResult)
//! - [`SingleEvaluation`](evaluation::SingleEvaluation)
//!
//! - Hooks:
//!     - [`HookResult`](evaluation::HookResult)
//!     - [`HookFailure`](evaluation::HookFailure)
//!
//! ## Output format
//! - [`PSpec`](evaluation::PSpec)
//! - [`PCategory`](evaluation::PCategory)
//! - [`PMainTest`](evaluation::PMainTest)
//!     - [`PGeneralTest`](evaluation::PGeneralTest)
//!         - [`ProcessedTestStatus`](evaluation::ProcessedTestStatus)
//!         - [`Rundata`](evaluation::Rundata)
//! - [`PAlternativeTest`](evaluation::PAlternativeTest)
//!     - [`PGeneralTest`](evaluation::PGeneralTest)
//!         - [`ProcessedTestStatus`](evaluation::ProcessedTestStatus)
//!         - [`Rundata`](evaluation::Rundata)
//! - Errors
//!     - [`JustFailTestRunError`](evaluation::JustFailTestRunError)
//!     - [`EntryPointMissing`](evaluation::EntryPointMissing)
//!     - [`SingleEvaluationError`](evaluation::SingleEvaluationError)
//!     - [`SpecRunError`]
//!     - [`FatalRunError`](evaluation::FatalRunError)
//!
//!
//! ## Auxiliary Types
//! - [`WithNotice`](evaluation::WithNotice)
//! - [`WithEffects`](evaluation::WithEffects)
//! - [`Effects`](evaluation::Effects)
//! - [`MappingData`](evaluation::MappingData)
//!     - [`SourceFlag`](evaluation::SourceFlag)
//!     - [`SourceParam`](evaluation::SourceParam)
//! - [`SelFal`](evaluation::SelFal)
//! - [`FinalMainTestStatus`](evaluation::FinalMainTestStatus)

pub(crate) mod catchable;
pub mod evaluation;
mod helpers;
mod log_pfx;
pub(crate) mod messages;
pub mod spec;

pub use catchable::cerr;
pub use evaluation::Features;
pub use evaluation::SpecRunError;
pub use helpers::ProgramDocError;
pub use log_pfx::LogPfx;
pub use messages::{Message, MessageSeverity, Messages};

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, sync::Arc};

    use reqwest::Url;

    use crate::{
        Features,
        evaluation::{ReportBuilder, SelectableSource, SingleEvaluation},
        spec::{self, Array, RuntimeAny, RuntimeValue},
    };

    pub struct DummyData {
        data: RuntimeAny,
    }
    impl SelectableSource for DummyData {
        async fn request<'a>(
            &'a self,
            _selector: &spec::Selector,
            _allowed_features: Features,
        ) -> Result<Cow<'a, spec::RuntimeAny>, spec::FatalError> {
            Ok(Cow::Borrowed(&self.data))
        }
    }

    #[test]
    fn test_urls() {
        let url: Url = "https://example.org:80/abc".parse().unwrap();
        let x = url.host_str().unwrap();
        assert_eq!("example.org", x);
        assert_eq!("https", url.scheme());
        assert_eq!(Some(80), url.port_or_known_default());
    }

    #[tokio::test]
    async fn single_eval() {
        let _ = dotenvy::dotenv();
        // env_logger::init();

        let s = std::fs::read_to_string("data/compare.json").unwrap();
        let v: spec::Bast3StSpec = serde_json::from_str(&s).unwrap();
        let entities = v.entities();
        let output: Array = [1, 2, 3]
            .iter()
            .map(|x| RuntimeValue::Prim(spec::PrimitiveValue::Str(x.to_string().into())))
            .collect();
        let selectable = DummyData {
            data: RuntimeAny::from(output),
        };
        let eval = SingleEvaluation::new(
            "eval".into(),
            entities,
            13.into(),
            &selectable,
            Features::all(),
            Arc::new(Ok),
        )
        .unwrap();
        let _eval = eval.run_to_end_with_early_return().await;
        /*panic!(
            "{:?} {:#?}\n\n\n{entities:#?}",
            eval.value(),
            eval.all_values()
        );*/
    }

    #[tokio::test]
    async fn run_net() {
        let _ = dotenvy::dotenv();
        env_logger::init();

        let s = std::fs::read_to_string("data/network.json").unwrap();
        let spec: spec::Bast3StSpec = serde_json::from_str(&s).unwrap();

        let json_doc = scratch_test_model::json_from_sb3_file("../bast3st-py/a1.sb3").unwrap();
        let doc = scratch_test_model::ProjectDoc::from_json(&json_doc).unwrap();

        let report = ReportBuilder::new_good_limits()
            .with_allowed_network(Some(Arc::new(Ok)))
            .with_log_pfx(Some("test-a1".into()))
            .with_spec(&spec);

        let data = report.run_from_unique_flag(doc).await;

        panic!("{data:#?}");
    }
}
