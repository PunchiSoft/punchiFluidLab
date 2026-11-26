// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The properties of the current selection, in a window the reader opens on request.
//!
//! The window is chrome, not a document: it follows the selection while it is open, every
//! field goes through `punchi-fluid-doc`'s own command, and closing it never enters the undo
//! history. It is non-modal, so the sheet stays usable behind it. The right dock is gone; the
//! layout no longer reserves a column for it.

use eframe::egui;
use punchi_fluid_catalog::Catalog;
use punchi_fluid_doc::{Connection, Document};
use punchi_fluid_draw::LineClass;
use punchi_fluid_i18n::Language;
use punchi_fluid_session::Model;

use crate::Window;
use crate::theme::Tokens;

mod fields;

use fields::{empty_body, multi_body, pipe_body, ports, tube_row};

/// The window's width when it opens, and the bounds a resize may take it to.
pub const DEFAULT_WIDTH: f32 = 360.0;
pub const MIN_WIDTH: f32 = 300.0;
pub const MAX_WIDTH: f32 = 520.0;

/// An edit the window asks the window to make, always through the document's own command.
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

/// The window's content: the designation being typed and the placement it belongs to, so a
/// frame does not throw away what the reader wrote.
#[derive(Debug, Clone, Default)]
struct Fields {
    buffer: String,
    buffer_for: Option<u64>,
}

/// What one frame of the window asked for.
#[derive(Debug, Default)]
pub(crate) struct Shown {
    pub(crate) action: Option<Action>,
    pub(crate) closed: bool,
}

/// The floating properties window: whether it is open, how wide it is and the fields of the
/// current selection.
#[derive(Debug, Clone)]
pub struct Inspector {
    open: bool,
    width: f32,
    fields: Fields,
    /// The language the window and its content draw with; the window sets it each frame.
    pub(crate) language: Language,
}

impl Default for Inspector {
    fn default() -> Self {
        Self {
            open: false,
            width: DEFAULT_WIDTH,
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

    /// Whether the window is open: the settings file remembers it and the tests read it back.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.open
    }

    #[must_use]
    pub fn width(&self) -> f32 {
        self.width
    }

    /// Hides the window, or brings it back at the width it had.
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    /// **Restablecer vista**: close it again at the preferred width.
    pub fn reset(&mut self) {
        self.close();
        self.width = DEFAULT_WIDTH;
        self.fields.buffer_for = None;
    }

    /// The window as a settings file left it: open or closed at the remembered width.
    pub fn restore(&mut self, open: bool, width: f32) {
        self.open = open;
        self.set_width(width);
    }

    /// The width the reader gave the window, kept inside its drag bounds.
    pub fn set_width(&mut self, width: f32) {
        if width.is_finite() {
            self.width = width.clamp(MIN_WIDTH, MAX_WIDTH);
        }
    }

    /// Draws the window if it is open and returns the edit it asked for. The caller returns
    /// the focus to the sheet when `closed` is true.
    #[expect(
        clippy::too_many_arguments,
        reason = "the window reads the document, the catalog, the model and the selection"
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
        escape: bool,
    ) -> Shown {
        if !self.open {
            return Shown::default();
        }
        let title = self.language.text("inspector.title");
        let area = context.available_rect();
        let width = self.width.min((area.width() - tokens.unit).max(MIN_WIDTH));
        let default_pos = egui::pos2(
            (area.right() - width - tokens.unit * 0.5).max(area.left() + tokens.unit),
            area.top() + tokens.unit * 0.5,
        );
        let mut open = true;
        let mut close = false;
        let mut action = None;
        let response = egui::Window::new(title)
            .id(egui::Id::new("inspector.window"))
            .open(&mut open)
            .collapsible(false)
            .resizable([true, false])
            .default_width(width)
            .min_width(MIN_WIDTH)
            .max_width(MAX_WIDTH)
            .max_height((area.height() - tokens.unit).max(tokens.unit * 8.0))
            .default_pos(default_pos)
            .constrain_to(area)
            .show(context, |ui| {
                if escape
                    && ui.input_mut(|input| {
                        input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)
                    })
                {
                    close = true;
                }
                action = self.fields.show(
                    ui,
                    tokens,
                    document,
                    catalog,
                    model,
                    selected,
                    selected_pipe,
                    self.language,
                );
            });
        if let Some(response) = response {
            self.width = response.response.rect.width().clamp(MIN_WIDTH, MAX_WIDTH);
        }
        if !open || close {
            self.close();
            return Shown {
                action,
                closed: true,
            };
        }
        Shown {
            action,
            closed: false,
        }
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
            .auto_shrink([false, true])
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
    /// Draws the properties window over the sheet. The shell calls this after the sheet, so
    /// the window is in front of the drawing but never reserves a column of it.
    pub(crate) fn show_properties(&mut self, context: &egui::Context) {
        self.inspector.language = self.language;
        let escape = !self.dialog_open()
            && !self.preferences.open
            && !self.page_setup.open
            && !self.block_editor.open
            && !self.export_options.open
            && !self.check_panel.open
            && !self.menu.is_open();
        let shown = self.inspector.show(
            context,
            &self.theme.tokens,
            &self.document,
            &self.catalog,
            self.model.as_ref(),
            &self.selected,
            self.selected_pipe,
            escape,
        );
        if let Some(action) = shown.action {
            self.apply_inspector(action);
        }
        if shown.closed {
            self.return_sheet_focus(context);
        }
    }

    /// The **Ver** commands the window owns: the catalog panel and the view reset. The zoom
    /// is dispatched directly by the window.
    pub(crate) fn view_command(&mut self, command: crate::ribbon::Command) {
        match command {
            crate::ribbon::Command::Catalog => {
                if self.layout_mode() == crate::layout::LayoutMode::Compact {
                    self.toggle_drawer(crate::shell::Drawer::Catalog);
                } else {
                    self.sidebar.toggle();
                }
            }
            crate::ribbon::Command::Properties => self.inspector.toggle(),
            crate::ribbon::Command::Autohide => self.toggle_autohide(),
            crate::ribbon::Command::ResetView => self.reset_view(),
            _ => {}
        }
    }

    /// **Ver ▸ Restablecer vista**: the catalog, the properties window and the view return to
    /// the defaults, and the answer is persisted at once so a close before the settle delay
    /// cannot undo it.
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

    /// Runs the edit the window asked for through the document's own command.
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

    /// Returns the keyboard focus to the sheet after the window closes: a visible control,
    /// and the one the reader was working on before opening it.
    pub(crate) fn return_sheet_focus(&self, context: &egui::Context) {
        if let Some(id) = self.sheet_focus {
            context.memory_mut(|memory| memory.request_focus(id));
        }
    }
}

#[cfg(test)]
mod tests;
