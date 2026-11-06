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

    color: session.background

    Rectangle {
        anchors.top: parent.top
        width: parent.width
        height: 1
        color: session.panel
    }

    Row {
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
    }

    Text {
        anchors.right: parent.right
        anchors.rightMargin: 16
        anchors.left: parent.left
        anchors.leftMargin: 520
        anchors.verticalCenter: parent.verticalCenter
        horizontalAlignment: Text.AlignRight
        elide: Text.ElideRight
        text: session.lastError.length > 0 ? session.lastError : status.hint
        color: session.muted
        font.pixelSize: 12
    }
}
