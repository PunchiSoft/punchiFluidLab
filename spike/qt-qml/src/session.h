// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <QColor>
#include <QObject>
#include <QPalette>
#include <QQmlEngine>
#include <QString>

#include "punchi_fluid.h"

/// One simulation, owned by the Rust core and seen from QML.
///
/// The properties are the whole of what the interface reads; everything else lives
/// behind the opaque `PfSession`. Colours are pushed the other way, from the desktop
/// palette into the core, so the drawing never picks one itself.
class FluidSession : public QObject
{
    Q_OBJECT
    QML_ELEMENT

    Q_PROPERTY(double pressureBar READ pressureBar NOTIFY changed)
    Q_PROPERTY(double positionMm READ positionMm NOTIFY changed)
    Q_PROPERTY(double velocityMmS READ velocityMmS NOTIFY changed)
    Q_PROPERTY(double timeS READ timeS NOTIFY changed)
    Q_PROPERTY(int valvePosition READ valvePosition NOTIFY changed)
    Q_PROPERTY(bool hasModel READ hasModel NOTIFY changed)
    Q_PROPERTY(QString lastError READ lastError NOTIFY changed)

    Q_PROPERTY(QColor background READ background NOTIFY changed)
    Q_PROPERTY(QColor foreground READ foreground NOTIFY changed)
    Q_PROPERTY(QColor accent READ accent NOTIFY changed)

public:
    explicit FluidSession(QObject *parent = nullptr);
    ~FluidSession() override;

    PfSession *handle() const { return m_session; }

    double pressureBar() const { return m_telemetry.pressure_pa / 1.0e5; }
    double positionMm() const { return m_telemetry.position_m * 1.0e3; }
    double velocityMmS() const { return m_telemetry.velocity_m_s * 1.0e3; }
    double timeS() const { return m_telemetry.time_s; }
    int valvePosition() const { return static_cast<int>(m_telemetry.valve_position); }
    bool hasModel() const { return m_telemetry.has_model == 1; }
    QString lastError() const { return m_error; }

    QColor background() const { return m_background; }
    QColor foreground() const { return m_foreground; }
    QColor accent() const { return m_accent; }

    /// Advances the model and republishes the telemetry.
    Q_INVOKABLE void advance(double dt);
    /// Moves the valve to its other position.
    Q_INVOKABLE void toggleValve();
    /// Reads the desktop palette into the core. Called on start and on theme change.
    Q_INVOKABLE void loadTheme();

signals:
    void changed();

private:
    void refresh();

    PfSession *m_session = nullptr;
    PfTelemetry m_telemetry {};
    QString m_error;
    QColor m_background;
    QColor m_foreground;
    QColor m_accent;
};
