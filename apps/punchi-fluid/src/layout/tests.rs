// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The layout tests: the mode follows from the room, never from a list of screen sizes, and
//! the sheet's floor survives every allocation.

use super::{
    CATALOG_MIN_WIDTH, CATALOG_PREFERRED_WIDTH, DOCK_MIN_WIDTH, DOCK_PREFERRED_WIDTH, LayoutMode,
    RAIL_WIDTH, SHEET_MIN_WIDTH, drawing_area, view_strip_gutter,
};
use eframe::egui::{Rect, pos2, vec2};

#[test]
fn the_drawing_area_leaves_the_strip_gutter_on_the_right() {
    let canvas = Rect::from_min_size(pos2(10.0, 20.0), vec2(800.0, 600.0));
    let drawing = drawing_area(canvas, 16.0);
    assert!(
        (drawing.min.x - canvas.min.x).abs() < f32::EPSILON
            && (drawing.min.y - canvas.min.y).abs() < f32::EPSILON
    );
    assert!(
        (drawing.max.y - (canvas.max.y - super::circuit_strip_height(16.0))).abs() < f32::EPSILON,
        "the drawing area reserves the circuit strip's band at the foot"
    );
    let strip = super::circuit_strip_rect(canvas, 16.0);
    assert!((strip.max.y - canvas.max.y).abs() < f32::EPSILON);
    assert!((strip.min.y - drawing.max.y).abs() < f32::EPSILON);
    assert!(
        strip.width() < canvas.width(),
        "the view strip keeps its gutter on the right"
    );
    assert!((view_strip_gutter(16.0) - 56.0).abs() < f32::EPSILON);
    assert!((canvas.right() - drawing.right() - view_strip_gutter(16.0)).abs() < f32::EPSILON);
}

#[test]
fn a_canvas_narrower_than_the_gutter_keeps_a_zero_width_drawing_area() {
    let canvas = Rect::from_min_size(pos2(0.0, 0.0), vec2(20.0, 600.0));
    let drawing = drawing_area(canvas, 16.0);
    assert!((drawing.right() - canvas.left()).abs() < f32::EPSILON);
}

#[test]
fn the_thresholds_are_the_declared_measurements() {
    assert!(
        (LayoutMode::wide_from()
            - (SHEET_MIN_WIDTH + CATALOG_PREFERRED_WIDTH + DOCK_PREFERRED_WIDTH))
            .abs()
            <= f32::EPSILON
    );
    assert!(
        (LayoutMode::medium_from() - (SHEET_MIN_WIDTH + CATALOG_MIN_WIDTH + DOCK_MIN_WIDTH)).abs()
            <= f32::EPSILON
    );
    // The numbers the interface document states, so a stray edit is visible here.
    assert!((LayoutMode::wide_from() - 1080.0).abs() <= f32::EPSILON);
    assert!((LayoutMode::medium_from() - 1000.0).abs() <= f32::EPSILON);
}

#[test]
fn the_three_modes_follow_from_the_available_width() {
    assert_eq!(LayoutMode::from_available(1920.0), LayoutMode::Wide);
    assert_eq!(
        LayoutMode::from_available(LayoutMode::wide_from()),
        LayoutMode::Wide
    );
    assert_eq!(
        LayoutMode::from_available(LayoutMode::wide_from() - 0.5),
        LayoutMode::Medium
    );
    assert_eq!(
        LayoutMode::from_available(LayoutMode::medium_from()),
        LayoutMode::Medium
    );
    assert_eq!(
        LayoutMode::from_available(LayoutMode::medium_from() - 0.5),
        LayoutMode::Compact
    );
    assert_eq!(LayoutMode::from_available(900.0), LayoutMode::Compact);
    assert_eq!(LayoutMode::from_available(0.0), LayoutMode::Compact);
}

#[test]
fn a_non_finite_room_is_compact_instead_of_panicking() {
    assert_eq!(LayoutMode::from_available(f32::NAN), LayoutMode::Compact);
    assert_eq!(LayoutMode::from_available(f32::INFINITY), LayoutMode::Wide);
}

#[test]
fn the_wide_mode_puts_everything_at_its_preferred_width() {
    let widths = LayoutMode::Wide.widths(1440.0, 240.0, 360.0);
    assert!((widths.catalog - CATALOG_PREFERRED_WIDTH).abs() <= f32::EPSILON);
    assert!((widths.dock - DOCK_PREFERRED_WIDTH).abs() <= f32::EPSILON);
    assert!(
        (LayoutMode::Wide.sheet_width(1440.0, 240.0, 360.0) - 840.0).abs() <= f32::EPSILON,
        "the sheet takes the rest in the wide mode"
    );
}

