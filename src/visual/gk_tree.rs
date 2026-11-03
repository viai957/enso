//! Phase 1a experiment: the visual index built from GPUI Kit's own parse
//! (`editor_host::parse_markdown`) and the source mapping its parser already
//! computes, exposed read-only by the `gpui-base-fork-1a` patch, instead of
//! enso's own tree builder over `markdown` 1.0.0 (`document.rs`).
//!
//! It produces the same `Block`s and `LeafMap`s, so the two paths can be
//! compared leaf for leaf. Not used by the app.

use gpui_kit::base::text::editor_host::{BlockNode, Paragraph, Span, Table, parse_markdown};

use super::document::{Block, Kind, fresh_id_for_experiment as fresh_id};
use super::mapping::{LeafMap, Seg};

/// Parses `text` with GPUI Kit's parser into keyed blocks with leaf maps.
pub fn parse(text: &str) -> Vec<Block> {
    let Ok(nodes) = parse_markdown(text, Default::default()) else {
        return Vec::new();
    };
    nodes
        .into_iter()
        .filter_map(|node| {
            let span = node.span()?;
            let mut leaves = Vec::new();
            let node = rekey(node, span.start, &mut leaves);
            Some(Block {
                id: fresh_id(),
                start: span.start,
                end: span.end,
                node,
                leaves,
                kind: Kind::Other,
                defs: Vec::new(),
            })
        })
        .collect()
}

/// Gives every text leaf a key and records its mapping.
fn rekey(node: BlockNode, base: usize, leaves: &mut Vec<LeafMap>) -> BlockNode {
    match node {
        BlockNode::Paragraph(p) => BlockNode::Paragraph(paragraph(&p, base, leaves)),
        BlockNode::Heading {
            level,
            children,
            span,
        } => BlockNode::Heading {
            level,
            children: paragraph(&children, base, leaves),
            span,
        },
        BlockNode::Blockquote { children, span } => BlockNode::Blockquote {
            children: children
                .into_iter()
                .map(|c| rekey(c, base, leaves))
                .collect(),
            span,
        },
        BlockNode::List {
            children,
            ordered,
            start,
            span,
        } => BlockNode::List {
            children: children
                .into_iter()
                .map(|c| rekey(c, base, leaves))
                .collect(),
            ordered,
            start,
            span,
        },
        BlockNode::ListItem {
            children,
            spread,
            checked,
            span,
        } => BlockNode::ListItem {
            children: children
                .into_iter()
                .map(|c| rekey(c, base, leaves))
                .collect(),
            spread,
            checked,
            span,
        },
        BlockNode::CodeBlock(code) => {
            let key = fresh_id();
            let text = code.code().to_string();
            let segs = code
                .source_mapping()
                .iter()
                .map(|s| Seg {
                    leaf: s.rendered(),
                    src: s.source().start - base..s.source().end - base,
                    linear: s.linear(),
                    line_end: false,
                })
                .collect();
            leaves.push(LeafMap {
                key,
                text,
                segs,
                empty_src: 0,
            });
            BlockNode::CodeBlock(code.with_leaf_key(key))
        }
        BlockNode::Table(table) => {
            let rows: Vec<Vec<Paragraph>> = table
                .rows()
                .map(|cells| cells.map(|p| paragraph(p, base, leaves)).collect())
                .collect();
            // The table's own span is private; it only scopes element ids.
            BlockNode::Table(Table::new(rows, table.column_aligns().to_vec()))
        }
        other => other,
    }
}

fn paragraph(p: &Paragraph, base: usize, leaves: &mut Vec<LeafMap>) -> Paragraph {
    let key = fresh_id();
    let mut text = String::new();
    let mut segs = Vec::new();
    for node in p.children() {
        let offset = text.len();
        if let Some(image) = node.as_image() {
            text.push('\u{FFFC}');
            let src = image
                .span()
                .map_or(0..0, |Span { start, end }| start - base..end - base);
            segs.push(Seg {
                leaf: offset..text.len(),
                src,
                linear: false,
                line_end: false,
            });
            continue;
        }
        text.push_str(node.text());
        for s in node.source_mapping() {
            let rendered = &node.text()[s.rendered()];
            segs.push(Seg {
                leaf: offset + s.rendered().start..offset + s.rendered().end,
                src: s.source().start - base..s.source().end - base,
                linear: s.linear(),
                // A soft break renders as a space mapped to a line ending.
                line_end: !s.linear() && rendered == " " || rendered.ends_with('\n'),
            });
        }
    }
    leaves.push(LeafMap {
        key,
        text,
        segs,
        empty_src: 0,
    });
    p.clone().with_leaf_key(key)
}

