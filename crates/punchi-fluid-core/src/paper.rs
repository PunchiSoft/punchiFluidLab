// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Paper, its orientation and the frame the drawing is printed inside.
//!
//! The sizes are the ISO 216 series plus the two American sheets and the Chilean
//! `Oficio`, written once in millimetres so no drawing, export or check depends on a
//! locale or on a printer driver. The frame follows ISO 5457: the drawing space sits
//! 20 mm from the filing edge and 10 mm from the other three.
//!
//! This lives in the domain because a sheet is a physical measurement: the document
//! stores it, the check measures against it and the window draws it, always the same
//! millimetres. Every value is total: an unusable paper is refused where the caller still
//! has somewhere to put the refusal, never clamped into a plausible-looking sheet.

/// Paper the sheet can be printed on.
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
    /// Smallest sheet accepted, in millimetres on either side.
    pub const MIN_BOUND_MM: f32 = 50.0;

    /// Largest sheet accepted, in millimetres on either side. Beyond this a raster path
    /// would ask for a buffer no machine has, so it is refused before the allocation and
    /// not after it.
    pub const MAX_BOUND_MM: f32 = 2000.0;

    /// Every named size, in the order the interface lists them. A custom sheet is absent
    /// on purpose: it carries a measurement, not a name.
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

    /// The named size whose label is `label`, or `None` for an unknown one.
    ///
    /// The custom size is not a name: a caller that has measurements builds it with
    /// [`PaperSize::custom`] so the bounds are checked in one place.
    #[must_use]
    pub fn from_label(label: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|paper| paper.label() == label)
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

    /// The index of this named size in [`PaperSize::ALL`], or `None` for a custom sheet.
    #[must_use]
    pub fn index(self) -> Option<usize> {
        Self::ALL.iter().position(|paper| *paper == self)
    }
}

/// Which way the paper is turned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Orientation {
    /// Taller than wide.
    Portrait,
    /// Wider than tall; the program opens landscape on A4 and portrait stays one click
    /// away in the ribbon (`A4 normal`).
    #[default]
    Landscape,
}

impl Default for PaperSize {
    /// A4, the sheet the project starts on.
    fn default() -> Self {
        Self::A4
    }
}

impl Orientation {
    /// `true` for the orientation that swaps the sides.
    #[must_use]
    pub const fn is_landscape(self) -> bool {
        matches!(self, Self::Landscape)
    }
}

/// Margin around the drawing space, in millimetres.
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
    /// The ISO 5457 frame: 20 mm on the filing edge and 10 mm on the other three.
    #[must_use]
    pub const fn iso5457() -> Self {
        Self {
            top: 10.0,
            right: 10.0,
            bottom: 10.0,
            left: 20.0,
        }
    }

    /// No frame at all, which is what the first sheets meant before the paper existed.
    #[must_use]
    pub const fn unframed() -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }

    /// The frame a printed technical drawing uses. Kept as the name callers already know.
    #[must_use]
    pub const fn standard() -> Self {
        Self::iso5457()
    }

    /// `true` when every margin is finite and not negative.
    #[must_use]
    pub fn is_valid(self) -> bool {
        [self.top, self.right, self.bottom, self.left]
            .iter()
            .all(|value| value.is_finite() && *value >= 0.0)
    }

    /// `true` when the margins leave a positive drawing space on `paper` turned `orientation`.
    #[must_use]
    pub fn fits(self, paper: PaperSize, orientation: Orientation) -> bool {
        if !self.is_valid() || !paper.is_valid() {
            return false;
        }
        let sheet = Sheet {
            paper,
            orientation,
            margins: self,
        };
        let (width, height) = sheet.size_mm();
        width - self.left - self.right > 0.0 && height - self.top - self.bottom > 0.0
    }
}

impl Default for Margins {
    fn default() -> Self {
        Self::iso5457()
    }
}

/// The sheet a document is drawn on: paper, orientation and frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Sheet {
    /// Paper.
    pub paper: PaperSize,
    /// Which way it is turned.
    pub orientation: Orientation,
    /// Frame around the drawing space.
    pub margins: Margins,
}

impl Sheet {
    /// A4 landscape with the ISO 5457 frame, the sheet a new document starts on.
    ///
    /// ISO 5457:1999/Amd 1:2010 allows both A4 orientations; the program opens horizontal
    /// and `A4 normal` in the ribbon returns to portrait.
    #[must_use]
    pub const fn a4() -> Self {
        Self {
            paper: PaperSize::A4,
            orientation: Orientation::Landscape,
            margins: Margins::iso5457(),
        }
    }

