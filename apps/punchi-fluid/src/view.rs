// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The view: the zoom and the pan of the sheet, outside the document.
//!
//! The document has no zoom and no pan; the reader's view does. The sheet fits the canvas
//! at zoom 1, and above that the sheet is magnified about the pointer or the centre and
//! dragged with the middle button, the wheel or the arrow keys. The pan is bounded to the
//! sheet's overhang and cleared whenever the view returns to the fit, the way the Qt window
//! behaves and its `--selftest` checks.

use eframe::egui;

/// Where the page sits on screen: the scale in pixels per millimetre and its top-left corner.
#[derive(Debug, Clone, Copy)]
pub struct Placement {
    pub scale: f32,
    pub origin: egui::Pos2,
}

/// The reader's view of the sheet.
#[derive(Debug, Clone)]
pub struct View {
    zoom: f32,
    pan: egui::Vec2,
}

impl Default for View {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
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

    /// The fit, with the pan dropped.
    pub fn reset(&mut self) {
        self.zoom = 1.0;
        self.pan = egui::Vec2::ZERO;
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
        self.zoom = zoom;
        if (zoom - 1.0).abs() < f32::EPSILON {
            self.pan = egui::Vec2::ZERO;
        }
    }

    /// Magnifies about a point of the canvas: the sheet point under it stays under it.
    pub fn zoom_about(
        &mut self,
        factor: f32,
        pointer: egui::Pos2,
        area: egui::Rect,
        page_mm: egui::Vec2,
    ) {
        let target = (self.zoom * factor).clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
        if (target - self.zoom).abs() < f32::EPSILON {
            return;
        }
        let before = self.placement(area, page_mm);
        let sheet = (pointer - before.origin) / before.scale;
        self.zoom = target;
        let scale = fit(area, page_mm) * target;
        let centred = area.center() - page_mm * scale * 0.5;
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
    /// the middle button pan, the arrows pan from the keyboard and `Ctrl+0` returns to 1:1.
    pub(crate) fn handle_input(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        area: egui::Rect,
        page_mm: egui::Vec2,
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
                    self.zoom_about((scroll.y * 0.005).exp(), pointer, area, page_mm);
                }
            } else {
                self.pan_by(scroll);
            }
        }
        if response.dragged_by(egui::PointerButton::Middle) {
            let delta = ui.input(|input| input.pointer.delta());
            self.pan_by(delta);
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
            self.reset();
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

    /// The placement of the page for this frame; the pan is bounded to the overhang and
    /// dropped once the page fits.
    #[must_use]
    pub fn placement(&mut self, area: egui::Rect, page_mm: egui::Vec2) -> Placement {
        let scale = fit(area, page_mm) * self.zoom;
        let page_px = page_mm * scale;
        let overhang = egui::vec2(
            ((page_px.x - area.width()) * 0.5).max(0.0),
            ((page_px.y - area.height()) * 0.5).max(0.0),
        );
        self.pan = egui::vec2(
            self.pan.x.clamp(-overhang.x, overhang.x),
            self.pan.y.clamp(-overhang.y, overhang.y),
        );
        Placement {
            scale,
            origin: area.center() - page_px * 0.5 + self.pan,
        }
    }
}

fn fit(area: egui::Rect, page_mm: egui::Vec2) -> f32 {
    (area.width() / page_mm.x).min(area.height() / page_mm.y)
}

#[cfg(test)]
mod tests {
    use eframe::egui::{Rect, pos2, vec2};

    use super::View;

    fn area() -> Rect {
        Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0))
    }

    fn page() -> eframe::egui::Vec2 {
        vec2(297.0, 210.0)
    }

    #[test]
    fn the_fit_centres_the_page() {
        let mut view = View::default();
        let placement = view.placement(area(), page());
        let expected = (800.0_f32 / 297.0).min(600.0 / 210.0);
        assert!((placement.scale - expected).abs() < 1e-4);
        let centre = placement.origin + page() * placement.scale * 0.5;
        assert!((centre - area().center()).length() < 0.01, "{centre:?}");
    }

    #[test]
    fn the_zoom_about_the_pointer_keeps_its_sheet_point() {
        let mut view = View::default();
        let pointer = pos2(300.0, 250.0);
        let before = view.placement(area(), page());
        let sheet = (pointer - before.origin) / before.scale;
        view.zoom_about(2.0, pointer, area(), page());
        let after = view.placement(area(), page());
        let under = (pointer - after.origin) / after.scale;
        assert!((under - sheet).length() < 0.01, "{under:?} != {sheet:?}");
    }

    #[test]
    fn the_pan_is_bounded_to_the_sheet() {
        let mut view = View::default();
        view.zoom_in();
        view.zoom_in();
        view.pan_by(vec2(10_000.0, 10_000.0));
        let placement = view.placement(area(), page());
        let page_px = page() * placement.scale;
        let top_left = area().center() - page_px * 0.5;
        let overhang = (page_px - area().size()) * 0.5;
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
        let placement = view.placement(area(), page());
        let centre = placement.origin + page() * placement.scale * 0.5;
        assert!((centre - area().center()).length() < 0.01);
    }

    #[test]
    fn the_ver_commands_stay_inside_the_zoom_range() {
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
}
