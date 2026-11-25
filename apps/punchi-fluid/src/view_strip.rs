// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The floating view strip: the commands a reader uses while keeping their eyes on the sheet.
//!
//! It floats over the right edge of the drawing area, inside the channel the drawing area
//! reserves for it, so it takes no width from the paper. It is not a fourth region: the
//! drawing area, the canvas and the sheet keep their places. Every button dispatches the same
//! command as the ribbon or the keyboard, and the strip folds to one button, in the compact
//! mode by itself.

use eframe::egui;
use punchi_fluid_i18n::Language;

use crate::ribbon::Command;
use crate::theme::Tokens;

/// One drawing of the action family: its id and its embedded bytes.
type Drawing = (&'static str, &'static [u8]);

const ICON_HAND: Drawing = (
    "hand",
    include_bytes!("../../../assets/icons/actions/hand.svg"),
);
const ICON_FIT: Drawing = (
    "view-fit",
    include_bytes!("../../../assets/icons/actions/view-fit.svg"),
);
const ICON_ZOOM_IN: Drawing = (
    "zoom-in",
    include_bytes!("../../../assets/icons/actions/zoom-in.svg"),
);
const ICON_ZOOM_OUT: Drawing = (
    "zoom-out",
    include_bytes!("../../../assets/icons/actions/zoom-out.svg"),
);
const ICON_BLOCK: Drawing = (
    "document-properties",
    include_bytes!("../../../assets/icons/actions/document-properties.svg"),
);
const ICON_START: Drawing = (
    "media-playback-start",
    include_bytes!("../../../assets/icons/actions/media-playback-start.svg"),
);
const ICON_PAUSE: Drawing = (
    "media-playback-pause",
    include_bytes!("../../../assets/icons/actions/media-playback-pause.svg"),
);
const ICON_FOLD: Drawing = (
    "go-bottom",
    include_bytes!("../../../assets/icons/actions/go-bottom.svg"),
);
const ICON_UNFOLD: Drawing = (
    "go-top",
    include_bytes!("../../../assets/icons/actions/go-top.svg"),
);
const ICON_BACK: Drawing = (
    "edit-undo",
    include_bytes!("../../../assets/icons/actions/edit-undo.svg"),
);

/// The strip's view state: folded in the wide and medium modes, its own fold in the compact
/// one, the zoom panel, and the rectangle it painted last (for the layout tests).
#[derive(Default)]
pub(crate) struct ViewStrip {
    /// Folded in the wide and medium modes; remembered in the settings file.
    pub folded: bool,
    /// The zoom panel is open.
    pub panel_open: bool,
    /// The compact mode's own fold: it starts folded and the reader opens it when needed.
    pub compact_open: bool,
    /// The strip's screen rectangle of the last frame.
    pub rect: Option<egui::Rect>,
}

/// What the window passes to the strip each frame: the theme, the language, the room and the
/// states the buttons show. A bundle, so the component keeps its call short.
#[derive(Clone, Copy)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "the strip's buttons show independent states of the window, not a state machine"
)]
pub(crate) struct StripContext<'a> {
    pub tokens: &'a Tokens,
    pub language: Language,
    pub canvas: egui::Rect,
    pub compact: bool,
    pub hand: bool,
    pub block_visible: bool,
    pub running: bool,
    pub can_start: bool,
    pub can_go_back: bool,
}

impl ViewStrip {
    /// The fold the mode in force gives the strip.
    #[must_use]
    pub(crate) fn folded_in(&self, compact: bool) -> bool {
        if compact {
            !self.compact_open
        } else {
            self.folded
        }
    }

