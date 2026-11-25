// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The right dock's rows: the tube series, the port list and the three bodies the panel can
//! show. Split from `inspector.rs` so both files keep the project's 800-line budget.

use eframe::egui;
use punchi_fluid_catalog::{Port, PortRole};
use punchi_fluid_doc::Document;
use punchi_fluid_draw::LineClass;
use punchi_fluid_i18n::Language;
use punchi_fluid_session::Model;

use super::Action;
use crate::theme::Tokens;

/// The metric entries of the tube series the inspector offers, in hundredths of a millimetre.
///
/// The window may not depend on `punchi-fluid-core` ([ADR 0035]), so the series the document
/// validates in `crates/punchi-fluid-core/src/tube.rs` is repeated here as interface copy.
/// `Document::set_pipe_tube` remains the one that refuses a value outside its bounds, so the
/// copy can never widen what the document accepts.
///
/// [ADR 0035]: ../../../docs/adr/0035-the-window-is-rust.md
const METRIC_TUBES: [u16; 13] = [
    400, 500, 600, 800, 1000, 1200, 1400, 1600, 2000, 2500, 3200, 4000, 5000,
];

/// The inch entries and the name the interface prints; the value is the exact conversion.
const INCH_TUBES: [(u16, &str); 10] = [
    (318, "1/8\""),
    (397, "5/32\""),
    (476, "3/16\""),
    (635, "1/4\""),
    (794, "5/16\""),
    (953, "3/8\""),
    (1270, "1/2\""),
    (1588, "5/8\""),
    (1905, "3/4\""),
    (2540, "1\""),
];

/// A section label, in the panel's small muted type.
pub(super) fn section(ui: &mut egui::Ui, tokens: &Tokens, title: &str) {
    ui.add_space(tokens.unit * 0.5);
    ui.label(
        egui::RichText::new(title)
            .size(tokens.text_small)
            .strong()
            .color(tokens.ink_muted),
    );
}

/// The line-class row of a hose: the three classes the ribbon offers, never the electrical
/// one, which a wire takes from its two terminals.
fn class_row(
    ui: &mut egui::Ui,
    index: usize,
    current: LineClass,
    language: Language,
) -> Option<Action> {
    let mut action = None;
    egui::ComboBox::from_label(language.text("inspector.line_class"))
        .selected_text(class_label(language, current))
        .show_ui(ui, |ui| {
            for class in [LineClass::Working, LineClass::Pilot, LineClass::Drain] {
                if ui
                    .selectable_label(current == class, class_label(language, class))
                    .clicked()
                {
                    action = Some(Action::PipeClass { index, class });
                }
            }
        });
    action
}

/// The tube-size row of a hose: the nominal series plus **Sin especificar**.
pub(super) fn tube_row(
    ui: &mut egui::Ui,
    index: usize,
    current: Option<u16>,
    language: Language,
) -> Option<Action> {
    let mut action = None;
    egui::ComboBox::from_label(language.text("inspector.tube"))
        .selected_text(tube_label(language, current))
        .show_ui(ui, |ui| {
            if ui
                .selectable_label(
                    current.is_none(),
                    language.text("inspector.tube_unspecified"),
                )
                .clicked()
            {
                action = Some(Action::PipeTube { index, tube: None });
            }
            for hundredths in METRIC_TUBES {
                if ui
                    .selectable_label(
                        current == Some(hundredths),
                        tube_label(language, Some(hundredths)),
                    )
                    .clicked()
                {
                    action = Some(Action::PipeTube {
                        index,
                        tube: Some(hundredths),
                    });
                }
            }
            for (hundredths, label) in INCH_TUBES {
                if ui
                    .selectable_label(current == Some(hundredths), label)
                    .clicked()
                {
                    action = Some(Action::PipeTube {
                        index,
                        tube: Some(hundredths),
                    });
                }
            }
        });
    action
}

/// The component's ports with their roles, and the model's reading when one is loaded.
pub(super) fn ports(
    ui: &mut egui::Ui,
    tokens: &Tokens,
    model: Option<&Model>,
    index: usize,
    definitions: &[Port],
    language: Language,
) {
    if definitions.is_empty() {
        return;
    }
    section(ui, tokens, language.text("inspector.ports"));
    for port in definitions {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(port.numeral())
                    .size(tokens.text_body)
                    .color(tokens.ink),
            );
            ui.label(
                egui::RichText::new(role_label(language, port.role))
                    .size(tokens.text_small)
                    .color(tokens.ink_muted),
            );
            if let Some(reading) = reading_of(model, index, port, language) {
                ui.label(
                    egui::RichText::new(reading)
                        .size(tokens.text_small)
                        .color(tokens.ink_dim),
                );
            }
        });
    }
}

/// What the model says about one port, in the unit the status bar uses: gauge bar.
fn reading_of(
    model: Option<&Model>,
    index: usize,
    port: &Port,
    language: Language,
) -> Option<String> {
    if matches!(port.role, PortRole::Mechanical | PortRole::Electrical) {
        return None;
    }
    let model = model?;
    let network_part = model.part_of(index)?;
    let pressure = model.network().port_pressure(network_part, &port.id)?;
    let gauge = (pressure - model.network().atmosphere_pa()) / 1.0e5;
    let reading = format!("{gauge:.2}");
    let reading = match language {
        Language::English => reading,
        Language::Spanish => reading.replace('.', ","),
    };
    Some(format!("{reading} bar"))
}

