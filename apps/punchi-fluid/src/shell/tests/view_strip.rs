// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The floating view strip: it lives in the channel the drawing area reserves, dispatches the
//! commands the ribbon and the keyboard own, folds to one button and opens its zoom panel.

use eframe::egui;
use egui_kittest::kittest::Queryable;

use super::{sheet_harness, window_with};

#[test]
fn the_strip_buttons_dispatch_the_same_commands() {
    let mut window = window_with();
    let mut harness = sheet_harness(&mut window, egui::vec2(1200.0, 800.0));
    assert!(harness.query_by_label("Pan").is_some(), "the hand button");
    harness.get_by_label("Pan").click();
    harness.run();
    assert!(harness.state().hand, "the strip turned the hand on");
    assert!(
        harness.query_by_label("Stop panning").is_some(),
        "the name follows the state"
    );

    harness.get_by_label("Hide the title block").click();
    harness.run();
    assert!(!harness.state().block, "the strip hid the block");
    assert!(
        harness.query_by_label("Show the title block").is_some(),
        "the name says the action"
    );
    harness.get_by_label("Show the title block").click();
    harness.run();
    assert!(harness.state().block, "the strip showed the block");

    harness.get_by_label("Zoom").click();
    harness.run();
    assert!(harness.state().strip_panel, "the zoom panel opened");
    harness.get_by_label("Zoom out").click();
    harness.run();
    assert!(harness.state().zoom < 1.0, "the panel zoomed out");
    harness.get_by_label("Fit the frame").click();
    harness.run();
    assert!(
        (harness.state().zoom - 1.0).abs() < 0.01,
        "the strip fitted the frame"
    );
    harness.get_by_label("Zoom").click();
    harness.run();
    assert!(!harness.state().strip_panel, "the panel closed");
}

#[test]
fn the_strip_folds_and_unfolds() {
    let mut window = window_with();
    let mut harness = sheet_harness(&mut window, egui::vec2(1200.0, 800.0));
    harness.get_by_label("Fold the view strip").click();
    harness.run();
    assert!(harness.state().strip_folded, "the strip folded");
    assert!(
        harness.query_by_label("Pan").is_none(),
        "only the fold button stays"
    );
    assert!(
        harness.query_by_label("Show the view strip").is_some(),
        "the one button shows it"
    );
    harness.get_by_label("Show the view strip").click();
    harness.run();
    assert!(!harness.state().strip_folded, "the strip unfolded");
    assert!(harness.query_by_label("Pan").is_some());
}

#[test]
fn the_compact_window_starts_with_the_strip_folded() {
    let mut window = window_with();
    let mut harness = sheet_harness(&mut window, egui::vec2(900.0, 700.0));
    assert!(
        harness.query_by_label("Show the view strip").is_some(),
        "the compact strip starts folded to one button"
    );
    assert!(harness.query_by_label("Pan").is_none());
    harness.get_by_label("Show the view strip").click();
    harness.run();
    assert!(harness.state().strip_compact, "the compact strip opened");
    assert!(harness.query_by_label("Pan").is_some());
}

#[test]
fn every_strip_button_has_an_accessible_name() {
    let mut window = window_with();
    let harness = sheet_harness(&mut window, egui::vec2(1200.0, 800.0));
    for label in [
        "Pan",
        "Fit the frame",
        "Zoom",
        "Hide the title block",
        "Start",
        "Fold the view strip",
    ] {
        assert!(
            harness.query_by_label(label).is_some(),
            "the strip has no {label} button"
        );
    }
}

#[test]
fn the_strip_floats_over_the_canvas_and_clear_of_the_frame() {
    let mut window = window_with();
    let harness = sheet_harness(&mut window, egui::vec2(1200.0, 800.0));
    let sheet = harness.state().sheet.expect("a sheet frame");
    let strip = sheet.strip.expect("the strip's rectangle");
    let eps = 0.5;
    assert!(
        strip.min.x >= sheet.canvas.min.x - eps
            && strip.max.x <= sheet.canvas.max.x + eps
            && strip.min.y >= sheet.canvas.min.y - eps
            && strip.max.y <= sheet.canvas.max.y + eps,
        "the strip leaves the canvas: {strip:?} in {:?}",
        sheet.canvas
    );
    assert!(
        strip.min.x >= sheet.frame.max.x - eps,
        "the strip covers the drawing frame: {strip:?} vs {:?}",
        sheet.frame
    );
    assert!(
        sheet.frame.max.x <= sheet.drawing.max.x + eps,
        "the frame takes no width the strip did not leave"
    );
}
