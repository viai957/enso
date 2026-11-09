use std::ops::Range;
use std::path::Path;

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, Sizable as _,
    button::{Button, ButtonVariants as _},
    input::{Input, InputEvent, InputState},
    switch::Switch,
    tooltip::Tooltip,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::document::Kind;
use super::image::{ImageAlign, ImageBlock, Length, inline_images};
use super::toolbar;
use super::view::{BODY_SIZE, VisualView};
use crate::tokens;
use gpui_kit::assets::IconName;

const MIN_WIDTH: f32 = tokens::MIN_IMAGE_WIDTH;
const MAX_SPACING: f64 = 200.;

#[derive(Clone, Copy, Debug)]
pub struct ImageResize {
    pub block: u64,
    pub start_x: Pixels,
    pub start_width: Pixels,
    pub width: Pixels,
    pub centered: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImageTarget {
    Insert(usize),
    Replace(u64),
    Inline(Range<usize>),
}

pub struct ImageDialog {
    pub target: ImageTarget,
    pub src: Entity<InputState>,
    pub alt: Entity<InputState>,
    pub width: Entity<InputState>,
    pub height: Entity<InputState>,
    pub hspace: Entity<InputState>,
    pub vspace: Entity<InputState>,
    pub align: ImageAlign,
    pub inline: bool,
    pub title: Option<String>,
    pub error: Option<String>,
    _subscriptions: Vec<Subscription>,
}

fn length_field(
    input: &Entity<InputState>,
    name: &str,
    cx: &App,
) -> Result<Option<Length>, String> {
    let text = input.read(cx).value().trim().to_string();
    if text.is_empty() || text.eq_ignore_ascii_case("auto") {
        return Ok(None);
    }
    Length::parse(&text)
        .map(Some)
        .ok_or_else(|| format!("{name} is a number of pixels (320) or a percentage (40%)."))
}

fn spacing_field(input: &Entity<InputState>, name: &str, cx: &App) -> Result<Option<u32>, String> {
    let text = input.read(cx).value().trim().to_string();
    if text.is_empty() {
        return Ok(None);
    }
    match text.trim_end_matches("px").trim().parse::<u32>() {
        Ok(0) => Ok(None),
        Ok(v) => Ok(Some(v)),
        Err(_) => Err(format!("{name} spacing is a whole number of pixels.")),
    }
}

impl ImageDialog {
    pub fn image(&self, cx: &App) -> Result<ImageBlock, String> {
        let url = self.src.read(cx).value().trim().to_string();
        if url.is_empty() {
            return Err("Choose a file or enter a URL.".into());
        }
        Ok(ImageBlock {
            url,
            alt: self.alt.read(cx).value().trim().to_string(),
            title: self.title.clone(),
            width: length_field(&self.width, "Width", cx)?,
            height: length_field(&self.height, "Height", cx)?,
            hspace: spacing_field(&self.hspace, "Horizontal", cx)?,
            vspace: spacing_field(&self.vspace, "Vertical", cx)?,
            align: if self.inline && self.align == ImageAlign::Center {
                ImageAlign::Inline
            } else {
                self.align
            },
        })
    }

