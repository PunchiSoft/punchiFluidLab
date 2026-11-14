// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// A command written as text: no icon and no frame, the label alone, the way the tabs
// themselves read. It is still a command — focusable, announced as a link and with the
// hint every other command carries — because a word on screen is a promise that pressing
// it does something.

import QtQuick
import PunchiFluid

Item {
    id: link

    property FluidSession session
    property string label: ""
    property string hint: ""
    /// `muted` in the tab row, where a link must not compete with the tabs; `foreground`
    /// inside a panel, where the link is the only thing to press. The accent stays on the
    /// focus ring: the measured accent is a 3:1 UI colour, not a 4.5:1 text colour.
    property string tone: "muted"
    /// A panel link is underlined at rest; a tab-row link only when the pointer or the
    /// keyboard is on it.
    property bool underlined: false

    signal triggered()

    readonly property bool hovered: pointer.containsMouse

    implicitWidth: labelText.implicitWidth + 10
    implicitHeight: 24
    activeFocusOnTab: true

    Accessible.role: Accessible.Link
    Accessible.name: label
    Accessible.description: hint
    Accessible.onPressAction: link.triggered()

    Text {
        id: labelText
        anchors.centerIn: parent
        text: link.label
        color: link.tone === "foreground"
               ? session.foreground
               : (link.hovered || link.activeFocus ? session.foreground : session.muted)
        font.pixelSize: 12
        font.underline: link.underlined || link.hovered || link.activeFocus
    }

    // Focus is a rectangle, never colour alone.
    Rectangle {
        anchors.fill: parent
        anchors.margins: 1
        radius: 3
        color: "transparent"
        border.width: link.activeFocus ? 1 : 0
        border.color: session.accent
    }

    MouseArea {
        id: pointer
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: link.triggered()
    }

    Keys.onSpacePressed: link.triggered()
    Keys.onReturnPressed: link.triggered()
    Keys.onEnterPressed: link.triggered()

    Rectangle {
        id: tip
        visible: link.hint.length > 0 && link.hovered && tipTimer.triggered
        z: 10
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.bottom
        anchors.topMargin: 2
        width: tipText.implicitWidth + 14
        height: tipText.implicitHeight + 8
        radius: 3
        color: session.panel
        border.width: 1
        border.color: session.muted

        Text {
            id: tipText
            anchors.centerIn: parent
            text: link.hint
            color: session.foreground
            font.pixelSize: 11
        }
    }

    Timer {
        id: tipTimer
        property bool triggered: false
        interval: 500
        running: link.hovered && link.hint.length > 0
        onRunningChanged: if (!running) triggered = false
        onTriggered: triggered = true
    }
}
