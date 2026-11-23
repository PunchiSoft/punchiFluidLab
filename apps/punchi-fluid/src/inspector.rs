// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The right dock: the properties of the current selection.
//!
//! A panel is chrome, never a document: hiding it, widening it and `Ctrl+J` are view state
//! the window remembers for the session, and **Ver ▸ Restablecer vista** puts it back. The
//! panel itself never edits the document; every field goes through `punchi-fluid-doc`'s own
//! command, so the undo history keeps working and hiding the panel never enters it.
//!
//! The fold lasts the theme's `motion_ms` and is instant when the theme reports reduced
//! motion, which it does by carrying a zero duration.

use eframe::egui;
use punchi_fluid_catalog::Catalog;
use punchi_fluid_doc::{Connection, Document};
use punchi_fluid_draw::LineClass;
use punchi_fluid_session::Model;

use crate::Window;
use crate::theme::Tokens;

mod fields;

use fields::{empty_body, multi_body, pipe_body, ports, tube_row};

/// The dock's width when it is open, and the bounds a drag may take it to. The preferred
/// width is the one `docs/ui-ux.md` gives the right panels.
pub const DEFAULT_WIDTH: f32 = 360.0;
const MIN_WIDTH: f32 = 300.0;
const MAX_WIDTH: f32 = 520.0;
/// The rail is one unit wide, never below the 24 px the interface specification gives it as
/// a target.
const MIN_RAIL: f32 = 24.0;

/// An edit the panel asks the window to make, always through the document's own command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// The designation field was committed; the text is trimmed by the document and an
    /// empty one clears the name.
    Designation { index: usize, text: String },
    /// The selected pipe took another line class.
    PipeClass { index: usize, class: LineClass },
    /// The selected pipe took another nominal outside diameter, or none.
    PipeTube { index: usize, tube: Option<u16> },
}

/// The dock's own state: whether it is open, how wide it is and the field being typed.
#[derive(Debug, Clone)]
pub struct Inspector {
    open: bool,
    width: f32,
    /// How far the fold has gone, `0` closed and `1` open, kept so the tests can see the
    /// animation without a screenshot.
    shade: f32,
    /// The designation being typed, and the placement it belongs to, so a frame does not
    /// throw away what the reader wrote.
    buffer: String,
    buffer_for: Option<u64>,
}

impl Default for Inspector {
    fn default() -> Self {
        Self {
            open: true,
            width: DEFAULT_WIDTH,
            shade: 1.0,
            buffer: String::new(),
            buffer_for: None,
        }
    }
}

