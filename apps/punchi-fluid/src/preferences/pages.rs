// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The settings pages: the nav, the appearance page and the panel page.

use eframe::egui;
use punchi_fluid_i18n::Language;

use crate::theme::{Appearance, Tokens};

use super::{Action, Chosen, Page, PageData, Slot};

/// The body: the page nav and the current page, inside the shared dialog scroll.
pub(super) fn preferences_body(ui: &mut egui::Ui, data: &PageData<'_>, chosen: &mut Chosen) {
    // A narrow dialog stacks the nav over the page; the wide one keeps the nav column.
    if ui.available_width() < 560.0 {
        ui.horizontal_wrapped(|ui| {
            for (candidate, label) in Page::ALL {
                if ui
                    .selectable_label(data.current_page(candidate), data.language.text(label))
                    .clicked()
                {
                    chosen.page = Some(candidate);
                }
            }
        });
        ui.separator();
        preferences_page(ui, data, chosen);
    } else {
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(data.tokens.unit * 10.5);
                ui.add_space(data.tokens.unit * 0.4);
                for (candidate, label) in Page::ALL {
                    if ui
                        .selectable_label(data.current_page(candidate), data.language.text(label))
                        .clicked()
                    {
                        chosen.page = Some(candidate);
                    }
                }
            });
            ui.separator();
            ui.vertical(|ui| {
                ui.set_width(ui.available_width());
                preferences_page(ui, data, chosen);
            });
        });
    }
}

fn preferences_page(ui: &mut egui::Ui, data: &PageData<'_>, chosen: &mut Chosen) {
    if data.current_page(Page::Appearance) {
        appearance_page(ui, data, chosen);
    } else {
        panels_page(ui, data, chosen);
    }
}

/// The language row: the visible language, with each option named in its own language.
fn language_row(ui: &mut egui::Ui, data: &PageData<'_>, chosen: &mut Chosen) {
    data.style.card(data.style.surface).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(data.language.text("language.label"))
                    .size(data.style.text_size)
                    .color(data.style.text),
            );
            let response = egui::ComboBox::from_id_salt("preferences.language")
                .selected_text(data.language.text(data.language.label_key()))
                .show_ui(ui, |ui| {
                    for language in Language::ALL {
                        if ui
                            .selectable_label(
                                data.language == language,
                                language.text(language.label_key()),
                            )
                            .clicked()
                        {
                            chosen.language = Some(language);
                        }
                    }
                });
            response.response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::ComboBox,
                    true,
                    data.language.text("language.label"),
                )
            });
        });
    });
}

/// The appearance page: the language, the three modes, the two system slots and the theme cards.
fn appearance_page(ui: &mut egui::Ui, data: &PageData<'_>, chosen: &mut Chosen) {
    language_row(ui, data, chosen);
    ui.add_space(8.0);
    data.style.card(data.style.surface).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(
            egui::RichText::new(data.language.text("preferences.page.appearance"))
                .size(data.style.text_size)
                .strong()
                .color(data.style.text),
        );
        ui.label(
            egui::RichText::new(data.language.text("preferences.appearance_body"))
                .size(data.style.text_size)
                .color(data.style.muted),
        );
        ui.add_space(data.tokens.unit * 0.6);
        mode_segments(ui, data, chosen);
        ui.add_space(data.tokens.unit * 0.7);
        if data.current == Appearance::System {
            slot_row(ui, data, chosen);
            ui.add_space(data.tokens.unit * 0.6);
        }
        theme_cards(ui, data, chosen);
        ui.add_space(data.tokens.unit * 0.6);
        if ui
            .button(data.language.text("preferences.reset_appearance"))
            .clicked()
        {
            chosen.action = Some(Action::ResetPage);
        }
    });
}

/// The three modes: the desktop palette, the system slots and a fixed theme.
fn mode_segments(ui: &mut egui::Ui, data: &PageData<'_>, chosen: &mut Chosen) {
    let fixed = match data.current {
        Appearance::Theme(id) => id,
        _ => data.day,
    };
    let modes = [
        Appearance::Desktop,
        Appearance::System,
        Appearance::Theme(fixed),
    ];
    ui.horizontal(|ui| {
        for mode in modes {
            let selected = match mode {
                Appearance::Desktop => data.current == Appearance::Desktop,
                Appearance::System => data.current == Appearance::System,
                Appearance::Theme(_) => matches!(data.current, Appearance::Theme(_)),
            };
            if ui
                .selectable_label(selected, data.language.text(mode.mode_label_key()))
                .clicked()
            {
                chosen.appearance = Some(mode);
            }
        }
    });
}

