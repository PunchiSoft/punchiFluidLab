// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// One command of the ribbon: an icon and a label, the way a native ribbon draws its
// primary actions, or just the label in the small variant. Every instance is reachable
// with the keyboard and says what it does to assistive technology.

import QtQuick
import PunchiFluid

Item {
    id: button

    property FluidSession session
    property string label: ""
    property string iconName: ""
    property string hint: ""
    property bool large: true
    /// `foreground`, `muted` or `accent`; the colour itself comes from the theme.
    property string tone: "foreground"
    property bool available: true
    /// Menu rows read left to right; a button in a group is centred on its own width.
    property bool leftAligned: false
    /// The option this button represents is the one in force. The name still carries the
    /// meaning; this only makes it visible at a glance.
    property bool selected: false

    signal triggered()

    readonly property bool hovered: pointer.containsMouse

    implicitWidth: large
                   ? Math.max(58, labelText.implicitWidth + 14)
                   : smallRow.implicitWidth + 14
    implicitHeight: large ? 62 : 24
    opacity: available ? 1 : 0.4
    activeFocusOnTab: available

    Accessible.role: Accessible.Button
    Accessible.name: label
    Accessible.description: hint
    Accessible.onPressAction: if (button.available) button.triggered()

    Rectangle {
        anchors.fill: parent
        anchors.margins: 1
        radius: 4
        opacity: button.available ? 1 : 1
        color: button.selected || button.hovered || button.activeFocus ? session.panel
                                                                      : "transparent"
        border.width: button.activeFocus ? 1 : 0
        border.color: session.accent
    }

    Column {
        anchors.centerIn: parent
        spacing: 4
        visible: button.large

        Image {
            anchors.horizontalCenter: parent.horizontalCenter
            width: 24
            height: 24
            // An empty source draws nothing, so there is no need for a visibility binding
            // that could be evaluated before `source` is.
            sourceSize.width: 24
            sourceSize.height: 24
            source: button.iconName.length > 0
                    ? "image://theme/" + button.iconName + "?tone=" + button.tone
                    : ""
        }

        Text {
            id: labelText
            anchors.horizontalCenter: parent.horizontalCenter
            text: button.label
            color: session.foreground
            font.pixelSize: 11
            horizontalAlignment: Text.AlignHCenter
        }
    }

    Row {
        id: smallRow
        anchors.verticalCenter: parent.verticalCenter
        anchors.left: button.leftAligned ? parent.left : undefined
        anchors.leftMargin: button.leftAligned ? 8 : 0
        anchors.horizontalCenter: button.leftAligned ? undefined : parent.horizontalCenter
        spacing: 6
        visible: !button.large

        Image {
            anchors.verticalCenter: parent.verticalCenter
            width: 16
            height: 16
            sourceSize.width: 16
            sourceSize.height: 16
            source: button.iconName.length > 0
                    ? "image://theme/" + button.iconName + "?tone=" + button.tone
                    : ""
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: button.label
            color: session.foreground
            font.pixelSize: 11
        }
    }

    MouseArea {
        id: pointer
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        enabled: button.available
        onClicked: button.triggered()
    }

    Keys.onSpacePressed: if (available) triggered()
    Keys.onReturnPressed: if (available) triggered()
    Keys.onEnterPressed: if (available) triggered()

    // The hint is only shown on hover and only after a pause, so a row of buttons does not
    // flicker with tooltips while the pointer crosses it.
    Rectangle {
        id: tip
        visible: button.hint.length > 0 && button.hovered && tipTimer.triggered
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
            text: button.hint
            color: session.foreground
            font.pixelSize: 11
        }
    }

    Timer {
        id: tipTimer
        property bool triggered: false
        interval: 500
        running: button.hovered && button.hint.length > 0
        onRunningChanged: if (!running) triggered = false
        onTriggered: triggered = true
    }
}
