// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The application's identity: the icon, compiled in from `assets/branding` so an installed
//! window carries it without an asset directory beside the binary, and the desktop name the
//! compositor matches it by.
//!
//! The window manager asks for pixels before the first frame exists, so no vector is
//! rasterised here: the 256 px plate is decoded from the same PNG the packaging installs. The
//! SVG beside it is the source of every size, and `assets/branding/README.md` says how they
//! are made and where the drawing comes from.

use eframe::egui;

/// The name the window announces as its desktop identity.
///
/// A Wayland compositor does not take an icon from the client: it looks up
/// `<app_id>.desktop` and reads that entry's `Icon=`. So this string is not a label, it is the
/// file name of [`packaging/org.punchisoft.PunchiFluidLab.desktop`], and the test below refuses
/// to compile the two apart. X11 gets the same string as its window class, which is why
/// `StartupWMClass` in that entry repeats it.
pub const APP_ID: &str = "org.punchisoft.PunchiFluidLab";

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

    /// A compositor finds the icon through the window's desktop name, so the entry's own file
    /// name is that name and the `app_id` and the entry cannot drift apart: the path is compiled
    /// in, and a rename that forgets one of them fails to build here.
    #[test]
    fn the_desktop_entry_is_named_after_the_desktop_name_the_window_announces() {
        let entry = include_str!("../../../packaging/org.punchisoft.PunchiFluidLab.desktop");
        assert_eq!(APP_ID, "org.punchisoft.PunchiFluidLab");
        assert!(
            entry.contains("Icon=punchi-fluid-lab"),
            "the entry names the icon the packaging installs"
        );
        assert!(
            entry.contains(&format!("StartupWMClass={APP_ID}")),
            "the entry is matched by the same name the window announces"
        );
    }
}
