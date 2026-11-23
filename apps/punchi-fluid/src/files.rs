// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The in-app file dialog and the window's file commands.
//!
//! The window carries no platform picker: a toolkit dialog is a dependency this project does
//! not take, and a dialog of its own keeps the theme tokens, the keyboard rules and the
//! Spanish strings of the rest of the window. `Abrir` lists the documents the sheet format
//! can read; `Guardar como` asks for a name and writes it; `Exportar` asks for a name and
//! prints the drawing as PDF, PNG or SVG.
//!
//! The dialog is modal the way the interface contract asks for: while it is open it takes the
//! keyboard, `Enter` confirms, `Esc` cancels, and a transparent layer over the page keeps the
//! pointer from reaching the rows behind it. It is an [`egui::Window`], not a fourth region.
//!
//! Every command that touches the filesystem lives here, because a save and an open must
//! remember the path in the same place: keeping them beside the dialog is what makes the
//! ribbon's `Guardar`, the dialog's own `Guardar` and the start page's rows agree.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use eframe::egui;
use punchi_fluid_doc::Document;
use punchi_fluid_export::{Page, TitleBlock};
use punchi_fluid_file::sheet::Sheet;
use punchi_fluid_session::recents::Recents;

use crate::Window;
use crate::theme::{Theme, Tokens};

/// Extension of a saved document, as the sheet format writes it.
pub(crate) const EXTENSION: &str = "pfluid";

/// Extensions the export dialog writes, in the order its name rule documents them. The
/// rendering itself picks the writer by these; the dialog only lets a reader type one of
/// them, so a name it accepts is a name the exporter can render.
pub(crate) const EXPORT_EXTENSIONS: [&str; 3] = ["pdf", "png", "svg"];

/// This build lists at most this many entries of one folder. A folder is untrusted input and
/// the dialog reads it on the frame it opens; the limit keeps a pathological folder from
/// making the window allocate without a bound. Reaching it is reported, never silent.
const MAX_ENTRIES: usize = 1_000;

/// Longest file name the dialog accepts, in bytes. It is well under `MAX_PATH` of the recent
/// list and the common filesystem limit, so a name that is too long is refused instead of
/// letting the save fail with a less clear message.
const MAX_NAME: usize = 255;

/// What the dialog is for: choosing a document to read, naming a document to write, or
/// naming an export to print.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    /// `Abrir`: pick an existing document.
    Open,
    /// `Guardar como`: name a new document or a new name for the current one.
    SaveAs,
    /// `Exportar`: name a drawing to print as PDF, PNG or SVG.
    Export,
}

/// The two modes that write a file. They share the name rules and the overwrite question,
/// and differ in the extension they default to and in the outcome they return; a mode that
/// does not write has no `Write`, so the confirmation never has to guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Write {
    /// A `.pfluid` document the window keeps editing.
    Document,
    /// A drawing printed to PDF, PNG or SVG.
    Export,
}

impl Mode {
    /// The dialog's title, in the reader's language.
    fn title(self) -> &'static str {
        match self {
            Self::Open => "Abrir",
            Self::SaveAs => "Guardar como",
            Self::Export => "Exportar",
        }
    }

    /// The main button's label: the same verb as the title.
    fn confirm_label(self) -> &'static str {
        match self {
            Self::Open => "Abrir",
            Self::SaveAs => "Guardar",
            Self::Export => "Exportar",
        }
    }

    /// What this dialog writes, or `None` when it only reads.
    fn write(self) -> Option<Write> {
        match self {
            Self::Open => None,
            Self::SaveAs => Some(Write::Document),
            Self::Export => Some(Write::Export),
        }
    }
}

/// What the dialog asks the window to do after the frame is drawn. It is returned instead of
/// acting directly so the dialog stays a view and the file work stays in the window, where
/// the path and the recent list live.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// Nothing happened: the reader cancelled or closed the dialog.
    Cancelled,
    /// Read this document.
    Open(PathBuf),
    /// Write the document to this path.
    Save(PathBuf),
    /// Print the drawing to this path.
    Export(PathBuf),
}

