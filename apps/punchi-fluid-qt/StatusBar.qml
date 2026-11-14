// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The status bar: the live readouts and one hint. State is never colour-only, so every
// reading is a number with its unit, not a coloured dot.

import QtQuick
import PunchiFluid

Rectangle {
    id: status
    property FluidSession session
    property string hint: ""
    /// Shown by the button below and obeyed by `SoundFeedback`; the window owns it.
    property bool muted: false

    signal muteToggled()

    color: session.background

    Rectangle {
        anchors.top: parent.top
        width: parent.width
        height: 1
        color: session.panel
    }

    Row {
        id: readings
        x: 16
        anchors.verticalCenter: parent.verticalCenter
        spacing: 26

        Text {
            text: qsTr("Presión %1 bar").arg(session.pressureBar.toFixed(2))
            color: session.foreground
            font.pixelSize: 13
        }
        Text {
            text: qsTr("Carrera %1 mm").arg(session.positionMm.toFixed(1))
            color: session.foreground
            font.pixelSize: 13
        }
        Text {
            text: qsTr("Velocidad %1 mm/s").arg(session.velocityMmS.toFixed(1))
            color: session.foreground
            font.pixelSize: 13
        }
        Text {
            text: qsTr("t %1 s").arg(session.timeS.toFixed(2))
            color: session.muted
            font.pixelSize: 13
        }
        // The magnification is a number, not a slider: a reader who is about to place a
        // symbol needs to know whether the sheet is at 1:1 before trusting the grid.
        Text {
            text: qsTr("Zoom %1 %").arg(Math.round(session.zoom * 100))
            color: session.muted
            font.pixelSize: 13
        }
        // Which paper the drawing is on, and its size in millimetres. Without it the sheet
        // is an unnamed rectangle and the frame has no measurement to trust.
        Text {
            text: qsTr("%1 · %2 × %3 mm")
                      .arg(session.sheetPaperName)
                      .arg(Math.round(session.sheetWidthMm))
                      .arg(Math.round(session.sheetHeightMm))
            color: session.muted
            font.pixelSize: 13
        }
    }

    Text {
        anchors.right: soundToggle.left
        anchors.rightMargin: 12
        // After the readings, so a new one never lands on top of the hint.
        anchors.left: readings.right
        anchors.leftMargin: 16
        anchors.verticalCenter: parent.verticalCenter
        horizontalAlignment: Text.AlignRight
        elide: Text.ElideRight
        text: session.lastError.length > 0 ? session.lastError : status.hint
        color: session.muted
        font.pixelSize: 12
    }

    // Sound is a command like any other: reachable by key, named for assistive technology,
    // and never the only signal that something happened.
    RibbonButton {
        id: soundToggle
        anchors.right: parent.right
        anchors.rightMargin: 12
        anchors.verticalCenter: parent.verticalCenter
        session: status.session
        large: false
        label: status.muted ? qsTr("Silencio") : qsTr("Sonido")
        iconName: status.muted ? "audio-volume-muted" : "audio-volume-high"
        tone: status.muted ? "muted" : "foreground"
        hint: status.muted ? qsTr("Ctrl+M · activar el sonido")
                           : qsTr("Ctrl+M · silenciar la ventana")
        onTriggered: status.muteToggled()
    }
}
