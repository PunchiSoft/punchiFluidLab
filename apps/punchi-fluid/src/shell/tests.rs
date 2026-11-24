// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The shell's tests, headless: the mode follows the room, the compact mode folds both panels
//! to their rails, the drawer opens over the sheet and closes with `Esc`, with its hide
//! control and with a click outside, and the sheet keeps the room.

use std::cell::Cell;
use std::rc::Rc;

use eframe::egui;
use egui_kittest::kittest::Queryable;
use punchi_fluid_doc::Document;
use punchi_fluid_i18n::Language;

use crate::Window;
use crate::layout::LayoutMode;
use crate::ribbon::Command;
use crate::shell::{Drawer, SheetProbe};

mod circuit_strip;
mod sheet_gestures;
mod view_strip;
use crate::theme::Theme;
use crate::window_state::{self, Geometry};

/// A folder of its own for the geometry test.
fn scratch() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "punchi-fluid-shell-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the scratch folder");
    dir
}

/// A frame of the whole shell, with the monitor the test simulates. The first frame applies
/// the geometry once the screen is known, so the test injects it before stepping.
fn frame_harness(window: &mut Window, screen: egui::Vec2) -> egui_kittest::Harness<'_> {
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(900.0, 768.0))
        .with_max_steps(120)
        .build(move |context| window.frame(context));
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the shell tests need one");
    }
    harness
        .input_mut()
        .viewports
        .get_mut(&egui::ViewportId::ROOT)
        .expect("the root viewport")
        .monitor_size = Some(screen);
    harness
}

fn commands<'a>(harness: &'a egui_kittest::Harness<'_>) -> &'a [egui::ViewportCommand] {
    &harness
        .output()
        .viewport_output
        .get(&egui::ViewportId::ROOT)
        .expect("the root viewport output")
        .commands
}

/// Puts the harness's viewport on `screen`, at `position` and with `size`, so a frame reads the
/// window the way a real desktop reports it.
fn place_viewport(
    harness: &mut egui_kittest::Harness<'_>,
    screen: egui::Vec2,
    position: egui::Pos2,
    size: egui::Vec2,
) {
    let viewport = harness
        .input_mut()
        .viewports
        .get_mut(&egui::ViewportId::ROOT)
        .expect("the root viewport");
    viewport.monitor_size = Some(screen);
    viewport.outer_rect = Some(egui::Rect::from_min_size(position, size));
    viewport.inner_rect = Some(egui::Rect::from_min_size(position, size));
}

/// The central panel's default frame: eight points of inner margin on each side, so the
/// sheet's available width is the room the panels leave minus this inset.
const SHEET_INSET: f32 = 16.0;

/// What one frame says about the shell, read from the harness after it runs.
#[derive(Debug, Default)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "the probe records independent facts of one frame, not a state machine"
)]
struct Probe {
    sheet_width: f32,
    drawer: Option<Drawer>,
    sheet: Option<SheetProbe>,
    hand: bool,
    zoom_armed: bool,
    pan: egui::Vec2,
    zoom: f32,
    block: bool,
    strip_folded: bool,
    strip_panel: bool,
    strip_compact: bool,
    connection_tubes: Vec<Option<u16>>,
    connection_dragging: bool,
    centres: Vec<(f32, f32)>,
    can_undo: bool,
    notice: Option<String>,
}

/// A window the tests can draw: no path, no dialog, no safety copy.
fn window_with() -> Window {
    Window::new(Theme::fallback(), Document::new(), None, None)
}

