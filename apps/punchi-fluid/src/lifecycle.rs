// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The document's life around the frame: whether it holds unsaved work, the safety copy that
//! work leaves while the window is open, the copy found at start, the question a close asks
//! and the two informational panels of the start rail.
//!
//! Everything here is about the document as a file the reader owns, not about editing it: the
//! window's own file commands stay in `files`, the page's two rows stay in `start`, and this
//! module joins them with one state and one frame hook so `main` keeps the layout only.
//!
//! The safety copy is not a save. It is written beside the recent list while the document has
//! unsaved changes, it is removed on a save, an open or a new document, and a copy found at
//! start is offered back by Inicio. A recovered document has no path, so the next save asks
//! for one, and it counts as unsaved work: the file the reader chose still does not hold it.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use eframe::egui;
use punchi_fluid_doc::Document;
use punchi_fluid_file::sheet::Sheet;
use punchi_fluid_i18n::Language;
use punchi_fluid_session::recents::Recents;
use punchi_fluid_session::safety::SafetyCopy;

use crate::{Window, files, inspector, ribbon, sidebar, theme, view};

/// How often the window rewrites the safety copy while the document has unsaved changes.
const AUTOSAVE_INTERVAL: Duration = Duration::from_secs(5);

/// The repository's licence, compiled into the window: the «Licencia» panel has to show the
/// terms where the source tree is not.
/// The project's public home: the popup names it and opens it only on an explicit click.
const REPOSITORY: &str = "https://github.com/PunchiSoft/punchiFluidLab";
/// Where a reader reports a problem.
const ISSUES: &str = "https://github.com/PunchiSoft/punchiFluidLab/issues";

const LICENCE: &str = include_str!("../../../LICENSE");

/// How the document stands against the files it came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Dirty {
    /// The document matches the file it was written to, or never had one.
    #[default]
    Clean,
    /// It was installed from the safety copy. The revision was reset to the one it was read
    /// at, but the work is still unsaved: no file the reader chose holds it.
    Recovered,
}

impl Dirty {
    fn is_dirty(self) -> bool {
        !matches!(self, Self::Clean)
    }
}

/// How far a close has gone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Close {
    /// No close is in progress.
    #[default]
    Idle,
    /// «Guardar y cerrar» was answered and its save is on its way: on the document's path, or
    /// through the dialog when the document has none.
    AfterSave,
    /// The viewport was asked to close; the copy is not written again.
    Sent,
}

/// The panel in front of the window, when one is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Panel {
    /// The application name, its version and what it is.
    About,
    /// The full text of the project's licence.
    Licence,
    /// «Hay cambios sin guardar».
    CloseAsk,
}

/// What the reader answered the close question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CloseAnswer {
    /// «Guardar y cerrar».
    Save,
    /// «Cerrar sin guardar».
    Discard,
    /// «Cancelar».
    Cancel,
}

/// The window's state outside the document: its saved revision, its safety copy, the recovery
/// it found at start, its title and the panel it shows.
pub(crate) struct Lifecycle {
    /// The document's revision when it was last saved, opened, recovered or started anew.
    saved_revision: u64,
    /// The copy written while the document has unsaved changes, and read back by a recovery.
    safety: SafetyCopy,
    /// When the copy was last written, so it is not written on every frame.
    written_at: Option<Instant>,
    /// The title sent to the viewport last, so a frame that does not change it sends nothing.
    title: Option<String>,
    /// How the document stands against the files.
    dirty: Dirty,
    /// How far a close has gone.
    close: Close,
    /// A copy found when the window opened, until the reader recovers or discards it. Once
    /// answered, the copy this window writes is its own work and is not offered back.
    offer: bool,
    /// The panel in front of the window, when one is open.
    panel: Option<Panel>,
}

impl Default for Lifecycle {
    fn default() -> Self {
        Self {
            saved_revision: 0,
            safety: SafetyCopy::default(),
            written_at: None,
            title: None,
            dirty: Dirty::Clean,
            close: Close::Idle,
            offer: false,
            panel: None,
        }
    }
}

impl Lifecycle {
    /// The life of `document` with the safety copy disabled, which is what a window that does
    /// not touch the reader's folder uses.
    fn new(document: &Document) -> Self {
        Self {
            saved_revision: document.revision(),
            ..Self::default()
        }
    }

