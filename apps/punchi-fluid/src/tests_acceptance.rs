// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The acceptance test of the student loop: Inicio, a template, an edit, a save, a reopen,
//! a simulation and an export.
//!
//! It drives the window the way a student does. The file dialogs are the only step skipped:
//! they wait for a pointer, so the test passes the path the reader would choose to the same
//! commands the dialog calls, and the dialog itself has its own tests in `files`.

use std::path::{Path, PathBuf};

use punchi_fluid_doc::Document;
use punchi_fluid_session::recents::Recents;
use punchi_fluid_session::safety::SafetyCopy;

use crate::Window;
use crate::start;
use crate::theme::Theme;

/// The PNG file signature: the eight bytes every PNG starts with.
const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

/// A folder of its own for the test: a student session never touches the reader's folder,
/// and no other test shares this one.
fn scratch() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "punchi-fluid-acceptance-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the scratch folder");
    dir
}

/// A window whose recents and safety copy live in `dir`, the way `main` builds it.
fn student_window(dir: &Path) -> Window {
    Window::with_safety(
        Theme::fallback(),
        Document::new(),
        None,
        None,
        Recents::at(Some(dir.join("recent.json"))),
        SafetyCopy::at(Some(dir.join("recovery.pfluid"))),
    )
}

/// Steps 1 and 2: the window opens on Inicio and the first template, through the page's own
/// action, becomes the document: six placements, Inicio closes and a template is a starting
/// point, not unsaved work.
fn open_the_first_template(dir: &Path) -> Window {
    let mut window = student_window(dir);
    assert!(window.start_open, "the window opens on Inicio");
    window.apply(start::Action::Template(0));
    assert_eq!(window.document.placements().len(), 6);
    assert_eq!(window.document.connections().len(), 6);
    assert!(!window.start_open, "a template closes Inicio");
    assert!(!window.is_modified(), "a template is not unsaved work");
    window
}

/// Steps 3 and 4: the student places one more component and saves. The file exists, the
/// document is clean and the safety copy the crash would have left is gone.
fn edit_and_save(window: &mut Window, dir: &Path) -> PathBuf {
    window.add_component("valve.3_2.spring_return");
    assert_eq!(window.document.placements().len(), 7);
    assert!(window.is_modified(), "an edit is unsaved work");
    let recovery = dir.join("recovery.pfluid");
    assert!(
        SafetyCopy::at(Some(recovery.clone())).write(&window.document),
        "the safety copy is written while the edit is unsaved"
    );
    assert!(recovery.is_file(), "the unsaved edit left a safety copy");
    let saved = dir.join("taller.pfluid");
    window.save_to(&saved).expect("the document is saved");
    assert!(saved.is_file(), "the save reaches the disk");
    assert!(!window.is_modified(), "a save is the saved revision");
    assert!(!recovery.exists(), "a save removes the safety copy");
    saved
}

/// Step 5: a second window reads the saved file and sees the same document, and the open is
/// not unsaved work.
fn reopen(window: &Window, dir: &Path, saved: &Path) -> Window {
    let mut reopened = student_window(dir);
    reopened.open_path(saved).expect("the saved document opens");
    assert!(!reopened.start_open, "an open closes Inicio");
    assert_eq!(
        reopened.document.placements().len(),
        7,
        "the reopen holds every placement"
    );
    assert_eq!(
        reopened.document.circuits(),
        window.document.circuits(),
        "the reopen holds the same circuit"
    );
    assert!(!reopened.is_modified(), "an open is not unsaved work");
    reopened
}

/// Step 6: the first valve of the OR circuit extends the single-acting cylinder, and the
/// clock stops when the run is over.
fn simulate(window: &mut Window) {
    window.start();
    assert!(window.running, "the clock starts when the model builds");
    let valve = window
        .document
        .placements()
        .iter()
        .position(|placement| placement.catalog_id.starts_with("valve."))
        .expect("the OR circuit has a valve");
    assert!(
        window
            .model
            .as_mut()
            .is_some_and(|model| model.press_valve(valve)),
        "the first valve is a valve the model holds"
    );
    let mut extended = false;
    for _ in 0..600 {
        window.advance(0.016);
        if window
            .cylinder_position()
            .is_some_and(|position| position > 0.0)
        {
            extended = true;
            break;
        }
    }
    assert!(
        extended,
        "holding the valve extends the single-acting cylinder"
    );
    window.stop();
    assert!(!window.running, "the clock stops");
    assert!(
        window.cylinder_position().is_none(),
        "stopping drops the model"
    );
}

/// Step 7: the student exports the same document as a PDF and as a PNG. An export is not a
/// save: the modified state, the document's path and the safety copy stay untouched.
fn export(window: &mut Window, dir: &Path, saved: &Path) {
    window.add_component("valve.5_2.spring_return");
    assert!(
        window.is_modified(),
        "the export is made while the document is modified"
    );
    let recovery = dir.join("recovery.pfluid");
    assert!(
        SafetyCopy::at(Some(recovery.clone())).write(&window.document),
        "the edited document has a safety copy"
    );
    assert!(
        recovery.is_file(),
        "the safety copy is on disk before the export"
    );

    let pdf = dir.join("circuito.pdf");
    window.export_to(&pdf).expect("the PDF export writes");
    let png = dir.join("circuito.png");
    window.export_to(&png).expect("the PNG export writes");
    let pdf_bytes = std::fs::read(&pdf).expect("the PDF is readable");
    assert!(
        pdf_bytes.starts_with(b"%PDF-"),
        "the PDF export starts with the PDF signature"
    );
    let png_bytes = std::fs::read(&png).expect("the PNG is readable");
    assert!(
        png_bytes.starts_with(&PNG_SIGNATURE),
        "the PNG export starts with the PNG signature"
    );
    assert!(
        window.is_modified(),
        "an export does not mark the document saved"
    );
    assert_eq!(
        window.path.as_deref(),
        Some(saved),
        "an export does not change the document's path"
    );
    assert!(
        recovery.is_file(),
        "an export does not touch the safety copy"
    );
}

#[test]
fn the_student_loop_from_the_start_page_to_an_export() {
    let dir = scratch();

    // 1 and 2: Inicio and the first start template.
    let mut window = open_the_first_template(&dir);

    // 3 and 4: the edit and the save.
    let saved = edit_and_save(&mut window, &dir);

    // 5 and 6: the reopen in a second window, and the simulation.
    let mut reopened = reopen(&window, &dir, &saved);
    simulate(&mut reopened);

    // 7: the PDF and the PNG, with the document's state untouched.
    export(&mut reopened, &dir, &saved);

    // 8: the recent list remembers the document the student saved, on disk as well.
    let remembered = Recents::at(Some(dir.join("recent.json")));
    assert_eq!(
        remembered.entries().len(),
        1,
        "the saved document is the only remembered one"
    );
    assert_eq!(
        remembered.entries()[0].path,
        saved.to_string_lossy(),
        "the remembered document is the saved one"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
