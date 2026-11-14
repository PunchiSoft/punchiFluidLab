// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The sheet as the file stores it: paper, orientation and frame.

use punchi_fluid_core::paper::{Margins, Orientation, PaperSize};
use serde::{Deserialize, Serialize};

/// Paper as the file stores it.
///
/// The names are part of the format, not the interface label: `a4` stays `a4` even if the
/// window ever calls it something else. A custom sheet carries its measurements, and the
/// bounds are checked when the file is read.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StoredPaper {
    /// 148 × 210 mm.
    A5,
    /// 210 × 297 mm.
    #[default]
    A4,
    /// 297 × 420 mm.
    A3,
    /// 420 × 594 mm.
    A2,
    /// 594 × 841 mm.
    A1,
    /// 841 × 1189 mm.
    A0,
    /// 215.9 × 279.4 mm.
    Letter,
    /// 215.9 × 355.6 mm.
    Legal,
    /// 216 × 330 mm.
    Oficio,
    /// A sheet of the given millimetres.
    Custom {
        /// Width in millimetres.
        width_mm: f32,
        /// Height in millimetres.
        height_mm: f32,
    },
}

impl StoredPaper {
    /// The domain paper, or `None` when a custom size is outside the bounds.
    fn to_paper(self) -> Option<PaperSize> {
        Some(match self {
            Self::A5 => PaperSize::A5,
            Self::A4 => PaperSize::A4,
            Self::A3 => PaperSize::A3,
            Self::A2 => PaperSize::A2,
            Self::A1 => PaperSize::A1,
            Self::A0 => PaperSize::A0,
            Self::Letter => PaperSize::Letter,
            Self::Legal => PaperSize::Legal,
            Self::Oficio => PaperSize::Oficio,
            Self::Custom {
                width_mm,
                height_mm,
            } => PaperSize::custom(width_mm, height_mm)?,
        })
    }

    /// The stored form of a domain paper.
    fn from_paper(paper: PaperSize) -> Self {
        match paper {
            PaperSize::A5 => Self::A5,
            PaperSize::A4 => Self::A4,
            PaperSize::A3 => Self::A3,
            PaperSize::A2 => Self::A2,
            PaperSize::A1 => Self::A1,
            PaperSize::A0 => Self::A0,
            PaperSize::Letter => Self::Letter,
            PaperSize::Legal => Self::Legal,
            PaperSize::Oficio => Self::Oficio,
            PaperSize::Custom {
                width_mm,
                height_mm,
            } => Self::Custom {
                width_mm,
                height_mm,
            },
        }
    }
}

/// Frame margins as the file stores them, in millimetres.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredMargins {
    /// Top.
    pub top: f32,
    /// Right.
    pub right: f32,
    /// Bottom.
    pub bottom: f32,
    /// Left.
    pub left: f32,
}

impl Default for StoredMargins {
    fn default() -> Self {
        Self::from(Margins::iso5457())
    }
}

impl From<Margins> for StoredMargins {
    fn from(margins: Margins) -> Self {
        Self {
            top: margins.top,
            right: margins.right,
            bottom: margins.bottom,
            left: margins.left,
        }
    }
}

/// The sheet a circuit is drawn on, as the file stores it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredSheet {
    /// Paper; absent means A4.
    #[serde(default)]
    pub paper: StoredPaper,
    /// `true` when the paper is turned on its side.
    #[serde(default, skip_serializing_if = "is_false")]
    pub landscape: bool,
    /// Frame around the drawing space.
    #[serde(default)]
    pub margins: StoredMargins,
}

impl Default for StoredSheet {
    /// The sheet a file written before format 7 meant: A4 with no frame. The drawing area
    /// was a bare rectangle, so nothing it holds may become "outside the frame" on opening.
    fn default() -> Self {
        Self {
            paper: StoredPaper::A4,
            landscape: false,
            margins: StoredMargins::from(Margins::unframed()),
        }
    }
}

/// Skips a field that carries the default.
// serde hands the field to the predicate by reference, whatever its size.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_false(value: &bool) -> bool {
    !*value
}

impl StoredSheet {
    /// A fresh A4 with the ISO 5457 frame, the sheet a new document starts on.
    pub(crate) fn a4() -> Self {
        Self::from(punchi_fluid_core::paper::Sheet::a4())
    }

    /// Resolves the stored sheet, refusing a paper or frame that cannot exist.
    pub(crate) fn resolve(self) -> Result<punchi_fluid_core::paper::Sheet, String> {
        let paper = self
            .paper
            .to_paper()
            .ok_or_else(|| String::from("the sheet's paper is outside the bounds"))?;
        let margins = Margins {
            top: self.margins.top,
            right: self.margins.right,
            bottom: self.margins.bottom,
            left: self.margins.left,
        };
        let sheet = punchi_fluid_core::paper::Sheet {
            paper,
            orientation: if self.landscape {
                Orientation::Landscape
            } else {
                Orientation::Portrait
            },
            margins,
        };
        if !sheet.is_valid() {
            return Err(String::from("the sheet's frame leaves no drawing space"));
        }
        Ok(sheet)
    }
}

impl From<punchi_fluid_core::paper::Sheet> for StoredSheet {
    fn from(sheet: punchi_fluid_core::paper::Sheet) -> Self {
        Self {
            paper: StoredPaper::from_paper(sheet.paper),
            landscape: sheet.orientation.is_landscape(),
            margins: StoredMargins::from(sheet.margins),
        }
    }
}