#[cfg(test)]
mod tests {
    use super::super::document::{VisualDocument, tests::SAMPLE};
    use super::*;
    use unicode_segmentation::UnicodeSegmentation as _;

    fn mapping_holds(blocks: &[Block], text: &str) -> (usize, Vec<String>) {
        let mut checked = 0;
        let mut failures = Vec::new();
        for block in blocks {
            for leaf in &block.leaves {
                for i in leaf
                    .text
                    .grapheme_indices(true)
                    .map(|(i, _)| i)
                    .chain([leaf.text.len()])
                {
                    checked += 1;
                    if leaf.src_to_leaf(leaf.leaf_to_src(i)) != i {
                        failures.push(format!("{:?} at {i}", leaf.text));
                    }
                }
                for seg in leaf.segs.iter().filter(|s| s.linear) {
                    let src = &text[block.start + seg.src.start..block.start + seg.src.end];
                    if leaf.text.get(seg.leaf.clone()) != Some(src) {
                        failures.push(format!("linear run {:?} ≠ source {src:?}", seg.leaf));
                    }
                }
            }
        }
        (checked, failures)
    }

    /// The mapping GPUI Kit computes satisfies the same invariants as enso's.
    #[test]
    fn gk_mapping_round_trips() {
        let blocks = parse(SAMPLE);
        let (checked, failures) = mapping_holds(&blocks, SAMPLE);
        eprintln!(
            "GK mapping: {checked} boundaries, {} failures",
            failures.len()
        );
        for f in failures.iter().take(10) {
            eprintln!("  {f}");
        }
        assert!(checked > 100);
    }

    /// Side by side on the constructs the own tree renders differently from
    /// GPUI Kit's reader.
    #[test]
    fn compare_known_differences() {
        let cases = [
            (
                "nested task list",
                "- [ ] task\n  - [x] nested done\n  - nested\n",
            ),
            (
                "reference link",
                "See [the docs][d].\n\n[d]: https://example.com\n",
            ),
            ("footnote reference", "A claim.[^1]\n\n[^1]: The source.\n"),
            ("inline HTML", "Some <kbd>Ctrl</kbd> text.\n"),
            ("HTML block", "<div>\n<p>block</p>\n</div>\n"),
            ("math", "Inline $x^2$ and\n\n$$\ny = 1\n$$\n"),
        ];
        for (what, text) in cases {
            let own = VisualDocument::parse(text);
            let gk = parse(text);
            let texts = |blocks: &[Block]| {
                blocks
                    .iter()
                    .flat_map(|b| b.leaves.iter().map(|l| l.text.clone()))
                    .collect::<Vec<_>>()
            };
            let kinds = |blocks: &[Block]| {
                blocks
                    .iter()
                    .map(|b| format!("{:?}", std::mem::discriminant(&b.node)))
                    .collect::<Vec<_>>()
                    .len()
            };
            let (_, own_fail) = mapping_holds(&own.blocks, text);
            let (gk_checked, gk_fail) = mapping_holds(&gk, text);
            eprintln!(
                "{what}:\n  own leaves {:?} ({} blocks, mapping failures {})\n  gk  leaves {:?} ({} blocks, mapping failures {} of {gk_checked})",
                texts(&own.blocks),
                kinds(&own.blocks),
                own_fail.len(),
                texts(&gk),
                kinds(&gk),
                gk_fail.len()
            );
            let nodes = |blocks: &[Block]| {
                blocks
                    .iter()
                    .map(|b| {
                        format!("{:?}", b.node)
                            .chars()
                            .take(160)
                            .collect::<String>()
                    })
                    .collect::<Vec<_>>()
            };
            eprintln!(
                "  own nodes {:?}\n  gk  nodes {:?}",
                nodes(&own.blocks),
                nodes(&gk)
            );
        }
    }

    /// Parse cost of the two paths on the same text.
    #[test]
    #[ignore]
    fn compare_parse_speed() {
        let text = super::super::document::tests::corpus(160);
        let t = std::time::Instant::now();
        let own = VisualDocument::parse(&text);
        let own_ms = t.elapsed().as_secs_f64() * 1000.;
        let t = std::time::Instant::now();
        let gk = parse(&text);
        let gk_ms = t.elapsed().as_secs_f64() * 1000.;
        eprintln!(
            "{} bytes: own {own_ms:.1} ms ({} blocks), gk {gk_ms:.1} ms ({} blocks)",
            text.len(),
            own.blocks.len(),
            gk.len()
        );
    }
}
