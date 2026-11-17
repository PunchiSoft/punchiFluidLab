// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The circuits of the project, as a strip at the foot of the sheet: the same region as the
// drawing, not a fourth one. This first step is read-only — it shows which circuit is being
// worked on. Switching, adding and renaming come next, on the same strip.

import QtQuick
import PunchiFluid

Rectangle {
    id: tabs
    property FluidSession session
    implicitHeight: Math.max(28, 2 * session.unit - 8)
    color: session.surfaceRaised

    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        height: 1
        color: session.line
    }

    Row {
        anchors.fill: parent
        anchors.leftMargin: 4
        spacing: 1

        Repeater {
            model: tabs.session.circuits

            delegate: Item {
                id: tab
                required property var modelData
                readonly property bool active: modelData.id === tabs.session.activeCircuit

                width: Math.max(132, nameText.implicitWidth + 46)
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
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: 32
                    text: tab.modelData.name
                    color: tab.active ? tabs.session.foreground : tabs.session.muted
                    font.pixelSize: tabs.session.textBody
                    elide: Text.ElideRight
                }

                // The active tab is not colour alone: it carries an accent rule at the seam.
                Rectangle {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    height: 2
                    color: tab.active ? tabs.session.accent : "transparent"
                }
            }
        }
    }
}
