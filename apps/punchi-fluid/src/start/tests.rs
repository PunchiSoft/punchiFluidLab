// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Inicio page tests: the rail, the cards, the thumbnails, the recent rows and the
//! placeholder, driven by the headless harness, plus the template labels and the document
//! each thumbnail draws.

use std::path::PathBuf;

use eframe::egui;
use egui_kittest::kittest::Queryable;
use punchi_fluid_catalog::Catalog;
use punchi_fluid_doc::{Document, Placement};
use punchi_fluid_draw::Pt;
use punchi_fluid_draw::draw_placed_preview;
use punchi_fluid_draw::surface::Recording;
use punchi_fluid_session::recents::Recents;
use punchi_fluid_session::templates::{self, Template};

use super::cards::{accessible_label, category, example_badge, labels, recent_ink, recent_missing};
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

/// A folder of its own for one test: the tests run in parallel and no two may share a recent
/// list or a document.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "punchi-fluid-start-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the scratch folder");
    dir
}

/// A document with one placement, so a reopened file is recognisably its own.
fn save_document(path: &std::path::Path) {
    let mut document = Document::new();
    document.place(Placement::new(
        "valve.5_2",
        Pt::new(100.0, 80.0),
        24.0,
        24.0,
    ));
    punchi_fluid_session::save(&document, path).expect("the fixture document is saved");
}

/// The accessibility label of a recent row: its name, a full stop and its folder.
fn recent_label(name: &str, folder: &std::path::Path) -> String {
    format!("{name}. {}", folder.display())
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
fn the_inicio_commands_create_a_project_and_open_the_file_dialog() {
    // Nuevo proyecto leaves Inicio on an empty sheet with no path.
    let mut window = window();
    {
        let mut harness = harness(&mut window);
        harness.get_by_label("Nuevo proyecto").click();
        harness.run();
    }
    assert!(
        !window.start_open,
        "Nuevo proyecto goes to the sheet instead of keeping the page"
    );
    assert!(window.path.is_none(), "a new project has no path");
    assert!(
        window.document.placements().is_empty(),
        "the new project is empty"
    );
    assert!(
        window
            .notice
            .as_deref()
            .is_some_and(|notice| notice.contains("nuevo")),
        "the status bar noticed the new document: {:?}",
        window.notice
    );

    // Abrir… opens the file dialog; the page stays behind it, as a modal should.
    let dir = scratch("inicio-open");
    let mut window = Window::new(
        Theme::fallback(),
        Document::new(),
        Some(dir.join("base.pfluid")),
        None,
    );
    {
        let mut harness = harness(&mut window);
        harness.get_by_label("Abrir…").click();
        harness.run();
    }
    assert!(window.start_open, "the page stays behind the dialog");
    assert!(window.dialog.is_some(), "Abrir… opened the file dialog");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_learning_rows_keep_the_page_open_behind_their_panel() {
    let mut window = window();
    {
        let mut harness = harness(&mut window);
        harness.get_by_label("Acerca de").click();
        harness.run();
        harness.get_by_label("Licencia").click();
        harness.run();
    }
    assert!(
        window.start_open,
        "the panel opens over the page, it does not leave it"
    );
    assert!(window.notice.is_none());
}

#[test]
fn the_inicio_shows_a_remembered_document_and_opens_it() {
    let dir = scratch("recent-open");
    let saved = dir.join("taller.pfluid");
    save_document(&saved);
    let mut recents = Recents::at(Some(dir.join("recent.json")));
    assert!(recents.remember(&saved));
    let mut window = Window::with_recents(Theme::fallback(), Document::new(), None, None, recents);
    {
        let mut harness = harness(&mut window);
        let row = recent_label("taller.pfluid", &dir);
        assert!(
            harness.query_by_label(&row).is_some(),
            "the remembered document is a row"
        );
        harness.get_by_label(&row).click();
        harness.run();
    }
    assert!(!window.start_open, "opening a recent closes Inicio");
    assert_eq!(window.path.as_deref(), Some(saved.as_path()));
    assert_eq!(
        window.document.placements().len(),
        1,
        "the remembered document is the one in the window"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_missing_recent_row_is_muted_and_refuses_to_open() {
    let dir = scratch("recent-missing");
    let gone = dir.join("borrado.pfluid");
    let mut recents = Recents::at(Some(dir.join("recent.json")));
    assert!(recents.remember(&gone));
    let mut window = Window::with_recents(Theme::fallback(), Document::new(), None, None, recents);
    {
        let mut harness = harness(&mut window);
        let row = format!("borrado.pfluid. {} · no encontrado", dir.display());
        assert!(
            harness.query_by_label(&row).is_some(),
            "a row whose file is gone says so"
        );
        harness.get_by_label(&row).click();
        harness.run();
    }
    assert!(window.start_open, "the missing row does not open");
    assert!(window.path.is_none(), "no document was adopted");
    assert!(
        window.document.placements().is_empty(),
        "the window still has its empty sheet"
    );
    let tokens = Theme::fallback().tokens;
    assert_eq!(
        recent_ink(&tokens, true),
        tokens.ink_muted,
        "a missing row is painted with the muted ink"
    );
    assert_eq!(recent_ink(&tokens, false), tokens.ink);
    assert!(recent_missing(&gone), "the file is not there");
    let present = dir.join("presente.pfluid");
    std::fs::write(&present, "{}").expect("a file that is there");
    assert!(
        !recent_missing(&present),
        "a file that exists is not reported missing"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn quitar_forgets_only_the_row_it_names() {
    let dir = scratch("recent-quitar");
    let first = dir.join("uno.pfluid");
    let second = dir.join("dos.pfluid");
    save_document(&first);
    save_document(&second);
    let recents_file = dir.join("recent.json");
    let mut recents = Recents::at(Some(recents_file.clone()));
    assert!(recents.remember(&first));
    assert!(recents.remember(&second));
    let mut window = Window::with_recents(Theme::fallback(), Document::new(), None, None, recents);
    {
        let mut harness = harness(&mut window);
        harness.get_by_label("Quitar uno.pfluid").click();
        harness.run();
    }
    assert!(window.start_open, "forgetting a row keeps the page");
    assert_eq!(window.recents.entries().len(), 1);
    assert_eq!(window.recents.entries()[0].name, "dos.pfluid");
    let reread = Recents::at(Some(recents_file));
    assert_eq!(
        reread.entries().len(),
        1,
        "the list on disk lost exactly one entry"
    );
    assert_eq!(reread.entries()[0].name, "dos.pfluid");
    let _ = std::fs::remove_dir_all(&dir);
}
