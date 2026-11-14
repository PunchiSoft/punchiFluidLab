// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Which way the paper is turned, as two choices rather than one toggle: the label says
// what the sheet will be, not what the button will do. The miniature is the same paper
// turned, so the choice reads before the word does.

import QtQuick
import PunchiFluid

Item {
    id: orientation

    property FluidSession session

    implicitWidth: column.implicitWidth
    implicitHeight: column.implicitHeight

    Column {
        id: column
        spacing: 2

        Text {
            text: qsTr("Orientación")
            color: session.muted
            font.pixelSize: 10
        }

        Row {
            spacing: 4

            Repeater {
                model: [
                    { label: qsTr("Vertical"), landscape: false },
                    { label: qsTr("Apaisado"), landscape: true }
                ]

                delegate: Rectangle {
                    required property var modelData

                    readonly property bool chosen: orientation.session.sheetLandscape
                                                  === modelData.landscape

                    width: 68
                    height: 46
                    radius: 4
                    color: pointer.containsMouse || chosen ? session.panel : "transparent"
                    border.width: chosen || activeFocus ? 1 : 0
                    border.color: chosen ? session.accent : session.muted
                    activeFocusOnTab: true

                    Accessible.role: Accessible.RadioButton
                    Accessible.name: modelData.label
                    Accessible.description: qsTr("la hoja %1")
                                            .arg(modelData.landscape ? qsTr("apaisada")
                                                                     : qsTr("vertical"))
                    Accessible.checked: chosen
                    Accessible.onPressAction: orientation.session.setSheetLandscape(
                                                  modelData.landscape)

                    Column {
                        anchors.centerIn: parent
                        spacing: 2

                        PaperPreview {
                            anchors.horizontalCenter: parent.horizontalCenter
                            session: orientation.session
                            paperWidthMm: orientation.session.sheetPaperWidthMm
                            paperHeightMm: orientation.session.sheetPaperHeightMm
                            landscape: modelData.landscape
                            maxWidth: 16
                            maxHeight: 20
                        }

                        Text {
                            text: modelData.label
                            color: session.foreground
                            font.pixelSize: 10
                        }
                    }

                    MouseArea {
                        id: pointer
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: orientation.session.setSheetLandscape(modelData.landscape)
                    }

                    Keys.onSpacePressed: orientation.session.setSheetLandscape(modelData.landscape)
                    Keys.onReturnPressed: orientation.session.setSheetLandscape(modelData.landscape)
                    Keys.onEnterPressed: orientation.session.setSheetLandscape(modelData.landscape)
                }
            }
        }
    }
}
