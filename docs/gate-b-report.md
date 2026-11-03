# Gate B report: core visual editing

Branch: `gate-b/core-editor`. The gpui-base fork is pinned at `gpui-base-fork-1a` @ `6dd53bc`: pristine 0.8.1, the undo and revision patch, the read-only accessors, and the approved table-slice patch (`Table::measure_column_widths`, `Table::chunk`).

Timings are **CPU-only**: they run from input to the end of paint and exclude GPU raster and presentation. The machine was Linux in a 4-vCPU container under Xvfb with lavapipe (software Vulkan). They show trends; acceptance happens on your Mac (`docs/macos-qa.md`).

## 1. Failures, missed targets and open items

1. **The first frame that shows a large table costs about 50 ms.** It measures every cell once to fix the column widths (400 rows: 49.7 ms frame). Later keystrokes in that table cost about 6.5 ms. A proper fix moves the measurement off the frame, for example by shaping cells on a background executor. That is planned for Gate C.
2. **A 20 KiB paste, and its redo, cost about 30 ms at 50 KiB.** That's one frame over budget: the provisional path lays out many new blocks at once. At 200 KiB and 255 KiB the same case costs 4 ms, so the cost depends on what the paste lands next to rather than on file size.
3. **A few typing p99s sit between 8.3 and 15 ms** (in "type: middle" at 50k, 200k and 255k, and "html comment opener"). Each is one or two samples out of 40, and every p95 is under 8.3 ms. Mac hardware has to show whether these survive outside software rendering.
4. **Some features are still edited only in the source view.** In the visual view, images render, but sizing and the images dialog wait for Gate C. Front matter shows as a code block until the panel arrives (Gate D). HTML blocks render read-only.
5. **Images render small** (about line height) because of GPUI Kit's inline image sizing. Natural size up to the column width is part of the Gate C images work.
6. **Italic shows upright in the Linux screenshots.** The UI font here is DejaVu Sans, which has no italic face installed in this container. Bold, strikethrough and code render correctly. macOS's system font has a true italic; check it there.
7. **Not run or unverified:**
   - The macOS build (Metal shaders need Apple tools).
   - Real IME, trackpad, ProMotion and VoiceOver.
   - Windows and Linux runtime.
   - The Windows **type-check passes** (`x86_64-pc-windows-gnu`).

## 2. What Gate B delivered

