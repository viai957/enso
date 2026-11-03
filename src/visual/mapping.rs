//! Conversion between Markdown source bytes and the bytes of a rendered leaf.
//!
//! A leaf is the text one paragraph, heading, code block or table cell
//! renders: `**bold** text` renders as `bold text`. GPUI Kit's
//! `editor_host` reports geometry in leaf bytes; the editor keeps its caret
//! in source bytes. A [`LeafMap`] pairs the two with segments, each a run of
//! leaf bytes and the source bytes it was rendered from.
//!
//! GPUI Kit records the same pairing internally (`SourceSegment`) but does not
//! expose it, so the host builds its own from `markdown` crate positions.

use std::ops::Range;

/// One run of a leaf and the source it came from. Offsets in `src` are
/// relative to the start of the block that owns the leaf, so that an edit
/// earlier in the document only shifts block starts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Seg {
    pub leaf: Range<usize>,
    pub src: Range<usize>,
    /// Each leaf byte came from the source byte at the same distance from
    /// the start (plain text). Otherwise the run maps only as a whole: an
    /// escape, an entity, an image atom, a hard break.
    pub linear: bool,
    /// The run is a line break (soft or hard): a caret after it belongs to
    /// the next line, past any container prefix.
    pub line_end: bool,
}

/// The source mapping of one keyed leaf.
#[derive(Clone, Debug, Default)]
pub struct LeafMap {
    pub key: u64,
    /// The rendered text, an image counting as one `U+FFFC`.
    pub text: String,
    pub segs: Vec<Seg>,
    /// Where text typed into the leaf goes while it is empty (block relative).
    pub empty_src: usize,
}

impl LeafMap {
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// The block-relative source range the leaf's text came from.
    pub fn src_range(&self) -> Range<usize> {
        match (self.segs.first(), self.segs.last()) {
            (Some(first), Some(last)) => first.src.start..last.src.end,
            _ => self.empty_src..self.empty_src,
        }
    }

    /// The source offset of the caret at leaf byte `index`.
    ///
    /// At the boundary between two runs the caret takes the end of the run
    /// before it, so text typed after `**bold**` continues the bold run, as a
    /// word processor continues the style of the preceding character.
    pub fn leaf_to_src(&self, index: usize) -> usize {
        let Some(first) = self.segs.first() else {
            return self.empty_src;
        };
        if index <= first.leaf.start {
            return first.src.start;
        }
        // At the start of a visual line the caret belongs to the run that
        // follows: the source between them is a container prefix (`> `,
        // list indentation), and text typed there must land after it.
        let after_newline = self.text[..index.min(self.text.len())].ends_with('\n');
        for (ix, seg) in self.segs.iter().enumerate() {
            if (after_newline || seg.line_end)
                && index == seg.leaf.end
                && let Some(next) = self.segs.get(ix + 1)
                && next.leaf.start == index
            {
                return next.src.start;
            }
            if index > seg.leaf.start && index <= seg.leaf.end {
                return if seg.linear {
                    seg.src.start + (index - seg.leaf.start)
                } else if index == seg.leaf.end {
                    seg.src.end
                } else {
                    seg.src.start
                };
            }
        }
        self.segs.last().map_or(self.empty_src, |seg| seg.src.end)
    }

    /// The leaf byte for block-relative source offset `src`. Source that
    /// renders nothing (a marker) snaps to the end of the run before it, or
    /// to the start of the leaf.
    pub fn src_to_leaf(&self, src: usize) -> usize {
        let Some(first) = self.segs.first() else {
            return 0;
        };
        if src <= first.src.start {
            return first.leaf.start;
        }
        let mut best = first.leaf.start;
        for seg in &self.segs {
            if src < seg.src.start {
                break;
            }
            if src <= seg.src.end {
                return if seg.linear {
                    let index = seg.leaf.start + (src - seg.src.start);
                    floor_char_boundary(&self.text, index.min(seg.leaf.end))
                } else if src == seg.src.start {
                    seg.leaf.start
                } else {
                    seg.leaf.end
                };
            }
            best = seg.leaf.end;
        }
        best
    }

    /// Whether the caret can rest at leaf byte `index`: not strictly inside a
    /// run that maps only as a whole (an entity, synthesized text like a
    /// footnote's `[1]`, an image atom).
    pub fn is_stop(&self, index: usize) -> bool {
        !self
            .segs
            .iter()
            .any(|s| !s.linear && s.leaf.start < index && index < s.leaf.end)
    }