    /// Draws the strip and its panel and returns the command the reader chose, if any. The
    /// fold and the panel are the strip's own state; the command is the window's.
    pub(crate) fn show(
        &mut self,
        context: &egui::Context,
        state: &StripContext<'_>,
    ) -> Option<Command> {
        let StripContext {
            tokens,
            language,
            canvas,
            compact,
            ..
        } = *state;
        let unit = tokens.unit;
        let side = unit * 2.0;
        let width = side + unit * 0.5;
        let pad = (unit * 0.25).max(1.0);
        let folded = self.folded_in(compact);
        let rows = if folded { 1.0 } else { 6.0 };
        let height = rows * side + (rows - 1.0) * (unit * 0.125);
        let position = egui::pos2(
            canvas.right() - unit - width,
            canvas.center().y - height * 0.5,
        );
        let mut command = None;
        let area = egui::Area::new(egui::Id::new("view.strip"))
            .order(egui::Order::Middle)
            .fixed_pos(position)
            .show(context, |ui| {
                egui::Frame::default()
                    .fill(tokens.panel)
                    .stroke(egui::Stroke::new(1.0_f32, tokens.line))
                    .corner_radius(tokens.radius_control)
                    .shadow(strip_shadow(tokens))
                    .inner_margin(egui::Margin::same(pad_round(pad)))
                    .show(ui, |ui| {
                        ui.set_width(side);
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, unit * 0.125);
                        if folded {
                            let name = language.text("strip.unfold");
                            if button(ui, tokens, ICON_UNFOLD, name, false, true) {
                                self.set_folded(compact, false);
                            }
                            return;
                        }
                        command = self.controls(ui, state);
                    });
            });
        self.rect = Some(area.response.rect);
        if self.panel_open && !folded {
            if let Some(panel_command) = Self::show_panel(context, state, position) {
                command = Some(panel_command);
            }
        }
        command
    }

    /// The open strip's controls, in reference order: the hand, the fit, the zoom panel, the
    /// block, the transport and the fold.
    fn controls(&mut self, ui: &mut egui::Ui, state: &StripContext<'_>) -> Option<Command> {
        let StripContext {
            tokens,
            language,
            hand,
            block_visible,
            running,
            can_start,
            compact,
            ..
        } = *state;
        let mut command = None;
        if button(
            ui,
            tokens,
            ICON_HAND,
            language.text(if hand { "strip.pan_on" } else { "strip.pan" }),
            hand,
            true,
        ) {
            command = Some(Command::Hand);
        }
        if button(
            ui,
            tokens,
            ICON_FIT,
            language.text("strip.fit"),
            false,
            true,
        ) {
            command = Some(Command::FitSheet);
        }
        if button(
            ui,
            tokens,
            ICON_ZOOM_IN,
            language.text("strip.zoom"),
            self.panel_open,
            true,
        ) {
            self.panel_open = !self.panel_open;
        }
        separator(ui, tokens);
        if button(
            ui,
            tokens,
            ICON_BLOCK,
            language.text(if block_visible {
                "strip.block_hide"
            } else {
                "strip.block_show"
            }),
            !block_visible,
            true,
        ) {
            command = Some(Command::BlockVisible);
        }
        if button(
            ui,
            tokens,
            if running { ICON_PAUSE } else { ICON_START },
            language.text(if running {
                "ribbon.item.pause"
            } else {
                "ribbon.item.start"
            }),
            false,
            can_start || running,
        ) {
            command = Some(if running {
                Command::Pause
            } else {
                Command::Start
            });
        }
        separator(ui, tokens);
        if button(
            ui,
            tokens,
            ICON_FOLD,
            language.text("strip.fold"),
            false,
            true,
        ) {
            self.panel_open = false;
            self.set_folded(compact, true);
        }
        command
    }

    /// The zoom panel to the left of the strip: decrease, increase, a zone and the step back.
    fn show_panel(
        context: &egui::Context,
        state: &StripContext<'_>,
        strip_position: egui::Pos2,
    ) -> Option<Command> {
        let StripContext {
            tokens,
            language,
            can_go_back,
            ..
        } = *state;
        let unit = tokens.unit;
        let pad = (unit * 0.25).max(1.0);
        let width = unit * 10.5;
        let row_height = unit * 1.75;
        let position = egui::pos2(strip_position.x - unit * 0.375 - width, strip_position.y);
        let mut command = None;
        egui::Area::new(egui::Id::new("view.strip.panel"))
            .order(egui::Order::Middle)
            .fixed_pos(position)
            .show(context, |ui| {
                egui::Frame::default()
                    .fill(tokens.panel)
                    .stroke(egui::Stroke::new(1.0_f32, tokens.line))
                    .corner_radius(tokens.radius_control)
                    .shadow(strip_shadow(tokens))
                    .inner_margin(egui::Margin::same(pad_round(pad)))
                    .show(ui, |ui| {
                        ui.set_width(width);
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, unit * 0.125);
                        if panel_row(
                            ui,
                            tokens,
                            width,
                            row_height,
                            ICON_ZOOM_OUT,
                            language.text("ribbon.item.zoom_out"),
                            true,
                        ) {
                            command = Some(Command::ZoomOut);
                        }
                        if panel_row(
                            ui,
                            tokens,
                            width,
                            row_height,
                            ICON_ZOOM_IN,
                            language.text("ribbon.item.zoom_in"),
                            true,
                        ) {
                            command = Some(Command::ZoomIn);
                        }
                        if panel_row(
                            ui,
                            tokens,
                            width,
                            row_height,
                            ICON_FIT,
                            language.text("ribbon.item.zoom_window"),
                            true,
                        ) {
                            command = Some(Command::ZoomWindow);
                        }
                        if panel_row(
                            ui,
                            tokens,
                            width,
                            row_height,
                            ICON_BACK,
                            language.text("ribbon.item.view_back"),
                            can_go_back,
                        ) {
                            command = Some(Command::ViewBack);
                        }
                    });
            });
        command
    }

    /// Folds or unfolds, closing the panel on the way down.
    fn set_folded(&mut self, compact: bool, folded: bool) {
        if compact {
            self.compact_open = !folded;
        } else {
            self.folded = folded;
        }
        if folded {
            self.panel_open = false;
        }
    }
}

