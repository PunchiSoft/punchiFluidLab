// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The ribbon: tabs of grouped actions. Only the actions that exist are here; a disabled
// button that does nothing is worse than a missing one, and everything on screen must be
// reachable with the keyboard.

import QtQuick
import PunchiFluid

Rectangle {
    id: ribbon
    property FluidSession session
    property int activeTab: 0

    color: session.background

    component ActionButton: Rectangle {
        id: button
        property string label: ""
        property string detail: ""
        property bool primary: false
        signal triggered()

        width: Math.max(170, caption.implicitWidth + 32)
        height: 52
        radius: 6
        color: pointer.containsMouse ? session.panel : "transparent"
        border.width: 1
        border.color: button.primary ? session.accent : session.muted

        Column {
            id: caption
            anchors.left: parent.left
            anchors.leftMargin: 12
            anchors.verticalCenter: parent.verticalCenter
            spacing: 2

            Text {
                text: button.label
                color: session.foreground
                font.pixelSize: 14
            }
            Text {
                text: button.detail
                color: session.muted
                font.pixelSize: 11
            }
        }

        MouseArea {
            id: pointer
            anchors.fill: parent
            hoverEnabled: true
            onClicked: button.triggered()
        }
    }

    Column {
        anchors.fill: parent

        Row {
            id: tabs
            x: 12
            height: 34
            spacing: 4

            Repeater {
                model: [qsTr("Inicio"), qsTr("Simulación"), qsTr("Vista")]

                delegate: Rectangle {
                    required property int index
                    required property string modelData

                    width: tabText.implicitWidth + 28
                    height: 30
                    radius: 5
                    color: ribbon.activeTab === index ? session.panel : "transparent"

                    Text {
                        id: tabText
                        anchors.centerIn: parent
                        text: modelData
                        color: ribbon.activeTab === index ? session.foreground : session.muted
                        font.pixelSize: 14
                    }

                    Rectangle {
                        anchors.bottom: parent.bottom
                        width: parent.width
                        height: 2
                        color: ribbon.activeTab === index ? session.accent : "transparent"
                    }

                    MouseArea {
                        anchors.fill: parent
                        onClicked: ribbon.activeTab = index
                    }
                }
            }
        }

        Row {
            id: actions
            x: 12
            y: 40
            spacing: 10

            ActionButton {
                label: session.valvePosition === 0 ? qsTr("Accionar la válvula")
                                                   : qsTr("Soltar la válvula")
                detail: qsTr("Espacio")
                primary: true
                onTriggered: session.toggleValve()
            }

            ActionButton {
                label: qsTr("Reiniciar la simulación")
                detail: qsTr("t = 0, misma presión de red")
                onTriggered: session.restart()
            }
        }
    }

    Text {
        visible: ribbon.activeTab === 2
        anchors.right: parent.right
        anchors.rightMargin: 16
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 8
        text: qsTr("Contraste: %1").arg(session.contrastSummary)
        color: session.muted
        font.pixelSize: 12
    }

    Rectangle {
        anchors.bottom: parent.bottom
        width: parent.width
        height: 1
        color: session.accent
    }
}
