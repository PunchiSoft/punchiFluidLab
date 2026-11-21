// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// A labelled group of the ribbon: its commands on top, its name underneath, and a hair
// line to the next group. The name underneath is what makes a ribbon readable — without
// it a row of icons is a guessing game.
//
// A group that does not fit keeps its name as a titled button and shows the same commands
// in a non-modal flyout (P-028). The body and the flyout are built from one component, so
// a command cannot exist in one form and be missing from the other.

import QtQuick
import QtQuick.Controls
import PunchiFluid

Item {
    id: group

    property FluidSession session
    /// The ribbon that owns this group: it answers the commands and receives the measured
    /// widths the one-row layout needs.
    property var ribbonBar: null
    /// The group entry of the ribbon's command table, shared with the flyout.
    property var groupData: null
    /// The position in the tab, so the layout can measure the row before deciding.
    property int groupIndex: 0
    property string title: ""
    /// Collapsed mode replaces the body with a titled button; the height does not change,
    /// so the sheet below keeps its room.
    property bool collapsed: false

    /// The body width is what the group would take expanded. It stays valid while the
    /// group is collapsed because the body is hidden, not destroyed.
    readonly property real expandedWidth: Math.max(bodyLoader.implicitWidth,
                                                   titleText.implicitWidth) + 22
    /// A collapsed group is its name and a chevron; the priority only collapses when this
    /// is narrower than the expanded body.
    readonly property real collapsedWidth: Math.max(44, collapsedButton.width + 6)
    /// The commands of the flyout, from the same table the expanded group draws.
    readonly property var flyoutActions: groupData && ribbonBar
                                         ? ribbonBar.groupActions(groupData) : []
    readonly property int commandCount: flyoutActions.length
    readonly property bool hasNote: !!groupData && !!groupData.note
    readonly property bool flyoutOpen: flyout.opened

    implicitWidth: collapsed ? collapsedWidth : expandedWidth
    implicitHeight: Math.max(84, 4 * session.unit + 12)

    /// The body is a Loader so the flyout can instantiate the same component again.
    Component {
        id: groupContent

        Row {
            id: contentRoot
            spacing: 2

            Loader {
                id: customLoader
                visible: !!group.groupData && !!group.groupData.custom
                sourceComponent: group.groupData && group.groupData.custom === "page-setup"
                                 ? pageSetupGroup
                                 : group.groupData && group.groupData.custom === "recent"
                                   ? recentGroup
                                   : group.groupData && group.groupData.custom === "tube"
                                     ? tubeGroup
                                     : null
            }

            Repeater {
                model: group.groupData ? (group.groupData.items || []) : []

                delegate: RibbonButton {
                    required property var modelData
                    objectName: "ribbonCommand"
                    session: group.session
                    label: modelData.label
                    iconName: modelData.icon || ""
                    hint: modelData.hint || ""
                    large: modelData.big === true
                    available: modelData.enabled === undefined || modelData.enabled === true
                    selected: modelData.current === true
                    tone: "foreground"
                    onTriggered: if (group.ribbonBar) {
                                     group.ribbonBar.invoke(modelData.id);
                                 }
                }
            }

            // A tab whose commands do not exist yet says so once, rather than showing grey
            // buttons that teach the user nothing.
            Item {
                visible: !!group.groupData && !!group.groupData.note
                // A fixed width, not one derived from the wrapped text: a text that wraps
                // needs the width it is measured with, and asking each for the other is a
                // binding loop.
                width: visible ? 180 : 0
                height: 62

                Text {
                    id: noteText
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width
                    text: group.groupData ? (group.groupData.note || "") : ""
                    color: group.session.muted
                    font.pixelSize: 11
                    wrapMode: Text.WordWrap
                }
            }
        }
    }

    // The expanded body. It stays loaded when the group collapses, so `expandedWidth`
    // keeps measuring the form the priority decided against and no remeasure shifts the row.
    Loader {
        id: bodyLoader
        sourceComponent: groupContent
        visible: !group.collapsed
        anchors.top: parent.top
        anchors.topMargin: 5
        anchors.horizontalCenter: parent.horizontalCenter
        onLoaded: {
            group.adoptTubeControl(group.tubeControlIn(item), false);
        }
    }

    Text {
        id: titleText
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        visible: !group.collapsed
        text: group.title
        color: group.session.muted
        font.pixelSize: group.session.textSmall
        font.letterSpacing: 0.4
    }

    // The collapsed form: the group name is the label, and the chevron says it opens. The
    // button stays keyboard reachable and reports its collapsed state to accessibility.
    Item {
        id: collapsedButton
        objectName: "ribbonGroupButton"
        visible: group.collapsed
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.verticalCenter: parent.verticalCenter
        width: collapsedRow.implicitWidth + 10
        height: Math.max(28, group.session.unit + 8)
        activeFocusOnTab: true

        Accessible.role: Accessible.Button
        Accessible.name: group.title
        Accessible.description: qsTr("grupo plegado; ábrelo para ver sus comandos")
        Accessible.onPressAction: group.openFlyout()

        Rectangle {
            anchors.fill: parent
            anchors.margins: 1
            radius: group.session.radiusSmall
            color: collapsedPointer.pressed ? group.session.surfaceSunken
                   : collapsedPointer.containsMouse ? group.session.surfaceRaised
                   : "transparent"
            border.width: collapsedButton.activeFocus ? 1 : 0
            border.color: group.session.accent
        }

        Row {
            id: collapsedRow
            anchors.centerIn: parent
            spacing: 2

            Text {
                id: collapsedLabel
                text: group.title
                color: group.session.foreground
                font.pixelSize: group.session.textBody
            }

            Image {
                id: chevron
                anchors.verticalCenter: parent.verticalCenter
                width: Math.max(8, Math.round(group.session.unit * 0.55))
                height: width
                sourceSize.width: width
                sourceSize.height: height
                source: "image://theme/pan-down?tone=muted&v=" + group.session.themeRevision
            }
        }

        MouseArea {
            id: collapsedPointer
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: group.openFlyout()
        }

        Keys.onReturnPressed: group.openFlyout()
        Keys.onEnterPressed: group.openFlyout()
        Keys.onSpacePressed: group.openFlyout()
        Keys.onDownPressed: group.openFlyout()
    }

    // The flyout: non-modal, closes on Esc or a click outside, takes the focus so a
    // keyboard user can walk its commands. The content is the component the body uses,
    // instantiated a second time only while the group is collapsed.
    Popup {
        id: flyout
        objectName: "ribbonGroupFlyout"
        parent: group
        x: 0
        y: group.height
        width: Math.max(120, flyoutLoader.implicitWidth + 24)
        height: flyoutLoader.implicitHeight + 24
        modal: false
        focus: true
        closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
        padding: 12

        background: Rectangle {
            color: group.session.panel
            border.width: 1
            border.color: group.session.line
            radius: group.session.radiusSmall
        }

        contentItem: Item {
            implicitWidth: flyoutLoader.implicitWidth
            implicitHeight: flyoutLoader.implicitHeight

            Loader {
                id: flyoutLoader
                anchors.centerIn: parent
                sourceComponent: group.collapsed ? groupContent : null
                onLoaded: group.adoptTubeControl(group.tubeControlIn(item), true);
            }
        }

        // Closing the flyout destroys its copy of the tube control. The body's control stays
        // loaded, so the tubeControl binding falls back to it instead of a spent copy.
        onClosed: if (group.ribbonBar) {
                      group.ribbonBar.flyoutTubeControl = null;
                  }
    }

    // The hair line between groups; a collapsed group keeps it, so the row still reads as
    // seven groups rather than a strip of unrelated buttons.
    Rectangle {
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.topMargin: 10
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 18
        width: 1
        color: group.session.line
    }

    /// Publishes both measurements, so the ribbon can decide the row without guessing.
    function publishMetrics() {
        if (group.ribbonBar) {
            group.ribbonBar.recordGroupMetrics(group.groupIndex, group.title,
                                               group.expandedWidth, group.collapsedWidth);
        }
    }

    /// A tube-size control is a control, not a command: the ribbon's Alt+D path keeps the
    /// one in the visible form and falls back to the flyout copy when the body is collapsed.
    function adoptTubeControl(item, fromFlyout) {
        if (!item || item.objectName !== "tubeSizeControl" || !group.ribbonBar) {
            return;
        }
        if (fromFlyout) {
            group.ribbonBar.flyoutTubeControl = item;
        } else {
            group.ribbonBar.bodyTubeControl = item;
        }
    }

    /// The content component may wrap the control in extra items; the ribbon only needs the
    /// control itself to open the diameter gallery from Alt+D.
    function tubeControlIn(rootItem) {
        if (!rootItem) {
            return null;
        }
        if (rootItem.objectName === "tubeSizeControl") {
            return rootItem;
        }
        const kids = rootItem.children;
        for (let i = 0; i < kids.length; ++i) {
            const found = group.tubeControlIn(kids[i]);
            if (found) {
                return found;
            }
        }
        return null;
    }

    function openFlyout() {
        if (!group.collapsed) {
            return;
        }
        flyout.open();
    }

    /// The flyout content is the command table instantiated; counting the buttons in the
    /// open flyout proves the commands are there, not just that the source list exists.
    function flyoutCommandCount() {
        let count = 0;
        const walk = function (item) {
            if (!item) {
                return;
            }
            if (item.objectName === "ribbonCommand") {
                ++count;
            }
            const kids = item.children || [];
            for (let i = 0; i < kids.length; ++i) {
                walk(kids[i]);
            }
        };
        walk(flyoutLoader.item);
        return count;
    }

    function closeFlyout() {
        flyout.close();
    }

    onExpandedWidthChanged: publishMetrics()
    onCollapsedWidthChanged: publishMetrics()
    onGroupIndexChanged: publishMetrics()
    onRibbonBarChanged: publishMetrics()
    /// Leaving the collapsed form unloads the flyout copy; its control must not outlive it.
    onCollapsedChanged: if (!group.collapsed && group.ribbonBar) {
                            group.ribbonBar.flyoutTubeControl = null;
                        }
    Component.onCompleted: publishMetrics()

    Component {
        id: pageSetupGroup

        ConfigurarPaginaGroup {
            objectName: "pageSetupGroup"
            session: group.session
            paperListRequested: group.ribbonBar ? group.ribbonBar.paperListRequested : false
            marginListRequested: group.ribbonBar ? group.ribbonBar.marginListRequested : false
            onPageSetupRequested: function (section) {
                if (group.ribbonBar) {
                    group.ribbonBar.pageSetupRequested(section);
                }
            }
        }
    }

    Component {
        id: recentGroup

        RecentCombo {
            objectName: "recentGroup"
            session: group.session
            showLabel: false
        }
    }

    Component {
        id: tubeGroup

        TubeSizeControl {
            objectName: "tubeSizeControl"
            session: group.session
        }
    }
}