    /// The life of a window that carries `safety`: the copy it writes, and the one it found at
    /// start, offered by Inicio until the reader answers.
    fn with_safety(mut self, safety: SafetyCopy) -> Self {
        self.offer = safety.exists();
        self.safety = safety;
        self
    }

    fn is_modified(&self, document: &Document) -> bool {
        self.dirty.is_dirty() || self.saved_revision != document.revision()
    }

    /// The document was written, opened or started anew: the revision becomes the saved one,
    /// and the safety copy has nothing left to keep.
    fn mark_saved(&mut self, revision: u64) {
        self.saved_revision = revision;
        self.dirty = Dirty::Clean;
        self.discard_copy();
    }

    /// The document was installed from the safety copy: the revision is the one it was read
    /// at, and it stays unsaved work until the reader saves it somewhere.
    fn mark_recovered(&mut self, revision: u64) {
        self.saved_revision = revision;
        self.dirty = Dirty::Recovered;
        self.offer = false;
        self.discard_copy();
    }

    fn discard_copy(&mut self) {
        let _ = self.safety.discard();
        self.written_at = None;
    }

    /// Sends the viewport its title while the document is being edited, and nothing when the
    /// title has not changed.
    fn sync_title(
        &mut self,
        context: &egui::Context,
        document: &Document,
        path: Option<&Path>,
        language: Language,
        start_open: bool,
    ) {
        let title = if start_open {
            language.text("app.name").to_owned()
        } else {
            let name = path
                .and_then(Path::file_name)
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or(language.text("status.unsaved"));
            let base = format!("punchi-fluid — {name}");
            if self.is_modified(document) {
                format!("• {base}")
            } else {
                base
            }
        };
        if self.title.as_deref() == Some(title.as_str()) {
            return;
        }
        context.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
        self.title = Some(title);
    }

    /// Writes the safety copy while the document has unsaved changes, at most once every
    /// [`AUTOSAVE_INTERVAL`]. A failed write costs the copy, never the reader's work: the next
    /// tick tries again, and the timer keeps the window repainting so that tick happens.
    fn autosave(&mut self, context: &egui::Context, document: &Document) {
        if !self.is_modified(document) || self.close == Close::Sent {
            return;
        }
        let now = Instant::now();
        if self
            .written_at
            .is_none_or(|written| now.duration_since(written) >= AUTOSAVE_INTERVAL)
        {
            let _ = self.safety.write(document);
            self.written_at = Some(now);
        }
        context.request_repaint_after(AUTOSAVE_INTERVAL);
    }
}

/// Reads the document named on the command line, if any, and the recent list.
///
/// A failed read leaves an empty document and returns the cause, so the window can report it
/// once it knows the language the reader chose.
pub(crate) fn load_startup(path: Option<&Path>) -> (Document, Option<String>, Recents) {
    let mut recents = Recents::load();
    let mut startup_error = None;
    let document = match path {
        Some(path) => match Sheet::read(path)
            .map_err(|cause| cause.to_string())
            .and_then(|sheet| punchi_fluid_session::load(&sheet))
        {
            Ok(document) => {
                // A document named on the command line is a document the page offers to
                // reopen, exactly as if it had been opened from the dialog.
                recents.remember(path);
                document
            }
            Err(cause) => {
                startup_error = Some(cause);
                Document::new()
            }
        },
        None => Document::new(),
    };
    (document, startup_error, recents)
}

