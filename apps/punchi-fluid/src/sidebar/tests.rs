// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The palette's tests: the port closures, the tiles, the menu and the keyboard.
use eframe::egui;
use punchi_fluid_catalog::Catalog;

use super::{is_entry, matches};

#[test]
fn the_palette_hides_the_port_closures() {
    let catalog = Catalog::builtin();
    assert!(!is_entry(
        catalog.get("plug.blind").expect("the plug exists")
    ));
    assert!(!is_entry(
        catalog
            .get("silencer.pneumatic")
            .expect("the silencer exists")
    ));
    assert!(is_entry(
        catalog
            .get("valve.3_2.spring_return")
            .expect("the valve exists")
    ));
}

#[test]
fn a_long_name_stays_on_two_lines() {
    let context = egui::Context::default();
    if crate::theme::install_font(&context, None).is_err() {
        return;
    }
    let job = super::name_job(
        "Válvula 3/2, solenoide, retorno por muelle",
        104.0,
        egui::FontId::proportional(11.0),
        egui::Color32::WHITE,
    );
    assert_eq!(job.wrap.max_rows, 2);
    assert_eq!(job.wrap.overflow_character, Some('…'));
    let mut rows = 0;
    let _ = context.run(egui::RawInput::default(), |context| {
        rows = context
            .fonts(|fonts| fonts.layout_job(job.clone()))
            .rows
            .len();
    });
    assert!(rows <= 2, "the tile draws two lines, not {rows}");
}

#[test]
fn the_search_matches_the_name_the_spanish_name_and_the_id() {
    let catalog = Catalog::builtin();
    let valve = catalog
        .get("valve.3_2.spring_return")
        .expect("the valve exists");
    assert!(matches(valve, ""));
    assert!(matches(valve, "válvula"));
    assert!(matches(valve, "3/2"));
    assert!(matches(valve, "valve.3_2"));
    assert!(!matches(valve, "cilindro"));
}

use std::cell::RefCell;
use std::rc::Rc;

use egui_kittest::kittest::Queryable;
use punchi_fluid_catalog::default_catalog;

use super::Sidebar;
use crate::theme::Theme;

const ENTRY: &str = "valve.3_2.spring_return";

/// The palette drawn headless, with the id `show` asked to add and the tiles' rectangles.
type TileHarness = (
    egui_kittest::Harness<'static>,
    Rc<RefCell<Option<String>>>,
    Rc<RefCell<Vec<(String, egui::Rect)>>>,
);

fn tile_harness() -> TileHarness {
    let mut sidebar = Sidebar::new();
    let catalog = default_catalog();
    let tokens = Theme::fallback().tokens;
    let returned = Rc::new(RefCell::new(None));
    let tiles = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&returned);
    let rects = Rc::clone(&tiles);
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(420.0, 1000.0))
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(120)
        .build(move |context| {
            if let Some(id) = sidebar.show(context, &tokens, "", &catalog, 420.0) {
                *sink.borrow_mut() = Some(id);
            }
            *rects.borrow_mut() = sidebar.tile_rects().to_vec();
        });
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the sidebar tests need one");
    }
    harness.run();
    (harness, returned, tiles)
}

/// The screen rectangle of a tile, as the palette painted it.
fn tile_rect(tiles: &Rc<RefCell<Vec<(String, egui::Rect)>>>) -> egui::Rect {
    tiles
        .borrow()
        .iter()
        .find(|(id, _)| id == ENTRY)
        .map(|(_, rect)| *rect)
        .expect("the test tile is on screen")
}

fn pointer_at(
    harness: &mut egui_kittest::Harness<'static>,
    position: egui::Pos2,
    button: egui::PointerButton,
) {
    harness
        .input_mut()
        .events
        .push(egui::Event::PointerMoved(position));
    for pressed in [true, false] {
        harness.input_mut().events.push(egui::Event::PointerButton {
            pos: position,
            button,
            pressed,
            modifiers: egui::Modifiers::default(),
        });
    }
    harness.run();
}

#[test]
fn a_click_chooses_a_tile_without_adding_it() {
    let (mut harness, returned, tiles) = tile_harness();
    pointer_at(
        &mut harness,
        tile_rect(&tiles).center(),
        egui::PointerButton::Primary,
    );
    assert!(
        returned.borrow().is_none(),
        "a click selects the tile and does not place it"
    );
}

#[test]
fn the_tile_menu_adds_the_entry_at_the_centre() {
    let (mut harness, returned, tiles) = tile_harness();
    pointer_at(
        &mut harness,
        tile_rect(&tiles).center(),
        egui::PointerButton::Secondary,
    );
    assert!(
        harness.query_by_label("Add to sheet").is_some(),
        "the right button opens the tile menu"
    );
    assert!(
        returned.borrow().is_none(),
        "opening the menu places nothing by itself"
    );
    harness.get_by_label("Add to sheet").click();
    harness.run();
    assert_eq!(returned.borrow().as_deref(), Some(ENTRY));
}

#[test]
fn the_keyboard_adds_the_chosen_tile() {
    let (mut harness, returned, tiles) = tile_harness();
    pointer_at(
        &mut harness,
        tile_rect(&tiles).center(),
        egui::PointerButton::Primary,
    );
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(
        returned.borrow().as_deref(),
        Some(ENTRY),
        "Enter places the focused tile at the centre"
    );
}
