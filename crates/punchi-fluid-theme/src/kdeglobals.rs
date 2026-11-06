// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reader for the desktop's own settings file.
//!
//! `kdeglobals` is an INI file written by the Plasma settings module. Only the
//! sections the interface needs are read, and every malformed line or value is
//! ignored rather than fatal: a theme file is untrusted input like any other.

use std::collections::BTreeMap;
use std::path::PathBuf;

use egui::Color32;

use crate::Scheme;

/// Parsed `kdeglobals`, section and key names as written by Qt.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KdeGlobals {
    sections: BTreeMap<String, BTreeMap<String, String>>,
}

impl KdeGlobals {
    /// Parses the text of a `kdeglobals` file. Never fails.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        let mut sections: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
        let mut current = String::new();
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                name.trim().clone_into(&mut current);
                sections.entry(current.clone()).or_default();
                continue;
            }
            if let Some((key, value)) = line.split_once('=')
                && !current.is_empty()
            {
                sections
                    .entry(current.clone())
                    .or_default()
                    .insert(key.trim().to_owned(), value.trim().to_owned());
            }
        }
        Self { sections }
    }

    /// Reads `$XDG_CONFIG_HOME/kdeglobals`, then `~/.config/kdeglobals`.
    ///
    /// A missing file is normal on GNOME or in a container, so it is reported as
    /// `None` and the caller falls back to the portal or to a built-in palette.
    #[must_use]
    pub fn load() -> Option<Self> {
        let path = config_dir()?.join("kdeglobals");
        let text = std::fs::read_to_string(path).ok()?;
        Some(Self::parse(&text))
    }

    /// Raw value of a key.
    #[must_use]
    pub fn value(&self, section: &str, key: &str) -> Option<&str> {
        self.sections.get(section)?.get(key).map(String::as_str)
    }

    /// Colour value of a key, in `r,g,b` or `#rrggbb` form.
    #[must_use]
    pub fn colour(&self, section: &str, key: &str) -> Option<Color32> {
        parse_colour(self.value(section, key)?)
    }

    /// Colour scheme implied by the window background.
    #[must_use]
    pub fn scheme(&self) -> Option<Scheme> {
        let colour = self.colour("Colors:Window", "BackgroundNormal")?;
        Some(if crate::colour::luminance(colour) < 0.5 {
            Scheme::Dark
        } else {
            Scheme::Light
        })
    }

    /// Animation duration scale, `[KDE] AnimationDurationFactor`.
    ///
    /// `0` disables easing everywhere; the value is clamped so a broken file cannot
    /// freeze the interface.
    #[must_use]
    pub fn animation_factor(&self) -> Option<f32> {
        let value = self.value("KDE", "AnimationDurationFactor")?;
        let parsed = value.trim().parse::<f32>().ok()?;
        Some(parsed.clamp(0.0, 2.0))
    }

    /// Interface font as `(family, point size)`, from `[General] font`.
    ///
    /// The value is a Qt font string (`Noto Sans,10,-1,5,50,0,0,0,0,0`): only the
    /// first two fields are meaningful here.
    #[must_use]
    pub fn font(&self) -> Option<(String, f32)> {
        let value = self.value("General", "font")?;
        let mut fields = value.split(',');
        let family = fields.next()?.trim().trim_matches('"').to_owned();
        let size = fields
            .next()
            .and_then(|field| field.trim().parse::<f32>().ok())
            .filter(|size| (5.0..=72.0).contains(size))
            .unwrap_or(10.0);
        if family.is_empty() {
            return None;
        }
        Some((family, size))
    }
}

/// Reads a Qt colour: `r,g,b`, `r,g,b,a`, `#rrggbb` or `#aarrggbb`.
#[must_use]
fn parse_colour(value: &str) -> Option<Color32> {
    let value = value.trim();
    if let Some(hex) = value.strip_prefix('#') {
        return parse_hex(hex);
    }

    let mut channels = value.split(',').map(|field| field.trim().parse::<u8>());
    let red = channels.next()?.ok()?;
    let green = channels.next()?.ok()?;
    let blue = channels.next()?.ok()?;
    Some(Color32::from_rgb(red, green, blue))
}

