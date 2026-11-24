// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The window's settings: which panels are open, how wide they are, whether auto-hide is on
//! and the appearance, remembered across sessions in a small `key=value` file of its own.
//!
//! `docs/ui-ux.md` §2.1: width and visibility of both panels are **view state**, never the
//! document's and never the undo history's; **Ver ▸ Restablecer vista** returns them to their
//! defaults. The file lives beside the recent list, at
//! `<XDG_DATA_HOME>/punchi-fluid/settings.state`, and its parser is as defensive as
//! `window_state`'s: bounded size and lines, unknown keys ignored, an unknown version rejected,
//! an out-of-range width ignored rather than plotted, and every write atomic. There is no
//! serde here and no new dependency.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use eframe::egui;
use punchi_fluid_i18n::Language;

use crate::Window;
use crate::layout::LayoutMode;
use crate::theme::{Appearance, ThemeId};

/// The file's size bound, its line bound and the name it lives under, beside the recent list.
pub const MAX_BYTES: usize = 4_096;
pub const FILE_NAME: &str = "settings.state";

/// How long the panels must stay still before the file is written, so a width drag does not
/// write once per frame.
pub const SETTLE: Duration = Duration::from_millis(500);

/// The format's version; a future format writes another one, and a file from it is ignored.
const VERSION: &str = "1";

/// The lines the parser reads, a bound on a file that is bounded already.
const MAX_LINES: usize = 32;

/// The width range a file may carry. A value outside it is ignored, not clamped: 24 points is
/// the rail and 4000 is already wider than any panel or window this interface offers.
pub const MIN_WIDTH: f32 = crate::panel::MIN_RAIL;
pub const MAX_WIDTH: f32 = 4_000.0;

/// One panel's remembered state: whether it is open and the width the reader gave it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanelState {
    pub open: bool,
    pub width: f32,
}

impl PanelState {
    fn is_valid(self) -> bool {
        self.width.is_finite() && (MIN_WIDTH..=MAX_WIDTH).contains(&self.width)
    }
}

/// Everything the settings file remembers: both panels, the auto-hide flag, the appearance
/// and the two themes the system mode uses. The defaults
/// are the `Restablecer vista` values: both panels open at their preferred widths and
/// auto-hide off.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    pub catalog: PanelState,
    pub inspector: PanelState,
    /// **Ver ▸ Ocultar al salir** is on: the rail opens after a dwell and the panel closes
    /// after the pointer leaves it.
    pub autohide: bool,
    /// The appearance the window wears: the desktop's palette or a built-in theme.
    pub appearance: Appearance,
    /// The theme the system mode uses when the desktop is light.
    pub theme_day: ThemeId,
    /// The theme the system mode uses when the desktop is dark.
    pub theme_night: ThemeId,
    /// The visible language; English is the source and the default.
    pub language: Language,
    /// Whether the floating view strip is folded to one button in the wide and medium modes.
    pub strip_folded: bool,
    /// Whether the sheet draws its title block on screen; the print keeps it either way.
    pub block_visible: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            catalog: PanelState {
                open: true,
                width: crate::sidebar::DEFAULT_WIDTH,
            },
            inspector: PanelState {
                open: true,
                width: crate::inspector::DEFAULT_WIDTH,
            },
            autohide: false,
            appearance: Appearance::Desktop,
            theme_day: ThemeId::Cad,
            theme_night: ThemeId::Slate,
            language: Language::English,
            strip_folded: false,
            block_visible: true,
        }
    }
}

impl Settings {
    /// The same settings with both widths clamped to the room `mode` leaves beside the sheet's
    /// floor: a file written on a wide screen cannot make a panel wider than the mode the
    /// window opened in can hold. The open flag and the auto-hide flag travel unchanged.
    #[must_use]
    pub fn clamped_to(self, mode: LayoutMode, available: f32) -> Self {
        let bounds = mode.max_widths(available);
        Self {
            catalog: PanelState {
                width: self.catalog.width.min(bounds.catalog),
                ..self.catalog
            },
            inspector: PanelState {
                width: self.inspector.width.min(bounds.dock),
                ..self.inspector
            },
            ..self
        }
    }
}

/// The file the panel settings live in: the user data folder beside the recent list, or the
/// folder the recents' environment override points at. `None` keeps them in memory only.
#[must_use]
pub fn default_path() -> Option<PathBuf> {
    let recents = punchi_fluid_session::recents::default_path()?;
    Some(recents.parent()?.join(FILE_NAME))
}

/// Reads the remembered settings; `None` when the file is absent, too large, unreadable or
/// from another format version.
#[must_use]
pub fn read(path: &Path) -> Option<Settings> {
    let text = crate::state_file::read_bounded(path, MAX_BYTES)?;
    parse(&text)
}

