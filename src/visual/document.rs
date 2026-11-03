//! An incremental index of the document's top-level blocks.
//!
//! The Markdown text (the editor's rope) stays the only source of truth.
//! Each top-level block keeps the [`BlockNode`] GPUI Kit renders and the
//! source mapping of every text leaf in it.
//!
//! An edit takes the cheapest path that is provably equivalent to a whole
//! parse, in this order:
//!
//! 1. **Fence body**: text typed inside a fenced code block (closed or not)
//!    that does not form a closing fence only changes the code text.
//! 2. **List item**: an edit inside one item of a top-level list reparses
//!    that item and its two neighbours, which must come back unchanged.
//! 3. **Table row**: an edit inside one body row, with no line break,
//!    reparses the header, the delimiter row and that row.
//! 4. **Isolated blocks**: the touched blocks alone, when blank lines
//!    separate them from their neighbours and nothing in them can reach
//!    across a blank line (fences, HTML blocks, lists, indented lines).
//! 5. **Neighbours**: the touched blocks plus one block on each side, which
//!    must come back unchanged; widened up to three times.
//!
//! When none of these applies within [`MAX_SYNC_BYTES`], the edit is applied
//! provisionally (the touched region parsed on its own, so the typed text
//! shows at once) and [`EditResult::needs_full`] asks the caller for a
//! background whole-document parse, merged by [`VisualDocument::reconcile`].

use std::{
    collections::HashMap,
    ops::Range,
    sync::atomic::{AtomicU64, Ordering},
};

use gpui_kit::base::input::{Rope, RopeExt as _};
use gpui_kit::base::text::editor_host::{
    BlockNode, CodeBlock, ColumnAlign, ImageNode, InlineNode, LinkMark, Paragraph, Span, Table,
    TextMark,
};
use markdown::mdast::{AlignKind, Node};

use super::mapping::{LeafBuilder, LeafMap};

/// The most source a keystroke may parse on the main thread. At about
/// 0.76 µs per byte (release build, Phase 0), 2 KiB stays well inside a
/// 120 Hz frame together with layout and paint.
pub const MAX_SYNC_BYTES: usize = 2 * 1024;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Set when a parse fell back to [`plain_blocks`]; an edit that saw it asks
/// for a whole parse.
static PARSE_FAILED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// A process-wide unique id for a block or a leaf. Usable from any thread, so
/// background parses produce keys that never collide with live ones.
fn fresh_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// What a block is, beyond its rendered node, for the edit fast paths.
#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    Other,
    /// A top-level fenced code block whose opening fence is not indented.
    Fence {
        /// Block-relative content: after the opening line, up to the start
        /// of the closing fence line (or the block end when unclosed).
        body: Range<usize>,
        closed: bool,
        ch: u8,
        len: usize,
        lang: Option<String>,
        meta: Option<String>,
    },
    /// A top-level list: each item's block-relative span and its leaves.
    List {
        items: Vec<Item>,
    },
    /// A top-level table: rows (row 0 is the header) with their spans,
    /// cells and leaves.
    Table {
        rows: Vec<Row>,
        aligns: Vec<ColumnAlign>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub span: Range<usize>,
    pub leaves: Range<usize>,
}

#[derive(Clone, Debug)]
pub struct Row {
    pub span: Range<usize>,
    pub cells: Vec<Paragraph>,
    pub leaves: Range<usize>,
}

impl PartialEq for Row {
    fn eq(&self, other: &Self) -> bool {
        self.span == other.span && self.leaves == other.leaves
    }
}

/// One top-level block.
pub struct Block {
    /// Stable while the block is edited in place; a reparse that replaces
    /// blocks positionally hands the old ids on.
    pub id: u64,
    /// Absolute source range.
    pub start: usize,
    pub end: usize,
    pub node: BlockNode,
    /// The block's text leaves in document order, offsets relative to `start`.
    pub leaves: Vec<LeafMap>,
    pub kind: Kind,
    /// Block-relative spans of the link reference definitions in the block,
    /// at any depth.
    pub defs: Vec<Range<usize>>,
}

impl Block {
    fn shift(&mut self, delta: isize) {
        self.start = add(self.start, delta);
        self.end = add(self.end, delta);
    }

    /// Whether `other` renders and maps the same, wherever it sits.
    fn same_content(&self, other: &Block) -> bool {
        self.end - self.start == other.end - other.start
            && self.node == other.node
            && self.leaves.len() == other.leaves.len()
            && self
                .leaves
                .iter()
                .zip(&other.leaves)
                .all(|(a, b)| a.text == b.text && a.segs == b.segs)
    }
}

fn add(offset: usize, delta: isize) -> usize {
    (offset as isize + delta) as usize
}

/// Which path an edit took (for tests and the performance overlay).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Path {
    FenceBody,
    ListItem,
    TableRow,
    Isolated,
    Neighbours,
    Provisional,
}

/// What an edit did to the block list: `old` blocks were replaced by
/// `new_len` blocks starting at `old.start`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Splice {
    pub old: Range<usize>,
    pub new_len: usize,
}

#[derive(Debug, Clone)]
pub struct EditResult {
    pub splice: Splice,
    pub path: Path,
    /// Bytes handed to the parser on this thread.
    pub parsed_bytes: usize,
    /// The result is provisional: a whole-document parse must reconcile it.
    pub needs_full: bool,
}

#[derive(Default)]
pub struct VisualDocument {
    pub blocks: Vec<Block>,
    /// Leaf key → block id, for geometry lookups.
    leaf_block: HashMap<u64, u64>,
    /// Block id → index in `blocks`.
    block_ix: HashMap<u64, usize>,
    /// The source of every top-level link reference definition, prepended to
    /// window parses.
    definitions: String,
    /// A splice added or removed a definition.
    definitions_touched: bool,
}

pub fn parse_options() -> markdown::ParseOptions {
    options_at(true)
}

/// `markdown::to_mdast`, with its panics (markdown 1.0.0 panics on some
/// inputs, e.g. "Cannot push to non-parent") turned into `None`.
fn to_mdast_safe(text: &str, options: &markdown::ParseOptions) -> Option<Node> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        markdown::to_mdast(text, options).ok()
    }))
    .ok()
    .flatten()
}

/// Blocks for text the parser cannot handle: one plain paragraph per run of
/// non-blank lines, mapped character for character, so the document still
/// shows and stays editable.
fn plain_blocks(text: &str, base: usize) -> Vec<Block> {
    PARSE_FAILED.store(true, Ordering::Relaxed);
    let mut blocks = Vec::new();
    let mut offset = 0;
    for chunk in text.split_inclusive("\n\n") {
        let body = chunk.trim_end_matches('\n');
        if !body.trim().is_empty() {
            let mut b = LeafBuilder::default();
            b.push_aligned(body, body, 0);
            let key = fresh_id();
            let mut paragraph = Paragraph::default();
            paragraph.push(InlineNode::new(b.text.clone()));
            blocks.push(Block {
                id: fresh_id(),
                start: base + offset,
                end: base + offset + body.len(),
                node: BlockNode::Paragraph(paragraph.with_leaf_key(key)),
                leaves: vec![LeafMap {
                    key,
                    text: b.text,
                    segs: b.segs,
                    empty_src: 0,
                }],
                kind: Kind::Other,
                defs: Vec::new(),
            });
        }
        offset += chunk.len();
    }
    blocks
}

/// Front matter can only open at the very start of the document; a window
/// parsed elsewhere must not read a leading `---` as front matter.
fn options_at(document_start: bool) -> markdown::ParseOptions {
    let mut options = markdown::ParseOptions::gfm();
    options.constructs.frontmatter = document_start;
    options
}

/// Parses `text`, which starts at absolute offset `base`, into blocks with
/// fresh ids. Safe to run on a background thread.
pub fn parse_blocks(text: &str, base: usize) -> Vec<Block> {
    parse_blocks_reusing(text, base, Vec::new(), "")
}

/// Parses `text` behind `prelude`, the document's link reference
/// definitions, so that references in a window resolve exactly as in a whole
/// parse. Returns the parsed string, the prelude's length in it, and the root
/// children that belong to `text`.
fn parse_with_prelude(
    text: &str,
    prelude: &str,
    document_start: bool,
) -> Option<(String, usize, Vec<Node>)> {
    let front_matter = document_start && (text.starts_with("---") || text.starts_with("+++"));
    if !prelude.is_empty() && front_matter {
        // Front matter must stay first: append the definitions instead, and
        // drop them again unless a block ran into them (then parse without).
        let full = format!("{text}\n\n{prelude}");
        let Some(Node::Root(root)) = to_mdast_safe(&full, &options_at(true)) else {
            return None;
        };
        if root.children.iter().any(|c| {
            c.position()
                .is_some_and(|p| p.start.offset < text.len() && p.end.offset > text.len())
        }) {
            return parse_with_prelude(text, "", document_start);
        }
        let children = root
            .children
            .into_iter()
            .filter(|c| c.position().is_some_and(|p| p.start.offset < text.len()))
            .collect();
        return Some((full, 0, children));
    }
    let (full, plen) = if prelude.is_empty() {
        (text.to_string(), 0)
    } else {
        let head = format!("{prelude}\n\n");
        let plen = head.len();
        (head + text, plen)
    };
    let options = options_at(document_start && plen == 0);
    let Some(Node::Root(root)) = to_mdast_safe(&full, &options) else {
        return None;
    };
    let children = root
        .children
        .into_iter()
        .filter(|c| c.position().is_some_and(|p| p.start.offset >= plen))
        .collect();
    Some((full, plen, children))
}

