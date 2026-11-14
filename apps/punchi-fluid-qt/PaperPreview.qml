// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The small page a size or a margin preset shows beside its name. It is drawn from
// millimetres, like the sheet, so the proportions are the paper's and not a guess. The
// colours come from the theme; an empty surface is the background, the paper edge is the
// muted ink and the frame is the accent.

import QtQuick
import PunchiFluid

Item {
    id: preview

    property FluidSession session
    property real paperWidthMm: 210
    property real paperHeightMm: 297
    property bool landscape: false
    property real marginTop: 0
    property real marginRight: 0
    property real marginBottom: 0
    property real marginLeft: 0
    property bool showMargins: false
    property real maxWidth: 42
    property real maxHeight: 52

    readonly property real pageWidthMm: landscape ? paperHeightMm : paperWidthMm
    readonly property real pageHeightMm: landscape ? paperWidthMm : paperHeightMm
    readonly property real scale: Math.min(maxWidth / Math.max(pageWidthMm, 1),
                                          maxHeight / Math.max(pageHeightMm, 1))

    implicitWidth: pageWidthMm * scale
    implicitHeight: pageHeightMm * scale

    Rectangle {
        anchors.fill: parent
        radius: 1
        color: preview.session ? preview.session.background : "transparent"
        border.width: 1
        border.color: preview.session ? preview.session.muted : "transparent"
    }

    // The margin band, shaded the way a page-setup gallery shades it: the paper is the
    // light area and the frame is the inner edge. The four bands do not overlap, so the
    // corners are not twice as dark.
    Rectangle {
        visible: preview.showMargins
        x: 0
        y: 0
        width: parent.width
        height: Math.min(preview.marginTop, preview.pageHeightMm) * preview.scale
        color: preview.session ? preview.session.accent : "transparent"
        opacity: 0.14
    }

    Rectangle {
        visible: preview.showMargins
        x: 0
        y: parent.height
           - Math.min(preview.marginBottom, preview.pageHeightMm) * preview.scale
        width: parent.width
        height: Math.min(preview.marginBottom, preview.pageHeightMm) * preview.scale
        color: preview.session ? preview.session.accent : "transparent"
        opacity: 0.14
    }

    Rectangle {
        visible: preview.showMargins
        x: 0
        y: Math.min(preview.marginTop, preview.pageHeightMm) * preview.scale
        width: Math.min(preview.marginLeft, preview.pageWidthMm) * preview.scale
        height: Math.max(0, parent.height
                            - (preview.marginTop + preview.marginBottom) * preview.scale)
        color: preview.session ? preview.session.accent : "transparent"
        opacity: 0.14
    }

    Rectangle {
        visible: preview.showMargins
        x: parent.width
           - Math.min(preview.marginRight, preview.pageWidthMm) * preview.scale
        y: Math.min(preview.marginTop, preview.pageHeightMm) * preview.scale
        width: Math.min(preview.marginRight, preview.pageWidthMm) * preview.scale
        height: Math.max(0, parent.height
                            - (preview.marginTop + preview.marginBottom) * preview.scale)
        color: preview.session ? preview.session.accent : "transparent"
        opacity: 0.14
    }

    Rectangle {
        visible: preview.showMargins
        x: Math.min(preview.marginLeft, preview.pageWidthMm) * preview.scale
        y: Math.min(preview.marginTop, preview.pageHeightMm) * preview.scale
        width: Math.max(0, preview.pageWidthMm - preview.marginLeft - preview.marginRight)
               * preview.scale
        height: Math.max(0, preview.pageHeightMm - preview.marginTop - preview.marginBottom)
                * preview.scale
        color: "transparent"
        border.width: 1
        border.color: preview.session ? preview.session.accent : "transparent"
    }
}
