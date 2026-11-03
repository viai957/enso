# Draft upstream request: public undo/redo and a text revision for editor hosts

**Title:** editor_host: let external editors drive `InputBaseState` undo/redo and detect text changes

`editor_host`'s docs ask to "extend the public boundary additively". An
external editor that renders through `editor_host` but keeps its text in
`EditorState` (to share one undo history and save path with the source
`Editor`) is missing two things:

1. **Undo/redo.** `InputBaseState::undo`/`redo` are `pub(super)` and only the
   `Editor` element registers them as action handlers. While the host's view
   is shown, the `Undo` action never reaches the state. Proposal: make both
   `pub` (or add `pub fn undo_transaction(&mut self, window, cx) -> bool`).
2. **Change detection.** `InputEvent::Change` carries no range, and
   `set_value` emits no event at all. Proposal: expose the existing
   `document_revision` counter read-only, so hosts know when to diff or
   resynchronize. Optionally, a later addition could report the changed byte
   range of the last transaction.

Both are additive. enso carries them as a two-function patch on 0.8.1
(see ENSO-PATCH.md) and can share tests.
