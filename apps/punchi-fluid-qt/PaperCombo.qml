// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The paper size as one control: what is in force, and the list the user opens to change
// it. Office calls this Tamaño; the list shows the paper's proportions, its normalised
// name and the common name, because "Letter" alone tells a student nothing.
//
// The list is the core's order and the core's numbers. Nothing here copies a size.

import QtQuick
import QtQuick.Controls
import PunchiFluid

Item {
    id: paperCombo

    property FluidSession session
    property bool openOnCreate: false
    signal pageSetupRequested(string section)

    readonly property var rows: {
        const list = session.papers();
        list.push({
            index: -1,
            custom: true,
            label: qsTr("Personalizado…"),
            size: session.sheetPaper < 0
                  ? session.formatPaperSize(session.sheetPaperWidthMm,
                                            session.sheetPaperHeightMm)
                  : qsTr("definir ancho y alto")
        });
        return list;
    }

    // A capture asks for the list before the window has painted; the timer lets
    // the control exist first. It never runs in the normal window.
    Timer {
        interval: 100
        running: openOnCreate
        onTriggered: openPopup()
    }

    implicitWidth: column.implicitWidth
    implicitHeight: column.implicitHeight

    function isCurrent(row) {
        return row.custom === true ? session.sheetPaper < 0 : session.sheetPaper === row.index;
    }

    function choose(row) {
        if (row.custom === true) {
            paperCombo.pageSetupRequested("size");
        } else {
            session.setSheetPaper(row.index);
        }
        popup.close();
    }

    function focusButton() {
        button.forceActiveFocus();
    }

    /// Opens the list from a keyboard command or a capture; the header does the same.
    function openPopup() {
        popup.open();
    }

    Column {
        id: column
        spacing: 2

        Text {
            text: qsTr("Tamaño")
            color: session.muted
            font.pixelSize: 10
        }

        Rectangle {
            id: button
            width: Math.max(160, headerRow.implicitWidth + 16)
            height: 26
            radius: 4
            color: pointer.containsMouse || popup.opened ? session.panel : "transparent"
            border.width: activeFocus ? 1 : 0
            border.color: session.accent
            activeFocusOnTab: true

            Accessible.role: Accessible.Button
            Accessible.name: qsTr("Tamaño: %1").arg(session.sheetPaperDisplay)
            Accessible.description: qsTr("abre la lista de tamaños de hoja")
            Accessible.onPressAction: popup.opened ? popup.close() : popup.open()

            Row {
                id: headerRow
                anchors.centerIn: parent
                spacing: 6

                PaperPreview {
                    anchors.verticalCenter: parent.verticalCenter
                    session: paperCombo.session
                    paperWidthMm: session.sheetPaperWidthMm
                    paperHeightMm: session.sheetPaperHeightMm
                    landscape: session.sheetLandscape
                    maxWidth: 14
                    maxHeight: 17
                }

                Text {
                    width: 118
                    anchors.verticalCenter: parent.verticalCenter
                    text: session.sheetPaperDisplay
                    color: session.foreground
                    font.pixelSize: 11
                    elide: Text.ElideRight
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: "▾"
                    color: session.muted
                    font.pixelSize: 10
                }
            }

            MouseArea {
                id: pointer
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: popup.opened ? popup.close() : popup.open()
            }

            Keys.onSpacePressed: popup.opened ? popup.close() : popup.open()
            Keys.onReturnPressed: popup.opened ? popup.close() : popup.open()
            Keys.onEnterPressed: popup.opened ? popup.close() : popup.open()
        }
    }

    Popup {
        id: popup
        parent: button
        x: 0
        y: button.height + 4
        width: 350
        height: Math.min(430, list.contentHeight + 12)
        padding: 6
        modal: false
        focus: true
        closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside

        background: Rectangle {
            color: session.panel
            border.width: 1
            border.color: session.muted
            radius: 4
        }

        contentItem: ListView {
            id: list
            clip: true
            model: paperCombo.rows
            boundsBehavior: Flickable.StopAtBounds
            keyNavigationEnabled: true
            currentIndex: {
                for (let i = 0; i < paperCombo.rows.length; ++i) {
                    if (paperCombo.isCurrent(paperCombo.rows[i])) {
                        return i;
                    }
                }
                return 0;
            }

            ScrollBar.vertical: ScrollBar {
                policy: ScrollBar.AsNeeded
            }

            delegate: Rectangle {
                id: row
                required property var modelData
                required property int index

                width: ListView.view.width
                height: 44
                radius: 3
                color: pointer.containsMouse || ListView.isCurrentItem ? session.panel : "transparent"
                border.width: paperCombo.isCurrent(modelData) ? 1 : 0
                border.color: session.accent

                Accessible.role: Accessible.Button
                Accessible.name: modelData.custom === true
                                 ? qsTr("Personalizado, definir ancho y alto")
                                 : qsTr("%1, %2").arg(modelData.label).arg(modelData.size)
                Accessible.description: qsTr("elige este tamaño de hoja")
                Accessible.checkable: true
                Accessible.checked: paperCombo.isCurrent(modelData)

                Row {
                    anchors.fill: parent
                    anchors.leftMargin: 6
                    anchors.rightMargin: 6
                    spacing: 8

                    PaperPreview {
                        anchors.verticalCenter: parent.verticalCenter
                        session: paperCombo.session
                        paperWidthMm: row.modelData.custom === true
                                      ? session.sheetPaperWidthMm : row.modelData.widthMm
                        paperHeightMm: row.modelData.custom === true
                                       ? session.sheetPaperHeightMm : row.modelData.heightMm
                        maxWidth: 26
                        maxHeight: 34
                    }

                    Column {
                        width: 258
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: 1

                        Text {
                            width: parent.width
                            text: row.modelData.label
                            color: session.foreground
                            font.pixelSize: 11
                            elide: Text.ElideRight
                        }

                        Text {
                            width: parent.width
                            text: row.modelData.size
                            color: session.muted
                            font.pixelSize: 10
                            elide: Text.ElideRight
                        }
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: paperCombo.isCurrent(row.modelData)
                        text: "✓"
                        color: session.accent
                        font.pixelSize: 12
                    }
                }

                MouseArea {
                    id: pointer
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: paperCombo.choose(row.modelData)
                }
            }

            Keys.onReturnPressed: paperCombo.choose(paperCombo.rows[currentIndex])
            Keys.onEnterPressed: paperCombo.choose(paperCombo.rows[currentIndex])
        }

        onOpened: list.forceActiveFocus()
    }
}
