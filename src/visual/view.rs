//! The visual surface: renders the block index through `editor_host`, keeps
//! it in sync with the editor state, hit tests, moves the caret and scrolls.
//!
//! # Keeping the index in sync
//!
//! The `EditorState` rope is the document. The view mirrors it in
//! [`VisualDocument`] and must stay correct for every change, wherever it
//! comes from: its own typing, undo and redo, typing in a source pane that
//! shares the state, and reloads through `set_value` (which emits no
//! `InputEvent::Change`). So the view observes the state and compares its
//! `document_revision` (exposed by enso's gpui-base patch) with the revision
//! it last mirrored. On a difference it recovers the changed range, from the
//! hint its own edit left or else from a prefix/suffix diff of the two ropes,
//! and applies it to the index incrementally.
//!
//! # Off the main thread
//!
//! Whole-document parses never run on the main thread. On open, the view
//! parses a prefix of about [`OPEN_PREFIX_BYTES`] synchronously, so the first
//! screen paints at once, and parses the rest in the background. When an edit
//! cannot be validated cheaply, the index takes a provisional local parse (the
//! typed text shows immediately) and a background parse reconciles it. Each
//! background result carries the revision it parsed; if the text moved on, the
//! edits since are diffed and applied to the result before it is merged, so
//! input is never dropped and no stale result is shown.

use std::{cell::RefCell, collections::HashMap, ops::Range, rc::Rc, sync::Arc, time::Duration};

use gpui_kit::base::input::Rope;
use gpui_kit::base::text::editor_host::{
    BlockNode, FlowPosition, LeafGeometry, RenderContext, Table, collect_leaf_geometry,
};
use gpui_kit::component::{ActiveTheme as _, input::EditorState};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use unicode_segmentation::UnicodeSegmentation as _;

use super::caret::Caret;
use super::document::{self, EditResult, Kind, VisualDocument};
use super::perf::{Perf, ms};
use super::rows::Rows;

actions!(
    visual_editor,
    [
        MoveLeft,
        MoveRight,
        MoveUp,
        MoveDown,
        MoveLineStart,
        MoveLineEnd,
        Backspace,
        Delete,
        Enter,
        Undo,
        Redo
    ]
);

pub const CONTEXT: &str = "VisualEditor";

/// How much of a document is parsed on the main thread when it opens.
pub const OPEN_PREFIX_BYTES: usize = 12 * 1024;

/// The page's body text size (placeholder token until the design reference
/// sets it).
pub const BODY_SIZE: f32 = 17.;

/// How long typing must pause before the background whole parse that
/// settles the index runs.
pub const SETTLE_AFTER: Duration = Duration::from_millis(300);

/// How many pre-edit scroll positions undo can restore.
const SCROLL_HISTORY: usize = 512;

pub fn init(cx: &mut App) {
    let (undo, redo, redo_alt) = if cfg!(target_os = "macos") {
        ("cmd-z", "cmd-shift-z", "cmd-y")
    } else {
        ("ctrl-z", "ctrl-shift-z", "ctrl-y")
    };
    cx.bind_keys([
        KeyBinding::new("left", MoveLeft, Some(CONTEXT)),
        KeyBinding::new("right", MoveRight, Some(CONTEXT)),
        KeyBinding::new("up", MoveUp, Some(CONTEXT)),
        KeyBinding::new("down", MoveDown, Some(CONTEXT)),
        KeyBinding::new("home", MoveLineStart, Some(CONTEXT)),
        KeyBinding::new("end", MoveLineEnd, Some(CONTEXT)),
        KeyBinding::new("backspace", Backspace, Some(CONTEXT)),
        KeyBinding::new("delete", Delete, Some(CONTEXT)),
        KeyBinding::new("enter", Enter, Some(CONTEXT)),
        KeyBinding::new(undo, Undo, Some(CONTEXT)),
        KeyBinding::new(redo, Redo, Some(CONTEXT)),
        KeyBinding::new(redo_alt, Redo, Some(CONTEXT)),
    ]);
}

/// The state of the background whole-document parse.
#[derive(Default)]
struct Background {
    /// Bumped for every request; a result for an older one is dropped.
    generation: u64,
    task: Option<Task<()>>,
    /// The first parse after open has not landed yet.
    loading: bool,
    /// The idle whole parse that settles local results.
    settle: Option<Task<()>>,
}

