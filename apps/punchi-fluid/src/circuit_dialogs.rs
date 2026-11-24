// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The circuit dialogs: rename the active circuit and close it.
//!
//! Renaming changes the circuit's name; closing removes its placements and pipes as one
//! undo, and the last circuit is refused. Both are document changes, never view state.

use crate::Window;
use crate::theme::Tokens;
use eframe::egui;
use punchi_fluid_i18n::Language;

/// The dialog waiting for an answer, if any.
pub(crate) enum CircuitDialog {
    Rename { title: String },
    Close { title: String },
}

impl Window {
    /// **Circuito ▸ Renombrar**: the dialog starts with the circuit's name.
    pub(crate) fn open_circuit_rename(&mut self) {
        let id = self.document.active_circuit();
        let Some(title) = self
            .document
            .circuits()
            .iter()
            .find(|circuit| circuit.id == id)
            .map(|circuit| circuit.title.clone())
        else {
            return;
        };
        self.circuit_dialog = Some(CircuitDialog::Rename { title });
    }

    /// **Circuito ▸ Cerrar**: the confirmation; the last circuit is refused with a notice.
    pub(crate) fn open_circuit_close(&mut self) {
        let id = self.document.active_circuit();
        let Some(title) = self
            .document
            .circuits()
            .iter()
            .find(|circuit| circuit.id == id)
            .map(|circuit| circuit.title.clone())
        else {
            return;
        };
        if self.document.circuits().len() <= 1 {
            self.notice = Some(self.language.text("circuit.last_circuit").to_owned());
            return;
        }
        self.circuit_dialog = Some(CircuitDialog::Close { title });
    }

    /// Applies the answer: one undo, and the page follows the circuit that remains.
    pub(crate) fn answer_circuit_dialog(&mut self, apply: bool) {
        let Some(dialog) = self.circuit_dialog.take() else {
            return;
        };
        if !apply {
            return;
        }
        let id = self.document.active_circuit();
        match dialog {
            CircuitDialog::Rename { title } => {
                let title = title.trim();
                if title.is_empty() {
                    self.notice = Some(self.language.text("circuit.empty_name").to_owned());
                    return;
                }
                if self.document.rename_circuit(id, title) {
                    self.notice = Some(self.language.text("circuit.renamed").to_owned());
                }
            }
            CircuitDialog::Close { .. } => {
                if !self.document.remove_circuit(id) {
                    self.notice = Some(self.language.text("circuit.last_circuit").to_owned());
                    return;
                }
                let (page, block) = crate::files::page_and_block(&self.document);
                self.page = page;
                self.block = block;
                self.selected.clear();
                self.selected_pipe = None;
                self.model = None;
                self.model_revision = 0;
                self.notice = Some(self.language.text("circuit.closed").to_owned());
            }
        }
    }

    /// Draws the dialog; `Esc` or **Cancelar** answers no.
    pub(crate) fn show_circuit_dialogs(&mut self, context: &egui::Context) {
        let Some(dialog) = self.circuit_dialog.as_mut() else {
            return;
        };
        if context.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.circuit_dialog = None;
            return;
        }
        let tokens = self.theme.tokens.clone();
        let language = self.language;
        let apply = circuit_dialog_window(context, &tokens, dialog, language);
        if let Some(apply) = apply {
            self.answer_circuit_dialog(apply);
        }
    }
}

/// `true` when the reader accepted, `false` when the dialog closed without an answer.
fn circuit_dialog_window(
    context: &egui::Context,
    tokens: &Tokens,
    dialog: &mut CircuitDialog,
    language: Language,
) -> Option<bool> {
    let mut answer = None;
    let screen = context.screen_rect();
    egui::Area::new(egui::Id::new("circuit-dialog.blocker"))
        .order(egui::Order::Middle)
        .fixed_pos(screen.min)
        .show(context, |ui| {
            ui.allocate_rect(screen, egui::Sense::click_and_drag());
        });
    let (title, apply_label) = match dialog {
        CircuitDialog::Rename { .. } => ("circuit.rename_title", "circuit.rename"),
        CircuitDialog::Close { .. } => ("circuit.close_title", "circuit.close"),
    };
    let (title, apply_label) = (language.text(title), language.text(apply_label));
    egui::Window::new(title)
        .id(egui::Id::new("window.circuit-dialog"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .fixed_size(egui::vec2(tokens.unit * 30.0, tokens.unit * 12.0))
        .show(context, |ui| {
            match dialog {
                CircuitDialog::Rename { title } => {
                    ui.label(
                        egui::RichText::new(language.text("circuit.name"))
                            .size(tokens.text_label)
                            .color(tokens.ink),
                    );
                    ui.add(egui::TextEdit::singleline(title).desired_width(ui.available_width()));
                }
                CircuitDialog::Close { title } => {
                    ui.label(
                        egui::RichText::new(
                            language.format("circuit.close_question", &[("title", title)]),
                        )
                        .size(tokens.text_label)
                        .color(tokens.ink),
                    );
                    ui.label(
                        egui::RichText::new(language.text("circuit.close_body"))
                            .size(tokens.text_body)
                            .color(tokens.ink_muted),
                    );
                }
            }
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button(apply_label).clicked() {
                    answer = Some(true);
                }
                if ui.button(language.text("circuit.cancel")).clicked() {
                    answer = Some(false);
                }
            });
        });
    answer
}

