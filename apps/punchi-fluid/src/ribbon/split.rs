// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The split command: the body runs one command and the arrow runs its variant. Both zones
//! are separate widgets with their own accessible names, hints and focus rings.

use eframe::egui;

use super::{
    Command, LARGE_HEIGHT, LARGE_ICON, LARGE_ICON_TOP, LARGE_LABEL_SIZE, LARGE_LABEL_TOP,
    LARGE_WIDTH, Palette, Ribbon, hint,
};

/// The control's width: the large body plus the arrow.
pub(super) const ARROW_WIDTH: f32 = 18.0;
pub(super) const WIDTH: f32 = LARGE_WIDTH + ARROW_WIDTH;

/// Draws one split command and puts the zone the reader activated in `picked`.
#[allow(
    clippy::too_many_arguments,
    reason = "the two commands, the two labels and the palette are the control's own data"
)]
pub(super) fn show(
    ribbon: &Ribbon,
    ui: &mut egui::Ui,
    id: &str,
    label: &'static str,
    arrow_label: &'static str,
    command: Command,
    arrow: Command,
    palette: &Palette,
    picked: &mut Option<Command>,
) {
    let enabled = ribbon.enabled(command);
    let arrow_enabled = ribbon.enabled(arrow);
    let text = ribbon.language.text(label);
    let arrow_text = ribbon.language.text(arrow_label);
    let ink = if enabled { palette.ink } else { palette.muted };
    let arrow_ink = if arrow_enabled {
        palette.ink
    } else {
        palette.muted
    };
    let (rect, _) = ui.allocate_exact_size(egui::vec2(WIDTH, LARGE_HEIGHT), egui::Sense::hover());
    let split_at = rect.right() - ARROW_WIDTH;
    let body = egui::Rect::from_min_max(rect.min, egui::pos2(split_at, rect.max.y));
    let arrow_rect = egui::Rect::from_min_max(egui::pos2(split_at, rect.min.y), rect.max);
    let body_id = ui.make_persistent_id(("ribbon.split", id, label));
    let arrow_id = ui.make_persistent_id(("ribbon.split.arrow", id, arrow_label));
    let body_response = ui.interact(body, body_id, egui::Sense::click());
    let arrow_response = ui.interact(arrow_rect, arrow_id, egui::Sense::click());
    let layer = ui.layer_id();
    ui.memory_mut(|memory| {
        memory.interested_in_focus(body_id, layer);
        memory.interested_in_focus(arrow_id, layer);
    });
    if ui.is_rect_visible(rect) {
        if enabled && (body_response.hovered() || body_response.has_focus()) {
            ui.painter().rect_filled(body, 3.0, palette.hover);
        }
        if arrow_enabled && (arrow_response.hovered() || arrow_response.has_focus()) {
            ui.painter().rect_filled(arrow_rect, 3.0, palette.hover);
        }
        if enabled && body_response.has_focus() {
            ui.painter().rect_stroke(
                body,
                3.0,
                egui::Stroke::new(2.0_f32, palette.accent),
                egui::StrokeKind::Inside,
            );
        }
        if arrow_enabled && arrow_response.has_focus() {
            ui.painter().rect_stroke(
                arrow_rect,
                3.0,
                egui::Stroke::new(2.0_f32, palette.accent),
                egui::StrokeKind::Inside,
            );
        }
        ui.painter().line_segment(
            [
                egui::pos2(split_at, rect.top() + 8.0),
                egui::pos2(split_at, rect.bottom() - 8.0),
            ],
            egui::Stroke::new(1.0_f32, palette.line),
        );
        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(body.center().x, rect.top() + LARGE_ICON_TOP),
            egui::vec2(LARGE_ICON, LARGE_ICON),
        );
        if let Some(bytes) = ribbon.icons.get(id) {
            Ribbon::image_with(bytes, id, text, ink).paint_at(ui, icon_rect);
        }
        ui.painter().text(
            egui::pos2(body.center().x, rect.top() + LARGE_LABEL_TOP),
            egui::Align2::CENTER_TOP,
            text,
            egui::FontId::proportional(LARGE_LABEL_SIZE),
            ink,
        );
        let centre = arrow_rect.center();
        ui.painter().add(egui::Shape::convex_polygon(
            vec![
                egui::pos2(centre.x - 3.0, centre.y - 2.0),
                egui::pos2(centre.x + 3.0, centre.y - 2.0),
                egui::pos2(centre.x, centre.y + 2.0),
            ],
            arrow_ink,
            egui::Stroke::NONE,
        ));
    }
    body_response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, text));
    arrow_response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, arrow_enabled, arrow_text)
    });
    if enabled && zone_activated(ui, &body_response) {
        *picked = Some(command);
    }
    if arrow_enabled && zone_activated(ui, &arrow_response) {
        *picked = Some(arrow);
    }
    body_response.on_hover_text(hint(text, Some(command)));
    arrow_response.on_hover_text(hint(arrow_text, Some(arrow)));
}

/// Whether a pointer click or the keyboard activated one zone of the control.
fn zone_activated(ui: &egui::Ui, response: &egui::Response) -> bool {
    response.clicked()
        || (response.has_focus()
            && ui.input(|input| {
                input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Space)
            }))
}
