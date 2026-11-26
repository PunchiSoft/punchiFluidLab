// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The properties window's tests: it opens on request, follows the selection, keeps its edit
//! as one undo, closes with `Esc` and returns the focus, and reserves no sheet width. The
//! window is driven headless, like the ribbon, the start page and the file dialog.

use std::cell::RefCell;
use std::rc::Rc;

use eframe::egui;
use egui_kittest::kittest::Queryable;
use punchi_fluid_doc::{Document, Endpoint, Placement};
use punchi_fluid_draw::LineClass;
use punchi_fluid_draw::Pt;
use punchi_fluid_i18n::Language;

use super::{Action, DEFAULT_WIDTH, MAX_WIDTH, MIN_WIDTH};
use crate::Window;
use crate::ribbon::Command;
use crate::theme::Theme;

/// A document with one valve, so the window has an instance to inspect.
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

/// A document with two equal valves, for the selection follow-up.
fn two_valves() -> Document {
    let mut document = Document::new();
    document.place(Placement::new(
        "valve.3_2.spring_return",
        Pt::new(100.0, 80.0),
        24.0,
        24.0,
    ));
    document.place(Placement::new(
        "valve.3_2.spring_return",
        Pt::new(200.0, 80.0),
        24.0,
        24.0,
    ));
    document
}

/// A window the tests can draw: no path, no dialog, no safety copy.
fn window_with(document: Document) -> Window {
    Window::new(Theme::fallback(), document, None, None)
}

/// The properties window drawn headless at the window's floor.
fn properties_harness(window: &mut Window) -> egui_kittest::Harness<'_> {
    window.inspector.open();
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1366.0, 768.0))
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(240)
        .build(move |context| window.show_properties(context));
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the inspector tests need one");
    }
    harness.run();
    harness
}

/// The keyboard path of the window, drawn without its regions: `sync_commands` reads `Ctrl+J`.
fn key_harness(window: &mut Window) -> egui_kittest::Harness<'_> {
    let mut harness = egui_kittest::Harness::new(move |context| window.sync_commands(context));
    harness.set_size(egui::vec2(1366.0, 768.0));
    harness.run();
    harness
}

/// The sheet width with the properties window open at `width`; the window floats and must not
/// change this number.
fn sheet_width(window: &mut Window) -> f32 {
    let mut harness = egui_kittest::Harness::new_state(
        move |context, width: &mut f32| {
            window.show_catalog(context);
            window.show_properties(context);
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
    *harness.state()
}

#[test]
fn the_properties_window_starts_closed_and_toggles() {
    let mut window = window_with(Document::new());
    assert!(!window.inspector.is_open(), "the window starts closed");
    window.dispatch(Command::Properties);
    assert!(window.inspector.is_open(), "Ver ▸ Propiedades opens it");
    window.dispatch(Command::Properties);
    assert!(!window.inspector.is_open(), "the same command closes it");

    {
        let mut harness = key_harness(&mut window);
        harness.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::J);
        harness.run();
    }
    assert!(window.inspector.is_open(), "Ctrl+J opens it");
    {
        let mut harness = key_harness(&mut window);
        harness.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::J);
        harness.run();
    }
    assert!(!window.inspector.is_open(), "Ctrl+J closes it again");
}

#[test]
fn the_window_keeps_its_width_inside_the_bounds() {
    let mut window = window_with(Document::new());
    window.inspector.set_width(10.0);
    assert!((window.inspector.width() - MIN_WIDTH).abs() <= f32::EPSILON);
    window.inspector.set_width(10_000.0);
    assert!((window.inspector.width() - MAX_WIDTH).abs() <= f32::EPSILON);
    assert!(window.inspector.width() > MIN_WIDTH);
}

#[test]
fn reset_view_closes_the_window_and_returns_its_width() {
    let mut window = window_with(Document::new());
    window.inspector.open();
    window.inspector.set_width(440.0);
    window.dispatch(Command::ResetView);
    assert!(
        !window.inspector.is_open(),
        "the reset closes the properties window"
    );
    assert!(
        (window.inspector.width() - DEFAULT_WIDTH).abs() <= f32::EPSILON,
        "the reset returns the preferred width"
    );
}

