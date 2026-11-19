// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <QImage>
#include <QQmlEngine>
#include <QQuickPaintedItem>
#include <QPointer>

#include "session.h"

/// A small, cached drawing of one authored start-page circuit.
class TemplatePreviewItem : public QQuickPaintedItem
{
    Q_OBJECT
    QML_ELEMENT

    Q_PROPERTY(FluidSession *session READ session WRITE setSession NOTIFY sessionChanged)
    Q_PROPERTY(int templateIndex READ templateIndex WRITE setTemplateIndex NOTIFY templateIndexChanged)
    Q_PROPERTY(bool ready READ ready NOTIFY readyChanged)

public:
    explicit TemplatePreviewItem(QQuickItem *parent = nullptr);

    void paint(QPainter *painter) override;

    FluidSession *session() const { return m_session; }
    void setSession(FluidSession *session);

    int templateIndex() const { return m_templateIndex; }
    void setTemplateIndex(int index);
    bool ready() const { return m_ready; }

signals:
    void sessionChanged();
    void templateIndexChanged();
    void readyChanged();

private:
    void invalidate();
    bool renderCache();

    static constexpr int kRasterWidth = 512;
    static constexpr int kRasterHeight = 288;
    static constexpr float kPreviewWidthMm = 180.0F;
    static constexpr float kPreviewHeightMm = 100.0F;

    QPointer<FluidSession> m_session;
    QImage m_image;
    int m_templateIndex = -1;
    bool m_attempted = false;
    bool m_ready = false;
};
