// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The two «Empezar» cards, the template gallery and the empty `Recientes` row.
//!
//! Each template card is its document rendered by `thumbnail` and the name and description
//! from `labels`, with the category chip of the circuit's own medium. The first template is
//! the owner's OR worked example, so it carries the example badge as well.

use eframe::egui;
use punchi_fluid_catalog::Catalog;
use punchi_fluid_session::templates::{self, Template};

use super::thumbnail::thumbnail;
use super::{Action, Drawing, ICON_NEW, ICON_OPEN, ICON_RECENT, activated, focusable, icon};
use crate::theme::Tokens;

/// The plate's height over its width, from the proposal's 512 × 288 thumbnail.
const PLATE_RATIO: f32 = 288.0 / 512.0;

/// The plate is 65 % of the card, the share the proposal measures.
const PLATE_SHARE: f32 = 0.65;

/// The «Empezar» card icon size, in grid units: the proposal's `unit * 1.8` raster rule.
const CARD_ICON: f32 = 1.8;

/// The two «Empezar» cards: the default way in and the quieter secondary one.
pub(super) fn begin_row(ui: &mut egui::Ui, tokens: &Tokens) -> Option<Action> {
    let mut action = None;
    let spacing = ui.spacing().item_spacing.x;
    let width = ((ui.available_width() - spacing) * 0.5).max(tokens.unit * 6.0);
    ui.horizontal(|ui| {
        // TODO(start page, next slice): "Nuevo proyecto" opens the new-project dialog.
        if begin_card(
            ui,
            tokens,
            ICON_NEW,
            "Nuevo proyecto",
            "Crea una hoja de trabajo desde cero",
            true,
            width,
        ) {
            action = Some(Action::Pending);
        }
        // TODO(start page, next slice): "Abrir…" opens the file picker for a saved project.
        if begin_card(
            ui,
            tokens,
            ICON_OPEN,
            "Abrir…",
            "Un proyecto guardado",
            false,
            width,
        ) {
            action = Some(Action::Pending);
        }
    });
    action
}

/// One «Empezar» card: the accent-tinted first way in or the quieter secondary one.
fn begin_card(
    ui: &mut egui::Ui,
    tokens: &Tokens,
    drawing: Drawing,
    title: &str,
    detail: &str,
    primary: bool,
    width: f32,
) -> bool {
    let height = (tokens.unit * 4.5).max(64.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let id = ui.make_persistent_id(("start.begin", title));
    let response = focusable(ui, rect, id);
    if !ui.is_rect_visible(rect) {
        return false;
    }
    let background = if response.is_pointer_button_down_on() {
        tokens.surface_sunken
    } else if primary {
        tokens.accent_soft
    } else {
        tokens.surface_raised
    };
    ui.painter().rect_filled(rect, tokens.radius, background);
    let border = if primary || response.hovered() || response.has_focus() {
        tokens.accent
    } else {
        tokens.line
    };
    ui.painter().rect_stroke(
        rect,
        tokens.radius,
        egui::Stroke::new(1.0_f32, border),
        egui::StrokeKind::Inside,
    );
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + tokens.unit * 1.4, rect.center().y),
        egui::vec2(tokens.unit * CARD_ICON, tokens.unit * CARD_ICON),
    );
    icon(ui, drawing, title, tokens.ink, icon_rect);
    ui.painter().text(
        egui::pos2(
            icon_rect.right() + tokens.unit * 0.6,
            rect.center().y - tokens.unit * 0.45,
        ),
        egui::Align2::LEFT_CENTER,
        title,
        egui::FontId::proportional(tokens.text_label),
        tokens.ink,
    );
    ui.painter().text(
        egui::pos2(
            icon_rect.right() + tokens.unit * 0.6,
            rect.center().y + tokens.unit * 0.45,
        ),
        egui::Align2::LEFT_CENTER,
        detail,
        egui::FontId::proportional(tokens.text_body),
        tokens.ink_dim,
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink(1.5),
            tokens.radius,
            egui::Stroke::new(2.0_f32, tokens.accent),
            egui::StrokeKind::Inside,
        );
    }
    let activated = activated(ui, &response);
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, format!("{title}. {detail}"))
    });
    response.on_hover_text(format!("{title}: {detail}"));
    activated
}