    /// A4 portrait without a frame, which is how a sheet saved before the paper existed
    /// reads: its drawing area was a bare 200 × 130 mm rectangle, so nothing it holds may
    /// become "outside the frame" by opening it in a newer build.
    #[must_use]
    pub const fn legacy() -> Self {
        Self {
            paper: PaperSize::A4,
            orientation: Orientation::Portrait,
            margins: Margins::unframed(),
        }
    }

    /// Width and height of the paper in millimetres, honouring the orientation.
    #[must_use]
    pub fn size_mm(&self) -> (f32, f32) {
        let (width, height) = self.paper.size_mm();
        if self.orientation.is_landscape() {
            (height, width)
        } else {
            (width, height)
        }
    }

    /// Width and height of the drawing space, in millimetres, or `(0, 0)` when the margins
    /// leave none.
    #[must_use]
    pub fn content_size_mm(&self) -> (f32, f32) {
        let (width, height) = self.size_mm();
        let inner_width = width - self.margins.left - self.margins.right;
        let inner_height = height - self.margins.top - self.margins.bottom;
        if inner_width <= 0.0 || inner_height <= 0.0 {
            (0.0, 0.0)
        } else {
            (inner_width, inner_height)
        }
    }

    /// `true` when the paper is printable and the frame leaves a drawing space.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.margins.fits(self.paper, self.orientation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            assert_eq!(PaperSize::from_label(paper.label()), Some(paper));
        }
        assert_eq!(PaperSize::ALL.len(), 9, "every named size is listed");
    }

    #[test]
    fn a_sheet_outside_the_bounds_is_refused_before_it_is_drawn() {
        assert!(PaperSize::custom(216.0, 330.0).is_some());
        assert!(PaperSize::custom(PaperSize::MIN_BOUND_MM, 100.0).is_some());
        assert!(PaperSize::custom(10.0, 330.0).is_none());
        assert!(PaperSize::custom(216.0, 5000.0).is_none());
        assert!(PaperSize::custom(f32::NAN, 330.0).is_none());
        assert!(PaperSize::custom(f32::INFINITY, 330.0).is_none());
        assert_eq!(PaperSize::from_label("A4"), Some(PaperSize::A4));
        assert_eq!(PaperSize::from_label("Personalizado"), None);
        assert_eq!(PaperSize::from_label("a4"), None);
        assert_eq!(PaperSize::from_label(""), None);
    }

    #[test]
    fn a_new_sheet_is_a4_landscape_and_a_legacy_one_is_portrait_and_unframed() {
        let sheet = Sheet::a4();
        assert_eq!(sheet.paper, PaperSize::A4);
        assert_eq!(sheet.orientation, Orientation::Landscape);
        assert_eq!(sheet.size_mm(), (297.0, 210.0));
        assert_eq!(sheet.margins, Margins::iso5457());
        assert!(sheet.is_valid());

        let legacy = Sheet::legacy();
        assert_eq!(legacy.paper, PaperSize::A4);
        assert_eq!(legacy.orientation, Orientation::Portrait);
        assert_eq!(legacy.size_mm(), (210.0, 297.0));
        assert_eq!(legacy.margins, Margins::unframed());
        assert!(legacy.is_valid());
    }

    #[test]
    fn landscape_swaps_the_sides_and_the_content_follows() {
        let landscape = Sheet::a4();
        let portrait = Sheet {
            orientation: Orientation::Portrait,
            ..landscape
        };
        assert_eq!(landscape.size_mm(), (297.0, 210.0));
        assert_eq!(portrait.size_mm(), (210.0, 297.0));
        let (width, height) = landscape.content_size_mm();
        assert!((width - (297.0 - 30.0)).abs() < f32::EPSILON);
        assert!((height - (210.0 - 20.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn the_iso_frame_is_twenty_on_the_filing_edge_and_ten_elsewhere() {
        let margins = Margins::iso5457();
        let close = |value: f32, expected: f32| (value - expected).abs() < f32::EPSILON;
        assert!(close(margins.left, 20.0));
        assert!(close(margins.top, 10.0));
        assert!(close(margins.right, 10.0));
        assert!(close(margins.bottom, 10.0));
        assert!(margins.fits(PaperSize::A5, Orientation::Portrait));
        // Margins larger than the paper leave no drawing space and are refused.
        let huge = Margins {
            left: 150.0,
            right: 150.0,
            ..margins
        };
        assert!(!huge.fits(PaperSize::A5, Orientation::Portrait));
        assert!(!huge.fits(PaperSize::A4, Orientation::Portrait));
        assert!(Margins::iso5457().fits(PaperSize::A4, Orientation::Portrait));
    }
}