    /// `index`, moved out of a whole-mapped run to its end (`forward`) or start.
    pub fn snap_index(&self, index: usize, forward: bool) -> usize {
        match self
            .segs
            .iter()
            .find(|s| !s.linear && s.leaf.start < index && index < s.leaf.end)
        {
            Some(s) if forward => s.leaf.end,
            Some(s) => s.leaf.start,
            None => index,
        }
    }

    /// Distance from `src` to this leaf's source, zero when inside.
    pub fn src_distance(&self, src: usize) -> usize {
        let range = self.src_range();
        if src < range.start {
            range.start - src
        } else {
            src.saturating_sub(range.end)
        }
    }
}

pub fn floor_char_boundary(text: &str, mut index: usize) -> usize {
    index = index.min(text.len());
    while !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

/// The length of the character reference (`&amp;`, `&#38;`, `&#x26;`) that
/// `text` starts with.
fn entity_len(text: &str) -> Option<usize> {
    let body = text.strip_prefix('&')?;
    let end = body.find(';')?;
    let name = &body[..end];
    let valid = !name.is_empty()
        && end <= 32
        && (name.chars().all(|c| c.is_ascii_alphanumeric())
            || name.strip_prefix('#').is_some_and(|n| {
                n.strip_prefix(['x', 'X']).map_or(
                    !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()),
                    |h| !h.is_empty() && h.chars().all(|c| c.is_ascii_hexdigit()),
                )
            }));
    valid.then_some(end + 2)
}

/// Builds a leaf: its text, its marks and its source segments.
#[derive(Default)]
pub struct LeafBuilder {
    pub text: String,
    pub segs: Vec<Seg>,
}

impl LeafBuilder {
    fn push_seg(&mut self, seg: Seg) {
        if let Some(last) = self.segs.last_mut()
            && last.linear
            && seg.linear
            && last.leaf.end == seg.leaf.start
            && last.src.end == seg.src.start
        {
            last.leaf.end = seg.leaf.end;
            last.src.end = seg.src.end;
            return;
        }
        self.segs.push(seg);
    }

    /// Appends `value` that the parser read from `source` (which starts at
    /// block-relative `base`), pairing each character with the source it
    /// came from. Source the value skips (container prefixes, stripped
    /// indentation) maps to nothing; escapes and entities map as a whole.
    pub fn push_aligned(&mut self, value: &str, source: &str, base: usize) {
        self.align(value, source, base, false);
    }

    /// Like [`push_aligned`](Self::push_aligned) for paragraph text, where a
    /// soft line break renders as a space (as GPUI Kit renders it) that maps
    /// as a whole to the line ending.
    pub fn push_prose(&mut self, value: &str, source: &str, base: usize) {
        self.align(value, source, base, true);
    }

    fn align(&mut self, value: &str, source: &str, base: usize, soft_breaks: bool) {
        let mut j = 0;
        let bytes = source.as_bytes();
        let mut chars = value.char_indices().peekable();
        while let Some((i, c)) = chars.next() {
            let leaf_start = self.text.len();
            if soft_breaks && (c == '\n' || c == '\r') {
                if c == '\r' && chars.peek().is_some_and(|&(_, n)| n == '\n') {
                    chars.next();
                }
                self.text.push(' ');
                // Trailing spaces before the line ending render nothing.
                while j < source.len() && matches!(bytes[j], b' ' | b'\t') {
                    j += 1;
                }
                let end = if source[j..].starts_with("\r\n") {
                    j + 2
                } else if j < source.len() && matches!(bytes[j], b'\n' | b'\r') {
                    j + 1
                } else {
                    j
                };
                self.push_seg(Seg {
                    leaf: leaf_start..self.text.len(),
                    src: base + j..base + end,
                    linear: false,
                    line_end: true,
                });
                j = end;
                continue;
            }
            self.text.push(c);
            let leaf = leaf_start..self.text.len();
            loop {
                if j >= source.len() {
                    // The value outran its source: map to the end, as a whole.
                    self.push_seg(Seg {
                        leaf: leaf.clone(),
                        src: base + source.len()..base + source.len(),
                        linear: false,
                        line_end: false,
                    });
                    break;
                }
                let rest = &source[j..];
                // A decoded entity, unless the value holds its text literally
                // (code spans do not decode).
                if bytes[j] == b'&'
                    && let Some(end) = entity_len(rest)
                    && !value[i..].starts_with(&rest[..end])
                {
                    self.push_seg(Seg {
                        leaf: leaf.clone(),
                        src: base + j..base + j + end,
                        linear: false,
                        line_end: false,
                    });
                    j += end;
                    break;
                }
                if rest.starts_with(c) {
                    self.push_seg(Seg {
                        leaf: leaf.clone(),
                        src: base + j..base + j + c.len_utf8(),
                        linear: true,
                        line_end: false,
                    });
                    j += c.len_utf8();
                    break;
                }
                if bytes[j] == b'\\' && rest[1..].starts_with(c) {
                    self.push_seg(Seg {
                        leaf: leaf.clone(),
                        src: base + j..base + j + 1 + c.len_utf8(),
                        linear: false,
                        line_end: false,
                    });
                    j += 1 + c.len_utf8();
                    break;
                }
                // Unrendered source: skip a character.
                j += rest.chars().next().map_or(1, char::len_utf8);
            }
        }
    }

