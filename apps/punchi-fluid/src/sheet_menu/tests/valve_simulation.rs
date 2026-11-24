// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Valve input belongs to a running simulation; stopped valves remain editable.

use eframe::egui;

use crate::ribbon::Command;

use super::{harness, page_centre, primary_press_at, primary_release_at, window_with_a_model};

#[test]
fn a_stopped_valve_is_draggable_without_being_actuated() {
    let mut window = window_with_a_model();
    let mut harness = harness(&mut window);
    let origin = page_centre(&harness);
    let before = harness
        .state()
        .valve_centre
        .expect("the circuit has a valve centre");

    primary_press_at(&mut harness, origin);
    assert_eq!(harness.state().valve_position, 0);
    assert!(!harness.state().pressed_valve);

    let drag_start = origin + egui::vec2(8.0, 0.0);
    harness
        .input_mut()
        .events
        .push(egui::Event::PointerMoved(drag_start));
    harness.run();
    let drop = origin + egui::vec2(40.0, 20.0);
    harness
        .input_mut()
        .events
        .push(egui::Event::PointerMoved(drop));
    harness.run();
    assert_eq!(harness.state().valve_position, 0);
    assert!(!harness.state().pressed_valve);
    primary_release_at(&mut harness, drop);

    let after = harness
        .state()
        .valve_centre
        .expect("the moved valve keeps its centre");
    assert!(after.x > before.x, "the stopped valve can be dragged");
    assert!(after.y > before.y, "the drag commits its vertical offset");
    assert_eq!(harness.state().valve_position, 0);
}

#[test]
fn a_running_valve_is_actuated_only_while_the_primary_button_is_held() {
    let mut window = window_with_a_model();
    window.running = true;
    let mut harness = harness(&mut window);
    let centre = page_centre(&harness);

    primary_press_at(&mut harness, centre);
    assert!(harness.state().pressed_valve);
    assert_ne!(harness.state().valve_position, 0);
    primary_release_at(&mut harness, centre);
    assert!(!harness.state().pressed_valve);
    assert_eq!(harness.state().valve_position, 0);
}

#[test]
fn pausing_releases_a_valve_held_by_the_pointer() {
    let mut window = window_with_a_model();
    window.running = true;
    {
        let mut harness = harness(&mut window);
        let centre = page_centre(&harness);
        primary_press_at(&mut harness, centre);
        assert!(harness.state().pressed_valve);
        assert_ne!(harness.state().valve_position, 0);
    }

    window.dispatch(Command::Pause);
    assert!(!window.running);
    assert!(window.pressed_valve.is_none());
    assert_eq!(
        window.model.as_ref().map(|model| model.valve_position(1)),
        Some(0),
        "pausing releases the spring-return valve"
    );
}
