// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The dialog palette: a fixed light and dark pair, kept out of the theme adapter's file so
//! its 800-line budget stays a real limit.

use eframe::egui::Color32;

use super::{Tokens, WHITE};

/// The approved dialog palette: a fixed light and dark pair, so the two pilot windows keep
/// the tones of the approved design on every desktop instead of following its palette.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DialogPalette {
    pub surface: Color32,
    pub band: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub border: Color32,
    pub accent: Color32,
    pub on_accent: Color32,
    pub selected: Color32,
    pub error: Color32,
    pub shadow: Color32,
    pub backdrop: Color32,
}

impl Tokens {
    /// The dialog palette for this light or dark mode.
    #[must_use]
    pub(crate) fn dialog_palette(&self) -> DialogPalette {
        let rgb = Color32::from_rgb;
        if self.dark {
            DialogPalette {
                surface: rgb(35, 45, 58),
                band: rgb(27, 36, 48),
                text: rgb(238, 243, 251),
                muted: rgb(189, 201, 218),
                border: rgb(88, 106, 128),
                accent: rgb(169, 202, 255),
                on_accent: rgb(20, 46, 84),
                selected: rgb(44, 67, 97),
                error: rgb(255, 180, 178),
                shadow: Color32::from_black_alpha(48),
                backdrop: Color32::from_black_alpha(60),
            }
        } else {
            DialogPalette {
                surface: WHITE.colour(),
                band: rgb(244, 246, 250),
                text: rgb(24, 38, 57),
                muted: rgb(80, 96, 118),
                border: rgb(196, 204, 216),
                accent: rgb(34, 87, 163),
                on_accent: WHITE.colour(),
                selected: rgb(230, 239, 254),
                error: rgb(160, 45, 41),
                shadow: Color32::from_black_alpha(48),
                backdrop: Color32::from_black_alpha(60),
            }
        }
    }
}
