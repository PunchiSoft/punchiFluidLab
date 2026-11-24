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
    /// The tile the reader chose: the accent bar and the name say it, never hover.
    chosen: Option<String>,
    /// The tile whose menu is open, with the screen point where it opened.
    menu: Option<(String, egui::Pos2)>,
    /// The tiles' rectangles of the last frame, for the tests that drive the pointer.
    #[cfg(test)]
    tiles: Vec<(String, egui::Rect)>,
    /// The tile the pointer is carrying from the palette to the sheet, if any.
    carry: Option<String>,
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

    /// The tiles' rectangles of the last frame, for the tests that drive the pointer.
    #[cfg(test)]
    pub(crate) fn tile_rects(&self) -> &[(String, egui::Rect)] {
        &self.body.tiles
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
    ) -> Option<CatalogAction> {
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

    /// Drops the carried tile: `Esc` and a lost gesture cancel the carry, never place it.
    pub(crate) fn cancel_carry(&mut self) {
        self.body.carry = None;
    }

    /// Starts a carry without the pointer, for the tests that check the state contract.
    #[cfg(test)]
    pub(crate) fn carry_for_test(&mut self, id: &str) {
        self.body.carry = Some(id.to_owned());
    }

    /// Whether a tile is being carried from the palette: the auto-hide and the keyboard
    /// handlers leave the gesture alone while this is true.
    #[must_use]
    pub(crate) fn carrying(&self) -> bool {
        self.body.carry.is_some()
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
    ) -> (Option<CatalogAction>, bool) {
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
    #[expect(
        clippy::too_many_lines,
        reason = "the palette's body is one pass: the search, the groups, the tiles and the carry"
    )]
    fn show(
        &mut self,
        ui: &mut egui::Ui,
        tokens: &Tokens,
        catalog: &Catalog,
        language: Language,
    ) -> Option<CatalogAction> {
        let mut chosen = None;
        let mut select = None;
        let mut open_menu = None;
        let mut carry = None;
        let mut drop = None;
        #[cfg(test)]
        self.tiles.clear();
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
            .scroll_source(egui::scroll_area::ScrollSource {
                scroll_bar: true,
                drag: false,
                mouse_wheel: true,
            })
            .max_height(ui.available_height() - footer_height)
            .show(ui, |ui| {
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
                        // The width comes from the group's body: the collapsing header indents
                        // it, and a width measured outside made the tiles wrap to one column.
                        let tile_width = (ui.available_width() - ui.spacing().item_spacing.x) * 0.5;
                        ui.horizontal_wrapped(|ui| {
                            for definition in &matching {
                                let (action, tile_rect) = tile(
                                    ui,
                                    tokens,
                                    definition,
                                    tile_width,
                                    language,
                                    self.chosen.as_deref() == Some(&definition.id),
                                );
                                #[cfg(test)]
                                self.tiles.push((definition.id.clone(), tile_rect));
                                #[cfg(not(test))]
                                let _ = tile_rect;
                                match action {
                                    TileAction::None => {}
                                    TileAction::Select => select = Some(definition.id.clone()),
                                    TileAction::Add => {
                                        chosen = Some(CatalogAction::Add(definition.id.clone()));
                                    }
                                    TileAction::Menu(position) => {
                                        open_menu = Some((definition.id.clone(), position));
                                    }
                                    TileAction::Carry => carry = Some(definition.id.clone()),
                                    TileAction::Drop(screen) => {
                                        drop = Some((definition.id.clone(), screen));
                                    }
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
        if let Some(id) = select {
            self.chosen = Some(id);
        }
        if let Some(menu) = open_menu {
            self.menu = Some(menu);
        }
        if let Some(id) = self.tile_menu(ui, tokens, language) {
            chosen = Some(CatalogAction::Add(id));
        }
        if let Some(action) = self.settle_carry(ui, tokens, catalog, carry, drop) {
            chosen = Some(action);
        }
        chosen
    }

    /// Keeps the carry, turns a release into the placement and draws the ghost. `Escape`
    /// drops the carry; the release that follows places nothing. The key is read, not
    /// consumed: another widget may have taken the escape already.
    fn settle_carry(
        &mut self,
        ui: &mut egui::Ui,
        tokens: &Tokens,
        catalog: &Catalog,
        carry: Option<String>,
        drop: Option<(String, egui::Pos2)>,
    ) -> Option<CatalogAction> {
        if let Some(id) = carry {
            self.carry = Some(id);
        }
        let placed = match drop {
            Some((id, screen)) if self.carry.take().as_deref() == Some(&id) => {
                Some(CatalogAction::Place { id, screen })
            }
            _ => None,
        };
        if self.carry.is_some() && ui.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.carry = None;
        }
        if let Some(id) = self.carry.clone() {
            if let Some(definition) = catalog.get(&id) {
                draw_carry_ghost(ui, tokens, definition);
            }
        }
        placed
    }

    /// The tile's menu: one entry, the same placement as the keyboard's. Returns the id the
    /// reader asked to add, or `None`.
    fn tile_menu(
        &mut self,
        ui: &mut egui::Ui,
        tokens: &Tokens,
        language: Language,
    ) -> Option<String> {
        let (id, position) = self.menu.clone()?;
        let mut add = false;
        let mut chosen = None;
        let area = egui::Area::new(egui::Id::new("catalog.tile.menu"))
            .order(egui::Order::Foreground)
            .fixed_pos(position)
            .show(ui.ctx(), |ui| {
                egui::Frame::default()
                    .fill(tokens.panel)
                    .stroke(egui::Stroke::new(1.0_f32, tokens.line))
                    .inner_margin(egui::Margin::same(6))
                    .show(ui, |ui| {
                        if ui.button(language.text("catalog.add_to_sheet")).clicked() {
                            add = true;
                        }
                    });
            });
        let pointer = ui.input(|input| input.pointer.interact_pos());
        let clicked_out = ui.input(|input| input.pointer.primary_clicked())
            && pointer.is_some_and(|at| !area.response.rect.contains(at));
        let escaped =
            ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        if add {
            self.chosen = Some(id.clone());
            self.menu = None;
            chosen = Some(id);
        } else if clicked_out || escaped {
            self.menu = None;
        }
        chosen
    }
}

/// What a tile asks for: nothing, a selection, the centre placement, its menu or the carry.
enum TileAction {
    None,
    Select,
    Add,
    Menu(egui::Pos2),
    Carry,
    Drop(egui::Pos2),
}

/// What the palette asks the window to do: place at the centre, or at the carried drop.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CatalogAction {
    /// Add the entry at the centre of the sheet, the menu's and the keyboard's path.
    Add(String),
    /// Place the carried entry where the pointer released it, in screen points.
    Place { id: String, screen: egui::Pos2 },
}

/// A painted control gets the keyboard focus and the accessibility layer's name.
fn focusable(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    id: egui::Id,
    sense: egui::Sense,
) -> egui::Response {
    let response = ui.interact(rect, id, sense);
    let layer = ui.layer_id();
    ui.memory_mut(|memory| memory.interested_in_focus(id, layer));
    response
}

/// Whether the reader activated the control with `Enter` or `Space`.
fn activated(ui: &egui::Ui, response: &egui::Response) -> bool {
    response.has_focus()
        && ui.input(|input| {
            input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Space)
        })
}

/// The carried symbol under the pointer, above every panel and the sheet: the reader sees
/// what will land where the drop happens.
fn draw_carry_ghost(ui: &egui::Ui, tokens: &Tokens, definition: &ComponentDefinition) {
    let Some(pointer) = ui.input(|input| input.pointer.interact_pos()) else {
        return;
    };
    let box_px = 56.0;
    let (box_width, box_height) = box_of(definition);
    let scale = box_px / box_width.max(box_height).max(0.05);
    let origin = pointer - egui::vec2(box_width * scale * 0.5, box_height * scale * 0.5);
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
    let overlay = ui.ctx().layer_painter(egui::LayerId::new(
        egui::Order::Tooltip,
        egui::Id::new("catalog.carry"),
    ));
    overlay.extend(drawing.shapes);
}

/// One tile: its own symbol, the name on at most two lines and the id. A click chooses it;
/// the keyboard or its menu add it; the right button opens the menu and never carries.
#[expect(
    clippy::too_many_lines,
    reason = "the tile measures, paints, names and drives itself in one place"
)]
fn tile(
    ui: &mut egui::Ui,
    tokens: &Tokens,
    definition: &ComponentDefinition,
    width: f32,
    language: Language,
    chosen: bool,
) -> (TileAction, egui::Rect) {
    let name_font = egui::FontId::proportional(tokens.text_body);
    let id_font = egui::FontId::proportional(tokens.text_small);
    // Two lines of name and one of id, measured rather than guessed: a 42-character name
    // used to draw four lines over the next row.
    let name_row = ui.fonts(|fonts| fonts.row_height(&name_font));
    let id_row = ui.fonts(|fonts| fonts.row_height(&id_font));
    let height = 32.0 + name_row * 2.0 + 4.0 + id_row + 2.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    if !ui.is_rect_visible(rect) {
        return (TileAction::None, rect);
    }
    let id = ui.make_persistent_id(("catalog.tile", &definition.id));
    let response = focusable(ui, rect, id, egui::Sense::click_and_drag());
    // Every tile is a card of its own in the reference: a subtle fill and a rule, inside
    // the panel's own card.
    ui.painter()
        .rect_filled(rect, tokens.radius_small, tokens.surface_sunken);
    ui.painter().rect_stroke(
        rect,
        tokens.radius_small,
        egui::Stroke::new(1.0_f32, tokens.line),
        egui::StrokeKind::Inside,
    );
    if chosen {
        ui.painter()
            .rect_filled(rect, tokens.radius_small, tokens.panel);
        ui.painter().rect_stroke(
            rect,
            tokens.radius_small,
            egui::Stroke::new(1.0_f32, tokens.accent),
            egui::StrokeKind::Inside,
        );
        ui.painter().rect_filled(
            egui::Rect::from_min_size(rect.min, egui::vec2(2.0, rect.height())),
            0.0,
            tokens.accent,
        );
    } else if response.hovered() || response.has_focus() {
        ui.painter()
            .rect_filled(rect, tokens.radius_small, tokens.accent_soft);
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect,
            4.0,
            egui::Stroke::new(1.0_f32, tokens.accent),
            egui::StrokeKind::Inside,
        );
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
        egui::WidgetInfo::selected(
            egui::WidgetType::Button,
            ui.is_enabled(),
            chosen,
            format!("{name}, {}", definition.id),
        )
    });
    let clicked = response.clicked();
    let secondary = response.secondary_clicked();
    let placed = activated(ui, &response);
    let started = response.drag_started();
    let stopped = response.drag_stopped();
    let release = response
        .interact_pointer_pos()
        .or_else(|| ui.input(|input| input.pointer.interact_pos()))
        .unwrap_or(rect.center());
    let focus = response.has_focus();
    let menu_key = focus && ui.input(|input| input.key_pressed(egui::Key::F10));
    response.on_hover_text(format!("{name}\n{}", definition.id));
    if clicked || started {
        // The keyboard follows the tile the reader picked or is carrying.
        ui.memory_mut(|memory| memory.request_focus(id));
    }
    let action = if stopped {
        TileAction::Drop(release)
    } else if started {
        TileAction::Carry
    } else if secondary || menu_key {
        TileAction::Menu(rect.center())
    } else if placed {
        TileAction::Add
    } else if clicked {
        TileAction::Select
    } else {
        TileAction::None
    };
    (action, rect)
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
        if let Some(action) = chosen {
            self.apply_catalog(action);
        }
    }
}

#[cfg(test)]
mod tests;
