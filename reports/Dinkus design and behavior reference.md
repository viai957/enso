# Dinkus sets the bar enso must clear

Dinkus is a $9.99, Mac-only, WYSIWYG Markdown "studio" that has been out for about 2.5 months. It opens any `.md` file as a rendered, editable page with no preview pane. It keeps raw source a click or ⌥⌘U away, either toggled in place or side by side. Its tables, images, code, front matter, KaTeX equations and Mermaid diagrams are edited visually through compact modal dialogs and small floating accent-coloured buttons. Its most distinctive feature comes from the author's novel-writing app Ishmael: a typewriter mode with a draggable dashed guide line, plus sentence and paragraph focus modes that dim everything else to about 40–45% contrast ([Home](https://dinkus.textualize.io/), [About](https://dinkus.textualize.io/about/)). Visually it is calm and native-looking. The document sits on a bordered page sheet about 70–76% of the pane width, in an Inter-like sans or a text serif, on Catppuccin Latte/Mocha-derived colours with a single mauve accent. Version 1.11 added new cream "Dinkus Light" and neutral "Dinkus Dark" themes ([styles.css](https://dinkus.textualize.io/assets/styles.css?v=11f8355c11), [App Store](https://apps.apple.com/app/dinkus-markdown-studio/id6799480413)). Under the hood it is a Tauri web-view app, not a native renderer ([Mastodon](https://mastodon.social/@willmcgugan/117190614437691468)). Users still call it "snappy", "smooth, intuitive, and reliable", and say it "doesn't look like a code editor". Their complaints are about Markdown fidelity edge cases (reference links and footnotes ignored in the visual view, SVG sizing), opinionated defaults (typewriter mode always on), and integrations, not about typing feel ([Tao of Mac](https://taoofmac.com/space/links/2026/08/23/1121), [App Store](https://apps.apple.com/us/app/dinkus-markdown-studio/id6799480413?mt=12)). That leaves enso a clear path: match the polish, beat the fidelity, ship saner defaults, and add the two things a web view cannot easily give, which are truly native rendering and Windows/Linux. Public evidence is thin: there is no manual, no FAQ, no shortcut list and no stated design values. So every measurement below comes from screenshots and videos and should be read as approximate.

## A visual editor that hides syntax but never traps you in it

