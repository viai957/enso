# Dinkus visual design: app measurements and website design tokens (as of 3 Nov 2026)

Scope: the Dinkus macOS app (Will McGugan / Textualize, App Store id6799480413) as seen in every public image I could get, plus the marketing website's CSS tokens, which are a separate thing. Each value is labelled **[APP]** or **[SITE]**, and also as *measured from <file>*, *stated at <URL>* or *inferred*.

Files are in `/home/user/enso/research_notes/Dinkus design and behavior reference/screenshots/`. Measurement scripts are in `/tmp/claude-0/scripts/` (runs.py: colour runs along a row or column; bands.py: text-line bands; dens.py: per-row ink density; samp.py: background colour and ink colour inside a box). Colours come from lossy WebP/PNG/H.264 sources, so expect ±2–6 per channel. "Ink" colours are the mean of the most extreme 3% of pixels, which pulls them slightly toward the background because of anti-aliasing.

**Scale assumption (inferred).** The website screenshots are 2624×2024 with a 2400 px window and a baked-in shadow. I treat them as 2× Retina captures of a 1200 pt-wide window. The App Store PNGs (2880×1800) are rescaled marketing composites with the window placed on a wallpaper, so their absolute px→pt scale is unknown. I use them only for ratios.

---

## 1. Which images show the actual app, and where they came from

### Takeaway
All 10 App Store screenshots, the 12 website feature screenshots and the 5 website videos show the real app. The OG image, the logo and the MacBook frame are marketing art. Note that the site deliberately shows the dark screenshot when the site is in light mode, and the light one when it is dark.