/// The shell drawn headless at `size`. The closure is the same order `frame` uses, so the
/// panels, the drawer and the central region are the real ones.
fn shell_harness(window: &mut Window, size: egui::Vec2) -> egui_kittest::Harness<'_, Probe> {
    let mut harness = egui_kittest::Harness::builder()
        .with_size(size)
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(240)
        .build_state(
            move |context, probe: &mut Probe| {
                window.update_layout_mode(context);
                window.sync_commands(context);
                window.show_ribbon(context);
                window.show_status(context);
                window.show_regions(context);
                egui::CentralPanel::default().show(context, |ui| {
                    probe.sheet_width = ui.available_width();
                });
                probe.drawer = window.shell.drawer;
            },
            Probe::default(),
        );
    // The window ships no font: the harness gets the desktop's, as the app does at start.
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the shell tests need one");
    }
    harness.run();
    harness
}

/// The shell drawn headless with the real sheet, so the probe records the canvas, the drawing
/// area, the paper and the clip of the last frame.
fn sheet_harness(window: &mut Window, size: egui::Vec2) -> egui_kittest::Harness<'_, Probe> {
    let mut harness = egui_kittest::Harness::builder()
        .with_size(size)
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(240)
        .build_state(
            move |context, probe: &mut Probe| {
                window.update_layout_mode(context);
                window.sync_commands(context);
                window.show_ribbon(context);
                window.show_status(context);
                window.show_regions(context);
                window.show_sheet(context);
                probe.sheet = window.sheet_probe;
                probe.hand = window.hand_mode;
                probe.zoom_armed = window.zoom_window_armed;
                probe.pan = window.view.pan();
                probe.zoom = window.view.zoom();
                probe.block = window.show_block;
                probe.strip_folded = window.strip.folded;
                probe.strip_panel = window.strip.panel_open;
                probe.strip_compact = window.strip.compact_open;
                probe.connection_tubes = window
                    .document
                    .connections()
                    .iter()
                    .map(punchi_fluid_doc::Connection::tube_od_hundredths)
                    .collect();
                probe.connection_dragging = window.connection_drag.is_some();
                probe.centres = window
                    .document
                    .placements()
                    .iter()
                    .map(|placement| (placement.centre.x, placement.centre.y))
                    .collect();
                probe.can_undo = window.document.can_undo();
                probe.notice.clone_from(&window.notice);
            },
            Probe::default(),
        );
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the shell tests need one");
    }
    harness.run();
    harness
}

/// Moves the pointer to `position`, as the window would report it.
fn move_pointer(harness: &mut egui_kittest::Harness<'_, Probe>, position: egui::Pos2) {
    harness
        .input_mut()
        .events
        .push(egui::Event::PointerMoved(position));
    harness.run();
}

/// The shell drawn headless with the catalog's probe reported after each frame.
fn shell_probe_harness(
    window: &mut Window,
    size: egui::Vec2,
) -> (
    egui_kittest::Harness<'_, Probe>,
    Rc<Cell<crate::panel::Probe>>,
) {
    let reported = Rc::new(Cell::new(crate::panel::Probe::default()));
    let sink = Rc::clone(&reported);
    let mut harness = egui_kittest::Harness::builder()
        .with_size(size)
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(240)
        .build_state(
            move |context, probe: &mut Probe| {
                window.update_layout_mode(context);
                window.sync_commands(context);
                window.show_ribbon(context);
                window.show_status(context);
                window.show_regions(context);
                probe.drawer = window.shell.drawer;
                sink.set(window.sidebar.probe());
            },
            Probe::default(),
        );
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the shell tests need one");
    }
    harness.run();
    (harness, reported)
}

/// A primary click at `position`, as the pointer would report it.
fn click_at(harness: &mut egui_kittest::Harness<'_, Probe>, position: egui::Pos2) {
    harness
        .input_mut()
        .events
        .push(egui::Event::PointerMoved(position));
    for pressed in [true, false] {
        harness.input_mut().events.push(egui::Event::PointerButton {
            pos: position,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::default(),
        });
    }
}

