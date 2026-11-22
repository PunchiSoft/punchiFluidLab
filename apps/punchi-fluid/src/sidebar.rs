// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The sidebar: the catalog as a palette of tiles.
//!
//! The third region of ADR 0035's fixed layout. Every entry is drawn with the symbol of its
//! own definition by the same renderer as the sheet, through the panel preview: the symbol
//! without the connection points the sheet adds. A port closure is not a palette entry: it
//! is fitted to a port instead of being placed (ADR 0033).

use eframe::egui;
use punchi_fluid_catalog::{Behaviour, Catalog, Category, ComponentDefinition, Kinematics};
use punchi_fluid_draw::surface::Colour;
use punchi_fluid_draw::symbol::Ink;
use punchi_fluid_draw::{CYLINDER_RATIO, Pt, Rect, SYMBOL_BOX_MM, draw_component_preview};

use crate::surface::EguiSurface;
use crate::theme::Tokens;

/// The categories in the palette's order, with the Spanish label the interface shows.
const CATEGORIES: [(Category, &str); 6] = [
    (Category::Supply, "Alimentación"),
    (Category::Valve, "Válvulas"),
    (Category::Actuator, "Actuadores"),
    (Category::Throttle, "Cierre y control de caudal"),
    (Category::Accessory, "Accesorios"),
    (Category::Sensor, "Sensores e instrumentos"),
];

/// Whether a definition is a palette entry: a port closure is not.
fn is_entry(definition: &ComponentDefinition) -> bool {
    !matches!(
        definition.behaviour,
        Behaviour::Plug | Behaviour::Silencer { .. }
    )
}

/// Whether the definition matches the search: its name, its Spanish name and its id.
fn matches(definition: &ComponentDefinition, filter: &str) -> bool {
    if filter.trim().is_empty() {
        return true;
    }
    let filter = filter.to_lowercase();
    definition.id.to_lowercase().contains(&filter)
        || definition.name.to_lowercase().contains(&filter)
        || definition
            .name_es
            .as_deref()
            .is_some_and(|name| name.to_lowercase().contains(&filter))
}

/// The symbol box of a definition, in millimetres: the canonical square, or the actuator's
/// wider barrel box, the same box the templates place.
fn box_of(definition: &ComponentDefinition) -> (f32, f32) {
    if definition.kinematics == Kinematics::LinearActuator {
        (SYMBOL_BOX_MM * CYLINDER_RATIO, SYMBOL_BOX_MM)
    } else {
        (SYMBOL_BOX_MM, SYMBOL_BOX_MM)
    }
}

/// The ink of a thumbnail: the desktop's colours, not the printed sheet's.
fn preview_ink(tokens: &Tokens) -> Ink {
    let colour = |colour: egui::Color32| Colour {
        r: colour.r(),
        g: colour.g(),
        b: colour.b(),
    };
    Ink {
        line: colour(tokens.ink),
        dim: colour(tokens.ink_dim),
        muted: colour(tokens.ink_muted),
        text: colour(tokens.ink),
        supply: colour(tokens.supply),
        work: colour(tokens.work),
        accent: colour(tokens.accent),
        background: colour(tokens.panel),
    }
}

/// The sidebar's state the window keeps between frames.
pub struct Sidebar {
    filter: String,
    /// The search asks for the keyboard on the next frame; Ctrl+F sets it.
    pub focus_search: bool,
}

impl Sidebar {
    #[must_use]
    pub fn new() -> Self {
        Self {
            filter: String::new(),
            focus_search: false,
        }
    }

