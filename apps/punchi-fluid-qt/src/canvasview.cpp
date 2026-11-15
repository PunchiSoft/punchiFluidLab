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

#include <algorithm>

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

void SchematicItem::fitToFrame()
{
    if (!m_session) {
        return;
    }
    const QSizeF sheet = sheetSizeMm();
    const qreal left = m_session->sheetMarginLeft();
    const qreal top = m_session->sheetMarginTop();
    const qreal right = sheet.width() - m_session->sheetMarginRight();
    const qreal bottom = sheet.height() - m_session->sheetMarginBottom();
    const qreal frameWidth = right - left;
    const qreal frameHeight = bottom - top;
    if (frameWidth <= 0.0 || frameHeight <= 0.0) {
        return;
    }
    const Fit fitted = fitArea(width(), height(), frameWidth, frameHeight);
    const qreal base = centredPlacement(1.0).pixelsPerMm;
    if (base <= 0.0) {
        return;
    }
    rememberView();
    m_session->setZoom(fitted.pixelsPerMm / base);
    // `centredPlacement` centres the paper; the frame sits off its centre by the margins,
    // so the pan is the distance between the two centres at the fitted scale.
    const qreal scale = fitted.pixelsPerMm;
    const QPointF frameCentre((left + right) * 0.5 - sheet.width() * 0.5,
                              (top + bottom) * 0.5 - sheet.height() * 0.5);
    m_fitted = true;
    m_pan = QPointF(-frameCentre.x() * scale, -frameCentre.y() * scale);
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

