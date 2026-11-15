// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// One command of the view strip: a square button with the project's own icon, a tooltip
// and a name for assistive technology. The strip sits on the right edge of the drawing, so
// the tooltip opens to its left; the icon, the box and the ink all come from the session.

import QtQuick
import PunchiFluid

Item {
    id: button
    property FluidSession session
    property string iconName: ""
    /// The accessible name. An icon alone would leave the control unnamed.
    property string name: ""
    property string hint: ""
    property bool available: true
    /// `true` while the command is the one in force, so the state is not colour-only: the
    /// name carries the meaning and the surface shows it.
    property bool selected: false

    signal triggered()

    readonly property bool hovered: pointer.containsMouse
    readonly property bool pressed: pointer.pressed
    readonly property int side: 2 * session.unit

    implicitWidth: side
    implicitHeight: side
    opacity: available ? 1 : 0.4
    activeFocusOnTab: available

    Accessible.role: Accessible.Button
    Accessible.name: name
    Accessible.description: hint
    Accessible.onPressAction: if (button.available) button.triggered()

    Rectangle {
        anchors.fill: parent
        radius: session.radiusControl
        color: button.pressed ? session.surfaceSunken
               : button.selected ? session.accentSoft
               : button.hovered ? session.surfaceRaised
               : "transparent"
        border.width: button.activeFocus ? 1 : 0
        border.color: session.accent
    }

    Image {
        anchors.centerIn: parent
        width: session.unit
        height: session.unit
        sourceSize.width: session.unit
        sourceSize.height: session.unit
        source: button.iconName.length > 0
                ? "image://theme/" + button.iconName + "?tone=foreground"
                : ""
    }

    MouseArea {
        id: pointer
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        enabled: button.available
        onClicked: button.triggered()
    }

    Keys.onSpacePressed: if (button.available) button.triggered()
    Keys.onReturnPressed: if (button.available) button.triggered()
    Keys.onEnterPressed: if (button.available) button.triggered()

    // The hint shows on hover after a pause, to the left of the strip: the pointer is on
    // the edge of the window, so a tooltip below would fall outside it.
    Rectangle {
        id: tip
        visible: button.hint.length > 0 && button.hovered && tipTimer.triggered
        z: 10
        anchors.right: parent.left
        anchors.rightMargin: 6
        anchors.verticalCenter: parent.verticalCenter
        width: tipText.implicitWidth + 14
        height: tipText.implicitHeight + 8
        radius: session.radiusTip
        color: session.surfaceRaised
        border.width: 1
        border.color: session.lineStrong

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
        interval: 600
        running: button.hovered
        triggeredOnStart: false
    }
}