impl Inspector {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The dock's state, read by the tests: the window itself only tells the panel to fold.
    #[cfg(test)]
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.open
    }

    #[cfg(test)]
    #[must_use]
    pub fn width(&self) -> f32 {
        self.width
    }

    #[cfg(test)]
    #[must_use]
    pub fn shade(&self) -> f32 {
        self.shade
    }

    /// Hides the dock, or brings it back at the width it had.
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    /// **Restablecer vista**: open again at the preferred width.
    pub fn reset(&mut self) {
        self.open = true;
        self.width = DEFAULT_WIDTH;
        self.buffer_for = None;
    }

    /// A width drag; clamped so the dock never takes the sheet's floor.
    pub fn resize_by(&mut self, delta: f32) {
        self.width = (self.width + delta).clamp(MIN_WIDTH, MAX_WIDTH);
    }

    /// Draws the panel (or its rail) and returns the edit it asked for, if any.
    ///
    /// The panel is always one `SidePanel`: its width is the fold's interpolation, so the
    /// sheet takes the room in the same gesture instead of after it.
    #[expect(
        clippy::too_many_arguments,
        reason = "the panel reads the document, the catalog, the model and the selection"
    )]
    pub fn show(
        &mut self,
        context: &egui::Context,
        tokens: &Tokens,
        document: &Document,
        catalog: &Catalog,
        model: Option<&Model>,
        selected: &[usize],
        selected_pipe: Option<usize>,
    ) -> Option<Action> {
        let rail = tokens.unit.max(MIN_RAIL);
        #[allow(
            clippy::cast_precision_loss,
            reason = "a motion token in milliseconds is far below the f32 integer range"
        )]
        let motion = tokens.motion_ms as f32 / 1_000.0;
        let id = egui::Id::new("inspector.fold");
        self.shade = context.animate_bool_with_time(id, self.open, motion);
        let width = rail + (self.width - rail) * self.shade;
        let title = title_of(document, catalog, selected, selected_pipe);
        let mut action = None;
        egui::SidePanel::right("inspector")
            .exact_width(width)
            .resizable(false)
            .frame(
                egui::Frame::default()
                    .fill(tokens.panel)
                    .inner_margin(egui::Margin::symmetric(5, 0)),
            )
            .show(context, |ui| {
                if self.shade <= 0.001 && !self.open {
                    self.rail(ui, tokens);
                    return;
                }
                self.header(ui, tokens, &title);
                if self.shade < 0.999 {
                    return;
                }
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        action = self.body(
                            ui,
                            tokens,
                            document,
                            catalog,
                            model,
                            selected,
                            selected_pipe,
                        );
                    });
            });
        action
    }

    /// The rail the closed dock leaves at the edge: one unit wide, and a button that brings
    /// the panel back. The width is the panel's, so the control is a target not a sliver.
    fn rail(&mut self, ui: &mut egui::Ui, tokens: &Tokens) {
        ui.add_space(tokens.unit * 0.5);
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), (tokens.unit * 1.6).max(30.0)),
            egui::Sense::hover(),
        );
        let response = focusable(ui, rect, ui.make_persistent_id("inspector.rail"));
        if ui.is_rect_visible(rect) {
            if response.hovered() || response.has_focus() {
                ui.painter()
                    .rect_filled(rect, tokens.radius_small, tokens.accent_soft);
            }
            draw_panel_glyph(ui, rect, tokens.ink, tokens.accent);
        }
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::Button,
                true,
                "Mostrar el panel de propiedades",
            )
        });
        if activated(ui, &response) {
            self.open();
        }
        response.on_hover_text("Mostrar el panel de propiedades (Ctrl+J)");
    }

    /// The header: the title and the control that hides the panel.
    fn header(&mut self, ui: &mut egui::Ui, tokens: &Tokens, title: &str) {
        ui.add_space(tokens.unit * 0.35);
        let mut hide = false;
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(9.0, (tokens.unit * 1.4).max(24.0)),
                egui::Sense::hover(),
            );
            let grip = focusable(ui, rect, ui.make_persistent_id("inspector.grip"));
            grip.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Other,
                    true,
                    "Ancho del panel de propiedades",
                )
            });
            if grip.hovered() || grip.dragged() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
            }
            if grip.dragged() {
                self.resize_by(-grip.drag_delta().x);
            }
            grip.on_hover_text("Arrastra para cambiar el ancho del panel");
            ui.label(
                egui::RichText::new(title)
                    .size(tokens.text_label)
                    .strong()
                    .color(tokens.ink),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(tokens.unit, tokens.unit),
                    egui::Sense::hover(),
                );
                let button = focusable(ui, rect, ui.make_persistent_id("inspector.hide"));
                if ui.is_rect_visible(rect) {
                    if button.hovered() || button.has_focus() {
                        ui.painter()
                            .rect_filled(rect, tokens.radius_small, tokens.accent_soft);
                    }
                    draw_panel_glyph(ui, rect, tokens.ink, tokens.accent);
                }
                button.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::WidgetType::Button,
                        true,
                        "Ocultar el panel de propiedades",
                    )
                });
                hide = activated(ui, &button);
                button.on_hover_text("Ocultar el panel de propiedades (Ctrl+J)");
            });
        });
        if hide {
            self.close();
        }
        ui.add_space(tokens.unit * 0.25);
        let line = ui.max_rect();
        ui.painter().line_segment(
            [
                egui::pos2(line.left(), ui.cursor().top()),
                egui::pos2(line.right(), ui.cursor().top()),
            ],
            egui::Stroke::new(1.0_f32, tokens.line),
        );
        ui.add_space(tokens.unit * 0.25);
    }

    /// The content of the current selection.
    #[expect(
        clippy::too_many_arguments,
        reason = "the panel reads the document, the catalog, the model and the selection"
    )]
    fn body(
        &mut self,
        ui: &mut egui::Ui,
        tokens: &Tokens,
        document: &Document,
        catalog: &Catalog,
        model: Option<&Model>,
        selected: &[usize],
        selected_pipe: Option<usize>,
    ) -> Option<Action> {
        if let Some(index) = selected_pipe {
            return pipe_body(ui, tokens, document, index);
        }
        match *selected {
            [index] => self.component_body(ui, tokens, document, catalog, model, index),
            [] => {
                empty_body(ui, tokens, document);
                None
            }
            _ => {
                multi_body(ui, tokens, selected.len());
                None
            }
        }
    }

    /// One component: its name and catalog id, its designation and, when the selection
    /// reaches exactly one hose, that hose's tube; then its ports and their readings.
    fn component_body(
        &mut self,
        ui: &mut egui::Ui,
        tokens: &Tokens,
        document: &Document,
        catalog: &Catalog,
        model: Option<&Model>,
        index: usize,
    ) -> Option<Action> {
        let placement = document.placements().get(index)?;
        let definition = catalog.get(&placement.catalog_id);
        let mut action = None;
        ui.add_space(tokens.unit * 0.35);
        match definition {
            Some(definition) => {
                let name = definition.name_es.as_deref().unwrap_or(&definition.name);
                ui.label(
                    egui::RichText::new(name)
                        .size(tokens.text_body)
                        .strong()
                        .color(tokens.ink),
                );
                ui.label(
                    egui::RichText::new(&definition.id)
                        .size(tokens.text_small)
                        .color(tokens.ink_muted),
                );
            }
            None => {
                ui.label(
                    egui::RichText::new(&placement.catalog_id)
                        .size(tokens.text_body)
                        .color(tokens.ink),
                );
            }
        }

        // The designation is text, not a number: the field sends it verbatim and the
        // document trims and bounds it, records one undo per committed edit and clears the
        // name when the text is emptied.
        ui.add_space(tokens.unit * 0.5);
        ui.label(
            egui::RichText::new("Designación")
                .size(tokens.text_small)
                .color(tokens.ink_muted),
        );
        if self.buffer_for != Some(placement.id()) {
            self.buffer = placement.designation.clone().unwrap_or_default();
            self.buffer_for = Some(placement.id());
        }
        let field = ui.add(
            egui::TextEdit::singleline(&mut self.buffer)
                .hint_text("Sin designación")
                .desired_width(f32::INFINITY),
        );
        field.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, true, "Designación")
        });
        if field.lost_focus() {
            action = Some(Action::Designation {
                index,
                text: self.buffer.clone(),
            });
            // Re-read the field from the document next frame, so the trimmed value is the
            // one on screen.
            self.buffer_for = None;
        }

        // The tube of the component's hose, when the selection reaches exactly one: a
        // component with two hoses has no single diameter to show here.
        let hoses: Vec<usize> = document
            .connections()
            .iter()
            .enumerate()
            .filter(|(_, connection)| {
                connection.circuit == placement.circuit && connection.touches(placement.id())
            })
            .map(|(index, _)| index)
            .collect();
        match hoses.as_slice() {
            [hose] => {
                ui.add_space(tokens.unit * 0.35);
                let tube = document
                    .connections()
                    .get(*hose)
                    .and_then(Connection::tube_od_hundredths);
                if let Some(chosen) = tube_row(ui, *hose, tube) {
                    action = Some(chosen);
                }
            }
            many if many.len() > 1 => {
                ui.add_space(tokens.unit * 0.35);
                ui.label(
                    egui::RichText::new(format!(
                        "{} conductos: selecciona uno en la hoja para su diámetro.",
                        many.len()
                    ))
                    .size(tokens.text_small)
                    .color(tokens.ink_muted),
                );
            }
            _ => {}
        }

        if let Some(definition) = definition {
            ports(ui, tokens, model, index, &definition.port);
        }
        action
    }
}

