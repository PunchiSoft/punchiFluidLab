// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The window geometry's tests: the first size against synthetic screens, the clamp of a
//! remembered one, the off-screen place and the state file's defensive read.

use std::path::PathBuf;
use std::time::Instant;

use eframe::egui::{Pos2, Vec2};

use super::{
    FILE_NAME, Geometry, MAX_BYTES, MIN_HEIGHT, MIN_WIDTH, SETTLE, Store, first_geometry, minimum,
    opening_geometry, parse, provisional, read, reanchored, remembered_geometry, restored_geometry,
    write,
};

/// The default spacing unit: the desktop font of the reference machine at 10 pt, 16 points.
const UNIT: f32 = 16.0;

/// A folder of its own for the test, removed at the end where possible.
fn scratch() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "punchi-fluid-window-state-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the scratch folder");
    dir
}

fn geometry(size: (f32, f32), position: Option<(f32, f32)>, maximized: bool) -> Geometry {
    Geometry {
        size: Vec2::new(size.0, size.1),
        position: position.map(|(x, y)| Pos2::new(x, y)),
        maximized,
    }
}

#[test]
fn the_first_window_is_the_preferred_size_on_a_screen_with_room() {
    let geometry = first_geometry(Vec2::new(1920.0, 1080.0), UNIT);
    assert_eq!(geometry.size, Vec2::new(1440.0, 900.0));
    assert_eq!(
        geometry.position,
        Some(Pos2::new(240.0, 50.0)),
        "the preferred window is centred inside the room above the taskbar"
    );
    assert!(!geometry.maximized);
}

#[test]
fn a_smaller_screen_takes_the_largest_size_that_fits_with_the_two_unit_margin() {
    let geometry = first_geometry(Vec2::new(1366.0, 768.0), UNIT);
    assert_eq!(
        geometry.size,
        Vec2::new(1334.0, 656.0),
        "the screen minus two 16 point units and the chrome reserve"
    );
    assert_eq!(geometry.position, Some(Pos2::new(16.0, 16.0)));
    assert!(
        geometry.maximized,
        "the preferred 900 points of height do not fit and the screen opens maximized"
    );
}

#[test]
fn a_screen_below_the_floor_keeps_its_room_as_the_minimum() {
    let screen = Vec2::new(800.0, 600.0);
    assert_eq!(
        minimum(screen, UNIT),
        Vec2::new(768.0, 488.0),
        "the floor never makes the window bigger than the screen"
    );
    let geometry = first_geometry(screen, UNIT);
    assert_eq!(geometry.size, Vec2::new(768.0, 488.0));
    assert_eq!(geometry.position, Some(Pos2::new(16.0, 16.0)));
    assert!(geometry.maximized);
}

#[test]
fn the_minimum_follows_the_room_and_keeps_the_design_floor_when_it_fits() {
    assert_eq!(
        minimum(Vec2::new(1920.0, 1080.0), UNIT),
        Vec2::new(900.0, 600.0)
    );
    assert_eq!(
        minimum(Vec2::new(1093.0, 614.0), UNIT),
        Vec2::new(900.0, 502.0),
        "1366 × 768 at 125 % has 502 points of height above the taskbar"
    );
}

#[test]
fn the_opening_geometry_is_the_first_one_without_a_state_and_the_restored_one_with_it() {
    let screen = Vec2::new(1920.0, 1080.0);
    let first = opening_geometry(screen, UNIT, None);
    assert_eq!(first.size, Vec2::new(1440.0, 900.0));
    let saved = geometry((1200.0, 800.0), Some((100.0, 50.0)), false);
    let restored = opening_geometry(screen, UNIT, Some(&saved));
    assert_eq!(restored.size, Vec2::new(1200.0, 800.0));
    assert_eq!(restored.position, saved.position);
}

#[test]
fn the_viewport_is_created_with_the_remembered_size_or_the_floor() {
    let floor = provisional(None);
    assert_eq!(floor.size, Vec2::new(MIN_WIDTH, MIN_HEIGHT));
    assert_eq!(floor.position, None);
    assert!(!floor.maximized);
    let saved = geometry((1200.0, 800.0), Some((100.0, 50.0)), true);
    assert_eq!(
        provisional(Some(&saved)),
        saved,
        "a returning window is created with the size and the maximized state it had"
    );
}

#[test]
fn a_window_left_outside_the_opening_monitor_is_brought_back() {
    let screen = Vec2::new(1366.0, 768.0);
    let outside = geometry((1200.0, 700.0), Some((2000.0, 100.0)), false);
    let fixed = reanchored(screen, UNIT, &outside).expect("the window is outside the room");
    assert_eq!(fixed.size, Vec2::new(1200.0, 656.0));
    assert_eq!(fixed.position, Some(Pos2::new(83.0, 16.0)));

    let inside = geometry((1200.0, 650.0), Some((100.0, 50.0)), false);
    assert_eq!(reanchored(screen, UNIT, &inside), None);
    let wayland = geometry((1200.0, 700.0), None, false);
    assert_eq!(
        reanchored(screen, UNIT, &wayland),
        None,
        "without a place the compositor decides"
    );
}