#[test]
fn the_status_bar_follows_the_language() {
    fn status_harness(window: &mut Window) -> egui_kittest::Harness<'_> {
        let mut harness = egui_kittest::Harness::new(move |context| window.show_status(context));
        harness.set_size(egui::vec2(1366.0, 768.0));
        if crate::theme::install_font(&harness.ctx, None).is_err() {
            eprintln!("no system font: the status test needs one");
        }
        harness.run();
        harness
    }
    let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
    {
        let harness = status_harness(&mut window);
        assert!(harness.query_all_by_label_contains("ms draw").count() >= 1);
        assert!(harness.query_all_by_label_contains("last:").count() >= 1);
        assert!(harness.query_all_by_label_contains("untitled").count() >= 1);
    }
    window.language = Language::Spanish;
    {
        let harness = status_harness(&mut window);
        assert!(harness.query_all_by_label_contains("ms de dibujo").count() >= 1);
        assert!(harness.query_all_by_label_contains("último:").count() >= 1);
        assert!(harness.query_all_by_label_contains("sin título").count() >= 1);
    }
}

#[test]
fn the_wide_mode_docks_both_panels_at_their_preferred_widths() {
    let mut window = window_with();
    let sheet;
    {
        let harness = shell_harness(&mut window, egui::vec2(1440.0, 900.0));
        sheet = harness.state().sheet_width;
        assert!(
            harness.query_by_label("Hide catalog panel").is_some(),
            "the catalog is docked"
        );
        assert!(
            harness.query_by_label("Hide properties panel").is_some(),
            "the dock is docked"
        );
    }
    assert_eq!(window.layout_mode(), LayoutMode::Wide);
    assert!(
        (sheet - (1440.0 - 240.0 - 360.0 - SHEET_INSET)).abs() < 1.0,
        "the sheet takes the rest: {sheet}"
    );
}

#[test]
fn the_medium_mode_docks_both_panels_at_their_minimums() {
    let mut window = window_with();
    let sheet;
    {
        let harness = shell_harness(&mut window, egui::vec2(1020.0, 768.0));
        sheet = harness.state().sheet_width;
    }
    assert_eq!(window.layout_mode(), LayoutMode::Medium);
    assert!(
        (sheet - (1020.0 - 220.0 - 300.0 - SHEET_INSET)).abs() < 1.0,
        "the panels are at their minimums: {sheet}"
    );
}

#[test]
fn a_wide_header_font_never_widens_a_docked_panel() {
    // The font a machine has is not part of the layout contract: the catalog keeps 220 and
    // the sheet keeps its floor even when the panel header and a category name are far wider
    // than the column. The condition is forced here, instead of trusting this machine's
    // font, because the same panel drew wider than its column on the continuous integration
    // runner, whose font lays the sensor category out 7 points wider.
    let mut window = window_with();
    window.theme.tokens.text_label = 46.0;
    window.theme.tokens.text_small = 38.0;
    window.theme.tokens.text_body = 30.0;
    let sheet;
    {
        let harness = shell_harness(&mut window, egui::vec2(1020.0, 768.0));
        sheet = harness.state().sheet_width;
    }
    assert_eq!(window.layout_mode(), LayoutMode::Medium);
    assert!(
        (sheet - (1020.0 - 220.0 - 300.0 - SHEET_INSET)).abs() < 1.0,
        "a wider header font must not widen the panels: {sheet}"
    );
}

#[test]
fn the_compact_mode_folds_both_panels_to_their_rails_and_leaves_the_sheet_its_room() {
    let mut window = window_with();
    let sheet;
    {
        let harness = shell_harness(&mut window, egui::vec2(900.0, 768.0));
        sheet = harness.state().sheet_width;
        assert!(
            harness.query_by_label("Show catalog panel").is_some(),
            "the closed catalog leaves its rail control"
        );
        assert!(
            harness.query_by_label("Show properties panel").is_some(),
            "the closed dock leaves its rail control"
        );
        assert!(
            harness.query_by_label("Built-in catalog").is_none(),
            "no panel content is docked in the compact mode"
        );
    }
    assert_eq!(window.layout_mode(), LayoutMode::Compact);
    assert!(
        (sheet - (900.0 - 2.0 * 24.0 - SHEET_INSET)).abs() < 1.0,
        "two rails, the rest for the sheet: {sheet}"
    );
    assert!(sheet >= 480.0, "the sheet keeps its floor");
}

