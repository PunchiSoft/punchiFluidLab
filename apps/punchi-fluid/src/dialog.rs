// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Shared dialog surfaces, sizing and actions.

use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui, vec2};

use crate::theme::Tokens;

/// Colours of the approved dialog design. A dialog inherits the window's light or dark mode.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DialogStyle {
    pub surface: Color32,
    pub band: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub border: Color32,
    pub accent: Color32,
    pub on_accent: Color32,
    pub selected: Color32,
    pub error: Color32,
    shadow: Color32,
    backdrop: Color32,
    pub text_size: f32,
    button_height: f32,
}

impl DialogStyle {
    pub(crate) fn new(tokens: &Tokens) -> Self {
        let palette = tokens.dialog_palette();
        Self {
            surface: palette.surface,
            band: palette.band,
            text: palette.text,
            muted: palette.muted,
            border: palette.border,
            accent: palette.accent,
            on_accent: palette.on_accent,
            selected: palette.selected,
            error: palette.error,
            shadow: palette.shadow,
            backdrop: palette.backdrop,
            text_size: tokens.text_body.max(14.0),
            button_height: (tokens.unit * 2.0).max(32.0),
        }
    }

    fn apply(self, ui: &mut Ui) {
        let style = ui.style_mut();
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::proportional(self.text_size),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::proportional(self.text_size),
        );
        style.text_styles.insert(
            egui::TextStyle::Small,
            egui::FontId::proportional(self.text_size * 0.86),
        );
        style.spacing.interact_size.y = self.text_size * 1.6;
        style.spacing.button_padding = vec2(10.0, 5.0);
        let visuals = &mut style.visuals;
        visuals.override_text_color = Some(self.text);
        visuals.extreme_bg_color = self.surface;
        visuals.faint_bg_color = self.band;
        visuals.selection.bg_fill = self.selected;
        visuals.selection.stroke = Stroke::new(1.0_f32, self.accent);
        for (widget, fill, border) in [
            (
                &mut visuals.widgets.noninteractive,
                self.surface,
                self.border,
            ),
            (&mut visuals.widgets.inactive, self.surface, self.border),
            (&mut visuals.widgets.hovered, self.selected, self.accent),
            (&mut visuals.widgets.active, self.selected, self.accent),
            (&mut visuals.widgets.open, self.selected, self.accent),
        ] {
            widget.bg_fill = fill;
            widget.weak_bg_fill = fill;
            widget.bg_stroke = Stroke::new(1.0_f32, border);
            widget.fg_stroke = Stroke::new(1.0_f32, self.text);
            widget.corner_radius = CornerRadius::same(6);
            widget.expansion = 0.0;
        }
    }

    pub(crate) fn card(self, fill: Color32) -> Frame {
        Frame::new()
            .fill(fill)
            .stroke(Stroke::new(1.0_f32, self.border))
            .corner_radius(8)
            .inner_margin(Margin::same(12))
    }

    fn button(
        self,
        ui: &mut Ui,
        label: &str,
        accessible: &str,
        role: ButtonRole,
        enabled: bool,
    ) -> egui::Response {
        let primary = matches!(role, ButtonRole::Primary) && enabled;
        let foreground = if primary {
            self.on_accent
        } else if enabled {
            self.text
        } else {
            self.muted
        };
        let mut button = egui::Button::new(RichText::new(label).color(foreground))
            .min_size(vec2(
                if matches!(role, ButtonRole::Close) {
                    self.button_height
                } else {
                    self.button_height * 2.0
                },
                self.button_height,
            ))
            .corner_radius(6)
            .wrap();
        if primary {
            button = button
                .fill(self.accent)
                .stroke(Stroke::new(1.0_f32, self.accent));
        } else if !enabled {
            button = button
                .fill(self.band)
                .stroke(Stroke::new(1.0_f32, self.border));
        }
        let response = ui.add_enabled(enabled, button);
        if label != accessible {
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, accessible)
            });
        }
        if response.has_focus() {
            ui.painter().rect_stroke(
                response.rect.expand(2.0),
                7,
                Stroke::new(1.5_f32, self.accent),
                egui::StrokeKind::Outside,
            );
        }
        response
    }
}

#[derive(Clone, Copy)]
pub(crate) enum ButtonRole {
    Primary,
    Secondary,
    Close,
}

pub(crate) struct Action<'a, T> {
    pub label: &'a str,
    pub value: T,
    pub role: ButtonRole,
    pub enabled: bool,
}

#[derive(Clone, Copy)]
pub(crate) enum DialogResult<T> {
    Action(T),
    Close,
}

