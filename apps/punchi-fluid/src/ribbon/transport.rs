// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The compact transport in the tab row: the mode word and the two frequent commands, with
//! the same command table, enabled rules and shortcuts as the full Simular group.

use eframe::egui;
use punchi_fluid_i18n::Language;

use super::{Command, Palette, Ribbon, hint};

/// The word for the state the window is in: editing, running or paused.
pub(crate) fn mode_key(running: bool, simulation: bool) -> &'static str {
    if running {
        "ribbon.transport_state.running"
    } else if simulation {
        "ribbon.transport_state.paused"
    } else {
        "ribbon.transport_state.editing"
    }
}

/// The mode word, when the tab row can afford it: the compact transport outranks it, so at a
/// 150 % font and 900 points the word yields and the status bar still names the mode.
pub(super) fn mode_label(
    ui: &mut egui::Ui,
    palette: &Palette,
    language: Language,
    running: bool,
    simulation: bool,
) {
    if ui.available_width() <= 500.0 {
        return;
    }
    ui.label(
        egui::RichText::new(language.text(mode_key(running, simulation)))
            .size(palette.text_body)
            .color(palette.muted),
    );
}

/// The Start/Pause toggle and Stop. The caller owns the picked command budget, so these are
/// the same commands the Simular tab, the shortcuts and the view strip dispatch.
pub(super) fn buttons(
    ui: &mut egui::Ui,
    palette: &Palette,
    language: Language,
    ribbon: &Ribbon,
    picked: &mut Option<Command>,
) {
    let (label, command, enabled) = if ribbon.running {
        (
            "ribbon.item.pause",
            Command::Pause,
            ribbon.enabled(Command::Pause),
        )
    } else {
        (
            "ribbon.item.start",
            Command::Start,
            ribbon.enabled(Command::Start),
        )
    };
    let text = language.text(label);
    let response = ui.add_enabled(
        enabled,
        egui::Button::new(egui::RichText::new(text).size(palette.text_body)),
    );
    let response = response.on_hover_text(hint(text, Some(command)));
    let accessible = if command == Command::Pause {
        "ribbon.item.pause_run"
    } else {
        "ribbon.item.start_run"
    };
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, language.text(accessible))
    });
    if response.clicked() {
        *picked = Some(command);
    }
    let advance = language.text("ribbon.item.step");
    let response = ui.add_enabled(
        ribbon.enabled(Command::Step),
        egui::Button::new(egui::RichText::new(advance).size(palette.text_body)),
    );
    let response = response.on_hover_text(hint(advance, Some(Command::Step)));
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            ribbon.enabled(Command::Step),
            language.text("ribbon.item.step_run"),
        )
    });
    if response.clicked() {
        *picked = Some(Command::Step);
    }
    let halt = language.text("ribbon.item.stop");
    let response = ui.add_enabled(
        ribbon.enabled(Command::Stop),
        egui::Button::new(egui::RichText::new(halt).size(palette.text_body)),
    );
    let response = response.on_hover_text(hint(halt, Some(Command::Stop)));
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            ribbon.enabled(Command::Stop),
            language.text("ribbon.item.stop_run"),
        )
    });
    if response.clicked() {
        *picked = Some(Command::Stop);
    }
}