/// Writes the settings best-effort through a temporary file in the same folder: a full disk
/// costs the memory, never the reader's work, and a half-written file never replaces a good
/// one. A failure leaves the previous file untouched.
pub fn write(path: &Path, settings: &Settings) -> bool {
    let Some(text) = format(settings) else {
        return false;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    crate::state_file::write_atomically(path, &text)
}

/// Parses the file's text. The version must be the one this format writes; anything else
/// rejects the file whole, because half-reading a future format is worse than forgetting it.
/// A broken or out-of-range value drops only that value, and unknown keys are ignored on
/// purpose: the format is additive.
#[must_use]
pub fn parse(text: &str) -> Option<Settings> {
    let mut settings = Settings::default();
    let mut version = None;
    for line in text.lines().take(MAX_LINES) {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "version" => version = Some(value),
            "catalog.open" => {
                if let Some(open) = flag(value) {
                    settings.catalog.open = open;
                }
            }
            "catalog.width" => {
                if let Some(width) = width(value) {
                    settings.catalog.width = width;
                }
            }
            "inspector.open" => {
                if let Some(open) = flag(value) {
                    settings.inspector.open = open;
                }
            }
            "inspector.width" => {
                if let Some(width) = width(value) {
                    settings.inspector.width = width;
                }
            }
            "autohide" => {
                if let Some(autohide) = flag(value) {
                    settings.autohide = autohide;
                }
            }
            "appearance" => settings.appearance = Appearance::from_value(value),
            "theme_day" => {
                if let Some(id) = ThemeId::from_id(value) {
                    settings.theme_day = id;
                }
            }
            "theme_night" => {
                if let Some(id) = ThemeId::from_id(value) {
                    settings.theme_night = id;
                }
            }
            "strip_folded" => {
                if let Some(folded) = flag(value) {
                    settings.strip_folded = folded;
                }
            }
            "block_visible" => {
                if let Some(visible) = flag(value) {
                    settings.block_visible = visible;
                }
            }
            "language" => {
                if let Some(language) = Language::from_code(value) {
                    settings.language = language;
                }
            }
            // Unknown keys are ignored on purpose: the format is additive.
            _ => {}
        }
    }
    (version == Some(VERSION)).then_some(settings)
}

/// Formats the settings for the file, `None` when a width cannot be trusted.
#[must_use]
pub fn format(settings: &Settings) -> Option<String> {
    if !settings.catalog.is_valid() || !settings.inspector.is_valid() {
        return None;
    }
    let mut text = format!(
        "# Punchi Fluid Lab window settings. Written by the program; safe to delete.\nversion={VERSION}\n"
    );
    let _ = write!(
        text,
        "catalog.open={}\ncatalog.width={:.3}\ninspector.open={}\ninspector.width={:.3}\nautohide={}\nappearance={}\ntheme_day={}\ntheme_night={}\nlanguage={}\nstrip_folded={}\nblock_visible={}\n",
        number(settings.catalog.open),
        settings.catalog.width,
        number(settings.inspector.open),
        settings.inspector.width,
        number(settings.autohide),
        settings.appearance.value(),
        settings.theme_day.id(),
        settings.theme_night.id(),
        settings.language.code(),
        number(settings.strip_folded),
        number(settings.block_visible),
    );
    Some(text)
}

fn flag(value: &str) -> Option<bool> {
    match value {
        "1" | "true" => Some(true),
        "0" | "false" => Some(false),
        _ => None,
    }
}

fn number(value: bool) -> u8 {
    u8::from(value)
}

fn width(value: &str) -> Option<f32> {
    value.parse::<f32>().ok().filter(|width| {
        PanelState {
            open: true,
            width: *width,
        }
        .is_valid()
    })
}

/// The settings' file, read once, with the pending write the shell triggers when the panels
/// have stopped moving. A store without a path keeps the settings in memory, which is the
/// honest degraded mode of a machine with no user data folder.
#[derive(Debug)]
pub struct Store {
    path: Option<PathBuf>,
    saved: Settings,
    /// Whether the file was found and read at start, so the window knows it has a remembered
    /// width to clamp to the mode it opens in.
    found: bool,
    pending: Option<Settings>,
    changed_at: Option<Instant>,
}

impl Store {
    /// Reads the settings from the default place.
    #[must_use]
    pub fn load() -> Self {
        Self::at(default_path())
    }

    /// Reads the settings from `path`; `None` keeps them in memory only.
    #[must_use]
    pub fn at(path: Option<PathBuf>) -> Self {
        let loaded = path.as_deref().and_then(read);
        Self {
            found: loaded.is_some(),
            saved: loaded.unwrap_or_default(),
            path,
            pending: None,
            changed_at: None,
        }
    }

    /// The settings the file held at start, or the defaults without a file.
    #[must_use]
    pub fn remembered(&self) -> &Settings {
        &self.saved
    }

