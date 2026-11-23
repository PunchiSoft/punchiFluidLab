// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The right dock's tests: the fold and its memory, the keyboard and the reset, the
//! designation field, and what the panel names with and without a selection. The panel is
//! driven headless, like the ribbon, the start page and the file dialog.

use eframe::egui;
use egui_kittest::kittest::Queryable;
use punchi_fluid_doc::{Document, Endpoint, Placement};
use punchi_fluid_draw::LineClass;
use punchi_fluid_draw::Pt;

use super::{Action, DEFAULT_WIDTH};
use crate::Window;
use crate::ribbon::Command;
use crate::theme::Theme;

/// A document with one valve, so the panel has an instance to inspect.
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

/// The dock drawn headless at the window's floor.
///
/// The step is a sixtieth of a second, so the theme's fold needs more than one frame and the
/// tests can see it move; `run` is allowed the frames that fold takes.
fn dock_harness(window: &mut Window) -> egui_kittest::Harness<'_> {
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1366.0, 768.0))
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(240)
        .build(move |context| window.show_inspector(context));
    // The window ships no font: the harness gets the desktop's, as the app does at start.
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the inspector tests need one");
    }
    harness.run();
    harness
}

/// The keyboard path of the window, drawn without its regions: `sync_commands` is what reads
/// `Ctrl+J`.
fn key_harness(window: &mut Window) -> egui_kittest::Harness<'_> {
    let mut harness = egui_kittest::Harness::new(move |context| window.sync_commands(context));
    harness.set_size(egui::vec2(1366.0, 768.0));
    harness.run();
    harness
}

#[test]
fn the_dock_toggles_remembers_its_width_and_resets() {
    let mut window = window_with(Document::new());
    assert!(window.inspector.is_open(), "the dock starts open");
    let width = window.inspector.width();
    window.inspector.close();
    assert!(!window.inspector.is_open());
    assert!(
        (window.inspector.width() - width).abs() <= f32::EPSILON,
        "hiding keeps the width"
    );
    window.inspector.open();
    assert!(window.inspector.is_open());
    assert!(
        (window.inspector.width() - width).abs() <= f32::EPSILON,
        "showing keeps the width"
    );

    window.inspector.resize_by(40.0);
    let widened = window.inspector.width();
    assert!(widened > width, "a drag widens the dock");
    window.inspector.toggle();
    window.inspector.toggle();
    assert!(
        (window.inspector.width() - widened).abs() <= f32::EPSILON,
        "the width survives a fold"
    );

    window.inspector.reset();
    assert!(window.inspector.is_open(), "the reset opens the dock");
    assert!(
        (window.inspector.width() - DEFAULT_WIDTH).abs() <= f32::EPSILON,
        "the reset returns the preferred width"
    );
}

#[test]
fn the_properties_command_toggles_the_dock() {
    let mut window = window_with(Document::new());
    window.dispatch(Command::Properties);
    assert!(!window.inspector.is_open(), "Ver ▸ Propiedades hides it");
    window.dispatch(Command::Properties);
    assert!(
        window.inspector.is_open(),
        "Ver ▸ Propiedades brings it back"
    );
}

#[test]
fn ctrl_j_toggles_the_dock() {
    let mut window = window_with(Document::new());
    {
        let mut harness = key_harness(&mut window);
        harness.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::J);
        harness.run();
    }
    assert!(!window.inspector.is_open(), "Ctrl+J hides the dock");
    {
        let mut harness = key_harness(&mut window);
        harness.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::J);
        harness.run();
    }
    assert!(window.inspector.is_open(), "Ctrl+J brings it back");
}

#[test]
fn reset_view_opens_the_dock_and_returns_its_width() {
    let mut window = window_with(Document::new());
    window.inspector.close();
    window.inspector.resize_by(-40.0);
    window.dispatch(Command::ResetView);
    assert!(window.inspector.is_open(), "the reset opens the dock");
    assert!(
        (window.inspector.width() - DEFAULT_WIDTH).abs() <= f32::EPSILON,
        "the reset returns the preferred width"
    );
}

