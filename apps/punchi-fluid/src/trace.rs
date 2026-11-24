// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Bounded CSV tracing for complete window-frame CPU time.

use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;

const BATCH_SIZE: usize = 256;

pub(crate) fn startup_args() -> (Option<PathBuf>, bool) {
    parse_args(std::env::args_os().skip(1))
}

fn parse_args(args: impl IntoIterator<Item = std::ffi::OsString>) -> (Option<PathBuf>, bool) {
    let mut path = None;
    let mut enabled = false;
    for argument in args {
        if argument == "--trace-frames" {
            enabled = true;
        } else if path.is_none() {
            path = Some(PathBuf::from(argument));
        }
    }
    (path, enabled)
}

pub(crate) fn install(window: &mut crate::Window, enabled: bool) {
    if enabled {
        match FrameTrace::new(std::io::stdout()) {
            Ok(trace) => window.frame_trace = Some(trace),
            Err(cause) => eprintln!("frame trace could not start: {cause}"),
        }
    }
}

pub(crate) fn update(window: &mut crate::Window, context: &eframe::egui::Context) {
    if window.frame_trace.is_none() {
        window.frame(context);
        return;
    }
    let started = std::time::Instant::now();
    window.frame(context);
    let trace_failed = window
        .frame_trace
        .as_mut()
        .is_some_and(|trace| trace.record(started.elapsed()).is_err());
    if trace_failed {
        window.frame_trace = None;
    }
}

pub(crate) fn finish(window: &mut crate::Window) {
    if let Some(trace) = window.frame_trace.as_mut() {
        let _ = trace.finish();
    }
}

/// Writes frame durations in bounded batches, so tracing does not grow with the session.
pub(crate) struct FrameTrace<W> {
    writer: W,
    samples_ns: [u64; BATCH_SIZE],
    len: usize,
    frame: u64,
}

impl<W: Write> FrameTrace<W> {
    /// Starts a CSV trace with a stable header.
    pub(crate) fn new(mut writer: W) -> io::Result<Self> {
        writeln!(writer, "frame,duration_ns")?;
        Ok(Self {
            writer,
            samples_ns: [0; BATCH_SIZE],
            len: 0,
            frame: 0,
        })
    }

    /// Adds one measured frame and writes a batch when the fixed buffer is full.
    pub(crate) fn record(&mut self, duration: Duration) -> io::Result<()> {
        #[allow(clippy::cast_possible_truncation)]
        let duration_ns = duration.as_nanos().min(u128::from(u64::MAX)) as u64;
        self.samples_ns[self.len] = duration_ns;
        self.len += 1;
        self.frame = self.frame.saturating_add(1);
        if self.len == BATCH_SIZE {
            self.flush_batch()?;
        }
        Ok(())
    }

    /// Writes any remaining samples when the window closes.
    pub(crate) fn finish(&mut self) -> io::Result<()> {
        self.flush_batch()?;
        self.writer.flush()
    }

    fn flush_batch(&mut self) -> io::Result<()> {
        let first_frame = self.frame - self.len as u64 + 1;
        for (index, duration_ns) in self.samples_ns[..self.len].iter().enumerate() {
            writeln!(
                self.writer,
                "{},{}",
                first_frame + index as u64,
                duration_ns
            )?;
        }
        self.len = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::PathBuf;
    use std::time::Duration;

    use super::{BATCH_SIZE, FrameTrace};

    #[test]
    fn the_trace_is_csv_and_keeps_only_a_fixed_batch() {
        let mut trace = FrameTrace::new(Vec::new()).expect("the in-memory writer accepts a header");
        for _ in 0..=BATCH_SIZE {
            trace
                .record(Duration::from_micros(16))
                .expect("the in-memory writer accepts a sample");
        }
        trace.finish().expect("the in-memory writer flushes");
        let output = String::from_utf8(trace.writer).expect("the CSV is UTF-8");
        assert!(output.starts_with("frame,duration_ns\n"));
        assert!(output.contains("1,16000\n"));
        assert!(output.contains("257,16000\n"));
        assert_eq!(output.lines().count(), BATCH_SIZE + 2);
    }

    #[test]
    fn the_trace_flag_keeps_an_optional_document_path() {
        let (path, enabled) = super::parse_args([
            OsString::from("example.pfluid"),
            OsString::from("--trace-frames"),
        ]);
        assert_eq!(path, Some(PathBuf::from("example.pfluid")));
        assert!(enabled);

        let (path, enabled) = super::parse_args([OsString::from("--trace-frames")]);
        assert_eq!(path, None);
        assert!(enabled);
    }
}