pub struct VisualView {
    pub editor: Entity<EditorState>,
    pub doc: VisualDocument,
    pub list: ListState,
    /// The caret as an absolute source byte offset: the only caret state.
    pub caret: usize,
    /// The x the caret keeps across vertical moves.
    goal_x: Option<Pixels>,
    /// The IME's marked text, a source byte range.
    pub marked: Option<Range<usize>>,
    /// Leaf geometry from the last prepaint.
    pub geometry: Rc<RefCell<HashMap<u64, LeafGeometry>>>,
    /// A vertical move waiting for the scroll that brings its target in.
    pending_vertical: Option<bool>,
    /// A block to bring into view, retried after layout: the list estimates
    /// the heights of blocks it has not measured, so one reveal can land
    /// short of a far block.
    pending_reveal: Option<(usize, u8)>,
    pub focus: FocusHandle,
    pub perf: Rc<RefCell<Perf>>,
    pub caret_blink: Caret,
    render_cx: RenderContext,
    /// The text and revision `doc` mirrors.
    synced_text: Rope,
    synced_revision: u64,
    /// The range of the edit this view just made, so sync can skip the diff.
    hint: Option<(Range<usize>, usize)>,
    background: Background,
    /// (caret before, scroll before) of this view's edits, for undo.
    scroll_history: Vec<(usize, ListOffset)>,
    /// Paths and sizes of the last sync, for tests and the overlay.
    pub last_edit: Option<EditResult>,
    /// The list's rows: blocks, with long lists and tables in slices.
    pub rows: Rows,
    /// Column widths of long tables rendered in slices, by block id: the row
    /// count they were measured for, and the widths. Edited rows widen them
    /// incrementally (see `table_dirty`).
    table_widths: HashMap<u64, (usize, Arc<Vec<f32>>)>,
    /// Rows edited since the widths were measured, by block id.
    table_dirty: HashMap<u64, Vec<usize>>,
    _observe: Subscription,
}

impl VisualView {
    pub fn new(editor: Entity<EditorState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let _ = window;
        let (text, revision) = {
            let state = editor.read(cx);
            (state.text().clone(), state.document_revision())
        };
        // A prefix now, the rest in the background: no blank first frame,
        // no long stall.
        let source = text.to_string();
        let cut = document::prefix_cut(&source, OPEN_PREFIX_BYTES);
        let doc = VisualDocument::from_blocks(document::parse_blocks(&source[..cut], 0), &text);
        let rows = Rows::build(&doc);
        let list = ListState::new(rows.len(), ListAlignment::Top, px(600.));
        let render_cx = RenderContext::from_app(cx).with_body_font_size(px(BODY_SIZE));
        let caret = doc
            .blocks
            .iter()
            .find(|b| !b.leaves.is_empty())
            .map_or(0, |b| b.start + b.leaves[0].leaf_to_src(0));
        let observe = cx.observe(&editor, |this, _, cx| this.sync(cx));
        let mut view = Self {
            editor,
            doc,
            list,
            caret,
            goal_x: None,
            marked: None,
            geometry: Rc::default(),
            pending_vertical: None,
            pending_reveal: None,
            focus: cx.focus_handle(),
            perf: Rc::new(RefCell::new(Perf::from_env())),
            caret_blink: Caret::new(cx),
            render_cx,
            synced_text: text,
            synced_revision: revision,
            hint: None,
            background: Background::default(),
            scroll_history: Vec::new(),
            last_edit: None,
            rows,
            table_widths: HashMap::new(),
            table_dirty: HashMap::new(),
            _observe: observe,
        };
        if cut < source.len() {
            view.background.loading = true;
            view.schedule_full_parse(cx);
        }
        view
    }

    /// Whether the document's first whole parse is still running.
    pub fn loading(&self) -> bool {
        self.background.loading
    }

    /// Whether a background parse is outstanding.
    pub fn parse_pending(&self) -> bool {
        self.background.task.is_some()
    }

    // ----- sync ---------------------------------------------------------

    /// Brings the index up to date with the editor state, whatever changed
    /// it. Cheap when nothing did: one integer comparison.
    pub fn sync(&mut self, cx: &mut Context<Self>) {
        let (revision, rope, cursor) = {
            let state = self.editor.read(cx);
            (
                state.document_revision(),
                state.text().clone(),
                state.cursor(),
            )
        };
        if revision == self.synced_revision {
            return;
        }
        let expected_len = |(old, len): &(Range<usize>, usize)| {
            self.synced_text.len() - old.len() + len == rope.len()
        };
        let change = match self.hint.take().filter(expected_len) {
            Some(hint) => Some(hint),
            None => document::diff(&self.synced_text, &rope),
        };
        self.synced_text = rope;
        self.synced_revision = revision;
        let Some((old, new_len)) = change else {
            return;
        };
        let result = self.doc.edit(&self.synced_text, old.clone(), new_len);
        self.follow_splice(
            result.splice.old.clone(),
            result.splice.new_len,
            result.inner.clone(),
            result.path == document::Path::TableRow,
        );
        self.perf
            .borrow_mut()
            .reparsed(result.parsed_bytes, result.path);
        if result.needs_full {
            self.schedule_full_parse(cx);
        } else if result.parsed_bytes > 0 {
            // Local reparses are exact except where markdown 1.0.0 reaches
            // across blocks (about 0.2% of adversarial random edits, none in
            // ordinary typing): settle with a whole parse once typing pauses.
            self.schedule_settle(cx);
        }
        self.last_edit = Some(result);
        // A change made elsewhere moves the caret like any text after it; a
        // caret inside the replaced text goes to its end.
        let caret = if self.caret <= old.start {
            self.caret
        } else if self.caret >= old.end {
            self.caret - old.len() + new_len
        } else {
            old.start + new_len
        };
        let _ = cursor;
        self.caret = self.doc.snap(caret.min(self.synced_text.len()));
        if let Some(marked) = &self.marked
            && marked.end > self.synced_text.len()
        {
            self.marked = None;
        }
        cx.notify();
    }

