// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The three regions of docs/ui-ux.md: ribbon on top, sidebar that mirrors the catalog,
// sheet. A fourth region needs an ADR. Every colour comes from the palette the session
// read, so there is no literal here either.

import QtQuick
import QtQuick.Dialogs
import PunchiFluid

Window {
    id: window
    width: 1280
    height: 800
    minimumWidth: 900
    minimumHeight: 560
    visible: true
    title: qsTr("Punchi Fluid Lab")
    color: session.background

    /// Silence is a property of the window, not of a button: the status bar draws it and
    /// the feedback obeys it.
    property bool muted: false

    FluidSession {
        id: session
    }

    /// Which catalog entry is being dragged out of the palette, and where the pointer is.
    /// A plain property rather than a Qt drag-and-drop session: one code path for the
    /// pointer and for anything that drives the window.
    property int dragCatalog: -1
    property point dragPoint: Qt.point(0, 0)

    SoundFeedback {
        session: session
        muted: window.muted
    }


    FileDialog {
        id: openDialog
        title: qsTr("Abrir circuito")
        nameFilters: [qsTr("Circuitos (*.pfluid)"), qsTr("Todos los archivos (*)")]
        fileMode: FileDialog.OpenFile
        onAccepted: session.openFrom(selectedFile)
    }

    FileDialog {
        id: saveDialog
        title: qsTr("Guardar circuito")
        nameFilters: [qsTr("Circuitos (*.pfluid)")]
        fileMode: FileDialog.SaveFile
        defaultSuffix: "pfluid"
        onAccepted: session.saveTo(selectedFile)
    }

    function saveCommand() {
        if (session.currentPath.length > 0) {
            session.saveCurrent();
        } else {
            saveDialog.open();
        }
    }

    // The normal path needs no modal, and no command is mouse-only. Space holds the valve
    // while it is down, which is the same gesture the pointer makes; the editor opens on
    // the valve instead, so the key is only armed while simulating.
    Shortcut {
        sequence: "Space"
        onActivated: if (!session.editing) {
                         session.pressValve();
                         releaseTimer.restart();
                     }
    }
    Timer {
        id: releaseTimer
        interval: 250
        onTriggered: session.releaseValve()
    }
    Shortcut {
        sequence: "F9"
        onActivated: session.startSimulation()
    }
    Shortcut {
        sequence: "F8"
        onActivated: session.pauseSimulation()
    }
    Shortcut {
        sequence: "F5"
        onActivated: session.stopSimulation()
    }
    Shortcut {
        sequence: "F10"
        onActivated: session.stepForward()
    }
    Shortcut {
        sequence: "Ctrl+R"
        onActivated: session.restart()
    }
    Shortcut {
        sequence: "Escape"
        onActivated: {
            window.dragCatalog = -1;
            session.closeInspector();
        }
    }
    Shortcut {
        sequence: "Ctrl+C"
        onActivated: session.copySelected()
    }
    Shortcut {
        sequence: "Ctrl+V"
        onActivated: session.pasteOffset()
    }
    Shortcut {
        sequence: "Ctrl+D"
        onActivated: session.duplicateSelected()
    }
    Shortcut {
        sequence: "Ctrl+Z"
        onActivated: session.undo()
    }
    Shortcut {
        sequence: "Ctrl+Shift+Z"
        onActivated: session.redo()
    }
    Shortcut {
        sequence: "Delete"
        onActivated: session.removeSelected()
    }
    // The reference turns a component with Ctrl and a double click; a keyboard needs its
    // own key, and Alt+R is not in the way of typing in the properties form.
    Shortcut {
        sequence: "Alt+R"
        onActivated: session.rotateSelected(false)
    }
    Shortcut {
        sequence: "Alt+M"
        onActivated: session.mirrorSelected()
    }
    Shortcut {
        sequence: "Ctrl+M"
        onActivated: window.muted = !window.muted
    }
    Shortcut {
        sequence: "Ctrl+S"
        onActivated: saveCommand()
    }
    Shortcut {
        sequence: "Ctrl+Shift+S"
        onActivated: saveDialog.open()
    }
    Shortcut {
        sequence: "Ctrl+O"
        onActivated: openDialog.open()
    }
    Shortcut {
        sequence: "Ctrl+N"
        onActivated: session.newDocument()
    }
    // The reference zooms with the keys `>` and `<`, and so does this one: a hand stays on
    // the same corner of the keyboard as the mouse.
    Shortcut {
        sequence: ">"
        onActivated: session.zoomIn()
    }
    Shortcut {
        sequence: "<"
        onActivated: session.zoomOut()
    }
    Shortcut {
        sequence: "Ctrl+0"
        onActivated: session.zoomReset()
    }
    Shortcut {
        sequence: "Ctrl+G"
        onActivated: session.toggleGrid()
    }

    // 60 Hz of simulated time in a timer rather than in the paint loop, so a slow frame
    // does not change how far the model advances. The timer stops when the model settles,
    // so an idle window neither redraws nor keeps the compositor awake.
    Timer {
        interval: 16
        running: !session.editing && session.running && !session.settled
        repeat: true
        onTriggered: session.advance(session.frameSeconds * session.speed)
    }

    // The component being carried, drawn with its own symbol as it moves.
    Rectangle {
        visible: window.dragCatalog >= 0
        x: window.dragPoint.x - width * 0.5
        y: window.dragPoint.y - height * 0.5
        width: 72
        height: 44
        radius: 4
        color: session.panel
        opacity: 0.9
        border.width: 1
        border.color: session.accent
        z: 100

        PreviewItem {
            anchors.fill: parent
            anchors.margins: 4
            session: session
            index: window.dragCatalog
        }
    }

    Column {
        anchors.fill: parent

        RibbonBar {
            id: ribbon
            objectName: "ribbon"
            width: parent.width
            height: ribbon.implicitHeight
            session: session
            onFileCommand: function (command) {
                if (command === "new") {
                    session.newDocument();
                } else if (command === "open") {
                    openDialog.open();
                } else if (command === "save-as") {
                    saveDialog.open();
                } else {
                    saveCommand();
                }
            }
        }

        Row {
            width: parent.width
            height: parent.height - ribbon.height - status.height

            Sidebar {
                id: sidebar
                width: 248
                height: parent.height
                session: session
            }

            Rectangle {
                width: parent.width - sidebar.width
                height: parent.height
                color: session.background

                SchematicItem {
                    id: sheet
                    anchors.fill: parent
                    anchors.margins: 12
                    session: session
                }

                // The circuit check, opened from the ribbon. It lists what is wrong and
                // where, and says so plainly when nothing is.
                Rectangle {
                    visible: session.checkOpen
                    anchors.right: parent.right
                    anchors.top: parent.top
                    anchors.margins: 24
                    width: 380
                    radius: 6
                    color: session.panel
                    border.width: 1
                    border.color: session.checkCount > 0 ? session.accent : session.muted
                    height: checkColumn.implicitHeight + 28
                    z: 30

                    Column {
                        id: checkColumn
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.top: parent.top
                        anchors.margins: 14
                        spacing: 10

                        Text {
                            width: parent.width
                            text: session.checkCount > 0
                                  ? qsTr("Comprobación: %1 falta(s)").arg(session.checkCount)
                                  : qsTr("Comprobación del circuito")
                            color: session.foreground
                            font.pixelSize: 12
                            font.bold: true
                        }
                        Text {
                            width: parent.width
                            text: session.checkReport
                            color: session.foreground
                            font.pixelSize: 12
                            wrapMode: Text.WordWrap
                        }
                        Rectangle {
                            width: 96
                            height: 26
                            radius: 4
                            color: "transparent"
                            border.width: 1
                            border.color: session.accent

                            Text {
                                anchors.centerIn: parent
                                text: qsTr("Cerrar")
                                color: session.foreground
                                font.pixelSize: 11
                            }

                            MouseArea {
                                anchors.fill: parent
                                onClicked: session.closeInspector()
                            }
                        }
                    }
                }

                // The properties of the valve, opened by a click in edit mode. It reads
                // what the definition file says rather than a form the front end invented.
                Rectangle {
                    visible: session.inspectorOpen
                    anchors.right: parent.right
                    anchors.top: parent.top
                    anchors.margins: 24
                    width: 360
                    radius: 6
                    color: session.panel
                    border.width: 1
                    border.color: session.accent
                    height: inspectorColumn.implicitHeight + 28
                    z: 30

                    Column {
                        id: inspectorColumn
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.top: parent.top
                        anchors.margins: 14
                        spacing: 10

                        Text {
                            width: parent.width
                            text: session.inspectorText
                            color: session.foreground
                            font.pixelSize: 12
                            wrapMode: Text.WordWrap
                        }
                        Repeater {
                            id: editRows
                            model: session.inspectorFields

                            delegate: Row {
                                width: parent.width
                                spacing: 8
                                property string editKey: modelData.key
                                property alias valueText: editValue.text

                                Text {
                                    width: 150
                                    text: modelData.label
                                    color: session.muted
                                    font.pixelSize: 11
                                }

                                Rectangle {
                                    width: parent.width - 158
                                    height: 24
                                    color: "transparent"
                                    border.width: 1
                                    border.color: session.muted

                                    TextInput {
                                        id: editValue
                                        anchors.fill: parent
                                        anchors.margins: 4
                                        verticalAlignment: TextInput.AlignVCenter
                                        text: String(modelData.value)
                                        color: session.foreground
                                        font.pixelSize: 11
                                        inputMethodHints: Qt.ImhFormattedNumbersOnly
                                    }
                                }
                            }
                        }

                        Row {
                            spacing: 8

                            Rectangle {
                                width: 96
                                height: 26
                                radius: 4
                                color: "transparent"
                                border.width: 1
                                border.color: session.accent
                                visible: editRows.count > 0

                                Text {
                                    anchors.centerIn: parent
                                    text: qsTr("Aplicar")
                                    color: session.foreground
                                    font.pixelSize: 11
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: {
                                        let values = [];
                                        for (let i = 0; i < editRows.count; ++i) {
                                            const row = editRows.itemAt(i);
                                            values.push({ key: row.editKey, value: parseFloat(row.valueText) });
                                        }
                                        session.applyInspectorEdits(values);
                                    }
                                }
                            }

                            Rectangle {
                                width: 96
                                height: 26
                                radius: 4
                                color: "transparent"
                                border.width: 1
                                border.color: session.accent

                                Text {
                                    anchors.centerIn: parent
                                    text: qsTr("Cerrar")
                                    color: session.foreground
                                    font.pixelSize: 11
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: session.closeInspector()
                                }
                            }
                        }
                    }
                }

                Text {
                    anchors.centerIn: sheet
                    visible: !session.hasModel
                    text: qsTr("No hay un circuito que dibujar")
                    color: session.muted
                    font.pixelSize: 16
                }
            }
        }

        StatusBar {
            id: status
            width: parent.width
            height: 40
            session: session
            muted: window.muted
            onMuteToggled: window.muted = !window.muted
            hint: sidebar.selectedName.length > 0
                  ? qsTr("Elegido en el catálogo: %1 — arrástralo a la hoja para colocarlo")
                        .arg(sidebar.selectedName)
                  : session.editing
                    ? qsTr("Modo edición: un clic sobre la válvula abre sus propiedades · F9 inicia la simulación")
                    : qsTr("Mantén pulsada la válvula para accionarla · F8 pausa · F5 detiene · Ctrl+M silencia")
        }
    }
}
