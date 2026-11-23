// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The right dock: the properties of the current selection.
//!
//! The panel itself never edits the document; every field goes through `punchi-fluid-doc`'s
//! own command, so the undo history keeps working and hiding the panel never enters it. The
//! fold, the header and the rail are the shared chrome in [`crate::panel`]; the dock keeps its
//! content, its width bounds and its own open fields.

use eframe::egui;
use punchi_fluid_catalog::Catalog;
use punchi_fluid_doc::{Connection, Document};
use punchi_fluid_draw::LineClass;
use punchi_fluid_session::Model;

use crate::Window;
use crate::panel::{Edge, Panel, Settings};
use crate::theme::Tokens;

mod fields;

use fields::{empty_body, multi_body, pipe_body, ports, tube_row};

/// The dock's width when it is open, and the bounds a drag may take it to. The preferred
/// width is the one `docs/ui-ux.md` gives the right panels.
pub const DEFAULT_WIDTH: f32 = 360.0;
const MIN_WIDTH: f32 = 300.0;
const MAX_WIDTH: f32 = 520.0;

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

/// The dock's content: the designation being typed and the placement it belongs to, so a
/// frame does not throw away what the reader wrote.
#[derive(Debug, Clone, Default)]
struct Fields {
    buffer: String,
    buffer_for: Option<u64>,
}

/// The dock: the shared chrome and the fields of the current selection.
#[derive(Debug, Clone)]
pub struct Inspector {
    chrome: Panel,
    fields: Fields,
}

impl Default for Inspector {
    fn default() -> Self {
        Self {
            chrome: Panel::new(Settings {
                edge: Edge::Right,
                id: "inspector",
                default_width: DEFAULT_WIDTH,
                min_width: MIN_WIDTH,
                max_width: MAX_WIDTH,
                width_label: "Ancho del panel de propiedades",
                show_label: "Mostrar el panel de propiedades",
                hide_label: "Ocultar el panel de propiedades",
                shortcut: "Ctrl+J",
                margin: 5,
            }),
            fields: Fields::default(),
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
        self.chrome.is_open()
    }

    #[cfg(test)]
    #[must_use]
    pub fn width(&self) -> f32 {
        self.chrome.width()
    }

    #[cfg(test)]
    #[must_use]
    pub fn shade(&self) -> f32 {
        self.chrome.shade()
    }

    /// Hides the dock, or brings it back at the width it had.
    pub fn toggle(&mut self) {
        self.chrome.toggle();
    }

    #[cfg(test)]
    pub fn open(&mut self) {
        self.chrome.open();
    }

    #[cfg(test)]
    pub fn close(&mut self) {
        self.chrome.close();
    }

    /// **Restablecer vista**: open again at the preferred width.
    pub fn reset(&mut self) {
        self.chrome.reset();
        self.fields.buffer_for = None;
    }

    #[cfg(test)]
    pub fn resize_by(&mut self, delta: f32) {
        self.chrome.resize_by(delta);
    }

    /// Draws the dock and returns the edit it asked for, if any.
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
        let title = title_of(document, catalog, selected, selected_pipe);
        let Inspector { chrome, fields } = self;
        chrome
            .show(context, tokens, &title, None, |ui| {
                fields.show(
                    ui,
                    tokens,
                    document,
                    catalog,
                    model,
                    selected,
                    selected_pipe,
                )
            })
            .flatten()
    }
}

impl Fields {
    /// The content of the current selection, inside the panel's scroll area.
    #[expect(
        clippy::too_many_arguments,
        reason = "the body reads the document, the catalog, the model and the selection"
    )]
    fn show(
        &mut self,
        ui: &mut egui::Ui,
        tokens: &Tokens,
        document: &Document,
        catalog: &Catalog,
        model: Option<&Model>,
        selected: &[usize],
        selected_pipe: Option<usize>,
    ) -> Option<Action> {
        let mut action = None;
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
        action
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

    /// The **Ver** commands the window owns: the zoom and pan of the view, and both panels.
    pub(crate) fn view_command(&mut self, command: &str) {
        self.view.command(command);
        match command {
            "Catálogo" => self.sidebar.toggle(),
            "Propiedades" => self.inspector.toggle(),
            "Restablecer vista" => {
                self.view.reset();
                self.sidebar.reset();
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

#[cfg(test)]
mod tests;
