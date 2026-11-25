// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The two «Empezar» cards, the template gallery and the recent documents.
//!
//! Each template card is its document rendered by `thumbnail` and the name and description
//! from `labels`, with the category chip of the circuit's own medium. The first template is
//! the owner's OR worked example, so it carries the example badge as well.
//!
//! The `Recientes` rows are the session's list: the file name, the folder, a click that
//! opens the document and a small `Quitar` button. A row whose file is gone stays visible,
//! muted and named as not found: the list explains itself instead of dropping a row behind
//! the reader's back.

use std::path::{Path, PathBuf};

use eframe::egui;
use punchi_fluid_catalog::Catalog;
use punchi_fluid_file::recent::Recent;
use punchi_fluid_i18n::Language;
use punchi_fluid_session::recents::Recents;
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

/// The offer to recover the safety copy found at start, above «Empezar»: what it is, the file
/// it belongs to when the document has one, and the two answers. The banner is only drawn
/// while a copy waits, so it leaves without a word once it is answered.
pub(super) fn recovery_banner(
    ui: &mut egui::Ui,
    tokens: &Tokens,
    document_name: Option<&str>,
    language: Language,
) -> Option<Action> {
    let mut action = None;
    egui::Frame::default()
        .fill(tokens.accent_soft)
        .stroke(egui::Stroke::new(1.0_f32, tokens.accent))
        .corner_radius(tokens.radius_control)
        .inner_margin(tokens.unit * 0.7)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                egui::RichText::new(language.text("start.recovery_title"))
                    .size(tokens.text_name)
                    .strong()
                    .color(tokens.ink),
            );
            let detail = match document_name {
                Some(name) => language.format("start.recovery_body", &[("name", name)]),
                None => language.text("start.recovery_body_unnamed").to_owned(),
            };
            ui.label(
                egui::RichText::new(detail)
                    .size(tokens.text_secondary)
                    .color(tokens.ink_dim),
            );
            ui.add_space(tokens.unit * 0.3);
            ui.horizontal(|ui| {
                if ui.button(language.text("start.recover")).clicked() {
                    action = Some(Action::Recover);
                }
                if ui.button(language.text("start.discard")).clicked() {
                    action = Some(Action::DiscardRecovery);
                }
            });
        });
    action
}

