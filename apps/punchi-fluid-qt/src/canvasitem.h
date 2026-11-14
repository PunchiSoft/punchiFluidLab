// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <QPointF>
#include <QPointer>
#include <QSizeF>
#include <QString>
#include <QQmlEngine>
#include <QQuickPaintedItem>

#include "punchi_fluid.h"
#include "replay.h"
#include "session.h"

class QWheelEvent;

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

    /// Where a point of the scene, in pixels, falls on the sheet, in millimetres. The
    /// inverse of `scenePointForSheet`, and the way a test states "the point under the
    /// pointer did not move" after a zoom.
    Q_INVOKABLE QPointF sheetPointForScene(const QPointF &scenePosition) const;

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
    void wheelEvent(QWheelEvent *event) override;

private:
    /// The paper the document is drawn on, in millimetres: the session's sheet, or A4 when
    /// there is no session yet.
    [[nodiscard]] QSizeF sheetSizeMm() const;

    /// `true` when the point falls on the valve, in millimetres of the drawing.
    [[nodiscard]] bool onValve(const QPointF &position) const;
    [[nodiscard]] Fit placement() const;
    /// The placement at `zoom` with the sheet centred, before the reader pans it.
    [[nodiscard]] Fit centredPlacement(double zoom) const;
    /// `m_pan` limited so the sheet never leaves a gap the reader cannot cross: when it
    /// is larger than the viewport the edge stops at the frame, and when it is smaller
    /// it stays centred.
    [[nodiscard]] QPointF limitedPan(const Fit &centred) const;
    /// Multiplies the zoom by `factor` while keeping the sheet point under `position`.
    void zoomAt(const QPointF &position, double factor);
    /// Moves the view by `delta` pixels, limited by `limitedPan`.
    void panBy(const QPointF &delta);

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
    /// The rubber band: the corner it started at and where the pointer is now.
    bool m_banding = false;
    QPointF m_bandFrom;
    QPointF m_bandTo;
    /// Component whose press carried `Mayús`. It becomes a copy when the pointer moves and
    /// a change of membership when it does not, which is how the reference tells the two
    /// gestures apart.
    int m_shiftPress = -1;
    QPointF m_shiftFrom;
    /// `true` while a drag moves the whole selection rather than one component.
    bool m_movingSelection = false;
    /// How far the sheet is moved inside the item, in pixels, on top of the zoom's own
    /// centring. View state, like the zoom: it never reaches the document.
    QPointF m_pan;
    /// A middle-button pan in progress, and where it started.
    bool m_panning = false;
    QPointF m_panFrom;
};
