// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The ribbon's commands, as values.
//!
//! The table carries a [`Command`], never a label: two controls with the same label are still
//! different commands, and a label cannot decide an action. The window's dispatch is
//! exhaustive, so a command without a handler does not compile.

use punchi_fluid_doc::Axis;
use punchi_fluid_doc::paper::{Orientation, PaperSize};
use punchi_fluid_doc::title_block::{BlockPosition, ContentModel};
use punchi_fluid_draw::LineClass;

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

/// The simulation's clock speed, the three the reference offers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Speed {
    Quarter,
    Normal,
    Double,
}

impl Speed {
    /// The factor the frame's time step is multiplied by.
    #[must_use]
    pub(crate) const fn factor(self) -> f32 {
        match self {
            Self::Quarter => 0.25,
            Self::Normal => 1.0,
            Self::Double => 2.0,
        }
    }

    /// The label the ribbon shows and the status bar repeats.
    #[must_use]
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Quarter => "0,25×",
            Self::Normal => "1×",
            Self::Double => "2×",
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
    ExportOptions,
    PageMargin(MarginPreset),
    PageOrientation(Orientation),
    PagePaper(PaperSize),
    BlockModel(ContentModel),
    BlockPosition(BlockPosition),
    BlockReset,
    BlockEdit,
    Check,
    CircuitRename,
    CircuitClose,
    PipeClass(LineClass),
    Tube,
    Distribute(Axis),
    SelectAll,
    SimReset,
    Speed(Speed),
    PageSetup,
    Home,
    OpenRecent(usize),
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
    pub(crate) const ALL: [Self; 51] = [
        Self::ProjectNew,
        Self::ProjectOpen,
        Self::ProjectSave,
        Self::ProjectSaveAs,
        Self::ProjectExport,
        Self::ExportOptions,
        Self::PageMargin(MarginPreset::Normal),
        Self::PageOrientation(Orientation::Portrait),
        Self::PagePaper(PaperSize::A3),
        Self::BlockModel(ContentModel::Industrial),
        Self::BlockPosition(BlockPosition::BottomRight),
        Self::BlockReset,
        Self::BlockEdit,
        Self::Check,
        Self::CircuitRename,
        Self::CircuitClose,
        Self::PipeClass(LineClass::Working),
        Self::Tube,
        Self::Distribute(Axis::Horizontal),
        Self::SelectAll,
        Self::SimReset,
        Self::Speed(Speed::Normal),
        Self::PageSetup,
        Self::Home,
        Self::OpenRecent(0),
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
            Self::ExportOptions => "Opciones…",
            Self::PageMargin(_) => "Márgenes",
            Self::PageOrientation(_) => "Orientación",
            Self::PagePaper(_) => "Tamaño",
            Self::BlockModel(_) => "Contenido",
            Self::BlockPosition(_) => "Posición",
            Self::BlockReset | Self::SimReset => "Restablecer",
            Self::BlockEdit => "Editar datos",
            Self::Check => "Comprobar",
            Self::CircuitRename => "Renombrar",
            Self::CircuitClose => "Cerrar",
            Self::PipeClass(_) => "Conducto",
            Self::Tube => "Diámetro",
            Self::Distribute(_) => "Repartir",
            Self::SelectAll => "Todo",
            Self::Speed(speed) => speed.label(),
            Self::PageSetup => "Configurar página",
            Self::Home => "Inicio",
            Self::OpenRecent(_) => "Abrir reciente",
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
