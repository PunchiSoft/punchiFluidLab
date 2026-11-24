// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The panel settings' tests: the file's round trip and its defensive reads, the width clamp of
//! the mode the window opened in, the restore on start, the settle write, the reset and the
//! auto-hide flag — and the promise that none of it touches the document or its history.

use std::path::PathBuf;
use std::time::Instant;

use eframe::egui;
use egui_kittest::kittest::Queryable;
use punchi_fluid_doc::{Document, Placement};
use punchi_fluid_draw::Pt;

use super::{
    FILE_NAME, MAX_BYTES, MIN_WIDTH, PanelState, SETTLE, Settings, Store, parse, read, write,
};
use crate::Window;
use crate::layout::{CATALOG_MIN_WIDTH, DOCK_MIN_WIDTH, LayoutMode, SHEET_MIN_WIDTH};
use crate::theme::Theme;

/// A folder of its own for the test, removed at the end where possible.
fn scratch() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "punchi-fluid-settings-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the scratch folder");
    dir
}

/// A window the tests can drive: no path, no dialog, no safety copy.
fn window_with(document: Document) -> Window {
    Window::new(Theme::fallback(), document, None, None)
}

/// A document with one valve, so the document promises have something to check.
fn valve_document() -> Document {
    let mut document = Document::new();
    document.place(Placement::new(
        "valve.3_2.spring_return",
        Pt::new(100.0, 80.0),
        24.0,
        24.0,
    ));
    document
}

/// Whether two panel widths are the same, to the float's own precision.
fn same(left: f32, right: f32) -> bool {
    (left - right).abs() <= f32::EPSILON
}

#[test]
fn the_appearance_round_trips_and_an_unknown_value_reads_the_desktop() {
    let parsed = parse("version=1\nappearance=slate\n").expect("the version is there");
    assert_eq!(
        parsed.appearance,
        crate::theme::Appearance::Theme(crate::theme::ThemeId::Slate)
    );
    let parsed = parse("version=1\nappearance=cad\n").expect("the version is there");
    assert_eq!(
        parsed.appearance,
        crate::theme::Appearance::Theme(crate::theme::ThemeId::Cad)
    );
    let parsed = parse("version=1\nappearance=nordic\n").expect("the version is there");
    assert_eq!(
        parsed.appearance,
        crate::theme::Appearance::Theme(crate::theme::ThemeId::Nordic)
    );
    let parsed = parse("version=1\nappearance=indigo\n").expect("the version is there");
    assert_eq!(
        parsed.appearance,
        crate::theme::Appearance::Theme(crate::theme::ThemeId::Indigo)
    );
    let parsed = parse("version=1\nappearance=graphite\n").expect("the version is there");
    assert_eq!(
        parsed.appearance,
        crate::theme::Appearance::Theme(crate::theme::ThemeId::Graphite)
    );
    let parsed = parse("version=1\nappearance=future\n").expect("the version is there");
    assert_eq!(parsed.appearance, crate::theme::Appearance::Desktop);

    let parsed = parse("version=1\nappearance=system\ntheme_day=slate\ntheme_night=cad\n")
        .expect("the version is there");
    assert_eq!(parsed.appearance, crate::theme::Appearance::System);
    assert_eq!(parsed.theme_day, crate::theme::ThemeId::Slate);
    assert_eq!(parsed.theme_night, crate::theme::ThemeId::Cad);
}

#[test]
fn an_unknown_slot_keeps_the_default_theme() {
    let parsed = parse("version=1\ntheme_day=future\ntheme_night=nope\n").expect("the version");
    let defaults = Settings::default();
    assert_eq!(parsed.theme_day, defaults.theme_day);
    assert_eq!(parsed.theme_night, defaults.theme_night);
}

#[test]
fn choosing_an_appearance_wears_it_and_remembers_it() {
    let mut window = window_with(valve_document());
    let context = egui::Context::default();
    window.apply_appearance(
        &context,
        crate::theme::Appearance::Theme(crate::theme::ThemeId::Slate),
    );
    assert_eq!(
        window.theme.appearance,
        crate::theme::Appearance::Theme(crate::theme::ThemeId::Slate)
    );
    assert_eq!(
        window.panel_settings().appearance,
        crate::theme::Appearance::Theme(crate::theme::ThemeId::Slate)
    );
}

fn settings(catalog: (bool, f32), inspector: (bool, f32), autohide: bool) -> Settings {
    Settings {
        catalog: PanelState {
            open: catalog.0,
            width: catalog.1,
        },
        inspector: PanelState {
            open: inspector.0,
            width: inspector.1,
        },
        autohide,
        appearance: crate::theme::Appearance::Desktop,
        theme_day: crate::theme::ThemeId::Cad,
        theme_night: crate::theme::ThemeId::Slate,
        language: punchi_fluid_i18n::Language::English,
    }
}

