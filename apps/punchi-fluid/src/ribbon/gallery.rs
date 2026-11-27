// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! One gallery card: its preview, its name, its real measures and the mark of the value in
//! force.

use eframe::egui;
use punchi_fluid_i18n::Language;

use super::model::{GalleryEntry, Preview};
use super::{Command, Palette, Ribbon};

#[allow(
    clippy::too_many_lines,
    reason = "one gallery card paints its preview, its name and its measures as one geometry"
)]
pub(super) fn card(
    ribbon: &Ribbon,
    ui: &mut egui::Ui,
    entry: &GalleryEntry,
    palette: &Palette,
    picked: &mut Option<Command>,
    language: Language,
) {
    let enabled = entry.command.is_some_and(|command| ribbon.enabled(command));
    let selected = entry
        .command
        .is_some_and(|command| ribbon.command_selected(command) || command == Command::PageSetup);
    let label = language.text(entry.label);
    let ink = if enabled { palette.ink } else { palette.muted };
    let (rect, response) = ui.allocate_exact_size(egui::vec2(132.0, 86.0), egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let background = if enabled && response.hovered() {
            palette.hover
        } else {
            palette.raised
        };
        ui.painter().rect_filled(rect, 4.0, background);
        ui.painter().rect_stroke(
            rect,
            4.0,
            egui::Stroke::new(
                if selected { 2.0_f32 } else { 1.0 },
                if selected {
                    palette.accent
                } else {
                    palette.line
                },
            ),
            egui::StrokeKind::Inside,
        );
        if selected {
            ui.painter().text(
                egui::pos2(rect.right() - 12.0, rect.top() + 12.0),
                egui::Align2::CENTER_CENTER,
                "•",
                egui::FontId::proportional(12.0),
                palette.accent,
            );
        }
        let preview =
            egui::Rect::from_min_size(rect.min + egui::vec2(8.0, 6.0), egui::vec2(116.0, 42.0));
        match entry.preview {
            Preview::Margins(fraction) => {
                let page = egui::Rect::from_center_size(preview.center(), egui::vec2(58.0, 40.0));
                ui.painter().rect_filled(page, 1.0, egui::Color32::WHITE);
                ui.painter().rect_stroke(
                    page,
                    1.0,
                    egui::Stroke::new(1.0_f32, palette.ink),
                    egui::StrokeKind::Inside,
                );
                let margin = fraction * 18.0;
                ui.painter().rect_stroke(
                    page.shrink(margin),
                    0.0,
                    egui::Stroke::new(1.0_f32, palette.muted),
                    egui::StrokeKind::Inside,
                );
            }
            Preview::Pipe(icon) => {
                if let Some(bytes) = ribbon.icons.get(icon) {
                    Ribbon::image(bytes, icon, label, palette).paint_at(
                        ui,
                        egui::Rect::from_center_size(preview.center(), egui::vec2(28.0, 28.0)),
                    );
                }
            }
        }
        let measures = match entry.command {
            Some(Command::PageMargin(preset)) => Some(preset.margins()),
            Some(Command::PageSetup) => Some(ribbon.margins),
            _ => None,
        };
        let title_y = if measures.is_some() {
            rect.max.y - 27.0
        } else {
            rect.max.y - 14.0
        };
        ui.painter().text(
            egui::pos2(rect.center().x, title_y),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(11.0),
            ink,
        );
        if let Some(margins) = measures {
            let text = language.format(
                "ribbon.gallery.margin.measures",
                &[
                    ("left", &format!("{}", margins.left)),
                    ("top", &format!("{}", margins.top)),
                    ("right", &format!("{}", margins.right)),
                    ("bottom", &format!("{}", margins.bottom)),
                ],
            );
            ui.painter().text(
                egui::pos2(rect.center().x, rect.max.y - 12.0),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(8.5),
                palette.muted,
            );
        }
    }
    if selected {
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Button, enabled, true, label)
        });
    } else {
        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, label));
    }
    if enabled && response.clicked() {
        if let Some(command) = entry.command {
            *picked = Some(command);
        }
        ui.close();
    }
}
