# Phase 0: visual editing spike on `editor_host`

Branch: `spike/phase0-editor-host` (throwaway). Base: `71ecd91`, GPUI Kit 0.8.1.

Run it: `cargo run --release --example visual_spike -- file.md`.
Benchmark: `ENSO_SPIKE_AUTOTYPE=500 cargo run --release --example visual_spike -- file.md`.

## What was built

- `src/visual/mapping.rs`: leaf-byte ↔ source-byte segments (linear runs, escapes, entities, soft breaks, image atoms, container prefixes).
- `src/visual/document.rs`: top-level block index. It builds GPUI Kit `BlockNode` trees with leaf keys from `markdown` 1.0.0 positions, reparses the edited blocks plus one neighbor on each side, and checks that the neighbors come back unchanged before accepting the reparse. Otherwise it widens the window and, as a last resort, reparses the whole document. Reparsed blocks keep their ids, so leaf keys stay stable.
- `src/visual/view.rs`: a virtualized `list` inside a custom element that prepaints in `collect_leaf_geometry`, paints the caret, hit-tests, handles left/right/up/down/home/end, and does Backspace, Delete and Enter.
- `src/visual/input.rs`: `EntityInputHandler`. Marked text lives in the rope, and ranges are UTF-16 over the source.
- `src/visual/caret.rs`: a fixed 530 ms blink.
- Edits go through `EditorState::set_selected_range` and `replace`, the same path as `apply_markdown`.

## Results

| Requirement | Status | Evidence |
|---|---|---|
| Click → source byte (paragraph, heading, list, quote, code, table) | Works | `clicks_resolve_to_source_bytes`: 193 caret positions across 6 block kinds, plus manual X11 clicks |
| Caret with emoji, Devanagari, CJK | Works | Grapheme-boundary round-trip tests, and the harness checks caret positions for `नमस्ते दुनिया।`, `中文`, `👍🏽`, `👩‍👩‍👧` |
| Typing → rope, reparse one block, no layout jump | Works | ~1 KB reparsed per keystroke (p50 1005 B, max 1254 B). A random-edit test (400 steps) matches a full parse |
| IME marked text | Works through the API | `ime_composition_marks_and_commits`. **No real IME was exercised** (none in the Linux container; no macOS) |
| Up/down keep goal x across sizes | Works | `vertical_moves_keep_the_goal_x` (heading → paragraph → list and back lands on the same byte) |
| Undo/redo | **Does not work** | `EditorState` registers its undo handlers on the `Editor` element, and `undo` / `redo` are `pub(super)`. See `undo_does_not_yet_reach_an_unrendered_editor` |

## Latency (release build)

Setup: Linux, Xvfb with lavapipe (software Vulkan), a 4-vCPU container, and a 206,649-byte generated corpus with 1,078 blocks (headings, emphasis, links, Devanagari, CJK, emoji, lists, tasks, quotes, code and tables). One sample runs from the key-down reaching the view to the end of the surface's paint, when the CPU scene is built. It excludes GPU raster, present and the compositor. It includes waiting for the next frame tick, about 60 Hz under Xvfb.

| Run | n | min | p50 | p90 | p95 | p99 | max |
|---|---|---|---|---|---|---|---|
| Autotype, caret mid-document | 500 | 3.17 | 12.15 | 18.57 | **19.58** | 20.73 | 23.67 |
| ⤷ input → frame start | 500 | 0.79 | 8.31 | 14.45 | 15.26 | 15.97 | 17.21 |
| ⤷ frame (render → end of paint) | 500 | 2.29 | 3.73 | 5.15 | 5.96 | 6.89 | 9.41 |
| ⤷ `EditorState::replace` | 500 | 0.15 | 0.27 | 0.37 | 0.41 | 0.48 | 1.16 |
| ⤷ block reparse | 500 | 0.32 | 0.51 | 0.68 | 0.75 | 1.19 | 1.83 |
| Autotype, caret near start | 300 | 2.52 | 10.51 | 17.12 | 17.96 | 19.16 | 20.14 |
| Autotype, caret near end | 300 | 2.79 | 12.41 | 18.70 | 20.28 | 21.83 | 28.02 |
| xdotool (real X11 key events) | 301 | 1.93 | 12.94 | 17.73 | 18.73 | 24.43 | 27.00 |

All values are in milliseconds. Opening the 200 KiB file takes 147–167 ms of full parsing on the main thread.

## Known differences and gaps

- **Inline source mapping needs a separate position-aware parse.** GPUI Kit computes `SourceSegment`s but keeps them `pub(crate)`, and `editor_host::parse_markdown` returns a tree without them. The spike builds its own tree from `markdown` 1.0.0, the same crate and version GPUI Kit parses with, so no extra code is compiled. Rendering differences are possible wherever GPUI Kit post-processes the AST:
  - A nested list inside a task item renders without its bullet.
  - Reference links render underlined but carry no URL.
  - Footnote references and inline HTML render as code.
  - Math is off.
  - HTML blocks render as code.
- In code blocks, GPUI's whole-layout hit test rounds an exact glyph boundary up by one character. The tests click 0.5 px inside.
- Not in the spike: selection, copy, Enter list continuation, accessibility, caret settings, scroll-past-viewport vertical movement (it is implemented as scroll and retry, but untested beyond the viewport), and background initial parse.
- Not run: macOS, Windows, a real IME, screen readers.
