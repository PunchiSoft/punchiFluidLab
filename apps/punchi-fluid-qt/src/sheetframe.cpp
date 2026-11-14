// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#include "sheetframe.h"

#include "session.h"

#include <QColor>
#include <QFontMetricsF>
#include <QPen>
#include <QPolygonF>

#include <algorithm>
#include <cmath>

namespace {
/// Ruling of the sheet's grid, in millimetres.
constexpr double kGridMm = 10.0;

/// The Spanish label of one block field, shared by every model.
QString blockLabel(uint32_t field)
{
    switch (field) {
    case PF_BLOCK_OWNER: return QStringLiteral("Empresa / propietario");
    case PF_BLOCK_PROJECT: return QStringLiteral("Proyecto / instalación");
    case PF_BLOCK_EQUIPMENT: return QStringLiteral("Equipo / sistema");
    case PF_BLOCK_TITLE: return QStringLiteral("Título");
    case PF_BLOCK_NUMBER: return QStringLiteral("Nº de plano");
    case PF_BLOCK_REVISION: return QStringLiteral("Revisión");
    case PF_BLOCK_DATE: return QStringLiteral("Fecha");
    case PF_BLOCK_PREPARED: return QStringLiteral("Dibujado");
    case PF_BLOCK_CHECKED: return QStringLiteral("Revisado");
    case PF_BLOCK_APPROVED: return QStringLiteral("Aprobado");
    case PF_BLOCK_SCALE: return QStringLiteral("Escala");
    case PF_BLOCK_NORMS: return QStringLiteral("Normas aplicadas");
    case PF_BLOCK_FIELD_SHEET: return QStringLiteral("Hoja");
    default: return {};
    }
}

QString blockValue(const FluidSession &session, uint32_t field)
{
    switch (field) {
    case PF_BLOCK_OWNER: return session.blockOwner();
    case PF_BLOCK_PROJECT: return session.blockProject();
    case PF_BLOCK_EQUIPMENT: return session.blockEquipment();
    case PF_BLOCK_TITLE: return session.blockTitle();
    case PF_BLOCK_NUMBER: return session.blockNumber();
    case PF_BLOCK_REVISION: return session.blockRevision();
    case PF_BLOCK_DATE: return session.blockDate();
    case PF_BLOCK_PREPARED: return session.blockPrepared();
    case PF_BLOCK_CHECKED: return session.blockChecked();
    case PF_BLOCK_APPROVED: return session.blockApproved();
    case PF_BLOCK_SCALE: return session.blockScale();
    case PF_BLOCK_NORMS: return session.blockNorms();
    case PF_BLOCK_FIELD_SHEET: return session.blockSheet();
    default: return {};
    }
}
} // namespace