#[test]
fn hiding_and_showing_the_dock_is_not_a_document_change() {
    let mut window = window_with(valve_document());
    let revision = window.document.revision();
    let undo = window.document.can_undo();
    let redo = window.document.can_redo();
    window.inspector.close();
    window.inspector.open();
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
fn the_fold_keeps_the_edit_history_intact() {
    let mut window = window_with(valve_document());
    assert!(window.document.designate(0, Some("1A1")), "one edit");
    let revision = window.document.revision();
    window.inspector.close();
    window.inspector.open();
    assert_eq!(
        window.document.revision(),
        revision,
        "the fold is view state, not an edit"
    );
    assert!(window.document.undo(), "the history still holds the edit");
    assert_eq!(
        window.document.designation_of(0),
        None,
        "undo took the name back"
    );
}

#[test]
fn the_designation_field_commits_one_undo_and_an_empty_name_clears_it() {
    let mut window = window_with(valve_document());
    window.selected = vec![0];
    {
        let mut harness = dock_harness(&mut window);
        let field = harness.get_by_role(egui::accesskit::Role::TextInput);
        field.focus();
        harness.run();
        harness
            .get_by_role(egui::accesskit::Role::TextInput)
            .type_text("1A1");
        harness.run();
        harness.key_press(egui::Key::Enter);
        harness.run();
    }
    assert_eq!(
        window.document.designation_of(0),
        Some("1A1"),
        "the committed text reaches the document"
    );
    let revision = window.document.revision();
    {
        let mut harness = dock_harness(&mut window);
        harness
            .get_by_role(egui::accesskit::Role::TextInput)
            .focus();
        harness.run();
        harness.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
        harness.run();
        harness.key_press(egui::Key::Backspace);
        harness.run();
        harness.key_press(egui::Key::Enter);
        harness.run();
    }
    assert_eq!(
        window.document.designation_of(0),
        None,
        "an emptied field clears the name"
    );
    assert_eq!(
        window.document.revision(),
        revision + 1,
        "one committed edit, one undo entry"
    );
    assert!(window.document.undo(), "one undo");
    assert_eq!(
        window.document.designation_of(0),
        Some("1A1"),
        "the undo brings the name back"
    );
}

#[test]
fn the_panel_names_the_selected_component_and_hints_when_there_is_none() {
    let mut window = window_with(valve_document());
    {
        let harness = dock_harness(&mut window);
        assert!(
            harness.query_by_label("Sin selección").is_some(),
            "the empty panel says there is no selection"
        );
        assert!(
            harness
                .query_by_label_contains("Selecciona un componente")
                .is_some(),
            "the empty panel explains the next step"
        );
        assert!(
            harness.query_by_label("Circuito 1").is_some(),
            "the active circuit is named"
        );
        assert!(
            harness.query_by_label_contains("1 componentes").is_some(),
            "the circuit counts its components"
        );
        assert!(
            harness.query_by_label("valve.3_2.spring_return").is_none(),
            "nothing is named while nothing is selected"
        );
    }
    window.selected = vec![0];
    {
        let harness = dock_harness(&mut window);
        assert!(
            harness.query_by_label("valve.3_2.spring_return").is_some(),
            "the catalog id is read-only text"
        );
        assert!(
            harness.query_all_by_label_contains("Válvula 3/2").count() >= 1,
            "the component's name heads the panel"
        );
        assert!(
            harness
                .query_by_role_and_label(egui::accesskit::Role::Label, "Designación")
                .is_some(),
            "the designation field is labelled"
        );
        assert!(
            harness
                .query_by_role(egui::accesskit::Role::TextInput)
                .is_some(),
            "the designation field is offered"
        );
        assert!(
            harness.query_by_label("Puertos").is_some(),
            "the ports block is offered"
        );
        assert!(
            harness.query_by_label("suministro").is_some(),
            "the port role is named"
        );
    }
}

#[test]
fn the_rail_brings_the_dock_back_at_the_width_it_had() {
    let mut window = window_with(Document::new());
    window.inspector.resize_by(30.0);
    let width = window.inspector.width();
    {
        let mut harness = dock_harness(&mut window);
        harness
            .get_by_label("Ocultar el panel de propiedades")
            .click();
        harness.run();
        assert!(
            harness
                .query_by_label("Mostrar el panel de propiedades")
                .is_some(),
            "the closed dock leaves the rail control"
        );
        assert!(
            harness.query_by_label("Propiedades").is_none(),
            "the header is gone with the panel"
        );
        harness
            .get_by_label("Mostrar el panel de propiedades")
            .click();
        harness.run();
        assert!(
            harness
                .query_by_label("Ocultar el panel de propiedades")
                .is_some(),
            "the header is back"
        );
    }
    assert!(window.inspector.is_open(), "the rail reopens the dock");
    assert!(
        (window.inspector.width() - width).abs() <= f32::EPSILON,
        "the dock came back at its own width"
    );
}

#[test]
fn the_fold_uses_the_theme_motion_and_reduced_motion_is_instant() {
    // Reduced motion is a zero duration in the theme, and the fold is done in one frame.
    let mut window = window_with(Document::new());
    window.theme.tokens.motion_ms = 0;
    {
        let mut harness = dock_harness(&mut window);
        harness
            .get_by_label("Ocultar el panel de propiedades")
            .click();
        // One frame applies the click, the next one draws the fold.
        harness.step();
        harness.step();
    }
    assert!(
        window.inspector.shade() < 1e-3,
        "reduced motion folds in one frame: {}",
        window.inspector.shade()
    );

    // The theme's own duration animates: one frame does not reach the end.
    let mut window = window_with(Document::new());
    {
        let mut harness = dock_harness(&mut window);
        harness
            .get_by_label("Ocultar el panel de propiedades")
            .click();
        harness.step();
        harness.step();
    }
    let shade = window.inspector.shade();
    assert!(
        shade > 0.0 && shade < 1.0,
        "the fold follows the theme's motion token: {shade}"
    );
}

#[test]
fn a_designation_edit_asked_for_by_the_panel_is_one_undo() {
    let mut document = valve_document();
    let revision = document.revision();
    assert!(document.designate(0, Some("  V1  ")));
    assert_eq!(
        document.designation_of(0),
        Some("V1"),
        "the document trims the text"
    );
    assert_eq!(
        document.revision(),
        revision + 1,
        "one committed edit is one revision"
    );
    assert!(
        document.designate(0, Some("")),
        "an empty text clears the name"
    );
    assert_eq!(document.designation_of(0), None);
}

#[test]
fn the_action_of_the_panel_is_applied_through_the_document() {
    let mut window = window_with(valve_document());
    window.selected = vec![0];
    window.apply_inspector(Action::Designation {
        index: 0,
        text: String::from("1A1"),
    });
    assert_eq!(window.document.designation_of(0), Some("1A1"));
    assert_eq!(
        window.notice.as_deref(),
        Some("propiedades actualizadas"),
        "the status bar says the edit happened"
    );
}

/// A compressor piped to a valve, and the window holding it.
fn window_with_a_pipe() -> Window {
    let mut document = Document::new();
    let supply = document.place(Placement::new(
        "supply.compressor",
        Pt::new(50.0, 50.0),
        24.0,
        24.0,
    ));
    let valve = document.place(Placement::new(
        "valve.3_2.spring_return",
        Pt::new(140.0, 50.0),
        24.0,
        24.0,
    ));
    let from = document.placements()[supply].id();
    let to = document.placements()[valve].id();
    assert!(document.connect(Endpoint::new(from, "1"), Endpoint::new(to, "1"), Some(1000)));
    window_with(document)
}

#[test]
fn a_pipe_class_edit_is_one_undo() {
    let mut window = window_with_a_pipe();
    window.selected_pipe = Some(0);
    let revision = window.document.revision();
    window.apply_inspector(Action::PipeClass {
        index: 0,
        class: LineClass::Pilot,
    });
    assert_eq!(window.document.connections()[0].class(), LineClass::Pilot);
    assert_eq!(window.document.revision(), revision + 1, "one undo entry");
    assert!(window.document.undo(), "the class edit undoes");
    assert_eq!(window.document.connections()[0].class(), LineClass::Working);
}

#[test]
fn a_pipe_tube_edit_is_one_undo() {
    let mut window = window_with_a_pipe();
    window.selected_pipe = Some(0);
    let revision = window.document.revision();
    window.apply_inspector(Action::PipeTube {
        index: 0,
        tube: Some(800),
    });
    assert_eq!(
        window.document.connections()[0].tube_od_hundredths(),
        Some(800)
    );
    assert_eq!(window.document.revision(), revision + 1, "one undo entry");
    assert!(window.document.undo(), "the tube edit undoes");
    assert_eq!(
        window.document.connections()[0].tube_od_hundredths(),
        Some(1000)
    );
}

#[test]
fn the_pipe_panel_offers_its_class_and_its_tube() {
    let mut window = window_with_a_pipe();
    window.selected_pipe = Some(0);
    let harness = dock_harness(&mut window);
    assert!(
        harness
            .query_by_role_and_label(egui::accesskit::Role::ComboBox, "Clase de línea")
            .is_some(),
        "the class is offered"
    );
    assert!(
        harness
            .query_by_role_and_label(egui::accesskit::Role::ComboBox, "Diámetro exterior")
            .is_some(),
        "the tube is offered"
    );
}

#[test]
fn the_pipe_panel_edits_the_class_from_its_menu() {
    let mut window = window_with_a_pipe();
    window.selected_pipe = Some(0);
    {
        let mut harness = dock_harness(&mut window);
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Clase de línea")
            .click();
        harness.run();
        harness.get_by_label("Pilotaje").click();
        harness.run();
    }
    assert_eq!(window.document.connections()[0].class(), LineClass::Pilot);
    assert!(window.document.can_undo(), "the choice is one undo");
}

#[test]
fn the_sheet_area_follows_the_dock() {
    let mut window = window_with(Document::new());
    // The right panel is drawn before the central one, as `update` does: the sheet keeps what
    // the dock does not take.
    let mut harness = egui_kittest::Harness::new_state(
        move |context, width: &mut f32| {
            window.show_inspector(context);
            egui::CentralPanel::default().show(context, |ui| {
                *width = ui.available_width();
            });
        },
        0.0_f32,
    );
    harness.set_size(egui::vec2(1366.0, 768.0));
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the inspector tests need one");
    }
    harness.run();
    let open = *harness.state();
    harness
        .get_by_label("Ocultar el panel de propiedades")
        .click();
    harness.run();
    let closed = *harness.state();
    assert!(
        closed > open,
        "the closed dock gives its width back to the sheet: {closed} > {open}"
    );
}

#[test]
fn the_panel_controls_answer_the_keyboard() {
    let mut window = window_with(Document::new());
    {
        let mut harness = dock_harness(&mut window);
        harness
            .get_by_label("Ocultar el panel de propiedades")
            .focus();
        harness.run();
        harness.key_press(egui::Key::Enter);
        harness.run();
        assert!(
            harness
                .query_by_label("Mostrar el panel de propiedades")
                .is_some(),
            "Enter on the hide control folds the dock"
        );
        harness
            .get_by_label("Mostrar el panel de propiedades")
            .focus();
        harness.run();
        harness.key_press(egui::Key::Enter);
        harness.run();
        assert!(
            harness
                .query_by_label("Ocultar el panel de propiedades")
                .is_some(),
            "Enter on the rail brings the dock back"
        );
    }
    assert!(window.inspector.is_open());
}
