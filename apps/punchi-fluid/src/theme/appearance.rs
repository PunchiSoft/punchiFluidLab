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
    /// The theme assigned to the light scheme by day and to the dark one by night.
    System,
    /// A built-in theme's palette, whatever the desktop scheme.
    Theme(ThemeId),
}

impl Appearance {
    /// The theme cards the gallery offers, in reading order: light, then dark. The desktop
    /// and the system modes are segments of their own, not cards.
    pub const CHOICES: [Self; 4] = [
        Self::Theme(ThemeId::Cad),
        Self::Theme(ThemeId::Slate),
        Self::Theme(ThemeId::Nordic),
        Self::Theme(ThemeId::Indigo),
    ];

    /// The settings-file value.
    #[must_use]
    pub fn value(self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::System => "system",
            Self::Theme(id) => id.id(),
        }
    }

    /// The appearance a settings value names; an unknown value reads as the desktop.
    #[must_use]
    pub fn from_value(value: &str) -> Self {
        match value {
            "desktop" => Self::Desktop,
            "system" => Self::System,
            other => ThemeId::from_id(other).map_or(Self::Desktop, Self::Theme),
        }
    }

    /// The label of the mode segment: a fixed theme reads as "Un tema", because the segment
    /// chooses the mode and the card chooses which theme.
    #[must_use]
    pub fn mode_label_es(self) -> &'static str {
        match self {
            Self::Desktop => "Escritorio",
            Self::System => "Sistema",
            Self::Theme(_) => "Un tema",
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
    /// The high-contrast palette: near-black chrome, pure white ink and a cobalt accent.
    Nordic,
    /// The deep-blue palette: navy chrome, a cobalt accent and a cyan link.
    Indigo,
}

impl ThemeId {
    /// Every built-in theme, in the order the gallery shows them: light, then dark.
    pub const ALL: [Self; 4] = [Self::Cad, Self::Slate, Self::Nordic, Self::Indigo];

    /// The stable settings-file id.
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::Cad => "cad",
            Self::Slate => "slate",
            Self::Nordic => "nordic",
            Self::Indigo => "indigo",
        }
    }

    /// The Spanish name the card shows.
    #[must_use]
    pub fn label_es(self) -> &'static str {
        match self {
            Self::Cad => "CAD claro",
            Self::Slate => "Pizarra",
            Self::Nordic => "Nórdico",
            Self::Indigo => "Azul profundo",
        }
    }

    pub(crate) fn from_id(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|id| id.id() == value)
    }

    pub(super) fn palette(self) -> Palette {
        match self {
            Self::Cad => cad_palette(),
            Self::Slate => slate_palette(),
            Self::Nordic => nordic_palette(),
            Self::Indigo => indigo_palette(),
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

/// The deep-blue palette: navy chrome, deep panels, a cobalt accent and a cyan link; the same
/// derivation lifts the paper and measures every pairing.
fn indigo_palette() -> Palette {
    Palette {
        window: Rgb::new(11, 15, 25),
        window_text: Rgb::new(248, 250, 252),
        base: Rgb::new(18, 24, 38),
        button: Rgb::new(30, 41, 59),
        highlight: Rgb::new(2, 132, 199),
        highlighted_text: WHITE,
        link: Rgb::new(56, 189, 248),
    }
}

/// The high-contrast palette: near-black chrome, pure white ink, a cobalt accent and a paper
/// lifted off the window by the same derivation.
fn nordic_palette() -> Palette {
    Palette {
        window: Rgb::new(13, 13, 17),
        window_text: Rgb::new(255, 255, 255),
        base: Rgb::new(22, 22, 28),
        button: Rgb::new(32, 32, 42),
        highlight: Rgb::new(37, 99, 235),
        highlighted_text: WHITE,
        link: Rgb::new(59, 130, 246),
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
