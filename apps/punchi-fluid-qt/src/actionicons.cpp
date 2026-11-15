// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#include "actionicons.h"

#include <QPainter>
#include <QPainterPath>
#include <QPen>
#include <QPointF>

#include <algorithm>

namespace actionicons {
namespace {

constexpr qreal kBox = 24.0;
constexpr qreal kStroke = 2.0;

const QStringList &allNames()
{
    static const QStringList names {
        QStringLiteral("document-new"),
        QStringLiteral("document-open"),
        QStringLiteral("document-save"),
        QStringLiteral("document-save-as"),
        QStringLiteral("document-export"),
        QStringLiteral("edit-undo"),
        QStringLiteral("edit-redo"),
        QStringLiteral("media-playback-start"),
    };
    return names;
}

/// A page with the top-right corner folded, the way a document is drawn in a line set.
void page(QPainter &painter, qreal left, qreal top, qreal right, qreal bottom, qreal fold)
{
    QPainterPath path;
    path.moveTo(left, top);
    path.lineTo(right - fold, top);
    path.lineTo(right, top + fold);
    path.lineTo(right, bottom);
    path.lineTo(left, bottom);
    path.closeSubpath();
    painter.drawPath(path);
    painter.drawLine(QPointF(right - fold, top), QPointF(right - fold, top + fold));
    painter.drawLine(QPointF(right - fold, top + fold), QPointF(right, top + fold));
}

void plus(QPainter &painter, qreal x, qreal y, qreal arm)
{
    painter.drawLine(QPointF(x - arm, y), QPointF(x + arm, y));
    painter.drawLine(QPointF(x, y - arm), QPointF(x, y + arm));
}

void arrowDown(QPainter &painter, qreal x, qreal top, qreal bottom)
{
    painter.drawLine(QPointF(x, top), QPointF(x, bottom));
    painter.drawLine(QPointF(x - 2.6, bottom - 2.6), QPointF(x, bottom));
    painter.drawLine(QPointF(x + 2.6, bottom - 2.6), QPointF(x, bottom));
}

void arrowOut(QPainter &painter, qreal fromX, qreal fromY, qreal toX, qreal toY)
{
    painter.drawLine(QPointF(fromX, fromY), QPointF(toX, toY));
    painter.drawLine(QPointF(toX - 4.2, toY), QPointF(toX, toY));
    painter.drawLine(QPointF(toX, toY + 4.2), QPointF(toX, toY));
}

void undoArrow(QPainter &painter)
{
    QPainterPath path;
    path.moveTo(17.0, 17.0);
    path.cubicTo(17.0, 10.0, 13.0, 7.0, 7.0, 8.0);
    painter.drawPath(path);
    painter.drawLine(QPointF(10.0, 5.4), QPointF(7.0, 8.0));
    painter.drawLine(QPointF(10.0, 10.6), QPointF(7.0, 8.0));
}

void redoArrow(QPainter &painter)
{
    QPainterPath path;
    path.moveTo(7.0, 17.0);
    path.cubicTo(7.0, 10.0, 11.0, 7.0, 17.0, 8.0);
    painter.drawPath(path);
    painter.drawLine(QPointF(14.0, 5.4), QPointF(17.0, 8.0));
    painter.drawLine(QPointF(14.0, 10.6), QPointF(17.0, 8.0));
}

void play(QPainter &painter, const QColor &ink)
{
    QPainterPath triangle;
    triangle.moveTo(8.0, 6.0);
    triangle.lineTo(19.0, 12.0);
    triangle.lineTo(8.0, 18.0);
    triangle.closeSubpath();
    painter.setPen(Qt::NoPen);
    painter.setBrush(ink);
    painter.fillPath(triangle, ink);
}

void draw(const QString &name, QPainter &painter, const QColor &ink)
{
    if (name == QStringLiteral("document-new")) {
        page(painter, 4.0, 3.0, 16.0, 20.0, 3.0);
        plus(painter, 19.0, 19.0, 3.0);
    } else if (name == QStringLiteral("document-open")) {
        QPainterPath folder;
        folder.moveTo(3.0, 8.0);
        folder.lineTo(9.0, 8.0);
        folder.lineTo(11.0, 10.0);
        folder.lineTo(21.0, 10.0);
        folder.lineTo(21.0, 20.0);
        folder.lineTo(3.0, 20.0);
        folder.closeSubpath();
        painter.drawPath(folder);
    } else if (name == QStringLiteral("document-save")) {
        page(painter, 4.0, 3.0, 20.0, 21.0, 4.0);
        arrowDown(painter, 12.0, 9.0, 17.0);
    } else if (name == QStringLiteral("document-save-as")) {
        page(painter, 4.0, 3.0, 16.0, 20.0, 3.0);
        arrowDown(painter, 10.0, 8.0, 15.0);
        plus(painter, 19.0, 19.0, 3.0);
    } else if (name == QStringLiteral("document-export")) {
        page(painter, 4.0, 3.0, 16.0, 20.0, 3.0);
        arrowOut(painter, 11.0, 15.0, 20.0, 7.0);
    } else if (name == QStringLiteral("edit-undo")) {
        undoArrow(painter);
    } else if (name == QStringLiteral("edit-redo")) {
        redoArrow(painter);
    } else if (name == QStringLiteral("media-playback-start")) {
        play(painter, ink);
    }
}

} // namespace

bool has(const QString &name)
{
    return allNames().contains(name);
}

QStringList names()
{
    return allNames();
}

QPixmap render(const QString &name, int pixels, qreal devicePixelRatio, const QColor &ink)
{
    if (!has(name) || pixels <= 0) {
        return {};
    }
    const qreal ratio = devicePixelRatio > 0.0 ? devicePixelRatio : 1.0;
    const int physical = std::max(1, qRound(pixels * ratio));
    QPixmap pixmap(physical, physical);
    pixmap.fill(Qt::transparent);
    pixmap.setDevicePixelRatio(ratio);
    QPainter painter(&pixmap);
    painter.setRenderHint(QPainter::Antialiasing, true);
    painter.scale(physical / kBox, physical / kBox);
    painter.setPen(QPen(ink, kStroke, Qt::SolidLine, Qt::RoundCap, Qt::RoundJoin));
    painter.setBrush(Qt::NoBrush);
    draw(name, painter, ink);
    painter.end();
    return pixmap;
}

} // namespace actionicons
