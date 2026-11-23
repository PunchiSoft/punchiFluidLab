// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The file dialog's tests: its naming rules, the overwrite question, Esc, the keyboard, and
//! the saves and opens that reach the document and the recent list. The dialog is driven
//! headless, like the ribbon and the start page.

use std::path::{Path, PathBuf};

use eframe::egui;
use egui_kittest::kittest::Queryable;
use punchi_fluid_doc::{Document, Placement};
use punchi_fluid_draw::Pt;
use punchi_fluid_file::sheet::Sheet;
use punchi_fluid_session::recents::Recents;

use super::{Dialog, Mode, Outcome};
use crate::Window;
use crate::theme::Theme;

/// A folder of its own for one test: the tests run in parallel, and two of them must never
/// share a recent list or a document.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "punchi-fluid-files-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the scratch folder");
    dir
}

/// A window whose recent list points at a test file instead of the reader's real one.
fn window_with(path: Option<PathBuf>, recents: Recents) -> Window {
    Window::with_recents(Theme::fallback(), Document::new(), path, None, recents)
}

/// The dialog drawn headless at the window's floor.
fn harness(window: &mut Window) -> egui_kittest::Harness<'_> {
    let mut harness = egui_kittest::Harness::new(move |context| window.show_files(context));
    harness.set_size(egui::vec2(1366.0, 768.0));
    // The window ships no font: the harness gets the desktop's, as the app does at start.
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the file-dialog tests need one");
    }
    harness.run();
    harness
}

/// A saved document with one placement, so a reopened file is recognisably its own.
fn document_with_a_valve(path: &Path) {
    let mut document = Document::new();
    document.place(Placement::new(
        "valve.5_2",
        Pt::new(100.0, 80.0),
        24.0,
        24.0,
    ));
    punchi_fluid_session::save(&document, path).expect("the fixture document is saved");
}

