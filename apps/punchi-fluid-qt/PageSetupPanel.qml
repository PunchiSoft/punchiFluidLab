// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Configurar página: the one place the document's sheet is edited in detail. The quick
// controls live in the Disposición tab and in the Archivo shortcut; this panel adds the
// four margins, the custom size and a preview that follows every change.
//
// There is no Aceptar: the sheet is the document, so every change is already applied and
// already in the undo history. The panel only closes.

import QtQuick
import QtQuick.Controls
import PunchiFluid

Rectangle {
    id: panel

    property FluidSession session
    property string focusSection: "page"
    property string localError: ""
    property string customWidthText: ""
    property string customHeightText: ""

    signal closed()

    anchors.centerIn: parent
    width: Math.min(680, parent.width - 48)
    height: column.implicitHeight + 36
    radius: 8
    color: session.panel
    border.width: 1
    border.color: session.accent
    focus: visible
    activeFocusOnTab: true
    z: 47

    Accessible.role: Accessible.Dialog
    Accessible.name: qsTr("Configurar página")

    function parseCm(text) {
        const value = parseFloat(text.replace(",", "."));
        return isFinite(value) ? value : NaN;
    }

    function syncCustomFields() {
        customWidthText = session.formatPaperMeasure(session.sheetPaperWidthMm);
        customHeightText = session.formatPaperMeasure(session.sheetPaperHeightMm);
    }

    function applyFocus() {
        if (focusSection === "margins") {
            marginFields.forceActiveFocus();
        } else if (focusSection === "size") {
            customWidth.forceActiveFocus();
        } else {
            sizeCombo.focusButton();
        }
    }

    function applyCustomSize() {
        const width = parseCm(customWidthText);
        const height = parseCm(customHeightText);
        const min = session.paperMinMm / 10.0;
        const max = session.paperMaxMm / 10.0;
        if (!isFinite(width) || !isFinite(height) || width < min || width > max
            || height < min || height > max) {
            localError = qsTr("El ancho y el alto van de %1 a %2 cm.")
                .arg(session.formatPaperMeasure(session.paperMinMm))
                .arg(session.formatPaperMeasure(session.paperMaxMm));
            return;
        }
        localError = "";
        session.setSheetCustomPaper(width * 10.0, height * 10.0);
    }

    Component.onCompleted: {
        if (visible) {
            syncCustomFields();
        }
    }

    onVisibleChanged: {
        if (visible) {
            syncCustomFields();
            localError = "";
            Qt.callLater(applyFocus);
        }
    }

    onFocusSectionChanged: {
        if (visible) {
            Qt.callLater(applyFocus);
        }
    }

    Keys.onEscapePressed: panel.closed()

    Column {
        id: column
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.margins: 18
        spacing: 12

        Text {
            width: parent.width
            text: qsTr("Configurar página")
            color: session.foreground
            font.pixelSize: 16
            font.bold: true
        }

        Text {
            width: parent.width
            text: qsTr("La hoja de este documento. Cada cambio se puede deshacer.")
            color: session.muted
            font.pixelSize: 11
            wrapMode: Text.WordWrap
        }

        Row {
            spacing: 18

            PaperCombo {
                id: sizeCombo
                session: panel.session
                onPageSetupRequested: function (section) {
                    if (section === "size") {
                        customWidth.forceActiveFocus();
                    }
                }
            }

            OrientationControl {
                session: panel.session
            }
        }

        Row {
            width: parent.width
            spacing: 20

            Item {
                id: previewSlot
                width: Math.min(210, panel.width * 0.32)
                height: 150

                PaperPreview {
                    anchors.centerIn: parent
                    session: panel.session
                    paperWidthMm: panel.session.sheetPaperWidthMm
                    paperHeightMm: panel.session.sheetPaperHeightMm
                    landscape: panel.session.sheetLandscape
                    marginTop: panel.session.sheetMarginTop
                    marginRight: panel.session.sheetMarginRight
                    marginBottom: panel.session.sheetMarginBottom
                    marginLeft: panel.session.sheetMarginLeft
                    showMargins: true
                    maxWidth: previewSlot.width
                    maxHeight: previewSlot.height
                }
            }

            Column {
                width: parent.width - previewSlot.width - 20
                spacing: 8

                // The same gallery the ribbon shows, so the profile is chosen by looking at
                // the page rather than by reading four numbers. The last row sends the user
                // to the fields below.
                MarginGallery {
                    id: marginGallery
                    session: panel.session
                    onCustomRequested: Qt.callLater(function () {
                        marginFields.forceActiveFocus();
                    })
                }

                Row {
                    id: marginFields
                    spacing: 6
                    activeFocusOnTab: true

                    Repeater {
                        model: [
                            { label: qsTr("Sup."), value: session.sheetMarginTop },
                            { label: qsTr("Der."), value: session.sheetMarginRight },
                            { label: qsTr("Inf."), value: session.sheetMarginBottom },
                            { label: qsTr("Izq."), value: session.sheetMarginLeft }
                        ]

                        delegate: Column {
                            required property var modelData
                            required property int index

                            spacing: 2

                            Text {
                                text: modelData.label
                                color: session.muted
                                font.pixelSize: 9
                            }

                            Rectangle {
                                width: 62
                                height: 24
                                radius: 3
                                color: "transparent"
                                border.width: 1
                                border.color: session.muted

                                TextInput {
                                    anchors.fill: parent
                                    anchors.margins: 4
                                    verticalAlignment: TextInput.AlignVCenter
                                    text: index === 0 ? String(session.sheetMarginTop)
                                        : index === 1 ? String(session.sheetMarginRight)
                                        : index === 2 ? String(session.sheetMarginBottom)
                                        : String(session.sheetMarginLeft)
                                    color: session.foreground
                                    font.pixelSize: 11
                                    inputMethodHints: Qt.ImhFormattedNumbersOnly
                                    onEditingFinished: {
                                        const value = parseCm(text);
                                        if (isFinite(value) && value >= 0) {
                                            session.setSheetMargin(index, value);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                Text {
                    width: parent.width
                    text: qsTr("Márgenes en milímetros.")
                    color: session.muted
                    font.pixelSize: 10
                }
            }
        }

        Row {
            spacing: 8

            Text {
                text: qsTr("Tamaño personalizado (cm)")
                color: session.muted
                font.pixelSize: 11
                width: 170
            }

            Rectangle {
                width: 66
                height: 26
                radius: 3
                color: "transparent"
                border.width: 1
                border.color: session.muted

                TextInput {
                    id: customWidth
                    anchors.fill: parent
                    anchors.margins: 4
                    verticalAlignment: TextInput.AlignVCenter
                    text: panel.customWidthText
                    color: session.foreground
                    font.pixelSize: 11
                    inputMethodHints: Qt.ImhFormattedNumbersOnly
                    onTextEdited: panel.customWidthText = text
                }
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: "×"
                color: session.muted
                font.pixelSize: 12
            }

            Rectangle {
                width: 66
                height: 26
                radius: 3
                color: "transparent"
                border.width: 1
                border.color: session.muted

                TextInput {
                    id: customHeight
                    anchors.fill: parent
                    anchors.margins: 4
                    verticalAlignment: TextInput.AlignVCenter
                    text: panel.customHeightText
                    color: session.foreground
                    font.pixelSize: 11
                    inputMethodHints: Qt.ImhFormattedNumbersOnly
                    onTextEdited: panel.customHeightText = text
                }
            }

            Rectangle {
                width: 110
                height: 26
                radius: 4
                color: "transparent"
                border.width: 1
                border.color: session.accent
                activeFocusOnTab: true

                Accessible.role: Accessible.Button
                Accessible.name: qsTr("Usar medidas personalizadas")
                Accessible.onPressAction: panel.applyCustomSize()

                Text {
                    anchors.centerIn: parent
                    text: qsTr("Usar medidas")
                    color: session.foreground
                    font.pixelSize: 10
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: panel.applyCustomSize()
                }

                Keys.onSpacePressed: panel.applyCustomSize()
                Keys.onReturnPressed: panel.applyCustomSize()
                Keys.onEnterPressed: panel.applyCustomSize()
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("de %1 a %2 cm")
                    .arg(session.formatPaperMeasure(session.paperMinMm))
                    .arg(session.formatPaperMeasure(session.paperMaxMm))
                color: session.muted
                font.pixelSize: 10
            }
        }

        Text {
            width: parent.width
            visible: panel.localError.length > 0
            text: panel.localError
            color: session.foreground
            font.pixelSize: 10
            wrapMode: Text.WordWrap
        }

        Row {
            spacing: 10

            Rectangle {
                width: 150
                height: 28
                radius: 4
                color: "transparent"
                border.width: 1
                border.color: session.muted
                activeFocusOnTab: true

                Accessible.role: Accessible.Button
                Accessible.name: qsTr("Restablecer A4 ISO")
                Accessible.onPressAction: {
                    panel.session.resetSheet();
                    panel.syncCustomFields();
                    panel.localError = "";
                }

                Text {
                    anchors.centerIn: parent
                    text: qsTr("Restablecer A4 ISO")
                    color: session.foreground
                    font.pixelSize: 11
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        panel.session.resetSheet();
                        panel.syncCustomFields();
                        panel.localError = "";
                    }
                }

                Keys.onSpacePressed: {
                    panel.session.resetSheet();
                    panel.syncCustomFields();
                    panel.localError = "";
                }
                Keys.onReturnPressed: {
                    panel.session.resetSheet();
                    panel.syncCustomFields();
                    panel.localError = "";
                }
                Keys.onEnterPressed: {
                    panel.session.resetSheet();
                    panel.syncCustomFields();
                    panel.localError = "";
                }
            }

            Rectangle {
                width: 100
                height: 28
                radius: 4
                color: "transparent"
                border.width: 1
                border.color: session.accent
                activeFocusOnTab: true

                Accessible.role: Accessible.Button
                Accessible.name: qsTr("Cerrar")
                Accessible.onPressAction: panel.closed()

                Text {
                    anchors.centerIn: parent
                    text: qsTr("Cerrar")
                    color: session.foreground
                    font.pixelSize: 11
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: panel.closed()
                }

                Keys.onSpacePressed: panel.closed()
                Keys.onReturnPressed: panel.closed()
                Keys.onEnterPressed: panel.closed()
            }
        }
    }
}
