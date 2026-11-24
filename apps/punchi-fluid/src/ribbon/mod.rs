// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The ribbon: a data table of tabs, groups and commands, a tab strip and the row of groups,
//! painted with the theme tokens and the embedded action icons.

use std::collections::HashMap;
use std::path::Path;

use eframe::egui;
use punchi_fluid_file::recent::Recent;
use punchi_fluid_i18n::Language;

use crate::theme::Tokens;

mod command;
mod enabled;
mod icons;
mod model;

pub(crate) use command::{Command, MarginPreset, Speed};
use model::{GalleryEntry, Group, Item, MenuEntry, Preview, TABS};

/// One action drawing by id, for the sheet menu, which is not a ribbon surface but uses the
/// same family.
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
            last: String::from("—"),
            savable: false,
            selected: 0,
            can_step: false,
            has_clipboard: false,
            can_undo: false,
            can_redo: false,
            can_go_back: false,
            can_reset_block: false,
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
            Command::Start => self.running = true,
            Command::Pause | Command::Stop => self.running = false,
            Command::Speed(speed) => self.active_speed = speed.label_key(),
            _ => {}
        }
        self.last = self.language.text(command.label_key()).to_string();
        self.pending = Some(command);
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
                    let expanded = fit_groups(ui, groups, palette, self.language);
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
                (text_width(ui, name, palette.text_label) + 44.0).max(96.0),
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
                                    Self::recents_rows(ui, recents, picked, self.language);
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
            egui::vec2(estimate(ui, group, palette, self.language).max(96.0), 78.0),
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
                let title_width = text_width(ui, name, palette.text_small);
                let width = ui.available_width();
                ui.horizontal(|ui| {
                    let launcher = if group.launcher { 30.0 } else { 0.0 };
                    ui.add_space(((width - title_width - launcher) * 0.5).max(0.0));
                    ui.label(
                        egui::RichText::new(name)
                            .size(palette.text_small)
                            .color(palette.muted),
                    );
                    if group.launcher
                        && ui
                            .add(egui::Button::new("…"))
                            .on_hover_text(self.language.text("ribbon.page_setup"))
                            .clicked()
                    {
                        *picked = Some(Command::PageSetup);
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
        let (rect, _) = ui.allocate_exact_size(egui::vec2(64.0, 58.0), egui::Sense::hover());
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
                egui::pos2(rect.center().x, rect.top() + 19.0),
                egui::vec2(32.0, 32.0),
            );
            if let Some(bytes) = self.icons.get(id) {
                Self::image_with(bytes, id, label, ink).paint_at(ui, icon_rect);
            }
            ui.painter().text(
                egui::pos2(rect.center().x, rect.top() + 37.0),
                egui::Align2::CENTER_TOP,
                label,
                egui::FontId::proportional(10.5),
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
            for entry in entries {
                let enabled = entry.command.is_some_and(|command| self.enabled(command));
                if ui
                    .add_enabled(enabled, egui::Button::new(self.language.text(entry.label)))
                    .clicked()
                {
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
                        self.card(ui, entry, palette, picked, self.language);
                        if index % 2 == 1 {
                            ui.end_row();
                        }
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
            Self::recents_rows(ui, recents, picked, self.language);
        });
        arrow(ui, response.rect, palette.ink);
    }

    /// The live documents, newest first; a file that is gone is named and disabled, never a
    /// command that would fail silently.
    fn recents_rows(
        ui: &mut egui::Ui,
        recents: &[Recent],
        picked: &mut Option<Command>,
        language: Language,
    ) {
        if recents.is_empty() {
            ui.add_enabled(
                false,
                egui::Button::new(language.text("ribbon.recents_empty")),
            );
            return;
        }
        for (index, recent) in recents.iter().enumerate() {
            let missing = !Path::new(&recent.path).is_file();
            let label = if missing {
                language.format("ribbon.recents_missing", &[("name", &recent.name)])
            } else {
                recent.name.clone()
            };
            if ui.add_enabled(!missing, egui::Button::new(label)).clicked() && !missing {
                *picked = Some(Command::OpenRecent(index));
            }
        }
    }

    fn card(
        &self,
        ui: &mut egui::Ui,
        entry: &GalleryEntry,
        palette: &Palette,
        picked: &mut Option<Command>,
        language: Language,
    ) {
        let enabled = entry.command.is_some_and(|command| self.enabled(command));
        let label = language.text(entry.label);
        let ink = if enabled { palette.ink } else { palette.muted };
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(132.0, 78.0), egui::Sense::click());
        if ui.is_rect_visible(rect) {
            let background = if enabled && response.hovered() {
                palette.hover
            } else {
                palette.raised
            };
            ui.painter().rect_filled(rect, 4.0, background);
            ui.painter().rect_stroke(
                rect,
                4.0,
                egui::Stroke::new(1.0_f32, palette.line),
                egui::StrokeKind::Inside,
            );
            let preview =
                egui::Rect::from_min_size(rect.min + egui::vec2(8.0, 6.0), egui::vec2(116.0, 42.0));
            match entry.preview {
                Preview::Margins(fraction) => {
                    let page =
                        egui::Rect::from_center_size(preview.center(), egui::vec2(58.0, 40.0));
                    ui.painter().rect_filled(page, 1.0, egui::Color32::WHITE);
                    ui.painter().rect_stroke(
                        page,
                        1.0,
                        egui::Stroke::new(1.0_f32, palette.ink),
                        egui::StrokeKind::Inside,
                    );
                    let margin = fraction * 18.0;
                    ui.painter().rect_stroke(
                        page.shrink(margin),
                        0.0,
                        egui::Stroke::new(1.0_f32, palette.muted),
                        egui::StrokeKind::Inside,
                    );
                }
                Preview::Pipe(icon) => {
                    if let Some(bytes) = self.icons.get(icon) {
                        Self::image(bytes, icon, label, palette).paint_at(
                            ui,
                            egui::Rect::from_center_size(preview.center(), egui::vec2(28.0, 28.0)),
                        );
                    }
                }
            }
            ui.painter().text(
                egui::pos2(rect.center().x, rect.max.y - 14.0),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(11.0),
                ink,
            );
        }
        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, label));
        if enabled && response.clicked() {
            if let Some(command) = entry.command {
                *picked = Some(command);
            }
            ui.close();
        }
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

