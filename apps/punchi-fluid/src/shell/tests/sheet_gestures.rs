// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The view gestures that need a pointer: the zoom window's rectangle and the hand's drag.

use eframe::egui;

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
