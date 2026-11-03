# Dinkus: third-party coverage, videos and community reception (as of 3 Nov 2026)

Scope: what people other than the product site have shown or said about Dinkus (Will McGugan / Textualize, Mac). Overall coverage is **thin**. Dinkus launched on 19 to 23 Aug 2026 and is about 2.5 months old. I found no YouTube or Vimeo review, no Hacker News discussion (one 2-point link with 0 comments), no Product Hunt page, and no review from a major publication. The material that does exist is: the Mac App Store listing (4 reviews), one Tao of Mac blog post, one Python Bytes podcast mention, the developer's own X and Mastodon posts (with screenshots and two short screen recordings), and Mastodon reply threads from users. Reddit search was blocked from this machine (HTTP 403), and web search turned up no Reddit threads.

Screenshots and media downloaded to `screenshots/` with the prefix `third-`. They are listed at the end of section 1.

## 1. What videos exist, and what do they show?

### Takeaway
No independent video review or tutorial exists. YouTube search for "dinkus markdown" returns only generic markdown-editor videos. The only screen recordings of Dinkus in use outside the product site are short silent clips from the developer on Mastodon, plus the demo embedded in his HDR-caret blog post. I could not watch the videos. I extracted frames with ffmpeg and inspected those contact sheets, so what follows describes the frames, not the motion between them.

