// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The ribbon tests: the table carries a command for every control the window handles, the
//! dispatch follows the state, a disabled command does not dispatch, a gallery opens and a
//! group collapses to its title.

use eframe::egui;
use egui_kittest::kittest::Queryable;

use super::model::{Item, TABS};
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
                    Item::Large(_, _, None) | Item::Small(_, _, None) => {}
                }
            }
        }
    }
    commands
}

#[test]
fn every_command_is_on_the_ribbon() {
    let commands = table_commands();
    for command in Command::ALL {
        assert!(
            commands.iter().any(|(carried, ..)| *carried == command),
            "{command:?} has no control"
        );
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
    let project = find("Archivo", "Proyecto", "Nuevo");
    let circuit = find("Archivo", "Circuito", "Nuevo circuito");
    assert_eq!(project, Command::ProjectNew);
    assert_eq!(circuit, Command::CircuitNew);
    assert_ne!(project, circuit, "the two new commands are different");
}

#[test]
fn a_disabled_command_does_not_dispatch() {
    let mut ribbon = Ribbon::new();
    ribbon.run_command(Command::Pause);
    assert_eq!(ribbon.take(), None, "Pausa is disabled before Iniciar");
    ribbon.run_command(Command::Undo);
    assert_eq!(ribbon.take(), None, "nothing to undo");
}

#[test]
fn pressing_every_control_yields_its_command() {
    for command in Command::ALL {
        let mut ribbon = Ribbon::new();
        ribbon.selected = 2;
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
    ribbon.activate("Simular");
    click(&mut ribbon, "Pausa");
    assert_eq!(ribbon.take(), None, "Pausa is disabled before Iniciar");
    assert_eq!(ribbon.last, "—");
    click(&mut ribbon, "Iniciar");
    assert_eq!(ribbon.take(), Some(Command::Start));
    assert_eq!(ribbon.last, "Iniciar");
    click(&mut ribbon, "Detener");
    assert_eq!(ribbon.take(), Some(Command::Stop));
    assert_eq!(ribbon.last, "Detener");
}

#[test]
fn the_margins_gallery_opens_its_cards() {
    let mut ribbon = Ribbon::new();
    let palette = palette();
    let mut harness = harness(&mut ribbon, &palette);
    harness.get_by_label("Márgenes   ").click();
    harness.run();
    assert!(
        harness.query_by_label("Estrecho").is_some(),
        "the gallery shows its cards"
    );
}

#[test]
fn the_row_keeps_a_group_title_when_it_collapses() {
    let mut ribbon = Ribbon::new();
    ribbon.activate("Editar");
    let palette = palette();
    let harness = harness(&mut ribbon, &palette);
    assert!(
        harness.query_by_label("Copiar").is_some(),
        "Componente stays open"
    );
    assert!(
        harness.query_by_label("Alinear   ").is_some(),
        "Alinear collapses to its title button"
    );
    assert!(
        harness.query_by_label("Izquierda").is_none(),
        "the collapsed group does not draw its commands in the row"
    );
}

fn harness<'a>(ribbon: &'a mut Ribbon, palette: &'a Palette) -> egui_kittest::Harness<'a> {
    let mut harness = egui_kittest::Harness::new_ui(move |ui| ribbon.show(ui, palette));
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
    let mut harness = harness(ribbon, &palette);
    harness.get_by_label(label).click();
    harness.run();
}

fn palette() -> Palette {
    Palette::from_theme(&crate::theme::Theme::fallback().tokens)
}