/// [`parse_blocks`] behind a definitions `prelude`, handing `keys` (from the
/// blocks being replaced) on to the new leaves in order.
fn parse_blocks_reusing(text: &str, base: usize, keys: Vec<u64>, prelude: &str) -> Vec<Block> {
    let mut reuse = keys.into_iter();
    let Some((full, plen, children)) = parse_with_prelude(text, prelude, base == 0) else {
        return plain_blocks(text, base);
    };
    let mut blocks = Vec::with_capacity(children.len());
    for child in &children {
        let Some(position) = child.position() else {
            continue;
        };
        let (start, end) = (position.start.offset, position.end.offset);
        let mut cx = BuildCx {
            source: &full,
            reuse: &mut reuse,
            block_start: start,
            leaves: Vec::new(),
            depth: 0,
            kind: Kind::Other,
            defs: Vec::new(),
        };
        let Some(node) = cx.block(child) else {
            continue;
        };
        blocks.push(Block {
            id: fresh_id(),
            start: base + start - plen,
            end: base + end - plen,
            node,
            leaves: cx.leaves,
            kind: cx.kind,
            defs: cx.defs,
        });
    }
    blocks
}

/// Where to cut `text` for a quick first parse of about `target` bytes: just
/// after a blank line, outside any code fence. Returns `text.len()` when the
/// text is short or no such line exists.
pub fn prefix_cut(text: &str, target: usize) -> usize {
    if text.len() <= target {
        return text.len();
    }
    let mut fence: Option<(u8, usize)> = None;
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let indent = line.bytes().take_while(|&b| b == b' ').count();
        let trimmed = &line[indent..];
        let run = |c: u8| trimmed.bytes().take_while(|&b| b == c).count();
        match fence {
            Some((c, n)) => {
                if indent < 4 && run(c) >= n && trimmed[run(c)..].trim().is_empty() {
                    fence = None;
                }
            }
            None => {
                for c in *b"`~" {
                    if indent < 4 && run(c) >= 3 {
                        fence = Some((c, run(c)));
                    }
                }
            }
        }
        offset += line.len();
        if offset >= target && line.trim().is_empty() && fence.is_none() {
            return offset;
        }
    }
    text.len()
}

impl VisualDocument {
    pub fn parse(text: &str) -> Self {
        Self::from_blocks(parse_blocks(text, 0), &Rope::from(text))
    }

    /// A document of `blocks` parsed from `text`.
    pub fn from_blocks(blocks: Vec<Block>, text: &Rope) -> Self {
        let mut doc = Self {
            blocks,
            ..Default::default()
        };
        doc.reindex_all();
        doc.refresh_definitions(text);
        doc
    }

    pub fn into_blocks(self) -> Vec<Block> {
        self.blocks
    }

    fn refresh_definitions(&mut self, text: &Rope) {
        let mut out = String::new();
        for block in &self.blocks {
            for def in &block.defs {
                let range = block.start + def.start..block.start + def.end;
                if range.end <= text.len() {
                    if !out.is_empty() {
                        out.push_str("\n\n");
                    }
                    out.push_str(&text.slice(range).to_string());
                }
            }
        }
        self.definitions = out;
    }

    fn reindex_all(&mut self) {
        self.leaf_block.clear();
        for block in &self.blocks {
            for leaf in &block.leaves {
                self.leaf_block.insert(leaf.key, block.id);
            }
        }
        self.reindex_blocks();
    }

    fn reindex_blocks(&mut self) {
        self.block_ix.clear();
        for (ix, block) in self.blocks.iter().enumerate() {
            self.block_ix.insert(block.id, ix);
        }
    }

    /// Replaces `range` with `new` blocks, handing the replaced ids on
    /// positionally so blocks edited in place keep their element identity.
    fn splice_blocks(&mut self, range: Range<usize>, mut new: Vec<Block>) -> Splice {
        let is_def = |b: &Block| !b.defs.is_empty();
        if self.blocks[range.clone()].iter().any(is_def) || new.iter().any(is_def) {
            self.definitions_touched = true;
        }
        for block in &self.blocks[range.clone()] {
            for leaf in &block.leaves {
                self.leaf_block.remove(&leaf.key);
            }
        }
        for (block, old) in new.iter_mut().zip(&self.blocks[range.clone()]) {
            block.id = old.id;
        }
        for block in &new {
            for leaf in &block.leaves {
                self.leaf_block.insert(leaf.key, block.id);
            }
        }
        let new_len = new.len();
        let count_changed = new_len != range.len();
        self.blocks.splice(range.clone(), new);
        if count_changed {
            self.reindex_blocks();
        }
        Splice {
            old: range,
            new_len,
        }
    }

    /// Parses `range` of `rope` with the first line after it as context, as a
    /// whole parse sees it (a list's trailing blank lines belong to it only
    /// when more text follows), keeping the blocks that start inside the
    /// range. The flag is false when a kept block runs into the context line.
    fn parse_in_context(
        &self,
        rope: &Rope,
        range: Range<usize>,
        keys: Vec<u64>,
    ) -> (Vec<Block>, bool) {
        let end = if range.end < rope.len() {
            let row = rope.offset_to_point(range.end).row;
            rope.line_end_offset(row).max(range.end)
        } else {
            range.end
        };
        let window = rope.slice(range.start..end).to_string();
        let mut blocks = parse_blocks_reusing(&window, range.start, keys, &self.definitions);
        blocks.retain(|b| b.start < range.end || range.start == range.end);
        let contained = blocks.iter().all(|b| b.end <= range.end);
        (blocks, contained)
    }

    /// The leaf keys of `range`, in order, to hand on to their replacements.
    fn keys_of(&self, range: Range<usize>) -> Vec<u64> {
        self.blocks[range]
            .iter()
            .flat_map(|b| b.leaves.iter().map(|l| l.key))
            .collect()
    }

    /// Block index and leaf index of a leaf key from the last frame.
    pub fn leaf_by_key(&self, key: u64) -> Option<(usize, usize)> {
        let bix = *self.block_ix.get(self.leaf_block.get(&key)?)?;
        let lix = self.blocks[bix].leaves.iter().position(|l| l.key == key)?;
        Some((bix, lix))
    }

    /// Brings the index up to date with `rope`, where `old` (a range of the
    /// previous text) was replaced by `new_len` bytes.
    pub fn edit(&mut self, rope: &Rope, old: Range<usize>, new_len: usize) -> EditResult {
        self.definitions_touched = false;
        PARSE_FAILED.store(false, Ordering::Relaxed);
        let mut result = self.edit_inner(rope, old, new_len);
        if PARSE_FAILED.swap(false, Ordering::Relaxed) {
            result.needs_full = true;
        }
        if self.definitions_touched {
            // A definition resolves references anywhere in the document.
            self.refresh_definitions(rope);
            result.needs_full = true;
        }
        if self.unclosed_front_matter(rope) {
            // A document that opens with `---` (or `+++`) that is not closed
            // front matter: the parser's state after the failed front matter
            // affects blocks anywhere below (markdown 1.0.0), which no local
            // window reproduces. Keep the local result, settle in the
            // background.
            result.needs_full = true;
        }
        result
    }

    /// Whether the text opens with a front matter fence that does not form
    /// front matter.
    fn unclosed_front_matter(&self, rope: &Rope) -> bool {
        let first = rope.slice_line(0).to_string();
        let first = first.trim_end();
        (first == "---" || first == "+++")
            && !self.blocks.first().is_some_and(|b| {
                b.start == 0
                    && matches!(&b.node, BlockNode::CodeBlock(c)
                        if matches!(c.lang().as_deref(), Some("yaml" | "toml")))
            })
    }

