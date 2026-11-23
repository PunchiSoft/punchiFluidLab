// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The window tests: the document, the selection and the drawing, without a frame.

use eframe::egui;
use punchi_fluid_doc::{Align, Document, Endpoint, Placement};
use punchi_fluid_draw::surface::Recording;

use super::{Drag, Pt, Window, sheet, theme};

#[test]
fn a_click_selects_the_placement_under_it_and_clears_the_rest() {
    let mut document = Document::new();
    document.place(Placement::new(
        "valve.5_2",
        Pt::new(100.0, 80.0),
        24.0,
        24.0,
    ));
    let mut window = Window::new(theme::Theme::fallback(), document, None, None);
    assert_eq!(window.select_at(Pt::new(101.0, 81.0), false), Some(0));
    assert_eq!(window.selected, vec![0]);
    assert_eq!(window.select_at(Pt::new(10.0, 10.0), false), None);
    assert!(window.selected.is_empty());
}

#[test]
fn shift_click_toggles_and_select_all_takes_the_whole_circuit() {
    let mut document = Document::new();
    document.place(Placement::new(
        "valve.3_2.spring_return",
        Pt::new(60.0, 60.0),
        20.0,
        20.0,
    ));
    document.place(Placement::new(
        "valve.3_2.spring_return",
        Pt::new(120.0, 60.0),
        20.0,
        20.0,
    ));
    let mut window = Window::new(theme::Theme::fallback(), document, None, None);
    assert_eq!(window.select_at(Pt::new(60.0, 60.0), false), Some(0));
    assert_eq!(window.select_at(Pt::new(120.0, 60.0), true), Some(1));
    assert_eq!(window.selected, vec![0, 1]);
    assert_eq!(window.select_at(Pt::new(60.0, 60.0), true), Some(0));
    assert_eq!(window.selected, vec![1]);
    window.select_all();
    assert_eq!(window.selected, vec![0, 1]);
}

#[test]
fn duplicating_the_selection_is_one_undo_step() {
    let mut document = Document::new();
    document.place(Placement::new(
        "valve.3_2.spring_return",
        Pt::new(60.0, 60.0),
        20.0,
        20.0,
    ));
    let mut window = Window::new(theme::Theme::fallback(), document, None, None);
    let _ = window.select_at(Pt::new(60.0, 60.0), false);
    window.duplicate_selected();
    assert_eq!(window.document.placements().len(), 2);
    assert_eq!(window.selected, vec![1]);
    window.undo();
    assert_eq!(
        window.document.placements().len(),
        1,
        "one undo removes the copy"
    );
}

#[test]
fn copying_and_pasting_is_one_undo_step() {
    let mut document = Document::new();
    document.place(Placement::new(
        "valve.3_2.spring_return",
        Pt::new(60.0, 60.0),
        20.0,
        20.0,
    ));
    let mut window = Window::new(theme::Theme::fallback(), document, None, None);
    let _ = window.select_at(Pt::new(60.0, 60.0), false);
    window.copy_selected();
    window.paste();
    assert_eq!(window.document.placements().len(), 2);
    assert_eq!(window.selected, vec![1]);
    window.undo();
    assert_eq!(
        window.document.placements().len(),
        1,
        "one undo removes the paste"
    );
}

#[test]
fn cutting_removes_the_selection_and_the_paste_brings_it_back() {
    let mut document = Document::new();
    document.place(Placement::new(
        "valve.3_2.spring_return",
        Pt::new(60.0, 60.0),
        20.0,
        20.0,
    ));
    let mut window = Window::new(theme::Theme::fallback(), document, None, None);
    let _ = window.select_at(Pt::new(60.0, 60.0), false);
    window.cut_selected();
    assert!(window.document.placements().is_empty());
    assert!(!window.clipboard.is_empty(), "the cut keeps the clipboard");
    window.paste();
    assert_eq!(window.document.placements().len(), 1);
    window.undo();
    assert!(
        window.document.placements().is_empty(),
        "undo takes the paste back to the cut"
    );
}

#[test]
fn aligning_the_selection_uses_the_furthest_edge() {
    let mut document = Document::new();
    document.place(Placement::new(
        "valve.3_2.spring_return",
        Pt::new(60.0, 60.0),
        20.0,
        20.0,
    ));
    document.place(Placement::new(
        "valve.3_2.spring_return",
        Pt::new(120.0, 90.0),
        20.0,
        20.0,
    ));
    let mut window = Window::new(theme::Theme::fallback(), document, None, None);
    window.select_all();
    window.align_selected(Align::Left);
    let lefts: Vec<f32> = window
        .document
        .placements()
        .iter()
        .map(|placement| placement.rect().min.x)
        .collect();
    assert!((lefts[0] - lefts[1]).abs() < 0.01);
    window.undo();
    let restored: Vec<f32> = window
        .document
        .placements()
        .iter()
        .map(|placement| placement.rect().min.x)
        .collect();
    assert!(
        (restored[0] - restored[1]).abs() > 0.01,
        "one undo restores the alignment"
    );
}

