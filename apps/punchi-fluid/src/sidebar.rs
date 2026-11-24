// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The sidebar: the catalog as a palette of tiles.
//!
//! The third region of ADR 0035's fixed layout. Every entry is drawn with the symbol of its
//! own definition by the same renderer as the sheet, through the panel preview: the symbol
//! without the connection points the sheet adds. A port closure is not a palette entry: it
//! is fitted to a port instead of being placed (ADR 0033).
//!
//! The fold, the header and the rail are the shared chrome in [`crate::panel`]: the catalog
//! folds to its rail, remembers its width for the session and **Restablecer vista** puts it
//! back, exactly as the right dock does.

use eframe::egui;
use punchi_fluid_catalog::{Behaviour, Catalog, Category, ComponentDefinition, Kinematics};
use punchi_fluid_draw::surface::Colour;
use punchi_fluid_draw::symbol::Ink;
use punchi_fluid_draw::{CYLINDER_RATIO, Pt, Rect, SYMBOL_BOX_MM, draw_component_preview};
use punchi_fluid_i18n::Language;

use crate::Window;
use crate::panel::{Edge, Panel, Probe, Settings};
use crate::surface::EguiSurface;
use crate::theme::Tokens;

/// The categories in the palette's order, with their catalog key.
const CATEGORIES: [(Category, &str); 6] = [
    (Category::Supply, "catalog.category.supply"),
    (Category::Valve, "catalog.category.valve"),
    (Category::Actuator, "catalog.category.actuator"),
    (Category::Throttle, "catalog.category.throttle"),
    (Category::Accessory, "catalog.category.accessory"),
    (Category::Sensor, "catalog.category.sensor"),
];

/// The catalog's width when it is open, and the bounds a drag may take it to. The preferred
/// width and the floor are the interface constants `docs/ui-ux.md` §2.2 fixes for the
/// sidebar; the maximum is the drag bound, not a layout measurement.
pub const DEFAULT_WIDTH: f32 = crate::layout::CATALOG_PREFERRED_WIDTH;
const MIN_WIDTH: f32 = crate::layout::CATALOG_MIN_WIDTH;
const MAX_WIDTH: f32 = 400.0;

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

/// The catalog's content state: the search text and the frame that asks for its focus.
#[derive(Debug, Clone, Default)]
struct Body {
    filter: String,
    /// The search asks for the keyboard on the next frame; Ctrl+F sets it.
    focus_search: bool,
}

/// The sidebar: the shared panel chrome and the palette's own search state.
pub struct Sidebar {
    chrome: Panel,
    body: Body,
    /// The language the panel and its content draw with; the window sets it each frame.
    pub(crate) language: Language,
}

impl Sidebar {
    #[must_use]
    pub fn new() -> Self {
        Self {
            chrome: Panel::new(Settings {
                edge: Edge::Left,
                id: "catalog",
                default_width: DEFAULT_WIDTH,
                min_width: MIN_WIDTH,
                max_width: MAX_WIDTH,
                width_label: "panel.catalog.width",
                show_label: "panel.catalog.show",
                hide_label: "panel.catalog.hide",
                shortcut: "Ctrl+B",
                margin: 6,
            }),
            body: Body::default(),
            language: Language::English,
        }
    }

    /// How many entries the panel counts in its header: a port closure is not an entry.
    #[must_use]
    pub fn entry_count(catalog: &Catalog) -> usize {
        CATEGORIES
            .iter()
            .flat_map(|(category, _)| catalog.by_category(*category))
            .filter(|definition| is_entry(definition))
            .count()
    }