    fn edit_inner(&mut self, rope: &Rope, old: Range<usize>, new_len: usize) -> EditResult {
        let delta = new_len as isize - old.len() as isize;
        let n = self.blocks.len();
        // Blocks the edit touches (an edit at a block's edge touches it).
        let first = self.blocks.partition_point(|b| b.end < old.start);
        let last = self.blocks.partition_point(|b| b.start <= old.end);
        if first + 1 == last {
            let result = match self.blocks[first].kind {
                Kind::Fence { .. } => {
                    self.edit_fence(first, rope, &old, new_len, false)
                        .or_else(|| {
                            // A fence too large to reparse here: keep showing it
                            // as code with the edit applied, and let the
                            // background parse settle its new structure.
                            let block = &self.blocks[first];
                            (block.end - block.start > MAX_SYNC_BYTES)
                                .then(|| self.edit_fence(first, rope, &old, new_len, true))
                                .flatten()
                        })
                }
                Kind::List { .. } => self.edit_list(first, rope, &old, new_len),
                Kind::Table { .. } => self.edit_table_row(first, rope, &old, new_len),
                Kind::Other => None,
            };
            if let Some(result) = result {
                for block in &mut self.blocks[first + 1..] {
                    block.shift(delta);
                }
                return result;
            }
        }
        let shift = |offset: usize| add(offset, delta);
        // The touched region, in new-text offsets, widened to whole lines.
        let lo_block = first.min(last);
        let region_start = {
            let s = if first < last {
                self.blocks[first].start.min(old.start)
            } else {
                old.start
            };
            rope.line_start_offset(rope.offset_to_point(s.min(rope.len())).row)
        };
        let region_end = if last > first {
            shift(self.blocks[last - 1].end.max(old.end))
        } else {
            shift(old.end)
        };
        let region_end = {
            let row = rope.offset_to_point(region_end.min(rope.len())).row;
            rope.line_end_offset(row).max(region_start)
        };

        // 4. Isolated: blank lines on both sides, nothing that can reach out.
        let prev_end = lo_block.checked_sub(1).map(|p| self.blocks[p].end);
        let next_start = (last < n).then(|| shift(self.blocks[last].start));
        let window_end = next_start.unwrap_or(rope.len()).max(region_start);
        let blank_row = |row: usize| rope.slice_line(row).chars().all(char::is_whitespace);
        let start_row = rope.offset_to_point(region_start).row;
        let end_row = rope.offset_to_point(region_end).row;
        let gap_before_blank = match prev_end {
            None => true,
            Some(p) => p <= region_start && start_row > 0 && blank_row(start_row - 1),
        };
        let gap_after_blank = match next_start {
            None => true,
            Some(s) => {
                region_end <= s
                    && end_row + 1 < rope.lines_len()
                    && end_row < rope.offset_to_point(s).row
                    && blank_row(end_row + 1)
            }
        };
        // Front matter opens at the document start and closes at the next
        // `---`/`+++` line, however far: no local window can validate it.
        let front_matter_risk = region_start == 0 && {
            let first = rope.slice_line(0).to_string();
            let first = first.trim_end();
            first == "---" || first == "+++"
        };
        if gap_before_blank
            && gap_after_blank
            && !front_matter_risk
            && window_end - region_start <= MAX_SYNC_BYTES
        {
            let window = rope.slice(region_start..window_end).to_string();
            let (parsed, contained) =
                self.parse_in_context(rope, region_start..window_end, self.keys_of(lo_block..last));
            // A list's span takes trailing blank lines depending on what
            // follows: an edit in the blank gap after a list reaches it.
            let after_list = lo_block > 0
                && matches!(self.blocks[lo_block - 1].node, BlockNode::List { .. })
                && rope
                    .slice(self.blocks[lo_block - 1].end.min(old.start)..old.start)
                    .chars()
                    .all(char::is_whitespace);
            let reaches = !contained
                || after_list
                || parsed
                    .iter()
                    .any(|b| reaches_out(b, &window[b.start - region_start..b.end - region_start]))
                || window.starts_with([' ', '\t'])
                || next_start.is_some_and(|s| matches!(rope.char_at(s), Some(' ' | '\t')));
            if !reaches {
                for block in &mut self.blocks[last..] {
                    block.shift(delta);
                }
                let splice = self.splice_blocks(lo_block..last, parsed);
                return EditResult {
                    splice,
                    path: Path::Isolated,
                    parsed_bytes: window.len(),
                    needs_full: false,
                };
            }
        }

        // 5. Neighbours.
        let (mut lo, mut hi) = (first.saturating_sub(1), (last + 1).min(n));
        let mut provisional: Option<(Vec<Block>, Range<usize>, usize)> = None;
        for _ in 0..3 {
            let start = if lo == 0 {
                0
            } else {
                let s = self.blocks[lo].start.min(region_start);
                rope.line_start_offset(rope.offset_to_point(s).row)
            };
            let end = if hi == n {
                rope.len()
            } else {
                shift(self.blocks[hi].start)
            };
            if end.saturating_sub(start) > MAX_SYNC_BYTES
                || end < start
                || (start == 0 && front_matter_risk)
            {
                break;
            }
            let window = rope.slice(start..end).to_string();
            let (parsed, contained) = self.parse_in_context(rope, start..end, self.keys_of(lo..hi));
            let left_ok = lo == first
                || parsed.first().is_some_and(|b| {
                    (b.start, b.end) == (self.blocks[lo].start, self.blocks[lo].end)
                });
            let right_ok = hi == last
                || parsed.last().is_some_and(|b| {
                    (b.start, b.end)
                        == (
                            shift(self.blocks[hi - 1].start),
                            shift(self.blocks[hi - 1].end),
                        )
                });
            let fences_ok = hi == n
                || !contained
                || parsed
                    .iter()
                    .all(|b| !unclosed_fence(&window[b.start - start..b.end - start]));
            let defs_ok = !parsed
                .iter()
                .any(|b| matches!(b.node, BlockNode::Definition { .. }));
            if contained && left_ok && right_ok && fences_ok && defs_ok {
                for block in &mut self.blocks[hi..] {
                    block.shift(delta);
                }
                let splice = self.splice_blocks(lo..hi, parsed);
                return EditResult {
                    splice,
                    path: Path::Neighbours,
                    parsed_bytes: window.len(),
                    needs_full: false,
                };
            }
            provisional = Some((parsed, lo..hi, window.len()));
            if lo == 0 && hi == n {
                break;
            }
            lo = lo.saturating_sub(1);
            hi = (hi + 1).min(n);
        }

        // Provisional: show the edited text now, reconcile in the background.
        let (parsed, range, bytes) = match provisional {
            Some(p) => p,
            None => {
                let start = region_start.min(rope.len());
                let end = window_end.max(start);
                if end - start > MAX_SYNC_BYTES {
                    // Too much to parse here (a large paste): show the text
                    // as plain paragraphs now, formatted by the background
                    // parse a moment later.
                    let end = region_end.max(start);
                    let window = rope.slice(start..end).to_string();
                    (plain_blocks(&window, start), lo_block..last, 0)
                } else {
                    let window = rope.slice(start..end).to_string();
                    let keys = self.keys_of(lo_block..last);
                    (
                        parse_blocks_reusing(&window, start, keys, &self.definitions),
                        lo_block..last,
                        window.len(),
                    )
                }
            }
        };
        for block in &mut self.blocks[range.end..] {
            block.shift(delta);
        }
        let splice = self.splice_blocks(range, parsed);
        EditResult {
            splice,
            path: Path::Provisional,
            parsed_bytes: bytes,
            needs_full: true,
        }
    }

    /// 1. Fence body: rebuild the code text without parsing.
    fn edit_fence(
        &mut self,
        bix: usize,
        rope: &Rope,
        old: &Range<usize>,
        new_len: usize,
        provisional: bool,
    ) -> Option<EditResult> {
        let block = &self.blocks[bix];
        let Kind::Fence {
            body,
            closed,
            ch,
            len,
            lang,
            meta,
        } = &block.kind
        else {
            return None;
        };
        let body_abs = block.start + body.start..block.start + body.end;
        let inside = old.start >= body_abs.start
            && if *closed {
                old.end < body_abs.end
            } else {
                old.end <= block.end
            };
        if !inside {
            return None;
        }
        let delta = new_len as isize - old.len() as isize;
        // The edited lines must not form a closing fence.
        let first_row = rope.offset_to_point(old.start).row;
        let last_row = rope
            .offset_to_point((old.start + new_len).min(rope.len()))
            .row;
        for row in first_row..=last_row {
            let line = rope.slice_line(row).to_string();
            if !provisional && is_closing_fence(&line, *ch, *len) {
                return None;
            }
        }
        let new_end = add(block.end, delta);
        let body_end = if *closed {
            add(body_abs.end, delta)
        } else {
            new_end.max(body_abs.start)
        };
        let mut value = rope.slice(body_abs.start..body_end).to_string();
        if value.ends_with('\n') {
            value.pop();
            if value.ends_with('\r') {
                value.pop();
            }
        }
        let key = block.leaves[0].key;
        let mut b = LeafBuilder::default();
        b.push_aligned(&value, &value, body.start);
        let leaf = LeafMap {
            key,
            text: b.text,
            segs: b.segs,
            empty_src: body.start,
        };
        let node = BlockNode::CodeBlock(
            CodeBlock::new(value.into(), lang.clone().map(Into::into), None::<Span>)
                .with_meta(meta.clone().map(Into::into))
                .with_leaf_key(key),
        );
        let new_body_end = if *closed {
            add(body.end, delta)
        } else {
            new_end - block.start
        };
        let block = &mut self.blocks[bix];
        if let Kind::Fence { body, .. } = &mut block.kind {
            body.end = new_body_end.max(body.start);
        }
        block.node = node;
        block.leaves = vec![leaf];
        block.end = new_end;
        Some(EditResult {
            splice: Splice {
                old: bix..bix + 1,
                new_len: 1,
            },
            path: if provisional {
                Path::Provisional
            } else {
                Path::FenceBody
            },
            parsed_bytes: 0,
            needs_full: provisional,
        })
    }

