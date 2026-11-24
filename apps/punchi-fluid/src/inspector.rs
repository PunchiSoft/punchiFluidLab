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
use punchi_fluid_i18n::Language;
use punchi_fluid_session::Model;

use crate::Window;
use crate::panel::{Edge, Panel, Probe, Settings};
use crate::theme::Tokens;

mod fields;

use fields::{empty_body, multi_body, pipe_body, ports, tube_row};

/// The dock's width when it is open, and the bounds a drag may take it to. The preferred
/// width and the floor are the interface constants `docs/ui-ux.md` §2.2 fixes for the right
/// dock; the maximum is the drag bound, not a layout measurement.
pub const DEFAULT_WIDTH: f32 = crate::layout::DOCK_PREFERRED_WIDTH;
const MIN_WIDTH: f32 = crate::layout::DOCK_MIN_WIDTH;
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
    /// The language the panel and its content draw with; the window sets it each frame.
    pub(crate) language: Language,
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
                width_label: "panel.inspector.width",
                show_label: "panel.inspector.show",
                hide_label: "panel.inspector.hide",
                shortcut: "Ctrl+J",
                margin: 5,
            }),
            fields: Fields::default(),
            language: Language::English,
        }
    }
}

impl Inspector {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the panel is open: the window reads it into the settings file and the tests
    /// read it back.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.chrome.is_open()
    }

    #[must_use]
    pub fn width(&self) -> f32 {
        self.chrome.width()
    }

    /// The width the panel takes when it floats over the sheet in the compact mode.
    #[must_use]
    pub fn floating_width(&self) -> f32 {
        self.chrome.floating_width()
    }

    /// The panel frame's horizontal inner margin, so the drawer's content lines up with the
    /// docked one.
    #[must_use]
    pub fn margin(&self) -> i8 {
        self.chrome.margin()
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

    /// Opens the panel: the docked one in the wide and medium modes, or its drawer.
    pub fn open(&mut self) {
        self.chrome.open();
    }

    /// Closes the panel, whether it was docked or floated.
    pub fn close(&mut self) {
        self.chrome.close();
    }

    /// **Restablecer vista**: open again at the preferred width.
    pub fn reset(&mut self) {
        self.chrome.reset();
        self.fields.buffer_for = None;
    }

    /// The panel as a settings file left it: open or folded at the remembered width.
    pub fn restore(&mut self, open: bool, width: f32) {
        self.chrome.restore(open, width);
    }

    /// The width the layout mode allows, clamped to the panel's own drag bounds.
    pub fn set_width(&mut self, width: f32) {
        self.chrome.set_width(width);
    }

    /// What the last frame told the auto-hide timers: the shell reads it after the panel drew.
    #[must_use]
    pub fn probe(&self) -> Probe {
        self.chrome.probe()
    }

    /// Records the drawer's surface, which the shell draws itself in the compact mode.
    pub fn observe_surface(&mut self, context: &eframe::egui::Context, rect: eframe::egui::Rect) {
        self.chrome.observe_surface(context, rect);
    }

    #[cfg(test)]
    pub fn probe_for_test(&mut self, rail_hovered: bool, pointer_inside: bool, focus_inside: bool) {
        self.chrome
            .probe_for_test(rail_hovered, pointer_inside, focus_inside);
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
        docked_width: f32,
    ) -> Option<Action> {
        let title = title_of(document, catalog, selected, selected_pipe, self.language);
        let Inspector {
            chrome,
            fields,
            language,
        } = self;
        chrome.set_language(*language);
        chrome
            .show(context, tokens, &title, None, docked_width, |ui| {
                fields.show(
                    ui,
                    tokens,
                    document,
                    catalog,
                    model,
                    selected,
                    selected_pipe,
                    *language,
                )
            })
            .flatten()
    }

    /// Draws only the rail, for the compact shell.
    pub fn show_rail(&mut self, context: &egui::Context, tokens: &Tokens) -> bool {
        self.chrome.set_language(self.language);
        self.chrome.show_rail(context, tokens)
    }

    /// Draws the dock's content in the floating drawer. Reports the edit it asked for and
    /// whether the hide control asked to close the panel.
    #[expect(
        clippy::too_many_arguments,
        reason = "the drawer reads the document, the catalog, the model and the selection"
    )]
    pub fn show_drawer(
        &mut self,
        ui: &mut egui::Ui,
        tokens: &Tokens,
        document: &Document,
        catalog: &Catalog,
        model: Option<&Model>,
        selected: &[usize],
        selected_pipe: Option<usize>,
    ) -> (Option<Action>, bool) {
        let title = title_of(document, catalog, selected, selected_pipe, self.language);
        let Inspector {
            chrome,
            fields,
            language,
        } = self;
        chrome.set_language(*language);
        chrome.show_body(ui, tokens, &title, None, |ui| {
            fields.show(
                ui,
                tokens,
                document,
                catalog,
                model,
                selected,
                selected_pipe,
                *language,
            )
        })
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
        language: Language,
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
                    language,
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
        language: Language,
    ) -> Option<Action> {
        if let Some(index) = selected_pipe {
            return pipe_body(ui, tokens, document, index, language);
        }
        match *selected {
            [index] => self.component_body(ui, tokens, document, catalog, model, index, language),
            [] => {
                empty_body(ui, tokens, document, language);
                None
            }
            _ => {
                multi_body(ui, tokens, selected.len(), language);
                None
            }
        }
    }

    /// One component: its name and catalog id, its designation and, when the selection
    /// reaches exactly one hose, that hose's tube; then its ports and their readings.
    #[expect(
        clippy::too_many_arguments,
        reason = "the body reads the document, the catalog, the model, the selection and the language"
    )]
    fn component_body(
        &mut self,
        ui: &mut egui::Ui,
        tokens: &Tokens,
        document: &Document,
        catalog: &Catalog,
        model: Option<&Model>,
        index: usize,
        language: Language,
    ) -> Option<Action> {
        let placement = document.placements().get(index)?;
        let definition = catalog.get(&placement.catalog_id);
        let mut action = None;
        ui.add_space(tokens.unit * 0.35);
        match definition {
            Some(definition) => {
                let name = match language {
                    Language::Spanish => definition.name_es.as_deref().unwrap_or(&definition.name),
                    Language::English => &definition.name,
                };
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
            egui::RichText::new(language.text("inspector.designation"))
                .size(tokens.text_small)
                .color(tokens.ink_muted),
        );
        if self.buffer_for != Some(placement.id()) {
            self.buffer = placement.designation.clone().unwrap_or_default();
            self.buffer_for = Some(placement.id());
        }
        let field = ui.add(
            egui::TextEdit::singleline(&mut self.buffer)
                .hint_text(language.text("inspector.no_designation"))
                .desired_width(f32::INFINITY),
        );
        field.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::TextEdit,
                true,
                language.text("inspector.designation"),
            )
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
                if let Some(chosen) = tube_row(ui, *hose, tube, language) {
                    action = Some(chosen);
                }
            }
            many if many.len() > 1 => {
                ui.add_space(tokens.unit * 0.35);
                ui.label(
                    egui::RichText::new(language.format(
                        "inspector.hoses_multi",
                        &[("count", &many.len().to_string())],
                    ))
                    .size(tokens.text_small)
                    .color(tokens.ink_muted),
                );
            }
            _ => {}
        }

        if let Some(definition) = definition {
            ports(ui, tokens, model, index, &definition.port, language);
        }
        action
    }
}