    /// Whether the panel is open: the window reads it into the settings file and the tests
    /// read it back.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.chrome.is_open()
    }

    #[must_use]
    pub fn width(&self) -> f32 {
        self.chrome.width()
    }

    /// The width the panel takes when it floats over the sheet in the compact mode.
    #[must_use]
    pub fn floating_width(&self) -> f32 {
        self.chrome.floating_width()
    }

    /// The panel frame's horizontal inner margin, so the drawer's content lines up with the
    /// docked one.
    #[must_use]
    pub fn margin(&self) -> i8 {
        self.chrome.margin()
    }

    #[cfg(test)]
    #[must_use]
    pub fn shade(&self) -> f32 {
        self.chrome.shade()
    }

    /// Hides the catalog, or brings it back at the width it had.
    pub fn toggle(&mut self) {
        self.chrome.toggle();
    }

    /// Asks the search field for the keyboard on the next frame.
    pub fn focus_search(&mut self) {
        self.body.focus_search = true;
    }

    /// Opens the panel: the docked one in the wide and medium modes, or its drawer.
    pub fn open(&mut self) {
        self.chrome.open();
    }

    /// Closes the panel, whether it was docked or floated.
    pub fn close(&mut self) {
        self.chrome.close();
    }

    /// **Restablecer vista**: open again at the preferred width.
    pub fn reset(&mut self) {
        self.chrome.reset();
    }

    /// The panel as a settings file left it: open or folded at the remembered width.
    pub fn restore(&mut self, open: bool, width: f32) {
        self.chrome.restore(open, width);
    }

    /// The width the layout mode allows, clamped to the panel's own drag bounds.
    pub fn set_width(&mut self, width: f32) {
        self.chrome.set_width(width);
    }

    /// What the last frame told the auto-hide timers: the shell reads it after the panel drew.
    #[must_use]
    pub fn probe(&self) -> Probe {
        self.chrome.probe()
    }

    /// Records the drawer's surface, which the shell draws itself in the compact mode.
    pub fn observe_surface(&mut self, context: &eframe::egui::Context, rect: eframe::egui::Rect) {
        self.chrome.observe_surface(context, rect);
    }

    #[cfg(test)]
    pub fn probe_for_test(&mut self, rail_hovered: bool, pointer_inside: bool, focus_inside: bool) {
        self.chrome
            .probe_for_test(rail_hovered, pointer_inside, focus_inside);
    }

    #[cfg(test)]
    pub fn resize_by(&mut self, delta: f32) {
        self.chrome.resize_by(delta);
    }

    /// Draws the palette and returns the entry the reader clicked, if any.
    pub fn show(
        &mut self,
        context: &egui::Context,
        tokens: &Tokens,
        subtitle: &str,
        catalog: &Catalog,
        docked_width: f32,
    ) -> Option<String> {
        let Sidebar {
            chrome,
            body,
            language,
        } = self;
        chrome.set_language(*language);
        let title = language.text("catalog.title");
        chrome
            .show(context, tokens, title, Some(subtitle), docked_width, |ui| {
                body.show(ui, tokens, catalog, *language)
            })
            .flatten()
    }

    /// Draws only the rail, for the compact shell: the catalog is one click from its drawer.
    pub fn show_rail(&mut self, context: &egui::Context, tokens: &Tokens) -> bool {
        self.chrome.set_language(self.language);
        self.chrome.show_rail(context, tokens)
    }

    /// Draws the palette in the floating drawer. Reports the entry chosen and whether the
    /// hide control asked to close the panel.
    pub fn show_drawer(
        &mut self,
        ui: &mut egui::Ui,
        tokens: &Tokens,
        subtitle: &str,
        catalog: &Catalog,
    ) -> (Option<String>, bool) {
        let Sidebar {
            chrome,
            body,
            language,
        } = self;
        chrome.set_language(*language);
        let title = language.text("catalog.title");
        chrome.show_body(ui, tokens, title, Some(subtitle), |ui| {
            body.show(ui, tokens, catalog, *language)
        })
    }
}

impl Body {
    /// The search, the category groups, the tiles and the footer.
    fn show(
        &mut self,
        ui: &mut egui::Ui,
        tokens: &Tokens,
        catalog: &Catalog,
        language: Language,
    ) -> Option<String> {
        let mut chosen = None;
        ui.add_space(6.0);
        let search = ui.add(
            egui::TextEdit::singleline(&mut self.filter)
                .hint_text(language.text("catalog.search"))
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
                    let label = language.text(label);
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
                                if tile(ui, tokens, definition, tile_width, language) {
                                    chosen = Some(definition.id.clone());
                                }
                            }
                        });
                    });
                }
            });
        let skipped = catalog.problems().len();
        let footer_text = if skipped == 0 {
            language.text("catalog.integrated").to_owned()
        } else {
            let skipped = skipped.to_string();
            language.format("catalog.integrated_skipped", &[("count", &skipped)])
        };
        ui.label(
            egui::RichText::new(footer_text)
                .size(tokens.text_small)
                .color(tokens.ink_muted),
        );
        chosen
    }
}