/// The four template cards, each one a thumbnail of its own document and its label.
pub(super) fn template_grid(ui: &mut egui::Ui, tokens: &Tokens, catalog: &Catalog) -> Option<u32> {
    let mut chosen = None;
    let indices: Vec<u32> = (0..templates::COUNT).collect();
    let spacing = ui.spacing().item_spacing.x;
    let min_width = tokens.unit * 10.0;
    let available = ui.available_width();
    let columns = columns(available, spacing, min_width);
    let width = (available - spacing * f32::from(columns - 1)) / f32::from(columns);
    for row in indices.chunks(usize::from(columns)) {
        ui.horizontal(|ui| {
            for index in row {
                if template_card(ui, tokens, catalog, *index, width) {
                    chosen = Some(*index);
                }
            }
        });
    }
    chosen
}

/// How many cards fit in one row: four while each keeps ten grid units, as the proposal asks,
/// and a second row starts below that width.
fn columns(available: f32, spacing: f32, min_width: f32) -> u8 {
    let fits =
        |count: u8| available >= min_width * f32::from(count) + spacing * f32::from(count - 1);
    if fits(4) {
        4
    } else if fits(3) {
        3
    } else if fits(2) {
        2
    } else {
        1
    }
}

/// One template card: the rendered document, the name, the description and the category
/// chip. The first template is the owner's OR worked example, so it carries the example
/// badge as well.
fn template_card(
    ui: &mut egui::Ui,
    tokens: &Tokens,
    catalog: &Catalog,
    index: u32,
    width: f32,
) -> bool {
    let Some(template) = Template::from_index(index) else {
        return false;
    };
    let (name, description) = labels(template);
    let padding = tokens.unit * 0.5;
    let plate_width = width - padding * 2.0;
    let plate_height = plate_width * PLATE_RATIO;
    let height = plate_height / PLATE_SHARE;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let id = ui.make_persistent_id(("start.template", index));
    let response = focusable(ui, rect, id);
    if !ui.is_rect_visible(rect) {
        return false;
    }
    let background = if response.is_pointer_button_down_on() {
        tokens.surface_sunken
    } else {
        tokens.surface_raised
    };
    ui.painter().rect_filled(rect, tokens.radius, background);
    let border = if response.hovered() || response.has_focus() {
        tokens.accent
    } else {
        tokens.line
    };
    ui.painter().rect_stroke(
        rect,
        tokens.radius,
        egui::Stroke::new(1.0_f32, border),
        egui::StrokeKind::Inside,
    );
    let plate = egui::Rect::from_min_size(
        rect.min + egui::vec2(padding, padding),
        egui::vec2(plate_width, plate_height),
    );
    ui.painter()
        .rect_filled(plate, tokens.radius_small, tokens.panel);
    thumbnail(ui, tokens, catalog, index, plate);
    let top = plate.top() + tokens.unit * 0.4;
    let mut right = plate.right() - tokens.unit * 0.4;
    let category = chip(ui, tokens, egui::pos2(right, top), category(template), true);
    right = category.left() - tokens.unit * 0.35;
    if let Some(badge) = example_badge(template) {
        chip(ui, tokens, egui::pos2(right, top), badge, false);
    }
    let name_top = egui::pos2(plate.left(), plate.bottom() + tokens.unit * 0.45);
    ui.painter().text(
        name_top,
        egui::Align2::LEFT_TOP,
        name,
        egui::FontId::proportional(tokens.text_label),
        tokens.ink,
    );
    let detail = elided(ui, tokens, description, plate.width());
    let detail_top = egui::pos2(
        plate.left(),
        name_top.y + tokens.text_label + tokens.unit * 0.15,
    );
    ui.painter().galley(detail_top, detail, tokens.ink_dim);
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink(1.5),
            tokens.radius,
            egui::Stroke::new(2.0_f32, tokens.accent),
            egui::StrokeKind::Inside,
        );
    }
    let activated = activated(ui, &response);
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, accessible_label(template))
    });
    response.on_hover_text(format!("{name}: {description}"));
    activated
}

