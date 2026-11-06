// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <QColor>
#include <QHash>
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
/// The URL is `image://theme/<name>?tone=<tone>&size=<px>`, where tone is `foreground`,
/// `muted` or `accent`. A name the theme does not have produces an empty pixmap: the
/// button then shows its label alone, which is what a native application does on a machine
/// with no icon theme.
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
    ThemeIconProvider();

    QPixmap requestPixmap(const QString &id, QSize *size, const QSize &requestedSize) override;

private:
    QColor colourFor(const QString &tone) const;

    mutable QHash<QString, QPixmap> m_cache;
};
