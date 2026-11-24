// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The title block editor: the stored fields, the model that chooses their defaults and the
//! position on the sheet.
//!
//! The panel edits a copy and only touches the document when the reader applies; the change
//! is one undo and the page the window draws follows it.

use eframe::egui;
use punchi_fluid_doc::title_block::{BlockPosition, ContentModel, TitleBlock, TitleField};
use punchi_fluid_i18n::Language;

use crate::Window;
use crate::theme::Tokens;

/// The editor's draft: a copy of the active circuit's block.
#[derive(Default)]
pub(crate) struct BlockEditor {
    pub(crate) open: bool,
    pub(crate) block: TitleBlock,
}

impl Window {
    /// **Editar datos…**: opens the editor with the active circuit's block.
    pub(crate) fn open_block_editor(&mut self) {
        let id = self.document.active_circuit();
        let Some(block) = self.document.block(id).cloned() else {
            return;
        };
        self.block_editor.block = block;
        self.block_editor.open = true;
    }

    /// Applies the draft as one undo and closes the editor.
    pub(crate) fn apply_block_editor(&mut self) {
        let id = self.document.active_circuit();
        if !self.document.set_block(id, self.block_editor.block.clone()) {
            self.notice = Some(self.language.text("notice.block_refused").to_owned());
            return;
        }
        self.refresh_page();
        self.block_editor.open = false;
        self.notice = Some(self.language.text("block.updated").to_owned());
    }

    /// Draws the editor; `Esc` or **Cerrar** discards the draft.
    pub(crate) fn show_block_editor(&mut self, context: &egui::Context) {
        if !self.block_editor.open {
            return;
        }
        if context.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.block_editor.open = false;
            return;
        }
        let tokens = self.theme.tokens.clone();
        let language = self.language;
        let action = block_editor_window(context, &tokens, &mut self.block_editor.block, language);
        match action {
            Some(Action::Apply) => self.apply_block_editor(),
            Some(Action::Close) => self.block_editor.open = false,
            None => {}
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Apply,
    Close,
}

fn block_editor_window(
    context: &egui::Context,
    tokens: &Tokens,
    block: &mut TitleBlock,
    language: Language,
) -> Option<Action> {
    let mut action = None;
    let screen = context.screen_rect();
    egui::Area::new(egui::Id::new("block-editor.blocker"))
        .order(egui::Order::Middle)
        .fixed_pos(screen.min)
        .show(context, |ui| {
            ui.allocate_rect(screen, egui::Sense::click_and_drag());
        });
    egui::Window::new(language.text("block.title"))
        .id(egui::Id::new("window.block-editor"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .fixed_size(egui::vec2(tokens.unit * 48.0, tokens.unit * 32.0))
        .show(context, |ui| {
            ui.vertical(|ui| {
                let height = (ui.available_height() - tokens.unit * 2.4).max(0.0);
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .max_height(height)
                    .show(ui, |ui| {
                        model_and_position(ui, tokens, block, language);
                        ui.add_space(tokens.unit * 0.4);
                        fields(ui, tokens, block, language);
                    });
            });
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button(language.text("dialog.apply")).clicked() {
                    action = Some(Action::Apply);
                }
                if ui.button(language.text("dialog.close")).clicked() {
                    action = Some(Action::Close);
                }
            });
        });
    action
}

fn model_and_position(
    ui: &mut egui::Ui,
    tokens: &Tokens,
    block: &mut TitleBlock,
    language: Language,
) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(language.text("block.model"))
                .size(tokens.text_label)
                .strong()
                .color(tokens.ink),
        );
        egui::ComboBox::from_id_salt("block-editor.model")
            .selected_text(model_label(language, block.model))
            .show_ui(ui, |ui| {
                for model in ContentModel::ALL {
                    if ui
                        .selectable_label(block.model == model, model_label(language, model))
                        .clicked()
                    {
                        block.model = model;
                        block.visible = model.defaults();
                    }
                }
            });
    });
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(language.text("block.position"))
                .size(tokens.text_label)
                .strong()
                .color(tokens.ink),
        );
        egui::ComboBox::from_id_salt("block-editor.position")
            .selected_text(position_label(language, block.position))
            .show_ui(ui, |ui| {
                for position in BlockPosition::ALL {
                    if ui
                        .selectable_label(
                            block.position == position,
                            position_label(language, position),
                        )
                        .clicked()
                    {
                        block.position = position;
                    }
                }
            });
    });
}

fn fields(ui: &mut egui::Ui, tokens: &Tokens, block: &mut TitleBlock, language: Language) {
    egui::Grid::new("block-editor.fields")
        .num_columns(3)
        .spacing([tokens.unit * 0.4, tokens.unit * 0.2])
        .show(ui, |ui| {
            for field in TitleField::ALL {
                ui.label(field_label(language, field));
                ui.add(
                    egui::TextEdit::singleline(field_text(block, field))
                        .desired_width(tokens.unit * 22.0),
                );
                let mut visible = block.visible.contains(field);
                if ui
                    .checkbox(&mut visible, language.text("block.show"))
                    .changed()
                {
                    block.visible.set(field, visible);
                }
                ui.end_row();
            }
        });
}

