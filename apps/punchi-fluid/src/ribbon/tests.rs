// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The ribbon tests: the table carries a command for every control the window handles, the
//! dispatch follows the state, a disabled command does not dispatch, a gallery opens and a
//! group collapses to its title.

use eframe::egui;
use egui_kittest::kittest::Queryable;
use punchi_fluid_file::recent::Recent;
use punchi_fluid_i18n::Language;

use super::hint;
use super::model::{Item, TABS};
use super::transport::mode_key;
use super::{Command, Palette, Ribbon};

#[test]
fn the_file_commands_are_live() {
    let ribbon = Ribbon::new();
    for command in [
        Command::ProjectNew,
        Command::ProjectOpen,
        Command::ProjectSave,
        Command::ProjectSaveAs,
        Command::ProjectExport,
    ] {
        assert!(ribbon.enabled(command), "{command:?} is live");
    }
}

#[test]
fn the_edit_buttons_follow_the_selection_and_the_history() {
    let mut ribbon = Ribbon::new();
    assert!(!ribbon.enabled(Command::Delete), "no selection, no delete");
    ribbon.selected = 1;
    assert!(
        ribbon.enabled(Command::Delete),
        "a selection enables the edit commands"
    );
    assert!(ribbon.enabled(Command::Rotate));
    assert!(ribbon.enabled(Command::Mirror));
    assert!(ribbon.enabled(Command::Duplicate));
    assert!(
        !ribbon.enabled(Command::AlignLeft),
        "one placement cannot be aligned"
    );
    ribbon.selected = 2;
    assert!(ribbon.enabled(Command::AlignLeft));
    assert!(
        ribbon.enabled(Command::Copy),
        "a selection enables the copy"
    );
    assert!(
        !ribbon.enabled(Command::Paste),
        "an empty clipboard cannot paste"
    );
    ribbon.has_clipboard = true;
    assert!(ribbon.enabled(Command::Paste));
    assert!(!ribbon.enabled(Command::Undo), "nothing to undo");
    ribbon.can_undo = true;
    assert!(ribbon.enabled(Command::Undo));
    assert!(!ribbon.enabled(Command::Redo), "nothing to redo");
    ribbon.can_redo = true;
    assert!(ribbon.enabled(Command::Redo));
}

/// Every command the table carries, with the tab and group it lives in.
fn table_commands() -> Vec<(Command, &'static str, &'static str, &'static str)> {
    let mut commands = Vec::new();
    for tab in TABS {
        for group in tab.groups {
            for item in group.items {
                match item {
                    Item::Large(_, label, Some(command)) | Item::Small(_, label, Some(command)) => {
                        commands.push((*command, tab.name, group.name, *label));
                    }
                    Item::Menu(_, _, entries) => {
                        for entry in *entries {
                            if let Some(command) = entry.command {
                                commands.push((command, tab.name, group.name, entry.label));
                            }
                        }
                    }
                    Item::Gallery(_, _, entries) => {
                        for entry in *entries {
                            if let Some(command) = entry.command {
                                commands.push((command, tab.name, group.name, entry.label));
                            }
                        }
                    }
                    Item::Large(_, _, None) | Item::Small(_, _, None) | Item::Recents(..) => {}
                }
            }
        }
    }
    commands
}

#[test]
fn every_command_is_on_the_ribbon() {
    let commands = table_commands();
    let has_recents = TABS.iter().any(|tab| {
        tab.groups.iter().any(|group| {
            group
                .items
                .iter()
                .any(|item| matches!(item, Item::Recents(..)))
        })
    });
    let has_launcher = TABS
        .iter()
        .any(|tab| tab.groups.iter().any(|group| group.launcher));
    for command in Command::ALL {
        let covered = if matches!(command, Command::OpenRecent(_)) {
            // The rows are the session's list at paint time; the table carries the menu.
            has_recents
        } else if matches!(command, Command::PageSetup) {
            // The page setup panel is the group's launcher, not a table item.
            has_launcher
        } else {
            commands.iter().any(|(carried, ..)| {
                std::mem::discriminant(carried) == std::mem::discriminant(&command)
            })
        };
        assert!(covered, "{command:?} has no control");
    }
    for (carried, ..) in &commands {
        assert!(
            Command::ALL
                .iter()
                .any(|command| std::mem::discriminant(command) == std::mem::discriminant(carried)),
            "{carried:?} has no dispatch arm"
        );
    }
}

