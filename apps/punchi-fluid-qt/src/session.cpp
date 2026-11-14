// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#include "session.h"

#include <QDate>
#include <QDir>
#include <QFileInfo>
#include <QStandardPaths>
#include <QUrl>

#include <QEvent>

#include <algorithm>
#include <cmath>
#include <QGuiApplication>
#include <QVariantMap>

#include <string>
#include <vector>

namespace {

PfRgb toRgb(const QColor &colour)
{
    return PfRgb { static_cast<uint8_t>(colour.red()), static_cast<uint8_t>(colour.green()),
                   static_cast<uint8_t>(colour.blue()) };
}

} // namespace

FluidSession::FluidSession(QObject *parent)
    : QObject(parent)
{
    m_session = pf_session_new(nullptr);
    if (!m_session) {
        m_error = QStringLiteral("the core refused to create a session");
        return;
    }
    m_valveSpringReturn = pf_session_valve_spring_return(m_session) == 1;
    // The core does not choose a directory: the front end knows where the desktop keeps
    // application data, and tells it once.
    const QByteArray recent = recentPath().toUtf8();
    pf_session_set_recent_file(m_session, recent.constData());
    readCatalog();
    loadTheme();
    refreshRecent();
    // A safety copy left by a previous run is offered, never loaded on its own: the user
    // decides whether it is the work they lost or something they already saved.
    m_recoveryOffered = QFileInfo::exists(autosavePath());
    refreshDocument();
    refresh();
}

FluidSession::~FluidSession()
{
    pf_session_free(m_session);
}

bool FluidSession::event(QEvent *event)
{
    if (event->type() == QEvent::ApplicationPaletteChange) {
        loadTheme();
    }
    return QObject::event(event);
}

void FluidSession::setRunning(bool running)
{
    if (m_running == running) {
        return;
    }
    m_running = running;
    // Leaving the settled state is what restarts the loop: a pause and a resume both have
    // to clear the counter, or the window would come back still.
    m_steadyFrames = 0;
    if (m_settled) {
        m_settled = false;
        emit settledChanged();
    }
    emit runningChanged();
    emit changed();
    // Only an explicit start or pause sounds. The window opens already running — the model
    // is charging the cylinder while the user looks at it — and a simulator that makes a
    // noise before it is touched would be a surprise, not feedback.
    if (m_running) {
        emit transportStarted();
    } else {
        emit transportStopped();
    }
}

void FluidSession::toggleRunning()
{
    setRunning(!m_running);
}

void FluidSession::setSpeed(double speed)
{
    const double clamped = std::clamp(speed, 0.05, 8.0);
    if (qFuzzyCompare(m_speed, clamped)) {
        return;
    }
    m_speed = clamped;
    emit speedChanged();
}

void FluidSession::stepForward()
{
    if (!m_session) {
        return;
    }
    // A step is taken from a pause, so the display does not lurch on while the user is
    // pressing it, and it always repaints even if the model had settled.
    setRunning(false);
    advanceInternal(kFrameSeconds * m_speed, true);
}

bool FluidSession::runUntilChange()
{
    if (!m_session) {
        return false;
    }
    startSimulation();
    setRunning(false);
    const int valve = valvePosition();
    const double stroke = m_telemetry.stroke_m * 1000.0;
    const int frames = static_cast<int>(4.0 / kFrameSeconds);
    for (int frame = 0; frame < frames; ++frame) {
        advanceInternal(kFrameSeconds, false);
        if (!m_error.isEmpty()) {
            return false;
        }
        if (valvePosition() != valve) {
            return true;
        }
        if (stroke > 0.0 && positionMm() >= stroke - 1e-3) {
            return true;
        }
    }
    return false;
}

void FluidSession::advance(double dt)
{
    advanceInternal(dt, false);
}

void FluidSession::advanceInternal(double dt, bool forced)
{
    if (!m_session || pf_session_advance(m_session, dt) != PF_OK) {
        const char *message = m_session ? pf_session_last_error(m_session) : nullptr;
        if (message && *message) {
            m_error = QString::fromUtf8(message);
        }
        return;
    }
    refresh(forced);
}

void FluidSession::startSimulation()
{
    if (!m_session) {
        return;
    }
    const bool wasEditing = m_editing;
    m_editing = false;
    m_steadyFrames = 0;
    if (m_settled) {
        m_settled = false;
        emit settledChanged();
    }
    if (wasEditing) {
        emit modeChanged();
    }
    setRunning(true);
    emit changed();
}

void FluidSession::pauseSimulation()
{
    if (!m_session) {
        return;
    }
    // From editing this is the way in: the simulation becomes active without starting,
    // so the components can be looked at before the model moves.
    const bool wasEditing = m_editing;
    m_editing = false;
    setRunning(false);
    if (wasEditing) {
        emit modeChanged();
        emit changed();
    }
}

void FluidSession::stopSimulation()
{
    if (!m_session) {
        return;
    }
    setRunning(false);
    if (!m_editing) {
        m_editing = true;
        emit modeChanged();
    }
    emit changed();
}

void FluidSession::inspectComponent(int index)
{
    if (!m_session || index < 0) {
        return;
    }
    char buffer[1600] = {};
    if (pf_session_describe_component(m_session, static_cast<uint32_t>(index), buffer,
                                      sizeof(buffer))
        < 0) {
        return;
    }
    m_inspectorText = QString::fromUtf8(buffer);
    m_inspectorIndex = index;
    m_inspectorFields = QVariantList();
    PfEditSet set {};
    if (pf_session_component_edits(m_session, static_cast<uint32_t>(index), &set) == PF_OK) {
        const auto add = [this, &set](uint32_t bit, const char *key, const char *label,
                                      double value) {
            if ((set.mask & bit) == 0) {
                return;
            }
            QVariantMap row;
            row.insert(QStringLiteral("key"), QString::fromUtf8(key));
            row.insert(QStringLiteral("label"), QString::fromUtf8(label));
            row.insert(QStringLiteral("value"), value);
            m_inspectorFields.append(row);
        };
        add(PF_EDIT_PRESSURE, "pressure_bar", "Presión (bar)", set.pressure_bar);
        add(PF_EDIT_CONDUCTANCE, "conductance", "Conductancia (m³/s·Pa)", set.conductance);
        add(PF_EDIT_STROKE, "stroke_mm", "Carrera (mm)", set.stroke_mm);
        add(PF_EDIT_BORE, "bore_mm", "Émbolo (mm)", set.bore_mm);
        add(PF_EDIT_ROD, "rod_mm", "Vástago (mm)", set.rod_mm);
        add(PF_EDIT_MASS, "moving_mass_kg", "Masa móvil (kg)", set.moving_mass_kg);
    }
    // The name comes first in the reference's dialog, and it is the one field that is text:
    // `numeric` tells the form which keyboard to ask for and which parser to use.
    QVariantMap name;
    name.insert(QStringLiteral("key"), QStringLiteral("designation"));
    name.insert(QStringLiteral("label"), QStringLiteral("Designación"));
    name.insert(QStringLiteral("value"), designationOf(index));
    name.insert(QStringLiteral("numeric"), false);
    m_inspectorFields.prepend(name);
    m_inspectorOpen = true;
    emit inspectorChanged();
}

