// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Paper, margins and the identification block.
//!
//! Sizes are the ISO 216 series plus the two American sheets the reference material
//! also uses, in millimetres. A drawing never decides its own paper: the page owns the
//! content rectangle and the drawing goes inside it.

use punchi_fluid_draw::{Pt, Rect};

/// Paper the sheet can be printed on.
///
/// The ISO 216 series with the ISO 5457 frame, the two American sheets, the Chilean
/// `Oficio` — which is neither of them — and a custom size for the sheet that is not on
/// the list. Every size is in millimetres, written once, so a drawing never depends on a
/// locale or on a printer driver.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaperSize {
    /// 148 × 210 mm.
    A5,
    /// 210 × 297 mm.
    A4,
    /// 297 × 420 mm.
    A3,
    /// 420 × 594 mm.
    A2,
    /// 594 × 841 mm.
    A1,
    /// 841 × 1189 mm.
    A0,
    /// 215.9 × 279.4 mm, "US Letter".
    Letter,
    /// 215.9 × 355.6 mm, "US Legal".
    Legal,
    /// 216 × 330 mm. The Chilean `Oficio`/`Folio`: longer than A4 and narrower than Legal,
    /// and the paper that is actually in the printer in Chile.
    Oficio,
    /// A sheet of the given millimetres, inside [`PaperSize::MIN_BOUND_MM`] and
    /// [`PaperSize::MAX_BOUND_MM`].
    Custom {
        /// Width in millimetres.
        width_mm: f32,
        /// Height in millimetres.
        height_mm: f32,
    },
}

impl PaperSize {
    /// Smallest sheet the export accepts, in millimetres on either side.
    pub const MIN_BOUND_MM: f32 = 50.0;

    /// Largest sheet the export accepts, in millimetres on either side. Beyond this the
    /// raster path would ask for a buffer no machine has, so it is refused before the
    /// allocation and not after it.
    pub const MAX_BOUND_MM: f32 = 2000.0;

    /// Every named size, in the order the panel lists them. A custom sheet is absent on
    /// purpose: it carries a measurement, not a name.
    pub const ALL: [Self; 9] = [
        Self::A5,
        Self::A4,
        Self::A3,
        Self::A2,
        Self::A1,
        Self::A0,
        Self::Letter,
        Self::Legal,
        Self::Oficio,
    ];

    /// A sheet of the given millimetres, or `None` when a side is outside the bounds.
    ///
    /// A size that cannot be printed is refused here, where the caller still has somewhere
    /// to put the refusal, and not later inside a buffer allocation.
    #[must_use]
    pub fn custom(width_mm: f32, height_mm: f32) -> Option<Self> {
        let inside = |value: f32| {
            value.is_finite() && (Self::MIN_BOUND_MM..=Self::MAX_BOUND_MM).contains(&value)
        };
        (inside(width_mm) && inside(height_mm)).then_some(Self::Custom {
            width_mm,
            height_mm,
        })
    }

    /// `true` for a size that can be printed: a named one, or a custom one inside the bounds.
    #[must_use]
    pub fn is_valid(self) -> bool {
        let (width, height) = self.size_mm();
        let inside = |value: f32| {
            value.is_finite() && (Self::MIN_BOUND_MM..=Self::MAX_BOUND_MM).contains(&value)
        };
        inside(width) && inside(height)
    }

    /// Name shown in the interface and written in the title block.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::A5 => "A5",
            Self::A4 => "A4",
            Self::A3 => "A3",
            Self::A2 => "A2",
            Self::A1 => "A1",
            Self::A0 => "A0",
            Self::Letter => "Letter",
            Self::Legal => "Legal",
            // The interface names it in Spanish, because that is the name on the ream.
            Self::Oficio => "Oficio",
            Self::Custom { .. } => "Personalizado",
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
            Self::A1 => (594.0, 841.0),
            Self::A0 => (841.0, 1189.0),
            Self::Letter => (215.9, 279.4),
            Self::Legal => (215.9, 355.6),
            Self::Oficio => (216.0, 330.0),
            Self::Custom {
                width_mm,
                height_mm,
            } => (width_mm, height_mm),
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
    fn every_named_size_is_the_paper_it_says_it_is() {
        // The measurements, written once: an ISO sheet whose short side halves from the one
        // before it, and the three sheets that are not ISO.
        for (paper, width, height) in [
            (PaperSize::A5, 148.0, 210.0),
            (PaperSize::A4, 210.0, 297.0),
            (PaperSize::A3, 297.0, 420.0),
            (PaperSize::A2, 420.0, 594.0),
            (PaperSize::A1, 594.0, 841.0),
            (PaperSize::A0, 841.0, 1189.0),
            (PaperSize::Letter, 215.9, 279.4),
            (PaperSize::Legal, 215.9, 355.6),
            (PaperSize::Oficio, 216.0, 330.0),
        ] {
            assert_eq!(
                paper.size_mm(),
                (width, height),
                "{} is {width} x {height} mm",
                paper.label()
            );
            assert!(paper.is_valid(), "{} must be printable", paper.label());
        }
        assert_eq!(PaperSize::ALL.len(), 9, "every named size is listed");
        // Oficio is the sheet that makes this list necessary: longer than A4, shorter than
        // Legal, and 0.1 mm wider than Legal — near enough to be confused with it and not the
        // same paper, which is exactly why it is a size of its own and not a rounding.
        let (oficio_w, oficio_h) = PaperSize::Oficio.size_mm();
        assert!(oficio_h > PaperSize::A4.size_mm().1);
        assert!(oficio_h < PaperSize::Legal.size_mm().1);
        assert!((oficio_w - PaperSize::Legal.size_mm().0).abs() <= 0.2);
    }

    #[test]
    fn a_sheet_outside_the_bounds_is_refused_before_it_is_drawn() {
        assert!(PaperSize::custom(216.0, 330.0).is_some());
        assert!(PaperSize::custom(PaperSize::MIN_BOUND_MM, 100.0).is_some());
        // Too small to hold a frame, too large to hold in a buffer, and a size that is not
        // a measurement at all.
        assert!(PaperSize::custom(10.0, 330.0).is_none());
        assert!(PaperSize::custom(216.0, 5000.0).is_none());
        assert!(PaperSize::custom(f32::NAN, 330.0).is_none());
        assert!(PaperSize::custom(f32::INFINITY, 330.0).is_none());
        let far = PaperSize::Custom {
            width_mm: 1.0,
            height_mm: 330.0,
        };
        assert!(!far.is_valid(), "a size built by hand is still checked");
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