#[test]
fn the_wide_mode_still_keeps_the_sheet_floor_when_the_reader_widened_the_panels() {
    // Both panels at their furthest drag: the sheet's floor wins and the panels narrow.
    let widths = LayoutMode::Wide.widths(LayoutMode::wide_from(), 400.0, 520.0);
    let sheet = LayoutMode::wide_from() - widths.catalog - widths.dock;
    assert!(
        sheet >= SHEET_MIN_WIDTH - f32::EPSILON,
        "the sheet keeps its floor: {sheet}"
    );
    assert!(widths.catalog >= CATALOG_MIN_WIDTH);
    assert!(widths.dock >= DOCK_MIN_WIDTH);
}

#[test]
fn the_medium_mode_puts_both_panels_at_their_minimums() {
    let widths = LayoutMode::Medium.widths(LayoutMode::medium_from(), 240.0, 360.0);
    assert!((widths.catalog - CATALOG_MIN_WIDTH).abs() <= f32::EPSILON);
    assert!((widths.dock - DOCK_MIN_WIDTH).abs() <= f32::EPSILON);
    assert!(
        (LayoutMode::Medium.sheet_width(LayoutMode::medium_from(), 240.0, 360.0) - SHEET_MIN_WIDTH)
            .abs()
            <= f32::EPSILON,
        "at the medium threshold the sheet is exactly at its floor"
    );
}

#[test]
fn the_compact_mode_folds_both_panels_to_their_rails_and_leaves_the_sheet_its_floor() {
    let widths = LayoutMode::Compact.widths(528.0, 240.0, 360.0);
    assert!((widths.catalog - RAIL_WIDTH).abs() <= f32::EPSILON);
    assert!((widths.dock - RAIL_WIDTH).abs() <= f32::EPSILON);
    assert!(
        (LayoutMode::Compact.sheet_width(528.0, 240.0, 360.0) - SHEET_MIN_WIDTH).abs()
            <= f32::EPSILON,
        "two rails plus the sheet's floor is the narrowest compact shell"
    );
}

#[test]
fn the_mode_is_a_consequence_of_the_room_and_not_of_a_screen_list() {
    // The same function answers for a window and for a synthetic width: a 1.25x window at
    // 1200 points is the same question as an unscaled 1200 points.
    for available in [900.0, 999.0, 1000.0, 1079.0, 1080.0, 1366.0, 1920.0, 3840.0] {
        let mode = LayoutMode::from_available(available);
        let widths = mode.widths(available, 240.0, 360.0);
        let sheet = available - widths.catalog - widths.dock;
        assert!(
            sheet >= SHEET_MIN_WIDTH - f32::EPSILON,
            "at {available} the sheet kept {sheet} points"
        );
        assert_eq!(
            mode,
            LayoutMode::from_available(available),
            "the derivation is a pure function of the room"
        );
    }
}

#[test]
fn the_mode_bounds_the_width_a_remembered_panel_may_open_at() {
    // A file written on a 1920 screen cannot make a panel wider than the medium mode holds.
    let bounds = LayoutMode::Medium.max_widths(1020.0);
    assert!((bounds.catalog - (1020.0 - SHEET_MIN_WIDTH - DOCK_MIN_WIDTH)).abs() <= f32::EPSILON);
    assert!((bounds.dock - (1020.0 - SHEET_MIN_WIDTH - CATALOG_MIN_WIDTH)).abs() <= f32::EPSILON);
    // The wide mode still keeps the sheet's floor and the other panel's floor.
    let bounds = LayoutMode::Wide.max_widths(1440.0);
    assert!(bounds.catalog + DOCK_MIN_WIDTH + SHEET_MIN_WIDTH <= 1440.0);
    assert!(bounds.dock + CATALOG_MIN_WIDTH + SHEET_MIN_WIDTH <= 1440.0);
    // Compact docks no panel: the drawer floats over the sheet at most as wide as the window.
    let bounds = LayoutMode::Compact.max_widths(900.0);
    assert!((bounds.catalog - 900.0).abs() <= f32::EPSILON);
    assert!((bounds.dock - 900.0).abs() <= f32::EPSILON);
    assert!((LayoutMode::Compact.max_widths(f32::NAN).catalog - RAIL_WIDTH).abs() <= f32::EPSILON);
}
