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
        painter->setPen(QPen(frameInk, std::max<qreal>(1.0, 0.7 * fit.pixelsPerMm)));
        painter->drawRect(QRectF(fit.originX + left * fit.pixelsPerMm,
                                 fit.originY + top * fit.pixelsPerMm,
                                 (right - left) * fit.pixelsPerMm,
                                 (bottom - top) * fit.pixelsPerMm));

        // ISO 5457 centring marks: a line from the frame across the border to the paper
        // edge, with a small arrow. The form is free; the line is the recommended part and
        // spanning the whole border keeps the left filing margin and the other three sides
        // looking the same instead of leaving a gap on the left.
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
        const qreal arrow = 3.0 * scale;
        const qreal half = 1.5 * scale;
        painter->setBrush(frameInk);
        const auto triangle = [painter](QPointF tip, QPointF leftBase, QPointF rightBase) {
            painter->drawPolygon(QPolygonF { tip, leftBase, rightBase });
        };
        if (frameTop > paperTop + 0.5) {
            painter->drawLine(QPointF(middleX, frameTop), QPointF(middleX, paperTop));
            triangle(QPointF(middleX, std::min(paperTop + arrow, frameTop)),
                     QPointF(middleX - half, paperTop),
                     QPointF(middleX + half, paperTop));
        }
        if (frameBottom < paperBottom - 0.5) {
            painter->drawLine(QPointF(middleX, frameBottom), QPointF(middleX, paperBottom));
            triangle(QPointF(middleX, std::max(paperBottom - arrow, frameBottom)),
                     QPointF(middleX - half, paperBottom),
                     QPointF(middleX + half, paperBottom));
        }
        if (frameLeft > paperLeft + 0.5) {
            painter->drawLine(QPointF(frameLeft, middleY), QPointF(paperLeft, middleY));
            triangle(QPointF(std::min(paperLeft + arrow, frameLeft), middleY),
                     QPointF(paperLeft, middleY - half),
                     QPointF(paperLeft, middleY + half));
        }
        if (frameRight < paperRight - 0.5) {
            painter->drawLine(QPointF(frameRight, middleY), QPointF(paperRight, middleY));
            triangle(QPointF(std::max(paperRight - arrow, frameRight), middleY),
                     QPointF(paperRight, middleY - half),
                     QPointF(paperRight, middleY + half));
        }
    }
    painter->restore();
}
