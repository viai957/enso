//! The visual surface: renders the block index through `editor_host`, hit
//! tests, moves the caret and scrolls.

use std::{
    cell::RefCell,
    collections::HashMap,
    ops::Range,
    rc::Rc,
    time::{Duration, Instant},
};

use gpui_kit::base::text::editor_host::{
    BlockNode, FlowPosition, LeafGeometry, RenderContext, collect_leaf_geometry,
};
use gpui_kit::component::{ActiveTheme as _, input::EditorState};
use gpui_kit::*;
use unicode_segmentation::UnicodeSegmentation as _;

use super::caret::Caret;
use super::document::{VisualDocument, key_block_id, key_leaf_ix, leaf_key};

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

pub fn init(cx: &mut App) {
    let (undo, redo) = if cfg!(target_os = "macos") {
        ("cmd-z", "cmd-shift-z")
    } else {
        ("ctrl-z", "ctrl-shift-z")
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
    ]);
}

/// Keystroke-to-paint timing: from the key event reaching the surface to the
/// end of the paint that shows its effect.
#[derive(Default)]
pub struct Latency {
    pub pending: Option<Instant>,
    /// When the frame that will show the pending input started rendering.
    pub frame_start: Option<Instant>,
    pub samples: Vec<Duration>,
    /// Of each sample: waiting for the frame, and the frame's own work
    /// (render, layout, prepaint, paint) up to the end of the surface's paint.
    pub wait: Vec<Duration>,
    pub frame: Vec<Duration>,
    /// Time spent in `EditorState::replace` and in the reparse, per edit.
    pub replace: Vec<Duration>,
    pub reparse: Vec<Duration>,
    pub parsed_bytes: Vec<usize>,
}

pub fn summary(name: &str, samples: &[Duration]) -> String {
    if samples.is_empty() {
        return format!("{name}: no samples");
    }
    let mut ms: Vec<f64> = samples.iter().map(|d| d.as_secs_f64() * 1000.).collect();
    ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q = |p: f64| ms[((ms.len() - 1) as f64 * p).round() as usize];
    let mean = ms.iter().sum::<f64>() / ms.len() as f64;
    format!(
        "{name}: n={} min={:.2} p50={:.2} p90={:.2} p95={:.2} p99={:.2} max={:.2} mean={:.2} ms",
        ms.len(),
        ms[0],
        q(0.5),
        q(0.9),
        q(0.95),
        q(0.99),
        ms[ms.len() - 1],
        mean
    )
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
    pub focus: FocusHandle,
    pub latency: Rc<RefCell<Latency>>,
    pub caret_blink: Caret,
    render_cx: RenderContext,
}

impl VisualView {
    pub fn new(editor: Entity<EditorState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let text = editor.read(cx).text().to_string();
        let doc = VisualDocument::parse(&text);
        let list = ListState::new(doc.blocks.len(), ListAlignment::Top, px(600.));
        let render_cx = RenderContext::from_app(cx).with_body_font_size(px(17.));
        let focus = cx.focus_handle();
        let caret = doc
            .blocks
            .iter()
            .find(|b| !b.leaves.is_empty())
            .map_or(0, |b| b.start + b.leaves[0].leaf_to_src(0));
        let _ = window;
        Self {
            editor,
            doc,
            list,
            caret,
            goal_x: None,
            marked: None,
            geometry: Rc::default(),
            pending_vertical: None,
            focus,
            latency: Rc::default(),
            caret_blink: Caret::new(cx),
            render_cx,
        }
    }

    // ----- editing -------------------------------------------------------

    /// Replaces source `range` with `text` through the editor state (one undo
    /// step, the save path sees it), then reparses the touched blocks.
    pub fn edit(
        &mut self,
        range: Range<usize>,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let started = Instant::now();
        self.editor.update(cx, |state, cx| {
            state.set_selected_range(range.clone(), cx);
            state.replace(text.to_string(), window, cx);
        });
        let replaced = Instant::now();
        let splice = {
            let rope = self.editor.read(cx).text();
            self.doc.edit(rope, range.clone(), text.len())
        };
        let reparsed = Instant::now();
        self.list.splice(splice.old.clone(), splice.new_len);
        {
            let mut latency = self.latency.borrow_mut();
            latency.replace.push(replaced - started);
            latency.reparse.push(reparsed - replaced);
            latency.parsed_bytes.push(splice.parsed_bytes);
        }
        self.caret = range.start + text.len();
        self.goal_x = None;
        self.after_caret_change(cx);
    }

