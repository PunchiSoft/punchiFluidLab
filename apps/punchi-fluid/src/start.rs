// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Inicio page: the start rail, the two ways in, the template gallery and the recent
//! list placeholder.
//!
//! The page is a whole-window screen: while it is open the ribbon, the status bar and the
//! catalog are hidden, and the start rail takes the catalog's column, as the interface
//! specification asks (`docs/ui-ux.md` §2.3). It is not a fourth region and not a document.
//!
//! The templates are composed in `punchi-fluid-session` and shared with the ABI, so the page
//! offers the same documents the old window offered; `thumbnail` draws each one with the
//! sheet's own preview path. Choosing a card loads that document and closes the page.
//!
//! The page is split over this module and its children so every file keeps the project's
//! 800-line budget: `nav` is the rail, `cards` is the page's choices and `thumbnail` is the
//! rendered document.

use std::path::PathBuf;

use eframe::egui;
use punchi_fluid_catalog::Catalog;
use punchi_fluid_doc::Document;
use punchi_fluid_draw::Rect as SheetRect;
use punchi_fluid_session::recents::Recents;
use punchi_fluid_session::templates;

use crate::Window;
use crate::theme::Tokens;

mod cards;
mod nav;
mod thumbnail;

#[cfg(test)]
mod tests;

/// What the page asks the window to do after the frame is drawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Action {
    /// Load the start template with this index into a clean window.
    Template(u32),
    /// Close Inicio and show the sheet.
    Sheet,
    /// Start an empty project on the sheet.
    NewProject,
    /// Open the file dialog to read a document.
    OpenDialog,
    /// Read this remembered document and close Inicio.
    OpenRecent(PathBuf),
    /// Drop this remembered document from the list; the file is not touched.
    ForgetRecent(String),
    /// Read the safety copy found at start and install it, with no path.
    Recover,
    /// Throw the safety copy found at start away.
    DiscardRecovery,
    /// Open the «Acerca de» panel of the rail.
    About,
    /// Open the «Licencia» panel of the rail.
    Licence,
}

/// One drawing of the ribbon's action family: its id and its embedded bytes.
type Drawing = (&'static str, &'static [u8]);

// The ribbon owns the action family's table (`ribbon::icons`); the start page embeds the
// drawings it uses so the ribbon and its tests stay untouched. ADR 0035 keeps the family in
// `assets/icons/actions`, so both tables read the same files.
const ICON_HOME: Drawing = (
    "go-home",
    include_bytes!("../../../assets/icons/actions/go-home.svg"),
);
const ICON_NEW: Drawing = (
    "document-new",
    include_bytes!("../../../assets/icons/actions/document-new.svg"),
);
const ICON_OPEN: Drawing = (
    "document-open",
    include_bytes!("../../../assets/icons/actions/document-open.svg"),
);
const ICON_BACK: Drawing = (
    "view-restore",
    include_bytes!("../../../assets/icons/actions/view-restore.svg"),
);
const ICON_ABOUT: Drawing = (
    "x-office-address-book",
    include_bytes!("../../../assets/icons/actions/x-office-address-book.svg"),
);
const ICON_LICENSE: Drawing = (
    "x-office-document",
    include_bytes!("../../../assets/icons/actions/x-office-document.svg"),
);
const ICON_RECENT: Drawing = (
    "document-open-recent",
    include_bytes!("../../../assets/icons/actions/document-open-recent.svg"),
);

/// Builds the document of a start template, when the catalog provides it.
fn document(index: u32, catalog: &Catalog, area: SheetRect) -> Option<Document> {
    templates::document(index, catalog, area).ok()
}

