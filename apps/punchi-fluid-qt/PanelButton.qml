// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// A button for a popup panel. It is the same contract as a ribbon command — a label,
// keyboard reach, a name for the accessibility layer and the hand on hover — but its
// surface is the panel, so the hover plane is the sheet background, not the panel's own
// colour. A large ribbon button would be invisible here.

import QtQuick
import PunchiFluid

Item {
    id: button

    property FluidSession session
    property string label: ""
    /// The action the panel exists for; the close button is not one.
    property bool primary: false

    signal triggered()

    implicitWidth: labelText.implicitWidth + 24
    implicitHeight: 26
    activeFocusOnTab: true

    Accessible.role: Accessible.Button
    Accessible.name: label
    Accessible.onPressAction: button.triggered()

    Rectangle {
        anchors.fill: parent
        radius: 4
        color: pointer.containsMouse || button.activeFocus ? session.background : "transparent"
        border.width: 1
        border.color: button.primary ? session.accent : session.muted
    }

    Text {
        id: labelText
        anchors.centerIn: parent
        text: button.label
        color: session.foreground
        font.pixelSize: 11
    }

    MouseArea {
        id: pointer
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: button.triggered()
    }

    Keys.onReturnPressed: button.triggered()
    Keys.onEnterPressed: button.triggered()
    Keys.onSpacePressed: button.triggered()
}