void FluidSession::inspectValve()
{
    if (!m_session) {
        return;
    }
    int index = m_selected;
    if (index < 0) {
        const int count = static_cast<int>(pf_session_component_count(m_session));
        for (int candidate = 0; candidate < count; ++candidate) {
            if (pf_session_component_valve_position(m_session,
                                                    static_cast<uint32_t>(candidate))
                >= 0) {
                index = candidate;
                break;
            }
        }
    }
    if (index >= 0) {
        inspectComponent(index);
    }
}

QString FluidSession::designationOf(int index) const
{
    if (!m_session || index < 0) {
        return QString();
    }
    char buffer[128] = {};
    const int32_t written = pf_session_component_designation(
        m_session, static_cast<uint32_t>(index), buffer, sizeof(buffer));
    return written > 0 ? QString::fromUtf8(buffer, written) : QString();
}

void FluidSession::setDesignation(int index, const QString &text)
{
    if (!m_session || index < 0) {
        return;
    }
    const QByteArray encoded = text.toUtf8();
    if (pf_session_set_component_designation(m_session, static_cast<uint32_t>(index),
                                             encoded.constData())
        != PF_OK) {
        const char *message = pf_session_last_error(m_session);
        m_error = message ? QString::fromUtf8(message) : QStringLiteral("designación no válida");
        emit changed();
        return;
    }
    refreshDocument();
    emit changed();
}

QVariantList FluidSession::chartCandidates()
{
    QVariantList list;
    if (!m_session || m_inspectorIndex < 0) {
        return list;
    }
    const auto index = static_cast<uint32_t>(m_inspectorIndex);
    PfChartSource bound {};
    int bound_id = -1;
    if (pf_session_chart_source(m_session, index, &bound) == 1) {
        bound_id = static_cast<int>(bound.placement);
    }
    const uint32_t count = pf_session_chart_candidate_count(m_session, index);
    list.reserve(static_cast<int>(count));
    for (uint32_t slot = 0; slot < count; ++slot) {
        PfChartSource candidate {};
        if (pf_session_chart_candidate(m_session, index, slot, &candidate) != PF_OK) {
            continue;
        }
        char label[256] = {};
        if (pf_session_chart_candidate_label(m_session, index, slot, label, sizeof label) < 0) {
            continue;
        }
        QVariantMap row;
        row.insert(QStringLiteral("placement"), static_cast<int>(candidate.placement));
        row.insert(QStringLiteral("quantity"), static_cast<int>(candidate.quantity));
        row.insert(QStringLiteral("label"), QString::fromUtf8(label));
        row.insert(QStringLiteral("current"), static_cast<int>(candidate.placement) == bound_id
                                             && candidate.quantity == bound.quantity
                                             && bound_id >= 0);
        list.append(row);
    }
    return list;
}

int FluidSession::chartSourcePlacement()
{
    if (!m_session || m_inspectorIndex < 0) {
        return -1;
    }
    PfChartSource source {};
    if (pf_session_chart_source(m_session, static_cast<uint32_t>(m_inspectorIndex), &source) != 1) {
        return -1;
    }
    return static_cast<int>(source.placement);
}

bool FluidSession::setChartSource(int placement, int quantity)
{
    if (!m_session || m_inspectorIndex < 0) {
        return false;
    }
    if (pf_session_set_chart_source(m_session, static_cast<uint32_t>(m_inspectorIndex),
                                    static_cast<uint32_t>(qMax(placement, 0)),
                                    static_cast<uint32_t>(qMax(quantity, 0)))
        != PF_OK) {
        const char *message = pf_session_last_error(m_session);
        m_error = message ? QString::fromUtf8(message)
                          : QStringLiteral("no se pudo enlazar el registrador");
        emit changed();
        return false;
    }
    m_error.clear();
    refreshDocument();
    emit inspectorChanged();
    emit changed();
    return true;
}

bool FluidSession::clearChartSource()
{
    if (!m_session || m_inspectorIndex < 0) {
        return false;
    }
    if (pf_session_clear_chart_source(m_session, static_cast<uint32_t>(m_inspectorIndex)) != PF_OK) {
        const char *message = pf_session_last_error(m_session);
        m_error = message ? QString::fromUtf8(message)
                          : QStringLiteral("no se pudo quitar la fuente");
        emit changed();
        return false;
    }
    m_error.clear();
    refreshDocument();
    emit inspectorChanged();
    emit changed();
    return true;
}

void FluidSession::applyInspectorEdits(const QVariantList &values)
{
    if (!m_session || m_inspectorIndex < 0) {
        return;
    }
    PfEditSet set {};
    for (const QVariant &entry : values) {
        const QVariantMap row = entry.toMap();
        const QString key = row.value(QStringLiteral("key")).toString();
        if (key == QLatin1String("designation")) {
            // A name is text, not a number, so it does not travel in the numeric edit set.
            setDesignation(m_inspectorIndex, row.value(QStringLiteral("value")).toString());
            continue;
        }
        bool ok = false;
        const double value = row.value(QStringLiteral("value")).toDouble(&ok);
        if (!ok) {
            continue;
        }
        if (key == QLatin1String("pressure_bar")) {
            set.mask |= PF_EDIT_PRESSURE;
            set.pressure_bar = value;
        } else if (key == QLatin1String("conductance")) {
            set.mask |= PF_EDIT_CONDUCTANCE;
            set.conductance = value;
        } else if (key == QLatin1String("stroke_mm")) {
            set.mask |= PF_EDIT_STROKE;
            set.stroke_mm = value;
        } else if (key == QLatin1String("bore_mm")) {
            set.mask |= PF_EDIT_BORE;
            set.bore_mm = value;
        } else if (key == QLatin1String("rod_mm")) {
            set.mask |= PF_EDIT_ROD;
            set.rod_mm = value;
        } else if (key == QLatin1String("moving_mass_kg")) {
            set.mask |= PF_EDIT_MASS;
            set.moving_mass_kg = value;
        }
    }
    if (set.mask == 0) {
        return;
    }
    const int32_t code = pf_session_set_component_edits(
        m_session, static_cast<uint32_t>(m_inspectorIndex), &set);
    const char *message = pf_session_last_error(m_session);
    m_error = (code == PF_OK || !message) ? QString() : QString::fromUtf8(message);
    refreshDocument();
    refresh(true);
    // Re-open with the effective values, so a rejected or clamped edit is visible.
    inspectComponent(m_inspectorIndex);
    emit changed();
}

void FluidSession::checkCircuit()
{
    if (!m_session) {
        return;
    }
    char buffer[2048] = {};
    const int32_t count = pf_session_check(m_session, buffer, sizeof(buffer));
    m_checkCount = count > 0 ? static_cast<int>(count) : 0;
    m_checkReport = m_checkCount > 0
                        ? QString::fromUtf8(buffer)
                        : QStringLiteral("Sin faltas: el circuito se puede simular.");
    m_checkOpen = true;
    m_inspectorOpen = false;
    emit inspectorChanged();
}

