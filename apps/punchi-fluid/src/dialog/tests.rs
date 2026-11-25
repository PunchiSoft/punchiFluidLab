// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The footer's tests: the reading order and its right edge, the default's accent and
//! focus, `Enter`, and a disabled default that takes neither.

use std::cell::Cell;
use std::rc::Rc;

use eframe::egui;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

use super::{Action, actions};
use crate::theme::Theme;

/// A harness with one footer on it; the reader's choice lands in the shared cell.
fn footer(spec: Vec<Action<'static>>) -> (Harness<'static>, Rc<Cell<Option<usize>>>) {
    let tokens = Theme::fallback().tokens;
    let chosen = Rc::new(Cell::new(None));
    let sink = Rc::clone(&chosen);
    let harness = Harness::new(move |context| {
        egui::CentralPanel::default().show(context, |ui| {
            if let Some(index) = actions(ui, &tokens, &spec) {
                sink.set(Some(index));
            }
        });
    });
    (harness, chosen)
}

#[test]
fn the_last_action_sits_at_the_right_edge() {
    let (mut harness, _chosen) = footer(vec![
        Action::new("Save"),
        Action::new("Discard"),
        Action::default("Cancel"),
    ]);
    harness.run();
    let save = harness.get_by_label("Save").rect();
    let discard = harness.get_by_label("Discard").rect();
    let cancel = harness.get_by_label("Cancel").rect();
    assert!(
        save.right() < discard.right() && discard.right() < cancel.right(),
        "the actions read left to right and end with the safe one"
    );
    assert!(
        (save.center().y - cancel.center().y).abs() < 1.0,
        "one row, not a stack"
    );
}

#[test]
fn the_default_takes_the_focus() {
    let (mut harness, _chosen) = footer(vec![Action::new("Discard"), Action::default("Cancel")]);
    harness.run();
    assert!(
        harness.get_by_label("Cancel").is_focused(),
        "the default is where the keyboard lands"
    );
}

#[test]
fn enter_runs_the_default() {
    let (mut harness, chosen) = footer(vec![Action::new("Discard"), Action::default("Cancel")]);
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(
        chosen.get(),
        Some(1),
        "Enter answers with the focused default"
    );
}

#[test]
fn a_disabled_default_takes_neither_the_focus_nor_enter() {
    let (mut harness, chosen) = footer(vec![Action::default("Apply").enabled(false)]);
    harness.run();
    assert!(!harness.get_by_label("Apply").is_focused());
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(chosen.get(), None, "a disabled action cannot run");
}
