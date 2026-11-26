// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The status bar: mode, view, selection, reading and warning, in that order. The frame's
//! draw time is a diagnostic and stays out of the normal bar; the file name lives in the
//! window title. Every value comes from the same window state the ribbon and the sheet use.
//!
//! Mode and notice are the priorities. The notice is reserved on the right and the middle
//! groups are elided in place, so a long selection or a warning never pushes the other off
//! the window; the full text survives as the accessible name and the elided tooltip.

use eframe::egui;
use punchi_fluid_i18n::Language;

use crate::Window;
use crate::theme::Tokens;

impl Window {
    pub(crate) fn show_status_bar(&mut self, context: &egui::Context) {
        let language = self.language;
        let mode = self.status_mode(language);
        let view = self.status_view(language);
        let selection = self.status_selection(language);
        let reading = self.status_reading(language);
        let notice = self.notice.clone();
        let tokens = &self.theme.tokens;
        egui::TopBottomPanel::bottom("status").show(context, |ui| {
            status_row(
                ui,
                tokens,
                &mode,
                &view,
                &selection,
                &reading,
                notice.as_deref(),
            );
        });
    }

    fn status_view(&self, language: Language) -> String {
        let (width, height) = self.page.size_mm();
        let zoom = format!("{:.0}", self.view.zoom() * 100.0);
        let width = format!("{width:.0}");
        let height = format!("{height:.0}");
        language.format(
            "status.view",
            &[("width", &width), ("height", &height), ("zoom", &zoom)],
        )
    }

    fn status_mode(&self, language: Language) -> String {
        let mode = language.text(crate::ribbon::mode_key(self.running, self.simulation));
        let Some(telemetry) = self.model.as_ref().map(|model| model.network().telemetry()) else {
            return mode.to_owned();
        };
        let time = language.decimal(telemetry.time, 2);
        language.format("status.mode_time", &[("mode", mode), ("time", &time)])
    }

    fn status_reading(&self, language: Language) -> String {
        let Some(index) = self.selected.as_slice().first().copied() else {
            return language.text("status.reading_none").to_owned();
        };
        if self.selected.len() != 1 || self.selected_pipe.is_some() {
            return language.text("status.reading_none").to_owned();
        }
        let Some(placement) = self.document.placements().get(index) else {
            return language.text("status.reading_none").to_owned();
        };
        let Some(definition) = self.catalog.get(&placement.catalog_id) else {
            return language.text("status.reading_none").to_owned();
        };
        let Some(model) = self.model.as_ref() else {
            return language.text("status.reading_none").to_owned();
        };
        let Some(network_part) = model.part_of(index) else {
            return language.text("status.reading_none").to_owned();
        };
        let Some(port) = definition.port.iter().find(|port| {
            !matches!(
                port.role,
                punchi_fluid_catalog::PortRole::Mechanical
                    | punchi_fluid_catalog::PortRole::Electrical
            )
        }) else {
            return language.text("status.reading_none").to_owned();
        };
        let Some(pressure) = model.network().port_pressure(network_part, &port.id) else {
            return language.text("status.reading_none").to_owned();
        };
        let gauge = (pressure - model.network().atmosphere_pa()) / 1.0e5;
        let value = language.decimal(gauge, 2);
        language.format(
            "status.reading_port",
            &[("port", port.numeral()), ("value", &value)],
        )
    }

    fn status_selection(&self, language: Language) -> String {
        let circuit = self.active_circuit_title();
        if self.selected_pipe.is_some() {
            return language.format("status.selection_pipe", &[("circuit", circuit)]);
        }
        match self.selected.as_slice() {
            [] => language.text("status.selection_none").to_owned(),
            [index] => {
                let name = self
                    .selected_component_name(*index)
                    .unwrap_or_else(|| language.text("status.selection_unknown").to_owned());
                language.format(
                    "status.selection_component",
                    &[("name", &name), ("circuit", circuit)],
                )
            }
            selected => {
                let count = selected.len().to_string();
                language.format("status.selection_many", &[("count", &count)])
            }
        }
    }

    /// The instance's designation first, because it is the name the drawing and the parts
    /// list use, and the catalog name in the active language otherwise.
    fn selected_component_name(&self, index: usize) -> Option<String> {
        let placement = self.document.placements().get(index)?;
        if let Some(designation) = placement.designation.as_deref()
            && !designation.trim().is_empty()
        {
            return Some(designation.to_owned());
        }
        let definition = self.catalog.get(&placement.catalog_id)?;
        let name = match self.language {
            Language::Spanish => definition.name_es.as_deref().unwrap_or(&definition.name),
            Language::English => &definition.name,
        };
        Some(name.to_owned())
    }

    fn active_circuit_title(&self) -> &str {
        let active = self.document.active_circuit();
        self.document
            .circuits()
            .iter()
            .find(|circuit| circuit.id == active)
            .map_or("", |circuit| circuit.title.as_str())
    }
}