#[test]
fn ctrl_b_opens_the_catalog_as_a_drawer_over_the_sheet() {
    let mut window = window_with();
    let (closed_sheet, open_sheet);
    {
        let mut harness = shell_harness(&mut window, egui::vec2(900.0, 768.0));
        closed_sheet = harness.state().sheet_width;
        harness.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::B);
        harness.run();
        open_sheet = harness.state().sheet_width;
        assert_eq!(harness.state().drawer, Some(Drawer::Catalog));
        assert!(
            harness.query_by_label("Built-in catalog").is_some(),
            "the drawer holds the palette"
        );
        assert!(
            harness.query_by_label("Hide catalog panel").is_some(),
            "the drawer has its hide control"
        );
    }
    assert_eq!(window.shell.drawer, Some(Drawer::Catalog));
    assert!(
        (open_sheet - closed_sheet).abs() < 1.0,
        "the drawer floats over the sheet instead of pushing it: {open_sheet} vs {closed_sheet}"
    );
    assert!(
        open_sheet >= 480.0,
        "the sheet keeps its floor under the drawer"
    );
}

#[test]
fn escape_closes_the_drawer() {
    let mut window = window_with();
    {
        let mut harness = shell_harness(&mut window, egui::vec2(900.0, 768.0));
        harness.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::J);
        harness.run();
        assert_eq!(harness.state().drawer, Some(Drawer::Inspector));
        harness.key_press(egui::Key::Escape);
        harness.run();
        assert_eq!(harness.state().drawer, None, "Esc dismisses the drawer");
        assert!(
            harness.query_by_label("Properties").is_none(),
            "the dock's content is gone with the drawer"
        );
    }
}

#[test]
fn a_click_outside_the_drawer_closes_it() {
    let mut window = window_with();
    {
        let mut harness = shell_harness(&mut window, egui::vec2(900.0, 768.0));
        harness.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::B);
        harness.run();
        assert_eq!(harness.state().drawer, Some(Drawer::Catalog));
        // Well to the right of the 240-point catalog drawer.
        click_at(&mut harness, egui::pos2(700.0, 600.0));
        harness.run();
        assert_eq!(harness.state().drawer, None, "a click outside dismisses it");
    }
}

#[test]
fn ctrl_b_toggles_the_drawer_closed() {
    let mut window = window_with();
    {
        let mut harness = shell_harness(&mut window, egui::vec2(900.0, 768.0));
        harness.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::B);
        harness.run();
        assert_eq!(harness.state().drawer, Some(Drawer::Catalog));
        harness.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::B);
        harness.run();
        assert_eq!(harness.state().drawer, None, "Ctrl+B closes what it opened");
    }
}

#[test]
fn the_drawer_reports_the_pointer_and_the_keyboard_inside_it() {
    let mut window = window_with();
    let (mut harness, probe) = shell_probe_harness(&mut window, egui::vec2(900.0, 768.0));
    harness.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::B);
    harness.run();
    assert_eq!(harness.state().drawer, Some(Drawer::Catalog));
    move_pointer(&mut harness, egui::pos2(100.0, 300.0));
    assert!(
        probe.get().pointer_inside,
        "the pointer inside the drawer is reported"
    );
    move_pointer(&mut harness, egui::pos2(700.0, 400.0));
    assert!(
        !probe.get().pointer_inside,
        "the sheet beside the drawer is outside it"
    );
    harness
        .get_by_role(egui::accesskit::Role::TextInput)
        .focus();
    harness.run();
    assert!(
        probe.get().focus_inside,
        "the search field has the keyboard, so the drawer's close is suspended"
    );
}