void FluidSession::closeInspector()
{
    if (m_checkOpen) {
        m_checkOpen = false;
        emit inspectorChanged();
    }
    if (!m_inspectorOpen) {
        return;
    }
    m_inspectorOpen = false;
    emit inspectorChanged();
}

void FluidSession::pressValve()
{
    if (!m_session) {
        return;
    }
    m_steadyFrames = 0;
    if (m_settled) {
        m_settled = false;
        emit settledChanged();
    }
    // A spring-return valve is only actuated while the gesture lasts; a latched one has no
    // release to come back from, so the same gesture toggles it.
    const int before = valvePosition();
    const int32_t code = m_valveSpringReturn ? pf_session_press_valve(m_session)
                                             : pf_session_toggle_valve(m_session);
    const char *message = pf_session_last_error(m_session);
    m_error = (code == PF_OK || !message) ? QString() : QString::fromUtf8(message);
    refresh();
    // The sound follows the spool, not the finger: a press that the definition refuses to
    // move is silent, because nothing happened.
    if (code == PF_OK && valvePosition() != before) {
        emit valveActuated();
    }
}

bool FluidSession::toggleContactAt(int index)
{
    if (!m_session || index < 0) {
        return false;
    }
    if (pf_session_toggle_contact(m_session, static_cast<uint32_t>(index)) != PF_OK) {
        return false;
    }
    refresh();
    emit changed();
    return true;
}

void FluidSession::pressValveAt(int index)
{
    if (!m_session || index < 0) {
        return;
    }
    const int before = valvePositionAt(index);
    pf_session_press_component_valve(m_session, static_cast<uint32_t>(index));
    refresh();
    if (valvePositionAt(index) != before) {
        emit valveActuated();
    }
}

void FluidSession::releaseValveAt(int index)
{
    if (!m_session || index < 0) {
        return;
    }
    const int before = valvePositionAt(index);
    pf_session_release_component_valve(m_session, static_cast<uint32_t>(index));
    refresh();
    if (valvePositionAt(index) != before) {
        emit valveActuated();
    }
}

int FluidSession::valvePositionAt(int index) const
{
    if (!m_session || index < 0) {
        return -1;
    }
    return pf_session_component_valve_position(m_session, static_cast<uint32_t>(index));
}

bool FluidSession::valveSpringReturnAt(int index) const
{
    if (!m_session || index < 0) {
        return false;
    }
    return pf_session_component_valve_spring_return(m_session, static_cast<uint32_t>(index)) == 1;
}

void FluidSession::releaseValve()
{
    if (!m_session || !m_valveSpringReturn) {
        return;
    }
    const int before = valvePosition();
    const int32_t code = pf_session_release_valve(m_session);
    const char *message = pf_session_last_error(m_session);
    m_error = (code == PF_OK || !message) ? QString() : QString::fromUtf8(message);
    refresh();
    if (code == PF_OK && valvePosition() != before) {
        emit valveActuated();
    }
}

void FluidSession::restart()
{
    if (!m_session) {
        return;
    }
    m_steadyFrames = 0;
    // A restarted model has no history: it is back at t = 0, so no arrival has happened yet
    // and none of the bookkeeping from before the restart may suppress the first impact.
    m_lastEnd = -1;
    m_wasAtEnd = false;
    m_awaySince = 0.0;
    m_lastVelocity = 0.0;
    m_lastExtension = 0.0;
    if (m_settled) {
        m_settled = false;
        emit settledChanged();
    }
    const int32_t code = pf_session_restart(m_session);
    const char *message = pf_session_last_error(m_session);
    m_error = (code == PF_OK || !message) ? QString() : QString::fromUtf8(message);
    refresh();
}

void FluidSession::refresh(bool forced)
{
    if (!m_session) {
        return;
    }
    refreshBuzzer();
    PfTelemetry telemetry {};
    if (pf_session_telemetry(m_session, &telemetry) != PF_OK) {
        return;
    }
    // The arrival is judged before the "unchanged" shortcut below: the frame on which the
    // piston stops is exactly a frame where nothing moves, so an announcement placed after
    // that check would never fire. The velocity that carried it into the stop is kept for
    // the comparison, because the solver zeroes it in the same step.
    const double carried = m_telemetry.velocity_m_s;
    m_lastVelocity = carried;
    announceImpact(telemetry);

    // "Unchanged" is measured, not compared bit for bit: an isolated volume approaches
    // equilibrium asymptotically, so the last decimal of the pressure moves for ever.
    // The thresholds are the smallest change a person could notice on screen.
    const bool still = std::abs(telemetry.position_m - m_telemetry.position_m) < 1.0e-6
                       && std::abs(telemetry.velocity_m_s - m_telemetry.velocity_m_s) < 1.0e-6
                       && std::abs(telemetry.pressure_pa - m_telemetry.pressure_pa) < 1.0
                       && telemetry.valve_position == m_telemetry.valve_position;
    if (still && !forced) {
        // Nothing visible moved: no repaint, and after half a second of stillness the
        // caller stops stepping altogether.
        if (++m_steadyFrames > 30 && !m_settled) {
            m_settled = true;
            emit settledChanged();
        }
        m_telemetry.time_s = telemetry.time_s;
        return;
    }
    m_steadyFrames = 0;
    if (m_settled) {
        m_settled = false;
        emit settledChanged();
    }
    m_telemetry = telemetry;
    emit changed();
}

void FluidSession::announceImpact(const PfTelemetry &telemetry)
{
    if (telemetry.stroke_m <= 0.0) {
        return;
    }
    const double extension = telemetry.position_m / telemetry.stroke_m;
    // At the end of the stroke, within the slack a discrete step leaves.
    constexpr double kSlack = 0.004;
    constexpr double kImpactSpeed = 0.02;
    const int atEnd = extension <= kSlack ? 0 : (extension >= 1.0 - kSlack ? 1 : -1);
    // Arriving is not the same as being there: the piston has to have been travelling
    // towards this end on the frame before. A piston that creeps up to the stop, or one
    // that is simply still against it, has nothing to report.
    const bool arrived = (atEnd == 1 && m_lastVelocity >= kImpactSpeed)
                         || (atEnd == 0 && m_lastVelocity <= -kImpactSpeed);
    // A piston that reaches the stop can be pushed back a few millimetres while the gas
    // that outran the supply refills the chamber, and it comes straight back. To a
    // listener that is one hit, not two, so an arrival inside this window is the same
    // contact; outside it, it is news.
    constexpr double kReboundSeconds = 0.12;
    if (atEnd < 0) {
        if (m_wasAtEnd) {
            m_awaySince = telemetry.time_s;
        }
        if (m_lastEnd >= 0 && (telemetry.time_s - m_awaySince) >= kReboundSeconds) {
            // Away from the end for good: the next arrival is a new contact.
            m_lastEnd = -1;
        }
    } else if (arrived && atEnd != m_lastEnd) {
        const bool rebound = m_lastEnd >= 0
                             && (telemetry.time_s - m_awaySince) < kReboundSeconds;
        if (!rebound) {
            m_lastEnd = atEnd;
            emit cylinderReachedEnd();
        }
    }
    m_wasAtEnd = atEnd >= 0;
    m_lastExtension = extension;
}

