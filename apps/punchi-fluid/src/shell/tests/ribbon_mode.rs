// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The ribbon's mode word at the start of a session.

use eframe::egui;
use egui_kittest::kittest::Queryable;
use punchi_fluid_catalog::Catalog;
use punchi_fluid_doc::Document;
use punchi_fluid_session::templates;

use crate::Window;
use crate::theme::Theme;

#[test]
fn the_ribbon_starts_in_editing_with_a_model_available() {
    let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
    let catalog = Catalog::builtin();
    window.document = templates::document(0, &catalog, window.page.rect())
        .expect("the worked example is a template");
    window.rebuild_if_stale();
    assert!(window.model.is_some(), "the circuit has a model to step");
    let harness = super::sheet_harness(&mut window, egui::vec2(1366.0, 768.0));
    assert!(
        harness.query_by_label("Editing").is_some(),
        "a model available at launch does not mean a paused simulation"
    );
    assert!(
        harness.query_by_label("Paused").is_none(),
        "the mode word does not claim a pause nobody asked for"
    );
}
