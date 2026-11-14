// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <QPainter>

#include "replay.h"

class FluidSession;

/// Draws the paper, the ISO 5457 frame, the grid and the centring marks behind the sheet.
///
/// The schematic itself is not drawn here: this is the page the drawing sits on, in the same
/// millimetres, so the frame and the marks never drift from the paper the export prints.
void drawSheetFrame(QPainter *painter, const FluidSession &session, const Fit &fit,
                    qreal sheetWidthMm, qreal sheetHeightMm);