    /// 2. List item: reparse one item of a top-level list with its neighbours.
    fn edit_list(
        &mut self,
        bix: usize,
        rope: &Rope,
        old: &Range<usize>,
        new_len: usize,
    ) -> Option<EditResult> {
        let block = &self.blocks[bix];
        let Kind::List { items } = &block.kind else {
            return None;
        };
        let rel = old.start - block.start..old.end - block.start;
        let k = items
            .iter()
            .position(|i| i.span.start < rel.start && rel.end <= i.span.end)?;
        // Only interior items: an edit in the first or last item can change
        // what the list absorbs from the text around it (an emptied first
        // item cannot interrupt a paragraph; a last item can take lazy
        // continuation lines).
        if k == 0 || k + 1 >= items.len() {
            return None;
        }
        let (lo, hi) = (k - 1, k + 2);
        let delta = new_len as isize - old.len() as isize;
        let start = block.start + items[lo].span.start;
        let start = rope.line_start_offset(rope.offset_to_point(start).row);
        let end = if hi < items.len() {
            add(block.start + items[hi].span.start, delta)
        } else {
            add(block.end, delta)
        };
        if end <= start || end - start > MAX_SYNC_BYTES {
            return None;
        }
        let window = rope.slice(start..end).to_string();
        let (full, plen, children) = parse_with_prelude(&window, &self.definitions, start == 0)?;
        let [Node::List(list)] = children.as_slice() else {
            return None;
        };
        // Definitions resolve references anywhere: leave them to the paths
        // that track them.
        if window.contains("]:") || !block.defs.is_empty() {
            return None;
        }
        let BlockNode::List { ordered, .. } = &block.node else {
            return None;
        };
        if list.ordered != *ordered || list.children.len() != hi - lo {
            return None;
        }
        // The window starts at a line start that may precede the block start
        // (an indented first item); offsets stay relative to the block.
        let to_rel = start as isize - block.start as isize - plen as isize;
        let spans: Vec<Range<usize>> = list
            .children
            .iter()
            .map(|c| {
                let p = c.position().unwrap();
                add(p.start.offset, to_rel)..add(p.end.offset, to_rel)
            })
            .collect();
        // The neighbours must come back where they were.
        if lo < k && spans[0] != items[lo].span {
            return None;
        }
        if hi > k + 1 {
            let next = &items[k + 1].span;
            if spans[spans.len() - 1] != (add(next.start, delta)..add(next.end, delta)) {
                return None;
            }
        }
        // At the end of the list, the list must end with its last item.
        if hi == items.len() {
            let list_end = add(block.end, delta) - block.start;
            if spans[spans.len() - 1].end != list_end {
                return None;
            }
        }
        let mut reuse = block.leaves[items[lo].leaves.start..items[hi - 1].leaves.end]
            .iter()
            .map(|l| l.key)
            .collect::<Vec<_>>()
            .into_iter();
        let mut cx = BuildCx {
            source: &full,
            reuse: &mut reuse,
            block_start: 0,
            leaves: Vec::new(),
            depth: 1,
            kind: Kind::Other,
            defs: Vec::new(),
        };
        let mut new_items = Vec::new();
        let mut new_nodes = Vec::new();
        for child in &list.children {
            let before = cx.leaves.len();
            new_nodes.push(cx.block(child)?);
            new_items.push(before..cx.leaves.len());
        }
        let mut new_leaves = cx.leaves;
        for leaf in &mut new_leaves {
            shift_leaf(leaf, to_rel);
        }
        let parsed_bytes = window.len();
        let block_id = block.id;
        let block = &mut self.blocks[bix];
        let Kind::List { items } = &mut block.kind else {
            return None;
        };
        let old_leaves = items[lo].leaves.start..items[hi - 1].leaves.end;
        let leaf_delta = new_leaves.len() as isize - old_leaves.len() as isize;
        for leaf in &block.leaves[old_leaves.clone()] {
            self.leaf_block.remove(&leaf.key);
        }
        for leaf in &new_leaves {
            self.leaf_block.insert(leaf.key, block_id);
        }
        for leaf in &mut block.leaves[old_leaves.end..] {
            shift_leaf(leaf, delta);
        }
        let leaf_base = old_leaves.start;
        block.leaves.splice(old_leaves, new_leaves);
        let replaced: Vec<Item> = spans
            .into_iter()
            .zip(new_items)
            .map(|(span, leaves)| Item {
                span,
                leaves: leaf_base + leaves.start..leaf_base + leaves.end,
            })
            .collect();
        for item in &mut items[hi..] {
            item.span = add(item.span.start, delta)..add(item.span.end, delta);
            item.leaves = add(item.leaves.start, leaf_delta)..add(item.leaves.end, leaf_delta);
        }
        items.splice(lo..hi, replaced);
        if let BlockNode::List { children, .. } = &mut block.node {
            children.splice(lo..hi, new_nodes);
        }
        block.end = add(block.end, delta);
        Some(EditResult {
            splice: Splice {
                old: bix..bix + 1,
                new_len: 1,
            },
            path: Path::ListItem,
            parsed_bytes,
            needs_full: false,
        })
    }

    /// 3. Table row: an edit inside one body row, with no line break.
    fn edit_table_row(
        &mut self,
        bix: usize,
        rope: &Rope,
        old: &Range<usize>,
        new_len: usize,
    ) -> Option<EditResult> {
        let block = &self.blocks[bix];
        let Kind::Table { rows, aligns } = &block.kind else {
            return None;
        };
        let rel = old.start - block.start..old.end - block.start;
        let r = rows
            .iter()
            .position(|row| row.span.start < rel.start && rel.end <= row.span.end)?;
        if r == 0 {
            return None;
        }
        let delta = new_len as isize - old.len() as isize;
        let row_abs = block.start + rows[r].span.start..add(block.start + rows[r].span.end, delta);
        let row_text = rope.slice(row_abs.clone()).to_string();
        if row_text.contains(['\n', '\r']) {
            return None;
        }
        let header = rope
            .slice(block.start + rows[0].span.start..block.start + rows[0].span.end)
            .to_string();
        let header_row = rope.offset_to_point(block.start + rows[0].span.end).row;
        let delimiter = rope.slice_line(header_row + 1).to_string();
        let delimiter = delimiter.trim_end_matches(['\n', '\r']);
        let mini = format!("{header}\n{delimiter}\n{row_text}");
        let (mini, _, children) = parse_with_prelude(&mini, &self.definitions, false)?;
        let [Node::Table(table)] = children.as_slice() else {
            return None;
        };
        if table.children.len() != 2 || table.position.as_ref()?.end.offset != mini.len() {
            return None;
        }
        let Node::TableRow(row) = &table.children[1] else {
            return None;
        };
        let row_start_mini = mini.len() - row_text.len();
        let row_span_mini = row.position.as_ref()?;
        let mut reuse = block.leaves[rows[r].leaves.clone()]
            .iter()
            .map(|l| l.key)
            .collect::<Vec<_>>()
            .into_iter();
        let mut cx = BuildCx {
            source: &mini,
            reuse: &mut reuse,
            block_start: 0,
            leaves: Vec::new(),
            depth: 1,
            kind: Kind::Other,
            defs: Vec::new(),
        };
        let mut cells = Vec::new();
        for cell in &row.children {
            let Node::TableCell(c) = cell else { continue };
            let span = BuildCx::span_of(cell)?;
            cells.push(cx.paragraph(&c.children, span.end));
        }
        // Mini offsets → block-relative.
        let to_rel = (row_abs.start - block.start) as isize - row_start_mini as isize;
        let mut new_leaves = cx.leaves;
        for leaf in &mut new_leaves {
            shift_leaf(leaf, to_rel);
        }
        let new_span =
            add(row_span_mini.start.offset, to_rel)..add(row_span_mini.end.offset, to_rel);
        let block_id = block.id;
        let aligns = aligns.clone();
        let parsed_bytes = mini.len();
        let block = &mut self.blocks[bix];
        let Kind::Table { rows, .. } = &mut block.kind else {
            return None;
        };
        let old_leaves = rows[r].leaves.clone();
        let leaf_delta = new_leaves.len() as isize - old_leaves.len() as isize;
        for leaf in &block.leaves[old_leaves.clone()] {
            self.leaf_block.remove(&leaf.key);
        }
        for leaf in &new_leaves {
            self.leaf_block.insert(leaf.key, block_id);
        }
        for leaf in &mut block.leaves[old_leaves.end..] {
            shift_leaf(leaf, delta);
        }
        let new_leaf_range = old_leaves.start..add(old_leaves.end, leaf_delta);
        block.leaves.splice(old_leaves, new_leaves);
        rows[r] = Row {
            span: new_span,
            cells,
            leaves: new_leaf_range,
        };
        for row in &mut rows[r + 1..] {
            row.span = add(row.span.start, delta)..add(row.span.end, delta);
            row.leaves = add(row.leaves.start, leaf_delta)..add(row.leaves.end, leaf_delta);
        }
        let all: Vec<Vec<Paragraph>> = rows.iter().map(|row| row.cells.clone()).collect();
        block.node = BlockNode::Table(Table::new(all, aligns));
        block.end = add(block.end, delta);
        Some(EditResult {
            splice: Splice {
                old: bix..bix + 1,
                new_len: 1,
            },
            path: Path::TableRow,
            parsed_bytes,
            needs_full: false,
        })
    }

    /// Takes a whole-document parse made in the background for the current
    /// text. Blocks that come back unchanged keep their identity (no
    /// remeasure, no element-state churn); only the differing middle is
    /// spliced.
    pub fn reconcile(&mut self, mut fresh: Vec<Block>, text: &Rope) -> Splice {
        let n = self.blocks.len();
        let m = fresh.len();
        let mut prefix = 0;
        while prefix < n.min(m)
            && self.blocks[prefix].start == fresh[prefix].start
            && self.blocks[prefix].same_content(&fresh[prefix])
        {
            prefix += 1;
        }
        let mut suffix = 0;
        while suffix < (n - prefix).min(m - prefix)
            && self.blocks[n - 1 - suffix].start == fresh[m - 1 - suffix].start
            && self.blocks[n - 1 - suffix].same_content(&fresh[m - 1 - suffix])
        {
            suffix += 1;
        }
        let middle: Vec<Block> = fresh.drain(prefix..m - suffix).collect();
        let splice = self.splice_blocks(prefix..n - suffix, middle);
        self.refresh_definitions(text);
        splice
    }

    /// The block holding absolute offset `src`, else the nearest block with
    /// text.
    pub fn block_at(&self, src: usize) -> Option<usize> {
        let ix = self.blocks.partition_point(|b| b.end < src);
        if let Some(b) = self.blocks.get(ix)
            && b.start <= src
            && !b.leaves.is_empty()
        {
            return Some(ix);
        }
        let distance = |b: &Block| {
            if src < b.start {
                b.start - src
            } else {
                src.saturating_sub(b.end)
            }
        };
        // Search outward from `ix`: the nearest text block is close by.
        let below = (ix..self.blocks.len()).find(|&i| !self.blocks[i].leaves.is_empty());
        let above = (0..ix.min(self.blocks.len()))
            .rev()
            .find(|&i| !self.blocks[i].leaves.is_empty());
        match (above, below) {
            (Some(a), Some(b)) => Some(if distance(&self.blocks[a]) <= distance(&self.blocks[b]) {
                a
            } else {
                b
            }),
            (a, b) => a.or(b),
        }
    }

