// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#include "sheetframe.h"

#include "session.h"

#include <QColor>
#include <QFontMetricsF>
#include <QPen>

#include <algorithm>
#include <cmath>

namespace {
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

/// Cap height of a reference-zone character, in millimetres (ISO 5457 / NCh 15).
constexpr qreal kZoneCharacterMm = 3.5;
/// Distance a label centre keeps from a centring mark, in millimetres.
constexpr qreal kZoneClearanceMm = 1.0;
/// A band narrower than the character plus its clearance cannot hold a label.
constexpr qreal kZoneBandMm = kZoneCharacterMm + kZoneClearanceMm;

/// Places a label centre clear of a centring mark, or returns `false` when it cannot move.
///
/// A centring mark crosses the middle of its band, so only the coordinate along the band
/// gives way. The label keeps half its cap height plus the clearance from the line and
/// never travels further than half a field, which would make it read as its neighbour's.
bool clearOfMark(qreal centre, qreal mark, qreal field, qreal *placed)
{
    if (!std::isfinite(centre) || !std::isfinite(mark) || !std::isfinite(field)
        || field <= 0.0) {
        return false;
    }
    const qreal clearance = kZoneCharacterMm * 0.5 + kZoneClearanceMm;
    if (std::abs(centre - mark) >= clearance) {
        *placed = centre;
        return true;
    }
    const qreal wanted = centre >= mark ? mark + clearance : mark - clearance;
    if (std::abs(wanted - centre) <= field * 0.5
        && std::abs(wanted - mark) >= kZoneClearanceMm) {
        *placed = wanted;
        return true;
    }
    return false;
}
} // namespace

ZoneBands sheetZoneBands(const QString &paperName, qreal sheetWidthMm, qreal sheetHeightMm)
{
    if (paperName == QLatin1String("A4")) {
        return ZoneBands::A4;
    }
    if (std::min(sheetWidthMm, sheetHeightMm) >= 297.0 - 0.01) {
        return ZoneBands::FourSides;
    }
    return ZoneBands::None;
}

int sheetFieldCount(qreal lengthMm)
{
    if (!std::isfinite(lengthMm) || lengthMm <= 0.0) {
        return 1;
    }
    return std::max(1, static_cast<int>(std::ceil(lengthMm / 50.0)));
}

QString sheetRowLabel(int index)
{
    // The 24 field letters of ISO 5457 §4.4. A custom sheet can reach 2000 mm, which is 40
    // fields, so the sequence continues with AA, AB, … over the same letters; no label ever
    // contains a forbidden I or O.
    static const char letters[] = "ABCDEFGHJKLMNPQRSTUVWXYZ";
    constexpr int base = 24;
    QString label;
    int value = std::max(index, 0);
    do {
        label.prepend(QChar(letters[value % base]));
        value = value / base - 1;
    } while (value >= 0);
    return label;
}

QString sheetColumnLabel(int index)
{
    return QString::number(std::max(index, 0) + 1);
}

