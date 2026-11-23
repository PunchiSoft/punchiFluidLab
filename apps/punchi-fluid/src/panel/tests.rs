// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The panel chrome's tests: the catalog folds and unfolds, remembers its width, resets with
//! the view, never touches the document and leaves the sheet its room. The panel is driven
//! headless, like the inspector's own tests; the right dock's tests keep proving its half.

use std::cell::Cell;
use std::rc::Rc;

use eframe::egui;
use egui_kittest::kittest::Queryable;
use punchi_fluid_doc::{Document, Placement};
use punchi_fluid_draw::Pt;

use crate::Window;
use crate::inspector;
use crate::ribbon::Command;
use crate::sidebar;
use crate::theme::Theme;

use super::Probe;

/// A document with one valve, so the dock has an instance to inspect.
fn valve_document() -> Document {
    let mut document = Document::new();
    document.place(Placement::new(
        "valve.3_2.spring_return",
        Pt::new(100.0, 80.0),
        24.0,
        24.0,
    ));
    document
}

/// A window the tests can draw: no path, no dialog, no safety copy.
fn window_with(document: Document) -> Window {
    Window::new(Theme::fallback(), document, None, None)
}

/// The catalog drawn headless at the window's floor.
///
/// The step is a sixtieth of a second, so the theme's fold needs more than one frame and the
/// tests can see it move; `run` is allowed the frames that fold takes.
fn catalog_harness(window: &mut Window) -> egui_kittest::Harness<'_> {
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1366.0, 768.0))
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(240)
        .build(move |context| window.show_catalog(context));
    // The window ships no font: the harness gets the desktop's, as the app does at start.
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the panel tests need one");
    }
    harness.run();
    harness
}

/// The catalog drawn headless with the probe the panel reports after each frame, kept in a
/// cell the test can read while the harness borrows the window.
fn catalog_probe_harness(window: &mut Window) -> (egui_kittest::Harness<'_>, Rc<Cell<Probe>>) {
    let probe = Rc::new(Cell::new(Probe::default()));
    let reported = Rc::clone(&probe);
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1366.0, 768.0))
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(240)
        .build(move |context| {
            window.show_catalog(context);
            reported.set(window.sidebar.probe());
        });
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the panel tests need one");
    }
    harness.run();
    (harness, probe)
}

/// Moves the pointer to `position` and lets the harness draw one settled frame.
fn move_pointer(harness: &mut egui_kittest::Harness<'_>, position: egui::Pos2) {
    harness
        .input_mut()
        .events
        .push(egui::Event::PointerMoved(position));
    harness.run();
}

/// The keyboard path of the window, drawn without its regions: `sync_commands` is what reads
/// the panel shortcuts.
fn key_harness(window: &mut Window) -> egui_kittest::Harness<'_> {
    let mut harness = egui_kittest::Harness::new(move |context| window.sync_commands(context));
    harness.set_size(egui::vec2(1366.0, 768.0));
    harness.run();
    harness
}

#[test]
fn ctrl_b_toggles_the_catalog() {
    let mut window = window_with(Document::new());
    assert!(window.sidebar.is_open(), "the catalog starts open");
    {
        let mut harness = key_harness(&mut window);
        harness.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::B);
        harness.run();
    }
    assert!(!window.sidebar.is_open(), "Ctrl+B hides the catalog");
    {
        let mut harness = key_harness(&mut window);
        harness.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::B);
        harness.run();
    }
    assert!(window.sidebar.is_open(), "Ctrl+B brings it back");
}

#[test]
fn the_catalog_command_toggles_the_panel() {
    let mut window = window_with(Document::new());
    window.dispatch(Command::Catalog);
    assert!(!window.sidebar.is_open(), "Ver ▸ Catálogo hides it");
    window.dispatch(Command::Catalog);
    assert!(window.sidebar.is_open(), "Ver ▸ Catálogo brings it back");
}

