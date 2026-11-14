// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// A panel that folds to a rail. The header carries the title and the hide control; the
// content is clipped and the width is the only thing that moves, so the sheet takes the
// room in the same gesture. Hiding is view state: it never touches the document.
//
// The rail is one unit wide (24 px minimum, so the control stays a target) and keeps the
// same toggle in both states, which is also what keeps the keyboard focus from being lost
// when the panel closes.

import QtQuick
import PunchiFluid

Item {
    id: dock
    property FluidSession session
    property string title: ""
    property bool open: true
    property int panelWidth: 248
    property int railWidth: 24
    /// `true` when the panel sits on the right of the window.
    property bool sideRight: false
    /// The two accessible names of the control, so the title can be a sentence.
    property string hideName: qsTr("Ocultar el panel")
    property string showName: qsTr("Mostrar el panel")

    default property alias content: contentHolder.data

    signal toggleRequested()

    width: open ? panelWidth : railWidth
    Behavior on width {
        NumberAnimation {
            duration: dock.session ? dock.session.motionMs : 180
            easing.type: Easing.InOutQuad
        }
    }
    clip: true
    readonly property bool railVisible: width <= railWidth + 0.5

    Rectangle {
        anchors.fill: parent
        color: dock.railVisible ? dock.session.surfaceRaised : dock.session.background
    }

    // The edge against the sheet, so the panel is not a floating rectangle. It stays
    // while the panel is folded, which is what makes the rail read as a panel edge.
    Rectangle {
        width: 1
        height: parent.height
        anchors.right: dock.sideRight ? undefined : parent.right
        anchors.left: dock.sideRight ? parent.left : undefined
        color: dock.session.line
    }

    Item {
        id: panelBody
        anchors.fill: parent
        visible: !dock.railVisible
        clip: true

        Text {
            anchors.top: parent.top
            anchors.topMargin: 13
            anchors.left: parent.left
            anchors.leftMargin: dock.sideRight ? 32 : 14
            anchors.right: parent.right
            anchors.rightMargin: dock.sideRight ? 12 : 32
            text: dock.title
            color: dock.session.muted
            font.pixelSize: 12
            font.letterSpacing: 1
            elide: Text.ElideRight
        }

        Item {
            id: contentHolder
            anchors.top: parent.top
            anchors.topMargin: 36
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            clip: true
        }
    }

    Rectangle {
        anchors.fill: parent
        visible: dock.railVisible
        color: dock.session.surfaceRaised

        MouseArea {
            anchors.fill: parent
            cursorShape: Qt.PointingHandCursor
            onClicked: dock.toggleRequested()
        }
    }

    PanelToggle {
        id: toggle
        session: dock.session
        sideRight: dock.sideRight
        open: dock.open
        name: dock.open ? dock.hideName : dock.showName
        width: 24
        height: 24
        x: dock.sideRight ? 0 : dock.width - width
        y: dock.railVisible ? (dock.height - height) / 2 : 8
        Behavior on y {
            NumberAnimation { duration: dock.session ? dock.session.motionMs : 180 }
        }
        onActivated: dock.toggleRequested()
    }
}
