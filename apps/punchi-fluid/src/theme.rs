// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The window's theme: colour, type and motion taken from the desktop.
//!
//! The Qt window reads its palette from the platform palette, which has already merged
//! `kdeglobals` and the appearance portal. The Rust window has no platform theme to ask,
//! so it reads the same `kdeglobals` the desktop writes, derives the tokens with the same
//! rules (contrast floors, two planes, softened accent), and applies them to egui. A
//! machine without that file starts on the light fallback the Qt adapter also carries.
//!
//! Everything here is read once at start; nothing may ask the theme per frame.

use std::collections::HashMap;
use std::path::PathBuf;

use eframe::egui;
use egui::{
    Color32, CornerRadius, FontDefinitions, FontFamily, FontId, Stroke, TextStyle, Visuals,
};

const WHITE: Rgb = Rgb::new(255, 255, 255);
const BLACK: Rgb = Rgb::new(0, 0, 0);

/// A colour parsed from the desktop file, before egui sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Rgb {
    const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    fn colour(self) -> Color32 {
        Color32::from_rgb(self.r, self.g, self.b)
    }

    fn with_alpha(self, alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(self.r, self.g, self.b, alpha)
    }
}

mod appearance;

pub use appearance::{Appearance, ThemeId};

/// The palette the window's tokens are derived from: the desktop's or a built-in theme's.
#[derive(Debug, Clone, Copy)]
struct Palette {
    window: Rgb,
    window_text: Rgb,
    base: Rgb,
    button: Rgb,
    highlight: Rgb,
    highlighted_text: Rgb,
    link: Rgb,
}

/// The font the desktop asks for, before it is loaded.
#[derive(Debug, Clone)]
struct DesktopFont {
    family: Option<String>,
    point_size: f32,
}

impl Default for DesktopFont {
    fn default() -> Self {
        Self {
            family: None,
            point_size: 10.0,
        }
    }
}

/// What the window paints with. Derived from the desktop palette, never from a literal in a
/// widget.
#[allow(
    dead_code,
    reason = "the ribbon slice reads the tokens the sheet does not"
)]
#[derive(Debug, Clone)]
pub struct Tokens {
    pub dark: bool,
    pub background: Color32,
    pub panel: Color32,
    /// The sheet's paper: the plane the drawing and the frame sit on.
    pub paper: Color32,
    pub surface_raised: Color32,
    pub surface_sunken: Color32,
    pub ink: Color32,
    pub ink_dim: Color32,
    pub ink_muted: Color32,
    pub accent: Color32,
    pub on_accent: Color32,
    pub accent_soft: Color32,
    pub line: Color32,
    pub line_strong: Color32,
    pub shadow: Color32,
    pub supply: Color32,
    pub work: Color32,
    pub grid_minor: Color32,
    pub grid_major: Color32,
    pub unit: f32,
    pub radius: f32,
    pub radius_control: f32,
    pub radius_small: f32,
    pub radius_tip: f32,
    pub text_small: f32,
    pub text_body: f32,
    pub text_label: f32,
    pub text_display: f32,
    pub motion_ms: u32,
}

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

/// The pairings `docs/ui-ux.md` fixes and their measured ratios, proved by the tests.
#[cfg(test)]
#[derive(Debug, Clone, Copy)]
pub struct Contrast {
    pub text: f64,
    pub dim: f64,
    pub accent: f64,
    pub raised_text: f64,
    pub raised_dim: f64,
    pub line: f64,
    pub line_raised: f64,
    pub accent_soft_text: f64,
    pub accent_text: f64,
    pub sheet_edge: f64,
    pub paper_sunken: f64,
    pub paper_text: f64,
}

/// The theme read once per process: the tokens, the appearance they came from, and the desktop
/// font and motion that a preview needs to derive another palette with the same sizes.
#[derive(Debug, Clone)]
pub struct Theme {
    pub tokens: Tokens,
    /// The appearance these tokens were built from.
    pub appearance: Appearance,
    font_family: Option<String>,
    font: DesktopFont,
    motion: u32,
}

