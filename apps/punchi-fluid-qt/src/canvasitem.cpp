// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#include "canvasitem.h"

#include "replay.h"

#include <QCursor>
#include <QDateTime>
#include <QGuiApplication>
#include <QStyleHints>
#include <QDragEnterEvent>
#include <QDropEvent>
#include <QKeyEvent>
#include <QMimeData>
#include <QHoverEvent>
#include <QMouseEvent>
#include <QPainter>

namespace {
/// How close, in millimetres of the sheet, a press must be to a port to mean it.
constexpr double kPortToleranceMm = 2.5;
/// Ruling of the sheet's grid, in millimetres.
constexpr double kGridMm = 10.0;
} // namespace

SchematicItem::SchematicItem(QQuickItem *parent)
    : QQuickPaintedItem(parent)
{
    setAntialiasing(true);
    setOpaquePainting(false);
    setAcceptHoverEvents(true);
    // Without this the item accepts no button at all and Qt never delivers a press, so
    // the sheet looks inert to the mouse: no selection, no actuation, no dragging.
    setAcceptedMouseButtons(Qt::LeftButton);
    // The sidebar drops catalog entries here; the sheet is the only place that accepts
    // them, so a drop anywhere else is refused by Qt itself.
    setFlag(QQuickItem::ItemAcceptsDrops, true);
    setFlag(QQuickItem::ItemIsFocusScope, true);
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
    const Fit placement = this->placement();
    PfFrame frame {};
    const int32_t code = pf_session_build_frame(m_session->handle(), kLogicalWidth, kLogicalHeight,
                                                static_cast<float>(placement.pixelsPerMm), &frame);
    if (code != PF_OK) {
        return;
    }
    if (m_session->grid()) {
        const Fit fit = placement;
        // The grid is the drawing's own millimetre ruling, so a component lines up with it
        // instead of floating. Drawn first and faint: it is a guide, never a mark on the
        // document.
        QColor line = m_session->tokens().inkMuted;
        line.setAlphaF(0.22F);
        painter->save();
        painter->setPen(QPen(line, 1.0));
        for (double x = kGridMm; x < kLogicalWidth; x += kGridMm) {
            const qreal px = fit.originX + x * fit.pixelsPerMm;
            painter->drawLine(QPointF(px, fit.originY),
                              QPointF(px, fit.originY + kLogicalHeight * fit.pixelsPerMm));
        }
        for (double y = kGridMm; y < kLogicalHeight; y += kGridMm) {
            const qreal py = fit.originY + y * fit.pixelsPerMm;
            painter->drawLine(QPointF(fit.originX, py),
                              QPointF(fit.originX + kLogicalWidth * fit.pixelsPerMm, py));
        }
        painter->restore();
    }
    {
        // The edge of the sheet. Without it "outside the sheet" is a finding the user cannot
        // see, and a circuit that overflows looks no different from one that fits.
        const Fit fit = placement;
        QColor edge = m_session->tokens().inkMuted;
        edge.setAlphaF(0.55F);
        painter->save();
        painter->setPen(QPen(edge, 1.0));
        painter->setBrush(Qt::NoBrush);
        painter->drawRect(QRectF(fit.originX, fit.originY, kLogicalWidth * fit.pixelsPerMm,
                                 kLogicalHeight * fit.pixelsPerMm));
        painter->restore();
    }
    replayFrame(painter, frame, placement);

    if (m_connecting >= 0 && m_session) {
        const Fit fit = placement;
        painter->save();
        QPen pen(m_session->tokens().accent, 1.5);
        pen.setStyle(Qt::DashLine);
        painter->setPen(pen);
        painter->setBrush(Qt::NoBrush);
        painter->drawLine(QPointF(fit.originX + m_connectFrom.x() * fit.pixelsPerMm,
                                  fit.originY + m_connectFrom.y() * fit.pixelsPerMm),
                          QPointF(fit.originX + m_connectCursor.x() * fit.pixelsPerMm,
                                  fit.originY + m_connectCursor.y() * fit.pixelsPerMm));
        painter->restore();
    }

    // Every fault the check found, marked where it happens. A ring and a cross, not a colour
    // alone: the marker has to read in a screenshot and to a colour-blind user.
    if (m_session && m_session->checkOpen()) {
        const Fit fit = placement;
        const qreal radius = 4.0 * fit.pixelsPerMm;
        painter->save();
        QPen pen(m_session->tokens().accent, 1.5);
        pen.setStyle(Qt::DashLine);
        painter->setPen(pen);
        painter->setBrush(Qt::NoBrush);
        for (int index = 0; index < m_session->checkCount(); ++index) {
            float at[2] = {0.0F, 0.0F};
            if (pf_session_finding_at(m_session->handle(), static_cast<uint32_t>(index), at)
                != PF_OK) {
                continue;
            }
            const QPointF centre(fit.originX + at[0] * fit.pixelsPerMm,
                                 fit.originY + at[1] * fit.pixelsPerMm);
            painter->drawEllipse(centre, radius, radius);
            painter->drawLine(centre + QPointF(-radius, -radius),
                              centre + QPointF(radius, radius));
            painter->drawLine(centre + QPointF(-radius, radius),
                              centre + QPointF(radius, -radius));
        }
        painter->restore();
    }

    // The ring is drawn around the valve's real box, which the core knows and the front
    // end does not have to guess.
    if (m_hoveringValve && m_session) {
        float box[4] = {0.0F, 0.0F, 0.0F, 0.0F};
        if (pf_session_valve_rect(m_session->handle(), box) == PF_OK) {
            const Fit fit = placement;
            const qreal margin = 2.0 * fit.pixelsPerMm;
            const QRectF ring(fit.originX + box[0] * fit.pixelsPerMm - margin,
                              fit.originY + box[1] * fit.pixelsPerMm - margin,
                              (box[2] - box[0]) * fit.pixelsPerMm + margin * 2.0,
                              (box[3] - box[1]) * fit.pixelsPerMm + margin * 2.0);
            painter->save();
            QPen pen(m_session->tokens().accent, 1.5);
            pen.setStyle(Qt::DashLine);
            painter->setPen(pen);
            painter->setBrush(Qt::NoBrush);
            painter->drawRoundedRect(ring, 4, 4);
            painter->restore();
        }
    }
}

