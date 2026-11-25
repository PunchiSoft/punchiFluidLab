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

    /// The catalog key of the label the ribbon shows.
    #[must_use]
    pub(crate) const fn label_key(self) -> &'static str {
        match self {
            Self::Quarter => "ribbon.item.speed_quarter",
            Self::Normal => "ribbon.item.speed_normal",
            Self::Double => "ribbon.item.speed_double",
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
    FitSheet,
    ViewBack,
    BlockVisible,
    ZoomWindow,
    Hand,
    Grid,
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
    pub(crate) const ALL: [Self; 57] = [
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
        Self::FitSheet,
        Self::ViewBack,
        Self::BlockVisible,
        Self::ZoomWindow,
        Self::Hand,
        Self::Grid,
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

    /// The catalog key the status bar shows and the hint uses; the visible label of the
    /// control is the table's, and for the two "new" commands they differ on purpose.
    #[must_use]
    pub(crate) fn label_key(self) -> &'static str {
        match self {
            Self::ProjectNew => "ribbon.item.new",
            Self::ProjectOpen => "ribbon.item.open",
            Self::ProjectSave => "ribbon.item.save",
            Self::ProjectSaveAs => "ribbon.item.save_as",
            Self::ProjectExport => "ribbon.item.export",
            Self::ExportOptions => "ribbon.item.options",
            Self::PageMargin(_) => "ribbon.item.margins",
            Self::PageOrientation(_) => "ribbon.item.orientation",
            Self::PagePaper(_) => "ribbon.item.paper",
            Self::BlockModel(_) => "ribbon.item.block_content",
            Self::BlockPosition(_) => "ribbon.item.block_position",
            Self::BlockReset | Self::SimReset => "ribbon.item.reset",
            Self::BlockEdit => "ribbon.item.block_edit",
            Self::Check => "ribbon.item.check",
            Self::CircuitRename => "ribbon.item.circuit_rename",
            Self::CircuitClose => "ribbon.item.circuit_close",
            Self::PipeClass(_) => "ribbon.item.pipe",
            Self::Tube => "ribbon.item.diameter",
            Self::Distribute(_) => "ribbon.item.distribute",
            Self::SelectAll => "ribbon.item.select_all",
            Self::Speed(speed) => speed.label_key(),
            Self::PageSetup => "ribbon.page_setup",
            Self::Home => "ribbon.item.home",
            Self::OpenRecent(_) => "ribbon.item.open_recent",
            Self::CircuitNew => "ribbon.item.circuit_new",
            Self::Undo => "ribbon.item.undo",
            Self::Redo => "ribbon.item.redo",
            Self::Copy => "ribbon.item.copy",
            Self::Paste => "ribbon.item.paste",
            Self::Duplicate => "ribbon.item.duplicate",
            Self::Delete => "ribbon.item.delete",
            Self::Rotate => "ribbon.item.rotate",
            Self::Mirror => "ribbon.item.mirror",
            Self::AlignLeft => "ribbon.item.align_left",
            Self::AlignRight => "ribbon.item.align_right",
            Self::AlignTop => "ribbon.item.align_top",
            Self::AlignBottom => "ribbon.item.align_bottom",
            Self::ZoomIn => "ribbon.item.zoom_in",
            Self::ZoomOut => "ribbon.item.zoom_out",
            Self::ZoomReset => "ribbon.item.zoom_reset",
            Self::FitSheet => "ribbon.item.fit_sheet",
            Self::ViewBack => "ribbon.item.view_back",
            Self::BlockVisible => "ribbon.item.block_visible",
            Self::ZoomWindow => "ribbon.item.zoom_window",
            Self::Hand => "ribbon.item.hand",
            Self::Grid => "ribbon.item.grid",
            Self::Catalog => "ribbon.item.catalog",
            Self::Properties => "ribbon.item.properties",
            Self::Autohide => "ribbon.item.autohide",
            Self::ResetView => "ribbon.item.reset_view",
            Self::Theme => "ribbon.item.theme",
            Self::Start => "ribbon.item.start",
            Self::Pause => "ribbon.item.pause",
            Self::Step => "ribbon.item.step",
            Self::UntilChange => "ribbon.item.until_change",
            Self::Stop => "ribbon.item.stop",
        }
    }
}
