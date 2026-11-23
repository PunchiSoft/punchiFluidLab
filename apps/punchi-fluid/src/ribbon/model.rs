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
}

pub(super) const MARGINS: &[GalleryEntry] = &[
    GalleryEntry::command(
        "Normal",
        Preview::Margins(0.3),
        Command::PageMargin(MarginPreset::Normal),
    ),
    GalleryEntry::command(
        "Estrecho",
        Preview::Margins(0.15),
        Command::PageMargin(MarginPreset::Narrow),
    ),
    GalleryEntry::command(
        "Moderado",
        Preview::Margins(0.45),
        Command::PageMargin(MarginPreset::Moderate),
    ),
    GalleryEntry::command(
        "Ancho",
        Preview::Margins(0.6),
        Command::PageMargin(MarginPreset::Wide),
    ),
];

pub(super) const CONDUCTO: &[GalleryEntry] = &[
    GalleryEntry::command(
        "Trabajo",
        Preview::Pipe("pipe-working"),
        Command::PipeClass(LineClass::Working),
    ),
    GalleryEntry::command(
        "Pilotaje",
        Preview::Pipe("pipe-pilot"),
        Command::PipeClass(LineClass::Pilot),
    ),
    GalleryEntry::command(
        "Drenaje",
        Preview::Pipe("pipe-drain"),
        Command::PipeClass(LineClass::Drain),
    ),
    GalleryEntry::command("Diámetro", Preview::Pipe("tube-diameter"), Command::Tube),
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
        name: "Proyecto",
        launcher: false,
        items: &[
            Item::Large("document-new", "Nuevo", Some(Command::ProjectNew)),
            Item::Large("document-open", "Abrir", Some(Command::ProjectOpen)),
            Item::Large("document-save", "Guardar", Some(Command::ProjectSave)),
            Item::Small(
                "document-save-as",
                "Guardar como",
                Some(Command::ProjectSaveAs),
            ),
            Item::Small("document-export", "Exportar", Some(Command::ProjectExport)),
        ],
    },
    Group {
        name: "Circuito",
        launcher: false,
        items: &[
            Item::Small("list-add", "Nuevo circuito", Some(Command::CircuitNew)),
            Item::Small("edit-rename", "Renombrar", None),
            Item::Small("window-close", "Cerrar", None),
        ],
    },
    Group {
        name: "Configurar página",
        launcher: true,
        items: &[
            Item::Gallery("document-properties", "Márgenes", MARGINS),
            Item::Menu(
                "document-properties",
                "Orientación",
                &[
                    MenuEntry::command(
                        "Vertical",
                        Command::PageOrientation(punchi_fluid_doc::paper::Orientation::Portrait),
                    ),
                    MenuEntry::command(
                        "Apaisado",
                        Command::PageOrientation(punchi_fluid_doc::paper::Orientation::Landscape),
                    ),
                ],
            ),
            Item::Menu(
                "x-office-document",
                "Tamaño",
                &[
                    MenuEntry::command(
                        "A4",
                        Command::PagePaper(punchi_fluid_doc::paper::PaperSize::A4),
                    ),
                    MenuEntry::command(
                        "A3",
                        Command::PagePaper(punchi_fluid_doc::paper::PaperSize::A3),
                    ),
                    MenuEntry::command(
                        "A2",
                        Command::PagePaper(punchi_fluid_doc::paper::PaperSize::A2),
                    ),
                    MenuEntry::unavailable("Personalizado…"),
                ],
            ),
        ],
    },
    Group {
        name: "Recientes",
        launcher: false,
        items: &[Item::Menu(
            "document-open-recent",
            "Recientes",
            &[MenuEntry::unavailable("Sin recientes")],
        )],
    },
];

pub(super) const EDITAR: &[Group] = &[
    Group {
        name: "Historial",
        launcher: false,
        items: &[
            Item::Small("edit-undo", "Deshacer", Some(Command::Undo)),
            Item::Small("edit-redo", "Rehacer", Some(Command::Redo)),
        ],
    },
    Group {
        name: "Componente",
        launcher: false,
        items: &[
            Item::Small("edit-copy", "Copiar", Some(Command::Copy)),
            Item::Small("edit-paste", "Pegar", Some(Command::Paste)),
            Item::Small("edit-duplicate", "Duplicar", Some(Command::Duplicate)),
            Item::Small("edit-delete", "Borrar", Some(Command::Delete)),
            Item::Small("edit-select-all", "Todo", Some(Command::SelectAll)),
        ],
    },
    Group {
        name: "Transformar",
        launcher: false,
        items: &[
            Item::Small("object-rotate-right", "Girar", Some(Command::Rotate)),
            Item::Small("object-flip-horizontal", "Reflejar", Some(Command::Mirror)),
        ],
    },
    Group {
        name: "Alinear",
        launcher: false,
        items: &[
            Item::Small(
                "align-horizontal-left",
                "Izquierda",
                Some(Command::AlignLeft),
            ),
            Item::Small(
                "align-horizontal-right",
                "Derecha",
                Some(Command::AlignRight),
            ),
            Item::Small("align-vertical-top", "Arriba", Some(Command::AlignTop)),
            Item::Small("align-vertical-bottom", "Abajo", Some(Command::AlignBottom)),
            Item::Small("align-ports", "Puertos", None),
        ],
    },
    Group {
        name: "Repartir",
        launcher: false,
        items: &[
            Item::Small(
                "distribute-horizontal-equal",
                "Horizontal",
                Some(Command::Distribute(Axis::Horizontal)),
            ),
            Item::Small(
                "distribute-vertical-equal",
                "Vertical",
                Some(Command::Distribute(Axis::Vertical)),
            ),
        ],
    },
    Group {
        name: "Conducto",
        launcher: false,
        items: &[Item::Gallery("pipe-working", "Conducto", CONDUCTO)],
    },
];

