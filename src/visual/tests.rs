//! UI harness tests for the spike: hit testing, caret placement, typing, IME
//! and vertical movement through a real GPUI window (test platform).

use gpui_kit::component::input::EditorState;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Entity, EntityInputHandler as _, TestAppContext,
    WindowBounds, WindowOptions, point, px, size,
};
use unicode_segmentation::UnicodeSegmentation as _;

use super::document::{key_block_id, key_leaf_ix};
use super::view::VisualView;

const DOC: &str = "# Heading with **bold** words\n\nA paragraph with *emphasis*, `code`, a [link](u) and \\* escape &amp; more.\nSoft break नमस्ते दुनिया। 中文 👍🏽 end.\n\n- first item\n- [ ] second **task**\n  - nested\n\n> quoted\n> lines\n\n```rust\nfn main() {}\nlet 中 = 1;\n```\n\n| a | b |\n|---|--:|\n| 中文 | 👍🏽 x |\n";

fn open(cx: &mut TestAppContext, text: &str) -> (AnyWindowHandle, Entity<VisualView>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        super::view::init(cx);
    });
    let text = text.to_string();
    let (handle, view) = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Default::default(),
                    size: size(px(1000.), px(2000.)),
                })),
                ..Default::default()
            },
            cx,
            |window, cx| {
                let editor = cx.new(|cx| EditorState::new(window, cx).default_value(text));
                let view = cx.new(|cx| VisualView::new(editor, window, cx));
                view.update(cx, |v, cx| window.focus(&v.focus, cx));
                view
            },
        )
        .unwrap()
    });
    draw(cx, handle);
    (handle, view)
}

fn draw(cx: &mut TestAppContext, handle: AnyWindowHandle) {
    cx.update_window(handle, |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.run_until_parked();
}

fn source(cx: &mut TestAppContext, view: &Entity<VisualView>) -> String {
    cx.update(|cx| view.read(cx).editor.read(cx).text().to_string())
}

/// Clicking at every caret position of every leaf (paragraphs, headings,
/// list items, quotes, code, table cells) resolves to the source byte of
/// that position, and that byte holds the character rendered there.
#[gpui_kit::test]
fn clicks_resolve_to_source_bytes(cx: &mut TestAppContext) {
    let (_, view) = open(cx, DOC);
    cx.update(|cx| {
        let view = view.read(cx);
        let geometry = view.geometry.borrow();
        let mut checked = 0;
        let mut kinds = std::collections::HashSet::new();
        for (&key, leaf) in geometry.iter() {
            let bix = view
                .doc
                .blocks
                .iter()
                .position(|b| b.id == key_block_id(key))
                .unwrap();
            let lix = key_leaf_ix(key);
            let block = &view.doc.blocks[bix];
            kinds.insert(std::mem::discriminant(&block.node));
            let map = &block.leaves[lix];
            assert_eq!(leaf.len(), map.len(), "leaf {:?}", map.text);
            let boundaries = map
                .text
                .grapheme_indices(true)
                .map(|(i, _)| i)
                .chain([map.len()]);
            for i in boundaries {
                let at = leaf.position_for_index(i);
                // Half a pixel left of the boundary: GPUI's whole-layout hit
                // test rounds an exact boundary up in code text.
                let click = point(at.x - px(0.5), at.y + leaf.line_height() / 2.);
                let src = view.source_for_point(click).unwrap();
                let expected = view.doc.src_of(bix, lix, i);
                // A soft-wrap edge is one caret position for two indices.
                let (b2, l2, i2) = view.doc.locate(src).unwrap();
                assert_eq!((b2, l2), (bix, lix), "{:?} at {i}", map.text);
                assert_eq!(
                    leaf.position_for_index(i2),
                    at,
                    "{:?} at {i}: clicked {src}, expected {expected}",
                    map.text
                );
                // The source the mapping gives for a character inside a
                // plain run holds that character.
                if let Some(c) = map.text[i..].chars().next()
                    && let Some(seg) = map.segs.iter().find(|s| s.linear && s.leaf.contains(&i))
                {
                    let at = block.start + seg.src.start + (i - seg.leaf.start);
                    assert!(DOC[at..].starts_with(c), "{:?} at {i}", map.text);
                }
                let _ = expected;
                checked += 1;
            }
        }
        // Heading, paragraph, list, quote, code and table all took part.
        assert!(kinds.len() >= 6, "{} block kinds", kinds.len());
        assert!(checked > 150, "{checked}");
    });
}

/// Typing at a caret in Devanagari, CJK and after an emoji inserts at the
/// source byte, keeps the caret after the new text and reparses one block.
#[gpui_kit::test]
fn typing_updates_the_rope_and_reparses_one_block(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, DOC);
    for anchor in ["दुनिया", "文 👍🏽", " end."] {
        let before = source(cx, &view);
        let at = before.find(anchor).unwrap();
        let blocks = cx.update(|cx| view.read(cx).doc.blocks.len());
        cx.update(|cx| view.update(cx, |v, _| v.caret = at));
        cx.simulate_input(handle, "x");
        draw(cx, handle);
        let after = source(cx, &view);
        let mut expected = before.clone();
        expected.insert(at, 'x');
        assert_eq!(after, expected);
        cx.update(|cx| {
            let v = view.read(cx);
            assert_eq!(v.caret, at + 1);
            assert_eq!(v.doc.blocks.len(), blocks);
            let parsed = *v.latency.borrow().parsed_bytes.last().unwrap();
            assert!(parsed < 400, "reparsed {parsed} bytes");
            // The caret paints in the edited leaf.
            assert!(v.caret_position().is_some());
        });
    }
}