    /// The leaf (block index, leaf index) and leaf byte for absolute `src`.
    pub fn locate(&self, src: usize) -> Option<(usize, usize, usize)> {
        let bix = self.block_at(src)?;
        let block = &self.blocks[bix];
        let rel = src.saturating_sub(block.start);
        let lix = (0..block.leaves.len()).min_by_key(|&i| block.leaves[i].src_distance(rel))?;
        Some((bix, lix, block.leaves[lix].src_to_leaf(rel)))
    }

    /// Absolute source offset of leaf byte `index` in a block's leaf.
    pub fn src_of(&self, bix: usize, leaf_ix: usize, index: usize) -> usize {
        let block = &self.blocks[bix];
        block.start + block.leaves[leaf_ix].leaf_to_src(index)
    }

    /// Snaps a source offset to where a caret can rest (inside rendered
    /// text), so a caret stays valid after any edit.
    pub fn snap(&self, src: usize) -> usize {
        match self.locate(src) {
            Some((b, l, i)) => self.src_of(b, l, i),
            None => 0,
        }
    }

    /// The (block, leaf) after or before `(bix, leaf_ix)` in document order,
    /// skipping blocks without text leaves.
    pub fn step_leaf(&self, bix: usize, leaf_ix: usize, forward: bool) -> Option<(usize, usize)> {
        if forward {
            if leaf_ix + 1 < self.blocks[bix].leaves.len() {
                return Some((bix, leaf_ix + 1));
            }
            (bix + 1..self.blocks.len())
                .find(|&b| !self.blocks[b].leaves.is_empty())
                .map(|b| (b, 0))
        } else {
            if leaf_ix > 0 {
                return Some((bix, leaf_ix - 1));
            }
            (0..bix)
                .rev()
                .find(|&b| !self.blocks[b].leaves.is_empty())
                .map(|b| (b, self.blocks[b].leaves.len() - 1))
        }
    }
}

fn shift_leaf(leaf: &mut LeafMap, delta: isize) {
    for seg in &mut leaf.segs {
        seg.src = add(seg.src.start, delta)..add(seg.src.end, delta);
    }
    leaf.empty_src = add(leaf.empty_src, delta);
}

/// Whether a block can change how text after a blank line parses: an open
/// fence or HTML block, a list or footnote (indented continuation), a
/// definition (resolves links elsewhere).
fn reaches_out(block: &Block, text: &str) -> bool {
    reaches_out_block(block)
        || unclosed_fence(text)
        || text.trim_start().starts_with('<')
        || text.trim_start().starts_with('[')
}

fn reaches_out_block(block: &Block) -> bool {
    matches!(
        block.node,
        BlockNode::List { .. } | BlockNode::Definition { .. } | BlockNode::Blockquote { .. }
    ) || matches!(&block.node, BlockNode::CodeBlock(code) if code.lang().as_deref() == Some("html"))
}

/// Whether `block` opens a code fence it never closes.
fn unclosed_fence(block: &str) -> bool {
    let first = block.lines().next().unwrap_or("");
    let indent = first.bytes().take_while(|&b| b == b' ').count();
    let first = &first[indent..];
    let Some(ch) = first.bytes().next().filter(|c| matches!(c, b'`' | b'~')) else {
        return false;
    };
    let len = first.bytes().take_while(|&c| c == ch).count();
    if indent > 3 || len < 3 {
        return false;
    }
    !block
        .lines()
        .skip(1)
        .any(|line| is_closing_fence(line, ch, len))
}

/// Whether `line` closes a fence of `len` × `ch`: up to three spaces, at
/// least `len` fence characters, then only whitespace.
fn is_closing_fence(line: &str, ch: u8, len: usize) -> bool {
    let line = line.trim_end_matches(['\n', '\r']);
    let indent = line.bytes().take_while(|&b| b == b' ').count();
    if indent > 3 {
        return false;
    }
    let rest = &line[indent..];
    let run = rest.bytes().take_while(|&b| b == ch).count();
    run >= len && rest[run..].trim().is_empty()
}

struct BuildCx<'a> {
    /// The parsed window; mdast offsets index it.
    source: &'a str,
    /// Leaf keys of the leaves being replaced, handed on in order so that
    /// geometry from the last frame still addresses the edited text (the IME
    /// asks for bounds before the next frame).
    reuse: &'a mut std::vec::IntoIter<u64>,
    block_start: usize,
    leaves: Vec<LeafMap>,
    /// 0 for a top-level block.
    depth: usize,
    /// The fast-path description of the top-level block.
    kind: Kind,
    defs: Vec<Range<usize>>,
}

/// A paragraph under construction: inline runs and the leaf's mapping.
#[derive(Default)]
struct InlineBuilder {
    leaf: LeafBuilder,
    nodes: Vec<InlineNode>,
    run_start: usize,
    run_marks: Vec<(Range<usize>, TextMark)>,
}

impl InlineBuilder {
    fn flush(&mut self) {
        let end = self.leaf.text.len();
        if end > self.run_start {
            let text = self.leaf.text[self.run_start..end].to_string();
            self.nodes
                .push(InlineNode::new(text).marks(std::mem::take(&mut self.run_marks)));
        }
        self.run_start = end;
    }

    fn mark(&mut self, start: usize, mark: &TextMark) {
        let end = self.leaf.text.len();
        if end > start {
            self.run_marks
                .push((start - self.run_start..end - self.run_start, mark.clone()));
        }
    }
}

