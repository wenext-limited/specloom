#![forbid(unsafe_code)]

mod build;
mod model;
mod transform_plan;

pub use build::{UiSpecBuildError, build_ui_spec};
pub use model::{NodeType, UI_SPEC_VERSION, UiSpec};
pub use transform_plan::{
    ChildPolicy, ChildPolicyMode, SuggestedNodeType, TRANSFORM_PLAN_VERSION, TransformDecision,
    TransformPlan, TransformPlanValidationError,
};

#[cfg(test)]
mod tests;
