// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The ribbon's commands, as values.
//!
//! The table carries a [`Command`], never a label: two controls with the same label are still
//! different commands, and a label cannot decide an action. The window's dispatch is
//! exhaustive, so a command without a handler does not compile.

use punchi_fluid_doc::paper::{Orientation, PaperSize};

/// A named set of margins, so the gallery carries a command and the window maps it to the
/// document's millimetres.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MarginPreset {
    Narrow,
    Normal,
    Moderate,
    Wide,
}

impl MarginPreset {
    /// The margins in millimetres; ISO 5457 for `Normal`, a plain frame for the rest.
    #[must_use]
    pub(crate) fn margins(self) -> punchi_fluid_doc::paper::Margins {
        match self {
            Self::Narrow => punchi_fluid_doc::paper::Margins {
                top: 5.0,
                right: 5.0,
                bottom: 5.0,
                left: 5.0,
            },
            Self::Normal => punchi_fluid_doc::paper::Margins::iso5457(),
            Self::Moderate => punchi_fluid_doc::paper::Margins {
                top: 15.0,
                right: 15.0,
                bottom: 15.0,
                left: 15.0,
            },
            Self::Wide => punchi_fluid_doc::paper::Margins {
                top: 25.0,
                right: 25.0,
                bottom: 25.0,
                left: 25.0,
            },
        }
    }
}

/// One action the ribbon can ask the window to run.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Command {
    ProjectNew,
    ProjectOpen,
    ProjectSave,
    ProjectSaveAs,
    ProjectExport,
    PageMargin(MarginPreset),
    PageOrientation(Orientation),
    PagePaper(PaperSize),
    CircuitNew,
    Undo,
    Redo,
    Copy,
    Paste,
    Duplicate,
    Delete,
    Rotate,
    Mirror,
    AlignLeft,
    AlignRight,
    AlignTop,
    AlignBottom,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    Catalog,
    Properties,
    Autohide,
    ResetView,
    Theme,
    Start,
    Pause,
    Step,
    UntilChange,
    Stop,
}

impl Command {
    /// Every command the window handles, for the tests that prove the table and the dispatch
    /// cover the same set.
    #[allow(
        dead_code,
        reason = "the tests iterate the set; the compile-time half is the exhaustive dispatch"
    )]
    pub(crate) const ALL: [Self; 34] = [
        Self::ProjectNew,
        Self::ProjectOpen,
        Self::ProjectSave,
        Self::ProjectSaveAs,
        Self::ProjectExport,
        Self::PageMargin(MarginPreset::Normal),
        Self::PageOrientation(Orientation::Portrait),
        Self::PagePaper(PaperSize::A3),
        Self::CircuitNew,
        Self::Undo,
        Self::Redo,
        Self::Copy,
        Self::Paste,
        Self::Duplicate,
        Self::Delete,
        Self::Rotate,
        Self::Mirror,
        Self::AlignLeft,
        Self::AlignRight,
        Self::AlignTop,
        Self::AlignBottom,
        Self::ZoomIn,
        Self::ZoomOut,
        Self::ZoomReset,
        Self::Catalog,
        Self::Properties,
        Self::Autohide,
        Self::ResetView,
        Self::Theme,
        Self::Start,
        Self::Pause,
        Self::Step,
        Self::UntilChange,
        Self::Stop,
    ];

    /// The Spanish name the status bar shows and the hint uses; the visible label of the
    /// control is the table's, and for the two "new" commands they differ on purpose.
    #[must_use]
    pub(crate) fn label_es(self) -> &'static str {
        match self {
            Self::ProjectNew => "Nuevo",
            Self::ProjectOpen => "Abrir",
            Self::ProjectSave => "Guardar",
            Self::ProjectSaveAs => "Guardar como",
            Self::ProjectExport => "Exportar",
            Self::PageMargin(_) => "Márgenes",
            Self::PageOrientation(_) => "Orientación",
            Self::PagePaper(_) => "Tamaño",
            Self::CircuitNew => "Nuevo circuito",
            Self::Undo => "Deshacer",
            Self::Redo => "Rehacer",
            Self::Copy => "Copiar",
            Self::Paste => "Pegar",
            Self::Duplicate => "Duplicar",
            Self::Delete => "Borrar",
            Self::Rotate => "Girar",
            Self::Mirror => "Reflejar",
            Self::AlignLeft => "Izquierda",
            Self::AlignRight => "Derecha",
            Self::AlignTop => "Arriba",
            Self::AlignBottom => "Abajo",
            Self::ZoomIn => "Aumentar",
            Self::ZoomOut => "Disminuir",
            Self::ZoomReset => "Tamaño original",
            Self::Catalog => "Catálogo",
            Self::Properties => "Propiedades",
            Self::Autohide => "Ocultar al salir",
            Self::ResetView => "Restablecer vista",
            Self::Theme => "Tema",
            Self::Start => "Iniciar",
            Self::Pause => "Pausa",
            Self::Step => "Paso",
            Self::UntilChange => "Hasta cambio",
            Self::Stop => "Detener",
        }
    }
}
