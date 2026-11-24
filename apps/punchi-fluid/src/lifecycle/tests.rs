// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The life outside the document's tests: the modified state, the safety copy, the recovery
//! Inicio offers, the close question and the two informational panels. Every frame is driven
//! headless, either through a bare `egui::Context` for the commands and a `egui_kittest`
//! harness for the widgets; the safety copies live in a folder of their own, never in the
//! reader's.

use std::path::{Path, PathBuf};

use eframe::egui;
use egui_kittest::kittest::Queryable;
use punchi_fluid_doc::{Document, Placement};
use punchi_fluid_draw::Pt;
use punchi_fluid_file::sheet::Sheet;
use punchi_fluid_i18n::Language;
use punchi_fluid_session::recents::Recents;
use punchi_fluid_session::safety::SafetyCopy;

use super::{Close, Panel};
use crate::Window;
use crate::theme::Theme;

/// A folder of its own for one test: the tests run in parallel, and no two may share a safety
/// copy or a document.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "punchi-fluid-lifecycle-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the scratch folder");
    dir
}

/// A document with one placement, so a recovered copy is recognisably its own.
fn valve_document() -> Document {
    let mut document = Document::new();
    document.place(Placement::new(
        "valve.5_2",
        Pt::new(100.0, 80.0),
        24.0,
        24.0,
    ));
    document
}

/// The window whose life is tested, with its safety copy in the scratch folder.
fn window_with(path: Option<PathBuf>, safety: SafetyCopy) -> Window {
    Window::with_safety(
        Theme::fallback(),
        Document::new(),
        path,
        None,
        Recents::default(),
        safety,
    )
}

/// The viewport commands one frame sent, so a test can see the title and the close.
fn frame(
    context: &egui::Context,
    window: &mut Window,
    input: egui::RawInput,
) -> Vec<egui::ViewportCommand> {
    let output = context.run(input, |context| window.lifecycle_frame(context));
    viewport_commands(&output)
}

fn viewport_commands(output: &egui::FullOutput) -> Vec<egui::ViewportCommand> {
    output
        .viewport_output
        .values()
        .flat_map(|viewport| viewport.commands.clone())
        .collect()
}

/// The titles one frame sent.
fn titles(commands: &[egui::ViewportCommand]) -> Vec<String> {
    commands
        .iter()
        .filter_map(|command| match command {
            egui::ViewportCommand::Title(title) => Some(title.clone()),
            _ => None,
        })
        .collect()
}

/// One frame of raw input that asks the viewport to close.
fn close_input() -> egui::RawInput {
    let mut input = egui::RawInput::default();
    input.viewports.insert(
        egui::ViewportId::ROOT,
        egui::ViewportInfo {
            events: vec![egui::ViewportEvent::Close],
            ..Default::default()
        },
    );
    input
}

/// The life's panels drawn headless at the window's floor, with the Inicio page under them.
fn page_harness(window: &mut Window) -> egui_kittest::Harness<'_> {
    let mut harness = egui_kittest::Harness::new(move |context| {
        window.lifecycle_frame(context);
        window.show_start(context);
    });
    harness.set_size(egui::vec2(1366.0, 768.0));
    egui_extras::install_image_loaders(&harness.ctx);
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the lifecycle tests need one");
    }
    harness.run();
    harness.run_steps(2);
    harness
}

/// Only the life's panels, with no page under them.
fn panel_harness(window: &mut Window) -> egui_kittest::Harness<'_> {
    let mut harness = egui_kittest::Harness::new(move |context| window.lifecycle_frame(context));
    harness.set_size(egui::vec2(1366.0, 768.0));
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the lifecycle tests need one");
    }
    harness.run();
    harness.run_steps(2);
    harness
}

/// Writes a safety copy holding the valve document, as a crash would have left it.
fn leave_copy(copy: &Path) -> SafetyCopy {
    let safety = SafetyCopy::at(Some(copy.to_path_buf()));
    assert!(
        safety.write(&valve_document()),
        "the fixture copy is written"
    );
    safety
}

