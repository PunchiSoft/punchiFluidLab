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

    /// Below this width a ribbon is worse than a menu, so it folds away.
    readonly property bool collapsed: width < 1100
    property int current: 3
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
            groups: [{ title: qsTr("Documento"), note: qsTr("Guardar, abrir y exportar llegan en M3") }]
        },
        {
            name: qsTr("Editar"),
            groups: [{ title: qsTr("Historial"), note: qsTr("Deshacer y rehacer llegan con el editor") }]
        },
        {
            name: qsTr("Ver"),
            groups: [
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
            name: qsTr("Simular"),
            groups: [
                {
                    title: qsTr("Modo"),
                    items: [
                        { id: "edit", label: qsTr("Editar"), icon: "document-properties", hint: qsTr("un clic sobre la válvula abre sus propiedades"), big: true, current: session.editing },
                        { id: "simulate", label: qsTr("Simular"), icon: "media-playback-start", hint: qsTr("un clic sobre la válvula la acciona"), big: true, current: !session.editing }
                    ]
                },
                {
                    title: qsTr("Paso a paso"),
                    items: [
                        { id: "step", label: qsTr("Paso"), icon: "media-seek-forward", hint: qsTr("F10 · un cuadro y se queda en pausa"), big: true },
                        { id: "run", label: qsTr("Animar"), icon: "media-playback-start", hint: qsTr("F5 · reproduce o pausa"), big: true }
                    ]
                },
                {
                    title: qsTr("Velocidad"),
                    items: [
                        { id: "speed025", label: qsTr("0,25×"), hint: qsTr("un cuarto de tiempo real") },
                        { id: "speed1", label: qsTr("1×"), hint: qsTr("tiempo real") },
                        { id: "speed2", label: qsTr("2×"), hint: qsTr("el doble de rápido") }
                    ]
                }
            ]
        }
    ]

    implicitHeight: collapsed ? tabRow.height + 1 : tabRow.height + body.implicitHeight + 1

    function invoke(id) {
        if (id === "valve") {
            session.toggleValve();
        } else if (id === "restart") {
            session.restart();
        } else if (id === "edit") {
            session.editing = true;
        } else if (id === "simulate") {
            session.editing = false;
            session.setRunning(true);
        } else if (id === "step") {
            session.stepForward();
        } else if (id === "run") {
            session.toggleRunning();
        } else if (id === "speed025") {
            session.speed = 0.25;
        } else if (id === "speed1") {
            session.speed = 1.0;
        } else if (id === "speed2") {
            session.speed = 2.0;
        } else if (id === "scheme") {
            session.loadTheme();
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

            RibbonButton {
                session: ribbon.session
                label: qsTr("Reproducir")
                iconName: "media-playback-start"
                hint: session.editing ? qsTr("entra en modo Simular")
                                      : (session.running ? qsTr("ya está en marcha") : qsTr("F5 · animar el modelo"))
                large: false
                tone: session.running ? "muted" : "accent"
                available: !session.editing && !session.running
                onTriggered: session.setRunning(true)
            }

            RibbonButton {
                session: ribbon.session
                label: qsTr("Pausa")
                iconName: "media-playback-pause"
                hint: session.editing ? qsTr("entra en modo Simular")
                                      : (session.running ? qsTr("F5 · detener el modelo donde está") : qsTr("ya está en pausa"))
                large: false
                available: !session.editing && session.running
                onTriggered: session.setRunning(false)
            }

            RibbonButton {
                session: ribbon.session
                label: qsTr("Paso")
                iconName: "media-seek-forward"
                hint: session.editing ? qsTr("entra en modo Simular")
                                      : qsTr("F10 · un cuadro de %1 s y se queda en pausa")
                                            .arg((session.frameSeconds * session.speed).toFixed(3))
                large: false
                available: !session.editing
                onTriggered: session.stepForward()
            }

            RibbonButton {
                session: ribbon.session
                label: qsTr("Reiniciar")
                iconName: "view-refresh"
                hint: qsTr("Ctrl+R · vuelve a t = 0")
                large: false
                onTriggered: session.restart()
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
                            selected: modelData.current === true
                            tone: "foreground"
                            onTriggered: ribbon.invoke(modelData.id)
                        }
                    }

                    // A tab whose commands do not exist yet says so once, rather than
                    // showing grey buttons that teach the user nothing.
                    Item {
                        visible: !!modelData.note
                        width: visible ? Math.min(260, noteText.implicitWidth) : 0
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
