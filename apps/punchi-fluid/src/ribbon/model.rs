// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The ribbon data: tabs, groups, commands and the gallery previews.
//!
//! Every control carries a [`Command`] or nothing. A control with nothing is painted disabled:
//! it is a command the window does not have yet, and the ribbon never pretends otherwise.

use super::{Command, MarginPreset, Speed};
use punchi_fluid_doc::Axis;
use punchi_fluid_doc::title_block::{BlockPosition, ContentModel};
use punchi_fluid_draw::LineClass;

pub(super) enum Preview {
    Margins(f32),
    Pipe(&'static str),
}

pub(super) struct GalleryEntry {
    pub(super) label: &'static str,
    pub(super) preview: Preview,
    pub(super) command: Option<Command>,
}

impl GalleryEntry {
    /// An entry that carries its command.
    const fn command(label: &'static str, preview: Preview, command: Command) -> Self {
        Self {
            label,
            preview,
            command: Some(command),
        }
    }
}

pub(super) struct MenuEntry {
    pub(super) label: &'static str,
    pub(super) command: Option<Command>,
}

impl MenuEntry {
    /// An option the window cannot run yet: painted, never dispatched.
    const fn unavailable(label: &'static str) -> Self {
        Self {
            label,
            command: None,
        }
    }

    /// An option that carries its command.
    const fn command(label: &'static str, command: Command) -> Self {
        Self {
            label,
            command: Some(command),
        }
    }
}

pub(super) enum Item {
    Large(&'static str, &'static str, Option<Command>),
    Small(&'static str, &'static str, Option<Command>),
    Menu(&'static str, &'static str, &'static [MenuEntry]),
    Gallery(&'static str, &'static str, &'static [GalleryEntry]),
    Recents(&'static str, &'static str),
}

pub(super) const MARGINS: &[GalleryEntry] = &[
    GalleryEntry::command(
        "ribbon.gallery.margin.normal",
        Preview::Margins(0.3),
        Command::PageMargin(MarginPreset::Normal),
    ),
    GalleryEntry::command(
        "ribbon.gallery.margin.narrow",
        Preview::Margins(0.15),
        Command::PageMargin(MarginPreset::Narrow),
    ),
    GalleryEntry::command(
        "ribbon.gallery.margin.moderate",
        Preview::Margins(0.45),
        Command::PageMargin(MarginPreset::Moderate),
    ),
    GalleryEntry::command(
        "ribbon.gallery.margin.wide",
        Preview::Margins(0.6),
        Command::PageMargin(MarginPreset::Wide),
    ),
];

pub(super) const CONDUCTO: &[GalleryEntry] = &[
    GalleryEntry::command(
        "ribbon.gallery.pipe.working",
        Preview::Pipe("pipe-working"),
        Command::PipeClass(LineClass::Working),
    ),
    GalleryEntry::command(
        "ribbon.gallery.pipe.pilot",
        Preview::Pipe("pipe-pilot"),
        Command::PipeClass(LineClass::Pilot),
    ),
    GalleryEntry::command(
        "ribbon.gallery.pipe.drain",
        Preview::Pipe("pipe-drain"),
        Command::PipeClass(LineClass::Drain),
    ),
    GalleryEntry::command(
        "ribbon.item.diameter",
        Preview::Pipe("tube-diameter"),
        Command::Tube,
    ),
];

pub(super) struct Group {
    pub(super) name: &'static str,
    pub(super) launcher: bool,
    pub(super) items: &'static [Item],
}

pub(super) struct Tab {
    pub(super) name: &'static str,
    pub(super) groups: &'static [Group],
}

pub(super) const ARCHIVO: &[Group] = &[
    Group {
        name: "ribbon.group.project",
        launcher: false,
        items: &[
            Item::Large("document-new", "ribbon.item.new", Some(Command::ProjectNew)),
            Item::Large(
                "document-open",
                "ribbon.item.open",
                Some(Command::ProjectOpen),
            ),
            Item::Large(
                "document-save",
                "ribbon.item.save",
                Some(Command::ProjectSave),
            ),
            Item::Small(
                "document-save-as",
                "ribbon.item.save_as",
                Some(Command::ProjectSaveAs),
            ),
            Item::Small(
                "document-export",
                "ribbon.item.export",
                Some(Command::ProjectExport),
            ),
            Item::Small(
                "document-export",
                "ribbon.item.options",
                Some(Command::ExportOptions),
            ),
        ],
    },
    Group {
        name: "ribbon.group.circuit",
        launcher: false,
        items: &[
            Item::Small(
                "list-add",
                "ribbon.item.circuit_new",
                Some(Command::CircuitNew),
            ),
            Item::Small(
                "edit-rename",
                "ribbon.item.circuit_rename",
                Some(Command::CircuitRename),
            ),
            Item::Small(
                "window-close",
                "ribbon.item.circuit_close",
                Some(Command::CircuitClose),
            ),
        ],
    },
    Group {
        name: "ribbon.group.page",
        launcher: true,
        items: &[
            Item::Gallery("document-properties", "ribbon.item.margins", MARGINS),
            Item::Menu(
                "document-properties",
                "ribbon.item.orientation",
                &[
                    MenuEntry::command(
                        "ribbon.menu.orientation.portrait",
                        Command::PageOrientation(punchi_fluid_doc::paper::Orientation::Portrait),
                    ),
                    MenuEntry::command(
                        "ribbon.menu.orientation.landscape",
                        Command::PageOrientation(punchi_fluid_doc::paper::Orientation::Landscape),
                    ),
                ],
            ),
            Item::Menu(
                "x-office-document",
                "ribbon.item.paper",
                &[
                    MenuEntry::command(
                        "ribbon.menu.paper.a4",
                        Command::PagePaper(punchi_fluid_doc::paper::PaperSize::A4),
                    ),
                    MenuEntry::command(
                        "ribbon.menu.paper.a3",
                        Command::PagePaper(punchi_fluid_doc::paper::PaperSize::A3),
                    ),
                    MenuEntry::command(
                        "ribbon.menu.paper.a2",
                        Command::PagePaper(punchi_fluid_doc::paper::PaperSize::A2),
                    ),
                    MenuEntry::unavailable("ribbon.menu.paper.custom"),
                ],
            ),
        ],
    },
    Group {
        name: "ribbon.group.recents",
        launcher: false,
        items: &[
            Item::Recents("document-open-recent", "ribbon.item.recents"),
            Item::Small("go-home", "ribbon.item.home", Some(Command::Home)),
        ],
    },
];

pub(super) const EDITAR: &[Group] = &[
    Group {
        name: "ribbon.group.history",
        launcher: false,
        items: &[
            Item::Small("edit-undo", "ribbon.item.undo", Some(Command::Undo)),
            Item::Small("edit-redo", "ribbon.item.redo", Some(Command::Redo)),
        ],
    },
    Group {
        name: "ribbon.group.component",
        launcher: false,
        items: &[
            Item::Small("edit-copy", "ribbon.item.copy", Some(Command::Copy)),
            Item::Small("edit-paste", "ribbon.item.paste", Some(Command::Paste)),
            Item::Small(
                "edit-duplicate",
                "ribbon.item.duplicate",
                Some(Command::Duplicate),
            ),
            Item::Small("edit-delete", "ribbon.item.delete", Some(Command::Delete)),
            Item::Small(
                "edit-select-all",
                "ribbon.item.select_all",
                Some(Command::SelectAll),
            ),
        ],
    },
    Group {
        name: "ribbon.group.transform",
        launcher: false,
        items: &[
            Item::Small(
                "object-rotate-right",
                "ribbon.item.rotate",
                Some(Command::Rotate),
            ),
            Item::Small(
                "object-flip-horizontal",
                "ribbon.item.mirror",
                Some(Command::Mirror),
            ),
        ],
    },
    Group {
        name: "ribbon.group.align",
        launcher: false,
        items: &[
            Item::Small(
                "align-horizontal-left",
                "ribbon.item.align_left",
                Some(Command::AlignLeft),
            ),
            Item::Small(
                "align-horizontal-right",
                "ribbon.item.align_right",
                Some(Command::AlignRight),
            ),
            Item::Small(
                "align-vertical-top",
                "ribbon.item.align_top",
                Some(Command::AlignTop),
            ),
            Item::Small(
                "align-vertical-bottom",
                "ribbon.item.align_bottom",
                Some(Command::AlignBottom),
            ),
            Item::Small("align-ports", "ribbon.item.align_ports", None),
        ],
    },
    Group {
        name: "ribbon.group.distribute",
        launcher: false,
        items: &[
            Item::Small(
                "distribute-horizontal-equal",
                "ribbon.item.distribute_h",
                Some(Command::Distribute(Axis::Horizontal)),
            ),
            Item::Small(
                "distribute-vertical-equal",
                "ribbon.item.distribute_v",
                Some(Command::Distribute(Axis::Vertical)),
            ),
        ],
    },
    Group {
        name: "ribbon.group.review",
        launcher: false,
        items: &[Item::Small(
            "check",
            "ribbon.item.check",
            Some(Command::Check),
        )],
    },
    Group {
        name: "ribbon.group.pipe",
        launcher: false,
        items: &[Item::Gallery("pipe-working", "ribbon.item.pipe", CONDUCTO)],
    },
];

pub(super) const VER: &[Group] = &[
    Group {
        name: "ribbon.group.view",
        launcher: false,
        items: &[
            Item::Small("zoom-in", "ribbon.item.zoom_in", Some(Command::ZoomIn)),
            Item::Small("zoom-out", "ribbon.item.zoom_out", Some(Command::ZoomOut)),
            Item::Small(
                "zoom-original",
                "ribbon.item.zoom_reset",
                Some(Command::ZoomReset),
            ),
            Item::Small(
                "edit-undo",
                "ribbon.item.view_back",
                Some(Command::ViewBack),
            ),
            Item::Small(
                "document-properties",
                "ribbon.item.block_visible",
                Some(Command::BlockVisible),
            ),
        ],
    },
    Group {
        name: "ribbon.group.panels",
        launcher: false,
        items: &[
            Item::Small(
                "view-list-details",
                "ribbon.item.catalog",
                Some(Command::Catalog),
            ),
            Item::Small(
                "document-properties",
                "ribbon.item.properties",
                Some(Command::Properties),
            ),
            Item::Small("go-bottom", "ribbon.item.autohide", Some(Command::Autohide)),
            Item::Small(
                "edit-reset",
                "ribbon.item.reset_view",
                Some(Command::ResetView),
            ),
        ],
    },
    Group {
        name: "ribbon.group.theme",
        launcher: false,
        items: &[Item::Small(
            "preferences-desktop-theme",
            "ribbon.item.theme",
            Some(Command::Theme),
        )],
    },
];

pub(super) const CAJETIN: &[Group] = &[
    Group {
        name: "ribbon.group.content",
        launcher: false,
        items: &[Item::Menu(
            "x-office-document",
            "ribbon.item.block_content",
            &[
                MenuEntry::command(
                    "ribbon.menu.content.technical",
                    Command::BlockModel(ContentModel::Technical),
                ),
                MenuEntry::command(
                    "ribbon.menu.content.school",
                    Command::BlockModel(ContentModel::School),
                ),
                MenuEntry::command(
                    "ribbon.menu.content.industrial",
                    Command::BlockModel(ContentModel::Industrial),
                ),
                MenuEntry::command(
                    "ribbon.menu.content.none",
                    Command::BlockPosition(BlockPosition::None),
                ),
            ],
        )],
    },
    Group {
        name: "ribbon.group.position",
        launcher: false,
        items: &[Item::Menu(
            "go-top",
            "ribbon.item.block_position",
            &[
                MenuEntry::command(
                    "ribbon.menu.position.top_right",
                    Command::BlockPosition(BlockPosition::TopRight),
                ),
                MenuEntry::command(
                    "ribbon.menu.position.bottom_right",
                    Command::BlockPosition(BlockPosition::BottomRight),
                ),
                MenuEntry::command(
                    "ribbon.menu.position.bottom_full",
                    Command::BlockPosition(BlockPosition::BottomFull),
                ),
            ],
        )],
    },
    Group {
        name: "ribbon.group.data",
        launcher: false,
        items: &[
            Item::Small(
                "x-office-address-book",
                "ribbon.item.block_edit",
                Some(Command::BlockEdit),
            ),
            Item::Small("edit-reset", "ribbon.item.reset", Some(Command::BlockReset)),
        ],
    },
];

pub(super) const SIMULAR: &[Group] = &[
    Group {
        name: "ribbon.group.transport",
        launcher: false,
        items: &[
            Item::Large(
                "media-playback-start",
                "ribbon.item.start",
                Some(Command::Start),
            ),
            Item::Large(
                "media-playback-pause",
                "ribbon.item.pause",
                Some(Command::Pause),
            ),
            Item::Large(
                "media-skip-forward",
                "ribbon.item.step",
                Some(Command::Step),
            ),
            Item::Large(
                "media-seek-forward",
                "ribbon.item.until_change",
                Some(Command::UntilChange),
            ),
            Item::Large(
                "media-playback-stop",
                "ribbon.item.stop",
                Some(Command::Stop),
            ),
            Item::Large("view-refresh", "ribbon.item.reset", Some(Command::SimReset)),
        ],
    },
    Group {
        name: "ribbon.group.speed",
        launcher: false,
        items: &[
            Item::Large(
                "speed-0.25",
                "ribbon.item.speed_quarter",
                Some(Command::Speed(Speed::Quarter)),
            ),
            Item::Large(
                "speed-1",
                "ribbon.item.speed_normal",
                Some(Command::Speed(Speed::Normal)),
            ),
            Item::Large(
                "speed-2",
                "ribbon.item.speed_double",
                Some(Command::Speed(Speed::Double)),
            ),
        ],
    },
];

pub(super) const TABS: &[Tab] = &[
    Tab {
        name: "ribbon.tab.file",
        groups: ARCHIVO,
    },
    Tab {
        name: "ribbon.tab.edit",
        groups: EDITAR,
    },
    Tab {
        name: "ribbon.tab.view",
        groups: VER,
    },
    Tab {
        name: "ribbon.tab.block",
        groups: CAJETIN,
    },
    Tab {
        name: "ribbon.tab.simulate",
        groups: SIMULAR,
    },
];
