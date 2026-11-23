// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The window's size and place: the first geometry, the remembered one and its file.
//!
//! The shell sizes itself from the screen it finds, never from a remembered resolution:
//! [`first_geometry`] and [`restored_geometry`] take the monitor's size in device-independent
//! points, so the same functions answer for a 1366 laptop, a 4K monitor or a synthetic screen
//! in a test. The remembered geometry lives beside the recent list in a small `key=value`
//! file the reader parses defensively: bounded size, unknown keys ignored, anything
//! unreadable ignored, because a stale file must never keep a window off-screen or below the
//! floor. There is no serde here and no new dependency.

use std::fmt::Write as _;
use std::io::{Read, Write as _};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use eframe::egui::{Pos2, Rect, Vec2};

/// The window's minimum content size: the support floor of `docs/ui-ux.md` §2.2. A smaller
/// screen cannot make the window smaller than this, only let it hang over the edge.
pub const MIN_WIDTH: f32 = 900.0;
pub const MIN_HEIGHT: f32 = 600.0;

/// The size the window opens with when the screen has room for it.
pub const PREFERRED_WIDTH: f32 = 1440.0;
pub const PREFERRED_HEIGHT: f32 = 900.0;

/// The screen-aware margin around the window, in spacing units: one unit on each side, two
/// in total, so a 1366×768 screen opens the window about 1330×732 at 18 points per unit.
pub const SCREEN_MARGIN_UNITS: f32 = 2.0;

/// How much of the window has to land on the screen for a remembered place to be kept.
/// Anything smaller is treated as a place on a monitor that is no longer there.
pub const MIN_VISIBLE: f32 = 64.0;

/// The biggest width or height the file may carry, and the biggest absolute coordinate. They
/// bound a corrupt or hostile file: the value is ignored, not interpreted as a window larger
/// than any monitor.
pub const MAX_DIMENSION: f32 = 100_000.0;
pub const MAX_COORDINATE: f32 = 1_000_000.0;

/// The file's size bound and the name it lives under, beside the recent list.
pub const MAX_BYTES: usize = 4_096;
pub const FILE_NAME: &str = "window.state";

/// How long the geometry must stay still before it is written, so a drag does not write the
/// file once per frame.
pub const SETTLE: Duration = Duration::from_millis(500);

/// The format's version; a future format writes another one, and an older file is ignored.
const VERSION: &str = "1";

/// The lines the parser reads, a bound on a file that is bounded already.
const MAX_LINES: usize = 32;

/// One window geometry, in device-independent points: the content size, where the window's
/// top-left corner sits in monitor space when the desktop reports it, and whether it is
/// maximized.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geometry {
    pub size: Vec2,
    pub position: Option<Pos2>,
    pub maximized: bool,
}

/// The two-unit margin around a window, in points, for a spacing unit.
#[must_use]
pub fn margin(unit: f32) -> f32 {
    let unit = if unit.is_finite() && unit > 0.0 {
        unit
    } else {
        0.0
    };
    SCREEN_MARGIN_UNITS * unit
}

/// The first geometry: 1440 × 900 when the screen has room, else the largest size that fits
/// inside the screen minus the two-unit margin, never below the minimum, centred.
#[must_use]
pub fn first_geometry(screen: Vec2, unit: f32) -> Geometry {
    let screen = usable(screen);
    let size = clamp_size(Vec2::new(PREFERRED_WIDTH, PREFERRED_HEIGHT), screen, unit);
    Geometry {
        size,
        position: Some(centred(screen, size)),
        maximized: false,
    }
}

/// The geometry the window opens with: the first one when there is nothing remembered, the
/// restored one otherwise. One pure function of the screen, the spacing unit and the state,
/// so the whole first-window decision is tested without a window.
#[must_use]
pub fn opening_geometry(screen: Vec2, unit: f32, remembered: Option<&Geometry>) -> Geometry {
    match remembered {
        Some(remembered) => restored_geometry(screen, unit, remembered),
        None => first_geometry(screen, unit),
    }
}

/// The geometry to reopen with: the remembered size clamped to the current screen with the
/// same margin and minimum, and the remembered place only when it still lands on that screen,
/// else centred. A maximized window keeps the size it had before, carried in `saved.size`.
///
/// The desktop reports the current monitor's size but not the virtual desktop's origin, so a
/// place that only intersects the screen rectangle from its origin is kept; a window saved on
/// a monitor placed to the left of the primary therefore reopens centred rather than possibly
/// out of sight. That is the conservative answer for the undocked laptop this is for.
#[must_use]
pub fn restored_geometry(screen: Vec2, unit: f32, saved: &Geometry) -> Geometry {
    let screen = usable(screen);
    let size = clamp_size(saved.size, screen, unit);
    let position = match saved.position {
        Some(position) if lands_on_screen(position, size, screen) => {
            Some(clamp_position(position, size, screen, unit))
        }
        _ => Some(centred(screen, size)),
    };
    Geometry {
        size,
        position,
        maximized: saved.maximized,
    }
}

