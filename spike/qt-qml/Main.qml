// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The spike's shell: a bar, the sheet and a status line. Every colour comes from the
// palette the session read, so there is no literal here either.

import QtQuick
import PunchiFluid

Window {
    id: window
    width: 1100
    height: 720
    visible: true
    title: qsTr("Punchi Fluid Lab — Qt Quick spike")
    color: session.background

    FluidSession {
        id: session
    }

    // 60 Hz of simulated time, in a timer instead of the paint loop, so a slow frame
    // does not change how far the model advances.
    Timer {
        interval: 16
        running: true
        repeat: true
        onTriggered: session.advance(0.016)
    }

    Column {
        anchors.fill: parent

        // ---- bar --------------------------------------------------------------
        Rectangle {
            id: bar
            width: parent.width
            height: 64
            color: session.background

            Rectangle {
                anchors.bottom: parent.bottom
                width: parent.width
                height: 1
                color: session.accent
            }

            Text {
                anchors.left: parent.left
                anchors.leftMargin: 16
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Válvula 3/2 y cilindro de simple efecto")
                color: session.foreground
                font.pixelSize: 18
            }

            Rectangle {
                id: actuate
                anchors.right: parent.right
                anchors.rightMargin: 16
                anchors.verticalCenter: parent.verticalCenter
                width: 190
                height: 40
                radius: 6
                color: mouse.containsMouse ? session.accent : "transparent"
                border.width: 1
                border.color: session.accent

                Text {
                    anchors.centerIn: parent
                    text: session.valvePosition === 0 ? qsTr("Activar la válvula")
                                                      : qsTr("Soltar la válvula")
                    color: session.foreground
                    font.pixelSize: 14
                }

                MouseArea {
                    id: mouse
                    anchors.fill: parent
                    hoverEnabled: true
                    onClicked: session.toggleValve()
                }
            }
        }

        // ---- sheet ------------------------------------------------------------
        Rectangle {
            width: parent.width
            height: parent.height - bar.height - status.height
            color: session.background

            SchematicItem {
                id: sheet
                anchors.fill: parent
                anchors.margins: 8
                session: session
            }
        }

        // ---- status -----------------------------------------------------------
        Rectangle {
            id: status
            width: parent.width
            height: 44
            color: session.background

            Rectangle {
                anchors.top: parent.top
                width: parent.width
                height: 1
                color: session.accent
            }

            Row {
                anchors.left: parent.left
                anchors.leftMargin: 16
                anchors.verticalCenter: parent.verticalCenter
                spacing: 28

                Text {
                    text: qsTr("Presión %1 bar").arg(session.pressureBar.toFixed(2))
                    color: session.foreground
                    font.pixelSize: 14
                }
                Text {
                    text: qsTr("Carrera %1 mm").arg(session.positionMm.toFixed(1))
                    color: session.foreground
                    font.pixelSize: 14
                }
                Text {
                    text: qsTr("Velocidad %1 mm/s").arg(session.velocityMmS.toFixed(1))
                    color: session.foreground
                    font.pixelSize: 14
                }
                Text {
                    text: qsTr("t %1 s").arg(session.timeS.toFixed(2))
                    color: session.foreground
                    font.pixelSize: 14
                }
            }

            Text {
                anchors.right: parent.right
                anchors.rightMargin: 16
                anchors.verticalCenter: parent.verticalCenter
                visible: session.lastError.length > 0
                text: session.lastError
                color: session.foreground
                font.pixelSize: 14
            }
        }
    }
}
