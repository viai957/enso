# enso patch to gpui-base 0.8.1

This branch is the crates.io `gpui-base` 0.8.1 package (Apache-2.0, upstream
`longbridge/gpui-kit` at `99576db`, `crates/base`) plus the smallest change
enso's visual editor needs. enso pins it by commit through
`[patch.crates-io]`; nothing else in the crate changes.

Changes, all in `src/input/base/state.rs`, all additive and read-only or
visibility-only:

1. `InputBaseState::undo` and `InputBaseState::redo` are `pub` instead of
   `pub(super)`. Their handlers are registered by the `Editor` element, so a
   host that renders the document through `editor_host` instead never reaches
   them; one undo stack per document then needs the host to call them.
2. `InputBaseState::document_revision()` returns the existing private
   `document_revision` counter. It changes on every text change, including
   `set_value`, which emits no `InputEvent::Change`, so a host mirroring the
   text knows when to resynchronize.

Upstream request: see `UPSTREAM-REQUEST.md`. Drop this fork once a released
GPUI Kit exposes the same APIs.
