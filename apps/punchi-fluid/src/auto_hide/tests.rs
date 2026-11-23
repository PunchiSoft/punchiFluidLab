// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The auto-hide timers and the window wiring that applies them: the dwell, the linger, a
//! return cancelling the close, and the three states that suspend the whole behaviour. The
//! clock is made up, so the tests never sleep.

use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use eframe::egui;
use punchi_fluid_doc::Document;

use super::{AutoHide, DWELL, LINGER, Slot};
use crate::Window;
use crate::theme::Theme;

/// A window the tests can drive: no path, no dialog, no safety copy.
fn window() -> Window {
    Window::new(Theme::fallback(), Document::new(), None, None)
}

/// A window whose panels are the screen (not Inicio) with auto-hide on.
fn autohide_window() -> Window {
    let mut window = window();
    window.start_open = false;
    window.autohide_on = true;
    window
}

#[test]
fn the_rail_opens_only_after_the_dwell() {
    let mut timers = AutoHide::new();
    let start = Instant::now();
    assert!(!timers.rail(Slot::Catalog, true, false, start));
    assert!(
        !timers.rail(Slot::Catalog, true, false, start + DWELL / 2),
        "half the dwell is not the dwell"
    );
    assert!(timers.rail(Slot::Catalog, true, false, start + DWELL));
}

#[test]
fn leaving_the_rail_cancels_the_dwell() {
    let mut timers = AutoHide::new();
    let start = Instant::now();
    assert!(!timers.rail(Slot::Catalog, true, false, start));
    assert!(!timers.rail(Slot::Catalog, false, false, start + DWELL));
    assert!(
        !timers.rail(Slot::Catalog, true, false, start + DWELL * 2),
        "the dwell starts again from zero"
    );
    assert!(timers.rail(Slot::Catalog, true, false, start + DWELL * 3));
}

#[test]
fn the_panel_closes_only_after_the_linger_and_a_return_cancels_it() {
    let mut timers = AutoHide::new();
    let start = Instant::now();
    assert!(!timers.away(Slot::Inspector, false, false, start));
    assert!(!timers.away(Slot::Inspector, false, false, start + LINGER / 2));
    assert!(
        !timers.away(Slot::Inspector, true, false, start + LINGER),
        "the return cancels the pending close"
    );
    assert!(!timers.away(Slot::Inspector, false, false, start + LINGER * 2));
    assert!(timers.away(Slot::Inspector, false, false, start + LINGER * 3));
}

#[test]
fn a_blocked_frame_forgets_both_timers() {
    let mut timers = AutoHide::new();
    let start = Instant::now();
    assert!(!timers.rail(Slot::Catalog, true, false, start));
    assert!(
        !timers.rail(Slot::Catalog, true, true, start + DWELL),
        "a drag or a dialog stops the dwell"
    );
    assert!(!timers.away(Slot::Catalog, false, true, start + DWELL * 2));
    assert!(!timers.away(Slot::Catalog, false, true, start + DWELL * 2 + LINGER));
}

#[test]
fn the_wait_names_the_next_timer_to_wake_for() {
    let mut timers = AutoHide::new();
    let start = Instant::now();
    assert_eq!(timers.wait(start), None, "nothing pending, nothing to wait");
    assert!(!timers.rail(Slot::Catalog, true, false, start));
    assert_eq!(timers.wait(start), Some(DWELL));
    assert_eq!(timers.wait(start + DWELL / 2), Some(DWELL / 2));
    timers.cancel_all();
    assert_eq!(timers.wait(start), None);
}

#[test]
fn the_dwell_opens_a_folded_panel_and_the_linger_closes_it_again() {
    let mut window = autohide_window();
    window.sidebar.close();
    window.sidebar.probe_for_test(true, false, false);
    let start = Instant::now();
    assert!(!window.apply_auto_hide(false, start));
    assert!(!window.sidebar.is_open(), "nothing opens before the dwell");
    assert!(window.apply_auto_hide(false, start + DWELL));
    assert!(window.sidebar.is_open(), "the dwell opens the catalog");

    // The pointer is not on the panel it opened and stays away for the whole linger. The other
    // panel is held with the pointer inside it, so its own linger does not muddy the answer.
    window.inspector.probe_for_test(false, true, false);
    window.sidebar.probe_for_test(false, false, false);
    let away = start + DWELL;
    assert!(!window.apply_auto_hide(false, away));
    assert!(window.sidebar.is_open(), "nothing closes before the linger");
    assert!(window.apply_auto_hide(false, away + LINGER));
    assert!(!window.sidebar.is_open(), "the linger closes it again");
}

#[test]
fn a_return_to_the_panel_cancels_the_pending_close() {
    let mut window = autohide_window();
    window.inspector.open();
    window.inspector.probe_for_test(false, false, false);
    // The catalog is held with the pointer inside it: this test is about the dock's timer.
    window.sidebar.probe_for_test(false, true, false);
    let start = Instant::now();
    assert!(!window.apply_auto_hide(false, start));
    // The pointer comes back before the linger is over and leaves again later.
    window.inspector.probe_for_test(false, true, false);
    assert!(!window.apply_auto_hide(false, start + LINGER / 2));
    window.inspector.probe_for_test(false, false, false);
    let back = start + LINGER;
    assert!(!window.apply_auto_hide(false, back));
    assert!(window.inspector.is_open(), "the return cancelled the close");
    assert!(window.apply_auto_hide(false, back + LINGER));
    assert!(!window.inspector.is_open(), "the new linger does close it");
}

#[test]
fn the_keyboard_inside_the_panel_suspends_the_close() {
    let mut window = autohide_window();
    window.sidebar.open();
    window.sidebar.probe_for_test(false, false, true);
    // The dock is held with the pointer inside it: this test is about the catalog's timer.
    window.inspector.probe_for_test(false, true, false);
    let start = Instant::now();
    assert!(!window.apply_auto_hide(false, start));
    assert!(
        !window.apply_auto_hide(false, start + LINGER * 2),
        "the keyboard inside the panel keeps it open"
    );
    assert!(window.sidebar.is_open());
}

#[test]
fn an_open_dialog_suspends_the_timers() {
    let mut window = autohide_window();
    window.sidebar.open();
    window.sidebar.probe_for_test(false, false, false);
    window.open_about();
    assert!(window.dialog_open(), "the About panel is a dialog");
    let context = egui::Context::default();
    window.drive_auto_hide(&context, Instant::now() + LINGER * 2);
    assert!(
        window.sidebar.is_open(),
        "the dialog keeps the panel open while it is in front"
    );
}

#[test]
fn a_pointer_drag_suspends_the_timers() {
    let mut window = autohide_window();
    window.sidebar.open();
    window.sidebar.probe_for_test(false, false, false);
    let clock = Rc::new(Cell::new(Instant::now()));
    {
        let mut harness = egui_kittest::Harness::new(|context| {
            window.drive_auto_hide(context, clock.get());
        });
        harness.set_size(egui::vec2(900.0, 600.0));
        // The linger starts on a still pointer, and would close the panel at the next step.
        harness.step();
        let position = egui::pos2(400.0, 300.0);
        harness
            .input_mut()
            .events
            .push(egui::Event::PointerMoved(position));
        harness.input_mut().events.push(egui::Event::PointerButton {
            pos: position,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::default(),
        });
        clock.set(clock.get() + LINGER + Duration::from_millis(1));
        harness.step();
    }
    assert!(window.sidebar.is_open(), "a drag in progress keeps it open");
}
