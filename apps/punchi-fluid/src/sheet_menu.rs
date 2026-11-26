// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The sheet's context menu: the commands of a placed symbol, on the symbol itself.
//!
//! The menu is a second path to the commands the ribbon and the keyboard already own, not a
//! second implementation: every entry dispatches the same `Window` method. It also carries the
//! two windows the reference opens from here, **Configuración** and **Descripción**, which are
//! empty mockups today and say so; their content is a later capability.
//!
//! The state is view state: opening or closing the menu never touches the document, the undo
//! history or the file.

use eframe::egui;
use punchi_fluid_draw::Pt;
use punchi_fluid_i18n::Language;

use crate::Window;
use crate::view;

/// One entry of the sheet menu, in the reference's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MenuAction {
    Move,
    Rotate,
    MirrorHorizontal,
    MirrorVertical,
    Configure,
    Properties,
    Describe,
    Delete,
}

/// Label, icon id, the image URI the SVG loader keys on (it must end in `.svg`) and
/// whether a rule separates the entry from the one before it.
type Entry = (MenuAction, &'static str, &'static str, &'static str, bool);

const ENTRIES: &[Entry] = &[
    (
        MenuAction::Move,
        "menu.move",
        "hand",
        "bytes://actions/hand.svg",
        false,
    ),
    (
        MenuAction::Rotate,
        "menu.rotate",
        "object-rotate-right",
        "bytes://actions/object-rotate-right.svg",
        false,
    ),
    (
        MenuAction::MirrorHorizontal,
        "menu.mirror_horizontal",
        "object-flip-horizontal",
        "bytes://actions/object-flip-horizontal.svg",
        false,
    ),
    (
        MenuAction::MirrorVertical,
        "menu.mirror_vertical",
        "object-flip-vertical",
        "bytes://actions/object-flip-vertical.svg",
        false,
    ),
    (
        MenuAction::Configure,
        "menu.configure",
        "configure",
        "bytes://actions/configure.svg",
        false,
    ),
    (
        MenuAction::Properties,
        "menu.properties",
        "document-properties",
        "bytes://actions/document-properties.svg",
        false,
    ),
    (
        MenuAction::Describe,
        "menu.describe",
        "describe",
        "bytes://actions/describe.svg",
        false,
    ),
    (
        MenuAction::Delete,
        "menu.delete",
        "edit-delete",
        "bytes://actions/edit-delete.svg",
        true,
    ),
];

/// The menu's state: where it opened, the document revision it belongs to and whether the
/// first frame still has to take the focus.
#[derive(Debug)]
pub(crate) struct SheetMenu {
    open: bool,
    position: egui::Pos2,
    revision: u64,
    just_opened: bool,
}

impl Default for SheetMenu {
    fn default() -> Self {
        Self {
            open: false,
            position: egui::Pos2::ZERO,
            revision: 0,
            just_opened: false,
        }
    }
}

impl SheetMenu {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub(crate) fn is_open(&self) -> bool {
        self.open
    }

    /// Opens the menu at `position`, tied to the document revision that was read.
    pub(crate) fn open_at(&mut self, position: egui::Pos2, revision: u64) {
        self.open = true;
        self.position = position;
        self.revision = revision;
        self.just_opened = true;
    }

    /// Closes the menu and forgets the frame that opened it.
    pub(crate) fn close(&mut self) {
        self.open = false;
        self.just_opened = false;
    }

    /// The first frame after opening takes the focus for the keyboard path.
    fn take_just_opened(&mut self) -> bool {
        std::mem::take(&mut self.just_opened)
    }

    /// A document edit made elsewhere closes the menu with the state it belonged to.
    fn is_stale(&self, revision: u64) -> bool {
        self.open && self.revision != revision
    }
}

/// The two empty windows the reference opens from the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mockup {
    Configure,
    Describe,
}

impl Mockup {
    /// The title bar, exactly the word the owner asked for.
    fn title(self) -> &'static str {
        match self {
            Self::Configure => "dialog.configuration",
            Self::Describe => "dialog.description",
        }
    }

    /// A stable surface per window, so the two never share the position egui remembers.
    fn id(self) -> &'static str {
        match self {
            Self::Configure => "window.component-configuration",
            Self::Describe => "window.component-description",
        }
    }

    /// The mockup's size in theme units, close to the dialog it will become.
    fn size(self, unit: f32) -> egui::Vec2 {
        match self {
            Self::Configure => egui::vec2(unit * 34.0, unit * 27.0),
            Self::Describe => egui::vec2(unit * 30.0, unit * 18.0),
        }
    }
}

