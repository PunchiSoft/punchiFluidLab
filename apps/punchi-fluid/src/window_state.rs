// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The window's size and place: the first geometry, the remembered one and its file.
//!
//! The shell sizes itself from the screen it finds, never from a remembered resolution:
//! [`first_geometry`] and [`restored_geometry`] take the monitor's size in device-independent
//! points, so the same functions answer for a 1366 laptop, a 4K monitor or a synthetic screen
//! in a test. The toolkit reports the monitor, not the work area, so a reserve for the title
//! bar and the taskbar or panel is taken from the height and a screen that cannot hold the
//! preferred size opens maximized, which lets the compositor fit the real work area. The
//! remembered geometry lives beside the recent list in a small `key=value`
//! file the reader parses defensively: bounded size, unknown keys ignored, anything
//! unreadable ignored, because a stale file must never keep a window off-screen or below the
//! floor. There is no serde here and no new dependency.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use eframe::egui::{Pos2, Rect, Vec2};

/// The window's minimum content size: the design support floor of `docs/ui-ux.md` §2.2. A
/// screen whose room is smaller lowers the floor to the room, so no window state hangs over
/// the taskbar.
pub const MIN_WIDTH: f32 = 900.0;
pub const MIN_HEIGHT: f32 = 600.0;

/// The size the window opens with when the room has it.
pub const PREFERRED_WIDTH: f32 = 1440.0;
pub const PREFERRED_HEIGHT: f32 = 900.0;

/// The screen-aware margin around the window, in spacing units: one unit on each side, two in
/// total. The room also takes [`CHROME_RESERVE`] from the height.
pub const SCREEN_MARGIN_UNITS: f32 = 2.0;

/// The room the toolkit does not report, in points: the title bar plus the taskbar or panel.
/// The desktop hands egui the monitor's size and nothing else, so this reserve is the honest
/// approximation of `desktopAvailableWidth/Height`. It covers Windows 10 and 11 (title 31-32,
/// taskbar 40-48) and KDE Breeze (title 30, panel 44); the exact work area needs the platform.
pub const CHROME_RESERVE: f32 = 80.0;

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
    SCREEN_MARGIN_UNITS * edge(unit)
}

/// The room a window may take, in points: the monitor minus one spacing unit on each side and
/// the chrome reserve at the bottom. The rectangle's origin is one unit in from the monitor's
/// top-left corner.
fn room(screen: Vec2, unit: f32) -> Vec2 {
    let screen = usable(screen);
    Vec2::new(
        (screen.x - margin(unit)).max(0.0),
        (screen.y - margin(unit) - CHROME_RESERVE).max(0.0),
    )
}

/// The window's floor: the design minimum, unless the room is smaller; then the room is the
/// floor. A screen can never make the window bigger than the room it has.
fn floor(room: Vec2) -> Vec2 {
    Vec2::new(
        MIN_WIDTH.min(room.x).max(1.0),
        MIN_HEIGHT.min(room.y).max(1.0),
    )
}

/// The minimum content size the viewport must enforce on `screen`: 900 × 600 when the room
/// holds it, the room itself when the screen is smaller (1366 × 768 at 125 % has 502 points of
/// height). Sending this to the viewport keeps unmaximizing from hiding the status bar under
/// the taskbar.
#[must_use]
pub fn minimum(screen: Vec2, unit: f32) -> Vec2 {
    floor(room(screen, unit))
}

