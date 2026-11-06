// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#include "session.h"

#include <QGuiApplication>

namespace {

PfRgb toRgb(const QColor &colour)
{
    return PfRgb { static_cast<uint8_t>(colour.red()), static_cast<uint8_t>(colour.green()),
                   static_cast<uint8_t>(colour.blue()) };
}

/// `towards` blended into `from` by `amount`, for the two greys the drawing needs and
/// the palette does not name.
QColor blend(const QColor &from, const QColor &towards, double amount)
{
    const double t = qBound(0.0, amount, 1.0);
    return QColor(qRound(from.red() + (towards.red() - from.red()) * t),
                  qRound(from.green() + (towards.green() - from.green()) * t),
                  qRound(from.blue() + (towards.blue() - from.blue()) * t));
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
    loadTheme();
    refresh();
    connect(qGuiApp, &QGuiApplication::paletteChanged, this, &FluidSession::loadTheme);
}

FluidSession::~FluidSession()
{
    pf_session_free(m_session);
}

void FluidSession::advance(double dt)
{
    if (!m_session || pf_session_advance(m_session, dt) != PF_OK) {
        const char *message = m_session ? pf_session_last_error(m_session) : nullptr;
        if (message && *message) {
            m_error = QString::fromUtf8(message);
        }
        return;
    }
    refresh();
}

void FluidSession::toggleValve()
{
    if (!m_session) {
        return;
    }
    const int32_t code = pf_session_toggle_valve(m_session);
    const char *message = pf_session_last_error(m_session);
    m_error = (code == PF_OK || !message) ? QString() : QString::fromUtf8(message);
    refresh();
}

void FluidSession::refresh()
{
    if (!m_session) {
        return;
    }
    PfTelemetry telemetry {};
    if (pf_session_telemetry(m_session, &telemetry) != PF_OK) {
        return;
    }
    m_telemetry = telemetry;
    emit changed();
}

void FluidSession::loadTheme()
{
    if (!m_session) {
        return;
    }
    // The platform plugin has already merged kdeglobals and the colour-scheme portal
    // into the application palette, so reading it is reading the desktop.
    const QPalette palette = QGuiApplication::palette();
    const QColor background = palette.color(QPalette::Window);
    const QColor ink = palette.color(QPalette::WindowText);
    const QColor accent = palette.color(QPalette::Highlight);
    m_background = background;
    m_foreground = ink;
    m_accent = accent;

    PfInk theme {};
    theme.line = toRgb(ink);
    theme.text = toRgb(ink);
    theme.dim = toRgb(blend(ink, background, 0.45));
    theme.muted = toRgb(blend(ink, background, 0.62));
    theme.supply = toRgb(palette.color(QPalette::Link));
    theme.work = toRgb(accent);
    theme.accent = toRgb(blend(accent, background, 0.82));
    theme.background = toRgb(background);

    pf_session_set_ink(m_session, &theme);
    emit changed();
}