#[test]
fn a_remembered_geometry_is_clamped_to_a_smaller_screen() {
    // Saved on a 1920 monitor, reopened on a 1366 laptop.
    let saved = geometry((1920.0, 1080.0), Some((500.0, 300.0)), false);
    let restored = restored_geometry(Vec2::new(1366.0, 768.0), UNIT, &saved);
    assert_eq!(
        restored.size,
        Vec2::new(1334.0, 656.0),
        "a window saved on 1920 reopens inside a 1366 laptop"
    );
    assert_eq!(
        restored.position,
        Some(Pos2::new(16.0, 16.0)),
        "the place is clamped into the room's margin"
    );
}

#[test]
fn a_remembered_size_below_the_floor_grows_to_the_minimum() {
    let saved = geometry((700.0, 500.0), Some((10.0, 10.0)), false);
    let restored = restored_geometry(Vec2::new(1920.0, 1080.0), UNIT, &saved);
    assert_eq!(restored.size, Vec2::new(900.0, 600.0));
    assert_eq!(restored.position, Some(Pos2::new(16.0, 16.0)));
}

#[test]
fn a_position_off_every_screen_is_rejected_and_the_window_opens_centred() {
    let saved = geometry((1200.0, 800.0), Some((-4000.0, -3000.0)), false);
    let restored = restored_geometry(Vec2::new(1366.0, 768.0), UNIT, &saved);
    assert_eq!(restored.size, Vec2::new(1200.0, 656.0));
    assert_eq!(
        restored.position,
        Some(Pos2::new(83.0, 16.0)),
        "an off-screen place falls back to the centre"
    );

    let aside = geometry((1200.0, 800.0), Some((5000.0, 4000.0)), false);
    let restored = restored_geometry(Vec2::new(1366.0, 768.0), UNIT, &aside);
    assert_eq!(restored.position, Some(Pos2::new(83.0, 16.0)));
}

#[test]
fn a_maximized_window_keeps_the_size_it_had_before() {
    let normal = geometry((1200.0, 800.0), Some((100.0, 50.0)), false);
    let live = geometry((1920.0, 1080.0), Some((0.0, 0.0)), true);
    let kept = remembered_geometry(live, Some(&normal));
    assert_eq!(
        kept.size, normal.size,
        "the maximized size is not remembered"
    );
    assert_eq!(kept.position, normal.position);
    assert!(kept.maximized);

    let restored = restored_geometry(Vec2::new(1920.0, 1080.0), UNIT, &kept);
    assert_eq!(
        restored.size, normal.size,
        "unmaximizing returns the old size"
    );
    assert!(restored.maximized, "the window still opens maximized");
}

#[test]
fn a_maximized_window_on_a_smaller_screen_keeps_the_clamped_previous_size() {
    let normal = geometry((1900.0, 1000.0), Some((10.0, 10.0)), false);
    let live = geometry((1920.0, 1080.0), None, true);
    let kept = remembered_geometry(live, Some(&normal));
    let restored = restored_geometry(Vec2::new(1366.0, 768.0), UNIT, &kept);
    assert_eq!(restored.size, Vec2::new(1334.0, 656.0));
    assert!(restored.maximized);
}

#[test]
fn the_latam_screen_matrix_opens_a_window_that_fits() {
    // The monitors the courses use, the normal size each opens with and whether it opens
    // maximized. The reserved room is the monitor minus two 16 point units and 80 points of
    // chrome; a normal window never crosses it, and a maximized one keeps a normal size that
    // fits for the day it is unmaximized.
    let cases = [
        ((1024.0, 768.0), (992.0, 656.0), true),
        ((1280.0, 720.0), (1248.0, 608.0), true),
        ((1280.0, 800.0), (1248.0, 688.0), true),
        ((1366.0, 768.0), (1334.0, 656.0), true),
        ((1536.0, 864.0), (1440.0, 752.0), true),
        ((1600.0, 900.0), (1440.0, 788.0), true),
        ((1280.0, 1024.0), (1248.0, 900.0), false),
        ((1920.0, 1080.0), (1440.0, 900.0), false),
        ((1920.0, 1200.0), (1440.0, 900.0), false),
        ((2560.0, 1440.0), (1440.0, 900.0), false),
        ((3840.0, 2160.0), (1440.0, 900.0), false),
        ((1093.0, 614.0), (1061.0, 502.0), true),
    ];
    for ((width, height), (expected_width, expected_height), maximized) in cases {
        let screen = Vec2::new(width, height);
        let geometry = first_geometry(screen, UNIT);
        assert_eq!(
            geometry.size,
            Vec2::new(expected_width, expected_height),
            "{width}×{height}"
        );
        assert_eq!(geometry.maximized, maximized, "{width}×{height}");
        let room = Vec2::new(width - 32.0, height - 32.0 - 80.0);
        assert!(
            geometry.size.x <= room.x && geometry.size.y <= room.y,
            "{width}×{height} keeps its normal size inside the room"
        );
        let position = geometry.position.expect("the first window is placed");
        assert!(
            position.x >= 16.0 && position.y >= 16.0,
            "{width}×{height} keeps the margin"
        );
        assert!(
            position.x + geometry.size.x <= 16.0 + room.x + 0.01
                && position.y + geometry.size.y <= 16.0 + room.y + 0.01,
            "{width}×{height} never leaves the room"
        );
    }
}