#[test]
fn the_panels_keep_their_own_shortcut() {
    let mut window = window_with(Document::new());
    {
        let mut harness = key_harness(&mut window);
        harness.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::B);
        harness.run();
    }
    assert!(!window.sidebar.is_open(), "Ctrl+B folds the catalog");
    assert!(
        window.inspector.is_open(),
        "Ctrl+B leaves the right dock alone"
    );
    {
        let mut harness = key_harness(&mut window);
        harness.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::J);
        harness.run();
    }
    assert!(!window.inspector.is_open(), "Ctrl+J folds the right dock");
    assert!(
        !window.sidebar.is_open(),
        "Ctrl+J leaves the catalog as it was"
    );
    {
        let mut harness = key_harness(&mut window);
        harness.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::B);
        harness.run();
    }
    assert!(window.sidebar.is_open(), "Ctrl+B still opens the catalog");
    assert!(
        !window.inspector.is_open(),
        "the dock stays folded through Ctrl+B"
    );
}

#[test]
fn the_catalog_rail_brings_it_back_at_the_width_it_had() {
    let mut window = window_with(Document::new());
    window.sidebar.resize_by(20.0);
    let width = window.sidebar.width();
    {
        let mut harness = catalog_harness(&mut window);
        harness
            .get_by_label("Ocultar el panel del catálogo")
            .click();
        harness.run();
        assert!(
            harness
                .query_by_label("Mostrar el panel del catálogo")
                .is_some(),
            "the closed catalog leaves the rail control"
        );
        assert!(
            harness.query_by_label("Catálogo").is_none(),
            "the header is gone with the panel"
        );
        harness
            .get_by_label("Mostrar el panel del catálogo")
            .click();
        harness.run();
        assert!(
            harness
                .query_by_label("Ocultar el panel del catálogo")
                .is_some(),
            "the header is back"
        );
    }
    assert!(window.sidebar.is_open(), "the rail reopens the catalog");
    assert!(
        (window.sidebar.width() - width).abs() <= f32::EPSILON,
        "the catalog came back at its own width"
    );
}

#[test]
fn the_catalog_fold_uses_the_theme_motion_and_reduced_motion_is_instant() {
    // Reduced motion is a zero duration in the theme, and the fold is done in one frame.
    let mut window = window_with(Document::new());
    window.theme.tokens.motion_ms = 0;
    {
        let mut harness = catalog_harness(&mut window);
        harness
            .get_by_label("Ocultar el panel del catálogo")
            .click();
        // One frame applies the click, the next one draws the fold.
        harness.step();
        harness.step();
    }
    assert!(
        window.sidebar.shade() < 1e-3,
        "reduced motion folds in one frame: {}",
        window.sidebar.shade()
    );

    // The theme's own duration animates: one frame does not reach the end.
    let mut window = window_with(Document::new());
    {
        let mut harness = catalog_harness(&mut window);
        harness
            .get_by_label("Ocultar el panel del catálogo")
            .click();
        harness.step();
        harness.step();
    }
    let shade = window.sidebar.shade();
    assert!(
        shade > 0.0 && shade < 1.0,
        "the fold follows the theme's motion token: {shade}"
    );
}

#[test]
fn reset_view_opens_both_panels_at_their_default_widths() {
    let mut window = window_with(Document::new());
    window.sidebar.close();
    window.inspector.close();
    window.sidebar.resize_by(-10.0);
    window.inspector.resize_by(40.0);
    window.dispatch(Command::ResetView);
    assert!(window.sidebar.is_open(), "the reset opens the catalog");
    assert!(
        (window.sidebar.width() - sidebar::DEFAULT_WIDTH).abs() <= f32::EPSILON,
        "the reset returns the catalog's preferred width"
    );
    assert!(window.inspector.is_open(), "the reset opens the right dock");
    assert!(
        (window.inspector.width() - inspector::DEFAULT_WIDTH).abs() <= f32::EPSILON,
        "the reset returns the dock's preferred width"
    );
}