fn parse_hex(hex: &str) -> Option<Color32> {
    let digits: Vec<u8> = hex
        .chars()
        .map(|c| c.to_digit(16).and_then(|digit| u8::try_from(digit).ok()))
        .collect::<Option<_>>()?;
    let byte = |a: u8, b: u8| a * 16 + b;
    match digits.as_slice() {
        [r1, r2, g1, g2, b1, b2] | [_, _, r1, r2, g1, g2, b1, b2] => Some(Color32::from_rgb(
            byte(*r1, *r2),
            byte(*g1, *g2),
            byte(*b1, *b2),
        )),
        _ => None,
    }
}

fn config_dir() -> Option<PathBuf> {
    if let Some(base) = std::env::var_os("XDG_CONFIG_HOME").filter(|value| !value.is_empty()) {
        return Some(PathBuf::from(base));
    }
    std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(|home| PathBuf::from(home).join(".config"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r"
[General]
font=Noto Sans,10,-1,5,50,0,0,0,0,0

[KDE]
AnimationDurationFactor=0.0

[Colors:Window]
BackgroundNormal=36,36,36
ForegroundNormal=252,252,252
ForegroundInactive=178,178,178
ForegroundNegative=218,68,83
ForegroundNeutral=246,116,0
ForegroundPositive=39,174,96

[Colors:Selection]
BackgroundNormal=49,91,239
";

    #[test]
    fn parses_colours_sections_and_comments() {
        let globals = KdeGlobals::parse(SAMPLE);
        assert_eq!(
            globals.colour("Colors:Window", "BackgroundNormal"),
            Some(Color32::from_rgb(36, 36, 36))
        );
        assert_eq!(
            globals.colour("Colors:Selection", "BackgroundNormal"),
            Some(Color32::from_rgb(49, 91, 239))
        );
        assert_eq!(
            globals.value("General", "font"),
            Some("Noto Sans,10,-1,5,50,0,0,0,0,0")
        );
    }

    #[test]
    fn a_dark_background_means_a_dark_scheme() {
        assert_eq!(KdeGlobals::parse(SAMPLE).scheme(), Some(Scheme::Dark));
        let light = SAMPLE.replace("BackgroundNormal=36,36,36", "BackgroundNormal=239,240,241");
        assert_eq!(KdeGlobals::parse(&light).scheme(), Some(Scheme::Light));
    }

    #[test]
    fn font_keeps_family_and_size_and_ignores_the_rest() {
        let globals = KdeGlobals::parse(SAMPLE);
        assert_eq!(globals.font(), Some((String::from("Noto Sans"), 10.0)));
        assert_eq!(
            KdeGlobals::parse("[General]\nfont=Hack,,2").font(),
            Some((String::from("Hack"), 10.0))
        );
    }

    #[test]
    fn malformed_values_are_ignored() {
        let globals = KdeGlobals::parse(
            "[Colors:Window]\nBackgroundNormal=not,a,colour\nForegroundNormal=300,0,0\n\
             [KDE]\nAnimationDurationFactor=fast\nsummary=Foo",
        );
        assert_eq!(globals.colour("Colors:Window", "BackgroundNormal"), None);
        assert_eq!(globals.colour("Colors:Window", "ForegroundNormal"), None);
        assert_eq!(globals.animation_factor(), None);
        assert_eq!(globals.scheme(), None);
        assert_eq!(globals.value("General", "font"), None);
    }

    #[test]
    fn animation_factor_is_clamped() {
        let globals = KdeGlobals::parse("[KDE]\nAnimationDurationFactor=0.0");
        assert_eq!(globals.animation_factor(), Some(0.0));
        let slower = KdeGlobals::parse("[KDE]\nAnimationDurationFactor=9");
        assert_eq!(slower.animation_factor(), Some(2.0));
    }

    #[test]
    fn hex_colours_are_accepted_with_and_without_alpha() {
        let globals = KdeGlobals::parse(
            "[Colors:Window]\nBackgroundNormal=#202124\nForegroundNormal=#ff202124",
        );
        assert_eq!(
            globals.colour("Colors:Window", "BackgroundNormal"),
            Some(Color32::from_rgb(0x20, 0x21, 0x24))
        );
        assert_eq!(
            globals.colour("Colors:Window", "ForegroundNormal"),
            Some(Color32::from_rgb(0x20, 0x21, 0x24))
        );
    }
}
