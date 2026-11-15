// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The turn of the paper as one dropdown with a page icon, the way Word draws Orientación.
// The button names the turn in force; the list offers Vertical and Apaisado with their
// proportions, so the choice reads before the word does.

import QtQuick
import QtQuick.Controls
import PunchiFluid

Item {
    id: orientation

    property FluidSession session
    property bool showLabel: true
    /// The ribbon uses a paper symbol over the command label, not orientation tiles.
    property bool ribbonMode: false
    property real controlWidth: 120
    property real currentLabelWidth: 56

    readonly property var rows: [
        { label: qsTr("Vertical"), landscape: false },
        { label: qsTr("Apaisado"), landscape: true }
    ]

    function currentLabel() {
        return session.sheetLandscape ? qsTr("Apaisado") : qsTr("Vertical");
    }

    function choose(landscape) {
        session.setSheetLandscape(landscape);
        popup.close();
    }

    implicitWidth: ribbonMode ? ribbonButton.width : column.implicitWidth
    implicitHeight: ribbonMode ? ribbonButton.height : column.implicitHeight

    Column {
        id: column
        spacing: 2

        Text {
            visible: orientation.showLabel && !orientation.ribbonMode
            text: qsTr("Orientación")
            color: session.muted
            font.pixelSize: 10
        }

        Rectangle {
            id: button
            visible: !orientation.ribbonMode
            width: orientation.controlWidth
            height: 26
            radius: session.radiusSmall
            color: pointer.containsMouse || popup.opened ? session.surfaceRaised : "transparent"
            border.width: activeFocus ? 1 : 0
            border.color: session.accent
            activeFocusOnTab: true

            Accessible.role: Accessible.Button
            Accessible.name: qsTr("Orientación: %1").arg(currentLabel())
            Accessible.description: qsTr("abre la lista de orientaciones")
            Accessible.onPressAction: popup.opened ? popup.close() : popup.open()

            Row {
                anchors.centerIn: parent
                spacing: 6

                PaperPreview {
                    anchors.verticalCenter: parent.verticalCenter
                    session: orientation.session
                    paperWidthMm: orientation.session.sheetPaperWidthMm
                    paperHeightMm: orientation.session.sheetPaperHeightMm
                    landscape: orientation.session.sheetLandscape
                    maxWidth: 14
                    maxHeight: 17
                }

                Text {
                    width: orientation.currentLabelWidth
                    anchors.verticalCenter: parent.verticalCenter
                    text: orientation.currentLabel()
                    color: session.foreground
                    font.pixelSize: 10
                    elide: Text.ElideRight
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: "▾"
                    color: session.muted
                    font.pixelSize: 9
                }
            }

            MouseArea {
                id: pointer
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

    Rectangle {
        id: ribbonButton
        visible: orientation.ribbonMode
        width: 72
        height: 66
        radius: session.radiusSmall
        color: ribbonPointer.containsMouse || popup.opened || activeFocus
               ? session.surfaceRaised : "transparent"
        border.width: activeFocus ? 1 : 0
        border.color: session.accent
        activeFocusOnTab: true

        Accessible.role: Accessible.Button
        Accessible.name: qsTr("Orientación: %1").arg(currentLabel())
        Accessible.description: qsTr("abre las orientaciones de la hoja")
        Accessible.onPressAction: popup.opened ? popup.close() : popup.open()

        Column {
            anchors.centerIn: parent
            spacing: 2

            PaperPreview {
                anchors.horizontalCenter: parent.horizontalCenter
                session: orientation.session
                paperWidthMm: orientation.session.sheetPaperWidthMm
                paperHeightMm: orientation.session.sheetPaperHeightMm
                landscape: orientation.session.sheetLandscape
                filledPage: true
                maxWidth: 30
                maxHeight: 31
            }

            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                text: qsTr("Orientación")
                color: session.foreground
                font.pixelSize: 10
            }
        }

        Text {
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.rightMargin: 4
            anchors.bottomMargin: 3
            text: "▾"
            color: session.muted
            font.pixelSize: 8
        }

        MouseArea {
            id: ribbonPointer
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: popup.opened ? popup.close() : popup.open()
        }

        Keys.onSpacePressed: popup.opened ? popup.close() : popup.open()
        Keys.onReturnPressed: popup.opened ? popup.close() : popup.open()
        Keys.onEnterPressed: popup.opened ? popup.close() : popup.open()
    }

    Popup {
        id: popup
        parent: orientation.ribbonMode ? ribbonButton : button
        x: 0
        y: parent.height + 4
        width: 170
        height: list.contentHeight + 12
        padding: 6
        modal: false
        focus: true
        closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside

        background: Rectangle {
            color: session.surfaceRaised
            border.width: 1
            border.color: session.lineStrong
            radius: session.radiusSmall
        }

        contentItem: ListView {
            id: list
            clip: true
            model: orientation.rows
            boundsBehavior: Flickable.StopAtBounds
            keyNavigationEnabled: true
            currentIndex: session.sheetLandscape ? 1 : 0

            delegate: Rectangle {
                required property var modelData
                required property int index

                readonly property bool chosen: orientation.session.sheetLandscape
                                              === modelData.landscape

                width: ListView.view.width
                height: 34
                radius: session.radiusTip
                color: pointer.containsMouse || ListView.isCurrentItem ? session.surfaceRaised : "transparent"
                border.width: chosen ? 1 : 0
                border.color: session.accent

                Accessible.role: Accessible.RadioButton
                Accessible.name: modelData.label
                Accessible.checked: chosen
                Accessible.onPressAction: orientation.choose(modelData.landscape)

                Row {
                    anchors.fill: parent
                    anchors.leftMargin: 6
                    anchors.rightMargin: 6
                    spacing: 8

                    PaperPreview {
                        anchors.verticalCenter: parent.verticalCenter
                        session: orientation.session
                        paperWidthMm: orientation.session.sheetPaperWidthMm
                        paperHeightMm: orientation.session.sheetPaperHeightMm
                        landscape: modelData.landscape
                        maxWidth: 16
                        maxHeight: 22
                    }

                    Text {
                        width: 100
                        anchors.verticalCenter: parent.verticalCenter
                        text: modelData.label
                        color: session.foreground
                        font.pixelSize: 11
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: chosen
                        text: "✓"
                        color: session.accent
                        font.pixelSize: 11
                    }
                }

                MouseArea {
                    id: pointer
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: orientation.choose(modelData.landscape)
                }
            }

            Keys.onReturnPressed: orientation.choose(orientation.rows[currentIndex].landscape)
            Keys.onEnterPressed: orientation.choose(orientation.rows[currentIndex].landscape)
        }

        onOpened: list.forceActiveFocus()
    }
}
