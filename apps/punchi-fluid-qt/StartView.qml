// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Inicio: the start view. It covers the sheet while no project is being worked on, with the
// navigation in the catalog's place: new, open, the recent projects and what a reader needs
// to begin. It is not a ribbon tab and not a fourth region (ADR 0031).
//
// The page is read in planes, like the settings of the desktop: the page on the window, and
// each group on a raised card with its label above it. The rail marks where the reader is
// with a filled row and a bar, and the words say it too: state is never colour alone.

import QtQuick
import QtQuick.Controls
import PunchiFluid

Rectangle {
    id: start
    property FluidSession session
    /// One raster rule, two sizes: the rail's rows and the two «Empezar» cards each ask
    /// the provider for the raster they draw, instead of upscaling the 22 px fallback.
    readonly property int railIconSize: Math.round(session.unit * 1.1)
    readonly property int cardIconSize: Math.round(session.unit * 1.8)
    /// `false` while there is nothing to go back to.
    property bool canReturn: true
    readonly property var templateDefinitions: [
        { index: 0, title: qsTr("Válvula 3/2"), detail: qsTr("Cilindro de simple efecto") },
        { index: 1, title: qsTr("Válvula 5/2"), detail: qsTr("Cilindro de doble efecto") },
        { index: 2, title: qsTr("Preparación del aire"), detail: qsTr("Filtro, unidad y válvula") },
        { index: 3, title: qsTr("Control eléctrico"), detail: qsTr("Pulsador y válvula solenoide") }
    ]
    readonly property int templateCardCount: templateDefinitions.length

    signal newProject()
    signal openProject()
    signal openRecent(int slot)
    signal forgetRecent(int slot)
    signal templateRequested(int templateIndex)
    signal aboutRequested()
    signal licenceRequested()
    signal returnRequested()
    property string templateError: ""

    function activateTemplate(templateIndex) {
        if (templateIndex < 0 || templateIndex >= session.templateCount) {
            return;
        }
        start.templateError = "";
        start.templateRequested(templateIndex);
    }

    color: session.background

    Row {
        anchors.fill: parent

        // The left rail takes the catalog's place while Inicio is open. It is the same
        // region, width and tokens as the catalog, so returning to the sheet returns the
        // catalog without the window jumping.
        Rectangle {
            id: rail
            width: Math.round(session.unit * 10)
            height: parent.height
            color: session.panel

            Rectangle {
                anchors.right: parent.right
                width: 1
                height: parent.height
                color: session.line
            }

            Column {
                anchors.fill: parent
                anchors.margins: session.unit / 2
                spacing: session.unit / 3

                Text {
                    width: parent.width
                    text: qsTr("Punchi Fluid Lab")
                    color: session.foreground
                    font.pixelSize: session.textBody
                    font.bold: true
                    elide: Text.ElideRight
                }

                Item {
                    visible: start.canReturn
                    width: parent.width
                    height: session.unit * 1.6

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.left: parent.left
                        anchors.leftMargin: session.unit / 3
                        text: qsTr("‹ Volver a la hoja")
                        color: session.muted
                        font.pixelSize: session.textSmall
                    }
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: start.returnRequested()
                    }
                }

                Item { width: 1; height: session.unit / 3 }

                Text {
                    width: parent.width
                    text: qsTr("Iniciar")
                    color: session.muted
                    font.pixelSize: session.textSmall
                    font.bold: true
                    padding: session.unit / 3
                }

                // Where the reader is, said with a filled row, a bar and the word.
                Rectangle {
                    objectName: "railHome"
                    width: parent.width
                    height: session.unit * 1.9
                    radius: session.radiusControl
                    // The current view is the filled row, as in the reference. Its text and
                    // icon are the measured `onAccent`, and the row has no hover of its own,
                    // so nothing repaints the fill under them.
                    color: session.accent

                    Row {
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.left: parent.left
                        anchors.leftMargin: session.unit / 2
                        anchors.right: parent.right
                        anchors.rightMargin: session.unit / 3
                        spacing: session.unit / 3

                        Image {
                            objectName: "railHomeIcon"
                            width: start.railIconSize
                            height: width
                            sourceSize.width: start.railIconSize
                            sourceSize.height: start.railIconSize
                            fillMode: Image.PreserveAspectFit
                            source: "image://theme/go-home?tone=onaccent&v="
                                    + session.themeRevision
                            smooth: true
                        }
                        Text {
                            width: parent.width - start.railIconSize - parent.spacing
                            text: qsTr("Inicio")
                            color: session.onAccent
                            font.pixelSize: session.textBody
                            font.bold: true
                            elide: Text.ElideRight
                        }
                    }
                }

                Repeater {
                    model: [
                        { label: qsTr("Nuevo proyecto"), icon: "document-new", action: "new",
                          hint: "Ctrl+N" },
                        { label: qsTr("Abrir…"), icon: "document-open", action: "open",
                          hint: "Ctrl+O" }
                    ]
                    delegate: Rectangle {
                        width: parent.width
                        height: session.unit * 1.9
                        radius: session.radiusControl
                        color: railArea.containsMouse ? session.surfaceRaised : "transparent"

                        Accessible.role: Accessible.Button
                        Accessible.name: modelData.label
                        Accessible.onPressAction: modelData.action === "new"
                                                    ? start.newProject() : start.openProject()

                        Row {
                            anchors.verticalCenter: parent.verticalCenter
                            anchors.left: parent.left
                            anchors.leftMargin: session.unit / 2
                            anchors.right: parent.right
                            anchors.rightMargin: session.unit / 3
                            spacing: session.unit / 3

                            Image {
                                width: start.railIconSize
                                height: width
                                sourceSize.width: start.railIconSize
                                sourceSize.height: start.railIconSize
                                fillMode: Image.PreserveAspectFit
                                source: "image://theme/" + modelData.icon + "?tone=muted&v="
                                        + session.themeRevision
                                smooth: true
                            }
                            Text {
                                width: parent.width - start.railIconSize - parent.spacing
                                text: modelData.label
                                color: session.foreground
                                font.pixelSize: session.textBody
                                elide: Text.ElideRight
                            }
                        }

                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            anchors.right: parent.right
                            anchors.rightMargin: session.unit / 3
                            text: modelData.hint
                            color: session.muted
                            font.pixelSize: session.textSmall
                        }

                        MouseArea {
                            id: railArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: modelData.action === "new" ? start.newProject()
                                                                  : start.openProject()
                        }
                    }
                }

                Rectangle { width: parent.width; height: 1; color: session.line }

                Text {
                    width: parent.width
                    text: qsTr("Aprender")
                    color: session.muted
                    font.pixelSize: session.textSmall
                    font.bold: true
                    padding: session.unit / 3
                }

                Repeater {
                    model: [
                        { label: qsTr("Acerca de"), icon: "x-office-address-book" },
                        { label: qsTr("Licencia"), icon: "x-office-document" }
                    ]
                    delegate: Rectangle {
                        width: parent.width
                        height: session.unit * 1.9
                        radius: session.radiusControl
                        color: learnArea.containsMouse ? session.surfaceRaised : "transparent"

                        Accessible.role: Accessible.Button
                        Accessible.name: modelData.label
                        Accessible.onPressAction: index === 0 ? start.aboutRequested()
                                                              : start.licenceRequested()

                        Row {
                            anchors.verticalCenter: parent.verticalCenter
                            anchors.left: parent.left
                            anchors.leftMargin: session.unit / 2
                            anchors.right: parent.right
                            anchors.rightMargin: session.unit / 3
                            spacing: session.unit / 3

                            Image {
                                width: start.railIconSize
                                height: width
                                sourceSize.width: start.railIconSize
                                sourceSize.height: start.railIconSize
                                fillMode: Image.PreserveAspectFit
                                source: "image://theme/" + modelData.icon + "?tone=muted&v="
                                        + session.themeRevision
                                smooth: true
                            }
                            Text {
                                width: parent.width - start.railIconSize - parent.spacing
                                text: modelData.label
                                color: session.foreground
                                font.pixelSize: session.textBody
                                elide: Text.ElideRight
                            }
                        }

                        MouseArea {
                            id: learnArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: index === 0 ? start.aboutRequested()
                                                   : start.licenceRequested()
                        }
                    }
                }
            }
        }

        // The page: the ways to start, the authored templates and the projects already known.
        Item {
            width: parent.width - rail.width
            height: parent.height

            Flickable {
                id: startScroll
                anchors.fill: parent
                contentWidth: width
                contentHeight: startContent.y + startContent.implicitHeight + startContent.y
                boundsBehavior: Flickable.StopAtBounds
                clip: true

                Column {
                    id: startContent
                    x: session.unit
                    y: session.unit
                    width: Math.max(0, startScroll.width - x * 2)
                    spacing: session.unit / 1.8

                    Text {
                        text: qsTr("Inicio")
                        color: session.foreground
                        font.pixelSize: session.textDisplay
                        font.bold: true
                    }
                    Text {
                        text: qsTr("Crea un proyecto, abre uno reciente o empieza desde una plantilla.")
                        color: session.muted
                        font.pixelSize: session.textSmall
                        width: parent.width
                        wrapMode: Text.WordWrap
                    }

                    // ----- Empezar ---------------------------------------------------
                    Text {
                        text: qsTr("Empezar")
                        color: session.foreground
                        font.pixelSize: session.textLabel
                        font.bold: true
                        topPadding: session.unit / 4
                    }

                    // The ways in are cards on the page, as in the reference: the page
                    // is one plane and each choice is a surface of its own. The default
                    // one is tinted and bordered with the accent; its text stays in the
                    // ink, so hover can never repaint a fill under a light-on-dark word.
                    Row {
                        id: startActions
                        width: parent.width
                        spacing: session.unit

                        Rectangle {
                            objectName: "startCardNew"
                            width: (parent.width - parent.spacing) / 2
                            height: session.unit * 3.9
                            radius: session.radius
                            color: session.accentSoft
                            border.width: newArea.containsMouse ? 3 : 2
                            border.color: session.accent

                            Accessible.role: Accessible.Button
                            Accessible.name: qsTr("Nuevo proyecto")
                            Accessible.onPressAction: start.newProject()

                            Row {
                                anchors.fill: parent
                                anchors.leftMargin: session.unit * 0.7
                                anchors.rightMargin: session.unit / 2
                                spacing: session.unit / 2

                                Item {
                                    objectName: "startCardNewIconColumn"
                                    width: Math.round(session.unit * 2)
                                    height: parent.height
                                    Image {
                                        objectName: "startCardNewIcon"
                                        anchors.centerIn: parent
                                        width: start.cardIconSize
                                        height: width
                                        sourceSize.width: start.cardIconSize
                                        sourceSize.height: start.cardIconSize
                                        fillMode: Image.PreserveAspectFit
                                        source: "image://theme/document-new?tone=accent&v="
                                                + session.themeRevision
                                        smooth: true
                                    }
                                }
                                Column {
                                    objectName: "startCardNewText"
                                    width: parent.width - Math.round(session.unit * 2)
                                           - parent.spacing
                                    anchors.verticalCenter: parent.verticalCenter
                                    spacing: 2
                                    Text {
                                        text: qsTr("Nuevo proyecto")
                                        color: session.foreground
                                        font.pixelSize: session.textLabel
                                        font.bold: true
                                        width: parent.width
                                        elide: Text.ElideRight
                                    }
                                    Text {
                                        text: qsTr("Una hoja vacía, A4 apaisado")
                                        color: session.muted
                                        font.pixelSize: session.textSmall
                                        width: parent.width
                                        wrapMode: Text.WordWrap
                                    }
                                }
                            }

                            MouseArea {
                                id: newArea
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: start.newProject()
                            }
                        }

                        Rectangle {
                            objectName: "startCardOpen"
                            width: (parent.width - parent.spacing) / 2
                            height: session.unit * 3.9
                            radius: session.radius
                            color: openArea.containsMouse ? session.surfaceRaised : session.panel
                            border.width: 1
                            border.color: openArea.containsMouse ? session.lineStrong
                                                                 : session.line

                            Accessible.role: Accessible.Button
                            Accessible.name: qsTr("Abrir…")
                            Accessible.onPressAction: start.openProject()

                            Row {
                                anchors.fill: parent
                                anchors.leftMargin: session.unit * 0.7
                                anchors.rightMargin: session.unit / 2
                                spacing: session.unit / 2

                                Item {
                                    objectName: "startCardOpenIconColumn"
                                    width: Math.round(session.unit * 2)
                                    height: parent.height
                                    Image {
                                        objectName: "startCardOpenIcon"
                                        anchors.centerIn: parent
                                        width: start.cardIconSize
                                        height: width
                                        sourceSize.width: start.cardIconSize
                                        sourceSize.height: start.cardIconSize
                                        fillMode: Image.PreserveAspectFit
                                        source: "image://theme/document-open?tone=muted&v="
                                                + session.themeRevision
                                        smooth: true
                                    }
                                }
                                Column {
                                    objectName: "startCardOpenText"
                                    width: parent.width - Math.round(session.unit * 2)
                                           - parent.spacing
                                    anchors.verticalCenter: parent.verticalCenter
                                    spacing: 2
                                    Text {
                                        text: qsTr("Abrir…")
                                        color: session.foreground
                                        font.pixelSize: session.textLabel
                                        width: parent.width
                                        elide: Text.ElideRight
                                    }
                                    Text {
                                        text: qsTr("Un proyecto guardado")
                                        color: session.muted
                                        font.pixelSize: session.textSmall
                                        width: parent.width
                                        wrapMode: Text.WordWrap
                                    }
                                }
                            }

                            MouseArea {
                                id: openArea
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: start.openProject()
                            }
                        }
                    }

                    // ----- Plantillas ------------------------------------------------
                    Text {
                        text: qsTr("Plantillas")
                        color: session.foreground
                        font.pixelSize: session.textLabel
                        font.bold: true
                        topPadding: session.unit / 4
                    }
                    Text {
                        text: qsTr("Elige un circuito de partida; podrás editarlo desde la hoja.")
                        color: session.muted
                        font.pixelSize: session.textSmall
                        width: parent.width
                        wrapMode: Text.WordWrap
                    }

                    // The cards are the content: the section heading is enough of a
                    // group, so no box wraps them (the reference's examples read the
                    // same way). The cards divide the page's width between them.
                    Grid {
                        id: templateGrid
                        width: parent.width
                        readonly property int minCardWidth: Math.round(session.unit * 10)
                        readonly property int columnsThatFit: Math.max(1, Math.floor(
                            (width + session.unit) / (minCardWidth + session.unit)))
                        columns: Math.min(start.templateCardCount, columnsThatFit)
                        columnSpacing: session.unit
                        rowSpacing: session.unit

                            Repeater {
                                objectName: "templateRepeater"
                                model: start.templateDefinitions
                                delegate: Button {
                                    id: templateCard
                                    objectName: "templateCard" + modelData.index
                                    width: Math.floor((templateGrid.width
                                                       - (templateGrid.columns - 1)
                                                         * templateGrid.columnSpacing)
                                                      / templateGrid.columns)
                                    // The height follows the width: the plate takes the
                                    // content width and its own 512:288 aspect, and the card
                                    // is the plate over 0.65, so the picture is 65 % of it.
                                    readonly property real plateWidth: width
                                                                       - 2 * (session.unit / 3)
                                                                       - 2 * (session.unit / 4)
                                    readonly property real plateHeight: plateWidth
                                                                        / (512.0 / 288.0)
                                    height: plateHeight / 0.65
                                    padding: session.unit / 3
                                    Accessible.role: Accessible.Button
                                    Accessible.name: modelData.title
                                    Accessible.description: modelData.detail

                                    HoverHandler { cursorShape: Qt.PointingHandCursor }

                                    ToolTip.visible: templateCard.hovered
                                    ToolTip.delay: 600
                                    ToolTip.text: modelData.title + " — " + modelData.detail

                                    contentItem: Item {
                                        Column {
                                            anchors.fill: parent
                                            anchors.margins: session.unit / 4
                                            spacing: session.unit / 5

                                            // The drawing sits on its own plate, as in the
                                            // reference: the thumbnail is the picture of the
                                            // circuit, and the words are the label under it.
                                            Rectangle {
                                                objectName: "templatePlate" + modelData.index
                                                width: parent.width
                                                height: templateCard.plateHeight
                                                radius: session.radiusSmall
                                                color: session.panel
                                                clip: true

                                                TemplatePreviewItem {
                                                    objectName: "templatePreview" + modelData.index
                                                    anchors.fill: parent
                                                    session: start.session
                                                    templateIndex: modelData.index
                                                }
                                            }

                                            Text {
                                                objectName: "templateTitle" + modelData.index
                                                text: modelData.title
                                                color: session.foreground
                                                font.pixelSize: session.textLabel
                                                font.bold: true
                                                width: parent.width
                                                elide: Text.ElideRight
                                            }
                                            Text {
                                                objectName: "templateDetail" + modelData.index
                                                text: modelData.detail
                                                color: session.muted
                                                font.pixelSize: session.textSmall
                                                width: parent.width
                                                maximumLineCount: 1
                                                elide: Text.ElideRight
                                            }
                                        }
                                    }

                                    background: Rectangle {
                                        anchors.fill: parent
                                        radius: session.radius
                                        color: templateCard.down ? session.surfaceSunken
                                                                 : session.surfaceRaised
                                        border.color: templateCard.activeFocus
                                                      || templateCard.hovered
                                                      ? session.accent : session.line
                                        border.width: templateCard.activeFocus
                                                      ? session.unit / 9 : session.unit / 18
                                    }

                                    onClicked: start.activateTemplate(modelData.index)
                                }
                            }
                        }

                    Text {
                        visible: start.templateError.length > 0
                        text: start.templateError
                        color: session.muted
                        font.pixelSize: session.textSmall
                        width: parent.width
                        wrapMode: Text.WordWrap
                    }

                    // ----- Recientes -------------------------------------------------
                    Text {
                        text: qsTr("Recientes")
                        color: session.foreground
                        font.pixelSize: session.textLabel
                        font.bold: true
                        topPadding: session.unit / 4
                    }
                    Text {
                        text: session.recent.length > session.recentPage.length
                              ? qsTr("Los proyectos que abras o guardes aparecerán aquí. "
                                     + "Los demás, en Archivo ▸ Recientes.")
                              : qsTr("Los proyectos que abras o guardes aparecerán aquí.")
                        color: session.muted
                        font.pixelSize: session.textSmall
                        width: parent.width
                        wrapMode: Text.WordWrap
                    }

                    Rectangle {
                        visible: start.session.recent.length > 0
                        width: parent.width
                        height: recentColumn.height + session.unit
                        radius: session.radius
                        color: session.panel
                        border.width: 1
                        border.color: session.line

                        Column {
                            id: recentColumn
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.verticalCenter: parent.verticalCenter
                            anchors.margins: session.unit / 2
                            spacing: 0

                            Repeater {
                                model: start.session.recentPage
                                delegate: Rectangle {
                                    id: recentRow
                                    objectName: "recentRow" + modelData.slot
                                    width: parent.width
                                    height: session.unit * 1.6
                                    color: recentArea.containsMouse && modelData.available
                                           ? session.surfaceRaised : "transparent"

                                    Column {
                                        anchors.verticalCenter: parent.verticalCenter
                                        anchors.left: parent.left
                                        anchors.leftMargin: session.unit / 3
                                        anchors.right: forgetButton.left
                                        anchors.rightMargin: session.unit / 3
                                        spacing: 0
                                        Text {
                                            text: modelData.name
                                            color: modelData.available ? session.foreground
                                                                       : session.muted
                                            font.pixelSize: session.textBody
                                            elide: Text.ElideRight
                                            width: parent.width
                                        }
                                        Text {
                                            text: modelData.available
                                                  ? modelData.path
                                                  : qsTr("no disponible · %1")
                                                    .arg(modelData.path)
                                            color: session.muted
                                            font.pixelSize: session.textSmall
                                            elide: Text.ElideMiddle
                                            width: parent.width
                                        }
                                    }

                                    Button {
                                        id: forgetButton
                                        objectName: "recentForget" + modelData.slot
                                        anchors.verticalCenter: parent.verticalCenter
                                        anchors.right: parent.right
                                        anchors.rightMargin: session.unit / 4
                                        height: session.unit * 1.4
                                        padding: session.unit / 3
                                        flat: true
                                        text: qsTr("quitar")
                                        Accessible.name: qsTr("Quitar de recientes: %1")
                                                         .arg(modelData.name)
                                        onClicked: start.forgetRecent(modelData.slot)

                                        contentItem: Text {
                                            text: forgetButton.text
                                            color: forgetButton.hovered ? session.foreground
                                                                        : session.muted
                                            font.pixelSize: session.textSmall
                                            horizontalAlignment: Text.AlignHCenter
                                            verticalAlignment: Text.AlignVCenter
                                        }
                                        background: Rectangle {
                                            radius: session.radiusSmall
                                            color: forgetButton.hovered
                                                   ? session.surfaceRaised : "transparent"
                                            border.width: forgetButton.activeFocus ? 1 : 0
                                            border.color: session.accent
                                        }
                                    }

                                    MouseArea {
                                        id: recentArea
                                        anchors.fill: parent
                                        anchors.rightMargin: forgetButton.width
                                                           + session.unit / 2
                                        hoverEnabled: true
                                        cursorShape: modelData.available
                                                     ? Qt.PointingHandCursor : Qt.ArrowCursor
                                        onClicked: if (modelData.available) {
                                                       start.openRecent(modelData.slot);
                                                   }
                                    }

                                    Rectangle {
                                        anchors.left: parent.left
                                        anchors.right: parent.right
                                        anchors.bottom: parent.bottom
                                        height: 1
                                        color: session.line
                                        visible: index < session.recentPage.length - 1
                                    }
                                }
                            }
                        }
                    }

                }

                ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
            }
        }
    }
}
