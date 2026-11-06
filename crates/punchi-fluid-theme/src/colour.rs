// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Colour arithmetic: WCAG contrast, blending and the repair of unreadable pairs.
//!
//! Every literal colour in the application lives behind this crate. A widget only
//! ever sees a [`egui::Color32`] that came from a token, never a constructor.

use egui::Color32;

/// Builds an opaque colour from the channels of a definition file or a default.
#[must_use]
pub const fn rgb(red: u8, green: u8, blue: u8) -> Color32 {
    Color32::from_rgb(red, green, blue)
}

/// Mixes two colours by `amount` (`0` keeps `from`, `1` returns `to`).
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn blend(from: Color32, to: Color32, amount: f32) -> Color32 {
    let mix = |a: u8, b: u8| {
        (f32::from(a) + (f32::from(b) - f32::from(a)) * amount.clamp(0.0, 1.0))
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Color32::from_rgb(
        mix(from.r(), to.r()),
        mix(from.g(), to.g()),
        mix(from.b(), to.b()),
    )
}

/// Relative luminance as defined by WCAG 2.2.
#[must_use]
pub fn luminance(colour: Color32) -> f32 {
    let channel = |value: u8| {
        let value = f32::from(value) / 255.0;
        if value <= 0.040_45 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(colour.r()) + 0.7152 * channel(colour.g()) + 0.0722 * channel(colour.b())
}

/// Contrast ratio between two opaque colours, from `1.0` to `21.0`.
#[must_use]
pub fn contrast(first: Color32, second: Color32) -> f32 {
    let (lighter, darker) = {
        let (a, b) = (luminance(first), luminance(second));
        if a >= b { (a, b) } else { (b, a) }
    };
    (lighter + 0.05) / (darker + 0.05)
}

/// Moves `foreground` towards black or white until it reaches `minimum` against
/// `background`.
///
/// A desktop theme is user input: it can pair grey text with a grey window. Repairing
/// the pair keeps the interface readable without abandoning the user's hues, and the
/// contrast test measures the result instead of trusting the source.
#[must_use]
pub fn ensure_contrast(foreground: Color32, background: Color32, minimum: f32) -> Color32 {
    if contrast(foreground, background) >= minimum {
        return foreground;
    }

    let mut best = foreground;
    let mut best_ratio = contrast(foreground, background);
    for step in 1..=20_u8 {
        #[allow(clippy::cast_possible_truncation)]
        let amount = f32::from(step) / 20.0;
        for target in [Color32::BLACK, Color32::WHITE] {
            let candidate = blend(foreground, target, amount);
            let ratio = contrast(candidate, background);
            if ratio >= minimum {
                return candidate;
            }
            if ratio > best_ratio {
                best = candidate;
                best_ratio = ratio;
            }
        }
    }
    best
}

/// Black or white, whichever reads better on `background`.
#[must_use]
pub fn readable_on(background: Color32) -> Color32 {
    if contrast(Color32::BLACK, background) >= contrast(Color32::WHITE, background) {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contrast_of_black_and_white_is_the_maximum() {
        let ratio = contrast(Color32::BLACK, Color32::WHITE);
        assert!((ratio - 21.0).abs() < 0.1, "ratio was {ratio}");
    }

    #[test]
    fn contrast_is_symmetric() {
        let a = rgb(36, 36, 36);
        let b = rgb(252, 252, 252);
        assert!((contrast(a, b) - contrast(b, a)).abs() < f32::EPSILON);
    }

    #[test]
    fn repair_reaches_the_minimum_without_changing_a_working_pair() {
        let surface = rgb(36, 36, 36);
        let text = rgb(252, 252, 252);
        assert_eq!(ensure_contrast(text, surface, 4.5), text);

        let muted = rgb(60, 60, 60);
        let repaired = ensure_contrast(muted, surface, 4.5);
        assert!(contrast(repaired, surface) >= 4.5);
    }

    #[test]
    fn blending_between_a_colour_and_itself_is_identity() {
        let colour = rgb(49, 91, 239);
        assert_eq!(blend(colour, colour, 0.5), colour);
        assert_eq!(blend(colour, Color32::BLACK, 0.0), colour);
        assert_eq!(blend(colour, Color32::BLACK, 1.0), Color32::BLACK);
    }
}