#[test]
fn the_modified_state_follows_the_revision_and_clears_on_save() {
    let dir = scratch("modified");
    let target = dir.join("trabajo.pfluid");
    let mut window = window_with(Some(target.clone()), SafetyCopy::at(None));
    assert!(
        !window.is_modified(),
        "a document that was just opened has no unsaved work"
    );

    window.document.place(Placement::new(
        "valve.5_2",
        Pt::new(100.0, 80.0),
        24.0,
        24.0,
    ));
    assert!(window.is_modified(), "an edit is unsaved work");

    window.save();
    assert!(!window.is_modified(), "a save clears the modified state");
    assert!(target.is_file(), "the save reached the disk");

    window.document.place(Placement::new(
        "valve.5_2",
        Pt::new(150.0, 80.0),
        24.0,
        24.0,
    ));
    assert!(window.is_modified(), "the next edit marks it again");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_title_names_the_document_and_marks_unsaved_work() {
    let dir = scratch("title");
    let target = dir.join("taller.pfluid");
    let mut window = window_with(Some(target), SafetyCopy::at(None));
    let context = egui::Context::default();
    assert_eq!(
        titles(&frame(&context, &mut window, egui::RawInput::default())),
        vec![String::from("punchi-fluid — taller.pfluid")],
        "a saved document is named without a mark"
    );
    assert!(
        titles(&frame(&context, &mut window, egui::RawInput::default())).is_empty(),
        "the same title is not sent again"
    );

    window.document.place(Placement::new(
        "valve.5_2",
        Pt::new(100.0, 80.0),
        24.0,
        24.0,
    ));
    assert_eq!(
        titles(&frame(&context, &mut window, egui::RawInput::default())),
        vec![String::from("• punchi-fluid — taller.pfluid")],
        "unsaved work gains the leading mark"
    );

    // A document with no path has no name: the title says so.
    let mut untitled = window_with(None, SafetyCopy::at(None));
    assert_eq!(
        titles(&frame(&context, &mut untitled, egui::RawInput::default())),
        vec![String::from("punchi-fluid — unsaved")]
    );
    untitled.language = punchi_fluid_i18n::Language::Spanish;
    assert_eq!(
        titles(&frame(&context, &mut untitled, egui::RawInput::default())),
        vec![String::from("punchi-fluid — sin guardar")],
        "the title follows the language"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_autosave_writes_a_copy_while_modified_and_a_save_removes_it() {
    let dir = scratch("autosave");
    let copy_path = dir.join("recovery.pfluid");
    let target = dir.join("trabajo.pfluid");
    let safety = SafetyCopy::at(Some(copy_path.clone()));
    let mut window = window_with(Some(target), safety);
    let context = egui::Context::default();
    let _ = frame(&context, &mut window, egui::RawInput::default());
    assert!(
        !copy_path.exists(),
        "a document with nothing unsaved writes no copy"
    );

    window.document.place(Placement::new(
        "valve.5_2",
        Pt::new(100.0, 80.0),
        24.0,
        24.0,
    ));
    let _ = frame(&context, &mut window, egui::RawInput::default());
    assert!(
        copy_path.exists(),
        "the first modified frame writes the copy"
    );
    let text = std::fs::read_to_string(&copy_path).expect("the copy is readable");
    assert!(Sheet::parse(&text).is_ok(), "the copy is a sheet");

    window.save();
    assert!(!window.is_modified());
    assert!(!copy_path.exists(), "a save removes the copy");
    let _ = frame(&context, &mut window, egui::RawInput::default());
    assert!(
        !copy_path.exists(),
        "a saved document writes no copy after the save"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_banner_offers_the_copy_and_recovering_installs_it_without_a_path() {
    let dir = scratch("recover");
    let copy_path = dir.join("recovery.pfluid");
    let safety = leave_copy(&copy_path);
    let mut window = window_with(Some(dir.join("trabajo.pfluid")), safety);
    {
        let mut harness = page_harness(&mut window);
        assert!(
            harness.query_by_label("Unsaved work was found").is_some(),
            "the copy found at start is offered above Empezar"
        );
        assert!(
            harness.query_by_label_contains("trabajo.pfluid").is_some(),
            "the offer names the document's file when it has one"
        );
        harness.get_by_label("Recover").click();
        harness.run();
    }
    assert!(!window.start_open, "recovering closes Inicio");
    assert!(window.path.is_none(), "a recovered document has no path");
    assert_eq!(window.document.placements().len(), 1);
    assert!(
        window.is_modified(),
        "recovered work counts as unsaved work"
    );
    assert_eq!(window.notice.as_deref(), Some("recovered"));
    assert!(!window.has_recovery(), "the offer has been answered");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn discarding_the_recovery_removes_the_copy_and_hides_the_banner() {
    let dir = scratch("discard");
    let copy_path = dir.join("recovery.pfluid");
    let safety = leave_copy(&copy_path);
    let mut window = window_with(None, safety);
    {
        let mut harness = page_harness(&mut window);
        assert!(harness.query_by_label("Unsaved work was found").is_some());
        harness.get_by_label("Discard").click();
        harness.run();
        harness.step();
        assert!(
            harness.query_by_label("Unsaved work was found").is_none(),
            "the answered banner is gone"
        );
    }
    assert!(!copy_path.exists(), "discarding removes the copy");
    assert!(window.start_open, "discarding keeps the page");
    assert_eq!(window.notice.as_deref(), Some("recovery copy discarded"));
    assert!(!window.has_recovery());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_corrupt_copy_keeps_the_banner_and_reports_the_reason() {
    let dir = scratch("corrupt");
    let copy_path = dir.join("recovery.pfluid");
    std::fs::write(&copy_path, "esto no es un documento").expect("the broken copy");
    let safety = SafetyCopy::at(Some(copy_path.clone()));
    let mut window = window_with(None, safety);
    {
        let mut harness = page_harness(&mut window);
        harness.get_by_label("Recover").click();
        harness.run();
        harness.step();
        assert!(
            harness.query_by_label("Unsaved work was found").is_some(),
            "a copy that cannot be read is still offered"
        );
    }
    assert!(window.start_open, "the page stays where the reader was");
    assert!(copy_path.exists(), "the unreadable copy is not thrown away");
    assert!(
        window
            .notice
            .as_deref()
            .is_some_and(|notice| notice.contains("Could not recover")),
        "the status bar says why: {:?}",
        window.notice
    );
    assert!(window.has_recovery());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_close_question_appears_and_cancel_keeps_the_window() {
    let dir = scratch("close-cancel");
    let mut window = window_with(Some(dir.join("trabajo.pfluid")), SafetyCopy::at(None));
    window.document = valve_document();
    let context = egui::Context::default();
    let commands = frame(&context, &mut window, close_input());
    assert!(
        commands.contains(&egui::ViewportCommand::CancelClose),
        "the close is cancelled while the question is asked"
    );
    assert!(
        !commands.contains(&egui::ViewportCommand::Close),
        "the window does not close before the reader answers"
    );
    assert_eq!(window.lifecycle.panel, Some(Panel::CloseAsk));

    {
        let mut harness = panel_harness(&mut window);
        assert!(harness.query_by_label("Save and close").is_some());
        harness.get_by_label("Cancel").click();
        harness.run();
        harness.step();
        assert!(harness.query_by_label("Save and close").is_none());
    }
    assert!(
        window.lifecycle.panel.is_none(),
        "Cancelar closes the modal"
    );
    assert_eq!(window.lifecycle.close, Close::Idle);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn save_and_close_saves_the_document_and_closes_the_window() {
    let dir = scratch("close-save");
    let copy_path = dir.join("recovery.pfluid");
    let target = dir.join("trabajo.pfluid");
    let safety = SafetyCopy::at(Some(copy_path.clone()));
    let mut window = window_with(Some(target.clone()), safety);
    window.document = valve_document();
    let context = egui::Context::default();
    let _ = frame(&context, &mut window, egui::RawInput::default());
    assert!(copy_path.exists(), "the unsaved work has its copy");
    let commands = frame(&context, &mut window, close_input());
    assert!(commands.contains(&egui::ViewportCommand::CancelClose));

    {
        let mut harness = panel_harness(&mut window);
        harness.get_by_label("Save and close").click();
        harness.run();
        harness.step();
    }
    assert_eq!(
        window.lifecycle.close,
        Close::Sent,
        "the window asks to close once the save landed"
    );
    assert!(target.is_file(), "the document was saved");
    assert!(!window.is_modified());
    assert!(!copy_path.exists(), "the saved work needs no copy");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn close_without_saving_discards_the_copy_and_closes_the_window() {
    let dir = scratch("close-discard");
    let copy_path = dir.join("recovery.pfluid");
    let target = dir.join("trabajo.pfluid");
    let safety = SafetyCopy::at(Some(copy_path.clone()));
    let mut window = window_with(Some(target), safety);
    window.document = valve_document();
    let context = egui::Context::default();
    let _ = frame(&context, &mut window, egui::RawInput::default());
    assert!(copy_path.exists(), "the unsaved work has its copy");
    let commands = frame(&context, &mut window, close_input());
    assert!(commands.contains(&egui::ViewportCommand::CancelClose));

    {
        let mut harness = panel_harness(&mut window);
        harness.get_by_label("Close without saving").click();
        harness.run();
        harness.step();
    }
    assert_eq!(
        window.lifecycle.close,
        Close::Sent,
        "the window closes without saving"
    );
    assert!(
        !copy_path.exists(),
        "the work the reader threw away leaves no copy"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn save_and_close_without_a_path_waits_for_the_save_dialog() {
    let dir = scratch("close-nopath");
    let mut window = window_with(None, SafetyCopy::at(None));
    window.document = valve_document();
    let context = egui::Context::default();
    let commands = frame(&context, &mut window, close_input());
    assert!(commands.contains(&egui::ViewportCommand::CancelClose));
    {
        let mut harness = panel_harness(&mut window);
        harness.get_by_label("Save and close").click();
        harness.run();
    }
    assert!(
        window.dialog.is_some(),
        "a document with no path asks for a name before it closes"
    );
    assert_eq!(window.lifecycle.close, Close::AfterSave);
    // The reader cancels the dialog: the window stays, and the waiting close is dropped.
    window.save_cancelled();
    assert_eq!(window.lifecycle.close, Close::Idle);
    assert!(window.is_modified(), "nothing was saved");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn save_and_close_from_the_dialog_closes_after_the_name_is_chosen() {
    let dir = scratch("close-dialog-save");
    let target = dir.join("guardado.pfluid");
    let mut window = window_with(None, SafetyCopy::at(None));
    window.document = valve_document();
    let context = egui::Context::default();
    let commands = frame(&context, &mut window, close_input());
    assert!(commands.contains(&egui::ViewportCommand::CancelClose));
    {
        let mut harness = panel_harness(&mut window);
        harness.get_by_label("Save and close").click();
        harness.run();
    }
    assert!(window.dialog.is_some(), "the name is still missing");
    assert_eq!(window.lifecycle.close, Close::AfterSave);
    // The reader names the document in the dialog, and the save lands.
    window.apply_files(crate::files::Outcome::Save(target.clone()));
    let commands = frame(&context, &mut window, egui::RawInput::default());
    assert!(
        commands.contains(&egui::ViewportCommand::Close),
        "the waiting close honours the save that followed it"
    );
    assert!(
        target.is_file(),
        "the document was saved where it was named"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_lifecycle_panels_follow_the_language() {
    let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
    {
        let mut harness = page_harness(&mut window);
        harness.get_by_label("About").click();
        harness.run();
        harness.step();
        assert!(harness.query_all_by_label("About").count() >= 1);
        assert!(harness.query_all_by_label_contains("Version ").count() >= 1);
        assert!(harness.query_by_label("Close").is_some());
        harness.key_press(egui::Key::Escape);
        harness.run();
        harness.step();
        harness.get_by_label("License").click();
        harness.run();
        harness.step();
        assert!(
            harness
                .query_all_by_label_contains("GNU General Public License, version 3")
                .count()
                >= 1
        );
        harness.key_press(egui::Key::Escape);
        harness.run();
        harness.step();
    }
    window.language = Language::Spanish;
    {
        let mut harness = page_harness(&mut window);
        harness.get_by_label("Acerca de").click();
        harness.run();
        harness.step();
        assert!(harness.query_all_by_label("Acerca de").count() >= 1);
        assert!(harness.query_all_by_label_contains("Versión ").count() >= 1);
        assert!(harness.query_by_label("Cerrar").is_some());
        harness.key_press(egui::Key::Escape);
        harness.run();
        harness.step();
        harness.get_by_label("Licencia").click();
        harness.run();
        harness.step();
        assert!(
            harness
                .query_all_by_label_contains("GNU General Public License, versión 3")
                .count()
                >= 1
        );
        harness.key_press(egui::Key::Escape);
        harness.run();
        harness.step();
    }
    let dir = scratch("lang-close");
    let mut window = window_with(Some(dir.join("trabajo.pfluid")), SafetyCopy::at(None));
    window.document = valve_document();
    window.language = Language::Spanish;
    let context = egui::Context::default();
    let _ = frame(&context, &mut window, close_input());
    {
        let harness = panel_harness(&mut window);
        assert!(
            harness
                .query_all_by_label("Hay cambios sin guardar")
                .count()
                >= 1
        );
        assert!(harness.query_by_label("Guardar y cerrar").is_some());
        assert!(harness.query_by_label("Cancelar").is_some());
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn about_and_licence_open_and_escape_closes_them() {
    let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
    {
        let mut harness = page_harness(&mut window);
        harness.get_by_label("About").click();
        harness.run();
        harness.step();
        assert!(
            harness
                .query_by_label_contains(&format!("Version {}", env!("CARGO_PKG_VERSION")))
                .is_some(),
            "the About panel names the version"
        );
        assert!(harness.query_by_label("Close").is_some());
        harness.key_press(egui::Key::Escape);
        harness.run();
        harness.step();
        assert!(
            harness.query_by_label("Close").is_none(),
            "Esc closes About"
        );
    }
    assert!(window.start_open, "the page stays under the panels");
    {
        let mut harness = page_harness(&mut window);
        harness.get_by_label("License").click();
        harness.run();
        harness.step();
        assert!(
            harness
                .query_by_label_contains("GNU GENERAL PUBLIC LICENSE")
                .is_some(),
            "the Licence panel shows the licence text"
        );
        harness.key_press(egui::Key::Escape);
        harness.run();
        harness.step();
        assert!(
            harness.query_by_label("Close").is_none(),
            "Esc closes Licence"
        );
    }
}

/// The About popup's harness: the panel drawn headless, with the desktop font.
fn about_harness(window: &mut Window) -> egui_kittest::Harness<'_> {
    window.open_about();
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1024.0, 800.0))
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(120)
        .build(move |context| window.show_panels(context));
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the About tests need one");
    }
    harness.run();
    harness
}

#[test]
fn the_about_popup_carries_the_message_the_links_and_the_licence() {
    let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
    let harness = about_harness(&mut window);
    for label in [
        "Punchi Fluid Lab",
        "Unreleased development version",
        "Website",
        "Help",
        "Report a problem",
        "© 2026 PunchiSoft · GPL-3.0-or-later",
        "Read the license",
    ] {
        assert!(
            harness.query_by_label_contains(label).is_some(),
            "the About popup shows {label}"
        );
    }
    assert!(
        harness
            .query_by_label_contains("Free and open, so that a student")
            .is_some(),
        "the popup carries the message the program wanted to offer"
    );
    assert!(
        harness.query_all_by_label_contains("PunchiSoft").count() >= 1,
        "the popup names the author"
    );
    assert!(
        harness
            .query_by_label_contains("Made for the community")
            .is_some(),
        "the popup closes with the community line"
    );
}

#[test]
fn the_licence_popup_shows_the_full_text_and_closes_with_escape() {
    let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
    window.open_licence();
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1024.0, 800.0))
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(120)
        .build(move |context| window.show_panels(context));
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the About tests need one");
    }
    harness.run();
    assert!(
        harness
            .query_by_label_contains("GNU General Public License, version 3")
            .is_some(),
        "the licence popup names the licence it carries"
    );
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(
        harness
            .query_by_label_contains("GNU General Public License, version 3")
            .is_none(),
        "Escape closes the licence popup"
    );
}
