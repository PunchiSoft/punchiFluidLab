// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The theme layer: the only place allowed to name a colour or a size.
//!
//! The interface follows the desktop it runs on. Colours come from `kdeglobals`
//! and from the `org.freedesktop.appearance color-scheme` portal; a missing file
//! or a missing bus falls back to a built-in palette so the window always looks
//! deliberate. Widgets ask for a [`Token`] and a [`Metrics`] length, never for a
//! literal, and a test scans the workspace to keep it that way.

pub mod colour;
pub mod kdeglobals;
pub mod metrics;
pub mod portal;
pub mod tokens;

use egui::{FontId, TextStyle};

pub use kdeglobals::KdeGlobals;
pub use metrics::Metrics;
pub use portal::SchemeWatcher;
pub use tokens::{Palette, Token};

/// A colour that paints nothing.
///
/// The menu bar and the toolbar are flat: their frames only appear under the pointer, so
/// the transparent fill is a named value of the theme and not a literal in a widget.
pub const NO_PAINT: egui::Color32 = egui::Color32::TRANSPARENT;

/// Colour scheme preferred for the interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Scheme {
    /// Dark background, light text.
    Dark,
    /// Light background, dark text.
    Light,
    /// The desktop did not express a preference; the background decides.
    #[default]
    NoPreference,
}

impl Scheme {
    /// Maps the portal's integer to a scheme.
    #[must_use]
    pub const fn from_portal(value: u32) -> Self {
        match value {
            1 => Self::Dark,
            2 => Self::Light,
            _ => Self::NoPreference,
        }
    }

    /// Uses the inferred scheme when there is no preference.
    #[must_use]
    pub const fn resolve(self, inferred: Self) -> Self {
        match self {
            Self::NoPreference => inferred,
            other => other,
        }
    }

    /// `true` for the dark side, including an unresolved preference.
    #[must_use]
    pub const fn is_dark(self) -> bool {
        matches!(self, Self::Dark | Self::NoPreference)
    }
}

/// Everything a widget may need from the desktop: colours, lengths and easing.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Effective scheme, never [`Scheme::NoPreference`].
    pub scheme: Scheme,
    /// Colour for every token.
    pub palette: Palette,
    /// Family named by the desktop, informational until the font can be loaded.
    pub font_family: Option<String>,
    /// Body text size in points.
    pub font_size: f32,
    /// `0` disables easing without stopping the simulation.
    pub animation_factor: f32,
    /// Lengths derived from the font size.
    pub metrics: Metrics,
}

impl Theme {
    /// Reads the desktop's settings once. Safe to call repeatedly, never per frame.
    #[must_use]
    pub fn detect() -> Self {
        Self::detect_with(Scheme::NoPreference)
    }

    /// Reads the desktop's settings, honouring a scheme from the portal.
    #[must_use]
    pub fn detect_with(scheme: Scheme) -> Self {
        let globals = KdeGlobals::load();
        Self::from_globals(globals.as_ref(), scheme)
    }

    /// Builds a theme from an already parsed file, without touching the disk.
    #[must_use]
    pub fn from_globals(globals: Option<&KdeGlobals>, requested: Scheme) -> Self {
        let inferred = globals.and_then(KdeGlobals::scheme).unwrap_or(Scheme::Dark);
        let scheme = requested.resolve(inferred);
        // Forcing the opposite scheme must not keep the file's colours: a light
        // interface built from a dark palette would be unreadable.
        let palette = if requested == Scheme::NoPreference || requested == inferred {
            Palette::from_globals(globals, scheme)
        } else {
            Palette::fallback(scheme)
        };
        let (font_family, point_size) = match globals.and_then(KdeGlobals::font) {
            Some((family, size)) => (Some(family), size),
            None => (None, 10.0),
        };
        let font_size = (point_size * 4.0 / 3.0).clamp(11.0, 22.0);
        Self {
            scheme,
            palette,
            font_family,
            font_size,
            animation_factor: globals
                .and_then(KdeGlobals::animation_factor)
                .unwrap_or(1.0),
            metrics: Metrics::from_font(font_size),
        }
    }

    /// Built-in theme with no disk access, used as the last fallback and by tests.
    #[must_use]
    pub fn builtin(scheme: Scheme) -> Self {
        let scheme = match scheme {
            Scheme::Light => Scheme::Light,
            Scheme::Dark | Scheme::NoPreference => Scheme::Dark,
        };
        Self {
            scheme,
            palette: Palette::fallback(scheme),
            font_family: None,
            font_size: 13.0,
            animation_factor: 1.0,
            metrics: Metrics::default(),
        }
    }