/// One tile: its own symbol, the name on at most two lines and the id.
fn tile(
    ui: &mut egui::Ui,
    tokens: &Tokens,
    definition: &ComponentDefinition,
    width: f32,
    language: Language,
) -> bool {
    let name_font = egui::FontId::proportional(tokens.text_body);
    let id_font = egui::FontId::proportional(tokens.text_small);
    // Two lines of name and one of id, measured rather than guessed: a 42-character name
    // used to draw four lines over the next row.
    let name_row = ui.fonts(|fonts| fonts.row_height(&name_font));
    let id_row = ui.fonts(|fonts| fonts.row_height(&id_font));
    let height = 32.0 + name_row * 2.0 + 4.0 + id_row + 2.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());
    if !ui.is_rect_visible(rect) {
        return false;
    }
    if response.hovered() || response.has_focus() {
        ui.painter().rect_filled(rect, 4.0, tokens.accent_soft);
    }
    let symbol = egui::Rect::from_min_size(rect.min, egui::vec2(width, 32.0));
    let (box_width, box_height) = box_of(definition);
    let scale = (symbol.width().min(symbol.height()) * 0.8 / box_width.max(box_height)).max(0.05);
    let origin = symbol.center() - egui::vec2(box_width * scale * 0.5, box_height * scale * 0.5);
    let mut drawing = EguiSurface::new(ui.ctx(), origin, scale);
    draw_component_preview(
        &mut drawing,
        Rect::from_min_size(Pt::new(0.0, 0.0), box_width, box_height),
        definition,
        0,
        0.0,
        None,
        &preview_ink(tokens),
    );
    ui.painter().extend(drawing.shapes);
    let name = match language {
        Language::Spanish => definition.name_es.as_deref().unwrap_or(&definition.name),
        Language::English => &definition.name,
    };
    let name_galley = ui.painter().layout_job(name_job(
        name,
        (width - 4.0).max(1.0),
        name_font,
        tokens.ink,
    ));
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
        id_font,
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
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            ui.is_enabled(),
            format!("{name}, {}", definition.id),
        )
    });
    response
        .on_hover_text(format!("{name}\n{}", definition.id))
        .clicked()
}

/// The name of a tile: at most two lines, elided, so a long name cannot grow over the next
/// tile. The full name stays in the tooltip and in the accessible name.
fn name_job(
    name: &str,
    width: f32,
    font: egui::FontId,
    colour: egui::Color32,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob {
        wrap: egui::text::TextWrapping {
            max_width: width,
            max_rows: 2,
            break_anywhere: true,
            overflow_character: Some('…'),
        },
        ..Default::default()
    };
    job.append(
        name,
        0.0,
        egui::TextFormat {
            font_id: font,
            color: colour,
            ..Default::default()
        },
    );
    job
}

impl Window {
    /// Draws the catalog between the ribbon and the sheet. The shell calls this before the
    /// right dock, so the sheet's available area follows both panels as they fold; the
    /// catalog computes its docked width from the same mode the shell derived.
    pub(crate) fn show_catalog(&mut self, context: &egui::Context) {
        self.sidebar.language = self.language;
        let mode = crate::layout::LayoutMode::from_available(context.screen_rect().width());
        if mode == crate::layout::LayoutMode::Compact {
            return;
        }
        let docked = mode
            .widths(
                context.screen_rect().width(),
                self.sidebar.width(),
                self.inspector.width(),
            )
            .catalog;
        let count = Sidebar::entry_count(&self.catalog).to_string();
        let subtitle = self
            .language
            .format("catalog.components", &[("count", &count)]);
        let chosen = self.sidebar.show(
            context,
            &self.theme.tokens,
            &subtitle,
            &self.catalog,
            docked,
        );
        if let Some(catalog_id) = chosen {
            self.add_component(&catalog_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use eframe::egui;
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
    fn a_long_name_stays_on_two_lines() {
        let context = egui::Context::default();
        if crate::theme::install_font(&context, None).is_err() {
            return;
        }
        let job = super::name_job(
            "Válvula 3/2, solenoide, retorno por muelle",
            104.0,
            egui::FontId::proportional(11.0),
            egui::Color32::WHITE,
        );
        assert_eq!(job.wrap.max_rows, 2);
        assert_eq!(job.wrap.overflow_character, Some('…'));
        let mut rows = 0;
        let _ = context.run(egui::RawInput::default(), |context| {
            rows = context
                .fonts(|fonts| fonts.layout_job(job.clone()))
                .rows
                .len();
        });
        assert!(rows <= 2, "the tile draws two lines, not {rows}");
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
