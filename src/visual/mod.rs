//! The visual editing surface (Phase 0 spike): Markdown edited in place in
//! the rendered page, through GPUI Kit's `editor_host`.

pub mod caret;
pub mod document;
mod input;
pub mod mapping;
#[cfg(all(test, feature = "ui-tests"))]
mod tests;
pub mod view;
