//! The visual editing surface: Markdown edited in place in the rendered
//! page, through GPUI Kit's `editor_host`.

mod build;
pub mod caret;
pub mod document;
mod input;
pub mod mapping;
pub mod perf;
pub mod rows;
#[cfg(all(test, feature = "ui-tests"))]
mod tests;
pub mod view;
