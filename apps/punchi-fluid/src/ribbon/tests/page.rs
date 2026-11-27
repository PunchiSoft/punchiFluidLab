// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Page setup and Title block groups: the current value is marked, the named launcher
//! opens the page panel, and closing a menu changes nothing.

use eframe::egui;
use egui_kittest::kittest::Queryable;
use punchi_fluid_doc::paper::{Orientation, PaperSize};
use punchi_fluid_doc::title_block::ContentModel;

use super::{harness, open_margins, palette};
use crate::ribbon::{Command, MarginPreset, Ribbon};

#[test]
fn margins_menu_marks_actual_value_and_opens_page_setup() {
    let mut ribbon = Ribbon::new();
    ribbon.margins = MarginPreset::Normal.margins();
    assert!(
        ribbon.command_selected(Command::PageMargin(MarginPreset::Normal)),
        "the active preset is marked"
    );
    assert!(
        !ribbon.command_selected(Command::PageMargin(MarginPreset::Narrow)),
        "a different preset is not marked"
    );
    let current = palette();
    {
        let mut harness = harness(&mut ribbon, &current, &[]);
        open_margins(&mut harness);
        assert!(
            harness.query_by_label("Normal").is_some(),
            "the gallery shows the active preset"
        );
    }
    {
        let mut harness = harness(&mut ribbon, &current, &[]);
        if harness.query_by_label("Page setup…").is_none() {
            harness.get_by_label("Page setup   ").click();
            harness.run();
        }
        harness.get_by_label("Page setup…").click();
        harness.run();
    }
    assert_eq!(ribbon.take(), Some(Command::PageSetup));
}

#[test]
fn closing_page_menu_without_selection_keeps_sheet() {
    let mut ribbon = Ribbon::new();
    let current = palette();
    {
        let mut harness = harness(&mut ribbon, &current, &[]);
        open_margins(&mut harness);
        harness.key_press(egui::Key::Escape);
        harness.run();
        assert!(
            harness.query_by_label("Narrow").is_none(),
            "the gallery is closed"
        );
    }
    assert_eq!(
        ribbon.take(),
        None,
        "closing a page menu dispatches nothing"
    );
}

#[test]
fn title_block_shows_active_model_and_opens_its_editor() {
    let mut ribbon = Ribbon::new();
    ribbon.block_model = ContentModel::Technical;
    ribbon.block_visible = true;
    ribbon.activate("ribbon.tab.block");
    assert!(
        ribbon.command_selected(Command::BlockModel(ContentModel::Technical)),
        "the active model is marked"
    );
    let current = palette();
    {
        let mut harness = harness(&mut ribbon, &current, &[]);
        assert!(
            harness.query_all_by_label_contains("Technical").count() >= 1,
            "the Content control names the active model"
        );
        harness.get_by_label("Edit data…").click();
        harness.run();
    }
    assert_eq!(ribbon.take(), Some(Command::BlockEdit));
}

#[test]
fn paper_and_orientation_mark_the_current_value() {
    let mut ribbon = Ribbon::new();
    ribbon.paper = PaperSize::A3;
    ribbon.orientation = Orientation::Portrait;
    assert!(ribbon.command_selected(Command::PagePaper(PaperSize::A3)));
    assert!(!ribbon.command_selected(Command::PagePaper(PaperSize::A4)));
    assert!(ribbon.command_selected(Command::PageOrientation(Orientation::Portrait)));
    assert!(!ribbon.command_selected(Command::PageOrientation(Orientation::Landscape)));
}
