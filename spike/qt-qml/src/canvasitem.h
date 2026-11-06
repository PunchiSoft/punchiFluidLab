// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <QPointer>
#include <QQuickPaintedItem>
#include <QQmlEngine>

#include "punchi_fluid.h"
#include "session.h"

/// The sheet: replays one [`PfFrame`] onto a `QPainter`.
///
/// This is the whole of the toolkit's involvement, the same way `EguiSurface` was on
/// the egui side. The drawing stays in `punchi-fluid-draw`, in millimetres; this class
/// applies a scale and a translation and then draws every call in order.
class SchematicItem : public QQuickPaintedItem
{
    Q_OBJECT
    QML_ELEMENT

    Q_PROPERTY(FluidSession *session READ session WRITE setSession NOTIFY sessionChanged)

public:
    explicit SchematicItem(QQuickItem *parent = nullptr);

    void paint(QPainter *painter) override;

    FluidSession *session() const { return m_session; }
    void setSession(FluidSession *session);

signals:
    void sessionChanged();

protected:
    void mousePressEvent(QMouseEvent *event) override;
    void mouseReleaseEvent(QMouseEvent *event) override;

private:
    /// The logical drawing area in millimetres, the same one the egui canvas used.
    static constexpr float kLogicalWidth = 200.0F;
    static constexpr float kLogicalHeight = 130.0F;

    struct Fit {
        qreal pixelsPerMm;
        qreal originX;
        qreal originY;
    };

    Fit fit() const;
    void replay(QPainter *painter, const PfFrame &frame, const Fit &fit);
    void drawTextCall(QPainter *painter, const PfFrame &frame, const PfCall &call, const Fit &fit);

    QPointer<FluidSession> m_session;
};
