// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <QPainter>
#include <QString>

#include "replay.h"

class FluidSession;

/// The bands an ISO 5457 reference grid uses on a sheet.
enum class ZoneBands {
    /// No zones: A5, Letter, Legal, Oficio and a custom sheet below A3.
    None,
    /// A3 and larger: numerals on the top and bottom bands, letters on the left and right.
    FourSides,
    /// A4: numerals on the top band only, letters on the right band only.
    A4,
};

/// The grid `paperName` uses at these sides.
///
/// `paperName` is the session's paper name (`Personalizado` for a custom sheet), so A4 is
/// recognised by its name and a custom sheet by its short side, exactly as the export does.
ZoneBands sheetZoneBands(const QString &paperName, qreal sheetWidthMm, qreal sheetHeightMm);

/// Number of reference fields along `lengthMm`: at most 50 mm each, never fewer than one.
int sheetFieldCount(qreal lengthMm);

/// The letter of reference row `index`: `A`, `B`, … skipping `I` and `O`.
QString sheetRowLabel(int index);

/// The numeral of reference column `index`: `1`, `2`, …
QString sheetColumnLabel(int index);

/// Draws the paper, the ISO 5457 frame, the grid and the centring marks behind the sheet.
///
/// The schematic itself is not drawn here: this is the page the drawing sits on, in the same
/// millimetres, so the frame and the marks never drift from the paper the export prints.
void drawSheetFrame(QPainter *painter, const FluidSession &session, const Fit &fit,
                    qreal sheetWidthMm, qreal sheetHeightMm);
