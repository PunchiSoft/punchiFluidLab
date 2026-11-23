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

use crate::Window;
use crate::layout::LayoutMode;
use crate::ribbon::Command;
use crate::shell::Drawer;
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

/// The central panel's default frame: eight points of inner margin on each side, so the
/// sheet's available width is the room the panels leave minus this inset.
const SHEET_INSET: f32 = 16.0;

/// What one frame says about the shell, read from the harness after it runs.
#[derive(Debug, Default)]
struct Probe {
    sheet_width: f32,
    drawer: Option<Drawer>,
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
fn the_wide_mode_docks_both_panels_at_their_preferred_widths() {
    let mut window = window_with();
    let sheet;
    {
        let harness = shell_harness(&mut window, egui::vec2(1440.0, 900.0));
        sheet = harness.state().sheet_width;
        assert!(
            harness
                .query_by_label("Ocultar el panel del catálogo")
                .is_some(),
            "the catalog is docked"
        );
        assert!(
            harness
                .query_by_label("Ocultar el panel de propiedades")
                .is_some(),
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
fn the_compact_mode_folds_both_panels_to_their_rails_and_leaves_the_sheet_its_room() {
    let mut window = window_with();
    let sheet;
    {
        let harness = shell_harness(&mut window, egui::vec2(900.0, 768.0));
        sheet = harness.state().sheet_width;
        assert!(
            harness
                .query_by_label("Mostrar el panel del catálogo")
                .is_some(),
            "the closed catalog leaves its rail control"
        );
        assert!(
            harness
                .query_by_label("Mostrar el panel de propiedades")
                .is_some(),
            "the closed dock leaves its rail control"
        );
        assert!(
            harness.query_by_label("Catálogo integrado").is_none(),
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
            harness.query_by_label("Catálogo integrado").is_some(),
            "the drawer holds the palette"
        );
        assert!(
            harness
                .query_by_label("Ocultar el panel del catálogo")
                .is_some(),
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
            harness.query_by_label("Propiedades").is_none(),
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
        harness
            .get_by_label("Mostrar el panel del catálogo")
            .click();
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
        harness
            .get_by_label("Ocultar el panel del catálogo")
            .click();
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
