// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// A panel that folds to a rail. The header carries the title and the hide control; the
// content is clipped and the width is the only thing that moves, so the sheet takes the
// room in the same gesture. Hiding is view state: it never touches the document.
//
// The rail is one unit wide (24 px minimum, so the control stays a target) and keeps the
// same toggle in both states, which is also what keeps the keyboard focus from being lost
// when the panel closes.

import QtQuick
import PunchiFluid

Item {
    id: dock
    property FluidSession session
    property string title: ""
    property bool open: true
    property int panelWidth: 248
    property int railWidth: 24
    /// `false` when the panel has nothing to show: it takes no width and shows no rail.
    property bool available: true
    /// `true` when the open panel can be resized by dragging its inner edge.
    property bool resizable: false
    property int minimumPanelWidth: 220
    property int maximumPanelWidth: 480
    /// `true` when the panel sits on the right of the window.
    property bool sideRight: false
    /// The two accessible names of the control, so the title can be a sentence.
    property string hideName: qsTr("Ocultar el panel")
    property string showName: qsTr("Mostrar el panel")
    /// Auto-hide is opt-in: a dwell on the rail opens the panel, and leaving it closes it.
    property bool autoHide: false
    /// A dialog or a catalog drag suspends auto-hide while it lasts.
    property bool autoHideBlocked: false
    property bool hovered: false
    property bool manualHide: false
    /// `true` when the panel floats over the sheet (the compact window) instead of taking a
    /// column: it carries the card's edge and shadow so it reads as above the drawing.
    property bool floating: false
    readonly property int autoHideOpenMs: 350
    readonly property int autoHideCloseMs: 800

    default property alias content: contentHolder.data

    signal toggleRequested()
    signal autoHideRequested(bool open)
    signal widthRequested(int width)

    width: available ? (open ? panelWidth : railWidth) : 0
    Behavior on width {
        NumberAnimation {
            duration: dock.session ? dock.session.motionMs : 180
            easing.type: Easing.InOutQuad
        }
    }
    // A floating panel draws its shadow outside its own bounds, so it may not clip; the
    // width animation is clipped by the body instead, which is the only part that moves.
    clip: !floating
    readonly property bool railVisible: available && width <= railWidth + 0.5

    Rectangle {
        visible: dock.floating && !dock.railVisible
        anchors.fill: parent
        anchors.margins: -3
        radius: 8
        color: dock.session ? dock.session.shadow : "transparent"
        z: -1
    }

    Rectangle {
        anchors.fill: parent
        color: dock.railVisible ? dock.session.surfaceRaised : dock.session.background
        border.width: dock.floating && !dock.railVisible ? 1 : 0
        border.color: dock.session ? dock.session.lineStrong : "transparent"
    }

    // The edge against the sheet, so the panel is not a floating rectangle. It stays
    // while the panel is folded, which is what makes the rail read as a panel edge.
    Rectangle {
        visible: !dock.floating
        width: 1
        height: parent.height
        anchors.right: dock.sideRight ? undefined : parent.right
        anchors.left: dock.sideRight ? parent.left : undefined
        color: dock.session.line
    }

    Item {
        id: panelBody
        anchors.fill: parent
        visible: dock.available && !dock.railVisible
        clip: true

        Text {
            anchors.top: parent.top
            anchors.topMargin: 13
            anchors.left: parent.left
            anchors.leftMargin: dock.sideRight ? 32 : 14
            anchors.right: parent.right
            anchors.rightMargin: dock.sideRight ? 12 : 32
            text: dock.title
            color: dock.session.muted
            font.pixelSize: 12
            font.letterSpacing: 1
            elide: Text.ElideRight
        }

        Item {
            id: contentHolder
            anchors.top: parent.top
            anchors.topMargin: 36
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            clip: true
        }
    }

    Rectangle {
        anchors.fill: parent
        visible: dock.railVisible
        color: dock.session.surfaceRaised

        MouseArea {
            anchors.fill: parent
            cursorShape: Qt.PointingHandCursor
            onClicked: dock.toggleRequested()
        }
    }

    // The inner edge is the resize handle while the panel is open. It sits under the
    // toggle so the control keeps its click; the cursor is the resize one.
    Rectangle {
        id: resizeHandle
        visible: dock.resizable && dock.available && !dock.railVisible
        width: 6
        height: parent.height
        anchors.right: dock.sideRight ? undefined : parent.right
        anchors.left: dock.sideRight ? parent.left : undefined
        color: handlePointer.containsMouse || handlePointer.pressed
               ? dock.session.accentSoft : "transparent"
        z: 2

        MouseArea {
            id: handlePointer
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.SplitHCursor
            property real pressX: 0
            property int pressWidth: 0

            onPressed: {
                pressX = mapToItem(null, mouse.x, 0).x;
                pressWidth = dock.width;
            }
            onPositionChanged: if (pressed) {
                const delta = mapToItem(null, mouse.x, 0).x - pressX;
                const next = dock.sideRight ? pressWidth - delta : pressWidth + delta;
                dock.widthRequested(Math.round(Math.max(dock.minimumPanelWidth,
                                                       Math.min(dock.maximumPanelWidth, next))));
            }
        }
    }

    PanelToggle {
        id: toggle
        session: dock.session
        sideRight: dock.sideRight
        open: dock.open
        name: dock.open ? dock.hideName : dock.showName
        width: 24
        height: 24
        x: dock.sideRight ? 0 : dock.width - width
        y: dock.railVisible ? (dock.height - height) / 2 : 8
        visible: dock.available
        z: 3
        Behavior on y {
            NumberAnimation { duration: dock.session ? dock.session.motionMs : 180 }
        }
        onActivated: dock.toggleRequested()
    }

    // The pointer over the rail or the panel: the same item covers both, so the edge is the
    // handle. A HoverHandler does not swallow clicks the way a MouseArea would.
    HoverHandler {
        id: hover
        onHoveredChanged: dock.hovered = hovered
    }

    Timer {
        id: showTimer
        interval: dock.autoHideOpenMs
        repeat: false
        onTriggered: {
            if (dock.autoHide && !dock.open && !dock.manualHide && !dock.autoHideBlocked
                && dock.available) {
                dock.autoHideRequested(true);
            }
        }
    }

    Timer {
        id: hideTimer
        interval: dock.autoHideCloseMs
        repeat: false
        onTriggered: {
            // A panel with the keyboard inside it, or a dialog on top, is never taken away.
            if (dock.autoHide && dock.open && !dock.autoHideBlocked && !dock.focusInside()) {
                dock.autoHideRequested(false);
            }
        }
    }

    onHoveredChanged: {
        showTimer.stop();
        hideTimer.stop();
        if (hovered) {
            if (!open && !manualHide && autoHide && !autoHideBlocked) {
                showTimer.start();
            }
        } else {
            // Leaving the rail clears a manual hide, so the next visit opens it again.
            manualHide = false;
            if (open && autoHide && !autoHideBlocked) {
                hideTimer.start();
            }
        }
    }

    onOpenChanged: {
        showTimer.stop();
        hideTimer.stop();
        if (open) {
            manualHide = false;
            if (autoHide && !hovered && !autoHideBlocked) {
                hideTimer.start();
            }
        } else {
            // Hiding from the control while the pointer is on it must not reopen at once.
            manualHide = hovered;
        }
    }

    onAutoHideChanged: {
        if (!autoHide) {
            showTimer.stop();
            hideTimer.stop();
            manualHide = false;
        } else if (open && !hovered && !autoHideBlocked) {
            hideTimer.start();
        } else if (!open && hovered && !manualHide && !autoHideBlocked) {
            showTimer.start();
        }
    }

    onAutoHideBlockedChanged: {
        if (autoHideBlocked) {
            showTimer.stop();
            hideTimer.stop();
        } else if (autoHide && open && !hovered) {
            hideTimer.start();
        } else if (autoHide && !open && hovered && !manualHide) {
            showTimer.start();
        }
    }

    /// `true` when the keyboard focus is anywhere inside this panel.
    function focusInside() {
        let item = dock.Window.window ? dock.Window.window.activeFocusItem : null;
        while (item) {
            if (item === dock) {
                return true;
            }
            item = item.parent;
        }
        return false;
    }
}
