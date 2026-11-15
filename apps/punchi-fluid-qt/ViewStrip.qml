// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The view strip: the commands a reader uses while keeping their eyes on the sheet. It
// floats over the right edge of the drawing area, takes no column from the paper, and
// calls the same operations as the ribbon and the keyboard. No minimap: fitting the frame
// answers the question a second drawing of the sheet would only repeat.

import QtQuick
import PunchiFluid

Card {
    id: strip

    /// The canvas the view commands act on. The strip never edits the document.
    property Item canvas
    property bool folded: false
    property bool panelOpen: false

    signal foldToggled()

    readonly property int side: 2 * session.unit
    implicitWidth: side + 8
    implicitHeight: column.implicitHeight + 8
    width: implicitWidth
    height: implicitHeight

    /// One row of the zoom panel: an icon, a label and the command.
    component PanelRow: Rectangle {
        id: row
        property string label: ""
        property string icon: ""
        property bool available: true
        signal triggered()

        width: panel.width - 16
        height: 2 * strip.session.unit - 6
        radius: 4
        color: rowArea.containsMouse && row.available ? strip.session.surfaceRaised
                                                     : "transparent"
        opacity: row.available ? 1 : 0.4

        Accessible.role: Accessible.Button
        Accessible.name: row.label
        Accessible.onPressAction: if (row.available) row.triggered()

        Image {
            anchors.verticalCenter: parent.verticalCenter
            anchors.left: parent.left
            anchors.leftMargin: 6
            width: strip.session.unit
            height: strip.session.unit
            sourceSize.width: strip.session.unit
            sourceSize.height: strip.session.unit
            source: row.available && row.icon.length > 0
                    ? "image://theme/" + row.icon + "?tone=foreground"
                    : ""
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            anchors.left: parent.left
            anchors.leftMargin: strip.session.unit + 12
            text: row.label
            color: strip.session.foreground
            font.pixelSize: 11
        }

        MouseArea {
            id: rowArea
            anchors.fill: parent
            hoverEnabled: true
            enabled: row.available
            cursorShape: Qt.PointingHandCursor
            onClicked: row.triggered()
        }
    }

    /// The one place a view command is decided, so a test can drive it without a pointer.
    function invoke(id) {
        if (canvas === null || canvas === undefined) {
            return;
        }
        if (id === "fit") {
            canvas.fitToFrame();
        } else if (id === "zoom-in") {
            session.zoomIn();
        } else if (id === "zoom-out") {
            session.zoomOut();
        } else if (id === "reset") {
            session.zoomReset();
        } else if (id === "back") {
            canvas.previousView();
        } else if (id === "panel") {
            strip.panelOpen = !strip.panelOpen;
        } else if (id === "block") {
            session.blockVisible = !session.blockVisible;
        } else if (id === "play") {
            if (session.running) {
                session.pauseSimulation();
            } else {
                session.startSimulation();
            }
        }
    }

    Column {
        id: column
        anchors.top: parent.top
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.topMargin: 4
        spacing: 2

        Column {
            id: openRows
            visible: !strip.folded
            spacing: 2

            ViewStripButton {
                session: strip.session
                iconName: "zoom-original"
                name: qsTr("Ajustar al marco")
                hint: qsTr("Ajustar al marco · doble clic central")
                onTriggered: strip.invoke("fit")
            }

            ViewStripButton {
                session: strip.session
                iconName: "zoom-in"
                name: qsTr("Zoom")
                hint: qsTr("Zoom · disminuir, 100 %, aumentar y vista anterior")
                selected: strip.panelOpen
                onTriggered: strip.invoke("panel")
            }

            Rectangle {
                width: strip.side - 12
                height: 1
                anchors.horizontalCenter: parent.horizontalCenter
                color: strip.session.line
            }

            ViewStripButton {
                session: strip.session
                iconName: "document-properties"
                name: strip.session.blockVisible ? qsTr("Ocultar el cajetín")
                                                 : qsTr("Mostrar el cajetín")
                hint: strip.session.blockVisible
                      ? qsTr("Ocultar el cajetín · solo en pantalla")
                      : qsTr("Mostrar el cajetín · solo en pantalla")
                // Highlighted when the block is hidden: the strip flags the exceptional
                // state, not the default one.
                selected: !strip.session.blockVisible
                onTriggered: strip.invoke("block")
            }

            ViewStripButton {
                session: strip.session
                iconName: strip.session.running ? "media-playback-pause"
                                                : "media-playback-start"
                name: strip.session.running ? qsTr("Pausa") : qsTr("Iniciar")
                hint: strip.session.running
                      ? qsTr("Pausa · F8, se detiene donde está")
                      : qsTr("Iniciar · F9, anima el modelo")
                onTriggered: strip.invoke("play")
            }

            Rectangle {
                width: strip.side - 12
                height: 1
                anchors.horizontalCenter: parent.horizontalCenter
                color: strip.session.line
            }
        }

        ViewStripButton {
            session: strip.session
            iconName: strip.folded ? "go-top" : "go-bottom"
            name: strip.folded
                  ? qsTr("Mostrar la tira de vista")
                  : qsTr("Ocultar la tira de vista")
            hint: strip.folded
                  ? qsTr("Mostrar la tira de vista")
                  : qsTr("Ocultar la tira de vista · queda un solo botón")
            onTriggered: strip.foldToggled()
        }
    }

    // ---- the zoom panel -----------------------------------------------------
    Card {
        id: panel
        visible: strip.panelOpen && !strip.folded
        anchors.right: parent.left
        anchors.rightMargin: 6
        anchors.verticalCenter: parent.verticalCenter
        width: 168
        height: panelColumn.implicitHeight + 12

        Column {
            id: panelColumn
            anchors.centerIn: parent
            spacing: 2

            PanelRow {
                icon: "zoom-out"
                label: qsTr("Disminuir")
                onTriggered: strip.invoke("zoom-out")
            }
            PanelRow {
                icon: "zoom-original"
                label: qsTr("100 % · el papel a 1:1")
                onTriggered: strip.invoke("reset")
            }
            PanelRow {
                icon: "zoom-in"
                label: qsTr("Aumentar")
                onTriggered: strip.invoke("zoom-in")
            }
            Rectangle {
                width: panel.width - 24
                height: 1
                anchors.horizontalCenter: parent.horizontalCenter
                color: strip.session.line
            }
            PanelRow {
                icon: "edit-undo"
                label: qsTr("Vista anterior")
                available: strip.canvas !== null && strip.canvas !== undefined
                           && strip.canvas.canGoBack
                onTriggered: strip.invoke("back")
            }
        }
    }
}