    /// Re-reads the whole document from the editor state, after an undo or
    /// redo whose extent the state does not report.
    fn resync(&mut self, cx: &mut Context<Self>) {
        let text = self.editor.read(cx).text().to_string();
        let old = self.doc.blocks.len();
        self.doc = VisualDocument::parse(&text);
        self.list.splice(0..old, self.doc.blocks.len());
        self.caret = self.editor.read(cx).cursor().min(text.len());
        self.after_caret_change(cx);
    }

    fn after_caret_change(&mut self, cx: &mut Context<Self>) {
        self.caret_blink.reset(cx);
        if let Some((bix, _, _)) = self.doc.locate(self.caret) {
            self.list.scroll_to_reveal_item(bix);
        }
        cx.notify();
    }

    fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        let Some((bix, lix, index)) = self.doc.locate(self.caret) else {
            return;
        };
        let leaf = &self.doc.blocks[bix].leaves[lix];
        if index > 0 {
            let prev = prev_boundary(&leaf.text, index);
            let start = self.doc.src_of(bix, lix, prev);
            // The caret's own source position: delete back to the previous
            // grapheme, whatever markers lie between.
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
        let Some((bix, lix, index)) = self.doc.locate(self.caret) else {
            return;
        };
        let leaf = &self.doc.blocks[bix].leaves[lix];
        if index < leaf.len() {
            let next = next_boundary(&leaf.text, index);
            let end = self.doc.src_of(bix, lix, next);
            if end > self.caret {
                self.edit(self.caret..end, "", window, cx);
            }
        }
    }

