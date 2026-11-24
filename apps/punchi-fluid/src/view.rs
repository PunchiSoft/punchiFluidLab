// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The view: the zoom and the pan of the sheet, outside the document.
//!
//! The document has no zoom and no pan; the reader's view does. The **frame inside the
//! page's margins** fits the canvas at zoom 1, and above that the sheet is magnified about the
//! pointer or the centre and dragged with the middle button, the wheel or the arrow keys. The
//! pan is bounded to the frame's overhang and cleared whenever the view returns to the fit.
//! The fit target is the frame, not the whole paper: the margins are part of the sheet, not
//! of the drawing, so at the fit they may lie beyond the canvas and are clipped there.

use eframe::egui;

/// Where the page sits on screen: the scale in pixels per millimetre and its top-left corner.
#[derive(Debug, Clone, Copy)]
pub struct Placement {
    pub scale: f32,
    pub origin: egui::Pos2,
}

/// The rectangle the view fits: the frame inside the page's margins, in sheet millimetres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fit {
    /// The frame's top-left corner in sheet millimetres.
    pub min: egui::Vec2,
    /// The frame's width and height in sheet millimetres.
    pub size: egui::Vec2,
}

/// The reader's view of the sheet.
#[derive(Debug, Clone)]
pub struct View {
    zoom: f32,
    pan: egui::Vec2,
    /// The view before the last discrete change: one step back, never a stack.
    previous: Option<(f32, egui::Vec2)>,
}

impl Default for View {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            previous: None,
        }
    }
}

impl View {
    /// The same range the Qt session enforces: below a tenth of the fit the sheet is a dot,
    /// above six times it is one port filling the window.
    pub const MIN_ZOOM: f32 = 0.4;
    pub const MAX_ZOOM: f32 = 6.0;
    /// The step of the ribbon's buttons and of a Ctrl+wheel notch.
    pub const STEP: f32 = 1.25;
    /// Pixels one arrow key pans.
    const ARROW: f32 = 40.0;

    #[must_use]
    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    /// The fit, with the pan dropped, and **not** recorded: the internals of `fit`.
    pub fn reset(&mut self) {
        self.zoom = 1.0;
        self.pan = egui::Vec2::ZERO;
    }

    /// The fit, keeping one step of history so **Vista anterior** can return.
    pub(crate) fn fit(&mut self) {
        if (self.zoom - 1.0).abs() > f32::EPSILON || self.pan.length() > f32::EPSILON {
            self.previous = Some((self.zoom, self.pan));
        }
        self.reset();
    }

    /// Whether **Vista anterior** has a view to return to.
    #[must_use]
    pub fn can_go_back(&self) -> bool {
        self.previous.is_some()
    }

    /// Returns to the view before the last fit, zoom or zoom window, once.
    pub(crate) fn back(&mut self) {
        if let Some((zoom, pan)) = self.previous.take() {
            self.zoom = zoom.clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
            self.pan = pan;
        }
    }

    pub fn zoom_in(&mut self) {
        self.set_zoom(self.zoom * Self::STEP);
    }

    pub fn zoom_out(&mut self) {
        self.set_zoom(self.zoom / Self::STEP);
    }

    fn set_zoom(&mut self, zoom: f32) {
        let zoom = zoom.clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
        if (zoom - self.zoom).abs() < f32::EPSILON {
            return;
        }
        self.previous = Some((self.zoom, self.pan));
        self.zoom = zoom;
        if (zoom - 1.0).abs() < f32::EPSILON {
            self.pan = egui::Vec2::ZERO;
        }
    }

    /// Magnifies about a point of the canvas: the sheet point under it stays under it.
    pub fn zoom_about(&mut self, factor: f32, pointer: egui::Pos2, area: egui::Rect, fit: Fit) {
        let target = (self.zoom * factor).clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
        if (target - self.zoom).abs() < f32::EPSILON {
            return;
        }
        let before = self.placement(area, fit);
        let sheet = (pointer - before.origin) / before.scale;
        self.zoom = target;
        let scale = fit_scale(area, fit.size) * target;
        let centred = area.center() - fit.size * scale * 0.5 - fit.min * scale;
        self.pan = pointer - sheet * scale - centred;
        if (target - 1.0).abs() < f32::EPSILON {
            self.pan = egui::Vec2::ZERO;
        }
    }

    pub fn pan_by(&mut self, delta: egui::Vec2) {
        self.pan += delta;
    }

    pub fn pan_arrow(&mut self, x: f32, y: f32) {
        self.pan += egui::vec2(x * Self::ARROW, y * Self::ARROW);
    }