/// The catalog drawn headless, as the panel tests do, so the rail can be clicked.
fn catalog_harness(window: &mut Window) -> egui_kittest::Harness<'_> {
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1366.0, 768.0))
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(240)
        .build(move |context| window.show_catalog(context));
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the settings tests need one");
    }
    harness.run();
    harness
}

/// The two panels drawn headless over a window of `size`, with the layout mode the frame would
/// derive: the restore clamp runs through `update_layout_mode`.
fn panel_harness(window: &mut Window, size: egui::Vec2) -> egui_kittest::Harness<'_> {
    let mut harness = egui_kittest::Harness::builder()
        .with_size(size)
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(240)
        .build(move |context| {
            window.update_layout_mode(context);
            window.show_catalog(context);
            window.show_inspector(context);
        });
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the settings tests need one");
    }
    harness.run();
    harness
}

#[test]
fn the_settings_file_round_trips() {
    let dir = scratch();
    let file = dir.join(FILE_NAME);
    let saved = settings((false, 300.5), (true, 420.25), true);
    assert!(write(&file, &saved));
    assert_eq!(read(&file), Some(saved));
    assert!(
        !crate::state_file::temporary_path(&file).exists(),
        "the write leaves no temporary file behind"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_language_round_trips_and_defaults_to_english() {
    assert_eq!(
        Settings::default().language,
        punchi_fluid_i18n::Language::English
    );
    let mut saved = settings((true, 300.0), (true, 420.0), false);
    saved.language = punchi_fluid_i18n::Language::Spanish;
    let dir = scratch();
    let file = dir.join(FILE_NAME);
    assert!(write(&file, &saved));
    assert_eq!(read(&file), Some(saved));
    assert_eq!(
        parse("version=1\nlanguage=es\n")
            .expect("the version and language")
            .language,
        punchi_fluid_i18n::Language::Spanish
    );
    assert_eq!(
        parse("version=1\nlanguage=fr\n")
            .expect("the version")
            .language,
        punchi_fluid_i18n::Language::English
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_file_without_panels_reads_the_defaults() {
    assert_eq!(parse("version=1\n"), Some(Settings::default()));
    assert_eq!(
        parse("# nothing yet\nversion=1\n"),
        Some(Settings::default())
    );
    assert_eq!(parse(""), None, "no version, no settings");
}

#[test]
fn a_truncated_value_drops_only_that_value() {
    let parsed = parse("version=1\ncatalog.open=0\ncatalog.wid").expect("the version is there");
    assert!(!parsed.catalog.open, "the whole flag was read");
    assert!(
        same(parsed.catalog.width, Settings::default().catalog.width),
        "a truncated width keeps its default"
    );
    // A file written by a handler that stopped mid-line is not a file from another format:
    // the version is enough, the broken number is ignored.
    let parsed = parse("version=1\ncatalog.width=\n").expect("the version is there");
    assert!(same(
        parsed.catalog.width,
        Settings::default().catalog.width
    ));
}

#[test]
fn an_out_of_range_width_is_ignored() {
    let defaults = Settings::default();
    for value in ["999999999", "-4", "0", "NaN", "24.0"] {
        let text = format!("version=1\ncatalog.width={value}\n");
        let parsed = parse(&text).expect("the version is there");
        if value == "24.0" {
            assert!(
                same(parsed.catalog.width, MIN_WIDTH),
                "the rail width is the smallest accepted value"
            );
        } else {
            assert!(
                same(parsed.catalog.width, defaults.catalog.width),
                "{value} cannot be a panel width"
            );
        }
    }
}

#[test]
fn unknown_keys_are_ignored_and_known_ones_are_read() {
    let parsed = parse(
        "# a comment\nversion=1\nfuture=yes\ncatalog.open=0\nribbon.minimized=1\ninspector.width=320\n",
    )
    .expect("the known keys are enough");
    assert!(!parsed.catalog.open);
    assert!(same(parsed.inspector.width, 320.0));
    assert!(same(
        parsed.catalog.width,
        Settings::default().catalog.width
    ));
}

#[test]
fn a_file_from_a_newer_version_is_refused() {
    assert_eq!(parse("version=2\ncatalog.open=0\n"), None);
    assert_eq!(parse("version=01\ncatalog.open=0\n"), None);
}

#[test]
fn the_line_bound_ignores_what_lies_beyond_it() {
    let mut text = String::from("version=1\n");
    for _ in 0..40 {
        text.push_str("# filler\n");
    }
    text.push_str("catalog.open=0\n");
    assert_eq!(
        parse(&text),
        Some(Settings::default()),
        "a key past the line bound is not read"
    );
}

#[test]
fn a_file_over_the_size_bound_is_ignored() {
    let text = format!("version=1\n#{}", "x".repeat(MAX_BYTES));
    let dir = scratch();
    let file = dir.join(FILE_NAME);
    std::fs::write(&file, text).expect("the huge file");
    assert_eq!(read(&file), None);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_remembered_width_is_clamped_to_the_mode_maximum() {
    let saved = settings((true, 400.0), (true, 520.0), true);
    let clamped = saved.clamped_to(LayoutMode::Medium, 1020.0);
    assert!(same(
        clamped.catalog.width,
        1020.0 - SHEET_MIN_WIDTH - DOCK_MIN_WIDTH
    ));
    assert!(same(
        clamped.inspector.width,
        1020.0 - SHEET_MIN_WIDTH - CATALOG_MIN_WIDTH
    ));
    assert!(clamped.catalog.open && clamped.inspector.open);
    assert!(clamped.autohide, "the flags travel unchanged");
    // A width that fits the wide mode is kept as the reader left it.
    let clamped = saved.clamped_to(LayoutMode::Wide, 1920.0);
    assert!(same(clamped.catalog.width, 400.0));
    assert!(same(clamped.inspector.width, 520.0));
}

#[test]
fn the_store_writes_once_the_settings_settle() {
    let dir = scratch();
    let file = dir.join(FILE_NAME);
    let mut store = Store::at(Some(file.clone()));
    assert!(!store.found());
    let wanted = settings((false, 280.0), (true, 360.0), true);
    let start = Instant::now();
    assert!(
        !store.observe(wanted, start),
        "the first note is not a write"
    );
    assert!(!store.observe(wanted, start + SETTLE / 2), "still moving");
    assert!(!file.exists(), "a moving panel does not write the file");
    assert!(
        store.observe(wanted, start + SETTLE),
        "the settled settings are written"
    );
    assert_eq!(read(&file), Some(wanted));
    let reread = Store::at(Some(file.clone()));
    assert!(reread.found(), "the file was found at start");
    assert_eq!(reread.remembered(), &wanted);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_store_writes_at_once_when_the_window_asks() {
    let dir = scratch();
    let file = dir.join(FILE_NAME);
    let mut store = Store::at(Some(file.clone()));
    let wanted = settings((true, 240.0), (false, 360.0), false);
    assert!(store.remember(wanted), "the close path writes immediately");
    assert_eq!(read(&file), Some(wanted));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_store_without_a_path_keeps_the_settings_in_memory() {
    let mut store = Store::at(None);
    let wanted = settings((false, 300.0), (true, 360.0), true);
    assert!(!store.remember(wanted), "nothing is written without a path");
    assert_eq!(
        store.remembered(),
        &wanted,
        "the memory keeps the change even though the file cannot"
    );
    assert!(!store.is_pending(), "nothing keeps waiting for a file");
}

#[test]
fn the_fold_and_the_width_survive_a_save_and_a_reload() {
    let dir = scratch();
    let file = dir.join(FILE_NAME);
    let mut window = window_with(Document::new()).with_settings(Store::at(Some(file.clone())));
    window.sidebar.resize_by(-20.0);
    window.inspector.close();
    let width = window.sidebar.width();
    let start = Instant::now();
    assert!(!window.observe_panel_settings(start));
    assert!(
        window.observe_panel_settings(start + SETTLE),
        "the settled fold and width are written"
    );
    drop(window);

    let reloaded = window_with(Document::new()).with_settings(Store::at(Some(file.clone())));
    assert!(!reloaded.inspector.is_open(), "the dock was folded");
    assert!(
        (reloaded.sidebar.width() - width).abs() <= f32::EPSILON,
        "the catalog came back at its own width"
    );
    assert_eq!(reloaded.autohide_on, Settings::default().autohide);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_panels_open_where_the_file_left_them_and_a_closed_one_starts_as_a_rail() {
    let dir = scratch();
    let file = dir.join(FILE_NAME);
    let saved = settings((false, 300.0), (true, 400.0), false);
    assert!(write(&file, &saved));
    let mut window = window_with(Document::new()).with_settings(Store::at(Some(file.clone())));
    assert!(!window.sidebar.is_open(), "the closed catalog stays folded");
    assert!(window.inspector.is_open(), "the open dock opens");
    assert!((window.sidebar.width() - 300.0).abs() <= f32::EPSILON);
    {
        let harness = panel_harness(&mut window, egui::vec2(1440.0, 900.0));
        assert!(
            harness.query_by_label("Show catalog panel").is_some(),
            "a closed panel starts as its rail"
        );
        assert!(
            harness.query_by_label("Built-in catalog").is_none(),
            "the closed panel has no content"
        );
        assert!(
            harness
                .query_by_label("Ocultar el panel de propiedades")
                .is_some(),
            "the open dock keeps its header"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_width_above_the_mode_maximum_is_clamped_when_the_window_opens() {
    let dir = scratch();
    let file = dir.join(FILE_NAME);
    let saved = settings((true, 400.0), (true, 520.0), false);
    assert!(write(&file, &saved));
    let mut window = window_with(Document::new()).with_settings(Store::at(Some(file.clone())));
    {
        let _harness = panel_harness(&mut window, egui::vec2(1020.0, 768.0));
    }
    let max = LayoutMode::Medium.max_widths(1020.0);
    assert!(
        (window.sidebar.width() - max.catalog).abs() <= f32::EPSILON,
        "the catalog is clamped to the medium mode: {}",
        window.sidebar.width()
    );
    assert!(
        (window.inspector.width() - max.dock).abs() <= f32::EPSILON,
        "the dock is clamped to the medium mode: {}",
        window.inspector.width()
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn reset_view_opens_the_panels_at_their_defaults_and_persists_them() {
    let dir = scratch();
    let file = dir.join(FILE_NAME);
    let saved = settings((false, 300.0), (false, 420.0), true);
    assert!(write(&file, &saved));
    let mut window = window_with(Document::new()).with_settings(Store::at(Some(file.clone())));
    assert!(window.autohide_on, "the flag came from the file");
    window.dispatch(crate::ribbon::Command::ResetView);
    assert!(window.sidebar.is_open(), "the reset opens the catalog");
    assert!(window.inspector.is_open(), "the reset opens the dock");
    assert!((window.sidebar.width() - crate::sidebar::DEFAULT_WIDTH).abs() <= f32::EPSILON);
    assert!((window.inspector.width() - crate::inspector::DEFAULT_WIDTH).abs() <= f32::EPSILON);
    assert!(!window.autohide_on, "the reset turns auto-hide off");
    assert_eq!(
        read(&file),
        Some(Settings::default()),
        "the defaults are persisted at once"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_autohide_command_flips_the_flag_and_remembers_it() {
    let dir = scratch();
    let file = dir.join(FILE_NAME);
    let mut window = window_with(Document::new()).with_settings(Store::at(Some(file.clone())));
    assert!(!window.autohide_on, "auto-hide is off by default");
    window.dispatch(crate::ribbon::Command::Autohide);
    assert!(window.autohide_on);
    assert_eq!(
        read(&file).map(|settings| settings.autohide),
        Some(true),
        "the flag is remembered"
    );
    window.dispatch(crate::ribbon::Command::Autohide);
    assert!(!window.autohide_on);
    assert_eq!(read(&file).map(|settings| settings.autohide), Some(false));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_click_on_the_rail_still_opens_the_panel_with_auto_hide_off() {
    let mut window = window_with(Document::new());
    assert!(!window.autohide_on);
    window.sidebar.close();
    {
        let mut harness = catalog_harness(&mut window);
        harness.get_by_label("Show catalog panel").click();
        harness.run();
    }
    assert!(
        window.sidebar.is_open(),
        "with auto-hide off the rail only answers a click"
    );
}

#[test]
fn remembering_the_panels_is_not_a_document_change() {
    let dir = scratch();
    let file = dir.join(FILE_NAME);
    assert!(write(
        &file,
        &settings((false, 300.0), (true, 400.0), false)
    ));
    let mut window = window_with(valve_document()).with_settings(Store::at(Some(file.clone())));
    let revision = window.document.revision();
    let undo = window.document.can_undo();
    let redo = window.document.can_redo();

    window.sidebar.resize_by(30.0);
    window.inspector.close();
    window.inspector.open();
    window.dispatch(crate::ribbon::Command::Catalog);
    window.dispatch(crate::ribbon::Command::Autohide);
    window.dispatch(crate::ribbon::Command::ResetView);
    // The auto-hide timers close and open the panels, which is still view state.
    window.start_open = false;
    window.autohide_on = true;
    window.sidebar.close();
    window.sidebar.probe_for_test(true, false, false);
    let start = Instant::now();
    let _ = window.apply_auto_hide(false, start + crate::auto_hide::DWELL);
    window.sidebar.probe_for_test(false, false, false);
    let _ = window.apply_auto_hide(
        false,
        start + crate::auto_hide::DWELL + crate::auto_hide::LINGER,
    );
    let _ = window.observe_panel_settings(start + SETTLE);
    window.flush_panel_settings();

    assert_eq!(
        window.document.revision(),
        revision,
        "the revision is untouched"
    );
    assert_eq!(window.document.can_undo(), undo, "no history entry appears");
    assert_eq!(window.document.can_redo(), redo);
    let _ = std::fs::remove_dir_all(&dir);
}
