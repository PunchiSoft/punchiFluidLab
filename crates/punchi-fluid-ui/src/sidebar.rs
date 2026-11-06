// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The sidebar *is* the catalog.
//!
//! Groups come from the definition's `category`, entries from the catalog itself and
//! every thumbnail is the component's own symbol, drawn by the same renderer as the
//! sheet. There is no second icon set that could drift from the diagram.

use std::path::Path;

use egui::{Align2, FontId, Pos2, Rect, Sense, Ui, Vec2};
use punchi_fluid_catalog::{Catalog, Category, ComponentDefinition};
use punchi_fluid_theme::{Theme, Token};

use crate::surface::{self, EguiSurface};

/// Categories in the order the sidebar shows them.
const GROUPS: [(Category, &str); 6] = [
    (Category::Supply, "Supply"),
    (Category::Valve, "Valves"),
    (Category::Actuator, "Actuators"),
    (Category::Throttle, "Flow control"),
    (Category::Accessory, "Accessories"),
    (Category::Sensor, "Instruments"),
];

/// Sidebar state that survives between frames.
#[derive(Debug, Default)]
pub struct SidebarState {
    /// Text of the search box.
    pub search: String,
    /// The next frame moves keyboard focus to the search box.
    pub focus_search: bool,
}

/// What the user asked of the sidebar this frame.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SidebarEvent {
    /// Identifier of a component the user double-clicked.
    pub place: Option<String>,
    /// The user asked to reload the catalog.
    pub reload: bool,
}

/// Draws the sidebar and reports the interaction.
pub fn show(
    ui: &mut Ui,
    theme: &Theme,
    catalog: &Catalog,
    state: &mut SidebarState,
    user_dir: Option<&Path>,
) -> SidebarEvent {
    let mut event = SidebarEvent::default();

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Catalog").strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .button("Reload")
                .on_hover_text("Re-read the built-in and user definitions (F5)")
                .clicked()
            {
                event.reload = true;
            }
        });
    });

    let search = ui.add(
        egui::TextEdit::singleline(&mut state.search)
            .hint_text("Search the catalog")
            .desired_width(f32::INFINITY),
    );
    if state.focus_search {
        search.request_focus();
        state.focus_search = false;
    }
    ui.separator();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let mut shown = 0_usize;
            for (category, title) in GROUPS {
                let entries: Vec<&ComponentDefinition> = catalog
                    .by_category(category)
                    .filter(|definition| matches(definition, &state.search))
                    .collect();
                if entries.is_empty() {
                    continue;
                }
                shown += entries.len();
                egui::CollapsingHeader::new(format!("{title} ({})", entries.len()))
                    .id_salt(category.as_str())
                    .default_open(true)
                    .show(ui, |ui| {
                        for definition in entries {
                            let id = egui::Id::unique(("catalog-entry", definition.id.as_str()));
                            let row = ui
                                .dnd_drag_source(id, definition.id.clone(), |ui| {
                                    entry(ui, theme, definition)
                                })
                                .response;
                            if row.double_clicked() {
                                event.place = Some(definition.id.clone());
                            }
                        }
                    });
            }
            if shown == 0 {
                ui.label(
                    egui::RichText::new("Nothing matches that search")
                        .small()
                        .color(theme.colour(Token::TextMuted)),
                );
            }
        });

    ui.separator();
    if let Some(dir) = user_dir {
        ui.label(
            egui::RichText::new(dir.display().to_string())
                .small()
                .color(theme.colour(Token::TextMuted)),
        );
    }
    let skipped = catalog.problems().len();
    if skipped == 0 {
        ui.label(
            egui::RichText::new(format!("{} definitions loaded", catalog.len()))
                .small()
                .color(theme.colour(Token::TextMuted)),
        );
    } else {
        ui.label(
            egui::RichText::new(format!("{skipped} file(s) skipped"))
                .small()
                .color(theme.colour(Token::Warning)),
        );
    }

    event
}

/// One catalog row: thumbnail from the definition, name and identifier.
fn entry(ui: &mut Ui, theme: &Theme, definition: &ComponentDefinition) -> egui::Response {
    let metrics = theme.metrics;
    let width = ui.available_width().max(metrics.thumbnail * 3.0);
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(width, metrics.row_height * 1.25), Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        if response.hovered() {
            painter.rect_filled(
                rect,
                metrics.radius,
                theme.palette.blend(Token::Surface, Token::Text, 0.06),
            );
        }
        // Square, because the symbol box of the catalog is square and a circle in a
        // wide box stops being a circle.
        let thumbnail = Rect::from_center_size(
            Pos2::new(
                rect.left() + metrics.thumbnail * 0.5 + metrics.grid_unit * 0.2,
                rect.center().y,
            ),
            Vec2::splat(metrics.thumbnail),
        );
        let ink = surface::ink(theme);
        let active = match &definition.behaviour {
            punchi_fluid_catalog::Behaviour::Valve {
                default_position, ..
            } => *default_position,
            _ => 0,
        };
        let mut target = EguiSurface::new(painter, thumbnail.min);
        let area = punchi_fluid_draw::Rect::from_min_size(
            punchi_fluid_draw::Pt::new(0.0, 0.0),
            thumbnail.width(),
            thumbnail.height(),
        );
        punchi_fluid_draw::draw_component(&mut target, area, definition, active, 0.4, &ink);

        let text_left = thumbnail.right() + metrics.grid_unit * 0.4;
        let centre = rect.center().y;
        painter.text(
            Pos2::new(text_left, centre - metrics.grid_unit * 0.35),
            Align2::LEFT_CENTER,
            &definition.name,
            FontId::proportional(theme.font_size * 0.95),
            theme.colour(Token::Text),
        );
        painter.text(
            Pos2::new(text_left, centre + metrics.grid_unit * 0.50),
            Align2::LEFT_CENTER,
            &definition.id,
            FontId::proportional(theme.font_size * 0.72),
            theme.colour(Token::TextMuted),
        );
    }

    response.on_hover_text(format!(
        "{}\n{}",
        definition.name,
        definition.summary.as_deref().unwrap_or_default()
    ))
}

/// Search over name, identifier and the Spanish name.
///
/// A pure function so the behaviour is tested without a window.
#[must_use]
pub fn matches(definition: &ComponentDefinition, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return true;
    }
    let contains = |value: &str| value.to_lowercase().contains(&query);
    contains(&definition.name)
        || contains(&definition.id)
        || definition
            .name_es
            .as_deref()
            .is_some_and(|name| name.to_lowercase().contains(&query))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_query_keeps_everything() {
        let catalog = Catalog::builtin();
        assert!(catalog.iter().all(|definition| matches(definition, "   ")));
    }

    #[test]
    fn the_search_finds_by_name_identifier_and_spanish_name() {
        let catalog = Catalog::builtin();
        let valve = catalog
            .get("valve.3_2.spring_return")
            .expect("the built-in valve exists");
        assert!(matches(valve, "3/2"));
        assert!(matches(valve, "VALVE.3_2"));
        assert!(matches(valve, "válvula"));
        assert!(!matches(valve, "compressor"));
    }

    #[test]
    fn every_group_has_a_title_and_a_category() {
        for (category, title) in GROUPS {
            assert!(!title.is_empty());
            assert!(!category.as_str().is_empty());
        }
    }
}
