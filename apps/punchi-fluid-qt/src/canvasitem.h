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

    /// Fits the whole sheet, margins included: the paper is what the reader asked to see,
    /// edges and all. It is also the middle button's double click. View state only, and one
    /// step for `previousView`.
    Q_INVOKABLE void fitToSheet();
    /// Returns to the view before the last fit or zoom at the pointer. One step, so a
    /// second call does nothing instead of walking a history nobody asked for.
    Q_INVOKABLE void previousView();
    Q_PROPERTY(bool canGoBack READ canGoBack NOTIFY viewHistoryChanged)
    /// A one-use gesture: the next left drag is a rectangle to zoom into.
    Q_PROPERTY(bool zoomWindowActive READ zoomWindowActive NOTIFY zoomWindowChanged)
    [[nodiscard]] bool zoomWindowActive() const { return m_zoomWindow; }
    Q_INVOKABLE void beginZoomWindow();
    Q_INVOKABLE void cancelZoomWindow();
    [[nodiscard]] bool canGoBack() const { return m_hasPreviousView; }
    /// Where the sheet sits in the item: the scale the zoom produced and the pan's offset.
    [[nodiscard]] Fit viewPlacement() const { return placement(); }

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
    void viewHistoryChanged();
    void zoomWindowChanged();

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
    /// Keeps the view for `previousView` before a fit or a zoom changes it.
    void rememberView();
    /// Zooms so the rectangle, in item pixels, fills the canvas and stays centred.
    void zoomToWindow(const QRectF &rect);
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
    /// The last middle press, so a second one in place is the fit and not a pan.
    qint64 m_lastMiddleMs = 0;
    QPointF m_lastMiddleFrom;
    /// The view before the last fit or zoom, so one step back is possible.
    double m_previousZoom = 1.0;
    QPointF m_previousPan;
    bool m_hasPreviousView = false;
    /// `true` while the view is the fitted frame. A fitted frame can be smaller than 1:1
    /// and still sit off the paper's centre, so its pan must survive below 1:1.
    bool m_fitted = false;
    /// The zoom window: armed, dragging, and the rectangle in item pixels.
    bool m_zoomWindow = false;
    bool m_zoomDragging = false;
    QPointF m_zoomFrom;
    QPointF m_zoomTo;
};
