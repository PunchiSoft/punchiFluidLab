// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#include "canvasitem.h"

#include "punchi_fluid.h"
#include "session.h"

#include <QFont>
#include <QFontMetricsF>
#include <QMouseEvent>
#include <QPainter>
#include <QPolygonF>
#include <QtMath>

#include <algorithm>

namespace {

QColor toColor(const PfRgb &colour)
{
    return QColor(colour.r, colour.g, colour.b);
}

} // namespace

SchematicItem::SchematicItem(QQuickItem *parent)
    : QQuickPaintedItem(parent)
{
    setAntialiasing(true);
    setOpaquePainting(false);
}

void SchematicItem::setSession(FluidSession *session)
{
    if (m_session == session) {
        return;
    }
    if (m_session) {
        disconnect(m_session, nullptr, this, nullptr);
    }
    m_session = session;
    if (m_session) {
        connect(m_session, &FluidSession::changed, this, [this] { update(); });
    }
    emit sessionChanged();
    update();
}

SchematicItem::Fit SchematicItem::fit() const
{
    const qreal width = std::max<qreal>(1.0, this->width());
    const qreal height = std::max<qreal>(1.0, this->height());
    const qreal scale = std::min(width / kLogicalWidth, height / kLogicalHeight);
    return Fit { scale, (width - kLogicalWidth * scale) * 0.5,
                 (height - kLogicalHeight * scale) * 0.5 };
}

void SchematicItem::paint(QPainter *painter)
{
    // Clear whatever the previous frame left: the item is transparent so the sheet
    // colour comes from the window, like any other Qt item.
    painter->setCompositionMode(QPainter::CompositionMode_Source);
    painter->fillRect(contentsBoundingRect(), Qt::transparent);
    painter->setCompositionMode(QPainter::CompositionMode_SourceOver);

    if (!m_session || !m_session->handle()) {
        return;
    }
    const Fit placement = fit();
    PfFrame frame {};
    const int32_t code = pf_session_build_frame(m_session->handle(), kLogicalWidth,
                                                kLogicalHeight,
                                                static_cast<float>(placement.pixelsPerMm), &frame);
    if (code != PF_OK) {
        return;
    }
    replay(painter, frame, placement);
}

void SchematicItem::replay(QPainter *painter, const PfFrame &frame, const Fit &placement)
{
    painter->save();
    painter->setRenderHint(QPainter::Antialiasing, true);
    painter->translate(placement.originX, placement.originY);
    // From here on everything is drawn in millimetres of the drawing, which is the
    // whole point of the Surface trait: the same numbers the PDF uses.
    painter->scale(placement.pixelsPerMm, placement.pixelsPerMm);

    for (uint32_t index = 0; index < frame.calls_len; ++index) {
        const PfCall &call = frame.calls[index];
        if (call.kind == PF_KIND_TEXT) {
            drawTextCall(painter, frame, call, placement);
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

void SchematicItem::drawTextCall(QPainter *painter, const PfFrame &frame, const PfCall &call,
                                 const Fit &placement)
{
    painter->save();
    // Text is drawn outside the millimetre transform: a font size is in device pixels,
    // and scaling it twice is the classic way to lose the numerals.
    painter->resetTransform();
    const QPointF anchor(placement.originX + call.rect[0] * placement.pixelsPerMm,
                         placement.originY + call.rect[1] * placement.pixelsPerMm);
    QFont font = painter->font();
    // `size` is a cap height and a font size is the em, roughly 1.4 times bigger. The
    // egui surface uses the same factor, so the window and the window agree.
    font.setPixelSize(std::max(4, qRound(call.size * 1.4 * placement.pixelsPerMm)));
    painter->setFont(font);
    painter->setPen(toColor(call.fill));

    const QString text = QString::fromUtf8(
        reinterpret_cast<const char *>(frame.text + call.text_at), static_cast<int>(call.text_len));
    const QFontMetricsF metrics(font);
    // A line box centred on the anchor, tall enough for ascenders and descenders: a box
    // the height of the cap clips the top of the first row, which is how the supply
    // label lost its "C" the first time this ran.
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

void SchematicItem::mousePressEvent(QMouseEvent *event)
{
    event->accept();
}

void SchematicItem::mouseReleaseEvent(QMouseEvent *event)
{
    if (!m_session || !m_session->handle() || event->button() != Qt::LeftButton) {
        return;
    }
    const Fit placement = fit();
    const qreal x = (event->position().x() - placement.originX) / placement.pixelsPerMm;
    const qreal y = (event->position().y() - placement.originY) / placement.pixelsPerMm;
    if (pf_session_hit_valve(m_session->handle(), static_cast<float>(x), static_cast<float>(y))
        == 1) {
        m_session->toggleValve();
        update();
    }
    event->accept();
}
