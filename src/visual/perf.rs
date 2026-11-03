//! Performance instrumentation for the visual view.
//!
//! Off unless `ENSO_PERF=1` (on-screen overlay) or `ENSO_PERF_LOG=<path>`
//! (one JSON object per input sample) is set when the view is created, or a
//! benchmark turns it on. When off, every hook is a branch on a `bool`.
//!
//! A sample is one input (key or click) and the frame that shows it:
//!
//! - `handler_ms`: from the input event reaching the view to the end of its
//!   handler (the rope edit, the reparse, the caret update).
//! - `frame_ms`: from the start of that frame's render to the end of the
//!   surface's paint (render, layout, prepaint, paint; CPU only).
//! - `work_ms` = `handler_ms` + `frame_ms`: the work the input cost, compared
//!   with the frame budget.
//! - `wait_ms`: from the end of the handler to the frame start. This is the
//!   wait for the next display frame, which code cannot shorten; it is
//!   reported separately and not part of `work_ms`.
//!
//! GPU rasterization, presentation and the compositor come after the paint
//! and are not included: every number here is CPU-side.

use std::{
    io::Write as _,
    time::{Duration, Instant},
};

use super::document::Path;

#[derive(Clone, Debug)]
pub struct Sample {
    pub label: &'static str,
    pub handler: Duration,
    pub frame: Duration,
    pub wait: Duration,
    pub parsed_bytes: usize,
    pub path: Option<Path>,
    pub blocks_laid_out: usize,
}

impl Sample {
    pub fn work(&self) -> Duration {
        self.handler + self.frame
    }

    pub fn json(&self) -> String {
        format!(
            "{{\"label\":\"{}\",\"work_ms\":{:.4},\"handler_ms\":{:.4},\"frame_ms\":{:.4},\"wait_ms\":{:.4},\"parsed_bytes\":{},\"path\":\"{}\",\"blocks_laid_out\":{}}}",
            self.label,
            ms(self.work()),
            ms(self.handler),
            ms(self.frame),
            ms(self.wait),
            self.parsed_bytes,
            self.path.map_or("none".to_string(), |p| format!("{p:?}")),
            self.blocks_laid_out
        )
    }
}

pub fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.
}

#[derive(Default)]
pub struct Perf {
    pub enabled: bool,
    pub overlay: bool,
    log: Option<std::fs::File>,
    /// What the next sample is labelled with (set by a benchmark driver).
    pub label: &'static str,
    // The input being measured.
    input_at: Option<Instant>,
    handler_done: Option<Instant>,
    frame_start: Option<Instant>,
    parsed_bytes: usize,
    path: Option<Path>,
    blocks_laid_out: usize,
    pub samples: Vec<Sample>,
    /// Every frame's render-to-paint time, for the overlay graph.
    pub frames: Vec<Duration>,
    current_frame: Option<Instant>,
}

impl Perf {
    pub fn from_env() -> Self {
        let overlay = std::env::var_os("ENSO_PERF").is_some_and(|v| v == "1");
        let log = std::env::var_os("ENSO_PERF_LOG").and_then(|path| {
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .ok()
        });
        Self {
            enabled: overlay || log.is_some(),
            overlay,
            log,
            label: "input",
            ..Default::default()
        }
    }

    pub fn input(&mut self) {
        if self.enabled && self.input_at.is_none() {
            self.input_at = Some(Instant::now());
            self.parsed_bytes = 0;
            self.path = None;
        }
    }

    pub fn reparsed(&mut self, bytes: usize, path: Path) {
        if self.enabled {
            self.parsed_bytes += bytes;
            self.path = Some(path);
        }
    }

    pub fn handler_done(&mut self) {
        if self.enabled && self.input_at.is_some() {
            self.handler_done = Some(Instant::now());
        }
    }

    pub fn frame_begin(&mut self) {
        if !self.enabled {
            return;
        }
        let now = Instant::now();
        self.current_frame = Some(now);
        if self.input_at.is_some() && self.frame_start.is_none() {
            self.frame_start = Some(now);
            self.blocks_laid_out = 0;
        }
    }

    pub fn block_laid_out(&mut self) {
        if self.enabled {
            self.blocks_laid_out += 1;
        }
    }

    pub fn frame_end(&mut self) {
        if !self.enabled {
            return;
        }
        let now = Instant::now();
        if let Some(start) = self.current_frame.take() {
            self.frames.push(now - start);
        }
        let (Some(input), Some(frame)) = (self.input_at, self.frame_start) else {
            return;
        };
        let handler_done = self.handler_done.unwrap_or(frame).min(frame);
        let sample = Sample {
            label: self.label,
            handler: handler_done - input,
            frame: now - frame,
            wait: frame - handler_done,
            parsed_bytes: self.parsed_bytes,
            path: self.path,
            blocks_laid_out: self.blocks_laid_out,
        };
        if let Some(log) = &mut self.log {
            let _ = writeln!(log, "{}", sample.json());
        }
        self.samples.push(sample);
        self.input_at = None;
        self.handler_done = None;
        self.frame_start = None;
    }

    /// Frames over `budget` among the recorded ones.
    pub fn over_budget(&self, budget: Duration) -> usize {
        self.frames.iter().filter(|f| **f > budget).count()
    }

    pub fn reset(&mut self) {
        self.samples.clear();
        self.frames.clear();
        self.input_at = None;
        self.handler_done = None;
        self.frame_start = None;
    }
}

/// Percentiles of real per-sample values (never sums of percentiles).
pub fn percentiles(values: &mut [f64]) -> Option<[f64; 5]> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q = |p: f64| values[((values.len() - 1) as f64 * p).round() as usize];
    Some([
        q(0.5),
        q(0.95),
        q(0.99),
        values[values.len() - 1],
        values[0],
    ])
}
