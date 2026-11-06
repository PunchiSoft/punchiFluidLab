// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <QPainter>

#include "punchi_fluid.h"

/// Where a frame of drawing goes in a widget: a scale and an origin.
struct Fit {
    qreal pixelsPerMm = 1.0;
    qreal originX = 0.0;
    qreal originY = 0.0;
};

/// Fits an area of the drawing into a pixel rectangle, centred.
[[nodiscard]] Fit fitArea(qreal widthPx, qreal heightPx, float areaWidthMm, float areaHeightMm);

/// Replays one [`PfFrame`] onto a painter, in millimetres.
///
/// The sheet and every sidebar thumbnail use this one function, so a component cannot
/// look different in the palette than on the sheet — the same argument that made the
/// export backends share `Surface`.
void replayFrame(QPainter *painter, const PfFrame &frame, const Fit &fit);
