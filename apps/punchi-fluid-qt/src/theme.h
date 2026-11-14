// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <QColor>
#include <QPalette>
#include <QString>

/// The colours the interface uses, taken from the desktop palette.
///
/// This is the whole theme layer of the Qt front end: `QPalette` has already merged
/// `kdeglobals` and the `org.freedesktop.appearance color-scheme` portal through the
/// platform plugin, so reading it *is* reading the desktop. Nothing below this file may
/// name a colour.
struct ThemeTokens {
    QColor background;
    QColor panel;
    QColor ink;
    QColor inkDim;
    QColor inkMuted;
    QColor accent;
    QColor supply;
    QColor work;
    /// The drawing grid: the muted ink at the two opacities the sheet uses.
    QColor gridMinor;
    QColor gridMajor;
};

/// Builds the tokens from a palette, blending the two greys the palette does not name.
///
/// Two things happen here that a plain palette copy would not do. The secondary greys
/// are blended, because the palette names neither. And every colour that carries meaning
/// is pushed away from the background until it reaches its criterion — a desktop is
/// free to hand us an accent that vanishes on its own window colour, and a simulator
/// whose valve outline disappears is worse than one that is a shade off-palette.
ThemeTokens tokensFrom(const QPalette &palette);

/// `colour` moved towards white or black until it reaches `minimum` against `background`.
///
/// Returns the colour unchanged when it already passes, so a desktop with a good
/// palette is never altered.
QColor ensureContrast(const QColor &colour, const QColor &background, double minimum);

/// WCAG 2.1 relative luminance.
double relativeLuminance(const QColor &colour);

/// WCAG 2.1 contrast ratio, `1.0 ..= 21.0`.
double contrastRatio(const QColor &a, const QColor &b);

/// The pairings `docs/ui-ux.md` fixes and their measured ratios.
struct Contrast {
    double text = 0.0;
    double dim = 0.0;
    double accent = 0.0;

    /// Text at 4.5:1 and secondary lines at 3:1, the acceptance criterion of M1.
    [[nodiscard]] bool passes() const { return text >= 4.5 && dim >= 3.0 && accent >= 3.0; }
};

/// Measures the three pairings the interface depends on.
Contrast measure(const ThemeTokens &tokens);

/// One line per pairing, for a log or a failing self-test.
QString describe(const Contrast &contrast);

/// A palette with a known contrast, for `--theme light|dark`.
///
/// The only colour literals in the application live here, in the theme adapter, which is
/// the same rule the egui front end had: a `Color32::` outside the theme crate was a bug
/// and a hex string outside this file is one now.
QPalette forcedPalette(bool dark);