#[cfg(test)]
mod tests {
    use eframe::egui;
    use egui_kittest::kittest::Queryable;
    use punchi_fluid_doc::Document;

    use crate::Window;
    use crate::theme::Theme;

    fn harness(window: &mut Window) -> egui_kittest::Harness<'_> {
        let mut harness =
            egui_kittest::Harness::new(move |context| window.show_circuit_dialogs(context));
        harness.set_size(egui::vec2(1366.0, 768.0));
        if crate::theme::install_font(&harness.ctx, None).is_err() {
            eprintln!("no system font: the circuit dialog test needs one");
        }
        harness.run();
        harness
    }

    fn window_with_two_circuits() -> Window {
        let mut document = Document::new();
        document.add_circuit("Circuito 2");
        Window::new(Theme::fallback(), document, None, None)
    }

    #[test]
    fn renaming_the_circuit_is_one_undo() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_circuit_rename();
        if let Some(super::CircuitDialog::Rename { title }) = &mut window.circuit_dialog {
            *title = String::from("Mando");
        }
        window.answer_circuit_dialog(true);
        let id = window.document.active_circuit();
        assert_eq!(
            window
                .document
                .circuits()
                .iter()
                .find(|circuit| circuit.id == id)
                .map(|circuit| circuit.title.as_str()),
            Some("Mando")
        );
        assert!(window.document.undo(), "the rename undoes");
        let id = window.document.active_circuit();
        assert_ne!(
            window
                .document
                .circuits()
                .iter()
                .find(|circuit| circuit.id == id)
                .map(|circuit| circuit.title.as_str()),
            Some("Mando")
        );
    }

    #[test]
    fn closing_the_circuit_is_one_undo() {
        let mut window = window_with_two_circuits();
        let id = window.document.active_circuit();
        window.open_circuit_close();
        window.answer_circuit_dialog(true);
        assert_eq!(window.document.circuits().len(), 1);
        assert!(window.document.undo(), "the close undoes");
        assert!(
            window
                .document
                .circuits()
                .iter()
                .any(|circuit| circuit.id == id)
        );
    }

    #[test]
    fn the_circuit_dialog_follows_the_language() {
        let mut window = window_with_two_circuits();
        window.open_circuit_rename();
        {
            let harness = harness(&mut window);
            assert!(
                harness
                    .query_by_role_and_label(egui::accesskit::Role::Button, "Rename")
                    .is_some(),
                "the English confirmation is offered"
            );
            assert!(
                harness
                    .query_by_role_and_label(egui::accesskit::Role::Button, "Cancel")
                    .is_some(),
                "the English cancel is offered"
            );
        }
        window.circuit_dialog = None;
        window.language = punchi_fluid_i18n::Language::Spanish;
        window.open_circuit_close();
        {
            let harness = harness(&mut window);
            assert!(
                harness
                    .query_by_role_and_label(egui::accesskit::Role::Button, "Cerrar circuito")
                    .is_some(),
                "the Spanish confirmation is offered"
            );
            assert!(
                harness
                    .query_by_role_and_label(egui::accesskit::Role::Button, "Cancelar")
                    .is_some(),
                "the Spanish cancel is offered"
            );
            assert!(
                harness.query_all_by_label_contains("¿Cerrar").count() >= 1,
                "the Spanish question is offered"
            );
        }
    }

    #[test]
    fn the_last_circuit_cannot_be_closed() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_circuit_close();
        assert!(window.circuit_dialog.is_none(), "the dialog does not open");
        assert!(
            window
                .notice
                .as_deref()
                .is_some_and(|notice| notice.contains("last circuit"))
        );
        window.answer_circuit_dialog(true);
        assert_eq!(window.document.circuits().len(), 1);
    }

    #[test]
    fn escape_cancels_the_rename() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_circuit_rename();
        {
            let mut harness = harness(&mut window);
            harness.key_press(egui::Key::Escape);
            harness.run();
        }
        assert!(window.circuit_dialog.is_none());
    }
}