/// One status row: the mode on the left, the notice reserved on the right and the middle
/// groups elided in place. The notice is a priority, so a long selection or a long warning
/// only costs the middle cells width, never the notice's visible rectangle.
fn status_row(
    ui: &mut egui::Ui,
    tokens: &Tokens,
    mode: &str,
    view: &str,
    selection: &str,
    reading: &str,
    notice: Option<&str>,
) {
    let full = ui.available_width();
    let height = ui.text_style_height(&egui::TextStyle::Body);
    // Measure with the font the labels actually inherit, not with the token alone: a test
    // harness or a context without the theme applied still lays the row out correctly.
    let font = ui
        .style()
        .text_styles
        .get(&egui::TextStyle::Body)
        .cloned()
        .unwrap_or_else(|| egui::FontId::proportional(tokens.text_body));
    let (mode_w, view_w, selection_w, reading_w, notice_w) = {
        let measure = |text: &str| {
            ui.fonts(|fonts| {
                fonts
                    .layout_no_wrap(text.to_owned(), font.clone(), tokens.ink)
                    .size()
                    .x
            })
        };
        (
            measure(mode),
            measure(view),
            measure(selection),
            measure(reading),
            notice.map_or(0.0, measure),
        )
    };

    let (row, _) = ui.allocate_exact_size(egui::vec2(full, height), egui::Sense::hover());
    let gap = ui
        .spacing()
        .item_spacing
        .x
        .max(ui.style().visuals.widgets.noninteractive.bg_stroke.width);
    let stroke = ui.style().visuals.widgets.noninteractive.bg_stroke;

    let mode_w = mode_w.min(full);
    let mut x = row.left();
    draw_cell(
        ui,
        egui::Rect::from_min_size(egui::pos2(x, row.top()), egui::vec2(mode_w, row.height())),
        mode,
        tokens.ink,
        true,
    );
    x += mode_w;

    // The notice keeps its own width on the right, but the middle keeps a readable floor:
    // without it, a long warning would erase the selection instead of eliding it.
    let middle_floor = tokens.unit * 8.0;
    let notice_max = (row.right() - x - gap * 2.0 - middle_floor).max(0.0);
    let notice_w = notice_w.min(notice_max);
    let notice_rect = notice.filter(|_| notice_w > 0.0).map(|_| {
        egui::Rect::from_min_size(
            egui::pos2(row.right() - notice_w, row.top()),
            egui::vec2(notice_w, row.height()),
        )
    });
    let middle_left = x + gap;
    let middle_right = notice_rect.map_or(row.right(), |rect| rect.left() - gap);
    let middle_w = (middle_right - middle_left).max(0.0);

    let natural = [view_w, selection_w, reading_w];
    let (keep, widths) = middle_layout(middle_w, natural, gap, tokens.target_min);
    let labels = [view, selection, reading];
    let colours = [tokens.ink_dim, tokens.ink, tokens.ink_dim];

    for index in 0..3 {
        if !keep[index] || widths[index] <= 0.0 {
            continue;
        }
        // A hair line between sections, drawn from the theme's own stroke.
        draw_separator(ui, x + gap * 0.5, row, stroke);
        x += gap;
        draw_cell(
            ui,
            egui::Rect::from_min_size(
                egui::pos2(x, row.top()),
                egui::vec2(widths[index], row.height()),
            ),
            labels[index],
            colours[index],
            false,
        );
        x += widths[index];
    }

    if let (Some(notice), Some(rect)) = (notice, notice_rect) {
        draw_separator(ui, rect.left() - gap * 0.5, row, stroke);
        draw_cell(ui, rect, notice, tokens.accent, false);
    }
}

/// The middle groups: the natural widths while they fit, then proportional shares. A group
/// whose share is below one target is dropped, reading first and view after, so the
/// selection keeps a readable cell instead of being erased by a long warning.
fn middle_layout(
    middle_w: f32,
    natural: [f32; 3],
    gap: f32,
    target_min: f32,
) -> ([bool; 3], [f32; 3]) {
    let mut keep = [true, true, true];
    let mut widths = [0.0_f32; 3];
    loop {
        let kept = keep.iter().filter(|kept| **kept).count();
        let gap_total = match kept {
            0 | 1 => 0.0,
            2 => gap,
            _ => gap * 2.0,
        };
        let usable = (middle_w - gap_total).max(0.0);
        let total: f32 = (0..3)
            .filter(|&index| keep[index])
            .map(|index| natural[index])
            .sum();
        if total <= 0.0 || usable <= 0.0 {
            break;
        }
        if total <= usable {
            widths = std::array::from_fn(|index| if keep[index] { natural[index] } else { 0.0 });
            break;
        }
        let candidate = natural.map(|width| width / total * usable);
        let drop = (0..3)
            .rev()
            .find(|&index| keep[index] && candidate[index] < target_min);
        match drop {
            Some(index) if kept > 1 => {
                keep[index] = false;
            }
            _ => {
                widths =
                    std::array::from_fn(|index| if keep[index] { candidate[index] } else { 0.0 });
                break;
            }
        }
    }
    (keep, widths)
}

/// A text cell with a maximum width: it elides inside it and keeps the full text as its
/// accessible name and its tooltip.
fn draw_cell(ui: &mut egui::Ui, rect: egui::Rect, text: &str, colour: egui::Color32, strong: bool) {
    if rect.width() <= 0.0 {
        return;
    }
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
        |ui| {
            let mut rich = egui::RichText::new(text).color(colour);
            if strong {
                rich = rich.strong();
            }
            ui.add(egui::Label::new(rich).truncate());
        },
    );
}

/// The hair line the bar already draws between its groups, from the theme's widget stroke.
fn draw_separator(ui: &egui::Ui, x: f32, row: egui::Rect, stroke: egui::Stroke) {
    let margin = row.height() * 0.15;
    ui.painter().vline(
        x,
        egui::Rangef::new(row.top() + margin, row.bottom() - margin),
        stroke,
    );
}