/// One row of the folder listing.
#[derive(Debug, Clone)]
struct Entry {
    path: PathBuf,
    name: String,
    folder: bool,
}

/// The state of one open file dialog.
#[derive(Debug, Clone)]
pub(crate) struct Dialog {
    mode: Mode,
    directory: PathBuf,
    entries: Vec<Entry>,
    /// Why the folder could not be listed, when it could not.
    listing_error: Option<String>,
    /// The name field: the selected file in open mode, the name to write in save mode.
    name: String,
    /// What the last confirmation could not do, shown under the fields.
    error: Option<String>,
    /// A save found the target taken and waits for a second confirmation. A second click is
    /// the whole overwrite prompt: the dialog says the file exists and what the next click
    /// does, so nothing is replaced without the reader having read the sentence.
    confirm_overwrite: bool,
    /// Whether the name field still has to take the keyboard, on the first frame.
    focus_name: bool,
}

impl Dialog {
    /// Opens a dialog in `mode`, starting in `start` and listing it.
    pub(crate) fn open(mode: Mode, start: PathBuf) -> Self {
        let mut dialog = Self {
            mode,
            directory: start,
            entries: Vec::new(),
            listing_error: None,
            name: String::new(),
            error: None,
            confirm_overwrite: false,
            focus_name: matches!(mode, Mode::SaveAs | Mode::Export),
        };
        dialog.read_directory();
        dialog
    }

    /// Puts a name in the field, as clicking a file row does, and forgets an answered
    /// overwrite: a different target is a different question.
    fn set_name(&mut self, name: &str) {
        name.clone_into(&mut self.name);
        self.confirm_overwrite = false;
        self.error = None;
    }

    /// Enters a folder and lists it.
    fn enter(&mut self, directory: PathBuf) {
        self.directory = directory;
        self.name.clear();
        self.confirm_overwrite = false;
        self.error = None;
        self.read_directory();
    }

    /// Reads the current folder, newest decisions first: folders before files, each group in
    /// name order, and only the files the mode offers. `Abrir` and `Guardar como` list the
    /// documents the sheet format can read; `Exportar` lists the files it can overwrite.
    fn read_directory(&mut self) {
        self.listing_error = None;
        let mut entries = Vec::new();
        let listed: fn(&Path) -> bool = match self.mode {
            Mode::Open | Mode::SaveAs => is_document,
            Mode::Export => is_export,
        };
        let read = match std::fs::read_dir(&self.directory) {
            Ok(read) => read,
            Err(cause) => {
                self.listing_error = Some(format!("no se pudo leer la carpeta: {cause}"));
                self.entries = entries;
                return;
            }
        };
        for item in read {
            if entries.len() >= MAX_ENTRIES {
                self.listing_error = Some(String::from(
                    "la carpeta tiene más entradas de las que esta ventana lista",
                ));
                break;
            }
            let Ok(item) = item else {
                continue;
            };
            let path = item.path();
            // A name that is not UTF-8 cannot be shown or typed back, so it is skipped
            // instead of being listed with a lossy name that would not open.
            let Some(name) = path.file_name().and_then(OsStr::to_str).map(str::to_owned) else {
                continue;
            };
            // The hidden entries of a home folder are noise in a file dialog, and the list
            // has to stay reachable: a dot-folder can hold thousands of files.
            if name.starts_with('.') {
                continue;
            }
            let folder = path.is_dir();
            if !folder && !listed(&path) {
                continue;
            }
            entries.push(Entry { path, name, folder });
        }
        entries.sort_by_cached_key(|entry| (!entry.folder, entry.name.to_lowercase()));
        self.entries = entries;
    }

