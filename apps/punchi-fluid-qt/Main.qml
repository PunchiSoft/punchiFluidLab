// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The three regions of docs/ui-ux.md: ribbon on top, sidebar that mirrors the catalog,
// sheet. A fourth region needs an ADR. Every colour comes from the palette the session
// read, so there is no literal here either.

import QtQuick
import PunchiFluid

Window {
    id: window
    width: 1200
    height: 780
    minimumWidth: 900
    minimumHeight: 560
    visible: true
    title: qsTr("Punchi Fluid Lab")
    color: session.background

    FluidSession {
        id: session
    }

    // The normal path needs no modal; the valve is one key away.
    Shortcut {
        sequence: "Space"
        onActivated: session.toggleValve()
    }

    // 60 Hz of simulated time in a timer rather than in the paint loop, so a slow frame
    // does not change how far the model advances. The timer stops when the model settles,
    // so an idle window neither redraws nor keeps the compositor awake.
    Timer {
        interval: 16
        running: !session.settled
        repeat: true
        onTriggered: session.advance(0.016)
    }

    Column {
        anchors.fill: parent

        Ribbon {
            id: ribbon
            width: parent.width
            height: 104
            session: session
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
            hint: sidebar.selectedName.length > 0
                  ? qsTr("Elegido en el catálogo: %1 — colocarlo en el lienzo llega en M3")
                        .arg(sidebar.selectedName)
                  : qsTr("Espacio acciona la válvula; un clic sobre ella la acciona igual")
        }
    }
}