QPointF SchematicItem::toSheet(const QPointF &position) const
{
    const Fit fit = this->placement();
    return QPointF((position.x() - fit.originX) / fit.pixelsPerMm,
                   (position.y() - fit.originY) / fit.pixelsPerMm);
}

Fit SchematicItem::placement() const
{
    const Fit fitted = fitArea(width(), height(), kLogicalWidth, kLogicalHeight);
    const double zoom = m_session ? m_session->zoom() : 1.0;
    if (qFuzzyCompare(zoom, 1.0)) {
        return fitted;
    }
    // Zoom keeps the middle of the sheet in the middle of the item, so magnifying does not
    // throw the circuit off screen.
    const qreal scale = fitted.pixelsPerMm * zoom;
    return Fit { scale, (width() - kLogicalWidth * scale) * 0.5,
                 (height() - kLogicalHeight * scale) * 0.5 };
}

bool SchematicItem::onValve(const QPointF &position) const
{
    if (!m_session || !m_session->handle()) {
        return false;
    }
    const Fit fit = placement();
    const qreal x = (position.x() - fit.originX) / fit.pixelsPerMm;
    const qreal y = (position.y() - fit.originY) / fit.pixelsPerMm;
    const int index = m_session->componentAt(x, y);
    return index >= 0 && m_session->valvePositionAt(index) >= 0;
}