    /// Confirms the dialog: what `Enter` and the main button do.
    ///
    /// `None` means the dialog stays open and [`Self::error`] says why. A document gets the
    /// sheet's extension when the typed name has none; an export gets `.pdf`, and a typed
    /// extension has to be one this build renders. A name that is empty, too long or carries
    /// a folder is refused. An existing target is confirmed once more before it is replaced.
    fn confirm(&mut self) -> Option<Outcome> {
        let name = self.name.trim().to_owned();
        if name.is_empty() {
            self.error = Some(String::from(match self.mode {
                Mode::Open => "Elige un documento de la lista.",
                Mode::SaveAs | Mode::Export => "Escribe un nombre de archivo.",
            }));
            return None;
        }
        // `Abrir` reads the name as it is: the listing already offered a document, and the
        // front end reports whatever the reader cannot open.
        let Some(write) = self.mode.write() else {
            return Some(Outcome::Open(self.directory.join(name)));
        };
        if name.len() > MAX_NAME {
            self.error = Some(String::from("El nombre es demasiado largo."));
            return None;
        }
        if name.contains(['/', '\\']) {
            self.error = Some(String::from("El nombre no puede llevar una carpeta."));
            return None;
        }
        let file_name = match write {
            Write::Document => document_name(name),
            Write::Export => match export_name(name) {
                Ok(file_name) => file_name,
                Err(message) => {
                    self.error = Some(message);
                    return None;
                }
            },
        };
        let target = self.directory.join(&file_name);
        if target.exists() && !self.confirm_overwrite {
            self.confirm_overwrite = true;
            let verb = match write {
                Write::Document => "Guardar",
                Write::Export => "Exportar",
            };
            self.error = Some(format!(
                "«{file_name}» ya existe: pulsa {verb} otra vez para reemplazarlo."
            ));
            return None;
        }
        self.confirm_overwrite = false;
        Some(match write {
            Write::Document => Outcome::Save(target),
            Write::Export => Outcome::Export(target),
        })
    }

    /// Draws the dialog over the page and returns what it asked for, if anything.
    fn show(&mut self, context: &egui::Context, tokens: &Tokens) -> Option<Outcome> {
        // The spec is explicit: `Enter` confirms and `Esc` cancels. `Esc` is answered before
        // the window is drawn so the dialog leaves on the same frame.
        if context.input(|input| input.key_pressed(egui::Key::Escape)) {
            return Some(Outcome::Cancelled);
        }
        // Read before the widgets run: an `Enter` that activates a focused button is the
        // button's answer, and this variable only confirms when nothing else did.
        let entered = context.input(|input| input.key_pressed(egui::Key::Enter));
        let mut outcome = None;
        let mut button_acted = false;
        Self::block_page(context);
        // A fixed frame, not an anchor: the dialog must keep its place under the pointer
        // while the reader clicks it, and a fixed size keeps every frame the same layout.
        let size = egui::vec2(tokens.unit * 30.0, tokens.unit * 26.0);
        let position = context.screen_rect().center() - size * 0.5;
        egui::Window::new(self.mode.title())
            .id(egui::Id::new("files.dialog"))
            .order(egui::Order::Foreground)
            .fixed_pos(position)
            .fixed_size(size)
            .collapsible(false)
            .resizable(false)
            .show(context, |ui| {
                button_acted = self.contents(ui, tokens, &mut outcome);
            });
        // An `Enter` that a focused button already answered is that button's, not a second
        // confirmation: otherwise a save over an existing file would replace it on the same
        // key press that asked the question.
        if outcome.is_none() && entered && !button_acted {
            outcome = self.confirm();
        }
        outcome
    }