pub(super) const VER: &[Group] = &[
    Group {
        name: "Vista",
        launcher: false,
        items: &[
            Item::Small("zoom-in", "Aumentar", Some(Command::ZoomIn)),
            Item::Small("zoom-out", "Disminuir", Some(Command::ZoomOut)),
            Item::Small("zoom-original", "Tamaño original", Some(Command::ZoomReset)),
        ],
    },
    Group {
        name: "Paneles",
        launcher: false,
        items: &[
            Item::Small("view-list-details", "Catálogo", Some(Command::Catalog)),
            Item::Small(
                "document-properties",
                "Propiedades",
                Some(Command::Properties),
            ),
            Item::Small("go-bottom", "Ocultar al salir", Some(Command::Autohide)),
            Item::Small("edit-reset", "Restablecer vista", Some(Command::ResetView)),
        ],
    },
    Group {
        name: "Tema",
        launcher: false,
        items: &[Item::Small(
            "preferences-desktop-theme",
            "Tema",
            Some(Command::Theme),
        )],
    },
];

pub(super) const CAJETIN: &[Group] = &[
    Group {
        name: "Contenido",
        launcher: false,
        items: &[Item::Menu(
            "x-office-document",
            "Contenido",
            &[
                MenuEntry::command("Técnico", Command::BlockModel(ContentModel::Technical)),
                MenuEntry::command("Escolar", Command::BlockModel(ContentModel::School)),
                MenuEntry::command("Industrial", Command::BlockModel(ContentModel::Industrial)),
                MenuEntry::command("Sin cajetín", Command::BlockPosition(BlockPosition::None)),
            ],
        )],
    },
    Group {
        name: "Posición",
        launcher: false,
        items: &[Item::Menu(
            "go-top",
            "Posición",
            &[
                MenuEntry::command(
                    "Superior derecha",
                    Command::BlockPosition(BlockPosition::TopRight),
                ),
                MenuEntry::command(
                    "Inferior derecha",
                    Command::BlockPosition(BlockPosition::BottomRight),
                ),
                MenuEntry::command(
                    "Inferior centrado",
                    Command::BlockPosition(BlockPosition::BottomFull),
                ),
            ],
        )],
    },
    Group {
        name: "Datos",
        launcher: false,
        items: &[
            Item::Small("x-office-address-book", "Editar datos…", None),
            Item::Small("edit-reset", "Restablecer", Some(Command::BlockReset)),
        ],
    },
];

pub(super) const SIMULAR: &[Group] = &[
    Group {
        name: "Transporte",
        launcher: false,
        items: &[
            Item::Large("media-playback-start", "Iniciar", Some(Command::Start)),
            Item::Large("media-playback-pause", "Pausa", Some(Command::Pause)),
            Item::Large("media-skip-forward", "Paso", Some(Command::Step)),
            Item::Large(
                "media-seek-forward",
                "Hasta cambio",
                Some(Command::UntilChange),
            ),
            Item::Large("media-playback-stop", "Detener", Some(Command::Stop)),
            Item::Large("view-refresh", "Restablecer", Some(Command::SimReset)),
        ],
    },
    Group {
        name: "Velocidad",
        launcher: false,
        items: &[
            Item::Large("speed-0.25", "0,25×", Some(Command::Speed(Speed::Quarter))),
            Item::Large("speed-1", "1×", Some(Command::Speed(Speed::Normal))),
            Item::Large("speed-2", "2×", Some(Command::Speed(Speed::Double))),
        ],
    },
];

pub(super) const TABS: &[Tab] = &[
    Tab {
        name: "Archivo",
        groups: ARCHIVO,
    },
    Tab {
        name: "Editar",
        groups: EDITAR,
    },
    Tab {
        name: "Ver",
        groups: VER,
    },
    Tab {
        name: "Cajetín",
        groups: CAJETIN,
    },
    Tab {
        name: "Simular",
        groups: SIMULAR,
    },
];
