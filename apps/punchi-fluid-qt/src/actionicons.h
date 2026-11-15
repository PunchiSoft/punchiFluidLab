// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <QColor>
#include <QPixmap>
#include <QString>
#include <QStringList>

/// The ribbon's own line icons, drawn here as geometry (ADR 0028).
///
/// The family is deliberately small at first: the primary commands, reviewed before the
/// rest. A name the family does not have falls back to the desktop's symbolic icon, and
/// then to the label alone. Nothing here is a component symbol; those come from the
/// catalog and are drawn by `punchi-fluid-draw`.
namespace actionicons {

/// `true` when this family draws `name`.
bool has(const QString &name);

/// The names the family covers, in the order the review sheet shows them.
QStringList names();

/// Rasterises one icon at `pixels` logical pixels per side, at `devicePixelRatio`, in one
/// ink. Returns a null pixmap for an unknown name.
QPixmap render(const QString &name, int pixels, qreal devicePixelRatio, const QColor &ink);

} // namespace actionicons
