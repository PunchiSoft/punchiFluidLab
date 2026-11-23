// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The appearance the window wears: the desktop's palette or a built-in theme.
//!
//! A built-in theme is a `Palette` seed, not a literal in a widget (ADR 0042); the theme's
//! derivation and contrast tests are in the parent module.

use super::{Palette, Rgb, WHITE};

/// The appearance the window is wearing: the desktop's palette or a built-in theme.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Appearance {
    /// The desktop's palette, read from `kdeglobals`; the default and what a theme returns to.
    #[default]
    Desktop,
    /// A built-in theme's palette.
    Theme(ThemeId),
}

impl Appearance {
    /// The choices the window offers, in reading order: the desktop, the light engineering
    /// theme and the dark one.
    pub const CHOICES: [Self; 3] = [
        Self::Desktop,
        Self::Theme(ThemeId::Cad),
        Self::Theme(ThemeId::Slate),
    ];

    /// The settings-file value.
    #[must_use]
    pub fn value(self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::Theme(id) => id.id(),
        }
    }

    /// The appearance a settings value names; an unknown value reads as the desktop.
    #[must_use]
    pub fn from_value(value: &str) -> Self {
        match ThemeId::from_id(value) {
            Some(id) => Self::Theme(id),
            None if value == "desktop" => Self::Desktop,
            None => Self::Desktop,
        }
    }

    /// The label the interface shows.
    #[must_use]
    pub fn label_es(self) -> &'static str {
        match self {
            Self::Desktop => "Escritorio",
            Self::Theme(id) => id.label_es(),
        }
    }
}

/// A compiled theme: a palette, never a file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeId {
    /// The light engineering palette: a white paper, a cool grey canvas and a blue accent.
    Cad,
    /// The dark engineering palette: deep slate chrome, a blue accent and a lifted paper.
    Slate,
}

impl ThemeId {
    /// Every built-in theme, in the order the gallery shows them: light, then dark.
    pub const ALL: [Self; 2] = [Self::Cad, Self::Slate];

    /// The stable settings-file id.
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::Cad => "cad",
            Self::Slate => "slate",
        }
    }

    /// The Spanish name the card shows.
    #[must_use]
    pub fn label_es(self) -> &'static str {
        match self {
            Self::Cad => "CAD claro",
            Self::Slate => "Pizarra",
        }
    }

    fn from_id(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|id| id.id() == value)
    }

    pub(super) fn palette(self) -> Palette {
        match self {
            Self::Cad => cad_palette(),
            Self::Slate => slate_palette(),
        }
    }
}

/// The light engineering palette: a white paper over a cool grey canvas, dark navy ink and a
/// blue accent; the same derivation lifts the raised plane and measures every pairing.
fn cad_palette() -> Palette {
    Palette {
        window: Rgb::new(248, 250, 252),
        window_text: Rgb::new(15, 23, 42),
        base: Rgb::new(255, 255, 255),
        button: Rgb::new(255, 255, 255),
        highlight: Rgb::new(29, 78, 216),
        highlighted_text: WHITE,
        link: Rgb::new(37, 99, 235),
    }
}

/// The dark engineering palette: deep slate chrome, a blue accent and a paper lifted off the
/// window by the derivation, the same as the desktop's.
fn slate_palette() -> Palette {
    Palette {
        window: Rgb::new(19, 22, 32),
        window_text: Rgb::new(241, 245, 249),
        base: Rgb::new(27, 33, 46),
        button: Rgb::new(34, 42, 58),
        highlight: Rgb::new(37, 136, 255),
        highlighted_text: WHITE,
        link: Rgb::new(56, 189, 248),
    }
}
