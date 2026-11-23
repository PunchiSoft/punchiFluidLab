// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Inicio page: the four start templates and the welcome.
//!
//! The templates are composed in `punchi-fluid-session` and shared with the ABI, so this
//! page reads the same documents the old window offered. The page is the Rust window's
//! first screen; choosing a card loads that template and closes the page.

use eframe::egui;
use punchi_fluid_draw::surface::Rect;
use punchi_fluid_session::templates::{self, Template};

/// Builds the document of a start template, when the catalog provides it.
pub fn document(
    index: u32,
    catalog: &punchi_fluid_catalog::Catalog,
    area: Rect,
) -> Option<punchi_fluid_doc::Document> {
    templates::document(index, catalog, area).ok()
}

/// Draws the page and returns the template the reader chose, by index.
#[must_use]
pub fn show(ui: &mut egui::Ui) -> Option<u32> {
    let mut chosen = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(16.0);
        ui.heading("Punchi Fluid Lab");
        ui.label("Elige una plantilla para empezar; todo se puede deshacer.");
        ui.add_space(16.0);
        ui.horizontal_wrapped(|ui| {
            for index in 0..templates::COUNT {
                let Some(template) = Template::from_index(index) else {
                    continue;
                };
                let (name, description) = labels(template);
                let card = egui::Button::new(
                    egui::RichText::new(format!("{name}\n{description}")).size(14.0),
                )
                .min_size(egui::vec2(210.0, 96.0));
                if ui.add(card).clicked() {
                    chosen = Some(index);
                }
            }
        });
        ui.add_space(24.0);
        ui.heading("Recientes");
        ui.label("Todavía no hay documentos recientes en esta ventana.");
    });
    chosen
}

/// The name and the one-line description of each start template.
fn labels(template: Template) -> (&'static str, &'static str) {
    match template {
        Template::WorkedExample => (
            "Circuito OR",
            "Dos entradas; el shuttle deja pasar la mayor y con las dos cierra la salida.",
        ),
        Template::DoubleActing => (
            "Cilindro de doble efecto",
            "Una válvula 5/2 y un cilindro de dos cámaras.",
        ),
        Template::AirPreparation => (
            "Preparación de aire",
            "Filtro, regulador y lubricador antes de la válvula.",
        ),
        Template::ElectricalControl => {
            ("Control eléctrico", "Un relé y una válvula con solenoide.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_template_has_a_name_and_a_description() {
        for index in 0..templates::COUNT {
            let template = Template::from_index(index).expect("a template");
            let (name, description) = labels(template);
            assert!(!name.is_empty() && !description.is_empty());
        }
    }
}
