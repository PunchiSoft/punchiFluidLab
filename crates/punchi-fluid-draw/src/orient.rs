// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Turning a placement: quarter turns and a mirror, applied as one transform.
//!
//! A symbol is written once, in its own axes, and the sheet may want it turned. Rather
//! than a second drawing per direction — which is how a catalogue grows a copy of every
//! valve — the whole symbol is drawn into a [`Turned`] surface that maps every coordinate
//! about the centre of its box. The drawing code does not know it was turned, so a new
//! direction is not a new code path, and the paper and the screen stay one drawing.
//!
//! Text is the one thing that does not turn: a numeral turned on its side is unreadable,
//! so the anchor point moves with the geometry and the glyphs stay upright.

use crate::surface::{Anchor, Colour, Pt, Rect, Stroke, Surface};

/// How a placement is turned on the sheet.
///
/// The mirror is applied about the symbol's own vertical axis first, then the quarter
/// turns clockwise, so the pair has one meaning and not four.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Orientation {
    quarter_turns: u8,
    mirrored: bool,
}

impl Orientation {
    /// The symbol as the file draws it.
    pub const NONE: Self = Self {
        quarter_turns: 0,
        mirrored: false,
    };

    /// An orientation from its two parts; the turns are reduced modulo four.
    #[must_use]
    pub const fn new(quarter_turns: u8, mirrored: bool) -> Self {
        Self {
            quarter_turns: quarter_turns % 4,
            mirrored,
        }
    }

    /// Clockwise quarter turns, `0..=3`.
    #[must_use]
    pub const fn quarter_turns(self) -> u8 {
        self.quarter_turns
    }

    /// `true` when the symbol is reflected about its own vertical axis.
    #[must_use]
    pub const fn mirrored(self) -> bool {
        self.mirrored
    }

    /// `true` when the symbol is drawn as its file describes it.
    #[must_use]
    pub const fn is_identity(self) -> bool {
        self.quarter_turns == 0 && !self.mirrored
    }

    /// A quarter turn clockwise: what `Edición ▸ Girar` does.
    #[must_use]
    pub const fn turned_clockwise(self) -> Self {
        Self::new(self.quarter_turns + 1, self.mirrored)
    }

    /// A quarter turn anticlockwise: what `Ctrl` and a double click does.
    #[must_use]
    pub const fn turned_anticlockwise(self) -> Self {
        Self::new(self.quarter_turns + 3, self.mirrored)
    }

    /// The mirror: what `Edición ▸ Reflejar` does.
    #[must_use]
    pub const fn reflected(self) -> Self {
        Self::new(self.quarter_turns, !self.mirrored)
    }

    /// `true` when the symbol stands on its side, so its box swaps width and height.
    #[must_use]
    pub const fn swaps_axes(self) -> bool {
        self.quarter_turns % 2 == 1
    }

    /// Maps a point of the unturned box to where it lands on the sheet.
    #[must_use]
    pub fn map(self, centre: Pt, point: Pt) -> Pt {
        let mut x = point.x - centre.x;
        let mut y = point.y - centre.y;
        if self.mirrored {
            x = -x;
        }
        for _ in 0..self.quarter_turns {
            // Clockwise on screen, where `y` grows downwards.
            let (turned_x, turned_y) = (-y, x);
            x = turned_x;
            y = turned_y;
        }
        Pt::new(centre.x + x, centre.y + y)
    }

    /// The box a placement occupies once it is turned: the same symbol, side on.
    #[must_use]
    pub fn box_of(self, rect: Rect) -> Rect {
        if self.swaps_axes() {
            Rect::from_center_size(rect.center(), rect.height(), rect.width())
        } else {
            rect
        }
    }

    /// Where an arc's ends land, in degrees clockwise from `+x` on screen.
    ///
    /// A mirror reverses the sweep, so the ends trade places; a quarter turn moves both.
    #[must_use]
    pub fn arc(self, start: f32, end: f32) -> (f32, f32) {
        let (start, end) = if self.mirrored {
            (180.0 - end, 180.0 - start)
        } else {
            (start, end)
        };
        let shift = 90.0 * f32::from(self.quarter_turns);
        (normalise(start + shift), normalise(end + shift))
    }