/// A hose, with no selection, says what the reader can do next.
pub(super) fn empty_body(
    ui: &mut egui::Ui,
    tokens: &Tokens,
    document: &Document,
    language: Language,
) {
    ui.add_space(tokens.unit * 0.5);
    ui.label(
        egui::RichText::new(language.text("inspector.empty_title"))
            .size(tokens.text_body)
            .strong()
            .color(tokens.ink),
    );
    ui.label(
        egui::RichText::new(language.text("inspector.empty_body"))
            .size(tokens.text_body)
            .color(tokens.ink_dim),
    );
    ui.add_space(tokens.unit * 0.8);
    let active = document.active_circuit();
    if let Some(circuit) = document
        .circuits()
        .iter()
        .find(|circuit| circuit.id == active)
    {
        ui.label(
            egui::RichText::new(&circuit.title)
                .size(tokens.text_label)
                .color(tokens.ink),
        );
    }
    let components = document
        .placements()
        .iter()
        .filter(|placement| placement.circuit == active)
        .count();
    let hoses = document
        .connections()
        .iter()
        .filter(|connection| connection.circuit == active)
        .count();
    ui.label(
        egui::RichText::new(language.format(
            "inspector.counts",
            &[
                ("components", &components.to_string()),
                ("hoses", &hoses.to_string()),
            ],
        ))
        .size(tokens.text_small)
        .color(tokens.ink_muted),
    );
}

/// More than one component: the panel edits one instance at a time.
pub(super) fn multi_body(ui: &mut egui::Ui, tokens: &Tokens, count: usize, language: Language) {
    ui.add_space(tokens.unit * 0.5);
    ui.label(
        egui::RichText::new(
            language.format("inspector.multi_title", &[("count", &count.to_string())]),
        )
        .size(tokens.text_body)
        .strong()
        .color(tokens.ink),
    );
    ui.label(
        egui::RichText::new(language.text("inspector.multi_body"))
            .size(tokens.text_body)
            .color(tokens.ink_dim),
    );
}

/// The name of a port role in the current language.
fn role_label(language: Language, role: PortRole) -> &'static str {
    let key = match role {
        PortRole::Supply => "inspector.port.supply",
        PortRole::Working => "inspector.port.working",
        PortRole::Exhaust => "inspector.port.exhaust",
        PortRole::Mechanical => "inspector.port.mechanical",
        PortRole::Electrical => "inspector.port.electrical",
    };
    language.text(key)
}

/// The name of a line class in the current language.
fn class_label(language: Language, class: LineClass) -> &'static str {
    let key = match class {
        LineClass::Working => "inspector.class.working",
        LineClass::Pilot => "inspector.class.pilot",
        LineClass::Drain => "inspector.class.drain",
        LineClass::Electrical => "inspector.class.electrical",
        // The menu never offers it (a hose is not an assembly boundary), but the label
        // keeps the match total for a value that came from somewhere else.
        LineClass::Boundary => "inspector.class.boundary",
    };
    language.text(key)
}

/// The label of a stored tube value: a named entry keeps its name, any other accepted value
/// reads as a measurement with the interface's decimal comma.
fn tube_label(language: Language, tube: Option<u16>) -> String {
    let Some(tube) = tube else {
        return language.text("inspector.tube_unspecified").to_owned();
    };
    if let Some((_, label)) = INCH_TUBES.iter().find(|(value, _)| *value == tube) {
        return (*label).to_owned();
    }
    let whole = tube / 100;
    let fraction = tube % 100;
    match (language, fraction) {
        (_, 0) => format!("{whole} mm"),
        (Language::English, fraction) => format!("{whole}.{fraction:02} mm"),
        (Language::Spanish, fraction) => format!("{whole},{fraction:02} mm"),
    }
}

/// One hose: its line class and its tube size, each one undo.
pub(super) fn pipe_body(
    ui: &mut egui::Ui,
    tokens: &Tokens,
    document: &Document,
    index: usize,
    language: Language,
) -> Option<Action> {
    let connection = document.connections().get(index)?;
    let mut action = None;
    ui.add_space(tokens.unit * 0.35);
    ui.label(
        egui::RichText::new(language.text("inspector.pipe"))
            .size(tokens.text_body)
            .strong()
            .color(tokens.ink),
    );
    ui.add_space(tokens.unit * 0.35);
    if let Some(chosen) = class_row(ui, index, connection.class(), language) {
        action = Some(chosen);
    }
    ui.add_space(tokens.unit * 0.35);
    if let Some(chosen) = tube_row(ui, index, connection.tube_od_hundredths(), language) {
        action = Some(chosen);
    }
    action
}

#[cfg(test)]
mod tests {
    use super::tube_label;
    use punchi_fluid_i18n::Language;

    #[test]
    fn the_tube_label_uses_the_language_decimal_separator() {
        assert_eq!(tube_label(Language::English, Some(450)), "4.50 mm");
        assert_eq!(tube_label(Language::Spanish, Some(450)), "4,50 mm");
        assert_eq!(tube_label(Language::English, None), "Not specified");
        assert_eq!(tube_label(Language::Spanish, None), "Sin especificar");
    }
}
