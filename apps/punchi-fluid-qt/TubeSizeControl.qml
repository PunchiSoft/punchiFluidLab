// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The tube diameter of the selected pipe: the value in force, and the nominal series the
// core enumerates through the ABI. The metric and inch entries are separate options, so
// 10 mm is never confused with 3/8"; an old pipe and a hand-written value have their own
// place, and an electrical wire has no size at all.
//
// The list is the core's order and the core's numbers. Nothing here copies a series.

import QtQuick
import QtQuick.Controls
import PunchiFluid

Item {
    id: tubeSize

    property FluidSession session

    readonly property int selected: session.selectedPipe
    readonly property bool available: selected >= 0 && !session.selectedPipeElectrical
    readonly property int currentOd: available ? session.selectedPipeTube : -1
    readonly property string shown: available ? session.selectedPipeTubeLabel : "—"

    /// The menu rows: the metric entries, the inch entries and `Sin especificar` last.
    ///
    /// The section header of the last row is empty, because the row itself names the
    /// absence: a second heading saying the same would be noise.
    readonly property var rows: {
        const out = [];
        const sizes = session.tubeSizes();
        for (let index = 0; index < sizes.length; ++index) {
            const size = sizes[index];
            out.push({
                od: size.od,
                label: size.label,
                group: size.nominal === 1 ? qsTr("Pulgada") : qsTr("Métrica")
            });
        }
        out.push({ od: 0, label: qsTr("Sin especificar"), group: "" });
        return out;
    }

    function isCurrent(row) {
        return tubeSize.available && row.od === tubeSize.currentOd;
    }

    function rowIndex() {
        for (let index = 0; index < tubeSize.rows.length; ++index) {
            if (tubeSize.rows[index].od === tubeSize.currentOd) {
                return index;
            }
        }
        return tubeSize.rows.length - 1;
    }

    function choose(row) {
        if (!tubeSize.available) {
            return;
        }
        tubeSize.session.setPipeTube(tubeSize.selected, row.od);
        popup.close();
    }

    /// Opens the list from the ribbon shortcut or a capture.
    function openPopup() {
        if (tubeSize.available) {
            popup.open();
        }
    }

    implicitWidth: 84
    implicitHeight: 66

    Rectangle {
        id: button
        anchors.fill: parent
        radius: session.radiusSmall
        color: pointer.containsMouse || popup.opened ? session.surfaceRaised : "transparent"
        border.width: activeFocus ? 1 : 0
        border.color: session.accent
        activeFocusOnTab: tubeSize.available
        opacity: tubeSize.available ? 1 : 0.4

        Accessible.role: Accessible.Button
        Accessible.name: tubeSize.available
                         ? qsTr("Diámetro del conducto: %1").arg(tubeSize.shown)
                         : qsTr("Diámetro del conducto: sin medida")
        Accessible.description: qsTr("abre la serie métrica y la de pulgadas")
        Accessible.onPressAction: tubeSize.openPopup()

        Column {
            anchors.centerIn: parent
            spacing: 2

            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                text: qsTr("Diámetro")
                color: session.foreground
                font.pixelSize: session.textSmall
            }

            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                width: 74
                horizontalAlignment: Text.AlignHCenter
                text: tubeSize.shown
                color: tubeSize.available ? session.accent : session.muted
                font.pixelSize: session.textBody
                elide: Text.ElideRight
            }
        }

        Text {
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.rightMargin: 4
            anchors.bottomMargin: 3
            text: "▾"
            color: session.muted
            font.pixelSize: 8
        }

        MouseArea {
            id: pointer
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            enabled: tubeSize.available
            onClicked: popup.opened ? popup.close() : tubeSize.openPopup()
        }

        Keys.onSpacePressed: popup.opened ? popup.close() : tubeSize.openPopup()
        Keys.onReturnPressed: popup.opened ? popup.close() : tubeSize.openPopup()
        Keys.onEnterPressed: popup.opened ? popup.close() : tubeSize.openPopup()
    }

    Popup {
        id: popup
        parent: button
        x: 0
        y: button.height + 4
        width: 220
        height: Math.min(390, list.contentHeight + 12)
        padding: 6
        modal: false
        focus: true
        closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside

        background: Rectangle {
            color: session.surfaceRaised
            border.width: 1
            border.color: session.lineStrong
            radius: session.radiusSmall
        }

        contentItem: ListView {
            id: list
            clip: true
            model: tubeSize.rows
            boundsBehavior: Flickable.StopAtBounds
            keyNavigationEnabled: true
            currentIndex: tubeSize.rowIndex()

            section.property: "group"
            section.criteria: ViewSection.FullString
            section.delegate: Item {
                required property string section

                width: ListView.view.width
                height: section.length > 0 ? 22 : 0

                Text {
                    anchors.left: parent.left
                    anchors.leftMargin: 6
                    anchors.verticalCenter: parent.verticalCenter
                    text: parent.section
                    color: session.muted
                    font.pixelSize: session.textSmall
                    font.letterSpacing: 0.4
                }
            }

            ScrollBar.vertical: ScrollBar {
                policy: ScrollBar.AsNeeded
            }

            delegate: Rectangle {
                id: row
                required property var modelData
                required property int index

                width: ListView.view.width
                height: 28
                radius: session.radiusTip
                color: pointer.containsMouse || ListView.isCurrentItem
                       ? session.surfaceRaised : "transparent"
                border.width: tubeSize.isCurrent(modelData) ? 1 : 0
                border.color: session.accent

                Accessible.role: Accessible.Button
                Accessible.name: modelData.label
                Accessible.description: qsTr("elige este diámetro para el conducto")
                Accessible.checkable: true
                Accessible.checked: tubeSize.isCurrent(modelData)

                Row {
                    anchors.fill: parent
                    anchors.leftMargin: 8
                    anchors.rightMargin: 8
                    spacing: 6

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        width: parent.width - 20
                        text: row.modelData.label
                        color: session.foreground
                        font.pixelSize: session.textBody
                        elide: Text.ElideRight
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: tubeSize.isCurrent(row.modelData)
                        text: "✓"
                        color: session.accent
                        font.pixelSize: session.textLabel
                    }
                }

                MouseArea {
                    id: pointer
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: tubeSize.choose(row.modelData)
                }
            }

            Keys.onReturnPressed: tubeSize.choose(tubeSize.rows[currentIndex])
            Keys.onEnterPressed: tubeSize.choose(tubeSize.rows[currentIndex])
        }

        onOpened: list.forceActiveFocus()
    }
}