#[test]
fn reset_view_closes_an_open_drawer() {
    let mut window = window_with();
    {
        let mut harness = shell_harness(&mut window, egui::vec2(900.0, 768.0));
        harness.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::B);
        harness.run();
        assert_eq!(harness.state().drawer, Some(Drawer::Catalog));
    }
    window.dispatch(Command::ResetView);
    assert_eq!(
        window.shell.drawer, None,
        "Restablecer vista dismisses the drawer with the panels and the view"
    );
}

#[test]
fn clicking_a_rail_opens_the_drawer() {
    let mut window = window_with();
    {
        let mut harness = shell_harness(&mut window, egui::vec2(900.0, 768.0));
        harness.get_by_label("Show catalog panel").click();
        harness.run();
        assert_eq!(
            harness.state().drawer,
            Some(Drawer::Catalog),
            "the rail opens the drawer in the compact mode"
        );
    }
}

#[test]
fn the_drawer_hide_control_closes_both_the_drawer_and_the_panel() {
    let mut window = window_with();
    {
        let mut harness = shell_harness(&mut window, egui::vec2(900.0, 768.0));
        harness.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::B);
        harness.run();
        harness.get_by_label("Hide catalog panel").click();
        harness.run();
        assert_eq!(harness.state().drawer, None, "the hide control closes it");
    }
    assert!(!window.sidebar.is_open(), "the panel is closed as well");
}

#[test]
fn the_first_frame_sizes_and_centres_the_window_from_the_monitor_it_finds() {
    let mut window = window_with();
    window.start_open = false;
    let mut harness = frame_harness(&mut window, egui::vec2(1920.0, 1080.0));
    harness.step();
    let sent = commands(&harness);
    assert!(
        sent.contains(&egui::ViewportCommand::InnerSize(egui::vec2(1440.0, 900.0))),
        "the preferred size lands on a screen with room: {sent:?}"
    );
    assert!(
        sent.contains(&egui::ViewportCommand::OuterPosition(egui::pos2(
            240.0, 50.0
        ))),
        "the window is centred above the taskbar: {sent:?}"
    );
    assert!(
        sent.contains(&egui::ViewportCommand::MinInnerSize(egui::vec2(
            900.0, 600.0
        ))),
        "the design floor goes to the viewport: {sent:?}"
    );
    drop(harness);
    assert!(window.shell.applied, "the first geometry was applied once");
}

#[test]
fn a_1366_laptop_opens_maximized_with_a_normal_size_that_fits() {
    let mut window = window_with();
    window.start_open = false;
    let mut harness = frame_harness(&mut window, egui::vec2(1366.0, 768.0));
    harness.step();
    let sent = commands(&harness);
    assert!(
        sent.contains(&egui::ViewportCommand::Maximized(true)),
        "the preferred size does not fit and the window opens maximized: {sent:?}"
    );
    assert!(
        sent.contains(&egui::ViewportCommand::InnerSize(egui::vec2(1334.0, 656.0))),
        "the normal size that unmaximizing returns fits the screen: {sent:?}"
    );
    assert!(
        sent.contains(&egui::ViewportCommand::MinInnerSize(egui::vec2(
            900.0, 600.0
        ))),
        "the design floor still fits this screen: {sent:?}"
    );
    drop(harness);
    assert!(window.shell.applied);
}

#[test]
fn a_1366_laptop_at_125_percent_opens_maximized_with_the_room_as_the_floor() {
    let mut window = window_with();
    window.start_open = false;
    let mut harness = frame_harness(&mut window, egui::vec2(1093.0, 614.0));
    harness.step();
    let sent = commands(&harness);
    assert!(
        sent.contains(&egui::ViewportCommand::Maximized(true)),
        "{sent:?}"
    );
    assert!(
        sent.contains(&egui::ViewportCommand::MinInnerSize(egui::vec2(
            900.0, 502.0
        ))),
        "the floor yields to the room the screen has: {sent:?}"
    );
    assert!(
        sent.contains(&egui::ViewportCommand::InnerSize(egui::vec2(1061.0, 502.0))),
        "the normal size is the room, never the 900 × 600 floor: {sent:?}"
    );
    drop(harness);
    assert!(window.shell.applied);
}