void FluidSession::readCatalog()
{
    if (!m_session) {
        return;
    }
    PfCatalog catalog {};
    if (pf_session_catalog(m_session, &catalog) != PF_OK) {
        return;
    }
    const auto text = QByteArray::fromRawData(reinterpret_cast<const char *>(catalog.text),
                                              static_cast<int>(catalog.text_len));
    const auto field = [&text](uint32_t at, uint32_t len) {
        return QString::fromUtf8(text.mid(static_cast<int>(at), static_cast<int>(len)));
    };

    m_catalog.clear();
    for (uint32_t index = 0; index < catalog.items_len; ++index) {
        const PfCatalogItem &item = catalog.items[index];
        QVariantMap row;
        row.insert(QStringLiteral("id"), field(item.id_at, item.id_len));
        row.insert(QStringLiteral("name"), field(item.name_at, item.name_len));
        row.insert(QStringLiteral("category"), static_cast<int>(item.category));
        row.insert(QStringLiteral("index"), static_cast<int>(index));
        m_catalog.append(row);
    }
    emit catalogChanged();
}

void FluidSession::loadTheme()
{
    if (!qGuiApp) {
        return;
    }
    applyPalette(QGuiApplication::palette());
}

void FluidSession::applyPalette(const QPalette &palette)
{
    m_tokens = tokensFrom(palette);
    m_contrast = measure(m_tokens);

    if (m_session) {
        PfInk ink {};
        ink.line = toRgb(m_tokens.ink);
        ink.text = toRgb(m_tokens.ink);
        ink.dim = toRgb(m_tokens.inkDim);
        ink.muted = toRgb(m_tokens.inkMuted);
        ink.supply = toRgb(m_tokens.supply);
        ink.work = toRgb(m_tokens.work);
        ink.accent = toRgb(m_tokens.accent.lighter(150));
        ink.background = toRgb(m_tokens.background);
        pf_session_set_ink(m_session, &ink);
    }
    emit changed();
}

bool FluidSession::exportWithOptions(const QVariantMap &options)
{
    if (!m_session) {
        return false;
    }
    const QString local = options.value(QStringLiteral("path")).toString();
    if (local.isEmpty()) {
        return false;
    }
    const QString path = QUrl(local).isLocalFile() ? QUrl(local).toLocalFile() : local;
    const QByteArray encoded = path.toUtf8();
    const QByteArray date = QDate::currentDate().toString(Qt::ISODate).toUtf8();
    const QVariantList margins = options.value(QStringLiteral("margins")).toList();
    const auto margin = [&margins](int index) -> float {
        // The ISO 5457 frame: 20 mm on the filing edge (left) and 10 mm elsewhere.
        if (margins.size() > index) {
            return margins.at(index).toFloat();
        }
        return index == 3 ? 20.0F : 10.0F;
    };
    PfExportOptions settings {};
    const int kind = options.value(QStringLiteral("kind"), 1).toInt();
    settings.kind = kind == 0 ? PF_EXPORT_PNG : (kind == 2 ? PF_EXPORT_SVG : PF_EXPORT_PDF);
    // The sheet is the paper: the print follows the document unless the panel asks for a
    // custom size for this one file.
    settings.paper = PF_EXPORT_DOCUMENT_PAPER;
    if (options.value(QStringLiteral("custom_paper")).toBool()) {
        settings.paper = PF_EXPORT_CUSTOM_PAPER;
        settings.width_mm = options.value(QStringLiteral("width")).toFloat();
        settings.height_mm = options.value(QStringLiteral("height")).toFloat();
    }
    settings.flags = 0;
    if (options.value(QStringLiteral("landscape")).toBool()) {
        settings.flags |= PF_EXPORT_LANDSCAPE;
    }
    if (options.value(QStringLiteral("parts_list")).toBool()) {
        settings.flags |= PF_EXPORT_PARTS_LIST;
    }
    settings.dpi = options.value(QStringLiteral("dpi")).toFloat();
    settings.margin_top = margin(0);
    settings.margin_right = margin(1);
    settings.margin_bottom = margin(2);
    settings.margin_left = margin(3);
    if (pf_session_export_with(m_session, encoded.constData(), date.constData(), &settings)
        != PF_OK) {
        const char *message = pf_session_last_error(m_session);
        m_error = message ? QString::fromUtf8(message) : QStringLiteral("no se pudo exportar");
        emit changed();
        return false;
    }
    m_error.clear();
    emit changed();
    return true;
}

bool FluidSession::exportTo(const QString &path, int kind, int paper, bool landscape,
                            qreal dpi)
{
    if (!m_session || path.isEmpty()) {
        return false;
    }
    const QUrl url(path);
    const QString local = url.isLocalFile() ? url.toLocalFile() : path;
    const QByteArray encoded = local.toUtf8();
    // The ABI does not read the clock: the front end supplies the local date the title
    // block carries, so an export is reproducible within the day.
    const QByteArray date = QDate::currentDate().toString(Qt::ISODate).toUtf8();
    const uint32_t format = kind == 1 ? PF_EXPORT_PDF
                                      : (kind == 2 ? PF_EXPORT_SVG : PF_EXPORT_PNG);
    if (pf_session_export(m_session, encoded.constData(), format,
                          static_cast<uint32_t>(qMax(paper, 0)), landscape ? 1U : 0U,
                          static_cast<float>(dpi), date.constData())
        != PF_OK) {
        const char *message = pf_session_last_error(m_session);
        m_error = message ? QString::fromUtf8(message)
                          : QStringLiteral("no se pudo exportar");
        emit changed();
        return false;
    }
    m_error.clear();
    emit changed();
    return true;
}

QStringList FluidSession::paperNames()
{
    QStringList names;
    const uint32_t count = pf_export_paper_count();
    for (uint32_t index = 0; index < count; ++index) {
        char label[64] = {};
        if (pf_export_paper_name(index, label, sizeof label) == PF_OK) {
            names.append(QString::fromUtf8(label));
        }
    }
    return names;
}

bool FluidSession::saveTo(const QString &path)
{
    if (!m_session || path.isEmpty()) {
        return false;
    }
    const QUrl url(path);
    const QString local = url.isLocalFile() ? url.toLocalFile() : path;
    const QByteArray encoded = local.toUtf8();
    if (pf_session_save(m_session, encoded.constData()) != PF_OK) {
        const char *message = pf_session_last_error(m_session);
        m_error = message ? QString::fromUtf8(message) : QStringLiteral("no se pudo guardar");
        emit changed();
        return false;
    }
    m_currentPath = local;
    m_error.clear();
    // The copy exists for a crash, not for a saved file: once the work is in the file it
    // has nothing left to rescue, and leaving it behind makes the next start offer to
    // recover work that is already safe.
    QFile::remove(autosavePath());
    m_recoveryOffered = false;
    refreshRecent();
    refreshDocument();
    emit changed();
    return true;
}

