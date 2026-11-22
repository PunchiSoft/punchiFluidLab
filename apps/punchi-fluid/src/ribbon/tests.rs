// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The ribbon tests: tabs reach their commands, the transport follows its state, a gallery
//! opens and a group collapses to its title.

use eframe::egui;
use egui_kittest::kittest::Queryable;

use super::{Palette, Ribbon};

#[test]
fn the_save_button_waits_for_a_path() {
    let mut ribbon = Ribbon::new();
    assert!(!ribbon.enabled("Guardar"), "no path, no save");
    ribbon.savable = true;
    assert!(ribbon.enabled("Guardar"), "a path enables the save");
    assert!(
        !ribbon.enabled("Guardar como"),
        "the dialog is not here yet"
    );
    assert!(!ribbon.enabled("Abrir"), "the open dialog is not here yet");
}
use crate::theme::Theme;

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

#[test]
fn the_tabs_reach_their_commands() {
    let mut ribbon = Ribbon::new();
    let palette = palette();
    let mut harness = harness(&mut ribbon, &palette);
    harness.get_by_label("Simular").click();
    harness.run();
    assert!(harness.query_by_label("Iniciar").is_some());
    harness.get_by_label("Archivo").click();
    harness.run();
    assert!(harness.query_by_label("Guardar").is_some());
}

fn click(ribbon: &mut Ribbon, label: &str) {
    let palette = palette();
    let mut harness = harness(ribbon, &palette);
    harness.get_by_label(label).click();
    harness.run();
}

#[test]
fn the_transport_follows_the_state() {
    let mut ribbon = Ribbon::new();
    ribbon.activate("Simular");
    click(&mut ribbon, "Pausa");
    assert_eq!(ribbon.last, "—", "Pausa is disabled before Iniciar");
    click(&mut ribbon, "Iniciar");
    assert_eq!(ribbon.last, "Iniciar");
    click(&mut ribbon, "Detener");
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

fn palette() -> Palette {
    Palette::from_theme(&Theme::fallback().tokens)
}