#[test]
fn the_first_geometry_is_applied_once_and_not_repeated() {
    let mut window = window_with();
    window.start_open = false;
    let mut harness = frame_harness(&mut window, egui::vec2(1920.0, 1080.0));
    harness.step();
    harness.step();
    let sent = commands(&harness);
    assert!(
        !sent.iter().any(|command| matches!(
            command,
            egui::ViewportCommand::InnerSize(_)
                | egui::ViewportCommand::OuterPosition(_)
                | egui::ViewportCommand::MinInnerSize(_)
                | egui::ViewportCommand::Maximized(_)
        )),
        "the second frame does not fight the window: {sent:?}"
    );
}

#[test]
fn a_window_left_on_a_disconnected_monitor_returns_to_the_opening_one() {
    let dir = scratch();
    let file = dir.join(window_state::FILE_NAME);
    let saved = Geometry {
        size: egui::vec2(1200.0, 700.0),
        position: Some(egui::pos2(16.0, 16.0)),
        maximized: false,
    };
    assert!(window_state::write(&file, &saved));
    let mut window = window_with().with_window_state(window_state::Store::at(Some(file.clone())));
    window.start_open = false;
    let mut harness = frame_harness(&mut window, egui::vec2(1366.0, 768.0));
    harness.step();
    // The window was moved to a 1920 extension monitor.
    place_viewport(
        &mut harness,
        egui::vec2(1920.0, 1080.0),
        egui::pos2(1366.0, 100.0),
        egui::vec2(1200.0, 700.0),
    );
    harness.step();
    // The extension is unplugged; the window still reports its old place.
    place_viewport(
        &mut harness,
        egui::vec2(1366.0, 768.0),
        egui::pos2(1366.0, 100.0),
        egui::vec2(1200.0, 700.0),
    );
    harness.step();
    let sent = commands(&harness);
    assert!(
        sent.contains(&egui::ViewportCommand::InnerSize(egui::vec2(1200.0, 656.0))),
        "the window is clamped to the room it came back to: {sent:?}"
    );
    assert!(
        sent.contains(&egui::ViewportCommand::OuterPosition(egui::pos2(
            83.0, 16.0
        ))),
        "the window is brought back into the screen it opened on: {sent:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_first_frame_restores_the_remembered_geometry() {
    let dir = scratch();
    let file = dir.join(window_state::FILE_NAME);
    let saved = Geometry {
        size: egui::vec2(1200.0, 800.0),
        position: Some(egui::pos2(100.0, 50.0)),
        maximized: false,
    };
    assert!(window_state::write(&file, &saved));
    let mut window = window_with().with_window_state(window_state::Store::at(Some(file.clone())));
    window.start_open = false;
    let mut harness = frame_harness(&mut window, egui::vec2(1920.0, 1080.0));
    harness.step();
    let sent = commands(&harness);
    assert!(
        sent.contains(&egui::ViewportCommand::InnerSize(egui::vec2(1200.0, 800.0))),
        "the remembered size comes back: {sent:?}"
    );
    assert!(
        sent.contains(&egui::ViewportCommand::OuterPosition(egui::pos2(
            100.0, 50.0
        ))),
        "the remembered place comes back: {sent:?}"
    );
    assert!(
        !sent
            .iter()
            .any(|command| matches!(command, egui::ViewportCommand::Maximized(_))),
        "a normal window does not ask to be maximized"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_frame_stays_inside_the_canvas_at_every_mode_and_size() {
    for size in [
        egui::vec2(700.0, 900.0),
        egui::vec2(900.0, 600.0),
        egui::vec2(1200.0, 800.0),
        egui::vec2(1920.0, 1080.0),
    ] {
        let mut window = window_with();
        let harness = sheet_harness(&mut window, size);
        let probe = harness.state().sheet.expect("a sheet frame");
        let eps = 0.5;
        assert!(
            probe.frame.min.x >= probe.canvas.min.x - eps
                && probe.frame.max.x <= probe.canvas.max.x + eps
                && probe.frame.min.y >= probe.canvas.min.y - eps
                && probe.frame.max.y <= probe.canvas.max.y + eps,
            "{size:?}: frame {:?} leaves the canvas {:?}",
            probe.frame,
            probe.canvas
        );
        assert!(
            probe.frame.max.x <= probe.drawing.max.x + eps,
            "{size:?}: frame {:?} enters the strip's gutter",
            probe.frame
        );
        assert!(
            (probe.clip.min.x - probe.canvas.min.x).abs() <= eps
                && (probe.clip.min.y - probe.canvas.min.y).abs() <= eps
                && (probe.clip.max.x - probe.canvas.max.x).abs() <= eps
                && (probe.clip.max.y - probe.canvas.max.y).abs() <= eps,
            "{size:?}: the drawing clips against the canvas, not the window"
        );
    }
}

#[test]
fn the_frame_leaves_the_strip_a_gutter_on_the_canvas() {
    let mut window = window_with();
    let unit = window.theme.tokens.unit;
    let gutter = crate::layout::view_strip_gutter(unit);
    let harness = sheet_harness(&mut window, egui::vec2(900.0, 700.0));
    let probe = harness.state().sheet.expect("a sheet frame");
    assert!(
        (probe.canvas.right() - probe.drawing.right() - gutter).abs() <= 0.5,
        "the drawing area does not start the gutter where the strip will float"
    );
    assert!(
        probe.canvas.right() - probe.frame.max.x >= gutter - 0.5,
        "the frame enters the strip's gutter: {:?}",
        probe.frame
    );
}

#[test]
fn the_frame_keeps_its_margins_inside_the_page() {
    let mut window = window_with();
    let (page_width, _) = window.page.size_mm();
    let margins = window.page.margins;
    let harness = sheet_harness(&mut window, egui::vec2(1200.0, 800.0));
    let probe = harness.state().sheet.expect("a sheet frame");
    let scale = probe.paper.width() / page_width;
    let frame = egui::Rect::from_min_max(
        probe.paper.min + egui::vec2(margins.left * scale, margins.top * scale),
        probe.paper.max - egui::vec2(margins.right * scale, margins.bottom * scale),
    );
    assert!(
        frame.min.x >= probe.paper.min.x
            && frame.min.y >= probe.paper.min.y
            && frame.max.x <= probe.paper.max.x
            && frame.max.y <= probe.paper.max.y,
        "the frame {frame:?} leaves the paper {:?}",
        probe.paper
    );
    assert!(
        (probe.frame.center() - frame.center()).length() <= 0.5,
        "the probe's frame {:?} is not the page's {:?}",
        probe.frame,
        frame
    );
}

#[test]
fn a_zoom_above_the_fit_clips_against_the_canvas_not_the_window() {
    let mut window = window_with();
    window.view.zoom_in();
    window.view.zoom_in();
    let harness = sheet_harness(&mut window, egui::vec2(900.0, 700.0));
    let probe = harness.state().sheet.expect("a sheet frame");
    assert!(
        (probe.clip.min.x - probe.canvas.min.x).abs() <= 0.5
            && (probe.clip.min.y - probe.canvas.min.y).abs() <= 0.5
            && (probe.clip.max.x - probe.canvas.max.x).abs() <= 0.5
            && (probe.clip.max.y - probe.canvas.max.y).abs() <= 0.5,
        "the zoomed drawing must clip against the canvas, not the window"
    );
    assert!(
        probe.paper.width() > probe.drawing.width() + 1.0
            || probe.paper.height() > probe.drawing.height() + 1.0,
        "the zoomed paper should be larger than the drawing area for the clip to matter"
    );
}