Dinkus calls itself "a distraction-free markdown studio for Mac" for "note-takers, bloggers, site builders, and AI pioneers". It promises editing "without a preview pane or compilation step" across "inline styles, headers, code blocks, images, tables, footnotes, and more" ([Home](https://dinkus.textualize.io/)). The About page frames this as "no risk of broken syntax" ([About](https://dinkus.textualize.io/about/)). The safety valve is the **source view**: "Switch to the source view with a click or edit the source side-by-side", then "return to visual mode when you are done to continue editing where you left off". That implies the caret position is mapped across views. The toolbar tooltip reads **"Edit Markdown Source ⌥⌘U"** ([hero video](https://dinkus.textualize.io/assets/img/dinkus.mp4)). Side by side places the rendered page on the left and syntax-highlighted source on the right, each pane with its own tab strip. In the source pane, headings are purple, URLs teal and code fences orange, and inline HTML such as `<img … align="right">` is kept verbatim ([source-light.webp](https://dinkus.textualize.io/assets/img/source-light.webp)). Since 1.8 the title bar also has a centred **Edit | Read** segmented control for a read-only mode. Since 1.9 panes split vertically or horizontally (shift-click the split icon), and their dividers and the sidebar divider can be dragged ([App Store](https://apps.apple.com/app/dinkus-markdown-studio/id6799480413), [Mastodon video](https://mastodon.social/@willmcgugan/117360286159032802)).

Structure changes go through a **paragraph-style popup** at the caret, titled "PARAGRAPH STYLE". It offers blockquote, bullet list, code block, heading 1–6, numbered list and paragraph, and it filters as you type ("2" selects heading 2). The bottom toolbar has a matching block-style dropdown, set in italic serif ("paragraph", "heading 1"). A transient pill naming the current block type ("Paragraph") appears near the bottom of the page ([hero video](https://dinkus.textualize.io/assets/img/dinkus.mp4), [Mastodon reply](https://mastodon.social/@willmcgugan/117366546507679801)). One interaction detail drew criticism: pressing Enter at the end of an H1 creates another H1, which a power user found "shouty" when he expected a paragraph ([Mastodon](https://mastodon.social/@rcarmo/117144571824832015)).

**Tables** are the feature users single out. An insert dialog sets columns × rows (default **3 × 3**, "header row included"). Its Header-row toggle explains that "Markdown tables always carry a header line; without one it saves with empty header cells". In the document, the selected column is tinted mauve, and two small pill buttons, "‹+" and "+›", insert a column on either side. Columns widen as you type, and cells accept images and inline syntax ([tables-light.webp](https://dinkus.textualize.io/assets/img/tables-light.webp), [Mastodon reply](https://mastodon.social/@willmcgugan/117366546507679801)). On the source side Dinkus "will even format and extend the column by adding spaces to keep your table source nice and tidy", which means it auto-aligns pipe tables ([Home](https://dinkus.textualize.io/)). A Typora power user reacted with "what, you can edit tables wysiwyg? I have to try it!" He still listed what Typora does that Dinkus does not: inline HTML in cells, raw `<table>` fallback, HTML comments and horizontal scrolling for wide tables. The developer said horizontal scrolling "might happen" ([Mastodon thread](https://mastodon.world/@samueljohn/117371422129210786)).

The **image dialog** is the most fully specified UI. It sits under a small-caps "IMAGE" header and has, in order:

- a preview in a dashed box with the natural size ("1280 × 640 px");
- a path/URL field and a **Browse…** button, with help text explaining relative paths, `/root`-relative paths and http(s) URLs;
- an alt-text field, explained as "the ![alt] in the markdown";
- **Size** as width × height ("pixels or a percentage (40%); empty for natural size");
- **Spacing** as hspace × vspace steppers ("how GitHub spaces floated images");
- four **Align** buttons (none, float left, centre, float right; for example "Float right — text wraps on the left");
- an **Inline** toggle ("Sized or aligned images save as HTML <img>; otherwise markdown");
- a red-outlined **Delete Image** button on the left, and Cancel and a filled-accent **Apply** on the right.

([images-light.webp](https://dinkus.textualize.io/assets/img/images-light.webp)) The serialisation rule is the important part: Dinkus "will generate standard Markdown markup from images wherever possible" and falls back to `<img>` only for GitHub-style floats and sizing ([Home](https://dinkus.textualize.io/)). A selected image gets an accent outline and two corner buttons: a magnifier, and a pencil that opens the dialog. Dragging a file from the sidebar inserts `![calculator.png](imgs/calculator.png)` ([hero video](https://dinkus.textualize.io/assets/img/dinkus.mp4)). One user found that a dropped SVG's "auto sizing bit has gone wonky" ([Mastodon](https://mastodon.social/@rcarmo/117144686461543691)).

**Code blocks** render on a slightly darker borderless panel, about 3–4 pt corner radius and about 13 pt left padding. A small grey language label ("python") sits top-right with a pencil edit button, and you type directly into the rendered block. The site claims highlighting for "common languages—and many uncommon ones" ([code-light.webp](https://dinkus.textualize.io/assets/img/code-light.webp), [Home](https://dinkus.textualize.io/)). **Front matter** is shown as a full-width monospace band above the page card, outside the white sheet, with a `{}` icon and YAML syntax colours. The copy claims "all the formats in common use today" but names none; only YAML is shown, and 1.7 added Pelican-style metadata ([frontmatter-light.webp](https://dinkus.textualize.io/assets/img/frontmatter-light.webp), [App Store](https://apps.apple.com/app/dinkus-markdown-studio/id6799480413)). **Equations** use an "EQUATION" dialog with a LaTeX textarea, a live preview and the note "Rendered with KaTeX. Emptying it removes the equation", with toolbar buttons for inline and display math ([hero video](https://dinkus.textualize.io/assets/img/dinkus.mp4)).

The App Store history shows how fast the rest arrived: **12 releases from 19 Aug to about 1 Nov 2026** ([App Store](https://apps.apple.com/app/dinkus-markdown-studio/id6799480413)).

| Version | What shipped |
|---|---|
| 1.1 | Folder view, equation editor |
| 1.3 | Welcome screen, restoring open files, more Markdown extensions, fixes for Asian languages |
| 1.5 | Collapsible headings, a "detail slider", tap-to-select |
| 1.6 | Mermaid |
| 1.7 | Checklists, a footer styles indicator |
| 1.8 | Read mode, full-text search, ⌘P quick find, #tags, outline view, vertical splits |
| 1.9 | Draggable dividers, printing, SVG and Word export |
| 1.10 | Better grammar-checker support |
| 1.11 | Dinkus Light/Dark themes, single-file HTML export with images |

The listing also mentions footnotes, raw HTML blocks and a link dialog that targets local headings, documents and URLs.

## Calm chrome around a page sheet, measured

The window is compact. A unified title bar and tab bar take about **60 pt** combined. The title bar has traffic lights, a sidebar toggle, a small grey "Dinkus — reptiles.md — Edited" title, the centred Edit | Read control, and right-side icon buttons (eye, brackets, split, document search) whose active state is a filled rounded square. Tabs show × or a dirty ●, and the active tab carries a **2–3 pt accent underline** ([code-light.webp](https://dinkus.textualize.io/assets/img/code-light.webp), [App Store](https://apps.apple.com/app/dinkus-markdown-studio/id6799480413)).

The optional left sidebar is about **19–20% of the window** (about 224 pt in the website capture). It has four pane tabs: folder, search, #tags and outline. The folder tree dims non-Markdown files, highlights the selected row with a rounded fill, and offers New File and New Folder. Search groups results into "TITLES" and "IN DOCUMENTS", with the match shown in the accent colour ([X 1.8.0 post](https://x.com/willmcgugan/status/2103121091979792837)). The empty state is the logo plus the hint "⌘P for quick open".

At the bottom, a **40 pt formatting toolbar** replaces a status bar. From left to right it holds: source toggle, block-style dropdown, bold, italic, strikethrough, underline, inline code, kbd, link, footnote, lists, image, rule, indent and outdent, code block, table, HTML block, and inline and block math. Active tools get a mauve-tinted rounded background ([wysiwyg-light.webp](https://dinkus.textualize.io/assets/img/wysiwyg-light.webp)). **No word count or status readout appears in any image.**

Dialogs (Image, Table, Equation, Settings, Quick Open) are modal cards over a dimmed window. Each has a small-caps header, grey secondary buttons, a filled accent primary button and a red-outlined destructive button. Settings uses a sidebar with General, Caret, Auto-correct, Markdown, Language and Tips ([caret-dialog.webp](https://dinkus.textualize.io/assets/img/caret-dialog.webp)). The macOS menu bar reads Dinkus, File, Edit, Insert, Go, View, Window, Help, Developer, but no menu contents have been published ([hero video](https://dinkus.textualize.io/assets/img/dinkus.mp4)).

The document is a **bordered page sheet** in a slightly different colour from the surround, centred in the editor pane:

- **Sheet width:** 75.5% of a 2400 px window with no sidebar, about 70–71% of the editor pane with the sidebar open, and 75.6% in the full-screen typewriter clip.
- **Text inset inside the sheet:** about 46 pt per side in the sans captures, about 26 pt in the video.
- **Gap above the sheet:** about 15 pt under the tab bar.

([code-light.webp](https://dinkus.textualize.io/assets/img/code-light.webp), [typewriter-light.mp4](https://dinkus.textualize.io/assets/img/typewriter-light.mp4)) Because the width scales with the window and has no cap, measured lines run **84–110 characters**. That is well past the 60–75 character comfort range, and the measurements reflect a proportional layout, not a reading-measure decision. The marketing site, by contrast, caps prose at `--max-measure: 38rem` ([styles.css](https://dinkus.textualize.io/assets/styles.css?v=11f8355c11)).

Typography, assuming the website captures are 2× Retina:

- **Body:** an **Inter-like sans** at about **17–18 pt**, with about **25 pt line pitch** (line-height about 1.45) and about **0.7 line** of extra space between paragraphs.
- **Headings:** the same colour as body text, scaled about **2.1× (H1), 1.5× (H2) and 1.25× (H3)** by x-height. H1 and H2 get a 1 pt hairline rule about 20 pt below the baseline, in GitHub style.
- **Code:** a JetBrains Mono-like face at about 15 pt with about 18 pt pitch.
- **Serif:** a bracketed text serif, about 22.8 pt pitch with about 0.66-line paragraph gaps, appears in the novel samples, the typewriter and focus clips, and the caret preview "Call me Dinkus."

([tables-light.webp](https://dinkus.textualize.io/assets/img/tables-light.webp), [focus-light.mp4](https://dinkus.textualize.io/assets/img/focus-light.mp4)) The same cream theme appears with sans in one App Store shot and serif in another, so the typeface is probably a setting independent of theme. Dinkus has never documented it. Table rows are about 38 pt tall, with about 10 pt vertical padding, 1 pt borders and a header row a shade darker than the page.

## Four palettes, one accent

The site's stylesheet states that its themes are a "two-theme model mirroring the editor app: Catppuccin Latte (light) and Mocha (dark)", with `--accent: var(--ctp-mauve)` and the comment "matches the app's brand" ([styles.css](https://dinkus.textualize.io/assets/styles.css?v=11f8355c11)). Pixel sampling of the app agrees, within ±2–6 per channel of WebP compression error. In the light shots:

- **Chrome:** mantle (#e6e8ec).
- **Page:** base (#eef0f4).
- **Toolbar:** crust (#dce0e6).
- **Text:** about #494b59.
- **Accent:** about #7a48e4.
- **Links:** underlined #4364c9.
- **Rules and borders:** #dcdde1 and #d1d3d7.
- **Syntax:** keywords in mauve, functions in blue, types in yellow-ochre and strings in green.

In dark, the chrome is about #181824, the surround about #1e2030, and the **page sheet is lifted to about #242638** (no Mocha colour matches it). Text is about #d5d8ef, and the accent is about #bca4f0 ([code-dark.webp](https://dinkus.textualize.io/assets/img/code-dark.webp)).

The newer App Store shots add a **cream light theme**: page #fefcf0, surround #f0f0e6, chrome #e4e4da, near-black text and a deep-purple accent around #544196. A late-October post shows a **neutral dark** theme: sidebar #121212, surround #181818, page #1f1f1f, text #cccccc. These are probably the 1.11 "Dinkus Light" and "Dinkus Dark" ([App Store](https://apps.apple.com/app/dinkus-markdown-studio/id6799480413), [Mastodon](https://mastodon.social/@willmcgugan/117537450620070510)). Asked for system dark mode, the developer replied that Dinkus Dark is "not exactly the same as the system theme, but it is close". In other words, **the app does not automatically follow macOS appearance** ([App Store](https://apps.apple.com/us/app/dinkus-markdown-studio/id6799480413?mt=12)).

The design idea all four palettes share is a three-step surface ramp (chrome darkest or most tinted, surround, then the page sheet as the "paper") with exactly one saturated accent. That accent covers the caret, tab underline, selection, the selected table column, primary buttons, toggles, divider-drag highlights and floating edit badges.

## Typewriter, focus and caret are where Dinkus shows its craft

**Typewriter mode** "keeps the caret anchored on a single horizontal line by moving the document vertically rather than the caret itself". The line is marked by a thin **dashed guide** across the full pane, at about mid-height by default. Uniquely, "you can drag the insertion guide away from the center of the screen" to keep more context above the caret ([Home](https://dinkus.textualize.io/)). The feature is also Dinkus's most criticised default. Python Bytes reported it as on for every file with no obvious way to switch it off, and an App Store reviewer called the design "Weird and Frustrating". The developer replied that **⌘T** toggles it and that the next version would remember the setting ([Python Bytes #494](https://pythonbytes.fm/episodes/show/494/python-wrapture), [App Store](https://apps.apple.com/us/app/dinkus-markdown-studio/id6799480413?mt=12)).

**Focus mode** "can highlight the sentence or paragraph you are working on and dim the rest". Dimming is a plain colour fade, with no blur. Measured, the unfocused text sits at about **40% contrast in light** (#979690 against #010101 on #fefcf0) and about **45% in dark** (#757890 against #d8dffd on #242638). Headings dim too, and a transient "Paragraph focus" pill with an eye icon confirms the mode change ([Home](https://dinkus.textualize.io/), [App Store](https://apps.apple.com/app/dinkus-markdown-studio/id6799480413)). The typewriter and focus clips show only the page sheet, the guide and the bottom toolbar, with no sidebar or title bar. That suggests a full-screen writing state, although no "hide chrome" feature is documented ([focus-light.mp4](https://dinkus.textualize.io/assets/img/focus-light.mp4)).

**Caret settings** are unusually deep. Settings → Caret has a live preview line and the following controls ([HDR caret post](https://dinkus.textualize.io/blog/hdr-caret/)):

- **Colour:** 16 pastel swatches, with the note "Theme colors adapt when you switch themes".
- **Size:** a width slider, shown at **2.5 px**.
- **Blink:** an on/off toggle.
- **Blink rate:** a slider, shown at **0.8 s**.
- **Blink pattern:** a **12-cell grid**, shown with 8 cells filled (visible for about two-thirds of the cycle).
- **HDR Brightness:** an opt-in toggle that, on an EDR display, makes the caret glow "brighter than pure white". It works by "clipping a tiny HDR image to the shape of the caret" because "HDR support in CSS is currently spotty".

Screenshots show the caret in mauve, peach and teal, which confirms that users change it ([Mastodon](https://mastodon.social/@rcarmo/117144571824832015)).

## Web-view engine, native-feeling results, and where fidelity breaks

The developer confirmed that Dinkus and Ishmael are built on **Tauri** (a web view in a native shell) ([Mastodon](https://mastodon.social/@willmcgugan/117190614437691468)) and that Dinkus "was built with the help of AI. Claude mostly" ([Mastodon](https://mastodon.social/@willmcgugan/117150365436495360)). The App Store binary is 11.3 MB, requires macOS 12+ on Apple silicon, and is listed under Developer Tools ([App Store](https://apps.apple.com/app/dinkus-markdown-studio/id6799480413)). **No latency, frame-rate, ProMotion or large-file claims exist anywhere**, and no source mentions lag. Perceived quality comes from polish: "snappy and polished", "smooth, intuitive, and reliable", "opinionated and detail oriented", "looks so sleek" ([Tao of Mac](https://taoofmac.com/space/links/2026/08/23/1121), [App Store](https://apps.apple.com/us/app/dinkus-markdown-studio/id6799480413?mt=12), [Mastodon](https://mastodon.world/@samueljohn/117367955621011173)).

The business model is part of the appeal: **$9.99 one-time**, sold only on the Mac App Store, "plain .md on disk · no account · works offline". It has no analytics, never "phones home", makes no update checks, and its only network traffic is fetching remote images a document references ([Pricing](https://dinkus.textualize.io/pricing/), [Privacy](https://dinkus.textualize.io/privacy/)). It is English-only, offers no trial, and spelling and grammar go through the macOS system service ([App Store](https://apps.apple.com/app/dinkus-markdown-studio/id6799480413)).

The weaknesses cluster around fidelity:

- **Reference links and footnotes are ignored in the visual view.** A reference definition like `[ultrawide]: blog/2021/08/21/1600` renders as plain body text, even though source view understands it. Rui Carmo called this "the only notable omission" ([Tao of Mac](https://taoofmac.com/space/links/2026/08/23/1121), [Mastodon](https://mastodon.social/@rcarmo/117146080204963255)).
- **Byte-preserving round trips are never claimed.** The copy promises only that documents are "saved as standard Markdown", and the automatic table padding is a normalisation by design ([Home](https://dinkus.textualize.io/)).
- **Some integrations misfire.** Grammarly had problems, a deleted folder failed to refresh the tree (suspected race condition), and SVG sizing broke ([App Store](https://apps.apple.com/us/app/dinkus-markdown-studio/id6799480413?mt=12), [Mastodon](https://mastodon.social/@rcarmo/117144484411112167)).
- **Requested features remain open**, including an iPad version and Windows/Linux, which nobody has promised ([Mastodon thread](https://mastodon.world/@samueljohn/117371422129210786)).

One critic rejects the premise altogether and wants plain text beside a preview. The developer concedes that a source-view default "could become an option" ([App Store](https://apps.apple.com/us/app/dinkus-markdown-studio/id6799480413?mt=12)).

## Concrete targets for enso

The values in this section are **enso's own choices, derived from the observations above. They are not Dinkus's official values**, which are unpublished. They deliberately depart from Dinkus where the evidence points to a weakness, such as the uncapped measure and appearance not following the OS. enso should use its own palette names, fonts it has licensed or that are open (Inter and JetBrains Mono are OFL and generic choices, but enso should pick its own pairing), and its own icons and copy.

### Proposed design tokens (enso choices)

| Token | Light | Dark | Rationale from observations |
|---|---|---|---|
| `surface.chrome` (title, tabs, sidebar) | #E7E8EC | #17181F | Darkest or most tinted step; Dinkus ramps chrome, then surround, then page |
| `surface.canvas` (around page) | #EEEFF2 | #1C1D26 | Middle step |
| `surface.page` (sheet) | #F8F8F6 | #23242F | Lifted "paper"; dark page lifted above canvas, as observed |
| `surface.toolbar` | #E1E3E8 | #17181F | Bottom bar matches chrome |
| `text.body` | #2E3038 | #D9DBE6 | Higher contrast than Dinkus Latte's #494b59 (about 7:1 vs about 6:1) |
| `text.muted` (paths, labels) | #6E717C | #9497A6 | Title text and code labels about #7a7c84 / #a1a2a8 observed |
| `text.dimmed` (focus mode) | body at 38% mix toward page | body at 42% mix | Observed about 40% / 45% |
| `accent` | #6D4AE0 | #B9A4F2 | One saturated accent, used everywhere interactive |
| `accent.tint` (active tool, selected column) | #DDD6F5 | #3A3652 | Observed #d2d0e6 / #dad2f0 tints |
| `link` | #3A5FC8 | #9FB3EC | Blue, distinct from accent |
| `rule` / `table.border` | #D6D8DE | #3A3C4B | Hairlines at 1 px |
| `code.bg` | #ECEDF1 | #2A2C3A | Borderless panel a step off the page |
| `danger` | #C8324A | #F08AA0 | Destructive outline buttons |
| Body font / size / line-height | humanist or neo-grotesque sans, 17 pt, 1.5 | same | Matches observed about 17–18 pt, 1.45; slightly airier |
| Optional prose serif | 18 pt, 1.45 | same | Serif mode exists in Dinkus but is undocumented; enso exposes it as a setting |
| Mono | 14.5 pt, 1.4 | same | Observed about 15 pt / 18 pt pitch |
| Heading scale | H1 2.0×, H2 1.5×, H3 1.25×, H4 1.1×, H5–H6 1.0× semibold | same | Observed 2.1 / 1.5 / 1.25; H1–H2 hairline rule |
| Paragraph gap | 0.7 line | same | Observed 0.66–0.7 |
| Measure | **max 72 ch**, page = min(76% pane, measure + 2×48 pt) | same | Departs from Dinkus's 84–110 ch |
| Page inset | 48 pt (36 pt on narrow panes) | same | Observed about 46 pt |
| Chrome heights | title+tabs 60 pt combined, bottom toolbar 40 pt, sidebar default 224 pt (draggable) | same | Observed |
| Radii | 4 pt code/badges, 8 pt dialogs, 999 pt pills | same | Observed code about 3–4 pt; site uses 6/12 px and pills |
| Table rows | 10 pt vertical padding, header one step darker | same | Observed about 38 pt rows |
| Caret | 2 px, accent colour, blink 1.0 s at 2/3 duty, blink off after 30 s idle-typing pause optional | same | Dinkus defaults look like 2.5 px / 0.8 s / 8-of-12 |
| Motion | 150–200 ms ease-out for dialogs and pills; typewriter scroll critically damped, at most 120 ms | same | Dinkus site zoom uses `cubic-bezier(0.2,0,0.05,1)`; no app timings known |

### Prioritized feature checklist

| Priority | Feature | Target behaviour |
|---|---|---|
| P0 | Visual editing with lossless round-trip | Unedited regions saved **byte-identical**; edits re-serialise only the touched block; list markers, emphasis chars, wrapping preserved |
| P0 | Source toggle and side by side | One shortcut (enso's own, e.g. ⌘/ or Ctrl+/); caret and scroll mapped both ways; synced scrolling in split |
| P0 | Core blocks | Headings with style popup (type-to-filter), lists, checklists, blockquotes, rules, highlighted code with language label and direct typing |
| P0 | Appearance | Light and dark that **follow the OS by default**, with manual override |
| P0 | Native performance | 120 Hz scrolling and caret on ProMotion; under 8 ms keystroke-to-paint; large-file (1 MB+) budget |
| P1 | Tables | Insert dialog (3×3, header toggle), column/row pill buttons, auto-widening, tidy pipe-table source as an **opt-in** setting, horizontal scroll for wide tables, inline HTML in cells preserved |
| P1 | Images | Dialog with path/Browse, alt, size (px/%), spacing, 4 alignments, inline toggle; Markdown unless `<img>` is required; correct SVG intrinsic sizing; drag from tree or Finder |
| P1 | Typewriter mode | **Off by default**, one shortcut, remembered per app (optionally per document); draggable guide; guide line can be hidden |
| P1 | Focus mode | Sentence and paragraph; colour fade at about 40%; transient mode pill |
| P1 | Front matter | YAML, TOML, JSON, Pelican/MultiMarkdown metadata in a highlighted band above the page; never rewritten unless edited |
| P1 | Footnotes and reference links in visual view | Rendered as superscripts and link targets with hover/peek, definitions shown as a styled end section, untouched on save |
| P2 | Caret settings | Colour, width, blink on/off, rate, pattern; EDR/HDR brightness on macOS via extended-range Metal layer where available |
| P2 | Workspace | Folder tree, full-text search (titles and in-document groups), quick open, outline, tags, tabs, draggable split panes |
| P2 | Math and diagrams | Inline and display LaTeX with live-preview dialog; Mermaid |
| P2 | Read mode, print, export | HTML (single self-contained file), PDF; Word as later stretch |
| P3 | Collapsible headings, link dialog to headings, files and URLs, word count in a quiet status area | — |

### Where enso can do better than Dinkus

The biggest opening is **file fidelity**. Dinkus never claims byte-preserving saves, normalises tables by design, and ignores reference links and footnotes in its visual view. A Markdown editor that guarantees "opening and saving without edits changes zero bytes", and that renders reference-style links and footnotes as live, navigable elements, answers the most specific complaint power users made ([Tao of Mac](https://taoofmac.com/space/links/2026/08/23/1121)).

The second opening is **defaults**. Typewriter mode should start off, be remembered, and be togglable from a shortcut shown in the menu and tooltip. Enter after a heading should yield a paragraph. Appearance should follow the system. Users can opt into a source-first default, which the App Store critic asked for. Line length should be capped for readability instead of stretching to 110 characters.

Third, enso has the **reach and rendering advantages** a Tauri app lacks: a GPU-native text pipeline where caret, typewriter scroll and focus fades run at display refresh, a real EDR caret without image tricks, and Windows and Linux builds, which nobody has promised Dinkus users. Finally, Dinkus's undocumented surface (no manual, no shortcut list, unlabelled title-bar icons) is a cheap win. Labelled tooltips with shortcuts, a searchable command palette and a short manual would make enso feel as opinionated but far more learnable.

## Conclusion

Dinkus shows that a Markdown editor wins on craft, not on technology. A web view with careful spacing, a single accent, honest dialogs that explain their Markdown consequences, and a few signature writing modes earned a "no. 1 paid app" claim and near-universal "polished" verdicts in weeks ([X](https://x.com/willmcgugan/status/2094053774893662302)). The quality bar enso must hit is therefore mostly typographic and interactional. Its chance to exceed Dinkus lies in the things Dinkus's own users ran into: fidelity, defaults and platform reach.

The design pattern worth adopting is the dialog-as-explainer. Every Dinkus field says what it will write to disk, for example "Sized or aligned images save as HTML <img>; otherwise markdown". Paired with guaranteed byte-for-byte preservation, that turns the WYSIWYG anxiety Dinkus's critic voiced ("for people who shouldn't be using markdown") into trust. Two caveats remain. Every measurement here comes from compressed marketing captures. And Dinkus is shipping weekly, so features like system-appearance tracking, horizontal table scrolling or reference-link rendering could close several of these gaps before enso ships.
