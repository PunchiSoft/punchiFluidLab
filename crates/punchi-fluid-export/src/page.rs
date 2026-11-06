// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Paper, margins and the identification block.
//!
//! Sizes are the ISO 216 series plus the two American sheets the reference material
//! also uses, in millimetres. A drawing never decides its own paper: the page owns the
//! content rectangle and the drawing goes inside it.

use punchi_fluid_draw::{Pt, Rect};

/// Paper the sheet can be printed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaperSize {
    /// 148 × 210 mm.
    A5,
    /// 210 × 297 mm.
    A4,
    /// 297 × 420 mm.
    A3,
    /// 420 × 594 mm.
    A2,
    /// 215.9 × 279.4 mm.
    Letter,
    /// 215.9 × 355.6 mm.
    Legal,
}

impl PaperSize {
    /// Every size, in the order the panel lists them.
    pub const ALL: [Self; 6] = [
        Self::A5,
        Self::A4,
        Self::A3,
        Self::A2,
        Self::Letter,
        Self::Legal,
    ];

    /// Name shown in the interface and written in the title block.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::A5 => "A5",
            Self::A4 => "A4",
            Self::A3 => "A3",
            Self::A2 => "A2",
            Self::Letter => "Letter",
            Self::Legal => "Legal",
        }
    }

    /// Width and height in portrait orientation, in millimetres.
    #[must_use]
    pub const fn size_mm(self) -> (f32, f32) {
        match self {
            Self::A5 => (148.0, 210.0),
            Self::A4 => (210.0, 297.0),
            Self::A3 => (297.0, 420.0),
            Self::A2 => (420.0, 594.0),
            Self::Letter => (215.9, 279.4),
            Self::Legal => (215.9, 355.6),
        }
    }
}

/// Which way the paper is turned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Orientation {
    /// Taller than wide.
    #[default]
    Portrait,
    /// Wider than tall.
    Landscape,
}

/// Margin around the content, in millimetres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Margins {
    /// Top margin.
    pub top: f32,
    /// Right margin.
    pub right: f32,
    /// Bottom margin.
    pub bottom: f32,
    /// Left margin.
    pub left: f32,
}

impl Margins {
    /// The 18 mm frame of a technical drawing.
    #[must_use]
    pub const fn standard() -> Self {
        Self {
            top: 18.0,
            right: 18.0,
            bottom: 18.0,
            left: 18.0,
        }
    }

    /// `true` when every margin is finite and not negative.
    #[must_use]
    pub fn is_valid(self) -> bool {
        [self.top, self.right, self.bottom, self.left]
            .iter()
            .all(|value| value.is_finite() && *value >= 0.0)
    }
}

impl Default for Margins {
    fn default() -> Self {
        Self::standard()
    }
}

/// Text of the identification block.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TitleBlock {
    /// Name of the circuit.
    pub title: String,
    /// Project or course the drawing belongs to.
    pub project: String,
    /// Revision identifier.
    pub revision: String,
    /// Date, as the caller wrote it.
    pub date: String,
    /// Extra warning, for example that the symbols are not verified yet.
    pub note: Option<String>,
}

impl TitleBlock {
    /// A block with the fields a reader needs and nothing else.
    #[must_use]
    pub fn new(title: impl Into<String>, date: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            project: String::new(),
            revision: String::from("A"),
            date: date.into(),
            note: None,
        }
    }

    /// Adds the note that the symbols have not been verified against the standard.
    #[must_use]
    pub fn with_unverified_note(mut self) -> Self {
        self.note = Some(String::from("symbols not yet verified against ISO 1219-1"));
        self
    }

    /// Same block with a note only when `unverified`.
    #[must_use]
    pub fn with_note_if(mut self, unverified: bool) -> Self {
        if unverified {
            self.note = Some(String::from("symbols not yet verified against ISO 1219-1"));
        } else {
            self.note = None;
        }
        self
    }
}

/// One sheet: paper, orientation, margins and the block.
#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    /// Paper.
    pub paper: PaperSize,
    /// Orientation.
    pub orientation: Orientation,
    /// Margins.
    pub margins: Margins,
    /// Identification block, drawn at the bottom-right of the content area.
    pub title_block: Option<TitleBlock>,
}

impl Page {
    /// A4 portrait with the standard frame, the default the panel offers.
    #[must_use]
    pub fn a4(title_block: TitleBlock) -> Self {
        Self {
            paper: PaperSize::A4,
            orientation: Orientation::Portrait,
            margins: Margins::standard(),
            title_block: Some(title_block),
        }
    }