void drawSheetFrame(QPainter *painter, const FluidSession &session, const Fit &fit,
                    qreal sheetWidth, qreal sheetHeight)
{
    const qreal left = session.sheetMarginLeft();
    const qreal top = session.sheetMarginTop();
    const qreal right = sheetWidth - session.sheetMarginRight();
    const qreal bottom = sheetHeight - session.sheetMarginBottom();

    if (session.grid()) {
        // The grid is the drawing's own millimetre ruling, so a component lines up with it
        // instead of floating. The lines are at multiples of the step from the paper's
        // top-left corner, the same origin the snap uses; starting at the frame edge would
        // offset them by a margin that is not a multiple of the step. It stops at the
        // frame, because the margin is paper, not drawing space. Drawn first and faint: it
        // is a guide, never a mark on the document.
        const double step = session.gridStepMm();
        const double major = step * 5.0;
        const QColor minor = session.tokens().gridMinor;
        const QColor majorInk = session.tokens().gridMajor;

        // Only the lines the viewport can show, and only when a line is at least six
        // pixels from the next: a 5 mm ruling on A0 or at a distant zoom would otherwise
        // ask QPainter for tens of thousands of invisible lines on every frame.
        const QRectF visible = painter->clipBoundingRect();
        const qreal scale = std::max<qreal>(fit.pixelsPerMm, 1.0e-9);
        double xFrom = left;
        double xTo = right;
        double yFrom = top;
        double yTo = bottom;
        if (visible.isValid() && visible.width() > 0.0 && visible.height() > 0.0) {
            xFrom = std::max(left, (visible.left() - fit.originX) / scale);
            xTo = std::min(right, (visible.right() - fit.originX) / scale);
            yFrom = std::max(top, (visible.top() - fit.originY) / scale);
            yTo = std::min(bottom, (visible.bottom() - fit.originY) / scale);
            if (xTo < xFrom || yTo < yFrom) {
                xFrom = left;
                xTo = right;
                yFrom = top;
                yTo = bottom;
            }
        }

        const auto first_line = [](double from, double spacing) {
            return std::ceil(from / spacing) * spacing;
        };

        painter->save();
        if (step * fit.pixelsPerMm >= 6.0) {
            painter->setPen(QPen(minor, 1.0));
            for (double x = first_line(xFrom, step); x <= xTo; x += step) {
                const qreal px = fit.originX + x * fit.pixelsPerMm;
                painter->drawLine(QPointF(px, fit.originY + top * fit.pixelsPerMm),
                                  QPointF(px, fit.originY + bottom * fit.pixelsPerMm));
            }
            for (double y = first_line(yFrom, step); y <= yTo; y += step) {
                const qreal py = fit.originY + y * fit.pixelsPerMm;
                painter->drawLine(QPointF(fit.originX + left * fit.pixelsPerMm, py),
                                  QPointF(fit.originX + right * fit.pixelsPerMm, py));
            }
        }
        if (major * fit.pixelsPerMm >= 6.0) {
            painter->setPen(QPen(majorInk, 1.0));
            for (double x = first_line(xFrom, major); x <= xTo; x += major) {
                const qreal px = fit.originX + x * fit.pixelsPerMm;
                painter->drawLine(QPointF(px, fit.originY + top * fit.pixelsPerMm),
                                  QPointF(px, fit.originY + bottom * fit.pixelsPerMm));
            }
            for (double y = first_line(yFrom, major); y <= yTo; y += major) {
                const qreal py = fit.originY + y * fit.pixelsPerMm;
                painter->drawLine(QPointF(fit.originX + left * fit.pixelsPerMm, py),
                                  QPointF(fit.originX + right * fit.pixelsPerMm, py));
            }
        }
        painter->restore();
    }

    // The edge of the paper, and the ISO 5457 frame inside it. Without them "outside the
    // sheet" is a finding the user cannot see.
    // One edge only: the paper card already has its shadow and fill, so the outer line is
    // the single boundary and uses the measured line token. The ISO frame is drawn inside
    // the margins and stays distinct.
    QColor edge = session.tokens().lineStrong;
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

        // ISO 5457 / NCh 13.Of93 reference zones. A3 and larger carry numerals left to
        // right along the top and bottom bands and letters top to bottom along the left and
        // right bands; A4 carries the standard's two-sided form, numerals on the top band
        // only and letters on the right band only. 3.5 mm characters with 0.35 mm rules,
        // all inside the margin bands and never in the drawing space. A5 and the non-ISO
        // sheets keep a clean sheet.
        const ZoneBands bands =
            sheetZoneBands(session.sheetPaperName(), sheetWidth, sheetHeight);
        const qreal narrowest =
            std::min({left, top, sheetWidth - right, sheetHeight - bottom});
        if (bands != ZoneBands::None && narrowest >= kZoneBandMm) {
            QColor reference = session.tokens().inkMuted;
            reference.setAlphaF(0.85F);
            painter->setPen(QPen(reference, std::max<qreal>(1.0, 0.35 * scale)));
            const int columns = sheetFieldCount(right - left);
            const int rows = sheetFieldCount(bottom - top);
            const qreal stepX = (right - left) / columns;
            const qreal stepY = (bottom - top) / rows;
            const bool four = bands == ZoneBands::FourSides;
            const qreal markX = sheetWidth * 0.5;
            const qreal markY = sheetHeight * 0.5;
            QFont font = painter->font();
            font.setPixelSize(std::max(1, qRound(3.5 * scale)));
            painter->setFont(font);

            for (int column = 1; column < columns; ++column) {
                const qreal x = fit.originX + (left + column * stepX) * scale;
                painter->drawLine(QPointF(x, fit.originY),
                                  QPointF(x, fit.originY + top * scale));
                if (four) {
                    painter->drawLine(QPointF(x, fit.originY + bottom * scale),
                                      QPointF(x, fit.originY + sheetHeight * scale));
                }
            }
            for (int row = 1; row < rows; ++row) {
                const qreal y = fit.originY + (top + row * stepY) * scale;
                if (four) {
                    painter->drawLine(QPointF(fit.originX, y),
                                      QPointF(fit.originX + left * scale, y));
                }
                painter->drawLine(QPointF(fit.originX + right * scale, y),
                                  QPointF(fit.originX + sheetWidth * scale, y));
            }

            const auto centred = [&](qreal centreX, qreal centreY, qreal width, qreal height,
                                     const QString &label) {
                painter->drawText(
                    QRectF(centreX - width * 0.5, centreY - height * 0.5, width, height),
                    Qt::AlignCenter, label);
            };
            for (int column = 0; column < columns; ++column) {
                const qreal centre = left + (column + 0.5) * stepX;
                qreal x = centre;
                if (!clearOfMark(centre, markX, stepX, &x)) {
                    continue;
                }
                const QString number = sheetColumnLabel(column);
                centred(fit.originX + x * scale, fit.originY + top * 0.5 * scale,
                        stepX * scale, top * scale, number);
                if (four) {
                    centred(fit.originX + x * scale,
                            fit.originY
                                + (bottom + (sheetHeight - bottom) * 0.5) * scale,
                            stepX * scale, (sheetHeight - bottom) * scale, number);
                }
            }
            for (int row = 0; row < rows; ++row) {
                const qreal centre = top + (row + 0.5) * stepY;
                qreal y = centre;
                if (!clearOfMark(centre, markY, stepY, &y)) {
                    continue;
                }
                const QString name = sheetRowLabel(row);
                if (four) {
                    centred(fit.originX + left * 0.5 * scale, fit.originY + y * scale,
                            left * scale, stepY * scale, name);
                }
                centred(fit.originX + (right + (sheetWidth - right) * 0.5) * scale,
                        fit.originY + y * scale, (sheetWidth - right) * scale,
                        stepY * scale, name);
            }
        }

        // ISO 5457 §4.3: four continuous lines of 0.7 mm at the ends of the symmetry
        // axes, starting at the frame and extending 10 mm beyond it. No arrowhead: the
        // standard fixes the form, and the filing edge keeps its line 10 mm inside the
        // paper instead of growing a longer mark than the other three.
        const qreal extent = 10.0 * scale;
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
        const auto mark = [painter](QPointF from, QPointF to) {
            if (std::hypot(to.x() - from.x(), to.y() - from.y()) >= 0.5) {
                painter->drawLine(from, to);
            }
        };
        painter->setBrush(Qt::NoBrush);
        painter->setPen(QPen(frameInk, std::max<qreal>(1.0, 0.7 * scale)));
        if (frameTop > paperTop + 0.5) {
            mark(QPointF(middleX, frameTop),
                 QPointF(middleX, std::max(paperTop, frameTop - extent)));
        }
        if (frameBottom < paperBottom - 0.5) {
            mark(QPointF(middleX, frameBottom),
                 QPointF(middleX, std::min(paperBottom, frameBottom + extent)));
        }
        if (frameLeft > paperLeft + 0.5) {
            mark(QPointF(frameLeft, middleY),
                 QPointF(std::max(paperLeft, frameLeft - extent), middleY));
        }
        if (frameRight < paperRight - 0.5) {
            mark(QPointF(frameRight, middleY),
                 QPointF(std::min(paperRight, frameRight + extent), middleY));
        }

        // The control-diagram block, from the rows the ABI resolved with the same layout the
        // export prints. The industrial block is 170 × 45 mm on A4 and 180 × 65 mm on A3;
        // the school and technical models are shorter, and a hidden field collapses its
        // cell. Text keeps the ISO 3098 heights (5 / 3.5 / 2.5 mm).
        const int preset = session.blockPreset();
        // Hidden by the view strip, or by the document's own "Sin cajetín": either way the
        // block is not drawn, and the band it reserves stays reserved.
        if (preset == 3 || !session.blockVisible()) {
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
