// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#include "previewitem.h"

#include "replay.h"

#include <QPainter>

PreviewItem::PreviewItem(QQuickItem *parent)
    : QQuickPaintedItem(parent)
{
    setAntialiasing(true);
    setOpaquePainting(false);
}

void PreviewItem::setSession(FluidSession *session)
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

void PreviewItem::setIndex(int index)
{
    if (m_index == index) {
        return;
    }
    m_index = index;
    emit indexChanged();
    update();
}

void PreviewItem::paint(QPainter *painter)
{
    painter->setCompositionMode(QPainter::CompositionMode_Source);
    painter->fillRect(contentsBoundingRect(), Qt::transparent);
    painter->setCompositionMode(QPainter::CompositionMode_SourceOver);

    if (!m_session || !m_session->handle() || m_index < 0) {
        return;
    }
    const Fit placement = fitArea(width(), height(), kAreaMm, kAreaMm);
    PfFrame frame {};
    const int32_t code =
        pf_session_preview(m_session->handle(), static_cast<uint32_t>(m_index), kAreaMm, kAreaMm,
                           static_cast<float>(placement.pixelsPerMm), &frame);
    if (code != PF_OK) {
        return;
    }
    // A thumbnail is small: the drawing already skips numerals below a device scale, and
    // the replay is the same one the sheet uses.
    replayFrame(painter, frame, placement);
}
