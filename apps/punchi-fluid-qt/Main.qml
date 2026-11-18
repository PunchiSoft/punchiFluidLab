// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The three regions of docs/ui-ux.md: ribbon on top, sidebar that mirrors the catalog,
// sheet. A fourth region needs an ADR. Every colour comes from the palette the session
// read, so there is no literal here either.

import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import PunchiFluid

Window {
    id: window
    width: 1280
    height: 800
    minimumWidth: 900
    minimumHeight: 600
    visible: true
    title: session.title
    color: session.background

    /// Silence is a property of the window, not of a button: the status bar draws it and
    /// the feedback obeys it.
    /// Automated runs never make noise: a capture, a selftest or a check would otherwise
    /// play the model over and over on the machine that is testing it.
    readonly property bool automatedRun: {
        const args = Qt.application.arguments;
        return args.indexOf("--mute") >= 0 || args.indexOf("--grab") >= 0
            || args.indexOf("--selftest") >= 0 || args.indexOf("--check-qml") >= 0
            || args.indexOf("--check-sounds") >= 0 || args.indexOf("--export") >= 0;
    }
    property bool muted: automatedRun

    /// What the confirmation is about when it opens: `new`, `open` or `close`. A pending
    /// path is kept so "Guardar y abrir" knows which file it was going to open.
    property string pendingCommand: ""
    property string pendingPath: ""
    property int pendingCircuitRemoval: -1
    property string pendingCircuitName: ""

    /// The rename panel for the circuit in view. It edits a project datum, not a placement.
    property bool renameCircuitOpen: false
    property string renameCircuitText: ""

    /// The help popups. They belong to the window, not to the document: opening one never
    /// changes the circuit and never enters the history.
    property bool aboutOpen: false
    property bool licenseOpen: false
    readonly property string repositoryUrl: "https://github.com/PunchiSoft/punchiFluidLab"
    readonly property string issuesUrl: repositoryUrl + "/issues"

    function openAbout() {
        window.licenseOpen = false;
        window.aboutOpen = true;
    }

    function openLicense() {
        window.aboutOpen = false;
        window.licenseOpen = true;
    }

    function closeHelp() {
        window.aboutOpen = false;
        window.licenseOpen = false;
    }

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

    function requestCircuitRemoval(id, name, components) {
        if (session.circuits.length <= 1) {
            return false;
        }
        if (Number(components) === 0) {
            return session.removeCircuit(Number(id));
        }
        window.pendingCircuitRemoval = Number(id);
        window.pendingCircuitName = String(name);
        return true;
    }

    function confirmCircuitRemoval() {
        const id = window.pendingCircuitRemoval;
        window.pendingCircuitRemoval = -1;
        window.pendingCircuitName = "";
        if (id >= 0) {
            return session.removeCircuit(Number(id));
        }
        return false;
    }

    function cancelCircuitRemoval() {
        window.pendingCircuitRemoval = -1;
        window.pendingCircuitName = "";
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

    // Opening the properties by clicking the sheet brings the dock back if it was hidden.
    Connections {
        target: session
        function onInspectorChanged() {
            if (session.inspectorOpen) {
                window.rightDockOpen = true;
            }
        }
    }

    /// Which catalog entry is being dragged out of the palette, and where the pointer is.
    /// A plain property rather than a Qt drag-and-drop session: one code path for the
    /// pointer and for anything that drives the window.
    /// The raster resolution. 300 dpi is what a printed sheet needs; the vector page has none.
    property int exportDpi: 300
    property bool exportPanelOpen: false
    /// The detailed sheet editor. `pageSetupFocus` only decides which field takes focus.
    property bool pageSetupOpen: false
    property string pageSetupFocus: "page"
    property bool titleBlockOpen: false
    /// The two panels are view state: hiding one gives the sheet its room and never
    /// changes the document, the history or the modified mark.
    property bool catalogOpen: true
    property int catalogWidth: 18 * session.unit
    /// The right dock holds the properties and the title-block editor. It is only
    /// available while one of them is open, and hiding it is view state.
    property bool rightDockOpen: true
    /// In the compact window a panel opens over the sheet and starts folded: the
    /// remembered preference must not be what covers the drawing on a small screen.
    property bool catalogOverlay: false
    property bool rightDockOverlay: false
    /// The view strip over the right edge of the drawing. Folding it is view state, and a
    /// compact window starts with it folded to one button that opens it again.
    property bool viewStripFolded: false
    property bool viewStripCompactOpen: false
    /// The strip of circuits at the foot of the sheet; the canvas gives it the room.
    readonly property int tabStripHeight: Math.max(28, 2 * session.unit - 8)
    onLayoutModeChanged: if (layoutMode !== "compact") {
                             catalogOverlay = false;
                             rightDockOverlay = false;
                             viewStripCompactOpen = false;
                         }
    readonly property bool rightDockHasContent: session.inspectorOpen || window.titleBlockOpen
    onTitleBlockOpenChanged: if (titleBlockOpen) rightDockOpen = true
    /// Auto-hide is opt-in; the panels remember it and their own visibility.
    property bool panelAutoHide: false
    /// The ribbon folded to its tab row, chosen by the user and remembered with the panels.
    property bool ribbonMinimized: false
    readonly property bool panelAutoHideBlocked: window.dragCatalog >= 0
        || window.pageSetupOpen || window.exportPanelOpen || window.renameCircuitOpen
        || window.aboutOpen || window.licenseOpen || window.pendingCommand.length > 0
        || openDialog.visible || saveDialog.visible || exportDialog.visible
    property bool viewStateReady: false

    /// The window is laid out from content minimums, not from a remembered resolution
    /// (interface-plan.md, stage 6a): the sheet floor and the two canvas margins are the
    /// room a panel may not take, and what is left is what the panels share. Every number
    /// here is a measurement or the spacing unit, never a width copied from a screenshot.
    readonly property int sheetFloor: 480
    readonly property int catalogFloor: 220
    readonly property int dockFloor: 300
    readonly property int dockPreference: 20 * session.unit
    readonly property int panelRoom: Math.max(0, window.width - 2 * session.unit)
    readonly property string layoutMode:
        window.panelRoom >= window.catalogWidth + window.dockPreference + window.sheetFloor
            ? "wide"
            : window.panelRoom >= window.catalogFloor + window.dockFloor + window.sheetFloor
                ? "medium"
                : "compact"
    /// What the panels may take once the drawing has its floor.
    readonly property int panelBudget: Math.max(0, window.panelRoom - window.sheetFloor)
    readonly property int effectiveDockWidth:
        window.layoutMode === "wide"
            ? window.dockPreference
            : Math.min(window.dockPreference,
                       Math.max(window.dockFloor, window.panelBudget - window.catalogFloor))
    readonly property int effectiveCatalogWidth:
        window.layoutMode === "wide"
            ? window.catalogWidth
            : Math.max(window.catalogFloor, window.panelBudget - window.effectiveDockWidth)

    onCatalogOpenChanged: if (viewStateReady) {
                              session.setViewSetting("panel/catalogOpen", catalogOpen);
                          }
    onCatalogWidthChanged: if (viewStateReady) {
                               session.setViewSetting("panel/catalogWidth", catalogWidth);
                           }
    onRightDockOpenChanged: if (viewStateReady) {
                                session.setViewSetting("panel/rightDockOpen", rightDockOpen);
                            }
    onPanelAutoHideChanged: if (viewStateReady) {
                                 session.setViewSetting("panel/autoHide", panelAutoHide);
                             }
    onRibbonMinimizedChanged: if (viewStateReady) {
                                  session.setViewSetting("panel/ribbonMinimized", ribbonMinimized);
                              }
    onViewStripFoldedChanged: if (viewStateReady) {
                                  session.setViewSetting("view/stripFolded", viewStripFolded);
                              }

    /// The block's show toggle is view state too: remembered here, never in the document.
    Connections {
        target: session
        function onBlockVisibleChanged() {
            if (window.viewStateReady) {
                session.setViewSetting("view/blockVisible", session.blockVisible);
            }
        }
    }

    Component.onCompleted: {
        // View state only: the document is untouched by which panel was open last time.
        window.catalogOpen = session.viewSetting("panel/catalogOpen", true);
        // A stored width is a preference; a fresh profile keeps the binding to the spacing
        // unit, so the panel opens at its real width even though the theme settles after
        // this call. Clamping a missing value to 220 parked the catalog at its minimum.
        const storedWidth = Number(session.viewSetting("panel/catalogWidth", 0));
        if (isFinite(storedWidth) && storedWidth > 0) {
            window.catalogWidth = Math.max(220, Math.min(480, storedWidth));
        }
        window.rightDockOpen = session.viewSetting("panel/rightDockOpen", true);
        window.panelAutoHide = session.viewSetting("panel/autoHide", false);
        window.ribbonMinimized = session.viewSetting("panel/ribbonMinimized", false);
        window.viewStripFolded = session.viewSetting("view/stripFolded", false);
        session.blockVisible = session.viewSetting("view/blockVisible", true);
        window.restoreGeometry();
        window.viewStateReady = true;
    }

    /// Panels back to their opening state, and the sheet back to 1:1 with no pan.
    function resetView() {
        window.catalogOpen = true;
        window.catalogWidth = 18 * session.unit;
        window.rightDockOpen = true;
        window.panelAutoHide = false;
        window.ribbonMinimized = false;
        window.viewStripFolded = false;
        window.viewStripCompactOpen = false;
        window.cancelCircuitRemoval();
        session.blockVisible = true;
        session.zoomReset();
    }

    function openPageSetup(section) {
        window.exportPanelOpen = false;
        window.pageSetupFocus = section;
        window.pageSetupOpen = true;
    }
    property bool blockCustomizing: false

    /// Heights of the real rows of one content model, for the position miniatures.
    function blockPreviewRows(model) {
        if (model === 0) {
            return [14, 21];
        }
        if (model === 1) {
            return [12, 14, 14];
        }
        return [14, 15, 12, 12, 12];
    }

    /// Positions the guide offers on this sheet. Portrait adds the full bottom band;
    /// landscape keeps the two corners.
    function positionOptions() {
        if (session.sheetLandscape) {
            return [
                { label: qsTr("Esquina superior derecha"), preset: 0 },
                { label: qsTr("Esquina inferior derecha"), preset: 1 }
            ];
        }
        return [
            { label: qsTr("Esquina superior derecha"), preset: 0 },
            { label: qsTr("Esquina inferior derecha"), preset: 1 },
            { label: qsTr("Abajo centrado de lado a lado"), preset: 2 }
        ];
    }

    /// Index of the current preset in the options offered for this orientation.
    function positionIndex() {
        const options = positionOptions();
        for (let index = 0; index < options.length; ++index) {
            if (options[index].preset === session.blockPreset) {
                return index;
            }
        }
        return -1;
    }

    /// The block flesh height on the current paper: 45/40/35 mm on A4, 65/50/40 on A3+.
    function blockPreviewHeight(model) {
        const large = Math.min(session.sheetWidthMm, session.sheetHeightMm) >= 297.0 - 0.01;
        if (model === 0) {
            return large ? 40 : 35;
        }
        if (model === 1) {
            return large ? 50 : 40;
        }
        return large ? 65 : 45;
    }

    /// One block field by name, so the panel can bind a TextInput without nine switches.
    function blockValue(key) {
        switch (key) {
        case "owner": return session.blockOwner;
        case "project": return session.blockProject;
        case "equipment": return session.blockEquipment;
        case "title": return session.blockTitle;
        case "number": return session.blockNumber;
        case "revision": return session.blockRevision;
        case "date": return session.blockDate;
        case "prepared": return session.blockPrepared;
        case "checked": return session.blockChecked;
        case "approved": return session.blockApproved;
        case "scale": return session.blockScale;
        case "norms": return session.blockNorms;
        default: return "";
        }
    }
    property bool exportPartsList: true

    /// The options the export dialog hands to the session, in one map. The sheet, its
    /// orientation and its margins are the document's, so the export does not repeat them.
    function exportOptions(path, kind) {
        return {
            path: path,
            kind: kind,
            dpi: window.exportDpi,
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
        window.saveGeometry();
        if (!session.closeConfirmed && session.hasUnsavedWork()) {
            close.accepted = false;
            window.pendingCommand = "close";
        }
    }

    /// The window's own size and place, remembered as view state: it is where the user
    /// works, not a property of the document. A maximized window keeps the size it had
    /// before, so reopening does not present a screen-sized normal window.
    function saveGeometry() {
        if (window.visibility !== Window.Windowed) {
            return;
        }
        session.setViewSetting("window/width", window.width);
        session.setViewSetting("window/height", window.height);
        session.setViewSetting("window/x", window.x);
        session.setViewSetting("window/y", window.y);
    }

    function restoreGeometry() {
        const width = Number(session.viewSetting("window/width", 0));
        const height = Number(session.viewSetting("window/height", 0));
        if (isFinite(width) && width >= window.minimumWidth) {
            window.width = width;
        }
        if (isFinite(height) && height >= window.minimumHeight) {
            window.height = height;
        }
        // A place is only restored when it still lands on a screen: a laptop undocked from
        // a second monitor must not reopen the window where nothing can reach it.
        const x = Number(session.viewSetting("window/x", NaN));
        const y = Number(session.viewSetting("window/y", NaN));
        const target = window.screen;
        if (!isFinite(x) || !isFinite(y) || target === null || target === undefined) {
            return;
        }
        const area = target.availableGeometry;
        const reachable = x < area.x + area.width - 160 && y < area.y + area.height - 120
                       && x + window.width > area.x + 160 && y + window.height > area.y + 120;
        if (reachable) {
            window.x = x;
            window.y = y;
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

    // The hand mode is left with Esc from anywhere, and the canvas also handles it when it
    // holds the focus; the two paths set the same view flag.
    // Ctrl+Tab walks the circuits; the strip shows the same change on the same click.
    Shortcut {
        sequence: "Ctrl+Tab"
        onActivated: {
            const list = session.circuits;
            if (list.length < 2) {
                return;
            }
            let index = 0;
            for (let i = 0; i < list.length; ++i) {
                if (list[i].id === session.activeCircuit) {
                    index = i;
                }
            }
            session.selectCircuit(list[(index + 1) % list.length].id);
        }
    }

    Shortcut {
        sequence: "Esc"
        enabled: session.handMode
        onActivated: session.setHandMode(false)
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
            window.closeHelp();
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
    // The catalog is chrome: Ctrl+B folds it to its rail and brings it back, and the
    // command in Ver does the same. It is view state, never a document change.
    Shortcut {
        sequence: "Ctrl+B"
        onActivated: window.catalogOpen = !window.catalogOpen
    }
    // The right dock is the same kind of view state: Ctrl+J folds it while it has content.
    Shortcut {
        sequence: "Ctrl+J"
        onActivated: if (window.rightDockHasContent) {
                         window.rightDockOpen = !window.rightDockOpen;
                     }
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
        radius: session.radiusSmall
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
            minimized: window.ribbonMinimized
            bodyInsetLeft: window.layoutMode === "compact" ? 0 : catalogDock.width
            bodyInsetRight: window.layoutMode === "compact" ? 0 : rightDock.width
            onMinimizeToggled: window.ribbonMinimized = !window.ribbonMinimized
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
                    window.pageSetupOpen = false;
                    exportDialog.open();
                } else {
                    saveCommand();
                }
            }
            onHelpCommand: function (command) {
                if (command === "about") {
                    window.openAbout();
                } else if (command === "license") {
                    window.openLicense();
                }
            }
            onPageSetupRequested: function (section) {
                window.openPageSetup(section);
            }
            catalogOpen: window.catalogOpen
            rightDockOpen: window.rightDockOpen
            rightDockAvailable: window.rightDockHasContent
            panelAutoHide: window.panelAutoHide
            onPanelCommand: function (command) {
                if (command === "catalog") {
                    window.catalogOpen = !window.catalogOpen;
                } else if (command === "right-dock" && window.rightDockHasContent) {
                    window.rightDockOpen = !window.rightDockOpen;
                } else if (command === "auto-hide") {
                    window.panelAutoHide = !window.panelAutoHide;
                } else if (command === "reset-view") {
                    window.resetView();
                }
            }
        }

        // The regions are anchored, not stacked by a Row: in the compact window a panel
        // floats over the sheet instead of taking a column, and anchors let the workspace
        // keep its full width while the panel is above it.
        Item {
            width: parent.width
            height: parent.height - ribbon.height - status.height

            DockPanel {
                id: catalogDock
                objectName: "catalogDock"
                session: session
                title: qsTr("Catálogo")
                hideName: qsTr("Ocultar el catálogo")
                showName: qsTr("Mostrar el catálogo")
                open: window.layoutMode === "compact" ? window.catalogOverlay
                                                      : window.catalogOpen
                panelWidth: window.layoutMode === "compact"
                              ? Math.min(window.catalogWidth, window.width - 4 * session.unit)
                              : window.effectiveCatalogWidth
                floating: window.layoutMode === "compact"
                z: window.layoutMode === "compact" ? 10 : 0
                anchors.left: parent.left
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                resizable: true
                minimumPanelWidth: 220
                maximumPanelWidth: 480
                autoHide: window.panelAutoHide
                autoHideBlocked: window.panelAutoHideBlocked
                onToggleRequested: {
                    if (window.layoutMode === "compact") {
                        window.catalogOverlay = !window.catalogOverlay;
                    } else {
                        window.catalogOpen = !window.catalogOpen;
                    }
                }
                onAutoHideRequested: function (open) {
                    if (window.layoutMode === "compact") {
                        window.catalogOverlay = open;
                    } else {
                        window.catalogOpen = open;
                    }
                }
                onWidthRequested: function (width) {
                    window.catalogWidth = width;
                }

                Sidebar {
                    id: sidebar
                    objectName: "catalogSidebar"
                    anchors.fill: parent
                    session: session
                }
            }

            Rectangle {
                // The workspace is the sunken plane around the paper. The block editor
                // docks beside the sheet, not over it: opening it gives the editor real
                // room and the sheet keeps the rest of the window.
                anchors.left: window.layoutMode === "compact"
                               ? parent.left : catalogDock.right
                anchors.right: window.layoutMode === "compact"
                                ? parent.right : rightDock.left
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                color: session.surfaceSunken

                SchematicItem {
                    id: sheet
                    anchors.fill: parent
                    // Half a spacing unit: the paper is the protagonist, and a full unit of
                    // bare workspace read as a thick frame around it.
                    anchors.margins: Math.max(4, Math.round(session.unit / 2))
                    anchors.bottomMargin: Math.max(4, Math.round(session.unit / 2))
                                          + window.tabStripHeight
                    session: session
                }

                // The circuits, at the foot of the sheet: read-only for now, so the reader
                // knows which one is being worked on.
                CircuitTabs {
                    objectName: "circuitTabs"
                    session: session
                    height: window.tabStripHeight
                    onRemoveRequested: function (id, name, components) {
                        window.requestCircuitRemoval(id, name, components);
                    }
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    z: 12
                }

                Card {
                    visible: window.pendingCircuitRemoval >= 0
                    anchors.centerIn: parent
                    width: 360
                    height: removeCircuitColumn.implicitHeight + 28
                    z: 42
                    session: session
                    borderColor: session.accent

                    Column {
                        id: removeCircuitColumn
                        anchors.fill: parent
                        anchors.margins: 14
                        spacing: 12

                        Text {
                            width: parent.width
                            text: qsTr("¿Cerrar «%1» y eliminar sus componentes y conductos?")
                                      .arg(window.pendingCircuitName)
                            color: session.foreground
                            font.pixelSize: session.textLabel
                            wrapMode: Text.WordWrap
                        }
                        Row {
                            spacing: 8
                            RibbonButton {
                                session: session
                                label: qsTr("Cancelar")
                                large: false
                                onTriggered: window.cancelCircuitRemoval()
                            }
                            RibbonButton {
                                session: session
                                label: qsTr("Cerrar circuito")
                                large: false
                                tone: "accent"
                                onTriggered: window.confirmCircuitRemoval()
                            }
                        }
                    }
                }

                // The strip floats over the right edge of the drawing: it takes no column
                // from the paper, and a compact window folds it to one button.
                ViewStrip {
                    id: viewStrip
                    objectName: "viewStrip"
                    session: session
                    canvas: sheet
                    folded: window.layoutMode === "compact"
                            ? !window.viewStripCompactOpen : window.viewStripFolded
                    anchors.right: parent.right
                    anchors.rightMargin: session.unit
                    anchors.verticalCenter: parent.verticalCenter
                    z: 20
                    onFoldToggled: {
                        if (window.layoutMode === "compact") {
                            window.viewStripCompactOpen = !window.viewStripCompactOpen;
                        } else {
                            window.viewStripFolded = !window.viewStripFolded;
                        }
                    }
                }

                // The safety copy of a previous run. It is offered, never loaded on its
                // own: the file may be work the user already saved, and deciding for them
                // is how a recovery becomes a loss.
                Card {
                    visible: session.recoveryOffered && window.pendingCommand.length === 0
                    anchors.left: parent.left
                    anchors.bottom: parent.bottom
                    anchors.margins: 24
                    width: 420
                    height: recoveryColumn.implicitHeight + 28
                    z: 30
                    session: session
                    borderColor: session.accent

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
                                radius: session.radiusSmall
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
                                radius: session.radiusSmall
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
                Card {
                    visible: window.pendingCommand.length > 0
                    anchors.centerIn: parent
                    width: 440
                    height: confirmColumn.implicitHeight + 32
                    z: 40
                    session: session
                    borderColor: session.accent

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
                                radius: session.radiusSmall
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
                                radius: session.radiusSmall
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
                                radius: session.radiusSmall
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
                Card {
                    visible: window.exportPanelOpen
                    anchors.centerIn: parent
                    width: 440
                    height: exportColumn.implicitHeight + 32
                    z: 46
                    session: session

                    Column {
                        id: exportColumn
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.top: parent.top
                        anchors.margins: 16
                        spacing: 8

                        Text {
                            width: parent.width
                            text: qsTr("Exportar")
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

                        Text {
                            width: parent.width
                            text: qsTr("La hoja, su orientación y sus márgenes se ajustan en Configurar página.")
                            color: session.muted
                            font.pixelSize: 10
                            wrapMode: Text.WordWrap
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
                                radius: session.radiusSmall
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
                                radius: session.radiusSmall
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
                                radius: session.radiusSmall
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

                // The detailed sheet editor. It opens from Archivo and from Disposición;
                // each change is already applied and undoable, so it has no Aceptar.
                PageSetupPanel {
                    id: pageSetupPanel
                    objectName: "pageSetupPanel"
                    visible: window.pageSetupOpen
                    session: session
                    focusSection: window.pageSetupFocus
                    onClosed: window.pageSetupOpen = false
                }

                // The circuit name is the one the selector and the printed title block use.
                // It is text, not a number, and the core refuses an empty or control-laden
                // one rather than storing a name nothing can draw.
                Card {
                    visible: window.renameCircuitOpen
                    anchors.centerIn: parent
                    width: 420
                    height: renameColumn.implicitHeight + 32
                    z: 45
                    session: session

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
                            radius: session.radiusSmall
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
                                radius: session.radiusSmall
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
                                radius: session.radiusSmall
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
                Card {
                    visible: session.checkOpen
                    anchors.right: parent.right
                    anchors.top: parent.top
                    anchors.margins: 24
                    width: 380
                    height: checkColumn.implicitHeight + 28
                    z: 30
                    session: session
                    borderColor: session.checkCount > 0 ? session.accent : session.lineStrong

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
                            radius: session.radiusSmall
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


                Text {
                    anchors.centerIn: sheet
                    visible: !session.hasModel
                    text: qsTr("No hay un circuito que dibujar")
                    color: session.muted
                    font.pixelSize: 16
                }
            }

            // The properties and the title block share the right dock: the sheet keeps its
            // column and every field edits the live circuit.
            DockPanel {
                id: rightDock
                objectName: "rightDock"
                session: session
                title: session.inspectorOpen && window.titleBlockOpen
                       ? qsTr("Propiedades y cajetín")
                       : (session.inspectorOpen ? qsTr("Propiedades") : qsTr("Cajetín"))
                hideName: qsTr("Ocultar el panel derecho")
                showName: qsTr("Mostrar el panel derecho")
                open: window.layoutMode === "compact" ? window.rightDockOverlay
                                                      : window.rightDockOpen
                available: window.rightDockHasContent
                panelWidth: window.layoutMode === "compact"
                                ? Math.min(window.dockPreference,
                                           window.width - 4 * session.unit)
                                : window.effectiveDockWidth
                floating: window.layoutMode === "compact"
                z: window.layoutMode === "compact" ? 10 : 0
                sideRight: true
                anchors.right: parent.right
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                autoHide: window.panelAutoHide
                autoHideBlocked: window.panelAutoHideBlocked
                onToggleRequested: {
                    if (window.layoutMode === "compact") {
                        window.rightDockOverlay = !window.rightDockOverlay;
                    } else {
                        window.rightDockOpen = !window.rightDockOpen;
                    }
                }
                onAutoHideRequested: function (open) {
                    if (window.layoutMode === "compact") {
                        window.rightDockOverlay = open;
                    } else {
                        window.rightDockOpen = open;
                    }
                }

                Flickable {
                    anchors.fill: parent
                    anchors.margins: 14
                    contentHeight: rightColumn.implicitHeight
                    clip: true

                    Column {
                        id: rightColumn
                        width: parent.width
                        spacing: 12

                        Column {
                            id: inspectorColumn
                            width: parent.width
                            spacing: 10
                            visible: session.inspectorOpen

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
                                        radius: session.radiusTip
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
                                    radius: session.radiusTip
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
                                    radius: session.radiusSmall
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
                                    radius: session.radiusSmall
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

                        Rectangle {
                            width: parent.width
                            height: 1
                            visible: session.inspectorOpen && window.titleBlockOpen
                            color: session.line
                        }

                        Column {
                            id: blockColumn
                            width: parent.width
                            spacing: 10
                            visible: window.titleBlockOpen

                            Row {
                                width: parent.width
                                spacing: 8

                                Text {
                                    width: parent.width - 76
                                    text: qsTr("Cajetín del circuito")
                                    color: session.foreground
                                    font.pixelSize: 13
                                    elide: Text.ElideRight
                                }

                                Rectangle {
                                    width: 68
                                    height: 24
                                    radius: session.radiusSmall
                                    color: "transparent"
                                    border.width: 1
                                    border.color: session.muted

                                    Text {
                                        anchors.centerIn: parent
                                        text: qsTr("Cerrar")
                                        color: session.foreground
                                        font.pixelSize: 10
                                    }

                                    MouseArea {
                                        anchors.fill: parent
                                        onClicked: window.titleBlockOpen = false
                                    }
                                }
                            }

                            // Content and position: two labelled combo boxes. The content
                            // order is the owner's (Técnico, Escolar, Industrial, Sin cajetín);
                            // the positions follow the sheet orientation.
                            Row {
                                width: parent.width
                                spacing: 8

                                Column {
                                    width: (parent.width - 8) / 2
                                    spacing: 2

                                    Text {
                                        text: qsTr("Contenido")
                                        color: session.muted
                                        font.pixelSize: 10
                                    }

                                    ComboBox {
                                        id: modelCombo
                                        width: parent.width
                                        model: [qsTr("Técnico"), qsTr("Escolar"),
                                                qsTr("Industrial"), qsTr("Sin cajetín")]
                                        currentIndex: session.blockPreset === 3
                                                      ? 3
                                                      : (session.blockModel === 1 ? 0
                                                         : (session.blockModel === 0 ? 1 : 2))
                                        font.pixelSize: 11
                                        Accessible.name: qsTr("Contenido del cajetín")
                                        onActivated: function(index) {
                                            if (index === 3) {
                                                session.setBlockPreset(3);
                                            } else {
                                                session.setBlockModel(index === 0 ? 1
                                                                                  : (index === 1 ? 0 : 2));
                                            }
                                        }
                                    }
                                }

                                Column {
                                    width: (parent.width - 8) / 2
                                    spacing: 2

                                    Text {
                                        text: qsTr("Posición")
                                        color: session.muted
                                        font.pixelSize: 10
                                    }

                                    ComboBox {
                                        id: positionCombo
                                        width: parent.width
                                        enabled: session.blockPreset !== 3
                                        model: window.positionOptions()
                                        textRole: "label"
                                        currentIndex: window.positionIndex()
                                        font.pixelSize: 11
                                        Accessible.name: qsTr("Posición del cajetín")
                                        delegate: ItemDelegate {
                                            width: positionCombo.width
                                            contentItem: Row {
                                                spacing: 8
                                                BlockThumb {
                                                    width: 54
                                                    height: 34
                                                    preset: modelData.preset
                                                    model: session.blockModel
                                                }
                                                Text {
                                                    text: modelData.label
                                                    color: session.foreground
                                                    font.pixelSize: 10
                                                    elide: Text.ElideRight
                                                    anchors.verticalCenter: parent.verticalCenter
                                                }
                                            }
                                        }
                                        onActivated: function(index) {
                                            if (index >= 0) {
                                                session.setBlockPreset(model[index].preset);
                                            }
                                        }
                                    }
                                }
                            }

                            Text {
                                text: session.blockCustomized ? qsTr("Personalizado") : qsTr("Modelo del cajetín")
                                color: session.blockCustomized ? session.accent : session.muted
                                font.pixelSize: 10
                            }

                            Row {
                                width: parent.width
                                spacing: 8

                                Rectangle {
                                    width: 118
                                    height: 26
                                    radius: session.radiusSmall
                                    color: window.blockCustomizing ? session.accent : "transparent"
                                    border.width: 1
                                    border.color: session.muted

                                    Text {
                                        anchors.centerIn: parent
                                        text: qsTr("Personalizar")
                                        color: session.foreground
                                        font.pixelSize: 10
                                    }

                                    MouseArea {
                                        anchors.fill: parent
                                        onClicked: window.blockCustomizing = !window.blockCustomizing
                                    }
                                }

                                Rectangle {
                                    width: 118
                                    height: 26
                                    radius: session.radiusSmall
                                    visible: session.blockCustomized
                                    color: "transparent"
                                    border.width: 1
                                    border.color: session.muted

                                    Text {
                                        anchors.centerIn: parent
                                        text: qsTr("Restablecer modelo")
                                        color: session.foreground
                                        font.pixelSize: 10
                                    }

                                    MouseArea {
                                        anchors.fill: parent
                                        onClicked: session.resetBlockVisibility()
                                    }
                                }
                            }

                            // Only the model's fields, in print order. Personalizar reveals a
                            // switch per field; hiding keeps the typed value.
                            Repeater {
                                model: session.blockFields

                                delegate: Column {
                                    required property var modelData
                                    width: parent.width
                                    spacing: 2
                                    visible: window.blockCustomizing || modelData.visible

                                    Row {
                                        width: parent.width
                                        spacing: 6

                                        Text {
                                            width: 120
                                            text: modelData.label
                                            color: session.muted
                                            font.pixelSize: 10
                                            elide: Text.ElideRight
                                        }

                                        Rectangle {
                                            width: parent.width - (window.blockCustomizing ? 174 : 128)
                                            height: 26
                                            color: session.background
                                            border.width: 1
                                            border.color: session.muted

                                            TextInput {
                                                anchors.fill: parent
                                                anchors.leftMargin: 5
                                                anchors.rightMargin: 5
                                                verticalAlignment: TextInput.AlignVCenter
                                                text: modelData.value
                                                color: session.foreground
                                                font.pixelSize: 11
                                                selectByMouse: true
                                                Accessible.name: modelData.label
                                                onEditingFinished: session.setBlockField(modelData.key, text)
                                            }
                                        }

                                        Rectangle {
                                            width: 42
                                            height: 26
                                            visible: window.blockCustomizing
                                            radius: session.radiusTip
                                            color: "transparent"
                                            border.width: 1
                                            border.color: modelData.visible ? session.accent : session.muted

                                            Text {
                                                anchors.centerIn: parent
                                                text: modelData.visible ? qsTr("Sí") : qsTr("No")
                                                color: session.foreground
                                                font.pixelSize: 10
                                            }

                                            MouseArea {
                                                anchors.fill: parent
                                                onClicked: session.setBlockFieldVisible(modelData.field,
                                                                                        !modelData.visible)
                                            }
                                        }
                                    }
                                }
                            }

                            // The free note. It is stored with the block but not printed yet;
                            // the opposite-corner placement is the next change.
                            Column {
                                width: parent.width
                                spacing: 3

                                Text {
                                    text: qsTr("Nota:")
                                    color: session.foreground
                                    font.pixelSize: 11
                                    font.bold: true
                                }

                                TextArea {
                                    id: remarkArea
                                    width: parent.width
                                    height: 78
                                    placeholderText: qsTr("escriba aquí su nota.")
                                    text: session.blockRemark
                                    wrapMode: TextArea.Wrap
                                    color: session.foreground
                                    placeholderTextColor: session.muted
                                    font.pixelSize: 11
                                    selectByMouse: true
                                    Accessible.name: qsTr("Nota del cajetín")
                                    background: Rectangle {
                                        color: session.background
                                        border.width: 1
                                        border.color: session.muted
                                    }
                                    onActiveFocusChanged: {
                                        if (!activeFocus) {
                                            session.setBlockRemark(text);
                                        }
                                    }
                                }
                            }

                            Row {
                                width: parent.width
                                spacing: 6

                                Text {
                                    width: 120
                                    text: qsTr("Hoja")
                                    color: session.muted
                                    font.pixelSize: 10
                                }

                                Text {
                                    height: 26
                                    verticalAlignment: Text.AlignVCenter
                                    text: session.blockSheet
                                    color: session.foreground
                                    font.pixelSize: 11
                                }
                            }
                        }
                    }
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

    // A help popup swallows the click that lands outside it, so a stray press cannot edit
    // the sheet behind while the user is reading. It is transparent: the sheet is not
    // dimmed, because this is a reference, not a warning.
    MouseArea {
        visible: window.aboutOpen || window.licenseOpen
        anchors.fill: parent
        z: 55
        onClicked: window.closeHelp()
    }

    // About: who wrote the program, which version this is, where the repository lives and
    // under which licence it travels. The version comes from the core, so the popup cannot
    // name a build the binary is not.
    Card {
        id: aboutPopup
        objectName: "aboutPanel"
        visible: window.aboutOpen
        anchors.centerIn: parent
        width: Math.min(560, parent.width - 48)
        height: aboutColumn.implicitHeight + 36
        z: 60
        session: session
        borderColor: session.accent

        Accessible.role: Accessible.Dialog
        Accessible.name: qsTr("Acerca de")

        Column {
            id: aboutColumn
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: 18
            spacing: 8

            Text {
                width: parent.width
                text: qsTr("Acerca de")
                color: session.foreground
                font.pixelSize: 16
                font.bold: true
            }

            Row {
                spacing: 8

                Text {
                    text: "Punchi Fluid Lab"
                    color: session.foreground
                    font.pixelSize: 14
                }

                Text {
                    objectName: "aboutVersion"
                    text: session.coreVersion
                    color: session.muted
                    font.pixelSize: 14
                }
            }

            Text {
                width: parent.width
                text: qsTr("Versión en desarrollo, sin publicar")
                color: session.muted
                font.pixelSize: 11
            }

            Text {
                width: parent.width
                text: qsTr("Simulador de circuitos neumáticos e hidráulicos. Libre y abierto, para que un estudiante sin licencia y sin presupuesto pueda aprender la materia.")
                color: session.foreground
                font.pixelSize: 12
                wrapMode: Text.WordWrap
            }

            Column {
                width: parent.width
                spacing: 2

                Text {
                    text: qsTr("Sitio web")
                    color: session.foreground
                    font.pixelSize: 11
                    font.bold: true
                }

                HelpLink {
                    session: session
                    label: window.repositoryUrl
                    tone: "foreground"
                    underlined: true
                    hint: qsTr("abre el repositorio en el navegador")
                    onTriggered: Qt.openUrlExternally(window.repositoryUrl)
                }
            }

            Column {
                width: parent.width
                spacing: 2

                Text {
                    text: qsTr("Ayuda")
                    color: session.foreground
                    font.pixelSize: 11
                    font.bold: true
                }

                HelpLink {
                    session: session
                    label: qsTr("Informar de un problema")
                    tone: "foreground"
                    underlined: true
                    hint: qsTr("abre las incidencias del repositorio en el navegador")
                    onTriggered: Qt.openUrlExternally(window.issuesUrl)
                }
            }

            Column {
                width: parent.width
                spacing: 2

                Text {
                    text: qsTr("Copyright")
                    color: session.foreground
                    font.pixelSize: 11
                    font.bold: true
                }

                Row {
                    spacing: 12

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: qsTr("© 2026 PunchiSoft · GPL-3.0-or-later")
                        color: session.foreground
                        font.pixelSize: 11
                    }

                    PanelButton {
                        session: session
                        label: qsTr("Leer la licencia")
                        primary: true
                        onTriggered: window.openLicense()
                    }
                }
            }

            Column {
                width: parent.width
                spacing: 2

                Text {
                    text: qsTr("Autoría")
                    color: session.foreground
                    font.pixelSize: 11
                    font.bold: true
                }

                Text {
                    text: "PunchiSoft"
                    color: session.foreground
                    font.pixelSize: 11
                }
            }

            Text {
                width: parent.width
                text: qsTr("Hecho para la comunidad, para que aprender neumática e hidráulica no dependa de una licencia comercial.")
                color: session.muted
                font.pixelSize: 11
                wrapMode: Text.WordWrap
            }

            Item {
                width: parent.width
                height: closeAbout.height

                PanelButton {
                    id: closeAbout
                    anchors.right: parent.right
                    session: session
                    label: qsTr("Cerrar")
                    focus: window.aboutOpen
                    onTriggered: window.closeHelp()
                }
            }
        }
    }

    // The licence: the summary a reader needs and the whole GPL text, which travels inside
    // the binary instead of depending on a file next to it. The link is not in the About
    // panel's text on purpose: the program never fetches anything, it only opens the
    // desktop's browser when the user presses one of the two links above.
    Card {
        id: licensePopup
        objectName: "licensePanel"
        visible: window.licenseOpen
        anchors.centerIn: parent
        width: Math.min(680, parent.width - 48)
        height: Math.min(560, parent.height - 48)
        z: 60
        session: session
        borderColor: session.accent

        Accessible.role: Accessible.Dialog
        Accessible.name: qsTr("Licencia")

        Column {
            id: licenseColumn
            anchors.fill: parent
            anchors.margins: 18
            spacing: 8

            Text {
                id: licenseTitle
                width: parent.width
                text: qsTr("Licencia")
                color: session.foreground
                font.pixelSize: 16
                font.bold: true
            }

            Text {
                id: licenseSummary
                width: parent.width
                text: qsTr("Punchi Fluid Lab se distribuye bajo la GNU General Public License, versión 3 o posterior (GPL-3.0-or-later). Es software libre: puedes redistribuirlo y modificarlo bajo sus términos. Se ofrece SIN GARANTÍA.")
                color: session.foreground
                font.pixelSize: 11
                wrapMode: Text.WordWrap
            }

            Text {
                id: licenseHint
                width: parent.width
                text: qsTr("Texto completo, desplazable:")
                color: session.muted
                font.pixelSize: 11
            }

            Rectangle {
                id: licenseBox
                width: parent.width
                height: Math.max(120, licenseColumn.height - licenseTitle.height
                                      - licenseSummary.height - licenseHint.height
                                      - licenseClose.height - 4 * licenseColumn.spacing)
                color: session.background
                border.width: 1
                border.color: session.muted

                Flickable {
                    anchors.fill: parent
                    anchors.margins: 8
                    contentWidth: width
                    contentHeight: licenseTextItem.implicitHeight
                    clip: true

                    TextEdit {
                        id: licenseTextItem
                        objectName: "licenseText"
                        width: parent.width
                        text: session.licenseText
                        readOnly: true
                        selectByMouse: true
                        wrapMode: TextEdit.Wrap
                        color: session.foreground
                        selectionColor: session.accent
                        selectedTextColor: session.panel
                        // The GPL is preformatted; a fixed-pitch face keeps its
                        // indentation readable. "monospace" is resolved by the platform,
                        // so no font file enters the repository.
                        font.family: "monospace"
                        font.pixelSize: 11
                    }
                }
            }

            Item {
                id: licenseCloseRow
                width: parent.width
                height: licenseClose.height

                PanelButton {
                    id: licenseClose
                    anchors.right: parent.right
                    session: session
                    label: qsTr("Cerrar")
                    focus: window.licenseOpen
                    onTriggered: window.closeHelp()
                }
            }
        }
    }

    // The miniature in the position combo: a sheet with the block at its real corner and
    // the model's real row count. It is the same information as the position label, so a
    // reader who cannot see the drawing can still read the text.
    component BlockThumb: Item {
        id: thumb
        property int preset: 0
        property int model: 2

        Rectangle {
            anchors.fill: parent
            color: session.background
            border.width: 1
            border.color: session.muted
        }

        Rectangle {
            visible: thumb.preset !== 3
            width: thumb.preset === 2
                   ? thumb.width - 6
                   : Math.round(thumb.width * 0.62)
            height: Math.max(4, width * window.blockPreviewHeight(thumb.model) / 170)
            x: thumb.preset === 2 ? 3 : thumb.width - width - 3
            y: thumb.preset === 0 ? 3 : thumb.height - height - 3
            color: session.background
            border.width: 1
            border.color: session.muted

            Column {
                anchors.fill: parent

                Repeater {
                    model: window.blockPreviewRows(thumb.model)

                    delegate: Rectangle {
                        required property var modelData
                        width: parent.width
                        height: parent.height * modelData
                             / (window.blockPreviewHeight(thumb.model) + 0.001)
                        color: "transparent"
                        border.width: 0.5
                        border.color: session.muted
                    }
                }
            }
        }
    }
}
