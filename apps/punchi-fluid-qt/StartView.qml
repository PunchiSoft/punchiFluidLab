// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Inicio: the start view. It covers the sheet while no project is being worked on, with the
// navigation in the catalog's place: new, open, the recent projects and what a reader needs
// to begin. It is not a ribbon tab and not a fourth region (ADR 0031).

import QtQuick
import PunchiFluid

Rectangle {
    id: start
    property FluidSession session
    /// `false` while there is nothing to go back to.
    property bool canReturn: true

    signal newProject()
    signal openProject()
    signal openRecent(int slot)
    signal aboutRequested()
    signal licenceRequested()
    signal returnRequested()

    color: session.background

    Row {
        anchors.fill: parent

        // The left rail takes the catalog's place while Inicio is open.
        Rectangle {
            width: Math.round(session.unit * 10)
            height: parent.height
            color: session.panel

            Column {
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.top: parent.top
                anchors.margins: session.unit / 2
                spacing: session.unit / 3

                Text {
                    visible: start.canReturn
                    width: parent.width
                    text: qsTr("‹ Volver a la hoja")
                    color: session.muted
                    font.pixelSize: session.textBody
                    padding: session.unit / 2
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: start.returnRequested()
                    }
                }

                Text {
                    width: parent.width
                    text: qsTr("Iniciar")
                    color: session.foreground
                    font.pixelSize: session.textLabel
                    font.bold: true
                    padding: session.unit / 2
                }

                Repeater {
                    model: [
                        { label: qsTr("Nuevo proyecto"), action: "new" },
                        { label: qsTr("Abrir…"), action: "open" }
                    ]
                    delegate: Rectangle {
                        width: parent.width
                        height: session.unit * 1.6
                        radius: session.radiusControl
                        color: railArea.containsMouse ? session.surfaceRaised : "transparent"

                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            anchors.left: parent.left
                            anchors.leftMargin: session.unit / 2
                            text: modelData.label
                            color: session.foreground
                            font.pixelSize: session.textBody
                        }
                        MouseArea {
                            id: railArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: modelData.action === "new" ? start.newProject()
                                                                  : start.openProject()
                        }
                    }
                }

                Item { width: 1; height: session.unit / 2 }

                Text {
                    width: parent.width
                    text: qsTr("Aprender")
                    color: session.muted
                    font.pixelSize: session.textSmall
                    padding: session.unit / 3
                }
                Repeater {
                    model: [qsTr("Acerca de"), qsTr("Licencia")]
                    delegate: Text {
                        width: parent.width
                        text: modelData
                        color: session.foreground
                        font.pixelSize: session.textBody
                        padding: session.unit / 3
                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: index === 0 ? start.aboutRequested()
                                                   : start.licenceRequested()
                        }
                    }
                }
            }
        }

        // The page: a title, the two ways to start and the projects already known.
        Item {
            width: parent.width - Math.round(session.unit * 10)
            height: parent.height

            Column {
                anchors.fill: parent
                anchors.margins: session.unit * 1.2
                spacing: session.unit / 2

                Text {
                    text: qsTr("Inicio")
                    color: session.foreground
                    font.pixelSize: session.textLabel
                    font.bold: true
                }
                Text {
                    text: qsTr("Crea un proyecto, abre uno reciente o empieza desde el catálogo.")
                    color: session.muted
                    font.pixelSize: session.textBody
                }

                Row {
                    spacing: session.unit

                    Repeater {
                        model: [
                            { title: qsTr("Nuevo proyecto"), detail: qsTr("Una hoja vacía, A4 apaisado") },
                            { title: qsTr("Abrir…"), detail: qsTr("Un proyecto guardado") }
                        ]
                        delegate: Rectangle {
                            width: session.unit * 12
                            height: session.unit * 3.2
                            radius: session.radius
                            color: cardArea.containsMouse ? session.surfaceRaised : session.panel
                            border.color: session.line
                            border.width: 1

                            Column {
                                anchors.fill: parent
                                anchors.margins: session.unit / 2
                                spacing: session.unit / 4
                                Text {
                                    text: modelData.title
                                    color: session.foreground
                                    font.pixelSize: session.textLabel
                                }
                                Text {
                                    text: modelData.detail
                                    color: session.muted
                                    font.pixelSize: session.textSmall
                                    width: parent.width
                                    wrapMode: Text.WordWrap
                                }
                            }
                            MouseArea {
                                id: cardArea
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: index === 0 ? start.newProject() : start.openProject()
                            }
                        }
                    }
                }

                Text {
                    text: qsTr("Recientes")
                    color: session.foreground
                    font.pixelSize: session.textLabel
                    topPadding: session.unit / 2
                }
                Text {
                    visible: session.recent.length === 0
                    text: qsTr("Los proyectos que abras o guardes aparecerán aquí.")
                    color: session.muted
                    font.pixelSize: session.textSmall
                }
                Repeater {
                    model: start.session.recent
                    delegate: Rectangle {
                        width: parent.width
                        height: session.unit * 2
                        radius: session.radiusControl
                        color: recentArea.containsMouse ? session.surfaceRaised : "transparent"

                        Column {
                            anchors.verticalCenter: parent.verticalCenter
                            anchors.left: parent.left
                            anchors.leftMargin: session.unit / 2
                            spacing: 2
                            Text {
                                text: modelData.name
                                color: session.foreground
                                font.pixelSize: session.textBody
                            }
                            Text {
                                text: modelData.path
                                color: session.muted
                                font.pixelSize: session.textSmall
                                elide: Text.ElideMiddle
                                width: parent.width
                            }
                        }
                        MouseArea {
                            id: recentArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: start.openRecent(index)
                        }
                    }
                }
            }
        }
    }
}
