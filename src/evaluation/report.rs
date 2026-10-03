use std::sync::Arc;

use derive_getters::Getters;
use scratch_test_interpreter::Limits;
use scratch_test_model::{Id, ProjectDoc};

use crate::{
    LogPfx,
    catchable::cerr,
    evaluation::{
        AllowNetData, Context, PSpec, SelectableSource, process_spec::SpecRunError,
        single_evaluation::AllowedNetClosure,
    },
    spec::{Bast3StSpec, RealMapping},
};

pub trait ReportBuilderState {
    type Spec<'a>;
}

#[derive(derive_more::Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WithoutSpec(());
#[derive(derive_more::Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WithSpec(());

impl ReportBuilderState for WithoutSpec {
    type Spec<'a> = ();
}
impl ReportBuilderState for WithSpec {
    type Spec<'a> = &'a Bast3StSpec;
}

#[derive(derive_more::Debug, Clone)]
pub struct ReportBuilder<'a, Fallback = (), State: ReportBuilderState = WithoutSpec> {
    limits: EvalLimitations,
    #[debug("allowed_network: ...")]
    allowed_network: Option<AllowedNetClosure>,
    log_pfx: Option<LogPfx>,
    param_my: Option<RealMapping>,
    fallback: Option<Fallback>,
    spec: State::Spec<'a>,
    _phantom: std::marker::PhantomData<State>,
}
impl Default for ReportBuilder<'static, (), WithoutSpec> {
    fn default() -> Self {
        Self::no_limits()
    }
}

impl<'a, Fallback: SelectableSource, X: ReportBuilderState> ReportBuilder<'a, Fallback, X> {
    pub const fn with_max_list_length(mut self, max: Option<u32>) -> Self {
        self.limits.max_list_length = max;
        self
    }
    pub const fn with_max_string_length(mut self, max: Option<usize>) -> Self {
        self.limits.max_string_length = max;
        self
    }
    pub const fn with_max_executed_stmts(mut self, max: Option<usize>) -> Self {
        self.limits.max_executed_stmts = max;
        self
    }
    pub const fn with_max_statement_stack_height(mut self, max: Option<usize>) -> Self {
        self.limits.max_statement_stack_height = max;
        self
    }
    pub const fn with_eval_limits(mut self, limits: EvalLimitations) -> Self {
        self.limits = limits;
        self
    }

    pub fn with_allowed_network(mut self, net: Option<AllowedNetClosure>) -> Self {
        self.allowed_network = net;
        self
    }
    pub fn with_log_pfx(mut self, log_pfx: Option<LogPfx>) -> Self {
        self.log_pfx = log_pfx;
        self
    }
    pub fn with_fallback(mut self, fb: Option<Fallback>) -> Self {
        self.fallback = fb;
        self
    }

    pub fn add_allow_network_map_check(
        self,
        closure: impl Fn(AllowNetData) -> Result<AllowNetData, (AllowNetData, cerr)>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        if let Some(first_check) = self.allowed_network.clone() {
            self.only_allow_network_map(move |input| {
                let input = first_check(input)?;
                closure(input)
            })
        } else {
            self.only_allow_network_map(closure)
        }
    }
    /// This removes all previouse verifying levels
    /// and sets the given closure as the only check
    fn only_allow_network_map(
        mut self,
        closure: impl Fn(AllowNetData) -> Result<AllowNetData, (AllowNetData, cerr)>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        self.allowed_network = Some(Arc::from(closure));
        self
    }
    #[allow(unused)]
    /// This removes all previouse verifying levels
    /// and sets the given closure as the only check
    fn only_allow_network_if(
        mut self,
        closure: impl Fn(&AllowNetData) -> bool + Send + Sync + 'static,
    ) -> Self {
        self.allowed_network = None;
        self.add_allow_network_if_check(closure)
    }
    pub fn add_allow_network_if_check(
        self,
        closure: impl Fn(&AllowNetData) -> bool + Send + Sync + 'static,
    ) -> Self {
        self.add_allow_network_map_check(move |x| {
            if closure(&x) {
                Ok(x)
            } else {
                Err((x, cerr::network_policy_other))
            }
        })
    }
    /// removes all checks and disallows all network access in this way
    pub fn disallow_network(mut self) -> Self {
        self.allowed_network = None;
        self
    }
}

