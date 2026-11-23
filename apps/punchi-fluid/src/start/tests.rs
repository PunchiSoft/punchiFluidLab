// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Inicio page tests: the rail, the cards, the thumbnails and the placeholder, driven by
//! the headless harness, plus the template labels and the document each thumbnail draws.

use eframe::egui;
use egui_kittest::kittest::Queryable;
use punchi_fluid_catalog::Catalog;
use punchi_fluid_doc::Document;
use punchi_fluid_draw::draw_placed_preview;
use punchi_fluid_draw::surface::Recording;
use punchi_fluid_session::templates::{self, Template};

use super::cards::{accessible_label, category, example_badge, labels};
use super::document;
use super::thumbnail::{thumbnail_area, thumbnail_ink};
use crate::Window;
use crate::theme::Theme;

/// The page drawn headless at the window's floor, as the ribbon tests draw the ribbon.
fn harness(window: &mut Window) -> egui_kittest::Harness<'_> {
    let mut harness = egui_kittest::Harness::new(move |context| window.show_start(context));
    harness.set_size(egui::vec2(1366.0, 768.0));
    egui_extras::install_image_loaders(&harness.ctx);
    // The window ships no font: the harness gets the desktop's, as the app does at start.
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the start-page tests need one");
    }
    harness.run();
    harness
}

fn window() -> Window {
    Window::new(Theme::fallback(), Document::new(), None, None)
}

#[test]
fn each_template_has_a_name_a_description_a_category_and_the_first_has_the_example_badge() {
    for index in 0..templates::COUNT {
        let template = Template::from_index(index).expect("a template");
        let (name, description) = labels(template);
        assert!(!name.is_empty() && !description.is_empty());
        assert!(!category(template).is_empty());
        assert!(!accessible_label(template).is_empty());
    }
    assert_eq!(category(Template::WorkedExample), "NEUMÁTICA");
    assert_eq!(category(Template::ElectricalControl), "ELÉCTRICA");
    assert_eq!(example_badge(Template::WorkedExample), Some("EJEMPLO"));
    assert_eq!(example_badge(Template::DoubleActing), None);
}

#[test]
fn every_template_builds_the_document_its_thumbnail_draws() {
    let catalog = Catalog::builtin();
    for index in 0..templates::COUNT {
        let document = document(index, &catalog, thumbnail_area())
            .expect("a template document for the thumbnail");
        assert!(!document.placements().is_empty());
    }
}

#[test]
fn every_thumbnail_document_draws_through_the_sheet_preview_path() {
    let catalog = Catalog::builtin();
    let ink = thumbnail_ink(&Theme::fallback().tokens);
    for index in 0..templates::COUNT {
        let document = document(index, &catalog, thumbnail_area())
            .expect("a template document for the thumbnail");
        let scene = crate::scene::build(&document, &catalog, document.active_circuit(), None);
        let mut recording = Recording::default();
        draw_placed_preview(
            &mut recording,
            &scene.placed,
            &scene.pipes,
            &catalog,
            &ink,
            &[],
        );
        assert!(
            !recording.is_empty(),
            "template {index} must draw its own thumbnail"
        );
    }
}

#[test]
fn the_four_cards_share_one_row_inside_the_window_floor() {
    let mut window = window();
    let harness = harness(&mut window);
    let cards: Vec<egui::Rect> = [
        "Circuito OR",
        "Cilindro de doble efecto",
        "Preparación de aire",
        "Control eléctrico",
    ]
    .iter()
    .map(|name| harness.get_by_label_contains(name).rect())
    .collect();
    let first = cards[0];
    for card in &cards {
        assert!(
            (card.top() - first.top()).abs() <= 1.0,
            "the four cards share one row: {card:?}"
        );
        assert!(
            card.width() >= 160.0,
            "a card keeps its ten grid units: {card:?}"
        );
        assert!(
            card.right() <= 1366.0 && card.bottom() <= 768.0,
            "the cards sit inside the window floor: {card:?}"
        );
    }
    for pair in cards.windows(2) {
        assert!(
            pair[1].left() > pair[0].left(),
            "the cards keep their order"
        );
    }
    let placeholder = harness
        .get_by_label("Todavía no hay documentos recientes")
        .rect();
    assert!(
        placeholder.bottom() <= 768.0,
        "the empty Recientes is on the first screen: {placeholder:?}"
    );
}

#[test]
fn the_page_draws_the_rail_the_two_ways_in_the_templates_and_the_placeholder() {
    let mut window = window();
    let harness = harness(&mut window);
    for label in ["Volver a la hoja", "Acerca de", "Licencia"] {
        assert!(
            harness.query_by_label(label).is_some(),
            "{label} is in the rail"
        );
    }
    for label in ["Nuevo proyecto", "Abrir…"] {
        assert!(
            harness.query_by_label(label).is_some(),
            "{label} is in the rail"
        );
    }
    assert_eq!(
        harness.query_all_by_label("Inicio").count(),
        2,
        "the rail marks Inicio and the page names it"
    );
    for label in ["Empezar", "Plantillas", "Recientes"] {
        assert!(
            harness.query_by_label(label).is_some(),
            "{label} is a section"
        );
    }
    assert!(
        harness
            .query_by_label("Todavía no hay documentos recientes")
            .is_some(),
        "the empty Recientes says so instead of a blank table"
    );
    for name in [
        "Circuito OR",
        "Cilindro de doble efecto",
        "Preparación de aire",
        "Control eléctrico",
    ] {
        assert!(
            harness.query_by_label_contains(name).is_some(),
            "{name} has a card"
        );
    }
}

#[test]
fn choosing_a_template_loads_it_and_closes_the_page() {
    let mut window = window();
    {
        let mut harness = harness(&mut window);
        harness
            .get_by_label_contains("Cilindro de doble efecto")
            .click();
        harness.run();
    }
    assert!(!window.start_open, "the page closes on the chosen template");
    assert_eq!(
        window.document.placements().len(),
        3,
        "the double-acting template has its three components"
    );
}

#[test]
fn the_keyboard_reaches_the_rail_and_volver_a_la_hoja_closes_the_page() {
    let mut window = window();
    {
        let mut harness = harness(&mut window);
        harness.get_by_label("Volver a la hoja").focus();
        harness.run();
        harness.key_press(egui::Key::Enter);
        harness.run();
    }
    assert!(
        !window.start_open,
        "Enter on the focused rail row returns to the sheet"
    );
}

#[test]
fn the_project_and_learning_commands_wait_for_their_dialogs() {
    let mut window = window();
    {
        let mut harness = harness(&mut window);
        harness.get_by_label("Nuevo proyecto").click();
        harness.run();
        harness.get_by_label("Abrir…").click();
        harness.run();
    }
    assert!(
        window.start_open,
        "a command without its dialog keeps the page open"
    );
    assert!(
        window.document.placements().is_empty(),
        "no project is invented while the dialogs are missing"
    );
    assert!(window.notice.is_none());
}