    /// The dialog's body: the folder, its rows, the name field and the buttons. Returns
    /// whether a button answered this frame, so `show` does not confirm twice.
    fn contents(
        &mut self,
        ui: &mut egui::Ui,
        tokens: &Tokens,
        outcome: &mut Option<Outcome>,
    ) -> bool {
        let mut button_acted = false;
        ui.label(
            egui::RichText::new(format!("Carpeta: {}", self.directory.display()))
                .size(tokens.text_body)
                .color(tokens.ink_dim),
        );
        if let Some(error) = self.listing_error.as_deref() {
            ui.label(
                egui::RichText::new(error)
                    .size(tokens.text_body)
                    .color(tokens.ink),
            );
        }
        // The entries are taken out for the frame so the list can be drawn while the
        // dialog's own fields are mutated afterwards; they are put back before anything
        // reads them again.
        let entries = std::mem::take(&mut self.entries);
        let mut navigate = None;
        let mut chosen = None;
        let selected = self.name.clone();
        egui::ScrollArea::vertical()
            .max_height(tokens.unit * 14.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                if let Some(parent) = self.directory.parent().map(Path::to_path_buf) {
                    let row = ui.add(egui::Button::new("..").frame(false));
                    row.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::WidgetType::Button,
                            true,
                            "Carpeta superior",
                        )
                    });
                    if row.clicked() {
                        navigate = Some(parent);
                    }
                }
                for entry in &entries {
                    let label = if entry.folder {
                        format!("{}/", entry.name)
                    } else {
                        entry.name.clone()
                    };
                    let is_selected = entry.name == selected;
                    let row = ui.add(egui::Button::selectable(is_selected, label));
                    if row.clicked() {
                        if entry.folder {
                            navigate = Some(entry.path.clone());
                        } else {
                            chosen = Some(entry.path.clone());
                        }
                    }
                }
            });
        self.entries = entries;
        if let Some(directory) = navigate {
            self.enter(directory);
            return true;
        }
        if let Some(path) = chosen {
            // A click selects the document; `Enter` or the main button opens or overwrites
            // it. In save mode it is also the way to take an existing name.
            if let Some(name) = path.file_name().and_then(OsStr::to_str) {
                self.set_name(name);
            }
        }
        if self.mode.write().is_some() {
            self.name_field(ui, tokens);
        }
        if let Some(error) = self.error.as_deref() {
            ui.label(
                egui::RichText::new(error)
                    .size(tokens.text_small)
                    .color(tokens.ink),
            );
        }
        ui.horizontal(|ui| {
            if ui.button(self.mode.confirm_label()).clicked() {
                *outcome = self.confirm();
                button_acted = true;
            }
            if ui.button("Cancelar").clicked() {
                *outcome = Some(Outcome::Cancelled);
                button_acted = true;
            }
        });
        button_acted
    }

    /// The name row of a dialog that writes: the field, its hint for the mode, and the
    /// keyboard focus it takes on the first frame.
    fn name_field(&mut self, ui: &mut egui::Ui, tokens: &Tokens) {
        let hint = match self.mode {
            Mode::Export => "circuito.pdf",
            Mode::Open | Mode::SaveAs => "documento.pfluid",
        };
        ui.horizontal(|ui| {
            let label = ui.label(
                egui::RichText::new("Nombre")
                    .size(tokens.text_body)
                    .color(tokens.ink_dim),
            );
            let field = ui
                .add(
                    egui::TextEdit::singleline(&mut self.name)
                        .hint_text(hint)
                        .desired_width(tokens.unit * 18.0),
                )
                .labelled_by(label.id);
            if self.focus_name {
                field.request_focus();
                self.focus_name = false;
            }
        });
    }

    /// A transparent layer over the page: the dialog is modal, so the pointer finds nothing
    /// to press behind it. It is not painted dim: the window is a panel over the page, and
    /// the page stays readable while the reader chooses a file.
    fn block_page(context: &egui::Context) {
        let screen = context.screen_rect();
        egui::Area::new(egui::Id::new("files.blocker"))
            .order(egui::Order::Middle)
            .fixed_pos(screen.min)
            .show(context, |ui| {
                ui.allocate_rect(screen, egui::Sense::click_and_drag());
            });
    }

    /// Puts a message in the dialog after a failed read or write, so the error is shown
    /// where the reader chose the file and the window stays where it was.
    fn fail(&mut self, message: String) {
        self.error = Some(message);
    }
}

/// `true` when the path names a file the sheet format could read.
fn is_document(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| extension.eq_ignore_ascii_case(EXTENSION))
}

/// `true` when the path names a file the export dialog can list, matching
/// [`EXPORT_EXTENSIONS`] in any case.
fn is_export(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(is_export_extension)
}