#[test]
fn a_drag_commit_moves_the_selection_and_leaves_one_undo_step() {
    let mut document = Document::new();
    document.place(Placement::new(
        "valve.5_2",
        Pt::new(100.0, 80.0),
        24.0,
        24.0,
    ));
    let mut window = Window::new(theme::Theme::fallback(), document, None, None);
    let _ = window.select_at(Pt::new(100.0, 80.0), false);
    window.drag = Some(Drag {
        pointer: egui::pos2(0.0, 0.0),
        starts: vec![(0, Pt::new(100.0, 80.0))],
        delta: egui::Vec2::ZERO,
    });
    window.commit_drag(1.0, egui::pos2(20.0, 10.0));
    let moved = window.document.placements()[0].centre;
    assert!((moved.x - 120.0).abs() < 0.01);
    assert!((moved.y - 90.0).abs() < 0.01);
    assert!(window.document.can_undo(), "one drag, one undo entry");
    assert!(window.drag.is_none());
}

#[test]
fn adding_a_component_places_it_at_the_sheet_centre_as_one_undo_step() {
    let mut window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
    let centre = window.page.rect().center();
    window.add_component("valve.3_2.spring_return");
    assert_eq!(window.document.placements().len(), 1);
    let placed = window.document.placements()[0].centre;
    assert!((placed.x - centre.x).abs() < 0.01);
    assert!((placed.y - centre.y).abs() < 0.01);
    assert_eq!(window.selected, vec![0]);
    window.undo();
    assert!(window.document.placements().is_empty());
}

#[test]
fn deleting_the_selection_is_one_undo_step() {
    let mut document = Document::new();
    document.place(Placement::new(
        "valve.5_2",
        Pt::new(100.0, 80.0),
        24.0,
        24.0,
    ));
    let mut window = Window::new(theme::Theme::fallback(), document, None, None);
    let _ = window.select_at(Pt::new(100.0, 80.0), false);
    window.remove_selected();
    assert!(window.document.placements().is_empty());
    assert!(window.selected.is_empty());
    window.undo();
    assert_eq!(
        window.document.placements().len(),
        1,
        "one undo brings it back"
    );
}

#[test]
fn turning_and_reflecting_the_selection_both_undo() {
    let mut document = Document::new();
    document.place(Placement::new(
        "valve.5_2",
        Pt::new(100.0, 80.0),
        24.0,
        24.0,
    ));
    let before = document.placements()[0].orientation;
    let mut window = Window::new(theme::Theme::fallback(), document, None, None);
    let _ = window.select_at(Pt::new(100.0, 80.0), false);
    window.rotate_selected();
    assert_ne!(window.document.placements()[0].orientation, before);
    window.undo();
    assert_eq!(window.document.placements()[0].orientation, before);
    let _ = window.select_at(Pt::new(100.0, 80.0), false);
    window.reflect_selected();
    assert_ne!(window.document.placements()[0].orientation, before);
    window.undo();
    assert_eq!(window.document.placements()[0].orientation, before);
}

#[test]
fn the_window_saves_the_document_it_was_given() {
    let path = std::env::temp_dir().join(format!(
        "punchi-fluid-window-save-{}.pfluid",
        std::process::id()
    ));
    let mut window = Window::new(
        theme::Theme::fallback(),
        Document::new(),
        Some(path.clone()),
        None,
    );
    window.save();
    let text = std::fs::read_to_string(&path).expect("the window wrote the file");
    assert!(
        punchi_fluid_file::sheet::Sheet::parse(&text).is_ok(),
        "the saved text is a sheet"
    );
    assert!(
        window
            .notice
            .as_deref()
            .is_some_and(|notice| notice.starts_with("guardado")),
        "the status bar says the save happened"
    );
    assert_eq!(
        window.recents.entries().len(),
        1,
        "a successful save is remembered"
    );
    assert_eq!(window.recents.entries()[0].path, path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
}

#[test]
fn nuevo_clears_the_path_and_starts_an_empty_document() {
    let mut document = Document::new();
    document.place(Placement::new(
        "valve.5_2",
        Pt::new(100.0, 80.0),
        24.0,
        24.0,
    ));
    let path = std::env::temp_dir().join(format!(
        "punchi-fluid-window-nuevo-{}.pfluid",
        std::process::id()
    ));
    let mut window = Window::new(theme::Theme::fallback(), document, Some(path), None);
    window.new_document();
    assert!(window.path.is_none(), "the new document has no path");
    assert!(
        window.document.placements().is_empty(),
        "the new document is empty"
    );
    assert!(!window.start_open, "Nuevo leaves the sheet in view");
    assert_eq!(window.notice.as_deref(), Some("documento nuevo"));
}

#[test]
fn guardar_without_a_path_asks_for_one() {
    let mut window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
    window.save();
    assert!(
        window.dialog.is_some(),
        "a document with no path opens the save dialog instead of failing"
    );
}

#[test]
fn ctrl_n_starts_a_new_document_and_ctrl_o_opens_the_file_dialog() {
    let dir = std::env::temp_dir().join(format!(
        "punchi-fluid-window-shortcuts-{}",
        std::process::id()
    ));
    let base = dir.join("base.pfluid");
    let mut window = Window::new(theme::Theme::fallback(), Document::new(), Some(base), None);
    let context = egui::Context::default();
    let _ = context.run(shortcut(egui::Key::N), |context| {
        window.sync_commands(context);
    });
    assert!(
        window.path.is_none(),
        "Ctrl+N starts a new document with no path"
    );
    assert_eq!(window.notice.as_deref(), Some("documento nuevo"));
    let _ = context.run(shortcut(egui::Key::O), |context| {
        window.sync_commands(context);
    });
    assert!(
        window.dialog.is_some(),
        "Ctrl+O opens the file dialog, as the ribbon and Inicio hint"
    );
}

/// One frame of raw input with `Ctrl` and `key` pressed.
fn shortcut(key: egui::Key) -> egui::RawInput {
    let mut input = egui::RawInput {
        modifiers: egui::Modifiers::CTRL,
        ..Default::default()
    };
    input.events.push(egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::CTRL,
    });
    input
}

