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
    title: session.title
    color: session.background

    /// Silence is a property of the window, not of a button: the status bar draws it and
    /// the feedback obeys it.
    property bool muted: false

    /// What the confirmation is about when it opens: `new`, `open` or `close`. A pending
    /// path is kept so "Guardar y abrir" knows which file it was going to open.
    property string pendingCommand: ""
    property string pendingPath: ""

    /// The rename panel for the circuit in view. It edits a project datum, not a placement.
    property bool renameCircuitOpen: false
    property string renameCircuitText: ""

    function activeCircuitName() {
        const circuits = session.circuits;
        for (let index = 0; index < circuits.length; ++index) {
            if (circuits[index].id === session.activeCircuit) {
                return circuits[index].name;
            }
        }
        return "";
    }

    function openRenameCircuit() {
        window.renameCircuitText = window.activeCircuitName();
        window.renameCircuitOpen = true;
    }

    /// Asks before throwing work away. Only reached when there is work to lose, so the
    /// normal path still has no modal.
    function guard(cmd, path) {
        if (!session.hasUnsavedWork()) {
            return false;
        }
        window.pendingCommand = cmd;
        window.pendingPath = path || "";
        return true;
    }

    function discardAndContinue() {
        const cmd = window.pendingCommand;
        window.pendingCommand = "";
        if (cmd === "close") {
            session.confirmClose();
            window.close();
        } else if (cmd === "open") {
            const path = window.pendingPath;
            window.pendingPath = "";
            session.discardAndOpen(path);
        } else {
            session.discardAndNewDocument();
            window.pendingPath = "";
        }
    }

    function saveAndContinue() {
        if (session.currentPath.length > 0) {
            if (session.saveCurrent()) {
                discardAndContinue();
            }
        } else {
            // The dialog remembers what was pending, so the save finishes the command.
            saveDialog.open();
        }
    }

    FluidSession {
        id: session
    }

    // The paper and the frame are the document's sheet: the ribbon picks them and the canvas,
    // the check and the print all follow. The panel keeps only what belongs to one print.
    Component.onCompleted: {
        window.exportWidthMm = Math.round(session.sheetWidthMm);
        window.exportHeightMm = Math.round(session.sheetHeightMm);
    }

    /// Which catalog entry is being dragged out of the palette, and where the pointer is.
    /// A plain property rather than a Qt drag-and-drop session: one code path for the
    /// pointer and for anything that drives the window.
    /// The raster resolution. 300 dpi is what a printed sheet needs; the vector page has none.
    property int exportDpi: 300
    /// A custom size for this one print; the working sheet stays the document's paper.
    property bool exportPanelOpen: false
    property bool exportCustomPaper: false
    property int exportWidthMm: 210
    property int exportHeightMm: 297
    property bool exportPartsList: true

    /// The options the export dialog hands to the session, in one map.
    function exportOptions(path, kind) {
        return {
            path: path,
            kind: kind,
            custom_paper: window.exportCustomPaper,
            width: window.exportWidthMm,
            height: window.exportHeightMm,
            landscape: session.sheetLandscape,
            dpi: window.exportDpi,
            margins: [session.sheetMarginTop, session.sheetMarginRight,
                      session.sheetMarginBottom, session.sheetMarginLeft],
            parts_list: window.exportPartsList
        };
    }

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
        onAccepted: function (file) {
            // Opening replaces the sheet, so unsaved work is asked about first.
            if (window.guard("open", file.toString())) {
                window.pendingPath = file.toString();
            } else {
                session.openFrom(file);
            }
        }
    }

    // The sheet goes out as a vector page or as a raster, on the paper the panel offers. The
    // export draws the document, not the example: what leaves is what is on screen.
    FileDialog {
        id: exportDialog
        title: qsTr("Exportar el plano")
        nameFilters: [qsTr("Documento PDF (*.pdf)"), qsTr("Imagen PNG (*.png)"),
                      qsTr("Dibujo vectorial SVG (*.svg)")]
        fileMode: FileDialog.SaveFile
        defaultSuffix: "pdf"
        onAccepted: function (file) {
            const name = file.toString().toLowerCase();
            const kind = name.endsWith(".png") ? 0 : (name.endsWith(".svg") ? 2 : 1);
            session.exportWithOptions(window.exportOptions(file, kind));
        }
    }

    FileDialog {
        id: saveDialog
        title: qsTr("Guardar circuito")
        nameFilters: [qsTr("Circuitos (*.pfluid)")]
        fileMode: FileDialog.SaveFile
        defaultSuffix: "pfluid"
        onAccepted: function (file) {
            if (session.saveTo(file) && window.pendingCommand.length > 0) {
                window.discardAndContinue();
            }
        }
    }

    onClosing: function (close) {
        if (!session.closeConfirmed && session.hasUnsavedWork()) {
            close.accepted = false;
            window.pendingCommand = "close";
        }
    }

    // A copy of the work every thirty seconds while it is unsaved, so a crash costs half a
    // minute instead of the session. It is a copy: the title keeps saying the circuit is
    // not saved, and saving still goes where the user chose.
    Timer {
        interval: 30000
        repeat: true
        running: session.modified
        onTriggered: session.autosave()
    }

    Shortcut {
        sequence: "Ctrl+Shift+W"
        onActivated: {
            if (!window.guard("close")) {
                session.confirmClose();
                window.close();
            }
        }
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
            window.renameCircuitOpen = false;
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
    // The reference marks everything with Ctrl+E; the sheet also marks with a rubber band
    // and with Mayús and a click.
    Shortcut {
        sequence: "Ctrl+E"
        onActivated: session.selectAll()
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
    // The keyboard path for the port magnet: a hose between two ports on the same axis is
    // straight once the other coordinate matches, and this moves one end onto it.
    Shortcut {
        sequence: "Alt+A"
        onActivated: session.straightenSelectedPipe()
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
    // Export shares the dialog with the ribbon; the extension still chooses PDF or PNG.
    Shortcut {
        sequence: "Ctrl+Shift+E"
        onActivated: exportDialog.open()
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
    // the same corner of the keyboard as the mouse. Ctrl and the plus or minus key is the
    // usual path from every other program, and Ctrl+0 returns to 1:1 and drops the pan.
    Shortcut {
        sequence: ">"
        onActivated: session.zoomIn()
    }
    Shortcut {
        sequence: "<"
        onActivated: session.zoomOut()
    }
    Shortcut {
        sequence: "Ctrl+="
        onActivated: session.zoomIn()
    }
    Shortcut {
        sequence: "Ctrl++"
        onActivated: session.zoomIn()
    }
    Shortcut {
        sequence: "Ctrl+-"
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
                    if (!window.guard("new")) {
                        session.newDocument();
                    }
                } else if (command === "open") {
                    openDialog.open();
                } else if (command === "save-as") {
                    saveDialog.open();
                } else if (command === "export") {
                    exportDialog.open();
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

                // The safety copy of a previous run. It is offered, never loaded on its
                // own: the file may be work the user already saved, and deciding for them
                // is how a recovery becomes a loss.
                Rectangle {
                    visible: session.recoveryOffered && window.pendingCommand.length === 0
                    anchors.left: parent.left
                    anchors.bottom: parent.bottom
                    anchors.margins: 24
                    width: 420
                    radius: 6
                    color: session.panel
                    border.width: 1
                    border.color: session.accent
                    height: recoveryColumn.implicitHeight + 28
                    z: 30

                    Column {
                        id: recoveryColumn
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.top: parent.top
                        anchors.margins: 14
                        spacing: 10

                        Text {
                            width: parent.width
                            text: qsTr("Hay una copia de seguridad de una sesión anterior.\n¿La recuperas?")
                            color: session.foreground
                            font.pixelSize: 12
                            wrapMode: Text.WordWrap
                        }
                        Row {
                            spacing: 10
                            Rectangle {
                                width: 96
                                height: 26
                                radius: 4
                                color: "transparent"
                                border.width: 1
                                border.color: session.accent
                                Text {
                                    anchors.centerIn: parent
                                    text: qsTr("Recuperar")
                                    color: session.foreground
                                    font.pixelSize: 11
                                }
                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: session.recoverAutosave()
                                }
                            }
                            Rectangle {
                                width: 96
                                height: 26
                                radius: 4
                                color: "transparent"
                                border.width: 1
                                border.color: session.muted
                                Text {
                                    anchors.centerIn: parent
                                    text: qsTr("Descartar")
                                    color: session.foreground
                                    font.pixelSize: 11
                                }
                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: session.discardAutosave()
                                }
                            }
                        }
                    }
                }

                // The one place the program stops to ask. It appears only when the answer
                // can lose work, and it names the work: a dialog that says "¿seguro?" and
                // nothing else teaches the user to click through it.
                Rectangle {
                    visible: window.pendingCommand.length > 0
                    anchors.centerIn: parent
                    width: 440
                    radius: 8
                    color: session.panel
                    border.width: 1
                    border.color: session.accent
                    height: confirmColumn.implicitHeight + 32
                    z: 40

                    Column {
                        id: confirmColumn
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.top: parent.top
                        anchors.margins: 16
                        spacing: 12

                        Text {
                            width: parent.width
                            text: window.pendingCommand === "close"
                                  ? qsTr("El circuito tiene cambios sin guardar.\n¿Cierro y los pierdo?")
                                  : (window.pendingCommand === "open"
                                     ? qsTr("El circuito tiene cambios sin guardar.\n¿Abro otro archivo y los pierdo?")
                                     : qsTr("El circuito tiene cambios sin guardar.\n¿Empiezo uno nuevo y los pierdo?"))
                            color: session.foreground
                            font.pixelSize: 13
                            wrapMode: Text.WordWrap
                        }
                        Row {
                            spacing: 10
                            Rectangle {
                                width: 120
                                height: 28
                                radius: 4
                                color: "transparent"
                                border.width: 1
                                border.color: session.accent
                                Text {
                                    anchors.centerIn: parent
                                    text: qsTr("Guardar")
                                    color: session.foreground
                                    font.pixelSize: 11
                                }
                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: window.saveAndContinue()
                                }
                            }
                            Rectangle {
                                width: 120
                                height: 28
                                radius: 4
                                color: "transparent"
                                border.width: 1
                                border.color: session.muted
                                Text {
                                    anchors.centerIn: parent
                                    text: qsTr("Descartar")
                                    color: session.foreground
                                    font.pixelSize: 11
                                }
                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: window.discardAndContinue()
                                }
                            }
                            Rectangle {
                                width: 120
                                height: 28
                                radius: 4
                                color: "transparent"
                                border.width: 1
                                border.color: session.muted
                                Text {
                                    anchors.centerIn: parent
                                    text: qsTr("Cancelar")
                                    color: session.foreground
                                    font.pixelSize: 11
                                }
                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: window.pendingCommand = ""
                                }
                            }
                        }
                    }
                }

                // The sheet's frame, the raster resolution and the parts list. The paper and
                // the margins are the document's, so the panel edits them through the session;
                // only the custom print size and the raster settings belong to one export.
                Rectangle {
                    visible: window.exportPanelOpen
                    anchors.centerIn: parent
                    width: 440
                    radius: 8
                    color: session.panel
                    border.width: 1
                    border.color: session.accent
                    height: exportColumn.implicitHeight + 32
                    z: 46

                    Column {
                        id: exportColumn
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.top: parent.top
                        anchors.margins: 16
                        spacing: 8

                        Text {
                            width: parent.width
                            text: qsTr("La hoja y lo que lleva")
                            color: session.foreground
                            font.pixelSize: 13
                        }

                        Text {
                            width: parent.width
                            text: qsTr("%1 · %2 × %3 mm")
                                      .arg(session.sheetPaperName)
                                      .arg(Math.round(session.sheetWidthMm))
                                      .arg(Math.round(session.sheetHeightMm))
                            color: session.muted
                            font.pixelSize: 11
                        }

                        Row {
                            spacing: 8
                            Text {
                                text: qsTr("Tamaño personalizado")
                                color: session.muted
                                font.pixelSize: 11
                                width: 150
                            }
                            Rectangle {
                                width: 200
                                height: 26
                                radius: 4
                                color: window.exportCustomPaper ? session.accent : "transparent"
                                border.width: 1
                                border.color: session.muted
                                Text {
                                    anchors.centerIn: parent
                                    text: qsTr("solo para esta impresión")
                                    color: session.foreground
                                    font.pixelSize: 11
                                }
                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: window.exportCustomPaper = !window.exportCustomPaper
                                }
                            }
                        }

                        Row {
                            spacing: 8
                            visible: window.exportCustomPaper
                            Text {
                                text: qsTr("Ancho × alto (mm)")
                                color: session.muted
                                font.pixelSize: 11
                                width: 150
                            }
                            TextInput {
                                width: 60
                                height: 26
                                verticalAlignment: TextInput.AlignVCenter
                                text: String(window.exportWidthMm)
                                color: session.foreground
                                font.pixelSize: 11
                                inputMethodHints: Qt.ImhFormattedNumbersOnly
                                onTextChanged: window.exportWidthMm = parseInt(text, 10) || 0
                            }
                            Text {
                                text: "×"
                                color: session.muted
                                font.pixelSize: 11
                            }
                            TextInput {
                                width: 60
                                height: 26
                                verticalAlignment: TextInput.AlignVCenter
                                text: String(window.exportHeightMm)
                                color: session.foreground
                                font.pixelSize: 11
                                inputMethodHints: Qt.ImhFormattedNumbersOnly
                                onTextChanged: window.exportHeightMm = parseInt(text, 10) || 0
                            }
                        }

                        // The four margins, in the order a page is read: above, right,
                        // below, left.
                        Row {
                            spacing: 6
                            Text {
                                text: qsTr("Márgenes (mm)")
                                color: session.muted
                                font.pixelSize: 11
                                width: 150
                            }
                            Repeater {
                                // A fixed model: the field binds to the session value and
                                // writes it only when the edit finishes, so typing a margin
                                // does not rebuild the row or record a change per keystroke.
                                model: 4
                                delegate: TextInput {
                                    width: 40
                                    height: 26
                                    verticalAlignment: TextInput.AlignVCenter
                                    text: index === 0 ? String(session.sheetMarginTop)
                                        : index === 1 ? String(session.sheetMarginRight)
                                        : index === 2 ? String(session.sheetMarginBottom)
                                        : String(session.sheetMarginLeft)
                                    color: session.foreground
                                    font.pixelSize: 11
                                    inputMethodHints: Qt.ImhFormattedNumbersOnly
                                    // 0 top, 1 right, 2 bottom, 3 left, the order the
                                    // session and the ABI use.
                                    onEditingFinished: {
                                        const parsed = parseInt(text, 10);
                                        if (!isNaN(parsed)) {
                                            session.setSheetMargin(index, parsed);
                                        }
                                    }
                                }
                            }
                        }

                        Row {
                            spacing: 8
                            Text {
                                text: qsTr("Resolución (dpi)")
                                color: session.muted
                                font.pixelSize: 11
                                width: 150
                            }
                            TextInput {
                                width: 60
                                height: 26
                                verticalAlignment: TextInput.AlignVCenter
                                text: String(window.exportDpi)
                                color: session.foreground
                                font.pixelSize: 11
                                inputMethodHints: Qt.ImhFormattedNumbersOnly
                                onTextChanged: window.exportDpi = parseInt(text, 10) || 300
                            }
                        }

                        Row {
                            spacing: 8
                            Text {
                                text: qsTr("Lista de piezas")
                                color: session.muted
                                font.pixelSize: 11
                                width: 150
                            }
                            Rectangle {
                                width: 60
                                height: 26
                                radius: 4
                                color: window.exportPartsList ? session.accent : "transparent"
                                border.width: 1
                                border.color: session.muted
                                Text {
                                    anchors.centerIn: parent
                                    text: window.exportPartsList ? qsTr("Sí") : qsTr("No")
                                    color: session.foreground
                                    font.pixelSize: 11
                                }
                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: window.exportPartsList = !window.exportPartsList
                                }
                            }
                        }

                        Row {
                            spacing: 10
                            Rectangle {
                                width: 130
                                height: 28
                                radius: 4
                                color: "transparent"
                                border.width: 1
                                border.color: session.accent
                                Text {
                                    anchors.centerIn: parent
                                    text: qsTr("Exportar…")
                                    color: session.foreground
                                    font.pixelSize: 11
                                }
                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: {
                                        window.exportPanelOpen = false;
                                        exportDialog.open();
                                    }
                                }
                            }
                            Rectangle {
                                width: 100
                                height: 28
                                radius: 4
                                color: "transparent"
                                border.width: 1
                                border.color: session.muted
                                Text {
                                    anchors.centerIn: parent
                                    text: qsTr("Cerrar")
                                    color: session.foreground
                                    font.pixelSize: 11
                                }
                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: window.exportPanelOpen = false
                                }
                            }
                        }
                    }
                }

                // The circuit name is the one the selector and the printed title block use.                // The circuit name is the one the selector and the printed title block use.
                // It is text, not a number, and the core refuses an empty or control-laden
                // one rather than storing a name nothing can draw.
                Rectangle {
                    visible: window.renameCircuitOpen
                    anchors.centerIn: parent
                    width: 420
                    radius: 8
                    color: session.panel
                    border.width: 1
                    border.color: session.accent
                    height: renameColumn.implicitHeight + 32
                    z: 45

                    Column {
                        id: renameColumn
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.top: parent.top
                        anchors.margins: 16
                        spacing: 12

                        Text {
                            width: parent.width
                            text: qsTr("Renombrar el circuito")
                            color: session.foreground
                            font.pixelSize: 13
                        }
                        Rectangle {
                            width: parent.width
                            height: 30
                            radius: 4
                            color: session.background
                            border.width: 1
                            border.color: session.muted
                            TextInput {
                                anchors.fill: parent
                                anchors.leftMargin: 8
                                anchors.rightMargin: 8
                                verticalAlignment: TextInput.AlignVCenter
                                text: window.renameCircuitText
                                color: session.foreground
                                font.pixelSize: 13
                                selectByMouse: true
                                focus: window.renameCircuitOpen
                                onTextChanged: window.renameCircuitText = text
                                onAccepted: {
                                    session.renameCircuit(session.activeCircuit,
                                                          window.renameCircuitText);
                                    window.renameCircuitOpen = false;
                                }
                            }
                        }
                        Row {
                            spacing: 10
                            Rectangle {
                                width: 120
                                height: 28
                                radius: 4
                                color: "transparent"
                                border.width: 1
                                border.color: session.accent
                                Text {
                                    anchors.centerIn: parent
                                    text: qsTr("Renombrar")
                                    color: session.foreground
                                    font.pixelSize: 11
                                }
                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: {
                                        session.renameCircuit(session.activeCircuit,
                                                              window.renameCircuitText);
                                        window.renameCircuitOpen = false;
                                    }
                                }
                            }
                            Rectangle {
                                width: 120
                                height: 28
                                radius: 4
                                color: "transparent"
                                border.width: 1
                                border.color: session.muted
                                Text {
                                    anchors.centerIn: parent
                                    text: qsTr("Cancelar")
                                    color: session.foreground
                                    font.pixelSize: 11
                                }
                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: window.renameCircuitOpen = false
                                }
                            }
                        }
                    }
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
                        // A recorder or a limit switch binds one component: the recorder
                        // charts a quantity, the switch follows a cylinder's position. The
                        // list is the core's — only what the circuit can report — and the
                        // bound source is marked, because a list with no mark is a guess.
                        Column {
                            width: parent.width
                            spacing: 4
                            visible: chartSources.count > 0

                            Repeater {
                                id: chartSources
                                model: session.chartCandidates

                                delegate: Rectangle {
                                    width: parent.width
                                    height: 24
                                    radius: 3
                                    color: modelData.current ? session.accent : "transparent"
                                    border.width: 1
                                    border.color: modelData.current ? session.accent : session.muted

                                    Text {
                                        anchors.left: parent.left
                                        anchors.right: parent.right
                                        anchors.verticalCenter: parent.verticalCenter
                                        anchors.leftMargin: 6
                                        anchors.rightMargin: 6
                                        elide: Text.ElideRight
                                        // The bound row says so in words, not only in
                                        // colour: a mark a colour-blind reader cannot see
                                        // is not a mark.
                                        text: modelData.current
                                              ? modelData.label + "  ·  " + qsTr("en pantalla")
                                              : modelData.label
                                        color: session.foreground
                                        font.pixelSize: 11
                                    }

                                    MouseArea {
                                        anchors.fill: parent
                                        onClicked: session.setChartSource(modelData.placement,
                                                                          modelData.quantity)
                                    }
                                }
                            }

                            Rectangle {
                                width: 120
                                height: 24
                                radius: 3
                                visible: session.chartSourcePlacement >= 0
                                color: "transparent"
                                border.width: 1
                                border.color: session.muted

                                Text {
                                    anchors.centerIn: parent
                                    text: qsTr("Quitar fuente")
                                    color: session.foreground
                                    font.pixelSize: 11
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: session.clearChartSource()
                                }
                            }
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
                                        // A designation is a name, not a number: the form
                                        // asks for one keyboard and parses accordingly.
                                        inputMethodHints: modelData.numeric === false
                                                          ? Qt.ImhNone
                                                          : Qt.ImhFormattedNumbersOnly
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
                                        // The text travels as it was typed: the form has
                                        // numbers and a name, and parsing every field as a
                                        // number here is how a designation becomes NaN.
                                        let values = [];
                                        for (let i = 0; i < editRows.count; ++i) {
                                            const row = editRows.itemAt(i);
                                            values.push({ key: row.editKey, value: row.valueText });
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
