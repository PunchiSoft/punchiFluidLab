// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The status bar: mode, view, selection, reading and warning, in that order. The frame's
//! draw time is a diagnostic and stays out of the normal bar; the file name lives in the
//! window title. Every value comes from the same window state the ribbon and the sheet use.

use eframe::egui;
use punchi_fluid_i18n::Language;

use crate::Window;

impl Window {
    pub(crate) fn show_status_bar(&mut self, context: &egui::Context) {
        let language = self.language;
        let mode = self.status_mode(language);
        let view = self.status_view(language);
        let selection = self.status_selection(language);
        let reading = self.status_reading(language);
        let notice = self.notice.clone();
        let tokens = &self.theme.tokens;
        egui::TopBottomPanel::bottom("status").show(context, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(mode).color(tokens.ink).strong());
                ui.separator();
                ui.label(egui::RichText::new(view).color(tokens.ink_dim));
                ui.separator();
                ui.label(egui::RichText::new(selection).color(tokens.ink));
                ui.separator();
                ui.label(egui::RichText::new(reading).color(tokens.ink_dim));
                if let Some(notice) = notice {
                    ui.separator();
                    let label = ui.add(
                        egui::Label::new(egui::RichText::new(&notice).color(tokens.accent))
                            .truncate(),
                    );
                    label.on_hover_text(notice);
                }
            });
        });
    }

    fn status_view(&self, language: Language) -> String {
        let (width, height) = self.page.size_mm();
        let zoom = format!("{:.0}", self.view.zoom() * 100.0);
        let width = format!("{width:.0}");
        let height = format!("{height:.0}");
        language.format(
            "status.view",
            &[("width", &width), ("height", &height), ("zoom", &zoom)],
        )
    }

    fn status_mode(&self, language: Language) -> String {
        let mode = language.text(crate::ribbon::mode_key(self.running, self.simulation));
        let Some(telemetry) = self.model.as_ref().map(|model| model.network().telemetry()) else {
            return mode.to_owned();
        };
        let time = language.decimal(telemetry.time, 2);
        language.format("status.mode_time", &[("mode", mode), ("time", &time)])
    }

    fn status_reading(&self, language: Language) -> String {
        let Some(index) = self.selected.as_slice().first().copied() else {
            return language.text("status.reading_none").to_owned();
        };
        if self.selected.len() != 1 || self.selected_pipe.is_some() {
            return language.text("status.reading_none").to_owned();
        }
        let Some(placement) = self.document.placements().get(index) else {
            return language.text("status.reading_none").to_owned();
        };
        let Some(definition) = self.catalog.get(&placement.catalog_id) else {
            return language.text("status.reading_none").to_owned();
        };
        let Some(model) = self.model.as_ref() else {
            return language.text("status.reading_none").to_owned();
        };
        let Some(network_part) = model.part_of(index) else {
            return language.text("status.reading_none").to_owned();
        };
        let Some(port) = definition.port.iter().find(|port| {
            !matches!(
                port.role,
                punchi_fluid_catalog::PortRole::Mechanical
                    | punchi_fluid_catalog::PortRole::Electrical
            )
        }) else {
            return language.text("status.reading_none").to_owned();
        };
        let Some(pressure) = model.network().port_pressure(network_part, &port.id) else {
            return language.text("status.reading_none").to_owned();
        };
        let gauge = (pressure - model.network().atmosphere_pa()) / 1.0e5;
        let value = language.decimal(gauge, 2);
        language.format(
            "status.reading_port",
            &[("port", port.numeral()), ("value", &value)],
        )
    }

    fn status_selection(&self, language: Language) -> String {
        let circuit = self.active_circuit_title();
        if self.selected_pipe.is_some() {
            return language.format("status.selection_pipe", &[("circuit", circuit)]);
        }
        match self.selected.as_slice() {
            [] => language.text("status.selection_none").to_owned(),
            [index] => {
                let name = self
                    .selected_component_name(*index)
                    .unwrap_or_else(|| language.text("status.selection_unknown").to_owned());
                language.format(
                    "status.selection_component",
                    &[("name", &name), ("circuit", circuit)],
                )
            }
            selected => {
                let count = selected.len().to_string();
                language.format("status.selection_many", &[("count", &count)])
            }
        }
    }

    fn selected_component_name(&self, index: usize) -> Option<String> {
        let placement = self.document.placements().get(index)?;
        let definition = self.catalog.get(&placement.catalog_id)?;
        let name = match self.language {
            Language::Spanish => definition.name_es.as_deref().unwrap_or(&definition.name),
            Language::English => &definition.name,
        };
        Some(name.to_owned())
    }

    fn active_circuit_title(&self) -> &str {
        let active = self.document.active_circuit();
        self.document
            .circuits()
            .iter()
            .find(|circuit| circuit.id == active)
            .map_or("", |circuit| circuit.title.as_str())
    }
}
