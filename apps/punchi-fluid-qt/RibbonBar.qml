// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The ribbon: tabs on top, the commands of the open tab in labelled groups underneath.
// The simulation transport is the Simular tab, like Archivo, Editar, Ver and Cajetín.
// The two help links sit at the right end of the tab row, as text, and are not tabs.
//
// The tabs are the five register of docs/ui-ux.md. A tab whose commands do not exist yet
// says so in one line instead of showing buttons that do nothing, and a command that is
// not implemented is not on screen at all.

import QtQuick
import PunchiFluid

Rectangle {
    id: ribbon
    property FluidSession session

    /// The Archivo commands that need a file dialog, which belongs to the window.
    signal fileCommand(string command)

    /// The help links open popups the window owns, like the file dialogs.
    signal helpCommand(string command)

    /// Below this width a ribbon is worse than a menu, so it folds away.
    readonly property bool collapsed: width < 1100
    property int current: 1
    property bool menuOpen: false

    color: session.background
    // The collapsed menu hangs below the tab row, over the sheet. Without a plane of its
    // own the sheet is painted after it and covers it, so the menu is a sibling problem,
    // not a child one.
    z: menuOpen ? 50 : 0

    // Every command of the ribbon, by tab. `id` is what invoke() switches on; the data is
    // here so the expanded groups and the collapsed menu share one definition.
    readonly property var tabs: [
        {
            name: qsTr("Archivo"),
            groups: [
                {
                    title: qsTr("Documento"),
                    items: [
                        { id: "new", label: qsTr("Nuevo"), icon: "document-new", hint: qsTr("Ctrl+N · empieza otro circuito"), big: true },
                        { id: "open", label: qsTr("Abrir"), icon: "document-open", hint: qsTr("Ctrl+O · abre un circuito"), big: true },
                        { id: "save", label: qsTr("Guardar"), icon: "document-save", hint: session.currentPath.length > 0 ? session.currentPath : qsTr("Ctrl+S · pide un archivo la primera vez"), big: true },
                        { id: "save-as", label: qsTr("Guardar como"), icon: "document-save-as", hint: qsTr("Ctrl+Shift+S · elige otro archivo") },
                        { id: "export", label: qsTr("Exportar"), icon: "document-export", hint: qsTr("Ctrl+Shift+E · el plano en la hoja que elijas, PDF o PNG"), big: true }
                    ]
                },
                {
                    // A project is one document with several circuits. The list is the
                    // core's, so the selector cannot name a circuit that is not there.
                    title: qsTr("Circuito"),
                    items: session.circuits.map(function(circuit, index) {
                        return {
                            id: "circuit:" + circuit.id,
                            label: circuit.name,
                            icon: "x-office-document",
                            hint: circuit.id === session.activeCircuit
                                  ? qsTr("en pantalla")
                                  : qsTr("abrir este circuito"),
                            current: circuit.id === session.activeCircuit
                        };
                    }).concat([
                        { id: "circuit-new", label: qsTr("Nuevo circuito"), icon: "list-add", hint: qsTr("crea una hoja vacía y la abre") },
                        { id: "circuit-rename", label: qsTr("Renombrar"), icon: "edit-rename", hint: qsTr("cambia el nombre del circuito en pantalla") }
                    ])
                },
                {
                    // The sheet the drawing lives on. The list is the export's, so the
                    // sidebar cannot offer a size the writer does not know; the one in use
                    // is marked, because a list of nine without a mark is a guess. Choosing
                    // one changes the document, the canvas and the print together.
                    title: qsTr("Hoja"),
                    items: session.paperNames().map(function (name, index) {
                        return {
                            id: "paper-" + index,
                            label: name,
                            icon: "x-office-document",
                            hint: qsTr("dibujar en %1").arg(name),
                            current: session.sheetPaper === index
                        };
                    }).concat([{
                        id: "paper-landscape",
                        label: qsTr("Apaisado"),
                        icon: "object-rotate-right",
                        hint: qsTr("la hoja girada; el dibujo es el mismo"),
                        current: session.sheetLandscape
                    }, {
                        id: "paper-a4",
                        label: qsTr("A4 normal"),
                        icon: "zoom-original",
                        hint: qsTr("vuelve al A4 vertical con el marco ISO 5457")
                    }, {
                        id: "export-panel",
                        label: qsTr("Márgenes, tamaño y lista…"),
                        icon: "document-export",
                        hint: qsTr("el marco de la hoja y lo que se imprime")
                    }])
                },
                {
                    // The documents the program already knows, so a circuit reopened a
                    // hundred times is one click away. The list is the core's; this only
                    // shows it.
                    title: qsTr("Recientes"),
                    items: session.recent.length === 0
                           ? [{ id: "recent-empty", label: qsTr("Ninguno"), icon: "document-open-recent", hint: qsTr("los archivos abiertos o guardados aparecen aquí"), enabled: false }]
                           : session.recent.map(function (entry, index) {
                                 return {
                                     id: "recent:" + index,
                                     label: entry.name,
                                     icon: "document-open-recent",
                                     hint: entry.path
                                 };
                             })
                }
            ]
        },
        {
            name: qsTr("Editar"),
            groups: [
                {
                    title: qsTr("Historial"),
                    items: [
                        { id: "undo", label: qsTr("Deshacer"), icon: "edit-undo", hint: qsTr("Ctrl+Z"), big: true, enabled: session.canUndo },
                        { id: "redo", label: qsTr("Rehacer"), icon: "edit-redo", hint: qsTr("Ctrl+Shift+Z"), big: true, enabled: session.canRedo }
                    ]
                },
                {
                    title: qsTr("Circuito"),
                    items: [
                        { id: "check", label: qsTr("Comprobar"), icon: "dialog-ok-apply", hint: qsTr("busca faltas antes de simular"), big: true }
                    ]
                },
                {
                    title: qsTr("Componente"),
                    items: [
                        { id: "copy", label: qsTr("Copiar"), icon: "edit-copy", hint: qsTr("Ctrl+C · copia lo seleccionado"), enabled: session.selected >= 0 },
                        { id: "paste", label: qsTr("Pegar"), icon: "edit-paste", hint: qsTr("Ctrl+V · pega junto al original") },
                        { id: "duplicate", label: qsTr("Duplicar"), icon: "edit-duplicate", hint: qsTr("Ctrl+D · copia y desplaza"), enabled: session.selected >= 0 },
                        { id: "rotate", label: qsTr("Girar"), icon: "object-rotate-right", hint: qsTr("Alt+R · un cuarto de vuelta; Ctrl y doble clic gira al revés"), enabled: session.selected >= 0 },
                        { id: "mirror", label: qsTr("Reflejar"), icon: "object-flip-horizontal", hint: qsTr("Alt+M · sobre su propio eje"), enabled: session.selected >= 0 },
                        { id: "select-all", label: qsTr("Todo"), icon: "edit-select-all", hint: qsTr("Ctrl+E · marca todo el circuito") },
                        { id: "delete", label: qsTr("Borrar"), icon: "edit-delete", hint: qsTr("Supr · borra lo seleccionado"), big: true, enabled: session.selected >= 0 }
                    ]
                },
                {
                    title: qsTr("Alinear"),
                    items: [
                        { id: "align-left", label: qsTr("Izquierda"), icon: "align-horizontal-left", hint: qsTr("sobre el objeto más a la izquierda"), enabled: session.selectionCount > 1 },
                        { id: "align-right", label: qsTr("Derecha"), icon: "align-horizontal-right", hint: qsTr("sobre el objeto más a la derecha"), enabled: session.selectionCount > 1 },
                        { id: "align-top", label: qsTr("Arriba"), icon: "align-vertical-top", hint: qsTr("sobre el objeto más alto"), enabled: session.selectionCount > 1 },
                        { id: "align-bottom", label: qsTr("Abajo"), icon: "align-vertical-bottom", hint: qsTr("sobre el objeto más bajo"), enabled: session.selectionCount > 1 },
                        // A hose between two ports on the same axis is straight only when
                        // the other coordinate matches; this moves one end onto it.
                        { id: "align-ports", label: qsTr("Puertos"), icon: "align-horizontal-center", hint: qsTr("mueve el otro extremo para dejar la manguera recta · Alt+A"), enabled: session.selectedPipe >= 0 }
                    ]
                },
                {
                    // A pipe is read by its dashes before anyone reads a label. The class
                    // is a property of the pipe, so the buttons act on the selected one.
                    // The three share the line icon on purpose: the theme has no dashed
                    // one, and inventing a second shape for a dash would be worse than
                    // letting the label say which class the button gives.
                    title: qsTr("Conducto"),
                    items: [
                        { id: "pipe-working", label: qsTr("Trabajo"), icon: "draw-line", hint: qsTr("línea llena, la del aire que trabaja"), enabled: session.selectedPipe >= 0 },
                        { id: "pipe-pilot", label: qsTr("Pilotaje"), icon: "draw-line", hint: qsTr("línea discontinua, la que manda"), enabled: session.selectedPipe >= 0 },
                        { id: "pipe-drain", label: qsTr("Drenaje"), icon: "draw-line", hint: qsTr("discontinua larga, la que evacua"), enabled: session.selectedPipe >= 0 }
                    ]
                },
                {
                    // Equal gaps between symbol boxes, which is what a hand-drawn sheet
                    // needs and what "even spacing" means to a reader.
                    title: qsTr("Repartir"),
                    items: [
                        { id: "space-across", label: qsTr("Horizontal"), icon: "distribute-horizontal-equal", hint: qsTr("huecos iguales entre las cajas, de izquierda a derecha"), enabled: session.selectionCount > 2 },
                        { id: "space-down", label: qsTr("Vertical"), icon: "distribute-vertical-equal", hint: qsTr("huecos iguales entre las cajas, de arriba abajo"), enabled: session.selectionCount > 2 }
                    ]
                },
                {
                    title: qsTr("Paleta"),
                    note: session.editing
                          ? qsTr("Arrastra desde la barra lateral")
                          : qsTr("Detén la simulación para editar")
                }
            ]
        },
        {
            name: qsTr("Ver"),
            groups: [
                {
                    title: qsTr("Vista"),
                    items: [
                        { id: "zoom-in", label: qsTr("Aumentar"), icon: "zoom-in", hint: qsTr("Ctrl+rueda · acerca la hoja") },
                        { id: "zoom-out", label: qsTr("Disminuir"), icon: "zoom-out", hint: qsTr("Ctrl+rueda · aleja la hoja") },
                        { id: "zoom-reset", label: qsTr("Tamaño original"), icon: "zoom-original", hint: qsTr("Ctrl+0 · quita el zoom y el desplazamiento") },
                        { id: "grid", label: qsTr("Cuadrícula"), icon: "view-grid", hint: session.grid ? qsTr("Visible · pulsa para ocultar") : qsTr("Oculta · pulsa para mostrar") }
                    ]
                },
                {
                    title: qsTr("Tema"),
                    items: [
                        { id: "scheme", label: qsTr("Esquema"), icon: "preferences-desktop-theme", hint: qsTr("Se lee del escritorio, nunca se fija aquí") }
                    ]
                },
                { title: qsTr("Contraste"), note: session.contrastSummary }
            ]
        },
        {
            name: qsTr("Cajetín"),
            groups: [
                {
                    title: qsTr("Contenido"),
                    items: [
                        {
                            id: "block-content",
                            label: qsTr("Contenido: %1").arg(
                                session.blockPreset === 3
                                ? qsTr("Sin cajetín")
                                : (session.blockModel === 0 ? qsTr("Escolar")
                                   : (session.blockModel === 1 ? qsTr("Técnico")
                                                              : qsTr("Industrial")))),
                            icon: "x-office-document",
                            hint: qsTr("elige el modelo en el editor lateral"),
                            big: true
                        }
                    ]
                },
                {
                    title: qsTr("Posición"),
                    items: (session.sheetLandscape ? [
                        { id: "position-top-right", label: qsTr("Superior derecha"), icon: "go-top", hint: qsTr("esquina superior derecha"), current: session.blockPreset === 0, enabled: session.blockPreset !== 3 },
                        { id: "position-bottom-right", label: qsTr("Inferior derecha"), icon: "go-bottom", hint: qsTr("la posición de la norma"), current: session.blockPreset === 1, enabled: session.blockPreset !== 3 }
                    ] : [
                        { id: "position-top-right", label: qsTr("Superior derecha"), icon: "go-top", hint: qsTr("esquina superior derecha"), current: session.blockPreset === 0, enabled: session.blockPreset !== 3 },
                        { id: "position-bottom-right", label: qsTr("Inferior derecha"), icon: "go-bottom", hint: qsTr("la posición de la norma"), current: session.blockPreset === 1, enabled: session.blockPreset !== 3 },
                        { id: "position-bottom-full", label: qsTr("Abajo centrado"), icon: "go-bottom", hint: qsTr("de lado a lado"), current: session.blockPreset === 2, enabled: session.blockPreset !== 3 }
                    ])
                },
                {
                    title: qsTr("Datos"),
                    items: [
                        { id: "edit-block", label: session.blockPreset === 3 ? qsTr("Añadir cajetín") : qsTr("Editar datos…"), icon: "x-office-address-book", hint: qsTr("abre el editor lateral; cada cambio se deshace"), big: true },
                        { id: "block-reset", label: qsTr("Restablecer"), icon: "edit-undo", hint: qsTr("vuelve al conjunto de campos del modelo"), enabled: session.blockCustomized }
                    ]
                }
            ]
        },
        {
            name: qsTr("Simular"),
            groups: [
                {
                    title: qsTr("Transporte"),
                    items: [
                        { id: "start", label: qsTr("Iniciar"), icon: "media-playback-start", hint: session.running ? qsTr("ya está en marcha") : qsTr("F9 · anima el modelo"), enabled: !session.running, big: true },
                        { id: "pause", label: qsTr("Pausa"), icon: "media-playback-pause", hint: session.editing ? qsTr("F8 · entra en simulación sin arrancar") : (session.running ? qsTr("F8 · detiene donde está") : qsTr("ya está en pausa")), enabled: session.running || session.editing, big: true },
                        { id: "step", label: qsTr("Paso"), icon: "media-seek-forward", hint: qsTr("F10 · un cuadro y se queda en pausa"), enabled: !session.editing, big: true },
                        { id: "until-change", label: qsTr("Hasta cambio"), icon: "media-skip-forward", hint: qsTr("avanza hasta que el vástago llegue a un tope o una válvula cambie"), enabled: !session.editing, big: true },
                        { id: "stop", label: qsTr("Detener"), icon: "media-playback-stop", hint: session.editing ? qsTr("ya está en modo edición") : qsTr("F5 · vuelve al modo edición"), enabled: !session.editing, big: true },
                        { id: "reset", label: qsTr("Restablecer"), icon: "view-refresh", hint: qsTr("Ctrl+R · vuelve a t = 0"), enabled: !session.editing, big: true }
                    ]
                },
                {
                    title: qsTr("Velocidad"),
                    items: [
                        { id: "speed-0.25", label: "0,25×", icon: "speedometer", hint: qsTr("velocidad de simulación"), current: Math.abs(session.speed - 0.25) < 0.001, big: true },
                        { id: "speed-1", label: "1×", icon: "speedometer", hint: qsTr("velocidad de simulación"), current: Math.abs(session.speed - 1.0) < 0.001, big: true },
                        { id: "speed-2", label: "2×", icon: "speedometer", hint: qsTr("velocidad de simulación"), current: Math.abs(session.speed - 2.0) < 0.001, big: true }
                    ]
                },
                { title: qsTr("Estado"), note: session.editing ? qsTr("edición") : (session.running ? qsTr("en marcha") : qsTr("en pausa")) + " · " + qsTr("t %1 s").arg(session.timeS.toFixed(2)) }
            ]
        },

    ]

    // The help links of the tab row, in reading order. They are not tabs and they have no
    // group: they open a popup about the program itself, which is window chrome and not a
    // command of the circuit. Keeping them in one table makes selfTest press them too.
    readonly property var helpLinks: [
        { id: "about", label: qsTr("Acerca de"), hint: qsTr("nombre, versión, licencia y autoría") },
        { id: "license", label: qsTr("Licencia"), hint: qsTr("el texto completo de la GPL") }
    ]

    implicitHeight: collapsed ? tabRow.height + 1 : tabRow.height + body.implicitHeight + 1

    function invoke(id) {
        if (id.indexOf("circuit:") === 0) {
            session.selectCircuit(parseInt(id.slice(8), 10));
            return;
        }
        if (id === "circuit-new") {
            session.addCircuit();
            return;
        }
        if (id === "circuit-rename") {
            window.openRenameCircuit();
            return;
        }
        if (id === "paper-landscape") {
            session.toggleSheetLandscape();
            return;
        }
        if (id === "paper-a4") {
            session.resetSheet();
            return;
        }
        if (id.indexOf("paper-") === 0) {
            const index = parseInt(id.slice(6), 10);
            if (!isNaN(index)) {
                session.setSheetPaper(index);
            }
            return;
        }
        if (id === "edit-block" || id === "block-content") {
            window.titleBlockOpen = true;
            return;
        }
        if (id.indexOf("position-") === 0) {
            const preset = {
                "position-top-right": 0,
                "position-bottom-right": 1,
                "position-bottom-full": 2
            }[id];
            if (preset !== undefined) {
                session.setBlockPreset(preset);
            }
            return;
        }
        if (id === "block-reset") {
            session.resetBlockVisibility();
            return;
        }
        if (id === "about" || id === "license") {
            ribbon.helpCommand(id);
            return;
        }
        if (id === "restart") {
            session.restart();
        } else if (id === "new" || id === "open" || id === "save" || id === "save-as"
                   || id === "export") {
            ribbon.fileCommand(id);
        } else if (id === "check") {
            session.checkCircuit();
        } else if (id === "undo") {
            session.undo();
        } else if (id === "redo") {
            session.redo();
        } else if (id === "copy") {
            session.copySelected();
        } else if (id === "paste") {
            session.pasteOffset();
        } else if (id === "duplicate") {
            session.duplicateSelected();
        } else if (id === "rotate") {
            session.rotateSelected(false);
        } else if (id === "mirror") {
            session.mirrorSelected();
        } else if (id.indexOf("recent:") === 0) {
            session.openRecent(parseInt(id.substring(7), 10));
        } else if (id === "align-left") {
            session.alignSelection(0);
        } else if (id === "align-right") {
            session.alignSelection(1);
        } else if (id === "align-top") {
            session.alignSelection(2);
        } else if (id === "align-bottom") {
            session.alignSelection(3);
        } else if (id === "align-ports") {
            session.straightenSelectedPipe();
        } else if (id === "pipe-working") {
            session.setPipeClass(0);
        } else if (id === "pipe-pilot") {
            session.setPipeClass(1);
        } else if (id === "pipe-drain") {
            session.setPipeClass(2);
        } else if (id === "space-across") {
            session.spaceSelection(0);
        } else if (id === "space-down") {
            session.spaceSelection(1);
        } else if (id === "select-all") {
            session.selectAll();
        } else if (id === "delete") {
            session.removeSelected();
        } else if (id === "start") {
            session.startSimulation();
        } else if (id === "stop") {
            session.stopSimulation();
        } else if (id === "step") {
            session.stepForward();
        } else if (id === "until-change") {
            session.runUntilChange();
        } else if (id === "play") {
            session.startSimulation();
        } else if (id === "pause") {
            session.pauseSimulation();
        } else if (id === "reset") {
            session.restart();
        } else if (id === "scheme") {
            session.loadTheme();
        } else if (id === "zoom-in") {
            session.zoomIn();
        } else if (id === "zoom-out") {
            session.zoomOut();
        } else if (id === "zoom-reset") {
            session.zoomReset();
        } else if (id === "grid") {
            session.toggleGrid();
        } else if (id.indexOf("speed-") === 0) {
            const value = parseFloat(id.slice(6));
            if (!isNaN(value)) {
                session.speed = value;
            }
        }
    }

    /// Runs every command once, so a broken handler fails a test instead of a user.
    ///
    /// A `TypeError` in an `onTriggered` handler is invisible until the button is pressed;
    /// this is the only way to press them all without a pointer.
    function selfTest() {
        const before = current;
        for (let t = 0; t < tabs.length; ++t) {
            current = t;
            const groups = tabs[t].groups || [];
            for (let g = 0; g < groups.length; ++g) {
                const items = groups[g].items || [];
                for (let i = 0; i < items.length; ++i) {
                    invoke(items[i].id);
                }
            }
            // The collapsed menu is generated from the same flattened list; reading it
            // proves every tab still has one action list and no tab is empty.
            flatActions();
        }
        // The help links are window chrome, not tab commands, but a broken handler fails
        // here as well.
        for (let h = 0; h < helpLinks.length; ++h) {
            invoke(helpLinks[h].id);
        }
        current = before;
    }

    /// Every command of the open tab, flattened, for the collapsed menu.
    function flatActions() {
        const out = [];
        const groups = ribbon.tabs[ribbon.current].groups;
        for (let g = 0; g < groups.length; ++g) {
            const items = groups[g].items || [];
            for (let i = 0; i < items.length; ++i) {
                out.push(items[i]);
            }
        }
        return out;
    }

    // ---- tabs ---------------------------------------------------------------
    Item {
        id: tabRow
        width: parent.width
        height: 34

        Row {
            id: tabsRow
            anchors.left: parent.left
            anchors.leftMargin: 8
            anchors.verticalCenter: parent.verticalCenter
            spacing: 2

            Repeater {
                model: ribbon.tabs

                delegate: Rectangle {
                    required property int index
                    required property var modelData

                    width: tabText.implicitWidth + 26
                    height: 28
                    radius: 4
                    color: ribbon.current === index ? session.panel : (tabPointer.containsMouse ? session.panel : "transparent")

                    Accessible.role: Accessible.PageTab
                    Accessible.name: modelData.name
                    activeFocusOnTab: true

                    Text {
                        id: tabText
                        anchors.centerIn: parent
                        text: modelData.name
                        color: ribbon.current === index ? session.foreground : session.muted
                        font.pixelSize: 12
                    }

                    Rectangle {
                        anchors.bottom: parent.bottom
                        anchors.horizontalCenter: parent.horizontalCenter
                        width: parent.width - 8
                        height: 2
                        color: ribbon.current === index ? session.accent : "transparent"
                    }

                    MouseArea {
                        id: tabPointer
                        anchors.fill: parent
                        hoverEnabled: true
                        onClicked: {
                            ribbon.current = index;
                            ribbon.menuOpen = false;
                        }
                    }

                    Keys.onLeftPressed: ribbon.current = Math.max(0, ribbon.current - 1)
                    Keys.onRightPressed: ribbon.current = Math.min(ribbon.tabs.length - 1, ribbon.current + 1)
                }
            }
        }

        // A ribbon has no width to spare below the threshold: it folds into one menu.
        RibbonButton {
            visible: ribbon.collapsed
            anchors.left: tabsRow.right
            anchors.leftMargin: 8
            anchors.verticalCenter: parent.verticalCenter
            session: ribbon.session
            label: qsTr("Menú")
            iconName: "open-menu-symbolic"
            hint: ribbon.tabs[ribbon.current].name
            large: false
            tone: "foreground"
            onTriggered: ribbon.menuOpen = !ribbon.menuOpen
        }

        // The help links are text, not buttons: the tab row is chrome, and two framed
        // buttons would read as two more tabs. They stay at the right when the ribbon
        // folds, so About and Licence never need a tab switch.
        Row {
            id: helpRow
            anchors.right: parent.right
            anchors.rightMargin: 10
            anchors.verticalCenter: parent.verticalCenter
            spacing: 4

            Repeater {
                model: ribbon.helpLinks

                delegate: HelpLink {
                    required property var modelData
                    session: ribbon.session
                    label: modelData.label
                    hint: modelData.hint
                    onTriggered: ribbon.invoke(modelData.id)
                }
            }
        }
    }

    // ---- body ---------------------------------------------------------------
    Item {
        id: body
        anchors.top: tabRow.bottom
        width: parent.width
        implicitHeight: 86
        visible: !ribbon.collapsed

        Row {
            id: groupsRow
            anchors.left: parent.left
            anchors.leftMargin: 6
            anchors.top: parent.top
            spacing: 0

            Repeater {
                model: ribbon.tabs[ribbon.current].groups

                delegate: RibbonGroup {
                    required property var modelData
                    session: ribbon.session
                    title: modelData.title

                    Repeater {
                        model: modelData.items || []

                        delegate: RibbonButton {
                            required property var modelData
                            session: ribbon.session
                            label: modelData.label
                            iconName: modelData.icon || ""
                            hint: modelData.hint || ""
                            large: modelData.big === true
                            available: modelData.enabled === undefined || modelData.enabled === true
                            selected: modelData.current === true
                            tone: "foreground"
                            onTriggered: ribbon.invoke(modelData.id)
                        }
                    }

                    // A tab whose commands do not exist yet says so once, rather than
                    // showing grey buttons that teach the user nothing.
                    Item {
                        visible: !!modelData.note
                        // A fixed width, not one derived from the wrapped text: a text that
                        // wraps needs the width it is measured with, and asking each for
                        // the other is a binding loop.
                        width: visible ? 180 : 0
                        height: 62

                        Text {
                            id: noteText
                            anchors.verticalCenter: parent.verticalCenter
                            width: parent.width
                            text: modelData.note || ""
                            color: session.muted
                            font.pixelSize: 11
                            wrapMode: Text.WordWrap
                        }
                    }
                }
            }
        }
    }

    // ---- collapsed menu -----------------------------------------------------
    Rectangle {
        id: menu
        visible: ribbon.collapsed && ribbon.menuOpen
        anchors.top: tabRow.bottom
        anchors.left: parent.left
        anchors.leftMargin: 8
        width: 240
        height: menuColumn.implicitHeight + 12
        color: session.panel
        radius: 4
        border.width: 1
        border.color: session.muted
        z: 20

        Column {
            id: menuColumn
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: 6
            spacing: 1

            Repeater {
                model: ribbon.menuOpen ? ribbon.flatActions() : []

                delegate: RibbonButton {
                    required property var modelData
                    width: menuColumn.width
                    session: ribbon.session
                    label: modelData.label
                    iconName: modelData.icon || ""
                    hint: modelData.hint || ""
                    large: false
                    leftAligned: true
                    tone: "foreground"
                    onTriggered: {
                        ribbon.invoke(modelData.id);
                        ribbon.menuOpen = false;
                    }
                }
            }
        }
    }

    Rectangle {
        anchors.bottom: parent.bottom
        width: parent.width
        height: 1
        color: session.panel
    }
}