    /// Whether a settings file was found and read at start.
    #[must_use]
    pub fn found(&self) -> bool {
        self.found
    }

    /// Notes the live settings and writes them once they have settled for [`SETTLE`]. Returns
    /// whether the file was written on this call.
    pub fn observe(&mut self, settings: Settings, now: Instant) -> bool {
        if self.saved == settings {
            self.pending = None;
            self.changed_at = None;
            return false;
        }
        if self.pending != Some(settings) {
            self.pending = Some(settings);
            self.changed_at = Some(now);
            return false;
        }
        let settled = self
            .changed_at
            .is_some_and(|at| now.duration_since(at) >= SETTLE);
        settled && self.write_pending()
    }

    /// Whether a change is waiting for the settle delay, so the frame loop can wake for it
    /// even when the reader has stopped touching the window.
    #[must_use]
    pub fn is_pending(&self) -> bool {
        self.pending.is_some()
    }

    /// Writes the settings now, whatever their age: the close path, the reset and the
    /// auto-hide toggle use it so the last change survives even if the window closes before
    /// the settle delay.
    pub fn remember(&mut self, settings: Settings) -> bool {
        if self.saved == settings {
            self.pending = None;
            self.changed_at = None;
            return false;
        }
        self.pending = Some(settings);
        self.write_pending()
    }

    fn write_pending(&mut self) -> bool {
        let Some(settings) = self.pending.take() else {
            return false;
        };
        self.changed_at = None;
        let Some(path) = self.path.as_deref() else {
            // No folder to write to: the settings still live in memory, so the window does not
            // keep retrying a write that cannot happen.
            self.saved = settings;
            return false;
        };
        let written = write(path, &settings);
        if written {
            self.saved = settings;
        }
        written
    }
}

impl Window {
    /// A window that carries the panel settings the reader's last session left: both panels
    /// open or folded as the file says, at the width it remembers, and auto-hide on when the
    /// flag was on.
    pub(crate) fn with_settings(mut self, store: Store) -> Self {
        let remembered = *store.remembered();
        self.sidebar
            .restore(remembered.catalog.open, remembered.catalog.width);
        self.inspector
            .restore(remembered.inspector.open, remembered.inspector.width);
        self.autohide_on = remembered.autohide;
        self.appearance_day = remembered.theme_day;
        self.appearance_night = remembered.theme_night;
        self.language = remembered.language;
        self.shell.restore_pending = store.found();
        self.settings = store;
        self
    }

    /// The live panel settings: what the panels show right now, plus the auto-hide flag.
    pub(crate) fn panel_settings(&self) -> Settings {
        Settings {
            catalog: PanelState {
                open: self.sidebar.is_open(),
                width: self.sidebar.width(),
            },
            inspector: PanelState {
                open: self.inspector.is_open(),
                width: self.inspector.width(),
            },
            autohide: self.autohide_on,
            appearance: self.theme.appearance,
            theme_day: self.appearance_day,
            theme_night: self.appearance_night,
            language: self.language,
            strip_folded: self.strip.folded,
            block_visible: self.show_block,
        }
    }

    /// Applies the widths a settings file left once the layout mode is known: a width above
    /// the mode's maximum is clamped. It runs once, on the first frame that is not the
    /// compact one, because the drawer keeps the width the reader gave it.
    pub(crate) fn restore_panels_to_mode(&mut self, context: &egui::Context) {
        if !self.shell.restore_pending || self.shell.mode == LayoutMode::Compact {
            return;
        }
        let restored = self
            .panel_settings()
            .clamped_to(self.shell.mode, context.screen_rect().width());
        self.sidebar.set_width(restored.catalog.width);
        self.inspector.set_width(restored.inspector.width);
        self.shell.restore_pending = false;
    }

    /// Notes the live panel settings and lets the store write them once they have settled:
    /// every fold and every width drag goes through here, never through the document.
    pub(crate) fn observe_panel_settings(&mut self, now: Instant) -> bool {
        let settings = self.panel_settings();
        self.settings.observe(settings, now)
    }

    /// Writes the panel settings now: the close path, so a fold or a width made just before
    /// the window closed survives.
    pub(crate) fn flush_panel_settings(&mut self) {
        let settings = self.panel_settings();
        self.settings.remember(settings);
    }

    /// **Ver ▸ Ocultar al salir**: turns auto-hide on or off and remembers the answer at once.
    /// The menu's label does not change; the flag is read by the timers in [`crate::auto_hide`].
    pub(crate) fn toggle_autohide(&mut self) {
        self.autohide_on = !self.autohide_on;
        self.auto_hide.cancel_all();
        let settings = self.panel_settings();
        self.settings.remember(settings);
    }
}

#[cfg(test)]
mod tests;