    /// Colour of one token.
    #[must_use]
    pub const fn colour(&self, token: Token) -> egui::Color32 {
        self.palette.colour(token)
    }

    /// egui visuals for the palette.
    #[must_use]
    pub fn visuals(&self) -> egui::Visuals {
        use egui::{Stroke, Visuals};

        let palette = &self.palette;
        let mut visuals = if self.scheme.is_dark() {
            Visuals::dark()
        } else {
            Visuals::light()
        };
        visuals.dark_mode = self.scheme.is_dark();
        visuals.override_text_color = None;
        visuals.weak_text_color = Some(palette.colour(Token::TextMuted));
        visuals.panel_fill = palette.colour(Token::Surface);
        visuals.window_fill = palette.colour(Token::SurfaceRaised);
        visuals.window_stroke = Stroke::new(self.metrics.stroke, palette.colour(Token::Outline));
        visuals.extreme_bg_color = palette.colour(Token::SurfaceAlt);
        visuals.text_edit_bg_color = Some(palette.colour(Token::SurfaceAlt));
        visuals.faint_bg_color = palette.colour(Token::SurfaceAlt);
        visuals.hyperlink_color = palette.colour(Token::Accent);
        visuals.warn_fg_color = palette.colour(Token::Warning);
        visuals.error_fg_color = palette.colour(Token::Danger);
        visuals.selection.bg_fill = palette.colour(Token::Accent);
        visuals.selection.stroke =
            Stroke::new(self.metrics.stroke, palette.colour(Token::AccentText));
        visuals.window_corner_radius = egui::CornerRadius::same(self.metrics.radius);
        visuals.menu_corner_radius = egui::CornerRadius::same(self.metrics.radius);

        let widgets = &mut visuals.widgets;
        widgets.noninteractive.bg_fill = palette.colour(Token::Surface);
        widgets.noninteractive.weak_bg_fill = palette.colour(Token::Surface);
        widgets.noninteractive.bg_stroke =
            Stroke::new(self.metrics.stroke, palette.colour(Token::Outline));
        widgets.noninteractive.fg_stroke =
            Stroke::new(self.metrics.stroke, palette.colour(Token::Text));
        widgets.inactive.bg_fill = palette.colour(Token::SurfaceAlt);
        widgets.inactive.weak_bg_fill = palette.colour(Token::SurfaceRaised);
        widgets.inactive.bg_stroke =
            Stroke::new(self.metrics.stroke, palette.colour(Token::Outline));
        widgets.inactive.fg_stroke = Stroke::new(self.metrics.stroke, palette.colour(Token::Text));
        widgets.hovered.bg_fill = palette.colour(Token::SurfaceRaised);
        widgets.hovered.weak_bg_fill = palette.blend(Token::SurfaceRaised, Token::Text, 0.08);
        widgets.hovered.bg_stroke = Stroke::new(self.metrics.stroke, palette.colour(Token::Accent));
        widgets.hovered.fg_stroke = Stroke::new(self.metrics.stroke, palette.colour(Token::Text));
        widgets.active.bg_fill = palette.colour(Token::SurfaceRaised);
        widgets.active.weak_bg_fill = palette.blend(Token::SurfaceRaised, Token::Text, 0.16);
        widgets.active.bg_stroke = Stroke::new(self.metrics.stroke, palette.colour(Token::Accent));
        widgets.active.fg_stroke = Stroke::new(self.metrics.stroke, palette.colour(Token::Text));
        widgets.open.bg_fill = palette.colour(Token::SurfaceRaised);
        widgets.open.weak_bg_fill = palette.colour(Token::SurfaceRaised);
        widgets.open.bg_stroke = Stroke::new(self.metrics.stroke, palette.colour(Token::Outline));
        widgets.open.fg_stroke = Stroke::new(self.metrics.stroke, palette.colour(Token::Text));

        visuals
    }

