// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <QColor>
#include <QObject>
#include <QPalette>
#include <QQmlEngine>
#include <QString>
#include <QVariantList>

#include "punchi_fluid.h"
#include "theme.h"

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

    Q_PROPERTY(bool running READ running WRITE setRunning NOTIFY runningChanged)
    Q_PROPERTY(double speed READ speed WRITE setSpeed NOTIFY speedChanged)

    Q_PROPERTY(QVariantList catalog READ catalog NOTIFY catalogChanged)
    Q_PROPERTY(QString contrastSummary READ contrastSummary NOTIFY changed)
    Q_PROPERTY(bool themePasses READ themePasses NOTIFY changed)
    Q_PROPERTY(bool settled READ settled NOTIFY settledChanged)

    Q_PROPERTY(QColor background READ background NOTIFY changed)
    Q_PROPERTY(QColor panel READ panel NOTIFY changed)
    Q_PROPERTY(QColor foreground READ foreground NOTIFY changed)
    Q_PROPERTY(QColor muted READ muted NOTIFY changed)
    Q_PROPERTY(QColor accent READ accent NOTIFY changed)

public:
    explicit FluidSession(QObject *parent = nullptr);
    ~FluidSession() override;

    PfSession *handle() const { return m_session; }
    const ThemeTokens &tokens() const { return m_tokens; }

    double pressureBar() const { return m_telemetry.pressure_pa / 1.0e5; }
    double positionMm() const { return m_telemetry.position_m * 1.0e3; }
    double velocityMmS() const { return m_telemetry.velocity_m_s * 1.0e3; }
    double timeS() const { return m_telemetry.time_s; }
    int valvePosition() const { return static_cast<int>(m_telemetry.valve_position); }
    bool hasModel() const { return m_telemetry.has_model == 1; }
    QString lastError() const { return m_error; }

    /// `true` while the transport is playing. Pausing stops the sheet loop without
    /// touching the model, so resuming continues from the same instant.
    bool running() const { return m_running; }
    void setRunning(bool running);
    Q_INVOKABLE void toggleRunning();

    /// Simulated seconds per real second. Clamped to `0.05 ..= 8`.
    double speed() const { return m_speed; }
    void setSpeed(double speed);

    /// `true` when the last frames left the state unchanged: the sheet loop can stop.
    ///
    /// Without this the window repaints a CPU-rasterised item sixty times a second
    /// for ever, and on a 144 Hz panel the compositor then recomposites every frame —
    /// which is how an idle simulator ends up looking like a busy GPU.
    bool settled() const { return m_settled; }

    QVariantList catalog() const { return m_catalog; }
    QString contrastSummary() const { return describe(m_contrast); }
    /// `true` when every pairing meets the criterion of `docs/ui-ux.md`.
    [[nodiscard]] bool themePasses() const { return m_contrast.passes(); }

    QColor background() const { return m_tokens.background; }
    QColor panel() const { return m_tokens.panel; }
    QColor foreground() const { return m_tokens.ink; }
    QColor muted() const { return m_tokens.inkMuted; }
    QColor accent() const { return m_tokens.accent; }

    /// Advances the model and republishes the telemetry.
    Q_INVOKABLE void advance(double dt);
    /// Moves the valve to its other position.
    Q_INVOKABLE void toggleValve();
    /// Back to t = 0.
    Q_INVOKABLE void restart();
    /// Reads the desktop palette into the core. Called on start and on theme change.
    Q_INVOKABLE void loadTheme();
    /// Uses `palette` instead of the application palette, for a forced light or dark run.
    void applyPalette(const QPalette &palette);

    /// Re-reads the palette when the desktop changes it, never per frame.
    bool event(QEvent *event) override;

signals:
    void changed();
    void catalogChanged();
    void settledChanged();
    void runningChanged();
    void speedChanged();

private:
    void refresh();
    void readCatalog();

    PfSession *m_session = nullptr;
    PfTelemetry m_telemetry {};
    QString m_error;
    QVariantList m_catalog;
    ThemeTokens m_tokens;
    Contrast m_contrast;
    bool m_settled = false;
    bool m_running = true;
    double m_speed = 1.0;
    int m_steadyFrames = 0;
};
