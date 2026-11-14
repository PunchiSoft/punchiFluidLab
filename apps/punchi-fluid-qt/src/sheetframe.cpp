// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#include "sheetframe.h"

#include "session.h"

#include <QColor>
#include <QPen>
#include <QPolygonF>

#include <algorithm>

namespace {
/// Ruling of the sheet's grid, in millimetres.
constexpr double kGridMm = 10.0;
} // namespace

void drawSheetFrame(QPainter *painter, const FluidSession &session, const Fit &fit,
                    qreal sheetWidth, qreal sheetHeight)
{
    const qreal left = session.sheetMarginLeft();
    const qreal top = session.sheetMarginTop();
    const qreal right = sheetWidth - session.sheetMarginRight();
    const qreal bottom = sheetHeight - session.sheetMarginBottom();

    if (session.grid()) {
        // The grid is the drawing's own millimetre ruling, so a component lines up with it
        // instead of floating. It stops at the frame, because the margin is paper, not
        // drawing space. Drawn first and faint: it is a guide, never a mark on the document.
        QColor line = session.tokens().inkMuted;
        line.setAlphaF(0.22F);
        painter->save();
        painter->setPen(QPen(line, 1.0));
        for (double x = left + kGridMm; x < right; x += kGridMm) {
            const qreal px = fit.originX + x * fit.pixelsPerMm;
            painter->drawLine(QPointF(px, fit.originY + top * fit.pixelsPerMm),
                              QPointF(px, fit.originY + bottom * fit.pixelsPerMm));
        }
        for (double y = top + kGridMm; y < bottom; y += kGridMm) {
            const qreal py = fit.originY + y * fit.pixelsPerMm;
            painter->drawLine(QPointF(fit.originX + left * fit.pixelsPerMm, py),
                              QPointF(fit.originX + right * fit.pixelsPerMm, py));
        }
        painter->restore();
    }

    // The edge of the paper, and the ISO 5457 frame inside it. Without them "outside the
    // sheet" is a finding the user cannot see.
    QColor edge = session.tokens().inkMuted;
    edge.setAlphaF(0.55F);
    painter->save();
    painter->setPen(QPen(edge, 1.0));
    painter->setBrush(Qt::NoBrush);
    painter->drawRect(QRectF(fit.originX, fit.originY, sheetWidth * fit.pixelsPerMm,
                             sheetHeight * fit.pixelsPerMm));
    if (left > 0.0 || top > 0.0 || right < sheetWidth || bottom < sheetHeight) {
        QColor frameInk = edge;
        frameInk.setAlphaF(0.8F);
        painter->setPen(QPen(frameInk, 1.0));
        painter->drawRect(QRectF(fit.originX + left * fit.pixelsPerMm,
                                 fit.originY + top * fit.pixelsPerMm,
                                 (right - left) * fit.pixelsPerMm,
                                 (bottom - top) * fit.pixelsPerMm));

        // ISO 5457 centring marks: a line from the frame outwards and a small arrow at the
        // paper edge. The form is free; the line is the recommended part.
        const qreal scale = fit.pixelsPerMm;
        const qreal paperLeft = fit.originX;
        const qreal paperTop = fit.originY;
        const qreal paperRight = paperLeft + sheetWidth * scale;
        const qreal paperBottom = paperTop + sheetHeight * scale;
        const qreal frameLeft = paperLeft + left * scale;
        const qreal frameTop = paperTop + top * scale;
        const qreal frameRight = paperLeft + right * scale;
        const qreal frameBottom = paperTop + bottom * scale;
        const qreal middleX = (paperLeft + paperRight) * 0.5;
        const qreal middleY = (paperTop + paperBottom) * 0.5;
        const qreal reach = 10.0 * scale;
        const qreal arrow = 3.0 * scale;
        const qreal half = 1.5 * scale;
        painter->setBrush(frameInk);
        const auto triangle = [painter](QPointF tip, QPointF leftBase, QPointF rightBase) {
            painter->drawPolygon(QPolygonF { tip, leftBase, rightBase });
        };
        const qreal topOuter = std::max(paperTop, frameTop - reach);
        if (topOuter < frameTop - 0.5) {
            painter->drawLine(QPointF(middleX, frameTop), QPointF(middleX, topOuter));
            triangle(QPointF(middleX, std::min(topOuter + arrow, frameTop)),
                     QPointF(middleX - half, topOuter),
                     QPointF(middleX + half, topOuter));
        }
        const qreal bottomOuter = std::min(paperBottom, frameBottom + reach);
        if (bottomOuter > frameBottom + 0.5) {
            painter->drawLine(QPointF(middleX, frameBottom), QPointF(middleX, bottomOuter));
            triangle(QPointF(middleX, std::max(bottomOuter - arrow, frameBottom)),
                     QPointF(middleX - half, bottomOuter),
                     QPointF(middleX + half, bottomOuter));
        }
        const qreal leftOuter = std::max(paperLeft, frameLeft - reach);
        if (leftOuter < frameLeft - 0.5) {
            painter->drawLine(QPointF(frameLeft, middleY), QPointF(leftOuter, middleY));
            triangle(QPointF(std::min(leftOuter + arrow, frameLeft), middleY),
                     QPointF(leftOuter, middleY - half),
                     QPointF(leftOuter, middleY + half));
        }
        const qreal rightOuter = std::min(paperRight, frameRight + reach);
        if (rightOuter > frameRight + 0.5) {
            painter->drawLine(QPointF(frameRight, middleY), QPointF(rightOuter, middleY));
            triangle(QPointF(std::max(rightOuter - arrow, frameRight), middleY),
                     QPointF(rightOuter, middleY - half),
                     QPointF(rightOuter, middleY + half));
        }
    }
    painter->restore();
}
