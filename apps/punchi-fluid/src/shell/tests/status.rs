// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The status bar's groups: mode, view, selection and reading from the real window state.

use eframe::egui;
use egui_kittest::kittest::Queryable;
use punchi_fluid_catalog::Catalog;
use punchi_fluid_doc::{Document, Placement};
use punchi_fluid_draw::Pt;
use punchi_fluid_i18n::Language;
use punchi_fluid_session::templates;

use crate::Window;
use crate::theme::Theme;

#[test]
fn the_status_bar_names_the_mode_and_the_selection() {
    let mut window = super::window_with();
    window.document.place(Placement::new(
        "valve.5_2",
        Pt::new(100.0, 100.0),
        24.0,
        24.0,
    ));
    window.selected = vec![0];
    let harness = super::sheet_harness(&mut window, egui::vec2(1366.0, 768.0));
    assert!(harness.query_all_by_label_contains("Editing").count() >= 1);
    assert!(harness.query_all_by_label_contains("5/2 valve").count() >= 1);
    assert!(harness.query_all_by_label_contains("Circuito 1").count() >= 1);
}

#[test]
fn a_model_reading_keeps_its_units() {
    let mut window = super::window_with();
    let catalog = Catalog::builtin();
    window.document = templates::document(0, &catalog, window.page.rect())
        .expect("the worked example is a template");
    window.rebuild_if_stale();
    let valve = window
        .document
        .placements()
        .iter()
        .position(|placement| placement.catalog_id.starts_with("valve."))
        .expect("the template has a valve");
    window.selected = vec![valve];
    let harness = super::sheet_harness(&mut window, egui::vec2(1366.0, 768.0));
    assert!(harness.query_all_by_label_contains("Port 1").count() >= 1);
    assert!(harness.query_all_by_label_contains("bar").count() >= 1);
    assert_eq!(harness.query_all_by_label_contains("—").count(), 0);
}

#[test]
fn a_multiple_selection_does_not_borrow_one_reading() {
    let mut window = super::window_with();
    let catalog = Catalog::builtin();
    window.document = templates::document(0, &catalog, window.page.rect())
        .expect("the worked example is a template");
    window.rebuild_if_stale();
    window.selected = vec![0, 1];
    let harness = super::sheet_harness(&mut window, egui::vec2(1366.0, 768.0));
    assert!(harness.query_all_by_label_contains("2 selected").count() >= 1);
    assert_eq!(harness.query_all_by_label_contains("Port 1").count(), 0);
}

#[test]
fn the_status_bar_follows_the_simulation_state() {
    let mut window = super::window_with();
    let catalog = Catalog::builtin();
    window.document = templates::document(0, &catalog, window.page.rect())
        .expect("the worked example is a template");
    window.rebuild_if_stale();
    window.start();
    {
        let harness = super::sheet_harness(&mut window, egui::vec2(1366.0, 768.0));
        assert!(harness.query_all_by_label_contains("Running").count() >= 1);
    }
    window.pause();
    {
        let harness = super::sheet_harness(&mut window, egui::vec2(1366.0, 768.0));
        assert!(harness.query_all_by_label_contains("Paused").count() >= 1);
    }
    window.stop();
    {
        let harness = super::sheet_harness(&mut window, egui::vec2(1366.0, 768.0));
        assert!(harness.query_all_by_label_contains("Editing").count() >= 1);
    }
}

#[test]
fn the_status_bar_follows_the_language() {
    fn status_harness(window: &mut Window) -> egui_kittest::Harness<'_> {
        let mut harness = egui_kittest::Harness::new(move |context| window.show_status(context));
        harness.set_size(egui::vec2(1366.0, 768.0));
        if crate::theme::install_font(&harness.ctx, None).is_err() {
            eprintln!("no system font: the status test needs one");
        }
        harness.run();
        harness
    }
    let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
    {
        let harness = status_harness(&mut window);
        assert!(harness.query_all_by_label_contains("Editing").count() >= 1);
        assert!(harness.query_all_by_label_contains("No selection").count() >= 1);
        assert!(harness.query_all_by_label_contains("—").count() >= 1);
        assert!(harness.query_all_by_label_contains("mm").count() >= 1);
        assert_eq!(harness.query_all_by_label_contains("ms draw").count(), 0);
        assert_eq!(harness.query_all_by_label_contains("last:").count(), 0);
    }
    window.language = Language::Spanish;
    {
        let harness = status_harness(&mut window);
        assert!(harness.query_all_by_label_contains("Edición").count() >= 1);
        assert!(harness.query_all_by_label_contains("Sin selección").count() >= 1);
        assert!(harness.query_all_by_label_contains("—").count() >= 1);
        assert_eq!(
            harness.query_all_by_label_contains("ms de dibujo").count(),
            0
        );
        assert_eq!(harness.query_all_by_label_contains("último:").count(), 0);
    }
}
