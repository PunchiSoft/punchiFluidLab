// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The sidebar mirrors the catalog: every entry is shown by the definition's own symbol,
// which is the rule. Selecting one is all the editor does today; placing it on the sheet
// is M3 and the hint in the status bar says so instead of pretending.

import QtQuick
import PunchiFluid

Rectangle {
    id: sidebar
    property FluidSession session
    property string selectedName: ""
    property int selectedIndex: -1

    color: session.background

    Rectangle {
        anchors.right: parent.right
        width: 1
        height: parent.height
        color: session.panel
    }

    Text {
        id: title
        x: 16
        y: 14
        text: qsTr("Catálogo")
        color: session.muted
        font.pixelSize: 12
        font.letterSpacing: 1
    }

    ListView {
        id: list
        anchors.top: title.bottom
        anchors.topMargin: 10
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        clip: true
        model: session.catalog
        spacing: 2

        delegate: Rectangle {
            required property var modelData

            width: list.width
            height: 60
            color: sidebar.selectedIndex === modelData.index ? session.panel : "transparent"

            Rectangle {
                anchors.left: parent.left
                width: 3
                height: parent.height
                visible: sidebar.selectedIndex === modelData.index
                color: session.accent
            }

            PreviewItem {
                anchors.left: parent.left
                anchors.leftMargin: 12
                anchors.verticalCenter: parent.verticalCenter
                width: 48
                height: 48
                session: sidebar.session
                index: modelData.index
            }

            Column {
                anchors.left: parent.left
                anchors.leftMargin: 70
                anchors.right: parent.right
                anchors.rightMargin: 8
                anchors.verticalCenter: parent.verticalCenter
                spacing: 3

                Text {
                    width: parent.width
                    text: modelData.name
                    color: session.foreground
                    font.pixelSize: 13
                    elide: Text.ElideRight
                }
                Text {
                    width: parent.width
                    text: modelData.id
                    color: session.muted
                    font.pixelSize: 11
                    elide: Text.ElideRight
                }
            }

            MouseArea {
                id: dragArea
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: session.editing ? Qt.OpenHandCursor : Qt.ArrowCursor

                property bool dragging: false

                onClicked: {
                    sidebar.selectedIndex = modelData.index;
                    sidebar.selectedName = modelData.name;
                }

                // Dragging out of the palette is the editor's gesture. It is done by hand
                // rather than with Qt's drag and drop so the same code path is used by a
                // real pointer and by anything that drives the window, and so a drop has a
                // single, testable entry point.
                onPressed: if (session.editing) {
                               dragging = true;
                               window.dragCatalog = modelData.index;
                               window.dragPoint = mapToItem(null, mouse.x, mouse.y);
                           }
                onPositionChanged: if (dragging) {
                                       window.dragPoint = mapToItem(null, mouse.x, mouse.y);
                                   }
                onReleased: {
                    if (!dragging) {
                        return;
                    }
                    dragging = false;
                    const landing = mapToItem(sheet, mouse.x, mouse.y);
                    if (sheet.contains(landing)) {
                        sheet.placeCatalogAt(window.dragCatalog, landing);
                    }
                    window.dragCatalog = -1;
                }
                onCanceled: {
                    dragging = false;
                    window.dragCatalog = -1;
                }
            }
        }
    }
}