    /// Brings the rows and the list up to date with a splice of the blocks.
    fn follow_splice(
        &mut self,
        old: Range<usize>,
        new_len: usize,
        inner: Option<Range<usize>>,
        table_row: bool,
    ) {
        // Long tables keep their measured widths across row edits, widened by
        // the edited rows; any other change to them measures again.
        for block in &self.doc.blocks[old.start..old.start + new_len] {
            match (&inner, table_row) {
                (Some(rows), true) => self
                    .table_dirty
                    .entry(block.id)
                    .or_default()
                    .extend(rows.clone()),
                _ => {
                    self.table_widths.remove(&block.id);
                    self.table_dirty.remove(&block.id);
                }
            }
        }
        let (rows, count) = self.rows.splice(&self.doc, old, new_len, inner);
        self.splice_list(rows, count);
    }

    /// `ListState::splice`, keeping the scroll position: the list resets its
    /// anchor to the top of a replaced item, which would jump the view to the
    /// start of a tall block on every keystroke inside it.
    fn splice_list(&mut self, old: Range<usize>, new_len: usize) {
        let top = self.list.logical_scroll_top();
        self.list.splice(old.clone(), new_len);
        if old.contains(&top.item_ix) && new_len > 0 {
            let item_ix = top.item_ix.min(old.start + new_len - 1);
            self.list.scroll_to(ListOffset {
                item_ix,
                offset_in_item: top.offset_in_item,
            });
        }
    }

    /// Starts a whole-document parse of the current text on a background
    /// thread, replacing any parse still running.
    fn schedule_full_parse(&mut self, cx: &mut Context<Self>) {
        self.background.generation += 1;
        let generation = self.background.generation;
        let snapshot = self.synced_text.clone();
        let revision = self.synced_revision;
        let worker = snapshot.clone();
        let parse = cx
            .background_executor()
            .spawn(async move { document::parse_blocks(&worker.to_string(), 0) });
        self.background.task = Some(cx.spawn(async move |this, cx| {
            let blocks = parse.await;
            let _ = this.update(cx, |this, cx| {
                this.finish_full_parse(generation, snapshot, revision, blocks, cx)
            });
        }));
    }