void SchematicItem::mousePressEvent(QMouseEvent *event)
{
    if (!m_session || event->button() != Qt::LeftButton) {
        return;
    }
    const QPointF sheet = toSheet(event->position());

    // In the editor a click on the valve selects it for dragging, and a click on the
    // valve's own box opens its properties through the keyboard or a double click. In
    // simulation the same click actuates it, held for as long as the button is down —
    // the split every schematic simulator has.
    if (m_session->editing()) {
        // Ctrl and a double click turns the component under the pointer, anticlockwise;
        // Shift with it turns clockwise. It is the reference's gesture, so it is here
        // before the drag that the same press would start. The window system turns the
        // second click of a hand into a double click; a replayed press sequence carries no
        // such mark, so the item counts two presses in place itself.
        const Qt::KeyboardModifiers modifiers = event->modifiers();
        const qint64 now = QDateTime::currentMSecsSinceEpoch();
        const bool repeated =
            m_lastPressMs > 0 && now - m_lastPressMs <= QGuiApplication::styleHints()->mouseDoubleClickInterval()
            && (sheet - m_lastPressSheet).manhattanLength() <= 6.0;
        m_lastPressMs = now;
        m_lastPressSheet = sheet;
        const bool turned = event->type() == QEvent::MouseButtonDblClick
            || (event->type() == QEvent::MouseButtonPress && repeated);
        if (turned && (modifiers & Qt::ControlModifier) != 0) {
            const int target = m_session->componentAt(sheet.x(), sheet.y());
            if (target >= 0) {
                m_session->select(target);
                m_session->rotateSelected((modifiers & Qt::ShiftModifier) == 0);
                event->accept();
                return;
            }
        }
        // A port under the pointer is the start of a pipe, exactly as the reference does
        // it: press on one connection, drag, release on another.
        int portIndex = -1;
        QString port;
        if (m_session->portAt(sheet.x(), sheet.y(), kPortToleranceMm, &portIndex, &port)) {
            QPointF start;
            if (m_session->portPoint(portIndex, port, &start)) {
                m_connecting = portIndex;
                m_connectPort = port;
                m_connectFrom = start;
                m_connectCursor = sheet;
                setCursor(QCursor(Qt::CrossCursor));
                update();
                event->accept();
                return;
            }
        }
        int index = m_session->componentAt(sheet.x(), sheet.y());
        // Shift+drag copies: the reference duplicates the selection and drags the copy,
        // leaving the original in place.
        if (index >= 0 && (event->modifiers() & Qt::ShiftModifier) != 0) {
            const int copy = m_session->duplicateComponent(index, 0.0, 0.0);
            if (copy >= 0) {
                index = copy;
            }
        }
        if (index >= 0) {
            m_session->select(index);
        }
        if (index < 0) {
            const int pipe = m_session->pipeAt(sheet.x(), sheet.y(), kPortToleranceMm);
            if (pipe >= 0) {
                m_session->selectPipe(pipe);
                // Selecting and re-routing start on the same press: the reference lets the
                // reader drag the hose until the circuit reads clearly, and a click that
                // does not move is only a selection.
                m_routing = pipe;
                setFocus(true);
                update();
                event->accept();
                return;
            }
        }
        m_session->select(index);
        if (index >= 0) {
            m_dragging = index;
            m_moved = false;
            float bounds[4] = {0.0F, 0.0F, 0.0F, 0.0F};
            if (pf_session_component_rect(m_session->handle(), static_cast<uint32_t>(index), bounds)
                == PF_OK) {
                m_dragOffset = QPointF(sheet.x() - (bounds[0] + bounds[2]) * 0.5,
                                       sheet.y() - (bounds[1] + bounds[3]) * 0.5);
            }
            setFocus(true);
        }
        event->accept();
        return;
    }

    const int index = m_session->componentAt(sheet.x(), sheet.y());
    if (index >= 0 && m_session->valvePositionAt(index) >= 0) {
        m_pressedValveIndex = index;
        m_session->pressValveAt(index);
        event->accept();
    }
}

QPointF SchematicItem::scenePointForSheet(const QPointF &sheetMm) const
{
    const Fit fit = placement();
    return QPointF(fit.originX + sheetMm.x() * fit.pixelsPerMm,
                   fit.originY + sheetMm.y() * fit.pixelsPerMm);
}

void SchematicItem::placeCatalogAt(int catalogIndex, const QPointF &position)
{
    if (!m_session || !m_session->editing() || catalogIndex < 0) {
        return;
    }
    const QPointF sheet = toSheet(position);
    m_session->placeComponent(catalogIndex, sheet.x(), sheet.y());
    setFocus(true);
    update();
}

void SchematicItem::mouseMoveEvent(QMouseEvent *event)
{
    if (m_connecting >= 0 && m_session) {
        m_connectCursor = toSheet(event->position());
        update();
        event->accept();
        return;
    }
    if (m_routing >= 0 && m_session) {
        const QPointF sheet = toSheet(event->position());
        m_session->routePipe(m_routing, sheet.x(), sheet.y());
        update();
        event->accept();
        return;
    }
    if (m_dragging < 0 || !m_session) {
        return;
    }
    const QPointF sheet = toSheet(event->position());
    m_moved = true;
    // Live movement while dragging: the document coalesces the moves into one undo entry,
    // so a drag costs one Ctrl+Z, not one per pixel.
    m_session->moveComponent(m_dragging, sheet.x() - m_dragOffset.x(),
                             sheet.y() - m_dragOffset.y());
    update();
    event->accept();
}

