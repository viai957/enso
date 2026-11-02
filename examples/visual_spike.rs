//! Phase 0 feasibility spike: one window that renders a Markdown file through
//! GPUI Kit's `editor_host` and edits it in place.
//!
//! ```sh
//! cargo run --release --example visual_spike -- path/to/file.md
//! ```
//!
//! Environment:
//! - `ENSO_SPIKE_AUTOTYPE=N` types N characters through the window's key
//!   dispatch (the platform's own path for a key press), one per 50 ms, at
//!   `ENSO_SPIKE_CARET` (a source byte offset, default: the middle), prints
//!   the latency distribution and quits.
//! - `ENSO_SPIKE_REPORT_EVERY=N` prints the distribution every N samples, for
//!   typing driven from outside (xdotool).
//! - `ENSO_SPIKE_NO_BLINK=1` keeps the caret solid, for screenshots.

#[path = "../src/visual/mod.rs"]
#[allow(dead_code)]
mod visual;

use std::time::Duration;

use gpui_kit::component::input::EditorState;
use gpui_kit::*;
use visual::view::{VisualView, summary};

fn env_usize(name: &str) -> Option<usize> {
    std::env::var(name).ok().and_then(|v| v.parse().ok())
}

fn report(view: &VisualView) {
    let latency = view.latency.borrow();
    eprintln!(
        "{}",
        summary(
            "keystroke-to-paint (CPU, to end of paint)",
            &latency.samples
        )
    );
    eprintln!(
        "{}",
        summary("  input to frame start (includes the edit)", &latency.wait)
    );
    eprintln!(
        "{}",
        summary("  frame: render..end of paint", &latency.frame)
    );
    eprintln!("{}", summary("  EditorState::replace", &latency.replace));
    eprintln!("{}", summary("  block reparse", &latency.reparse));
    if !latency.parsed_bytes.is_empty() {
        let mut bytes = latency.parsed_bytes.clone();
        bytes.sort_unstable();
        eprintln!(
            "  bytes reparsed per edit: p50={} max={}",
            bytes[bytes.len() / 2],
            bytes[bytes.len() - 1]
        );
    }
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: visual_spike <file.md>");
    let text = std::fs::read_to_string(&path).expect("a UTF-8 file");
    eprintln!("{path}: {} bytes", text.len());
    gpui_kit::application()
        .with_assets(gpui_kit::assets::AllAssets)
        .run(move |cx| {
            gpui_kit::init(cx);
            visual::view::init(cx);
            if std::env::var("ENSO_SPIKE_THEME").as_deref() == Ok("dark") {
                gpui_kit::component::Theme::change(gpui_kit::component::ThemeMode::Dark, None, cx);
            }
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1000.), px(780.)),
                    cx,
                ))),
                ..Default::default()
            };
            let len = text.len();
            let (handle, view) = gpui_kit::open_window(options, cx, |window, cx| {
                let started = std::time::Instant::now();
                let editor = cx.new(|cx| {
                    EditorState::new(window, cx)
                        .language("markdown")
                        .folding(false)
                        .default_value(text)
                });
                let view = cx.new(|cx| VisualView::new(editor, window, cx));
                eprintln!(
                    "open: {:?} for {} blocks",
                    started.elapsed(),
                    view.read(cx).doc.blocks.len()
                );
                view.update(cx, |view, cx| window.focus(&view.focus, cx));
                view
            })
            .expect("a window");
            let report_every = env_usize("ENSO_SPIKE_REPORT_EVERY");
            if let Some(every) = report_every {
                let view = view.clone();
                cx.spawn(async move |cx| {
                    let mut last = 0;
                    loop {
                        cx.background_executor()
                            .timer(Duration::from_millis(500))
                            .await;
                        let count = cx.update(|cx| view.read(cx).latency.borrow().samples.len());
                        if count >= last + every {
                            last = count;
                            cx.update(|cx| report(view.read(cx)));
                        }
                    }
                })
                .detach();
            }
            let Some(count) = env_usize("ENSO_SPIKE_AUTOTYPE") else {
                return;
            };
            let caret = env_usize("ENSO_SPIKE_CARET").unwrap_or(len / 2);
            cx.spawn(async move |cx| {
                cx.background_executor()
                    .timer(Duration::from_millis(1500))
                    .await;
                let _ = handle.update(cx, |_, window, cx| {
                    view.update(cx, |view, cx| {
                        // Snap to a leaf: the caret only rests where text renders.
                        let (b, l, i) = view.doc.locate(caret).expect("a text block");
                        view.caret = view.doc.src_of(b, l, i);
                        {
                            let mut l = view.latency.borrow_mut();
                            *l = Default::default();
                        }
                        cx.notify();
                    });
                    window.refresh();
                });
                cx.background_executor()
                    .timer(Duration::from_millis(300))
                    .await;
                cx.update(|cx| *view.read(cx).latency.borrow_mut() = Default::default());
                let keys = ["a", "b", "space", "c", "d", "e", "space"];
                for i in 0..count {
                    let key = keys[i % keys.len()];
                    let _ = handle.update(cx, |_, window, cx| {
                        window.dispatch_keystroke(Keystroke::parse(key).unwrap(), cx);
                    });
                    cx.background_executor()
                        .timer(Duration::from_millis(50))
                        .await;
                }
                cx.background_executor()
                    .timer(Duration::from_millis(300))
                    .await;
                cx.update(|cx| {
                    report(view.read(cx));
                    cx.quit();
                });
            })
            .detach();
        });
}
