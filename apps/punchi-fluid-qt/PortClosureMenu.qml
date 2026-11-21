// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The port closure menu: a plug or a silencer belongs to the port, so the sheet opens this
// on a repeated press over one. It is not modal, it closes with Esc or a click outside, and
// a port that still carries a hose refuses a closure instead of deleting the hose. The
// closure definitions come from the session's catalog; the menu only names them.

import QtQuick
import QtQuick.Controls
import PunchiFluid

Popup {
    id: menu

    property FluidSession session
    /// The placement the menu was opened on, and the port it is showing.
    property int componentIndex: -1
    property string port: ""
    /// The point in the sheet item, so the menu opens beside the port it belongs to.
    property real anchorX: 0
    property real anchorY: 0
    /// The row the arrow keys are on; 0 is `Sin cierre` and the rest follow `choices`.
    property int focusIndex: 0

    x: Math.max(8, Math.min(anchorX, parent ? parent.width - width - 8 : anchorX))
    y: Math.max(8, Math.min(anchorY, parent ? parent.height - height - 8 : anchorY))
    width: 240
    padding: 8
    modal: false
    focus: true
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside

    readonly property var ports: componentIndex >= 0 ? session.portClosurePorts(componentIndex) : []
    readonly property string current: (componentIndex >= 0 && port.length > 0)
                                      ? session.portClosure(componentIndex, port) : ""
    readonly property bool hasPipe: (componentIndex >= 0 && port.length > 0)
                                    ? session.portHasPipe(componentIndex, port) : false
    readonly property string title: (componentIndex >= 0 && port.length > 0)
                                    ? session.portLabel(componentIndex, port) : ""

    /// The closure choices the role accepts: a plug on any pneumatic port and a silencer
    /// only on an exhaust. A mechanical or an electrical port returns none.
    readonly property var closureChoices: {
        const out = [];
        if (componentIndex < 0 || port.length === 0) {
            return out;
        }
        const list = session.portClosures(componentIndex, port);
        for (let index = 0; index < list.length; ++index) {
            if (list[index].available) {
                out.push(list[index]);
            }
        }
        return out;
    }

    /// The rows of the list, `Sin cierre` first, with everything a delegate prints.
    readonly property var rows: {
        const out = [{
            id: "",
            label: qsTr("Sin cierre"),
            enabled: true,
            current: menu.current.length === 0
        }];
        for (let index = 0; index < closureChoices.length; ++index) {
            const choice = closureChoices[index];
            out.push({
                id: choice.id,
                label: choice.plug ? qsTr("Tapón") : qsTr("Silenciador"),
                enabled: !menu.hasPipe,
                current: menu.current === choice.id
            });
        }
        return out;
    }

    function openAt(index, portId, scenePoint) {
        menu.componentIndex = index;
        menu.port = portId;
        if (scenePoint) {
            menu.anchorX = scenePoint.x + 12;
            menu.anchorY = scenePoint.y + 12;
        }
        menu.resetFocus();
        menu.open();
    }

    function resetFocus() {
        menu.focusIndex = 0;
        for (let index = 0; index < menu.rows.length; ++index) {
            if (menu.rows[index].current) {
                menu.focusIndex = index;
                return;
            }
        }
    }

    function choose(id) {
        if (!menu.opened) {
            return;
        }
        // Picking the closure already fitted is not a document change: it only closes.
        if (id === menu.current) {
            menu.close();
            return;
        }
        if (menu.hasPipe) {
            // The rows are disabled; this guards a stray Enter on them.
            return;
        }
        menu.session.setPortClosure(menu.componentIndex, menu.port, id);
        menu.close();
    }

    function activateFocus() {
        if (menu.focusIndex < 0 || menu.focusIndex >= menu.rows.length) {
            return;
        }
        const row = menu.rows[menu.focusIndex];
        if (row.enabled) {
            menu.choose(row.id);
        }
    }

    function moveFocus(step) {
        if (menu.rows.length === 0) {
            return;
        }
        let next = menu.focusIndex;
        for (let count = 0; count < menu.rows.length; ++count) {
            next = (next + step + menu.rows.length) % menu.rows.length;
            if (menu.rows[next].enabled) {
                menu.focusIndex = next;
                return;
            }
        }
    }

    onOpened: body.forceActiveFocus()
    onPortChanged: resetFocus()

    background: Rectangle {
        color: menu.session.surfaceRaised
        border.width: 1
        border.color: menu.session.lineStrong
        radius: menu.session.radiusSmall
    }

    contentItem: Column {
        id: body
        focus: true
        spacing: 6

        Accessible.role: Accessible.Dialog
        Accessible.name: qsTr("Cierre de puerto: %1").arg(menu.title)
        Accessible.description: qsTr("elige un cierre para el puerto del componente seleccionado")

        Keys.onUpPressed: menu.moveFocus(-1)
        Keys.onDownPressed: menu.moveFocus(1)
        Keys.onReturnPressed: menu.activateFocus()
        Keys.onEnterPressed: menu.activateFocus()
        Keys.onEscapePressed: menu.close()

        Text {
            width: parent.width
            text: qsTr("Cierre de puerto")
            color: menu.session.muted
            font.pixelSize: menu.session.textSmall
        }

        Text {
            width: parent.width
            text: menu.title
            color: menu.session.foreground
            font.pixelSize: menu.session.textBody
            font.bold: true
            elide: Text.ElideRight
        }

        // One menu serves every closable port of the component: the valve's exhausts are
        // all here, so a second exhaust is one click away instead of a second menu.
        Repeater {
            model: menu.ports

            delegate: Rectangle {
                required property var modelData

                width: body.width
                height: 24
                radius: menu.session.radiusTip
                color: modelData.id === menu.port ? menu.session.accentSoft : "transparent"
                border.width: modelData.id === menu.port ? 1 : 0
                border.color: menu.session.accent

                Accessible.role: Accessible.Button
                Accessible.name: modelData.label
                                    + (modelData.hasPipe ? qsTr(", con tubo") : "")
                Accessible.description: qsTr("muestra los cierres de este puerto")

                Text {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.leftMargin: 6
                    anchors.rightMargin: 6
                    text: modelData.label
                          + (modelData.hasPipe ? "  ·  " + qsTr("con tubo") : "")
                    color: menu.session.foreground
                    font.pixelSize: menu.session.textBody
                    elide: Text.ElideRight
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: menu.port = modelData.id
                }
            }
        }

        Rectangle {
            width: parent.width
            height: 1
            color: menu.session.line
        }

        Repeater {
            model: menu.rows

            delegate: Rectangle {
                id: row

                required property var modelData
                required property int index

                width: body.width
                height: 26
                radius: menu.session.radiusTip
                color: menu.focusIndex === index && body.activeFocus
                       ? menu.session.surfaceSunken
                       : (modelData.current ? menu.session.panel : "transparent")
                border.width: menu.focusIndex === index && body.activeFocus ? 1 : 0
                border.color: menu.session.accent
                opacity: modelData.enabled ? 1 : 0.45

                Accessible.role: Accessible.Button
                Accessible.name: modelData.label
                                    + (modelData.current ? qsTr(", actual") : "")
                Accessible.description: modelData.enabled
                                        ? ""
                                        : qsTr("quita el tubo del puerto antes de poner un cierre")
                Accessible.onPressAction: {
                    if (modelData.enabled) {
                        menu.choose(modelData.id);
                    }
                }

                Text {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.leftMargin: 6
                    anchors.rightMargin: 6
                    // The current row says so in words and in a check: a mark that only a
                    // colour carries is not a mark.
                    text: (modelData.current ? "✓ " : "") + modelData.label
                          + (modelData.current ? "  ·  " + qsTr("actual") : "")
                    color: menu.session.foreground
                    font.pixelSize: menu.session.textBody
                    elide: Text.ElideRight
                }

                MouseArea {
                    anchors.fill: parent
                    enabled: modelData.enabled
                    cursorShape: enabled ? Qt.PointingHandCursor : Qt.ForbiddenCursor
                    onClicked: menu.choose(modelData.id)
                }
            }
        }

        Text {
            width: parent.width
            visible: menu.hasPipe
            text: qsTr("Quita el tubo del puerto antes de poner un cierre.")
            color: menu.session.muted
            font.pixelSize: menu.session.textSmall
            wrapMode: Text.WordWrap
        }
    }
}
