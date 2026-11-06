// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#include "replay.h"

#include <QFont>
#include <QFontMetricsF>
#include <QPolygonF>

#include <algorithm>

namespace {

QColor toColor(const PfRgb &colour)
{
    return QColor(colour.r, colour.g, colour.b);
}

void drawTextCall(QPainter *painter, const PfFrame &frame, const PfCall &call, const Fit &fit)
{
    painter->save();
    // Text is drawn outside the millimetre transform: a font size is in device pixels,
    // and scaling it twice is the classic way to lose the numerals.
    painter->resetTransform();
    const QPointF anchor(fit.originX + call.rect[0] * fit.pixelsPerMm,
                         fit.originY + call.rect[1] * fit.pixelsPerMm);
    QFont font = painter->font();
    // `size` is a cap height and a font size is the em, roughly 1.4 times bigger; the
    // export backend uses the same factor, so the window and the paper agree.
    font.setPixelSize(std::max(4, qRound(call.size * 1.4 * fit.pixelsPerMm)));
    painter->setFont(font);
    painter->setPen(toColor(call.fill));

    const QString text = QString::fromUtf8(reinterpret_cast<const char *>(frame.text + call.text_at),
                                           static_cast<int>(call.text_len));
    const QFontMetricsF metrics(font);
    // A line box centred on the anchor, tall enough for ascenders and descenders: a box
    // the height of the cap clips the top of the first row.
    const qreal height = metrics.height();
    QRectF box(anchor.x() - 500.0, anchor.y() - height * 0.5, 1000.0, height);
    Qt::Alignment alignment = Qt::AlignHCenter;
    if (call.anchor == 0U) {
        alignment = Qt::AlignLeft;
        box.setLeft(anchor.x());
    } else if (call.anchor == 2U) {
        alignment = Qt::AlignRight;
        box.setRight(anchor.x());
    }
    painter->drawText(box, alignment | Qt::AlignVCenter, text);
    painter->restore();
}

} // namespace

Fit fitArea(qreal widthPx, qreal heightPx, float areaWidthMm, float areaHeightMm)
{
    const qreal width = std::max<qreal>(1.0, widthPx);
    const qreal height = std::max<qreal>(1.0, heightPx);
    const qreal scale = std::min(width / areaWidthMm, height / areaHeightMm);
    return Fit { scale, (width - areaWidthMm * scale) * 0.5, (height - areaHeightMm * scale) * 0.5 };
}

void replayFrame(QPainter *painter, const PfFrame &frame, const Fit &fit)
{
    painter->save();
    painter->setRenderHint(QPainter::Antialiasing, true);
    painter->translate(fit.originX, fit.originY);
    // From here on everything is drawn in millimetres of the drawing: the same numbers
    // the PDF uses.
    painter->scale(fit.pixelsPerMm, fit.pixelsPerMm);

    for (uint32_t index = 0; index < frame.calls_len; ++index) {
        const PfCall &call = frame.calls[index];
        if (call.kind == PF_KIND_TEXT) {
            drawTextCall(painter, frame, call, fit);
            continue;
        }
        QPen pen(toColor(call.stroke), call.width);
        pen.setCapStyle(Qt::FlatCap);
        pen.setJoinStyle(Qt::MiterJoin);
        painter->setPen(pen);
        painter->setBrush((call.flags & PF_HAS_FILL) != 0U ? QBrush(toColor(call.fill))
                                                          : QBrush(Qt::NoBrush));

        switch (call.kind) {
        case PF_KIND_LINE: {
            QPolygonF polygon;
            polygon.reserve(static_cast<int>(call.points_len));
            for (uint32_t i = 0; i < call.points_len; ++i) {
                const PfPoint &point = frame.points[call.points_at + i];
                polygon << QPointF(point.x, point.y);
            }
            painter->setBrush(Qt::NoBrush);
            painter->drawPolyline(polygon);
            break;
        }
        case PF_KIND_POLYGON: {
            QPolygonF polygon;
            polygon.reserve(static_cast<int>(call.points_len));
            for (uint32_t i = 0; i < call.points_len; ++i) {
                const PfPoint &point = frame.points[call.points_at + i];
                polygon << QPointF(point.x, point.y);
            }
            painter->drawPolygon(polygon);
            break;
        }
        case PF_KIND_RECT: {
            const QRectF rect(call.rect[0], call.rect[1], call.rect[2] - call.rect[0],
                              call.rect[3] - call.rect[1]);
            if (call.corner > 0.0F) {
                painter->drawRoundedRect(rect, call.corner, call.corner);
            } else {
                painter->drawRect(rect);
            }
            break;
        }
        case PF_KIND_CIRCLE: {
            painter->drawEllipse(QPointF(call.rect[0], call.rect[1]), call.rect[2], call.rect[2]);
            break;
        }
        case PF_KIND_ARC: {
            const qreal radius = call.rect[2];
            const QRectF box(call.rect[0] - radius, call.rect[1] - radius, radius * 2.0,
                             radius * 2.0);
            painter->setBrush(Qt::NoBrush);
            // The drawing measures angles clockwise from the positive x axis; Qt scans
            // anticlockwise from the same axis and wants sixteenths of a degree.
            painter->drawArc(box, qRound(-call.angles[0] * 16.0),
                             qRound(-(call.angles[1] - call.angles[0]) * 16.0));
            break;
        }
        default:
            break;
        }
    }
    painter->restore();
}