/// IME composition: marked text is in the rope with its range reported in
/// UTF-16, and committing replaces it.
#[gpui_kit::test]
fn ime_composition_marks_and_commits(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, DOC);
    let at = DOC.find("end.").unwrap();
    cx.update(|cx| view.update(cx, |v, _| v.caret = at));
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |v, cx| {
            v.replace_and_mark_text_in_range(None, "ni", Some(2..2), window, cx);
            let marked = v.marked_text_range(window, cx).unwrap();
            let start16 = v.editor.read(cx).text().to_string()[..at]
                .encode_utf16()
                .count();
            assert_eq!(marked, start16..start16 + 2);
            assert_eq!(v.caret, at + 2);
            v.replace_and_mark_text_in_range(None, "你h", Some(2..2), window, cx);
            let bounds = v.bounds_for_range(marked.clone(), Bounds::default(), window, cx);
            assert!(bounds.is_some(), "the IME window can be placed");
            v.replace_text_in_range(None, "你好", window, cx);
            assert!(v.marked_text_range(window, cx).is_none());
        })
    })
    .unwrap();
    draw(cx, handle);
    let text = source(cx, &view);
    assert!(text.contains("中文 👍🏽 你好end."), "{text}");
    cx.update(|cx| assert_eq!(view.read(cx).caret, at + "你好".len()));
}

/// Down, down, up, up returns to the same character across a heading (large
/// text), a paragraph and a list: the goal x survives different line heights.
#[gpui_kit::test]
fn vertical_moves_keep_the_goal_x(cx: &mut TestAppContext) {
    let doc = "# A heading line that is big\n\nA paragraph of ordinary body text that runs along.\n\n- a list item with text\n";
    let (handle, view) = open(cx, doc);
    let start = doc.find("that").unwrap();
    cx.update(|cx| view.update(cx, |v, _| v.caret = start));
    draw(cx, handle);
    let mut ys = vec![];
    for key in ["down", "down", "up", "up"] {
        cx.simulate_keystrokes(handle, key);
        draw(cx, handle);
        ys.push(cx.update(|cx| view.read(cx).caret_position().unwrap().0.y));
    }
    assert!(ys[0] < ys[1], "down moves down: {ys:?}");
    assert!(ys[2] < ys[1] && ys[3] < ys[2], "up moves up: {ys:?}");
    cx.update(|cx| assert_eq!(view.read(cx).caret, start, "back to the same character"));
    // Left and right step over hidden markers by grapheme.
    let emoji = "x 👍🏽 **b** y\n";
    let (handle, view) = open(cx, emoji);
    cx.update(|cx| view.update(cx, |v, _| v.caret = 2));
    cx.simulate_keystrokes(handle, "right");
    cx.update(|cx| assert_eq!(view.read(cx).caret, 2 + "👍🏽".len()));
    cx.simulate_keystrokes(handle, "right right");
    cx.update(|cx| assert_eq!(view.read(cx).caret, emoji.find("b**").unwrap() + 1));
}

/// Known gap (open decision for Phase 1): `EditorState`'s undo and redo
/// handlers are registered by the `Editor` element, and `undo`/`redo` are not
/// public, so while only the visual view is rendered they never run. This test
/// pins the current behaviour so the fix shows up as a change here.
#[gpui_kit::test]
fn undo_does_not_yet_reach_an_unrendered_editor(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, "one two\n");
    cx.update(|cx| view.update(cx, |v, _| v.caret = 3));
    cx.simulate_input(handle, "x");
    draw(cx, handle);
    assert_eq!(source(cx, &view), "onex two\n");
    cx.simulate_keystrokes(handle, "ctrl-z");
    draw(cx, handle);
    assert_eq!(source(cx, &view), "onex two\n");
}