void SchematicItem::keyPressEvent(QKeyEvent *event)
{
    if (!m_session) {
        return;
    }
    if (event->key() == Qt::Key_Delete || event->key() == Qt::Key_Backspace) {
        m_session->removeSelected();
        update();
        event->accept();
        return;
    }
    QQuickPaintedItem::keyPressEvent(event);
}

void SchematicItem::dragEnterEvent(QDragEnterEvent *event)
{
    if (event->mimeData()->hasFormat(QStringLiteral("application/x-punchi-catalog"))) {
        event->setDropAction(Qt::CopyAction);
        event->accept();
        return;
    }
    event->ignore();
}

void SchematicItem::dragMoveEvent(QDragMoveEvent *event)
{
    if (event->mimeData()->hasFormat(QStringLiteral("application/x-punchi-catalog"))) {
        event->setDropAction(Qt::CopyAction);
        event->accept();
        return;
    }
    event->ignore();
}

void SchematicItem::dropEvent(QDropEvent *event)
{
    if (!m_session || !m_session->editing()
        || !event->mimeData()->hasFormat(QStringLiteral("application/x-punchi-catalog"))) {
        event->ignore();
        return;
    }
    bool ok = false;
    const int catalogIndex = QString::fromUtf8(
                                 event->mimeData()
                                     ->data(QStringLiteral("application/x-punchi-catalog")))
                                 .toInt(&ok);
    if (!ok) {
        event->ignore();
        return;
    }
    const QPointF sheet = toSheet(event->position());
    m_session->placeComponent(catalogIndex, sheet.x(), sheet.y());
    setFocus(true);
    update();
    event->acceptProposedAction();
}

void SchematicItem::mouseReleaseEvent(QMouseEvent *event)
{
    if (event->button() != Qt::LeftButton) {
        return;
    }
    if (m_connecting >= 0) {
        const QPointF sheet = toSheet(event->position());
        int targetIndex = -1;
        QString targetPort;
        if (m_session
            && m_session->portAt(sheet.x(), sheet.y(), kPortToleranceMm, &targetIndex, &targetPort)) {
            m_session->connectPorts(m_connecting, m_connectPort, targetIndex, targetPort);
        } else if (m_session) {
            // No port under the pointer: a hose released on a hose leaves a T where it
            // landed, which is the gesture the reference documents. A crossing is not a
            // connection, so this is the only reading that makes the drop mean anything.
            const int pipe = m_session->pipeAt(sheet.x(), sheet.y(), kPortToleranceMm);
            if (pipe >= 0) {
                m_session->splicePipeAt(pipe, sheet.x(), sheet.y(), m_connecting,
                                        m_connectPort);
            }
        }
        m_connecting = -1;
        m_connectPort.clear();
        unsetCursor();
        update();
        event->accept();
        return;
    }
    if (m_pressedValveIndex >= 0) {
        if (m_session) {
            m_session->releaseValveAt(m_pressedValveIndex);
        }
        m_pressedValveIndex = -1;
    }
    if (m_routing >= 0) {
        m_routing = -1;
    }
    if (m_dragging >= 0) {
        // A click that did not move selects; a double click opens the properties, which is
        // the gesture the editor uses for "look at this component".
        if (!m_moved && m_session) {
            m_session->inspectComponent(m_dragging);
        }
        m_dragging = -1;
        m_moved = false;
    }
    event->accept();
}

void SchematicItem::hoverMoveEvent(QHoverEvent *event)
{
    // A port is what the editor needs to find; the valve cursor is for simulation.
    bool overPort = false;
    if (m_session && m_session->editing()) {
        const QPointF sheet = toSheet(event->position());
        int index = -1;
        QString port;
        overPort = m_session->portAt(sheet.x(), sheet.y(), kPortToleranceMm, &index, &port);
        if (overPort) {
            setCursor(QCursor(Qt::CrossCursor));
            event->accept();
            return;
        }
    }
    const bool over = onValve(event->position());
    if (over != m_hoveringValve) {
        m_hoveringValve = over;
        // The valve is the only thing on the sheet that answers to a click, so it has to
        // say so before the user tries: the cursors differ between the two modes.
        if (over) {
            setCursor(QCursor(m_session && m_session->editing() ? Qt::SizeAllCursor
                                                                : Qt::OpenHandCursor));
        } else {
            unsetCursor();
        }
        update();
    }
    event->accept();
}

void SchematicItem::hoverLeaveEvent(QHoverEvent *event)
{
    if (m_hoveringValve) {
        m_hoveringValve = false;
        unsetCursor();
        update();
    }
    event->accept();
}