impl Theme {
    /// Reads the desktop's `kdeglobals` once and derives the tokens for `appearance`. A
    /// machine without the file gets the light fallback for the desktop and the theme's own
    /// palette for a built-in.
    #[must_use]
    pub fn for_appearance(appearance: Appearance, day: ThemeId, night: ThemeId) -> Self {
        let text = kdeglobals_path().and_then(|path| std::fs::read_to_string(path).ok());
        let desktop = text.as_deref().and_then(palette_from);
        let desktop_dark = desktop.is_some_and(|palette| relative_luminance(palette.window) < 0.35);
        let palette = match appearance {
            Appearance::Desktop => desktop.unwrap_or_else(fallback_palette),
            Appearance::System => {
                if desktop_dark {
                    night.palette()
                } else {
                    day.palette()
                }
            }
            Appearance::Theme(id) => id.palette(),
        };
        let font = text.as_deref().map_or_else(DesktopFont::default, font_from);
        let motion = text.as_deref().map_or(180, motion_from);
        let tokens = tokens_from(palette, &font, motion);
        Self {
            tokens,
            appearance,
            font_family: font.family.clone(),
            font,
            motion,
        }
    }

    /// The tokens `appearance` would have with this window's font and motion: what a theme
    /// card shows before the theme is applied.
    #[must_use]
    pub fn preview(&self, appearance: Appearance) -> Tokens {
        match appearance {
            Appearance::Desktop | Appearance::System => self.tokens.clone(),
            Appearance::Theme(id) => tokens_from(id.palette(), &self.font, self.motion),
        }
    }

    /// The light fallback, for tests and for a machine with no desktop file.
    #[cfg(test)]
    #[must_use]
    pub fn fallback() -> Self {
        let font = DesktopFont::default();
        Self {
            tokens: tokens_from(fallback_palette(), &font, 180),
            appearance: Appearance::Desktop,
            font_family: None,
            font,
            motion: 180,
        }
    }

    #[must_use]
    pub fn font_family(&self) -> Option<&str> {
        self.font_family.as_deref()
    }

    /// Applies the tokens to egui: visuals and the text sizes every widget inherits.
    pub fn apply(&self, context: &egui::Context) {
        let tokens = &self.tokens;
        context.set_theme(if tokens.dark {
            egui::ThemePreference::Dark
        } else {
            egui::ThemePreference::Light
        });

        let mut visuals = if tokens.dark {
            Visuals::dark()
        } else {
            Visuals::light()
        };
        visuals.override_text_color = Some(tokens.ink);
        visuals.panel_fill = tokens.background;
        visuals.window_fill = tokens.background;
        visuals.extreme_bg_color = tokens.panel;
        visuals.faint_bg_color = tokens.surface_raised;
        visuals.hyperlink_color = tokens.supply;
        visuals.selection.bg_fill = tokens.accent;
        visuals.selection.stroke = Stroke::new(1.0_f32, tokens.accent_soft);
        visuals.window_stroke = Stroke::new(1.0_f32, tokens.line);
        visuals.window_corner_radius = corner(tokens.radius);
        visuals.menu_corner_radius = corner(tokens.radius_control);
        visuals.widgets.noninteractive = widget(
            tokens.background,
            tokens.background,
            tokens.line,
            tokens.ink,
            tokens.radius_small,
        );
        visuals.widgets.inactive = widget(
            tokens.surface_raised,
            tokens.surface_raised,
            tokens.line,
            tokens.ink,
            tokens.radius_control,
        );
        visuals.widgets.hovered = widget(
            tokens.accent_soft,
            tokens.accent_soft,
            tokens.line_strong,
            tokens.ink,
            tokens.radius_control,
        );
        visuals.widgets.active = widget(
            tokens.accent,
            tokens.accent,
            tokens.accent,
            tokens.on_accent,
            tokens.radius_control,
        );
        visuals.widgets.open = widget(
            tokens.accent_soft,
            tokens.accent_soft,
            tokens.line_strong,
            tokens.ink,
            tokens.radius_control,
        );
        context.set_visuals(visuals);

        let mut style = (*context.style()).clone();
        style
            .text_styles
            .insert(TextStyle::Small, FontId::proportional(tokens.text_small));
        style
            .text_styles
            .insert(TextStyle::Body, FontId::proportional(tokens.text_body));
        style
            .text_styles
            .insert(TextStyle::Button, FontId::proportional(tokens.text_body));
        style.text_styles.insert(
            TextStyle::Heading,
            FontId::proportional(tokens.text_display),
        );
        style
            .text_styles
            .insert(TextStyle::Monospace, FontId::monospace(tokens.text_body));
        context.set_style(style);
    }
}