    /// Applies the theme to a context: style, text sizes, easing and the palette
    /// that widgets read back.
    ///
    /// Call it on start and on a theme change, never per frame.
    pub fn apply(&self, context: &egui::Context) {
        let mut style = (*context.global_style()).clone();
        style.visuals = self.visuals();
        style.spacing.item_spacing =
            egui::vec2(self.metrics.grid_unit * 0.45, self.metrics.grid_unit * 0.35);
        style.spacing.button_padding =
            egui::vec2(self.metrics.grid_unit * 0.55, self.metrics.grid_unit * 0.30);
        style.spacing.interact_size =
            egui::vec2(self.metrics.grid_unit * 1.6, self.metrics.grid_unit * 1.33);
        #[allow(clippy::cast_possible_truncation)]
        let margin = (self.metrics.grid_unit * 0.6) as i8;
        style.spacing.window_margin = egui::Margin::same(margin);
        style.spacing.menu_margin = egui::Margin::same(margin);
        style.animation_time = 0.2 * self.animation_factor;
        style.text_styles = [
            (
                TextStyle::Small,
                FontId::proportional(self.font_size * 0.85),
            ),
            (TextStyle::Body, FontId::proportional(self.font_size)),
            (TextStyle::Button, FontId::proportional(self.font_size)),
            (
                TextStyle::Heading,
                FontId::proportional(self.font_size * 1.3),
            ),
            (
                TextStyle::Monospace,
                FontId::monospace(self.font_size * 0.95),
            ),
        ]
        .into();

        let egui_theme = if self.scheme.is_dark() {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        };
        context.set_theme(egui_theme);
        context.set_style_of(egui_theme, style);
        let theme = self.clone();
        context.data_mut(|data| {
            data.insert_temp(theme_id(), theme);
            data.insert_temp(palette_id(), self.palette);
        });
    }
}

/// The theme currently installed in a context.
///
/// Widgets use this instead of reading the disk: a missing installation (a headless
/// test, a widget drawn before the shell starts) falls back to a built-in theme that
/// matches the context's light or dark flag.
#[must_use]
pub fn current(context: &egui::Context) -> Theme {
    if let Some(theme) = context.data(|data| data.get_temp::<Theme>(theme_id())) {
        return theme;
    }
    let scheme = if context.global_style().visuals.dark_mode {
        Scheme::Dark
    } else {
        Scheme::Light
    };
    Theme::builtin(scheme)
}

/// Palette of the theme installed in a context.
#[must_use]
pub fn palette(context: &egui::Context) -> Palette {
    context
        .data(|data| data.get_temp::<Palette>(palette_id()))
        .unwrap_or_else(|| current(context).palette)
}

fn theme_id() -> egui::Id {
    egui::Id::unique("punchi-fluid-theme/theme")
}

fn palette_id() -> egui::Id {
    egui::Id::unique("punchi-fluid-theme/palette")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_portal_wins_and_no_preference_keeps_the_background() {
        assert_eq!(Scheme::Dark.resolve(Scheme::Light), Scheme::Dark);
        assert_eq!(Scheme::Light.resolve(Scheme::Dark), Scheme::Light);
        assert_eq!(Scheme::NoPreference.resolve(Scheme::Light), Scheme::Light);
    }

    #[test]
    fn a_theme_without_a_file_is_still_usable() {
        let theme = Theme::builtin(Scheme::Light);
        assert_eq!(theme.scheme, Scheme::Light);
        assert!(!theme.visuals().dark_mode);
        assert_eq!(
            theme.colour(Token::Surface),
            theme.palette.colour(Token::Surface)
        );
        assert!(theme.metrics.sidebar_width > theme.metrics.sidebar_min);
    }

    #[test]
    fn a_desktop_file_overrides_the_fallback() {
        let globals = KdeGlobals::parse(
            "[Colors:Window]\nBackgroundNormal=250,250,250\nForegroundNormal=20,20,20\n\
             [Colors:Selection]\nBackgroundNormal=30,60,200\n[KDE]\nAnimationDurationFactor=0",
        );
        let theme = Theme::from_globals(Some(&globals), Scheme::NoPreference);
        assert_eq!(theme.scheme, Scheme::Light);
        assert_eq!(
            theme.colour(Token::Surface),
            egui::Color32::from_rgb(250, 250, 250)
        );
        assert!((theme.animation_factor - 0.0).abs() < f32::EPSILON);
        assert_eq!(theme.visuals().panel_fill, theme.colour(Token::Surface));
    }

    #[test]
    fn forcing_light_ignores_a_dark_desktop_palette() {
        let globals = KdeGlobals::parse(
            "[Colors:Window]\nBackgroundNormal=20,20,20\nForegroundNormal=240,240,240",
        );
        let theme = Theme::from_globals(Some(&globals), Scheme::Light);
        assert_eq!(theme.scheme, Scheme::Light);
        assert!(!theme.visuals().dark_mode);
        assert!(colour::luminance(theme.colour(Token::Surface)) > 0.5);
    }

    #[test]
    fn applying_installs_the_theme_for_widgets() {
        let context = egui::Context::default();
        let theme = Theme::builtin(Scheme::Dark);
        theme.apply(&context);
        assert_eq!(current(&context).palette, theme.palette);
        assert_eq!(palette(&context), theme.palette);
    }
}
