//! An incremental index of the document's top-level blocks.
//!
//! The Markdown text (the editor's rope) stays the only source of truth.
//! Each top-level block keeps the [`BlockNode`] GPUI Kit renders and the
//! source mapping of every text leaf in it. An edit reparses only the blocks
//! it touches plus one neighbour on each side, and falls back to a full parse
//! when the neighbours do not come back unchanged (an opened code fence that
//! now swallows the rest of the document, for example).

use std::ops::Range;

use gpui_kit::base::input::Rope;
use gpui_kit::base::text::editor_host::{
    BlockNode, CodeBlock, ColumnAlign, ImageNode, InlineNode, LinkMark, Paragraph, Span, Table,
    TextMark,
};
use markdown::mdast::{AlignKind, Node};

use super::mapping::{LeafBuilder, LeafMap};

/// One top-level block.
pub struct Block {
    /// Stable for the life of the block; new blocks from a reparse get new ids.
    pub id: u64,
    /// Absolute source range.
    pub start: usize,
    pub end: usize,
    pub node: BlockNode,
    /// The block's text leaves in document order, offsets relative to `start`.
    pub leaves: Vec<LeafMap>,
}

/// Bits of a leaf key that number leaves within one block.
const LEAF_BITS: u32 = 16;

pub fn leaf_key(block_id: u64, leaf_ix: usize) -> u64 {
    (block_id << LEAF_BITS) | leaf_ix as u64
}

pub fn key_block_id(key: u64) -> u64 {
    key >> LEAF_BITS
}

pub fn key_leaf_ix(key: u64) -> usize {
    (key & ((1 << LEAF_BITS) - 1)) as usize
}

/// What an edit did to the block list: `old` blocks were replaced by `new_len`
/// blocks starting at `old.start`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Splice {
    pub old: Range<usize>,
    pub new_len: usize,
    pub full: bool,
    /// Bytes handed to the parser.
    pub parsed_bytes: usize,
}

#[derive(Default)]
pub struct VisualDocument {
    pub blocks: Vec<Block>,
    next_id: u64,
}

fn parse_options() -> markdown::ParseOptions {
    let mut options = markdown::ParseOptions::gfm();
    options.constructs.frontmatter = true;
    options
}

impl VisualDocument {
    pub fn parse(text: &str) -> Self {
        let mut doc = Self {
            blocks: Vec::new(),
            next_id: 1,
        };
        doc.blocks = doc.parse_window(text, 0, &[]);
        doc
    }

    /// Parses `text`, which starts at absolute offset `base`, into blocks.
    /// The blocks take the ids in `reuse` first, in order: a block reparsed in
    /// place keeps its identity, its leaf keys and its element state.
    fn parse_window(&mut self, text: &str, base: usize, reuse: &[u64]) -> Vec<Block> {
        let Ok(Node::Root(root)) = markdown::to_mdast(text, &parse_options()) else {
            return Vec::new();
        };
        let mut blocks = Vec::with_capacity(root.children.len());
        for child in &root.children {
            let Some(position) = child.position() else {
                continue;
            };
            let (start, end) = (position.start.offset, position.end.offset);
            let id = match reuse.get(blocks.len()) {
                Some(&id) => id,
                None => {
                    self.next_id += 1;
                    self.next_id - 1
                }
            };
            let mut cx = BuildCx {
                source: text,
                block_start: start,
                block_id: id,
                leaves: Vec::new(),
            };
            let Some(node) = cx.block(child) else {
                continue;
            };
            blocks.push(Block {
                id,
                start: base + start,
                end: base + end,
                node,
                leaves: cx.leaves,
            });
        }
        blocks
    }

