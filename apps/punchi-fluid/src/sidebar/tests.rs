// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The palette's tests: the port closures, the tiles, the menu and the keyboard.
use eframe::egui;
use punchi_fluid_catalog::Catalog;

use super::{is_entry, matches, split_name};

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
        2,
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
fn a_word_wider_than_the_tile_is_elided_on_one_line() {
    let context = egui::Context::default();
    if crate::theme::install_font(&context, None).is_err() {
        return;
    }
    let job = super::name_job(
        "Compressor",
        40.0,
        egui::FontId::proportional(15.0),
        egui::Color32::WHITE,
        1,
    );
    assert_eq!(job.wrap.max_rows, 1);
    assert_eq!(job.wrap.overflow_character, Some('…'));
    let mut rows = 0;
    let _ = context.run(egui::RawInput::default(), |context| {
        rows = context
            .fonts(|fonts| fonts.layout_job(job.clone()))
            .rows
            .len();
    });
    assert_eq!(rows, 1, "the wide word is elided, not split across rows");
}

#[test]
fn a_large_font_uses_one_column_instead_of_eliding_every_name() {
    let context = egui::Context::default();
    if crate::theme::install_font(&context, None).is_err() {
        return;
    }
    let mut width = 0.0;
    let _ = context.run(egui::RawInput::default(), |context| {
        egui::CentralPanel::default().show(context, |ui| {
            width = super::tile_width_for(
                ui,
                160.0,
                8.0,
                ["Compressor", "Compressed air network"].into_iter(),
                &egui::FontId::proportional(15.0),
                egui::Color32::WHITE,
                60.0,
            );
        });
    });
    assert!((width - 160.0).abs() < 0.01, "one column at the large font");
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

use egui::MouseWheelUnit;
use egui_kittest::kittest::Queryable;
use punchi_fluid_catalog::{Category, default_catalog};

use super::{CatalogAction, Sidebar};
use crate::theme::Theme;

const ENTRY: &str = "valve.3_2.spring_return";

/// The palette drawn headless, with the id `show` asked to add and the tiles' rectangles.
type TileHarness = (
    egui_kittest::Harness<'static>,
    Rc<RefCell<Option<CatalogAction>>>,
    Rc<RefCell<Vec<(String, egui::Rect)>>>,
);

fn tile_harness() -> TileHarness {
    tile_harness_at(420.0, 1000.0)
}

fn tile_harness_at(width: f32, height: f32) -> TileHarness {
    let mut sidebar = Sidebar::new();
    let catalog = default_catalog();
    let tokens = Theme::fallback().tokens;
    let returned = Rc::new(RefCell::new(None));
    let tiles = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&returned);
    let rects = Rc::clone(&tiles);
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(width, height))
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(120)
        .build(move |context| {
            if let Some(id) = sidebar.show(context, &tokens, "", &catalog, width) {
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

/// The palette with its scroll viewport and reserved bar track recorded.
type ScrollHarness = (
    egui_kittest::Harness<'static>,
    Rc<RefCell<Vec<(String, egui::Rect)>>>,
    Rc<RefCell<(Option<egui::Rect>, Option<egui::Rect>)>>,
);

fn scroll_harness(width: f32, height: f32) -> ScrollHarness {
    let mut sidebar = Sidebar::new();
    let catalog = default_catalog();
    let tokens = Theme::fallback().tokens;
    let tiles = Rc::new(RefCell::new(Vec::new()));
    let geometry = Rc::new(RefCell::new((None, None)));
    let rects = Rc::clone(&tiles);
    let geom = Rc::clone(&geometry);
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(width, height))
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(120)
        .build(move |context| {
            let _ = sidebar.show(context, &tokens, "", &catalog, width);
            *rects.borrow_mut() = sidebar.tile_rects().to_vec();
            *geom.borrow_mut() = sidebar.scroll_geometry();
        });
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the sidebar tests need one");
    }
    harness.run();
    (harness, tiles, geometry)
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
    assert_eq!(
        returned.borrow().as_ref(),
        Some(&CatalogAction::Add(ENTRY.to_owned()))
    );
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
        returned.borrow().as_ref(),
        Some(&CatalogAction::Add(ENTRY.to_owned())),
        "Enter places the focused tile at the centre"
    );
}

/// Presses on a tile, moves to `to` in two steps and releases: the carry survives the path.
fn drag(harness: &mut egui_kittest::Harness<'static>, from: egui::Pos2, to: egui::Pos2) {
    harness
        .input_mut()
        .events
        .push(egui::Event::PointerMoved(from));
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: from,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::default(),
    });
    harness.run();
    harness
        .input_mut()
        .events
        .push(egui::Event::PointerMoved(from + egui::vec2(20.0, 20.0)));
    harness.run();
    harness
        .input_mut()
        .events
        .push(egui::Event::PointerMoved(to));
    harness.run();
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: to,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::default(),
    });
    harness.run();
}