/// The geometry to remember from what the viewport reported: the normal size and place, even
/// while maximized, so unmaximizing on the next start returns the window it was.
#[must_use]
pub fn remembered_geometry(live: Geometry, normal: Option<&Geometry>) -> Geometry {
    let mut kept = if live.maximized {
        normal.copied().unwrap_or(live)
    } else {
        live
    };
    kept.maximized = live.maximized;
    kept
}

/// Sends the geometry to the viewport. The first frame knows the screen; the commands take
/// effect on the next one.
pub fn apply(context: &eframe::egui::Context, geometry: &Geometry) {
    context.send_viewport_cmd(eframe::egui::ViewportCommand::InnerSize(geometry.size));
    if let Some(position) = geometry.position {
        context.send_viewport_cmd(eframe::egui::ViewportCommand::OuterPosition(position));
    }
    if geometry.maximized {
        context.send_viewport_cmd(eframe::egui::ViewportCommand::Maximized(true));
    }
}

/// The file the geometry lives in: the user data folder beside the recent list, or the
/// folder the recents' environment override points at. `None` keeps it in memory only.
#[must_use]
pub fn default_path() -> Option<PathBuf> {
    let recents = punchi_fluid_session::recents::default_path()?;
    Some(recents.parent()?.join(FILE_NAME))
}

/// Reads the remembered geometry; `None` when the file is absent, too large or unreadable.
#[must_use]
pub fn read(path: &Path) -> Option<Geometry> {
    let text = read_bounded(path)?;
    parse(&text)
}

/// Writes the geometry best-effort through a temporary file in the same folder: a full disk
/// costs the memory, never the reader's work, and a half-written file never replaces a good
/// one. A failure leaves the previous file untouched.
pub fn write(path: &Path, geometry: &Geometry) -> bool {
    let Some(text) = format(geometry) else {
        return false;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    write_atomically(path, &text)
}

/// Writes through a temporary created with `create_new`, flushed, and renamed over the
/// target: a planted symlink at the predictable temporary name is removed first, and a
/// failure never leaves the previous file truncated.
fn write_atomically(path: &Path, text: &str) -> bool {
    let temporary = temporary_path(path);
    let _ = std::fs::remove_file(&temporary);
    let Ok(mut file) = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
    else {
        return false;
    };
    let written = file
        .write_all(text.as_bytes())
        .and_then(|()| file.sync_all())
        .is_ok();
    drop(file);
    if !written {
        let _ = std::fs::remove_file(&temporary);
        return false;
    }
    if std::fs::rename(&temporary, path).is_err() {
        let _ = std::fs::remove_file(&temporary);
        return false;
    }
    true
}

/// The temporary file beside the target, in the same folder so the rename stays on one
/// filesystem.
fn temporary_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    path.with_file_name(name)
}

/// Parses the file's text. A file without a usable size is ignored whole; a broken place is
/// dropped alone, so a truncated position still restores the size and centres the window.
#[must_use]
pub fn parse(text: &str) -> Option<Geometry> {
    let mut width = None;
    let mut height = None;
    let mut x = None;
    let mut y = None;
    let mut maximized = false;
    for line in text.lines().take(MAX_LINES) {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "version" => {
                if value.trim() != VERSION {
                    // A file written by a newer format is ignored, not half-read.
                    return None;
                }
            }
            "width" => width = dimension(value.trim()),
            "height" => height = dimension(value.trim()),
            "x" => x = coordinate(value.trim()),
            "y" => y = coordinate(value.trim()),
            "maximized" => maximized = matches!(value.trim(), "1" | "true"),
            // Unknown keys are ignored on purpose: the format is additive.
            _ => {}
        }
    }
    let size = Vec2::new(width?, height?);
    Some(Geometry {
        size,
        position: match (x, y) {
            (Some(x), Some(y)) => Some(Pos2::new(x, y)),
            _ => None,
        },
        maximized,
    })
}

/// Formats the geometry for the file, `None` when it cannot be trusted.
#[must_use]
pub fn format(geometry: &Geometry) -> Option<String> {
    if !valid_dimension(geometry.size.x, MAX_DIMENSION)
        || !valid_dimension(geometry.size.y, MAX_DIMENSION)
    {
        return None;
    }
    if let Some(position) = geometry.position
        && (!valid_coordinate(position.x) || !valid_coordinate(position.y))
    {
        return None;
    }
    let mut text = format!(
        "# Punchi Fluid Lab window geometry. Written by the program; safe to delete.\nversion={VERSION}\nwidth={:.3}\nheight={:.3}\n",
        geometry.size.x, geometry.size.y
    );
    if let Some(position) = geometry.position {
        let _ = write!(text, "x={:.3}\ny={:.3}\n", position.x, position.y);
    }
    text.push_str(if geometry.maximized {
        "maximized=1\n"
    } else {
        "maximized=0\n"
    });
    Some(text)
}

fn read_bounded(path: &Path) -> Option<String> {
    let file = std::fs::File::open(path).ok()?;
    let mut text = String::new();
    file.take((MAX_BYTES as u64) + 1)
        .read_to_string(&mut text)
        .ok()?;
    (text.len() <= MAX_BYTES).then_some(text)
}

