// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The circuit dialogs: rename the active circuit and close it.
//!
//! Renaming changes the circuit's name; closing removes its placements and pipes as one
//! undo, and the last circuit is refused. Both are document changes, never view state.

use crate::Window;
use crate::dialog::{Action, ButtonRole, Dialog, DialogResult, DialogStyle};
use eframe::egui;

/// The dialog waiting for an answer, if any.
pub(crate) enum CircuitDialog {
    Rename { title: String, focus_name: bool },
    Close { title: String, focus_cancel: bool },
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
        self.circuit_dialog = Some(CircuitDialog::Rename {
            title,
            focus_name: true,
        });
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
        self.circuit_dialog = Some(CircuitDialog::Close {
            title,
            focus_cancel: true,
        });
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
            CircuitDialog::Rename { title, .. } => {
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

    /// Draws the dialog; the ×, `Esc` or **Cancelar** answers no.
    pub(crate) fn show_circuit_dialogs(&mut self, context: &egui::Context) {
        if self.close_question_open() {
            return;
        }
        let Some(dialog) = self.circuit_dialog.as_mut() else {
            return;
        };
        let style = DialogStyle::new(&self.theme.tokens);
        let unit = self.theme.tokens.unit;
        let language = self.language;
        let title = match dialog {
            CircuitDialog::Rename { .. } => language.text("circuit.rename_title"),
            CircuitDialog::Close { .. } => language.text("circuit.close_title"),
        };
        let dialog_ui = Dialog {
            id: egui::Id::new("window.circuit-dialog"),
            title,
            close_label: language.text("dialog.close"),
            preferred_width: (unit * 30.0).max(480.0),
            style,
        };
        let response = dialog_ui.show(
            context,
            dialog,
            |ui, dialog| match dialog {
                CircuitDialog::Rename { title, focus_name } => {
                    let label = ui.label(
                        egui::RichText::new(language.text("circuit.name"))
                            .color(style.text)
                            .strong(),
                    );
                    let response = ui
                        .add_sized(
                            egui::vec2(ui.available_width(), (unit * 2.0).max(34.0)),
                            egui::TextEdit::singleline(title).desired_width(ui.available_width()),
                        )
                        .labelled_by(label.id);
                    if std::mem::take(focus_name) {
                        response.request_focus();
                    }
                    if title.trim().is_empty() {
                        ui.label(
                            egui::RichText::new(language.text("circuit.empty_name"))
                                .color(style.error),
                        );
                    }
                }
                CircuitDialog::Close { title, .. } => {
                    ui.label(
                        egui::RichText::new(
                            language.format("circuit.close_question", &[("title", title.as_str())]),
                        )
                        .color(style.text)
                        .strong(),
                    );
                    ui.label(
                        egui::RichText::new(language.text("circuit.close_body")).color(style.muted),
                    );
                }
            },
            |dialog| {
                let (label, enabled, focus_cancel) = match dialog {
                    CircuitDialog::Rename { title, .. } => (
                        language.text("circuit.rename"),
                        !title.trim().is_empty(),
                        false,
                    ),
                    CircuitDialog::Close { focus_cancel, .. } => (
                        language.text("circuit.close"),
                        true,
                        std::mem::take(focus_cancel),
                    ),
                };
                vec![
                    Action {
                        label,
                        value: true,
                        role: ButtonRole::Primary,
                        enabled,
                        focus: false,
                    },
                    Action {
                        label: language.text("circuit.cancel"),
                        value: false,
                        role: ButtonRole::Secondary,
                        enabled: true,
                        focus: focus_cancel,
                    },
                ]
            },
        );
        match response.inner {
            Some(DialogResult::Action(apply)) => self.answer_circuit_dialog(apply),
            Some(DialogResult::Close) => self.answer_circuit_dialog(false),
            None if response.should_close() => self.answer_circuit_dialog(false),
            None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use eframe::egui;
    use egui_kittest::kittest::{NodeT, Queryable};
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
        if let Some(super::CircuitDialog::Rename { title, .. }) = &mut window.circuit_dialog {
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
            assert!(
                harness.query_by_label("Close").is_some(),
                "the English close button is offered"
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
                harness.query_by_label("Cerrar").is_some(),
                "the Spanish close button is offered"
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

    #[test]
    fn the_rename_dialog_focuses_the_name_field() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_circuit_rename();
        let expected = circuit_title(&window);
        {
            let harness = harness(&mut window);
            let field = harness.get_by_label("Circuit name");
            assert!(field.is_focused(), "the name field takes the focus");
            assert_eq!(field.value(), Some(expected));
            assert!(
                harness.get_by_label("Close").rect().size().min_elem() >= 32.0,
                "the close button has a full target"
            );
            assert!(
                harness.get_by_label("Rename").rect().height() >= 32.0,
                "the rename action has a full height"
            );
        }
    }

    #[test]
    fn a_valid_name_renames_from_the_dialog() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_circuit_rename();
        if let Some(super::CircuitDialog::Rename { title, .. }) = &mut window.circuit_dialog {
            *title = String::from("Mando");
        }
        {
            let mut harness = harness(&mut window);
            harness.get_by_label("Rename").hover();
            harness.run();
            harness.get_by_label("Rename").click();
            harness.run();
        }
        assert!(window.circuit_dialog.is_none());
        assert_eq!(circuit_title(&window), "Mando");
    }

    #[test]
    fn an_empty_name_keeps_the_rename_dialog_open() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        let expected = circuit_title(&window);
        window.open_circuit_rename();
        if let Some(super::CircuitDialog::Rename { title, .. }) = &mut window.circuit_dialog {
            title.clear();
        }
        {
            let mut harness = harness(&mut window);
            assert!(
                harness.query_by_label_contains("cannot be empty").is_some(),
                "the empty name is explained"
            );
            assert!(
                harness
                    .get_by_label("Rename")
                    .accesskit_node()
                    .is_disabled(),
                "the empty name cannot be applied"
            );
            harness.get_by_label("Rename").click();
            harness.run();
        }
        assert!(
            window.circuit_dialog.is_some(),
            "an empty name does not close the dialog"
        );
        assert_eq!(circuit_title(&window), expected);
    }

    #[test]
    fn the_rename_dialog_cancels_with_the_close_button() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        let expected = circuit_title(&window);
        window.open_circuit_rename();
        {
            let mut harness = harness(&mut window);
            harness.get_by_label("Close").click();
            harness.run();
        }
        assert!(window.circuit_dialog.is_none());
        assert_eq!(circuit_title(&window), expected);
    }

    #[test]
    fn the_close_confirmation_names_the_circuit_and_removes_it() {
        let mut window = window_with_two_circuits();
        window.open_circuit_close();
        {
            let mut harness = harness(&mut window);
            assert!(
                harness.query_by_label_contains("Close “").is_some(),
                "the question names the circuit"
            );
            harness
                .query_by_role_and_label(egui::accesskit::Role::Button, "Close circuit")
                .expect("the destructive action is offered")
                .hover();
            harness.run();
            harness
                .query_by_role_and_label(egui::accesskit::Role::Button, "Close circuit")
                .expect("the destructive action is offered")
                .click();
            harness.run();
        }
        assert!(window.circuit_dialog.is_none());
        assert_eq!(window.document.circuits().len(), 1);
    }

    #[test]
    fn the_close_confirmation_cancels_with_the_close_button() {
        let mut window = window_with_two_circuits();
        window.open_circuit_close();
        {
            let mut harness = harness(&mut window);
            harness.get_by_label("Close").hover();
            harness.run();
            harness.get_by_label("Close").click();
            harness.run();
        }
        assert!(window.circuit_dialog.is_none());
        assert_eq!(window.document.circuits().len(), 2);
    }

    #[test]
    fn the_close_confirmation_focuses_cancel() {
        let mut window = window_with_two_circuits();
        window.open_circuit_close();
        let harness = harness(&mut window);
        assert!(
            harness.get_by_label("Cancel").is_focused(),
            "Cancel takes the default focus, not the destructive action"
        );
    }

    #[test]
    fn the_circuit_dialog_fits_a_narrow_window() {
        let mut window = window_with_two_circuits();
        window.open_circuit_close();
        {
            let mut harness = harness(&mut window);
            harness.set_size(egui::vec2(420.0, 420.0));
            harness.run_steps(5);
            let close = harness.get_by_label("Close");
            assert!(close.rect().size().min_elem() >= 32.0);
            for (role, label) in [
                (egui::accesskit::Role::Button, "Close circuit"),
                (egui::accesskit::Role::Button, "Cancel"),
            ] {
                let rect = harness
                    .query_by_role_and_label(role, label)
                    .expect("the action is offered")
                    .rect();
                assert!(rect.height() >= 32.0, "{label} has a full height target");
                assert!(
                    rect.left() >= 0.0
                        && rect.right() <= 420.0
                        && rect.top() >= 0.0
                        && rect.bottom() <= 420.0,
                    "{label} fits the narrow window: {rect:?}"
                );
            }
            assert!(close.rect().right() <= 420.0);
        }
    }

    /// The active circuit's title, for a test that compares before and after.
    fn circuit_title(window: &Window) -> String {
        let id = window.document.active_circuit();
        window
            .document
            .circuits()
            .iter()
            .find(|circuit| circuit.id == id)
            .map(|circuit| circuit.title.clone())
            .expect("the active circuit")
    }
}