#[test]
#[expect(
    clippy::match_same_arms,
    reason = "the menu and the gallery carry different entry types; the assertion is the same"
)]
fn every_ribbon_label_is_in_the_catalog() {
    let english = Language::English;
    for command in Command::ALL {
        assert_ne!(
            english.text(command.label_key()),
            command.label_key(),
            "{command:?}"
        );
    }
    for key in [
        "ribbon.about",
        "ribbon.license",
        "ribbon.page_setup",
        "ribbon.recents_empty",
        "ribbon.recents_missing",
    ] {
        assert_ne!(english.text(key), key, "{key}");
    }
    for tab in TABS {
        assert_ne!(english.text(tab.name), tab.name, "{}", tab.name);
        for group in tab.groups {
            assert_ne!(english.text(group.name), group.name, "{}", group.name);
            for item in group.items {
                match item {
                    Item::Large(_, label, _) | Item::Small(_, label, _) => {
                        assert_ne!(english.text(label), *label, "{label}");
                    }
                    Item::Menu(_, label, entries) => {
                        assert_ne!(english.text(label), *label, "{label}");
                        for entry in *entries {
                            assert_ne!(english.text(entry.label), entry.label, "{}", entry.label);
                        }
                    }
                    Item::Gallery(_, label, entries) => {
                        assert_ne!(english.text(label), *label, "{label}");
                        for entry in *entries {
                            assert_ne!(english.text(entry.label), entry.label, "{}", entry.label);
                        }
                    }
                    Item::Recents(_, label) => {
                        assert_ne!(english.text(label), *label, "{label}");
                    }
                }
            }
        }
    }
}

#[test]
fn the_ribbon_follows_the_language() {
    let mut ribbon = Ribbon::new();
    ribbon.language = Language::Spanish;
    let palette = palette();
    let harness = harness(&mut ribbon, &palette, &[]);
    assert!(harness.query_by_label("Archivo").is_some());
    assert!(harness.query_by_label("File").is_none());
}

#[test]
fn the_command_label_follows_the_language() {
    let mut ribbon = Ribbon::new();
    ribbon.selected = 1;
    ribbon.language = Language::Spanish;
    ribbon.run_command(Command::Rotate);
    assert_eq!(ribbon.last, "Girar");
    ribbon.language = Language::English;
    ribbon.run_command(Command::Rotate);
    assert_eq!(ribbon.last, "Rotate");
}

#[test]
fn no_hint_without_a_handler() {
    fn check(label: &str, command: Option<Command>) {
        let text = hint(label, command);
        match command {
            Some(command) => {
                assert!(
                    Command::ALL
                        .iter()
                        .any(|known| std::mem::discriminant(known)
                            == std::mem::discriminant(&command)),
                    "{command:?} has a hint and no handler"
                );
                if crate::shortcuts::label(command).is_some() {
                    assert!(text.contains('('), "{label} has a shortcut and no hint");
                }
            }
            None => assert_eq!(text, label, "a control with no command announces no key"),
        }
    }
    for tab in TABS {
        for group in tab.groups {
            for item in group.items {
                match item {
                    Item::Large(_, label, command) | Item::Small(_, label, command) => {
                        check(label, *command);
                    }
                    Item::Menu(_, _, entries) => {
                        for entry in *entries {
                            check(entry.label, entry.command);
                        }
                    }
                    Item::Gallery(_, _, entries) => {
                        for entry in *entries {
                            check(entry.label, entry.command);
                        }
                    }
                    Item::Recents(_, label) => check(label, None),
                }
            }
        }
    }
}

#[test]
fn a_command_is_not_its_label() {
    let commands = table_commands();
    let find = |tab: &str, group: &str, label: &str| {
        commands
            .iter()
            .find(|(_, at_tab, at_group, at_label)| {
                *at_tab == tab && *at_group == group && *at_label == label
            })
            .map(|(command, ..)| *command)
            .expect("the control is in the table")
    };
    let project = find("ribbon.tab.file", "ribbon.group.project", "ribbon.item.new");
    let circuit = find(
        "ribbon.tab.file",
        "ribbon.group.circuit",
        "ribbon.item.circuit_new",
    );
    assert_eq!(project, Command::ProjectNew);
    assert_eq!(circuit, Command::CircuitNew);
    assert_ne!(project, circuit, "the two new commands are different");
}

#[test]
fn the_speed_marks_the_active_one() {
    let mut ribbon = Ribbon::new();
    ribbon.activate("ribbon.tab.simulate");
    click(&mut ribbon, "2×");
    assert_eq!(ribbon.take(), Some(Command::Speed(super::Speed::Double)));
    assert_eq!(ribbon.active_speed, "ribbon.item.speed_double");
}

