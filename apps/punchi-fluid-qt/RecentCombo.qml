// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The recent documents as one dropdown instead of a row of buttons. The button names the
// newest file (or Ninguno), and the list opens the one the user picks; the path is shown
// under the name so two files called the same are still distinguishable.

import QtQuick
import QtQuick.Controls
import PunchiFluid

Item {
    id: recent

    property FluidSession session
    property bool showLabel: true

    readonly property var rows: session.recent.length > 0
                                ? session.recent
                                : [{ name: qsTr("Ninguno"), path: "", empty: true }]

    function currentLabel() {
        if (session.recent.length === 0) {
            return qsTr("Ninguno");
        }
        const first = session.recent[0];
        return first && first.name ? first.name : qsTr("Ninguno");
    }

    implicitWidth: column.implicitWidth
    implicitHeight: column.implicitHeight

    Column {
        id: column
        spacing: 2

        Text {
            visible: recent.showLabel
            text: qsTr("Recientes")
            color: session.muted
            font.pixelSize: 10
        }

        Rectangle {
            id: button
            width: 100
            height: 26
            radius: 4
            color: pointer.containsMouse || popup.opened ? session.panel : "transparent"
            border.width: activeFocus ? 1 : 0
            border.color: session.accent
            activeFocusOnTab: true

            Accessible.role: Accessible.Button
            Accessible.name: qsTr("Recientes: %1").arg(currentLabel())
            Accessible.description: qsTr("abre la lista de documentos recientes")
            Accessible.onPressAction: popup.opened ? popup.close() : popup.open()

            Row {
                anchors.fill: parent
                anchors.leftMargin: 8
                anchors.rightMargin: 8
                spacing: 4

                Text {
                    width: parent.width - 14
                    anchors.verticalCenter: parent.verticalCenter
                    text: currentLabel()
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

    Popup {
        id: popup
        parent: button
        x: 0
        y: button.height + 4
        width: 280
        height: Math.min(260, list.contentHeight + 12)
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
            model: recent.rows
            boundsBehavior: Flickable.StopAtBounds
            keyNavigationEnabled: true

            delegate: Rectangle {
                id: row
                required property var modelData
                required property int index

                width: ListView.view.width
                height: modelData.empty === true ? 28 : 38
                radius: 3
                color: pointer.containsMouse && modelData.empty !== true ? session.panel : "transparent"

                Accessible.role: Accessible.Button
                Accessible.name: modelData.empty === true
                                 ? qsTr("Ninguno")
                                 : qsTr("%1, %2").arg(modelData.name).arg(modelData.path)
                Accessible.description: modelData.empty === true
                                        ? qsTr("no hay documentos recientes")
                                        : qsTr("abre este documento")
                Accessible.onPressAction: if (modelData.empty !== true) {
                                              recent.session.openRecent(index);
                                              popup.close();
                                          }

                Column {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.leftMargin: 6
                    anchors.rightMargin: 6
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 1

                    Text {
                        width: parent.width
                        text: row.modelData.name
                        color: row.modelData.empty === true ? session.muted : session.foreground
                        font.pixelSize: 11
                        elide: Text.ElideRight
                    }

                    Text {
                        width: parent.width
                        visible: row.modelData.empty !== true
                        text: row.modelData.path
                        color: session.muted
                        font.pixelSize: 9
                        elide: Text.ElideMiddle
                    }
                }

                MouseArea {
                    id: pointer
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: row.modelData.empty === true ? Qt.ArrowCursor : Qt.PointingHandCursor
                    enabled: row.modelData.empty !== true
                    onClicked: {
                        recent.session.openRecent(index);
                        popup.close();
                    }
                }
            }

            Keys.onReturnPressed: {
                const entry = recent.rows[currentIndex];
                if (entry && entry.empty !== true) {
                    recent.session.openRecent(currentIndex);
                    popup.close();
                }
            }
            Keys.onEnterPressed: {
                const entry = recent.rows[currentIndex];
                if (entry && entry.empty !== true) {
                    recent.session.openRecent(currentIndex);
                    popup.close();
                }
            }
        }

        onOpened: list.forceActiveFocus()
    }
}
