# Gate A report: foundations

Branch: `gate-a/foundations`, from `main` (`71ecd91`). The gpui-base fork is pinned at `gpui-base-fork` @ `5f739b4`. The parser experiment lives on `gate-a/parser-1a` with the fork at `gpui-base-fork-1a` @ `ba13470`.

All timings are **CPU-only**: they run from input to the end of paint and exclude GPU raster and presentation. The machine was Linux in a 4-vCPU container under Xvfb with lavapipe (software Vulkan). These numbers show trends but can't pass the contract; acceptance happens on your Mac (`docs/macos-qa.md`).

## 1. Failures and missed targets

1. **Large top-level lists and tables blow the frame budget when they are on screen.** The 400-item list costs work p50 13–15 ms and p99 40–52 ms per keystroke. The 400-row table costs p50 29–40 ms and p99 57–92 ms. Typing next to the list costs p50 15 ms, and scrolling over both at 50 KiB costs a p50 of 12.6 ms per frame.
   - Reparsing is no longer the cause (≤ 300 bytes per keystroke). GPUI Kit renders a whole list or table as one element.
   - Tables re-measure every cell on each edit, because the column-width cache belongs to each `Table` value and `Table::new` starts empty.
   - Lists can be fixed without a patch, by rendering chunks of items as separate list rows. `FlowPosition::first/last` exist for this. Tables need a decision (see 5b).
2. **Some single frames run long.**
   - Jumping to the end of a long file: 34–44 ms.
   - Undoing a 20 KiB paste at 50 KiB and above: 32–47 ms.
   - Both frames lay out many unmeasured blocks at once.
3. **markdown 1.0.0 is not always local.** Across 90,000 adversarial random edits (Markdown punctuation inserted anywhere), 0.156% of immediate results differed from a whole parse until the background pass that runs after 300 ms of idle reconciled them. Ordinary typing never differed in 3,000 keystrokes. markdown 1.0.0 also **panics** on some inputs ("Cannot push to non-parent"). Those panics are now caught: a window parse falls back to a provisional result, and a whole parse falls back to plain paragraphs. GPUI Kit's own reader uses the same parser and is exposed to the same panics.
4. **Open is not instant on Linux.** The main thread spends 117–153 ms before the first frame at every size. `VisualView` accounts for 4–6 ms (a 12 KiB prefix parse) and `EditorState` for 0.3 ms. The rest is window and GPU surface creation under lavapipe, which I couldn't measure on a Mac.
5. **Undo restores the exact scroll position only for edits made in the visual view.** It matches the restored caret against positions saved before each edit. For edits made in the source pane, it keeps the restored caret in view instead.
6. **Not run or unverified:**
   - macOS compile: `cargo check --target aarch64-apple-darwin` fails here because GPUI's Metal shaders need Apple tools.
   - Real IME, trackpad, ProMotion, VoiceOver, and Windows and Linux runtime.
   - The Windows **type-check passes** (`x86_64-pc-windows-gnu`).

## 2. Issues

| # | Status | Evidence |
|---|---|---|
| 1 Undo/redo | **Closed**: one history across views, and the selection is restored. Scroll is restored for visual edits; see failure 5. | `undo_and_redo_work_across_views`, `undo_restores_the_scroll_position`; fork patch `5f739b4` (+16 lines) |
| 2 Index sync | **Closed**: the view observes `document_revision`, and the range comes from its own edit's hint or a chunked prefix/suffix rope diff. Covers typing, undo, redo, the source pane and `set_value` reloads. | `mixed_edits_keep_the_index_in_sync` (250 steps of all sources against a whole parse, caret valid at every step), `diff_recovers_the_edited_range` |
| 3 Benchmark | **Closed**: `make bench` runs 4 sizes and the structural cases; for each case it reports the worst input with its reparsed bytes and path. | `examples/bench.rs`, `scripts/bench.sh`, `target/bench/*.jsonl` |
| 4 Unclosed fence | **Closed**: typing in a fence parses 0 bytes, and opening or closing one parses at most 2 KiB with the rest settled in the background. | `unclosed_fence_typing_is_a_fast_path`; bench: fence typing p99 ≤ 4 ms |
| 5 Large containers | **Reparse closed** (list-item and table-row paths, ≤ 300 bytes). **Render cost open**: failure 1. | `fast_paths_are_taken_and_match`; bench rows "large list" and "large table" |
| 9 Open | **Closed for parsing**: a 12 KiB prefix (4–6 ms) shows at once and the rest parses in the background with revision checks, so there's no blank first frame. | `open_parses_a_prefix_then_the_rest_in_the_background`; bench "Open" lines |
| 10 Formatting/toolchain | **Closed**: formatting is its own commit (`2a960dd`) and `rust-toolchain.toml` pins 1.97.0. | `cargo fmt --check` |
| 13 Shell split | **Closed**: the sidebar, tab header and status bar are cached views, and `Enso` notifies only when a document first becomes unsaved. | `typing_does_not_rerender_the_shell_parts`. Profile, 150 keystrokes: total p50 6.87 → **1.37 ms**, p95 10.46 → **2.18 ms**, p99 12.40 → **8.39 ms**; parts rendered 154 → 3 times |
| Parser decision | Prototyped; recommendation in section 5 | `gate-a/parser-1a`: `gk_tree.rs` and its tests |
| Perf tooling | **Closed**: `ENSO_PERF=1` overlay, `ENSO_PERF_LOG` JSON for the editor and the shell, percentiles from per-sample totals. Off by default (a `bool` check). | `src/visual/perf.rs`, `src/shell_perf.rs` |

