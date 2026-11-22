// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The theme tests: the desktop palette, the contrast floors, the fallback and the font.

use super::{
    Theme, Tokens, font_from, groups, install_font, motion_from, palette_from, tokens_from,
};

const SAMPLE: &str = "[Colors:Window]\n\
BackgroundNormal=255,255,255\n\
ForegroundNormal=36,36,36\n\
ForegroundLink=41,128,185\n\
[Colors:View]\n\
BackgroundNormal=255,255,255\n\
ForegroundNormal=36,36,36\n\
[Colors:Button]\n\
BackgroundNormal=255,255,255\n\
[Colors:Selection]\n\
BackgroundNormal=49,91,239\n\
ForegroundNormal=255,255,255\n\
[General]\n\
font=Noto Sans,10,-1,5,50,0,0,0,0,0\n\
[KDE]\n\
AnimationDurationFactor=1.0\n";

fn sample_tokens() -> Tokens {
    let palette = palette_from(SAMPLE).expect("the sample has the palette groups");
    let font = font_from(SAMPLE);
    tokens_from(palette, &font, 180)
}

#[test]
fn the_palette_comes_from_the_kde_groups() {
    let groups = groups(SAMPLE);
    assert_eq!(
        groups
            .get("colors:window")
            .and_then(|g| g.get("backgroundnormal")),
        Some(&String::from("255,255,255"))
    );
    let palette = palette_from(SAMPLE).expect("the sample has the palette groups");
    assert_eq!(
        (palette.window.r, palette.window.g, palette.window.b),
        (255, 255, 255)
    );
    assert_eq!(
        (
            palette.highlight.r,
            palette.highlight.g,
            palette.highlight.b
        ),
        (49, 91, 239)
    );
    assert_eq!(
        (palette.link.r, palette.link.g, palette.link.b),
        (41, 128, 185)
    );
}

#[test]
fn the_tokens_meet_the_contrast_criteria() {
    let contrast = sample_tokens().contrast();
    assert!(contrast.text >= 4.5, "text {}", contrast.text);
    assert!(contrast.dim >= 3.0, "secondary {}", contrast.dim);
    assert!(contrast.accent >= 3.0, "accent {}", contrast.accent);
    assert!(
        contrast.raised_text >= 4.5,
        "raised text {}",
        contrast.raised_text
    );
    assert!(
        contrast.raised_dim >= 3.0,
        "raised secondary {}",
        contrast.raised_dim
    );
    assert!(contrast.line >= 3.0, "line {}", contrast.line);
    assert!(
        contrast.line_raised >= 3.0,
        "raised line {}",
        contrast.line_raised
    );
    assert!(
        contrast.accent_soft_text >= 4.5,
        "soft accent text {}",
        contrast.accent_soft_text
    );
    assert!(
        contrast.accent_text >= 4.5,
        "accent text {}",
        contrast.accent_text
    );
    assert!(
        contrast.sheet_edge >= 3.0,
        "sheet edge {}",
        contrast.sheet_edge
    );
}

#[test]
fn the_font_and_motion_come_from_the_desktop() {
    let font = font_from(SAMPLE);
    assert_eq!(font.family.as_deref(), Some("Noto Sans"));
    assert!((font.point_size - 10.0).abs() < f32::EPSILON);
    assert_eq!(motion_from(SAMPLE), 180);
    assert_eq!(motion_from("[KDE]\nAnimationDurationFactor=0\n"), 0);
    assert_eq!(motion_from("[KDE]\nAnimationDurationFactor=1.5\n"), 270);
    assert_eq!(motion_from("[KDE]\nAnimationDurationFactor=nope\n"), 180);
    assert_eq!(motion_from(""), 180);
}

#[test]
fn the_fallback_theme_is_the_light_desktop() {
    let theme = Theme::fallback();
    assert!(!theme.tokens.dark);
    let contrast = theme.tokens.contrast();
    assert!(contrast.text >= 4.5, "fallback text {}", contrast.text);
    assert!(theme.font_family().is_none());
}

#[test]
fn the_theme_applies_to_an_egui_context() {
    let context = eframe::egui::Context::default();
    Theme::fallback().apply(&context);
    assert!(
        context
            .style()
            .text_styles
            .contains_key(&eframe::egui::TextStyle::Body),
        "the body text size comes from the theme"
    );
}

#[test]
fn the_desktop_font_lays_out_spanish_text() {
    let context = eframe::egui::Context::default();
    if install_font(&context, None).is_err() {
        eprintln!("no system font on this machine: the window cannot start here");
        return;
    }
    let mut width = 0.0;
    let _ = context.run(eframe::egui::RawInput::default(), |context| {
        let galley = context.fonts(|fonts| {
            fonts.layout_no_wrap(
                "áñ".to_owned(),
                eframe::egui::FontId::proportional(12.0),
                eframe::egui::Color32::BLACK,
            )
        });
        width = galley.size().x;
    });
    assert!(width > 0.0, "the accents must have glyphs");
}
