// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The circuits of the project, as a strip at the foot of the sheet: the same region as the
// drawing, not a fourth one. Switching is view state; adding and renaming are document
// edits, and renaming is undoable like every other edit.

import QtQuick
import QtQuick.Controls
import PunchiFluid

Rectangle {
    id: tabs
    property FluidSession session
    property int editing: -1
    property string draft: ""
    readonly property bool canCloseCircuit: session.circuits.length > 1
    signal removeRequested(int id, string name, int components)

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

    function ensureActiveVisible() {
        let index = -1;
        for (let i = 0; i < session.circuits.length; ++i) {
            if (session.circuits[i].id === session.activeCircuit) {
                index = i;
                break;
            }
        }
        if (index < 0) {
            return;
        }
        const item = tabRepeater.itemAt(index);
        if (!item) {
            return;
        }
        if (item.x < tabScroll.contentX) {
            tabScroll.contentX = item.x;
        } else if (item.x + item.width > tabScroll.contentX + tabScroll.width) {
            tabScroll.contentX = Math.min(tabScroll.contentWidth - tabScroll.width,
                                          item.x + item.width - tabScroll.width);
        }
    }

    Connections {
        target: tabs.session
        function onChanged() { Qt.callLater(tabs.ensureActiveVisible); }
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

    Flickable {
        id: tabScroll
        anchors.left: parent.left
        anchors.leftMargin: 4
        anchors.right: addButton.left
        anchors.rightMargin: 2
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        clip: true
        contentWidth: tabRow.implicitWidth
        contentHeight: height
        boundsBehavior: Flickable.StopAtBounds

        Row {
            id: tabRow
            height: parent.height
            spacing: 1

        Repeater {
            id: tabRepeater
            model: tabs.session.circuits

            delegate: Item {
                id: tab
                required property var modelData
                readonly property bool active: modelData.id === tabs.session.activeCircuit

                width: Math.min(200, Math.max(132, nameText.implicitWidth + 58))
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
                    anchors.right: closeButton.visible ? closeButton.left : parent.right
                    anchors.rightMargin: 8
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
                    anchors.right: closeButton.visible ? closeButton.left : parent.right
                    anchors.rightMargin: 8
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

                Item {
                    id: closeButton
                    objectName: "circuitCloseButton_" + tab.modelData.id
                    visible: tabs.canCloseCircuit
                    width: visible ? 24 : 0
                    height: 24
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    z: 3
                    Accessible.role: Accessible.Button
                    Accessible.name: qsTr("Cerrar %1").arg(tab.modelData.name)
                    Accessible.description: tab.modelData.components > 0
                        ? qsTr("Cierra el circuito y elimina sus componentes y conductos")
                        : qsTr("Cierra este circuito vacío")

                    Rectangle {
                        anchors.fill: parent
                        anchors.margins: 2
                        radius: tabs.session.radiusSmall
                        color: closeArea.containsMouse ? tabs.session.surfaceRaised : "transparent"
                    }
                    Text {
                        anchors.centerIn: parent
                        text: "×"
                        color: tabs.session.muted
                        font.pixelSize: tabs.session.textLabel
                    }
                    MouseArea {
                        id: closeArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: tabs.removeRequested(tab.modelData.id,
                                                       tab.modelData.name,
                                                       tab.modelData.components)
                    }
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
        }
    }

    Item {
            id: addButton
            objectName: "addCircuitButton"
            enabled: session.circuits.length < session.circuitLimit
            width: 30
            height: tabs.height
            anchors.right: parent.right
            anchors.top: parent.top
            Accessible.role: Accessible.Button
            Accessible.name: session.circuits.length >= session.circuitLimit
                             ? qsTr("Límite de circuitos alcanzado")
                             : qsTr("Nuevo circuito")
            Accessible.description: qsTr("Máximo: %1 circuitos").arg(session.circuitLimit)
            ToolTip.visible: plusArea.containsMouse
            ToolTip.delay: 600
            ToolTip.text: enabled
                            ? qsTr("Añadir circuito · %1 de %2")
                                  .arg(session.circuits.length).arg(session.circuitLimit)
                            : qsTr("Límite de %1 circuitos alcanzado").arg(session.circuitLimit)

            Rectangle {
                anchors.fill: parent
                anchors.margins: 4
                radius: session.radiusSmall
                color: plusArea.containsMouse && plusArea.enabled
                       ? session.surfaceRaised : "transparent"
            }
            Rectangle {
                anchors.centerIn: parent
                width: 12
                height: 2
                color: addButton.enabled ? session.foreground : session.muted
            }
            Rectangle {
                anchors.centerIn: parent
                width: 2
                height: 12
                color: addButton.enabled ? session.foreground : session.muted
            }
            MouseArea {
                id: plusArea
                anchors.fill: parent
                hoverEnabled: true
                enabled: tabs.session.circuits.length < tabs.session.circuitLimit
                cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                onClicked: tabs.session.addCircuit()
            }
        }
    }
