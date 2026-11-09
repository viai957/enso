use std::fmt;
use std::ops::Range;

use gpui_kit::base::text::editor_host::Paragraph;
use gpui_kit::{AbsoluteLength, DefiniteLength};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageAlign {
    #[default]
    Inline,
    Left,
    Center,
    Right,
}

impl ImageAlign {
    pub const ALL: [ImageAlign; 4] = [
        ImageAlign::Inline,
        ImageAlign::Left,
        ImageAlign::Center,
        ImageAlign::Right,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Inline => "Default",
            Self::Left => "Left",
            Self::Center => "Center",
            Self::Right => "Right",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Length {
    Px(u32),
    Percent(u32),
}

impl Length {
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        if let Some(percent) = text.strip_suffix('%') {
            return match percent.trim().parse::<f32>() {
                Ok(p) if p > 0. && p <= 100. => Some(Self::Percent(p.round() as u32)),
                _ => None,
            };
        }
        match text.trim_end_matches("px").trim().parse::<f32>() {
            Ok(p) if p >= 1. => Some(Self::Px(p.round() as u32)),
            _ => None,
        }
    }

    fn from_definite(length: Option<DefiniteLength>) -> Option<Self> {
        match length? {
            DefiniteLength::Absolute(AbsoluteLength::Pixels(px)) => {
                Some(Self::Px(f32::from(px).round() as u32))
            }
            DefiniteLength::Fraction(f) if f > 0. => Some(Self::Percent((f * 100.).round() as u32)),
            _ => None,
        }
    }
}

impl fmt::Display for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Px(px) => write!(f, "{px}"),
            Self::Percent(p) => write!(f, "{p}%"),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ImageBlock {
    pub url: String,
    pub alt: String,
    pub title: Option<String>,
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub hspace: Option<u32>,
    pub vspace: Option<u32>,
    pub align: ImageAlign,
}

impl ImageBlock {
    pub fn needs_html(&self) -> bool {
        self.width.is_some()
            || self.height.is_some()
            || self.hspace.is_some()
            || self.vspace.is_some()
            || self.align != ImageAlign::Inline
    }

    pub fn markup(&self) -> String {
        let title = self.title.as_ref().filter(|t| !t.is_empty());
        if !self.needs_html() {
            let title = title
                .map(|t| format!(" \"{}\"", t.replace('"', "\\\"")))
                .unwrap_or_default();
            return format!(
                "![{}]({}{})",
                self.alt.replace(']', "\\]"),
                markdown_url(&self.url),
                title
            );
        }
        let mut tag = format!(
            "<img src=\"{}\" alt=\"{}\"",
            attr(&self.url),
            attr(&self.alt)
        );
        if let Some(title) = title {
            tag.push_str(&format!(" title=\"{}\"", attr(title)));
        }
        if let Some(width) = self.width {
            tag.push_str(&format!(" width=\"{width}\""));
        }
        if let Some(height) = self.height {
            tag.push_str(&format!(" height=\"{height}\""));
        }
        if let Some(hspace) = self.hspace {
            tag.push_str(&format!(" hspace=\"{hspace}\""));
        }
        if let Some(vspace) = self.vspace {
            tag.push_str(&format!(" vspace=\"{vspace}\""));
        }
        match self.align {
            ImageAlign::Left => tag.push_str(" align=\"left\""),
            ImageAlign::Right => tag.push_str(" align=\"right\""),
            _ => {}
        }
        tag.push('>');
        if self.align == ImageAlign::Center {
            format!("<p align=\"center\">{tag}</p>")
        } else {
            tag
        }
    }
}

fn attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}