#[test]
fn the_carry_returns_the_drop_point() {
    let (mut harness, returned, tiles) = tile_harness();
    let target = egui::pos2(300.0, 320.0);
    drag(&mut harness, tile_rect(&tiles).center(), target);
    let action = returned.borrow().clone();
    assert!(
        matches!(action, Some(CatalogAction::Place { .. })),
        "the carried tile must ask for a placement, got {action:?}"
    );
    let Some(CatalogAction::Place { id, screen }) = action else {
        return;
    };
    assert_eq!(id, ENTRY);
    assert!(
        screen.distance(target) < 2.0,
        "the drop follows the release: {screen:?}"
    );
}

#[test]
fn a_diagonal_gesture_still_places_the_component() {
    let (mut harness, returned, tiles) = tile_harness();
    // The pointer drifts up and right inside the palette before reaching the sheet: the
    // scroll area must not take the gesture and cancel the carry.
    let start = tile_rect(&tiles).center();
    let target = egui::pos2(60.0, 380.0);
    drag(&mut harness, start, target);
    let action = returned.borrow().clone();
    assert!(
        matches!(action, Some(CatalogAction::Place { .. })),
        "a diagonal path still places, got {action:?}"
    );
}

#[test]
fn a_carry_can_be_cancelled() {
    let mut sidebar = Sidebar::new();
    assert!(!sidebar.carrying());
    sidebar.carry_for_test(ENTRY);
    assert!(sidebar.carrying());
    sidebar.cancel_carry();
    assert!(
        !sidebar.carrying(),
        "Escape drops the carry and the release places nothing"
    );
}

#[test]
fn the_palette_lays_two_columns_of_tiles() {
    let (_harness, _returned, tiles) = tile_harness();
    let tiles = tiles.borrow();
    assert!(!tiles.is_empty(), "the palette paints its tiles");
    let Some((_, first)) = tiles.first() else {
        return;
    };
    let neighbour = tiles.iter().find(|(_, rect)| {
        (rect.min.y - first.min.y).abs() < 1.0 && (rect.min.x - first.min.x).abs() > 1.0
    });
    assert!(
        neighbour.is_some(),
        "the palette wraps two tiles per row at the default width"
    );
}

#[test]
fn a_name_splits_at_its_first_comma() {
    assert_eq!(
        split_name("3/2 valve, spring return"),
        ("3/2 valve", Some("spring return"))
    );
    assert_eq!(split_name("Buzzer"), ("Buzzer", None));
    assert_eq!(split_name(", spring"), (", spring", None));
}

#[test]
fn a_search_with_no_matches_says_so() {
    let mut sidebar = Sidebar::new();
    sidebar.body.filter = String::from("zzzz");
    let catalog = default_catalog();
    let tokens = Theme::fallback().tokens;
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(420.0, 600.0))
        .build(move |context| {
            let _ = sidebar.show(context, &tokens, "", &catalog, 420.0);
        });
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the sidebar tests need one");
    }
    harness.run();
    assert!(
        harness
            .query_by_label_contains("No components match")
            .is_some(),
        "an empty search says so instead of showing a blank panel"
    );
}

#[test]
fn catalog_tiles_do_not_intersect_the_scroll_track() {
    let (_harness, tiles, geometry) = scroll_harness(300.0, 600.0);
    let (viewport, track) = *geometry.borrow();
    let viewport = viewport.expect("the scroll viewport is recorded");
    let track = track.expect("the scroll track is recorded");
    assert!(
        track.width() > 0.0,
        "the catalog reserves a real bar track: {track:?}"
    );
    assert!(
        viewport.right() <= track.left() + 0.5,
        "the viewport ends before the track: {viewport:?} / {track:?}"
    );
    for (id, rect) in tiles.borrow().iter() {
        assert!(
            rect.left() >= viewport.left() - 0.5 && rect.right() <= viewport.right() + 0.5,
            "{id} crosses the scroll track: {rect:?} inside {viewport:?}"
        );
    }
}

#[test]
fn the_catalog_at_its_preferred_width_wraps_two_columns() {
    let (_harness, _returned, tiles) = tile_harness_at(300.0, 1000.0);
    let tiles = tiles.borrow();
    let Some((_, first)) = tiles.first() else {
        return;
    };
    assert!(
        tiles.iter().any(|(_, rect)| {
            (rect.min.y - first.min.y).abs() < 1.0 && (rect.min.x - first.min.x).abs() > 1.0
        }),
        "300 points hold two columns: the preferred width is 300"
    );
}