    pub fn can_inline(&self) -> bool {
        !matches!(self.target, ImageTarget::Replace(_))
    }
}

pub fn length_px(length: Length) -> DefiniteLength {
    match length {
        Length::Px(v) => px(v as f32).into(),
        Length::Percent(p) => relative(p as f32 / 100.),
    }
}

pub fn resolve_image(base: Option<&Path>, url: &str) -> ImageSource {
    match base {
        Some(base) if !url.contains(':') => ImageSource::from(base.join(url)),
        _ => ImageSource::from(SharedUri::from(url.to_string())),
    }
}

fn relative_to(base: Option<&Path>, path: &Path) -> String {
    base.and_then(|base| path.strip_prefix(base).ok())
        .map(|rel| rel.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

impl VisualView {
    pub fn image_block_index(&self, id: u64) -> Option<usize> {
        self.doc.blocks.iter().position(|b| b.id == id)
    }

    pub fn selected_image(&self) -> Option<(usize, ImageBlock)> {
        let ix = self.image_block_index(self.selected_image?)?;
        match &self.doc.blocks[ix].kind {
            Kind::Image(image) => Some((ix, image.clone())),
            _ => None,
        }
    }

    pub fn select_image(&mut self, id: Option<u64>, cx: &mut Context<Self>) {
        if self.selected_image != id {
            self.selected_image = id;
            cx.notify();
        }
    }

    pub fn write_image(
        &mut self,
        target: ImageTarget,
        image: &ImageBlock,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let markup = image.markup();
        let at = match target {
            ImageTarget::Replace(id) => {
                let Some(ix) = self.image_block_index(id) else {
                    return;
                };
                let block = &self.doc.blocks[ix];
                let range = block.start..block.end;
                self.edit(range.clone(), &markup, window, cx);
                range.start
            }
            ImageTarget::Insert(offset) => {
                let text = self.editor.read(cx).text().to_string();
                let offset = offset.min(text.len());
                let end = text[offset..].find('\n').map_or(text.len(), |i| offset + i);
                let before = if end == 0 || text[..end].ends_with("\n\n") {
                    ""
                } else if text[..end].ends_with('\n') {
                    "\n"
                } else {
                    "\n\n"
                };
                let after = if text[end..].starts_with("\n\n") || end == text.len() {
                    ""
                } else if text[end..].starts_with('\n') {
                    "\n"
                } else {
                    "\n\n"
                };
                let insert = format!("{before}{markup}{after}");
                self.edit(end..end, &insert, window, cx);
                end + before.len()
            }
            ImageTarget::Inline(range) => {
                self.edit(range.clone(), &markup, window, cx);
                self.selected_image = None;
                let caret = range.start + markup.len();
                self.place_caret(caret, None, cx);
                cx.notify();
                return;
            }
        };
        self.selected_image = self
            .doc
            .blocks
            .iter()
            .find(|b| b.start <= at && at < b.end.max(b.start + 1))
            .filter(|b| matches!(b.kind, Kind::Image(_)))
            .map(|b| b.id);
        cx.notify();
    }

    pub fn delete_selected_image(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some((ix, _)) = self.selected_image() else {
            return false;
        };
        let text = self.editor.read(cx).text().to_string();
        let block = &self.doc.blocks[ix];
        let mut range = block.start..block.end;
        let trailing = text[range.end..]
            .chars()
            .take_while(|c| *c == '\n' || *c == '\r')
            .count();
        if range.end + trailing < text.len() {
            range.end += trailing;
        } else {
            let leading = text[..range.start]
                .chars()
                .rev()
                .take_while(|c| *c == '\n' || *c == '\r')
                .count();
            range.start -= leading;
            range.end += trailing;
        }
        self.selected_image = None;
        self.edit(range, "", window, cx);
        true
    }

    pub fn set_image_align(
        &mut self,
        align: ImageAlign,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some((ix, mut image)) = self.selected_image() {
            image.align = align;
            let id = self.doc.blocks[ix].id;
            self.write_image(ImageTarget::Replace(id), &image, window, cx);
        }
    }

    pub fn open_image_dialog(
        &mut self,
        target: ImageTarget,
        existing: Option<ImageBlock>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let inline = matches!(target, ImageTarget::Inline(_));
        let existing = existing.unwrap_or_default();
        let text = |placeholder: &'static str,
                    value: String,
                    window: &mut Window,
                    cx: &mut Context<Self>| {
            cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(placeholder)
                    .default_value(value)
            })
        };
        let number = |placeholder: &'static str,
                      value: Option<u32>,
                      window: &mut Window,
                      cx: &mut Context<Self>| {
            cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(placeholder)
                    .min(0.)
                    .max(MAX_SPACING)
                    .default_value(value.map(|v| v.to_string()).unwrap_or_default())
            })
        };
        let src = text(
            "pictures/photo.png or https://…",
            existing.url.clone(),
            window,
            cx,
        );
        let alt = text("Describe the image", existing.alt.clone(), window, cx);
        let width = text(
            "auto",
            existing.width.map(|w| w.to_string()).unwrap_or_default(),
            window,
            cx,
        );
        let height = text(
            "auto",
            existing.height.map(|h| h.to_string()).unwrap_or_default(),
            window,
            cx,
        );
        let hspace = number("h", existing.hspace, window, cx);
        let vspace = number("v", existing.vspace, window, cx);
        let mut subscriptions = Vec::new();
        for input in [&src, &alt, &width, &height, &hspace, &vspace] {
            subscriptions.push(cx.subscribe_in(
                input,
                window,
                |this, _, event: &InputEvent, window, cx| match event {
                    InputEvent::Change => {
                        if let Some(dialog) = &mut this.image_dialog {
                            dialog.error = None;
                            cx.notify();
                        }
                    }
                    InputEvent::PressEnter { .. } => this.confirm_image_dialog(window, cx),
                    _ => {}
                },
            ));
        }
        src.update(cx, |state, cx| state.focus(window, cx));
        self.image_dialog = Some(ImageDialog {
            target,
            src,
            alt,
            width,
            height,
            hspace,
            vspace,
            align: existing.align,
            inline,
            title: existing.title,
            error: None,
            _subscriptions: subscriptions,
        });
        cx.notify();
    }

    pub fn inline_image_at(&self, offset: usize, cx: &App) -> Option<(Range<usize>, ImageBlock)> {
        let text = self.editor.read(cx).text().to_string();
        let offset = offset.min(text.len());
        let start = text[..offset].rfind('\n').map_or(0, |i| i + 1);
        let end = text[offset..].find('\n').map_or(text.len(), |i| offset + i);
        inline_images(&text[start..end])
            .into_iter()
            .map(|(range, image)| (range.start + start..range.end + start, image))
            .find(|(range, _)| range.start <= offset && offset <= range.end)
    }

    pub fn insert_image_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.selected_image() {
            Some((ix, image)) => {
                let id = self.doc.blocks[ix].id;
                self.open_image_dialog(ImageTarget::Replace(id), Some(image), window, cx);
            }
            None => {
                let at = self.caret;
                match self.inline_image_at(at, cx) {
                    Some((range, image)) => {
                        self.open_image_dialog(ImageTarget::Inline(range), Some(image), window, cx)
                    }
                    None => self.open_image_dialog(ImageTarget::Insert(at), None, window, cx),
                }
            }
        }
    }

    pub fn close_image_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.image_dialog.take().is_some() {
            window.focus(&self.focus, cx);
            cx.notify();
        }
    }

    pub fn confirm_image_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(dialog) = &mut self.image_dialog else {
            return;
        };
        match dialog.image(cx) {
            Ok(image) => {
                let target = dialog.target.clone();
                let inline = dialog.inline && dialog.can_inline();
                self.image_dialog = None;
                window.focus(&self.focus, cx);
                match (target, inline) {
                    (ImageTarget::Insert(at), true) => {
                        self.write_image(ImageTarget::Inline(at..at), &image, window, cx)
                    }
                    (ImageTarget::Inline(range), false) => {
                        self.edit(range.clone(), "", window, cx);
                        self.write_image(ImageTarget::Insert(range.start), &image, window, cx);
                    }
                    (target, _) => self.write_image(target, &image, window, cx),
                }
            }
            Err(error) => {
                dialog.error = Some(error);
                cx.notify();
            }
        }
    }

    pub fn delete_dialog_image(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(dialog) = self.image_dialog.take() else {
            return;
        };
        window.focus(&self.focus, cx);
        match dialog.target {
            ImageTarget::Replace(id) => {
                self.selected_image = Some(id);
                self.delete_selected_image(window, cx);
            }
            ImageTarget::Inline(range) => {
                let caret = range.start;
                self.edit(range, "", window, cx);
                self.place_caret(caret, None, cx);
            }
            ImageTarget::Insert(_) => {}
        }
        cx.notify();
    }

    fn set_dialog_source(&mut self, path: &Path, window: &mut Window, cx: &mut Context<Self>) {
        let value = relative_to(self.base.as_deref(), path);
        if let Some(dialog) = &self.image_dialog {
            dialog
                .src
                .update(cx, |state, cx| state.set_value(value, window, cx));
        }
    }

    fn choose_image_file(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Choose".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = receiver.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            let _ = this.update_in(cx, |this, window, cx| {
                this.set_dialog_source(&path, window, cx);
            });
        })
        .detach();
    }

    pub fn image_resize_move(&mut self, x: Pixels, cx: &mut Context<Self>) -> bool {
        let Some(resize) = &mut self.image_resize else {
            return false;
        };
        let column = self.list.viewport_bounds().size.width.max(px(MIN_WIDTH));
        let delta = (x - resize.start_x) * if resize.centered { 2. } else { 1. };
        resize.width = (resize.start_width + delta).clamp(px(MIN_WIDTH), column);
        cx.notify();
        true
    }

    pub fn image_resize_end(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(resize) = self.image_resize.take() else {
            return false;
        };
        if let Some(ix) = self.image_block_index(resize.block)
            && let Kind::Image(image) = &self.doc.blocks[ix].kind
        {
            let mut image = image.clone();
            image.width = Some(Length::Px(f32::from(resize.width).round() as u32));
            self.write_image(ImageTarget::Replace(resize.block), &image, window, cx);
        }
        true
    }

    pub fn render_image_block(
        &self,
        id: u64,
        image: &ImageBlock,
        pad_top: Pixels,
        pad_bottom: Pixels,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selected = self.selected_image == Some(id);
        let toolbar = selected.then(|| self.image_toolbar(image.align, cx));
        let theme = cx.theme().clone();
        let width: Option<DefiniteLength> = match self.image_resize {
            Some(resize) if resize.block == id => Some(resize.width.into()),
            _ => image.width.map(length_px),
        };
        let height = match image.height {
            Some(Length::Px(h)) if !matches!(self.image_resize, Some(r) if r.block == id) => {
                Some(px(h as f32))
            }
            _ => None,
        };
        let hspace = px(image.hspace.unwrap_or(0) as f32);
        let vspace = px(image.vspace.unwrap_or(0) as f32);
        let source = resolve_image(self.base.as_deref(), &image.url);
        let store = self.image_bounds.clone();
        let muted = theme.muted_foreground;
        let ring = theme.ring;
        let surface = theme.secondary;
        let url = image.url.clone();
        let centered = image.align == ImageAlign::Center;
        let picture = img(source)
            .max_w_full()
            .when(width.is_some(), |img| img.w_full())
            .when_some(height, |img, height| img.h(height))
            .rounded(tokens::RADIUS_SM)
            .with_loading(move || {
                div()
                    .w(tokens::PLACEHOLDER_WIDTH)
                    .h(tokens::PREVIEW_HEIGHT)
                    .rounded(tokens::RADIUS_SM)
                    .bg(surface)
                    .into_any_element()
            })
            .with_fallback(move || {
                div()
                    .px_3()
                    .py_2()
                    .rounded(tokens::RADIUS_SM)
                    .bg(surface)
                    .text_sm()
                    .text_color(muted)
                    .child(format!("Image not found: {url}"))
                    .into_any_element()
            });
        let handle = selected.then(|| {
            div()
                .id(("image-resize", id))
                .absolute()
                .right(tokens::HANDLE_INSET)
                .bottom(tokens::HANDLE_INSET)
                .size(tokens::HANDLE)
                .rounded_full()
                .bg(ring)
                .border_2()
                .border_color(theme.background)
                .cursor_ew_resize()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                        cx.stop_propagation();
                        window.focus(&this.focus, cx);
                        let start = this
                            .image_bounds
                            .borrow()
                            .get(&id)
                            .map_or(tokens::PLACEHOLDER_WIDTH, |b| b.size.width);
                        this.image_resize = Some(ImageResize {
                            block: id,
                            start_x: event.position.x,
                            start_width: start,
                            width: start,
                            centered,
                        });
                        cx.notify();
                    }),
                )
        });
        div()
            .w_full()
            .pt(pad_top + px(BODY_SIZE * 0.4) + vspace)
            .pb(pad_bottom + px(BODY_SIZE * 0.6) + vspace)
            .px(hspace)
            .flex()
            .when(centered, |row| row.justify_center())
            .when(image.align == ImageAlign::Right, |row| row.justify_end())
            .child(
                div()
                    .id(("image", id))
                    .relative()
                    .max_w_full()
                    .when_some(width, |frame, width| frame.w(width))
                    .cursor_pointer()
                    .rounded(tokens::RADIUS_MD)
                    .border_2()
                    .border_color(if selected {
                        ring
                    } else {
                        gpui_kit::transparent_black()
                    })
                    .child(picture)
                    .child(
                        canvas(
                            move |bounds, _, _| {
                                store.borrow_mut().insert(id, bounds);
                            },
                            |_, _, _, _| {},
                        )
                        .absolute()
                        .size_full(),
                    )
                    .children(toolbar)
                    .children(handle)
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                            cx.stop_propagation();
                            window.focus(&this.focus, cx);
                            this.anchor = None;
                            this.select_image(Some(id), cx);
                            if event.click_count >= 2
                                && let Some((_, image)) = this.selected_image()
                            {
                                this.open_image_dialog(
                                    ImageTarget::Replace(id),
                                    Some(image),
                                    window,
                                    cx,
                                );
                            }
                        }),
                    ),
            )
            .into_any_element()
    }

    fn image_toolbar(&self, align: ImageAlign, cx: &mut Context<Self>) -> AnyElement {
        let aligns = [
            (
                "image-left",
                IconName::TextAlignStart,
                "Align left",
                ImageAlign::Left,
            ),
            (
                "image-center",
                IconName::TextAlignCenter,
                "Center",
                ImageAlign::Center,
            ),
            (
                "image-right",
                IconName::TextAlignEnd,
                "Float right",
                ImageAlign::Right,
            ),
        ];
        toolbar::floating_toolbar("image-toolbar", cx)
            .absolute()
            .top_2()
            .right_2()
            .children(aligns.into_iter().map(|(id, icon, tip, value)| {
                let target = if align == value {
                    ImageAlign::Inline
                } else {
                    value
                };
                toolbar::tool(id, icon, tip, align == value).on_click(cx.listener(
                    move |this, _, window, cx| {
                        cx.stop_propagation();
                        this.set_image_align(target, window, cx);
                    },
                ))
            }))
            .child(toolbar::divider(cx))
            .child(
                toolbar::tool("image-edit", IconName::SquarePen, "Edit image…", false).on_click(
                    cx.listener(|this, _, window, cx| {
                        cx.stop_propagation();
                        this.insert_image_dialog(window, cx);
                    }),
                ),
            )
            .child(
                toolbar::tool("image-delete", IconName::Trash, "Remove image", false).on_click(
                    cx.listener(|this, _, window, cx| {
                        cx.stop_propagation();
                        this.delete_selected_image(window, cx);
                    }),
                ),
            )
            .into_any_element()
    }

    fn natural_size(&self, url: &str, window: &mut Window, cx: &mut App) -> Option<(u32, u32)> {
        if url.is_empty() {
            return None;
        }
        let ImageSource::Resource(resource) = resolve_image(self.base.as_deref(), url) else {
            return None;
        };
        let image = window.use_asset::<ImgResourceLoader>(&resource, cx)?.ok()?;
        let size = image.size(0);
        Some((size.width.0 as u32, size.height.0 as u32))
    }

    pub fn render_image_dialog(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let dialog = self.image_dialog.as_ref()?;
        let theme = cx.theme().clone();
        let preview_url = dialog.src.read(cx).value().trim().to_string();
        let natural = self.natural_size(&preview_url, window, cx);
        let editing = !matches!(dialog.target, ImageTarget::Insert(_));
        let html = dialog.image(cx).is_ok_and(|image| image.needs_html());
        let muted = theme.muted_foreground;
        let ring = theme.ring;
        let helper = |text: &'static str| {
            div()
                .text_xs()
                .line_height(relative(tokens::LINE_RATIO))
                .text_color(muted)
                .child(text)
        };
        let label = |text: &'static str| {
            div()
                .w(tokens::FIELD_COMPACT)
                .flex_none()
                .h(tokens::CONTROL)
                .flex()
                .items_center()
                .justify_end()
                .text_sm()
                .text_color(theme.foreground)
                .child(text)
        };
        let times = || div().text_sm().text_color(muted).child("×");
        let setting = |name: &'static str, control: AnyElement, note: AnyElement| {
            div().flex().gap_3().child(label(name)).child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_1p5()
                    .child(control)
                    .child(note),
            )
        };

        let picture = if preview_url.is_empty() {
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .text_color(muted)
                .child(Icon::new(IconName::Image).size(tokens::CONTROL))
                .child(div().text_sm().child("Drop an image here or browse"))
                .into_any_element()
        } else {
            let url = preview_url.clone();
            img(resolve_image(self.base.as_deref(), &preview_url))
                .max_h(tokens::IMAGE_PREVIEW_HEIGHT - tokens::SPACE_7 * 2.)
                .max_w_full()
                .object_fit(ObjectFit::Contain)
                .rounded(tokens::RADIUS_MD)
                .with_fallback(move || {
                    div()
                        .px_3()
                        .text_xs()
                        .text_color(muted)
                        .child(format!("Can't load {url}"))
                        .into_any_element()
                })
                .into_any_element()
        };
        let preview = div()
            .id("image-dialog-preview")
            .h(tokens::IMAGE_PREVIEW_HEIGHT)
            .w_full()
            .p(tokens::SPACE_7)
            .rounded(tokens::RADIUS_LG)
            .border(tokens::DASH_BORDER)
            .border_dashed()
            .border_color(theme.border)
            .bg(theme.foreground.opacity(tokens::GLASS_SHADE))
            .flex()
            .items_center()
            .justify_center()
            .overflow_hidden()
            .drag_over::<ExternalPaths>(move |style, _, _, _| style.border_color(ring))
            .on_drop(cx.listener(|this, paths: &ExternalPaths, window, cx| {
                if let Some(path) = paths.paths().first() {
                    this.set_dialog_source(&path.clone(), window, cx);
                }
            }))
            .child(picture);
        let dimensions = div()
            .h(tokens::CONTROL_SM)
            .flex()
            .justify_end()
            .text_xs()
            .text_color(muted)
            .children(natural.map(|(w, h)| format!("{w} × {h} px")));

        let source = div()
            .flex()
            .flex_col()
            .gap_1p5()
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(div().flex_1().min_w_0().child(Input::new(&dialog.src)))
                    .child(
                        Button::new("image-choose")
                            .outline()
                            .label("Browse…")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.choose_image_file(window, cx)
                            })),
                    ),
            )
            .child(helper(
                "A path relative to the document, a /path from the opened folder's root, or an http(s) URL. Browse fills in the relative path.",
            ));
        let description = div()
            .flex()
            .flex_col()
            .gap_1p5()
            .child(Input::new(&dialog.alt))
            .child(helper(
                "Read aloud by screen readers and shown if the image can't load: the ![alt] in the Markdown.",
            ));

        let size = div()
            .flex()
            .items_center()
            .gap_2()
            .child(
                div()
                    .w(tokens::FIELD_COMPACT)
                    .child(Input::new(&dialog.width)),
            )
            .child(times())
            .child(
                div()
                    .w(tokens::FIELD_COMPACT)
                    .child(Input::new(&dialog.height)),
            )
            .into_any_element();
        let spacing = div()
            .flex()
            .items_center()
            .gap_2()
            .child(
                div()
                    .w(tokens::FIELD_COMPACT)
                    .child(stepper("hspace", &dialog.hspace, cx)),
            )
            .child(times())
            .child(
                div()
                    .w(tokens::FIELD_COMPACT)
                    .child(stepper("vspace", &dialog.vspace, cx)),
            )
            .into_any_element();
        let align = dialog.align;
        let inline = dialog.inline;
        let aligns = div()
            .flex()
            .gap_2()
            .children(ImageAlign::ALL.into_iter().map(|value| {
                let disabled = inline && value == ImageAlign::Center;
                align_tile(value, value == align, disabled, cx)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if let Some(dialog) = &mut this.image_dialog
                            && !disabled
                        {
                            dialog.align = value;
                            dialog.error = None;
                            cx.notify();
                        }
                    }))
                    .into_any_element()
            }))
            .into_any_element();
        let can_inline = dialog.can_inline();
        let inline_switch = Switch::new("image-inline")
            .checked(inline)
            .disabled(!can_inline)
            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                if let Some(dialog) = &mut this.image_dialog {
                    dialog.inline = *checked;
                    cx.notify();
                }
            }))
            .into_any_element();
        let format_note = if html {
            "Saved as an HTML <img>, so size, spacing and alignment survive on GitHub."
        } else {
            "Saved as a plain Markdown image."
        };

        let left = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(preview)
                    .child(dimensions),
            )
            .child(source)
            .child(description);
        let right = div()
            .w(tokens::IMAGE_DIALOG_SIDE)
            .flex_none()
            .flex()
            .flex_col()
            .gap_5()
            .pt(tokens::SPACE_6)
            .child(setting(
                "Size",
                size,
                helper("Width × height in pixels or as a percentage (40%). Leave empty for the natural size.")
                    .into_any_element(),
            ))
            .child(setting(
                "Spacing",
                spacing,
                helper("Margins around the image in pixels (hspace × vspace), the way GitHub spaces floated images.")
                    .into_any_element(),
            ))
            .child(setting(
                "Align",
                aligns,
                helper("Left and right float the image so the text wraps around it, as on GitHub.")
                    .into_any_element(),
            ))
            .child(setting(
                "Inline",
                inline_switch,
                helper(if can_inline {
                    "Flows with the text, like a badge."
                } else {
                    "This image has its own line. Insert a new image to place one inside a sentence."
                })
                .into_any_element(),
            ))
            .child(
                div()
                    .pl(tokens::FIELD_COMPACT + tokens::SPACE_5)
                    .child(helper(format_note)),
            );

        let footer = div()
            .flex()
            .items_center()
            .gap_2()
            .pt(tokens::SPACE_6)
            .border_t_1()
            .border_color(theme.border)
            .when(editing, |row| {
                row.child(
                    Button::new("image-delete-dialog")
                        .outline()
                        .danger()
                        .label("Delete Image")
                        .on_click(
                            cx.listener(|this, _, window, cx| this.delete_dialog_image(window, cx)),
                        ),
                )
            })
            .child(div().flex_1())
            .child(
                Button::new("image-cancel")
                    .outline()
                    .label("Cancel")
                    .on_click(
                        cx.listener(|this, _, window, cx| this.close_image_dialog(window, cx)),
                    ),
            )
            .child(
                Button::new("image-confirm")
                    .primary()
                    .label(if editing { "Apply" } else { "Insert" })
                    .on_click(
                        cx.listener(|this, _, window, cx| this.confirm_image_dialog(window, cx)),
                    ),
            );

        let panel = crate::glass::surface(div(), crate::glass::Depth::Panel, cx)
            .id("image-dialog")
            .w(tokens::IMAGE_DIALOG_WIDTH)
            .max_w_full()
            .p(tokens::SPACE_7)
            .flex()
            .flex_col()
            .gap_5()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(
                div()
                    .flex()
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(muted)
                            .child(if editing { "IMAGE" } else { "INSERT IMAGE" }),
                    )
                    .child(
                        Button::new("image-dialog-close")
                            .ghost()
                            .xsmall()
                            .icon(IconName::Close)
                            .tooltip("Close")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.close_image_dialog(window, cx)
                            })),
                    ),
            )
            .child(div().flex().gap_7().child(left).child(right))
            .when_some(dialog.error.clone(), |panel, error| {
                panel.child(div().text_sm().text_color(theme.danger).child(error))
            })
            .child(footer);

        Some(
            div()
                .id("image-dialog-backdrop")
                .absolute()
                .inset_0()
                .p(tokens::SPACE_7)
                .flex()
                .items_center()
                .justify_center()
                .bg(theme.overlay)
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .child(panel)
                .into_any_element(),
        )
    }
}