    fn schedule_settle(&mut self, cx: &mut Context<Self>) {
        self.background.settle = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(SETTLE_AFTER).await;
            let _ = this.update(cx, |this, cx| {
                this.background.settle = None;
                if this.background.task.is_none() {
                    this.schedule_full_parse(cx);
                }
            });
        }));
    }

    fn finish_full_parse(
        &mut self,
        generation: u64,
        snapshot: Rope,
        revision: u64,
        blocks: Vec<document::Block>,
        cx: &mut Context<Self>,
    ) {
        if generation != self.background.generation {
            return;
        }
        self.background.task = None;
        // Make sure edits the observer has not seen yet are counted.
        self.sync(cx);
        let mut fresh = VisualDocument::from_blocks(blocks, &snapshot);
        if revision != self.synced_revision
            && let Some((old, new_len)) = document::diff(&snapshot, &self.synced_text)
        {
            // The text moved on while parsing: replay the difference.
            let result = fresh.edit(&self.synced_text, old, new_len);
            if result.needs_full {
                self.schedule_full_parse(cx);
            }
        }
        let splice = self.doc.reconcile(fresh.into_blocks(), &self.synced_text);
        self.follow_splice(splice.old, splice.new_len, None, false);
        self.background.loading = false;
        self.caret = self.doc.snap(self.caret);
        cx.notify();
    }

    // ----- editing -------------------------------------------------------

    /// Replaces source `range` with `text` through the editor state (one undo
    /// history, the save path sees it), then syncs the index.
    pub fn edit(
        &mut self,
        range: Range<usize>,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.remember_scroll();
        self.hint = Some((range.clone(), text.len()));
        self.editor.update(cx, |state, cx| {
            state.set_selected_range(range.clone(), cx);
            state.replace(text.to_string(), window, cx);
        });
        self.sync(cx);
        self.caret = range.start + text.len();
        self.goal_x = None;
        self.after_caret_change(cx);
        self.perf.borrow_mut().handler_done();
    }

    fn remember_scroll(&mut self) {
        if self.scroll_history.len() >= SCROLL_HISTORY {
            self.scroll_history.drain(..SCROLL_HISTORY / 2);
        }
        self.scroll_history
            .push((self.caret, self.list.logical_scroll_top()));
    }

    fn after_caret_change(&mut self, cx: &mut Context<Self>) {
        self.caret_blink.reset(cx);
        if let Some(row) = self.caret_row()
            && !self.reveal_caret_step(row)
        {
            // Correct with the next frame's layout.
            self.pending_reveal = Some((row, 4));
        }
        cx.notify();
    }

    fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        // A bound key runs its action instead of the key-down listeners.
        self.perf.borrow_mut().input();
        let Some((bix, lix, index)) = self.doc.locate(self.caret) else {
            return;
        };
        let leaf = &self.doc.blocks[bix].leaves[lix];
        if index > 0 {
            let prev = leaf.snap_index(prev_boundary(&leaf.text, index), false);
            let start = self.doc.src_of(bix, lix, prev);
            let end = self.caret.max(start);
            let end = if end == start {
                self.doc.src_of(bix, lix, index)
            } else {
                end
            };
            self.edit(start..end, "", window, cx);
        } else if let Some((pb, pl)) = self.doc.step_leaf(bix, lix, false) {
            // Join with the previous leaf: remove the source between them.
            let prev_len = self.doc.blocks[pb].leaves[pl].len();
            let start = self.doc.src_of(pb, pl, prev_len);
            if start < self.caret {
                self.edit(start..self.caret, "", window, cx);
            }
        }
    }

    fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        self.perf.borrow_mut().input();
        let Some((bix, lix, index)) = self.doc.locate(self.caret) else {
            return;
        };
        let leaf = &self.doc.blocks[bix].leaves[lix];
        if index < leaf.len() {
            let next = leaf.snap_index(next_boundary(&leaf.text, index), true);
            let end = self.doc.src_of(bix, lix, next);
            if end > self.caret {
                self.edit(self.caret..end, "", window, cx);
            }
        }
    }

    fn enter(&mut self, _: &Enter, window: &mut Window, cx: &mut Context<Self>) {
        self.perf.borrow_mut().input();
        let in_code = self
            .doc
            .locate(self.caret)
            .is_some_and(|(b, _, _)| matches!(self.doc.blocks[b].node, BlockNode::CodeBlock(_)));
        let caret = self.caret;
        self.edit(
            caret..caret,
            if in_code { "\n" } else { "\n\n" },
            window,
            cx,
        );
    }

    /// Undo or redo in the one history the document has, shared with every
    /// view of it. The state restores text and selection; the view restores
    /// the scroll position the edit was made at when it made it, and
    /// otherwise keeps the restored caret in view.
    pub fn replay(&mut self, undo: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.perf.borrow_mut().input();
        self.marked = None;
        self.editor.update(cx, |state, cx| {
            if undo {
                state.undo(&gpui_kit::component::input::Undo, window, cx);
            } else {
                state.redo(&gpui_kit::component::input::Redo, window, cx);
            }
        });
        self.sync(cx);
        let selection = self.editor.read(cx).selected_range();
        self.caret = self.doc.snap(selection.start.min(self.synced_text.len()));
        self.goal_x = None;
        let restored = undo
            && self
                .scroll_history
                .iter()
                .rev()
                .find(|(caret, _)| *caret == selection.start)
                .map(|(_, offset)| *offset)
                .map(|offset| self.list.scroll_to(offset))
                .is_some();
        if restored {
            self.pending_reveal = None;
        }
        self.caret_blink.reset(cx);
        if !restored {
            self.reveal_caret_if_hidden();
        }
        self.perf.borrow_mut().handler_done();
        cx.notify();
    }

    /// The list row showing the caret.
    fn caret_row(&self) -> Option<usize> {
        let (bix, lix, _) = self.doc.locate(self.caret)?;
        Some(self.rows.row_of(&self.doc, bix, lix))
    }

    /// One step of bringing the caret into view, with this frame's layout.
    /// Returns true when it is in view. The list only knows the heights of
    /// blocks it has measured, so for a far block this first jumps to the
    /// block, then scrolls to the caret line once the block is laid out
    /// (which also works for a caret deep inside a tall block).
    fn reveal_caret_step(&mut self, row: usize) -> bool {
        let viewport = self.list.viewport_bounds();
        let Some((top, line_height, _)) = self.caret_position() else {
            self.list.scroll_to(ListOffset {
                item_ix: row,
                offset_in_item: px(0.),
            });
            return false;
        };
        let margin = line_height.min(viewport.size.height / 4.);
        if top.y >= viewport.top() + margin && top.y + line_height <= viewport.bottom() - margin {
            return true;
        }
        // Anchor the list on the caret's block, offset to the caret line: an
        // anchor is not clamped by the estimated heights of unmeasured blocks,
        // as a relative scroll is.
        let Some(item) = self.list.bounds_for_item(row) else {
            self.list.scroll_to(ListOffset {
                item_ix: row,
                offset_in_item: px(0.),
            });
            return false;
        };
        let within = top.y - item.top();
        let goal = if top.y < viewport.top() + margin {
            margin
        } else {
            viewport.size.height - margin - line_height
        };
        if within >= goal {
            // The block starts above the viewport once the caret is in place.
            self.list.scroll_to(ListOffset {
                item_ix: row,
                offset_in_item: within - goal,
            });
        } else {
            // The block's top stays in view: scroll by the difference.
            self.list.scroll_by(top.y - (viewport.top() + goal));
        }
        false
    }

    /// Scrolls only when the caret's block is outside the viewport.
    fn reveal_caret_if_hidden(&mut self) {
        if let Some(row) = self.caret_row()
            && !self.reveal_caret_step(row)
        {
            self.pending_reveal = Some((row, 4));
        }
    }

    // ----- caret movement -------------------------------------------------

    fn move_horizontal(&mut self, forward: bool, cx: &mut Context<Self>) {
        self.perf.borrow_mut().input();
        let Some((bix, lix, index)) = self.doc.locate(self.caret) else {
            return;
        };
        let leaf = &self.doc.blocks[bix].leaves[lix];
        let target = if forward && index < leaf.len() {
            let next = next_boundary(&leaf.text, index);
            Some((bix, lix, leaf.snap_index(next, true)))
        } else if !forward && index > 0 {
            let prev = prev_boundary(&leaf.text, index);
            Some((bix, lix, leaf.snap_index(prev, false)))
        } else {
            self.doc.step_leaf(bix, lix, forward).map(|(b, l)| {
                let len = self.doc.blocks[b].leaves[l].len();
                (b, l, if forward { 0 } else { len })
            })
        };
        if let Some((b, l, i)) = target {
            self.caret = self.doc.src_of(b, l, i);
        }
        self.goal_x = None;
        self.after_caret_change(cx);
    }

    /// The window position, line height and caret height of the caret, from
    /// the last frame.
    pub fn caret_position(&self) -> Option<(Point<Pixels>, Pixels, Pixels)> {
        let (bix, lix, index) = self.doc.locate(self.caret)?;
        let key = self.doc.blocks[bix].leaves[lix].key;
        let geometry = self.geometry.borrow();
        let leaf = geometry.get(&key)?;
        Some((
            leaf.position_for_index(index),
            leaf.line_height(),
            leaf.caret_height(),
        ))
    }

    /// Moves to the visual line above or below at the goal x, across blocks
    /// of any size. Returns false when the target is not laid out yet.
    fn try_vertical(&mut self, down: bool) -> bool {
        let Some((top, line_height, _)) = self.caret_position() else {
            return false;
        };
        let goal = *self.goal_x.get_or_insert(top.x);
        let geometry = self.geometry.borrow();
        let mut y = if down {
            top.y + line_height + px(1.)
        } else {
            top.y - px(1.)
        };
        for _ in 0..600 {
            let hit = geometry
                .iter()
                .filter(|(_, g)| {
                    let b = g.bounds();
                    b.top() <= y && y < b.bottom()
                })
                .min_by(|a, b| {
                    let d = |g: &LeafGeometry| {
                        let b = g.bounds();
                        if goal < b.left() {
                            b.left() - goal
                        } else if goal > b.right() {
                            goal - b.right()
                        } else {
                            px(0.)
                        }
                    };
                    d(a.1)
                        .partial_cmp(&d(b.1))
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            if let Some((&key, g)) = hit {
                let index = g.index_for_position(point(goal, y));
                let at = g.position_for_index(index);
                let moved = if down {
                    at.y > top.y + px(0.5)
                } else {
                    at.y < top.y - px(0.5)
                };
                if moved && let Some((bix, lix)) = self.doc.leaf_by_key(key) {
                    self.caret = self.doc.src_of(bix, lix, index);
                    return true;
                }
            }
            y += if down { px(2.) } else { px(-2.) };
        }
        false
    }

    fn move_vertical(&mut self, down: bool, cx: &mut Context<Self>) {
        self.perf.borrow_mut().input();
        if self.try_vertical(down) {
            self.pending_vertical = None;
            self.caret_blink.reset(cx);
            cx.notify();
            return;
        }
        let step = self.caret_position().map_or(px(40.), |(_, h, _)| h * 2.);
        self.list.scroll_by(if down { step } else { -step });
        self.pending_vertical = Some(down);
        cx.notify();
    }

    fn line_edge(&mut self, end: bool, cx: &mut Context<Self>) {
        self.perf.borrow_mut().input();
        let Some((top, line_height, _)) = self.caret_position() else {
            return;
        };
        let Some((bix, lix, _)) = self.doc.locate(self.caret) else {
            return;
        };
        let key = self.doc.blocks[bix].leaves[lix].key;
        let index = {
            let geometry = self.geometry.borrow();
            let Some(g) = geometry.get(&key) else { return };
            let x = if end { px(1e6) } else { px(-1e6) };
            g.index_for_position(point(x, top.y + line_height / 2.))
        };
        self.caret = self.doc.src_of(bix, lix, index);
        self.goal_x = None;
        self.after_caret_change(cx);
    }

    /// The source offset under a window point, from the last frame's geometry.
    pub fn source_for_point(&self, at: Point<Pixels>) -> Option<usize> {
        let geometry = self.geometry.borrow();
        let dist = |g: &LeafGeometry| {
            let b = g.bounds();
            let dy = if at.y < b.top() {
                b.top() - at.y
            } else if at.y > b.bottom() {
                at.y - b.bottom()
            } else {
                px(0.)
            };
            let dx = if at.x < b.left() {
                b.left() - at.x
            } else if at.x > b.right() {
                at.x - b.right()
            } else {
                px(0.)
            };
            (dy, dx)
        };
        let (&key, g) = geometry.iter().min_by(|a, b| {
            dist(a.1)
                .partial_cmp(&dist(b.1))
                .unwrap_or(std::cmp::Ordering::Equal)
        })?;
        let index = g.index_for_position(at);
        let (bix, lix) = self.doc.leaf_by_key(key)?;
        Some(self.doc.src_of(bix, lix, index))
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.perf.borrow_mut().input();
        window.focus(&self.focus, cx);
        if let Some(src) = self.source_for_point(event.position) {
            self.caret = src;
            self.marked = None;
            self.goal_x = None;
            self.caret_blink.reset(cx);
            cx.notify();
        }
        self.perf.borrow_mut().handler_done();
    }

    // ----- rendering --------------------------------------------------------

    /// The column widths of a long table rendered in slices: measured once
    /// over the whole table, then widened by the rows edited since.
    fn table_widths(
        &mut self,
        bix: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Arc<Vec<f32>> {
        let block = &self.doc.blocks[bix];
        let Kind::Table { rows, aligns } = &block.kind else {
            return Arc::default();
        };
        let id = block.id;
        let table = |rows: Vec<Vec<_>>| Table::new(rows, aligns.clone());
        let measured = match self.table_widths.get(&id) {
            Some((count, widths)) if *count == rows.len() => {
                let dirty = self.table_dirty.remove(&id).unwrap_or_default();
                if dirty.is_empty() {
                    return widths.clone();
                }
                let mut sample = vec![rows[0].cells.clone()];
                sample.extend(
                    dirty
                        .iter()
                        .filter_map(|&r| rows.get(r))
                        .map(|r| r.cells.clone()),
                );
                let fresh = table(sample).measure_column_widths(&self.render_cx, window, cx);
                let mut widths = widths.to_vec();
                widths.resize(fresh.len().max(widths.len()), 0.);
                for (w, f) in widths.iter_mut().zip(fresh) {
                    *w = w.max(f);
                }
                widths
            }
            _ => {
                self.table_dirty.remove(&id);
                table(rows.iter().map(|r| r.cells.clone()).collect()).measure_column_widths(
                    &self.render_cx,
                    window,
                    cx,
                )
            }
        };
        let count = match &self.doc.blocks[bix].kind {
            Kind::Table { rows, .. } => rows.len(),
            _ => 0,
        };
        let measured = Arc::new(measured);
        self.table_widths.insert(id, (count, measured.clone()));
        measured
    }

    fn render_row(&mut self, ix: usize, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let Some(row) = self.rows.get(ix).cloned() else {
            return div().into_any_element();
        };
        let bix = row.block;
        if bix >= self.doc.blocks.len() {
            return div().into_any_element();
        }
        self.perf.borrow_mut().block_laid_out();
        let from = bix.saturating_sub(3);
        let refs: Vec<&BlockNode> = self.doc.blocks[from..=bix]
            .iter()
            .map(|b| &b.node)
            .collect();
        let flow =
            FlowPosition::at(&refs, bix - from, bix + 1 == self.doc.blocks.len()).first(bix == 0);
        let Some(part) = row.part else {
            let block = &self.doc.blocks[bix];
            return div()
                .w_full()
                .child(block.node.render_with_id(
                    ElementId::Integer(block.id),
                    flow,
                    &self.render_cx,
                    window,
                    cx,
                ))
                .into_any_element();
        };
        // A slice of a long list or table: no gap between slices.
        let total = match &self.doc.blocks[bix].kind {
            Kind::List { items } => items.len(),
            Kind::Table { rows, .. } => rows.len(),
            _ => 0,
        };
        let mut flow = flow;
        if part.start > 0 {
            flow = flow.first(true);
        }
        if part.end < total {
            flow = flow.last(true);
        }
        let node = match (&self.doc.blocks[bix].node, &self.doc.blocks[bix].kind) {
            (
                BlockNode::List {
                    children,
                    ordered,
                    start,
                    ..
                },
                _,
            ) => BlockNode::List {
                children: children[part.clone()].to_vec(),
                ordered: *ordered,
                start: if *ordered {
                    Some(start.unwrap_or(1) + part.start as u32)
                } else {
                    *start
                },
                span: None,
            },
            (_, Kind::Table { .. }) => {
                let widths = self.table_widths(bix, window, cx);
                let Kind::Table { rows, aligns } = &self.doc.blocks[bix].kind else {
                    return div().into_any_element();
                };
                BlockNode::Table(
                    Table::new(
                        rows[part.clone()].iter().map(|r| r.cells.clone()).collect(),
                        aligns.clone(),
                    )
                    .chunk(
                        part.start == 0,
                        part.start == 0,
                        part.end == total,
                        widths,
                    ),
                )
            }
            _ => self.doc.blocks[bix].node.clone(),
        };
        let id = self.doc.blocks[bix].id;
        // GPUI Kit spaces list items after the first by a quarter of the
        // body size; a slice that continues a list keeps that spacing.
        let continues_list = part.start > 0 && matches!(node, BlockNode::List { .. });
        div()
            .w_full()
            .when(continues_list, |row| row.pt(px(BODY_SIZE * 0.25)))
            .child(node.render_with_id(
                ElementId::Integer((id << 20) | part.start as u64),
                flow,
                &self.render_cx,
                window,
                cx,
            ))
            .into_any_element()
    }

    /// Decorations: the IME's marked text.
    fn decorations(&self, cx: &App) -> HashMap<u64, Vec<(Range<usize>, Hsla)>> {
        let mut out = HashMap::new();
        if let Some(marked) = &self.marked
            && let (Some((b, l, s)), Some((_, _, e))) =
                (self.doc.locate(marked.start), self.doc.locate(marked.end))
        {
            let key = self.doc.blocks[b].leaves[l].key;
            out.insert(key, vec![(s..e.max(s), cx.theme().selection)]);
        }
        out
    }

    /// The `ENSO_PERF=1` overlay: frame-time graph against both budgets,
    /// frames over budget, and the last input's work and reparse.
    fn perf_overlay(&self, cx: &App) -> Option<AnyElement> {
        let perf = self.perf.borrow();
        if !perf.overlay {
            return None;
        }
        let theme = cx.theme();
        let budget_120 = Duration::from_micros(8_333);
        let budget_60 = Duration::from_micros(16_667);
        let frames = &perf.frames[perf.frames.len().saturating_sub(120)..];
        let scale = 2.4; // px per ms
        let bars = frames.iter().map(|f| {
            let height = (ms(*f) as f32 * scale).min(60.);
            let color = if *f > budget_60 {
                theme.danger
            } else if *f > budget_120 {
                theme.warning
            } else {
                theme.success
            };
            div().w(px(2.)).h(px(height)).bg(color)
        });
        let last = perf.samples.last();
        let line = |s: String| div().child(s);
        Some(
            div()
                .absolute()
                .top_2()
                .right_2()
                .p_2()
                .rounded_md()
                .bg(theme.popover)
                .border_1()
                .border_color(theme.border)
                .text_xs()
                .font_family("monospace")
                .child(
                    div()
                        .h(px(60.))
                        .w(px(240.))
                        .flex()
                        .items_end()
                        .gap(px(0.))
                        .children(bars),
                )
                .child(line(format!(
                    "over 8.3 ms: {}  over 16.7 ms: {}",
                    perf.over_budget(budget_120),
                    perf.over_budget(budget_60)
                )))
                .child(line(match last {
                    Some(s) => format!(
                        "last input: work {:.2} ms (handler {:.2} + frame {:.2})",
                        ms(s.work()),
                        ms(s.handler),
                        ms(s.frame)
                    ),
                    None => "last input: none".into(),
                }))
                .child(line(match last {
                    Some(s) => format!(
                        "reparsed {} B ({:?}), {} blocks laid out",
                        s.parsed_bytes, s.path, s.blocks_laid_out
                    ),
                    None => String::new(),
                }))
                .into_any_element(),
        )
    }
}

pub fn next_boundary(text: &str, index: usize) -> usize {
    text[index..]
        .grapheme_indices(true)
        .nth(1)
        .map_or(text.len(), |(i, _)| index + i)
}

pub fn prev_boundary(text: &str, index: usize) -> usize {
    text[..index]
        .grapheme_indices(true)
        .next_back()
        .map_or(0, |(i, _)| i)
}

impl Focusable for VisualView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for VisualView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.perf.borrow_mut().frame_begin();
        let theme = cx.theme();
        let list = list(self.list.clone(), cx.processor(Self::render_row))
            .w_full()
            .h_full();
        let loading = self.background.loading.then(|| {
            // A quiet note while the rest of a long file parses.
            div()
                .absolute()
                .bottom_2()
                .left_0()
                .right_0()
                .flex()
                .justify_center()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child("Preparing the rest of the document…")
        });
        let page = div()
            .size_full()
            .flex()
            .justify_center()
            .child(div().w(px(680.)).max_w_full().h_full().px_4().child(list));
        div()
            .id("visual-editor")
            .key_context(CONTEXT)
            .track_focus(&self.focus)
            .relative()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .capture_key_down(cx.listener(|this, _: &KeyDownEvent, _, _| {
                this.perf.borrow_mut().input();
            }))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_action(cx.listener(|this, _: &MoveLeft, _, cx| this.move_horizontal(false, cx)))
            .on_action(cx.listener(|this, _: &MoveRight, _, cx| this.move_horizontal(true, cx)))
            .on_action(cx.listener(|this, _: &MoveUp, _, cx| this.move_vertical(false, cx)))
            .on_action(cx.listener(|this, _: &MoveDown, _, cx| this.move_vertical(true, cx)))
            .on_action(cx.listener(|this, _: &MoveLineStart, _, cx| this.line_edge(false, cx)))
            .on_action(cx.listener(|this, _: &MoveLineEnd, _, cx| this.line_edge(true, cx)))
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::enter))
            .on_action(cx.listener(|this, _: &Undo, window, cx| this.replay(true, window, cx)))
            .on_action(cx.listener(|this, _: &Redo, window, cx| this.replay(false, window, cx)))
            .child(Surface {
                view: cx.entity(),
                child: Some(page.into_any_element()),
                decorations: self.decorations(cx),
            })
            .children(loading)
            .children(self.perf_overlay(cx))
    }
}