/// The two «Empezar» cards: the default way in and the quieter secondary one.
pub(super) fn begin_row(ui: &mut egui::Ui, tokens: &Tokens, language: Language) -> Option<Action> {
    let mut action = None;
    let spacing = ui.spacing().item_spacing.x;
    let width = ((ui.available_width() - spacing) * 0.5).max(tokens.unit * 6.0);
    ui.horizontal(|ui| {
        if begin_card(
            ui,
            tokens,
            ICON_NEW,
            language.text("start.begin_new"),
            language.text("start.begin_new_detail"),
            true,
            width,
        ) {
            action = Some(Action::NewProject);
        }
        if begin_card(
            ui,
            tokens,
            ICON_OPEN,
            language.text("start.begin_open"),
            language.text("start.begin_open_detail"),
            false,
            width,
        ) {
            action = Some(Action::OpenDialog);
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
        egui::FontId::proportional(tokens.text_name),
        tokens.ink,
    );
    ui.painter().text(
        egui::pos2(
            icon_rect.right() + tokens.unit * 0.6,
            rect.center().y + tokens.unit * 0.45,
        ),
        egui::Align2::LEFT_CENTER,
        detail,
        egui::FontId::proportional(tokens.text_secondary),
        tokens.ink_dim,
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink(1.5),
            tokens.radius,
            egui::Stroke::new(tokens.focus_width, tokens.accent),
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
pub(super) fn template_grid(
    ui: &mut egui::Ui,
    tokens: &Tokens,
    catalog: &Catalog,
    language: Language,
) -> Option<u32> {
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
                if template_card(ui, tokens, catalog, *index, width, language) {
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
    language: Language,
) -> bool {
    let Some(template) = Template::from_index(index) else {
        return false;
    };
    let (name, description) = labels(language, template);
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
    let category = chip(
        ui,
        tokens,
        egui::pos2(right, top),
        category(language, template),
        true,
    );
    right = category.left() - tokens.unit * 0.35;
    if let Some(badge) = example_badge(language, template) {
        chip(ui, tokens, egui::pos2(right, top), badge, false);
    }
    let name_top = egui::pos2(plate.left(), plate.bottom() + tokens.unit * 0.45);
    ui.painter().text(
        name_top,
        egui::Align2::LEFT_TOP,
        name,
        egui::FontId::proportional(tokens.text_name),
        tokens.ink,
    );
    let detail = elided(ui, tokens, description, plate.width());
    let detail_top = egui::pos2(
        plate.left(),
        name_top.y + tokens.text_name + tokens.unit * 0.15,
    );
    ui.painter().galley(detail_top, detail, tokens.ink_dim);
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink(1.5),
            tokens.radius,
            egui::Stroke::new(tokens.focus_width, tokens.accent),
            egui::StrokeKind::Inside,
        );
    }
    let activated = activated(ui, &response);
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            true,
            accessible_label(language, template),
        )
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

/// The `Recientes` rows, newest first, or the empty state when nothing is remembered.
pub(super) fn recent_rows(
    ui: &mut egui::Ui,
    tokens: &Tokens,
    recents: &Recents,
    language: Language,
) -> Option<Action> {
    if recents.entries().is_empty() {
        recent_placeholder(ui, tokens, language);
        return None;
    }
    let mut action = None;
    for recent in recents.entries() {
        match recent_row(ui, tokens, recent, language) {
            Some(RowAction::Open) => {
                action = Some(Action::OpenRecent(PathBuf::from(&recent.path)));
            }
            Some(RowAction::Forget) => {
                action = Some(Action::ForgetRecent(recent.path.clone()));
            }
            None => {}
        }
        ui.add_space(tokens.unit * 0.25);
    }
    action
}

/// What a recent row asked for.
enum RowAction {
    /// Read the remembered document.
    Open,
    /// Drop the row from the list.
    Forget,
}

/// One remembered document: its name, its folder, a click that opens it and a small button
/// that forgets it. A row whose file is gone is painted muted with `no encontrado`, cannot
/// be opened, and stays until the reader removes it.
fn recent_row(
    ui: &mut egui::Ui,
    tokens: &Tokens,
    recent: &Recent,
    language: Language,
) -> Option<RowAction> {
    let path = Path::new(&recent.path);
    let missing = recent_missing(path);
    let height = (tokens.unit * 2.6).max(40.0);
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::hover(),
    );
    // The remove target keeps the interface's 24 px minimum even when the row is at its
    // floor, and never overlaps the open target: a click does one thing.
    let remove_rect = egui::Rect::from_center_size(
        egui::pos2(rect.right() - tokens.unit * 1.5, rect.center().y),
        egui::vec2(
            tokens.unit * 2.4,
            (tokens.unit * 1.6).max(tokens.target_min),
        ),
    );
    let open_rect = egui::Rect::from_min_max(
        rect.min,
        egui::pos2(
            (remove_rect.left() - tokens.unit * 0.4).max(rect.left()),
            rect.max.y,
        ),
    );
    let open = focusable(
        ui,
        open_rect,
        ui.make_persistent_id(("start.recent", &recent.path)),
    );
    let remove = focusable(
        ui,
        remove_rect,
        ui.make_persistent_id(("start.recent.remove", &recent.path)),
    );
    if !ui.is_rect_visible(rect) {
        return None;
    }
    let layout = RecentLayout {
        rect,
        remove_rect,
        open,
        remove,
    };
    let detail = recent_detail(path, missing, language);
    paint_recent_row(ui, tokens, recent, missing, &detail, &layout, language);
    let accessible = format!("{}. {detail}", recent.name);
    layout.open.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, !missing, accessible.clone())
    });
    let forget_label = language.format("start.forget_named", &[("name", &recent.name)]);
    layout.remove.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, forget_label.clone())
    });
    let forget = activated(ui, &layout.remove);
    let open_activated = !missing && activated(ui, &layout.open);
    if missing {
        layout
            .open
            .on_hover_text(language.text("start.missing_recent"));
    } else {
        layout
            .open
            .on_hover_text(language.format("start.open_named", &[("name", &recent.name)]));
    }
    layout
        .remove
        .on_hover_text(language.format("start.forget_named", &[("name", &recent.name)]));
    if forget {
        Some(RowAction::Forget)
    } else if open_activated {
        Some(RowAction::Open)
    } else {
        None
    }
}

/// The rectangles and presses one recent row owns.
struct RecentLayout {
    rect: egui::Rect,
    remove_rect: egui::Rect,
    open: egui::Response,
    remove: egui::Response,
}

