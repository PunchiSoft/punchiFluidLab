// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <QPointer>
#include <QQmlEngine>
#include <QQuickPaintedItem>

#include "punchi_fluid.h"
#include "replay.h"
#include "session.h"

/// The sheet: one frame of the drawing, replayed onto a `QPainter`.
///
/// This is the whole of the toolkit's involvement, the same way `EguiSurface` was on
/// the egui side. The drawing stays in `punchi-fluid-draw`, in millimetres; this class
/// applies a scale and a translation and then draws every call in order.
class SchematicItem : public QQuickPaintedItem
{
    Q_OBJECT
    QML_ELEMENT

    Q_PROPERTY(FluidSession *session READ session WRITE setSession NOTIFY sessionChanged)
    Q_PROPERTY(int paintCount READ paintCount NOTIFY paintCountChanged)

public:
    explicit SchematicItem(QQuickItem *parent = nullptr);

    void paint(QPainter *painter) override;

    FluidSession *session() const { return m_session; }
    void setSession(FluidSession *session);

    /// Frames replayed since start. Exposed so a test can prove an idle window stops.
    int paintCount() const { return m_paintCount; }

signals:
    void sessionChanged();
    void paintCountChanged();

protected:
    void mousePressEvent(QMouseEvent *event) override;
    void mouseReleaseEvent(QMouseEvent *event) override;
    void hoverMoveEvent(QHoverEvent *event) override;
    void hoverLeaveEvent(QHoverEvent *event) override;

private:
    /// The logical drawing area in millimetres, the same one the egui canvas used.
    static constexpr float kLogicalWidth = 200.0F;
    static constexpr float kLogicalHeight = 130.0F;

    /// `true` when the point falls on the valve, in millimetres of the drawing.
    [[nodiscard]] bool onValve(const QPointF &position) const;
    [[nodiscard]] Fit placement() const;

    QPointer<FluidSession> m_session;
    int m_paintCount = 0;
    bool m_pressedOnValve = false;
    bool m_hoveringValve = false;
};