fn align_tile(value: ImageAlign, selected: bool, disabled: bool, cx: &App) -> Stateful<Div> {
    let theme = cx.theme();
    let ink = if selected {
        theme.primary
    } else {
        theme.muted_foreground
    };
    let line = |width: Pixels| {
        div()
            .w(width)
            .h(tokens::GLYPH_STROKE)
            .rounded_full()
            .bg(ink.opacity(tokens::FADED))
    };
    let block = |width: Pixels| {
        div()
            .w(width)
            .h(tokens::GLYPH_BLOCK)
            .rounded(tokens::GLYPH_STROKE)
            .bg(ink)
    };
    let lines = || {
        div()
            .flex()
            .flex_col()
            .gap(tokens::GLYPH_STROKE)
            .child(line(tokens::GLYPH_HALF))
            .child(line(tokens::GLYPH_HALF))
            .child(line(tokens::GLYPH_HALF))
    };
    let glyph = match value {
        ImageAlign::Inline | ImageAlign::Center => div()
            .w(tokens::GLYPH_WIDTH)
            .flex()
            .flex_col()
            .items_center()
            .gap(tokens::GLYPH_STROKE)
            .child(line(tokens::GLYPH_WIDTH))
            .child(block(if value == ImageAlign::Inline {
                tokens::GLYPH_WIDTH
            } else {
                tokens::GLYPH_HALF
            }))
            .child(line(tokens::GLYPH_WIDTH)),
        ImageAlign::Left => div()
            .flex()
            .gap(tokens::GLYPH_STROKE)
            .child(block(tokens::GLYPH_BLOCK + tokens::GLYPH_STROKE))
            .child(lines()),
        ImageAlign::Right => div()
            .flex()
            .gap(tokens::GLYPH_STROKE)
            .child(lines())
            .child(block(tokens::GLYPH_BLOCK + tokens::GLYPH_STROKE)),
    };
    let tip: SharedString = match value {
        ImageAlign::Inline => "Default".into(),
        ImageAlign::Left => "Float left".into(),
        ImageAlign::Center => "Center".into(),
        ImageAlign::Right => "Float right".into(),
    };
    div()
        .id(SharedString::from(format!(
            "dialog-align-{}",
            value.label()
        )))
        .w(tokens::ALIGN_TILE)
        .h(tokens::CONTROL_LG)
        .flex()
        .items_center()
        .justify_center()
        .rounded(tokens::RADIUS_MD)
        .border_1()
        .border_color(if selected {
            theme.primary
        } else {
            theme.border
        })
        .when(selected, |tile| {
            tile.bg(theme.primary.opacity(tokens::SUBTLE))
        })
        .when(disabled, |tile| tile.opacity(tokens::DISABLED))
        .when(!disabled, |tile| {
            tile.cursor_pointer()
                .hover(|style| style.border_color(theme.ring))
        })
        .tooltip(move |window, cx| Tooltip::new(tip.clone()).build(window, cx))
        .child(glyph)
}

fn stepper(id: &'static str, input: &Entity<InputState>, cx: &mut Context<VisualView>) -> Input {
    let step = |icon: IconName, delta: i64, cx: &mut Context<VisualView>| {
        let input = input.clone();
        Button::new(SharedString::from(format!("{id}-{delta}")))
            .ghost()
            .xsmall()
            .icon(Icon::new(icon).size(tokens::ICON_SM))
            .h(tokens::CONTROL_SM / 2.)
            .on_click(cx.listener(move |_, _, window, cx| {
                input.update(cx, |state, cx| {
                    let value = state.value().trim().parse::<i64>().unwrap_or(0);
                    let next = (value + delta).clamp(0, MAX_SPACING as i64);
                    let text = if next == 0 {
                        String::new()
                    } else {
                        next.to_string()
                    };
                    state.set_value(text, window, cx);
                });
            }))
    };
    Input::new(input).suffix(
        div()
            .flex()
            .flex_col()
            .child(step(IconName::ChevronUp, 1, cx))
            .child(step(IconName::ChevronDown, -1, cx)),
    )
}