/// The stored text of `field`.
fn field_text(block: &mut TitleBlock, field: TitleField) -> &mut String {
    match field {
        TitleField::Owner => &mut block.owner,
        TitleField::Project => &mut block.project,
        TitleField::Equipment => &mut block.equipment,
        TitleField::Title => &mut block.title,
        TitleField::Number => &mut block.number,
        TitleField::Revision => &mut block.revision,
        TitleField::Date => &mut block.date,
        TitleField::Prepared => &mut block.prepared,
        TitleField::Checked => &mut block.checked,
        TitleField::Approved => &mut block.approved,
        TitleField::Scale => &mut block.scale,
        TitleField::Norms => &mut block.norms,
    }
}

fn field_label(language: Language, field: TitleField) -> &'static str {
    language.text(match field {
        TitleField::Owner => "block.field.owner",
        TitleField::Project => "block.field.project",
        TitleField::Equipment => "block.field.equipment",
        TitleField::Title => "block.field.title",
        TitleField::Number => "block.field.number",
        TitleField::Revision => "block.field.revision",
        TitleField::Date => "block.field.date",
        TitleField::Prepared => "block.field.prepared",
        TitleField::Checked => "block.field.checked",
        TitleField::Approved => "block.field.approved",
        TitleField::Scale => "block.field.scale",
        TitleField::Norms => "block.field.norms",
    })
}

fn model_label(language: Language, model: ContentModel) -> &'static str {
    language.text(match model {
        ContentModel::School => "block.content.school",
        ContentModel::Technical => "block.content.technical",
        ContentModel::Industrial => "block.content.industrial",
    })
}

fn position_label(language: Language, position: BlockPosition) -> &'static str {
    language.text(match position {
        BlockPosition::TopRight => "block.placement.top_right",
        BlockPosition::BottomRight => "block.placement.bottom_right",
        BlockPosition::BottomFull => "block.placement.bottom_full",
        BlockPosition::None => "block.placement.none",
    })
}

#[cfg(test)]
mod tests {
    use eframe::egui;
    use egui_kittest::kittest::Queryable;
    use punchi_fluid_doc::Document;
    use punchi_fluid_doc::title_block::{BlockPosition, TitleField};
    use punchi_fluid_i18n::Language;

    use crate::Window;
    use crate::theme::Theme;

    fn harness(window: &mut Window) -> egui_kittest::Harness<'_> {
        let mut harness =
            egui_kittest::Harness::new(move |context| window.show_block_editor(context));
        harness.set_size(egui::vec2(1366.0, 768.0));
        if crate::theme::install_font(&harness.ctx, None).is_err() {
            eprintln!("no system font: the block editor test needs one");
        }
        harness.run();
        harness
    }

    #[test]
    fn the_block_editor_follows_the_language() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_block_editor();
        {
            let harness = harness(&mut window);
            assert!(harness.query_all_by_label("Edit title block data").count() >= 1);
            assert!(harness.query_all_by_label("Owner").count() >= 1);
            assert!(
                harness
                    .query_by_role_and_label(egui::accesskit::Role::Button, "Apply")
                    .is_some()
            );
        }
        window.block_editor.open = false;
        window.language = Language::Spanish;
        window.open_block_editor();
        {
            let harness = harness(&mut window);
            assert!(
                harness
                    .query_all_by_label("Editar datos del cajetín")
                    .count()
                    >= 1
            );
            assert!(harness.query_all_by_label("Propietario").count() >= 1);
            assert!(
                harness
                    .query_by_role_and_label(egui::accesskit::Role::Button, "Aplicar")
                    .is_some()
            );
        }
    }

    #[test]
    fn the_editor_starts_with_the_circuit_block() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_block_editor();
        let id = window.document.active_circuit();
        let block = window.document.block(id).expect("the circuit has a block");
        assert_eq!(window.block_editor.block, *block);
    }

    #[test]
    fn a_field_edit_changes_the_block_as_one_undo() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_block_editor();
        window.block_editor.block.title = String::from("Circuito de prueba");
        window.apply_block_editor();
        let id = window.document.active_circuit();
        assert_eq!(
            window.document.block(id).map(|block| block.title.as_str()),
            Some("Circuito de prueba")
        );
        assert!(window.document.can_undo(), "the edit is one undo");
        assert!(!window.block_editor.open, "the editor closes on apply");
    }

    #[test]
    fn hiding_a_field_is_kept() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_block_editor();
        window.block_editor.block.visible.remove(TitleField::Norms);
        window.apply_block_editor();
        let id = window.document.active_circuit();
        assert!(
            window
                .document
                .block(id)
                .is_some_and(|block| !block.visible.contains(TitleField::Norms)),
            "the hidden field stays hidden"
        );
    }

    #[test]
    fn the_position_changes_from_the_editor() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_block_editor();
        window.block_editor.block.position = BlockPosition::TopRight;
        window.apply_block_editor();
        let id = window.document.active_circuit();
        assert_eq!(
            window.document.block(id).map(|block| block.position),
            Some(BlockPosition::TopRight)
        );
    }

    #[test]
    fn escape_closes_the_editor() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_block_editor();
        {
            let mut harness = harness(&mut window);
            harness.key_press(egui::Key::Escape);
            harness.run();
        }
        assert!(!window.block_editor.open);
    }
}