Two scroll bugs turned up and are fixed:

- **`ListState::splice` scroll jump:** splicing an item resets the scroll anchor to that item's top. Every keystroke in a block taller than the viewport therefore jumped the view.
- **`scroll_to_reveal_item` falling short:** it lands short of far blocks that haven't been measured yet, and can't reach a caret deep inside a tall block.

Test: `caret_deep_in_a_tall_block_is_revealed_and_typing_keeps_the_view`.

## 3. Benchmark (CPU-only, Linux/Xvfb/lavapipe)

Work p50 / p99 in ms, from real per-sample totals. The frame wait (p50 ≈ 7–9 ms, uniform) is reported apart and not included.

| Case | 1k | 50k | 200k | 255k | Worst parsed bytes |
|---|---|---|---|---|---|
| type: start | 1.7 / 4.5 | 1.8 / 2.7 | 1.7 / 2.5 | 1.8 / 3.6 | 397 |
| type: middle | 2.0 / 3.0 | 13.6 / 54.6 | 15.5 / 55.3 | 14.3 / 61.4 | 395 |
| type: end | 1.8 / 3.1 | 26.2 / 65.6 | 2.4 / 7.0 | 2.7 / 8.3 | 699 |
| fence: open near top | 3.5 / 10.2 | 6.4 / 10.2 | 8.2 / 12.2 | 6.0 / 11.3 | 1,478 |
| fence: type inside unclosed | 1.2 / 4.0 | 1.3 / 4.0 | 1.4 / 3.2 | 1.2 / 3.1 | 0 |
| fence: close | 1.3 / 4.5 | 1.3 / 1.5 | 1.3 / 1.6 | 1.5 / 1.6 | 1,046 |
| setext underline | 1.9 / 12.9 | 2.3 / 7.5 | 2.5 / 12.6 | 2.2 / 13.1 | 0 |
| HTML comment opener | 1.9 / 3.2 | 5.4 / 9.1 | 5.6 / 10.1 | 5.3 / 11.9 | 548 |
| link reference definition | 1.5 / 3.9 | 2.0 / 6.8 | 2.6 / 8.2 | 2.1 / 6.3 | 1,322 |
| large list: type in item | — | 14.8 / 44.7 | 14.8 / 51.7 | 13.4 / 40.2 | 285 |
| large table: type in cell | — | 30.2 / 57.1 | 40.5 / 68.4 | 29.3 / 92.3 | 195 |
| paste 20 KiB (one input) | 3.2 | 26.5 | 10.4 | 6.0 | 0 |
| undo paste (one input) | 4.2 | 32.5 | 44.4 | 47.0 | 1,027 |
| scroll, per frame | 0.8 / 1.4 | 12.6 / 19.2 | 2.0 / 15.0 | 1.7 / 4.9 | – |
| jump to end, one frame | 1.5 | 34.0 | 43.7 | 40.5 | – |

The "type: middle" rows at 50k and above and "type: end" at 50k land next to the large list or table: failure 1.

These parts of the suite can't be measured yet:

- switching views, switching tabs, and dragging the sidebar, because the visual view isn't in the app until Gate B;
- typewriter and focus modes, which arrive in Gate C.

The shell profile in issue 13 covers typing in the app.

## 4. Screenshots

These were captured on Linux/Xvfb and sent with the report:

- the shell before and after the split, which look identical;
- the visual editor in both themes with the perf overlay.

## 5. Decisions for you

**a. Parser path: I recommend GPUI Kit's own parse.**

It needs a read-only patch of +37 lines (`gpui-base-fork-1a`): public `SourceSegment` getters, `source_mapping()` and `ImageNode::span()`. It matches Read mode by construction. On the six known differences:

- it fixes reference links, which become real links;
- it fixes inline HTML and math;
- footnotes and HTML blocks render as GPUI Kit renders them, but their synthesized text has no source mapping, so they would be atoms edited in source;
- nested task lists come out identical on both paths.

Other measurements:

- Parse speed is the same (49 KB: 35.3 ms own tree, 36.2 ms GPUI Kit).
- The builder is ~130 lines, against ~360 lines of own tree plus ~200 of alignment.
- Its mapping round-trips at every grapheme boundary in the sample.
- One quirk: GPUI Kit flags a collapsed soft break as "linear".

Your call: should I pin `ba13470` and move the index onto it in Gate B?

**b. Large tables.** Options:

- **(i)** A non-read-only patch: a stable column-width cache across edits, or row replacement on an existing `Table`.
- **(ii)** Render tables in chunks, accepting columns that may not line up between chunks.
- **(iii)** Leave tables as they are and make large lists cheap only.

I'd choose (i) with lists chunked on our side. It needs your approval because it grows the patch beyond read-only.

**c. Where the fork lives.** It is currently branches of `viai957/enso`, and CI gets a token step in case the repository is private. Move it to a dedicated repository? `UPSTREAM-REQUEST.md` on that branch is ready to file with GPUI Kit; I can't file it from here.

**d. Design reference (issue 15).** Please add Dinkus screenshots to `docs/design-reference/`. Placeholder tokens stay until then; the page uses a 680 px column and stock theme colors.
