#![forbid(unsafe_code)]

mod build;
mod model;

pub use build::{UiSpecBuildError, build_ui_spec};
pub use model::{NodeType, UI_SPEC_VERSION, UiSpec};

#[cfg(test)]
mod tests;
