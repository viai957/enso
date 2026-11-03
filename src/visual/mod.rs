//! The visual editing surface: Markdown edited in place in the rendered
//! page, through GPUI Kit's `editor_host`.

pub mod caret;
pub mod document;
pub mod gk_tree;
mod input;
pub mod mapping;
pub mod perf;
#[cfg(all(test, feature = "ui-tests"))]
mod tests;
pub mod view;
