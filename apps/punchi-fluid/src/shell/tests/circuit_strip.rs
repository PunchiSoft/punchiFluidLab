// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The circuit strip: one tab per circuit, the home control and the view change that follows a
//! tab. The band it takes at the foot of the canvas is the layout's, tested there.

use eframe::egui;
use egui_kittest::kittest::Queryable;

use super::{sheet_harness, window_with};

#[test]
fn the_strip_lists_one_tab_per_circuit_and_the_home_control() {
    let mut window = window_with();
    window.document.add_circuit("Second circuit");
    let titles: Vec<String> = window
        .document
        .circuits()
        .iter()
        .map(|circuit| circuit.title.clone())
        .collect();
    let harness = sheet_harness(&mut window, egui::vec2(1200.0, 800.0));
    for title in &titles {
        assert!(
            harness.query_all_by_label_contains(title).count() >= 1,
            "the strip lists the circuit {title}"
        );
    }
    assert!(
        harness.query_all_by_label("Home").count() >= 1,
        "the strip keeps the home control at its left"
    );
    assert!(
        harness.query_all_by_label("New circuit").count() >= 1,
        "the plus after the last tab adds a circuit"
    );
}

#[test]
fn the_last_circuit_has_no_close_cross() {
    let mut window = window_with();
    let title = window
        .document
        .circuits()
        .first()
        .expect("the project starts with a circuit")
        .title
        .clone();
    let harness = sheet_harness(&mut window, egui::vec2(1200.0, 800.0));
    assert!(
        harness
            .query_all_by_label_contains(&format!("Close {title}"))
            .count()
            == 0,
        "the only circuit cannot be closed from its tab"
    );
    assert!(
        harness.query_all_by_label("New circuit").count() >= 1,
        "a single circuit can still add another"
    );
}

#[test]
fn the_strip_add_and_close_open_the_window_paths() {
    let mut window = window_with();
    window.document.add_circuit("Second circuit");
    let before = window.document.circuits().len();
    window.new_circuit();
    assert_eq!(
        window.document.circuits().len(),
        before + 1,
        "the plus adds a circuit as one undo"
    );
    let active = window.document.active_circuit();
    window.open_circuit_rename();
    assert!(
        window.circuit_dialog.is_some(),
        "the rename gesture opens the window's rename dialog"
    );
    window.circuit_dialog = None;
    assert_ne!(active, 0);
    window.open_circuit_close();
    assert!(
        window.circuit_dialog.is_some(),
        "the close cross opens the window's close dialog"
    );
}

#[test]
fn switching_a_circuit_is_a_view_change_without_undo() {
    let mut window = window_with();
    let second = window.document.add_circuit("Second circuit");
    let revision = window.document.revision();
    window.switch_circuit(second);
    assert_eq!(window.document.active_circuit(), second);
    assert_eq!(
        window.document.revision(),
        revision,
        "switching circuits is a view change, never an undo"
    );
    assert!(
        window.selected.is_empty(),
        "the selection belongs to the circuit in view and clears"
    );
}

#[test]
fn the_inline_editor_renames_in_the_tab() {
    let mut window = window_with();
    let id = window.document.active_circuit();
    window.strip_edit = Some(crate::circuit_strip::Edit {
        id,
        title: "Renamed circuit".to_owned(),
        focus: true,
    });
    {
        let mut harness = sheet_harness(&mut window, egui::vec2(1200.0, 800.0));
        harness.key_press(egui::Key::Enter);
        harness.run();
    }
    let title = window
        .document
        .circuits()
        .iter()
        .find(|circuit| circuit.id == id)
        .map(|circuit| circuit.title.clone());
    assert_eq!(
        title.as_deref(),
        Some("Renamed circuit"),
        "Enter commits the editor as a rename"
    );
}
