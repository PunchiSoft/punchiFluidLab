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
