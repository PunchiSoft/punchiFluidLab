// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Archivo controls: the split Save and the Recents list that tells two files apart.

use eframe::egui;
use egui_kittest::kittest::Queryable;
use punchi_fluid_file::recent::Recent;

use super::{harness, harness_at, palette};
use crate::ribbon::{Command, Ribbon};

#[test]
fn save_button_and_arrow_dispatch_distinct_actions() {
    let mut ribbon = Ribbon::new();
    let current = palette();
    {
        let mut harness = harness(&mut ribbon, &current, &[]);
        harness.get_by_label("Save").click();
        harness.run();
    }
    assert_eq!(ribbon.take(), Some(Command::ProjectSave));
    {
        let mut harness = harness(&mut ribbon, &current, &[]);
        harness.get_by_label("Save as").click();
        harness.run();
    }
    assert_eq!(ribbon.take(), Some(Command::ProjectSaveAs));
    {
        let mut harness = harness(&mut ribbon, &current, &[]);
        harness.get_by_label("Save as").focus();
        harness.run();
        harness.key_press(egui::Key::Enter);
        harness.run();
    }
    assert_eq!(
        ribbon.take(),
        Some(Command::ProjectSaveAs),
        "the arrow is its own keyboard stop"
    );
}

#[test]
fn recents_distinguish_same_name_paths() {
    let root = std::env::temp_dir().join(format!("punchi-ribbon-homonyms-{}", std::process::id()));
    let first = root.join("alpha").join("same.pfluid");
    let second = root.join("beta").join("same.pfluid");
    for path in [&first, &second] {
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("a folder");
        std::fs::write(path, b"").expect("a temporary document");
    }
    let recents = vec![
        Recent::new(first.to_string_lossy()),
        Recent::new(second.to_string_lossy()),
    ];
    let mut ribbon = Ribbon::new();
    {
        let current = palette();
        let mut harness = harness(&mut ribbon, &current, &recents);
        harness.get_by_label("Recent   ").click();
        harness.run();
        assert!(
            harness.query_by_label("same.pfluid — alpha").is_some(),
            "the first homonym names its folder"
        );
        assert!(
            harness.query_by_label("same.pfluid — beta").is_some(),
            "the second homonym names its folder"
        );
        harness.get_by_label("same.pfluid — beta").click();
        harness.run();
    }
    assert_eq!(ribbon.take(), Some(Command::OpenRecent(1)));
    {
        let current = palette();
        let mut harness = harness(&mut ribbon, &current, &recents);
        harness.get_by_label("Recent   ").click();
        harness.run();
        harness.get_by_label("same.pfluid — alpha").click();
        harness.run();
    }
    assert_eq!(ribbon.take(), Some(Command::OpenRecent(0)));
    std::fs::remove_file(&second).ok();
    {
        let current = palette();
        let mut harness = harness(&mut ribbon, &current, &recents);
        harness.get_by_label("Recent   ").click();
        harness.run();
        assert!(
            harness
                .query_by_label("same.pfluid — beta (not found)")
                .is_some(),
            "a missing homonym keeps the folder and says it is gone"
        );
        harness
            .get_by_label("same.pfluid — beta (not found)")
            .click();
        harness.run();
    }
    assert_eq!(ribbon.take(), None, "a missing file does not dispatch");
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn collapsed_project_keeps_save_as_and_export_options() {
    let mut ribbon = Ribbon::new();
    let current = palette();
    {
        let mut harness = harness_at(&mut ribbon, &current, &[], 600.0);
        harness.get_by_label("Project   ").click();
        harness.run();
        assert!(
            harness.query_by_label("Save as").is_some(),
            "the flyout keeps Save as"
        );
        assert!(
            harness.query_by_label("Options…").is_some(),
            "the flyout keeps the export options"
        );
        harness.get_by_label("Save as").click();
        harness.run();
    }
    assert_eq!(ribbon.take(), Some(Command::ProjectSaveAs));
}