| Item | Status | Evidence |
|---|---|---|
| GPUI Kit's parser | **Done**: the visual index is built from `parse_markdown` and its source mapping. Blocks compare by a position-independent signature. The fuzz mismatch rate before the idle settle is 0.24%, and every mismatch is reconciled by the background pass. | `f20a439`; `document.rs` tests, `ENSO_FUZZ` |
| Long lists and tables | **Done**: they are split into 24-row slices. An edit re-renders one slice, and table column widths are shared across slices. | `52ad730`; `rows.rs` test; bench "large list" p99 2.6–7.4 ms, "large table" p99 2.7–10.9 ms |
| Selection | **Done**: shift+arrows, shift+Home/End (⌘⇧←/→ on Mac), mouse drag with edge autoscroll, shift-click, double-click to select a word, triple-click to select a paragraph. The selection paints across leaves, list items and blocks, and the platform input handler replaces it. | `selection_copy_and_replace`, `word_and_paragraph_selection`; screenshot check |
| Copy, cut, paste | **Done**: copy and cut put the exact Markdown source on the clipboard. Paste follows the document's line endings. | `selection_copy_and_replace` |
| Formatting commands | **Done**: bold, italic, strikethrough, inline code, link, H1–H6, body text, bullet, numbered and task lists, toggling a task done, quote, code block, indent and outdent. Each runs as one undoable edit. | `formatting_shortcuts_toggle_markdown` |
| List continuation | **Done**: Enter continues bullet, task and numbered lists, and Enter on an empty item ends the list. | `enter_continues_lists` |
| Vertical movement past the viewport | **Done** (Gate A); the goal x is kept. | `vertical_moves_keep_the_goal_x`, `caret_deep_in_a_tall_block_…` |
| Views in the app | **Done**: Visual, Source and Side by side. `⌘E`/`Ctrl+E` toggles Visual and Source; `⌘\`/`Ctrl+\` toggles side by side. New files open in Visual. Files opened over 256 KiB keep the read-only large reader in Visual. The old read-only preview is gone. | `bbbbe56` |
| Caret preserved across views | **Done**: a switch carries the selection and keeps the caret line within one line of the same height on screen, in both directions. In side by side, the visual side follows the source caret while the source has focus. | `switching_views_keeps_the_caret_and_its_place` |
| Issue 6: render parity | **Mostly done**: headings, emphasis, strikethrough, code, links, reference links, footnotes, quotes, nested quotes, task lists, ordered lists, tables with alignment, code blocks with highlighting, rules, HTML and images all render. See failures 4–6 for the gaps. | Screenshot check of a feature sample |
| Issue 7: reference definitions | **Done**: `[text][id]` resolves through the document's definitions, matched ignoring case. Editing a definition updates every link that uses it. A click places the caret; ⌘-click (Ctrl elsewhere) follows the link. | `reference_definitions_update_every_link` |
| Issue 8: exact-edge clicks | **Done**: GPUI's whole-layout hit test rounds an exact boundary up (seen in code text and table cells). Hits are now refined to the nearest grapheme boundary on the same visual line. The click test no longer offsets by half a pixel. | `clicks_resolve_to_source_bytes` |
| Scroll anchoring | **Done**: an edit above the viewport leaves the content in view where it was. | `edits_above_the_viewport_keep_the_view_anchored` |
| Layout stability | **Done**: typing within a line moves no block, and a new paragraph moves only what is below it. | `typing_does_not_shift_the_layout` |
| Round trip | **Done**: an LF file and a CRLF file with odd but valid Markdown open and go through every view without a write. Their bytes and modification times are unchanged. | `visual_round_trip_and_minimal_diff_on_disk` |
| Minimal diff | **Done**: a word typed in the visual editor autosaves as exactly that insertion, in LF and CRLF files. | same test |

Unchanged behaviour still holds: autosave, serialized writes, external-change detection, the save flush on close, tabs, multi-window and shortcuts. All 63 tests in `cargo test --features ui-tests` pass (2 ignored: the long fuzz and the case replay).

## 3. Benchmark (`make bench`, release)

Work time in ms (input handler + frame), CPU-only. `>8.33` counts samples over the 120 Hz budget.

| Case | 1k p99 | 50k p99 | 200k p99 | 255k p99 | Notes |
|---|---|---|---|---|---|
| type: start | 3.6 | 3.9 | 3.4 | 6.3 | |
| type: middle | 5.7 | 10.4 | 14.7 | 12.8 | 1–2 samples over budget per size; p95 ≤ 7.7 |
| type: end | – | 52.6 (first) / 7.6 p95 | 8.8 | 11.2 | First keystroke shows a 400-row table: failure 1 |
| fence: type inside unclosed | – | 4.9 | 2.2 | 1.9 | 0 bytes reparsed |
| large list: type in item | – | 7.4 | 2.6 | 3.3 | |
| large table: type in cell | – | 10.9 | 2.6 | 2.7 | |
| paste 20 KiB | 5.2 | 29.5 | 4.2 | 3.4 | failure 2 |
| scroll (per frame) | – | 7.6 | 1.8 | 2.0 | |

Open (main thread before the first frame): 131–179 ms at every size. `VisualView` accounts for 1–8 ms and `EditorState` for under 1 ms; the rest is window and surface creation under lavapipe. The full tables are in `target/bench/*.jsonl` after `make bench`.

## 4. For you to check on the Mac

- Type, select (drag, double-click, triple-click, shift+arrows) and use the formatting shortcuts in a real document. Check that italic draws in italic.
- `⌘E` and `⌘\` with the caret deep in a long file: the caret line should stay at the same height.
- ⌘-click a link, including a `[text][ref]` link; a plain click should only place the caret.
- Paste a large block, and open a file with a 400-row table: does the first frame hitch?
- IME (Japanese, Chinese, emoji picker) in the visual editor.

## 5. Next (Gate C)

- Move table width measurement off the frame (failure 1), and split large provisional pastes (failure 2).
- Images: natural size up to the column, the images dialog, and pasting images in the visual view.
- Tables and code editing affordances.
- Begin the surface work: page sheet, typography and theme tokens, hiding chrome while typing, and caret settings.