impl Window {
    /// Draws the right dock between the ribbon and the status bar. `update` calls this after
    /// the status panel and before the sheet, so the sheet's available area follows it.
    pub(crate) fn show_inspector(&mut self, context: &egui::Context) {
        let action = self.inspector.show(
            context,
            &self.theme.tokens,
            &self.document,
            &self.catalog,
            self.model.as_ref(),
            &self.selected,
            self.selected_pipe,
        );
        if let Some(action) = action {
            self.apply_inspector(action);
        }
    }

    /// The keyboard path of the dock, besides **Ver ▸ Propiedades**.
    pub(crate) fn inspector_keyboard(&mut self, context: &egui::Context) {
        if context.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::J)) {
            self.inspector.toggle();
        }
    }

    /// The **Ver** commands the window owns: the zoom and pan of the view, and the panel.
    pub(crate) fn view_command(&mut self, command: &str) {
        self.view.command(command);
        match command {
            "Propiedades" => self.inspector.toggle(),
            "Restablecer vista" => {
                self.view.reset();
                self.inspector.reset();
            }
            _ => {}
        }
    }

    /// Runs the edit the panel asked for through the document's own command.
    fn apply_inspector(&mut self, action: Action) {
        let changed = match action {
            Action::Designation { index, text } => self.document.designate(index, Some(&text)),
            Action::PipeClass { index, class } => self.document.set_pipe_class(index, class),
            Action::PipeTube { index, tube } => self.document.set_pipe_tube(index, tube),
        };
        if changed {
            self.notice = Some(String::from("propiedades actualizadas"));
        }
    }
}