impl Window {
    /// The window `main` opens, with the safety copy disabled: the tests use this one, so they
    /// never touch the reader's folder.
    pub(crate) fn new(
        theme: theme::Theme,
        document: Document,
        path: Option<PathBuf>,
        notice: Option<String>,
    ) -> Self {
        let (page, block) = files::page_and_block(&document);
        Self {
            lifecycle: Lifecycle::new(&document),
            document,
            catalog: punchi_fluid_catalog::default_catalog(),
            path,
            notice,
            page,
            block,
            view: view::View::default(),
            magnification: 1.0,
            frame_ms: 0.0,
            frame_trace: None,
            #[cfg(test)]
            sheet_probe: None,
            theme,
            ribbon: ribbon::Ribbon::new(),
            speed: ribbon::Speed::Normal,
            page_setup: crate::page_setup::PageSetup::default(),
            block_editor: crate::block_editor::BlockEditor::default(),
            circuit_dialog: None,
            export_options: crate::export_options::ExportOptions::default(),
            export_options_draft: crate::export_options::ExportOptions::default(),
            check_panel: crate::check_panel::CheckPanel::default(),
            sidebar: sidebar::Sidebar::new(),
            inspector: inspector::Inspector::new(),
            selected: Vec::new(),
            selected_pipe: None,
            drag: None,
            connection_drag: None,
            moving: None,
            menu: crate::sheet_menu::SheetMenu::new(),
            mockups: crate::sheet_menu::Mockups::new(),
            valve_editor: None,
            clipboard: Vec::new(),
            model: None,
            model_revision: 0,
            running: false,
            show_block: true,
            hand_mode: false,
            zoom_window_armed: false,
            zoom_drag: None,
            strip: crate::view_strip::ViewStrip::default(),
            grid: crate::grid::GridView::default(),
            pending_drop: None,
            strip_edit: None,
            pressed_valve: None,
            start_open: true,
            recents: Recents::default(),
            dialog: None,
            appearance_day: crate::theme::ThemeId::Cad,
            appearance_night: crate::theme::ThemeId::Slate,
            preferences: crate::preferences::Preferences::new(),
            language: punchi_fluid_i18n::Language::English,
            shell: crate::shell::Shell::new(),
            settings: crate::settings::Store::at(None),
            auto_hide: crate::auto_hide::AutoHide::new(),
            autohide_on: false,
        }
    }

    /// Attaches the geometry store read at start, so the first frame can size and place the
    /// window from the screen the viewport reports.
    pub(crate) fn with_window_state(mut self, state: crate::window_state::Store) -> Self {
        self.shell = crate::shell::Shell::with_store(state);
        self
    }

    /// The window `main` opens, carrying the safety copy it loaded. The recovery it found is
    /// offered by Inicio on its first frames.
    pub(crate) fn with_safety(
        theme: theme::Theme,
        document: Document,
        path: Option<PathBuf>,
        notice: Option<String>,
        recents: Recents,
        safety: SafetyCopy,
    ) -> Self {
        let mut window = Self::with_recents(theme, document, path, notice, recents);
        window.lifecycle = Lifecycle::new(&window.document).with_safety(safety);
        window
    }

    /// The one call `update` makes into this module: the title, the safety copy, the close
    /// question and the panels, before the page or the sheet draws under them.
    pub(crate) fn lifecycle_frame(&mut self, context: &egui::Context, start_open: bool) {
        self.lifecycle.sync_title(
            context,
            &self.document,
            self.path.as_deref(),
            self.language,
            start_open,
        );
        self.lifecycle.autosave(context, &self.document);
        // The save «Guardar y cerrar» awaited finished, on the document's path or through the
        // dialog that asked for one: the window can go.
        if self.lifecycle.close == Close::AfterSave && self.dialog.is_none() && !self.is_modified()
        {
            self.lifecycle.close = Close::Sent;
            context.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if context.input(|input| input.viewport().close_requested()) && self.is_modified() {
            if self.lifecycle.close == Close::Sent {
                // The close we asked for comes back as a close request of its own: egui-winit
                // turns `ViewportCommand::Close` into a `ViewportEvent::Close` for the next
                // frame. Cancelling that echo would cancel the close the reader just chose and
                // reopen the question for ever; letting it through closes the window. A
                // platform that refuses it gets the question again on the next attempt.
                self.lifecycle.close = Close::Idle;
            } else {
                context.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.lifecycle.panel = Some(Panel::CloseAsk);
            }
        }
        self.show_panels(context);
    }

    /// Whether the document holds work that is not in the file the reader chose.
    pub(crate) fn is_modified(&self) -> bool {
        self.lifecycle.is_modified(&self.document)
    }

    /// Whether a dialog is in front of the window: the file dialog, one of the informational
    /// panels, the valve editor or a circuit dialog. The auto-hide timers never fire while one
    /// is open.
    pub(crate) fn dialog_open(&self) -> bool {
        self.dialog.is_some()
            || self.lifecycle.panel.is_some()
            || self.mockups.is_open()
            || self.valve_editor.is_some()
            || self.circuit_dialog.is_some()
    }

    /// Turns the current revision of the document into the saved one, and throws the copy
    /// away: called by every save and every open that succeeded.
    pub(crate) fn mark_saved(&mut self) {
        self.lifecycle.mark_saved(self.document.revision());
    }

    /// Whether a copy found at start is still waiting for the reader's answer.
    pub(crate) fn has_recovery(&self) -> bool {
        self.lifecycle.offer
    }

    /// Reads the copy back and installs it, with no path: the next save asks for one. A copy
    /// that cannot be read keeps the offer and reports the reason.
    pub(crate) fn recover(&mut self) {
        match self.lifecycle.safety.read() {
            Ok(document) => {
                let revision = document.revision();
                self.replace_document(document, None);
                self.lifecycle.mark_recovered(revision);
                self.notice = Some(self.language.text("lifecycle.recovered").to_owned());
            }
            Err(cause) => {
                self.notice = Some(
                    self.language
                        .format("lifecycle.recover_failed", &[("cause", &cause)]),
                );
            }
        }
    }

    /// Throws the copy found at start away, so the offer disappears with it.
    pub(crate) fn discard_recovery(&mut self) {
        self.lifecycle.discard_copy();
        self.lifecycle.offer = false;
        self.notice = Some(self.language.text("lifecycle.discarded").to_owned());
    }

    /// A save the reader cancelled: a «Guardar y cerrar» waiting for it stops waiting.
    pub(crate) fn save_cancelled(&mut self) {
        if self.lifecycle.close == Close::AfterSave {
            self.lifecycle.close = Close::Idle;
        }
    }

    pub(crate) fn open_about(&mut self) {
        self.lifecycle.panel = Some(Panel::About);
    }

    pub(crate) fn open_licence(&mut self) {
        self.lifecycle.panel = Some(Panel::Licence);
    }

    /// Draws the panel in front of the window, and closes it with `Esc` or with its answer.
    fn show_panels(&mut self, context: &egui::Context) {
        if self.lifecycle.panel.is_some()
            && context
                .input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            if self.lifecycle.panel.take() == Some(Panel::CloseAsk) {
                self.lifecycle.close = Close::Idle;
            }
            return;
        }
        match self.lifecycle.panel {
            Some(Panel::About) => self.show_about(context),
            Some(Panel::Licence) => self.show_licence(context),
            Some(Panel::CloseAsk) => self.show_close_ask(context),
            None => {}
        }
    }

