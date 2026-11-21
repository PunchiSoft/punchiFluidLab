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
    /// The window that owns the dialogs, the guards and the export panel. The ribbon only
    /// asks it, and the window passes itself in.
    property var host: null
    /// The catalog's view state, so the Ver command can say what it will do.
    property bool catalogOpen: true
    /// The right dock's view state and whether it has content to show.
    property bool rightDockOpen: true
    property bool rightDockAvailable: true
    /// The opt-in auto-hide, so the Ver command can show whether it is on.
    property bool panelAutoHide: false

    /// The Archivo commands that need a file dialog, which belongs to the window.
    signal fileCommand(string command)

    /// Panel commands act on the window's view state, not on the session.
    signal panelCommand(string command)

    /// The help links open popups the window owns, like the file dialogs.
    signal helpCommand(string command)

    /// The page controls ask the window for the detailed sheet editor. `section` is
    /// `page`, `size` or `margins`, and only decides which field takes the focus.
    signal pageSetupRequested(string section)

    /// At this width and below a ribbon is worse than a menu, so it folds away: 1280 is the
    /// common small-laptop width and the Editar row no longer fits before it (P-027).
    readonly property bool collapsed: width <= 1280
    /// The tab row grows with the font and the spacing unit; at the default unit it is the
    /// 34 px the ribbon was drawn with.
    readonly property int tabHeight: Math.max(2 * session.unit - 2,
                                             session.textLabel + 2 * session.unit / 3)
    /// The folded menu scrolls when the window cannot hold every command.
    readonly property real menuMaxHeight: ribbon.Window.window
        ? Math.max(160, ribbon.Window.window.height - ribbon.y - tabRow.height - 56)
        : 480
    property int current: 1
    property bool menuOpen: false
    /// The user folded the ribbon to its tab row. The open tab's groups appear as a panel
    /// over the sheet and close after a command or `Esc`; the state is view state and is
    /// remembered with the panels, never written to the document.
    property bool minimized: false
    property bool bodyOpen: false
    readonly property bool bodyShown: !ribbon.collapsed && (!ribbon.minimized || ribbon.bodyOpen)
    /// The room the panels take beside the drawing. A folded body opens over the sheet, not
    /// over their headers, so it starts where the workspace starts.
    property int bodyInsetLeft: 0
    property int bodyInsetRight: 0
    signal minimizeToggled()
    /// Capture and review hooks: open a list as soon as its control exists.
    property bool paperListRequested: false
    property bool marginListRequested: false
    /// The loaded Diámetro control, so the window shortcut can open its list even when the
    /// user is on another tab. The body's control stays loaded when a group collapses; a
    /// flyout copy is used only while it is open, so a closed flyout cannot leave the
    /// shortcut pointing at a destroyed control.
    readonly property var tubeControl: flyoutTubeControl || bodyTubeControl
    /// The two instances of the control: the body keeps one even when its group collapses,
    /// and the flyout owns one while it is open. Alt+D reaches whichever is visible.
    property var bodyTubeControl: null
    property var flyoutTubeControl: null
    /// The measured groups of the open tab, by index, so the row can decide its collapse
    /// set before it lays the groups out. Each entry is `{ title, expanded, compact }`.
    property var groupMetrics: ({})
    /// The capture hook: `--ribbon-flyout` names a collapsed group to open after layout.
    property string flyoutRequest: ""
    /// The collapse order is a product decision, from the least to the most used (P-028).
    readonly property var collapsePriority: [
        "Repartir", "Alinear", "Paleta", "Historial", "Circuito", "Componente", "Conducto"
    ]

    color: session.background
    // The collapsed menu hangs below the tab row, over the sheet. Without a plane of its
    // own the sheet is painted after it and covers it, so the menu is a sibling problem,
    // not a child one.
    z: menuOpen || (minimized && bodyOpen) ? 50 : 0

    // Every command of the ribbon, by tab. `id` is what invoke() switches on; the data is
    // here so the expanded groups and the collapsed menu share one definition.
    readonly property var tabs: [
        {
            name: qsTr("Archivo"),
            groups: [
                {
                    title: qsTr("Proyecto"),
                    items: [
                        { id: "new", label: qsTr("Nuevo"), icon: "document-new", hint: qsTr("Ctrl+N · empieza un proyecto vacío"), big: true },
                        { id: "open", label: qsTr("Abrir"), icon: "document-open", hint: qsTr("Ctrl+O · abre un proyecto"), big: true },
                        { id: "save", label: qsTr("Guardar"), icon: "document-save", hint: session.currentPath.length > 0 ? session.currentPath : qsTr("Ctrl+S · pide un archivo la primera vez"), big: true },
                        { id: "save-as", label: qsTr("Guardar como"), icon: "document-save-as", hint: qsTr("Ctrl+Shift+S · elige otro archivo") },
                        { id: "export", label: qsTr("Exportar"), icon: "document-export", hint: qsTr("Ctrl+Shift+E · el plano en la hoja que elijas, PDF o PNG"), big: true },
                        { id: "export-panel", label: qsTr("Opciones…"), icon: "document-properties", hint: qsTr("dpi, lista de piezas y formato de la exportación") }
                    ]
                },
                {
                    // The strip at the foot of the sheet is the list of circuits; this group
                    // holds only the commands that act on the circuit in view, so its width
                    // is the same with one circuit or with thirty-two.
                    title: qsTr("Circuito"),
                    items: [
                        { id: "circuit-new", label: qsTr("Nuevo"), icon: "list-add", hint: qsTr("Ctrl+T · crea un circuito vacío y lo abre") },
                        { id: "circuit-rename", label: qsTr("Renombrar"), icon: "edit-rename", hint: qsTr("F2 · cambia el nombre del circuito en pantalla") },
                        { id: "circuit-close", label: qsTr("Cerrar"), icon: "window-close", hint: qsTr("Ctrl+W · cierra el circuito en pantalla"), enabled: session.circuits.length > 1 }
                    ]
                },
                {
                    // Inicio is reachable from here with a project open, and returns with
                    // Volver or Esc (ADR 0031).
                    title: qsTr("Inicio"),
                    items: [
                        { id: "home", label: qsTr("Inicio"), icon: "go-home", hint: qsTr("la página de inicio: nuevo, abrir y recientes") }
                    ]
                },
                {
                    // The page as a dropdown and a launcher, not a tab of its own: the
                    // margin profiles, the custom size and the grid live in the panel the
                    // launcher opens.
                    title: qsTr("Configurar página"),
                    custom: "page-setup"
                },
                {
                    // The documents the program already knows, as one dropdown instead of
                    // a row that grows with every file opened.
                    title: qsTr("Recientes"),
                    custom: "recent"
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
                        { id: "check", label: qsTr("Comprobar"), icon: "check", hint: qsTr("busca faltas antes de simular"), big: true }
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
                        { id: "align-ports", label: qsTr("Puertos"), icon: "align-ports", hint: qsTr("mueve el otro extremo para dejar la manguera recta · Alt+A"), enabled: session.selectedPipe >= 0 }
                    ]
                },
                {
                    // A pipe is read by its dashes before anyone reads a label. The class
                    // is a property of the pipe, so the buttons act on the selected one.
                    // Each class has its own drawing with the sheet's pattern, so the
                    // button shows the line it gives instead of relying on the label.
                    title: qsTr("Conducto"),
                    custom: "tube",
                    items: [
                        { id: "pipe-working", label: qsTr("Trabajo"), icon: "pipe-working", hint: qsTr("línea llena, la del aire que trabaja"), enabled: session.selectedPipe >= 0 },
                        { id: "pipe-pilot", label: qsTr("Pilotaje"), icon: "pipe-pilot", hint: qsTr("línea discontinua, la que manda"), enabled: session.selectedPipe >= 0 },
                        { id: "pipe-drain", label: qsTr("Drenaje"), icon: "pipe-drain", hint: qsTr("discontinua larga, la que evacua"), enabled: session.selectedPipe >= 0 }
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
                        { id: "zoom-reset", label: qsTr("Tamaño original"), icon: "zoom-original", hint: qsTr("Ctrl+0 · quita el zoom y el desplazamiento") }
                    ]
                },
                {
                    // Hiding a panel is view state: each command says which way it will go,
                    // and the panel it controls is marked while it is open.
                    title: qsTr("Paneles"),
                    items: [
                        { id: "catalog", label: qsTr("Catálogo"), icon: "view-list-details", hint: ribbon.catalogOpen ? qsTr("Ctrl+B · oculta la barra del catálogo") : qsTr("Ctrl+B · muestra la barra del catálogo"), current: ribbon.catalogOpen, panel: true },
                        // The right dock holds the properties and the title block; the
                        // command is only available while one of them is open.
                        { id: "right-dock", label: qsTr("Propiedades"), icon: "document-properties", hint: ribbon.rightDockOpen ? qsTr("Ctrl+J · oculta el panel derecho") : qsTr("Ctrl+J · muestra el panel derecho"), current: ribbon.rightDockOpen && ribbon.rightDockAvailable, enabled: ribbon.rightDockAvailable, panel: true },
                        { id: "auto-hide", label: qsTr("Ocultar al salir"), icon: "view-restore", hint: ribbon.panelAutoHide ? qsTr("activado · el panel vuelve al pasar por el borde") : qsTr("apagado · el panel se queda abierto"), current: ribbon.panelAutoHide, panel: true },
                        { id: "reset-view", label: qsTr("Restablecer vista"), icon: "edit-reset", hint: qsTr("paneles, zoom y desplazamiento a su estado inicial") }
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
                        { id: "start", label: qsTr("Iniciar"), icon: "media-playback-start", hint: session.running ? qsTr("ya está en marcha") : qsTr("F9 · anima el modelo"), enabled: !session.running && !session.untilChangeRunning, big: true },
                        { id: "pause", label: qsTr("Pausa"), icon: "media-playback-pause", hint: session.editing ? qsTr("F8 · entra en simulación sin arrancar") : (session.running ? qsTr("F8 · detiene donde está") : (session.untilChangeRunning ? qsTr("F8 · cancela el avance") : qsTr("ya está en pausa"))), enabled: session.running || session.editing || session.untilChangeRunning, big: true },
                        { id: "step", label: qsTr("Paso"), icon: "media-seek-forward", hint: qsTr("F10 · un cuadro y se queda en pausa"), enabled: !session.editing && !session.untilChangeRunning, big: true },
                        { id: "until-change", label: qsTr("Hasta cambio"), icon: "media-skip-forward", hint: qsTr("avanza hasta que el vástago llegue a un tope o una válvula cambie"), enabled: !session.editing && !session.untilChangeRunning, big: true },
                        { id: "stop", label: qsTr("Detener"), icon: "media-playback-stop", hint: session.editing ? qsTr("ya está en modo edición") : qsTr("F5 · vuelve al modo edición o cancela el avance"), enabled: !session.editing, big: true },
                        { id: "reset", label: qsTr("Restablecer"), icon: "view-refresh", hint: qsTr("Ctrl+R · vuelve a t = 0"), enabled: !session.editing, big: true }
                    ]
                },
                {
                    title: qsTr("Velocidad"),
                    items: [
                        { id: "speed-0.25", label: "0,25×", icon: "speed-0.25", hint: qsTr("velocidad de simulación"), current: Math.abs(session.speed - 0.25) < 0.001, big: true },
                        { id: "speed-1", label: "1×", icon: "speed-1", hint: qsTr("velocidad de simulación"), current: Math.abs(session.speed - 1.0) < 0.001, big: true },
                        { id: "speed-2", label: "2×", icon: "speed-2", hint: qsTr("velocidad de simulación"), current: Math.abs(session.speed - 2.0) < 0.001, big: true }
                    ]
                },
                { title: qsTr("Estado"), note: session.untilChangeRunning ? qsTr("hasta cambio") : (session.editing ? qsTr("edición") : (session.running ? qsTr("en marcha") : qsTr("en pausa"))) }
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

    implicitHeight: collapsed || minimized
                    ? tabRow.height + 1
                    : tabRow.height + body.implicitHeight + 1

    // `Esc` folds the minimized body before the key reaches the sheet, which uses it to
    // leave simulation mode.
    Shortcut {
        sequence: "Esc"
        enabled: ribbon.minimized && ribbon.bodyOpen
        onActivated: ribbon.bodyOpen = false
    }

    /// The paper choices as commands, for the collapsed menu and the self test. They are
    /// rebuilt on every read so the mark follows the sheet instead of a frozen snapshot.
    function paperActions() {
        const out = session.papers().map(function (paper, index) {
            return {
                id: "paper-" + index,
                label: paper.label,
                icon: "x-office-document",
                hint: paper.label + " · " + paper.size,
                current: session.sheetPaper === index
            };
        });
        out.push({
            id: "paper-custom",
            label: qsTr("Personalizado…"),
            icon: "document-properties",
            hint: qsTr("ancho y alto de la hoja, guardados con el proyecto"),
            current: session.sheetPaper < 0
        });
        return out;
    }

    function orientationActions() {
        return [
            { id: "orientation-portrait", label: qsTr("Vertical"), icon: "x-office-document", hint: qsTr("la hoja más alta que ancha"), current: !session.sheetLandscape },
            { id: "orientation-landscape", label: qsTr("Apaisado"), icon: "object-rotate-right", hint: qsTr("la hoja girada; el dibujo es el mismo"), current: session.sheetLandscape }
        ];
    }

    function marginActions() {
        const out = session.marginPresets().map(function (preset, index) {
            return {
                id: "margin-" + index,
                label: preset.label,
                icon: "x-office-document",
                hint: preset.detail,
                current: session.sheetMarginPreset === index
            };
        });
        out.push({
            id: "margin-custom",
            label: qsTr("Márgenes personalizados…"),
            icon: "document-properties",
            hint: qsTr("los cuatro márgenes, en milímetros"),
            current: session.sheetMarginPreset < 0
        });
        return out;
    }

    function recentActions() {
        if (session.recent.length === 0) {
            return [{
                id: "recent-empty",
                label: qsTr("Ninguno"),
                icon: "document-open-recent",
                hint: qsTr("los archivos abiertos o guardados aparecen aquí"),
                enabled: false
            }];
        }
        return session.recent.map(function (entry, index) {
            return {
                id: "recent:" + index,
                label: entry.name,
                icon: "document-open-recent",
                hint: entry.path
            };
        });
    }

    /// The commands of one group, whether it draws buttons or a custom control.
    function groupActions(group) {
        if (group.custom === "page-setup") {
            return marginActions().concat(orientationActions(), paperActions(), [{
                id: "page-setup-file",
                label: qsTr("Configurar página…"),
                icon: "document-properties",
                hint: qsTr("tamaño, márgenes, cuadrícula y tamaño personalizado")
            }]);
        }
        if (group.custom === "recent") {
            return recentActions();
        }
        return group.items || [];
    }

    /// `--ribbon-flyout` names a group; a short timer lets the row measure and collapse
    /// before the flyout opens, so the capture shows the real resting layout.
    Timer {
        id: flyoutRequestTimer
        interval: 80
        repeat: false
        onTriggered: ribbon.openGroupFlyout(ribbon.flyoutRequest)
    }

    onFlyoutRequestChanged: if (ribbon.flyoutRequest.length > 0) {
                               flyoutRequestTimer.restart();
                           }

    /// The row stores what each group measures before the collapse is decided. Replacing
    /// the map is what wakes the `collapsedTitles` binding; mutating it would not.
    function recordGroupMetrics(index, title, expanded, compact) {
        const next = {};
        const keys = Object.keys(ribbon.groupMetrics);
        for (let i = 0; i < keys.length; ++i) {
            next[keys[i]] = ribbon.groupMetrics[keys[i]];
        }
        next[index] = { title: title, expanded: expanded, compact: compact };
        ribbon.groupMetrics = next;
    }

    /// The groups the open tab has to collapse so the one row fits its content. A group
    /// with no command has nothing to put in a flyout, so it is not collapsible: Paleta's
    /// note stays on the row and the next priority takes its turn. The order is greedy and
    /// deterministic, from the least to the most used.
    function computeCollapsedTitles() {
        const groups = ribbon.tabs[ribbon.current].groups || [];
        const entries = [];
        for (let i = 0; i < groups.length; ++i) {
            const measured = ribbon.groupMetrics[i];
            if (!measured) {
                return [];
            }
            entries.push({ title: measured.title, expanded: measured.expanded,
                           compact: measured.compact, group: groups[i] });
        }
        let total = 0;
        for (let i = 0; i < entries.length; ++i) {
            total += entries[i].expanded;
        }
        const available = ribbon.width - 6;
        const ordered = [];
        for (let p = 0; p < ribbon.collapsePriority.length; ++p) {
            const want = ribbon.collapsePriority[p];
            for (let i = 0; i < entries.length; ++i) {
                if (entries[i].title === want && ordered.indexOf(entries[i]) < 0) {
                    ordered.push(entries[i]);
                }
            }
        }
        // A tab whose groups are not in the Editar table (the other tabs) collapses from
        // the right, so the leftmost, always-present groups stay readable.
        for (let i = entries.length - 1; i >= 0; --i) {
            if (ordered.indexOf(entries[i]) < 0) {
                ordered.push(entries[i]);
            }
        }
        const collapsed = [];
        for (let i = 0; i < ordered.length; ++i) {
            if (total <= available) {
                break;
            }
            const entry = ordered[i];
            // A group with no command has nothing to put in a flyout, so it keeps its
            // (narrow) note on the row and the next priority takes its turn.
            if (!ribbon.groupHasCommands(entry.group)) {
                continue;
            }
            if (entry.expanded <= entry.compact) {
                continue;
            }
            total -= entry.expanded - entry.compact;
            collapsed.push(entry.title);
        }
        return collapsed;
    }

    function groupHasCommands(group) {
        if (!group) {
            return false;
        }
        return ribbon.groupActions(group).length > 0;
    }

    readonly property var collapsedTitles: ribbon.computeCollapsedTitles()

    function groupIsCollapsed(title) {
        return ribbon.collapsedTitles.indexOf(title) >= 0;
    }

    /// The live delegate of a group, for `Alt+D` and the capture hook.
    function groupFor(title) {
        const kids = groupsRow.children;
        for (let i = 0; i < kids.length; ++i) {
            if (kids[i].objectName === "ribbonGroup" && kids[i].title === title) {
                return kids[i];
            }
        }
        return null;
    }

    function openGroupFlyout(title) {
        const group = ribbon.groupFor(title);
        if (group) {
            group.openFlyout();
        }
    }

    function invoke(id) {
        // A command ends the minimized body: the panel was a visit to the commands, not a
        // new resting state.
        if (ribbon.minimized) {
            ribbon.bodyOpen = false;
        }
        if (id === "catalog") {
            ribbon.panelCommand("catalog");
            return;
        }
        if (id === "right-dock") {
            ribbon.panelCommand("right-dock");
            return;
        }
        if (id === "auto-hide") {
            ribbon.panelCommand("auto-hide");
            return;
        }
        if (id === "reset-view") {
            ribbon.panelCommand("reset-view");
            return;
        }
        if (id === "home") {
            if (ribbon.host) {
                ribbon.host.openStartView();
            }
            return;
        }
        if (id === "circuit-new") {
            session.addCircuit();
            return;
        }
        if (id === "circuit-rename") {
            if (ribbon.host) {
                ribbon.host.openRenameCircuit();
            }
            return;
        }
        if (id === "circuit-close") {
            if (ribbon.host) {
                ribbon.host.requestActiveCircuitRemoval();
            }
            return;
        }
        if (id === "paper-custom") {
            pageSetupRequested("size");
            return;
        }
        if (id === "margin-custom") {
            pageSetupRequested("margins");
            return;
        }
        if (id === "page-setup-file") {
            pageSetupRequested("page");
            return;
        }
        if (id === "export-panel") {
            if (ribbon.host) {
                ribbon.host.exportPanelOpen = true;
                ribbon.host.pageSetupOpen = false;
            }
            return;
        }
        if (id === "orientation-portrait") {
            session.setSheetLandscape(false);
            return;
        }
        if (id === "orientation-landscape") {
            session.setSheetLandscape(true);
            return;
        }
        if (id.indexOf("paper-") === 0) {
            const index = parseInt(id.slice(6), 10);
            if (!isNaN(index)) {
                session.setSheetPaper(index);
            }
            return;
        }
        if (id.indexOf("margin-") === 0) {
            const index = parseInt(id.slice(7), 10);
            if (!isNaN(index)) {
                session.applyMarginPreset(index);
            }
            return;
        }
        if (id === "edit-block" || id === "block-content") {
            if (ribbon.host) {
                ribbon.host.titleBlockOpen = true;
            }
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
        } else if (id === "tube-diameter") {
            // The folded ribbon reaches the same gallery the body's control owns; the tab
            // switch is already Editar, where the row appears.
            ribbon.openTubeMenu();
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
            session.reloadTheme();
        } else if (id === "zoom-in") {
            session.zoomIn();
        } else if (id === "zoom-out") {
            session.zoomOut();
        } else if (id === "zoom-reset") {
            session.zoomReset();
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
                const items = groupActions(groups[g]);
                for (let i = 0; i < items.length; ++i) {
                    // A panel command toggles, so it is invoked twice: the handler must
                    // run, and the self test must leave the window as it found it.
                    const panel = items[i].panel === true;
                    invoke(items[i].id);
                    if (panel) {
                        invoke(items[i].id);
                    }
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
            const items = groupActions(groups[g]);
            for (let i = 0; i < items.length; ++i) {
                out.push(items[i]);
            }
            // The tube diameter is a control, not a command, so the flattened list has no
            // row for it. The folded ribbon still has to reach it: one row opens the same
            // gallery, with the size in force in its name. The control itself stays loaded
            // in the hidden body, so Alt+D and this row share it.
            if (groups[g].custom === "tube") {
                out.push({
                    id: "tube-diameter",
                    label: ribbon.tubeControl
                           ? qsTr("Diámetro: %1").arg(ribbon.tubeControl.shown)
                           : qsTr("Diámetro"),
                    icon: "tube-diameter",
                    hint: qsTr("abre la serie métrica y la de pulgadas · Alt+D")
                });
            }
        }
        return out;
    }

    /// Every icon name the ribbon uses, once, for `--check-icons`. The menu button is not
    /// a tab command, so it is added here.
    function iconNames() {
        const out = [];
        for (let t = 0; t < tabs.length; ++t) {
            const groups = tabs[t].groups || [];
            for (let g = 0; g < groups.length; ++g) {
                const items = groupActions(groups[g]);
                for (let i = 0; i < items.length; ++i) {
                    const name = items[i].icon || "";
                    if (name.length > 0 && out.indexOf(name) < 0) {
                        out.push(name);
                    }
                }
            }
        }
        if (out.indexOf("open-menu-symbolic") < 0) {
            out.push("open-menu-symbolic");
        }
        // The diameter row is generated by the tube group, not listed in its items, so
        // the icon it shows is declared here for the same check.
        if (out.indexOf("tube-diameter") < 0) {
            out.push("tube-diameter");
        }
        // The collapsed-group chevron is not a tab command but it is a theme icon the
        // ribbon asks for, so the icon check covers it.
        if (out.indexOf("pan-down") < 0) {
            out.push("pan-down");
        }
        return out;
    }

    // ---- tabs ---------------------------------------------------------------
    Item {
        id: tabRow
        width: parent.width
        height: ribbon.tabHeight

        // The tab row is the raised plane of the ribbon; the body below returns to the
        // window plane, so the active tab reads as part of the body.
        Rectangle {
            anchors.fill: parent
            color: session.surfaceRaised
        }
        Rectangle {
            anchors.bottom: parent.bottom
            width: parent.width
            height: 1
            color: session.line
        }

        Row {
            id: tabsRow
            anchors.left: parent.left
            anchors.leftMargin: 8
            anchors.verticalCenter: parent.verticalCenter
            spacing: 2

            Repeater {
                model: ribbon.tabs

                delegate: Item {
                    id: tab
                    required property int index
                    required property var modelData

                    readonly property bool active: ribbon.current === index

                    width: tabText.implicitWidth + 26
                    height: ribbon.tabHeight

                    Accessible.role: Accessible.PageTab
                    Accessible.name: modelData.name
                    activeFocusOnTab: true

                    // The active tab reaches the body and drops its bottom corners, so it
                    // is the same surface as the ribbon below instead of a card cut out of
                    // it. The accent underline runs the full width at the seam.
                    Rectangle {
                        id: tabSurface
                        anchors.top: parent.top
                        anchors.topMargin: tab.active ? 0 : 3
                        width: parent.width
                        height: tab.active ? parent.height : 28
                        radius: session.radiusSmall
                        topLeftRadius: 4
                        topRightRadius: 4
                        bottomLeftRadius: tab.active ? 0 : 4
                        bottomRightRadius: tab.active ? 0 : 4
                        color: tab.active ? session.background
                               : tabPointer.pressed ? session.surfaceSunken
                               : tabPointer.containsMouse ? session.surfaceRaised
                               : "transparent"
                        border.width: tab.activeFocus ? 1 : 0
                        border.color: session.accent
                    }

                    Text {
                        id: tabText
                        anchors.centerIn: parent
                        text: modelData.name
                        color: tab.active ? session.foreground : session.muted
                        font.pixelSize: session.textLabel
                    }

                    Rectangle {
                        anchors.bottom: parent.bottom
                        width: parent.width
                        height: 2
                        color: tab.active ? session.accent : "transparent"
                    }

                    MouseArea {
                        id: tabPointer
                        anchors.fill: parent
                        hoverEnabled: true
                        onClicked: {
                            ribbon.current = index;
                            ribbon.menuOpen = false;
                            if (ribbon.minimized) {
                                ribbon.bodyOpen = true;
                            }
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

        // Folding the ribbon to its tab row and bringing it back: the same control in both
        // directions, an icon for the shape and a name for the reader.
        RibbonButton {
            visible: !ribbon.collapsed
            anchors.right: helpRow.left
            anchors.rightMargin: 12
            anchors.verticalCenter: parent.verticalCenter
            session: ribbon.session
            label: ""
            accessibleName: ribbon.minimized ? qsTr("Mostrar la cinta") : qsTr("Minimizar la cinta")
            iconName: ribbon.minimized ? "go-top" : "go-bottom"
            hint: ribbon.minimized
                  ? qsTr("Mostrar la cinta · Esc la vuelve a plegar")
                  : qsTr("Minimizar la cinta · deja las pestañas a la vista")
            large: false
            tone: "muted"
            onTriggered: ribbon.minimizeToggled()
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
        objectName: "ribbonBody"
        anchors.top: tabRow.bottom
        anchors.left: parent.left
        anchors.leftMargin: ribbon.minimized ? ribbon.bodyInsetLeft : 0
        anchors.right: parent.right
        anchors.rightMargin: ribbon.minimized ? ribbon.bodyInsetRight : 0
        implicitHeight: Math.max(86, 4 * session.unit + 14)
        visible: ribbon.bodyShown

        Rectangle {
            anchors.fill: parent
            // Folded, the body is a panel in front of the sheet, so it takes the card's
            // surface instead of the ribbon's own plane.
            color: ribbon.minimized ? session.panel : session.background
        }

        // Over the sheet the body needs an edge, or it reads as part of the drawing.
        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            height: 1
            color: session.line
            visible: ribbon.minimized
        }

        // A panel in front of the sheet casts the theme's shadow; without it the catalog's
        // header looks cut rather than covered.
        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.bottom
            height: 4
            color: session.shadow
            visible: ribbon.minimized
        }

        Row {
            id: groupsRow
            objectName: "ribbonGroupsRow"
            anchors.left: parent.left
            anchors.leftMargin: 6
            anchors.top: parent.top
            spacing: 0

            Repeater {
                model: ribbon.tabs[ribbon.current].groups

                delegate: RibbonGroup {
                    required property int index
                    required property var modelData
                    objectName: "ribbonGroup"
                    session: ribbon.session
                    ribbonBar: ribbon
                    groupData: modelData
                    groupIndex: index
                    title: modelData.title
                    collapsed: ribbon.groupIsCollapsed(modelData.title)
                }
            }
        }
    }

    // ---- collapsed menu -----------------------------------------------------
    Rectangle {
        visible: menu.visible
        anchors.top: menu.top
        anchors.left: menu.left
        anchors.topMargin: 3
        anchors.leftMargin: 3
        width: menu.width
        height: menu.height
        radius: session.radiusSmall
        color: session.shadow
        z: 19
    }

    Rectangle {
        id: menu
        objectName: "ribbonMenu"
        visible: ribbon.collapsed && ribbon.menuOpen
        anchors.top: tabRow.bottom
        anchors.left: parent.left
        anchors.leftMargin: 8
        width: 240
        height: Math.min(menuColumn.implicitHeight + 12, ribbon.menuMaxHeight)
        color: session.surfaceRaised
        radius: session.radiusSmall
        border.width: 1
        border.color: session.line
        z: 20

        Flickable {
            anchors.fill: parent
            contentHeight: menuColumn.implicitHeight + 12
            clip: true

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
    }

    Rectangle {
        anchors.bottom: parent.bottom
        width: parent.width
        height: 1
        color: session.line
    }

    /// The window shortcut `Alt+D`: bring the Editar tab up, then open the list. When the
    /// Conducto group is collapsed the control lives in its flyout, so the flyout opens
    /// first and the diameter gallery follows it instead of pointing at a hidden control.
    function openTubeMenu() {
        ribbon.current = 1;
        Qt.callLater(function () {
            const conducto = ribbon.groupFor("Conducto");
            if (conducto && conducto.collapsed) {
                conducto.openFlyout();
                Qt.callLater(function () {
                    if (ribbon.flyoutTubeControl) {
                        ribbon.flyoutTubeControl.openPopup();
                    }
                });
            } else if (ribbon.tubeControl) {
                ribbon.tubeControl.openPopup();
            }
        });
    }
}
