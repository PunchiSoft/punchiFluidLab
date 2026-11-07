// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#include "session.h"

#include <QUrl>

#include <QEvent>

#include <algorithm>
#include <cmath>
#include <QGuiApplication>
#include <QVariantMap>

#include <string>

namespace {
/// Zoom limits and the step a button takes. The reference zooms by buttons and by the
/// keys `>` and `<`; a step of a quarter is what makes five presses double the size.
constexpr double kMinZoom = 0.4;
constexpr double kMaxZoom = 6.0;
constexpr double kZoomStep = 1.25;
} // namespace

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
    readCatalog();
    loadTheme();
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

void FluidSession::applyInspectorEdits(const QVariantList &values)
{
    if (!m_session || m_inspectorIndex < 0) {
        return;
    }
    PfEditSet set {};
    for (const QVariant &entry : values) {
        const QVariantMap row = entry.toMap();
        const QString key = row.value(QStringLiteral("key")).toString();
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
    emit documentChanged();
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

void FluidSession::refreshDocument()
{
    if (!m_session) {
        return;
    }
    const int count = static_cast<int>(pf_session_component_count(m_session));
    const int selected = pf_session_selected(m_session);
    const bool canUndo = pf_session_can_undo(m_session) == 1;
    const bool canRedo = pf_session_can_redo(m_session) == 1;
    if (count == m_componentCount && selected == m_selected && canUndo == m_canUndo
        && canRedo == m_canRedo) {
        return;
    }
    m_componentCount = count;
    m_selected = selected;
    m_canUndo = canUndo;
    m_canRedo = canRedo;
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
    // One key, two kinds of thing: whichever is selected is the one Delete removes.
    if (pf_session_selected_pipe(m_session) >= 0) {
        pf_session_remove_selected_pipe(m_session);
        refreshDocument();
        emit changed();
        return;
    }
    removeComponent(m_selected);
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
