# macOS checks for Gate A

These are the checks I could not run. I worked on Linux in a container, under Xvfb with software rendering, and had no IME, trackpad, ProMotion display or VoiceOver. Each step says what to do, what should happen, and what to send back. Work from the branch `gate-a/foundations`.

Setup, once:

```sh
git fetch origin gate-a/foundations && git checkout gate-a/foundations
cargo build --release --example visual_spike --example bench
```

`rust-toolchain.toml` pins Rust 1.97.0, and rustup installs it on the first build. The first build also fetches the patched `gpui-base` from this repository's `gpui-base-fork` branch. If that fetch fails because the repository is private, see step 7.

## 1. Benchmark (`make bench`)

**Do:** Run `make bench` on the Apple silicon Mac, plugged in. Use the built-in display first. If you have an external 60 Hz display, run it again there.

**Expect:** A table for each of four sizes (1k, 50k, 200k, 255k). In every row except "large table", work p99 should be under 8.33 ms. The large-table row is expected to be over budget; it is a known issue.

**Send back:** The whole output. Also send `target/bench/*.jsonl` and your Mac model and display refresh rate. The numbers are CPU-side (render to end of paint). Note anything you see stutter while the benchmark window types and scrolls by itself.

## 2. Real IME: Devanagari, Japanese, Chinese

**Do:** Open a note with `ENSO_SPIKE_THEME=light cargo run --release --example visual_spike -- "Get Started.md"`, click into a paragraph, and for each input source:

- **Devanagari (Hindi, INSCRIPT or Phonetic):** Type `नमस्ते दुनिया।`. Then backspace through it one character at a time.
- **Japanese (Kotoeri, Romaji):** Type `nihongo`. While the text is still underlined, press Space to see candidates, pick one, and press Return.
- **Chinese (Pinyin, Simplified):** Type `zhongwen`, pick 中文 from the candidate window, and confirm.

**Expect:**

- The candidate window sits just below the text being composed, not in a corner of the window or screen.
- The composing text shows highlighted in place and is replaced by the committed text.
- After committing, the caret sits right after the new text.
- No stray Latin letters remain.
- Backspace removes one grapheme at a time. For Devanagari conjuncts, one visible cluster per press is expected.

**Send back:** Pass or fail for each language. On a failure, send a screen recording with QuickTime (File, New Screen Recording) plus the exact keys pressed.

## 3. Click accuracy and caret

**Do:** In the same window, click between characters in a heading, a paragraph, a list item, a table cell and a code block. Include clicks between emoji (`👍🏽`), Devanagari and CJK. Then use the arrow keys, including Up and Down across a heading and a paragraph.

**Expect:** The caret lands at the gap nearest the click, never inside a character. Up and Down keep roughly the same horizontal position across blocks of different sizes.

**Send back:** Any spot where the caret lands wrong, with a screenshot.

## 4. Trackpad scrolling and ProMotion

**Do:** On a 120 Hz (ProMotion) display, run `ENSO_PERF=1 cargo run --release --example visual_spike -- target/bench/200k.md`. Step 1 writes that file (the benchmark corpus). Scroll from top to bottom with two-finger swipes, slowly and then quickly. Then type a paragraph.

**Expect:** Scrolling stays smooth with no visible hitches. In the overlay at the top right, the "over 8.3 ms" count stays at or near 0 while scrolling and typing in prose.

**Known issue:** frames that show the 400-item list or the 400-row table in the middle of the file cost 13–40 ms on Linux, so expect hitches there. Gate A reports this; please measure it.

**Send back:** A screenshot of the overlay after scrolling and after typing, and whether anything looked uneven.

## 5. Window resize and sidebar drag (shell)

**Do:** Run `cargo run --release -- <a folder of notes>` (the main app), open a note, and press ⌘E for Edit. Drag the window's corner continuously. Then drag the sidebar's edge. Then type in the editor.

**Expect:** Text reflows while you drag, without tearing or a blank frame. The sidebar edge follows the pointer. The sidebar, tabs and status bar look exactly as they did before this branch; they are now drawn as cached views. The word count updates within about a second of pausing.

**Send back:** Anything that lags the pointer, flickers or looks different from `main`. A before and after screenshot pair helps.

## 6. VoiceOver

**Do:** Turn VoiceOver on (⌘F5) and move through the main app's sidebar, tabs and buttons with VO-arrow keys. Then move into the visual editor window.

**Expect:** The shell's buttons and rows read their labels, as on `main`. The visual editor surface has no accessibility tree yet (planned for Gate C). VoiceOver will not read its text, and that is a known gap, not a regression.

**Send back:** Whether the shell still reads as it does on `main`.

## 7. If the build cannot fetch `gpui-base`

`Cargo.toml` pins `gpui-base` to a commit on this repository's `gpui-base-fork` branch. If the repository is private and Cargo cannot fetch it, run:

```sh
CARGO_NET_GIT_FETCH_WITH_CLI=true cargo build --release
```

That makes Cargo use your git credentials. The same applies to CI.

**Send back:** Whether you needed this.
