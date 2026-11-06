// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The palettes the interface can show must be readable, measured, not eyeballed.
//!
//! Text is held to WCAG AA (4.5:1) and lines to the non-text minimum (3:1), for the
//! built-in palettes, for a file copied from this machine and for a hostile file
//! whose foreground equals its background.

use punchi_fluid_theme::{KdeGlobals, Palette, Scheme, Token, colour::contrast};

const TEXT_MINIMUM: f32 = 4.5;
const LINE_MINIMUM: f32 = 3.0;

fn assert_text(palette: &Palette, token: Token, surface: Token, minimum: f32, case: &str) {
    let ratio = contrast(palette.colour(token), palette.colour(surface));
    assert!(
        ratio >= minimum,
        "{case}: {token:?} on {surface:?} is {ratio:.2}:1, below {minimum}:1"
    );
}

fn check_palette(palette: &Palette, case: &str) {
    for surface in [Token::Surface, Token::Sheet] {
        for token in [Token::Text, Token::TextMuted] {
            assert_text(palette, token, surface, TEXT_MINIMUM, case);
        }
        for token in [Token::Danger, Token::Warning, Token::Ok] {
            assert_text(palette, token, surface, TEXT_MINIMUM, case);
        }
        for token in [
            Token::SymbolLine,
            Token::SymbolDim,
            Token::SupplyLine,
            Token::WorkLine,
            Token::Outline,
            Token::Accent,
        ] {
            assert_text(palette, token, surface, LINE_MINIMUM, case);
        }
        // A disabled label is still read by a person: dimmed, not unreadable. The toolbar
        // is a row of them, so it is held to the text minimum, not the line one.
        assert_text(palette, Token::TextDisabled, surface, TEXT_MINIMUM, case);
    }
    assert_text(
        palette,
        Token::AccentText,
        Token::Accent,
        TEXT_MINIMUM,
        case,
    );

    // Grid lines stay below the content so they never compete with a symbol.
    for grid in [Token::GridMinor, Token::GridMajor] {
        let ratio = contrast(palette.colour(grid), palette.colour(Token::Sheet));
        assert!(ratio > 1.0, "{case}: {grid:?} is invisible");
        assert!(
            ratio < LINE_MINIMUM,
            "{case}: {grid:?} at {ratio:.2}:1 is too loud for a guide"
        );
    }
}

#[test]
fn built_in_palettes_are_readable() {
    check_palette(&Palette::fallback(Scheme::Dark), "fallback dark");
    check_palette(&Palette::fallback(Scheme::Light), "fallback light");
}

#[test]
fn this_machines_palette_is_readable() {
    let globals = KdeGlobals::parse(
        "[General]\nfont=Noto Sans,10,-1,5,50,0,0,0,0,0\n\
         [Colors:Window]\nBackgroundNormal=36,36,36\nForegroundNormal=252,252,252\n\
         ForegroundInactive=178,178,178\nForegroundNegative=218,68,83\n\
         ForegroundNeutral=246,116,0\nForegroundPositive=39,174,96\n\
         [Colors:View]\nBackgroundNormal=30,30,30\n\
         [Colors:Button]\nBackgroundNormal=51,51,51\n\
         [Colors:Selection]\nBackgroundNormal=49,91,239",
    );
    check_palette(
        &Palette::from_globals(Some(&globals), Scheme::Dark),
        "plasma dark",
    );
}

#[test]
fn a_hostile_theme_is_repaired_instead_of_accepted() {
    let globals = KdeGlobals::parse(
        "[Colors:Window]\nBackgroundNormal=90,90,90\nForegroundNormal=90,90,90\n\
         ForegroundInactive=95,95,95\nForegroundNegative=100,100,100\n\
         ForegroundNeutral=100,100,100\nForegroundPositive=100,100,100\n\
         [Colors:View]\nBackgroundNormal=90,90,90\n\
         [Colors:Button]\nBackgroundNormal=90,90,90\n\
         [Colors:Selection]\nBackgroundNormal=120,120,120",
    );
    check_palette(
        &Palette::from_globals(Some(&globals), Scheme::Light),
        "hostile",
    );
}

#[test]
fn light_and_dark_are_not_the_same_palette() {
    let dark = Palette::fallback(Scheme::Dark);
    let light = Palette::fallback(Scheme::Light);
    assert_ne!(dark.colour(Token::Surface), light.colour(Token::Surface));
    assert_ne!(dark.colour(Token::Text), light.colour(Token::Text));
    assert!(contrast(dark.colour(Token::Text), dark.colour(Token::Surface)) >= TEXT_MINIMUM);
    assert!(contrast(light.colour(Token::Text), light.colour(Token::Surface)) >= TEXT_MINIMUM);
}