#[test]
fn folding_either_panel_is_not_a_document_change() {
    let mut window = window_with(valve_document());
    let revision = window.document.revision();
    let undo = window.document.can_undo();
    let redo = window.document.can_redo();
    window.sidebar.toggle();
    window.sidebar.toggle();
    window.inspector.toggle();
    window.inspector.toggle();
    window.dispatch(Command::Catalog);
    window.dispatch(Command::Catalog);
    window.dispatch(Command::Properties);
    window.dispatch(Command::Properties);
    assert_eq!(
        window.document.revision(),
        revision,
        "the revision is untouched"
    );
    assert_eq!(window.document.can_undo(), undo, "no history entry appears");
    assert_eq!(window.document.can_redo(), redo);
}

#[test]
fn the_rail_reports_the_pointer_the_auto_hide_dwell_reads() {
    let mut window = window_with(Document::new());
    window.sidebar.close();
    let (mut harness, probe) = catalog_probe_harness(&mut window);
    assert!(!probe.get().rail_hovered, "the pointer starts away");
    move_pointer(&mut harness, egui::pos2(10.0, 20.0));
    assert!(
        probe.get().rail_hovered,
        "the pointer on the rail is reported"
    );
    move_pointer(&mut harness, egui::pos2(700.0, 400.0));
    assert!(
        !probe.get().rail_hovered,
        "the sheet is not the rail and cancels the dwell"
    );
}

#[test]
fn the_open_panel_reports_the_pointer_and_the_keyboard_inside_it() {
    let mut window = window_with(Document::new());
    let (mut harness, probe) = catalog_probe_harness(&mut window);
    move_pointer(&mut harness, egui::pos2(100.0, 100.0));
    assert!(
        probe.get().pointer_inside,
        "the pointer inside the open panel is reported"
    );
    move_pointer(&mut harness, egui::pos2(700.0, 400.0));
    assert!(
        !probe.get().pointer_inside,
        "the pointer on the sheet starts the linger"
    );
    harness
        .get_by_role(egui::accesskit::Role::TextInput)
        .focus();
    harness.run();
    assert!(
        probe.get().focus_inside,
        "the search field has the keyboard, so the close is suspended"
    );
}

#[test]
fn the_catalog_content_stays_in_the_open_panel() {
    let mut window = window_with(Document::new());
    let harness = catalog_harness(&mut window);
    assert!(
        harness
            .query_by_role(egui::accesskit::Role::TextInput)
            .is_some(),
        "the search is offered"
    );
    assert!(
        harness.query_by_label_contains("componentes").is_some(),
        "the header counts the entries"
    );
    assert!(
        harness.query_by_label_contains("Alimentación").is_some(),
        "the categories are there"
    );
    assert!(
        harness.query_by_label("Catálogo integrado").is_some(),
        "the footer is there"
    );
}

#[test]
fn the_sheet_area_follows_the_catalog() {
    let mut window = window_with(Document::new());
    // Both panels are drawn before the central one, as `update` does: the sheet keeps what
    // the panels do not take.
    let mut harness = egui_kittest::Harness::new_state(
        move |context, width: &mut f32| {
            window.show_catalog(context);
            window.show_inspector(context);
            egui::CentralPanel::default().show(context, |ui| {
                *width = ui.available_width();
            });
        },
        0.0_f32,
    );
    harness.set_size(egui::vec2(1366.0, 768.0));
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the panel tests need one");
    }
    harness.run();
    let open = *harness.state();
    harness
        .get_by_label("Ocultar el panel del catálogo")
        .click();
    harness.run();
    let closed = *harness.state();
    assert!(
        closed > open,
        "the closed catalog gives its width back to the sheet: {closed} > {open}"
    );
    harness
        .get_by_label("Mostrar el panel del catálogo")
        .click();
    harness.run();
    let reopened = *harness.state();
    assert!(
        (reopened - open).abs() < 1.0,
        "the rail gives the room back to the catalog: {reopened} != {open}"
    );
}
