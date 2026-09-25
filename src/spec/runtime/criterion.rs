use crate::{
    catchable::cerr,
    spec::{
        MachineConstruction, Numeric, PrimitiveValue, RuntimeAction, SpecialCritVariant, Text,
        ValueReference,
        machine::Machine,
        runtime::{MaybeEval, RuntimeValue},
    },
};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RuntimeCriterion {
    fulfilled: Result<(), Option<Text>>,
    inner: InnerRuntimeCriterion,
}

impl InnerRuntimeCriterion {
    pub fn build(self, is_fulfilled: bool, failure_explaination: Option<Text>) -> RuntimeCriterion {
        RuntimeCriterion::new(is_fulfilled, failure_explaination, self)
    }
    pub fn build_fulfilled(self) -> RuntimeCriterion {
        RuntimeCriterion::new_fulfilled(self)
    }
    pub fn build_unfulfilled(self, failure_explaination: Option<Text>) -> RuntimeCriterion {
        RuntimeCriterion::new_unfulfilled(failure_explaination, self)
    }
}
impl RuntimeCriterion {
    pub fn new(
        is_fulfilled: bool,
        failure_explaination: Option<Text>,
        inner: InnerRuntimeCriterion,
    ) -> Self {
        if is_fulfilled {
            Self::new_fulfilled(inner)
        } else {
            Self::new_unfulfilled(failure_explaination, inner)
        }
    }
    pub fn new_fulfilled(inner: InnerRuntimeCriterion) -> Self {
        Self {
            inner,
            fulfilled: Ok(()),
        }
    }
    pub fn new_unfulfilled(
        failure_explaination: Option<Text>,
        inner: InnerRuntimeCriterion,
    ) -> Self {
        Self {
            inner,
            fulfilled: Err(failure_explaination),
        }
    }
    pub fn is_fulfilled(&self) -> bool {
        self.fulfilled.is_ok()
    }
    pub fn is_not_fulfilled(&self) -> bool {
        self.fulfilled.is_err()
    }
    pub fn failure_explaination(&self) -> Option<&Text> {
        self.fulfilled.as_ref().err().and_then(|x| x.as_ref())
    }
    pub fn set_failure_explaination_if_failed(
        mut self,
        mut failure_explaination: Option<ValueReference>,
        overwrite_existing: bool,
    ) -> Machine<RuntimeCriterion> {
        if !overwrite_existing || self.is_fulfilled() {
            failure_explaination = None;
        }
        failure_explaination.query(|failure_explaination| {
            if self.is_not_fulfilled() {
                self.fulfilled = Err(failure_explaination);
            }
            self.into()
        })
    }
}

#[derive(Debug, PartialEq, Clone, PartialOrd)]
pub enum InnerRuntimeCriterion {
    Negated {
        clause: Box<RuntimeCriterion>,
    },
    AnyOf {
        clauses: Vec<MaybeEval<RuntimeCriterion>>,
    },
    AllOf {
        clauses: Vec<MaybeEval<RuntimeCriterion>>,
    },
    ContainOnlynum {
        sub: PrimitiveValue,
        sup: Text,
        found: Vec<Numeric>,
    },
    ContainNum {
        sub: Numeric,
        sup: Text,
        found: Vec<Numeric>,
    },
    ContainText {
        sub: PrimitiveValue,
        sup: Text,
    },

    ContainWithGaps {
        parts: Vec<PrimitiveValue>,
        unevaluated_parts: usize,
        sup: Text,
    },
    MatchesRegex {
        pattern: Text,
        sup: RuntimeValue,
    },

    GreaterThen {
        left: PrimitiveValue,
        right: PrimitiveValue,
    },
    LowerThen {
        left: PrimitiveValue,
        right: PrimitiveValue,
    },
    GreaterEqual {
        left: PrimitiveValue,
        right: PrimitiveValue,
    },
    LowerEqual {
        left: PrimitiveValue,
        right: PrimitiveValue,
    },
    NotEqual {
        left: PrimitiveValue,
        right: PrimitiveValue,
    },
    Equal {
        left: PrimitiveValue,
        right: PrimitiveValue,
    },
    Catch {
        criterion: Box<RuntimeCriterion>,
        error: cerr,
        only_if: Option<Box<MaybeEval<RuntimeCriterion>>>,
        default_value: Option<Box<MaybeEval<RuntimeCriterion>>>,
        action: Option<MaybeEval<RuntimeAction>>,
    },
    IfThenElse {
        if_: Box<RuntimeCriterion>,
        selected_branch: Box<RuntimeCriterion>,
        other_branch: Box<MaybeEval<RuntimeCriterion>>,
    },
    SpecialCrit {
        variant: SpecialCritVariant,
    },
}
