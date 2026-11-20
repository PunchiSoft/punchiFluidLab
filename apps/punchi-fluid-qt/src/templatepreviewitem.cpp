// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#include "templatepreviewitem.h"

#include "replay.h"

#include <QPainter>

TemplatePreviewItem::TemplatePreviewItem(QQuickItem *parent)
    : QQuickPaintedItem(parent)
{
    setAntialiasing(true);
    setOpaquePainting(false);
}

void TemplatePreviewItem::setSession(FluidSession *session)
{
    if (m_session == session) {
        return;
    }
    if (m_session) {
        disconnect(m_session, nullptr, this, nullptr);
    }
    m_session = session;
    if (m_session) {
        connect(m_session, &FluidSession::themeChanged, this, [this] { invalidate(); });
    }
    emit sessionChanged();
    invalidate();
}

void TemplatePreviewItem::setTemplateIndex(int index)
{
    if (m_templateIndex == index) {
        return;
    }
    m_templateIndex = index;
    emit templateIndexChanged();
    invalidate();
}

void TemplatePreviewItem::invalidate()
{
    m_image = QImage();
    m_attempted = false;
    if (m_ready) {
        m_ready = false;
        emit readyChanged();
    }
    update();
}

bool TemplatePreviewItem::renderCache()
{
    m_attempted = true;
    if (!m_session || !m_session->handle() || m_templateIndex < 0
        || m_templateIndex >= m_session->templateCount()) {
        return false;
    }

    QImage image(kRasterWidth, kRasterHeight, QImage::Format_ARGB32_Premultiplied);
    if (image.isNull()) {
        return false;
    }
    image.fill(Qt::transparent);

    const Fit fit = fitArea(image.width(), image.height(), kPreviewWidthMm, kPreviewHeightMm);
    PfFrame frame {};
    if (pf_session_preview_template(m_session->handle(), static_cast<uint32_t>(m_templateIndex),
                                    kPreviewWidthMm, kPreviewHeightMm,
                                    static_cast<float>(fit.pixelsPerMm), &frame)
        != PF_OK) {
        return false;
    }

    QPainter imagePainter(&image);
    replayFrame(&imagePainter, frame, fit);
    imagePainter.end();
    m_image = std::move(image);
    m_ready = true;
    emit readyChanged();
    return true;
}

void TemplatePreviewItem::paint(QPainter *painter)
{
    if (m_image.isNull() && !m_attempted) {
        renderCache();
    }
    if (m_image.isNull()) {
        return;
    }
    // Keep the drawing's aspect inside the plate: the plate is authored at the raster's
    // 512:288 ratio, so this fills the width; a mismatch leaves a transparent letterbox
    // instead of stretching the symbol.
    const QRectF bounds = boundingRect();
    const QSizeF imageSize = QSizeF(m_image.size());
    const QSizeF scaled = imageSize.scaled(bounds.size(), Qt::KeepAspectRatio);
    const QRectF target(bounds.center().x() - scaled.width() * 0.5,
                        bounds.center().y() - scaled.height() * 0.5, scaled.width(),
                        scaled.height());
    painter->drawImage(target, m_image);
}