    /// «Acerca de»: the application, its version, what it is and where to find it. The
    /// popup swallows the clicks under it and closes with Cerrar, `Esc` or a click outside;
    /// opening it never changes the document.
    fn show_about(&mut self, context: &egui::Context) {
        let tokens = self.theme.tokens.clone();
        let language = self.language;
        let mut close = false;
        let mut read_licence = false;
        // The reference popup does not dim the window; it does keep the clicks under it.
        let screen = context.screen_rect();
        let mut clicked_out = false;
        egui::Area::new(egui::Id::new("lifecycle.about.blocker"))
            .order(egui::Order::Middle)
            .fixed_pos(screen.min)
            .show(context, |ui| {
                clicked_out = ui.allocate_rect(screen, egui::Sense::click()).clicked();
            });
        egui::Window::new(language.text("about.title"))
            .id(egui::Id::new("lifecycle.about"))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .show(context, |ui| {
                ui.set_width(tokens.unit * 20.0);
                ui.label(
                    egui::RichText::new("Punchi Fluid Lab")
                        .size(tokens.text_display)
                        .strong()
                        .color(tokens.ink),
                );
                ui.label(
                    egui::RichText::new(
                        language.format("about.version", &[("version", env!("CARGO_PKG_VERSION"))]),
                    )
                    .size(tokens.text_body)
                    .color(tokens.ink_dim),
                );
                ui.label(
                    egui::RichText::new(language.text("about.development"))
                        .size(tokens.text_body)
                        .color(tokens.accent),
                );
                ui.add_space(tokens.unit * 0.6);
                ui.label(
                    egui::RichText::new(language.text("about.body"))
                        .size(tokens.text_body)
                        .color(tokens.ink),
                );
                ui.add_space(tokens.unit * 0.6);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(language.text("about.repository"))
                            .size(tokens.text_small)
                            .color(tokens.ink_muted),
                    );
                    ui.hyperlink_to(REPOSITORY, REPOSITORY);
                });
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(language.text("about.help"))
                            .size(tokens.text_small)
                            .color(tokens.ink_muted),
                    );
                    ui.hyperlink_to(language.text("about.issues"), ISSUES);
                });
                ui.add_space(tokens.unit * 0.4);
                ui.label(
                    egui::RichText::new(language.text("about.copyright"))
                        .size(tokens.text_small)
                        .color(tokens.ink_muted),
                );
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(language.text("about.author"))
                            .size(tokens.text_small)
                            .color(tokens.ink_muted),
                    );
                    if ui.button(language.text("about.read_licence")).clicked() {
                        read_licence = true;
                    }
                });
                ui.add_space(tokens.unit * 0.6);
                ui.label(
                    egui::RichText::new(language.text("about.closing"))
                        .size(tokens.text_small)
                        .color(tokens.ink_muted),
                );
                ui.add_space(tokens.unit * 0.6);
                if ui.button(language.text("dialog.close")).clicked() {
                    close = true;
                }
            });
        if read_licence {
            self.open_licence();
            context.request_repaint();
            return;
        }
        if close || clicked_out {
            self.lifecycle.panel = None;
        }
    }

    /// «Licencia»: the GPL summary and the complete text, compiled into the binary.
    fn show_licence(&mut self, context: &egui::Context) {
        let tokens = self.theme.tokens.clone();
        let language = self.language;
        let mut close = false;
        egui::Window::new(language.text("licence.title"))
            .id(egui::Id::new("lifecycle.licence"))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(true)
            .default_size(egui::vec2(tokens.unit * 26.0, tokens.unit * 26.0))
            .show(context, |ui| {
                ui.set_width(tokens.unit * 22.0);
                ui.label(
                    egui::RichText::new(language.text("licence.version"))
                        .size(tokens.text_body)
                        .strong()
                        .color(tokens.ink),
                );
                egui::ScrollArea::vertical()
                    .max_height(tokens.unit * 20.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(LICENCE)
                                .size(tokens.text_small)
                                .color(tokens.ink_dim),
                        );
                    });
                ui.add_space(tokens.unit * 0.4);
                if ui.button(language.text("dialog.close")).clicked() {
                    close = true;
                }
            });
        if close {
            self.lifecycle.panel = None;
        }
    }

    pub(crate) fn close_question_open(&self) -> bool {
        self.lifecycle.panel == Some(Panel::CloseAsk)
    }

    /// «Hay cambios sin guardar»: the question a close asks before it throws work away.
    fn show_close_ask(&mut self, context: &egui::Context) {
        use crate::dialog::{Action, ButtonRole, Dialog, DialogResult, DialogStyle};

        let language = self.language;
        let style = DialogStyle::new(&self.theme.tokens);
        let question = Dialog {
            id: egui::Id::new("lifecycle.close"),
            title: language.text("unsaved.title"),
            close_label: language.text("dialog.close"),
            preferred_width: (self.theme.tokens.unit * 34.0).max(540.0),
            style,
        };
        let response = question.show(
            context,
            &mut (),
            |ui, ()| {
                ui.label(language.text("unsaved.body"));
            },
            |()| {
                vec![
                    Action {
                        label: language.text("unsaved.save_close"),
                        value: CloseAnswer::Save,
                        role: ButtonRole::Primary,
                        enabled: true,
                        focus: false,
                    },
                    Action {
                        label: language.text("unsaved.close_without"),
                        value: CloseAnswer::Discard,
                        role: ButtonRole::Secondary,
                        enabled: true,
                        focus: false,
                    },
                    Action {
                        label: language.text("unsaved.cancel"),
                        value: CloseAnswer::Cancel,
                        role: ButtonRole::Secondary,
                        enabled: true,
                        focus: false,
                    },
                ]
            },
        );
        match response.inner {
            Some(DialogResult::Action(answer)) => self.answer_close(answer, context),
            Some(DialogResult::Close) => self.answer_close(CloseAnswer::Cancel, context),
            None => {}
        }
    }

    /// Runs the answer to the close question. A save that asks for a name keeps the flag: the
    /// dialog's success path closes the window when the save lands.
    fn answer_close(&mut self, answer: CloseAnswer, context: &egui::Context) {
        self.lifecycle.panel = None;
        match answer {
            CloseAnswer::Save => {
                self.lifecycle.close = Close::AfterSave;
                self.save();
                if self.dialog.is_none() && self.is_modified() {
                    // The save failed on the document's own path: the status bar says why and
                    // the window stays open.
                    self.lifecycle.close = Close::Idle;
                }
            }
            CloseAnswer::Discard => {
                self.lifecycle.discard_copy();
                self.lifecycle.close = Close::Sent;
                context.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            CloseAnswer::Cancel => self.lifecycle.close = Close::Idle,
        }
    }
}

#[cfg(test)]
mod tests;
