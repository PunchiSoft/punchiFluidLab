// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The control that folds a panel: two panes, the one that collapses filled with the soft
// accent. The desktop icon themes have no panel glyph, so this one is drawn here; it is
// chrome, never a component symbol.

import QtQuick
import PunchiFluid

Item {
    id: toggle
    property FluidSession session
    /// `true` when the panel lives on the right of the window, so its glyph mirrors.
    property bool sideRight: false
    property bool open: true
    property string name: ""

    signal activated()

    implicitWidth: 24
    implicitHeight: 24
    activeFocusOnTab: true

    Accessible.role: Accessible.Button
    Accessible.name: toggle.name
    Accessible.description: toggle.open ? qsTr("ocultar el panel")
                                        : qsTr("mostrar el panel")
    Accessible.onPressAction: toggle.activated()

    Rectangle {
        anchors.fill: parent
        radius: session.radiusSmall
        color: toggle.activeFocus || pointer.containsMouse ? toggle.session.panel : "transparent"
        border.width: toggle.activeFocus ? 1 : 0
        border.color: toggle.session.accent
    }

    // The outer frame and the pane that folds, on the side the panel occupies.
    Rectangle {
        x: 4
        y: 6
        width: 16
        height: 12
        radius: 2
        color: "transparent"
        border.width: 1
        border.color: toggle.session.foreground
    }

    Rectangle {
        x: toggle.sideRight ? 13 : 5
        y: 7
        width: 6
        height: 10
        color: toggle.session.accentSoft
        border.width: 1
        border.color: toggle.session.foreground
    }

    MouseArea {
        id: pointer
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: toggle.activated()
    }

    Keys.onSpacePressed: toggle.activated()
    Keys.onReturnPressed: toggle.activated()
    Keys.onEnterPressed: toggle.activated()
}