/// The two slots the system mode uses: the day theme and the night theme.
fn slot_row(ui: &mut egui::Ui, data: &PageData<'_>, chosen: &mut Chosen) {
    ui.label(
        egui::RichText::new(data.language.text("preferences.day_night"))
            .size(data.style.text_size * 0.86)
            .color(data.style.muted),
    );
    ui.horizontal(|ui| {
        for (slot, id, preview) in [
            (Slot::Day, data.day, &data.day_preview),
            (Slot::Night, data.night, &data.night_preview),
        ] {
            ui.vertical(|ui| {
                let aimed = data.aimed == slot;
                if preview_card(
                    ui,
                    data.tokens,
                    preview,
                    aimed,
                    data.language.text(slot.label_key()),
                    data.style.accent,
                ) {
                    chosen.slot = Some(slot);
                }
                ui.label(
                    egui::RichText::new(data.language.text(slot.label_key()))
                        .size(data.style.text_size)
                        .color(data.style.text),
                );
                ui.label(
                    egui::RichText::new(if aimed {
                        data.language.text("preferences.editing")
                    } else {
                        data.language.text(id.label_key())
                    })
                    .size(data.style.text_size * 0.86)
                    .color(if aimed {
                        data.style.accent
                    } else {
                        data.style.muted
                    }),
                );
                ui.add_space(data.tokens.unit * 0.8);
            });
        }
    });
}

/// The theme cards: a light and a dark engineering theme.
fn theme_cards(ui: &mut egui::Ui, data: &PageData<'_>, chosen: &mut Chosen) {
    ui.label(
        egui::RichText::new(data.language.text("preferences.themes"))
            .size(data.style.text_size * 0.86)
            .color(data.style.muted),
    );
    ui.horizontal_wrapped(|ui| {
        for (appearance, preview) in data.previews {
            let Appearance::Theme(id) = appearance else {
                continue;
            };
            let selected = match data.current {
                Appearance::Theme(current) => current == *id,
                Appearance::System => match data.aimed {
                    Slot::Day => data.day == *id,
                    Slot::Night => data.night == *id,
                },
                Appearance::Desktop => false,
            };
            ui.vertical(|ui| {
                if preview_card(
                    ui,
                    data.tokens,
                    preview,
                    selected,
                    data.language.text(id.label_key()),
                    data.style.accent,
                ) {
                    chosen.theme = Some(*id);
                }
                ui.label(
                    egui::RichText::new(data.language.text(id.label_key()))
                        .size(data.style.text_size)
                        .color(data.style.text),
                );
                ui.label(
                    egui::RichText::new(if selected {
                        data.language.text("preferences.in_use")
                    } else {
                        data.language.text("preferences.built_in")
                    })
                    .size(data.style.text_size * 0.86)
                    .color(if selected {
                        data.style.accent
                    } else {
                        data.style.muted
                    }),
                );
                ui.add_space(data.tokens.unit);
            });
        }
    });
}

/// The panel page: the auto-hide flag and the view reset, the same settings as **Ver**.
fn panels_page(ui: &mut egui::Ui, data: &PageData<'_>, chosen: &mut Chosen) {
    data.style.card(data.style.surface).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(
            egui::RichText::new(data.language.text("preferences.page.panels"))
                .size(data.style.text_size)
                .strong()
                .color(data.style.text),
        );
        ui.label(
            egui::RichText::new(data.language.text("preferences.panels_body"))
                .size(data.style.text_size)
                .color(data.style.muted),
        );
        ui.add_space(data.tokens.unit * 0.8);
        let mut on = data.autohide_on;
        if ui
            .checkbox(&mut on, data.language.text("preferences.autohide"))
            .changed()
        {
            chosen.action = Some(Action::ToggleAutohide);
        }
        ui.add_space(data.tokens.unit * 0.6);
        if ui
            .button(data.language.text("ribbon.item.reset_view"))
            .clicked()
        {
            chosen.action = Some(Action::ResetView);
        }
    });
}

/// A mini window drawn with the theme's own tokens: ribbon, panel, canvas, paper and accent.
fn preview_card(
    ui: &mut egui::Ui,
    tokens: &Tokens,
    preview: &Tokens,
    selected: bool,
    label: &str,
    ring: egui::Color32,
) -> bool {
    let size = egui::vec2(tokens.unit * 8.6, tokens.unit * 5.2);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Button, ui.is_enabled(), selected, label)
    });
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    painter.rect_filled(rect, tokens.radius_control, preview.background);
    let strip = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), tokens.unit * 0.7));
    painter.rect_filled(strip, tokens.radius_control, preview.surface_raised);
    let panel = egui::Rect::from_min_size(
        egui::pos2(rect.left(), strip.bottom()),
        egui::vec2(rect.width() * 0.28, rect.height() - strip.height()),
    );
    painter.rect_filled(panel, 0.0, preview.panel);
    let paper = egui::Rect::from_min_max(
        egui::pos2(
            rect.left() + rect.width() * 0.34,
            strip.bottom() + tokens.unit * 0.3,
        ),
        egui::pos2(
            rect.right() - rect.width() * 0.06,
            rect.bottom() - tokens.unit * 0.4,
        ),
    );
    painter.rect_filled(paper, 0.0, preview.paper);
    painter.circle_filled(
        egui::pos2(
            paper.right() - tokens.unit * 0.35,
            paper.top() + tokens.unit * 0.35,
        ),
        tokens.unit * 0.15,
        preview.accent,
    );
    if selected {
        painter.rect_stroke(
            rect,
            tokens.radius_control,
            egui::Stroke::new(2.0_f32, ring),
            egui::StrokeKind::Inside,
        );
    }
    response.clicked()
}