/// The text a card shows, resolved once when the window opens: the static image and the
/// definition's English name, summary and id.
#[derive(Debug, Clone)]
pub(crate) struct CardView {
    image: &'static crate::cards::Card,
    name: String,
    summary: String,
    id: String,
}

/// The open window, if any, the component it names and the card it may show.
#[derive(Debug, Default)]
pub(crate) struct Mockups {
    open: Option<Mockup>,
    subject: String,
    card: Option<CardView>,
}

impl Mockups {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub(crate) fn is_open(&self) -> bool {
        self.open.is_some()
    }

    fn open(&mut self, kind: Mockup, subject: String, card: Option<CardView>) {
        self.open = Some(kind);
        self.subject = subject;
        self.card = card;
    }

    fn close(&mut self) {
        self.open = None;
        self.subject.clear();
        self.card = None;
    }
}

impl Window {
    /// Whether a window owns the keyboard: the menu does not open behind one of them.
    fn a_dialog_is_open(&self) -> bool {
        self.dialog_open()
            || self.page_setup.open
            || self.block_editor.open
            || self.export_options.open
            || self.preferences.open
            || self.check_panel.open
            || self.circuit_dialog.is_some()
            || self.mockups.is_open()
    }

    /// The sheet gestures that belong to this module: the armed move owns the pointer, the
    /// keyboard opens or closes the menu, and the right button opens it over a symbol.
    /// Returns whether the gesture consumed the frame.
    pub(crate) fn handle_menu_gestures(
        &mut self,
        response: &egui::Response,
        placement: &view::Placement,
        page_min: egui::Vec2,
    ) -> bool {
        if self.moving.is_some() {
            self.handle_moving(response, placement, page_min);
            return true;
        }
        if self.open_menu_with_keyboard(&response.ctx, placement, page_min) {
            return true;
        }
        if response.secondary_clicked() {
            if let Some(position) = response.interact_pointer_pos() {
                let point = Pt::new(
                    page_min.x + (position.x - placement.origin.x) / placement.scale,
                    page_min.y + (position.y - placement.origin.y) / placement.scale,
                );
                self.open_menu_at_point(point, position);
            }
            return true;
        }
        false
    }

    /// The right button on the sheet: select the symbol under `point` and open its menu,
    /// or do nothing when the pointer is not on a symbol. A hose keeps its selection.
    pub(crate) fn open_menu_at_point(&mut self, point: Pt, position: egui::Pos2) {
        let Some(index) = self.hit_placement(point) else {
            self.menu.close();
            return;
        };
        if self.selected_pipe.is_some() {
            self.selected_pipe = None;
        }
        if !self.selected.contains(&index) {
            self.selected = vec![index];
        }
        self.sync_selection_notice();
        self.menu.open_at(position, self.document.revision());
    }

    /// `Shift+F10`: the keyboard path to the same menu, anchored to the selected symbol.
    /// Returns whether the sheet consumed the key.
    pub(crate) fn open_menu_with_keyboard(
        &mut self,
        context: &egui::Context,
        placement: &view::Placement,
        page_min: egui::Vec2,
    ) -> bool {
        let pressed =
            context.input(|input| input.modifiers.shift && input.key_pressed(egui::Key::F10));
        if !pressed || context.wants_keyboard_input() || self.a_dialog_is_open() {
            return false;
        }
        if self.menu.is_open() {
            self.menu.close();
            return true;
        }
        let Some(index) = self.selected.last().copied() else {
            return false;
        };
        let Some(placed) = self.document.placements().get(index) else {
            return false;
        };
        let rect = placed.rect();
        let point = egui::pos2(
            placement.origin.x + (rect.min.x - page_min.x) * placement.scale,
            placement.origin.y + (rect.min.y - page_min.y) * placement.scale,
        );
        let screen = context.screen_rect().shrink(8.0);
        let position = egui::pos2(
            point.x.clamp(screen.left(), screen.right()),
            point.y.clamp(screen.top(), screen.bottom()),
        );
        self.menu.open_at(position, self.document.revision());
        true
    }

