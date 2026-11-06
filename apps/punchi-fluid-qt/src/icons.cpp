// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#include "icons.h"

#include <QGuiApplication>
#include <QIcon>
#include <QDir>
#include <QImage>
#include <QPainter>
#include <QSet>
#include <QStandardPaths>
#include <QUrlQuery>
#include <QtGlobal>

#include "theme.h"

namespace {

/// Smallest usable icon, so a mis-sized request still draws something legible.
constexpr int kMinPixels = 8;

/// `true` when the image is a single ink with varying alpha, which is what a symbolic
/// icon is. A handful of colours is allowed for anti-aliasing residue; a picture has far
/// more.
bool isMonochrome(const QPixmap &pixmap)
{
    const QImage image = pixmap.toImage().convertToFormat(QImage::Format_ARGB32);
    if (image.isNull()) {
        return false;
    }
    QSet<QRgb> colours;
    for (int y = 0; y < image.height(); ++y) {
        for (int x = 0; x < image.width(); ++x) {
            const QRgb pixel = image.pixel(x, y);
            if (qAlpha(pixel) < 128) {
                continue;
            }
            colours.insert(pixel & 0x00FF'FFFFU);
            if (colours.size() > 8) {
                return false;
            }
        }
    }
    return true;
}

} // namespace

void ensureIconTheme()
{
    QStringList paths = QIcon::themeSearchPaths();
    const QStringList dataDirs = QStandardPaths::standardLocations(QStandardPaths::GenericDataLocation);
    for (const QString &dir : dataDirs) {
        const QString icons = dir + QStringLiteral("/icons");
        if (!paths.contains(icons)) {
            paths.append(icons);
        }
    }
    const QString local = QDir::homePath() + QStringLiteral("/.local/share/icons");
    if (!paths.contains(local)) {
        paths.append(local);
    }
    if (paths != QIcon::themeSearchPaths()) {
        QIcon::setThemeSearchPaths(paths);
    }
    if (QIcon::themeName().isEmpty()) {
        // Breeze is the desktop this project targets; without a name, no lookup succeeds.
        QIcon::setThemeName(QStringLiteral("breeze"));
    }
}

ThemeIconProvider::ThemeIconProvider()
    : QQuickImageProvider(QQuickImageProvider::Pixmap)
{
}

QColor ThemeIconProvider::colourFor(const QString &tone) const
{
    const ThemeTokens tokens = tokensFrom(QGuiApplication::palette());
    if (tone == QStringLiteral("muted")) {
        return tokens.inkMuted;
    }
    if (tone == QStringLiteral("accent")) {
        return tokens.accent;
    }
    return tokens.ink;
}

QPixmap ThemeIconProvider::requestPixmap(const QString &id, QSize *size, const QSize &requestedSize)
{
    if (size != nullptr) {
        *size = QSize(0, 0);
    }

    const int queryAt = id.indexOf(u'?');
    const QString name = queryAt < 0 ? id : id.left(queryAt);
    QUrlQuery query(queryAt < 0 ? QString() : id.mid(queryAt + 1));
    const int requested = requestedSize.isValid() ? requestedSize.width() : 0;
    const int pixels = requested > kMinPixels ? requested : 22;
    const QString tone = query.queryItemValue(QStringLiteral("tone"));

    const QString key = name + u'|' + tone + u'|' + QString::number(pixels);
    const auto cached = m_cache.constFind(key);
    if (cached != m_cache.constEnd()) {
        if (size != nullptr) {
            *size = cached->size();
        }
        return *cached;
    }

    // Breeze ships both `foo` and `foo-symbolic`; the symbolic one is the flat glyph that
    // recolours, which is what a ribbon wants. When the platform plugin has not named a
    // theme at all — an offscreen run, a bare X session — Breeze is the desktop this
    // project targets, so it is named explicitly rather than losing every icon.
    ensureIconTheme();
    // A symbolic icon is a monochrome glyph meant to take the text colour. A full-colour
    // one is a picture, and tinting it would replace the picture with its silhouette — so
    // the tint is applied only when the result really is monochrome.
    QIcon icon = QIcon::fromTheme(name + QStringLiteral("-symbolic"));
    if (icon.isNull()) {
        icon = QIcon::fromTheme(name);
    }

    QPixmap pixmap;
    if (!icon.isNull()) {
        const QPixmap source = icon.pixmap(QSize(pixels, pixels));
        pixmap = QPixmap(source.size());
        pixmap.fill(Qt::transparent);
        QPainter painter(&pixmap);
        painter.drawPixmap(0, 0, source);
        if (isMonochrome(source)) {
            // `SourceIn` keeps the glyph's alpha and replaces the colour, which is how a
            // monochrome theme icon is tinted without a second asset.
            painter.setCompositionMode(QPainter::CompositionMode_SourceIn);
            painter.fillRect(pixmap.rect(), colourFor(tone));
        }
        painter.end();
    }

    m_cache.insert(key, pixmap);
    if (size != nullptr) {
        *size = pixmap.size();
    }
    return pixmap;
}