    /// The anchor a text run keeps while its glyphs stay upright.
    ///
    /// Under a mirror the text has moved to the other side of the box, so a run anchored
    /// at its start edge has to be anchored at its end edge to stay where it was drawn.
    #[must_use]
    pub fn anchor(self, anchor: Anchor) -> Anchor {
        match (self.mirrored, anchor) {
            (true, Anchor::Start) => Anchor::End,
            (true, Anchor::End) => Anchor::Start,
            _ => anchor,
        }
    }
}

/// An angle in `0..360` degrees.
fn normalise(degrees: f32) -> f32 {
    let wrapped = degrees % 360.0;
    if wrapped < 0.0 {
        wrapped + 360.0
    } else {
        wrapped
    }
}

/// A surface that draws a turned symbol into another one.
///
/// Everything but the text is mapped, including the rectangles, the circles and the arc
/// ends; the text only moves, because glyphs that lie on their side are not a drawing of
/// anything.
pub struct Turned<'a> {
    inner: &'a mut dyn Surface,
    orientation: Orientation,
    centre: Pt,
}

impl std::fmt::Debug for Turned<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Turned")
            .field("orientation", &self.orientation)
            .field("centre", &self.centre)
            .finish_non_exhaustive()
    }
}

impl<'a> Turned<'a> {
    /// Draws into `inner` as if the symbol at `centre` were turned.
    pub fn new(inner: &'a mut dyn Surface, orientation: Orientation, centre: Pt) -> Self {
        Self {
            inner,
            orientation,
            centre,
        }
    }

    fn map(&self, point: Pt) -> Pt {
        self.orientation.map(self.centre, point)
    }

    fn map_all(&self, points: &[Pt]) -> Vec<Pt> {
        points.iter().map(|point| self.map(*point)).collect()
    }
}

