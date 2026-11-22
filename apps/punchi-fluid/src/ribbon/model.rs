// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The ribbon data: tabs, groups, commands and the gallery previews.

pub(super) enum Preview {
    Margins(f32),
    Pipe(&'static str),
}

pub(super) struct GalleryEntry {
    pub(super) label: &'static str,
    pub(super) preview: Preview,
}

pub(super) enum Item {
    Large(&'static str, &'static str),
    Small(&'static str, &'static str),
    Menu(&'static str, &'static str, &'static [&'static str]),
    Gallery(&'static str, &'static str, &'static [GalleryEntry]),
}

pub(super) const MARGINS: &[GalleryEntry] = &[
    GalleryEntry {
        label: "Normal",
        preview: Preview::Margins(0.55),
    },
    GalleryEntry {
        label: "Estrecho",
        preview: Preview::Margins(0.25),
    },
    GalleryEntry {
        label: "Moderado",
        preview: Preview::Margins(0.4),
    },
    GalleryEntry {
        label: "Ancho",
        preview: Preview::Margins(0.15),
    },
];

pub(super) const CONDUCTO: &[GalleryEntry] = &[
    GalleryEntry {
        label: "Trabajo",
        preview: Preview::Pipe("pipe-working"),
    },
    GalleryEntry {
        label: "Pilotaje",
        preview: Preview::Pipe("pipe-pilot"),
    },
    GalleryEntry {
        label: "Drenaje",
        preview: Preview::Pipe("pipe-drain"),
    },
    GalleryEntry {
        label: "Diámetro",
        preview: Preview::Pipe("tube-diameter"),
    },
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
            Item::Large("document-new", "Nuevo"),
            Item::Large("document-open", "Abrir"),
            Item::Large("document-save", "Guardar"),
            Item::Small("document-save-as", "Guardar como"),
            Item::Small("document-export", "Exportar"),
        ],
    },
    Group {
        name: "Circuito",
        launcher: false,
        items: &[
            Item::Small("list-add", "Nuevo"),
            Item::Small("edit-rename", "Renombrar"),
            Item::Small("window-close", "Cerrar"),
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
                &["Vertical", "Apaisado"],
            ),
            Item::Menu(
                "x-office-document",
                "Tamaño",
                &["A4", "A3", "A2", "Personalizado…"],
            ),
        ],
    },
    Group {
        name: "Recientes",
        launcher: false,
        items: &[Item::Menu(
            "document-open-recent",
            "Recientes",
            &["Sin recientes"],
        )],
    },
];

pub(super) const EDITAR: &[Group] = &[
    Group {
        name: "Historial",
        launcher: false,
        items: &[
            Item::Small("edit-undo", "Deshacer"),
            Item::Small("edit-redo", "Rehacer"),
        ],
    },
    Group {
        name: "Componente",
        launcher: false,
        items: &[
            Item::Small("edit-copy", "Copiar"),
            Item::Small("edit-paste", "Pegar"),
            Item::Small("edit-duplicate", "Duplicar"),
            Item::Small("edit-delete", "Borrar"),
            Item::Small("edit-select-all", "Todo"),
        ],
    },
    Group {
        name: "Transformar",
        launcher: false,
        items: &[
            Item::Small("object-rotate-right", "Girar"),
            Item::Small("object-flip-horizontal", "Reflejar"),
        ],
    },
    Group {
        name: "Alinear",
        launcher: false,
        items: &[
            Item::Small("align-horizontal-left", "Izquierda"),
            Item::Small("align-horizontal-right", "Derecha"),
            Item::Small("align-vertical-top", "Arriba"),
            Item::Small("align-vertical-bottom", "Abajo"),
            Item::Small("align-ports", "Puertos"),
        ],
    },
    Group {
        name: "Repartir",
        launcher: false,
        items: &[
            Item::Small("distribute-horizontal-equal", "Horizontal"),
            Item::Small("distribute-vertical-equal", "Vertical"),
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
            Item::Small("zoom-in", "Aumentar"),
            Item::Small("zoom-out", "Disminuir"),
            Item::Small("zoom-original", "Tamaño original"),
        ],
    },
    Group {
        name: "Paneles",
        launcher: false,
        items: &[
            Item::Small("view-list-details", "Catálogo"),
            Item::Small("document-properties", "Propiedades"),
            Item::Small("go-bottom", "Ocultar al salir"),
            Item::Small("edit-reset", "Restablecer vista"),
        ],
    },
    Group {
        name: "Tema",
        launcher: false,
        items: &[Item::Small("preferences-desktop-theme", "Tema")],
    },
];

pub(super) const CAJETIN: &[Group] = &[
    Group {
        name: "Contenido",
        launcher: false,
        items: &[Item::Menu(
            "x-office-document",
            "Contenido",
            &["Técnico", "Escolar", "Industrial", "Sin cajetín"],
        )],
    },
    Group {
        name: "Posición",
        launcher: false,
        items: &[Item::Menu(
            "go-top",
            "Posición",
            &["Superior derecha", "Inferior derecha"],
        )],
    },
    Group {
        name: "Datos",
        launcher: false,
        items: &[
            Item::Small("x-office-address-book", "Editar datos…"),
            Item::Small("edit-reset", "Restablecer"),
        ],
    },
];

pub(super) const SIMULAR: &[Group] = &[
    Group {
        name: "Transporte",
        launcher: false,
        items: &[
            Item::Large("media-playback-start", "Iniciar"),
            Item::Large("media-playback-pause", "Pausa"),
            Item::Large("media-skip-forward", "Paso"),
            Item::Large("media-seek-forward", "Hasta cambio"),
            Item::Large("media-playback-stop", "Detener"),
            Item::Large("view-refresh", "Restablecer"),
        ],
    },
    Group {
        name: "Velocidad",
        launcher: false,
        items: &[
            Item::Large("speed-0.25", "0,25×"),
            Item::Large("speed-1", "1×"),
            Item::Large("speed-2", "2×"),
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
