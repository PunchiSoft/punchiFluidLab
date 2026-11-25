// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The view gestures that need a pointer: the zoom window's rectangle and the hand's drag.

use eframe::egui;
use punchi_fluid_core::DEFAULT_TUBE_HUNDREDTHS;
use punchi_fluid_doc::Placement;
use punchi_fluid_draw::{Pt, port_point};

use super::{Probe, sheet_harness, window_with};

/// A primary drag from `from` to `to`, as a hand would deliver it: one frame per step, so the
/// press, the move and the release are seen in order and the drag can start.
fn drag(harness: &mut egui_kittest::Harness<'_, Probe>, from: egui::Pos2, to: egui::Pos2) {
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

/// Places two ports on the sheet and returns their millimetre coordinates and page mapping.
fn window_with_ports(
    from_id: &str,
    from_port: &str,
    to_id: &str,
    to_port: &str,
) -> (super::Window, Pt, Pt, Pt, f32) {
    let mut window = window_with();
    let page = window.page.rect();
    let centre = page.center();
    let from_index = window.document.place(Placement::new(
        from_id,
        Pt::new(centre.x - 60.0, centre.y),
        24.0,
        24.0,
    ));
    let to_index = window.document.place(Placement::new(
        to_id,
        Pt::new(centre.x + 60.0, centre.y),
        24.0,
        24.0,
    ));
    let from_placement = &window.document.placements()[from_index];
    let from_definition = window
        .catalog
        .get(&from_placement.catalog_id)
        .expect("the source component is in the catalog");
    let from = port_point(
        from_placement.rect(),
        from_placement.orientation,
        from_definition,
        from_port,
    )
    .expect("the source port exists");
    let to_placement = &window.document.placements()[to_index];
    let to_definition = window
        .catalog
        .get(&to_placement.catalog_id)
        .expect("the target component is in the catalog");
    let to = port_point(
        to_placement.rect(),
        to_placement.orientation,
        to_definition,
        to_port,
    )
    .expect("the target port exists");
    (window, from, to, page.min, page.width())
}

fn screen_point(point: Pt, paper: egui::Rect, page_min: Pt, page_width: f32) -> egui::Pos2 {
    let scale = paper.width() / page_width;
    paper.min
        + egui::vec2(
            (point.x - page_min.x) * scale,
            (point.y - page_min.y) * scale,
        )
}

fn begin_drag(harness: &mut egui_kittest::Harness<'_, Probe>, from: egui::Pos2, to: egui::Pos2) {
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
        .push(egui::Event::PointerMoved(to));
    harness.run();
}

fn release_drag(harness: &mut egui_kittest::Harness<'_, Probe>, at: egui::Pos2) {
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: at,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::default(),
    });
    harness.run();
}

#[test]
fn dragging_between_pneumatic_ports_creates_one_tubed_connection_and_undoes_it() {
    let (mut window, from, to, page_min, page_width) =
        window_with_ports("supply.compressor", "1", "valve.5_2.base", "1");
    let initial_centres = window
        .document
        .placements()
        .iter()
        .map(|placement| (placement.centre.x, placement.centre.y))
        .collect::<Vec<_>>();
    let mut harness = sheet_harness(&mut window, egui::vec2(900.0, 700.0));
    let paper = harness.state().sheet.expect("the sheet is drawn").paper;
    let from = screen_point(from, paper, page_min, page_width);
    let to = screen_point(to, paper, page_min, page_width);

    begin_drag(&mut harness, from, to);
    assert!(
        harness.state().connection_dragging,
        "the hose preview is active"
    );
    release_drag(&mut harness, to);

    let result = harness.state();
    assert_eq!(result.connection_tubes, [Some(DEFAULT_TUBE_HUNDREDTHS)]);
    assert_eq!(
        result.centres, initial_centres,
        "the drag did not move a symbol"
    );
    assert!(!result.connection_dragging, "the gesture ended on release");

    harness.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::Z);
    harness.run();
    assert!(
        harness.state().connection_tubes.is_empty(),
        "one undo removes the connection"
    );
}