impl Window {
    /// Draws the Inicio screen while it is open. The page is the whole window: the ribbon,
    /// the status bar and the catalog are not drawn, and the start rail takes the catalog's
    /// column. Choosing a card or `Volver a la hoja` closes it.
    pub(crate) fn show_start(&mut self, context: &egui::Context) {
        if !self.start_open {
            return;
        }
        let tokens = &self.theme.tokens;
        let show_recovery = self.has_recovery();
        let document_name = self
            .path
            .as_deref()
            .and_then(std::path::Path::file_name)
            .and_then(std::ffi::OsStr::to_str);
        let mut action = None;
        egui::SidePanel::left("start-nav")
            .exact_width(crate::sidebar::DEFAULT_WIDTH)
            .resizable(false)
            .frame(egui::Frame::default().fill(tokens.background))
            .show(context, |ui| {
                action = nav::nav(ui, tokens);
            });
        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(tokens.background)
                    .inner_margin(tokens.unit * 1.4),
            )
            .show(context, |ui| {
                if action.is_none() {
                    action = page(
                        ui,
                        tokens,
                        &self.catalog,
                        &self.recents,
                        show_recovery,
                        document_name,
                    );
                }
            });
        // The page takes `Esc` before anything behind it, as the interface specification
        // says; with no document to return to, the sheet is the empty one the window opened.
        // While a dialog is open it owns `Esc`, and the page leaves the key alone.
        if action.is_none()
            && self.dialog.is_none()
            && context
                .input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            action = Some(Action::Sheet);
        }
        if let Some(action) = action {
            self.apply(action);
        }
    }

    /// Runs what the page asked for.
    pub(crate) fn apply(&mut self, action: Action) {
        match action {
            Action::Template(index) => {
                if let Some(document) = document(index, &self.catalog, self.page.rect()) {
                    self.reset_to(document, None, None);
                }
            }
            Action::NewProject => self.new_document(),
            Action::OpenDialog => self.open_dialog(crate::files::Mode::Open),
            Action::OpenRecent(path) => {
                if let Err(cause) = self.open_path(&path) {
                    self.notice = Some(format!("no se pudo abrir el documento: {cause}"));
                }
            }
            Action::ForgetRecent(path) => self.forget_recent(&path),
            Action::Recover => self.recover(),
            Action::DiscardRecovery => self.discard_recovery(),
            Action::About => self.open_about(),
            Action::Licence => self.open_licence(),
            Action::Sheet => self.start_open = false,
        }
    }

    /// Puts a document on a fresh sheet, keeping the window's theme, its recent list and its
    /// life outside the document: a template or a new project starts a new document, not a
    /// new window.
    fn reset_to(&mut self, document: Document, path: Option<PathBuf>, notice: Option<String>) {
        self.replace_document(document, path);
        self.notice = notice;
        self.mark_saved();
    }
}

/// Draws the page in reading order: the title, the recovery offer, `Empezar`, `Plantillas`
/// and `Recientes`.
fn page(
    ui: &mut egui::Ui,
    tokens: &Tokens,
    catalog: &Catalog,
    recents: &Recents,
    show_recovery: bool,
    document_name: Option<&str>,
) -> Option<Action> {
    let mut action = None;
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(tokens.unit * 0.5);
            ui.label(
                egui::RichText::new("Inicio")
                    .size(tokens.text_display)
                    .strong()
                    .color(tokens.ink),
            );
            ui.label(
                egui::RichText::new(
                    "Crea un proyecto nuevo o continúa trabajando en uno existente",
                )
                .size(tokens.text_body)
                .color(tokens.ink_dim),
            );
            ui.add_space(tokens.unit * 1.2);

            if show_recovery
                && let Some(recovery) = cards::recovery_banner(ui, tokens, document_name)
            {
                action = Some(recovery);
            }
            ui.add_space(tokens.unit * 1.2);

            section(ui, tokens, "Empezar", None);
            if action.is_none() {
                action = cards::begin_row(ui, tokens);
            }
            ui.add_space(tokens.unit * 1.2);

            section(
                ui,
                tokens,
                "Plantillas",
                Some("Elige un punto de partida; podrás editarlo desde la hoja."),
            );
            if let Some(index) = cards::template_grid(ui, tokens, catalog) {
                action = Some(Action::Template(index));
            }
            ui.add_space(tokens.unit * 1.2);

            section(
                ui,
                tokens,
                "Recientes",
                Some("Los proyectos que abras o guardes aparecerán aquí."),
            );
            if let Some(recent) = cards::recent_rows(ui, tokens, recents) {
                action = Some(recent);
            }
            ui.add_space(tokens.unit);
        });
    action
}

/// A section label with its one-line explanation, when it has one.
fn section(ui: &mut egui::Ui, tokens: &Tokens, title: &str, detail: Option<&str>) {
    ui.label(
        egui::RichText::new(title)
            .size(tokens.text_label)
            .strong()
            .color(tokens.ink),
    );
    if let Some(detail) = detail {
        ui.label(
            egui::RichText::new(detail)
                .size(tokens.text_body)
                .color(tokens.ink_dim),
        );
    }
    ui.add_space(tokens.unit * 0.6);
}

/// A painted control gets the keyboard focus the rail and the cards need: `Tab` reaches it,
/// and the accessibility layer finds it by its name.
fn focusable(ui: &mut egui::Ui, rect: egui::Rect, id: egui::Id) -> egui::Response {
    let response = ui.interact(rect, id, egui::Sense::click());
    let layer = ui.layer_id();
    ui.memory_mut(|memory| memory.interested_in_focus(id, layer));
    response
}

/// Whether the reader activated the control, with the pointer or with `Enter`/`Space` while
/// it has the keyboard.
fn activated(ui: &egui::Ui, response: &egui::Response) -> bool {
    response.clicked()
        || (response.has_focus()
            && ui.input(|input| {
                input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Space)
            }))
}

/// Paints one action drawing, tinted with the theme's ink.
fn icon(ui: &egui::Ui, drawing: Drawing, label: &str, ink: egui::Color32, rect: egui::Rect) {
    let (id, bytes) = drawing;
    egui::Image::from_bytes(format!("bytes://actions/{id}.svg"), bytes)
        .tint(ink)
        .alt_text(label)
        .paint_at(ui, rect);
}