bool FluidSession::openFrom(const QString &path)
{
    if (!m_session || path.isEmpty()) {
        return false;
    }
    const QUrl url(path);
    const QString local = url.isLocalFile() ? url.toLocalFile() : path;
    const QByteArray encoded = local.toUtf8();
    if (pf_session_open(m_session, encoded.constData()) != PF_OK) {
        const char *message = pf_session_last_error(m_session);
        m_error = message ? QString::fromUtf8(message) : QStringLiteral("no se pudo abrir");
        emit changed();
        return false;
    }
    m_currentPath = local;
    m_error.clear();
    refreshRecent();
    m_selected = -1;
    m_editing = true;
    m_running = false;
    m_settled = false;
    m_valveSpringReturn = pf_session_valve_spring_return(m_session) == 1;
    readCatalog();
    refreshDocument();
    refresh(true);
    emit catalogChanged();
    emit modeChanged();
    emit settledChanged();
    emit runningChanged();
    emit changed();
    return true;
}

QString FluidSession::title() const
{
    const QString name = m_currentPath.isEmpty() ? QStringLiteral("Sin título")
                                                 : QFileInfo(m_currentPath).fileName();
    // The mark is a character, not a colour: a window title has no other channel, and a
    // tint alone would be invisible in a screenshot or to a colour-blind reader.
    return m_modified ? QStringLiteral("• %1 — Punchi Fluid Lab").arg(name)
                      : QStringLiteral("%1 — Punchi Fluid Lab").arg(name);
}

QVariantList FluidSession::circuits() const
{
    QVariantList list;
    if (!m_session) {
        return list;
    }
    const uint32_t count = pf_session_circuit_count(m_session);
    for (uint32_t index = 0; index < count; ++index) {
        char name[256] = {};
        if (pf_session_circuit_name(m_session, index, name, sizeof name) < 0) {
            continue;
        }
        const int id = pf_session_circuit_id(m_session, index);
        if (id < 0) {
            continue;
        }
        QVariantMap row;
        row.insert(QStringLiteral("id"), id);
        row.insert(QStringLiteral("name"), QString::fromUtf8(name));
        list.append(row);
    }
    return list;
}

int FluidSession::activeCircuit() const
{
    return m_session ? pf_session_active_circuit(m_session) : 0;
}

bool FluidSession::selectCircuit(int id)
{
    if (!m_session) {
        return false;
    }
    if (pf_session_select_circuit(m_session, static_cast<uint32_t>(qMax(id, 0))) != PF_OK) {
        const char *message = pf_session_last_error(m_session);
        m_error = message ? QString::fromUtf8(message)
                          : QStringLiteral("no se pudo abrir el circuito");
        emit changed();
        return false;
    }
    m_error.clear();
    m_selected = -1;
    m_inspectorOpen = false;
    m_editing = true;
    m_running = false;
    m_settled = false;
    m_valveSpringReturn = pf_session_valve_spring_return(m_session) == 1;
    refreshDocument();
    refresh(true);
    emit inspectorChanged();
    emit modeChanged();
    emit settledChanged();
    emit runningChanged();
    emit changed();
    return true;
}

int FluidSession::addCircuit()
{
    if (!m_session) {
        return -1;
    }
    const uint32_t count = pf_session_circuit_count(m_session);
    const QString title = QStringLiteral("Circuito %1").arg(count + 1);
    const QByteArray encoded = title.toUtf8();
    const int id = pf_session_add_circuit(m_session, encoded.constData());
    if (id < 0) {
        const char *message = pf_session_last_error(m_session);
        m_error = message ? QString::fromUtf8(message)
                          : QStringLiteral("no se pudo añadir el circuito");
        emit changed();
        return -1;
    }
    m_error.clear();
    m_selected = -1;
    m_inspectorOpen = false;
    m_editing = true;
    m_running = false;
    m_settled = false;
    m_valveSpringReturn = pf_session_valve_spring_return(m_session) == 1;
    refreshDocument();
    refresh(true);
    emit inspectorChanged();
    emit modeChanged();
    emit settledChanged();
    emit runningChanged();
    emit changed();
    return id;
}

bool FluidSession::renameCircuit(int id, const QString &title)
{
    if (!m_session || title.isEmpty()) {
        return false;
    }
    const QByteArray encoded = title.toUtf8();
    if (pf_session_rename_circuit(m_session, static_cast<uint32_t>(qMax(id, 0)),
                                  encoded.constData())
        != PF_OK) {
        const char *message = pf_session_last_error(m_session);
        m_error = message ? QString::fromUtf8(message)
                          : QStringLiteral("no se pudo renombrar el circuito");
        emit changed();
        return false;
    }
    m_error.clear();
    refreshDocument();
    emit changed();
    return true;
}

QString FluidSession::recentPath() const
{
    QString directory = QStandardPaths::writableLocation(QStandardPaths::AppDataLocation);
    if (directory.isEmpty()) {
        directory = QDir::homePath();
    }
    return directory + QStringLiteral("/recent.json");
}

void FluidSession::refreshBuzzer()
{
    const bool active = m_session && pf_session_buzzer_active(m_session) == 1;
    if (active == m_buzzerActive) {
        return;
    }
    m_buzzerActive = active;
    emit buzzerChanged();
}

void FluidSession::refreshRecent()
{
    m_recent.clear();
    if (!m_session) {
        emit recentChanged();
        return;
    }
    const uint32_t count = pf_session_recent_count(m_session);
    for (uint32_t slot = 0; slot < count; ++slot) {
        std::vector<char> buffer(4096, '\0');
        const int32_t written = pf_session_recent_path(m_session, slot, buffer.data(),
                                                       static_cast<uint32_t>(buffer.size()));
        if (written <= 0) {
            continue;
        }
        const QString path = QString::fromUtf8(buffer.data(), written);
        QVariantMap entry;
        entry.insert(QStringLiteral("path"), path);
        entry.insert(QStringLiteral("name"), QFileInfo(path).fileName());
        m_recent.append(entry);
    }
    emit recentChanged();
}

bool FluidSession::openRecent(int slot)
{
    if (!m_session || slot < 0) {
        return false;
    }
    if (pf_session_open_recent(m_session, static_cast<uint32_t>(slot)) != PF_OK) {
        const char *message = pf_session_last_error(m_session);
        m_error = message ? QString::fromUtf8(message) : QStringLiteral("no se pudo abrir");
        // A document that vanished is dropped from the list by the core, so the menu is
        // refreshed even when the open fails.
        refreshRecent();
        emit changed();
        return false;
    }
    // The path comes from the list, which is what the core just opened.
    if (slot < m_recent.size()) {
        m_currentPath = m_recent.at(slot).toMap().value(QStringLiteral("path")).toString();
    }
    m_error.clear();
    m_selected = -1;
    m_editing = true;
    m_running = false;
    m_settled = false;
    m_valveSpringReturn = pf_session_valve_spring_return(m_session) == 1;
    readCatalog();
    refreshRecent();
    refreshDocument();
    refresh(true);
    emit catalogChanged();
    emit modeChanged();
    emit settledChanged();
    emit runningChanged();
    emit changed();
    return true;
}

