// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// A labelled group of the ribbon: its commands on top, its name underneath, and a hair
// line to the next group. The name underneath is what makes a ribbon readable — without
// it a row of icons is a guessing game.

import QtQuick
import PunchiFluid

Item {
    id: group

    property FluidSession session
    property string title: ""
    default property alias content: row.data

    implicitWidth: Math.max(row.implicitWidth, titleText.implicitWidth) + 22
    implicitHeight: 84

    Row {
        id: row
        anchors.top: parent.top
        anchors.topMargin: 5
        anchors.horizontalCenter: parent.horizontalCenter
        spacing: 2
    }

    Text {
        id: titleText
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        text: group.title
        color: session.muted
        font.pixelSize: 10
        font.letterSpacing: 0.4
    }

    Rectangle {
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.topMargin: 10
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 18
        width: 1
        color: session.panel
    }
}