/// Installs the desktop's sans-serif face in egui.
///
/// The interface rule says colour, size and font come from the desktop, and the toolkit
/// ships none of them: egui's default fonts are disabled so no font licence travels with the
/// window. The export crate picks its face the same way, so the window's text and the
/// printed page agree.
///
/// # Errors
///
/// Returns an I/O error when the machine has no font database entry that can be read.
pub fn install_font(context: &egui::Context, family: Option<&str>) -> Result<(), std::io::Error> {
    let mut database = fontdb::Database::new();
    database.load_system_fonts();
    let mut named: Vec<fontdb::Family<'_>> = Vec::new();
    if let Some(family) = family {
        named.push(fontdb::Family::Name(family));
    }
    named.push(fontdb::Family::SansSerif);
    named.push(fontdb::Family::Name("DejaVu Sans"));
    named.push(fontdb::Family::Name("Noto Sans"));
    named.push(fontdb::Family::Name("Liberation Sans"));
    let id = database
        .query(&fontdb::Query {
            families: &named,
            weight: fontdb::Weight::NORMAL,
            style: fontdb::Style::Normal,
            stretch: fontdb::Stretch::Normal,
        })
        .or_else(|| database.faces().next().map(|face| face.id))
        .ok_or_else(|| std::io::Error::other("no system font is installed"))?;
    let face = database
        .face(id)
        .ok_or_else(|| std::io::Error::other("the system font database has no face"))?;
    let data = match &face.source {
        fontdb::Source::Binary(binary) => binary.as_ref().as_ref().to_vec(),
        fontdb::Source::File(path) => std::fs::read(path)?,
        fontdb::Source::SharedFile(_path, binary) => binary.as_ref().as_ref().to_vec(),
    };
    let name = String::from("desktop");
    let mut definitions = FontDefinitions::default();
    definitions.font_data.insert(
        name.clone(),
        std::sync::Arc::new(egui::FontData {
            font: std::borrow::Cow::Owned(data),
            index: face.index,
            tweak: egui::FontTweak::default(),
        }),
    );
    definitions
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, name.clone());
    definitions
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, name);
    context.set_fonts(definitions);
    Ok(())
}

fn corner(radius: f32) -> CornerRadius {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the radii are design values from 3 to 8"
    )]
    CornerRadius::same(radius.round().max(0.0) as u8)
}

fn widget(
    background: Color32,
    weak: Color32,
    line: Color32,
    ink: Color32,
    radius: f32,
) -> egui::style::WidgetVisuals {
    egui::style::WidgetVisuals {
        bg_fill: background,
        weak_bg_fill: weak,
        bg_stroke: Stroke::new(1.0_f32, line),
        corner_radius: corner(radius),
        fg_stroke: Stroke::new(1.0_f32, ink),
        expansion: 0.0,
    }
}

fn kdeglobals_path() -> Option<PathBuf> {
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    let path = config.join("kdeglobals");
    path.is_file().then_some(path)
}

