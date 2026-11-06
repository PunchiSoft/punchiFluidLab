// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#include "canvasitem.h"

#include "replay.h"

#include <QCursor>
#include <QHoverEvent>
#include <QMouseEvent>
#include <QPainter>

SchematicItem::SchematicItem(QQuickItem *parent)
    : QQuickPaintedItem(parent)
{
    setAntialiasing(true);
    setOpaquePainting(false);
    setAcceptHoverEvents(true);
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

void SchematicItem::paint(QPainter *painter)
{
    ++m_paintCount;
    emit paintCountChanged();
    // The item is transparent so the sheet colour comes from the window, like any other
    // Qt item.
    painter->setCompositionMode(QPainter::CompositionMode_Source);
    painter->fillRect(contentsBoundingRect(), Qt::transparent);
    painter->setCompositionMode(QPainter::CompositionMode_SourceOver);

    if (!m_session || !m_session->handle()) {
        return;
    }
    const Fit placement = this->placement();
    PfFrame frame {};
    const int32_t code = pf_session_build_frame(m_session->handle(), kLogicalWidth, kLogicalHeight,
                                                static_cast<float>(placement.pixelsPerMm), &frame);
    if (code != PF_OK) {
        return;
    }
    replayFrame(painter, frame, placement);

    // The ring is drawn around the valve's real box, which the core knows and the front
    // end does not have to guess.
    if (m_hoveringValve && m_session) {
        float box[4] = {0.0F, 0.0F, 0.0F, 0.0F};
        if (pf_session_valve_rect(m_session->handle(), box) == PF_OK) {
            const Fit fit = placement;
            const qreal margin = 2.0 * fit.pixelsPerMm;
            const QRectF ring(fit.originX + box[0] * fit.pixelsPerMm - margin,
                              fit.originY + box[1] * fit.pixelsPerMm - margin,
                              (box[2] - box[0]) * fit.pixelsPerMm + margin * 2.0,
                              (box[3] - box[1]) * fit.pixelsPerMm + margin * 2.0);
            painter->save();
            QPen pen(m_session->tokens().accent, 1.5);
            pen.setStyle(Qt::DashLine);
            painter->setPen(pen);
            painter->setBrush(Qt::NoBrush);
            painter->drawRoundedRect(ring, 4, 4);
            painter->restore();
        }
    }
}

Fit SchematicItem::placement() const
{
    return fitArea(width(), height(), kLogicalWidth, kLogicalHeight);
}

bool SchematicItem::onValve(const QPointF &position) const
{
    if (!m_session || !m_session->handle()) {
        return false;
    }
    const Fit fit = placement();
    const qreal x = (position.x() - fit.originX) / fit.pixelsPerMm;
    const qreal y = (position.y() - fit.originY) / fit.pixelsPerMm;
    return pf_session_hit_valve(m_session->handle(), static_cast<float>(x), static_cast<float>(y)) == 1;
}

void SchematicItem::mousePressEvent(QMouseEvent *event)
{
    if (!m_session || event->button() != Qt::LeftButton) {
        return;
    }
    if (!onValve(event->position())) {
        return;
    }
    // In the editor a click selects and opens; in simulation the same click actuates, held
    // for as long as the button is down — the split every schematic simulator has.
    if (m_session->editing()) {
        m_session->inspectValve();
    } else {
        m_pressedOnValve = true;
        m_session->pressValve();
    }
    event->accept();
}

void SchematicItem::mouseReleaseEvent(QMouseEvent *event)
{
    if (event->button() != Qt::LeftButton) {
        return;
    }
    if (m_pressedOnValve) {
        m_pressedOnValve = false;
        if (m_session) {
            m_session->releaseValve();
        }
    }
    event->accept();
}

void SchematicItem::hoverMoveEvent(QHoverEvent *event)
{
    const bool over = onValve(event->position());
    if (over != m_hoveringValve) {
        m_hoveringValve = over;
        // The valve is the only thing on the sheet that answers to a click, so it has to
        // say so before the user tries: the cursors differ between the two modes.
        if (over) {
            setCursor(QCursor(m_session && m_session->editing() ? Qt::PointingHandCursor
                                                                : Qt::OpenHandCursor));
        } else {
            unsetCursor();
        }
        update();
    }
    event->accept();
}

void SchematicItem::hoverLeaveEvent(QHoverEvent *event)
{
    if (m_hoveringValve) {
        m_hoveringValve = false;
        unsetCursor();
        update();
    }
    event->accept();
}
