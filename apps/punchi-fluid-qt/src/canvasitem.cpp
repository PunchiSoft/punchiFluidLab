// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#include "canvasitem.h"

#include "replay.h"

#include <QMouseEvent>
#include <QPainter>

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
    const Fit placement = fitArea(width(), height(), kLogicalWidth, kLogicalHeight);
    PfFrame frame {};
    const int32_t code = pf_session_build_frame(m_session->handle(), kLogicalWidth, kLogicalHeight,
                                                static_cast<float>(placement.pixelsPerMm), &frame);
    if (code != PF_OK) {
        return;
    }
    replayFrame(painter, frame, placement);
}

void SchematicItem::mouseReleaseEvent(QMouseEvent *event)
{
    if (!m_session || !m_session->handle() || event->button() != Qt::LeftButton) {
        return;
    }
    const Fit placement = fitArea(width(), height(), kLogicalWidth, kLogicalHeight);
    const qreal x = (event->position().x() - placement.originX) / placement.pixelsPerMm;
    const qreal y = (event->position().y() - placement.originY) / placement.pixelsPerMm;
    if (pf_session_hit_valve(m_session->handle(), static_cast<float>(x), static_cast<float>(y)) == 1) {
        m_session->toggleValve();
        update();
    }
    event->accept();
}