/// The floating card's shadow, from the theme: the same offset and blur the drawer and the
/// menus use, so the strip reads as one more surface and not as a painted border.
fn strip_shadow(tokens: &Tokens) -> egui::epaint::Shadow {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the radius token is 3-8 points, far inside the shadow's 0-255 blur"
    )]
    let blur = tokens.radius_control.round() as u8;
    egui::epaint::Shadow {
        offset: [1, 2],
        blur,
        spread: 0,
        color: tokens.shadow,
    }
}

/// A square painted button: hover, focus, the selected state and the accessible name.
fn button(
    ui: &mut egui::Ui,
    tokens: &Tokens,
    drawing: Drawing,
    name: &str,
    selected: bool,
    enabled: bool,
) -> bool {
    let side = tokens.unit * 2.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(side, side), egui::Sense::hover());
    let id = ui.make_persistent_id(("view.strip", name));
    let response = focusable(ui, rect, id);
    if !ui.is_rect_visible(rect) {
        return false;
    }
    paint_background(ui, tokens, rect, selected, enabled, &response);
    paint_icon(ui, tokens, drawing, rect, name, selected, enabled);
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Button, enabled, selected, name)
    });
    let fired = activated(ui, &response);
    response.on_hover_text(name);
    fired && enabled
}

/// One row of the zoom panel: the drawing, the label and the command.
fn panel_row(
    ui: &mut egui::Ui,
    tokens: &Tokens,
    width: f32,
    height: f32,
    drawing: Drawing,
    name: &str,
    enabled: bool,
) -> bool {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let id = ui.make_persistent_id(("view.strip.panel", name));
    let response = focusable(ui, rect, id);
    if !ui.is_rect_visible(rect) {
        return false;
    }
    paint_background(ui, tokens, rect, false, enabled, &response);
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + tokens.unit * 1.05, rect.center().y),
        egui::vec2(tokens.unit, tokens.unit),
    );
    paint_icon(ui, tokens, drawing, icon_rect, name, false, enabled);
    ui.painter().text(
        egui::pos2(icon_rect.right() + tokens.unit * 0.375, rect.center().y),
        egui::Align2::LEFT_CENTER,
        name,
        egui::FontId::proportional(tokens.text_small),
        if enabled {
            tokens.ink
        } else {
            tokens.ink_muted
        },
    );
    response
        .widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button, enabled, false, name));
    let fired = activated(ui, &response);
    response.on_hover_text(name);
    fired && enabled
}

/// The strip's background: the pressed, selected, hover and focus states over the panel.
fn paint_background(
    ui: &egui::Ui,
    tokens: &Tokens,
    rect: egui::Rect,
    selected: bool,
    enabled: bool,
    response: &egui::Response,
) {
    let fill = if !enabled {
        egui::Color32::TRANSPARENT
    } else if response.is_pointer_button_down_on() {
        tokens.surface_sunken
    } else if selected {
        tokens.accent_soft
    } else if response.hovered() || response.has_focus() {
        tokens.surface_raised
    } else {
        egui::Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, tokens.radius_control, fill);
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect,
            tokens.radius_control,
            egui::Stroke::new(1.0_f32, tokens.accent),
            egui::StrokeKind::Inside,
        );
    }
}

/// One drawing of the family, tinted with the theme's ink.
fn paint_icon(
    ui: &egui::Ui,
    tokens: &Tokens,
    drawing: Drawing,
    rect: egui::Rect,
    name: &str,
    selected: bool,
    enabled: bool,
) {
    let (id, bytes) = drawing;
    let ink = if !enabled {
        tokens.ink_dim
    } else if selected {
        tokens.accent
    } else {
        tokens.ink
    };
    egui::Image::from_bytes(format!("bytes://actions/{id}.svg"), bytes)
        .tint(ink)
        .alt_text(name)
        .paint_at(ui, rect);
}

/// A one-point rule between groups of controls.
fn separator(ui: &mut egui::Ui, tokens: &Tokens) {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(tokens.unit * 1.25, 1.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, tokens.line);
}

fn pad_round(pad: f32) -> i8 {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the strip's padding is 4 points, far inside the i8 margin"
    )]
    let pad = pad.round() as i8;
    pad
}

/// A painted control gets the keyboard focus and the accessibility layer's name.
fn focusable(ui: &mut egui::Ui, rect: egui::Rect, id: egui::Id) -> egui::Response {
    let response = ui.interact(rect, id, egui::Sense::click());
    let layer = ui.layer_id();
    ui.memory_mut(|memory| memory.interested_in_focus(id, layer));
    response
}

/// Whether the reader activated the control with the pointer or with `Enter`/`Space`.
fn activated(ui: &egui::Ui, response: &egui::Response) -> bool {
    response.clicked()
        || (response.has_focus()
            && ui.input(|input| {
                input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Space)
            }))
}