void drawSheetFrame(QPainter *painter, const FluidSession &session, const Fit &fit,
                    qreal sheetWidth, qreal sheetHeight)
{
    const qreal left = session.sheetMarginLeft();
    const qreal top = session.sheetMarginTop();
    const qreal right = sheetWidth - session.sheetMarginRight();
    const qreal bottom = sheetHeight - session.sheetMarginBottom();

    if (session.grid()) {
        // The grid is the drawing's own millimetre ruling, so a component lines up with it
        // instead of floating. It stops at the frame, because the margin is paper, not
        // drawing space. Drawn first and faint: it is a guide, never a mark on the document.
        QColor line = session.tokens().inkMuted;
        line.setAlphaF(0.22F);
        painter->save();
        painter->setPen(QPen(line, 1.0));
        for (double x = left + kGridMm; x < right; x += kGridMm) {
            const qreal px = fit.originX + x * fit.pixelsPerMm;
            painter->drawLine(QPointF(px, fit.originY + top * fit.pixelsPerMm),
                              QPointF(px, fit.originY + bottom * fit.pixelsPerMm));
        }
        for (double y = top + kGridMm; y < bottom; y += kGridMm) {
            const qreal py = fit.originY + y * fit.pixelsPerMm;
            painter->drawLine(QPointF(fit.originX + left * fit.pixelsPerMm, py),
                              QPointF(fit.originX + right * fit.pixelsPerMm, py));
        }
        painter->restore();
    }

    // The edge of the paper, and the ISO 5457 frame inside it. Without them "outside the
    // sheet" is a finding the user cannot see.
    QColor edge = session.tokens().inkMuted;
    edge.setAlphaF(0.55F);
    painter->save();
    painter->setPen(QPen(edge, 1.0));
    painter->setBrush(Qt::NoBrush);
    painter->drawRect(QRectF(fit.originX, fit.originY, sheetWidth * fit.pixelsPerMm,
                             sheetHeight * fit.pixelsPerMm));
    if (left > 0.0 || top > 0.0 || right < sheetWidth || bottom < sheetHeight) {
        QColor frameInk = edge;
        frameInk.setAlphaF(0.8F);
        painter->setPen(QPen(frameInk, std::max<qreal>(1.0, 0.7 * fit.pixelsPerMm)));
        painter->drawRect(QRectF(fit.originX + left * fit.pixelsPerMm,
                                 fit.originY + top * fit.pixelsPerMm,
                                 (right - left) * fit.pixelsPerMm,
                                 (bottom - top) * fit.pixelsPerMm));

        const qreal scale = fit.pixelsPerMm;

        // ISO 5457 / NCh 13.Of93 reference zones, from A3 upwards: the drawing space is
        // divided into fields of at most 50 mm, numerals down the left and right bands and
        // letters across the top and bottom, 3.5 mm characters with 0.35 mm rules. They
        // stay in the margin bands, never in the drawing space. A4 keeps its clean sheet.
        if (std::min(sheetWidth, sheetHeight) >= 297.0 - 0.01) {
            QColor reference = session.tokens().inkMuted;
            reference.setAlphaF(0.85F);
            painter->setPen(QPen(reference, std::max<qreal>(1.0, 0.35 * scale)));
            const qreal fieldLimit = 50.0;
            const int columns =
                std::max(1, static_cast<int>(std::ceil((right - left) / fieldLimit)));
            const int rows = std::max(1, static_cast<int>(std::ceil((bottom - top)
                                                                     / fieldLimit)));
            const qreal stepX = (right - left) / columns;
            const qreal stepY = (bottom - top) / rows;
            QFont font = painter->font();
            font.setPixelSize(std::max(1, qRound(3.5 * scale)));
            painter->setFont(font);

            const auto letter = [](int index) {
                QString name;
                int value = index;
                do {
                    name.prepend(QChar('A' + value % 26));
                    value = value / 26 - 1;
                } while (value >= 0);
                return name;
            };

            for (int column = 1; column < columns; ++column) {
                const qreal x = fit.originX + (left + column * stepX) * scale;
                painter->drawLine(QPointF(x, fit.originY),
                                  QPointF(x, fit.originY + top * scale));
                painter->drawLine(QPointF(x, fit.originY + bottom * scale),
                                  QPointF(x, fit.originY + sheetHeight * scale));
            }
            for (int row = 1; row < rows; ++row) {
                const qreal y = fit.originY + (top + row * stepY) * scale;
                painter->drawLine(QPointF(fit.originX, y),
                                  QPointF(fit.originX + left * scale, y));
                painter->drawLine(QPointF(fit.originX + right * scale, y),
                                  QPointF(fit.originX + sheetWidth * scale, y));
            }

            const auto centred = [&](const QRectF &area, const QString &label) {
                painter->drawText(area, Qt::AlignCenter, label);
            };
            for (int row = 0; row < rows; ++row) {
                const QString number = QString::number(row + 1);
                const qreal y = fit.originY + (top + row * stepY) * scale;
                centred(QRectF(fit.originX, y, left * scale, stepY * scale), number);
                centred(QRectF(fit.originX + right * scale, y,
                               (sheetWidth - right) * scale, stepY * scale),
                        number);
            }
            for (int column = 0; column < columns; ++column) {
                const QString name = letter(column);
                const qreal x = fit.originX + (left + column * stepX) * scale;
                centred(QRectF(x, fit.originY, stepX * scale, top * scale), name);
                centred(QRectF(x, fit.originY + bottom * scale, stepX * scale,
                               (sheetHeight - bottom) * scale),
                        name);
            }
        }

        // ISO 5457 centring marks: a line from the frame across the border to the paper
        // edge, with a small arrow. The form is free; the line is the recommended part and
        // spanning the whole border keeps the left filing margin and the other three sides
        // looking the same instead of leaving a gap on the left.
        const qreal paperLeft = fit.originX;
        const qreal paperTop = fit.originY;
        const qreal paperRight = paperLeft + sheetWidth * scale;
        const qreal paperBottom = paperTop + sheetHeight * scale;
        const qreal frameLeft = paperLeft + left * scale;
        const qreal frameTop = paperTop + top * scale;
        const qreal frameRight = paperLeft + right * scale;
        const qreal frameBottom = paperTop + bottom * scale;
        const qreal middleX = (paperLeft + paperRight) * 0.5;
        const qreal middleY = (paperTop + paperBottom) * 0.5;
        const qreal arrow = 3.0 * scale;
        const qreal half = 1.5 * scale;
        painter->setBrush(frameInk);
        const auto triangle = [painter](QPointF tip, QPointF leftBase, QPointF rightBase) {
            painter->drawPolygon(QPolygonF { tip, leftBase, rightBase });
        };
        if (frameTop > paperTop + 0.5) {
            painter->drawLine(QPointF(middleX, frameTop), QPointF(middleX, paperTop));
            triangle(QPointF(middleX, std::min(paperTop + arrow, frameTop)),
                     QPointF(middleX - half, paperTop),
                     QPointF(middleX + half, paperTop));
        }
        if (frameBottom < paperBottom - 0.5) {
            painter->drawLine(QPointF(middleX, frameBottom), QPointF(middleX, paperBottom));
            triangle(QPointF(middleX, std::max(paperBottom - arrow, frameBottom)),
                     QPointF(middleX - half, paperBottom),
                     QPointF(middleX + half, paperBottom));
        }
        if (frameLeft > paperLeft + 0.5) {
            painter->drawLine(QPointF(frameLeft, middleY), QPointF(paperLeft, middleY));
            triangle(QPointF(std::min(paperLeft + arrow, frameLeft), middleY),
                     QPointF(paperLeft, middleY - half),
                     QPointF(paperLeft, middleY + half));
        }
        if (frameRight < paperRight - 0.5) {
            painter->drawLine(QPointF(frameRight, middleY), QPointF(paperRight, middleY));
            triangle(QPointF(std::max(paperRight - arrow, frameRight), middleY),
                     QPointF(paperRight, middleY - half),
                     QPointF(paperRight, middleY + half));
        }

        // The control-diagram block, from the rows the ABI resolved with the same layout the
        // export prints. The industrial block is 170 × 45 mm on A4 and 180 × 65 mm on A3;
        // the school and technical models are shorter, and a hidden field collapses its
        // cell. Text keeps the ISO 3098 heights (5 / 3.5 / 2.5 mm).
        const int preset = session.blockPreset();
        if (preset == 3) {
            painter->restore();
            return;
        }
        PfBlockLayout block {};
        if (pf_session_block_layout(session.handle(), &block) != PF_OK || block.rows_len == 0
            || block.width_mm <= 0.0F || block.height_mm <= 0.0F) {
            painter->restore();
            return;
        }
        const qreal blockWidth = preset == 2
                                     ? right - left
                                     : std::min<qreal>(block.width_mm, right - left);
        const qreal blockHeight = std::min<qreal>(block.height_mm, bottom - top);
        const qreal blockX = preset == 2 ? left : right - blockWidth;
        const qreal blockY = preset == 0 ? top : bottom - blockHeight;
        const qreal sx = blockWidth / block.width_mm;
        const qreal sy = blockHeight / block.height_mm;
        const auto mx = [&](qreal mm) { return fit.originX + mm * scale; };
        const auto my = [&](qreal mm) { return fit.originY + mm * scale; };

        // Paper colour under the block, so it reads as a band of information and not as a
        // frame over the drawing. The live layout already keeps the circuit clear of it.
        painter->fillRect(QRectF(mx(blockX), my(blockY), blockWidth * scale,
                                 blockHeight * scale),
                          session.background());
        painter->setPen(QPen(frameInk, std::max<qreal>(1.0, 0.7 * scale)));
        painter->setBrush(Qt::NoBrush);
        painter->drawRect(QRectF(mx(blockX), my(blockY), blockWidth * scale,
                                 blockHeight * scale));
        painter->setPen(QPen(frameInk, std::max<qreal>(1.0, 0.35 * scale)));

        const auto cell = [&](qreal x, qreal y, qreal width, qreal height, uint32_t field) {
            const qreal pad = 1.6 * scale;
            const qreal topPad = 0.6 * scale;
            const qreal bottomPad = 0.4 * scale;
            const qreal labelSize = 2.5 * scale;
            const QString value = blockValue(session, field);

            QFont font = painter->font();
            font.setPixelSize(std::max(1, qRound(labelSize)));
            painter->setFont(font);
            painter->setPen(QPen(session.tokens().inkMuted, 1.0));
            painter->drawText(
                QRectF(mx(x) + pad, my(y) + topPad, width * scale - 2.0 * pad,
                       labelSize * 1.15),
                Qt::AlignLeft | Qt::AlignTop, blockLabel(field));
            if (value.isEmpty()) {
                return;
            }
            const qreal valueTop = topPad + labelSize + 0.4 * scale;
            const qreal preferred = (field == PF_BLOCK_OWNER || field == PF_BLOCK_TITLE)
                                        ? 5.0 * scale
                                        : 3.5 * scale;
            const qreal available = height * scale - bottomPad - valueTop;
            const qreal size = std::min(preferred, available);
            if (size < 2.0 * scale) {
                return;
            }
            font.setPixelSize(std::max(1, qRound(size)));
            painter->setFont(font);
            painter->setPen(QPen(session.tokens().ink, 1.0));
            const QFontMetricsF metrics(font);
            const QString shown =
                metrics.elidedText(value, Qt::ElideRight, width * scale - 2.0 * pad);
            painter->drawText(QRectF(mx(x) + pad, my(y) + valueTop,
                                     width * scale - 2.0 * pad, size * 1.15),
                              Qt::AlignLeft | Qt::AlignTop, shown);
        };

        qreal y = blockY;
        for (uint32_t rowIndex = 0; rowIndex < block.rows_len; ++rowIndex) {
            const PfBlockRow &row = block.rows[rowIndex];
            const qreal height = row.height_mm * sy;
            if (rowIndex > 0) {
                painter->drawLine(QPointF(mx(blockX), my(y)),
                                  QPointF(mx(blockX + blockWidth), my(y)));
            }
            qreal x = blockX;
            for (uint32_t cellIndex = 0; cellIndex < row.cells_len; ++cellIndex) {
                const uint32_t at = row.cells_at + cellIndex;
                if (at >= block.cells_len) {
                    break;
                }
                const PfBlockCell &spec = block.cells[at];
                const qreal width = spec.width_mm * sx;
                if (cellIndex > 0) {
                    painter->drawLine(QPointF(mx(x), my(y)),
                                      QPointF(mx(x), my(y + height)));
                }
                cell(x, y, width, height, spec.field);
                x += width;
            }
            y += height;
        }
    }
    painter->restore();
}
