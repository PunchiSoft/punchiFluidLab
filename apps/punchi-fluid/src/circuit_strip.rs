// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The circuit strip at the foot of the sheet: one tab per circuit, the way back to Inicio and
//! the switch between circuits. Switching is a view change (the document's `select_circuit`),
//! never an undo; the strip is not a fourth region: it lives in the band the drawing area
//! reserves at the bottom of the canvas.

use eframe::egui;
use punchi_fluid_doc::Document;
use punchi_fluid_i18n::Language;

use crate::theme::Tokens;

/// The home drawing, the same file the start rail uses.
const ICON_HOME: (&str, &[u8]) = (
    "go-home",
    include_bytes!("../../../assets/icons/actions/go-home.svg"),
);
/// The circuit's own drawing in its tab: a document, as the reference shows it.
const ICON_CIRCUIT: (&str, &[u8]) = (
    "x-office-document",
    include_bytes!("../../../assets/icons/actions/x-office-document.svg"),
);

/// What the strip asks the window to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Action {
    /// The home control: open Inicio.
    Home,
    /// A tab, or `Ctrl+Tab`, chose another circuit.
    Switch(u32),
    /// The tab's close cross: close that circuit through the window's dialog.
    Close(u32),
    /// The tab's rename gesture (`F2` or a double click): rename the circuit in view.
    Rename(u32),
    /// The plus after the last tab: add a circuit.
    Add,
}

/// Draws the strip in `rect` and returns the reader's choice. The tabs are painted controls,
/// named for the accessibility layer and reachable with `Tab`, `Enter` and `Space`.
pub(crate) fn show(
    context: &egui::Context,
    rect: egui::Rect,
    tokens: &Tokens,
    language: Language,
    document: &Document,
) -> Option<Action> {
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return None;
    }
    let mut action = None;
    // `Ctrl+Tab` and `Ctrl+Shift+Tab` walk the circuits, as the reference did.
    if document.circuits().len() > 1 {
        let backwards = context.input(|input| input.modifiers.shift);
        if context.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::Tab)) {
            action = next_circuit(document, backwards);
        }
    }
    egui::Area::new(egui::Id::new("circuit.strip"))
        .order(egui::Order::Middle)
        .fixed_pos(rect.min)
        .show(context, |ui| {
            egui::Frame::default()
                .fill(tokens.surface_sunken)
                .stroke(egui::Stroke::new(1.0_f32, tokens.line))
                .inner_margin(egui::Margin::symmetric(4, 2))
                .show(ui, |ui| {
                    ui.set_width(rect.width() - 8.0);
                    ui.set_height(rect.height() - 4.0);
                    ui.horizontal_centered(|ui| {
                        if home(ui, tokens, language) {
                            action = Some(Action::Home);
                        }
                        ui.separator();
                        egui::ScrollArea::horizontal()
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let many = document.circuits().len() > 1;
                                    for circuit in document.circuits() {
                                        let active = circuit.id == document.active_circuit();
                                        let (response, close, rename, tab_rect) =
                                            tab(ui, tokens, language, &circuit.title, active, many);
                                        if active {
                                            // Scroll only when the circuit changes: a
                                            // scroll every frame never settles and repaints.
                                            let id = egui::Id::new("circuit.strip.last");
                                            let last =
                                                ui.ctx().data(|data| data.get_temp::<u32>(id));
                                            if last != Some(circuit.id) {
                                                ui.ctx().data_mut(|data| {
                                                    data.insert_temp(id, circuit.id);
                                                });
                                                ui.scroll_to_rect(
                                                    tab_rect,
                                                    Some(egui::Align::Center),
                                                );
                                            }
                                        }
                                        if close {
                                            action = Some(Action::Close(circuit.id));
                                        } else if rename {
                                            action = Some(Action::Rename(circuit.id));
                                        } else if response && !active {
                                            action = Some(Action::Switch(circuit.id));
                                        }
                                    }
                                    if document.circuits().len() < punchi_fluid_doc::MAX_CIRCUITS
                                        && add(ui, tokens, language)
                                    {
                                        action = Some(Action::Add);
                                    }
                                });
                            });
                    });
                });
        });
    action
}

/// The circuit after (or before) the active one, as a view change.
fn next_circuit(document: &Document, backwards: bool) -> Option<Action> {
    let circuits = document.circuits();
    let index = circuits
        .iter()
        .position(|circuit| circuit.id == document.active_circuit())?;
    let next = if backwards {
        (index + circuits.len() - 1) % circuits.len()
    } else {
        (index + 1) % circuits.len()
    };
    Some(Action::Switch(circuits[next].id))
}

/// The fixed home control at the strip's left: not a circuit, never in the list.
fn home(ui: &mut egui::Ui, tokens: &Tokens, language: Language) -> bool {
    let label = language.text("circuit.home");
    let side = tokens.unit * 1.8;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(side, side), egui::Sense::hover());
    let id = ui.make_persistent_id("circuit.strip.home");
    let response = focusable(ui, rect, id);
    if !ui.is_rect_visible(rect) {
        return false;
    }
    if response.hovered() || response.has_focus() {
        ui.painter()
            .rect_filled(rect, tokens.radius_control, tokens.surface_raised);
    }
    paint_icon(ui, ICON_HOME, rect, tokens.ink);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
    let clicked = response.clicked() || activated(ui, &response);
    response.on_hover_text(label);
    clicked
}