impl BuildCx<'_> {
    fn rel(&self, offset: usize) -> usize {
        offset - self.block_start
    }

    fn span_of(node: &Node) -> Option<Range<usize>> {
        node.position().map(|p| p.start.offset..p.end.offset)
    }

    fn key(&mut self) -> u64 {
        self.reuse.next().unwrap_or_else(fresh_id)
    }

    fn push_leaf(&mut self, b: InlineBuilder, empty_src: usize) -> Paragraph {
        let key = self.key();
        let mut paragraph = Paragraph::default();
        for node in b.nodes {
            paragraph.push(node);
        }
        self.leaves.push(LeafMap {
            key,
            text: b.leaf.text,
            segs: b.leaf.segs,
            empty_src,
        });
        paragraph.with_leaf_key(key)
    }

    fn paragraph(&mut self, children: &[Node], empty_src: usize) -> Paragraph {
        let mut b = InlineBuilder::default();
        for child in children {
            self.inline(child, &TextMark::default(), &mut b);
        }
        b.flush();
        self.push_leaf(b, self.rel(empty_src))
    }

    fn inline(&mut self, node: &Node, mark: &TextMark, b: &mut InlineBuilder) {
        let Some(span) = Self::span_of(node) else {
            return;
        };
        let base = self.rel(span.start);
        let start = b.leaf.text.len();
        match node {
            Node::Text(text) => {
                b.leaf
                    .push_prose(&text.value, &self.source[span.clone()], base);
                b.mark(start, mark);
            }
            Node::Strong(n) => self.children(&n.children, &mark.clone().bold(), b),
            Node::Emphasis(n) => self.children(&n.children, &mark.clone().italic(), b),
            Node::Delete(n) => self.children(&n.children, &mark.clone().strikethrough(), b),
            Node::Link(n) => {
                let link = LinkMark {
                    url: n.url.clone().into(),
                    title: n.title.clone().map(Into::into),
                    ..Default::default()
                };
                self.children(&n.children, &mark.clone().link(link), b)
            }
            Node::LinkReference(n) => self.children(&n.children, &mark.clone().underline(), b),
            Node::InlineCode(code) => {
                let src = &self.source[span.clone()];
                let ticks = src.bytes().take_while(|&c| c == b'`').count();
                b.leaf
                    .push_aligned(&code.value, &src[ticks..], base + ticks);
                b.mark(start, &mark.clone().code());
            }
            Node::InlineMath(math) => {
                b.leaf
                    .push_aligned(&math.value, &self.source[span.clone()], base);
                b.mark(start, &mark.clone().code());
            }
            Node::Html(html) => {
                b.leaf
                    .push_aligned(&html.value, &self.source[span.clone()], base);
                b.mark(start, &mark.clone().code());
            }
            Node::Break(_) => {
                b.leaf.push_atomic("\n", base..self.rel(span.end));
                b.mark(start, mark);
            }
            Node::FootnoteReference(_) => {
                b.leaf
                    .push_aligned(&self.source[span.clone()], &self.source[span.clone()], base);
                b.mark(start, &mark.clone().code());
            }
            Node::Image(image) => {
                b.flush();
                let mut node = ImageNode::default();
                node.url = image.url.clone().into();
                node.alt = Some(image.alt.clone().into());
                node.title = image.title.clone().map(Into::into);
                b.leaf.push_atomic("\u{FFFC}", base..self.rel(span.end));
                b.nodes.push(InlineNode::image(node));
                b.run_start = b.leaf.text.len();
            }
            other => {
                if let Some(children) = other.children() {
                    self.children(children, mark, b);
                }
            }
        }
    }

    fn children(&mut self, children: &[Node], mark: &TextMark, b: &mut InlineBuilder) {
        for child in children {
            self.inline(child, mark, b);
        }
    }

    /// A code-like leaf: `value` read from the node's source, skipping a
    /// fence line when there is one.
    fn code(
        &mut self,
        value: &str,
        lang: Option<String>,
        meta: Option<String>,
        span: Range<usize>,
    ) -> BlockNode {
        let src = &self.source[span.clone()];
        let first = src.trim_start();
        let fenced = first.starts_with("```")
            || first.starts_with("~~~")
            || first.starts_with("---")
            || first.starts_with("+++");
        let body = if fenced {
            src.find('\n').map_or(src.len(), |i| i + 1)
        } else {
            0
        };
        let mut b = LeafBuilder::default();
        b.push_aligned(value, &src[body..], self.rel(span.start) + body);
        let key = self.key();
        self.leaves.push(LeafMap {
            key,
            text: b.text,
            segs: b.segs,
            empty_src: self.rel(span.start) + body,
        });
        // The fence fast path needs a top-level, unindented ``` or ~~~ fence.
        let starts_line = self.source[..span.start]
            .chars()
            .next_back()
            .is_none_or(|c| c == '\n');
        let front_matter = matches!(lang.as_deref(), Some("yaml" | "toml"))
            && self.block_start == 0
            && span.start == 0;
        if self.depth == 0
            && starts_line
            && let Some(ch) = src
                .bytes()
                .next()
                .filter(|c| matches!(c, b'`' | b'~') || (front_matter && matches!(c, b'-' | b'+')))
        {
            let len = src.bytes().take_while(|&c| c == ch).count();
            if len >= 3 {
                let trimmed = src.trim_end_matches(['\n', '\r']);
                let last_line_start = trimmed.rfind('\n').map(|i| i + 1);
                let closed = last_line_start
                    .filter(|&s| s >= body)
                    .is_some_and(|s| is_closing_fence(&src[s..], ch, len));
                let body_end = if closed {
                    last_line_start.unwrap()
                } else {
                    src.len()
                };
                self.kind = Kind::Fence {
                    body: body..body_end.max(body),
                    closed,
                    ch,
                    len,
                    lang: lang.clone(),
                    meta: meta.clone(),
                };
            }
        }
        BlockNode::CodeBlock(
            CodeBlock::new(value.to_string().into(), lang.map(Into::into), None::<Span>)
                .with_meta(meta.map(Into::into))
                .with_leaf_key(key),
        )
    }

    fn nested(&mut self, node: &Node) -> Option<BlockNode> {
        self.depth += 1;
        let out = self.block(node);
        self.depth -= 1;
        out
    }

    fn block(&mut self, node: &Node) -> Option<BlockNode> {
        let span = Self::span_of(node)?;
        Some(match node {
            Node::Paragraph(p) => BlockNode::Paragraph(self.paragraph(&p.children, span.end)),
            Node::Heading(h) => BlockNode::Heading {
                level: h.depth,
                children: self.paragraph(&h.children, span.end),
                span: None,
            },
            Node::ThematicBreak(_) => BlockNode::HorizontalRule { span: None },
            Node::Blockquote(q) => BlockNode::Blockquote {
                children: q.children.iter().filter_map(|c| self.nested(c)).collect(),
                span: None,
            },
            Node::FootnoteDefinition(d) => BlockNode::Blockquote {
                children: d.children.iter().filter_map(|c| self.nested(c)).collect(),
                span: None,
            },
            Node::List(list) => {
                let top = self.depth == 0;
                let mut items = Vec::new();
                let mut children = Vec::new();
                for child in &list.children {
                    let before = self.leaves.len();
                    let Some(node) = self.nested(child) else {
                        continue;
                    };
                    if let Some(s) = Self::span_of(child) {
                        items.push(Item {
                            span: self.rel(s.start)..self.rel(s.end),
                            leaves: before..self.leaves.len(),
                        });
                    }
                    children.push(node);
                }
                if top {
                    self.kind = Kind::List { items };
                }
                BlockNode::List {
                    children,
                    ordered: list.ordered,
                    start: list.start,
                    span: None,
                }
            }
            Node::ListItem(item) => {
                let mut children: Vec<BlockNode> = item
                    .children
                    .iter()
                    .filter_map(|c| self.nested(c))
                    .collect();
                if children.is_empty() {
                    // An empty item still takes a caret.
                    let p = self.push_leaf(InlineBuilder::default(), self.rel(span.end));
                    children.push(BlockNode::Paragraph(p));
                }
                BlockNode::ListItem {
                    children,
                    spread: item.spread,
                    checked: item.checked,
                    span: None,
                }
            }
            Node::Code(code) => self.code(&code.value, code.lang.clone(), code.meta.clone(), span),
            Node::Math(math) => self.code(&math.value, Some("math".into()), None, span),
            Node::Html(html) => self.code(&html.value, Some("html".into()), None, span),
            Node::Yaml(y) => self.code(&y.value, Some("yaml".into()), None, span),
            Node::Toml(t) => self.code(&t.value, Some("toml".into()), None, span),
            Node::Table(table) => {
                let aligns: Vec<ColumnAlign> = table
                    .align
                    .iter()
                    .map(|a| match a {
                        AlignKind::Left => ColumnAlign::Left,
                        AlignKind::Right => ColumnAlign::Right,
                        AlignKind::Center => ColumnAlign::Center,
                        AlignKind::None => ColumnAlign::None,
                    })
                    .collect();
                let mut rows = Vec::new();
                for row in &table.children {
                    let Node::TableRow(row_node) = row else {
                        continue;
                    };
                    let before = self.leaves.len();
                    let mut cells = Vec::new();
                    for cell in &row_node.children {
                        let Node::TableCell(c) = cell else { continue };
                        let cell_span = Self::span_of(cell).unwrap_or(span.clone());
                        cells.push(self.paragraph(&c.children, cell_span.end));
                    }
                    let row_span = Self::span_of(row).unwrap_or(span.clone());
                    rows.push(Row {
                        span: self.rel(row_span.start)..self.rel(row_span.end),
                        cells,
                        leaves: before..self.leaves.len(),
                    });
                }
                let all = rows.iter().map(|r| r.cells.clone()).collect();
                if self.depth == 0 {
                    self.kind = Kind::Table {
                        rows,
                        aligns: aligns.clone(),
                    };
                }
                BlockNode::Table(Table::new(all, aligns))
            }
            Node::Definition(def) => {
                self.defs.push(self.rel(span.start)..self.rel(span.end));
                BlockNode::Definition {
                    identifier: def.identifier.clone().into(),
                    url: def.url.clone().into(),
                    title: def.title.clone().map(Into::into),
                    span: None,
                }
            }
            _ => return None,
        })
    }
}

/// The edited range: the `old` range of `before` and the length of its
/// replacement that turn `before` into `after`. Compares rope chunks from
/// both ends, so it costs the length of the unchanged text at memory-compare
/// speed, without allocating.
pub fn diff(before: &Rope, after: &Rope) -> Option<(Range<usize>, usize)> {
    let prefix = common_prefix(before, after);
    if prefix == before.len() && prefix == after.len() {
        return None;
    }
    let max_suffix = (before.len() - prefix).min(after.len() - prefix);
    let mut suffix = common_suffix(before, after).min(max_suffix);
    while suffix > 0
        && (!before.is_char_boundary(before.len() - suffix)
            || !after.is_char_boundary(after.len() - suffix))
    {
        suffix -= 1;
    }
    Some((prefix..before.len() - suffix, after.len() - suffix - prefix))
}

fn common_prefix(a: &Rope, b: &Rope) -> usize {
    let mut ca = a.chunks();
    let mut cb = b.chunks();
    let (mut sa, mut sb): (&[u8], &[u8]) = (&[], &[]);
    let mut total = 0;
    loop {
        if sa.is_empty() {
            match ca.next() {
                Some(c) => sa = c.as_bytes(),
                None => break,
            }
            continue;
        }
        if sb.is_empty() {
            match cb.next() {
                Some(c) => sb = c.as_bytes(),
                None => break,
            }
            continue;
        }
        let n = sa.len().min(sb.len());
        let same = if sa[..n] == sb[..n] {
            n
        } else {
            sa[..n]
                .iter()
                .zip(&sb[..n])
                .take_while(|(x, y)| x == y)
                .count()
        };
        total += same;
        if same < n {
            break;
        }
        sa = &sa[n..];
        sb = &sb[n..];
    }
    while total > 0 && !(a.is_char_boundary(total) && b.is_char_boundary(total)) {
        total -= 1;
    }
    total
}

fn common_suffix(a: &Rope, b: &Rope) -> usize {
    let mut ca = a.chunks_at(a.len()).0.reversed();
    let mut cb = b.chunks_at(b.len()).0.reversed();
    let (mut sa, mut sb): (&[u8], &[u8]) = (&[], &[]);
    let mut total = 0;
    loop {
        if sa.is_empty() {
            match ca.next() {
                Some(c) => sa = c.as_bytes(),
                None => break,
            }
            continue;
        }
        if sb.is_empty() {
            match cb.next() {
                Some(c) => sb = c.as_bytes(),
                None => break,
            }
            continue;
        }
        let n = sa.len().min(sb.len());
        let (ta, tb) = (&sa[sa.len() - n..], &sb[sb.len() - n..]);
        let same = if ta == tb {
            n
        } else {
            ta.iter()
                .rev()
                .zip(tb.iter().rev())
                .take_while(|(x, y)| x == y)
                .count()
        };
        total += same;
        if same < n {
            break;
        }
        sa = &sa[..sa.len() - n];
        sb = &sb[..sb.len() - n];
    }
    total
}

#[cfg(test)]
pub(crate) mod tests {
    use super::super::mapping::Seg;
    use super::*;
    use unicode_segmentation::UnicodeSegmentation as _;

    pub(crate) const SAMPLE: &str = "---\ntitle: x\n---\n\n# Heading **one**\n\nSome *emphasis*, `code`, a [link](https://e.com) and \\* escape &amp; entity.\nSecond line.\n\n- item one\n- [ ] task two\n  - nested\n\n> quoted\n> text\n\n```rust\nfn main() {}\n```\n\n| a | b |\n|---|--:|\n| 中文 | 👍🏽 |\n\nनमस्ते दुनिया। ![img](x.png) end\n";