    /// Draws the palette; returns the entry the reader clicked, if any.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        tokens: &Tokens,
        catalog: &Catalog,
    ) -> Option<String> {
        let entries: Vec<&ComponentDefinition> = CATEGORIES
            .iter()
            .flat_map(|(category, _)| catalog.by_category(*category))
            .filter(|definition| is_entry(definition))
            .collect();
        let mut chosen = None;
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Catálogo")
                    .size(tokens.text_label)
                    .strong()
                    .color(tokens.ink),
            );
            ui.label(
                egui::RichText::new(format!("{} componentes", entries.len()))
                    .size(tokens.text_small)
                    .color(tokens.ink_muted),
            );
        });
        ui.add_space(2.0);
        let search = ui.add(
            egui::TextEdit::singleline(&mut self.filter)
                .hint_text("Buscar…")
                .desired_width(f32::INFINITY),
        );
        if self.focus_search {
            search.request_focus();
            self.focus_search = false;
        }
        ui.add_space(4.0);
        let footer_height = 18.0;
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - footer_height)
            .show(ui, |ui| {
                let tile_width = (ui.available_width() - ui.spacing().item_spacing.x) * 0.5;
                for (category, label) in CATEGORIES {
                    let matching: Vec<&ComponentDefinition> = catalog
                        .by_category(category)
                        .filter(|definition| {
                            is_entry(definition) && matches(definition, &self.filter)
                        })
                        .collect();
                    if matching.is_empty() {
                        continue;
                    }
                    egui::CollapsingHeader::new(
                        egui::RichText::new(format!("{label}  {}", matching.len()))
                            .size(tokens.text_body)
                            .color(tokens.ink),
                    )
                    .id_salt(category.as_str())
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            for definition in &matching {
                                if Self::tile(ui, tokens, definition, tile_width) {
                                    chosen = Some(definition.id.clone());
                                }
                            }
                        });
                    });
                }
            });
        let skipped = catalog.problems().len();
        let footer_text = if skipped == 0 {
            String::from("Catálogo integrado")
        } else {
            format!("Catálogo integrado · {skipped} omitidos")
        };
        ui.label(
            egui::RichText::new(footer_text)
                .size(tokens.text_small)
                .color(tokens.ink_muted),
        );
        chosen
    }

    /// One tile: its own symbol, the name on as many lines as it needs, and the id.
    fn tile(
        ui: &mut egui::Ui,
        tokens: &Tokens,
        definition: &ComponentDefinition,
        width: f32,
    ) -> bool {
        let height = 32.0 + tokens.text_body * 2.6 + tokens.text_small * 1.6;
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());
        if !ui.is_rect_visible(rect) {
            return false;
        }
        if response.hovered() || response.has_focus() {
            ui.painter().rect_filled(rect, 4.0, tokens.accent_soft);
        }
        let symbol = egui::Rect::from_min_size(rect.min, egui::vec2(width, 32.0));
        let (box_width, box_height) = box_of(definition);
        let scale =
            (symbol.width().min(symbol.height()) * 0.8 / box_width.max(box_height)).max(0.05);
        let origin =
            symbol.center() - egui::vec2(box_width * scale * 0.5, box_height * scale * 0.5);
        let mut drawing = EguiSurface::new(ui.ctx(), origin, scale);
        draw_component_preview(
            &mut drawing,
            Rect::from_min_size(Pt::new(0.0, 0.0), box_width, box_height),
            definition,
            0,
            0.0,
            &preview_ink(tokens),
        );
        ui.painter().extend(drawing.shapes);
        let name = definition.name_es.as_deref().unwrap_or(&definition.name);
        let name_galley = ui.painter().layout(
            name.to_owned(),
            egui::FontId::proportional(tokens.text_body),
            tokens.ink,
            width - 4.0,
        );
        ui.painter().galley(
            egui::pos2(
                rect.center().x - name_galley.size().x * 0.5,
                symbol.max.y + 1.0,
            ),
            name_galley,
            tokens.ink,
        );
        let id_galley = ui.painter().layout(
            definition.id.clone(),
            egui::FontId::proportional(tokens.text_small),
            tokens.ink_muted,
            width - 4.0,
        );
        ui.painter().galley(
            egui::pos2(
                rect.center().x - id_galley.size().x * 0.5,
                rect.max.y - id_galley.size().y - 1.0,
            ),
            id_galley,
            tokens.ink_muted,
        );
        response
            .on_hover_text(format!("{name}\n{}", definition.id))
            .clicked()
    }
}

#[cfg(test)]
mod tests {
    use punchi_fluid_catalog::Catalog;

    use super::{is_entry, matches};

    #[test]
    fn the_palette_hides_the_port_closures() {
        let catalog = Catalog::builtin();
        assert!(!is_entry(
            catalog.get("plug.blind").expect("the plug exists")
        ));
        assert!(!is_entry(
            catalog
                .get("silencer.pneumatic")
                .expect("the silencer exists")
        ));
        assert!(is_entry(
            catalog
                .get("valve.3_2.spring_return")
                .expect("the valve exists")
        ));
    }

    #[test]
    fn the_search_matches_the_name_the_spanish_name_and_the_id() {
        let catalog = Catalog::builtin();
        let valve = catalog
            .get("valve.3_2.spring_return")
            .expect("the valve exists");
        assert!(matches(valve, ""));
        assert!(matches(valve, "válvula"));
        assert!(matches(valve, "3/2"));
        assert!(matches(valve, "valve.3_2"));
        assert!(!matches(valve, "cilindro"));
    }
}