    /// Page size in millimetres, honouring the orientation.
    #[must_use]
    pub fn size_mm(&self) -> (f32, f32) {
        let (width, height) = self.paper.size_mm();
        match self.orientation {
            Orientation::Portrait => (width, height),
            Orientation::Landscape => (height, width),
        }
    }

    /// Whole page in millimetres.
    #[must_use]
    pub fn rect(&self) -> Rect {
        let (width, height) = self.size_mm();
        Rect::from_min_size(Pt::new(0.0, 0.0), width, height)
    }

    /// Area the drawing may use.
    ///
    /// Margins that do not fit leave an empty rectangle instead of an inside-out one:
    /// a caller that asks for 80 mm margins on a 148 mm sheet gets a refusal, not a
    /// 12 mm strip.
    #[must_use]
    pub fn content_rect(&self) -> Rect {
        let page = self.rect();
        let width = page.width() - self.margins.left - self.margins.right;
        let height = page.height() - self.margins.top - self.margins.bottom;
        if width <= 0.0 || height <= 0.0 {
            return Rect::default();
        }
        Rect::from_min_size(
            Pt::new(
                page.min.x + self.margins.left,
                page.min.y + self.margins.top,
            ),
            width,
            height,
        )
    }

    /// Rectangle of the identification block, inside the content area.
    #[must_use]
    pub fn title_rect(&self) -> Rect {
        let content = self.content_rect();
        let height = (content.height() * 0.14).clamp(12.0, 32.0);
        let width = (content.width() * 0.42).clamp(50.0, 110.0);
        Rect::from_corners(
            Pt::new(content.max.x - width, content.max.y - height),
            content.max,
        )
    }

    /// Area the drawing itself may use: the content without the title block.
    #[must_use]
    pub fn drawing_rect(&self) -> Rect {
        let content = self.content_rect();
        if self.title_block.is_none() {
            return content;
        }
        let title = self.title_rect();
        Rect::from_corners(content.min, Pt::new(content.max.x, title.min.y))
    }

    /// Pixel size of a raster export at `dpi`.
    #[must_use]
    pub fn pixel_size(&self, dpi: f32) -> (u32, u32) {
        let (width, height) = self.size_mm();
        let per_mm = dpi / 25.4;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        (
            (width * per_mm).round() as u32,
            (height * per_mm).round() as u32,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a4_portrait_at_300_dpi_is_2480_by_3508() {
        let page = Page::a4(TitleBlock::default());
        assert_eq!(page.pixel_size(300.0), (2480, 3508));
    }

    #[test]
    fn a3_landscape_at_300_dpi_is_4961_by_3508() {
        let page = Page {
            paper: PaperSize::A3,
            orientation: Orientation::Landscape,
            ..Page::a4(TitleBlock::default())
        };
        assert_eq!(page.pixel_size(300.0), (4961, 3508));
    }

    #[test]
    fn landscape_swaps_the_sides() {
        let portrait = Page::a4(TitleBlock::default());
        let landscape = Page {
            orientation: Orientation::Landscape,
            ..portrait.clone()
        };
        assert_eq!(portrait.size_mm(), (210.0, 297.0));
        assert_eq!(landscape.size_mm(), (297.0, 210.0));
    }

    #[test]
    fn the_content_rect_is_the_page_minus_the_margins() {
        let page = Page::a4(TitleBlock::default());
        let content = page.content_rect();
        assert!((content.min.x - 18.0).abs() < f32::EPSILON);
        assert!((content.width() - (210.0 - 36.0)).abs() < f32::EPSILON);
        assert!((content.height() - (297.0 - 36.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn the_title_block_sits_inside_the_content_and_reserves_its_height() {
        let page = Page::a4(TitleBlock::default());
        let content = page.content_rect();
        let title = page.title_rect();
        let drawing = page.drawing_rect();
        assert!(content.contains(title.min) && content.contains(title.max));
        assert!(drawing.max.y <= title.min.y + f32::EPSILON);
        assert!(drawing.height() > content.height() * 0.5);
    }

    #[test]
    fn every_paper_is_a_valid_page() {
        for paper in PaperSize::ALL {
            let page = Page {
                paper,
                ..Page::a4(TitleBlock::default())
            };
            assert!(!page.content_rect().is_empty(), "{paper:?}");
            assert!(page.margins.is_valid());
        }
    }
}