/// Splits the INI file into lower-cased sections of lower-cased keys.
fn groups(text: &str) -> HashMap<String, HashMap<String, String>> {
    let mut groups: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut current = String::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some(section) = line
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
        {
            current = section.trim().to_lowercase();
            groups.entry(current.clone()).or_default();
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            groups
                .entry(current.clone())
                .or_default()
                .insert(key.trim().to_lowercase(), value.trim().to_string());
        }
    }
    groups
}

fn value<'a>(
    groups: &'a HashMap<String, HashMap<String, String>>,
    section: &str,
    key: &str,
) -> Option<&'a str> {
    groups.get(section)?.get(key).map(String::as_str)
}

fn colour(
    groups: &HashMap<String, HashMap<String, String>>,
    section: &str,
    key: &str,
) -> Option<Rgb> {
    parse_colour(value(groups, section, key)?)
}

fn parse_colour(text: &str) -> Option<Rgb> {
    let mut parts = text.split(',');
    let r = parts.next()?.trim().parse::<u8>().ok()?;
    let g = parts.next()?.trim().parse::<u8>().ok()?;
    let b = parts.next()?.trim().parse::<u8>().ok()?;
    Some(Rgb::new(r, g, b))
}

fn palette_from(text: &str) -> Option<Palette> {
    let groups = groups(text);
    let window = colour(&groups, "colors:window", "backgroundnormal")?;
    let window_text = colour(&groups, "colors:window", "foregroundnormal")?;
    let base = colour(&groups, "colors:view", "backgroundnormal").unwrap_or(window);
    let button = colour(&groups, "colors:button", "backgroundnormal").unwrap_or(window);
    let highlight = colour(&groups, "colors:selection", "backgroundnormal")?;
    let highlighted_text = colour(&groups, "colors:selection", "foregroundnormal").unwrap_or(WHITE);
    let link = colour(&groups, "colors:window", "foregroundlink")
        .or_else(|| colour(&groups, "colors:view", "foregroundlink"))
        .unwrap_or(highlight);
    Some(Palette {
        window,
        window_text,
        base,
        button,
        highlight,
        highlighted_text,
        link,
    })
}

fn fallback_palette() -> Palette {
    Palette {
        window: Rgb::new(242, 242, 242),
        window_text: Rgb::new(26, 26, 26),
        base: Rgb::new(255, 255, 255),
        button: Rgb::new(239, 239, 239),
        highlight: Rgb::new(49, 91, 239),
        highlighted_text: WHITE,
        link: Rgb::new(26, 102, 204),
    }
}

fn font_from(text: &str) -> DesktopFont {
    let groups = groups(text);
    let Some(value) = value(&groups, "general", "font") else {
        return DesktopFont::default();
    };
    let mut parts = value.split(',');
    let family = parts
        .next()
        .map(str::trim)
        .filter(|family| !family.is_empty())
        .map(str::to_owned);
    let point_size = parts
        .next()
        .and_then(|size| size.trim().parse::<f32>().ok())
        .filter(|size| *size > 0.0)
        .unwrap_or(10.0);
    DesktopFont { family, point_size }
}

fn motion_from(text: &str) -> u32 {
    let groups = groups(text);
    let Some(value) = value(&groups, "kde", "animationdurationfactor") else {
        return 180;
    };
    let Ok(factor) = value.parse::<f64>() else {
        return 180;
    };
    if factor < 0.0 {
        return 180;
    }
    if factor == 0.0 {
        return 0;
    }
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the factor is a small non-negative number"
    )]
    let duration = (180.0 * factor.min(2.0)).round() as u32;
    duration
}

