// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#include "theme.h"

#include <QFontMetrics>
#include <QGuiApplication>
#include <QSettings>
#include <QStandardPaths>
#include <QStringList>

#include <algorithm>
#include <cmath>

namespace {

QColor blend(const QColor &from, const QColor &towards, double amount)
{
    const double t = std::clamp(amount, 0.0, 1.0);
    return QColor(qRound(from.red() + (towards.red() - from.red()) * t),
                  qRound(from.green() + (towards.green() - from.green()) * t),
                  qRound(from.blue() + (towards.blue() - from.blue()) * t));
}

double channel(double value)
{
    return value <= 0.03928 ? value / 12.92 : std::pow((value + 0.055) / 1.055, 2.4);
}

/// A plane below the window: a dark desktop is pressed down harder than a light one.
QColor planeBelow(const QColor &background)
{
    const double amount = relativeLuminance(background) < 0.35 ? 0.18 : 0.05;
    return blend(background, QColor(0, 0, 0), amount);
}

/// The accent used as a fill behind text: softened towards the surface until the ink reads.
QColor soften(const QColor &accent, const QColor &surface, const QColor &ink, double minimum)
{
    QColor result = blend(accent, surface, 0.82);
    for (int step = 0; step < 24 && contrastRatio(ink, result) < minimum; ++step) {
        result = blend(result, surface, 0.35);
    }
    return result;
}

/// The spacing unit follows the desktop font, rounded to an even number and never tiny.
int unitFromFont()
{
    const QFontMetrics metrics(QGuiApplication::font());
    const int height = std::max(16, metrics.height());
    return std::max(16, ((height + 1) / 2) * 2);
}

/// `[KDE] AnimationDurationFactor`; a missing key means 180 ms, and `0` means no motion.
///
/// Read once per process: an icon request asks for the tokens too, and a settings file
/// should not be opened again for a value that only changes with the session.
int motionFromDesktop()
{
    static const int duration = [] {
        const QString config =
            QStandardPaths::writableLocation(QStandardPaths::GenericConfigLocation);
        if (config.isEmpty()) {
            return 180;
        }
        QSettings settings(config + QStringLiteral("/kdeglobals"), QSettings::IniFormat);
        bool ok = false;
        const double factor =
            settings.value(QStringLiteral("KDE/AnimationDurationFactor"), 1.0).toDouble(&ok);
        if (!ok || factor < 0.0) {
            return 180;
        }
        if (factor == 0.0) {
            return 0;
        }
        return qRound(180.0 * std::min(factor, 2.0));
    }();
    return duration;
}

} // namespace

