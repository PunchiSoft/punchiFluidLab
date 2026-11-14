// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#include "theme.h"

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
    tokens.inkDim = ensureContrast(blend(tokens.ink, tokens.background, 0.30), tokens.background, 3.0);
    tokens.inkMuted =
        ensureContrast(blend(tokens.ink, tokens.background, 0.55), tokens.background, 3.0);
    tokens.accent = ensureContrast(palette.color(QPalette::Highlight), tokens.background, 3.0);
    tokens.supply = ensureContrast(palette.color(QPalette::Link), tokens.background, 3.0);
    tokens.work = tokens.accent;
    tokens.gridMinor = tokens.inkMuted;
    tokens.gridMinor.setAlphaF(0.22F);
    tokens.gridMajor = tokens.inkMuted;
    tokens.gridMajor.setAlphaF(0.34F);
    return tokens;
}

Contrast measure(const ThemeTokens &tokens)
{
    Contrast contrast;
    contrast.text = contrastRatio(tokens.ink, tokens.background);
    contrast.dim = contrastRatio(tokens.inkMuted, tokens.background);
    contrast.accent = contrastRatio(tokens.accent, tokens.background);
    return contrast;
}

QString describe(const Contrast &contrast)
{
    const QStringList lines {
        QStringLiteral("text %1:1 (needs 4.50)").arg(contrast.text, 0, 'f', 2),
        QStringLiteral("secondary %1:1 (needs 3.00)").arg(contrast.dim, 0, 'f', 2),
        QStringLiteral("accent %1:1 (needs 3.00)").arg(contrast.accent, 0, 'f', 2),
    };
    return lines.join(QStringLiteral(", "));
}

QPalette forcedPalette(bool dark)
{
    QPalette palette;
    if (dark) {
        palette.setColor(QPalette::Window, QColor(36, 36, 36));
        palette.setColor(QPalette::Base, QColor(46, 46, 46));
        palette.setColor(QPalette::WindowText, QColor(222, 222, 222));
        palette.setColor(QPalette::Highlight, QColor(49, 91, 239));
        palette.setColor(QPalette::Link, QColor(120, 170, 255));
        palette.setColor(QPalette::HighlightedText, QColor(255, 255, 255));
    } else {
        palette.setColor(QPalette::Window, QColor(242, 242, 242));
        palette.setColor(QPalette::Base, QColor(255, 255, 255));
        palette.setColor(QPalette::WindowText, QColor(26, 26, 26));
        palette.setColor(QPalette::Highlight, QColor(49, 91, 239));
        palette.setColor(QPalette::Link, QColor(26, 102, 204));
        palette.setColor(QPalette::HighlightedText, QColor(255, 255, 255));
    }
    return palette;
}