#[test]
fn the_pipe_commands_need_a_selected_pipe() {
    let mut ribbon = Ribbon::new();
    assert!(!ribbon.enabled(Command::PipeClass(punchi_fluid_draw::LineClass::Working)));
    assert!(!ribbon.enabled(Command::Tube));
    ribbon.has_pipe = true;
    assert!(ribbon.enabled(Command::PipeClass(punchi_fluid_draw::LineClass::Working)));
    assert!(ribbon.enabled(Command::Tube));
}

#[test]
fn distributing_needs_two_placements() {
    let mut ribbon = Ribbon::new();
    assert!(!ribbon.enabled(Command::Distribute(punchi_fluid_doc::Axis::Horizontal)));
    ribbon.selected = 2;
    assert!(ribbon.enabled(Command::Distribute(punchi_fluid_doc::Axis::Horizontal)));
}

#[test]
fn the_circuit_close_needs_two() {
    let mut ribbon = Ribbon::new();
    assert!(!ribbon.enabled(Command::CircuitClose));
    ribbon.can_close_circuit = true;
    assert!(ribbon.enabled(Command::CircuitClose));
}

#[test]
fn the_block_reset_is_disabled_when_default() {
    let mut ribbon = Ribbon::new();
    assert!(
        !ribbon.enabled(Command::BlockReset),
        "a default block has nothing to reset"
    );
    ribbon.can_reset_block = true;
    assert!(
        ribbon.enabled(Command::BlockReset),
        "a custom block can reset"
    );
}

#[test]
fn a_disabled_command_does_not_dispatch() {
    let mut ribbon = Ribbon::new();
    ribbon.run_command(Command::Pause);
    assert_eq!(ribbon.take(), None, "Pause is disabled before Start");
    ribbon.run_command(Command::Undo);
    assert_eq!(ribbon.take(), None, "nothing to undo");
}

#[test]
fn pressing_every_control_yields_its_command() {
    for command in Command::ALL {
        let mut ribbon = Ribbon::new();
        ribbon.selected = 2;
        ribbon.has_pipe = true;
        ribbon.has_clipboard = true;
        ribbon.can_undo = true;
        ribbon.can_redo = true;
        ribbon.can_step = true;
        ribbon.savable = true;
        if matches!(command, Command::Pause | Command::Stop) {
            ribbon.running = true;
        }
        let expected = ribbon.enabled(command).then_some(command);
        ribbon.run_command(command);
        assert_eq!(ribbon.take(), expected, "{command:?}");
    }
}

#[test]
fn the_transport_follows_the_state() {
    let mut ribbon = Ribbon::new();
    ribbon.activate("ribbon.tab.simulate");
    click(&mut ribbon, "Pause");
    assert_eq!(ribbon.take(), None, "Pause is disabled before Start");
    assert_eq!(ribbon.last, "—");
    click(&mut ribbon, "Start");
    assert_eq!(ribbon.take(), Some(Command::Start));
    assert_eq!(ribbon.last, "Start");
    click(&mut ribbon, "Stop");
    assert_eq!(ribbon.take(), Some(Command::Stop));
    assert_eq!(ribbon.last, "Stop");
}

#[test]
fn the_tab_row_transport_starts_and_stops() {
    let mut ribbon = Ribbon::new();
    click(&mut ribbon, "Start run");
    assert_eq!(ribbon.take(), Some(Command::Start));
    assert!(ribbon.running);
    click(&mut ribbon, "Stop run");
    assert_eq!(ribbon.take(), Some(Command::Stop));
    assert!(!ribbon.running);
}

#[test]
fn the_collapsed_ribbon_keeps_the_transport_keyboard_reachable() {
    let mut ribbon = Ribbon::new();
    let palette = palette();
    {
        let mut harness = harness(&mut ribbon, &palette, &[]);
        harness.get_by_label("Start run").focus();
        harness.run();
        harness.key_press(egui::Key::Enter);
        harness.run();
    }
    assert!(ribbon.running, "Enter on the compact Start starts the run");
}

#[test]
fn stop_waits_for_a_run_or_a_pause() {
    let mut ribbon = Ribbon::new();
    assert!(!ribbon.enabled(Command::Stop), "nothing to stop");
    ribbon.running = true;
    assert!(ribbon.enabled(Command::Stop), "a run can be stopped");
    ribbon.running = false;
    ribbon.simulation = true;
    assert!(
        ribbon.enabled(Command::Stop),
        "a paused simulation can be stopped"
    );
}

#[test]
fn the_mode_word_follows_the_real_state() {
    assert_eq!(mode_key(false, false), "ribbon.transport_state.editing");
    assert_eq!(mode_key(true, true), "ribbon.transport_state.running");
    assert_eq!(mode_key(false, true), "ribbon.transport_state.paused");
}