fn channel(value: f64) -> f64 {
    if value <= 0.039_28 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn relative_luminance(colour: Rgb) -> f64 {
    0.2126 * channel(f64::from(colour.r) / 255.0)
        + 0.7152 * channel(f64::from(colour.g) / 255.0)
        + 0.0722 * channel(f64::from(colour.b) / 255.0)
}

fn contrast_ratio(a: Rgb, b: Rgb) -> f64 {
    let first = relative_luminance(a);
    let second = relative_luminance(b);
    let lighter = first.max(second);
    let darker = first.min(second);
    (lighter + 0.05) / (darker + 0.05)
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the channel is clamped to 0..=255 before the cast"
)]
fn round_channel(value: f64) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

fn blend(from: Rgb, towards: Rgb, amount: f64) -> Rgb {
    let t = amount.clamp(0.0, 1.0);
    Rgb::new(
        round_channel(f64::from(from.r) + (f64::from(towards.r) - f64::from(from.r)) * t),
        round_channel(f64::from(from.g) + (f64::from(towards.g) - f64::from(from.g)) * t),
        round_channel(f64::from(from.b) + (f64::from(towards.b) - f64::from(from.b)) * t),
    )
}

fn lighten(colour: Rgb) -> Rgb {
    Rgb::new(
        round_channel(f64::from(colour.r) * 1.04),
        round_channel(f64::from(colour.g) * 1.04),
        round_channel(f64::from(colour.b) * 1.04),
    )
}

fn darken(colour: Rgb) -> Rgb {
    Rgb::new(
        round_channel(f64::from(colour.r) / 1.04),
        round_channel(f64::from(colour.g) / 1.04),
        round_channel(f64::from(colour.b) / 1.04),
    )
}

/// `colour` moved away from `background` until it reaches `minimum`.
fn ensure_contrast(colour: Rgb, background: Rgb, minimum: f64) -> Rgb {
    if contrast_ratio(colour, background) >= minimum {
        return colour;
    }
    let lighten_side = relative_luminance(background) < 0.35;
    let mut adjusted = colour;
    for _ in 0..120 {
        let next = if lighten_side {
            lighten(adjusted)
        } else {
            darken(adjusted)
        };
        if next == adjusted {
            break;
        }
        adjusted = next;
        if contrast_ratio(adjusted, background) >= minimum {
            break;
        }
    }
    adjusted
}

/// A plane below the window: a dark desktop is pressed down harder than a light one.
fn plane_below(background: Rgb) -> Rgb {
    let amount = if relative_luminance(background) < 0.35 {
        0.18
    } else {
        0.05
    };
    blend(background, BLACK, amount)
}

/// The accent used as a fill behind text: softened towards the surface until the ink reads.
fn soften(accent: Rgb, surface: Rgb, ink: Rgb, minimum: f64) -> Rgb {
    let mut result = blend(accent, surface, 0.82);
    for _ in 0..24 {
        if contrast_ratio(ink, result) >= minimum {
            break;
        }
        result = blend(result, surface, 0.35);
    }
    result
}

fn unit_from(font: &DesktopFont) -> f32 {
    let pixels = font.point_size * 4.0 / 3.0;
    let unit = (pixels * 1.2).max(16.0);
    (unit / 2.0).round() * 2.0
}