    /// The sheet's view gestures: `Ctrl` and the wheel zoom about the pointer, the wheel and
    /// the middle button pan, the middle double click and `Ctrl+0` fit the frame, and the
    /// arrows pan from the keyboard.
    pub(crate) fn handle_input(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        area: egui::Rect,
        fit: Fit,
    ) {
        let (scroll, ctrl, pointer) = ui.input(|input| {
            (
                input.raw_scroll_delta,
                input.modifiers.ctrl,
                input.pointer.hover_pos(),
            )
        });
        if response.hovered() && scroll != egui::Vec2::ZERO {
            if ctrl {
                if let Some(pointer) = pointer {
                    self.zoom_about((scroll.y * 0.005).exp(), pointer, area, fit);
                }
            } else {
                self.pan_by(scroll);
            }
        }
        if response.dragged_by(egui::PointerButton::Middle) {
            let delta = ui.input(|input| input.pointer.delta());
            self.pan_by(delta);
        }
        // The middle single press pans; its double click is the reader's fit, the gesture the
        // reference teaches for a schematic.
        if response.double_clicked_by(egui::PointerButton::Middle) {
            self.reset();
        }
        let (left, right, up, down, reset) = ui.input(|input| {
            (
                input.key_pressed(egui::Key::ArrowLeft),
                input.key_pressed(egui::Key::ArrowRight),
                input.key_pressed(egui::Key::ArrowUp),
                input.key_pressed(egui::Key::ArrowDown),
                input.modifiers.ctrl && input.key_pressed(egui::Key::Num0),
            )
        });
        if reset {
            self.fit();
        }
        let mut x = 0.0;
        let mut y = 0.0;
        if left {
            x += 1.0;
        }
        if right {
            x -= 1.0;
        }
        if up {
            y += 1.0;
        }
        if down {
            y -= 1.0;
        }
        if x != 0.0 || y != 0.0 {
            self.pan_arrow(x, y);
        }
    }

    /// The placement of the page for this frame; the pan is bounded to the frame's overhang
    /// and dropped once the frame fits.
    #[must_use]
    pub fn placement(&mut self, area: egui::Rect, fit: Fit) -> Placement {
        let scale = fit_scale(area, fit.size) * self.zoom;
        let frame_px = fit.size * scale;
        let overhang = egui::vec2(
            ((frame_px.x - area.width()) * 0.5).max(0.0),
            ((frame_px.y - area.height()) * 0.5).max(0.0),
        );
        self.pan = egui::vec2(
            self.pan.x.clamp(-overhang.x, overhang.x),
            self.pan.y.clamp(-overhang.y, overhang.y),
        );
        Placement {
            scale,
            origin: area.center() - frame_px * 0.5 - fit.min * scale + self.pan,
        }
    }
}

fn fit_scale(area: egui::Rect, frame_mm: egui::Vec2) -> f32 {
    if frame_mm.x <= 0.0 || frame_mm.y <= 0.0 {
        return 0.0;
    }
    (area.width() / frame_mm.x).min(area.height() / frame_mm.y)
}

impl crate::Window {
    /// The sheet's gestures: Ctrl+wheel zooms about the pointer, the wheel and the middle
    /// button pan, the arrows pan from the keyboard and Ctrl+0 returns to the fit.
    pub(crate) fn handle_view(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        area: egui::Rect,
        fit: Fit,
    ) {
        // The armed move pins the view: the pointer carries the symbol, so the sheet must not
        // slide under it with a wheel or an arrow.
        if self.moving.is_some() {
            return;
        }
        // The arrow keys belong to the open menu while it is up; panning the sheet under it
        // would both move the anchor and steal the key.
        if self.menu.is_open() {
            return;
        }
        self.view.handle_input(ui, response, area, fit);
    }
}

#[cfg(test)]
mod tests {
    use eframe::egui::{Rect, pos2, vec2};

    use super::{Fit, View};