fn unescape(value: &str) -> String {
    value
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

fn markdown_url(url: &str) -> String {
    if url.contains(' ') || url.contains(')') {
        format!("<{url}>")
    } else {
        url.to_string()
    }
}

fn html_attr(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut from = 0;
    while let Some(found) = lower[from..].find(name) {
        let at = from + found;
        from = at + name.len();
        if at == 0 || !bytes[at - 1].is_ascii_whitespace() {
            continue;
        }
        let rest = lower[from..].trim_start();
        let Some(rest) = rest.strip_prefix('=') else {
            continue;
        };
        let value_at = lower.len() - rest.trim_start().len();
        let original = &tag[value_at..];
        let value = match original.chars().next() {
            Some(q @ ('"' | '\'')) => original[1..].split(q).next().unwrap_or(""),
            _ => original
                .split(|c: char| c.is_whitespace() || c == '>' || c == '/')
                .next()
                .unwrap_or(""),
        };
        return Some(unescape(value));
    }
    None
}

fn img_tag(source: &str) -> Option<&str> {
    let lower = source.to_ascii_lowercase();
    let start = lower.find("<img")?;
    let end = source[start..]
        .find('>')
        .map_or(source.len(), |e| start + e + 1);
    Some(&source[start..end])
}

fn align_in(source: &str) -> ImageAlign {
    let text = source.trim_start();
    if !text.starts_with('<') {
        return ImageAlign::Inline;
    }
    let tag = img_tag(text).unwrap_or(text);
    let value = html_attr(tag, "align").or_else(|| html_attr(text, "align"));
    match value.as_deref().map(str::to_ascii_lowercase).as_deref() {
        Some("center" | "middle") => ImageAlign::Center,
        Some("right") => ImageAlign::Right,
        Some("left") => ImageAlign::Left,
        _ => {
            let lower = text.to_ascii_lowercase();
            if lower.contains("text-align:center") || lower.contains("text-align: center") {
                ImageAlign::Center
            } else {
                ImageAlign::Inline
            }
        }
    }
}

fn from_html(tag: &str, align: ImageAlign) -> Option<ImageBlock> {
    let url = html_attr(tag, "src")?;
    let number = |name| {
        html_attr(tag, name)
            .and_then(|v| v.trim().trim_end_matches("px").parse::<f32>().ok())
            .map(|v| v.round() as u32)
    };
    Some(ImageBlock {
        url,
        alt: html_attr(tag, "alt").unwrap_or_default(),
        title: html_attr(tag, "title"),
        width: html_attr(tag, "width").and_then(|v| Length::parse(&v)),
        height: html_attr(tag, "height").and_then(|v| Length::parse(&v)),
        hspace: number("hspace"),
        vspace: number("vspace"),
        align,
    })
}

pub fn lone_image(paragraph: &Paragraph, source: &str) -> Option<ImageBlock> {
    let mut found = None;
    for node in paragraph.children() {
        if let Some(image) = node.as_image() {
            if found.is_some() {
                return None;
            }
            found = Some(image.clone());
        } else if !node.text().trim().is_empty() || node.as_custom().is_some() {
            return None;
        }
    }
    let image = found?;
    let text = source.trim();
    if text.starts_with('[') {
        return None;
    }
    let align = align_in(text);
    if text.starts_with('<')
        && let Some(block) = img_tag(text).and_then(|tag| from_html(tag, align))
    {
        return Some(block);
    }
    Some(ImageBlock {
        url: image.url.to_string(),
        alt: image.alt.map(|a| a.to_string()).unwrap_or_default(),
        title: image.title.map(|t| t.to_string()),
        width: Length::from_definite(image.width),
        height: Length::from_definite(image.height),
        hspace: None,
        vspace: None,
        align,
    })
}

fn markdown_image(text: &str, start: usize) -> Option<(Range<usize>, ImageBlock)> {
    let bytes = text.as_bytes();
    let mut i = start + 2;
    let mut depth = 0;
    let alt_start = i;
    loop {
        match *bytes.get(i)? {
            b'\\' => i += 1,
            b'[' => depth += 1,
            b']' if depth == 0 => break,
            b']' => depth -= 1,
            b'\n' if bytes.get(i + 1) == Some(&b'\n') => return None,
            _ => {}
        }
        i += 1;
    }
    let alt = text[alt_start..i].replace("\\]", "]").replace("\\[", "[");
    i += 1;
    if bytes.get(i) != Some(&b'(') {
        return None;
    }
    i += 1;
    while bytes.get(i).is_some_and(|b| *b == b' ') {
        i += 1;
    }
    let url = if bytes.get(i) == Some(&b'<') {
        let end = i + text[i..].find('>')?;
        let url = text[i + 1..end].to_string();
        i = end + 1;
        url
    } else {
        let url_start = i;
        let mut parens = 0;
        while let Some(&b) = bytes.get(i) {
            match b {
                b'(' => parens += 1,
                b')' if parens == 0 => break,
                b')' => parens -= 1,
                b' ' | b'\n' | b'\t' => break,
                _ => {}
            }
            i += 1;
        }
        text[url_start..i].to_string()
    };
    while bytes.get(i).is_some_and(|b| b.is_ascii_whitespace()) {
        i += 1;
    }
    let mut title = None;
    if let Some(&q @ (b'"' | b'\'')) = bytes.get(i) {
        let end = i + 1 + text[i + 1..].find(q as char)?;
        title = Some(text[i + 1..end].replace("\\\"", "\""));
        i = end + 1;
        while bytes.get(i).is_some_and(|b| b.is_ascii_whitespace()) {
            i += 1;
        }
    }
    if bytes.get(i) != Some(&b')') || url.is_empty() {
        return None;
    }
    Some((
        start..i + 1,
        ImageBlock {
            url,
            alt,
            title,
            ..Default::default()
        },
    ))
}

pub fn inline_images(text: &str) -> Vec<(Range<usize>, ImageBlock)> {
    let lower = text.to_ascii_lowercase();
    let mut out = Vec::new();
    let mut i = 0;
    while i < text.len() {
        let rest = &text[i..];
        if rest.starts_with("![")
            && (i == 0 || text.as_bytes()[i - 1] != b'\\')
            && let Some((range, image)) = markdown_image(text, i)
        {
            i = range.end;
            out.push((range, image));
            continue;
        }
        if lower[i..].starts_with("<img")
            && lower[i + 4..]
                .chars()
                .next()
                .is_some_and(|c| c.is_whitespace() || c == '/' || c == '>')
        {
            let end = text[i..].find('>').map_or(text.len(), |e| i + e + 1);
            let tag = &text[i..end];
            if let Some(image) = from_html(tag, align_in(tag)) {
                out.push((i..end, image));
            }
            i = end;
            continue;
        }
        i += rest.chars().next().map_or(1, char::len_utf8);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(width: Option<Length>, align: ImageAlign) -> ImageBlock {
        ImageBlock {
            url: "pics/a b.png".into(),
            alt: "A \"view\"".into(),
            width,
            align,
            ..Default::default()
        }
    }

    #[test]
    fn markdown_unless_size_spacing_or_alignment_needs_html() {
        assert_eq!(
            image(None, ImageAlign::Inline).markup(),
            "![A \"view\"](<pics/a b.png>)"
        );
        assert_eq!(
            image(Some(Length::Px(320)), ImageAlign::Inline).markup(),
            "<img src=\"pics/a b.png\" alt=\"A &quot;view&quot;\" width=\"320\">"
        );
        assert_eq!(
            image(None, ImageAlign::Center).markup(),
            "<p align=\"center\"><img src=\"pics/a b.png\" alt=\"A &quot;view&quot;\"></p>"
        );
        assert_eq!(
            image(Some(Length::Px(200)), ImageAlign::Right).markup(),
            "<img src=\"pics/a b.png\" alt=\"A &quot;view&quot;\" width=\"200\" align=\"right\">"
        );
        let spaced = ImageBlock {
            url: "a.png".into(),
            width: Some(Length::Percent(40)),
            height: Some(Length::Px(120)),
            hspace: Some(12),
            vspace: Some(8),
            align: ImageAlign::Left,
            ..Default::default()
        };
        assert_eq!(
            spaced.markup(),
            "<img src=\"a.png\" alt=\"\" width=\"40%\" height=\"120\" hspace=\"12\" vspace=\"8\" align=\"left\">"
        );
    }

    #[test]
    fn lengths_accept_pixels_and_percentages() {
        assert_eq!(Length::parse("320"), Some(Length::Px(320)));
        assert_eq!(Length::parse(" 320px "), Some(Length::Px(320)));
        assert_eq!(Length::parse("40%"), Some(Length::Percent(40)));
        assert_eq!(Length::parse("140%"), None);
        assert_eq!(Length::parse("auto"), None);
        assert_eq!(Length::parse(""), None);
    }

    #[test]
    fn alignment_comes_from_html() {
        assert_eq!(
            align_in("<p align=\"center\"><img src=x></p>"),
            ImageAlign::Center
        );
        assert_eq!(align_in("<img src='x' align='right'>"), ImageAlign::Right);
        assert_eq!(align_in("<img src=x align=left>"), ImageAlign::Left);
        assert_eq!(align_in("![x](y)"), ImageAlign::Inline);
    }

    #[test]
    fn html_round_trips_every_attribute() {
        let tag = "<img src=\"a b.png\" alt=\"Tom &amp; Jerry\" title=\"T\" width=\"40%\" height=\"90\" hspace=\"4\" vspace=\"6\" align=\"right\">";
        let image = from_html(tag, align_in(tag)).unwrap();
        assert_eq!(image.url, "a b.png");
        assert_eq!(image.alt, "Tom & Jerry");
        assert_eq!(image.width, Some(Length::Percent(40)));
        assert_eq!(image.height, Some(Length::Px(90)));
        assert_eq!((image.hspace, image.vspace), (Some(4), Some(6)));
        assert_eq!(image.markup(), tag);
        assert_eq!(
            html_attr("<img data-width=\"5\" width=7>", "width").as_deref(),
            Some("7")
        );
    }

    #[test]
    fn finds_inline_images_inside_text_and_links() {
        let text = "Badges [![chat](https://img/chat.svg)](https://chat) and <img src=\"py.svg\" height=\"20\"> or \\![not](x).";
        let found = inline_images(text);
        assert_eq!(found.len(), 2, "{found:?}");
        assert_eq!(&text[found[0].0.clone()], "![chat](https://img/chat.svg)");
        assert_eq!(found[0].1.alt, "chat");
        assert_eq!(
            &text[found[1].0.clone()],
            "<img src=\"py.svg\" height=\"20\">"
        );
        assert_eq!(found[1].1.height, Some(Length::Px(20)));
        let titled = inline_images("![a \\] b](<x y.png> \"Hi\")");
        assert_eq!(titled[0].1.alt, "a ] b");
        assert_eq!(titled[0].1.url, "x y.png");
        assert_eq!(titled[0].1.title.as_deref(), Some("Hi"));
    }
}
