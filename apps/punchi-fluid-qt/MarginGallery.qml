// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The margin preset as a small gallery: the page with its frame drawn inside, the name
// and the four values. The ISO frame is the first row, because it is the default a
// technical drawing expects; the other two say they are not ISO instead of pretending.

import QtQuick
import QtQuick.Controls
import PunchiFluid

Item {
    id: margins

    property FluidSession session
    property bool openOnCreate: false
    /// The last row asks for the four numeric fields; the window focuses them.
    signal customRequested()

    function mm(value) {
        return (Math.round(value * 10) / 10).toString().replace(".", ",");
    }

    function currentDetail() {
        return qsTr("Sup. %1 · Der. %2 · Inf. %3 · Izq. %4 mm")
            .arg(mm(session.sheetMarginTop))
            .arg(mm(session.sheetMarginRight))
            .arg(mm(session.sheetMarginBottom))
            .arg(mm(session.sheetMarginLeft));
    }

    function currentLabel() {
        const index = session.sheetMarginPreset;
        if (index < 0) {
            return qsTr("Personalizado");
        }
        const list = session.marginPresets();
        return index < list.length ? list[index].label : qsTr("Personalizado");
    }

    readonly property var rows: {
        const list = session.marginPresets();
        for (let i = 0; i < list.length; ++i) {
            list[i].index = i;
        }
        list.push({
            index: -1,
            custom: true,
            label: qsTr("Márgenes personalizados…"),
            detail: currentDetail()
        });
        return list;
    }

    function isCurrent(row) {
        return row.custom === true ? session.sheetMarginPreset < 0
                                   : session.sheetMarginPreset === row.index;
    }

    function choose(row) {
        if (row.custom === true) {
            margins.customRequested();
        } else {
            session.applyMarginPreset(row.index);
        }
        popup.close();
    }

    function focusButton() {
        button.forceActiveFocus();
    }

    /// Opens the gallery from a keyboard command or a capture; the header does the same.
    function openPopup() {
        popup.open();
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

    Column {
        id: column
        spacing: 2

        Text {
            text: qsTr("Márgenes")
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
            Accessible.name: qsTr("Márgenes: %1").arg(currentLabel())
            Accessible.description: qsTr("abre los tipos de margen de la hoja")
            Accessible.onPressAction: popup.opened ? popup.close() : popup.open()

            Row {
                id: headerRow
                anchors.centerIn: parent
                spacing: 6

                PaperPreview {
                    anchors.verticalCenter: parent.verticalCenter
                    session: margins.session
                    paperWidthMm: session.sheetPaperWidthMm
                    paperHeightMm: session.sheetPaperHeightMm
                    landscape: session.sheetLandscape
                    marginTop: session.sheetMarginTop
                    marginRight: session.sheetMarginRight
                    marginBottom: session.sheetMarginBottom
                    marginLeft: session.sheetMarginLeft
                    showMargins: true
                    maxWidth: 14
                    maxHeight: 17
                }

                Text {
                    width: 116
                    anchors.verticalCenter: parent.verticalCenter
                    text: currentLabel()
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
        width: 370
        height: Math.min(260, list.contentHeight + 12)
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
            model: margins.rows
            boundsBehavior: Flickable.StopAtBounds
            keyNavigationEnabled: true
            currentIndex: {
                for (let i = 0; i < margins.rows.length; ++i) {
                    if (margins.isCurrent(margins.rows[i])) {
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
                height: 56
                radius: 3
                color: pointer.containsMouse || ListView.isCurrentItem ? session.panel : "transparent"
                border.width: margins.isCurrent(modelData) ? 1 : 0
                border.color: session.accent

                Accessible.role: Accessible.Button
                Accessible.name: qsTr("%1, %2").arg(modelData.label).arg(modelData.detail)
                Accessible.description: qsTr("aplica este tipo de margen")
                Accessible.checkable: true
                Accessible.checked: margins.isCurrent(modelData)

                Row {
                    anchors.fill: parent
                    anchors.leftMargin: 6
                    anchors.rightMargin: 6
                    spacing: 8

                    PaperPreview {
                        anchors.verticalCenter: parent.verticalCenter
                        session: margins.session
                        paperWidthMm: session.sheetPaperWidthMm
                        paperHeightMm: session.sheetPaperHeightMm
                        landscape: session.sheetLandscape
                        marginTop: row.modelData.custom === true
                                   ? session.sheetMarginTop : row.modelData.top
                        marginRight: row.modelData.custom === true
                                     ? session.sheetMarginRight : row.modelData.right
                        marginBottom: row.modelData.custom === true
                                      ? session.sheetMarginBottom : row.modelData.bottom
                        marginLeft: row.modelData.custom === true
                                    ? session.sheetMarginLeft : row.modelData.left
                        showMargins: true
                        maxWidth: 34
                        maxHeight: 44
                    }

                    Column {
                        width: 280
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
                            text: row.modelData.detail
                            color: session.muted
                            font.pixelSize: 10
                            elide: Text.ElideRight
                        }
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: margins.isCurrent(row.modelData)
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
                    onClicked: margins.choose(row.modelData)
                }
            }

            Keys.onReturnPressed: margins.choose(margins.rows[currentIndex])
            Keys.onEnterPressed: margins.choose(margins.rows[currentIndex])
        }

        onOpened: list.forceActiveFocus()
    }
}