/// Which groups keep their row at the current width: the last ones collapse until the
/// row fits, each to its own title button (P-028), never to a global menu.
fn fit_groups(ui: &egui::Ui, groups: &[Group], palette: &Palette, language: Language) -> Vec<bool> {
    let available = ui.available_width();
    let expanded: Vec<f32> = groups
        .iter()
        .map(|group| estimate(ui, group, palette, language))
        .collect();
    let collapsed: Vec<f32> = groups
        .iter()
        .map(|group| {
            (text_width(ui, language.text(group.name), palette.text_label) + 44.0).max(96.0)
        })
        .collect();
    let mut shown = vec![true; groups.len()];
    let total = |shown: &[bool]| {
        expanded
            .iter()
            .zip(collapsed.iter())
            .zip(shown.iter())
            .map(|((wide, narrow), shown)| if *shown { *wide } else { *narrow } + 8.0)
            .sum::<f32>()
    };
    for index in (0..groups.len()).rev() {
        if total(&shown) <= available {
            break;
        }
        shown[index] = false;
    }
    shown
}

fn hint(label: &str, command: Option<Command>) -> String {
    match command.and_then(crate::shortcuts::label) {
        Some(key) => format!("{label} ({key})"),
        None => label.to_string(),
    }
}

fn estimate(ui: &egui::Ui, group: &Group, palette: &Palette, language: Language) -> f32 {
    let mut width = 0.0_f32;
    for item in group.items {
        width += match item {
            Item::Large(..) => 64.0 + 6.0,
            Item::Small(_, label, _)
            | Item::Menu(_, label, _)
            | Item::Gallery(_, label, _)
            | Item::Recents(_, label) => {
                16.0 + 6.0 + text_width(ui, language.text(label), palette.text_body) + 22.0 + 6.0
            }
        };
    }
    width.max(96.0) + 12.0
}

fn text_width(ui: &egui::Ui, text: &str, size: f32) -> f32 {
    ui.fonts(|fonts| {
        fonts
            .layout_no_wrap(
                text.to_owned(),
                egui::FontId::proportional(size),
                egui::Color32::WHITE,
            )
            .size()
            .x
    })
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