    /// Brings the index up to date with `rope`, where `old` (a range of the
    /// previous text) was replaced by `new_len` bytes.
    pub fn edit(&mut self, rope: &Rope, old: Range<usize>, new_len: usize) -> Splice {
        use gpui_kit::base::input::RopeExt as _;
        let delta = new_len as isize - old.len() as isize;
        let shift = |offset: usize| (offset as isize + delta) as usize;
        let n = self.blocks.len();
        // Blocks the edit touches (an edit at a block's edge touches it).
        let first = self.blocks.partition_point(|b| b.end < old.start);
        let last = self.blocks.partition_point(|b| b.start <= old.end);
        let (mut lo, mut hi) = (first.saturating_sub(1), (last + 1).min(n));
        for _ in 0..3 {
            if lo == 0 && hi == n {
                break;
            }
            let start = if lo == 0 {
                0
            } else {
                let row = rope.offset_to_point(self.blocks[lo].start).row;
                rope.line_start_offset(row)
            };
            // Through the gap to the first untouched block, so the last
            // block parses with what follows it as in a whole-document parse.
            let end = if hi == n {
                rope.len()
            } else {
                shift(self.blocks[hi].start)
            };
            let window = rope.slice(start..end).to_string();
            let reuse: Vec<u64> = self.blocks[lo..hi].iter().map(|b| b.id).collect();
            let parsed = self.parse_window(&window, start, &reuse);
            // The neighbours, whose text the edit did not touch, must come
            // back as the same blocks: then nothing beyond them changed.
            let left_ok = lo == first
                || parsed.first().is_some_and(|b| {
                    (b.start, b.end) == (self.blocks[lo].start, self.blocks[lo].end)
                });
            let right_ok = hi == last
                || hi == n && last == n
                || parsed.last().is_some_and(|b| {
                    (b.start, b.end)
                        == (
                            shift(self.blocks[hi - 1].start),
                            shift(self.blocks[hi - 1].end),
                        )
                });
            let fences_ok = parsed
                .iter()
                .all(|b| !unclosed_fence(&window[b.start - start..b.end - start]));
            if left_ok && right_ok && fences_ok {
                for block in &mut self.blocks[hi..] {
                    block.start = shift(block.start);
                    block.end = shift(block.end);
                }
                let new_len = parsed.len();
                self.blocks.splice(lo..hi, parsed);
                return Splice {
                    old: lo..hi,
                    new_len,
                    full: false,
                    parsed_bytes: window.len(),
                };
            }
            lo = lo.saturating_sub(1);
            hi = (hi + 1).min(n);
        }
        let text = rope.to_string();
        let reuse: Vec<u64> = self.blocks.iter().map(|b| b.id).collect();
        let blocks = self.parse_window(&text, 0, &reuse);
        let new_len = blocks.len();
        self.blocks = blocks;
        Splice {
            old: 0..n,
            new_len,
            full: true,
            parsed_bytes: text.len(),
        }
    }

    /// The block holding absolute offset `src`, else the nearest one.
    pub fn block_at(&self, src: usize) -> Option<usize> {
        if self.blocks.is_empty() {
            return None;
        }
        let ix = self.blocks.partition_point(|b| b.end < src);
        if ix >= self.blocks.len() {
            return Some(self.blocks.len() - 1);
        }
        if ix > 0 && self.blocks[ix].start > src {
            // In the gap between two blocks: take the nearer.
            let before = src - self.blocks[ix - 1].end;
            let after = self.blocks[ix].start - src;
            return Some(if before <= after { ix - 1 } else { ix });
        }
        Some(ix)
    }

    /// The leaf (block index, leaf index) and leaf byte for absolute `src`.
    pub fn locate(&self, src: usize) -> Option<(usize, usize, usize)> {
        let bix = self.block_at(src)?;
        let leaf_ix = self.nearest_leaf(bix, src)?;
        let block = &self.blocks[bix];
        let rel = src.saturating_sub(block.start);
        Some((bix, leaf_ix, block.leaves[leaf_ix].src_to_leaf(rel)))
    }

    fn nearest_leaf(&self, bix: usize, src: usize) -> Option<usize> {
        let block = &self.blocks[bix];
        let rel = src.saturating_sub(block.start);
        (0..block.leaves.len()).min_by_key(|&i| block.leaves[i].src_distance(rel))
    }

