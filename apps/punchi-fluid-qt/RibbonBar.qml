// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The ribbon: tabs on top, the commands of the open tab in labelled groups underneath, and
// the transport pinned to the right so running the model never needs a tab switch.
//
// The tabs are the four register of docs/ui-ux.md. A tab whose commands do not exist yet
// says so in one line instead of showing buttons that do nothing, and a command that is
// not implemented is not on screen at all.

import QtQuick
import PunchiFluid

Rectangle {
    id: ribbon
    property FluidSession session

    /// The Archivo commands that need a file dialog, which belongs to the window.
    signal fileCommand(string command)

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
                        { id: "export", label: qsTr("Exportar"), icon: "document-export", hint: qsTr("Ctrl+E · el plano en la hoja que elijas, PDF o PNG"), big: true }
                    ]
                },
                {
                    // The paper the sheet will be printed on. The list is the export's, so the
                    // panel cannot offer a size the writer does not know; the one in use is
                    // marked, because a list of nine without a mark is a guess.
                    title: qsTr("Hoja"),
                    items: session.paperNames().map(function (name, index) {
                        return {
                            id: "paper-" + index,
                            label: name,
                            icon: "x-office-document",
                            hint: qsTr("exportar en %1").arg(name),
                            current: window.exportPaper === index
                        };
                    }).concat([{
                        id: "paper-landscape",
                        label: qsTr("Apaisado"),
                        icon: "object-rotate-right",
                        hint: qsTr("la hoja girada; el dibujo es el mismo"),
                        current: window.exportLandscape
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
                        { id: "align-bottom", label: qsTr("Abajo"), icon: "align-vertical-bottom", hint: qsTr("sobre el objeto más bajo"), enabled: session.selectionCount > 1 }
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
                        { id: "zoom-in", label: qsTr("Aumentar"), icon: "zoom-in", hint: qsTr("> · acerca la hoja") },
                        { id: "zoom-out", label: qsTr("Disminuir"), icon: "zoom-out", hint: qsTr("< · aleja la hoja") },
                        { id: "zoom-reset", label: qsTr("Tamaño original"), icon: "zoom-original", hint: qsTr("Ctrl+0 · quita el zoom") },
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

    ]

    implicitHeight: collapsed ? tabRow.height + 1 : tabRow.height + body.implicitHeight + 1

    function invoke(id) {
        if (id.indexOf("paper-") === 0) {
            const index = parseInt(id.slice(6), 10);
            if (!isNaN(index)) {
                window.exportPaper = index;
            }
            return;
        }
        if (id === "paper-landscape") {
            window.exportLandscape = !window.exportLandscape;
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
        }
    }

    /// Runs every command once, so a broken handler fails a test instead of a user.
    ///
    /// A `TypeError` in an `onTriggered` handler is invisible until the button is pressed;
    /// this is the only way to press them all without a pointer.
    function selfTest() {
        const transport = ["start", "pause", "step", "until-change", "stop", "reset", "zoom-in", "zoom-out",
                           "zoom-reset", "grid"];
        for (let t = 0; t < tabs.length; ++t) {
            const groups = tabs[t].groups || [];
            for (let g = 0; g < groups.length; ++g) {
                const items = groups[g].items || [];
                for (let i = 0; i < items.length; ++i) {
                    invoke(items[i].id);
                }
            }
        }
        for (let i = 0; i < transport.length; ++i) {
            invoke(transport[i]);
        }
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

        // ---- transport, always visible ---------------------------------------
        // Start, pause, step, reset: the four commands of a simulation transport. Start and
        // pause are each available only in the state where they mean something, and the
        // hint says why when they are not — a grey button is only acceptable when it
        // explains itself.
        Row {
            anchors.right: parent.right
            anchors.rightMargin: 8
            anchors.verticalCenter: parent.verticalCenter
            spacing: 2

            // Start, pause, step, stop, reset: the transport of the reference, and the only
            // thing that changes the mode. There is no "edit" button because leaving the
            // simulation *is* stopping it.
            RibbonButton {
                session: ribbon.session
                label: qsTr("Iniciar")
                iconName: "media-playback-start"
                hint: session.running ? qsTr("ya está en marcha") : qsTr("F9 · anima el modelo")
                large: false
                tone: session.running ? "muted" : "accent"
                available: !session.running
                onTriggered: ribbon.invoke("start")
            }

            RibbonButton {
                session: ribbon.session
                label: qsTr("Pausa")
                iconName: "media-playback-pause"
                hint: session.editing
                      ? qsTr("F8 · entra en simulación sin arrancar")
                      : (session.running ? qsTr("F8 · detiene donde está") : qsTr("ya está en pausa"))
                large: false
                available: session.running || session.editing
                onTriggered: ribbon.invoke("pause")
            }

            RibbonButton {
                session: ribbon.session
                label: qsTr("Paso")
                iconName: "media-seek-forward"
                hint: qsTr("F10 · un cuadro de %1 s y se queda en pausa")
                      .arg((session.frameSeconds * session.speed).toFixed(3))
                large: false
                available: !session.editing
                onTriggered: ribbon.invoke("step")
            }

            RibbonButton {
                session: ribbon.session
                label: qsTr("Hasta cambio")
                iconName: "media-skip-forward"
                hint: qsTr("avanza hasta que el vástago llegue a un tope o una válvula cambie")
                large: false
                available: !session.editing
                onTriggered: ribbon.invoke("until-change")
            }

            RibbonButton {
                session: ribbon.session
                label: qsTr("Detener")
                iconName: "media-playback-stop"
                hint: session.editing ? qsTr("ya está en modo edición") : qsTr("F5 · vuelve al modo edición")
                large: false
                available: !session.editing
                onTriggered: ribbon.invoke("stop")
            }

            RibbonButton {
                session: ribbon.session
                label: qsTr("Restablecer")
                iconName: "view-refresh"
                hint: qsTr("Ctrl+R · vuelve a t = 0")
                large: false
                available: !session.editing
                onTriggered: ribbon.invoke("reset")
            }

            Rectangle {
                width: 1
                height: 18
                anchors.verticalCenter: parent.verticalCenter
                color: session.panel
            }

            Repeater {
                model: [0.25, 1.0, 2.0]

                delegate: RibbonButton {
                    required property real modelData
                    session: ribbon.session
                    label: (modelData === 0.25 ? "0,25" : modelData.toFixed(0)) + "×"
                    hint: qsTr("velocidad de simulación")
                    large: false
                    tone: Math.abs(session.speed - modelData) < 0.001 ? "accent" : "muted"
                    onTriggered: session.speed = modelData
                }
            }

            Rectangle {
                width: 1
                height: 18
                anchors.verticalCenter: parent.verticalCenter
                color: session.panel
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                rightPadding: 4
                text: session.editing
                      ? qsTr("edición")
                      : (session.running ? qsTr("en marcha") : qsTr("en pausa"))
                        + " · " + qsTr("t %1 s").arg(session.timeS.toFixed(2))
                color: session.muted
                font.pixelSize: 11
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
