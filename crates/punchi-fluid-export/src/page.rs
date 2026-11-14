// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Paper, margins and the identification block.
//!
//! The paper and its frame are domain values: [`punchi_fluid_core::paper`] owns the
//! ISO 216 sizes and the ISO 5457 margins, and this crate composes them into the page a
//! writer prints. A drawing never decides its own paper: the page owns the content
//! rectangle and the drawing goes inside it.

pub use punchi_fluid_core::{BlockPosition, Margins, Orientation, PaperSize};
use punchi_fluid_draw::{Pt, Rect};

use crate::parts::PartsList;
pub use punchi_fluid_core::title_block::TitleBlock;

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
    /// A4 landscape with the standard frame, the default the panel offers.
    ///
    /// The program opens horizontal (ISO 5457:1999/Amd 1:2010); portrait is one click away
    /// and every test that is about portrait geometry asks for it explicitly.
    #[must_use]
    pub fn a4(title_block: TitleBlock) -> Self {
        Self {
            paper: PaperSize::A4,
            orientation: Orientation::Landscape,
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
    ///
    /// The block follows the Chilean guide but is fitted to the paper and the content model:
    /// **A4/A5 industrial is 170 × 45 mm**, the school and technical models are shorter, and
    /// **A3+ industrial is 180 × 65 mm** (docs/reference/title-block.md). A hidden field
    /// collapses its cell and an empty row shortens the block, so the rectangle is the
    /// layout's own height. On a sheet that cannot hold it, both sides shrink to the content
    /// area. `NCh 14.Of93` / `ISO 7200` still bounds the identification zone at 170 mm.
    #[must_use]
    pub fn title_rect(&self) -> Rect {
        let content = self.content_rect();
        let position = self
            .title_block
            .as_ref()
            .map_or(BlockPosition::BottomRight, |b| b.position);
        if position == BlockPosition::None {
            return Rect::default();
        }
        let (block_width, block_height) = self.block_size_mm();
        let width = if position == BlockPosition::BottomFull {
            content.width()
        } else {
            content.width().clamp(0.0, block_width)
        };
        let height = content.height().clamp(0.0, block_height);
        let left = if position == BlockPosition::BottomFull {
            content.min.x
        } else {
            content.max.x - width
        };
        let top = if position == BlockPosition::TopRight {
            content.min.y
        } else {
            content.max.y - height
        };
        Rect::from_corners(Pt::new(left, top), Pt::new(left + width, top + height))
    }

    /// Size of the block on this paper, from its content model and visible fields.
    ///
    /// The guide's 180 × 65 mm proposal is the A3 industrial template; on A4 the owner
    /// asked for 170 × 45 mm, and the school and technical models are shorter. A hidden
    /// field or an empty row gives a shorter block, so this is the resolved layout's size.
    #[must_use]
    pub fn block_size_mm(&self) -> (f32, f32) {
        let block = self.title_block.clone().unwrap_or_default();
        let layout = crate::block_layout::layout(&block, self.paper);
        (layout.width, layout.height)
    }

    /// `true` when the sheet carries the optional reference zones.
    ///
    /// ISO 5457 makes the zones optional; the Chilean guide recommends them for large
    /// sheets and multi-sheet sets. This project draws them from A3 upwards and keeps A4
    /// clean, so a one-sheet sketch is not covered in a grid it does not need.
    #[must_use]
    pub fn has_reference_zones(&self) -> bool {
        is_large_sheet(self.paper)
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
        let title = self.title_rect();
        if title.is_empty() {
            return content;
        }
        let parts = self.parts_rect();
        let right = if parts.is_empty() {
            content.max.x
        } else {
            parts.min.x - 2.0
        };
        let position = self
            .title_block
            .as_ref()
            .map_or(BlockPosition::BottomRight, |block| block.position);
        if position == BlockPosition::TopRight {
            // The block occupies the top band; the single drawing rectangle starts below it.
            return Rect::from_corners(
                Pt::new(content.min.x, title.max.y),
                Pt::new(right, content.max.y),
            );
        }
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

/// `true` when `paper` is A3 or larger.
///
/// The threshold decides two things together: the guide's 180 × 65 mm block and the
/// optional reference zones. A custom sheet joins A3 as soon as its short side reaches
/// 297 mm.
#[must_use]
pub fn is_large_sheet(paper: PaperSize) -> bool {
    match paper {
        PaperSize::A3 | PaperSize::A2 | PaperSize::A1 | PaperSize::A0 => true,
        PaperSize::Custom {
            width_mm,
            height_mm,
        } => width_mm.min(height_mm) >= 297.0 - f32::EPSILON,
        PaperSize::A5
        | PaperSize::A4
        | PaperSize::Letter
        | PaperSize::Legal
        | PaperSize::Oficio => false,
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
    fn a4_landscape_at_300_dpi_is_3508_by_2480() {
        let page = Page::a4(TitleBlock::default());
        assert_eq!(page.orientation, Orientation::Landscape);
        assert_eq!(page.pixel_size(300.0), (3508, 2480));
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
        let landscape = Page::a4(TitleBlock::default());
        let portrait = Page {
            orientation: Orientation::Portrait,
            ..landscape.clone()
        };
        assert_eq!(portrait.size_mm(), (210.0, 297.0));
        assert_eq!(landscape.size_mm(), (297.0, 210.0));
    }

    #[test]
    fn the_content_rect_is_the_page_minus_the_margins() {
        let page = Page::a4(TitleBlock::default());
        let content = page.content_rect();
        // A4 landscape with the ISO 5457 frame: 20 mm on the filing edge, 10 mm elsewhere.
        assert!((content.min.x - 20.0).abs() < f32::EPSILON);
        assert!((content.min.y - 10.0).abs() < f32::EPSILON);
        assert!((content.width() - (297.0 - 30.0)).abs() < f32::EPSILON);
        assert!((content.height() - (210.0 - 20.0)).abs() < f32::EPSILON);
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
        // On A4 the fitted block is 170 × 45 mm: it leaves 10 mm to the left of the 180 mm
        // drawing space, as the owner asked.
        assert!((title.width() - 170.0).abs() < 0.01, "{}", title.width());
        assert!((title.height() - 45.0).abs() < 0.01, "{}", title.height());
        assert!((title.max.x - content.max.x).abs() < f32::EPSILON);
        assert!((title.max.y - content.max.y).abs() < f32::EPSILON);
    }

    #[test]
    fn the_block_can_sit_top_right_bottom_right_bottom_full_or_not_at_all() {
        let content = Page::a4(TitleBlock::default()).content_rect();
        let with = |position| {
            let mut page = Page::a4(TitleBlock::default());
            if let Some(block) = page.title_block.as_mut() {
                block.position = position;
            }
            page
        };
        let top = with(BlockPosition::TopRight).title_rect();
        assert!((top.width() - 170.0).abs() < 0.01);
        assert!((top.max.x - content.max.x).abs() < f32::EPSILON);
        assert!((top.min.y - content.min.y).abs() < f32::EPSILON);
        assert!(with(BlockPosition::TopRight).drawing_rect().min.y >= top.max.y);
        let bottom = with(BlockPosition::BottomRight).title_rect();
        assert!((bottom.width() - 170.0).abs() < 0.01);
        assert!((bottom.max.x - content.max.x).abs() < f32::EPSILON);
        assert!((bottom.max.y - content.max.y).abs() < f32::EPSILON);
        let full = with(BlockPosition::BottomFull).title_rect();
        assert!((full.width() - content.width()).abs() < 0.01);
        assert!((full.min.x - content.min.x).abs() < f32::EPSILON);
        assert!((full.max.y - content.max.y).abs() < f32::EPSILON);
        let none = with(BlockPosition::None);
        assert!(none.title_rect().is_empty());
        assert_eq!(none.drawing_rect(), none.content_rect());
        for position in [
            BlockPosition::TopRight,
            BlockPosition::BottomRight,
            BlockPosition::BottomFull,
        ] {
            let rect = with(position).title_rect();
            assert!((rect.height() - 45.0).abs() < 0.01, "{position:?}");
        }
    }

    #[test]
    fn every_position_is_geometrically_sound_in_both_orientations() {
        for orientation in [Orientation::Portrait, Orientation::Landscape] {
            for position in [
                BlockPosition::TopRight,
                BlockPosition::BottomRight,
                BlockPosition::BottomFull,
            ] {
                let block = TitleBlock {
                    position,
                    ..TitleBlock::default()
                };
                let page = Page {
                    orientation,
                    ..Page::a4(block)
                };
                let content = page.content_rect();
                let title = page.title_rect();
                assert!(content.contains(title.min) && content.contains(title.max));
                match position {
                    BlockPosition::TopRight => {
                        assert!((title.min.y - content.min.y).abs() < f32::EPSILON);
                        assert!((title.max.x - content.max.x).abs() < f32::EPSILON);
                    }
                    BlockPosition::BottomRight => {
                        assert!((title.max.y - content.max.y).abs() < f32::EPSILON);
                        assert!((title.max.x - content.max.x).abs() < f32::EPSILON);
                    }
                    BlockPosition::BottomFull => {
                        assert!((title.min.x - content.min.x).abs() < f32::EPSILON);
                        assert!((title.width() - content.width()).abs() < 0.01);
                        assert!((title.max.y - content.max.y).abs() < f32::EPSILON);
                    }
                    BlockPosition::None => {}
                }
            }
        }
    }

    #[test]
    fn a3_uses_the_guide_block_and_a4_the_fitted_one() {
        let block = TitleBlock::default();
        let page = |paper| Page {
            paper,
            ..Page::a4(block.clone())
        };
        let close = |paper, expected: (f32, f32)| {
            let size = page(paper).block_size_mm();
            assert!(
                (size.0 - expected.0).abs() < 0.01 && (size.1 - expected.1).abs() < 0.01,
                "{}: {size:?} is not {expected:?}",
                paper.label()
            );
        };
        close(PaperSize::A4, (170.0, 45.0));
        close(PaperSize::A5, (170.0, 45.0));
        close(PaperSize::Letter, (170.0, 45.0));
        close(PaperSize::A3, (180.0, 65.0));
        close(PaperSize::A2, (180.0, 65.0));
        close(PaperSize::A1, (180.0, 65.0));
        close(PaperSize::A0, (180.0, 65.0));
        close(
            PaperSize::custom(297.0, 420.0).expect("a custom A3"),
            (180.0, 65.0),
        );

        let a4 = Page::a4(block.clone());
        assert!((a4.title_rect().width() - 170.0).abs() < 0.01);
        assert!((a4.title_rect().height() - 45.0).abs() < 0.01);
        let a3 = Page {
            paper: PaperSize::A3,
            orientation: Orientation::Landscape,
            ..Page::a4(block)
        };
        assert!((a3.title_rect().width() - 180.0).abs() < 0.01);
        assert!((a3.title_rect().height() - 65.0).abs() < 0.01);
    }

    #[test]
    fn the_reference_zones_are_drawn_from_a3_upwards() {
        // The optional zones of ISO 5457: on for a large or multi-sheet set, off on A4 so
        // the one-sheet sketch stays clean.
        for (paper, expected) in [
            (PaperSize::A5, false),
            (PaperSize::A4, false),
            (PaperSize::Letter, false),
            (PaperSize::Legal, false),
            (PaperSize::Oficio, false),
            (PaperSize::A3, true),
            (PaperSize::A2, true),
            (PaperSize::A1, true),
            (PaperSize::A0, true),
            (PaperSize::custom(297.0, 420.0).expect("a custom A3"), true),
            (PaperSize::custom(210.0, 297.0).expect("a custom A4"), false),
        ] {
            let page = Page {
                paper,
                ..Page::a4(TitleBlock::default())
            };
            assert_eq!(
                page.has_reference_zones(),
                expected,
                "{}: reference zones",
                paper.label()
            );
        }
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
