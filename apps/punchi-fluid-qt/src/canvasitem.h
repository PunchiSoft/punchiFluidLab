// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <QPointF>
#include <QPointer>
#include <QString>
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

    /// Places a catalog entry at a point of this item, in widget pixels.
    ///
    /// The sidebar drag ends here so the conversion from pixels to sheet millimetres
    /// happens once, in the same function the drop handler uses.
    Q_INVOKABLE void placeCatalogAt(int catalogIndex, const QPointF &position);

    /// Where a point of the sheet, in millimetres, sits inside this item, in pixels.
    /// Exposed so a test can aim at the valve without duplicating the fit.
    Q_INVOKABLE QPointF scenePointForSheet(const QPointF &sheetMm) const;

signals:
    void sessionChanged();
    void paintCountChanged();

protected:
    void mousePressEvent(QMouseEvent *event) override;
    void mouseReleaseEvent(QMouseEvent *event) override;
    void hoverMoveEvent(QHoverEvent *event) override;
    void hoverLeaveEvent(QHoverEvent *event) override;
    void mouseMoveEvent(QMouseEvent *event) override;
    void dragEnterEvent(QDragEnterEvent *event) override;
    void dragMoveEvent(QDragMoveEvent *event) override;
    void dropEvent(QDropEvent *event) override;
    void keyPressEvent(QKeyEvent *event) override;

private:
    /// The logical drawing area in millimetres, the same one the egui canvas used.
    static constexpr float kLogicalWidth = 200.0F;
    static constexpr float kLogicalHeight = 130.0F;

    /// `true` when the point falls on the valve, in millimetres of the drawing.
    [[nodiscard]] bool onValve(const QPointF &position) const;
    [[nodiscard]] Fit placement() const;

    QPointer<FluidSession> m_session;
    int m_paintCount = 0;
    /// Sheet position of a point of the widget, in millimetres.
    [[nodiscard]] QPointF toSheet(const QPointF &position) const;

    /// Placement whose valve is held down in simulation, or `-1`.
    int m_pressedValveIndex = -1;
    bool m_hoveringValve = false;
    /// Which component the press landed on, and where inside it, so dragging keeps the
    /// grab offset instead of snapping the component's centre to the pointer.
    /// The port a connection drag started from. `-1` when no pipe is being drawn; the
    /// string is the port id, and the two points are in millimetres of the sheet.
    int m_connecting = -1;
    QString m_connectPort;
    QPointF m_connectFrom;
    QPointF m_connectCursor;
    bool m_hoveringPort = false;

    int m_dragging = -1;
    QPointF m_dragOffset;
    bool m_moved = false;
    /// The pipe whose middle segment is being dragged. `-1` when no pipe is being
    /// re-routed; a click that lands on a pipe starts here and a move makes it an edit.
    int m_routing = -1;
    /// When and where the last press landed, so a second press in place counts as the
    /// double click the reference turns a component with.
    qint64 m_lastPressMs = 0;
    QPointF m_lastPressSheet;
};