    fn enter(&mut self, _: &Enter, window: &mut Window, cx: &mut Context<Self>) {
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

    fn undo(&mut self, _: &Undo, window: &mut Window, cx: &mut Context<Self>) {
        let focus = self.editor.read(cx).focus_handle(cx);
        focus.dispatch_action(&gpui_kit::component::input::Undo, window, cx);
        self.resync(cx);
    }

    fn redo(&mut self, _: &Redo, window: &mut Window, cx: &mut Context<Self>) {
        let focus = self.editor.read(cx).focus_handle(cx);
        focus.dispatch_action(&gpui_kit::component::input::Redo, window, cx);
        self.resync(cx);
    }

    // ----- caret movement -------------------------------------------------

    fn move_horizontal(&mut self, forward: bool, cx: &mut Context<Self>) {
        let Some((bix, lix, index)) = self.doc.locate(self.caret) else {
            return;
        };
        let leaf = &self.doc.blocks[bix].leaves[lix];
        let target = if forward && index < leaf.len() {
            Some((bix, lix, next_boundary(&leaf.text, index)))
        } else if !forward && index > 0 {
            Some((bix, lix, prev_boundary(&leaf.text, index)))
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

    /// The window position and line height of the caret, from the last frame.
    pub fn caret_position(&self) -> Option<(Point<Pixels>, Pixels, Pixels)> {
        let (bix, lix, index) = self.doc.locate(self.caret)?;
        let key = leaf_key(self.doc.blocks[bix].id, lix);
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
        // Scan away from the caret line until a leaf has a line there.
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
                if moved {
                    let block = self
                        .doc
                        .blocks
                        .iter()
                        .position(|b| b.id == key_block_id(key));
                    if let Some(bix) = block {
                        self.caret = self.doc.src_of(bix, key_leaf_ix(key), index);
                        return true;
                    }
                }
            }
            y += if down { px(2.) } else { px(-2.) };
        }
        false
    }

    fn move_vertical(&mut self, down: bool, cx: &mut Context<Self>) {
        if self.try_vertical(down) {
            self.pending_vertical = None;
            self.caret_blink.reset(cx);
            cx.notify();
            return;
        }
        // Bring more of the document in and retry after the next layout.
        let step = self.caret_position().map_or(px(40.), |(_, h, _)| h * 2.);
        self.list.scroll_by(if down { step } else { -step });
        self.pending_vertical = Some(down);
        cx.notify();
    }

    fn line_edge(&mut self, end: bool, cx: &mut Context<Self>) {
        let Some((top, line_height, _)) = self.caret_position() else {
            return;
        };
        let Some((bix, lix, _)) = self.doc.locate(self.caret) else {
            return;
        };
        let key = leaf_key(self.doc.blocks[bix].id, lix);
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
        let bix = self
            .doc
            .blocks
            .iter()
            .position(|b| b.id == key_block_id(key))?;
        Some(self.doc.src_of(bix, key_leaf_ix(key), index))
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.latency
            .borrow_mut()
            .pending
            .get_or_insert_with(Instant::now);
        window.focus(&self.focus, cx);
        if let Some(src) = self.source_for_point(event.position) {
            self.caret = src;
            self.marked = None;
            self.goal_x = None;
            self.caret_blink.reset(cx);
            cx.notify();
        }
    }

    // ----- rendering --------------------------------------------------------

    fn render_block(
        &mut self,
        ix: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(block) = self.doc.blocks.get(ix) else {
            return div().into_any_element();
        };
        let from = ix.saturating_sub(3);
        let refs: Vec<&BlockNode> = self.doc.blocks[from..=ix].iter().map(|b| &b.node).collect();
        let flow =
            FlowPosition::at(&refs, ix - from, ix + 1 == self.doc.blocks.len()).first(ix == 0);
        div()
            .w_full()
            .child(block.node.render_with_id(
                ElementId::Integer(block.id),
                flow,
                &self.render_cx,
                window,
                cx,
            ))
            .into_any_element()
    }

    /// Selection-like decorations: the IME's marked text.
    fn decorations(&self, cx: &App) -> HashMap<u64, Vec<(Range<usize>, Hsla)>> {
        let mut out = HashMap::new();
        if let Some(marked) = &self.marked
            && let (Some((b, l, s)), Some((_, _, e))) =
                (self.doc.locate(marked.start), self.doc.locate(marked.end))
        {
            let key = leaf_key(self.doc.blocks[b].id, l);
            out.insert(key, vec![(s..e.max(s), cx.theme().selection)]);
        }
        out
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
        {
            let mut latency = self.latency.borrow_mut();
            if latency.pending.is_some() && latency.frame_start.is_none() {
                latency.frame_start = Some(Instant::now());
            }
        }
        let theme = cx.theme();
        let list = list(self.list.clone(), cx.processor(Self::render_block))
            .w_full()
            .h_full();
        let page = div()
            .size_full()
            .flex()
            .justify_center()
            .child(div().w(px(680.)).max_w_full().h_full().px_4().child(list));
        div()
            .id("visual-editor")
            .key_context(CONTEXT)
            .track_focus(&self.focus)
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .capture_key_down(cx.listener(|this, _: &KeyDownEvent, _, _| {
                this.latency
                    .borrow_mut()
                    .pending
                    .get_or_insert_with(Instant::now);
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
            .on_action(cx.listener(Self::undo))
            .on_action(cx.listener(Self::redo))
            .child(Surface {
                view: cx.entity(),
                child: Some(page.into_any_element()),
                decorations: self.decorations(cx),
            })
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
        let mut style = Style::default();
        style.size = size(relative(1.).into(), relative(1.).into());
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
        let latency = view.latency.clone();
        let focus = view.focus.clone();
        window.handle_input(
            &focus,
            ElementInputHandler::new(bounds, self.view.clone()),
            cx,
        );
        let mut latency = latency.borrow_mut();
        if let Some(started) = latency.pending.take() {
            let now = Instant::now();
            latency.samples.push(now - started);
            if let Some(frame) = latency.frame_start.take() {
                latency.wait.push(frame - started);
                latency.frame.push(now - frame);
            }
        }
    }
}