/// Paints one row: its plate, the two text lines and the small `Quitar` label.
fn paint_recent_row(
    ui: &egui::Ui,
    tokens: &Tokens,
    recent: &Recent,
    missing: bool,
    detail: &str,
    layout: &RecentLayout,
    language: Language,
) {
    let rect = layout.rect;
    let remove_rect = layout.remove_rect;
    let hot = !missing && (layout.open.hovered() || layout.open.is_pointer_button_down_on());
    ui.painter().rect_filled(
        rect,
        tokens.radius_control,
        if hot {
            tokens.surface_sunken
        } else {
            tokens.surface_raised
        },
    );
    if (!missing && layout.open.has_focus()) || layout.remove.has_focus() {
        ui.painter().rect_stroke(
            rect,
            tokens.radius_control,
            egui::Stroke::new(1.0_f32, tokens.accent),
            egui::StrokeKind::Inside,
        );
    }
    if layout.remove.hovered() || layout.remove.has_focus() {
        ui.painter()
            .rect_filled(remove_rect, tokens.radius_small, tokens.accent_soft);
    }
    let title_left = rect.left() + tokens.unit * 0.7;
    ui.painter().text(
        egui::pos2(title_left, rect.center().y - tokens.unit * 0.4),
        egui::Align2::LEFT_CENTER,
        &recent.name,
        egui::FontId::proportional(tokens.text_name),
        recent_ink(tokens, missing),
    );
    ui.painter().text(
        egui::pos2(title_left, rect.center().y + tokens.unit * 0.5),
        egui::Align2::LEFT_CENTER,
        detail,
        egui::FontId::proportional(tokens.text_small),
        if missing {
            tokens.ink_muted
        } else {
            tokens.ink_dim
        },
    );
    ui.painter().text(
        remove_rect.center(),
        egui::Align2::CENTER_CENTER,
        language.text("start.forget"),
        egui::FontId::proportional(tokens.text_small),
        if layout.remove.hovered() || layout.remove.has_focus() {
            tokens.accent
        } else {
            tokens.ink_dim
        },
    );
}

/// The second line of a recent row: the folder, and that the file is gone when it is.
fn recent_detail(path: &Path, missing: bool, language: Language) -> String {
    let folder = folder_of(path);
    if missing {
        language.format("start.not_found", &[("folder", &folder)])
    } else {
        folder
    }
}

/// Whether the remembered document is not there to be read. A folder or a dangling link is
/// not a document either, so the row says so instead of failing when the reader clicks it.
pub(super) fn recent_missing(path: &Path) -> bool {
    !path.is_file()
}

/// The ink of a recent row's name: muted when the document is gone.
pub(super) fn recent_ink(tokens: &Tokens, missing: bool) -> egui::Color32 {
    if missing {
        tokens.ink_muted
    } else {
        tokens.ink
    }
}

/// The folder a remembered document lives in, or `.` for a path with none.
fn folder_of(path: &Path) -> String {
    path.parent()
        .filter(|folder| !folder.as_os_str().is_empty())
        .map_or_else(|| String::from("."), |folder| folder.display().to_string())
}

/// The empty `Recientes` state: the next step in words, not a blank table.
pub(super) fn recent_placeholder(ui: &mut egui::Ui, tokens: &Tokens, language: Language) {
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
                icon(
                    ui,
                    ICON_RECENT,
                    language.text("start.section_recents"),
                    tokens.ink_muted,
                    icon_rect,
                );
                ui.allocate_space(egui::vec2(tokens.unit * 1.4, tokens.unit * 1.2));
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(language.text("start.empty_title"))
                            .size(tokens.text_name)
                            .color(tokens.ink),
                    );
                    ui.label(
                        egui::RichText::new(language.text("start.empty_body"))
                            .size(tokens.text_secondary)
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
            font_id: egui::FontId::proportional(tokens.text_secondary),
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
pub(super) fn labels(language: Language, template: Template) -> (&'static str, &'static str) {
    let (name, description) = match template {
        Template::WorkedExample => ("start.template.or_name", "start.template.or_description"),
        Template::DoubleActing => (
            "start.template.double_name",
            "start.template.double_description",
        ),
        Template::AirPreparation => ("start.template.air_name", "start.template.air_description"),
        Template::ElectricalControl => (
            "start.template.electrical_name",
            "start.template.electrical_description",
        ),
    };
    (language.text(name), language.text(description))
}

/// The domain chip of each template: the circuit's own medium.
pub(super) fn category(language: Language, template: Template) -> &'static str {
    language.text(match template {
        Template::ElectricalControl => "start.template.electrical",
        Template::WorkedExample | Template::DoubleActing | Template::AirPreparation => {
            "start.template.pneumatic"
        }
    })
}

/// The badge of the owner's worked example, and nothing on the other three cards.
pub(super) fn example_badge(language: Language, template: Template) -> Option<&'static str> {
    match template {
        Template::WorkedExample => Some(language.text("start.template.example")),
        Template::DoubleActing | Template::AirPreparation | Template::ElectricalControl => None,
    }
}

/// The name the accessibility layer reads for a template card: its title, its description
/// and, for the first one, that it is the owner's worked example.
pub(super) fn accessible_label(language: Language, template: Template) -> String {
    let (name, description) = labels(language, template);
    match example_badge(language, template) {
        Some(_) => language.format(
            "start.template.example_accessible",
            &[("name", name), ("description", description)],
        ),
        None => format!("{name}. {description}"),
    }
}