void FluidSession::forgetRecent()
{
    if (!m_session) {
        return;
    }
    pf_session_forget_recent(m_session);
    refreshRecent();
}

QString FluidSession::autosavePath() const
{
    QString directory = QStandardPaths::writableLocation(QStandardPaths::AppDataLocation);
    if (directory.isEmpty()) {
        directory = QDir::homePath();
    }
    return directory + QStringLiteral("/autosave.pfluid");
}

void FluidSession::writeAutosave()
{
    if (!m_session) {
        return;
    }
    const QString path = autosavePath();
    const QString directory = QFileInfo(path).absolutePath();
    QDir().mkpath(directory);
    const QByteArray encoded = path.toUtf8();
    pf_session_autosave(m_session, encoded.constData());
}

void FluidSession::autosave()
{
    if (!m_session || !m_modified) {
        return;
    }
    writeAutosave();
}

bool FluidSession::recoverAutosave()
{
    if (!m_session) {
        return false;
    }
    const QByteArray encoded = autosavePath().toUtf8();
    if (pf_session_recover(m_session, encoded.constData()) != PF_OK) {
        const char *message = pf_session_last_error(m_session);
        m_error = message ? QString::fromUtf8(message) : QStringLiteral("no se pudo recuperar");
        emit changed();
        return false;
    }
    // The copy is the user's work from a run that ended badly; it has no path of its own,
    // so the next save asks where it goes. The copy itself is consumed: what is on the
    // sheet is the document now, and the timer writes a new one while it is unsaved.
    QFile::remove(autosavePath());
    m_currentPath.clear();
    m_recoveryOffered = false;
    m_selected = -1;
    m_editing = true;
    m_running = false;
    m_settled = false;
    m_valveSpringReturn = pf_session_valve_spring_return(m_session) == 1;
    refreshDocument();
    refresh(true);
    emit catalogChanged();
    emit modeChanged();
    emit settledChanged();
    emit runningChanged();
    emit changed();
    return true;
}

void FluidSession::discardAutosave()
{
    QFile::remove(autosavePath());
    m_recoveryOffered = false;
    emit documentChanged();
}

void FluidSession::discardAndNewDocument()
{
    newDocument();
    // Starting over is only safe once what was there is gone from the safety copy too.
    QFile::remove(autosavePath());
    m_recoveryOffered = false;
    emit documentChanged();
}

bool FluidSession::discardAndOpen(const QString &path)
{
    const bool opened = openFrom(path);
    if (opened) {
        QFile::remove(autosavePath());
        m_recoveryOffered = false;
        emit documentChanged();
    }
    return opened;
}

void FluidSession::confirmClose()
{
    // Closing is not a crash: whichever way the user leaves — saved or discarded — the copy
    // must not be offered on the next start. Only a run that ends without this call leaves
    // one, which is exactly the case it is for.
    QFile::remove(autosavePath());
    m_recoveryOffered = false;
    m_closeConfirmed = true;
    emit changed();
}

bool FluidSession::saveCurrent()
{
    return !m_currentPath.isEmpty() && saveTo(m_currentPath);
}

void FluidSession::newDocument()
{
    if (!m_session) {
        return;
    }
    pf_session_new_document(m_session);
    m_currentPath.clear();
    m_selected = -1;
    m_editing = true;
    m_running = false;
    m_settled = false;
    m_valveSpringReturn = pf_session_valve_spring_return(m_session) == 1;
    refreshDocument();
    refresh(true);
    emit documentChanged();
    emit modeChanged();
    emit settledChanged();
    emit runningChanged();
    emit changed();
}

void FluidSession::refreshSelection()
{
    if (!m_session) {
        return;
    }
    // Marking a component is not an edit: the document is untouched and undo must not
    // grow an entry, so this announces the selection and nothing else.
    m_selected = pf_session_selected(m_session);
    emit documentChanged();
}

bool FluidSession::refreshSheet()
{
    if (!m_session) {
        return false;
    }
    PfSheet sheet {};
    if (pf_session_sheet(m_session, &sheet) != PF_OK) {
        return false;
    }
    const bool changed = sheet.paper != m_sheet.paper || sheet.landscape != m_sheet.landscape
        || sheet.width_mm != m_sheet.width_mm || sheet.height_mm != m_sheet.height_mm
        || sheet.margin_top != m_sheet.margin_top || sheet.margin_right != m_sheet.margin_right
        || sheet.margin_bottom != m_sheet.margin_bottom
        || sheet.margin_left != m_sheet.margin_left;
    m_sheet = sheet;
    m_sheetWidthMm = sheet.landscape != 0 ? sheet.height_mm : sheet.width_mm;
    m_sheetHeightMm = sheet.landscape != 0 ? sheet.width_mm : sheet.height_mm;
    if (sheet.paper == PF_SHEET_CUSTOM_PAPER) {
        m_sheetPaperName = QStringLiteral("Personalizado");
    } else {
        char label[64] = {};
        m_sheetPaperName = pf_export_paper_name(sheet.paper, label, sizeof label) == PF_OK
            ? QString::fromUtf8(label)
            : QString();
    }
    if (changed) {
        emit sheetChanged();
    }
    return changed;
}

void FluidSession::refreshDocument()
{
    if (!m_session) {
        return;
    }
    const bool sheetChanged = refreshSheet();
    const int count = static_cast<int>(pf_session_component_count(m_session));
    const int selected = pf_session_selected(m_session);
    const bool canUndo = pf_session_can_undo(m_session) == 1;
    const bool canRedo = pf_session_can_redo(m_session) == 1;
    const bool modified = pf_session_is_modified(m_session) == 1;
    if (count == m_componentCount && selected == m_selected && canUndo == m_canUndo
        && canRedo == m_canRedo && modified == m_modified && !sheetChanged) {
        return;
    }
    m_componentCount = count;
    m_selected = selected;
    m_canUndo = canUndo;
    m_canRedo = canRedo;
    m_modified = modified;
    emit documentChanged();
}

int FluidSession::placeComponent(int catalogIndex, double xMm, double yMm)
{
    if (!m_session) {
        return -1;
    }
    const int32_t index = pf_session_place_component(m_session, static_cast<uint32_t>(catalogIndex),
                                                     static_cast<float>(xMm),
                                                     static_cast<float>(yMm));
    m_error.clear();
    refreshDocument();
    emit changed();
    return index;
}

void FluidSession::moveComponent(int index, double xMm, double yMm)
{
    if (!m_session || index < 0) {
        return;
    }
    pf_session_move_component(m_session, static_cast<uint32_t>(index), static_cast<float>(xMm),
                              static_cast<float>(yMm));
    refreshDocument();
    emit changed();
}

void FluidSession::routePipe(int index, double xMm, double yMm)
{
    if (!m_session || index < 0) {
        return;
    }
    pf_session_route_pipe(m_session, static_cast<uint32_t>(index), static_cast<float>(xMm),
                          static_cast<float>(yMm));
    refreshDocument();
    emit changed();
}