    /// Every leaf byte at a grapheme boundary maps to source and back.
    #[test]
    fn leaf_to_source_round_trips_for_every_leaf() {
        let doc = VisualDocument::parse(SAMPLE);
        let mut leaves = 0;
        for block in &doc.blocks {
            for leaf in &block.leaves {
                leaves += 1;
                let boundaries = leaf
                    .text
                    .grapheme_indices(true)
                    .map(|(i, _)| i)
                    .chain([leaf.text.len()]);
                for i in boundaries {
                    let src = leaf.leaf_to_src(i);
                    assert_eq!(leaf.src_to_leaf(src), i, "{:?} at {i}", leaf.text);
                }
            }
        }
        assert!(leaves >= 12, "{leaves}");
    }

    /// The source byte a leaf character maps to holds that character.
    #[test]
    fn linear_runs_point_at_their_own_characters() {
        let doc = VisualDocument::parse(SAMPLE);
        for block in &doc.blocks {
            for leaf in &block.leaves {
                for seg in leaf.segs.iter().filter(|s| s.linear) {
                    let src = &SAMPLE[block.start + seg.src.start..block.start + seg.src.end];
                    assert_eq!(&leaf.text[seg.leaf.clone()], src);
                }
            }
        }
    }

    #[test]
    fn rendered_text_hides_markers() {
        let doc = VisualDocument::parse(SAMPLE);
        let texts: Vec<&str> = doc
            .blocks
            .iter()
            .flat_map(|b| b.leaves.iter().map(|l| l.text.as_str()))
            .collect();
        assert!(texts.contains(&"Heading one"), "{texts:?}");
        assert!(texts.contains(&"task two"), "{texts:?}");
        assert!(texts.contains(&"fn main() {}"), "{texts:?}");
        assert!(
            texts
                .iter()
                .any(|t| t.starts_with("Some emphasis, code, a link and * escape & entity."))
        );
    }

    type Shape = Vec<(usize, usize, Vec<(String, Vec<Seg>)>)>;

    /// Spans, leaf texts and leaf mappings: everything an edit must agree
    /// with a whole parse on (ids and keys aside).
    pub(crate) fn shape(doc: &VisualDocument) -> Shape {
        doc.blocks
            .iter()
            .map(|b| {
                (
                    b.start,
                    b.end,
                    b.leaves
                        .iter()
                        .map(|l| (l.text.clone(), l.segs.clone()))
                        .collect(),
                )
            })
            .collect()
    }

    fn same_as_whole(doc: &VisualDocument, whole: &VisualDocument) -> bool {
        shape(doc) == shape(whole)
            && doc
                .blocks
                .iter()
                .zip(&whole.blocks)
                .all(|(x, y)| x.node == y.node && x.kind == y.kind)
    }

    pub(crate) fn assert_matches_full_parse(doc: &VisualDocument, text: &str, what: &str) {
        let full = VisualDocument::parse(text);
        let (a, b) = (shape(doc), shape(&full));
        if a != b {
            let first = a
                .iter()
                .zip(&b)
                .position(|(x, y)| x != y)
                .unwrap_or(a.len().min(b.len()));
            panic!(
                "{what}: differs from a whole parse at block {first} ({} vs {} blocks):\n  incremental {:?}\n  whole       {:?}",
                a.len(),
                b.len(),
                a.get(first),
                b.get(first)
            );
        }
        for (x, y) in doc.blocks.iter().zip(&full.blocks) {
            assert!(x.node == y.node, "{what}: node differs at {}", x.start);
            assert_eq!(x.kind, y.kind, "{what}: kind differs at {}", x.start);
        }
        for (bix, block) in doc.blocks.iter().enumerate() {
            for (lix, leaf) in block.leaves.iter().enumerate() {
                assert_eq!(doc.leaf_by_key(leaf.key), Some((bix, lix)), "{what}");
            }
        }
    }

    /// Applies an edit, finishing provisional results the way the view does
    /// in the background.
    pub(crate) fn apply(
        doc: &mut VisualDocument,
        text: &mut String,
        range: Range<usize>,
        insert: &str,
    ) -> EditResult {
        text.replace_range(range.clone(), insert);
        let rope = Rope::from(text.as_str());
        let result = doc.edit(&rope, range, insert.len());
        if result.needs_full {
            doc.reconcile(parse_blocks(text, 0), &rope);
        }
        result
    }

    #[test]
    fn typing_reparses_only_the_block() {
        let mut text = SAMPLE.repeat(20);
        let mut doc = VisualDocument::parse(&text);
        let total = doc.blocks.len();
        let at = text.find("Second line").unwrap() + 3;
        let result = apply(&mut doc, &mut text, at..at, "x");
        assert_eq!(result.path, Path::Isolated);
        assert!(result.parsed_bytes < 120, "{result:?}");
        assert_eq!(doc.blocks.len(), total);
        assert_matches_full_parse(&doc, &text, "typing");
    }

    #[test]
    fn fast_paths_are_taken_and_match() {
        let big_list: String = (0..400)
            .map(|i| format!("- item {i} with **text**\n"))
            .collect();
        let big_table: String = "| a | b |\n|---|---|\n".to_string()
            + &(0..400)
                .map(|i| format!("| r{i} | `c` |\n"))
                .collect::<String>();
        let fence = "```rust\nlet a = 1;\nlet b = 2;\n```\n";
        let base = format!("# T\n\n{big_list}\npara\n\n{big_table}\n{fence}\nend\n");
        let cases: Vec<(&str, usize, usize, &str, Path)> = vec![
            (
                "list item",
                base.find("item 200").unwrap() + 5,
                0,
                "x",
                Path::ListItem,
            ),
            (
                "list item delete",
                base.find("item 201").unwrap() + 5,
                3,
                "",
                Path::ListItem,
            ),
            (
                "table row",
                base.find("r300").unwrap() + 1,
                0,
                "z",
                Path::TableRow,
            ),
            (
                "fence body",
                base.find("let b").unwrap(),
                0,
                "// hi\n",
                Path::FenceBody,
            ),
            (
                "paragraph",
                base.find("para").unwrap() + 2,
                0,
                "q",
                Path::Isolated,
            ),
        ];
        for (what, at, len, insert, path) in cases {
            let mut text = base.clone();
            let mut doc = VisualDocument::parse(&text);
            let result = apply(&mut doc, &mut text, at..at + len, insert);
            assert_eq!(result.path, path, "{what}");
            assert!(result.parsed_bytes <= 200, "{what}: {result:?}");
            assert_matches_full_parse(&doc, &text, what);
        }
    }

    #[test]
    fn unclosed_fence_typing_is_a_fast_path() {
        let rest = "para\n\n".repeat(2000);
        let mut text = format!("# T\n\n```\nlet x = 1;\n{rest}");
        let mut doc = VisualDocument::parse(&text);
        assert!(matches!(
            doc.blocks[1].kind,
            Kind::Fence { closed: false, .. }
        ));
        let start = text.find("let x").unwrap() + 4;
        for i in 0..20 {
            let at = start + i;
            let r = apply(&mut doc, &mut text, at..at, "y");
            assert_eq!(r.path, Path::FenceBody);
            assert_eq!(r.parsed_bytes, 0);
        }
        assert_matches_full_parse(&doc, &text, "unclosed fence");
        // Typing the closing fence leaves the fast path and asks for a full
        // parse instead of doing one on this thread.
        let at = text.find("para").unwrap();
        let r = apply(&mut doc, &mut text, at..at, "```\n");
        assert_ne!(r.path, Path::FenceBody);
        assert!(r.needs_full);
        assert!(r.parsed_bytes <= MAX_SYNC_BYTES, "{r:?}");
        assert_matches_full_parse(&doc, &text, "closing");
    }

    #[test]
    fn structural_edits_match_a_full_parse() {
        let base = SAMPLE.repeat(3);
        let edits: Vec<(&str, usize, usize, &str)> = vec![
            ("merge heading", base.find("# Heading").unwrap() - 1, 1, ""),
            ("to heading", base.find("Some *emph").unwrap(), 0, "## "),
            ("open fence", base.find("> quoted").unwrap(), 0, "```\n"),
            ("split paragraph", base.find("Second").unwrap(), 0, "\n"),
            ("delete across", base.find("- item").unwrap(), 30, ""),
            ("lazy quote", base.find("> text").unwrap() + 6, 1, ""),
            (
                "setext",
                base.find("Second line.").unwrap() + 12,
                0,
                "\n---",
            ),
            (
                "html comment",
                base.find("Some *emph").unwrap(),
                0,
                "<!--\n",
            ),
            (
                "definition",
                base.find("Some *emph").unwrap(),
                0,
                "[link]: http://x\n\n",
            ),
        ];
        for (what, at, len, insert) in edits {
            let mut text = base.clone();
            let mut doc = VisualDocument::parse(&text);
            apply(&mut doc, &mut text, at..at + len, insert);
            assert_matches_full_parse(&doc, &text, what);
        }
    }

    #[test]
    fn unclosed_fence_detection() {
        assert!(unclosed_fence("```rust\nfn x"));
        assert!(!unclosed_fence("```rust\nfn x\n```"));
        assert!(!unclosed_fence("plain"));
        assert!(unclosed_fence("~~~~\na\n~~~"));
    }

    #[test]
    fn diff_recovers_the_edited_range() {
        let cases = [
            ("hello world", "hello brave world"),
            ("aaa", "aaaa"),
            ("中文 👍🏽", "中 👍🏽"),
            ("same", "same"),
            ("", "x"),
            ("abc", ""),
        ];
        for (a, b) in cases {
            let (ra, rb) = (Rope::from(a), Rope::from(b));
            match diff(&ra, &rb) {
                None => assert_eq!(a, b),
                Some((old, new_len)) => {
                    let mut s = a.to_string();
                    s.replace_range(old.clone(), &b[old.start..old.start + new_len]);
                    assert_eq!(s, b, "{a:?} → {b:?}");
                }
            }
        }
        let big = "x".repeat(300_000);
        let mut edited = big.clone();
        edited.insert(150_000, 'y');
        let (old, len) = diff(&Rope::from(big.as_str()), &Rope::from(edited.as_str())).unwrap();
        let mut s = big.clone();
        s.replace_range(old.clone(), &edited[old.start..old.start + len]);
        assert_eq!(s, edited);
        assert!(old.len() <= 1 && len <= 2, "{old:?} {len}");
    }

