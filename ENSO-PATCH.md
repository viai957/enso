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

## Branch gpui-base-fork-1a (experiment, not pinned by enso)

Adds read-only access to the source mapping GPUI Kit's parser already
computes, to evaluate driving enso's visual view from GPUI Kit's own parse:
`SourceSegment` becomes public with `rendered()`, `source()` and `linear()`
getters (fields stay private), plus `InlineNode::source_mapping()`,
`CodeBlock::source_mapping()` and `ImageNode::span()`. No behaviour changes.

## Tables rendered in slices (enso patch, approved non-read-only change)

`Table::measure_column_widths` exposes the column measurement rendering
already does (and caches). `Table::chunk(header, top, bottom, widths)` renders
a table as one slice of a longer table: header styling only on the first
slice, frame borders and corners only at the ends, fixed column widths so the
slices line up. enso renders long tables as slices in a virtualized list, so
a 400-row table no longer lays out every row on every frame. Additive; tables
that never call `chunk` render exactly as before.
