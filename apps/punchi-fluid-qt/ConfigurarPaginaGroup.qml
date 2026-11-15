// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The page group in Archivo follows the reference order: margin profiles, orientation,
// paper size, then the small page-setup launcher. The launcher opens the detailed editor.

import QtQuick
import PunchiFluid

Item {
    id: group

    property FluidSession session
    /// A capture asks for the list before the window has painted.
    property bool paperListRequested: false
    property bool marginListRequested: false
    signal pageSetupRequested(string section)

    implicitWidth: controls.implicitWidth + 24
    implicitHeight: controls.implicitHeight

    Row {
        id: controls
        spacing: 8

        MarginGallery {
            objectName: "ribbonMarginGallery"
            session: group.session
            ribbonMode: true
            controlWidth: 140
            labelWidth: 90
            openOnCreate: group.marginListRequested
            onCustomRequested: group.pageSetupRequested("margins")
        }

        OrientationControl {
            session: group.session
            showLabel: true
            ribbonMode: true
            controlWidth: 96
            currentLabelWidth: 38
        }

        PaperCombo {
            session: group.session
            label: qsTr("Tamaño")
            ribbonMode: true
            controlWidth: 98
            labelWidth: 48
            openOnCreate: group.paperListRequested
            onPageSetupRequested: function (section) {
                group.pageSetupRequested(section);
            }
        }
    }

    // The small page launcher at the bottom-right, as the reference ribbon draws it.
    Rectangle {
        id: launcher
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        width: 20
        height: 20
        radius: session.radiusTip
        color: pointer.containsMouse || activeFocus ? session.surfaceRaised : "transparent"
        border.width: activeFocus ? 1 : 0
        border.color: session.accent
        activeFocusOnTab: true

        Accessible.role: Accessible.Button
        Accessible.name: qsTr("Configurar página")
        Accessible.description: qsTr("abre el editor detallado de la hoja")
        Accessible.onPressAction: group.pageSetupRequested("page")

        Item {
            anchors.centerIn: parent
            width: 15
            height: 18

            Rectangle {
                anchors.fill: parent
                color: "transparent"
                border.width: 1
                border.color: session.lineStrong
                radius: 1
            }

            Rectangle {
                x: 3
                y: 5
                width: 6
                height: 1
                color: session.muted
            }

            Rectangle {
                x: 3
                y: 8
                width: 6
                height: 1
                color: session.muted
            }

            Rectangle {
                x: 3
                y: 11
                width: 5
                height: 1
                color: session.muted
            }

            Rectangle {
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                width: 5
                height: 5
                color: session.surfaceRaised
                border.width: 1
                border.color: session.accent
            }
        }

        MouseArea {
            id: pointer
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: group.pageSetupRequested("page")
        }

        Keys.onSpacePressed: group.pageSetupRequested("page")
        Keys.onReturnPressed: group.pageSetupRequested("page")
        Keys.onEnterPressed: group.pageSetupRequested("page")
    }
}
