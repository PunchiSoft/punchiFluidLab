// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The ribbon: a data table of tabs, groups and commands, a tab strip and the row of groups,
//! painted with the theme tokens and the embedded action icons.

use std::collections::HashMap;

use eframe::egui;
use punchi_fluid_doc::paper::{Margins, Orientation, PaperSize, Sheet};
use punchi_fluid_doc::title_block::{BlockPosition, ContentModel};
use punchi_fluid_file::recent::Recent;
use punchi_fluid_i18n::Language;

use crate::theme::Tokens;

mod command;
mod enabled;
mod gallery;
mod icons;
mod layout;
mod model;
mod recents;
mod split;
mod transport;
pub(crate) use transport::mode_key;

pub(crate) use command::{Command, MarginPreset, Speed};
use model::{GalleryEntry, Group, Item, MenuEntry, TABS};

/// The large command's box and label, shared by the plain and the split control. The theme
/// move and the font-scaled label are phase 3 of the interface plan.
pub(super) const LARGE_WIDTH: f32 = 64.0;
pub(super) const LARGE_HEIGHT: f32 = 58.0;
pub(super) const LARGE_ICON: f32 = 32.0;
pub(super) const LARGE_ICON_TOP: f32 = 19.0;
pub(super) const LARGE_LABEL_TOP: f32 = 37.0;
pub(super) const LARGE_LABEL_SIZE: f32 = 10.5;

/// One action drawing by id, for the sheet menu.
pub(crate) fn icon_bytes(id: &str) -> Option<&'static [u8]> {
    icons::bytes(id)
}

/// The colours the ribbon paints with, taken from the theme tokens.
pub struct Palette {
    pub raised: egui::Color32,
    pub window: egui::Color32,
    pub ink: egui::Color32,
    pub muted: egui::Color32,
    pub accent: egui::Color32,
    pub hover: egui::Color32,
    pub line: egui::Color32,
    pub text_small: f32,
    pub text_body: f32,
    pub text_label: f32,
}

impl Palette {
    #[must_use]
    pub fn from_theme(tokens: &Tokens) -> Self {
        Self {
            raised: tokens.surface_raised,
            window: tokens.background,
            ink: tokens.ink,
            muted: tokens.ink_muted,
            accent: tokens.accent,
            hover: tokens.accent_soft,
            line: tokens.line,
            text_small: tokens.text_small,
            text_body: tokens.text_body,
            text_label: tokens.text_label,
        }
    }
}

#[allow(
    clippy::struct_excessive_bools,
    reason = "the ribbon's state is five independent flags, not a bit set to encode"
)]
pub struct Ribbon {
    icons: HashMap<String, &'static [u8]>,
    active: usize,
    /// The visible language the table's keys resolve with; the window sets it each frame.
    pub(crate) language: Language,
    pub(crate) active_speed: &'static str,
    pub(crate) running: bool,
    pub(crate) simulation: bool,
    pub last: String,
    /// Whether there is a path to save to; the window sets it every frame.
    pub savable: bool,
    /// How many placements are selected; the window sets it every frame, so a command can
    /// ask for one, two (align) or the clipboard's own rules.
    pub selected: usize,
    /// Whether the sheet has a model the step commands can advance.
    pub can_step: bool,
    /// Whether the window's clipboard holds placements; the window sets it every frame.
    pub has_clipboard: bool,
    /// Whether the document has something to undo or redo; the window sets both every frame.
    pub can_undo: bool,
    pub can_redo: bool,
    /// Whether the view has a previous state for **Vista anterior**.
    pub can_go_back: bool,
    /// Whether the active circuit's block differs from its default; the window sets it.
    pub can_reset_block: bool,
    /// The sheet the active project draws on; the window sets it each frame.
    pub(crate) paper: PaperSize,
    pub(crate) orientation: Orientation,
    pub(crate) margins: Margins,
    /// The active circuit's title block, for the current model, position and visibility.
    pub(crate) block_model: ContentModel,
    pub(crate) block_position: BlockPosition,
    pub(crate) block_visible: bool,
    /// Whether a pipe is selected; the window sets it every frame.
    pub has_pipe: bool,
    /// Whether the document has more than one circuit; the window sets it.
    pub can_close_circuit: bool,
    pending: Option<Command>,
}