    /// Appends text that maps as a whole to `src` (an image atom, a break).
    pub fn push_atomic(&mut self, text: &str, src: Range<usize>) {
        let start = self.text.len();
        self.text.push_str(text);
        self.push_seg(Seg {
            leaf: start..self.text.len(),
            src,
            linear: false,
            line_end: text == "\n",
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use unicode_segmentation::UnicodeSegmentation as _;

    fn leaf(value: &str, source: &str) -> LeafMap {
        let mut b = LeafBuilder::default();
        b.push_aligned(value, source, 0);
        LeafMap {
            key: 0,
            text: b.text,
            segs: b.segs,
            empty_src: 0,
        }
    }

    #[test]
    fn plain_text_is_one_linear_run() {
        let map = leaf("héllo 世界 👍🏽", "héllo 世界 👍🏽");
        assert_eq!(map.segs.len(), 1);
        assert!(map.segs[0].linear);
    }

    #[test]
    fn soft_breaks_render_as_spaces_mapped_to_the_line_ending() {
        let mut b = LeafBuilder::default();
        b.push_prose("one\ntwo", "one  \r\n> two", 0);
        let map = LeafMap {
            key: 0,
            text: b.text,
            segs: b.segs,
            empty_src: 0,
        };
        assert_eq!(map.text, "one two");
        assert_eq!(map.leaf_to_src(3), 3);
        assert_eq!(map.leaf_to_src(4), 9, "after the break: after the prefix");
        assert_eq!(
            map.src_to_leaf(5),
            3,
            "the line ending maps to the space as a whole"
        );
    }

    #[test]
    fn escapes_and_entities_map_as_a_whole() {
        let map = leaf("a*b&c", r"a\*b&amp;c");
        assert_eq!(map.leaf_to_src(1), 1);
        assert_eq!(map.leaf_to_src(2), 3);
        assert_eq!(map.leaf_to_src(4), 9);
        assert_eq!(map.src_to_leaf(2), 2, "inside an escape snaps to its end");
    }

    #[test]
    fn container_prefixes_are_skipped() {
        let map = leaf("one\ntwo", "one\n> two");
        assert_eq!(map.leaf_to_src(4), 6, "a new line starts after the prefix");
        assert_eq!(map.src_to_leaf(5), 4);
    }

    #[test]
    fn leaf_to_src_to_leaf_is_identity_at_grapheme_boundaries() {
        let cases = [
            ("नमस्ते दुनिया। अगला वाक्य।", "नमस्ते दुनिया। अगला वाक्य।"),
            ("中文 and 👍🏽 x", "中文 and 👍🏽 x"),
            ("a*b&c", r"a\*b&amp;c"),
            ("one\ntwo", "one\n> two"),
        ];
        for (value, source) in cases {
            let map = leaf(value, source);
            let boundaries = value
                .grapheme_indices(true)
                .map(|(i, _)| i)
                .chain([value.len()]);
            for i in boundaries {
                assert_eq!(map.src_to_leaf(map.leaf_to_src(i)), i, "{value:?} at {i}");
            }
        }
    }
}