#[test]
fn the_state_file_round_trips() {
    let dir = scratch();
    let file = dir.join(FILE_NAME);
    let saved = geometry((1330.25, 732.5), Some((18.5, 20.25)), true);
    assert!(write(&file, &saved));
    assert_eq!(read(&file), Some(saved));
    assert!(
        !crate::state_file::temporary_path(&file).exists(),
        "the write leaves no temporary file behind"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_state_file_without_a_place_round_trips_and_opens_centred() {
    let dir = scratch();
    let file = dir.join(FILE_NAME);
    let saved = geometry((1330.0, 732.0), None, false);
    assert!(write(&file, &saved));
    assert_eq!(read(&file), Some(saved));
    let restored = restored_geometry(Vec2::new(1366.0, 768.0), UNIT, &saved);
    assert_eq!(restored.position, Some(Pos2::new(18.0, 16.0)));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_truncated_file_reads_as_no_state_at_all() {
    let dir = scratch();
    let file = dir.join(FILE_NAME);
    std::fs::write(&file, "version=1\nwidth=1440.000\nheight=").expect("the broken file");
    assert_eq!(read(&file), None);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_truncated_place_drops_only_the_place() {
    let parsed = parse("version=1\nwidth=1440\nheight=900\nx=10").expect("the size survives");
    assert_eq!(parsed.size, Vec2::new(1440.0, 900.0));
    assert_eq!(parsed.position, None);
}

#[test]
fn an_out_of_range_size_is_ignored() {
    let dir = scratch();
    let file = dir.join(FILE_NAME);
    std::fs::write(&file, "version=1\nwidth=999999999\nheight=900\n").expect("the bad file");
    assert_eq!(read(&file), None);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_state_file_over_the_size_bound_is_ignored() {
    let dir = scratch();
    let file = dir.join(FILE_NAME);
    let huge = format!("width=1440\nheight=900\n#{}", "x".repeat(MAX_BYTES));
    std::fs::write(&file, huge).expect("the huge file");
    assert_eq!(read(&file), None);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_state_file_from_a_newer_format_is_ignored() {
    assert_eq!(parse("version=2\nwidth=1440\nheight=900\n"), None);
}

#[test]
fn unknown_keys_and_comments_are_ignored() {
    let parsed = parse("# a comment\nversion=1\nwidth=1440\nfuture=yes\nheight=900\n")
        .expect("the known keys are enough");
    assert_eq!(parsed.size, Vec2::new(1440.0, 900.0));
}

#[test]
fn a_file_without_a_size_is_ignored() {
    assert_eq!(parse("width=1440\n"), None);
    assert_eq!(parse(""), None);
    assert_eq!(parse("esto no es el formato"), None);
}

#[test]
fn the_store_writes_once_the_geometry_settles() {
    let dir = scratch();
    let file = dir.join(FILE_NAME);
    let mut store = Store::at(Some(file.clone()));
    assert_eq!(store.remembered(), None);

    let live = geometry((1200.0, 800.0), Some((100.0, 80.0)), false);
    let start = Instant::now();
    assert!(
        !store.observe(live, None, start),
        "the first note is not a write"
    );
    assert!(
        !store.observe(live, None, start + SETTLE / 2),
        "still moving"
    );
    assert!(!file.exists(), "a moving window does not write the file");
    assert!(
        store.observe(live, None, start + SETTLE),
        "the settled geometry is written"
    );
    assert_eq!(read(&file), Some(live));

    let reread = Store::at(Some(file.clone()));
    assert_eq!(reread.remembered(), Some(&live));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_store_keeps_the_previous_size_of_a_maximized_window() {
    let dir = scratch();
    let file = dir.join(FILE_NAME);
    let mut store = Store::at(Some(file.clone()));
    let normal = geometry((1200.0, 800.0), Some((100.0, 50.0)), false);
    let start = Instant::now();
    let _ = store.observe(normal, None, start);
    let _ = store.observe(normal, None, start + SETTLE);
    let live = geometry((1920.0, 1080.0), Some((0.0, 0.0)), true);
    assert!(store.remember(super::remembered_geometry(live, Some(&normal))));
    assert_eq!(
        read(&file),
        Some(super::remembered_geometry(live, Some(&normal)))
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_store_without_a_path_keeps_the_geometry_in_memory() {
    let mut store = Store::at(None);
    let live = geometry((1200.0, 800.0), None, false);
    assert!(!store.remember(live), "nothing is written without a path");
    assert_eq!(store.remembered(), None);
}
