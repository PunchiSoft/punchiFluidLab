// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#include "session.h"

#include <QEvent>

#include <algorithm>
#include <cmath>
#include <QGuiApplication>
#include <QVariantMap>

#include <string>

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

void FluidSession::setEditing(bool editing)
{
    if (m_editing == editing) {
        return;
    }
    m_editing = editing;
    if (editing) {
        // Entering the editor stops the animation: a model that keeps running while the
        // user edits is a model that moves under their hands.
        setRunning(false);
        m_steadyFrames = 0;
        if (m_settled) {
            m_settled = false;
            emit settledChanged();
        }
    }
    emit modeChanged();
    emit changed();
}

void FluidSession::inspectValve()
{
    if (!m_session) {
        return;
    }
    char buffer[1024] = {};
    if (pf_session_describe_valve(m_session, buffer, sizeof(buffer)) < 0) {
        return;
    }
    m_inspectorText = QString::fromUtf8(buffer);
    m_inspectorOpen = true;
    emit inspectorChanged();
}

void FluidSession::closeInspector()
{
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
    const int32_t code = m_valveSpringReturn ? pf_session_press_valve(m_session)
                                             : pf_session_toggle_valve(m_session);
    const char *message = pf_session_last_error(m_session);
    m_error = (code == PF_OK || !message) ? QString() : QString::fromUtf8(message);
    refresh();
}

void FluidSession::releaseValve()
{
    if (!m_session || !m_valveSpringReturn) {
        return;
    }
    const int32_t code = pf_session_release_valve(m_session);
    const char *message = pf_session_last_error(m_session);
    m_error = (code == PF_OK || !message) ? QString() : QString::fromUtf8(message);
    refresh();
}

void FluidSession::restart()
{
    if (!m_session) {
        return;
    }
    m_steadyFrames = 0;
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
