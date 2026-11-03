# Dinkus product website (dinkus.textualize.io): features, behavior, pricing, platform and visuals

Scope: every non-blog page on https://dinkus.textualize.io/ as of 2026-11-03. The sitemap (https://dinkus.textualize.io/sitemap.xml) lists only: `/`, `/about/`, `/pricing/`, `/privacy/`, `/support/`, `/will-mcgugan/`, `/blog/` (+ 2 blog posts, out of scope). Probed and **404**: `/docs/`, `/help/`, `/manual/`, `/faq/`, `/changelog/`, `/release-notes/`, `/press/`, `/features/`. **The site has no manual, FAQ, changelog, press kit or features page.**

Tags used: **[site]** = stated in site text; **[img: file]** = inferred from a product image or video frame saved in `screenshots/`. Text quotes are short. All images are in `/home/user/enso/research_notes/Dinkus design and behavior reference/screenshots/`.

Source URLs:
- Home: https://dinkus.textualize.io/
- About: https://dinkus.textualize.io/about/
- Pricing: https://dinkus.textualize.io/pricing/
- Privacy: https://dinkus.textualize.io/privacy/
- Support: https://dinkus.textualize.io/support/
- Will McGugan: https://dinkus.textualize.io/will-mcgugan/
- Mac App Store: https://apps.apple.com/app/dinkus-markdown-studio/id6799480413 (App Store name: "Dinkus Markdown Studio"; not fetched)
- Site scripts: https://dinkus.textualize.io/assets/theme.js, https://dinkus.textualize.io/assets/zoom.js

---

## 1. What features are listed, and how is each one described?

### Takeaway
The home page lists seven feature sections: visual editing, source editor (one-click toggle or side by side), images (dialog with size, spacing, alignment and float), tables, syntax-highlighted code, front matter, typewriter mode with a draggable guide, and sentence or paragraph focus modes. The demo video also shows features the text never mentions: LaTeX equations rendered with KaTeX, a paragraph-style popup, a file sidebar, tabs and a bottom formatting toolbar. The site says nothing about caret settings, themes or export.

