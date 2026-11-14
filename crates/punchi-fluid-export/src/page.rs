// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Paper, margins and the identification block.
//!
//! The paper and its frame are domain values: [`punchi_fluid_core::paper`] owns the
//! ISO 216 sizes and the ISO 5457 margins, and this crate composes them into the page a
//! writer prints. A drawing never decides its own paper: the page owns the content
//! rectangle and the drawing goes inside it.

pub use punchi_fluid_core::{Margins, Orientation, PaperSize};
use punchi_fluid_draw::{Pt, Rect};

use crate::parts::PartsList;

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
    /// Scale of the drawing on the sheet, for example `1:2`; absent when it is 1:1.
    pub scale: Option<String>,
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
            scale: None,
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
    /// Parts list, drawn in a column above the block. `None` prints no list.
    pub parts: Option<PartsList>,
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
            parts: None,
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

    /// Column the parts list occupies, or an empty rectangle when there is no list.
    ///
    /// It shares the width of the block and grows upward from it, so the printed sheet
    /// reads block, list, drawing from the bottom right, and the drawing keeps the rest.
    #[must_use]
    pub fn parts_rect(&self) -> Rect {
        let content = self.content_rect();
        let Some(parts) = &self.parts else {
            return Rect::default();
        };
        if parts.is_empty() {
            return Rect::default();
        }
        let title = self.title_rect();
        let width = title.width();
        #[allow(clippy::cast_precision_loss)]
        let rows = parts.rows.len() as f32 + if parts.omitted > 0 { 1.0 } else { 0.0 };
        let height = (rows * 4.4 + 6.0).min(content.height() * 0.55);
        Rect::from_corners(
            Pt::new(title.min.x, title.min.y - 2.0 - height),
            Pt::new(title.min.x + width, title.min.y - 2.0),
        )
    }

    /// Area the drawing itself may use: the content without the title block and the list.
    #[must_use]
    pub fn drawing_rect(&self) -> Rect {
        let content = self.content_rect();
        if self.title_block.is_none() {
            return content;
        }
        let title = self.title_rect();
        let parts = self.parts_rect();
        let right = if parts.is_empty() {
            content.max.x
        } else {
            parts.min.x - 2.0
        };
        Rect::from_corners(content.min, Pt::new(right, title.min.y))
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
    fn the_parts_column_takes_room_from_the_drawing() {
        let mut page = Page::a4(TitleBlock::default());
        let without = page.drawing_rect();
        page.parts = Some(PartsList::group(
            &punchi_fluid_catalog::Catalog::builtin(),
            [
                ("cylinder.single_acting_20_100", Some("1A1")),
                ("supply.compressor", None),
                ("valve.3_2.spring_return", Some("1V1")),
            ],
        ));
        let with = page.drawing_rect();
        let parts = page.parts_rect();
        assert!(!parts.is_empty());
        assert!(with.max.x < without.max.x, "the list kept its room");
        assert!(
            with.max.x <= parts.min.x,
            "the drawing runs under the parts list"
        );
        let content = page.content_rect();
        assert!(
            parts.min.x >= content.min.x
                && parts.max.x <= content.max.x
                && parts.min.y >= content.min.y
                && parts.max.y <= content.max.y,
            "the list left the page: {parts:?} against {content:?}"
        );
        // A page without a list draws exactly where it always did.
        page.parts = None;
        assert_eq!(page.drawing_rect(), without);
        assert!(page.parts_rect().is_empty());
    }

    #[test]
    fn a4_portrait_at_300_dpi_is_2480_by_3508() {
        let page = Page::a4(TitleBlock::default());
        assert_eq!(page.pixel_size(300.0), (2480, 3508));
    }

    #[test]
    fn the_content_area_stays_inside_the_paper_for_every_size() {
        for paper in PaperSize::ALL {
            let page = Page {
                paper,
                ..Page::a4(TitleBlock::default())
            };
            let page_rect = page.rect();
            let content = page.content_rect();
            let drawing = page.drawing_rect();
            assert!(
                page_rect.contains(content.min) && page_rect.contains(content.max),
                "{}: the content leaves the paper",
                paper.label()
            );
            assert!(
                content.contains(drawing.min) && content.contains(drawing.max),
                "{}: the drawing leaves the content area",
                paper.label()
            );
            let (width, height) = page.pixel_size(96.0);
            assert!(
                width > 0 && height > 0,
                "{}: a printable sheet has pixels",
                paper.label()
            );
        }
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
        // ISO 5457: 20 mm on the filing edge and 10 mm on the other three.
        assert!((content.min.x - 20.0).abs() < f32::EPSILON);
        assert!((content.min.y - 10.0).abs() < f32::EPSILON);
        assert!((content.width() - (210.0 - 30.0)).abs() < f32::EPSILON);
        assert!((content.height() - (297.0 - 20.0)).abs() < f32::EPSILON);
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
