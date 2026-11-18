// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

// The view mathematics of the sheet: fitting the frame, keeping one step of history and
// the pan that centres a fitted frame. It lives beside `canvasitem.cpp` rather than inside
// it because the canvas is already the largest file of the window and this is a unit of
// its own: none of it touches the document.

#include "canvasitem.h"

#include "replay.h"

#include <QPointF>
#include <QSizeF>

#include <QCursor>
#include <QKeyEvent>

#include <algorithm>

namespace {
/// One arrow-key pan step, in pixels.
constexpr double kPanPerArrow = 24.0;
} // namespace

void SchematicItem::rememberView()
{
    if (!m_session) {
        return;
    }
    // One step, not a history: the view is the one the reader had before this gesture.
    const bool wasFresh = !m_hasPreviousView;
    m_previousZoom = m_session->zoom();
    m_previousPan = m_pan;
    m_hasPreviousView = true;
    if (wasFresh) {
        emit viewHistoryChanged();
    }
}

void SchematicItem::fitToSheet()
{
    if (!m_session) {
        return;
    }
    // The sheet, not the frame inside its margins: the whole paper is what the reader
    // asked to see, edges included. 100 % already means "the paper fitted", so the fit
    // is that view with the pan cleared, and one step back is kept.
    rememberView();
    m_fitted = false;
    m_pan = QPointF();
    m_session->setZoom(1.0);
    update();
}

void SchematicItem::previousView()
{
    if (!m_session || !m_hasPreviousView) {
        return;
    }
    const double zoom = m_previousZoom;
    const QPointF pan = m_previousPan;
    m_hasPreviousView = false;
    m_fitted = false;
    m_session->setZoom(zoom);
    m_pan = pan;
    emit viewHistoryChanged();
    update();
}

Fit SchematicItem::centredPlacement(double zoom) const
{
    const QSizeF sheet = sheetSizeMm();
    const Fit fitted = fitArea(width(), height(), sheet.width(), sheet.height());
    // Zoom keeps the middle of the sheet in the middle of the item, so magnifying does not
    // throw the circuit off screen; the pan is what moves it afterwards.
    const qreal scale = fitted.pixelsPerMm * zoom;
    return Fit { scale, (width() - sheet.width() * scale) * 0.5,
                 (height() - sheet.height() * scale) * 0.5 };
}

QPointF SchematicItem::limitedPan(const Fit &centred) const
{
    // At 1:1 the sheet fits, so it stays centred: there is nothing to reveal and a fitted
    // sheet that could be pushed aside is a sheet the reader can lose. Above 1:1 the sheet
    // may be moved until only a sliver of it remains in view, which is wide enough to keep
    // a zoom anchored at the pointer without ever losing the drawing completely.
    if (!m_session || (m_session->zoom() <= 1.0 && !m_fitted)) {
        return QPointF();
    }
    const QSizeF sheet = sheetSizeMm();
    const qreal limitX = (width() + sheet.width() * centred.pixelsPerMm) * 0.5;
    const qreal limitY = (height() + sheet.height() * centred.pixelsPerMm) * 0.5;
    return QPointF(std::clamp(m_pan.x(), -limitX, limitX),
                   std::clamp(m_pan.y(), -limitY, limitY));
}

void SchematicItem::keyPressEvent(QKeyEvent *event)
{
    if (!m_session) {
        return;
    }
    if (event->key() == Qt::Key_Escape && (m_session->handMode() || m_zoomWindow)) {
        cancelZoomWindow();
        m_session->setHandMode(false);
        event->accept();
        return;
    }
    if (event->key() == Qt::Key_Delete || event->key() == Qt::Key_Backspace) {
        m_session->removeSelected();
        update();
        event->accept();
        return;
    }
    // The keyboard path for panning. It is the only way to reach a corner of a magnified
    // sheet without a pointer, and it stays out of the way because the sheet only sees a
    // key when it holds the focus. Shift takes a longer step.
    const double step = (event->modifiers() & Qt::ShiftModifier) != 0
        ? kPanPerArrow * 4.0
        : kPanPerArrow;
    switch (event->key()) {
    case Qt::Key_Left:
        panBy(QPointF(step, 0.0));
        event->accept();
        return;
    case Qt::Key_Right:
        panBy(QPointF(-step, 0.0));
        event->accept();
        return;
    case Qt::Key_Up:
        panBy(QPointF(0.0, step));
        event->accept();
        return;
    case Qt::Key_Down:
        panBy(QPointF(0.0, -step));
        event->accept();
        return;
    default:
        break;
    }
    QQuickPaintedItem::keyPressEvent(event);
}

void SchematicItem::beginZoomWindow()
{
    m_zoomWindow = true;
    m_zoomDragging = false;
    setCursor(QCursor(Qt::CrossCursor));
    setFocus(true);
    emit zoomWindowChanged();
}

void SchematicItem::cancelZoomWindow()
{
    if (!m_zoomWindow) {
        return;
    }
    m_zoomWindow = false;
    m_zoomDragging = false;
    setCursor(QCursor(m_session != nullptr && m_session->handMode() ? Qt::OpenHandCursor
                                                                    : Qt::ArrowCursor));
    update();
    emit zoomWindowChanged();
}

void SchematicItem::zoomToWindow(const QRectF &rect)
{
    if (!m_session) {
        return;
    }
    const Fit before = placement();
    if (before.pixelsPerMm <= 0.0 || rect.width() < 1.0 || rect.height() < 1.0) {
        return;
    }
    const qreal widthMm = rect.width() / before.pixelsPerMm;
    const qreal heightMm = rect.height() / before.pixelsPerMm;
    const Fit fitted = fitArea(width(), height(), static_cast<float>(widthMm),
                               static_cast<float>(heightMm));
    const qreal base = centredPlacement(1.0).pixelsPerMm;
    if (base <= 0.0) {
        return;
    }
    const QPointF centre((rect.center().x() - before.originX) / before.pixelsPerMm,
                         (rect.center().y() - before.originY) / before.pixelsPerMm);
    rememberView();
    const double zoom = fitted.pixelsPerMm / base;
    m_session->setZoom(zoom);
    const Fit centred = centredPlacement(m_session->zoom());
    m_fitted = true;
    m_pan = QPointF(width() * 0.5 - centre.x() * centred.pixelsPerMm - centred.originX,
                    height() * 0.5 - centre.y() * centred.pixelsPerMm - centred.originY);
    update();
}
