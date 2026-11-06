// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The icons of the toolbar.
//!
//! **The desktop's icon theme** — `document-new`, `edit-undo` and the rest, looked up and
//! rasterised by `punchi-fluid-icons`. This is what makes the window look like the rest of
//! the session: a native application does not ship its own pictograms. On a machine with
//! no icon theme the button shows its label alone, which is what a native application does
//! there too.
//!
//! Textures are cached by the ribbon, per pixel size and theme, so a frame uploads
//! nothing unless the icon is new. The cache is owned and passed in rather than parked in
//! the context: asking the context for its data while it is uploading a texture is a
//! deadlock, and this is one lock fewer.

use std::collections::HashMap;

use egui::TextureHandle;
use punchi_fluid_icons::IconTheme;
use punchi_fluid_theme::Theme;

use crate::ribbon::Action;

/// Name of the icon in the freedesktop theme, per action.
///
/// The standard names, so a desktop that ships its own theme answers with its own
/// pictograms.
#[must_use]
#[allow(clippy::match_same_arms)]
pub const fn theme_name(action: Action) -> Option<&'static str> {
    Some(match action {
        Action::New => "document-new",
        Action::Open => "document-open",
        Action::Save => "document-save",
        Action::SaveAs => "document-save-as",
        Action::ExportPng => "image-x-generic",
        Action::ExportPdf => "application-pdf",
        Action::Quit => "application-exit",
        Action::Undo => "edit-undo",
        Action::Redo => "edit-redo",
        Action::Cut => "edit-cut",
        Action::Copy => "edit-copy",
        Action::Paste => "edit-paste",
        Action::Duplicate => "edit-copy",
        Action::Delete => "edit-delete",
        Action::SelectAll => "edit-select-all",
        Action::Rotate => "object-rotate-right",
        Action::ToggleGrid => "view-grid",
        Action::SnapGrid => "snap-grid",
        Action::SnapPorts => "snap-nodes",
        Action::ZoomIn => "zoom-in",
        Action::ZoomOut => "zoom-out",
        Action::ZoomFit => "zoom-fit-best",
        Action::ThemeSystem => "preferences-desktop-color",
        Action::ThemeLight => "weather-clear",
        Action::ThemeDark => "weather-clear-night",
        Action::ReloadTheme => "view-refresh",
        Action::ToggleSidebar => "view-sidetree",
        Action::ToggleStatusBar => "view-statusbar",
        Action::PlayPause => "media-playback-start",
        Action::Step => "media-skip-forward",
        Action::Reset => "view-refresh",
        Action::ActuateValve => "flag",
        Action::SpeedSlower => "media-playback-start",
        Action::SpeedFaster => "media-seek-forward",
        Action::ToggleMode => "system-run",
        Action::ReloadCatalog => "view-refresh",
        Action::FocusSearch => "edit-find",
        Action::Help => "help-contents",
    })
}

/// Textures of the desktop's icons, by name and pixel size.
#[derive(Default)]
pub struct ThemeIcons {
    theme: Option<IconTheme>,
    /// Scheme the theme was resolved for. A change of scheme re-resolves it, because a
    /// light and a dark icon theme are different files.
    dark: Option<bool>,
    textures: HashMap<(&'static str, u16), Option<TextureHandle>>,
}

impl std::fmt::Debug for ThemeIcons {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ThemeIcons")
            .field("theme", &self.theme)
            .field("dark", &self.dark)
            .field("cached", &self.textures.len())
            .finish()
    }
}

impl Clone for ThemeIcons {
    fn clone(&self) -> Self {
        Self {
            theme: self.theme.clone(),
            dark: self.dark,
            textures: self.textures.clone(),
        }
    }
}

impl ThemeIcons {
    /// The texture for `action`, or `None` if the theme has no such icon.
    pub fn texture(
        &mut self,
        context: &egui::Context,
        action: Action,
        px: u16,
        dark: bool,
    ) -> Option<TextureHandle> {
        let name = theme_name(action)?;
        if self.theme.is_some() && self.dark != Some(dark) {
            self.textures.clear();
            self.theme = None;
        }
        if let Some(known) = self.textures.get(&(name, px)) {
            return known.clone();
        }
        self.dark = Some(dark);
        let theme = self
            .theme
            .get_or_insert_with(|| IconTheme::detect_for(dark));
        let handle = theme.render(name, px).map(|icon| {
            let image = egui::ColorImage::from_rgba_unmultiplied(
                [icon.size as usize, icon.size as usize],
                &icon.pixels,
            );
            context.load_texture(
                format!("theme:{name}:{px}"),
                image,
                egui::TextureOptions::LINEAR,
            )
        });
        self.textures.insert((name, px), handle.clone());
        handle
    }
}

/// The icon of `action` as a widget image, sized to the toolbar.
///
/// `None` when the desktop's theme has no such icon: the button then shows its label
/// alone, which is what a native application does on a machine with no icon theme.
pub fn texture_image(
    ui: &egui::Ui,
    action: Action,
    theme: &Theme,
    cache: &mut ThemeIcons,
) -> Option<egui::Image<'static>> {
    let side = theme.metrics.toolbar_icon;
    // An icon is a couple of dozen pixels: the cast is exact and the lint has no way to
    // know it.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let px = (side * ui.ctx().pixels_per_point())
        .round()
        .clamp(8.0, 512.0) as u16;
    let dark = theme.scheme == punchi_fluid_theme::Scheme::Dark;
    let texture = cache.texture(ui.ctx(), action, px, dark)?;
    Some(egui::Image::new(&texture).fit_to_exact_size(egui::vec2(side, side)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_action_has_a_freedesktop_name_or_says_it_has_none() {
        for action in Action::ALL {
            if let Some(name) = theme_name(*action) {
                assert!(
                    !name.is_empty() && !name.contains(char::is_whitespace),
                    "{} is not a freedesktop name: {name}",
                    action.label()
                );
            }
        }
        // The commands a toolbar shows must name an icon: a blank square is a bug.
        for action in [
            Action::New,
            Action::Open,
            Action::Save,
            Action::SaveAs,
            Action::Undo,
            Action::Redo,
            Action::PlayPause,
            Action::Step,
            Action::Reset,
        ] {
            assert!(
                theme_name(action).is_some(),
                "{} must have a theme icon",
                action.label()
            );
        }
    }
}
