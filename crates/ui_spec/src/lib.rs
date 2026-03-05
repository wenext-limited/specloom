#![forbid(unsafe_code)]

mod build;
mod model;
mod transform_plan;

pub use build::{UiSpecBuildError, apply_transform_plan, build_pre_layout_spec};
pub use model::{NodeType, UI_SPEC_VERSION, UiSpec, UiSpecRepeatError};
pub use transform_plan::{
    ChildPolicy, ChildPolicyMode, SuggestedNodeType, TRANSFORM_PLAN_VERSION, TransformDecision,
    TransformPlan, TransformPlanValidationError,
};

#[cfg(test)]
mod tests;
