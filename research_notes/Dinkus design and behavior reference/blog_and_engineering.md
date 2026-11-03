# Dinkus: blog, release notes and the developer's own writing (as of 2026-11-03)

How these notes were gathered: I fetched dinkus.textualize.io with curl on 2026-11-03 (home, /blog/, /feed.xml, both posts, /about/, /pricing/, /support/, /privacy/, /will-mcgugan/, sitemap.xml). I read the Mac App Store listing with WebFetch. I checked Will McGugan's personal blog and its feed, the Ishmael site and manual, Python Bytes episode #494, and web search results for his X posts. X itself returned HTTP 402, so I could not read X post bodies directly. Tweet text comes from search-result titles, and tweet dates come from the snowflake IDs.
Labels: **[explicit]** means the source says it. **[inferred]** means my own reasoning.

## 1. Blog posts (complete list) and what each one says

### Takeaway
The Dinkus blog has only **two posts** (`/blog/`, the Atom feed and sitemap.xml all agree). One is a launch stub. The other is "An HDR caret for a Markdown editor" (2026-08-26), the developer's only published engineering write-up about Dinkus. There is no other engineering writing about the rendering engine, round-tripping, performance and so on.

### Cited Findings
- Post list [explicit]: (1) **"Hello, Dinkus"**, 8 Aug 2026, https://dinkus.textualize.io/blog/hello-dinkus/. (2) **"An HDR caret for a Markdown editor"**, 26 Aug 2026, https://dinkus.textualize.io/blog/hdr-caret/. Feed `<updated>` is 2026-08-26 and the author is Will McGugan. — [Blog index](https://dinkus.textualize.io/blog/), [Atom feed](https://dinkus.textualize.io/feed.xml), [sitemap](https://dinkus.textualize.io/sitemap.xml)
- The blog's tagline is "News and notes from the making of Dinkus." [explicit] — [Blog](https://dinkus.textualize.io/blog/)
- "Hello, Dinkus" (8 Aug 2026) has the summary "We are live" and a body of only "The site is live." It contains no technical content [explicit]. — [post](https://dinkus.textualize.io/blog/hello-dinkus/)
- "An HDR caret…" (26 Aug 2026): summary is "Dinkus can render its caret in HDR, so it glows brighter than #FFFFFF on a capable display" [explicit]. Its details are in section 3 below. — [post](https://dinkus.textualize.io/blog/hdr-caret/)
- No changelog, release-notes or roadmap page exists on the site. `/changelog/`, `/releases/` and `/release-notes/` all return 404, and sitemap.xml lists only home, about, blog (two posts), pricing, privacy, support and /will-mcgugan/ [explicit, checked 2026-11-03]. — [sitemap](https://dinkus.textualize.io/sitemap.xml)

### Inferences
- The blog was last updated on 2026-08-26, which is before releases 1.5 to 1.11. Release history therefore lives only on the App Store and X (section 4).

### Gaps
- I found no posts on the editing engine, the Markdown parser or serializer, byte-for-byte round-tripping, table/image/front-matter internals, performance, frame rate, ProMotion or typography choices. The site does not cover these topics.

## 2. Technology stack; smoothness, latency, frame rate, ProMotion, HDR

### Takeaway
The developer has not published Dinkus's stack. Strong indirect evidence points to a **web-tech editor (HTML/CSS) inside a macOS WebView, probably Tauri 2 / WKWebView**, like his sibling app Ishmael. He calls it a "native app" in marketing. He has said nothing about latency, fps or 120 Hz.

### Cited Findings
- Explains the HDR caret by saying "HDR support in CSS is currently spotty". He presents the clipped HDR image as the workaround, and says it "could be added to any editor" [explicit wording]. This implies the caret is drawn with CSS/DOM [inferred]. — [HDR caret post](https://dinkus.textualize.io/blog/hdr-caret/)
- Marketing copy says "Visually author Markdown with a native app" (X post 2091528888471941223, dated 2026-08-23 by snowflake ID). The text comes from the search-result title because X was not readable [explicit, secondhand]. — [X](https://x.com/willmcgugan/status/2091528888471941223)
- The App Store description calls it "a native Markdown editor". The binary is 11.3 MB, needs macOS 12.0+ and an Apple M1 or later, and the category is Developer Tools [explicit]. — [App Store](https://apps.apple.com/app/dinkus-markdown-studio/id6799480413)
- The sibling app Ishmael (same author, and the source of Dinkus's typewriter and focus modes) ships an HTML manual. Its inline JS has comments such as "Esc closes the window (Tauri provides window.close()…", "The DOM `window.close()` API is a no-op on Tauri 2" and `window.__TAURI_INTERNALS__`, plus WebKit workarounds [explicit, in Ishmael's code]. Ishmael's site also offers "Try Ishmael in your browser… a preview of Ishmael's editor" [explicit]. Together these show that Ishmael is a Tauri 2 app with a web editor. — [Ishmael manual](https://ishmael.textualize.io/manual/), [Ishmael](https://ishmael.textualize.io/)
- The Dinkus About page says Ishmael's editing features "felt bigger than the one app" and that typewriter and focus modes "come straight from Ishmael" [explicit]. Shared code with Ishmael, and so Tauri/WebKit, is a reasonable guess but **unconfirmed** [inferred]. — [About](https://dinkus.textualize.io/about/)
- The website uses Inter Variable (`/assets/fonts/InterVariable.woff2`). The editor screenshots show a serif display/body face, for example "Call me Dinkus." in the caret pane and "Litany Against Fear" in the HDR video [explicit for the site font; the in-app font face is unidentified]. — [home](https://dinkus.textualize.io/), screenshots below
- HDR is discussed only for the caret (section 3). Nothing has been published on smoothness, input latency, frame rate or ProMotion/120 Hz [checked: blog, feed, App Store, About, X search results, Python Bytes].

### Inferences
- If Dinkus is Tauri/WKWebView, its scroll and caret smoothness comes from WebKit's compositor, which runs at 120 Hz on ProMotion Macs in recent Safari/WebKit. This is general knowledge, not a claim by the developer [inferred].

### Gaps
- I could not confirm the framework, language, Markdown library (for example markdown-it, ProseMirror, CodeMirror or Milkdown) or text engine. Getting the app bundle (Frameworks, Info.plist) would settle it, but the App Store binary was not downloadable here.

## 3. The HDR caret: how it works and how it looks

### Takeaway
"HDR Brightness" is an opt-in toggle in Settings → Caret. On an HDR (EDR) display the caret draws **brighter than #FFFFFF**. It is drawn by clipping a **tiny HDR image** to the caret's rectangle, because CSS HDR colour support is unreliable. The rest of the caret is configurable: colour swatches, width in pixels, a blink on/off toggle, blink rate, and a 12-step blink pattern.

### Cited Findings
- Caret settings are "color, thickness, blink, and even the blink pattern" [explicit]. — [HDR caret post](https://dinkus.textualize.io/blog/hdr-caret/)
- With "HDR Brightness" enabled on an HDR-capable display, "the caret is displayed brighter than pure white". He calls it novel ("I haven't seen any other app do this") and asks readers for prior art [explicit]. — [post](https://dinkus.textualize.io/blog/hdr-caret/)
- Mechanism: "Dinkus renders the caret by clipping a tiny HDR image to the shape of the caret". He says "the implementation is trivial" [explicit]. — [post](https://dinkus.textualize.io/blog/hdr-caret/)
- A later update in the post says the video does not look as bright as the caret does on the desktop, possibly because of encoding loss [explicit]. — [post](https://dinkus.textualize.io/blog/hdr-caret/)
- Demo video: 1460×1884, 60 fps, 25 s. It is offered in AV1, HEVC and VP9. I checked the HEVC file with ffprobe: 10-bit yuv420p10le, BT.2020 primaries, **SMPTE 2084 (PQ)** transfer [explicit metadata]. The caption reads "HDR caret (top window), standard SDR caret (bottom window) — best viewed on an HDR display in Safari or Chrome." — [post](https://dinkus.textualize.io/blog/hdr-caret/); file `screenshots/blog-hdr-caret.hevc.mp4`
- What the Settings → Caret screenshot (`screenshots/blog-caret-dialog.webp`/`.png`, 1424×1178, dark theme) shows [explicit from the image]:
  - Settings sidebar: General, Caret, Auto-correct, Markdown, Language, Tips.
  - A live preview strip at the top reads "Call me Dinkus." in a serif face, with an orange/peach caret.
  - **Color**: 16 pastel swatches in 2 rows (lavender, pale blue, grey-blue, cream, blush, pink, rose, salmon; peach (selected), sand, green, teal, aqua, sky, blue, periwinkle), with the note "Theme colors adapt when you switch themes."
  - **Size**: a slider set to "2.5 pixels" ("Width of the caret").
  - **Blink**: a toggle that is on ("Blink the caret when idle. Off keeps the caret solid.").
  - **Blink rate**: a slider at "0.8s".
  - **Blink pattern**: 12 squares, the first 8 filled ("filled squares are time the caret is visible"), so the caret is visible for about 2/3 of the cycle.
  - The HDR toggle sits below the visible area (the pane scrolls).
  - Buttons are Cancel and Save (lavender accent).
  - Chrome: a dark navy-charcoal (#1d1f2b-ish) modal with rounded corners and a thin border. — [post image](https://dinkus.textualize.io/assets/img/caret-dialog.webp)
- What the HDR video frame (`screenshots/blog-hdr-caret-frame.png`, 2 s, tone-mapped) shows: two stacked dark-theme windows with a tab "Untitled ●", the title "Untitled — Edited", traffic lights, right-side titlebar icons (eye, briefcase, brackets, pin, search) and a bottom formatting toolbar (¶, B, I, S, code, link, \*, image, list, table, `</>`, √ maths, etc.). The heading is "Litany Against Fear" in a large light-grey serif, with a dashed horizontal guide line under the caret line, which is the typewriter insertion guide [inferred]. In the top window the caret is a thin warm-cream/peach bar about the height of the heading line [explicit from the image; the HDR glow is lost when tone-mapped].

### Inferences
- In a WebKit context, the "tiny HDR image" is likely a 1×1 or small PNG/AVIF with a PQ or HLG colour profile (or an HDR video frame), used as the caret element's background and sized to the caret rectangle. WebKit on macOS displays HDR images with EDR headroom, while CSS colours above 1.0 (`color(rec2100-pq …)`) are not widely supported. The "clipping" wording suggests `background-image` + `background-size`/`clip` or a `mask`. This is my reconstruction, not stated.
- A native (Metal/CoreAnimation) app such as enso could instead use a CAMetalLayer with `wantsExtendedDynamicRangeContent` and EDR values above 1.0 for the caret colour, or an HDR-tagged image layer [inferred].
- The default caret appears to be a peach/orange 2.5 px bar blinking at 0.8 s with an 8/12 duty cycle. This is based on the selected values in the screenshot and may not be the shipped defaults [inferred].

### Gaps
- The HDR brightness level (nits or EDR multiplier), whether HDR uses the selected colour or white, and whether the caret animates (fades versus a hard blink) are not stated.

## 4. Release notes / version history

### Takeaway
The only release notes are in the Mac App Store version history: 12 versions in about 2.5 months (1.0 on Aug 19 through 1.11.0 around Nov 1, 2026). Social posts highlight 1.8.0. There is no public roadmap.

### Cited Findings (all from [App Store](https://apps.apple.com/app/dinkus-markdown-studio/id6799480413), read 2026-11-03; year inferred to be 2026)
- **1.0** (Aug 19): initial release.
- **1.1.0** (Aug 20): new UI design, folder view, equations editor.
- **1.2.0** (Aug 21): minor fixes ahead of launch.
- **1.3.0** (Aug 28): new welcome screen, restores open files when a folder is reopened, Markdown extensions beyond .md/.markdown, fixed Asian-language support (likely IME/CJK [inferred]).
- **1.4.0** (Aug 29): fixes and visual enhancements.
- **1.5.0** (Sep 6): collapsible headings, a "detail slider", tap-to-select, fixes.
- **1.6.0** (Sep 16): Mermaid diagrams, new welcome view.
- **1.7.0** (Sep 20): Pelican-style front matter, checklists, a "styles indicator" in the footer.
- **1.8.0** (Sep 24): Read mode, full-text search, quick-find (⌘P), tags, vertical splits (shift-click the split icon), outline view. An X post on 2026-09-24 says "New in Dinkus 1.8.0 ⁂ Full-text search ⁂ Document outline view ⁂ Hashtags" — [X](https://x.com/willmcgugan/status/2103121091979792837)
- **1.9.0** (Oct 1): draggable dividers for the sidebar and panes, printing, export to SVG and Word.
- **1.10.0** (Oct 9): new folder-view icon set, better grammar-checker support.
- **1.11.0** ("2d ago", so about 2026-11-01 [inferred]): Dinkus Light and Dinkus Dark themes, and export to a single self-contained HTML file with images.
- Listing details [explicit]: $9.99, no IAP, 11.3 MB, English only, Family Sharing, privacy label "Data Not Collected". The description adds features that are not on the website: inline and block equations, Mermaid, footnotes, raw HTML blocks, a link dialog (local headings, documents, URLs), drag-and-drop images with size and alignment, a folder pane, collapsible headings and the detail slider.
- Developer replies to reviews [explicit, paraphrased by WebFetch]: one user asked for system dark mode, and he pointed to the new Dinkus Dark theme. He also says a future version will remember the typewriter-mode setting (a reviewer disliked typewriter mode being on by default). Another review mentions a Grammarly compatibility issue. — [App Store](https://apps.apple.com/app/dinkus-markdown-studio/id6799480413)
- Python Bytes #494 (2026-09-01): Michael Kennedy says typewriter mode is on by default for every file and has no hotkey to turn it off (you use the on-screen button). He praises the tables, images and code highlighting [explicit]. — [Python Bytes #494](https://pythonbytes.fm/episodes/show/494/python-wrapture)

### Inferences
- The release pace is roughly weekly. Development appears to follow reviews and user requests (dark theme, typewriter default, grammar checkers).

### Gaps
- No public changelog with fix-level detail. No published roadmap (only the review reply about remembering typewriter mode).

## 5. Product claims on design topics (site copy, not engineering posts)

### Takeaway
These are the developer's own short design statements from the site. They cover WYSIWYG editing without a preview pane, a source view, standard Markdown on disk, the image dialog, the table editor, front matter, typewriter and focus modes. **Byte-preserving round-trips are never claimed.** The copy says only that the file is "saved as standard Markdown", which implies re-serialisation.

### Cited Findings (all from the [home page](https://dinkus.textualize.io/) unless noted; all [explicit])
- **Visual editing**: "without a preview pane or compilation step". Supports "inline styles, headers, code blocks, images, tables, footnotes, and more". "plain .md on disk · no account · works offline".
- **Source view / round-trip**: switch to source "with a click" or edit "side-by-side". "Return to visual mode… continue editing where you left off". "However you choose to edit, the document is saved as standard Markdown and works everywhere." The About page says "no risk of broken syntax" — [About](https://dinkus.textualize.io/about/). The X post says "Raw markdown is a keypress away" — [X](https://x.com/willmcgugan/status/2091528888471941223).
- **Images**: an image dialog for local files or URLs. It "will generate standard Markdown markup from images wherever possible, but can also output an `<img>`", for example to float images right. Remote images are the only network requests Dinkus makes — [Privacy](https://dinkus.textualize.io/privacy/).
- **Tables**: add or remove rows and columns, edit cells visually (inline syntax and images in cells). In source mode Dinkus "will even format and extend the column by adding spaces to keep your table source nice and tidy" (pipe-table auto-alignment).
- **Front matter**: "can read, write, and edit all the formats in common use today", syntax-highlighted. App Store 1.7.0 adds Pelican-style front matter.
- **Code blocks**: syntax highlighting for "common languages—and many uncommon ones".
- **Typewriter mode**: keeps the caret "anchored on a single horizontal line by moving the document vertically". You can "drag the insertion guide away from the center", to keep more context above the caret and a few lines below. Ishmael's manual describes the origin: the cursor stays in "the same narrow strip in the center", with "half above your edit point, half below", toggled with ⌘T in Ishmael — [Ishmael manual](https://ishmael.textualize.io/manual/).
- **Focus modes**: "highlight the sentence or paragraph you are working on and dim the rest of the document". Ishmael's focus mode "highlights the sentence you are editing and the paragraph it resides in" — [Ishmael manual](https://ishmael.textualize.io/manual/).
- **Spelling and grammar**: uses the macOS system service — [Privacy](https://dinkus.textualize.io/privacy/).
- **Platform and business**: Apple Silicon, macOS 12 Monterey+, $9.99 one-time, sold only on the Mac App Store, no analytics, no update checks — [Pricing](https://dinkus.textualize.io/pricing/), [Privacy](https://dinkus.textualize.io/privacy/) (policy dated 11 Aug 2026).

### Inferences
- "Format and extend the column by adding spaces" plus "saved as standard Markdown" suggest that Dinkus normalises some Markdown on save (at least tables). So it probably does **not** preserve original bytes exactly in every case. This is not stated either way [inferred].

### Gaps
- Round-trip fidelity (whether the original file bytes, list markers, emphasis characters, line wrapping and so on are preserved) is undocumented. Undo model, large-file performance and table edge cases are also undocumented.

## 6. What Will McGugan has said elsewhere

### Takeaway
Outside the Dinkus site, he talks about Dinkus only in short promotional X posts. His personal blog, the Textualize GitHub and the podcasts I checked contain no technical writing about Dinkus.

### Cited Findings
- X posts (text from search-result titles, dates from snowflake IDs):
  - 2026-08-23: launch, "native app", "no preview pane necessary", "typewriter mode and paragraph or sentence focus" — [X](https://x.com/willmcgugan/status/2091528888471941223).
  - 2026-08-30: "no.1 paid app on Mac App Store", discounted for the rest of the month — [X](https://x.com/willmcgugan/status/2094053774893662302).
  - 2026-09-24: 1.8.0 announcement — [X](https://x.com/willmcgugan/status/2103121091979792837).
- Personal blog: https://willmcgugan.github.io/ (linked from /will-mcgugan/). The latest posts are 2026-10-15 ("The Living Mountain Chapter 2"), 2026-06-23 ("Stop, don't Slop"), 2026-02-11 ("AI_POLICY.md") and earlier Toad posts. Its feed contains **0 mentions of Dinkus or Ishmael** [checked] — [blog](https://willmcgugan.github.io/), [feed](https://willmcgugan.github.io/feed.xml)
- Background: his Dec 2025 post says his terminal startup (Textualize) "wrapped up mid-2025 when the funding ran dry". Dinkus and Ishmael are later solo Mac apps under the Textualize name [explicit] — [blog](https://willmcgugan.github.io/)
- /will-mcgugan/ lists X (@willmcgugan), his blog, and Rich, Textual and Toad. It lists no Mastodon or Bluesky [explicit] — [page](https://dinkus.textualize.io/will-mcgugan/)
- Community support is on Discord (https://discord.gg/rx3bxd8ZRw), with email will@textualize.io — [Support](https://dinkus.textualize.io/support/)
- Podcast: Python Bytes #494 (2026-09-01) covered Dinkus as a third-party mention, not an interview. It says nothing about the stack or HDR — [Python Bytes](https://pythonbytes.fm/episodes/show/494/python-wrapture)

### Gaps
- I could not read X directly (HTTP 402), so replies and threads (which may contain technical remarks) went unchecked. I did not search Mastodon or Bluesky beyond general web search, which found nothing. I found no interviews. Dinkus has no GitHub repo (closed source).

## Screenshots saved (prefix blog-) in `screenshots/`
- `blog-caret-dialog.webp` / `.png`: Settings → Caret pane (described in section 3).
- `blog-hdr-caret.hevc.mp4`: the HDR demo, 10-bit PQ BT.2020, 60 fps, 25 s. `blog-hdr-caret-frame.png` is an SDR still from it.
- From the home page (same files as the `product-`/`app-` copies other researchers saved): `blog-wysiwyg-{light,dark}.webp`, `blog-source-*`, `blog-tables-*`, `blog-images-*`, `blog-code-*`, `blog-frontmatter-*`, `blog-typewriter-light.mp4`, `blog-focus-light.mp4`, `blog-dinkus.mp4` (hero video), `blog-og.png`, `blog-dinkus.webp` (logo), `blog-macbook-frame.webp`. I did not describe these here because they are product-page assets, not blog assets.
