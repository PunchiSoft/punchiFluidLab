// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The grid is a guide, not a mark on the sheet: its step is a view setting, shared by the
// ruling the canvas draws and the snap that places a component. Showing or hiding it and
// choosing its density live together, at the right of the page controls.

import QtQuick
import QtQuick.Controls
import PunchiFluid

Item {
    id: grid

    property FluidSession session

    /// Steps the drawing benefits from: fine enough for a detail, coarse enough to see
    /// the page, and all divisors of the 50 mm reference field in the frame.
    readonly property var steps: [5, 10, 20, 25]

    function stepLabel(mm) {
        return qsTr("%1 mm").arg(mm);
    }

    implicitWidth: column.implicitWidth
    implicitHeight: column.implicitHeight

    Column {
        id: column
        spacing: 2

        Text {
            text: qsTr("Cuadrícula")
            color: session.muted
            font.pixelSize: 10
        }

        Row {
            spacing: 4

            Rectangle {
                id: toggle
                width: 74
                height: 26
                radius: 4
                color: pointer.containsMouse || session.grid ? session.panel : "transparent"
                border.width: session.grid || activeFocus ? 1 : 0
                border.color: session.grid ? session.accent : session.muted
                activeFocusOnTab: true

                Accessible.role: Accessible.CheckBox
                Accessible.name: qsTr("Cuadrícula")
                Accessible.description: qsTr("muestra u oculta la guía; no se imprime")
                Accessible.checked: session.grid
                Accessible.onPressAction: session.toggleGrid()

                Text {
                    anchors.centerIn: parent
                    text: session.grid ? qsTr("Visible") : qsTr("Oculta")
                    color: session.foreground
                    font.pixelSize: 10
                }

                MouseArea {
                    id: pointer
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: session.toggleGrid()
                }

                Keys.onSpacePressed: session.toggleGrid()
                Keys.onReturnPressed: session.toggleGrid()
                Keys.onEnterPressed: session.toggleGrid()
            }

            Rectangle {
                id: stepButton
                width: 82
                height: 26
                radius: 4
                color: stepPointer.containsMouse || popup.opened ? session.panel : "transparent"
                border.width: activeFocus ? 1 : 0
                border.color: session.accent
                activeFocusOnTab: true

                Accessible.role: Accessible.Button
                Accessible.name: qsTr("Paso de cuadrícula: %1").arg(stepLabel(session.gridStepMm))
                Accessible.description: qsTr("elige la separación de la cuadrícula")
                Accessible.onPressAction: popup.opened ? popup.close() : popup.open()

                Row {
                    anchors.centerIn: parent
                    spacing: 4

                    Text {
                        text: stepLabel(session.gridStepMm)
                        color: session.foreground
                        font.pixelSize: 10
                    }

                    Text {
                        text: "▾"
                        color: session.muted
                        font.pixelSize: 9
                    }
                }

                MouseArea {
                    id: stepPointer
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: popup.opened ? popup.close() : popup.open()
                }

                Keys.onSpacePressed: popup.opened ? popup.close() : popup.open()
                Keys.onReturnPressed: popup.opened ? popup.close() : popup.open()
                Keys.onEnterPressed: popup.opened ? popup.close() : popup.open()
            }
        }

        Text {
            text: qsTr("Mayor cada %1 mm").arg(session.gridStepMm * 5)
            color: session.muted
            font.pixelSize: 9
        }
    }

    Popup {
        id: popup
        parent: stepButton
        x: 0
        y: stepButton.height + 4
        width: 120
        height: Math.min(180, list.contentHeight + 12)
        padding: 6
        modal: false
        focus: true
        closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside

        background: Rectangle {
            color: session.panel
            border.width: 1
            border.color: session.muted
            radius: 4
        }

        contentItem: ListView {
            id: list
            clip: true
            model: grid.steps
            boundsBehavior: Flickable.StopAtBounds
            keyNavigationEnabled: true
            currentIndex: {
                for (let i = 0; i < grid.steps.length; ++i) {
                    if (Math.abs(session.gridStepMm - grid.steps[i]) < 0.01) {
                        return i;
                    }
                }
                return 1;
            }

            delegate: Rectangle {
                required property var modelData
                required property int index

                readonly property bool chosen: Math.abs(session.gridStepMm - modelData) < 0.01

                width: ListView.view.width
                height: 28
                radius: 3
                color: pointer.containsMouse || ListView.isCurrentItem ? session.panel : "transparent"
                border.width: chosen ? 1 : 0
                border.color: session.accent

                Accessible.role: Accessible.RadioButton
                Accessible.name: grid.stepLabel(modelData)
                Accessible.checked: chosen
                Accessible.onPressAction: {
                    session.gridStepMm = modelData;
                    popup.close();
                }

                Text {
                    anchors.left: parent.left
                    anchors.leftMargin: 8
                    anchors.verticalCenter: parent.verticalCenter
                    text: grid.stepLabel(modelData)
                    color: session.foreground
                    font.pixelSize: 11
                }

                Text {
                    anchors.right: parent.right
                    anchors.rightMargin: 8
                    anchors.verticalCenter: parent.verticalCenter
                    visible: chosen
                    text: "✓"
                    color: session.accent
                    font.pixelSize: 11
                }

                MouseArea {
                    id: pointer
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        session.gridStepMm = modelData;
                        popup.close();
                    }
                }
            }

            Keys.onReturnPressed: {
                session.gridStepMm = grid.steps[currentIndex];
                popup.close();
            }
            Keys.onEnterPressed: {
                session.gridStepMm = grid.steps[currentIndex];
                popup.close();
            }
        }

        onOpened: list.forceActiveFocus()
    }
}
