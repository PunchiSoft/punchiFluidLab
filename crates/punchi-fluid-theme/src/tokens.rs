// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Semantic tokens: the only names a widget may use to ask for a colour.
//!
//! A token describes a job (`Surface`, `Text`, `WorkLine`), never a hue, so a
//! palette can come from the desktop, from a fallback or from a test without the
//! widgets noticing.

use egui::Color32;

use crate::Scheme;
use crate::colour::{blend, ensure_contrast, readable_on, rgb};
use crate::kdeglobals::KdeGlobals;

/// One job a colour has in the interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum Token {
    /// Window and panel background.
    Surface,
    /// Alternative surface: fields, view backgrounds.
    SurfaceAlt,
    /// Raised surface: buttons, menus, dialogs.
    SurfaceRaised,
    /// The sheet where the circuit is drawn.
    Sheet,
    /// Primary text.
    Text,
    /// Secondary text and hints.
    TextMuted,
    /// Text of a disabled action.
    TextDisabled,
    /// Selection, focus and the active valve position.
    Accent,
    /// Accent under the pointer.
    AccentHover,
    /// Text drawn on top of the accent.
    AccentText,
    /// Error and destructive action.
    Danger,
    /// Warning and skipped files.
    Warning,
    /// Confirmation and the pressurised working line.
    Ok,
    /// Borders between regions.
    Outline,
    /// Light grid of the sheet.
    GridMinor,
    /// Heavy grid of the sheet.
    GridMajor,
    /// Outline of a component symbol.
    SymbolLine,
    /// Symbol parts that are not active.
    SymbolDim,
    /// Supply line, always under pressure.
    SupplyLine,
    /// Working line, coloured by the pressure it carries.
    WorkLine,
}

impl Token {
    /// Every token, in declaration order.
    pub const ALL: [Self; 20] = [
        Self::Surface,
        Self::SurfaceAlt,
        Self::SurfaceRaised,
        Self::Sheet,
        Self::Text,
        Self::TextMuted,
        Self::TextDisabled,
        Self::Accent,
        Self::AccentHover,
        Self::AccentText,
        Self::Danger,
        Self::Warning,
        Self::Ok,
        Self::Outline,
        Self::GridMinor,
        Self::GridMajor,
        Self::SymbolLine,
        Self::SymbolDim,
        Self::SupplyLine,
        Self::WorkLine,
    ];

    /// Position in the palette array.
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// Colour for every token, built once per theme change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    colours: [Color32; Token::ALL.len()],
}

impl Palette {
    /// Built-in palette used when the desktop provides nothing usable.
    ///
    /// The values follow Breeze's light and dark defaults so the window looks
    /// deliberate, not like a toolkit default, on a machine without `kdeglobals`.
    #[must_use]
    pub fn fallback(scheme: Scheme) -> Self {
        let mut palette = Self {
            colours: [Color32::BLACK; Token::ALL.len()],
        };
        let (
            surface,
            surface_alt,
            raised,
            text,
            muted,
            disabled,
            accent,
            danger,
            warning,
            ok,
            supply,
        ) = match scheme {
            Scheme::Light => (
                rgb(239, 240, 241),
                rgb(252, 252, 252),
                rgb(255, 255, 255),
                rgb(35, 38, 41),
                rgb(107, 114, 123),
                rgb(150, 156, 163),
                rgb(61, 174, 233),
                rgb(192, 28, 43),
                rgb(180, 83, 9),
                rgb(29, 138, 66),
                rgb(41, 128, 185),
            ),
            Scheme::Dark | Scheme::NoPreference => (
                rgb(36, 36, 36),
                rgb(30, 30, 30),
                rgb(51, 51, 51),
                rgb(252, 252, 252),
                rgb(178, 178, 178),
                rgb(128, 128, 128),
                rgb(49, 91, 239),
                rgb(218, 68, 83),
                rgb(246, 116, 0),
                rgb(61, 174, 96),
                rgb(91, 156, 214),
            ),
        };

        palette.set(Token::Surface, surface);
        palette.set(Token::SurfaceAlt, surface_alt);
        palette.set(Token::SurfaceRaised, raised);
        palette.set(Token::Text, text);
        palette.set(Token::TextMuted, muted);
        palette.set(Token::TextDisabled, disabled);
        palette.set(Token::Accent, accent);
        palette.set(Token::AccentText, readable_on(accent));
        palette.set(Token::Danger, danger);
        palette.set(Token::Warning, warning);
        palette.set(Token::Ok, ok);
        palette.set(Token::SupplyLine, supply);
        palette.derive();
        palette.repair();
        palette
    }