impl ReportBuilder<'static, (), WithoutSpec> {
    pub const fn no_limits() -> Self {
        Self {
            limits: EvalLimitations::no_limits(),
            allowed_network: None,
            log_pfx: None,
            param_my: None,
            fallback: None,
            spec: (),
            _phantom: std::marker::PhantomData,
        }
    }
    pub const fn new_good_limits() -> Self {
        Self::no_limits().with_eval_limits(EvalLimitations::NICELY_RESTRICTIVE)
    }
}

impl<Fallback: SelectableSource> ReportBuilder<'static, Fallback, WithoutSpec> {
    pub fn with_spec<'a>(self, spec: &'a Bast3StSpec) -> ReportBuilder<'a, Fallback, WithSpec> {
        ReportBuilder {
            limits: self.limits,
            allowed_network: self.allowed_network,
            log_pfx: self.log_pfx,
            param_my: self.param_my,
            fallback: self.fallback,
            spec,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<'a, Fallback: SelectableSource> ReportBuilder<'a, Fallback, WithSpec> {
    pub fn build(
        self,
        doc: ProjectDoc,
        initial_block: Id,
    ) -> (Context, ExtraBuilderInfo<'a, Fallback>) {
        (
            Context {
                max_list_length: self.limits.max_list_length.unwrap_or(u32::MAX),
                limits: self.limits.into(),
                doc,
                initial_block,
                entities: self.spec.entities().clone(),
                allowed_network: self
                    .allowed_network
                    .unwrap_or(Arc::new(|x| Err((x, cerr::network_policy_other)))),
            },
            (
                self.log_pfx.unwrap_or(LogPfx::new("")),
                self.param_my,
                self.fallback,
                self.spec,
            ),
        )
    }
    pub async fn run(
        self,
        doc: ProjectDoc,
        initial_block: Option<Id>,
    ) -> Result<PSpec, SpecRunError>
    where
        Fallback: Clone + 'static,
    {
        PSpec::new(self, doc, initial_block).await
    }

    pub async fn run_from_unique_flag(self, doc: ProjectDoc) -> Result<PSpec, SpecRunError>
    where
        Fallback: Clone + 'static,
    {
        PSpec::new(self, doc, None).await
    }
}
pub type ExtraBuilderInfo<'a, Fallback> = (
    LogPfx,
    Option<RealMapping>,
    Option<Fallback>,
    &'a Bast3StSpec,
);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default, Getters)]
pub struct EvalLimitations {
    max_list_length: Option<u32>,
    max_string_length: Option<usize>,
    max_executed_stmts: Option<usize>,
    max_statement_stack_height: Option<usize>,
}

impl EvalLimitations {
    pub const NICELY_RESTRICTIVE: EvalLimitations = EvalLimitations {
        max_list_length: Some(255),
        max_string_length: Some(255),
        max_executed_stmts: Some(10 * 1024),
        max_statement_stack_height: Some(128),
    };

    pub const fn no_limits() -> Self {
        Self {
            max_list_length: None,
            max_string_length: None,
            max_executed_stmts: None,
            max_statement_stack_height: None,
        }
    }
    pub const fn from_limits(max_list_length: Option<u32>, limits: Limits) -> Self {
        Self {
            max_list_length,
            max_string_length: *limits.max_string_length(),
            max_executed_stmts: *limits.max_executed_stmts(),
            max_statement_stack_height: *limits.max_statement_stack_height(),
        }
    }
}

impl From<Limits> for EvalLimitations {
    fn from(value: Limits) -> Self {
        Self::from_limits(None, value)
    }
}
impl From<EvalLimitations> for Limits {
    fn from(value: EvalLimitations) -> Self {
        Self::new()
            .with_max_string_length(value.max_string_length)
            .with_max_executed_stmts(value.max_executed_stmts)
            .with_max_statement_stack_height(value.max_statement_stack_height)
    }
}