    fn area() -> Rect {
        Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0))
    }

    /// The ISO 5457 frame of an A4 landscape page: 20 mm on the filing edge and 10 on the rest.
    fn fit() -> Fit {
        Fit {
            min: vec2(20.0, 10.0),
            size: vec2(267.0, 190.0),
        }
    }

    fn frame_rect(placement: &super::Placement, fit: Fit) -> Rect {
        Rect::from_min_size(
            placement.origin + fit.min * placement.scale,
            fit.size * placement.scale,
        )
    }

    #[test]
    fn the_fit_frames_the_drawing_inside_the_margins() {
        let mut view = View::default();
        let placement = view.placement(area(), fit());
        let expected = (800.0_f32 / 267.0).min(600.0 / 190.0);
        assert!((placement.scale - expected).abs() < 1e-4);
        let frame = frame_rect(&placement, fit());
        assert!(
            (frame.center() - area().center()).length() < 0.01,
            "{frame:?}"
        );
        assert!(frame.width() <= area().width() + 0.01);
        assert!(frame.height() <= area().height() + 0.01);
    }

    #[test]
    fn the_zoom_about_the_pointer_keeps_its_sheet_point() {
        let mut view = View::default();
        let pointer = pos2(300.0, 250.0);
        let before = view.placement(area(), fit());
        let sheet = (pointer - before.origin) / before.scale;
        view.zoom_about(2.0, pointer, area(), fit());
        let after = view.placement(area(), fit());
        let under = (pointer - after.origin) / after.scale;
        assert!((under - sheet).length() < 0.01, "{under:?} != {sheet:?}");
    }

    #[test]
    fn the_pan_is_bounded_to_the_frame() {
        let mut view = View::default();
        view.zoom_in();
        view.zoom_in();
        view.pan_by(vec2(10_000.0, 10_000.0));
        let placement = view.placement(area(), fit());
        let frame_px = fit().size * placement.scale;
        let top_left = area().center() - frame_px * 0.5 - fit().min * placement.scale;
        let overhang = (frame_px - area().size()) * 0.5;
        let expected = top_left + overhang;
        assert!(
            (placement.origin - expected).length() < 0.01,
            "{:?} != {expected:?}",
            placement.origin
        );
    }

    #[test]
    fn a_reset_returns_to_the_fit() {
        let mut view = View::default();
        view.zoom_in();
        view.pan_by(vec2(120.0, 30.0));
        view.reset();
        assert!((view.zoom() - 1.0).abs() < f32::EPSILON);
        let placement = view.placement(area(), fit());
        assert!((frame_rect(&placement, fit()).center() - area().center()).length() < 0.01);
    }

    #[test]
    fn the_view_commands_stay_inside_the_zoom_range() {
        let mut view = View::default();
        view.zoom_in();
        assert!((view.zoom() - View::STEP).abs() < 1e-5);
        for _ in 0..40 {
            view.zoom_in();
        }
        assert!((view.zoom() - View::MAX_ZOOM).abs() < f32::EPSILON);
        for _ in 0..40 {
            view.zoom_out();
        }
        assert!((view.zoom() - View::MIN_ZOOM).abs() < f32::EPSILON);
        view.reset();
        assert!((view.zoom() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn one_step_of_view_history_goes_back() {
        let mut view = View::default();
        assert!(!view.can_go_back());
        view.zoom_in();
        assert!(view.can_go_back());
        view.back();
        assert!((view.zoom() - 1.0).abs() < f32::EPSILON);
        assert!(!view.can_go_back());
    }

    #[test]
    fn the_fit_records_the_view_it_leaves() {
        let mut view = View::default();
        view.zoom_in();
        view.pan_by(vec2(100.0, 40.0));
        view.fit();
        assert!((view.zoom() - 1.0).abs() < f32::EPSILON);
        view.back();
        assert!((view.zoom() - View::STEP).abs() < 1e-5);
        assert!(view.pan.length() > 0.0, "the pan came back with the view");
    }

    #[test]
    fn back_without_history_does_nothing() {
        let mut view = View::default();
        view.back();
        assert!((view.zoom() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn the_frame_fits_left_of_the_strip_gutter() {
        let mut view = View::default();
        let canvas = area();
        let drawing = crate::layout::drawing_area(canvas, 16.0);
        let placement = view.placement(drawing, fit());
        let frame = frame_rect(&placement, fit());
        let eps = 0.01;
        assert!(
            frame.min.x >= drawing.min.x - eps,
            "{frame:?} outside {drawing:?}"
        );
        assert!(
            frame.min.y >= drawing.min.y - eps,
            "{frame:?} outside {drawing:?}"
        );
        assert!(
            frame.max.x <= drawing.max.x + eps,
            "{frame:?} outside {drawing:?}"
        );
        assert!(
            frame.max.y <= drawing.max.y + eps,
            "{frame:?} outside {drawing:?}"
        );
        assert!(
            frame.max.x <= canvas.max.x - crate::layout::view_strip_gutter(16.0) + eps,
            "the frame enters the view strip's gutter"
        );
    }
}