void FluidSession::removeComponent(int index)
{
    if (!m_session || index < 0) {
        return;
    }
    pf_session_remove_component(m_session, static_cast<uint32_t>(index));
    refreshDocument();
    emit changed();
}

void FluidSession::copySelected()
{
    if (!m_session) {
        return;
    }
    if (pf_session_copy_selected(m_session) != PF_OK) {
        const char *message = pf_session_last_error(m_session);
        m_error = message ? QString::fromUtf8(message) : QStringLiteral("no hay nada que copiar");
        emit changed();
    }
}

int FluidSession::pasteAt(double xMm, double yMm)
{
    if (!m_session) {
        return -1;
    }
    const int32_t index = pf_session_paste(m_session, static_cast<float>(xMm),
                                           static_cast<float>(yMm));
    refreshDocument();
    emit changed();
    return index;
}

void FluidSession::pasteOffset()
{
    if (!m_session) {
        return;
    }
    // Without a pointer, the copy lands next to where it was taken.
    pf_session_paste_offset(m_session, 10.0F, 10.0F);
    refreshDocument();
    emit changed();
}

void FluidSession::duplicateSelected()
{
    if (selectionCount() > 1) {
        duplicateSelection(10.0, 10.0);
        return;
    }
    const int index = selected();
    if (index >= 0) {
        duplicateComponent(index, 10.0, 10.0);
    }
}

int FluidSession::duplicateComponent(int index, double dxMm, double dyMm)
{
    if (!m_session || index < 0) {
        return -1;
    }
    const int32_t copy = pf_session_duplicate_component(m_session, static_cast<uint32_t>(index),
                                                        static_cast<float>(dxMm),
                                                        static_cast<float>(dyMm));
    refreshDocument();
    emit changed();
    return copy;
}

void FluidSession::toggleSelect(int index)
{
    if (!m_session || index < 0) {
        return;
    }
    if (pf_session_toggle_select(m_session, index) != PF_OK) {
        const char *message = pf_session_last_error(m_session);
        m_error = message ? QString::fromUtf8(message) : QStringLiteral("no se puede marcar");
        emit changed();
        return;
    }
    refreshSelection();
    emit changed();
}

void FluidSession::selectAll()
{
    if (!m_session) {
        return;
    }
    pf_session_select_all(m_session);
    refreshSelection();
    emit changed();
}

void FluidSession::selectRect(double x0Mm, double y0Mm, double x1Mm, double y1Mm)
{
    if (!m_session) {
        return;
    }
    pf_session_select_rect(m_session, static_cast<float>(x0Mm), static_cast<float>(y0Mm),
                           static_cast<float>(x1Mm), static_cast<float>(y1Mm));
    refreshSelection();
    emit changed();
}

int FluidSession::selectionCount() const
{
    return m_session ? static_cast<int>(pf_session_selection_count(m_session)) : 0;
}

int FluidSession::selectionAt(int slot) const
{
    return m_session ? pf_session_selection_at(m_session, static_cast<uint32_t>(slot)) : -1;
}

bool FluidSession::isSelected(int index) const
{
    if (!m_session || index < 0) {
        return false;
    }
    const int count = selectionCount();
    for (int slot = 0; slot < count; ++slot) {
        if (selectionAt(slot) == index) {
            return true;
        }
    }
    return false;
}

void FluidSession::moveSelection(double rightMm, double downMm)
{
    if (!m_session) {
        return;
    }
    if (pf_session_move_selection(m_session, static_cast<float>(rightMm),
                                  static_cast<float>(downMm))
        != PF_OK) {
        return;
    }
    refreshDocument();
    emit changed();
}

void FluidSession::duplicateSelection(double rightMm, double downMm)
{
    if (!m_session) {
        return;
    }
    if (pf_session_duplicate_selection(m_session, static_cast<float>(rightMm),
                                       static_cast<float>(downMm))
        <= 0) {
        const char *message = pf_session_last_error(m_session);
        m_error = message ? QString::fromUtf8(message) : QStringLiteral("no hay nada que copiar");
        emit changed();
        return;
    }
    refreshDocument();
    emit changed();
}

void FluidSession::removeSelection()
{
    if (!m_session) {
        return;
    }
    if (pf_session_remove_selection(m_session) <= 0) {
        const char *message = pf_session_last_error(m_session);
        m_error = message ? QString::fromUtf8(message) : QStringLiteral("no hay nada que borrar");
        emit changed();
        return;
    }
    refreshDocument();
    emit changed();
}

void FluidSession::alignSelection(int edge)
{
    if (!m_session || edge < 0) {
        return;
    }
    if (pf_session_align_selection(m_session, static_cast<uint32_t>(edge)) != PF_OK) {
        const char *message = pf_session_last_error(m_session);
        m_error = message ? QString::fromUtf8(message) : QStringLiteral("no hay nada que alinear");
        emit changed();
        return;
    }
    refreshDocument();
    emit changed();
}

int FluidSession::selectedPipe() const
{
    return m_session ? pf_session_selected_pipe(m_session) : -1;
}

void FluidSession::setPipeClass(int lineClass)
{
    if (!m_session || selectedPipe() < 0) {
        return;
    }
    if (pf_session_set_pipe_class(m_session, static_cast<uint32_t>(selectedPipe()),
                                  static_cast<uint32_t>(lineClass))
        != PF_OK) {
        const char *message = pf_session_last_error(m_session);
        m_error = message ? QString::fromUtf8(message) : QStringLiteral("no se pudo cambiar la clase");
        emit changed();
        return;
    }
    refreshDocument();
    emit changed();
}

void FluidSession::spaceSelection(int axis)
{
    if (!m_session || axis < 0) {
        return;
    }
    if (pf_session_space_selection(m_session, static_cast<uint32_t>(axis)) != PF_OK) {
        const char *message = pf_session_last_error(m_session);
        m_error = message ? QString::fromUtf8(message) : QStringLiteral("no hay nada que repartir");
        emit changed();
        return;
    }
    refreshDocument();
    emit changed();
}

void FluidSession::rotateSelected(bool anticlockwise)
{
    if (!m_session) {
        return;
    }
    if (pf_session_rotate_selected(m_session, anticlockwise ? 1 : 0) != PF_OK) {
        const char *message = pf_session_last_error(m_session);
        m_error = message ? QString::fromUtf8(message) : QStringLiteral("no hay nada que girar");
        emit changed();
        return;
    }
    refreshDocument();
    emit changed();
}

void FluidSession::mirrorSelected()
{
    if (!m_session) {
        return;
    }
    if (pf_session_mirror_selected(m_session) != PF_OK) {
        const char *message = pf_session_last_error(m_session);
        m_error = message ? QString::fromUtf8(message) : QStringLiteral("no hay nada que reflejar");
        emit changed();
        return;
    }
    refreshDocument();
    emit changed();
}

void FluidSession::removeSelected()
{
    if (!m_session) {
        return;
    }
    // One key, two kinds of thing: whichever is selected is the one Delete removes, and a
    // selection of several components goes away in one press and one undo.
    if (pf_session_selected_pipe(m_session) >= 0) {
        pf_session_remove_selected_pipe(m_session);
        refreshDocument();
        emit changed();
        return;
    }
    if (selectionCount() > 0) {
        removeSelection();
    }
}