/// `true` when `extension` (without the dot) is one this build renders, in any case.
pub(crate) fn is_export_extension(extension: &str) -> bool {
    EXPORT_EXTENSIONS
        .iter()
        .any(|allowed| extension.eq_ignore_ascii_case(allowed))
}

/// The name a document save writes: the sheet's extension when the reader typed none.
fn document_name(name: String) -> String {
    if Path::new(&name).extension().is_none() {
        format!("{name}.{EXTENSION}")
    } else {
        name
    }
}

/// The name an export writes: `.pdf` when the reader typed no extension, else the typed
/// name, whose extension is kept as typed. A name that asks for another format is refused
/// with the reason, because a reader who typed `.docx` wanted a file this build cannot draw.
fn export_name(name: String) -> Result<String, String> {
    match Path::new(&name).extension().and_then(OsStr::to_str) {
        None => Ok(format!("{name}.pdf")),
        Some(extension) if is_export_extension(extension) => Ok(name),
        Some(_) => Err(String::from(
            "Un archivo de exportación tiene que ser pdf, png o svg.",
        )),
    }
}

/// Where a file dialog starts: beside the current document, else the user's home, else the
/// working folder. `HOME` and `USERPROFILE` are read directly because
/// `std::env::home_dir` is deprecated; the session's recent list resolves the same way.
pub(crate) fn start_directory(path: Option<&Path>) -> PathBuf {
    if let Some(folder) = path
        .and_then(Path::parent)
        .filter(|folder| !folder.as_os_str().is_empty() && folder.is_dir())
    {
        return folder.to_path_buf();
    }
    if let Some(home) = home_directory() {
        return home;
    }
    // The working folder always exists; if it cannot be read the dialog reports that and the
    // reader can still navigate from `..`.
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// The user's home, from the environment the session's recent list also reads.
fn home_directory() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .filter(|home| !home.as_os_str().is_empty())
}

/// The name a document is shown by: its file name, or the whole path when it has none.
fn display_name(path: &Path) -> String {
    path.file_name()
        .and_then(OsStr::to_str)
        .map_or_else(|| path.display().to_string(), str::to_owned)
}

/// The page and the title block the sheet draws, from the document's active circuit.
pub(crate) fn page_and_block(document: &Document) -> (Page, TitleBlock) {
    let sheet = document.sheet();
    let block = document
        .block(document.active_circuit())
        .cloned()
        .unwrap_or_default();
    let page = Page {
        paper: sheet.paper,
        orientation: sheet.orientation,
        margins: sheet.margins,
        title_block: Some(block.clone()),
        parts: None,
    };
    (page, block)
}

impl Window {
    /// A window that remembers its documents in `recents`: `main` loads the list and hands it
    /// over, so a document opened from the command line is remembered too. `Window::new`
    /// keeps the list in memory, which is what the tests want: they never touch the reader's
    /// real list.
    pub(crate) fn with_recents(
        theme: Theme,
        document: Document,
        path: Option<PathBuf>,
        notice: Option<String>,
        recents: Recents,
    ) -> Self {
        let mut window = Self::new(theme, document, path, notice);
        window.recents = recents;
        window
    }

    /// Opens the file dialog in `mode`, starting where the document or the reader is.
    pub(crate) fn open_dialog(&mut self, mode: Mode) {
        let start = start_directory(self.path.as_deref());
        self.dialog = Some(Dialog::open(mode, start));
    }

    /// Draws the dialog while it is open and runs what it asked for.
    pub(crate) fn show_files(&mut self, context: &egui::Context) {
        let Some(dialog) = self.dialog.as_mut() else {
            return;
        };
        let outcome = dialog.show(context, &self.theme.tokens);
        let Some(outcome) = outcome else {
            return;
        };
        self.apply_files(outcome);
    }

