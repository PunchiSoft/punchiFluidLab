// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The sidebar content is the catalog: a search, the categories the definitions declare,
// and every entry shown by its own symbol. The title, the background and the hide control
// belong to `DockPanel`.
//
// The group label is translated in the session, so this file never learns the numeric
// category enum. The collapsed groups are view state and are remembered per user.

import QtQuick
import QtQuick.Controls
import PunchiFluid

Item {
    id: sidebar
    property FluidSession session
    property string selectedName: ""
    property int selectedIndex: -1
    /// `true` per category key while the group is open; missing means open.
    property var expanded: ({})
    property string searchText: ""
    /// Entries that match the current search; the footer and the empty state read it.
    readonly property int matchCount: {
        const text = sidebar.searchText;
        let count = 0;
        const groups = session.catalogGroups;
        for (let g = 0; g < groups.length; ++g) {
            const entries = groups[g].entries;
            for (let i = 0; i < entries.length; ++i) {
                if (sidebar.matches(entries[i], text)) {
                    ++count;
                }
            }
        }
        return count;
    }

    function fold(text) {
        return String(text).toLowerCase()
            .replace(/[áàäâ]/g, "a")
            .replace(/[éèëê]/g, "e")
            .replace(/[íìïî]/g, "i")
            .replace(/[óòöô]/g, "o")
            .replace(/[úùüû]/g, "u")
            .replace(/ñ/g, "n");
    }

    /// The few words a student types that are not in the definition's own name.
    function synonyms(id) {
        if (id.indexOf("valve") >= 0) return "valvula";
        if (id.indexOf("cylinder") >= 0) return "cilindro";
        if (id.indexOf("compressor") >= 0) return "compresor";
        if (id.indexOf("supply") >= 0) return "fuente aire";
        if (id.indexOf("gauge") >= 0) return "manometro presion";
        if (id.indexOf("flow") >= 0) return "caudalimetro caudal";
        if (id.indexOf("silencer") >= 0) return "silenciador";
        if (id.indexOf("plug") >= 0) return "tapón ciego";
        if (id.indexOf("relay") >= 0) return "rele rele";
        if (id.indexOf("switch") >= 0) return "interruptor contacto";
        if (id.indexOf("lamp") >= 0) return "lampara piloto";
        return "";
    }

    function matches(row, query) {
        const text = fold(query || "").trim();
        if (text.length === 0) {
            return true;
        }
        const haystack = fold(row.name + " " + row.id + " " + synonyms(row.id));
        return text.split(/\s+/).every(function (part) {
            return haystack.indexOf(part) >= 0;
        });
    }

    function firstMatch(query) {
        const groups = session.catalogGroups;
        for (let g = 0; g < groups.length; ++g) {
            const entries = groups[g].entries;
            for (let i = 0; i < entries.length; ++i) {
                if (matches(entries[i], query)) {
                    return entries[i];
                }
            }
        }
        return null;
    }

    function isExpanded(key) {
        return expanded[key] !== false;
    }

    function toggleGroup(key) {
        let next = {};
        for (const current in expanded) {
            next[current] = expanded[current];
        }
        next[key] = !isExpanded(key);
        expanded = next;
        session.setViewSetting("catalog/group-" + key, next[key]);
    }

    Component.onCompleted: {
        let map = {};
        const groups = session.catalogGroups;
        for (let i = 0; i < groups.length; ++i) {
            const key = groups[i].key;
            map[key] = session.viewSetting("catalog/group-" + key, true);
        }
        expanded = map;
    }

    Shortcut {
        sequence: "Ctrl+F"
        onActivated: {
            search.forceActiveFocus();
            search.selectAll();
        }
    }

    Text {
        id: count
        x: 14
        y: 8
        text: session.catalog.length === 1
              ? qsTr("1 componente")
              : qsTr("%1 componentes").arg(session.catalog.length)
        color: session.muted
        font.pixelSize: 10
    }

    TextField {
        id: search
        objectName: "catalogSearch"
        x: 14
        width: parent.width - 28
        height: 28
        anchors.top: count.bottom
        anchors.topMargin: 4
        placeholderText: qsTr("Buscar…")
        placeholderTextColor: session.muted
        color: session.foreground
        font.pixelSize: 12
        selectByMouse: true
        Accessible.name: qsTr("Buscar en el catálogo")
        background: Rectangle {
            radius: 4
            color: session.surfaceSunken
            border.width: 1
            border.color: search.activeFocus ? session.accent : session.lineStrong
        }
        onTextChanged: sidebar.searchText = text
        onAccepted: {
            const row = sidebar.firstMatch(search.text);
            if (row) {
                sheet.placeCatalogAt(row.index, Qt.point(sheet.width / 2, sheet.height / 2));
            }
        }
        Keys.onEscapePressed: {
            search.text = "";
            sheet.forceActiveFocus();
        }
    }

    Flickable {
        id: scroller
        anchors.top: search.bottom
        anchors.topMargin: 8
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: footer.top
        contentHeight: groupsColumn.implicitHeight
        clip: true

        Column {
            id: groupsColumn
            width: scroller.width
            spacing: 2

            Repeater {
                model: session.catalogGroups

                delegate: Column {
                    id: group
                    required property var modelData
                    width: groupsColumn.width
                    spacing: 0
                    readonly property var shown: {
                        const text = sidebar.searchText;
                        if (text.length === 0) {
                            return modelData.entries;
                        }
                        return modelData.entries.filter(function (row) {
                            return sidebar.matches(row, text);
                        });
                    }
                    visible: shown.length > 0

                    Rectangle {
                        width: parent.width
                        height: 28
                        color: headerPointer.containsMouse || activeFocus
                               ? session.surfaceRaised : "transparent"
                        border.width: activeFocus ? 1 : 0
                        border.color: session.accent
                        activeFocusOnTab: true

                        Keys.onSpacePressed: sidebar.toggleGroup(group.modelData.key)
                        Keys.onReturnPressed: sidebar.toggleGroup(group.modelData.key)
                        Keys.onEnterPressed: sidebar.toggleGroup(group.modelData.key)

                        Row {
                            anchors.left: parent.left
                            anchors.leftMargin: 10
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: 6

                            Text {
                                text: sidebar.isExpanded(group.modelData.key) ? "▾" : "▸"
                                color: session.muted
                                font.pixelSize: 10
                            }
                            Text {
                                text: group.modelData.label
                                color: session.foreground
                                font.pixelSize: 11
                                font.bold: true
                            }
                            Text {
                                text: sidebar.searchText.length > 0
                                      ? group.shown.length + " / " + group.modelData.entries.length
                                      : String(group.modelData.entries.length)
                                color: session.muted
                                font.pixelSize: 10
                            }
                        }

                        MouseArea {
                            id: headerPointer
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: sidebar.toggleGroup(group.modelData.key)
                        }

                        Accessible.role: Accessible.Button
                        Accessible.name: group.modelData.label
                        Accessible.description: sidebar.isExpanded(group.modelData.key)
                                                ? qsTr("plegar el grupo")
                                                : qsTr("desplegar el grupo")
                    }

                    Repeater {
                        model: sidebar.isExpanded(group.modelData.key) ? group.shown : []

                        delegate: Rectangle {
                            id: row
                            required property var modelData
                            width: group.width
                            height: 60
                            color: sidebar.selectedIndex === modelData.index
                                   ? session.panel
                                   : rowPointer.containsMouse ? session.surfaceRaised
                                   : "transparent"

                            Rectangle {
                                anchors.left: parent.left
                                width: 3
                                height: parent.height
                                visible: sidebar.selectedIndex === modelData.index
                                color: session.accent
                            }

                            PreviewItem {
                                anchors.left: parent.left
                                anchors.leftMargin: 12
                                anchors.verticalCenter: parent.verticalCenter
                                width: 48
                                height: 48
                                session: sidebar.session
                                index: modelData.index
                            }

                            Column {
                                anchors.left: parent.left
                                anchors.leftMargin: 70
                                anchors.right: parent.right
                                anchors.rightMargin: 8
                                anchors.verticalCenter: parent.verticalCenter
                                spacing: 3

                                Text {
                                    width: parent.width
                                    text: modelData.name
                                    color: session.foreground
                                    font.pixelSize: 13
                                    elide: Text.ElideRight
                                }
                                Text {
                                    width: parent.width
                                    text: modelData.id
                                    color: session.muted
                                    font.pixelSize: 11
                                    elide: Text.ElideRight
                                }
                            }

                            MouseArea {
                                id: rowPointer
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: session.editing ? Qt.OpenHandCursor : Qt.ArrowCursor

                                property bool dragging: false

                                onClicked: {
                                    sidebar.selectedIndex = modelData.index;
                                    sidebar.selectedName = modelData.name;
                                }

                                // Dragging out of the palette is the editor's gesture. It
                                // is done by hand rather than with Qt's drag and drop so
                                // one code path serves the pointer and anything that drives
                                // the window, and a drop has one testable entry point.
                                onPressed: if (session.editing) {
                                               dragging = true;
                                               window.dragCatalog = modelData.index;
                                               window.dragPoint = mapToItem(null, mouse.x, mouse.y);
                                           }
                                onPositionChanged: if (dragging) {
                                                       window.dragPoint = mapToItem(null, mouse.x, mouse.y);
                                                   }
                                onReleased: {
                                    if (!dragging) {
                                        return;
                                    }
                                    dragging = false;
                                    const landing = mapToItem(sheet, mouse.x, mouse.y);
                                    if (sheet.contains(landing)) {
                                        sheet.placeCatalogAt(window.dragCatalog, landing);
                                    }
                                    window.dragCatalog = -1;
                                }
                                onCanceled: {
                                    dragging = false;
                                    window.dragCatalog = -1;
                                }
                            }
                        }
                    }
                }
            }
        }

        Text {
            anchors.centerIn: parent
            visible: sidebar.searchText.length > 0 && sidebar.firstMatch(sidebar.searchText) === null
            text: qsTr("Sin resultados")
            color: session.muted
            font.pixelSize: 12
        }
    }

    Rectangle {
        id: footer
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        height: 44
        color: session.surfaceRaised

        Rectangle {
            anchors.top: parent.top
            width: parent.width
            height: 1
            color: session.line
        }

        Column {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.leftMargin: 12
            anchors.rightMargin: 12
            anchors.verticalCenter: parent.verticalCenter
            spacing: 2

            Text {
                width: parent.width
                text: session.catalogDir.length > 0
                      ? session.catalogDir
                      : qsTr("Catálogo integrado")
                color: session.muted
                font.pixelSize: 10
                elide: Text.ElideMiddle
            }
            Text {
                text: session.catalogProblems === 0
                      ? qsTr("Sin archivos omitidos")
                      : session.catalogProblems === 1
                        ? qsTr("1 archivo omitido")
                        : qsTr("%1 archivos omitidos").arg(session.catalogProblems)
                color: session.catalogProblems === 0 ? session.muted : session.foreground
                font.pixelSize: 10
            }
        }
    }
}