fn dimension(value: &str) -> Option<f32> {
    value
        .parse::<f32>()
        .ok()
        .filter(|value| valid_dimension(*value, MAX_DIMENSION))
}

fn coordinate(value: &str) -> Option<f32> {
    value
        .parse::<f32>()
        .ok()
        .filter(|value| valid_coordinate(*value))
}

fn valid_dimension(value: f32, max: f32) -> bool {
    value.is_finite() && value > 0.0 && value <= max
}

fn valid_coordinate(value: f32) -> bool {
    value.is_finite() && value.abs() <= MAX_COORDINATE
}

/// A screen size that can be reasoned about: finite and at least the window's minimum.
fn usable(screen: Vec2) -> Vec2 {
    Vec2::new(
        if screen.x.is_finite() && screen.x > 0.0 {
            screen.x
        } else {
            MIN_WIDTH
        },
        if screen.y.is_finite() && screen.y > 0.0 {
            screen.y
        } else {
            MIN_HEIGHT
        },
    )
}

fn clamp_size(size: Vec2, screen: Vec2, unit: f32) -> Vec2 {
    let width = (screen.x - margin(unit)).max(MIN_WIDTH);
    let height = (screen.y - margin(unit)).max(MIN_HEIGHT);
    Vec2::new(
        size.x.max(MIN_WIDTH).min(width),
        size.y.max(MIN_HEIGHT).min(height),
    )
}

fn centred(screen: Vec2, size: Vec2) -> Pos2 {
    Pos2::new(
        ((screen.x - size.x) * 0.5).max(0.0),
        ((screen.y - size.y) * 0.5).max(0.0),
    )
}

fn lands_on_screen(position: Pos2, size: Vec2, screen: Vec2) -> bool {
    let window = Rect::from_min_size(position, size);
    let visible = window.intersect(Rect::from_min_size(Pos2::ZERO, screen));
    visible.width() >= MIN_VISIBLE && visible.height() >= MIN_VISIBLE
}

/// Keeps one spacing unit between the window and each screen edge, which is the two-unit
/// margin around both sides.
fn clamp_position(position: Pos2, size: Vec2, screen: Vec2, unit: f32) -> Pos2 {
    let edge = if unit.is_finite() && unit > 0.0 {
        unit
    } else {
        0.0
    };
    let max = Pos2::new(
        (screen.x - size.x - edge).max(edge),
        (screen.y - size.y - edge).max(edge),
    );
    Pos2::new(position.x.clamp(edge, max.x), position.y.clamp(edge, max.y))
}

/// The geometry's file, read once, with the pending write the shell triggers when the window
/// has stopped moving. A store without a path keeps the geometry in memory, which is the
/// honest degraded mode of a machine with no user data folder.
#[derive(Debug)]
pub struct Store {
    path: Option<PathBuf>,
    saved: Option<Geometry>,
    pending: Option<Geometry>,
    changed_at: Option<Instant>,
}

impl Store {
    /// Reads the geometry from the default place.
    #[must_use]
    pub fn load() -> Self {
        Self::at(default_path())
    }

    /// Reads the geometry from `path`; `None` keeps it in memory only.
    #[must_use]
    pub fn at(path: Option<PathBuf>) -> Self {
        let saved = path.as_deref().and_then(read);
        Self {
            path,
            saved,
            pending: None,
            changed_at: None,
        }
    }

    /// The geometry the file held at start, for the shell to clamp to the screen it found.
    #[must_use]
    pub fn remembered(&self) -> Option<&Geometry> {
        self.saved.as_ref()
    }

    /// Notes the live geometry and writes it once it has settled for [`SETTLE`]. Returns
    /// whether the file was written on this call.
    pub fn observe(&mut self, live: Geometry, normal: Option<&Geometry>, now: Instant) -> bool {
        let wanted = remembered_geometry(live, normal);
        if self.saved.as_ref() == Some(&wanted) {
            self.pending = None;
            self.changed_at = None;
            return false;
        }
        if self.pending.as_ref() != Some(&wanted) {
            self.pending = Some(wanted);
            self.changed_at = Some(now);
            return false;
        }
        let settled = self
            .changed_at
            .is_some_and(|at| now.duration_since(at) >= SETTLE);
        settled && self.write_pending()
    }

    /// Writes the geometry now, whatever its age: the close path uses it so the last move
    /// survives even if the reader closed the window before it settled.
    pub fn remember(&mut self, geometry: Geometry) -> bool {
        if self.saved.as_ref() == Some(&geometry) {
            self.pending = None;
            self.changed_at = None;
            return false;
        }
        self.pending = Some(geometry);
        self.write_pending()
    }

    fn write_pending(&mut self) -> bool {
        let Some(geometry) = self.pending.take() else {
            return false;
        };
        self.changed_at = None;
        let written = self
            .path
            .as_deref()
            .is_some_and(|path| write(path, &geometry));
        if written {
            self.saved = Some(geometry);
        }
        written
    }
}

#[cfg(test)]
mod tests;
