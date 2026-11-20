// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <QCache>
#include <QColor>
#include <QStringList>
#include <QPixmap>
#include <QQuickImageProvider>

/// Icons from the desktop's own theme, for QML.
///
/// This replaces the retired `punchi-fluid-icons` crate: the rule did not change, only
/// who resolves the icon. `QIcon::fromTheme` reads the same Breeze theme the rest of the
/// desktop uses, and a symbolic icon is recoloured with the ink the theme layer picked, so
/// an icon never carries a colour of its own.
///
/// The URL is `image://theme/<name>?tone=<tone>&v=<revision>`, where tone is
/// `foreground`, `muted`, `accent` or `onaccent`; the raster size comes from QML's
/// `sourceSize` and `v` is the session's theme revision, which QML uses to invalidate its
/// own image cache after a palette change. A name the theme does not have produces an
/// empty pixmap: the button then shows its label alone, which is what a native application
/// does on a machine with no icon theme.
/// Makes the desktop's icon theme findable before the first lookup.
///
/// `QIcon::fromTheme` searches `QIcon::themeSearchPaths()`, and only a platform theme
/// normally fills that in: under `offscreen` it is the single entry `:/icons`, so every
/// lookup fails. The XDG data directories are where those icons actually live, so they are
/// added when they are missing. A session that already named its theme and its paths is
/// left exactly as it was.
void ensureIconTheme();

class ThemeIconProvider : public QQuickImageProvider
{
public:
    /// The measured top of the cache, one cost per entry. Every icon size and every tint
    /// the window asks for is one entry; the palette key makes a theme change a new one.
    /// Measured at 1x, 1.25x, 1.5x and 2x with Inicio open: 58 entries, largest group 21.
    /// 58 + 25 % rounds up to 73; a palette change adds a new set and the LRU evicts the
    /// retired one, so the cap never holds two palettes at once.
    static constexpr int kMaxCachedIcons = 73;

    ThemeIconProvider();

    QPixmap requestPixmap(const QString &id, QSize *size, const QSize &requestedSize) override;

    /// How many rasters the bounded cache holds.
    int cacheSize() const { return m_cache.count(); }
    /// The largest number of entries for one `(pixels, tone)` pair, the unit of the
    /// engineering budget.
    int maxGroupSize() const;
    /// Whether `id` requested at `pixels` is cached under the current device ratio,
    /// palette and tone. The id alone cannot resolve the key.
    bool contains(const QString &id, int pixels) const;

private:
    QColor colourFor(const QString &tone) const;
    QString cacheKey(const QString &name, const QString &tone, int pixels) const;

    mutable QCache<QString, QPixmap> m_cache;
};