/// The panel's title: the selected component's name, `Conducto` for a hose, `Propiedades`
/// otherwise.
fn title_of(
    document: &Document,
    catalog: &Catalog,
    selected: &[usize],
    selected_pipe: Option<usize>,
) -> String {
    if let Some(index) = selected_pipe
        && document.connections().get(index).is_some()
    {
        return String::from("Conducto");
    }
    if let [index] = *selected
        && let Some(placement) = document.placements().get(index)
    {
        if let Some(designation) = placement.designation.as_deref()
            && !designation.trim().is_empty()
        {
            return designation.to_owned();
        }
        if let Some(definition) = catalog.get(&placement.catalog_id) {
            return definition
                .name_es
                .clone()
                .unwrap_or_else(|| definition.name.clone());
        }
        return placement.catalog_id.clone();
    }
    String::from("Propiedades")
}

/// The project's panel glyph: two rectangles with the side that folds marked. The desktop
/// icon themes have no panel glyph, and a label alone is not a control.
fn draw_panel_glyph(ui: &egui::Ui, rect: egui::Rect, ink: egui::Color32, accent: egui::Color32) {
    // Every measure is a fraction of the control, so the glyph follows the theme's unit
    // instead of carrying a size of its own.
    let size = rect.size().min_elem() * 0.72;
    let body = egui::Rect::from_center_size(rect.center(), egui::vec2(size, size * 0.8));
    ui.painter().rect_stroke(
        body,
        size * 0.14,
        egui::Stroke::new(1.0_f32, ink),
        egui::StrokeKind::Inside,
    );
    let folded = egui::Rect::from_min_max(
        egui::pos2(body.left() + size * 0.12, body.top() + size * 0.14),
        egui::pos2(body.left() + size * 0.32, body.bottom() - size * 0.14),
    );
    ui.painter().rect_filled(folded, size * 0.06, accent);
}

/// A painted control gets the keyboard focus: `Tab` reaches it and the accessibility layer
/// finds it by its name.
fn focusable(ui: &mut egui::Ui, rect: egui::Rect, id: egui::Id) -> egui::Response {
    let response = ui.interact(rect, id, egui::Sense::click_and_drag());
    let layer = ui.layer_id();
    ui.memory_mut(|memory| memory.interested_in_focus(id, layer));
    response
}

/// Whether the reader activated the control with the pointer or with `Enter`/`Space` while
/// it has the keyboard.
fn activated(ui: &egui::Ui, response: &egui::Response) -> bool {
    response.clicked()
        || (response.has_focus()
            && ui.input(|input| {
                input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Space)
            }))
}

#[cfg(test)]
mod tests;