#[test]
fn opening_and_closing_the_window_is_not_a_document_change() {
    let mut window = window_with(valve_document());
    let revision = window.document.revision();
    let undo = window.document.can_undo();
    let redo = window.document.can_redo();
    window.inspector.open();
    window.inspector.close();
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
fn the_window_follows_the_edit_history() {
    let mut window = window_with(valve_document());
    assert!(window.document.designate(0, Some("1A1")), "one edit");
    let revision = window.document.revision();
    window.inspector.close();
    window.inspector.open();
    assert_eq!(
        window.document.revision(),
        revision,
        "opening the window is view state, not an edit"
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
        let mut harness = properties_harness(&mut window);
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
        let mut harness = properties_harness(&mut window);
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
fn the_window_follows_the_language() {
    let mut window = window_with(Document::new());
    window.inspector.open();
    {
        let harness = properties_harness(&mut window);
        assert!(harness.query_by_label("Properties").is_some());
        assert!(harness.query_by_label("No selection").is_some());
        assert!(
            harness
                .query_by_label_contains("Select a component")
                .is_some()
        );
    }
    window.language = Language::Spanish;
    {
        let harness = properties_harness(&mut window);
        assert!(harness.query_by_label("Propiedades").is_some());
        assert!(harness.query_by_label("Sin selección").is_some());
        assert!(
            harness
                .query_by_label_contains("Selecciona un componente")
                .is_some()
        );
    }

    let mut window = window_with(valve_document());
    window.selected = vec![0];
    {
        let harness = properties_harness(&mut window);
        assert!(
            harness
                .query_all_by_label_contains("3/2 valve, spring return")
                .count()
                >= 1
        );
        assert!(
            harness
                .query_by_role_and_label(egui::accesskit::Role::Label, "Designation")
                .is_some()
        );
        assert!(harness.query_by_label("Ports").is_some());
        assert!(harness.query_by_label("supply").is_some());
    }
    window.language = Language::Spanish;
    {
        let harness = properties_harness(&mut window);
        assert!(
            harness
                .query_all_by_label_contains("Válvula 3/2, retorno por muelle")
                .count()
                >= 1
        );
        assert!(
            harness
                .query_by_role_and_label(egui::accesskit::Role::Label, "Designación")
                .is_some()
        );
        assert!(harness.query_by_label("Puertos").is_some());
        assert!(harness.query_by_label("suministro").is_some());
    }

    let mut window = window_with_a_pipe();
    window.selected_pipe = Some(0);
    {
        let harness = properties_harness(&mut window);
        assert!(harness.query_all_by_label("Hose").count() >= 1);
        assert!(
            harness
                .query_by_role_and_label(egui::accesskit::Role::ComboBox, "Line class")
                .is_some()
        );
        assert!(
            harness
                .query_by_role_and_label(egui::accesskit::Role::ComboBox, "Outside diameter")
                .is_some()
        );
    }
    window.language = Language::Spanish;
    {
        let harness = properties_harness(&mut window);
        assert!(harness.query_all_by_label("Conducto").count() >= 1);
        assert!(
            harness
                .query_by_role_and_label(egui::accesskit::Role::ComboBox, "Clase de línea")
                .is_some()
        );
        assert!(
            harness
                .query_by_role_and_label(egui::accesskit::Role::ComboBox, "Diámetro exterior")
                .is_some()
        );
    }
}

#[test]
fn the_window_names_the_selected_component_and_hints_when_there_is_none() {
    let mut window = window_with(valve_document());
    {
        let harness = properties_harness(&mut window);
        assert!(
            harness.query_by_label("No selection").is_some(),
            "the empty window says there is no selection"
        );
        assert!(
            harness
                .query_by_label_contains("Select a component")
                .is_some(),
            "the empty window explains the next step"
        );
        assert!(
            harness.query_by_label("Circuito 1").is_some(),
            "the active circuit is named"
        );
        assert!(
            harness.query_by_label("valve.3_2.spring_return").is_none(),
            "nothing is named while nothing is selected"
        );
    }
    window.selected = vec![0];
    {
        let harness = properties_harness(&mut window);
        assert!(
            harness.query_by_label("valve.3_2.spring_return").is_some(),
            "the catalog id is read-only text"
        );
        assert!(
            harness.query_all_by_label_contains("3/2 valve").count() >= 1,
            "the component's name heads the window"
        );
        assert!(
            harness
                .query_by_role_and_label(egui::accesskit::Role::Label, "Designation")
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
            harness.query_by_label("Ports").is_some(),
            "the ports block is offered"
        );
        assert!(
            harness.query_by_label("supply").is_some(),
            "the port role is named"
        );
    }
}

#[test]
fn the_window_follows_the_selection_without_a_stale_edit() {
    let window = Rc::new(RefCell::new(window_with(two_valves())));
    let live = Rc::clone(&window);
    {
        live.borrow_mut().inspector.open();
    }
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1366.0, 768.0))
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(240)
        .build_state(
            move |context, selected: &mut usize| {
                let mut window = live.borrow_mut();
                window.selected = vec![*selected];
                window.show_properties(context);
            },
            0usize,
        );
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the inspector tests need one");
    }
    harness.run();
    assert!(
        harness
            .query_all_by_label_contains("3/2 valve, spring return")
            .count()
            >= 1
    );
    // Start editing the first component and change the selection before committing: the edit
    // must reach the placement it belonged to, not the one now on screen.
    harness
        .get_by_role(egui::accesskit::Role::TextInput)
        .focus();
    harness.run();
    harness
        .get_by_role(egui::accesskit::Role::TextInput)
        .type_text("V1");
    harness.run();
    *harness.state_mut() = 1;
    harness.run();
    let window = window.borrow();
    assert_eq!(
        window.document.designation_of(0),
        None,
        "the uncommitted edit is discarded, not written over the old placement"
    );
    assert_eq!(
        window.document.designation_of(1),
        None,
        "the new placement is not touched by the old draft"
    );
    assert!(
        harness
            .query_all_by_label_contains("3/2 valve, spring return")
            .count()
            >= 1,
        "the window keeps showing the newly selected component"
    );
}

