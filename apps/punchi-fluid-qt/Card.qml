// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// A floating editor on the workspace: one raised surface, one edge line and one shadow
// drawn once, so the page setup, the export, the confirmation and the check do not each
// invent their own radius and border. The content declared by the caller sits above the
// surface, as it did when the editor was a bare Rectangle.

import QtQuick
import PunchiFluid

Item {
    id: card
    property FluidSession session
    property int radius: 8
    property color borderColor: session ? session.lineStrong : "transparent"

    Rectangle {
        id: shadow
        anchors.fill: surface
        anchors.margins: -3
        radius: card.radius + 2
        color: card.session ? card.session.shadow : "transparent"
        z: -1
    }

    Rectangle {
        id: surface
        anchors.fill: parent
        radius: card.radius
        color: card.session ? card.session.panel : "transparent"
        border.width: 1
        border.color: card.borderColor
    }
}