    /// Draws and runs the menu. The entries are the reference's, in its order; every one
    /// calls the method the ribbon or the keyboard already uses.
    pub(crate) fn show_sheet_menu(&mut self, sheet: &egui::Response) {
        if !self.menu.is_open() {
            return;
        }
        if self.menu.is_stale(self.document.revision())
            || self.dialog_open()
            || self.mockups.is_open()
        {
            self.menu.close();
            return;
        }
        let tokens = self.theme.tokens.clone();
        let enabled: [bool; ENTRIES.len()] =
            std::array::from_fn(|index| self.menu_enabled(ENTRIES[index].0));
        let width = tokens.unit * 14.0;
        let row = tokens.unit * 1.7;
        let just_opened = self.menu.take_just_opened();
        let position = self.menu.position;
        let language = self.language;
        let mut open = self.menu.open;
        let mut action = None;
        egui::Popup::context_menu(sheet)
            .open_bool(&mut open)
            .at_position(position)
            .show(|ui| {
                ui.set_min_width(width);
                for (index, entry) in ENTRIES.iter().enumerate() {
                    if entry.4 {
                        ui.separator();
                    }
                    let response =
                        menu_row(ui, entry, enabled[index], &tokens, width, row, language);
                    if index == 0 && just_opened {
                        response.request_focus();
                    }
                    if enabled[index] && response.clicked() {
                        action = Some(entry.0);
                        ui.close();
                    }
                }
            });
        self.menu.open = open;
        if !open {
            self.menu.close();
        }
        if let Some(action) = action {
            self.menu.close();
            self.run_menu_action(action, sheet.id);
        }
    }

    /// Whether one entry may run now. The mockups are keyed to one symbol because they name
    /// it; the transforms and the removal work on the whole selection.
    fn menu_enabled(&self, action: MenuAction) -> bool {
        match action {
            MenuAction::Configure => self.selected.len() == 1 && !self.running,
            MenuAction::Describe => self.selected.len() == 1,
            _ => !self.selected.is_empty(),
        }
    }

    /// Runs the entry. The transform and removal methods are the ribbon's own, so the menu
    /// cannot drift from it.
    fn run_menu_action(&mut self, action: MenuAction, sheet_focus: egui::Id) {
        match action {
            MenuAction::Move => self.arm_move(),
            MenuAction::Rotate => self.rotate_selected(),
            MenuAction::MirrorHorizontal => self.reflect_selected(),
            MenuAction::MirrorVertical => self.reflect_vertical_selected(),
            MenuAction::Configure => {
                if let Some(index) = self.selected.last().copied()
                    && self.open_valve_editor(index, sheet_focus)
                {
                    return;
                }
                self.open_mockup(Mockup::Configure);
            }
            MenuAction::Properties => self.inspector.open(),
            MenuAction::Describe => self.open_mockup(Mockup::Describe),
            MenuAction::Delete => self.remove_selected(),
        }
    }

    /// Opens the window with the selected component's name and, for a description with a
    /// card, the card text resolved once.
    fn open_mockup(&mut self, kind: Mockup) {
        let catalog_id = self
            .selected
            .last()
            .and_then(|index| self.document.placements().get(*index))
            .map(|placed| placed.catalog_id.clone());
        let definition = catalog_id.as_deref().and_then(|id| self.catalog.get(id));
        let language = self.language;
        let subject = definition.map_or_else(
            || language.text("dialog.component").to_owned(),
            |definition| match language {
                punchi_fluid_i18n::Language::Spanish => definition
                    .name_es
                    .clone()
                    .unwrap_or_else(|| definition.name.clone()),
                punchi_fluid_i18n::Language::English => definition.name.clone(),
            },
        );
        let card = if kind == Mockup::Describe {
            catalog_id
                .as_deref()
                .and_then(crate::cards::image)
                .zip(definition)
                .map(|(image, definition)| CardView {
                    image,
                    name: match language {
                        punchi_fluid_i18n::Language::Spanish => definition
                            .name_es
                            .clone()
                            .unwrap_or_else(|| definition.name.clone()),
                        punchi_fluid_i18n::Language::English => definition.name.clone(),
                    },
                    summary: match language {
                        punchi_fluid_i18n::Language::Spanish => definition
                            .summary_es
                            .clone()
                            .or_else(|| definition.summary.clone()),
                        punchi_fluid_i18n::Language::English => definition.summary.clone(),
                    }
                    .unwrap_or_default(),
                    id: definition.id.clone(),
                })
        } else {
            None
        };
        self.mockups.open(kind, subject, card);
    }