    /// Reads or writes the document the dialog chose. An error keeps the dialog open with
    /// the reason; a success closes it.
    pub(crate) fn apply_files(&mut self, outcome: Outcome) {
        match outcome {
            Outcome::Cancelled => {
                // A «Guardar y cerrar» waiting for this dialog stops waiting: the reader chose
                // to keep working.
                self.save_cancelled();
                self.dialog = None;
            }
            Outcome::Open(path) => match self.open_path(&path) {
                Ok(()) => self.dialog = None,
                Err(cause) => {
                    if let Some(dialog) = self.dialog.as_mut() {
                        dialog.fail(format!("no se pudo abrir: {cause}"));
                    }
                }
            },
            Outcome::Save(path) => match self.save_to(&path) {
                Ok(()) => {
                    // The dialog's save is the one that can take a new path: `save_to` leaves
                    // the window's path alone so an export can reuse it without touching it.
                    self.path = Some(path);
                    self.dialog = None;
                }
                Err(cause) => {
                    if let Some(dialog) = self.dialog.as_mut() {
                        dialog.fail(format!("no se pudo guardar: {cause}"));
                    }
                }
            },
            Outcome::Export(path) => match self.export_to(&path) {
                Ok(()) => {
                    self.dialog = None;
                    self.notice = Some(format!("exportado {}", display_name(&path)));
                }
                Err(cause) => {
                    if let Some(dialog) = self.dialog.as_mut() {
                        dialog.fail(cause);
                    }
                }
            },
        }
    }

    /// Reads a document and makes it the window's: the sheet, the path, the drawing's page
    /// and the recent list. Nothing changes when the file cannot be read.
    ///
    /// # Errors
    ///
    /// Returns the sheet reader's or the session's message when the file is not a document
    /// this build can open.
    pub(crate) fn open_path(&mut self, path: &Path) -> Result<(), String> {
        let sheet = Sheet::read(path).map_err(|cause| cause.to_string())?;
        let document = punchi_fluid_session::load(&sheet)?;
        self.replace_document(document, Some(path.to_path_buf()));
        self.recents.remember(path);
        self.notice = Some(format!("abierto {}", display_name(path)));
        self.mark_saved();
        Ok(())
    }

    /// Drops one document from the start page's list. The file is not touched.
    pub(crate) fn forget_recent(&mut self, path: &str) {
        if self.recents.forget(path) {
            self.notice = Some(String::from("quitado de recientes"));
        }
    }

    /// `Nuevo`: an empty document with no path, on the sheet instead of Inicio.
    pub(crate) fn new_document(&mut self) {
        self.replace_document(Document::new(), None);
        self.notice = Some(String::from("documento nuevo"));
        self.mark_saved();
    }

    /// `Guardar`: writes to the current path, or asks for one when the document has none.
    pub(crate) fn save(&mut self) {
        let Some(path) = self.path.clone() else {
            self.open_dialog(Mode::SaveAs);
            return;
        };
        if let Err(cause) = self.save_to(&path) {
            self.notice = Some(format!("no se pudo guardar: {cause}"));
        }
    }

    /// Writes the document to `path`, remembers it and marks it saved. The window's own path
    /// is left alone: the commands that own the document decide it, and an export prints the
    /// document without touching it.
    ///
    /// # Errors
    ///
    /// Returns the session's message when the file cannot be written; nothing is marked
    /// saved in that case.
    pub(crate) fn save_to(&mut self, path: &Path) -> Result<(), String> {
        punchi_fluid_session::save(&self.document, path)?;
        self.recents.remember(path);
        self.notice = Some(format!("guardado {}", display_name(path)));
        self.mark_saved();
        Ok(())
    }

    /// Makes `document` the window's, closes Inicio and the dialog, and rebinds the page the
    /// sheet draws to the new document's active circuit. The model is dropped so the next
    /// start builds the new circuit at `t = 0`. The safety copy is not touched here: the
    /// command that installed the document decides whether to discard it or mark it recovered.
    pub(crate) fn replace_document(&mut self, document: Document, path: Option<PathBuf>) {
        let (page, block) = page_and_block(&document);
        self.document = document;
        self.path = path;
        self.page = page;
        self.block = block;
        self.selected.clear();
        self.drag = None;
        self.clipboard.clear();
        self.pressed_valve = None;
        self.stop();
        self.dialog = None;
        self.start_open = false;
    }
}

#[cfg(test)]
mod tests;
