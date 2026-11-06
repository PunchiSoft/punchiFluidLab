// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <QQmlEngine>
#include <QQuickPaintedItem>

#include "punchi_fluid.h"
#include "session.h"

/// One catalog entry, drawn with its own symbol.
///
/// The sidebar shows components by the definition's own geometry, never by a separate
/// icon set: a second drawing of a symbol is a second thing that can be wrong.
class PreviewItem : public QQuickPaintedItem
{
    Q_OBJECT
    QML_ELEMENT

    Q_PROPERTY(FluidSession *session READ session WRITE setSession NOTIFY sessionChanged)
    Q_PROPERTY(int index READ index WRITE setIndex NOTIFY indexChanged)

public:
    explicit PreviewItem(QQuickItem *parent = nullptr);

    void paint(QPainter *painter) override;

    FluidSession *session() const { return m_session; }
    void setSession(FluidSession *session);

    int index() const { return m_index; }
    void setIndex(int index);

signals:
    void sessionChanged();
    void indexChanged();

private:
    static constexpr float kAreaMm = 40.0F;

    QPointer<FluidSession> m_session;
    int m_index = -1;
};