    /// Palette built from the desktop file, falling back token by token.
    ///
    /// `scheme` is the effective scheme (never `NoPreference`).
    #[must_use]
    pub fn from_globals(globals: Option<&KdeGlobals>, scheme: Scheme) -> Self {
        let Some(globals) = globals else {
            return Self::fallback(scheme);
        };
        let mut palette = Self::fallback(scheme);

        if let Some(colour) = globals.colour("Colors:Window", "BackgroundNormal") {
            palette.set(Token::Surface, colour);
        }
        if let Some(colour) = globals.colour("Colors:View", "BackgroundNormal") {
            palette.set(Token::SurfaceAlt, colour);
        }
        if let Some(colour) = globals.colour("Colors:Button", "BackgroundNormal") {
            palette.set(Token::SurfaceRaised, colour);
        }
        if let Some(colour) = globals.colour("Colors:Window", "ForegroundNormal") {
            palette.set(Token::Text, colour);
        }
        if let Some(colour) = globals.colour("Colors:Window", "ForegroundInactive") {
            palette.set(Token::TextMuted, colour);
            palette.set(Token::TextDisabled, blend(colour, palette.surface(), 0.35));
        }
        if let Some(colour) = globals.colour("Colors:Window", "ForegroundNegative") {
            palette.set(Token::Danger, colour);
        }
        if let Some(colour) = globals.colour("Colors:Window", "ForegroundNeutral") {
            palette.set(Token::Warning, colour);
        }
        if let Some(colour) = globals.colour("Colors:Window", "ForegroundPositive") {
            palette.set(Token::Ok, colour);
        }
        if let Some(colour) = globals.colour("Colors:Selection", "BackgroundNormal") {
            palette.set(Token::Accent, colour);
        }
        if let Some(colour) = globals.colour("Colors:Selection", "ForegroundNormal") {
            palette.set(Token::AccentText, colour);
        }

        palette.derive();
        palette.repair();
        palette
    }

    /// Colour of one token.
    #[must_use]
    pub const fn colour(&self, token: Token) -> Color32 {
        self.colours[token.index()]
    }

    /// Mixes two tokens, for a value that moves between two states.
    #[must_use]
    pub fn blend(&self, from: Token, to: Token, amount: f32) -> Color32 {
        blend(self.colour(from), self.colour(to), amount)
    }

    fn set(&mut self, token: Token, colour: Color32) {
        self.colours[token.index()] = colour;
    }

    fn surface(&self) -> Color32 {
        self.colour(Token::Surface)
    }

    fn text(&self) -> Color32 {
        self.colour(Token::Text)
    }

    /// Fills every token derived from the desktop's own colours.
    fn derive(&mut self) {
        let surface = self.surface();
        let text = self.text();
        let accent = self.colour(Token::Accent);
        let muted = self.colour(Token::TextMuted);
        let ok = self.colour(Token::Ok);

        self.set(Token::Sheet, self.colour(Token::SurfaceAlt));
        self.set(Token::Outline, blend(surface, text, 0.25));
        self.set(Token::AccentHover, blend(accent, text, 0.15));
        self.set(Token::SymbolLine, text);
        self.set(Token::SymbolDim, blend(surface, muted, 0.55));
        self.set(Token::WorkLine, ok);
    }

    /// Keeps every text and line token readable on the surface it sits on.
    fn repair(&mut self) {
        let surface = self.surface();
        let sheet = self.colour(Token::Sheet);
        self.set(Token::Text, ensure_contrast(self.text(), surface, 4.5));
        self.set(
            Token::TextMuted,
            ensure_contrast(self.colour(Token::TextMuted), surface, 4.5),
        );
        // A disabled label is still a label: dimmed, not invisible. 3:1 is the floor for
        // a large glyph, and a toolbar button is small text on a dark bar, so it gets the
        // same 4.5:1 as the rest. The dimming comes from the colour chosen, not from
        // being unreadable.
        self.set(
            Token::TextDisabled,
            ensure_contrast(self.colour(Token::TextDisabled), surface, 4.5),
        );
        self.set(
            Token::Danger,
            ensure_contrast(self.colour(Token::Danger), surface, 4.5),
        );
        self.set(
            Token::Warning,
            ensure_contrast(self.colour(Token::Warning), surface, 4.5),
        );
        self.set(
            Token::Ok,
            ensure_contrast(self.colour(Token::Ok), surface, 4.5),
        );
        self.set(
            Token::AccentText,
            ensure_contrast(
                self.colour(Token::AccentText),
                self.colour(Token::Accent),
                4.5,
            ),
        );
        for token in [
            Token::Outline,
            Token::SymbolLine,
            Token::SymbolDim,
            Token::SupplyLine,
            Token::WorkLine,
            Token::Accent,
        ] {
            let colour = ensure_contrast(self.colour(token), sheet, 3.0);
            let colour = ensure_contrast(colour, surface, 3.0);
            self.set(token, colour);
        }

        // Grid lines are a guide by design: they may stay below the 3:1 ratio so
        // they never compete with the content drawn on top of them. They are mixed
        // from the repaired text, so a file with equal foreground and background
        // still produces a visible grid.
        let text = self.colour(Token::Text);
        self.set(Token::GridMinor, blend(sheet, text, 0.08));
        self.set(Token::GridMajor, blend(sheet, text, 0.16));
    }
}
