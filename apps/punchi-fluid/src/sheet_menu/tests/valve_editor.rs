// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

use eframe::egui;
use egui_kittest::kittest::Queryable;
use punchi_fluid_i18n::Language;

use super::{harness, page_centre, primary_click_at, right_click_at, window_with_component};

#[test]
fn a_double_click_opens_the_valve_editor_and_only_apply_commits() {
    let mut window = window_with_component();
    let mut harness = harness(&mut window);
    let revision = harness.state().revision;
    let centre = page_centre(&harness);
    primary_click_at(&mut harness, centre);
    assert!(!harness.state().valve_editor, "a single click only selects");
    primary_click_at(&mut harness, centre);
    assert!(
        harness.state().valve_editor,
        "double-click opens the editor"
    );
    assert!(harness.get_by_label("Designation").is_focused());
    harness.get_by_label("actuated").click();
    harness.run();
    assert_eq!(harness.state().revision, revision, "the preview is a draft");
    harness.get_by_label("Cancel").click();
    harness.run();
    assert!(!harness.state().valve_editor);
    assert_eq!(harness.state().revision, revision);

    right_click_at(&mut harness, centre);
    harness.get_by_label("Configure").click();
    harness.run();
    assert!(harness.state().valve_editor);
    harness.get_by_label("actuated").click();
    harness.run();
    harness.get_by_label("Apply").click();
    harness.run();
    assert!(!harness.state().valve_editor);
    assert_eq!(harness.state().initial_position, Some(1));
    assert_eq!(harness.state().revision, revision + 1);
}

#[test]
fn the_valve_editor_names_the_component_in_both_languages() {
    let mut window = window_with_component();
    {
        let mut harness = harness(&mut window);
        let canvas = harness.get_by_label("drawing sheet").rect();
        let centre = page_centre(&harness);
        let sheet = egui::Rect::from_center_size(centre, canvas.size());
        right_click_at(&mut harness, sheet.center());
        harness.get_by_label("Configure").click();
        harness.run();
        assert!(
            harness.query_by_label_contains("3/2 valve").is_some(),
            "the valve editor names the component in English"
        );
        harness.key_press(egui::Key::Escape);
        harness.run();
    }
    window.language = Language::Spanish;
    {
        let mut harness = harness(&mut window);
        let canvas = harness.get_by_label("hoja de dibujo").rect();
        let centre = page_centre(&harness);
        let sheet = egui::Rect::from_center_size(centre, canvas.size());
        right_click_at(&mut harness, sheet.center());
        harness.get_by_label("Configurar").click();
        harness.run();
        assert!(
            harness.query_by_label_contains("Válvula").is_some(),
            "the valve editor names the component in Spanish"
        );
    }
}
