//! Caret blink. Settings (color, thickness, blink pattern) arrive in Phase 2;
//! the spike blinks at a fixed rate and paints a 2 px bar in the theme color.

use std::time::Duration;

use gpui_kit::{Context, Task};

use super::view::VisualView;

const BLINK: Duration = Duration::from_millis(530);

pub struct Caret {
    visible: bool,
    epoch: usize,
    _task: Option<Task<()>>,
}

impl Caret {
    pub fn new(cx: &mut Context<VisualView>) -> Self {
        let _ = cx;
        Self {
            visible: true,
            epoch: 0,
            _task: None,
        }
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    /// Shows the caret solid and restarts the blink, after any caret move.
    pub fn reset(&mut self, cx: &mut Context<VisualView>) {
        self.visible = true;
        self.epoch += 1;
        if std::env::var_os("ENSO_SPIKE_NO_BLINK").is_some() {
            return;
        }
        let epoch = self.epoch;
        self._task = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(BLINK).await;
                let alive = this.update(cx, |view, cx| {
                    if view.caret_blink.epoch != epoch {
                        return false;
                    }
                    view.caret_blink.visible = !view.caret_blink.visible;
                    cx.notify();
                    true
                });
                if !matches!(alive, Ok(true)) {
                    break;
                }
            }
        }));
    }
}