    #[test]
    fn prefix_cut_lands_after_a_blank_line_outside_fences() {
        let text = format!(
            "{}```\n\n\ncode\n```\n\nafter\n\nmore\n",
            "para\n\n".repeat(10)
        );
        let cut = prefix_cut(&text, 62);
        assert!(text[..cut].ends_with("\n\n"), "{:?}", &text[..cut]);
        // The cut is not inside the fence.
        assert!(
            text[..cut].matches("```").count() % 2 == 0,
            "{:?}",
            &text[..cut]
        );
    }

    /// Random edits with Markdown-significant text, each checked against a
    /// whole-document parse.
    #[test]
    fn random_edits_match_a_full_parse() {
        let pieces = [
            "*",
            "**",
            "`",
            "```",
            "\n",
            "\n\n",
            "# ",
            "- ",
            "> ",
            "|",
            "x",
            "中",
            "।",
            " ",
            "~~~\n",
            "1. ",
            "---\n",
            "[",
            "](u)",
            "<!--",
            "-->",
            "[x]: /u\n",
            "===",
            "  ",
        ];
        // ENSO_FUZZ=<runs> runs more seeds (each 1500 steps) for a deeper check.
        let runs: u64 = std::env::var("ENSO_FUZZ")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);
        let mut differed = 0;
        for run in 0..runs {
            differed += random_edit_run(
                0x2545F4914F6CDD1D ^ run.wrapping_mul(0x9E3779B97F4A7C15),
                &pieces,
            );
        }
        let steps = runs as usize * 1500;
        eprintln!(
            "random edits: {differed} of {steps} immediate results differed from a whole parse ({:.3}%)",
            100. * differed as f64 / steps as f64
        );
        // Adversarial edits (random Markdown punctuation anywhere) hit
        // markdown 1.0.0's non-local behaviour now and then; ordinary typing
        // does not (see `typing_paths_match_without_settling`). Keep it rare.
        assert!(differed * 100 <= steps, "{differed} of {steps}");
    }

    /// Runs 1500 random edits. Returns how many immediate results differed
    /// from a whole parse (each is then settled the way the view's idle pass
    /// settles it, and must match exactly).
    fn random_edit_run(mut seed: u64, pieces: &[&str]) -> usize {
        let mut next = |n: usize| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed % n as u64) as usize
        };
        let big_list: String = (0..30).map(|i| format!("- item {i}\n")).collect();
        let mut text = SAMPLE.repeat(2)
            + &big_list
            + "\n| h | i |\n|---|---|\n| 1 | 2 |\n| 3 | 4 |\n\n```\ncode\n```\n";
        let mut doc = VisualDocument::parse(&text);
        let mut differed = 0;
        for step in 0..1500 {
            let mut at = next(text.len() + 1);
            while !text.is_char_boundary(at) {
                at -= 1;
            }
            let mut end = (at + next(4)).min(text.len());
            while !text.is_char_boundary(end) {
                end += 1;
            }
            let insert = if next(3) == 0 {
                ""
            } else {
                pieces[next(pieces.len())]
            };
            let before = text.clone();
            let result = apply(&mut doc, &mut text, at..end, insert);
            let whole = VisualDocument::parse(&text);
            if !same_as_whole(&doc, &whole) {
                differed += 1;
                if let Ok(dir) = std::env::var("ENSO_FUZZ_DUMP") {
                    let _ = std::fs::write(format!("{dir}/before.md"), &before);
                    let _ = std::fs::write(
                        format!("{dir}/edit.txt"),
                        format!("{at}..{end} {insert:?} {result:?}"),
                    );
                }
                // The idle pass: a whole parse reconciled in.
                doc.reconcile(parse_blocks(&text, 0), &Rope::from(text.as_str()));
            }
            assert_matches_full_parse(&doc, &text, &format!("step {step}: {at}..{end} {insert:?}"));
        }
        differed
    }

    /// A mixed document like the benchmark corpus.
    pub(crate) fn corpus(sections: usize) -> String {
        let mut out = String::from("---\ntitle: Corpus\n---\n\n");
        for k in 0..sections {
            out += &format!(
                "## Section {k}\n\nA *quiet* page with **bold** text, `code` and a [link](https://e.com).\nA second line.\n\nनमस्ते दुनिया। यह एक वाक्य है। 中文的句子。 Emoji 👍🏽.\n\n- first\n- [ ] second **task**\n  - nested\n- third\n\n> quoted\n> text\n\n"
            );
            if k % 3 == 0 {
                out += "```rust\nfn main() {}\n```\n\n";
            }
            if k % 4 == 0 {
                out += "| a | b |\n|---|--:|\n| 中 | `c` |\n| x | y |\n\n";
            }
        }
        out
    }

    /// Ordinary typing (letters, spaces, punctuation, paragraph breaks and
    /// backspaces at caret positions) matches a whole parse after every
    /// keystroke, without any settling.
    #[test]
    fn typing_paths_match_without_settling() {
        let mut text = corpus(12);
        let mut doc = VisualDocument::parse(&text);
        let keys = ["a", "e", " ", "t", ".", ",", "\n\n", "s", "中", "।", "BACK"];
        let mut seed: u64 = 0xD1B54A32D192ED03;
        let mut next = |n: usize| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed % n as u64) as usize
        };
        let mut paths = std::collections::HashMap::new();
        for step in 0..3000 {
            // A caret position: a grapheme boundary of some leaf.
            let with_text: Vec<usize> = (0..doc.blocks.len())
                .filter(|&b| !doc.blocks[b].leaves.is_empty())
                .collect();
            let bix = with_text[next(with_text.len())];
            let lix = next(doc.blocks[bix].leaves.len());
            let leaf = &doc.blocks[bix].leaves[lix];
            let bounds: Vec<usize> = leaf
                .text
                .grapheme_indices(true)
                .map(|(i, _)| i)
                .chain([leaf.text.len()])
                .collect();
            let index = bounds[next(bounds.len())];
            let caret = doc.src_of(bix, lix, index);
            let key = keys[next(keys.len())];
            let result = if key == "BACK" {
                if index == 0 {
                    continue;
                }
                let prev = bounds[bounds.iter().position(|&b| b == index).unwrap() - 1];
                let start = doc.src_of(bix, lix, prev);
                if start >= caret {
                    continue;
                }
                apply(&mut doc, &mut text, start..caret, "")
            } else {
                apply(&mut doc, &mut text, caret..caret, key)
            };
            *paths.entry(format!("{:?}", result.path)).or_insert(0) += 1;
            let whole = VisualDocument::parse(&text);
            assert!(
                same_as_whole(&doc, &whole),
                "step {step}: {key:?} at {caret} ({result:?})\n{:?}\nincremental {:?}\nwhole {:?}",
                &text[text.floor_char_boundary(caret.saturating_sub(60))
                    ..text.ceil_char_boundary(caret + 60)],
                doc.blocks
                    .iter()
                    .map(|b| (b.start, b.end))
                    .filter(|b| b.1 + 80 > caret && b.0 < caret + 80)
                    .collect::<Vec<_>>(),
                whole
                    .blocks
                    .iter()
                    .map(|b| (b.start, b.end))
                    .filter(|b| b.1 + 80 > caret && b.0 < caret + 80)
                    .collect::<Vec<_>>(),
            );
        }
        eprintln!("typing paths: {paths:?}");
    }

    /// Replays a case the fuzzer dumped (`ENSO_FUZZ_DUMP`): set `ENSO_CASE`
    /// to that directory. Does nothing otherwise.
    #[test]
    fn replay_dumped_case() {
        let Ok(dir) = std::env::var("ENSO_CASE") else {
            return;
        };
        let mut text = std::fs::read_to_string(format!("{dir}/before.md")).unwrap();
        let edit = std::fs::read_to_string(format!("{dir}/edit.txt")).unwrap();
        let (range, rest) = edit.split_once(' ').unwrap();
        let (a, b) = range.split_once("..").unwrap();
        let (a, b): (usize, usize) = (a.parse().unwrap(), b.parse().unwrap());
        let insert = rest
            .split_once(" EditResult")
            .unwrap()
            .0
            .trim_matches('"')
            .replace("\\n", "\n");
        let mut doc = VisualDocument::parse(&text);
        let result = apply(&mut doc, &mut text, a..b, &insert);
        eprintln!("{result:?}");
        let full = VisualDocument::parse(&text);
        let show = |d: &VisualDocument| {
            d.blocks
                .iter()
                .map(|b| {
                    format!(
                        "{}..{} {:?}",
                        b.start,
                        b.end,
                        text.get(b.start..b.end.min(b.start + 30)).unwrap_or("?")
                    )
                })
                .collect::<Vec<_>>()
        };
        for (x, y) in doc.blocks.iter().zip(&full.blocks) {
            if x.node != y.node {
                eprintln!("NODE inc   {:?}\nNODE whole {:?}", x.node, y.node);
            }
        }
        for (x, y) in shape(&doc).iter().zip(shape(&full)) {
            if *x != y {
                eprintln!("LEAF inc   {x:?}\nLEAF whole {y:?}");
            }
        }
        for (x, y) in show(&doc).iter().zip(show(&full)) {
            eprintln!("{} {x}\n   whole {y}", if *x == y { " " } else { "!" });
        }
    }
}