/// One circuit tab: the document, its name, the close cross and, when active, the accent rule
/// on the seam. Returns whether the tab, the cross or the rename gesture fired, with its
/// rectangle for scrolling.
fn tab(
    ui: &mut egui::Ui,
    tokens: &Tokens,
    language: Language,
    title: &str,
    active: bool,
    many: bool,
) -> (bool, bool, bool, egui::Rect) {
    let font = egui::FontId::proportional(tokens.text_body);
    let text_width = ui.fonts(|fonts| {
        fonts
            .layout_no_wrap(title.to_owned(), font.clone(), tokens.ink)
            .size()
            .x
    });
    let width = text_width.min(200.0) + tokens.unit * 3.2 + if many { tokens.unit } else { 0.0 };
    let height = (tokens.unit * 2.0).max(28.0) - 6.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let close_width = if many { tokens.unit * 1.2 } else { 0.0 };
    let label_rect = egui::Rect::from_min_max(
        rect.min,
        egui::pos2(rect.right() - close_width, rect.bottom()),
    );
    let tab_id = ui.make_persistent_id(("circuit.strip.tab", title));
    let response = focusable(ui, label_rect, tab_id);
    if !ui.is_rect_visible(rect) {
        return (false, false, false, rect);
    }
    if active {
        ui.painter()
            .rect_filled(rect, tokens.radius_control, tokens.panel);
        ui.painter().rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(rect.left() + 2.0, rect.bottom() - 2.0),
                egui::pos2(rect.right() - 2.0, rect.bottom()),
            ),
            0.0,
            tokens.accent,
        );
    } else if response.hovered() || response.has_focus() {
        ui.painter()
            .rect_filled(rect, tokens.radius_control, tokens.surface_raised);
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect,
            tokens.radius_control,
            egui::Stroke::new(1.0_f32, tokens.accent),
            egui::StrokeKind::Inside,
        );
    }
    let ink = if active { tokens.ink } else { tokens.ink_muted };
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + tokens.unit * 0.9, rect.center().y),
        egui::vec2(tokens.unit, tokens.unit),
    );
    paint_icon(ui, ICON_CIRCUIT, icon_rect, ink);
    let galley = ui.painter().layout(
        title.to_owned(),
        font,
        ink,
        label_rect.width() - tokens.unit * 2.4,
    );
    ui.painter().galley(
        egui::pos2(
            icon_rect.right() + tokens.unit * 0.4,
            rect.center().y - galley.size().y * 0.5,
        ),
        galley,
        ink,
    );
    response
        .widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button, true, active, title));
    let rename = response.double_clicked()
        || (response.has_focus() && ui.input(|input| input.key_pressed(egui::Key::F2)));
    let clicked = response.clicked() || activated(ui, &response);
    response.on_hover_text(title);
    let mut close = false;
    if many {
        let close_rect =
            egui::Rect::from_min_max(egui::pos2(rect.right() - close_width, rect.top()), rect.max);
        let close_id = ui.make_persistent_id(("circuit.strip.close", title));
        let close_response = focusable(ui, close_rect, close_id);
        let label = language.format("circuit.close_tab", &[("title", title)]);
        if close_response.hovered() || close_response.has_focus() {
            ui.painter()
                .rect_filled(close_rect, tokens.radius_control, tokens.surface_raised);
        }
        let close_ink = tokens.ink_muted;
        ui.painter().text(
            close_rect.center(),
            egui::Align2::CENTER_CENTER,
            "×",
            egui::FontId::proportional(tokens.text_body),
            close_ink,
        );
        close_response
            .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &label));
        close = close_response.clicked() || activated(ui, &close_response);
        close_response.on_hover_text(label);
    }
    (clicked, close, rename, rect)
}

/// The plus after the last tab: another circuit, as one undo.
fn add(ui: &mut egui::Ui, tokens: &Tokens, language: Language) -> bool {
    let label = language.text("circuit.new_tab");
    let side = tokens.unit * 1.6;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(side, side), egui::Sense::hover());
    let id = ui.make_persistent_id("circuit.strip.add");
    let response = focusable(ui, rect, id);
    if !ui.is_rect_visible(rect) {
        return false;
    }
    if response.hovered() || response.has_focus() {
        ui.painter()
            .rect_filled(rect, tokens.radius_control, tokens.surface_raised);
    }
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "+",
        egui::FontId::proportional(tokens.text_body),
        tokens.ink,
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
    let clicked = response.clicked() || activated(ui, &response);
    response.on_hover_text(label);
    clicked
}

/// A painted control gets the keyboard focus and the accessibility layer's name.
fn focusable(ui: &mut egui::Ui, rect: egui::Rect, id: egui::Id) -> egui::Response {
    let response = ui.interact(rect, id, egui::Sense::click());
    let layer = ui.layer_id();
    ui.memory_mut(|memory| memory.interested_in_focus(id, layer));
    response
}

/// Whether the reader activated the control with `Enter` or `Space`.
fn activated(ui: &egui::Ui, response: &egui::Response) -> bool {
    response.has_focus()
        && ui.input(|input| {
            input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Space)
        })
}

/// One drawing of the family, tinted with the given ink.
fn paint_icon(
    ui: &egui::Ui,
    drawing: (&'static str, &'static [u8]),
    rect: egui::Rect,
    ink: egui::Color32,
) {
    let (id, bytes) = drawing;
    egui::Image::from_bytes(format!("bytes://actions/{id}.svg"), bytes)
        .tint(ink)
        .paint_at(ui, rect);
}