/// One chip on the plate: the category of the template, or the example badge of the first
/// one. `top_right` is its top-right corner, so chips stack from the plate's right edge.
fn chip(
    ui: &egui::Ui,
    tokens: &Tokens,
    top_right: egui::Pos2,
    text: &str,
    soft: bool,
) -> egui::Rect {
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        egui::FontId::proportional(tokens.text_small),
        tokens.ink,
    );
    let padding = egui::vec2(tokens.unit * 0.35, tokens.unit * 0.18);
    let size = galley.size() + padding * 2.0;
    let rect = egui::Rect::from_min_size(egui::pos2(top_right.x - size.x, top_right.y), size);
    let background = if soft {
        tokens.accent_soft
    } else {
        tokens.surface_raised
    };
    ui.painter()
        .rect_filled(rect, tokens.radius_tip, background);
    if !soft {
        ui.painter().rect_stroke(
            rect,
            tokens.radius_tip,
            egui::Stroke::new(1.0_f32, tokens.line),
            egui::StrokeKind::Inside,
        );
    }
    ui.painter().galley(rect.min + padding, galley, tokens.ink);
    rect
}

/// The empty `Recientes` state: the next step in words, not a blank table. The Rust session
/// keeps no recent list yet, so the page does not invent one.
pub(super) fn recent_placeholder(ui: &mut egui::Ui, tokens: &Tokens) {
    egui::Frame::default()
        .fill(tokens.surface_raised)
        .stroke(egui::Stroke::new(1.0_f32, tokens.line))
        .corner_radius(tokens.radius_control)
        .inner_margin(tokens.unit * 0.6)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let icon_rect = egui::Rect::from_min_size(
                    ui.cursor().min,
                    egui::vec2(tokens.unit * 1.2, tokens.unit * 1.2),
                );
                icon(ui, ICON_RECENT, "Recientes", tokens.ink_muted, icon_rect);
                ui.allocate_space(egui::vec2(tokens.unit * 1.4, tokens.unit * 1.2));
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("Todavía no hay documentos recientes")
                            .size(tokens.text_body)
                            .color(tokens.ink),
                    );
                    ui.label(
                        egui::RichText::new(
                            "Crea un proyecto nuevo o abre un archivo; aparecerá aquí.",
                        )
                        .size(tokens.text_small)
                        .color(tokens.ink_dim),
                    );
                });
            });
        });
}

/// Lays a description out on one line, elided with an ellipsis when the card is narrow.
fn elided(ui: &egui::Ui, tokens: &Tokens, text: &str, width: f32) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::single_section(
        text.to_owned(),
        egui::TextFormat {
            font_id: egui::FontId::proportional(tokens.text_body),
            color: tokens.ink_dim,
            ..Default::default()
        },
    );
    job.wrap = egui::text::TextWrapping {
        max_width: width,
        max_rows: 1,
        break_anywhere: true,
        overflow_character: Some('…'),
    };
    ui.fonts(|fonts| fonts.layout_job(job))
}

/// The name and the one-line description of each start template.
pub(super) fn labels(template: Template) -> (&'static str, &'static str) {
    match template {
        Template::WorkedExample => (
            "Circuito OR",
            "Dos entradas; el shuttle deja pasar la mayor y con las dos cierra la salida.",
        ),
        Template::DoubleActing => (
            "Cilindro de doble efecto",
            "Una válvula 5/2 y un cilindro de dos cámaras.",
        ),
        Template::AirPreparation => (
            "Preparación de aire",
            "Filtro, regulador y lubricador antes de la válvula.",
        ),
        Template::ElectricalControl => {
            ("Control eléctrico", "Un relé y una válvula con solenoide.")
        }
    }
}

/// The domain chip of each template: the circuit's own medium.
pub(super) fn category(template: Template) -> &'static str {
    match template {
        Template::ElectricalControl => "ELÉCTRICA",
        Template::WorkedExample | Template::DoubleActing | Template::AirPreparation => "NEUMÁTICA",
    }
}

/// The badge of the owner's worked example, and nothing on the other three cards.
pub(super) fn example_badge(template: Template) -> Option<&'static str> {
    match template {
        Template::WorkedExample => Some("EJEMPLO"),
        Template::DoubleActing | Template::AirPreparation | Template::ElectricalControl => None,
    }
}

/// The name the accessibility layer reads for a template card: its title, its description
/// and, for the first one, that it is the owner's worked example.
pub(super) fn accessible_label(template: Template) -> String {
    let (name, description) = labels(template);
    match example_badge(template) {
        Some(_) => format!("{name}. {description} Ejemplo resuelto."),
        None => format!("{name}. {description}"),
    }
}
