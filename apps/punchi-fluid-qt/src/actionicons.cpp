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
        QStringLiteral("document-properties"),
        QStringLiteral("x-office-document"),
        QStringLiteral("list-add"),
        QStringLiteral("edit-rename"),
        QStringLiteral("object-rotate-right"),
        QStringLiteral("document-open-recent"),
        QStringLiteral("dialog-ok-apply"),
        QStringLiteral("edit-copy"),
        QStringLiteral("edit-paste"),
        QStringLiteral("edit-duplicate"),
        QStringLiteral("object-flip-horizontal"),
        QStringLiteral("edit-select-all"),
        QStringLiteral("edit-delete"),
        QStringLiteral("zoom-in"),
        QStringLiteral("zoom-out"),
        QStringLiteral("zoom-original"),
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

void documentLines(QPainter &painter, qreal left, qreal right, qreal top, qreal gap)
{
    for (int line = 0; line < 3; ++line) {
        painter.drawLine(QPointF(left, top + line * gap), QPointF(right, top + line * gap));
    }
}

void magnifier(QPainter &painter, qreal centerX, qreal centerY, qreal radius)
{
    painter.drawEllipse(QPointF(centerX, centerY), radius, radius);
    const qreal offset = radius * 0.72;
    painter.drawLine(QPointF(centerX + offset, centerY + offset), QPointF(20.5, 20.5));
}

void pencil(QPainter &painter)
{
    QPolygonF body({QPointF(11.0, 19.0), QPointF(13.0, 18.2), QPointF(21.0, 10.2),
                    QPointF(19.0, 8.2)});
    painter.drawPolygon(body);
    painter.drawLine(QPointF(18.0, 9.2), QPointF(20.0, 11.2));
}

void clipboard(QPainter &painter)
{
    painter.drawRoundedRect(QRectF(5.0, 6.0, 14.0, 15.0), 2.0, 2.0);
    painter.drawRoundedRect(QRectF(9.0, 3.5, 6.0, 4.0), 1.5, 1.5);
    painter.drawLine(QPointF(8.0, 11.0), QPointF(16.0, 11.0));
    painter.drawLine(QPointF(8.0, 14.0), QPointF(16.0, 14.0));
}

void trash(QPainter &painter)
{
    painter.drawLine(QPointF(6.0, 7.0), QPointF(18.0, 7.0));
    painter.drawLine(QPointF(9.0, 4.0), QPointF(15.0, 4.0));
    painter.drawLine(QPointF(10.0, 4.0), QPointF(10.0, 7.0));
    painter.drawLine(QPointF(14.0, 4.0), QPointF(14.0, 7.0));
    QPainterPath body;
    body.moveTo(7.0, 7.0);
    body.lineTo(8.5, 21.0);
    body.lineTo(15.5, 21.0);
    body.lineTo(17.0, 7.0);
    painter.drawPath(body);
    painter.drawLine(QPointF(10.5, 10.0), QPointF(10.5, 18.0));
    painter.drawLine(QPointF(13.5, 10.0), QPointF(13.5, 18.0));
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
    } else if (name == QStringLiteral("document-properties")) {
        page(painter, 4.0, 3.0, 20.0, 21.0, 4.0);
        painter.drawLine(QPointF(8.0, 10.0), QPointF(16.0, 10.0));
        painter.drawEllipse(QPointF(10.0, 10.0), 1.6, 1.6);
        painter.drawLine(QPointF(8.0, 14.0), QPointF(16.0, 14.0));
        painter.drawEllipse(QPointF(14.0, 14.0), 1.6, 1.6);
    } else if (name == QStringLiteral("x-office-document")) {
        page(painter, 5.0, 3.0, 19.0, 21.0, 4.0);
        documentLines(painter, 8.0, 16.0, 10.0, 3.0);
    } else if (name == QStringLiteral("list-add")) {
        documentLines(painter, 4.0, 15.0, 7.0, 4.0);
        plus(painter, 18.5, 18.5, 2.5);
    } else if (name == QStringLiteral("edit-rename")) {
        page(painter, 4.0, 3.0, 14.0, 20.0, 3.0);
        pencil(painter);
    } else if (name == QStringLiteral("object-rotate-right")) {
        const QRectF circle(6.0, 6.0, 12.0, 12.0);
        painter.drawArc(circle, 40 * 16, 280 * 16);
        painter.drawLine(QPointF(13.2, 16.4), QPointF(16.6, 15.9));
        painter.drawLine(QPointF(16.6, 15.9), QPointF(16.6, 12.5));
    } else if (name == QStringLiteral("document-open-recent")) {
        painter.drawEllipse(QPointF(12.0, 12.0), 8.0, 8.0);
        painter.drawLine(QPointF(12.0, 7.0), QPointF(12.0, 12.0));
        painter.drawLine(QPointF(12.0, 12.0), QPointF(16.0, 12.0));
    } else if (name == QStringLiteral("dialog-ok-apply")) {
        painter.drawLine(QPointF(5.0, 12.0), QPointF(10.0, 17.0));
        painter.drawLine(QPointF(10.0, 17.0), QPointF(19.0, 7.0));
    } else if (name == QStringLiteral("edit-copy")) {
        page(painter, 4.0, 6.0, 15.0, 20.0, 3.0);
        page(painter, 8.0, 3.0, 19.0, 17.0, 3.0);
    } else if (name == QStringLiteral("edit-paste")) {
        clipboard(painter);
    } else if (name == QStringLiteral("edit-duplicate")) {
        page(painter, 3.0, 6.0, 14.0, 20.0, 3.0);
        page(painter, 7.0, 3.0, 18.0, 17.0, 3.0);
        plus(painter, 20.0, 20.0, 2.5);
    } else if (name == QStringLiteral("object-flip-horizontal")) {
        QPainterPath left;
        left.moveTo(4.0, 7.0);
        left.lineTo(4.0, 17.0);
        left.lineTo(10.0, 12.0);
        left.closeSubpath();
        painter.drawPath(left);
        QPainterPath right;
        right.moveTo(20.0, 7.0);
        right.lineTo(20.0, 17.0);
        right.lineTo(14.0, 12.0);
        right.closeSubpath();
        painter.drawPath(right);
        painter.drawLine(QPointF(12.0, 4.0), QPointF(12.0, 20.0));
    } else if (name == QStringLiteral("edit-select-all")) {
        QPen dashed = painter.pen();
        dashed.setStyle(Qt::DashLine);
        painter.setPen(dashed);
        painter.drawRect(QRectF(4.0, 4.0, 16.0, 16.0));
    } else if (name == QStringLiteral("edit-delete")) {
        trash(painter);
    } else if (name == QStringLiteral("zoom-in")) {
        magnifier(painter, 10.0, 10.0, 6.0);
        plus(painter, 10.0, 10.0, 2.5);
    } else if (name == QStringLiteral("zoom-out")) {
        magnifier(painter, 10.0, 10.0, 6.0);
        painter.drawLine(QPointF(7.5, 10.0), QPointF(12.5, 10.0));
    } else if (name == QStringLiteral("zoom-original")) {
        magnifier(painter, 10.0, 10.0, 6.0);
        painter.setBrush(ink);
        painter.drawEllipse(QPointF(10.0, 10.0), 1.4, 1.4);
        painter.setBrush(Qt::NoBrush);
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