/// The first geometry: 1440 × 900 when the room has it, else the largest size that fits inside
/// the room, never below its floor, centred. A screen whose room is shorter than the preferred
/// 900 points opens maximized, so the compositor fits the work area instead of the shell
/// guessing the taskbar.
#[must_use]
pub fn first_geometry(screen: Vec2, unit: f32) -> Geometry {
    let room = room(screen, unit);
    let floor = floor(room);
    let size = clamp_size(Vec2::new(PREFERRED_WIDTH, PREFERRED_HEIGHT), room, floor);
    Geometry {
        size,
        position: Some(centred(room, size, unit)),
        maximized: room.y < PREFERRED_HEIGHT,
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

/// The geometry the viewport is created with, before the screen is known: the remembered size
/// and maximized state, or the support floor. The place is applied on the first frame, not
/// here, so a window saved on a second monitor does not move the opening anchor away from the
/// primary the disconnect fallback returns to.
#[must_use]
pub fn provisional(remembered: Option<&Geometry>) -> Geometry {
    remembered.copied().unwrap_or(Geometry {
        size: Vec2::new(MIN_WIDTH, MIN_HEIGHT),
        position: None,
        maximized: false,
    })
}

/// The geometry that brings a live window back into the room when it is outside it: a window
/// left on an extension monitor that was unplugged. `None` when the window still lands on the
/// room or when the desktop reports no place (Wayland, where the compositor places it).
#[must_use]
pub fn reanchored(screen: Vec2, unit: f32, live: &Geometry) -> Option<Geometry> {
    let position = live.position?;
    let room = room(screen, unit);
    if lands_on_screen(position, live.size, room, unit) {
        return None;
    }
    Some(restored_geometry(screen, unit, live))
}

/// The geometry to reopen with: the remembered size clamped to the current screen's room with
/// the same margin, reserve and minimum, and the remembered place only when it still lands on
/// that room, else centred. A maximized window keeps the size it had before, carried in
/// `saved.size`.
///
/// The desktop reports the current monitor's size but not the virtual desktop's origin, so a
/// place that only intersects the room from its origin is kept; a window saved on a monitor
/// placed to the left of the primary therefore reopens centred rather than possibly out of
/// sight. That is the conservative answer for the undocked laptop this is for.
#[must_use]
pub fn restored_geometry(screen: Vec2, unit: f32, saved: &Geometry) -> Geometry {
    let room = room(screen, unit);
    let floor = floor(room);
    let size = clamp_size(saved.size, room, floor);
    let position = match saved.position {
        Some(position) if lands_on_screen(position, size, room, unit) => {
            Some(clamp_position(position, size, room, unit))
        }
        _ => Some(centred(room, size, unit)),
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

/// Sends the minimum the viewport must enforce on the screen it found. Sent before the size,
/// so a screen smaller than 900 × 600 never has to clamp a window to a floor it cannot hold.
pub fn apply_minimum(context: &eframe::egui::Context, minimum: Vec2) {
    context.send_viewport_cmd(eframe::egui::ViewportCommand::MinInnerSize(minimum));
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
    let text = crate::state_file::read_bounded(path, MAX_BYTES)?;
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
    crate::state_file::write_atomically(path, &text)
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

/// A screen size that can be reasoned about: finite and positive.
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

/// One spacing unit as a length, or zero when the unit is not usable.
fn edge(unit: f32) -> f32 {
    if unit.is_finite() && unit > 0.0 {
        unit
    } else {
        0.0
    }
}

fn clamp_size(size: Vec2, room: Vec2, floor: Vec2) -> Vec2 {
    Vec2::new(
        size.x.max(floor.x).min(room.x.max(floor.x)),
        size.y.max(floor.y).min(room.y.max(floor.y)),
    )
}

/// Centres the window in the room, whose origin is one unit in from the monitor's corner.
fn centred(room: Vec2, size: Vec2, unit: f32) -> Pos2 {
    let edge = edge(unit);
    Pos2::new(
        edge + ((room.x - size.x) * 0.5).max(0.0),
        edge + ((room.y - size.y) * 0.5).max(0.0),
    )
}

fn lands_on_screen(position: Pos2, size: Vec2, room: Vec2, unit: f32) -> bool {
    let edge = edge(unit);
    let window = Rect::from_min_size(position, size);
    let visible = window.intersect(Rect::from_min_size(Pos2::new(edge, edge), room));
    visible.width() >= MIN_VISIBLE && visible.height() >= MIN_VISIBLE
}

/// Keeps the window inside the room, so its bottom stays above the taskbar and its top-left
/// keeps the margin.
fn clamp_position(position: Pos2, size: Vec2, room: Vec2, unit: f32) -> Pos2 {
    let edge = edge(unit);
    let max = Pos2::new(
        (edge + room.x - size.x).max(edge),
        (edge + room.y - size.y).max(edge),
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