/// A centred dialog with a separate header and footer. Only the body scrolls.
pub(crate) struct Dialog<'a> {
    pub id: egui::Id,
    pub title: &'a str,
    pub close_label: &'a str,
    pub preferred_width: f32,
    pub style: DialogStyle,
}

impl<'a> Dialog<'a> {
    pub(crate) fn show<S, A: Copy>(
        &self,
        context: &egui::Context,
        state: &mut S,
        body: impl FnOnce(&mut Ui, &mut S),
        actions: impl FnOnce(&S) -> Vec<Action<'a, A>>,
    ) -> egui::ModalResponse<Option<DialogResult<A>>> {
        let style = self.style;
        let size = context.screen_rect().size();
        let width = self.preferred_width.min((size.x - 28.0).max(1.0));
        let outer = Frame::new()
            .fill(style.surface)
            .stroke(Stroke::new(1.0_f32, style.border))
            .corner_radius(12)
            .shadow(egui::epaint::Shadow {
                offset: [0, 12],
                blur: 36,
                spread: 0,
                color: style.shadow,
            });
        egui::Modal::new(self.id)
            .frame(outer)
            .backdrop_color(style.backdrop)
            .show(context, |ui| {
                style.apply(ui);
                ui.set_width(width);
                ui.spacing_mut().item_spacing.y = 0.0;
                let close = self.header(ui, width);
                divider(ui, style.border);
                egui::ScrollArea::vertical()
                    .max_height((size.y - 170.0).max(1.0))
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        Frame::new()
                            .inner_margin(Margin::symmetric(18, 16))
                            .show(ui, |ui| {
                                ui.set_width((width - 36.0).max(1.0));
                                ui.spacing_mut().item_spacing = vec2(10.0, 8.0);
                                body(ui, state);
                            });
                    });
                divider(ui, style.border);
                let answer = self.footer(ui, width, &actions(state));
                if close {
                    Some(DialogResult::Close)
                } else {
                    answer.map(DialogResult::Action)
                }
            })
    }

    fn header(&self, ui: &mut Ui, width: f32) -> bool {
        let style = self.style;
        Frame::new()
            .fill(style.band)
            .corner_radius(CornerRadius {
                nw: 11,
                ne: 11,
                sw: 0,
                se: 0,
            })
            .inner_margin(Margin::symmetric(18, 12))
            .show(ui, |ui| {
                let inner_width = (width - 36.0).max(1.0);
                let (row, _) = ui.allocate_exact_size(
                    vec2(inner_width, style.button_height),
                    egui::Sense::hover(),
                );
                let mut close_rect = row;
                close_rect.set_left((row.right() - style.button_height).max(row.left()));
                let mut title_rect = row;
                title_rect.set_right((close_rect.left() - 12.0).max(row.left()));
                let layout = egui::Layout::left_to_right(egui::Align::Center);
                let mut title_ui =
                    ui.new_child(egui::UiBuilder::new().max_rect(title_rect).layout(layout));
                title_ui.add(
                    egui::Label::new(
                        RichText::new(self.title)
                            .color(style.text)
                            .size(style.text_size * 1.3)
                            .strong(),
                    )
                    .truncate(),
                );
                let mut close_ui =
                    ui.new_child(egui::UiBuilder::new().max_rect(close_rect).layout(layout));
                style
                    .button(
                        &mut close_ui,
                        "×",
                        self.close_label,
                        ButtonRole::Close,
                        true,
                    )
                    .clicked()
            })
            .inner
    }

    fn footer<A: Copy>(&self, ui: &mut Ui, width: f32, actions: &[Action<'_, A>]) -> Option<A> {
        Frame::new()
            .fill(self.style.band)
            .corner_radius(CornerRadius {
                nw: 0,
                ne: 0,
                sw: 11,
                se: 11,
            })
            .inner_margin(Margin::symmetric(18, 12))
            .show(ui, |ui| {
                ui.set_width((width - 36.0).max(1.0));
                ui.spacing_mut().item_spacing = vec2(10.0, 8.0);
                let mut answer = None;
                ui.allocate_ui_with_layout(
                    vec2(ui.available_width(), self.style.button_height),
                    egui::Layout::right_to_left(egui::Align::Min).with_main_wrap(true),
                    |ui| {
                        for action in actions.iter().rev() {
                            let response = self.style.button(
                                ui,
                                action.label,
                                action.label,
                                action.role,
                                action.enabled,
                            );
                            if response.clicked() {
                                answer = Some(action.value);
                            }
                        }
                    },
                );
                answer
            })
            .inner
    }
}

fn divider(ui: &mut Ui, color: Color32) {
    let (line, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().rect_filled(line, 0, color);
}