/// The root of the page: prepaints the blocks inside a geometry collection
/// scope, paints the caret over them and takes platform text input.
struct Surface {
    view: Entity<VisualView>,
    child: Option<AnyElement>,
    decorations: HashMap<u64, Vec<(Range<usize>, Hsla)>>,
}

impl IntoElement for Surface {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for Surface {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, AnyElement) {
        let mut child = self.child.take().expect("surface child");
        let child_layout = child.request_layout(window, cx);
        let style = Style {
            size: size(relative(1.).into(), relative(1.).into()),
            ..Default::default()
        };
        (window.request_layout(style, [child_layout], cx), child)
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        child: &mut AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) {
        let decorations = std::mem::take(&mut self.decorations);
        let (_, geometry) = collect_leaf_geometry(decorations, || child.prepaint(window, cx));
        self.view.update(cx, |view, cx| {
            *view.geometry.borrow_mut() = geometry;
            if let Some((bix, tries)) = view.pending_reveal.take()
                && tries > 0
                && !view.reveal_caret_step(bix)
            {
                view.pending_reveal = Some((bix, tries - 1));
                cx.notify();
            }
            if let Some(down) = view.pending_vertical.take() {
                if !view.try_vertical(down) {
                    view.pending_vertical = Some(down);
                }
                cx.notify();
            }
        });
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        child: &mut AnyElement,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        child.paint(window, cx);
        let view = self.view.read(cx);
        let focused = view.focus.is_focused(window);
        if focused
            && view.caret_blink.visible()
            && let Some((top, line_height, caret_height)) = view.caret_position()
        {
            let y = top.y + (line_height - caret_height) / 2.;
            let caret = Bounds::new(point(top.x, y), size(px(2.), caret_height));
            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                window.paint_quad(fill(caret, cx.theme().caret));
            });
        }
        let perf = view.perf.clone();
        let focus = view.focus.clone();
        window.handle_input(
            &focus,
            ElementInputHandler::new(bounds, self.view.clone()),
            cx,
        );
        perf.borrow_mut().frame_end();
    }
}
