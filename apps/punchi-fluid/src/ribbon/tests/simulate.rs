// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The simulation tab: Verify uses the same Check command as Editar, and the compact
//! transport follows the running state.

use egui_kittest::kittest::Queryable;

use super::{harness_at, palette};
use crate::ribbon::{Command, Ribbon};

#[test]
fn verify_actions_open_the_same_check_panel() {
    let mut ribbon = Ribbon::new();
    let current = palette();
    ribbon.activate("ribbon.tab.simulate");
    {
        let mut harness = harness_at(&mut ribbon, &current, &[], 1366.0);
        harness.get_by_label("Verify circuit").click();
        harness.run();
    }
    assert_eq!(ribbon.take(), Some(Command::Check));
    ribbon.activate("ribbon.tab.edit");
    {
        let mut harness = harness_at(&mut ribbon, &current, &[], 1366.0);
        if harness.query_by_label("Check").is_none() {
            harness.get_by_label("Review   ").click();
            harness.run();
        }
        harness.get_by_label("Check").click();
        harness.run();
    }
    assert_eq!(ribbon.take(), Some(Command::Check));
}

#[test]
fn compact_transport_matches_simulation_state() {
    let mut ribbon = Ribbon::new();
    let current = palette();
    {
        let mut harness = harness_at(&mut ribbon, &current, &[], 1366.0);
        harness.get_by_label("Start run").click();
        harness.run();
    }
    assert_eq!(ribbon.take(), Some(Command::Start));
    assert!(ribbon.running, "the compact Start enters the run");
    {
        let mut harness = harness_at(&mut ribbon, &current, &[], 1366.0);
        harness.get_by_label("Pause run").click();
        harness.run();
    }
    assert_eq!(ribbon.take(), Some(Command::Pause));
    assert!(!ribbon.running && ribbon.simulation, "Pause keeps the mode");
}

#[test]
fn verify_remains_reachable_at_a_large_font() {
    let mut palette = palette();
    palette.text_label *= 1.5;
    palette.text_body *= 1.5;
    palette.text_small *= 1.5;
    let mut ribbon = Ribbon::new();
    ribbon.activate("ribbon.tab.simulate");
    let harness = harness_at(&mut ribbon, &palette, &[], 900.0);
    assert!(
        harness.query_all_by_label_contains("Verification").count() >= 1,
        "the Verification group stays named at 900 with a large font"
    );
}