#[test]
fn dragging_a_right_column_tile_still_places_that_tile() {
    let (mut harness, returned, tiles) = tile_harness();
    let right = {
        let tiles = tiles.borrow();
        let first = tiles.first().expect("the palette paints tiles");
        tiles
            .iter()
            .find(|(_, rect)| {
                (rect.min.y - first.1.min.y).abs() < 1.0 && rect.min.x > first.1.min.x
            })
            .cloned()
            .expect("the first row has a right column")
    };
    let target = egui::pos2(360.0, 420.0);
    drag(&mut harness, right.1.center(), target);
    let action = returned.borrow().clone();
    assert!(
        matches!(action, Some(CatalogAction::Place { .. })),
        "the right-column tile must be carried, got {action:?}"
    );
    let Some(CatalogAction::Place { id, .. }) = action else {
        return;
    };
    assert_eq!(id, right.0);
}

#[test]
fn the_last_group_is_reachable_by_scrolling() {
    let (mut harness, tiles, geometry) = scroll_harness(300.0, 600.0);
    let catalog = default_catalog();
    let last = catalog
        .by_category(Category::Sensor)
        .next()
        .expect("the built-in catalog has a sensor")
        .id
        .clone();
    let initial = tiles
        .borrow()
        .iter()
        .find(|(id, _)| id == &last)
        .map(|(_, rect)| *rect)
        .expect("the last tile has a rectangle");
    let (viewport, _track) = *geometry.borrow();
    let viewport = viewport.expect("the scroll viewport is recorded");
    assert!(
        !viewport.contains(initial.center()),
        "the last group starts below the fold: {initial:?} in {viewport:?}"
    );
    harness
        .input_mut()
        .events
        .push(egui::Event::PointerMoved(egui::pos2(150.0, 300.0)));
    harness.run();
    for _ in 0..3 {
        harness.input_mut().events.push(egui::Event::MouseWheel {
            unit: MouseWheelUnit::Point,
            delta: egui::vec2(0.0, -400.0),
            modifiers: egui::Modifiers::NONE,
        });
        harness.run();
    }
    let after = tiles
        .borrow()
        .iter()
        .find(|(id, _)| id == &last)
        .map(|(_, rect)| *rect)
        .expect("the last tile keeps its rectangle");
    assert!(
        viewport.contains(after.center()),
        "scrolling reaches the last group: {after:?} in {viewport:?}"
    );
}

#[test]
fn a_tile_announces_the_carry_with_the_right_hand() {
    assert_eq!(super::tile_cursor(false, false), None);
    assert_eq!(
        super::tile_cursor(true, false),
        Some(egui::CursorIcon::Grab)
    );
    assert_eq!(
        super::tile_cursor(true, true),
        Some(egui::CursorIcon::Grabbing)
    );
}

#[test]
fn tiles_with_and_without_variant_have_equal_height() {
    let (_harness, _returned, tiles) = tile_harness();
    let tiles = tiles.borrow();
    let with_variant = tiles
        .iter()
        .find(|(id, _)| id == ENTRY)
        .map(|(_, rect)| *rect)
        .expect("the variant tile is visible");
    let without_variant = tiles
        .iter()
        .find(|(id, _)| id == "supply.compressor")
        .map(|(_, rect)| *rect)
        .expect("the plain tile is visible");
    assert!(
        (with_variant.height() - without_variant.height()).abs() < 0.5,
        "both tiles reserve the same rows: {with_variant:?} / {without_variant:?}"
    );
}

#[test]
fn uniform_tiles_keep_scroll_and_drag_reachable() {
    let (_harness, tiles, geometry) = scroll_harness(300.0, 600.0);
    let (viewport, track) = *geometry.borrow();
    let viewport = viewport.expect("the scroll viewport is recorded");
    let track = track.expect("the scroll track is recorded");
    assert!(track.width() > 0.0, "the catalog keeps its reserved track");
    for (id, rect) in tiles.borrow().iter() {
        assert!(
            rect.left() >= viewport.left() - 0.5 && rect.right() <= viewport.right() + 0.5,
            "{id} stays out of the scroll track: {rect:?}"
        );
    }
    let first = tiles.borrow().first().map(|(_, rect)| *rect);
    let last = tiles.borrow().last().map(|(_, rect)| *rect);
    assert!(first.is_some() && last.is_some(), "the tiles are laid out");
    assert_eq!(
        first.map(|rect| rect.width()),
        last.map(|rect| rect.width()),
        "the last tile keeps the column width of the first"
    );
}