### Cited Findings
- **YouTube search "dinkus markdown" (checked 3 Nov 2026):** none of the top 15 results is about Dinkus. They are generic videos such as "Best Markdown Editors for MacOS (2025)" and "Typora vs MarkText vs Obsidian". — [YouTube search](https://www.youtube.com/results?search_query=dinkus+markdown)
- **Mastodon video, "By popular demand, the next version of Dinkus will have draggable dividers"** (30 Sep 2026; 71.4 s; 3324×2160 at 60 fps; silent). — [Mastodon post](https://mastodon.social/@willmcgugan/117360286159032802). The frame contact sheet (one frame every 6 s) is `third-contactsheet-draggable-dividers-video.png`, and the video is `third-mastodon-2026-09-30-draggable-dividers.mp4` (re-encoded at 1280 px). Step by step, from the frames:
  1. The window opens with a folder tree on the left (dark navy theme with a lavender accent). The empty editor shows the dinkus logo ⁂ in pastel colours and a hint, "⌘P for quick open". The sidebar divider is dragged, and a thin lavender vertical line with a ↔ cursor marks it.
  2. The sidebar switches to its search tab, and the user types "readme". Results fall into two groups, "TITLES" and "IN DOCUMENTS", each showing a file path in muted text and the match highlighted in lavender.
  3. Documents open in the WYSIWYG view: a serif body font, inline images floated right, inline code chips, code blocks on a slightly lighter panel, and #hashtags shown as rounded pills (#foo #bar).
  4. The editor splits vertically and then horizontally. The divider between panes is dragged and shows a lavender line with a ↕ handle. Each pane has its own tab strip.
  5. The sidebar collapses. A quick-open palette (⌘P) appears as a centred floating list, and typing "wid" filters files.
  6. One frame shows a zoomed-in close-up of the bottom formatting toolbar: a WYSIWYG/source toggle, a "heading 1" style dropdown, then B I S U code, link, list, image and other icons. Pane dividers are dragged again.
- **Mastodon video, a reply to a user asking about Typora-style table editing** (1 Oct 2026; 38.8 s; 2828×1786 at 60 fps). — [Mastodon reply](https://mastodon.social/@willmcgugan/117366546507679801). Contact sheet: `third-contactsheet-table-editing-video.png`. From the frames:
  1. A "TABLE" modal sets the size (3 × 3, "Columns × rows, header row included") and has a "Header row" toggle with help text: "Markdown tables always carry a header line; without one it saves with empty header cells". Cancel and OK buttons sit at the bottom right, with OK in lavender.
  2. A grid table is inserted under the "Tables" H1, and a small lavender pencil (edit) badge floats at the table's top right. The user types into the cells, and columns widen automatically as text grows. A transient "Paragraph" pill appears near the bottom of the page, apparently showing the current block type.
  3. The user toggles to source view. The table appears as an aligned pipe table in monospace, and the H1 shows as `# Tables` in an accent colour. The user edits the raw text and toggles back. The visual table now shows the third column "Dinkus supports visual editing of tables", with the header underlined by a red spell-check squiggle. The source/WYSIWYG toggle is the first control on the bottom toolbar.
  4. The document sits on a centred "page" card that is slightly lighter than the window background, with wide margins. Window chrome is minimal: tabs, and Edit/Read segmented buttons at the top centre.
- **User-posted GIF:** Rui Carmo replied with a "shut up and take my money" GIF (23 Aug 2026). It is not a demo. — [Mastodon](https://mastodon.social/@rcarmo/117146040278942551)
- **HDR caret blog demo** (26 Aug 2026). It shows an HDR caret in one window beside a standard SDR caret in another. The author notes that the caret looks less bright in the video than on a real desktop. — [Dinkus blog](https://dinkus.textualize.io/blog/hdr-caret/). It was the only Dinkus item submitted to Hacker News (2 points, 0 comments). — [HN item 49450680](https://news.ycombinator.com/item?id=49450680)
- **X/Twitter posts carry still images only, no video.** The launch post (23 Aug 2026; 136 likes) has 4 screenshots with alt text "Screenshot of Dinkus Markdown Studio #1–3". The discount post (30 Aug 2026; 58 likes) has 4 images, and the 1.8.0 post (24 Sep 2026; 44 likes) has 4 images. — [X launch](https://x.com/willmcgugan/status/2091528888471941223), [X discount](https://x.com/willmcgugan/status/2094053774893662302), [X 1.8.0](https://x.com/willmcgugan/status/2103121091979792837)
- The 1.8.0 X image (`third-x-2026-09-24-v18-1.jpg`) shows the ⌘P quick-find palette searching "markdown". Results are split into TITLES and IN DOCUMENTS with highlighted context snippets. The window also shows the left sidebar tabs (folder / search / # tags / outline), the folder tree, New File and New Folder buttons, the Edit/Read toggle at the top centre, and the bottom toolbar. — [X 1.8.0](https://x.com/willmcgugan/status/2103121091979792837)
- None of the third-party material shows typewriter scrolling or focus dimming in motion. Only the official site videos (covered by other researchers) do.

### Inferences
- Visual cues worth matching: a single lavender accent used for dividers, selection, palette highlights and primary buttons; muted path text in search results; floating badges such as the table pencil; a transient block-type pill; and a document "page" card with generous margins. The developer's own clips use 60 fps capture, and the interactions they show are simple (direct manipulation, no elaborate transitions).

### Gaps
- No captions or transcripts exist (the clips are silent and there are no YouTube videos).
- I could not judge animation smoothness or timing from still frames.
- Videos attached to X posts, if any, were not retrievable (the syndication API showed photos only).

### Downloaded media (screenshots/third-*)
- `third-contactsheet-draggable-dividers-video.png`, `third-mastodon-2026-09-30-draggable-dividers.mp4`: described above.
- `third-contactsheet-table-editing-video.png`, `third-mastodon-2026-10-01-table-editing-reply.mp4`: described above.
- `third-x-2026-09-24-v18-1.jpg`, `-2.jpg`: the 1.8.0 promo images (quick find, outline and tags).
- `third-mastodon-2026-10-01-v19-1.png`, `-2.png`: the 1.9.0 promo images (print, Word/PDF export, dividers). — [Mastodon](https://mastodon.social/@willmcgugan/117366157515732153)
- `third-mastodon-2026-10-02-tree-icons.png`: the new file-type icon set in the tree view ("WDYT?"). — [Mastodon](https://mastodon.social/@willmcgugan/117372463237938113)
- `third-mastodon-2026-10-31-html-export.png`: a screenshot of the single-file HTML export, which includes images and a search index. — [Mastodon](https://mastodon.social/@willmcgugan/117537450620070510)
- `third-rcarmo-2026-08-23-h1-reflinks.png`: a user screenshot showing an unresolved reference link `[ultrawide]: blog/2021/08/21/1600` rendered as body text. It also shows the serif body text, a dashed horizontal rule, a red wavy spell-check underline, and a thick cyan/teal caret. — [Mastodon](https://mastodon.social/@rcarmo/117144571824832015)
- `third-rcarmo-2026-08-23-svg-sizing-1.png`: a user's tiny crop showing a dropped SVG sized wrongly. — [Mastodon](https://mastodon.social/@rcarmo/117144686461543691)

## 2. Mac App Store listing

### Takeaway
The app is listed as "Dinkus: Markdown Studio". It costs $9.99, is 11.3 MB, requires macOS 12+ on Apple silicon, and sits in Developer Tools. It shipped 12 versions between Aug 19 and about Nov 1, 2026. Only 4 written reviews exist, too few for Apple to display a rating.

### Cited Findings
All facts in this section are from the [App Store listing](https://apps.apple.com/us/app/dinkus-markdown-studio/id6799480413?mt=12), fetched 3 Nov 2026, unless another source is given.
- **Listing facts:** price $9.99 with no in-app purchases listed; version 1.11.0; 11.3 MB; macOS 12.0 or later and an Apple M1 chip or later; category Developer Tools; rated 4+. Seller is William Warren McGugan. English only. No data collected. Family Sharing is supported. The listing says the app "hasn't received enough ratings or reviews to display an overview."
- **Price history:** the hosts of Python Bytes (1 Sep 2026) described "about $9" as a sale price. — [Python Bytes #494](https://pythonbytes.fm/episodes/show/494/python-wrapture). McGugan said on 30 Aug 2026 that it was "no.1 paid app on Mac App Store" and discounted for the rest of the month. — [X](https://x.com/willmcgugan/status/2094053774893662302). A web search snippet attributes a "No. 1 in developer tools" claim to LinkedIn, which I did not verify. The ranking claim is the developer's own.
- **Version history:**
  - 1.0 (Aug 19)
  - 1.1.0 (Aug 20): new UI design and folder view; equations editor
  - 1.2.0 (Aug 21): fixes before the launch announcement
  - 1.3.0 (Aug 28): welcome screen; restores open files; opens other markdown file extensions; fixes for Asian languages
  - 1.4.0 (Aug 29): visual enhancements
  - 1.5.0 (Sep 6): collapsible headings, detail slider, tap-to-select
  - 1.6.0 (Sep 16): Mermaid; new welcome view
  - 1.7.0 (Sep 20): Pelican front matter; checklists; styles indicator in the footer
  - 1.8.0 (Sep 24): Read mode; full-text search; quick find ⌘P; tags; vertical splits (shift-click); outline
  - 1.9.0 (Oct 1): draggable dividers; printing; export to SVG and Word
  - 1.10.0 (Oct 9): folder icon set; better grammar-checker support
  - 1.11.0 (~Nov 1): Dinkus Light and Dinkus Dark themes; single-file HTML export
- **Screenshot captions:** none. The listing shows screenshots without caption text.
- **Reviews (all 4, each with a developer reply):**
  - "Excellent!" by Haemish (Sep 8, positive). Of many markdown editors tried, the reviewer found Dinkus "smooth, intuitive, and reliable", with good handling of unusual markdown variants and images. Grammarly support had problems, and the developer said he was looking into it.
  - "Weird and Frustrating Design" by _Andy_S_ (Sep 15, negative). The reviewer thinks a markdown editor should open files as plain text with text and preview side by side, dislikes typewriter mode being on by default, and says the app seems to be "for people who shouldn't be using markdown". The developer replied that Dinkus is a visual editor so a split preview is unnecessary, that a source-view default could become an option, that typewriter mode is core to his own workflow, and that the next version will remember the ⌘T toggle.
  - "opinionated and detail oriented…" by Antisimplistic (Sep 20, positive). Praises the attention to detail and the fresh, opinionated approach.
  - "Native system dark mode" by clipwrap (~Oct 31). Asks for the app to follow the system dark mode. The developer pointed to the new Dinkus Dark theme, which is close to but not identical to the system theme.

### Inferences
- The release cadence was very fast, roughly weekly. Several releases were "visual enhancements" and UI redesigns, which suggests polish was iterated publicly after launch.
- A default of typewriter mode on, without remembering the toggle, generated friction in two independent sources (an App Store review and Python Bytes).

### Gaps
- I could not see numeric star ratings, other storefronts, or the rank history.

## 3. What do users praise or criticize?

### Takeaway
The praise centres on polish, snappiness, detail, the visual handling of tables and images, and looking "not like a code editor". The criticisms are about defaults (typewriter mode always on), markdown fidelity edge cases (reference links and footnotes, SVG sizing, H1 splitting), integrations (Grammarly, system dark mode) and missing power features (Obsidian-style quick actions and file switching, inline HTML in tables, horizontal table scrolling, iPad). Several of these were addressed in later versions.

### Cited Findings
**Praise (opinions):**
- Rui Carmo (Tao of Mac, 23 Aug 2026): the experience is "great". He calls it snappy and polished, says it "doesn't look like a code editor", and calls it an instant buy that he paid for. — [Tao of Mac](https://taoofmac.com/space/links/2026/08/23/1121)
- Carmo again: "I've been writing on it all day now… You need to do an iPad version", and "love it despite the little weirdness around reference links and footnotes." — [Mastodon](https://mastodon.social/@rcarmo/117146040278942551), [Mastodon](https://mastodon.social/@rcarmo/117146080204963255)
- Python Bytes (1 Sep 2026): Calvin praised the handling of tables and images, which he says is usually weak in markdown tools, along with the code syntax highlighting, no account, and offline use. — [Python Bytes #494](https://pythonbytes.fm/episodes/show/494/python-wrapture)
- samueljohn, a Typora power user (2 Oct 2026): "Just bought your app, looks so sleek!" — [Mastodon](https://mastodon.world/@samueljohn/117367955621011173)
- allorimd (31 Aug 2026): "It was an instant-buy for me… Combine Dinkus + Ishmael, and you're 3/4 of the way" to a Scrivener replacement. — [Mastodon](https://sciences.social/@allorimd/117190403032302658)
- njr praised the single-file HTML export: "A single mailable/otherwise sharable file is so much better than a dir or zip." — [Mastodon](https://mathstodon.xyz/@njr/117537793726748499)
- App Store reviews: "smooth, intuitive, and reliable" (Haemish) and "opinionated and detail oriented" (Antisimplistic). — [App Store](https://apps.apple.com/us/app/dinkus-markdown-studio/id6799480413?mt=12)

**Criticisms and bug reports:**
- Reference links and footnotes at the bottom of a document are ignored in the visual view, although source view recognises them. Carmo calls this "the only notable omission". — [Tao of Mac](https://taoofmac.com/space/links/2026/08/23/1121). Carmo also suggested the UI label them "reference style" instead of "footnotes". — [Mastodon](https://mastodon.social/@rcarmo/117156803967556787)
- Pressing Enter at the end of an H1 creates another H1, which "feels shouty"; Carmo expected an H1 followed by a paragraph. — [Mastodon](https://mastodon.social/@rcarmo/117144571824832015)
- Deleting a folder did not update the workspace tree. The developer could not reproduce it and suspected a race condition. — [Mastodon](https://mastodon.social/@rcarmo/117144484411112167)
- A dropped SVG was detected but its "auto sizing bit has gone wonky". — [Mastodon](https://mastodon.social/@rcarmo/117144686461543691)
- Users asked for Obsidian-style quick actions and typeahead file switching. — [Mastodon](https://mastodon.social/@rcarmo/117144484411112167), [Tao of Mac](https://taoofmac.com/space/links/2026/08/23/1121). Fact: quick find ⌘P shipped in 1.8.0 on Sep 24. — [App Store](https://apps.apple.com/us/app/dinkus-markdown-studio/id6799480413?mt=12)
- Typewriter mode is "sort of a non-standard" feature, cannot be turned off by default, has to be turned off per file, and has no hotkey (Python Bytes, 1 Sep 2026). — [Python Bytes #494](https://pythonbytes.fm/episodes/show/494/python-wrapture). The developer's App Store reply says ⌘T exists and that the next version remembers the setting. — [App Store](https://apps.apple.com/us/app/dinkus-markdown-studio/id6799480413?mt=12)
- Users asked for draggable dividers "by popular demand", which shipped in 1.9.0. — [Mastodon](https://mastodon.social/@willmcgugan/117360286159032802)
- samueljohn asked for inline HTML inside table cells (colour spans, underline, comments, `<img>`), horizontal scrolling for wide tables, and an iPad version. The developer pointed to an HTML-block button in the footer and said horizontal scrolling and iPad "might happen". — [Mastodon thread](https://mastodon.world/@samueljohn/117371422129210786)
- Integration issues: Grammarly had problems, and 1.10.0 notes "improved grammar checker support". The app does not follow the system dark mode exactly. — [App Store](https://apps.apple.com/us/app/dinkus-markdown-studio/id6799480413?mt=12)
- The name divided opinion: "I don't love 'Dinkus'" (njr), "sounds a bit too 'meme'y" (Kroc). — [Mastodon](https://mathstodon.xyz/@njr/117144460555337872), [Mastodon](https://oldbytes.space/@Kroc/117150081156393169)
- An AI ethics objection: the developer confirmed that Dinkus "was built with the help of AI. Claude mostly." One Mastodon user objected on ethical grounds, and the exchange turned hostile. — [Mastodon](https://mastodon.social/@willmcgugan/117150365436495360)
- masukomi asked for a write-up on why another markdown editor was worth building. I found no such write-up. — [Mastodon](https://connectified.com/@masukomi/117371288418555101)

**Implementation facts from the developer (relevant to smoothness):**
- Dinkus and Ishmael are built on **Tauri** (a web view plus a native shell), not Textual. Both were born of frustration with Microsoft Word. — [Mastodon](https://mastodon.social/@willmcgugan/117190614437691468)
- The typewriter and focus modes come from his novel-writing app, Ishmael. — [Dinkus About](https://dinkus.textualize.io/about/)
- The HDR caret clips a small HDR image to the caret's shape, because CSS HDR support is limited. Caret colour, thickness, blink and blink pattern are configurable. — [HDR caret blog](https://dinkus.textualize.io/blog/hdr-caret/)

### Inferences
- Users perceived a Tauri/web-view app as "snappy" and "native", so perceived speed came more from polish and responsiveness than from the technology stack.
- Most complaints concern edge-case markdown fidelity and defaults, not core typing feel. I found no complaint about lag, jank or typing latency in any source.

### Gaps
- Reddit was inaccessible (403) and web search surfaced no Reddit threads. I found no Bluesky results (the search API returned an error) and no Product Hunt page. X reply threads were not retrievable.

## 4. How do reviewers compare it with iA Writer, Typora, Obsidian and others?

### Takeaway
No reviewer has published a structured comparison. The only comparisons are passing ones: against Obsidian (missing quick actions and file switching), Typora (the benchmark for WYSIWYG table editing, which Dinkus turned out to match), and Scrivener (the aspiration, paired with Ishmael). I found no source that compares Dinkus with iA Writer, Bear or Ulysses.

### Cited Findings
- **Obsidian:** Carmo misses Quick Actions and typeahead find for switching files, and otherwise prefers Dinkus's polish. — [Tao of Mac](https://taoofmac.com/space/links/2026/08/23/1121)
- **Typora:** samueljohn named Typora "the leading example" of visual table editing. After seeing Dinkus's table demo he replied "what, you can edit tables wysiwyg? I have to try it!" He listed Typora's remaining advantages: inline HTML, raw `<table>` as a fallback, HTML comments, and horizontal scrolling. — [Mastodon thread](https://mastodon.world/@samueljohn/117366509161286321)
- **Scrivener:** allorimd sees Dinkus plus Ishmael as most of the way to a markdown-based Scrivener, missing only Pandoc/LaTeX/Quarto build steps. — [Mastodon](https://sciences.social/@allorimd/117190403032302658)
- **Split-preview editors in general:** the App Store critic wants a source-plus-preview layout, while the developer's position is that WYSIWYG removes the need for a preview pane. — [App Store](https://apps.apple.com/us/app/dinkus-markdown-studio/id6799480413?mt=12)
- **iA Writer, Bear, Ulysses:** I found no source.

### Inferences
- Dinkus is positioned (and perceived) closest to Typora: WYSIWYG markdown with raw source a keypress away. Its typewriter and focus modes add iA Writer-style focus features. Users judge it on markdown fidelity like Typora users and on calm writing like iA Writer users.

### Gaps
- No head-to-head reviews exist on writing feel or visual design. A later search (2027 onward) may find more as coverage grows.