fn tokens_from(palette: Palette, font: &DesktopFont, motion: u32) -> Tokens {
    let background = palette.window;
    let ink = ensure_contrast(palette.window_text, background, 4.5);
    let mut surface_raised = palette.button;
    if contrast_ratio(surface_raised, background) < 1.08 {
        surface_raised = blend(background, ink, 0.07);
    }
    let surface_sunken = plane_below(background);
    let dark = relative_luminance(background) < 0.35;
    // Breeze dark makes `[Colors:View]` equal to `[Colors:Window]` (36,36,36), and some
    // dark schemes make it darker than the window. The sheet is the raised plane either
    // way, so the paper is lifted towards the ink when the desktop does not put it above
    // the chrome. A light scheme keeps the white view colour: there the canvas and the
    // edge do the separating.
    let mut paper = palette.base;
    let paper_is_above = relative_luminance(paper) > relative_luminance(background);
    if dark && (!paper_is_above || contrast_ratio(paper, background) < 1.10) {
        paper = blend(background, ink, 0.11);
    }
    // The secondary inks must read on both the paper and the raised plane: in a light
    // scheme the raised plane is the darker of the two and therefore the harder one, while
    // in a dark scheme it is the other way round. The second pass only moves the colour
    // further from the ink's side, so it cannot undo the first.
    let ink_dim = ensure_contrast(
        ensure_contrast(blend(ink, background, 0.30), paper, 3.0),
        surface_raised,
        3.0,
    );
    let ink_muted = ensure_contrast(
        ensure_contrast(blend(ink, background, 0.55), paper, 3.0),
        surface_raised,
        3.0,
    );
    let accent = ensure_contrast(palette.highlight, background, 3.0);
    let mut on_accent = palette.highlighted_text;
    if contrast_ratio(on_accent, accent) < 4.5 {
        on_accent = if contrast_ratio(WHITE, accent) >= contrast_ratio(BLACK, accent) {
            WHITE
        } else {
            BLACK
        };
    }
    let accent_soft = soften(accent, surface_raised, ink, 4.5);
    let line_strong = ensure_contrast(blend(ink, background, 0.45), surface_raised, 3.0);
    let line = blend(ink, background, 0.62);
    let shadow = if dark {
        Color32::from_black_alpha(90)
    } else {
        Color32::from_black_alpha(40)
    };
    let supply = ensure_contrast(palette.link, background, 3.0);
    let unit = unit_from(font);
    let text_small = (unit * 0.55).max(10.0).round();
    let text_body = (unit * 0.61).max(11.0).round();
    let text_label = (unit * 0.66).max(12.0).round();
    let text_display = (unit * 1.50).max(24.0).round();
    Tokens {
        dark,
        background: background.colour(),
        panel: palette.base.colour(),
        paper: paper.colour(),
        surface_raised: surface_raised.colour(),
        surface_sunken: surface_sunken.colour(),
        ink: ink.colour(),
        ink_dim: ink_dim.colour(),
        ink_muted: ink_muted.colour(),
        accent: accent.colour(),
        on_accent: on_accent.colour(),
        accent_soft: accent_soft.colour(),
        line: line.colour(),
        line_strong: line_strong.colour(),
        shadow,
        supply: supply.colour(),
        work: accent.colour(),
        grid_minor: ink_muted.with_alpha(56),
        grid_major: ink_muted.with_alpha(87),
        unit,
        radius: 8.0,
        radius_control: 6.0,
        radius_small: 4.0,
        radius_tip: 3.0,
        text_small,
        text_body,
        text_label,
        text_display,
        motion_ms: motion,
    }
}

impl Tokens {
    /// The pairings `docs/ui-ux.md` fixes, measured on the tokens.
    #[cfg(test)]
    #[must_use]
    pub fn contrast(&self) -> Contrast {
        let background = self.background;
        let raised = self.surface_raised;
        let ratio = |a: Color32, b: Color32| contrast_ratio(from_color32(a), from_color32(b));
        Contrast {
            text: ratio(self.ink, background),
            dim: ratio(self.ink_muted, background),
            accent: ratio(self.accent, background),
            raised_text: ratio(self.ink, raised),
            raised_dim: ratio(self.ink_muted, raised),
            line: ratio(self.line_strong, background),
            line_raised: ratio(self.line_strong, raised),
            accent_soft_text: ratio(self.ink, self.accent_soft),
            accent_text: ratio(self.on_accent, self.accent),
            sheet_edge: ratio(self.line_strong, self.surface_sunken),
            paper_sunken: ratio(self.paper, self.surface_sunken),
            paper_text: ratio(self.ink, self.paper),
        }
    }
}

#[cfg(test)]
fn from_color32(colour: Color32) -> Rgb {
    Rgb::new(colour.r(), colour.g(), colour.b())
}

#[cfg(test)]
mod tests;