/// Opens the margins gallery, or the collapsed group's flyout, which lists its cards.
fn open_margins(harness: &mut egui_kittest::Harness<'_>) {
    if harness.query_by_label("Margins   ").is_some() {
        harness.get_by_label("Margins   ").click();
    } else {
        harness.get_by_label("Page setup   ").click();
    }
    harness.run();
}

#[test]
fn the_margins_gallery_opens_its_cards() {
    let mut ribbon = Ribbon::new();
    let palette = palette();
    let mut harness = harness(&mut ribbon, &palette, &[]);
    open_margins(&mut harness);
    assert!(
        harness.query_by_label("Narrow").is_some(),
        "the gallery shows its cards"
    );
}

#[test]
fn the_recent_menu_lists_the_real_documents() {
    let path = std::env::temp_dir().join(format!(
        "punchi-ribbon-recent-{}.pfluid",
        std::process::id()
    ));
    std::fs::write(&path, b"").expect("a temporary document");
    let recents = vec![Recent::new(path.to_string_lossy())];
    let name = path
        .file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().into_owned());
    let mut ribbon = Ribbon::new();
    {
        let palette = palette();
        let mut harness = harness(&mut ribbon, &palette, &recents);
        harness.get_by_label("Recent   ").click();
        harness.run();
        assert!(
            harness.query_by_label(&name).is_some(),
            "the menu lists the remembered document"
        );
        harness.get_by_label(&name).click();
        harness.run();
    }
    assert_eq!(ribbon.take(), Some(Command::OpenRecent(0)));
    std::fs::remove_file(&path).ok();
}

#[test]
fn a_missing_recent_is_marked_and_refused() {
    let recents = vec![Recent::new("/punchi/no-existe.pfluid")];
    let mut ribbon = Ribbon::new();
    {
        let palette = palette();
        let mut harness = harness(&mut ribbon, &palette, &recents);
        harness.get_by_label("Recent   ").click();
        harness.run();
        assert!(
            harness
                .query_by_label("no-existe.pfluid (not found)")
                .is_some(),
            "the missing file is named"
        );
        harness.get_by_label("no-existe.pfluid (not found)").click();
        harness.run();
    }
    assert_eq!(ribbon.take(), None, "a missing file does not dispatch");
}

#[test]
fn the_page_launcher_dispatches_its_command() {
    let mut ribbon = Ribbon::new();
    ribbon.activate("ribbon.tab.file");
    let palette = palette();
    {
        let mut harness = harness(&mut ribbon, &palette, &[]);
        if harness.query_by_label("…").is_some() {
            harness.get_by_label("…").click();
        } else {
            // The row collapsed the group: the launcher is a row of its flyout.
            harness.get_by_label("Page setup   ").click();
            harness.run();
            harness.get_by_label("Page setup").click();
        }
        harness.run();
    }
    assert_eq!(ribbon.take(), Some(Command::PageSetup));
}

#[test]
fn the_margins_gallery_dispatches_its_command() {
    let mut ribbon = Ribbon::new();
    {
        let palette = palette();
        let mut harness = harness(&mut ribbon, &palette, &[]);
        open_margins(&mut harness);
        harness.get_by_label("Narrow").click();
        harness.run();
    }
    assert_eq!(
        ribbon.take(),
        Some(Command::PageMargin(super::MarginPreset::Narrow))
    );
}

#[test]
fn the_row_keeps_a_group_title_when_it_collapses() {
    let mut ribbon = Ribbon::new();
    ribbon.activate("ribbon.tab.edit");
    let palette = palette();
    let harness = harness(&mut ribbon, &palette, &[]);
    assert!(
        harness.query_by_label("Copy").is_some(),
        "Componente stays open"
    );
    assert!(
        harness.query_by_label("Align   ").is_some(),
        "Alinear collapses to its title button"
    );
    assert!(
        harness.query_by_label("Left").is_none(),
        "the collapsed group does not draw its commands in the row"
    );
}

fn harness<'a>(
    ribbon: &'a mut Ribbon,
    palette: &'a Palette,
    recents: &'a [Recent],
) -> egui_kittest::Harness<'a> {
    let mut harness = egui_kittest::Harness::new_ui(move |ui| ribbon.show(ui, palette, recents));
    harness.set_size(egui::vec2(1366.0, 768.0));
    // The window ships no font: the harness gets the desktop's, as the app does at start.
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the ribbon tests need one");
    }
    harness.run();
    harness
}

fn click(ribbon: &mut Ribbon, label: &str) {
    let palette = palette();
    let mut harness = harness(ribbon, &palette, &[]);
    harness.get_by_label(label).click();
    harness.run();
}

fn palette() -> Palette {
    Palette::from_theme(&crate::theme::Theme::fallback().tokens)
}