impl Surface for Turned<'_> {
    fn line(&mut self, points: &[Pt], stroke: Stroke) {
        let mapped = self.map_all(points);
        self.inner.line(&mapped, stroke);
    }

    fn polygon(&mut self, points: &[Pt], fill: Option<Colour>, stroke: Option<Stroke>) {
        let mapped = self.map_all(points);
        self.inner.polygon(&mapped, fill, stroke);
    }

    fn rect(&mut self, rect: Rect, corner: f32, fill: Option<Colour>, stroke: Option<Stroke>) {
        let corners = Rect::from_corners(self.map(rect.min), self.map(rect.max));
        let mapped = Rect::from_corners(
            Pt::new(
                corners.min.x.min(corners.max.x),
                corners.min.y.min(corners.max.y),
            ),
            Pt::new(
                corners.min.x.max(corners.max.x),
                corners.min.y.max(corners.max.y),
            ),
        );
        self.inner.rect(mapped, corner, fill, stroke);
    }

    fn circle(&mut self, centre: Pt, radius: f32, fill: Option<Colour>, stroke: Option<Stroke>) {
        self.inner.circle(self.map(centre), radius, fill, stroke);
    }

    fn arc(&mut self, centre: Pt, radius: f32, start: f32, end: f32, stroke: Stroke) {
        let (start, end) = self.orientation.arc(start, end);
        self.inner.arc(self.map(centre), radius, start, end, stroke);
    }

    fn text(&mut self, at: Pt, text: &str, size: f32, colour: Colour, anchor: Anchor) {
        let anchor = self.orientation.anchor(anchor);
        self.inner.text(self.map(at), text, size, colour, anchor);
    }

    fn device_scale(&self) -> f32 {
        self.inner.device_scale()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::{Call, Recording};

    fn centre() -> Pt {
        Pt::new(50.0, 20.0)
    }

    #[test]
    fn no_turn_leaves_a_point_where_it_was() {
        let point = Pt::new(55.0, 22.0);
        assert_eq!(Orientation::NONE.map(centre(), point), point);
    }

    #[test]
    fn a_quarter_turn_moves_the_right_edge_to_the_bottom() {
        // The right edge of the box, turned clockwise, ends up at the bottom.
        let right = Pt::new(60.0, 20.0);
        let landed = Orientation::new(1, false).map(centre(), right);
        assert!((landed.x - 50.0).abs() < 1e-4, "{landed:?}");
        assert!((landed.y - 30.0).abs() < 1e-4, "{landed:?}");
    }

    #[test]
    fn four_quarter_turns_are_the_symbol_as_it_was_drawn() {
        let mut orientation = Orientation::NONE;
        for _ in 0..4 {
            orientation = orientation.turned_clockwise();
        }
        assert!(orientation.is_identity());
    }

    #[test]
    fn anticlockwise_undoes_clockwise() {
        let orientation = Orientation::NONE.turned_clockwise().turned_anticlockwise();
        assert!(orientation.is_identity());
    }

    #[test]
    fn a_mirror_twice_is_the_symbol_as_it_was_drawn() {
        assert!(Orientation::NONE.reflected().reflected().is_identity());
    }

    #[test]
    fn a_mirror_moves_the_left_edge_to_the_right() {
        let left = Pt::new(40.0, 20.0);
        let landed = Orientation::NONE.reflected().map(centre(), left);
        assert!((landed.x - 60.0).abs() < 1e-4, "{landed:?}");
    }

    #[test]
    fn a_turned_box_swaps_its_sides() {
        let rect = Rect::from_center_size(centre(), 20.0, 10.0);
        let turned = Orientation::new(1, false).box_of(rect);
        assert!((turned.width() - 10.0).abs() < 1e-4);
        assert!((turned.height() - 20.0).abs() < 1e-4);
        assert_eq!(turned.center(), rect.center());
    }

    #[test]
    fn an_arc_turns_with_the_symbol_and_is_reversed_by_the_mirror() {
        assert_eq!(Orientation::new(1, false).arc(0.0, 90.0), (90.0, 180.0));
        assert_eq!(Orientation::NONE.reflected().arc(0.0, 90.0), (90.0, 180.0));
        assert_eq!(Orientation::new(3, false).arc(350.0, 10.0), (260.0, 280.0));
    }

    #[test]
    fn a_turned_surface_draws_the_same_calls_where_the_turn_puts_them() {
        let mut recording = Recording::default();
        {
            let mut turned = Turned::new(&mut recording, Orientation::new(1, false), centre());
            turned.line(
                &[Pt::new(50.0, 20.0), Pt::new(60.0, 20.0)],
                Stroke::new(1.0, Colour::BLACK),
            );
            turned.text(Pt::new(55.0, 20.0), "1", 3.0, Colour::BLACK, Anchor::Middle);
        }
        let calls = recording.calls();
        assert_eq!(calls.len(), 2, "{calls:?}");
        let lines: Vec<&[Pt]> = calls
            .iter()
            .filter_map(|call| match call {
                Call::Line { points, .. } => Some(points.as_slice()),
                _ => None,
            })
            .collect();
        assert_eq!(lines.len(), 1, "one line, found {calls:?}");
        assert!((lines[0][1].x - 50.0).abs() < 1e-4, "{:?}", lines[0]);
        assert!((lines[0][1].y - 30.0).abs() < 1e-4, "{:?}", lines[0]);
        let texts: Vec<(&str, Pt)> = calls
            .iter()
            .filter_map(|call| match call {
                Call::Text { at, text, .. } => Some((text.as_str(), *at)),
                _ => None,
            })
            .collect();
        assert_eq!(texts.len(), 1, "one text run, found {calls:?}");
        assert_eq!(texts[0].0, "1");
        assert!((texts[0].1.x - 50.0).abs() < 1e-4, "{:?}", texts[0]);
        assert!((texts[0].1.y - 25.0).abs() < 1e-4, "{:?}", texts[0]);
    }
}
