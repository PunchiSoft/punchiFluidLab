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
use crate::ribbon::Command;
use crate::theme::{Appearance, Theme, ThemeId};

/// The status bar alone, so a query cannot match the catalog's copy of a component name.
fn status_harness(window: &mut Window, size: egui::Vec2) -> egui_kittest::Harness<'_> {
    let mut harness = egui_kittest::Harness::new(move |context| window.show_status(context));
    harness.set_size(size);
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the status test needs one");
    }
    harness.run();
    harness
}

/// The status bar with `theme` applied to the context, so the tokens and the text sizes the
/// bar uses are the same ones the labels inherit.
fn themed_status_harness(
    window: &mut Window,
    theme: Theme,
    size: egui::Vec2,
) -> egui_kittest::Harness<'_> {
    let mut harness = egui_kittest::Harness::new(move |context| {
        theme.apply(context);
        window.show_status(context);
    });
    harness.set_size(size);
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the status test needs one");
    }
    harness.run();
    harness
}

/// A 60-character circuit title: long enough to take most of the status bar at 900 points.
const LONG_CIRCUIT: &str = "Circuito de prueba con un nombre deliberadamente largo 12345";

#[test]
fn the_status_bar_names_the_mode_and_the_selection() {
    let mut window = super::window_with();
    window.document.place(Placement::new(
        "valve.5_2.base",
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
    let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
    {
        let harness = status_harness(&mut window, egui::vec2(1366.0, 768.0));
        assert!(harness.query_all_by_label_contains("Editing").count() >= 1);
        assert!(harness.query_all_by_label_contains("No selection").count() >= 1);
        assert!(harness.query_all_by_label_contains("—").count() >= 1);
        assert!(harness.query_all_by_label_contains("mm").count() >= 1);
        assert_eq!(harness.query_all_by_label_contains("ms draw").count(), 0);
        assert_eq!(harness.query_all_by_label_contains("last:").count(), 0);
    }
    window.language = Language::Spanish;
    {
        let harness = status_harness(&mut window, egui::vec2(1366.0, 768.0));
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

#[test]
fn the_notice_stays_visible_at_900_points_with_long_names() {
    assert_eq!(LONG_CIRCUIT.chars().count(), 60);
    let mut window = super::window_with();
    let catalog = Catalog::builtin();
    window.document = templates::document(0, &catalog, window.page.rect())
        .expect("the worked example is a template");
    window.document.rename_circuit(0, LONG_CIRCUIT);
    window.rebuild_if_stale();
    let valve = window
        .document
        .placements()
        .iter()
        .position(|placement| placement.catalog_id.starts_with("valve."))
        .expect("the template has a valve");
    window.selected = vec![valve];
    let selection_name = window
        .catalog
        .get(&window.document.placements()[valve].catalog_id)
        .expect("the template's valve is in the catalog")
        .name
        .clone();
    let selection_prefix = selection_name
        .split(' ')
        .next()
        .expect("a catalog name has a first word");
    let notice = "The connection was refused: the two ends name the same port, so no hose was created. Choose another port and release again.";
    window.notice = Some(notice.to_owned());

    let harness = status_harness(&mut window, egui::vec2(900.0, 600.0));
    let rect = harness.get_by_label_contains(notice).rect();
    let screen = harness.ctx.screen_rect();
    assert!(
        rect.width() >= 24.0,
        "the notice keeps a visible rectangle: {rect:?}"
    );
    assert!(
        rect.left() >= screen.left() && rect.right() <= screen.right(),
        "the notice stays inside a 900-point window: {rect:?} in {screen:?}"
    );
    assert!(
        harness.query_all_by_label_contains("Editing").count() >= 1,
        "the mode keeps its place when the notice is long"
    );
    assert!(
        harness
            .query_all_by_label_contains(selection_prefix)
            .count()
            >= 1,
        "the selection keeps its cell, even if the reading yields first"
    );
}

#[test]
fn the_selection_shows_the_designation_before_the_catalog_name() {
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
    let english = window
        .catalog
        .get(&window.document.placements()[valve].catalog_id)
        .expect("the template's valve is in the catalog")
        .name
        .clone();
    let spanish = window
        .catalog
        .get(&window.document.placements()[valve].catalog_id)
        .expect("the template's valve is in the catalog")
        .name_es
        .clone()
        .unwrap_or_else(|| english.clone());
    assert!(window.document.designate(valve, Some("V_MAIN")));
    window.selected = vec![valve];
    {
        let harness = status_harness(&mut window, egui::vec2(1366.0, 768.0));
        assert!(harness.query_all_by_label_contains("V_MAIN").count() >= 1);
        assert_eq!(
            harness.query_all_by_label_contains(&english).count(),
            0,
            "the designation replaces the catalog name in the selection"
        );
    }

    assert!(window.document.designate(valve, None));
    {
        let harness = status_harness(&mut window, egui::vec2(1366.0, 768.0));
        assert!(
            harness.query_all_by_label_contains(&english).count() >= 1,
            "an empty designation falls back to the translated catalog name"
        );
    }

    window.language = Language::Spanish;
    {
        let harness = status_harness(&mut window, egui::vec2(1366.0, 768.0));
        assert!(
            harness.query_all_by_label_contains(&spanish).count() >= 1,
            "the fallback is the name in the active language"
        );
    }
}

#[test]
fn the_selection_distinguishes_two_equal_valves_by_their_designation() {
    let mut window = super::window_with();
    let catalog = Catalog::builtin();
    window.document = templates::document(0, &catalog, window.page.rect())
        .expect("the worked example is a template");
    let first = window
        .document
        .placements()
        .iter()
        .position(|placement| placement.catalog_id.starts_with("valve."))
        .expect("the template has a valve");
    let same_id = window.document.placements()[first].catalog_id.clone();
    let second = window
        .document
        .place(Placement::new(same_id, Pt::new(200.0, 100.0), 20.0, 20.0));
    assert!(window.document.designate(first, Some("V_MAIN")));
    assert!(window.document.designate(second, Some("V_AUX")));

    window.selected = vec![first];
    {
        let harness = status_harness(&mut window, egui::vec2(1366.0, 768.0));
        assert!(harness.query_all_by_label_contains("V_MAIN").count() >= 1);
        assert_eq!(harness.query_all_by_label_contains("V_AUX").count(), 0);
    }
    window.selected = vec![second];
    {
        let harness = status_harness(&mut window, egui::vec2(1366.0, 768.0));
        assert!(harness.query_all_by_label_contains("V_AUX").count() >= 1);
        assert_eq!(harness.query_all_by_label_contains("V_MAIN").count(), 0);
    }
}

#[test]
fn the_notice_stays_visible_with_a_large_font_in_both_themes_and_languages() {
    for theme_id in [ThemeId::Cad, ThemeId::Slate] {
        for language in [Language::English, Language::Spanish] {
            for width in [900.0, 1366.0] {
                let mut window = super::window_with();
                let catalog = Catalog::builtin();
                window.document = templates::document(0, &catalog, window.page.rect())
                    .expect("the worked example is a template");
                window.document.rename_circuit(0, LONG_CIRCUIT);
                window.rebuild_if_stale();
                let valve = window
                    .document
                    .placements()
                    .iter()
                    .position(|placement| placement.catalog_id.starts_with("valve."))
                    .expect("the template has a valve");
                window.selected = vec![valve];
                window.language = language;
                let notice = match language {
                    Language::English => {
                        "The connection was refused: the two ends name the same port, so no hose was created. Choose another port and release again."
                    }
                    Language::Spanish => {
                        "Se rechazó la conexión: los dos extremos nombran el mismo puerto y no se creó la tubería. Elige otro puerto y suelta de nuevo."
                    }
                };
                window.notice = Some(notice.to_owned());
                // A 150 % desktop font grows the spacing unit and the body size together, as
                // `tokens_from` does; the status bar must keep the notice inside the row.
                let theme = Theme::for_appearance(
                    Appearance::Theme(theme_id),
                    ThemeId::Cad,
                    ThemeId::Slate,
                );
                window.theme = scale_theme(theme, 1.5);
                let applied = window.theme.clone();
                let harness = themed_status_harness(&mut window, applied, egui::vec2(width, 768.0));
                let rect = harness.get_by_label_contains(notice).rect();
                let screen = harness.ctx.screen_rect();
                assert!(
                    rect.width() >= 24.0,
                    "{language:?} at {width} with {theme_id:?}: the notice keeps a visible rectangle: {rect:?}"
                );
                assert!(
                    rect.left() >= screen.left() && rect.right() <= screen.right(),
                    "{language:?} at {width} with {theme_id:?}: the notice stays inside the window: {rect:?} in {screen:?}"
                );
            }
        }
    }
}

/// The same theme with a larger desktop font: the unit and the text sizes grow together.
fn scale_theme(mut theme: Theme, factor: f32) -> Theme {
    theme.tokens.unit = (theme.tokens.unit * factor).round();
    theme.tokens.text_small = (theme.tokens.text_small * factor).round();
    theme.tokens.text_body = (theme.tokens.text_body * factor).round();
    theme.tokens.text_label = (theme.tokens.text_label * factor).round();
    theme
}

#[test]
fn a_step_from_editing_reads_paused_in_the_status_bar() {
    let mut window = super::window_with();
    let catalog = Catalog::builtin();
    window.document = templates::document(0, &catalog, window.page.rect())
        .expect("the worked example is a template");
    window.rebuild_if_stale();
    window.dispatch(Command::Step);
    let harness = status_harness(&mut window, egui::vec2(1366.0, 768.0));
    assert!(
        harness.query_all_by_label_contains("Paused").count() >= 1,
        "a step leaves the mode reading Paused"
    );
    assert_eq!(
        harness.query_all_by_label_contains("Editing").count(),
        0,
        "a step is no longer in editing"
    );
}
