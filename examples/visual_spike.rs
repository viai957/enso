//! Opens one Markdown file in the visual editor, outside the app shell.
//!
//! ```sh
//! cargo run --release --example visual_spike -- path/to/file.md
//! ENSO_PERF=1 cargo run --release --example visual_spike -- file.md   # overlay
//! ```

#[path = "../src/visual/mod.rs"]
#[allow(dead_code)]
mod visual;

use gpui_kit::component::input::EditorState;
use gpui_kit::*;
use visual::view::VisualView;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: visual_spike <file.md>");
    let text = std::fs::read_to_string(&path).expect("a UTF-8 file");
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
            gpui_kit::open_window(options, cx, |window, cx| {
                let editor = cx.new(|cx| {
                    EditorState::new(window, cx)
                        .language("markdown")
                        .folding(false)
                        .default_value(text)
                });
                let view = cx.new(|cx| VisualView::new(editor, window, cx));
                view.update(cx, |view, cx| window.focus(&view.focus, cx));
                view
            })
            .expect("a window");
        });
}