    /// Absolute source offset of leaf byte `index` in a block's leaf.
    pub fn src_of(&self, bix: usize, leaf_ix: usize, index: usize) -> usize {
        let block = &self.blocks[bix];
        block.start + block.leaves[leaf_ix].leaf_to_src(index)
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

/// Whether `block` opens a code fence it never closes.
fn unclosed_fence(block: &str) -> bool {
    let first = block.lines().next().unwrap_or("").trim_start();
    let fence: String = first
        .chars()
        .take_while(|&c| c == '`' || c == '~')
        .collect();
    if fence.len() < 3 || !fence.chars().all(|c| c == fence.chars().next().unwrap()) {
        return false;
    }
    let mut lines = block.lines().skip(1);
    let ch = fence.chars().next().unwrap();
    !lines.any(|line| {
        let t = line.trim();
        t.len() >= fence.len() && t.chars().all(|c| c == ch)
    })
}

struct BuildCx<'a> {
    /// The parsed window; mdast offsets index it.
    source: &'a str,
    block_start: usize,
    block_id: u64,
    leaves: Vec<LeafMap>,
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

    fn push_leaf(&mut self, b: InlineBuilder, empty_src: usize) -> (Paragraph, u64) {
        let key = leaf_key(self.block_id, self.leaves.len());
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
        (paragraph.with_leaf_key(key), key)
    }

    fn paragraph(&mut self, children: &[Node], span: Range<usize>, empty_src: usize) -> Paragraph {
        let mut b = InlineBuilder::default();
        for child in children {
            self.inline(child, &TextMark::default(), &mut b);
        }
        b.flush();
        let _ = span;
        self.push_leaf(b, self.rel(empty_src)).0
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
    fn code(&mut self, value: &str, lang: Option<String>, span: Range<usize>) -> BlockNode {
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
        let key = leaf_key(self.block_id, self.leaves.len());
        self.leaves.push(LeafMap {
            key,
            text: b.text,
            segs: b.segs,
            empty_src: self.rel(span.start) + body,
        });
        BlockNode::CodeBlock(
            CodeBlock::new(value.to_string().into(), lang.map(Into::into), None::<Span>)
                .with_leaf_key(key),
        )
    }

    fn block(&mut self, node: &Node) -> Option<BlockNode> {
        let span = Self::span_of(node)?;
        Some(match node {
            Node::Paragraph(p) => {
                BlockNode::Paragraph(self.paragraph(&p.children, span.clone(), span.end))
            }
            Node::Heading(h) => BlockNode::Heading {
                level: h.depth,
                children: self.paragraph(&h.children, span.clone(), span.end),
                span: None,
            },
            Node::ThematicBreak(_) => BlockNode::HorizontalRule { span: None },
            Node::Blockquote(q) => BlockNode::Blockquote {
                children: q.children.iter().filter_map(|c| self.block(c)).collect(),
                span: None,
            },
            Node::FootnoteDefinition(d) => BlockNode::Blockquote {
                children: d.children.iter().filter_map(|c| self.block(c)).collect(),
                span: None,
            },
            Node::List(list) => BlockNode::List {
                children: list.children.iter().filter_map(|c| self.block(c)).collect(),
                ordered: list.ordered,
                start: list.start,
                span: None,
            },
            Node::ListItem(item) => {
                let mut children: Vec<BlockNode> =
                    item.children.iter().filter_map(|c| self.block(c)).collect();
                if children.is_empty() {
                    // An empty item still takes a caret.
                    let p = self
                        .push_leaf(InlineBuilder::default(), self.rel(span.end))
                        .0;
                    children.push(BlockNode::Paragraph(p));
                }
                BlockNode::ListItem {
                    children,
                    spread: item.spread,
                    checked: item.checked,
                    span: None,
                }
            }
            Node::Code(code) => self.code(&code.value, code.lang.clone(), span),
            Node::Math(math) => self.code(&math.value, Some("math".into()), span),
            Node::Html(html) => self.code(&html.value, Some("html".into()), span),
            Node::Yaml(y) => self.code(&y.value, Some("yaml".into()), span),
            Node::Toml(t) => self.code(&t.value, Some("toml".into()), span),
            Node::Table(table) => {
                let aligns = table
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
                    let Node::TableRow(row) = row else { continue };
                    let mut cells = Vec::new();
                    for cell in &row.children {
                        let Node::TableCell(c) = cell else { continue };
                        let cell_span = Self::span_of(cell).unwrap_or(span.clone());
                        cells.push(self.paragraph(&c.children, cell_span.clone(), cell_span.end));
                    }
                    rows.push(cells);
                }
                BlockNode::Table(Table::new(rows, aligns))
            }
            Node::Definition(def) => BlockNode::Definition {
                identifier: def.identifier.clone().into(),
                url: def.url.clone().into(),
                title: def.title.clone().map(Into::into),
                span: None,
            },
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
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

    fn check_matches_full_parse(doc: &VisualDocument, text: &str) {
        let full = VisualDocument::parse(text);
        let spans = |d: &VisualDocument| {
            d.blocks
                .iter()
                .map(|b| (b.start, b.end))
                .collect::<Vec<_>>()
        };
        assert_eq!(spans(doc), spans(&full));
        let leaves = |d: &VisualDocument| {
            d.blocks
                .iter()
                .flat_map(|b| b.leaves.iter().map(|l| (l.text.clone(), l.segs.clone())))
                .collect::<Vec<_>>()
        };
        assert_eq!(leaves(doc), leaves(&full));
    }

    fn apply(
        doc: &mut VisualDocument,
        text: &mut String,
        range: Range<usize>,
        insert: &str,
    ) -> Splice {
        text.replace_range(range.clone(), insert);
        let rope = Rope::from(text.as_str());
        doc.edit(&rope, range, insert.len())
    }

    #[test]
    fn typing_reparses_only_the_neighbourhood() {
        let mut text = SAMPLE.repeat(20);
        let mut doc = VisualDocument::parse(&text);
        let total = doc.blocks.len();
        let at = text.find("Second line").unwrap() + 3;
        let splice = apply(&mut doc, &mut text, at..at, "x");
        assert!(!splice.full);
        assert!(splice.old.len() <= 3, "{splice:?}");
        assert!(splice.parsed_bytes < 400, "{splice:?}");
        assert_eq!(doc.blocks.len(), total);
        check_matches_full_parse(&doc, &text);
    }

    #[test]
    fn structural_edits_match_a_full_parse() {
        let base = SAMPLE.repeat(3);
        let edits: Vec<(usize, usize, &str)> = vec![
            // Merge a heading into the previous block.
            (base.find("# Heading").unwrap() - 1, 1, ""),
            // Turn a paragraph into a heading.
            (base.find("Some *emph").unwrap(), 0, "## "),
            // Open a fence that swallows the rest.
            (base.find("> quoted").unwrap(), 0, "```\n"),
            // Split a paragraph.
            (base.find("Second").unwrap(), 0, "\n"),
            // Delete across blocks.
            (base.find("- item").unwrap(), 30, ""),
            // Lazy continuation into a quote.
            (base.find("> text").unwrap() + 6, 1, ""),
        ];
        for (at, len, insert) in edits {
            let mut text = base.clone();
            let mut doc = VisualDocument::parse(&text);
            apply(&mut doc, &mut text, at..at + len, insert);
            check_matches_full_parse(&doc, &text);
        }
    }

    #[test]
    fn unclosed_fence_detection() {
        assert!(unclosed_fence("```rust\nfn x"));
        assert!(!unclosed_fence("```rust\nfn x\n```"));
        assert!(!unclosed_fence("plain"));
        assert!(unclosed_fence("~~~~\na\n~~~"));
    }

    /// Random edits with Markdown-significant text, each checked against a
    /// whole-document parse.
    #[test]
    fn random_edits_match_a_full_parse() {
        let pieces = [
            "*", "**", "`", "```", "\n", "\n\n", "# ", "- ", "> ", "|", "x", "中", "।", " ",
            "~~~\n", "1. ", "---\n", "[", "](u)",
        ];
        let mut seed: u64 = 0x2545F4914F6CDD1D;
        let mut next = |n: usize| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed % n as u64) as usize
        };
        let mut text = SAMPLE.repeat(2);
        let mut doc = VisualDocument::parse(&text);
        for step in 0..400 {
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
            apply(&mut doc, &mut text, at..end, insert);
            let full = VisualDocument::parse(&text);
            let spans = |d: &VisualDocument| {
                d.blocks
                    .iter()
                    .map(|b| (b.start, b.end))
                    .collect::<Vec<_>>()
            };
            assert_eq!(
                spans(&doc),
                spans(&full),
                "step {step}: {at}..{end} {insert:?}\n{text}"
            );
        }
    }
}
#[cfg(test)]
#[test]
fn spread_probe() {
    for t in ["- a\n- b\n- c\n", "- a\n\n- b\n- c\n", "- a\n- b\n\n  more\n- c\n"] {
        let Ok(Node::Root(r)) = markdown::to_mdast(t, &parse_options()) else { panic!() };
        let Node::List(l) = &r.children[0] else { panic!() };
        let items: Vec<bool> = l.children.iter().map(|c| match c { Node::ListItem(i) => i.spread, _ => false }).collect();
        eprintln!("{t:?}: list.spread={} items={items:?}", l.spread);
    }
}
