// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The ribbon's commands, as values.
//!
//! The table carries a [`Command`], never a label: two controls with the same label are still
//! different commands, and a label cannot decide an action. The window's dispatch is
//! exhaustive, so a command without a handler does not compile.

/// One action the ribbon can ask the window to run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Command {
    ProjectNew,
    ProjectOpen,
    ProjectSave,
    ProjectSaveAs,
    ProjectExport,
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
    pub(crate) const ALL: [Self; 31] = [
        Self::ProjectNew,
        Self::ProjectOpen,
        Self::ProjectSave,
        Self::ProjectSaveAs,
        Self::ProjectExport,
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