impl Ribbon {
    #[must_use]
    pub fn new() -> Self {
        Self {
            icons: icons::map(),
            active: 0,
            language: Language::English,
            active_speed: "ribbon.item.speed_normal",
            running: false,
            simulation: false,
            last: String::from("—"),
            savable: false,
            selected: 0,
            can_step: false,
            has_clipboard: false,
            can_undo: false,
            can_redo: false,
            can_go_back: false,
            can_reset_block: false,
            paper: PaperSize::A4,
            orientation: Orientation::Landscape,
            margins: Margins::iso5457(),
            block_model: ContentModel::Industrial,
            block_position: BlockPosition::BottomRight,
            block_visible: true,
            has_pipe: false,
            can_close_circuit: false,
            pending: None,
        }
    }

    /// Opens one tab by name, for a screenshot or a script.
    #[allow(dead_code, reason = "the tests and the screenshot tooling open a tab")]
    pub fn activate(&mut self, name: &str) {
        if let Some(index) = TABS.iter().position(|tab| {
            tab.name.eq_ignore_ascii_case(name)
                || self.language.text(tab.name).eq_ignore_ascii_case(name)
        }) {
            self.active = index;
        }
    }

    fn run_command(&mut self, command: Command) {
        if !self.enabled(command) {
            return;
        }
        match command {
            Command::Start => {
                self.running = true;
                self.simulation = true;
            }
            Command::Pause => {
                self.running = false;
                self.simulation = true;
            }
            Command::Stop => {
                self.running = false;
                self.simulation = false;
            }
            Command::Speed(speed) => self.active_speed = speed.label_key(),
            _ => {}
        }
        self.last = self.language.text(command.label_key()).to_string();
        self.pending = Some(command);
    }

    /// Takes the sheet the active project draws on.
    pub(crate) fn sync_sheet(&mut self, sheet: Sheet) {
        self.paper = sheet.paper;
        self.orientation = sheet.orientation;
        self.margins = sheet.margins;
    }

    /// Takes the active circuit's title block state.
    pub(crate) fn sync_block(
        &mut self,
        model: ContentModel,
        position: BlockPosition,
        visible: bool,
    ) {
        self.block_model = model;
        self.block_position = position;
        self.block_visible = visible;
    }

    /// Whether a command is the value in force, for the check a menu row draws.
    pub(super) fn command_selected(&self, command: Command) -> bool {
        match command {
            Command::PageMargin(preset) => self.margins == preset.margins(),
            Command::PageOrientation(orientation) => self.orientation == orientation,
            Command::PagePaper(paper) => self.paper == paper,
            Command::BlockModel(model) => self.block_visible && self.block_model == model,
            Command::BlockPosition(BlockPosition::None) => !self.block_visible,
            Command::BlockPosition(position) => {
                self.block_visible && self.block_position == position
            }
            _ => false,
        }
    }