double relativeLuminance(const QColor &colour)
{
    const double r = channel(colour.redF());
    const double g = channel(colour.greenF());
    const double b = channel(colour.blueF());
    return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

double contrastRatio(const QColor &a, const QColor &b)
{
    const double first = relativeLuminance(a);
    const double second = relativeLuminance(b);
    const double lighter = std::max(first, second);
    const double darker = std::min(first, second);
    return (lighter + 0.05) / (darker + 0.05);
}

QColor ensureContrast(const QColor &colour, const QColor &background, double minimum)
{
    if (contrastRatio(colour, background) >= minimum) {
        return colour;
    }
    // Away from the background: lighten on a dark desktop, darken on a light one.
    const bool lighten = relativeLuminance(background) < 0.35;
    QColor adjusted = colour;
    for (int step = 0; step < 120; ++step) {
        const QColor next = lighten ? adjusted.lighter(104) : adjusted.darker(104);
        if (next == adjusted) {
            break;
        }
        adjusted = next;
        if (contrastRatio(adjusted, background) >= minimum) {
            break;
        }
    }
    return adjusted;
}

ThemeTokens tokensFrom(const QPalette &palette)
{
    ThemeTokens tokens;
    tokens.background = palette.color(QPalette::Window);
    tokens.panel = palette.color(QPalette::Base);
    tokens.ink = ensureContrast(palette.color(QPalette::WindowText), tokens.background, 4.5);
    // Two planes around the window: the button colour above it and a pressed one below.
    tokens.surfaceRaised = palette.color(QPalette::Button);
    if (contrastRatio(tokens.surfaceRaised, tokens.background) < 1.08) {
        tokens.surfaceRaised = blend(tokens.background, tokens.ink, 0.07);
    }
    tokens.surfaceSunken = planeBelow(tokens.background);
    // The secondary inks are measured on the raised plane, which is the stricter case:
    // whatever reads there also reads on the window behind it.
    tokens.inkDim =
        ensureContrast(blend(tokens.ink, tokens.background, 0.30), tokens.surfaceRaised, 3.0);
    tokens.inkMuted =
        ensureContrast(blend(tokens.ink, tokens.background, 0.55), tokens.surfaceRaised, 3.0);
    tokens.accent = ensureContrast(palette.color(QPalette::Highlight), tokens.background, 3.0);
    // A soft accent is a fill, so it is measured against the ink that sits on it.
    tokens.accentSoft = soften(tokens.accent, tokens.surfaceRaised, tokens.ink, 4.5);
    // The line that carries meaning is measured on the raised plane, the stricter case;
    // the plain line is a separator and stays quiet on purpose.
    tokens.lineStrong =
        ensureContrast(blend(tokens.ink, tokens.background, 0.45), tokens.surfaceRaised, 3.0);
    tokens.line = blend(tokens.ink, tokens.background, 0.62);
    tokens.shadow = QColor(0, 0, 0, relativeLuminance(tokens.background) < 0.35 ? 90 : 40);
    tokens.supply = ensureContrast(palette.color(QPalette::Link), tokens.background, 3.0);
    tokens.work = tokens.accent;
    tokens.gridMinor = tokens.inkMuted;
    tokens.gridMinor.setAlphaF(0.22F);
    tokens.gridMajor = tokens.inkMuted;
    tokens.gridMajor.setAlphaF(0.34F);
    tokens.unit = unitFromFont();
    // The same font the unit follows, so a larger desktop grows both. At the default 18 px
    // unit these are the 10, 11 and 12 the ribbon was drawn with.
    tokens.textSmall = std::max(10, (tokens.unit * 55 + 50) / 100);
    tokens.textBody = std::max(11, (tokens.unit * 61 + 50) / 100);
    tokens.textLabel = std::max(12, (tokens.unit * 66 + 50) / 100);
    tokens.motionMs = motionFromDesktop();
    return tokens;
}

Contrast measure(const ThemeTokens &tokens)
{
    Contrast contrast;
    contrast.text = contrastRatio(tokens.ink, tokens.background);
    contrast.dim = contrastRatio(tokens.inkMuted, tokens.background);
    contrast.accent = contrastRatio(tokens.accent, tokens.background);
    contrast.raisedText = contrastRatio(tokens.ink, tokens.surfaceRaised);
    contrast.raisedDim = contrastRatio(tokens.inkMuted, tokens.surfaceRaised);
    contrast.line = contrastRatio(tokens.lineStrong, tokens.background);
    contrast.lineRaised = contrastRatio(tokens.lineStrong, tokens.surfaceRaised);
    contrast.accentSoftText = contrastRatio(tokens.ink, tokens.accentSoft);
    contrast.sheetEdge = contrastRatio(tokens.lineStrong, tokens.surfaceSunken);
    return contrast;
}

QString describe(const Contrast &contrast)
{
    const QStringList lines {
        QStringLiteral("text %1:1 (needs 4.50)").arg(contrast.text, 0, 'f', 2),
        QStringLiteral("secondary %1:1 (needs 3.00)").arg(contrast.dim, 0, 'f', 2),
        QStringLiteral("accent %1:1 (needs 3.00)").arg(contrast.accent, 0, 'f', 2),
        QStringLiteral("raised text %1:1 (needs 4.50)").arg(contrast.raisedText, 0, 'f', 2),
        QStringLiteral("raised secondary %1:1 (needs 3.00)").arg(contrast.raisedDim, 0, 'f', 2),
        QStringLiteral("line %1:1 (needs 3.00)").arg(contrast.line, 0, 'f', 2),
        QStringLiteral("raised line %1:1 (needs 3.00)").arg(contrast.lineRaised, 0, 'f', 2),
        QStringLiteral("soft accent text %1:1 (needs 4.50)")
            .arg(contrast.accentSoftText, 0, 'f', 2),
        QStringLiteral("sheet edge %1:1 (needs 3.00)").arg(contrast.sheetEdge, 0, 'f', 2),
    };
    return lines.join(QStringLiteral(", "));
}

QPalette forcedPalette(bool dark)
{
    QPalette palette;
    if (dark) {
        palette.setColor(QPalette::Window, QColor(36, 36, 36));
        palette.setColor(QPalette::Base, QColor(46, 46, 46));
        palette.setColor(QPalette::Button, QColor(52, 52, 52));
        palette.setColor(QPalette::WindowText, QColor(222, 222, 222));
        palette.setColor(QPalette::Highlight, QColor(49, 91, 239));
        palette.setColor(QPalette::Link, QColor(120, 170, 255));
        palette.setColor(QPalette::HighlightedText, QColor(255, 255, 255));
    } else {
        palette.setColor(QPalette::Window, QColor(242, 242, 242));
        palette.setColor(QPalette::Base, QColor(255, 255, 255));
        palette.setColor(QPalette::Button, QColor(239, 239, 239));
        palette.setColor(QPalette::WindowText, QColor(26, 26, 26));
        palette.setColor(QPalette::Highlight, QColor(49, 91, 239));
        palette.setColor(QPalette::Link, QColor(26, 102, 204));
        palette.setColor(QPalette::HighlightedText, QColor(255, 255, 255));
    }
    return palette;
}
