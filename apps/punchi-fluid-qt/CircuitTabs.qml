// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The circuits of the project, as a strip at the foot of the sheet: the same region as the
// drawing, not a fourth one. Switching is view state; adding and renaming are document
// edits, and renaming is undoable like every other edit.

import QtQuick
import PunchiFluid

Rectangle {
    id: tabs
    property FluidSession session
    property int editing: -1
    property string draft: ""

    implicitHeight: Math.max(28, 2 * session.unit - 8)
    color: session.surfaceRaised

    function beginRename(id, name) {
        tabs.editing = id;
        tabs.draft = name;
    }

    function commitRename() {
        if (tabs.editing >= 0 && tabs.draft.trim().length > 0) {
            tabs.session.renameCircuit(tabs.editing, tabs.draft.trim());
        }
        tabs.editing = -1;
        tabs.draft = "";
    }

    function cancelRename() {
        tabs.editing = -1;
        tabs.draft = "";
    }

    /// F2 renames the circuit in view, wherever the focus is.
    Shortcut {
        sequence: "F2"
        enabled: tabs.editing < 0
        onActivated: {
            const list = tabs.session.circuits;
            for (let i = 0; i < list.length; ++i) {
                if (list[i].id === tabs.session.activeCircuit) {
                    tabs.beginRename(list[i].id, list[i].name);
                    return;
                }
            }
        }
    }

    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        height: 1
        color: session.line
    }

    Row {
        anchors.fill: parent
        anchors.leftMargin: 4
        spacing: 1

        Repeater {
            model: tabs.session.circuits

            delegate: Item {
                id: tab
                required property var modelData
                readonly property bool active: modelData.id === tabs.session.activeCircuit

                width: Math.max(132, nameText.implicitWidth + 46)
                height: tabs.height

                Rectangle {
                    anchors.fill: parent
                    color: tab.active ? tabs.session.background : "transparent"
                }

                Image {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: 10
                    width: 16
                    height: 16
                    sourceSize.width: 16
                    sourceSize.height: 16
                    source: "image://theme/x-office-document?tone="
                            + (tab.active ? "foreground" : "muted")
                }

                Text {
                    id: nameText
                    visible: tabs.editing !== tab.modelData.id
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: 32
                    text: tab.modelData.name
                    color: tab.active ? tabs.session.foreground : tabs.session.muted
                    font.pixelSize: tabs.session.textBody
                    elide: Text.ElideRight
                }

                TextInput {
                    id: editor
                    visible: tabs.editing === tab.modelData.id
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: 32
                    width: parent.width - 40
                    text: visible ? tabs.draft : ""
                    color: tabs.session.foreground
                    font.pixelSize: tabs.session.textBody
                    selectByMouse: true
                    onTextEdited: tabs.draft = text
                    onAccepted: tabs.commitRename()
                    Keys.onEscapePressed: tabs.cancelRename()
                    onVisibleChanged: if (visible) {
                        forceActiveFocus();
                        selectAll();
                    }
                }

                Rectangle {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    height: 2
                    color: tab.active ? tabs.session.accent : "transparent"
                }

                MouseArea {
                    anchors.fill: parent
                    enabled: tabs.editing !== tab.modelData.id
                    cursorShape: Qt.PointingHandCursor
                    onClicked: tabs.session.selectCircuit(tab.modelData.id)
                    onDoubleClicked: tabs.beginRename(tab.modelData.id, tab.modelData.name)
                }
            }
        }

        Item {
            width: 30
            height: tabs.height
            Accessible.role: Accessible.Button
            Accessible.name: qsTr("Nuevo circuito")

            Rectangle {
                anchors.fill: parent
                anchors.margins: 4
                radius: session.radiusSmall
                color: plusArea.containsMouse ? session.surfaceRaised : "transparent"
            }
            Rectangle {
                anchors.centerIn: parent
                width: 12
                height: 2
                color: session.foreground
            }
            Rectangle {
                anchors.centerIn: parent
                width: 2
                height: 12
                color: session.foreground
            }
            MouseArea {
                id: plusArea
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: tabs.session.addCircuit()
            }
        }
    }
}