impl Window {
    /// Draws the right dock between the ribbon and the status bar. The shell calls this after
    /// the status panel and before the sheet, so the sheet's available area follows it; the
    /// dock computes its docked width from the same mode the shell derived.
    pub(crate) fn show_inspector(&mut self, context: &egui::Context) {
        self.inspector.language = self.language;
        let mode = crate::layout::LayoutMode::from_available(context.screen_rect().width());
        if mode == crate::layout::LayoutMode::Compact {
            return;
        }
        let docked = mode
            .widths(
                context.screen_rect().width(),
                self.sidebar.width(),
                self.inspector.width(),
            )
            .dock;
        let action = self.inspector.show(
            context,
            &self.theme.tokens,
            &self.document,
            &self.catalog,
            self.model.as_ref(),
            &self.selected,
            self.selected_pipe,
            docked,
        );
        if let Some(action) = action {
            self.apply_inspector(action);
        }
    }

    /// The **Ver** commands the window owns: the two panels and the view reset. The zoom is
    /// dispatched directly by the window.
    pub(crate) fn view_command(&mut self, command: crate::ribbon::Command) {
        match command {
            crate::ribbon::Command::Catalog => {
                if self.layout_mode() == crate::layout::LayoutMode::Compact {
                    self.toggle_drawer(crate::shell::Drawer::Catalog);
                } else {
                    self.sidebar.toggle();
                }
            }
            crate::ribbon::Command::Properties => {
                if self.layout_mode() == crate::layout::LayoutMode::Compact {
                    self.toggle_drawer(crate::shell::Drawer::Inspector);
                } else {
                    self.inspector.toggle();
                }
            }
            crate::ribbon::Command::Autohide => self.toggle_autohide(),
            crate::ribbon::Command::ResetView => self.reset_view(),
            _ => {}
        }
    }

    /// **Ver ▸ Restablecer vista**: the panels and the view return to the defaults, and the
    /// answer is persisted at once so a close before the settle delay cannot undo it.
    pub(crate) fn reset_view(&mut self) {
        self.view.reset();
        self.sidebar.reset();
        self.inspector.reset();
        self.close_drawer();
        self.autohide_on = false;
        self.auto_hide.cancel_all();
        let settings = self.panel_settings();
        self.settings.remember(settings);
    }

    /// Runs the edit the panel asked for through the document's own command.
    pub(crate) fn apply_inspector(&mut self, action: Action) {
        let changed = match action {
            Action::Designation { index, text } => self.document.designate(index, Some(&text)),
            Action::PipeClass { index, class } => self.document.set_pipe_class(index, class),
            Action::PipeTube { index, tube } => self.document.set_pipe_tube(index, tube),
        };
        if changed {
            self.notice = Some(self.language.text("inspector.updated").to_owned());
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
    language: Language,
) -> String {
    if let Some(index) = selected_pipe
        && document.connections().get(index).is_some()
    {
        return language.text("inspector.pipe").to_owned();
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
            return match language {
                Language::Spanish => definition
                    .name_es
                    .clone()
                    .unwrap_or_else(|| definition.name.clone()),
                Language::English => definition.name.clone(),
            };
        }
        return placement.catalog_id.clone();
    }
    language.text("inspector.title").to_owned()
}

#[cfg(test)]
mod tests;