#[test]
fn properties_open_without_reserving_sheet_width() {
    let mut window = window_with(Document::new());
    let closed = sheet_width(&mut window);
    window.inspector.open();
    window.inspector.set_width(MAX_WIDTH);
    let open = sheet_width(&mut window);
    assert!(
        (open - closed).abs() < 1.0,
        "the floating window reserves no sheet width: {open} vs {closed}"
    );
}

#[test]
fn legacy_inspector_width_does_not_shrink_sheet() {
    let mut window = window_with(Document::new());
    window.inspector.open();
    window.inspector.set_width(MAX_WIDTH);
    let wide = sheet_width(&mut window);
    window.inspector.set_width(MIN_WIDTH);
    let narrow = sheet_width(&mut window);
    assert!(
        (wide - narrow).abs() < 1.0,
        "an old wide dock setting cannot shrink the sheet: {wide} vs {narrow}"
    );
}

#[test]
fn properties_close_returns_focus_to_the_sheet() {
    let window = Rc::new(RefCell::new(window_with(valve_document())));
    let live = Rc::clone(&window);
    {
        live.borrow_mut().inspector.open();
    }
    let mut harness = egui_kittest::Harness::new_state(
        move |context, focus: &mut Option<egui::Id>| {
            let mut window = live.borrow_mut();
            window.show_sheet(context);
            window.show_properties(context);
            *focus = window.sheet_focus;
        },
        None,
    );
    harness.set_size(egui::vec2(1366.0, 768.0));
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the inspector tests need one");
    }
    harness.run();
    let sheet = harness.state().expect("the sheet has a focus id");
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(
        !window.borrow().inspector.is_open(),
        "Esc closes the window"
    );
    assert_eq!(
        harness.ctx.memory(egui::Memory::focused),
        Some(sheet),
        "the focus returns to the sheet"
    );
}

#[test]
fn a_designation_edit_asked_for_by_the_window_is_one_undo() {
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
fn the_action_of_the_window_is_applied_through_the_document() {
    let mut window = window_with(valve_document());
    window.selected = vec![0];
    window.apply_inspector(Action::Designation {
        index: 0,
        text: String::from("1A1"),
    });
    assert_eq!(window.document.designation_of(0), Some("1A1"));
    assert_eq!(
        window.notice.as_deref(),
        Some("properties updated"),
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
fn the_pipe_window_offers_its_class_and_its_tube() {
    let mut window = window_with_a_pipe();
    window.selected_pipe = Some(0);
    let harness = properties_harness(&mut window);
    assert!(
        harness
            .query_by_role_and_label(egui::accesskit::Role::ComboBox, "Line class")
            .is_some(),
        "the class is offered"
    );
    assert!(
        harness
            .query_by_role_and_label(egui::accesskit::Role::ComboBox, "Outside diameter")
            .is_some(),
        "the tube is offered"
    );
}

#[test]
fn the_pipe_window_edits_the_class_from_its_menu() {
    let mut window = window_with_a_pipe();
    window.selected_pipe = Some(0);
    {
        let mut harness = properties_harness(&mut window);
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Line class")
            .click();
        harness.run();
        harness.get_by_label("Pilot").click();
        harness.run();
    }
    assert_eq!(window.document.connections()[0].class(), LineClass::Pilot);
    assert!(window.document.can_undo(), "the choice is one undo");
}