    /// Draws the open window. **Descripción del componente** shows the card when the
    /// catalog id has one: the normalized image, the definition's English name, its summary
    /// and its id. Without a card it keeps the mockup note; **Configuración** has no card.
    pub(crate) fn show_mockups(&mut self, context: &egui::Context) {
        let Some(kind) = self.mockups.open else {
            return;
        };
        if context.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.mockups.close();
            return;
        }
        let tokens = self.theme.tokens.clone();
        let subject = &self.mockups.subject;
        let card = self.mockups.card.as_ref();
        let size = if card.is_some() {
            egui::vec2(tokens.unit * 35.0, tokens.unit * 29.0)
        } else {
            kind.size(tokens.unit)
        };
        let screen = context.screen_rect();
        egui::Area::new(egui::Id::new("component-mockup.blocker"))
            .order(egui::Order::Middle)
            .fixed_pos(screen.min)
            .show(context, |ui| {
                ui.allocate_rect(screen, egui::Sense::click_and_drag());
            });
        let mut close = false;
        let title = self.language.text(kind.title());
        egui::Window::new(title)
            .id(egui::Id::new(kind.id()))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .fixed_size(size)
            .show(context, |ui| {
                if let Some(card) = card {
                    card_body(ui, &tokens, card);
                } else {
                    ui.label(
                        egui::RichText::new(subject)
                            .size(tokens.text_label)
                            .strong()
                            .color(tokens.ink),
                    );
                    ui.add_space(tokens.unit * 0.5);
                    ui.label(
                        egui::RichText::new(self.language.text("dialog.mockup"))
                            .size(tokens.text_body)
                            .color(tokens.ink_muted),
                    );
                }
                ui.add_space(tokens.unit);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button(self.language.text("dialog.close")).clicked() {
                        close = true;
                    }
                });
            });
        if close {
            self.mockups.close();
        }
    }
}

/// The body of a description card: the normalized image, the definition's English name,
/// its summary and its id. Read-only; the window's own title and the **Cerrar** button stay
/// in `show_mockups`.
fn card_body(ui: &mut egui::Ui, tokens: &crate::theme::Tokens, card: &CardView) {
    let width = tokens.unit * 32.0;
    let height = width * 9.0 / 16.0;
    ui.add(
        egui::Image::from_bytes(card.image.uri, card.image.bytes)
            .fit_to_exact_size(egui::vec2(width, height))
            .alt_text(&card.name),
    );
    ui.add_space(tokens.unit * 0.5);
    ui.label(
        egui::RichText::new(&card.name)
            .size(tokens.text_label)
            .strong()
            .color(tokens.ink),
    );
    if !card.summary.is_empty() {
        ui.label(
            egui::RichText::new(&card.summary)
                .size(tokens.text_body)
                .color(tokens.ink_muted),
        );
    }
    ui.label(
        egui::RichText::new(&card.id)
            .size(tokens.text_small)
            .color(tokens.ink_muted),
    );
}

/// One row, left-aligned: the icon in a fixed column and the label beside it. The button's
/// own layout centres the pair, which makes a menu read as a stack of centred rows; the
/// reference, and a menu, align the icons in one column.
fn menu_row(
    ui: &mut egui::Ui,
    entry: &Entry,
    enabled: bool,
    tokens: &crate::theme::Tokens,
    width: f32,
    height: f32,
    language: Language,
) -> egui::Response {
    let label = language.text(entry.1);
    let reason = disabled_reason(entry.0).map(|key| language.text(key));
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());
    let mut response = response;
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, label));
    let hovered = enabled && response.hovered();
    let focused = enabled && response.has_focus();
    let fill = if !enabled {
        egui::Color32::TRANSPARENT
    } else if focused && !hovered {
        ui.visuals().selection.bg_fill
    } else {
        ui.style().interact(&response).weak_bg_fill
    };
    if fill.a() > 0 {
        ui.painter().rect_filled(rect, tokens.radius_small, fill);
    }
    let ink = if !enabled {
        tokens.ink_muted
    } else if focused && !hovered {
        tokens.on_accent
    } else {
        ui.style().interact(&response).text_color()
    };
    let icon = tokens.unit;
    let icon_rect = egui::Rect::from_min_size(
        egui::pos2(
            rect.left() + tokens.unit * 0.5,
            rect.center().y - icon * 0.5,
        ),
        egui::vec2(icon, icon),
    );
    if let Some(bytes) = crate::ribbon::icon_bytes(entry.2) {
        egui::Image::from_bytes(entry.3, bytes)
            .tint(ink)
            .alt_text(label)
            .fit_to_exact_size(egui::vec2(icon, icon))
            .paint_at(ui, icon_rect);
    }
    ui.painter().text(
        egui::pos2(icon_rect.right() + tokens.unit * 0.5, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(tokens.text_body),
        ink,
    );
    if !enabled {
        if let Some(reason) = reason {
            response = response.on_hover_text(reason);
        }
    }
    response
}

/// Why an entry is disabled, for the tooltip. A disabled control says why instead of
/// leaving the reader guessing.
fn disabled_reason(action: MenuAction) -> Option<&'static str> {
    match action {
        MenuAction::Configure | MenuAction::Describe => Some("menu.select_one"),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
