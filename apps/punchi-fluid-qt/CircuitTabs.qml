// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The circuits of the project, as a strip at the foot of the sheet: the same region as the
// drawing, not a fourth one. Switching is view state; adding, renaming and closing are
// document edits, and each of them is one undo.
//
// The plus sits right after the last tab, as a browser does, and scrolls with the tabs: the
// place a hand looks for it is the end of the row, not the edge of the window.
//
// Inicio is the fixed control before the scroll: the way back is not a circuit, so it never
// enters the model, the file or the undo history, and it does not move with the tabs.

import QtQuick
import QtQuick.Controls
import PunchiFluid

Rectangle {
    id: tabs
    property FluidSession session
    property int editing: -1
    property string draft: ""

    readonly property bool canCloseCircuit: session.circuits.length > 1
    readonly property bool canAddCircuit: session.circuits.length < session.circuitLimit

    signal removeRequested(int id, string name, int components)
    signal homeRequested()

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

    /// Scrolls the strip so the circuit in view is not hidden behind the edge.
    function ensureActiveVisible() {
        let index = -1;
        const list = tabs.session.circuits;
        for (let i = 0; i < list.length; ++i) {
            if (list[i].id === tabs.session.activeCircuit) {
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

    Connections {
        target: tabs.session
        function onChanged() { Qt.callLater(tabs.ensureActiveVisible); }
    }

    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        height: 1
        color: session.line
    }

    // The way back to Inicio: a compact icon button, outside the Flickable so the circuit
    // tabs scroll under it. P-014 replaces `go-home` with the chosen icon family.
    Button {
        id: homeTab
        objectName: "homeTab"
        anchors.left: parent.left
        anchors.leftMargin: 4
        anchors.verticalCenter: parent.verticalCenter
        width: Math.max(2 * session.unit, 24)
        height: parent.height
        activeFocusOnTab: true
        padding: 0
        text: ""
        readonly property string accessibleLabel: qsTr("Inicio")
        readonly property string tooltipText: qsTr("Inicio")
        Accessible.role: Accessible.Button
        Accessible.name: homeTab.accessibleLabel
        Accessible.description: qsTr("Vuelve a Inicio")
        ToolTip.visible: homeTab.hovered
        ToolTip.delay: 600
        ToolTip.text: homeTab.tooltipText
        onClicked: tabs.homeRequested()

        background: Rectangle {
            radius: session.radiusSmall
            color: homeTab.down ? session.surfaceSunken
                   : homeTab.hovered ? session.surfaceRaised
                   : "transparent"
            border.width: homeTab.activeFocus ? 1 : 0
            border.color: session.accent
        }

        contentItem: Image {
            source: "image://theme/go-home?tone=foreground&v="
                    + session.themeRevision
            sourceSize.width: 16
            sourceSize.height: 16
            fillMode: Image.PreserveAspectFit
        }
    }

    // The seam between the way back and the project.
    Rectangle {
        anchors.left: homeTab.right
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: 1
        color: session.line
    }

    Flickable {
        id: tabScroll
        objectName: "tabScroll"
        anchors.left: homeTab.right
        anchors.leftMargin: 4
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        clip: true
        contentWidth: tabRow.implicitWidth
        contentHeight: height
        boundsBehavior: Flickable.StopAtBounds
        flickableDirection: Flickable.HorizontalFlick

        Row {
            id: tabRow
            height: tabScroll.height
            spacing: 1

            Repeater {
                id: tabRepeater
                objectName: "tabRepeater"
                model: tabs.session.circuits

                delegate: Item {
                    id: tab
                    required property var modelData
                    readonly property bool active: modelData.id === tabs.session.activeCircuit

                    objectName: "circuitTab_" + modelData.id
                    width: Math.min(200, Math.max(132, nameText.implicitWidth + 58))
                    height: tabRow.height
                    activeFocusOnTab: true
                    Accessible.role: Accessible.PageTab
                    Accessible.name: modelData.name
                    Accessible.description: modelData.components > 0
                        ? qsTr("Circuito con %1 componente(s)").arg(modelData.components)
                        : qsTr("Circuito vacío")

                    Keys.onSpacePressed: tabs.session.selectCircuit(tab.modelData.id)
                    Keys.onReturnPressed: tabs.session.selectCircuit(tab.modelData.id)
                    Keys.onEnterPressed: tabs.session.selectCircuit(tab.modelData.id)

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
                                + "&v=" + tabs.session.themeRevision
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

                    // Renaming happens in the tab itself: Enter saves, Esc cancels.
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

                    // The active tab is not colour alone: it carries an accent rule at the seam.
                    Rectangle {
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.bottom: parent.bottom
                        height: 2
                        color: tab.active ? tabs.session.accent : "transparent"
                    }

                    // Keyboard focus is a rectangle, never colour alone.
                    Rectangle {
                        anchors.fill: parent
                        color: "transparent"
                        border.width: tab.activeFocus ? 1 : 0
                        border.color: tabs.session.accent
                        z: 4
                    }

                    // Closing removes the circuit with its contents; the last tab has none.
                    Item {
                        id: closeButton
                        objectName: "circuitCloseButton_" + tab.modelData.id
                        visible: tabs.canCloseCircuit
                        width: visible ? 24 : 0
                        height: 24
                        anchors.right: parent.right
                        anchors.rightMargin: 2
                        anchors.verticalCenter: parent.verticalCenter
                        z: 3
                        Accessible.role: Accessible.Button
                        Accessible.name: qsTr("Cerrar %1").arg(tab.modelData.name)
                        Accessible.description: tab.modelData.components > 0
                            ? qsTr("Cierra el circuito y elimina sus componentes y conductos")
                            : qsTr("Cierra este circuito vacío")
                        // The tip says what the click costs, not only what it does: closing a
                        // circuit with components removes them, and the undo is the only way
                        // back.
                        ToolTip.visible: closeArea.containsMouse
                        ToolTip.delay: 600
                        ToolTip.text: tab.modelData.components > 0
                                      ? qsTr("Cerrar circuito · se eliminan sus componentes y conductos")
                                      : qsTr("Cerrar circuito")

                        Rectangle {
                            anchors.fill: parent
                            anchors.margins: 2
                            radius: tabs.session.radiusSmall
                            color: closeArea.containsMouse ? tabs.session.surfaceRaised
                                                           : "transparent"
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

            // The plus, right after the last tab and scrolling with them.
            Item {
                id: addButton
                objectName: "addCircuitButton"
                enabled: tabs.canAddCircuit
                width: 30
                height: tabRow.height
                activeFocusOnTab: true
                Accessible.role: Accessible.Button
                Accessible.name: tabs.canAddCircuit
                                 ? qsTr("Nuevo circuito")
                                 : qsTr("Límite de circuitos alcanzado")
                Accessible.description: qsTr("Máximo: %1 circuitos").arg(tabs.session.circuitLimit)
                ToolTip.visible: plusArea.containsMouse
                ToolTip.delay: 600
                ToolTip.text: tabs.canAddCircuit
                              ? qsTr("Añadir circuito · %1 de %2")
                                    .arg(tabs.session.circuits.length)
                                    .arg(tabs.session.circuitLimit)
                              : qsTr("Límite de %1 circuitos alcanzado")
                                    .arg(tabs.session.circuitLimit)

                Rectangle {
                    anchors.fill: parent
                    anchors.margins: 4
                    radius: tabs.session.radiusSmall
                    color: plusArea.containsMouse && addButton.enabled
                           ? tabs.session.surfaceRaised : "transparent"
                    border.width: addButton.activeFocus ? 1 : 0
                    border.color: tabs.session.accent
                }
                Rectangle {
                    anchors.centerIn: parent
                    width: 12
                    height: 2
                    color: addButton.enabled ? tabs.session.foreground : tabs.session.muted
                }
                Rectangle {
                    anchors.centerIn: parent
                    width: 2
                    height: 12
                    color: addButton.enabled ? tabs.session.foreground : tabs.session.muted
                }
                MouseArea {
                    id: plusArea
                    anchors.fill: parent
                    hoverEnabled: true
                    enabled: addButton.enabled
                    cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                    onClicked: tabs.session.addCircuit()
                }
                Keys.onSpacePressed: if (addButton.enabled) tabs.session.addCircuit()
                Keys.onReturnPressed: if (addButton.enabled) tabs.session.addCircuit()
                Keys.onEnterPressed: if (addButton.enabled) tabs.session.addCircuit()
            }
        }
    }
}