### Cited Findings
- App Store listing "Dinkus Markdown Studio" exists. It has 10 Mac screenshots (Apple serves them as screenshot4–9 and 12–15), which I downloaded at 2880×1800 as `app-appstore-screenshot{4..9,12..15}.png`. Base URLs have the form `https://is1-ssl.mzstatic.com/image/thumb/PurpleSource2xx/v4/<hash>/screenshotN.png/2880x1800bb.png`; the full list was extracted from the page HTML. — [App Store](https://apps.apple.com/app/dinkus-markdown-studio/id6799480413)
  - Page order: 5, 4, 13, 12, 6, 7, 14, 15, 9, 8.
  - Content, from viewing each file:
    - 4: light cream theme, outline sidebar, table with image cells.
    - 5: the same view in a dark theme.
    - 6 and 7: Image dialog (dark and light).
    - 8 and 9: quick-open / full-text search (light and dark).
    - 12 and 13: split view, WYSIWYG on the left and source on the right, with folder sidebar (light and dark).
    - 14 and 15: serif novel text in "Paragraph focus" (dark and light).
- Website product images are downloaded with the `app-` prefix from `https://dinkus.textualize.io/assets/img/<name>`. — [dinkus.textualize.io](https://dinkus.textualize.io/)
  - Screenshots, 2624×2024 each: wysiwyg-{light,dark}.webp, source-{light,dark}.webp, images-{light,dark}.webp, tables-{light,dark}.webp, code-{light,dark}.webp, frontmatter-{light,dark}.webp.
  - Videos:
    - dinkus.mp4: 3324×2160, 140 s, a real desktop recording with the macOS menu bar.
    - typewriter-{light,dark}.mp4: 1792×1264.
    - focus-{light,dark}.mp4: 1792×1264.
  - The HTML maps `data-src-on-light="/assets/img/wysiwyg-dark.webp"` and `data-src-on-dark=".../wysiwyg-light.webp"`, so the light site shows dark app shots and the dark site shows light ones. — [dinkus.textualize.io](https://dinkus.textualize.io/)
- Marketing art, not app UI:
  - `app-og.png` (2400×1260): dark slate background, three pastel asterisks (lavender, peach, teal) forming a "dinkus", the wordmark "Dinkus" in a large geometric sans, "MARKDOWN STUDIO" and "# No account • Works offline" in a monospace. — [og.png](https://dinkus.textualize.io/assets/img/og.png)
  - `app-dinkus.webp` is the logo. `app-macbook-frame.webp` is the device bezel around the hero video. — [dinkus.textualize.io](https://dinkus.textualize.io/)
- Blog images:
  - "An HDR caret…" contains `caret-dialog.webp` (Settings → Caret, dark), saved as `app-blog-caret-dialog.webp`, and `hdr-caret.hevc.mp4`. — [blog/hdr-caret](https://dinkus.textualize.io/blog/hdr-caret/)
  - "Hello, Dinkus" has no images. — [blog/hello-dinkus](https://dinkus.textualize.io/blog/hello-dinkus/)
- Social-post images saved by another researcher (`third-*` files) show the app too. I did not record their source URLs; see the community notes. I sampled two of them:
  - `third-mastodon-2026-10-31-html-export.png`
  - `third-mastodon-2026-10-02-tree-icons.png`

### Inferences
- The App Store shots look newer than the website shots. They include the cream light theme, a Read/Edit segmented control, the outline pane and the search dialog, while the website shots have no Edit/Read control in the title bar. Release dates from the App Store version history: 1.0 on 19 Aug 2026, Read mode in 1.6 or 1.7 (Sep 2026), and the "Dinkus Light/Dark" themes in 1.11 on 31 Oct 2026. — [App Store](https://apps.apple.com/app/dinkus-markdown-studio/id6799480413)

### Gaps
- I found no press articles with original screenshots.
- App Store screenshot slots 1–3 and 10–11 are not served, which suggests they were replaced.

---

## 2. Layout: page, text column, margins, chrome sizes (APP)

### Takeaway
The app draws the document as a **bordered "page" sheet**, slightly different in colour from the area around it, centred in the editor pane. The sheet is about **70–76% of the editor pane's width**. Inside it, the text column has an inset of about 46 pt (sans shots) or about 26 pt (typewriter video) on each side. At the widths shown, lines run to about **85–110 characters**, which is wide. The width tracks the window, not a fixed measure.

### Cited Findings (all measured; [APP])
- **Website code-light / tables-light** (window 2400 px, no sidebar), measured from `app-code-light.webp` and `app-tables-light.webp`:
  - Window x 112–2511 and y 78–1875.
  - Title bar plus tab bar y 78–197: 120 px, about 60 pt at 2×.
  - 1 px divider, then a 31 px (about 15 pt) gap above the page.
  - Page sheet x 405–2215 (1811 px, **75.5% of window/editor width**). Its top edge is at y 233, with a 1 pt border line at x 403–404 and 2216–2217.
  - Text column x ≈498–2121 (≈1623 px ≈ 811 pt). Inset inside the sheet: 93 px left and 94 px right (≈46–47 pt).
  - Bottom formatting toolbar y 1796–1875: 80 px ≈ **40 pt**.
  - About 93 characters per body line, e.g. "Python properties work well with type checkers such Mypy and friends. This is to be expected,".
- **App Store screenshot4** (folder/outline sidebar visible), measured from `app-appstore-screenshot4.png`:
  - Window x ≈282–2598.
  - Sidebar x 283–740: **≈19.7% of window**.
  - Editor pane x 741–2598.
  - Page sheet x ≈1015–2320 (1305 px, **≈70% of the editor pane**, 56% of the window).
  - Text and table x 1092–2242. The inset is about 77 px each side, about 6% of the sheet.
  - About 102 characters per body line.
- **App Store screenshot15** (serif document), measured from `app-appstore-screenshot15.png`:
  - Editor pane x 693–2685.
  - Page x 977–2398 (**≈71% of the editor pane**).
  - Text x 1059–~2316. Inset ≈82 px.
- **Typewriter/focus video**, measured from `app-focus-light.mp4` (frame at 8 s) and `app-typewriter-light.mp4`. The frame is 1792×1264 and shows the full editor with no sidebar and no title bar visible:
  - Page x 228–1582: **75.6% of width**.
  - Text x ≈280–1530. The inset is ≈52 px, about 26 pt.
  - About 84 characters per serif line.
- **Website wysiwyg-light**, measured from `app-wysiwyg-light.webp`: the sidebar is about 447 px of 2400 (≈18.6%, ≈224 pt at 2×).
- **Typewriter guide.** A thin **dashed horizontal line** crosses the whole editor pane at the vertical centre: y ≈606/1218 px of the editor height in the typewriter video. Measured from `app-typewriter-light.mp4`; also visible in `app-wysiwyg-light.webp` and `app-dinkus.mp4`. The caret line stays on it while typing ("Typewriter mode keeps a fixed amount of context above and below your caret" — [App Store](https://apps.apple.com/app/dinkus-markdown-studio/id6799480413)).

### Inferences
- Margins appear to be a proportion of the pane: page ≈ 70–76% of pane width. The left/right gutters were 145 pt in the website shot and 114 pt in the video, so not a constant. No reading-measure cap is evident (85–110 characters per line).
  - For enso, a cap of about 65–75 characters would be a deliberate departure.
  - An App Store bullet mentions a "detail slider", which may control density or width, but I could not confirm what it does.

### Gaps
- I found no stated text width, font-size setting or margin setting. I also could not determine whether the page width is user-adjustable.

---

## 3. Typefaces, font sizes, line height, headings, paragraph spacing (APP)

### Takeaway
The default body face is a **neo-grotesque sans that looks like Inter**, at about 17 pt with a line height of about 1.45–1.5. A **text serif** is used in prose documents: the novel samples, the typewriter/focus videos and the Caret settings preview. Code is monospace. Headings use the same colour as body text. Measured scale relative to body: **H1 ≈ 2.1×, H2 ≈ 1.5×, H3 ≈ 1.25×**, with a hairline rule under H1 and H2. Paragraph gap is about 0.7 of a line.

### Cited Findings
- **Body sans** [APP], measured from `app-code-light.webp`:
  - Line pitch 50 px (339→389→439): **≈25 pt at 2×**.
  - Ink box from ascender to descender 33 px.
  - x-height 19 px (rows 346→365); ascender 26 px; descender 7 px.
- **Paragraph spacing** [APP], measured from `app-code-light.webp`: the last line of a paragraph at 439 and the next paragraph at 524 give 85 px vs a 50 px pitch, so **an extra 35 px (≈17.5 pt ≈ 0.7 line)**.
- **Body in screenshot4** [APP], measured from `app-appstore-screenshot4.png`: pitch ≈33.5 px, x-height 12 px. The pitch/x-height ratio is 2.8 here and 2.63 in the website shot.
- **Headings** [APP]:
  - Measured from `app-tables-light.webp`: "Tables" (top-level, with rule) has x-height 40 px vs body 19 px, so **≈2.1×**, and cap height ≈56 px. The rule sits ≈40 px below the baseline, 2 px (1 pt) thick, colour ≈#dcdde1.
  - Measured from `app-appstore-screenshot4.png`:
    - "Conservation" (with rule) has x-height 18 px vs body 12, so **≈1.5×**.
    - "Reproduction" (no rule, semibold) has x-height 15 px vs 12, so **≈1.25×**.
  - Measured from the `app-typewriter-light.mp4` frame: serif H1 "Typewriter mode" ink height 69 px vs body line ink 31–32 px, so ≈2.15×.
  - Measured from `app-appstore-screenshot15.png`: serif "Glitch" and "Chapter 1." are both underlined by a rule spanning the text column; the second heading is only slightly smaller (ink 44 vs 42 px).
- **Serif body** [APP], measured from the `app-focus-light.mp4` frame:
  - Line pitch 45.6 px (≈22.8 pt at 2×).
  - Paragraph gap: 76 px vs 45.6 px pitch, so an extra ≈30 px (≈0.66 line).
  - The same pattern appears in `app-appstore-screenshot15.png`: pitch ≈42 px, paragraph extra ≈28 px.
- **Code** [APP], measured from `app-code-light.webp`:
  - Line pitch 36 px (≈18 pt).
  - x-height 16 px vs body 19.
  - A 4-space indent spans 72 px, so 18 px per character ≈ 9 pt advance, which implies a ≈15 pt monospace if the advance is 0.6 em.
- **Typeface identification:**
  - Sans: shapes in a full-resolution crop of `app-code-light.webp` (flat-cut 't', straight-tailed 'y', two-storey 'a', tall x-height) match **Inter**; SF Pro is the other candidate. *Inferred.*
  - The website's own CSS says its palette mirrors "the editor app", but the CSS states fonts only for the site: Inter and JetBrains Mono. — [styles.css](https://dinkus.textualize.io/assets/styles.css?v=11f8355c11)
  - The serif is a bracketed text serif, unidentified (*inferred*). It appears in:
    - `app-appstore-screenshot14/15.png`
    - `app-typewriter-*.mp4`, `app-focus-*.mp4`
    - `app-blog-caret-dialog.webp` ("Call me Dinkus.")
    - `third-mastodon-2026-10-02-tree-icons.png`, `third-mastodon-2026-10-31-html-export.png`
  - The code monospace resembles JetBrains Mono (*inferred*; the site uses JetBrains Mono).
- **Tables** [APP], measured from `app-tables-light.webp`:
  - Rows are 76 px (≈38 pt) tall. With ≈33 px text ink, vertical cell padding is ≈10 pt.
  - 1 pt borders ≈#d1d3d7 (light).
  - Header row background ≈#e5e8f0, a little darker than the page (#eef0f4).
  - The selected column is tinted mauve (#dad2f0).
  - Small purple pill buttons (+ and arrow) float above or beside the selected column or row.

### Inferences
- Body ≈17–18 pt (x-height 19 px ÷ Inter's 0.546 ≈ 34.8 px ≈ 17.4 pt) and line-height ≈1.45 em (25/17.4). Serif body is probably 16–18 pt with line-height ≈1.35–1.45. These come from the 2× assumption and are not stated.
- The serif may be a per-document or per-theme font choice. App Store screenshot4 (sans) and screenshot15 (serif) use the same cream theme, so the font is likely a setting separate from the theme.
- The blog says "HDR support in CSS is currently spotty, so Dinkus renders the caret by clipping a tiny HDR image to the shape of the caret". That suggests the editor surface is rendered with web technology (HTML/CSS in a WebView). *Inferred.* — [blog/hdr-caret](https://dinkus.textualize.io/blog/hdr-caret/)

### Gaps
- I found no stated font names, sizes or line heights for the app, and no font picker was visible in any image.
- H4–H6 do not appear in any image.
- Heading levels are inferred from appearance and rules, not from visible Markdown.

---

## 4. Colours: light and dark (APP), and the Catppuccin question

### Takeaway
The website screenshots of the app use **Catppuccin Latte** for light and a **Mocha-like dark**: the sidebar and toolbar use mantle, the surrounding area uses base, and the page sheet is slightly lifted. The website CSS states that the site's two themes mirror "the editor app: Catppuccin Latte (light) and Mocha (dark)". The App Store screenshots also show a **cream light theme** (#fefcf0 page, black text, deep-purple accent). A late-October image shows a **neutral grey dark theme** (#121212/#181818/#1f1f1f). These match version 1.11's new "Dinkus Light" and "Dinkus Dark" themes (*inferred*).

### Cited Findings
- **Website CSS** [SITE]: "Two-theme model mirroring the editor app: Catppuccin Latte (light) and Mocha (dark)". The `--accent: var(--ctp-mauve)` token carries the comment "matches the app's brand". — [styles.css](https://dinkus.textualize.io/assets/styles.css?v=11f8355c11)
- **1.11 release notes** [APP] (31 Oct 2026): "Two new themes: Dinkus Light and Dinkus Dark". On 2 Nov 2026 the developer replied to a review asking for system dark mode: "Try the new Dinkus Dark theme. It's not exactly the same as the system theme, but it is close." — [App Store](https://apps.apple.com/app/dinkus-markdown-studio/id6799480413)
- **Caret settings** [APP]:
  - 16 caret colour swatches in pastel and accent hues, captioned "Theme colors adapt when you switch themes".
  - Size slider default 2.5 pixels.
  - Blink rate 0.8 s.
  - Blink pattern is a 12-cell grid.
  - Measured from `app-blog-caret-dialog.webp`; the post describes HDR caret brightness. — [blog/hdr-caret](https://dinkus.textualize.io/blog/hdr-caret/)

**Table A. Light, Catppuccin Latte-like.** Measured from `app-code-light.webp` and `app-tables-light.webp` [APP]. The last column gives the nearest Latte colour; the mapping is inferred.

| Role | Measured | Nearest Latte |
|---|---|---|
| Title bar / tab bar / editor surround | #e6e8ec | mantle #e6e9ef |
| Page sheet | #eef0f4 | base #eff1f5 |
| Bottom toolbar | #dce0e6 | crust #dce0e8 |
| Body text / headings | #494b59 / #484a5c | text #4c4f69 |
| Title-bar filename | #7a7c84 | overlay/subtext |
| Active-tab label | #4c4e5a on tab bg #dcdee4 | — |
| Active tab underline / accent | #7a48e4 | mauve #8839ef |
| Edit pencil button | #7c48e6 | mauve #8839ef |
| Toolbar active button | bg #d2d0e6, icon #7151c0 | mauve tint |
| Toolbar icons | #93979d | overlay |
| Link | #4364c9, underlined | blue #1e66f5 |
| Code block bg | #e6e8ec (on #eef0f4 page) | mantle |
| Code language label | #a1a2a8 | — |
| Syntax: keyword (from/import/def) | #6e50ba | mauve |
| Syntax: function name | #4766c0 | blue |
| Syntax: builtin type `int` | #b99051 | yellow/peach |
| Syntax: string/docstring | #6a8f4f | green |
| Heading rule / table borders | #dcdde1 / #d1d3d7 | surface0 #ccd0da (AA-lightened) |

**Table B. Dark, Mocha-like.** Measured from `app-code-dark.webp`, `app-appstore-screenshot5.png` and `app-appstore-screenshot14.png` [APP].

| Role | Measured | Nearest Mocha |
|---|---|---|
| Sidebar / title bar (App Store) | #181824 | mantle #181825 |
| Bottom toolbar | #181824 | mantle #181825 |
| Editor surround / website title bar | #1e202e to #1e2030 | base #1e1e2e |
| Page sheet | #242638 to #262638 | none; between base #1e1e2e and surface0 #313244 |
| Body text | #d5d8ef to #dde3fe | text #cdd6f4 |
| Accent (tab underline, Edit button, row buttons) | #bca4f0 to #c0a6f4 | mauve #cba6f7 |
| Link | #a1b2e7 | blue #89b4fa |
| Code block bg | #2c3040 | — |
| Syntax: keyword | #b8afe6 | mauve |
| Syntax: function | #a2b1e3 | blue |
| Syntax: type | #e2d8bd | yellow #f9e2af |
| Syntax: string | #c0d3b4 | green #a6e3a1 |
| Table header bg | #2e3044 | — |
| Selected table row | #403e5a | — |
| Sidebar selected row | #36324a | — |

Some measured dark values (#1e2030 and #181926-like) are also close to Catppuccin Macchiato's mantle and crust. Given the CSS comment, Mocha with a custom lifted page colour is the better fit (*inferred*).

**Table C. Cream light theme in App Store shots (probably "Dinkus Light", inferred).** Measured from `app-appstore-screenshot4.png` and `app-appstore-screenshot15.png` [APP].

| Role | Measured |
|---|---|
| Sidebar / title bar | #e4e4da |
| Editor surround / bottom toolbar | #f0f0e6 |
| Page sheet | #fefcf0 |
| Body and heading text | ≈#000000 |
| Title-bar text | #666561 |
| Accent / row buttons / tab underline | #544196 to #584498 |
| Table header bg | #f0eee2 |
| Selected table row | #e0dae0 |
| Sidebar selected row | #ccc6ce |
| Table borders | #645d4e (darker, warm) |

**Table D. Neutral dark (probably "Dinkus Dark", inferred).** Measured from `third-mastodon-2026-10-31-html-export.png`, posted on the 1.11 release day.

| Role | Measured |
|---|---|
| Sidebar | #121212 |
| Surround | #181818 |
| Page | #1e1e1e to #1f1f1f |
| Text | #cccccc |
| Link-blue sidebar highlight | #7a9dcb |

- **Focus-mode dimming** [APP]:
  - Light: non-focused text #979690 vs focused #010101 on #fefcf0, about 40% opacity. Measured from `app-appstore-screenshot15.png`.
  - Dark: #757890 vs #d8dffd on #242638, about 45%. Measured from `app-appstore-screenshot14.png`.
  - Headings are dimmed too. A transient pill reading "Paragraph focus" with an eye icon appears at the top of the page.
  - Sentence focus in the focus video works the same way, at sentence granularity. Measured from `app-focus-light.mp4` frames.

### Inferences
- In the light theme the page sits on base and the chrome on mantle/crust. In the dark theme the page is lifted above base. A ramp of chrome = mantle, surround = base, page = base or slightly lifted, with mauve as the single accent, reproduces the look.

### Gaps
- No public list of the app's theme names before 1.11. I cannot confirm that the app ships named "Catppuccin" themes; the only evidence is the site CSS comment.
- Exact hex values in the app's theme files are not public.

---

## 5. Chrome: title bar, tabs, sidebar, toolbar, buttons, and what is visible while writing (APP)

### Takeaway
The chrome is compact and native-looking: a unified title and tab bar (about 60 pt combined), an optional left sidebar (about 19% of the window) and a ~40 pt bottom formatting toolbar. While writing, the page sheet dominates. Typewriter mode adds only a dashed guide line. Focus modes dim everything except the current sentence or paragraph.

### Cited Findings (measured or seen in the named files)
- **Title bar**, seen in `app-appstore-screenshot4.png`, `app-appstore-screenshot5.png` and `app-dinkus.mp4`:
  - Traffic lights, then a sidebar toggle.
  - Window title "Dinkus — reptiles.md — Edited" in small grey text, left-aligned after the sidebar.
  - A centred **Edit | Read** segmented control; Edit is selected, with a filled grey pill and pencil icon.
  - Right-side icon buttons (eye, briefcase-like, bracket and split icons, document-search); active ones have a filled rounded square.
  - Older website shots have a centred filename title instead.
- **Tab bar**: tabs show the filename, with × to close or ● when unsaved. The active tab has a 2–3 pt accent-coloured underline and a slightly darker fill. A split-pane button sits at the right end. Measured from `app-code-light.webp`: underline y≈196–199 in mauve #7a48e4.
- **Sidebar**, seen in `app-appstore-screenshot4/8/12.png` and `app-wysiwyg-light.webp`:
  - Pane switcher icons at the top: folder, search, #tags, outline. The active pane has an accent underline.
  - Folder tree with chevrons and file/folder line icons. Non-Markdown files are dimmed.
  - The selected row has a rounded filled highlight (light #ccc6ce, dark #36324a).
  - "New File" and "New Folder" at the bottom.
  - The outline pane shows the heading hierarchy.
- **Bottom toolbar** (about 40 pt), seen in `app-code-light.webp` and `app-appstore-screenshot4.png`:
  - Edit-mode toggles, then a block-style dropdown in italic serif ("paragraph", "heading 1").
  - B, I, S, U, inline code, kbd, link, footnote (*), bullet and numbered lists, image, rule, indent and outdent, code block, table, HTML, inline and block math.
  - The active tool has a mauve-tinted rounded background.
  - Release 1.5 notes add a "Styles indicator in footer". — [App Store](https://apps.apple.com/app/dinkus-markdown-studio/id6799480413)
- **In-document affordances**: a floating rounded-square mauve pencil "edit" button at the top-right of code blocks and images, and purple pill row/column insert buttons on tables. Seen in `app-code-light.webp` and `app-tables-light.webp`.
- **Dialogs**: Image, Settings and Quick open (⌘P) are modal cards over a dimmed window.
  - Uppercase small-caps section labels ("IMAGE", "SETTINGS").
  - Grey secondary buttons; a filled mauve primary button ("Apply", "Save"); a red-outlined destructive button ("Delete Image").
  - Toggles in accent colour.
  - Seen in `app-appstore-screenshot6/7.png`, `app-blog-caret-dialog.webp` and `app-appstore-screenshot8/9.png`.
- **Empty/welcome state**: the three-asterisk logo centred, with the hint "⌘P for quick open". Seen in `app-appstore-screenshot8.png`.
- **While writing (typewriter/focus videos)**: the frame shows only the page sheet on the surround and the bottom toolbar. No sidebar or title bar is visible, presumably full-screen. A dashed horizontal guide line marks the caret line, and the caret is a thin mauve bar. Seen in `app-typewriter-light.mp4` and `app-focus-light.mp4`.
- **Code blocks**:
  - Full text-column width.
  - Corner radius ≈6–8 px at 2× (≈3–4 pt).
  - Left padding ≈27 px (≈13 pt).
  - A small grey language label ("python") at the top-right.
  - No border.
  - Measured from `app-code-light.webp`.
- **Front matter**: a monospace block pinned above the page, with a `{}` icon and YAML syntax colours. Seen in `app-frontmatter-light.webp`.

### Inferences
- The app has no traditional status bar. The bottom bar is the formatting toolbar, and the "styles indicator" is part of it.

### Gaps
- No word count or status display was visible in any image.
- Exact icon set and toolbar button sizes not determined, beyond about 22 pt buttons implied by the 40 pt bar.

---

## 6. Website design tokens ([SITE], not the app)

### Takeaway
The site uses Inter Variable for UI and body text and JetBrains Mono for code. Its colours are the exact Catppuccin Latte and Mocha palettes, with mauve as the accent. Reading measure is 38rem, and body text is 1.05rem with a line-height of 1.65.

### Cited Findings (all stated at [styles.css](https://dinkus.textualize.io/assets/styles.css?v=11f8355c11) unless noted)
- The file header reads "Ishmael website — base stylesheet". "Ishmael" is apparently an internal codename.
- **Fonts**:
  - `@font-face` Inter from /assets/fonts/InterVariable.woff2 and InterVariable-Italic.woff2, weights 100–900.
  - JetBrains Mono from JetBrainsMono.woff2, weights 100–800.
  - `--font-ui: "Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`.
  - `--font-mono: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, Consolas, monospace`.
  - The page preloads InterVariable.woff2. — [dinkus.textualize.io](https://dinkus.textualize.io/)
- **Light (Latte) role tokens**:
  - `--bg #eff1f5`, `--bg-elevated #e6e9ef`, `--surface #ccd0da`, `--rule #bcc0cc`
  - `--text #4c4f69`, `--text-dim #6c6f85`
  - `--accent var(--ctp-mauve)` (#8839ef), `--accent-hover #7a2ee0`
  - `--link` mauve, `--link-hover` blue (#1e66f5)
  - The full `--ctp-*` named palette: rosewater #dc8a78, flamingo #dd7878, pink #ea76cb, mauve #8839ef, red #d20f39, maroon #e64553, peach #fe640b, yellow #df8e1d, green #40a02b, teal #179299, sky #04a5e5, sapphire #209fb5, blue #1e66f5, lavender #7287fd.
- **Dark (Mocha) role tokens**:
  - `--bg #1e1e2e`, `--bg-elevated #181825`, `--surface #313244`, `--rule #45475a`
  - `--text #cdd6f4`, `--text-dim #bac2de`
  - `--accent-hover #d4b8ff`
  - Palette: mauve #cba6f7, blue #89b4fa, green #a6e3a1, yellow #f9e2af, peach #fab387, red #f38ba8, teal #94e2d5, sky #89dceb, sapphire #74c7ec, pink #f5c2e7, flamingo #f2cdcd, rosewater #f5e0dc, maroon #eba0ac.
  - Lavender is written as #b4befc; canonical Mocha lavender is #b4befe.
- **Theme mechanics**: `data-theme` on `<html>`, set before paint from localStorage key `dinkus-site-theme`, otherwise from `prefers-color-scheme`. The `theme-color` meta tags are #eff1f5 (light) and #1e1e2e (dark). — [dinkus.textualize.io](https://dinkus.textualize.io/)
- **Type and spacing**:
  - body `font-size: 1.05rem; line-height: 1.65`.
  - At small widths `html { font-size: 84.375% }` (13.5px).
  - `code` is 0.92em on `--bg-elevated`.
  - `--max-measure: 38rem`.
  - Weights used: 400, 500, 600, 700.
  - Common sizes: 0.78–1.25rem for UI, 1.6–4rem for headings.
  - Line-heights used: 1, 1.1, 1.15, 1.25, 1.4, 1.45, 1.5, 1.55, 1.65.
  - Common margins: 0 0 1rem, 2rem 0, 3rem bottom; gaps of 0.4–3rem.
- **Radii**: 6px (4×), 5px (2×), 12px (2×), 8px, 3px, 0.75rem, 0.65em, 999px pills, 50% circles.
- **Shadows**:
  - Light: `--img-shadow: 0 1px 2px rgba(0,0,0,.05), 0 6px 18px rgba(0,0,0,.12)` and `--shot-shadow: 0 4px 10px rgba(0,0,0,.20), 0 14px 34px rgba(0,0,0,.34)`.
  - Dark: `0 1px 2px rgba(0,0,0,.3), 0 10px 26px rgba(0,0,0,.5)` and `0 4px 10px rgba(0,0,0,.45), 0 16px 38px rgba(0,0,0,.55)`.

### Inferences
- The site's Latte/Mocha tokens are a good starting reference for the app's original two themes, since the CSS comment ties them together. The app's own measured values add a separate, lifted page-sheet colour in dark mode, which the site does not have.

### Gaps
- The site does not state the app's fonts or sizes. Inter and JetBrains Mono are confirmed for the site only.