#[test]
fn the_sheet_of_a_new_document_draws_into_a_recording() {
    let window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
    let mut recording = Recording::default();
    sheet::draw(&mut recording, &window.page, &window.block);
    assert!(
        !recording.is_empty(),
        "the sheet must produce calls: {}",
        recording.len()
    );
}
#[test]
fn the_transport_runs_the_model_the_session_builds() {
    let mut document = Document::new();
    let supply = document.place(Placement::new(
        "supply.compressor",
        Pt::new(60.0, 100.0),
        20.0,
        20.0,
    ));
    let valve = document.place(Placement::new(
        "valve.5_2.spring_return",
        Pt::new(120.0, 100.0),
        20.0,
        20.0,
    ));
    let cylinder = document.place(Placement::new(
        "cylinder.double_acting_32_200",
        Pt::new(200.0, 100.0),
        44.0,
        20.0,
    ));
    let supply_id = document.placements()[supply].id();
    let valve_id = document.placements()[valve].id();
    let cylinder_id = document.placements()[cylinder].id();
    document.connect(
        Endpoint::new(supply_id, "1"),
        Endpoint::new(valve_id, "1"),
        None,
    );
    document.connect(
        Endpoint::new(valve_id, "2"),
        Endpoint::new(cylinder_id, "1"),
        None,
    );
    document.connect(
        Endpoint::new(valve_id, "4"),
        Endpoint::new(cylinder_id, "2"),
        None,
    );
    let mut window = Window::new(theme::Theme::fallback(), document, None, None);
    window.start();
    assert!(window.running);
    assert!(window.model.is_some());
    window.advance(0.01);
    let clock = window
        .model
        .as_ref()
        .map_or(0.0, |model| model.network().telemetry().time);
    assert!(clock > 0.0);
    window.stop();
    assert!(!window.running);
    assert!(window.model.is_none());
}

#[test]
fn an_empty_sheet_does_not_start_the_clock() {
    let mut window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
    window.start();
    assert!(!window.running);
    assert!(window.model.is_none());
}

#[test]
fn until_change_stops_at_the_piston_end() {
    let mut document = Document::new();
    let supply = document.place(Placement::new(
        "supply.compressor",
        Pt::new(60.0, 100.0),
        20.0,
        20.0,
    ));
    let valve = document.place(Placement::new(
        "valve.5_2.spring_return",
        Pt::new(120.0, 100.0),
        20.0,
        20.0,
    ));
    let cylinder = document.place(Placement::new(
        "cylinder.double_acting_32_200",
        Pt::new(200.0, 100.0),
        44.0,
        20.0,
    ));
    let supply_id = document.placements()[supply].id();
    let valve_id = document.placements()[valve].id();
    let cylinder_id = document.placements()[cylinder].id();
    document.connect(
        Endpoint::new(supply_id, "1"),
        Endpoint::new(valve_id, "1"),
        None,
    );
    document.connect(
        Endpoint::new(valve_id, "2"),
        Endpoint::new(cylinder_id, "1"),
        None,
    );
    document.connect(
        Endpoint::new(valve_id, "4"),
        Endpoint::new(cylinder_id, "2"),
        None,
    );
    let mut window = Window::new(theme::Theme::fallback(), document, None, None);
    window.start();
    assert!(window.model.is_some());
    window.until_change();
    assert!(
        window
            .notice
            .as_deref()
            .is_some_and(|notice| notice.contains("tope")),
        "the piston reached an end: {:?}",
        window.notice
    );
    let position = window
        .model
        .as_ref()
        .map_or(0.0, |model| model.network().telemetry().position);
    assert!(position > 0.0, "the piston moved towards the far end");
}