### Cited Findings
**Positioning**
- Tagline: "A distraction-free markdown studio for Mac". It is aimed at "note-takers, bloggers, site builders, and AI pioneers". You can "Edit your documents visually or in a syntax highlighted editor." [site] — [Home](https://dinkus.textualize.io/)
- About page: "a distraction-free WYSIWYG markdown studio for Mac, built by Will McGugan" [site] — [About](https://dinkus.textualize.io/about/)
- Typewriter and focus modes "come straight from Ishmael", Will's writing app for novelists. They are "designed to help you get in a flow state and stay there." The page also promises "No preview pane necessary, and no risk of broken syntax." [site] — [About](https://dinkus.textualize.io/about/) (links to https://ishmael.textualize.io/ and https://ishmael.textualize.io/manual/#c=ch03)

**Visual editing ("Post-syntax Markdown")** [site] — [Home](https://dinkus.textualize.io/)
- You open any Markdown document and edit it visually, "without a preview pane or compilation step".
- It claims support for "all of Markdown's syntax: inline styles, headers, code blocks, images, tables, footnotes, and more." Footnotes are named only in this sentence. No image shows them.

**Source editor ("Raw Markdown when you need it")** [site] — [Home](https://dinkus.textualize.io/)
- "Switch to the source view with a click or edit the source side-by-side."
- "Return to visual mode when you are done to continue editing where you left off." This implies the caret or position is kept across the switch.
- The file is always saved as standard Markdown.
- The toolbar button tooltip reads **"Edit Markdown Source ⌥⌘U"** [img: product-dinkus.mp4 at ~100 s]. In the video the same tab flips between rendered and raw syntax-highlighted source [img: product-dinkus.mp4 ~35–45 s, ~95–100 s].
- Side by side is two panes: the rendered document on the left and syntax-highlighted source on the right. Each pane has its own tab strip showing "README.md" and a split icon. In the source pane, Markdown punctuation is coloured (headings in purple, URLs in teal, code fences in orange) and inline HTML `<img ... align="right">` is kept [img: product-source-light.webp / product-source-dark.webp].

**Images ("Tune your images")** [site] — [Home](https://dinkus.textualize.io/)
- You add local images or URL images through "an intuitive image dialog", so you don't need to remember syntax or check paths.
- Dinkus writes standard Markdown image syntax where it can, and writes an `<img>` tag when extra features are needed. Example given: floating images right, "a feature not supported in plain Markdown."
- Image alt text on the site: "Dinkus displaying images inline in a markdown document, with sizing and alignment controls".
- The **image dialog** has the header "IMAGE" and a close ×. It contains [img: product-images-light.webp, product-frame-dinkus-15.jpg]:
  - A preview in a dashed box, with the natural size below it ("1280 × 640 px").
  - A path/URL field plus a **Browse…** button. Help text: "A path relative to the document, a /path from the opened folder's root, or an http(s) URL. Browse fills in the relative path."
  - An alt-text field. Help text: "Read aloud by screen readers and shown if the image can't load — the ![alt] in the markdown."
  - **Size** as width × height (placeholder "auto"). Help text: "pixels or a percentage (40%); empty for natural size." The video shows "50%" typed in.
  - **Spacing** as h × v steppers. Help text: "Margins around the image (hspace × vspace) — how GitHub spaces floated images."
  - **Align**: four icon buttons, apparently none/block, left float, centre and right float. Hover tooltip: "Float right — text wraps on the left". Help text: "Left / right float the image and wrap the text around it, as on GitHub."
  - **Inline** toggle. Help text: "Flows with the text, like a badge. Sized or aligned images save as HTML <img>; otherwise markdown."
  - Footer buttons: red-outlined **Delete Image** on the left; **Cancel** and purple **Apply** on the right.
- A selected image in the document gets a purple outline and two small purple buttons at its top right: a magnifier (likely zoom or view) and a pencil (edit, which opens the dialog) [img: product-frame-dinkus-0.jpg; v3a frames ~5 s, ~135 s].
- An image dragged from the sidebar file tree into the document is inserted as `![calculator.png](imgs/calculator.png)` [img: product-dinkus.mp4 ~65–80 s; source view at ~95 s].
- A right-floated image (the clock screenshot) has body text wrapping to its left in visual mode [img: product-wysiwyg-light.webp].

**Tables ("Easy edit tables")** [site] — [Home](https://dinkus.textualize.io/)
- You add or remove columns and rows and edit cells visually, and cells can hold images and inline syntax. You can also drop into source mode.
- "Dinkus will even format and extend the column by adding spaces to keep your table source nice and tidy." In other words, it auto-pads the pipe-table source. The source view in the video shows aligned pipes after typing "Where there is a Will" [img: product-dinkus.mp4 ~95 s].
- In visual mode the selected column (Paradigm) is tinted lavender. Two small purple pill buttons above the column, "‹+" and "+›", insert a column left or right. The header row has a grey background and cells have thin borders [img: product-tables-light.webp].
- A **Table** insert dialog has: Size as columns × rows steppers (default 3 × 3, "header row included"); a **Header row** toggle, on by default, with a note that Markdown tables always carry a header line and that without one the table saves with empty header cells; and Cancel / OK buttons [img: product-dinkus.mp4 ~85 s; low resolution, so the wording is approximate].

**Code blocks ("Work with code")** [site] — [Home](https://dinkus.textualize.io/)
- Syntax-highlighted code blocks support "common languages—and many uncommon ones". The use cases given are blogging, docs and "sharing with an agent".
- Code blocks render on a light grey panel in a monospace font. The language label ("python") sits small at top right, and a purple pencil edit button floats at the top right of the block. The caret is shown inside the code [img: product-code-light.webp]. You can type directly into the rendered code block [img: product-dinkus.mp4 ~20–25 s, "Dinkus is awesome"].

**Front matter ("Edit your meta data")** [site] — [Home](https://dinkus.textualize.io/)
- Dinkus "can read, write, and edit all the formats in common use today". **YAML, TOML and JSON are not named in the text.** Only the alt text says "YAML front matter in a dedicated panel above the rendered page". Front matter is syntax highlighted.
- The front matter sits in a full-width band above the page card, outside the white page. It has a `{}` icon at the left and monospaced, coloured `key: value` lines (layout, title, categories, `hello: "World"`) with the caret in it. Below it, the page shows a Jekyll blog post [img: product-frontmatter-light.webp].

**Typewriter mode ("Keep your eye on the line")** [site] — [Home](https://dinkus.textualize.io/)
- It "keeps the caret anchored on a single horizontal line by moving the document vertically rather than the caret itself."
- The stated benefits are less cognitive burden, a fixed amount of context around the caret, and no scrolling needed to see what you just wrote.
- "In Dinkus you can drag the insertion guide away from the center of the screen", which gives more context above the caret while leaving a few lines below.
- The guide is a thin dotted horizontal line that runs the full window width. The caret's text line sits on it while text grows above [img: product-frame-typewriter-light-1.jpg, -30.jpg]. Video aria-label: "the active line held at the guide while the document scrolls beneath it". The dashed guide line also shows in the normal screenshots at about 40–50% of window height [img: product-wysiwyg-light.webp; product-frame-dinkus-*].
- The typewriter and focus clips use a **serif** body and heading font, unlike the sans (Inter-like) font in the other shots. This suggests a font or theme option, but the site never mentions one [img: product-frame-typewriter-light-*.jpg, product-frame-focus-light-*.jpg].

**Focus modes ("One sentence at a time")** [site] — [Home](https://dinkus.textualize.io/)
- Dinkus "can highlight the sentence or paragraph you are working on and dim the rest of the document". That makes two granularities: sentence and paragraph.
- It is pitched as pairing with typewriter mode. Video aria-label: "focus mode highlighting the active sentence and gently dimming the rest".
- In the frame, the active paragraph is dark text and the other paragraphs are light grey, with no blur. The caret is a purple bar [img: product-frame-focus-light-6.jpg].

**Other features visible only in images or video (not in the site text)**
- **Equations**: an "EQUATION" dialog with a LaTeX textarea and a live Preview. Footer text: "Rendered with KaTeX. Emptying it removes the equation." Buttons are Cancel and Apply. The equation then renders centred as display math [img: product-dinkus.mp4 ~110–130 s; product-frame-dinkus-120.jpg]. The bottom toolbar has √ and boxed-√ buttons, probably inline and block math [img: product-wysiwyg-light.webp].
- **Paragraph style popup**: a small list titled "PARAGRAPH STYLE" pops up at the caret's left. Options: blockquote, bullet list, code block, heading 1–6, numbered list, paragraph. The current item is highlighted purple, and it appears to filter as you type ("2" leads to heading 2) [img: product-frame-dinkus-90.jpg; ~105 s frame].
- **Window chrome**:
  - Left sidebar with a folder file tree. The header shows the folder name ("textual") and new-file, new-folder and refresh icons. Non-Markdown files are dimmed. A new file is named inline in the tree.
  - Title bar: sidebar toggle and outline/list icon on the left, centred filename with "— Edited" when dirty.
  - Right-side title-bar icons: eye (preview/read?), briefcase-like, a bracketed doc, a "→|←" icon (likely typewriter or focus), and a doc-search icon. Their meanings are not labelled.
  - Tabs with an × or ● dirty dot, a purple underline on the active tab, and a split-view icon at the far right [img: product-wysiwyg-light.webp; toolbars crop].
- **Bottom formatting toolbar**, left to right: Markdown-source toggle (M↓) | ¶ paragraph style | Bold, Italic, Strikethrough, inline code `<>`, a keyboard/kbd-like box, Link, a footnote(?) asterisk | Image, horizontal rule/align(?), insert list (with +), a disabled list item, code block, table, HTML `</>` | inline math √, block math. Active toggles are tinted purple. **Most icon meanings are inferred** [img: product-wysiwyg-light.webp].
- The macOS **menu bar** in the recording reads: **Dinkus, File, Edit, Insert, Go, View, Window, Help, Developer** [img: product-dinkus.mp4 frames]. Menu contents are never shown.
- **Document layout**: the page is a bordered card centred in a wider grey canvas, with a readable column of about 60–70% of the pane width. Headings are underlined with a thin rule, as in GitHub style. The caret is a thick **purple** bar [img: product-wysiwyg-light.webp, product-code-light.webp].

### Inferences
- Rendering conventions (GitHub-like `align`, `hspace`/`vspace` and badges inline) suggest Dinkus targets GitHub-flavoured README rendering.
- "Continue editing where you left off" suggests position mapping between the visual and source views.

### Gaps
- **Caret settings** (colour, thickness, blink, blink pattern): **not mentioned on any non-blog page.** The caret is visibly purple and thick in images. (The blog post "hdr-caret" in the sitemap probably covers this and belongs to the other researcher.)
- **Themes**: no theme list. Light and dark app appearance are both shown (each screenshot exists in light and dark), and there is an unexplained serif variant.
- **Export** (HTML, PDF and so on): not mentioned anywhere.
- **Footnote UI**: named but not shown or described.
- **Front matter formats**: the text says "all the formats in common use today". It does not name YAML, TOML or JSON; only YAML is shown.
- Settings or preferences windows are never shown or named.

---

## 2. What keyboard shortcuts, menus, settings pages or preferences are named?

### Takeaway
The site text names **none**. One shortcut, ⌥⌘U for "Edit Markdown Source", is readable in a video tooltip. The menu bar names come from the video.

### Cited Findings
- No shortcut, menu or preference is named anywhere in the text of the six pages [site] — [Home](https://dinkus.textualize.io/), [About](https://dinkus.textualize.io/about/), [Pricing](https://dinkus.textualize.io/pricing/), [Privacy](https://dinkus.textualize.io/privacy/), [Support](https://dinkus.textualize.io/support/)
- Tooltip "Edit Markdown Source ⌥⌘U" on the M↓ toolbar button [img: product-dinkus.mp4 ~100 s, bottom-left] — https://dinkus.textualize.io/assets/img/dinkus.mp4
- Menu bar: Dinkus / File / Edit / Insert / Go / View / Window / Help / **Developer** [img: product-dinkus.mp4] — https://dinkus.textualize.io/assets/img/dinkus.mp4
- Spelling and grammar use the macOS system service [site] — [Privacy](https://dinkus.textualize.io/privacy/)

### Inferences
- The "Insert" menu probably holds the image, table and equation dialogs, and the "Go" menu probably holds navigation. Both are unverified.

### Gaps
- There is no shortcut list, no documentation of the typewriter or focus toggles, and no settings screenshots.

---

## 3. UI hiding, performance, file handling, privacy, offline, pricing, trial, platform and distribution

### Takeaway
The app costs $9.99 as a one-time Mac App Store purchase, with no subscription and no account. Files are plain .md on disk. The app collects nothing, makes no network calls except fetching remote images a document references, and works offline. It requires an Apple Silicon Mac running macOS 12 Monterey or later. Nothing is said about a trial, chrome hiding or performance.

### Cited Findings
- "plain .md on disk · no account · works offline" [site] — [Home](https://dinkus.textualize.io/)
- "Requires an Apple Silicon Mac, macOS 12 Monterey or later" [site] — [Home](https://dinkus.textualize.io/)
- **$9.99**, "One-time purchase · no subscription", no account. Documents stay as plain Markdown, and the app "has no analytics and never phones home" [site] — [Pricing](https://dinkus.textualize.io/pricing/)
- The only distribution is the Mac App Store badge linking to https://apps.apple.com/app/dinkus-markdown-studio/id6799480413. There is no direct download [site] — [Home](https://dinkus.textualize.io/), [Pricing](https://dinkus.textualize.io/pricing/)
- Privacy policy, last updated 11 August 2026 [site] — [Privacy](https://dinkus.textualize.io/privacy/):
  - It is "local-first", with no account, sign-in, analytics or telemetry.
  - Its only network request fetches http/https images a document references, directly from the image host.
  - It does not check for updates or contact any Textualize server.
  - Spelling uses the macOS service.
  - Purchases and opt-in crash reports go through Apple.
  - The website runs on GitHub Pages with no cookies or analytics.
  - The support form opens your own mail app.
- Support: contact form (Name, Your email, Subject, Message, "Send email") or will@textualize.io, plus a Discord server (https://discord.gg/rx3bxd8ZRw) [site] — [Support](https://dinkus.textualize.io/support/)
- Developer: Will McGugan, "more than three decades of experience", creator of Rich, Textual and Toad. His open source projects are downloaded "over a billion times every month" [site] — [Will McGugan](https://dinkus.textualize.io/will-mcgugan/)
- Name origin: a dinkus is the three-asterisk (***) section-break mark [site] — [About](https://dinkus.textualize.io/about/). The app icon is three glossy asterisks in lavender, peach and teal [img: product-dinkus.webp].
- Chrome hiding: the site says only "distraction-free editing experience". It describes no full-screen mode or auto-hiding toolbar [site] — [Home](https://dinkus.textualize.io/). In the typewriter and focus clips, no sidebar or title bar is visible, only the page and the bottom toolbar. This may be a crop or a hidden-chrome mode [img: product-frame-typewriter-light-1.jpg, product-frame-focus-light-6.jpg].

### Gaps
- **Trial**: none mentioned. App Store pricing implies no trial, but this is unverified.
- **Performance**: no claims are made.
- No statement on iCloud, sync or autosave behaviour, or on Intel Macs beyond the Apple Silicon requirement.

---

## 4. Videos, GIFs and product screenshots on the site

### Takeaway
The home page uses one hero screen recording inside a MacBook frame, six feature screenshots and two feature videos. Every asset has light and dark variants. Oddly, the site shows the **dark-app** variant when the site is in light theme, and the reverse. Clicking a screenshot zooms it to the viewport centre with a blurred background, which is disabled below 48rem and honours reduced motion.

### Cited Findings
How the assets are wired:
- Each asset uses `data-src-on-light` and `data-src-on-dark` attributes, so the light site loads `*-dark` assets and vice versa. Off-screen videos are lazy-loaded through IntersectionObserver.
- Phones at 48rem or narrower get `dinkus-mobile.mp4`, a 30fps lower-resolution clip.
- A floating theme toggle is stored in localStorage key `dinkus-site-theme`.
— [theme.js](https://dinkus.textualize.io/assets/theme.js), [Home HTML](https://dinkus.textualize.io/)
- Click-to-zoom uses a FLIP animation over 500 ms with easing `cubic-bezier(0.2,0,0.05,1)` and a backdrop-blur shield. Escape or a click closes it — [zoom.js](https://dinkus.textualize.io/assets/zoom.js)

| Asset (URL under https://dinkus.textualize.io/assets/img/) | Saved as | What it shows |
|---|---|---|
| dinkus.mp4 (3324×2160, 60fps, 140 s); dinkus-mobile.mp4 | product-dinkus.mp4, product-dinkus-mobile.mp4, frames product-frame-dinkus-*.jpg | Full-screen dark-mode recording inside macbook-frame.webp. It starts with an empty window showing the asterisk logo, then opens Textual's README. It edits the image dialog (50%, float right), types into a code block, toggles source view, creates test.md in the sidebar, types headings, uses the paragraph-style popup, drags an image from the tree, inserts a 3×3 table and types in it, checks the tidy source, and inserts a KaTeX equation. It ends typing "And much more. Download". |
| wysiwyg-light/dark.webp (2624×2024) | product-wysiwyg-*.webp | Visual mode on Textual README.md, with the sidebar tree, tab, and badges row. The hero image is selected, with the dashed typewriter guide below it. The H1 "Textual" has the purple caret. A right-floated clock image has wrapped text, and a code block is visible. The bottom toolbar is shown. |
| source-light/dark.webp | product-source-*.webp | Side-by-side: rendered on the left, highlighted Markdown source on the right, each with its own tab. The sidebar is hidden. |
| images-light/dark.webp | product-images-*.webp | Image dialog modal over a dimmed document (fields listed in section 1). |
| tables-light/dark.webp | product-tables-*.webp | "Tables" H1 with a 5-column language table, the Paradigm column highlighted, and ‹+ / +› column buttons. The tab "Untitled ●" shows "— Edited". |
| code-light/dark.webp | product-code-*.webp | Blog post .md with a highlighted Python block, "python" label and pencil edit button. |
| frontmatter-light/dark.webp | product-frontmatter-*.webp | Jekyll site folder (willmcgugan.github.io) open. A YAML front matter band sits above the page, and a raw `<iframe>` HTML block is shown highlighted inside the page. |
| typewriter-light/dark.mp4 (1792×1264, 47 s / 60 s) | product-typewriter-*.mp4, product-frame-typewriter-*.jpg | Serif document. Text is typed while its line stays on the dotted guide and earlier text scrolls up. |
| focus-light/dark.mp4 (12 s / 10 s) | product-focus-*.mp4, product-frame-focus-*.jpg | Serif novel text ("Chapter 1."). The active paragraph is dark and the others dimmed grey. |
| macbook-frame.webp, dinkus.webp | product-macbook-frame.webp, product-dinkus.webp | Laptop bezel overlay; app icon / logo (three asterisks). |

Colour notes:
- Light app: canvas about #E8E9EF (lavender grey), a slightly whiter page card with a thin border, dark slate text, and purple accent (#7C4DFF-ish) for the caret, active tab underline, Apply button and selection.
- Dark app: navy/indigo canvas (about #23253A) with a lighter lavender accent.
- These colours are eyeballed, not exact [img: product-wysiwyg-*.webp].

Site screenshots taken (1440 px wide, full page, Playwright Chromium):
- site-{home,about,pricing,privacy,support,will-mcgugan}-{light,dark}.png
- site-home-mobile-light.png (390 px, 2× scale)

### Gaps
- There are no GIFs, no YouTube embeds and no separate poster images. The home "showcase" has no textual captions beyond the alt text and aria-labels quoted above.
- Toolbar icon meanings are not labelled in any visible tooltip except ⌥⌘U.