    /// The current value a block menu shows under its name, if it has one.
    fn block_current(&self, label: &'static str) -> Option<&'static str> {
        match label {
            "ribbon.item.block_content" => Some(if self.block_visible {
                match self.block_model {
                    ContentModel::School => "ribbon.menu.content.school",
                    ContentModel::Technical => "ribbon.menu.content.technical",
                    ContentModel::Industrial => "ribbon.menu.content.industrial",
                }
            } else {
                "ribbon.menu.content.none"
            }),
            "ribbon.item.block_position" if self.block_visible => Some(match self.block_position {
                BlockPosition::TopRight => "ribbon.menu.position.top_right",
                BlockPosition::BottomRight => "ribbon.menu.position.bottom_right",
                BlockPosition::BottomFull => "ribbon.menu.position.bottom_full",
                BlockPosition::None => "ribbon.menu.content.none",
            }),
            _ => None,
        }
    }

    /// Takes the command the reader just chose, for the window to act on; the last command
    /// stays in `last` for the status bar.
    pub fn take(&mut self) -> Option<Command> {
        self.pending.take()
    }

    /// Draws the strip and the active tab; the pick lands in `last`.
    pub fn show(&mut self, ui: &mut egui::Ui, palette: &Palette, recents: &[Recent]) {
        let mut picked: Option<Command> = None;
        let bar = egui::Frame::default()
            .fill(palette.raised)
            .inner_margin(egui::Margin::symmetric(8, 3))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Punchi Fluid Lab")
                            .size(palette.text_label)
                            .strong()
                            .color(palette.ink),
                    );
                    ui.separator();
                    for (index, tab) in TABS.iter().enumerate() {
                        let selected = index == self.active;
                        let colour = if selected { palette.ink } else { palette.muted };
                        let text = egui::RichText::new(self.language.text(tab.name))
                            .size(palette.text_label)
                            .color(colour);
                        let response = ui.add(egui::Button::new(text).frame(false));
                        if selected {
                            let rect = response.rect;
                            ui.painter().line_segment(
                                [
                                    egui::pos2(rect.left(), rect.bottom()),
                                    egui::pos2(rect.right(), rect.bottom()),
                                ],
                                egui::Stroke::new(2.0_f32, palette.accent),
                            );
                        }
                        if response.clicked() {
                            self.active = index;
                        }
                    }
                    ui.separator();
                    transport::mode_label(
                        ui,
                        palette,
                        self.language,
                        self.running,
                        self.simulation,
                    );
                    let simulate_tab = TABS[self.active].name == "ribbon.tab.simulate";
                    if !simulate_tab && ui.available_width() > 260.0 {
                        transport::buttons(ui, palette, self.language, self, &mut picked);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(self.language.text("ribbon.license"))
                                .size(palette.text_label)
                                .color(palette.muted),
                        );
                        ui.label(
                            egui::RichText::new(self.language.text("ribbon.about"))
                                .size(palette.text_label)
                                .color(palette.muted),
                        );
                    });
                });
            });
        let bottom = bar.response.rect.bottom();
        ui.painter().line_segment(
            [
                egui::pos2(bar.response.rect.left(), bottom),
                egui::pos2(bar.response.rect.right(), bottom),
            ],
            egui::Stroke::new(1.0_f32, palette.line),
        );

        egui::Frame::default()
            .fill(palette.window)
            .inner_margin(egui::Margin::symmetric(8, 5))
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    let groups = TABS[self.active].groups;
                    let expanded = layout::fit_groups(ui, groups, palette, self.language);
                    for (index, group) in groups.iter().enumerate() {
                        if expanded[index] {
                            self.group(ui, group, palette, recents, &mut picked);
                        } else {
                            self.collapsed_group(ui, group, palette, recents, &mut picked);
                        }
                        ui.separator();
                    }
                });
            });

        if let Some(picked) = picked {
            self.run_command(picked);
        }
    }

    fn collapsed_group(
        &self,
        ui: &mut egui::Ui,
        group: &Group,
        palette: &Palette,
        recents: &[Recent],
        picked: &mut Option<Command>,
    ) {
        let name = self.language.text(group.name);
        ui.allocate_ui_with_layout(
            egui::vec2(
                (layout::text_width(ui, name, palette.text_label) + 44.0).max(96.0),
                78.0,
            ),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                let response = ui.menu_button(
                    egui::RichText::new(format!("{name}   "))
                        .size(palette.text_label)
                        .color(palette.ink),
                    |ui| {
                        for item in group.items {
                            match item {
                                Item::Large(_, label, command) | Item::Small(_, label, command) => {
                                    self.flyout(ui, label, *command, picked);
                                }
                                Item::Split(_, label, arrow_label, command, arrow) => {
                                    self.flyout(ui, label, Some(*command), picked);
                                    self.flyout(ui, arrow_label, Some(*arrow), picked);
                                }
                                Item::Menu(_, _, entries) => {
                                    for entry in *entries {
                                        self.flyout(ui, entry.label, entry.command, picked);
                                    }
                                }
                                Item::Gallery(_, _, entries) => {
                                    for entry in *entries {
                                        self.flyout(ui, entry.label, entry.command, picked);
                                    }
                                }
                                Item::Recents(..) => {
                                    recents::rows(ui, recents, picked, self.language);
                                }
                            }
                        }
                        if group.launcher {
                            self.flyout(ui, "ribbon.page_setup", Some(Command::PageSetup), picked);
                        }
                    },
                );
                arrow(ui, response.response.rect, palette.ink);
            },
        );
    }

    /// One line of a collapsed group's flyout: enabled only when the window can run it now.
    fn flyout(
        &self,
        ui: &mut egui::Ui,
        label: &'static str,
        command: Option<Command>,
        picked: &mut Option<Command>,
    ) {
        let enabled = command.is_some_and(|command| self.enabled(command));
        let label = self.language.text(label);
        if ui.add_enabled(enabled, egui::Button::new(label)).clicked() {
            if let Some(command) = command {
                *picked = Some(command);
            }
        }
    }

    fn group(
        &self,
        ui: &mut egui::Ui,
        group: &Group,
        palette: &Palette,
        recents: &[Recent],
        picked: &mut Option<Command>,
    ) {
        let name = self.language.text(group.name);
        ui.allocate_ui_with_layout(
            egui::vec2(
                layout::estimate(ui, group, palette, self.language).max(96.0),
                78.0,
            ),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.horizontal_top(|ui| {
                    for item in group.items {
                        match item {
                            Item::Large(id, label, command) => {
                                if self.large(ui, id, label, *command, palette) {
                                    if let Some(command) = command {
                                        *picked = Some(*command);
                                    }
                                }
                            }
                            Item::Small(id, label, command) => {
                                if self.small(ui, id, label, *command, palette) {
                                    if let Some(command) = command {
                                        *picked = Some(*command);
                                    }
                                }
                            }
                            Item::Split(id, label, arrow_label, command, arrow) => {
                                split::show(
                                    self,
                                    ui,
                                    id,
                                    label,
                                    arrow_label,
                                    *command,
                                    *arrow,
                                    palette,
                                    picked,
                                );
                            }
                            Item::Menu(id, label, entries) => {
                                self.menu(ui, id, label, entries, palette, picked);
                            }
                            Item::Gallery(id, label, entries) => {
                                self.gallery(ui, id, label, entries, palette, picked);
                            }
                            Item::Recents(id, label) => {
                                self.recents_menu(ui, id, label, recents, palette, picked);
                            }
                        }
                    }
                });
                ui.add_space(2.0);
                let title = if group.launcher {
                    format!("{name}…")
                } else {
                    name.to_owned()
                };
                let title_width = layout::text_width(ui, &title, palette.text_small);
                let width = ui.available_width();
                ui.horizontal(|ui| {
                    ui.add_space(((width - title_width) * 0.5).max(0.0));
                    if group.launcher {
                        if ui
                            .add(egui::Button::new(
                                egui::RichText::new(&title)
                                    .size(palette.text_small)
                                    .color(palette.muted),
                            ))
                            .on_hover_text(&title)
                            .clicked()
                        {
                            *picked = Some(Command::PageSetup);
                        }
                    } else {
                        ui.label(
                            egui::RichText::new(&title)
                                .size(palette.text_small)
                                .color(palette.muted),
                        );
                    }
                });
            },
        );
    }

    fn large(
        &self,
        ui: &mut egui::Ui,
        id: &str,
        label: &'static str,
        command: Option<Command>,
        palette: &Palette,
    ) -> bool {
        let enabled = command.is_some_and(|command| self.enabled(command));
        let key = label;
        let label = self.language.text(key);
        let ink = if enabled { palette.ink } else { palette.muted };
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(LARGE_WIDTH, LARGE_HEIGHT), egui::Sense::hover());
        let widget = ui.make_persistent_id(("ribbon.large", id, key));
        let response = ui.interact(rect, widget, egui::Sense::click());
        let layer = ui.layer_id();
        ui.memory_mut(|memory| memory.interested_in_focus(widget, layer));
        let selected = self.active_speed == key;
        if ui.is_rect_visible(rect) {
            if enabled && (response.hovered() || response.has_focus()) {
                ui.painter().rect_filled(rect, 3.0, palette.hover);
            }
            if selected {
                ui.painter().rect_filled(
                    egui::Rect::from_min_max(
                        egui::pos2(rect.left() + 8.0, rect.bottom() - 3.0),
                        egui::pos2(rect.right() - 8.0, rect.bottom()),
                    ),
                    1.0,
                    palette.accent,
                );
            }
            if enabled && response.has_focus() {
                ui.painter().rect_stroke(
                    rect,
                    3.0,
                    egui::Stroke::new(2.0_f32, palette.accent),
                    egui::StrokeKind::Inside,
                );
            }
            let icon_rect = egui::Rect::from_center_size(
                egui::pos2(rect.center().x, rect.top() + LARGE_ICON_TOP),
                egui::vec2(LARGE_ICON, LARGE_ICON),
            );
            if let Some(bytes) = self.icons.get(id) {
                Self::image_with(bytes, id, label, ink).paint_at(ui, icon_rect);
            }
            ui.painter().text(
                egui::pos2(rect.center().x, rect.top() + LARGE_LABEL_TOP),
                egui::Align2::CENTER_TOP,
                label,
                egui::FontId::proportional(LARGE_LABEL_SIZE),
                ink,
            );
        }
        response.widget_info(|| {
            if selected {
                egui::WidgetInfo::selected(egui::WidgetType::Button, enabled, true, label)
            } else {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, label)
            }
        });
        let activated = enabled
            && (response.clicked()
                || (response.has_focus()
                    && ui.input(|input| {
                        input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Space)
                    })));
        response.on_hover_text(hint(label, command));
        activated
    }

    fn small(
        &self,
        ui: &mut egui::Ui,
        id: &str,
        label: &'static str,
        command: Option<Command>,
        palette: &Palette,
    ) -> bool {
        let enabled = command.is_some_and(|command| self.enabled(command));
        let label = self.language.text(label);
        let ink = if enabled { palette.ink } else { palette.muted };
        let response = match self.icons.get(id) {
            Some(bytes) => {
                let image = Self::image_with(bytes, id, label, ink)
                    .fit_to_exact_size(egui::vec2(16.0, 16.0));
                let text = egui::RichText::new(label)
                    .size(palette.text_body)
                    .color(ink);
                ui.add_enabled(enabled, egui::Button::image_and_text(image, text))
            }
            None => ui.add_enabled(
                enabled,
                egui::Button::new(
                    egui::RichText::new(label)
                        .size(palette.text_body)
                        .color(ink),
                ),
            ),
        };
        response.on_hover_text(hint(label, command)).clicked()
    }

    fn menu(
        &self,
        ui: &mut egui::Ui,
        id: &str,
        label: &'static str,
        entries: &'static [MenuEntry],
        palette: &Palette,
        picked: &mut Option<Command>,
    ) {
        let current = self.block_current(label).map(|key| self.language.text(key));
        let label = self.language.text(label);
        let text = current.map_or_else(
            || format!("{label}   "),
            |current| format!("{label}\n{current}   "),
        );
        let text = egui::RichText::new(text)
            .size(palette.text_body)
            .color(palette.ink);
        let button = match self.icons.get(id) {
            Some(bytes) => egui::Button::image_and_text(
                Self::image(bytes, id, label, palette).fit_to_exact_size(egui::vec2(16.0, 16.0)),
                text,
            ),
            None => egui::Button::new(text),
        };
        let (response, _) = egui::containers::menu::MenuButton::from_button(button).ui(ui, |ui| {
            for entry in entries {
                let enabled = entry.command.is_some_and(|command| self.enabled(command));
                let selected = entry
                    .command
                    .is_some_and(|command| self.command_selected(command));
                let plain = self.language.text(entry.label);
                let marker = if selected { "•  " } else { "   " };
                let response =
                    ui.add_enabled(enabled, egui::Button::new(format!("{marker}{plain}")));
                if selected {
                    response.widget_info(|| {
                        egui::WidgetInfo::selected(egui::WidgetType::Button, enabled, true, plain)
                    });
                } else {
                    response.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, plain)
                    });
                }
                if response.clicked() {
                    if let Some(command) = entry.command {
                        *picked = Some(command);
                    }
                }
            }
        });
        arrow(ui, response.rect, palette.ink);
    }

    fn gallery(
        &self,
        ui: &mut egui::Ui,
        id: &str,
        label: &'static str,
        entries: &'static [GalleryEntry],
        palette: &Palette,
        picked: &mut Option<Command>,
    ) {
        let label = self.language.text(label);
        let text = egui::RichText::new(format!("{label}   "))
            .size(palette.text_body)
            .color(palette.ink);
        let button = match self.icons.get(id) {
            Some(bytes) => egui::Button::image_and_text(
                Self::image(bytes, id, label, palette).fit_to_exact_size(egui::vec2(16.0, 16.0)),
                text,
            ),
            None => egui::Button::new(text),
        };
        let (response, _) = egui::containers::menu::MenuButton::from_button(button).ui(ui, |ui| {
            egui::Grid::new(("gallery", id))
                .num_columns(2)
                .spacing([6.0, 6.0])
                .show(ui, |ui| {
                    for (index, entry) in entries.iter().enumerate() {
                        gallery::card(self, ui, entry, palette, picked, self.language);
                        if index % 2 == 1 {
                            ui.end_row();
                        }
                    }
                    let preset_matches = entries.iter().any(|entry| {
                        matches!(entry.command, Some(Command::PageMargin(preset))
                            if self.margins == preset.margins())
                    });
                    if !preset_matches {
                        gallery::card(
                            self,
                            ui,
                            &model::MARGINS_CUSTOM,
                            palette,
                            picked,
                            self.language,
                        );
                    }
                });
        });
        arrow(ui, response.rect, palette.ink);
    }

    fn recents_menu(
        &self,
        ui: &mut egui::Ui,
        id: &str,
        label: &'static str,
        recents: &[Recent],
        palette: &Palette,
        picked: &mut Option<Command>,
    ) {
        let label = self.language.text(label);
        let text = egui::RichText::new(format!("{label}   "))
            .size(palette.text_body)
            .color(palette.ink);
        let button = match self.icons.get(id) {
            Some(bytes) => egui::Button::image_and_text(
                Self::image(bytes, id, label, palette).fit_to_exact_size(egui::vec2(16.0, 16.0)),
                text,
            ),
            None => egui::Button::new(text),
        };
        let (response, _) = egui::containers::menu::MenuButton::from_button(button).ui(ui, |ui| {
            recents::rows(ui, recents, picked, self.language);
        });
        arrow(ui, response.rect, palette.ink);
    }

    fn image<'a>(
        bytes: &'static [u8],
        id: &str,
        label: &str,
        palette: &Palette,
    ) -> egui::Image<'a> {
        Self::image_with(bytes, id, label, palette.ink)
    }

    fn image_with<'a>(
        bytes: &'static [u8],
        id: &str,
        label: &str,
        ink: egui::Color32,
    ) -> egui::Image<'a> {
        egui::Image::from_bytes(format!("bytes://actions/{id}.svg"), bytes)
            .tint(ink)
            .alt_text(label)
    }
}

fn hint(label: &str, command: Option<Command>) -> String {
    match command.and_then(crate::shortcuts::label) {
        Some(key) => format!("{label} ({key})"),
        None => label.to_string(),
    }
}

fn arrow(ui: &egui::Ui, rect: egui::Rect, ink: egui::Color32) {
    let centre = egui::pos2(rect.right() - 9.0, rect.center().y);
    let points = vec![
        egui::pos2(centre.x - 3.0, centre.y - 2.0),
        egui::pos2(centre.x + 3.0, centre.y - 2.0),
        egui::pos2(centre.x, centre.y + 2.0),
    ];
    ui.painter()
        .add(egui::Shape::convex_polygon(points, ink, egui::Stroke::NONE));
}

#[cfg(test)]
#[cfg(test)]
mod tests;