#[test]
fn the_save_dialog_refuses_an_empty_name_and_appends_the_extension() {
    let dir = scratch("naming");
    let mut dialog = Dialog::open(Mode::SaveAs, dir.clone());
    assert_eq!(dialog.confirm(), None, "an empty name is refused");
    assert!(
        dialog
            .error
            .as_deref()
            .is_some_and(|error| error.to_lowercase().contains("nombre")),
        "the dialog says a name is needed: {:?}",
        dialog.error
    );

    dialog.set_name("   ");
    assert_eq!(dialog.confirm(), None, "blank is empty");

    dialog.set_name("taller");
    assert_eq!(
        dialog.confirm(),
        Some(Outcome::Save(dir.join("taller.pfluid"))),
        "a name without an extension gets the sheet's"
    );

    dialog.set_name("taller.pfluid");
    assert_eq!(
        dialog.confirm(),
        Some(Outcome::Save(dir.join("taller.pfluid"))),
        "an extension already given is kept once"
    );

    dialog.set_name("con/carpeta");
    assert_eq!(dialog.confirm(), None, "a name cannot carry a folder");
    assert!(
        dialog
            .error
            .as_deref()
            .is_some_and(|error| error.contains("carpeta")),
        "the dialog explains the refusal: {:?}",
        dialog.error
    );

    let mut open = Dialog::open(Mode::Open, dir.clone());
    assert_eq!(open.confirm(), None, "open needs a chosen document");
    assert!(
        open.error
            .as_deref()
            .is_some_and(|error| error.contains("documento")),
        "the open dialog asks for a document: {:?}",
        open.error
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_save_asks_once_before_replacing_an_existing_document() {
    let dir = scratch("overwrite");
    let target = dir.join("existe.pfluid");
    std::fs::write(&target, "{}").expect("the existing file");
    let mut dialog = Dialog::open(Mode::SaveAs, dir.clone());
    dialog.set_name("existe");
    assert_eq!(dialog.confirm(), None, "the first confirmation only asks");
    assert!(
        dialog
            .error
            .as_deref()
            .is_some_and(|error| error.contains("reemplaz")),
        "the dialog says the next click replaces it: {:?}",
        dialog.error
    );
    assert_eq!(
        dialog.confirm(),
        Some(Outcome::Save(target)),
        "the second confirmation replaces"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_save_from_the_dialog_writes_the_document_and_remembers_it() {
    let dir = scratch("save");
    let recents_file = dir.join("recent.json");
    let base = dir.join("base.pfluid");
    let mut window = window_with(Some(base), Recents::at(Some(recents_file.clone())));
    window.open_dialog(Mode::SaveAs);
    {
        let dialog = window.dialog.as_mut().expect("the dialog is open");
        assert_eq!(dialog.mode, Mode::SaveAs);
        assert_eq!(dialog.directory, dir, "it starts beside the document");
        assert_eq!(dialog.confirm(), None, "an empty name is refused");
        dialog.set_name("taller");
        let outcome = dialog.confirm().expect("the name is confirmed");
        assert_eq!(outcome, Outcome::Save(dir.join("taller.pfluid")));
        window.apply_files(outcome);
    }
    let saved = dir.join("taller.pfluid");
    assert!(
        window.dialog.is_none(),
        "a successful save closes the dialog"
    );
    assert_eq!(window.path.as_deref(), Some(saved.as_path()));
    assert!(saved.is_file(), "the dialog wrote the document");
    let text = std::fs::read_to_string(&saved).expect("the saved text");
    assert!(Sheet::parse(&text).is_ok(), "the file is a sheet");
    let recents = Recents::at(Some(recents_file));
    assert_eq!(recents.entries().len(), 1, "the save is remembered");
    assert_eq!(recents.entries()[0].path, saved.to_string_lossy());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_open_failure_keeps_the_dialog_open_with_the_reason() {
    let dir = scratch("open-failure");
    let broken = dir.join("roto.pfluid");
    std::fs::write(&broken, "{ esto no es json").expect("the broken file");
    let mut window = window_with(
        Some(dir.join("base.pfluid")),
        Recents::at(Some(dir.join("recent.json"))),
    );
    window.open_dialog(Mode::Open);
    window.apply_files(Outcome::Open(broken));
    assert!(
        window.dialog.is_some(),
        "the dialog stays where the reader was"
    );
    assert!(
        window
            .dialog
            .as_ref()
            .and_then(|dialog| dialog.error.as_deref())
            .is_some_and(|error| error.contains("no se pudo abrir")),
        "the dialog shows the error instead of crashing"
    );
    assert!(
        window
            .path
            .as_deref()
            .is_some_and(|path| path.ends_with("base.pfluid")),
        "the document did not change"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_open_dialog_lists_its_folder_and_opens_the_chosen_document() {
    let dir = scratch("open");
    let saved = dir.join("circuito.pfluid");
    document_with_a_valve(&saved);
    std::fs::write(dir.join("notas.txt"), "no es un documento").expect("the other file");
    let recents_file = dir.join("recent.json");
    let mut window = window_with(
        Some(dir.join("nuevo.pfluid")),
        Recents::at(Some(recents_file.clone())),
    );
    window.start_open = true;
    window.open_dialog(Mode::Open);
    {
        let mut harness = harness(&mut window);
        assert!(
            harness.query_by_label_contains("circuito.pfluid").is_some(),
            "the document is listed"
        );
        assert!(
            harness.query_by_label_contains("notas.txt").is_none(),
            "a file the sheet format cannot read is not listed"
        );
        harness.get_by_label_contains("circuito.pfluid").click();
        harness.run();
        harness
            .get_by_role_and_label(egui::accesskit::Role::Button, "Abrir")
            .click();
        harness.run();
    }
    assert!(
        window.dialog.is_none(),
        "a successful open closes the dialog"
    );
    assert!(!window.start_open, "a successful open closes Inicio");
    assert_eq!(window.path.as_deref(), Some(saved.as_path()));
    assert_eq!(
        window.document.placements().len(),
        1,
        "the opened sheet's component is in the window"
    );
    let recents = Recents::at(Some(recents_file));
    assert_eq!(recents.entries().len(), 1, "the open is remembered");
    assert_eq!(recents.entries()[0].path, saved.to_string_lossy());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn escape_cancels_the_dialog() {
    let dir = scratch("escape");
    let mut window = window_with(
        Some(dir.join("base.pfluid")),
        Recents::at(Some(dir.join("recent.json"))),
    );
    window.open_dialog(Mode::Open);
    {
        let mut harness = harness(&mut window);
        harness.key_press(egui::Key::Escape);
        harness.run();
    }
    assert!(window.dialog.is_none(), "Esc closes the dialog");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_save_dialog_takes_its_name_from_the_keyboard_and_enter_writes_it() {
    let dir = scratch("keyboard");
    let recents_file = dir.join("recent.json");
    let mut window = window_with(
        Some(dir.join("base.pfluid")),
        Recents::at(Some(recents_file.clone())),
    );
    window.open_dialog(Mode::SaveAs);
    {
        let mut harness = harness(&mut window);
        harness
            .get_by_role(egui::accesskit::Role::TextInput)
            .focus();
        harness.run();
        harness
            .get_by_role(egui::accesskit::Role::TextInput)
            .type_text("teclado");
        harness.run();
        harness.key_press(egui::Key::Enter);
        harness.run();
    }
    let saved = dir.join("teclado.pfluid");
    assert!(window.dialog.is_none(), "Enter confirms the save");
    assert_eq!(window.path.as_deref(), Some(saved.as_path()));
    assert!(saved.is_file(), "the typed name reached the disk");
    assert_eq!(
        Recents::at(Some(recents_file)).entries().len(),
        1,
        "a keyboard save is remembered as well"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn enter_on_the_save_button_asks_before_replacing_and_a_second_enter_replaces() {
    let dir = scratch("enter-overwrite");
    let target = dir.join("existe.pfluid");
    std::fs::write(&target, "{}").expect("the existing file");
    let recents_file = dir.join("recent.json");
    let mut window = window_with(
        Some(dir.join("base.pfluid")),
        Recents::at(Some(recents_file)),
    );
    window.open_dialog(Mode::SaveAs);
    {
        let mut harness = harness(&mut window);
        harness
            .get_by_role(egui::accesskit::Role::TextInput)
            .focus();
        harness.run();
        harness
            .get_by_role(egui::accesskit::Role::TextInput)
            .type_text("existe");
        harness.run();
        // `Enter` on the focused button is the button's answer: one press, one question.
        harness
            .get_by_role_and_label(egui::accesskit::Role::Button, "Guardar")
            .focus();
        harness.run();
        harness.key_press(egui::Key::Enter);
        harness.run();
    }
    assert!(
        window.dialog.is_some(),
        "the first Enter asks before replacing, it does not replace"
    );
    assert!(
        window
            .dialog
            .as_ref()
            .and_then(|dialog| dialog.error.as_deref())
            .is_some_and(|error| error.contains("reemplaz")),
        "the dialog says the next Enter replaces it"
    );
    {
        let mut harness = harness(&mut window);
        harness.key_press(egui::Key::Enter);
        harness.run();
    }
    assert!(window.dialog.is_none(), "the second Enter replaces");
    assert_eq!(window.path.as_deref(), Some(target.as_path()));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_subfolder_row_enters_it_and_lists_its_documents() {
    let dir = scratch("folders");
    let subfolder = dir.join("proyectos");
    std::fs::create_dir_all(&subfolder).expect("the subfolder");
    document_with_a_valve(&subfolder.join("dentro.pfluid"));
    let mut window = window_with(
        Some(dir.join("base.pfluid")),
        Recents::at(Some(dir.join("recent.json"))),
    );
    window.open_dialog(Mode::Open);
    {
        let mut harness = harness(&mut window);
        harness.get_by_label_contains("proyectos/").click();
        harness.run();
        assert!(
            harness.query_by_label_contains("dentro.pfluid").is_some(),
            "the subfolder is listed"
        );
    }
    assert_eq!(
        window
            .dialog
            .as_ref()
            .map(|dialog| dialog.directory.clone()),
        Some(subfolder),
        "clicking a subfolder enters it"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