void FluidSession::selectPipe(int index)
{
    if (!m_session) {
        return;
    }
    pf_session_select_pipe(m_session, index);
    refreshDocument();
    emit changed();
}

int FluidSession::pipeAt(double xMm, double yMm, double toleranceMm)
{
    if (!m_session) {
        return -1;
    }
    return pf_session_pipe_at(m_session, static_cast<float>(xMm), static_cast<float>(yMm),
                              static_cast<float>(toleranceMm));
}

void FluidSession::select(int index)
{
    if (!m_session) {
        return;
    }
    pf_session_select(m_session, index);
    refreshDocument();
    emit changed();
}

int FluidSession::componentAt(double xMm, double yMm)
{
    if (!m_session) {
        return -1;
    }
    return pf_session_hit_component(m_session, static_cast<float>(xMm), static_cast<float>(yMm));
}

void FluidSession::setZoom(double zoom)
{
    // Bounded: a sheet zoomed to a hundredth is a dot and one zoomed to a hundred is a
    // single port filling the window. Neither is a view anyone asked for.
    const double clamped = std::clamp(zoom, kMinZoom, kMaxZoom);
    if (qFuzzyCompare(clamped, m_zoom)) {
        return;
    }
    m_zoom = clamped;
    emit viewChanged();
    emit changed();
}

void FluidSession::setGrid(bool on)
{
    if (m_grid == on) {
        return;
    }
    m_grid = on;
    emit viewChanged();
    emit changed();
}

void FluidSession::zoomIn()
{
    setZoom(m_zoom * kZoomStep);
}

void FluidSession::zoomOut()
{
    setZoom(m_zoom / kZoomStep);
}

void FluidSession::zoomReset()
{
    setZoom(1.0);
}

void FluidSession::toggleGrid()
{
    setGrid(!m_grid);
}

int FluidSession::sheetPaper() const
{
    // A custom sheet has no index in the paper list; -1 says so to the interface.
    return m_sheet.paper == PF_SHEET_CUSTOM_PAPER ? -1 : static_cast<int>(m_sheet.paper);
}

void FluidSession::setSheetPaper(int index)
{
    if (index < 0) {
        return;
    }
    PfSheet next = m_sheet;
    next.paper = static_cast<uint32_t>(index);
    applySheet(next);
}

void FluidSession::setSheetLandscape(bool landscape)
{
    PfSheet next = m_sheet;
    next.landscape = landscape ? 1U : 0U;
    applySheet(next);
}

void FluidSession::toggleSheetLandscape()
{
    setSheetLandscape(!sheetLandscape());
}

void FluidSession::setSheetMargin(int side, double mm)
{
    if (mm < 0.0) {
        return;
    }
    PfSheet next = m_sheet;
    const auto value = static_cast<float>(mm);
    switch (side) {
    case 0:
        next.margin_top = value;
        break;
    case 1:
        next.margin_right = value;
        break;
    case 2:
        next.margin_bottom = value;
        break;
    case 3:
        next.margin_left = value;
        break;
    default:
        return;
    }
    applySheet(next);
}

void FluidSession::resetSheet()
{
    PfSheet next = m_sheet;
    const int a4 = paperNames().indexOf(QStringLiteral("A4"));
    next.paper = static_cast<uint32_t>(a4 < 0 ? 0 : a4);
    next.landscape = 0;
    // ISO 5457: 10 mm on three sides and 20 mm on the filing edge.
    next.margin_top = 10.0F;
    next.margin_right = 10.0F;
    next.margin_bottom = 10.0F;
    next.margin_left = 20.0F;
    applySheet(next);
}

void FluidSession::applySheet(const PfSheet &sheet)
{
    if (!m_session) {
        return;
    }
    if (pf_session_set_sheet(m_session, &sheet) != PF_OK) {
        const char *message = pf_session_last_error(m_session);
        m_error = message ? QString::fromUtf8(message)
                          : QStringLiteral("no se pudo cambiar la hoja");
        emit changed();
        return;
    }
    m_error.clear();
    refreshDocument();
    emit changed();
}

void FluidSession::undo()
{
    if (!m_session) {
        return;
    }
    pf_session_undo(m_session);
    refreshDocument();
    emit changed();
}

void FluidSession::redo()
{
    if (!m_session) {
        return;
    }
    pf_session_redo(m_session);
    refreshDocument();
    emit changed();
}

bool FluidSession::portAt(double xMm, double yMm, double toleranceMm, int *index,
                          QString *port) const
{
    if (!m_session || index == nullptr || port == nullptr) {
        return false;
    }
    char name[32] = {};
    uint32_t found = 0;
    const int32_t code = pf_session_port_at(m_session, static_cast<float>(xMm),
                                            static_cast<float>(yMm),
                                            static_cast<float>(toleranceMm), &found, name,
                                            sizeof(name));
    if (code != PF_OK) {
        return false;
    }
    *index = static_cast<int>(found);
    *port = QString::fromUtf8(name);
    return true;
}

bool FluidSession::connectPorts(int fromIndex, const QString &fromPort, int toIndex,
                                const QString &toPort)
{
    if (!m_session) {
        return false;
    }
    const QByteArray from = fromPort.toUtf8();
    const QByteArray to = toPort.toUtf8();
    const int32_t code = pf_session_connect(m_session, static_cast<uint32_t>(fromIndex), from.constData(),
                                            static_cast<uint32_t>(toIndex), to.constData());
    const char *message = pf_session_last_error(m_session);
    m_error = (code == PF_OK || message == nullptr) ? QString() : QString::fromUtf8(message);
    refreshDocument();
    emit changed();
    return code == PF_OK;
}

bool FluidSession::splicePipeAt(int pipeIndex, double xMm, double yMm, int fromIndex,
                                const QString &fromPort)
{
    if (!m_session) {
        return false;
    }
    const QByteArray port = fromPort.toUtf8();
    const int32_t code = pf_session_splice_pipe(
        m_session, static_cast<uint32_t>(pipeIndex), static_cast<float>(xMm),
        static_cast<float>(yMm), static_cast<uint32_t>(fromIndex), port.constData());
    const char *message = pf_session_last_error(m_session);
    m_error = (code == PF_OK || message == nullptr) ? QString() : QString::fromUtf8(message);
    refreshDocument();
    emit changed();
    return code == PF_OK;
}

bool FluidSession::portPoint(int index, const QString &port, QPointF *point) const
{
    if (!m_session || point == nullptr) {
        return false;
    }
    const QByteArray name = port.toUtf8();
    float at[2] = {0.0F, 0.0F};
    if (pf_session_port_point(m_session, static_cast<uint32_t>(index), name.constData(), at)
        != PF_OK) {
        return false;
    }
    *point = QPointF(at[0], at[1]);
    return true;
}

int FluidSession::connectionCount() const
{
    return m_session ? static_cast<int>(pf_session_connection_count(m_session)) : 0;
}