#[test]
fn dropping_a_port_drag_on_empty_sheet_cancels_it() {
    let (mut window, from, _, page_min, page_width) =
        window_with_ports("supply.compressor", "1", "valve.5_2.base", "1");
    let mut harness = sheet_harness(&mut window, egui::vec2(900.0, 700.0));
    let paper = harness.state().sheet.expect("the sheet is drawn").paper;
    let from = screen_point(from, paper, page_min, page_width);
    let empty = paper.center();

    drag(&mut harness, from, empty);

    let result = harness.state();
    assert!(result.connection_tubes.is_empty(), "no pipe was added");
    assert!(!result.connection_dragging, "the cancelled gesture ended");
}

#[test]
fn escape_cancels_a_port_drag_without_adding_a_connection() {
    let (mut window, from, to, page_min, page_width) =
        window_with_ports("supply.compressor", "1", "valve.5_2.base", "1");
    let mut harness = sheet_harness(&mut window, egui::vec2(900.0, 700.0));
    let paper = harness.state().sheet.expect("the sheet is drawn").paper;
    let from = screen_point(from, paper, page_min, page_width);
    let to = screen_point(to, paper, page_min, page_width);
    begin_drag(&mut harness, from, to);
    assert!(
        harness.state().connection_dragging,
        "the connection is previewed"
    );

    harness.key_press(egui::Key::Escape);
    harness.run();

    let result = harness.state();
    assert!(result.connection_tubes.is_empty(), "Esc adds no connection");
    assert!(!result.connection_dragging, "Esc ends the preview");
}

#[test]
fn connecting_a_port_to_itself_is_refused_and_reported() {
    let (mut window, from, _, page_min, page_width) =
        window_with_ports("supply.compressor", "1", "valve.5_2.base", "1");
    let mut harness = sheet_harness(&mut window, egui::vec2(900.0, 700.0));
    let paper = harness.state().sheet.expect("the sheet is drawn").paper;
    let from = screen_point(from, paper, page_min, page_width);

    begin_drag(&mut harness, from, from + egui::vec2(10.0, 0.0));
    release_drag(&mut harness, from);

    let result = harness.state();
    assert!(
        result.connection_tubes.is_empty(),
        "the invalid pipe is absent"
    );
    assert_eq!(result.notice.as_deref(), Some("connection refused"));
}

#[test]
fn dragging_between_electrical_terminals_creates_a_wire_without_a_tube() {
    let (mut window, from, to, page_min, page_width) =
        window_with_ports("electrical.supply", "L", "electrical.button.mushroom", "1");
    let mut harness = sheet_harness(&mut window, egui::vec2(900.0, 700.0));
    let paper = harness.state().sheet.expect("the sheet is drawn").paper;
    let from = screen_point(from, paper, page_min, page_width);
    let to = screen_point(to, paper, page_min, page_width);

    drag(&mut harness, from, to);

    assert_eq!(harness.state().connection_tubes, [None]);
}

#[test]
fn escape_cancels_the_zoom_window() {
    let mut window = window_with();
    window.zoom_window_armed = true;
    let mut harness = sheet_harness(&mut window, egui::vec2(900.0, 700.0));
    assert!(harness.state().zoom_armed, "the zoom window starts armed");
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(!harness.state().zoom_armed, "Esc disarms the zoom window");
}

#[test]
fn the_zoom_window_magnifies_the_rectangle() {
    let mut window = window_with();
    window.zoom_window_armed = true;
    let mut harness = sheet_harness(&mut window, egui::vec2(900.0, 700.0));
    let centre = harness.state().sheet.expect("a sheet frame").paper.center();
    drag(
        &mut harness,
        centre - egui::vec2(80.0, 60.0),
        centre + egui::vec2(80.0, 60.0),
    );
    let probe = harness.state();
    assert!(!probe.zoom_armed, "the release ends the zoom window");
    assert!(probe.zoom > 1.0, "the zoom window magnified the sheet");
}

#[test]
fn the_hand_mode_pans_and_leaves_on_escape() {
    let mut window = window_with();
    window.hand_mode = true;
    window.view.zoom_in();
    window.view.zoom_in();
    let mut harness = sheet_harness(&mut window, egui::vec2(900.0, 700.0));
    let centre = harness.state().sheet.expect("a sheet frame").paper.center();
    let before = harness.state().pan;
    drag(&mut harness, centre, centre + egui::vec2(40.0, 20.0));
    let after = harness.state().pan;
    assert!(
        (after - before).length() > 10.0,
        "the hand drag panned the sheet: {before:?} -> {after:?}"
    );
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(!harness.state().hand, "Esc leaves the hand mode");
}
