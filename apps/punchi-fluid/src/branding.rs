// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The application icon, compiled in from `assets/branding` so an installed window carries it
//! without an asset directory beside the binary.
//!
//! The window manager asks for pixels before the first frame exists, so no vector is
//! rasterised here: the 256 px plate is decoded from the same PNG the packaging installs. The
//! SVG beside it is the source of every size, and `assets/branding/README.md` says how they
//! are made and where the drawing comes from.

use eframe::egui;

/// The one size the window manager gets: 256 px is what the desktops scale down from, and a
/// multiple of four, which [`egui::IconData`] asks for.
const ICON_PNG: &[u8] = include_bytes!("../../../assets/branding/punchi-fluid-lab-256.png");

/// The window's icon, decoded from the compiled-in plate.
///
/// `tiny_skia` hands the pixels premultiplied and egui wants them straight, so a pixel of
/// partial alpha is divided back; a fully opaque or fully transparent one is already equal,
/// and those are all but the antialiased arc of the plate's corners. A plate that does not
/// decode is a build fault the test below catches, and the window opens without an icon
/// rather than refusing to start.
pub fn window_icon() -> egui::IconData {
    let Ok(pixmap) = tiny_skia::Pixmap::decode_png(ICON_PNG) else {
        return egui::IconData::default();
    };
    let mut rgba = pixmap.data().to_vec();
    for pixel in rgba.chunks_exact_mut(4) {
        let alpha = u32::from(pixel[3]);
        if alpha != 0 && alpha != 255 {
            for channel in &mut pixel[..3] {
                let straight = u32::from(*channel) * 255 / alpha;
                *channel = u8::try_from(straight).unwrap_or(255);
            }
        }
    }
    egui::IconData {
        rgba,
        width: pixmap.width(),
        height: pixmap.height(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bytes in the tree are the icon: the right square, a cut corner where the plate
    /// rounds off and an opaque middle. A regenerated plate that lost its alpha or its size
    /// fails here instead of showing a broken window icon on someone's desktop.
    #[test]
    fn the_compiled_in_plate_is_the_256_squircle_the_window_shows() {
        let icon = window_icon();
        assert_eq!((icon.width, icon.height), (256, 256));
        assert_eq!(icon.rgba.len(), 256 * 256 * 4);
        let alpha = |x: usize, y: usize| icon.rgba[(y * 256 + x) * 4 + 3];
        assert_eq!(alpha(0, 0), 0, "the top-left corner is cut by the rounding");
        assert_eq!(
            alpha(255, 255),
            0,
            "the bottom-right corner is cut by it too"
        );
        assert_eq!(alpha(128, 128), 255, "the middle of the plate is opaque");
    }
}
