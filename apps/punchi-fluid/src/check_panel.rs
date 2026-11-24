// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The faults panel: the design check of the circuit in view, reached through the session.
//!
//! The rule lives in `punchi-fluid-check`; the session forwards it (ADR 0044), so the window
//! only paints the findings and points at them. Selecting one selects the component or the
//! pipe it names, which the sheet already highlights.

use eframe::egui;
use punchi_fluid_draw::surface::{Pt, Rect};
use punchi_fluid_i18n::Language;
use punchi_fluid_session::check::{Finding, check_circuit};

use crate::Window;
use crate::theme::Tokens;

/// The panel's state: the findings of the last run and the one the reader selected.
#[derive(Default)]
pub(crate) struct CheckPanel {
    pub(crate) open: bool,
    findings: Vec<Finding>,
    focused: Option<usize>,
}

/// The drawing space of `document`: its paper minus the margins, in millimetres, the frame
/// the check measures against (the same area the headless check uses).
fn sheet_area(document: &punchi_fluid_doc::Document) -> Rect {
    let paper = document.sheet();
    let (width, height) = paper.size_mm();
    let margins = paper.margins;
    Rect::from_min_size(
        Pt::new(margins.left, margins.top),
        (width - margins.left - margins.right).max(1.0),
        (height - margins.top - margins.bottom).max(1.0),
    )
}

impl Window {
    /// **Comprobar**: runs the check on the circuit in view and opens the list.
    pub(crate) fn open_check(&mut self) {
        let area = sheet_area(&self.document);
        let circuit = self.document.active_circuit();
        self.check_panel.findings = check_circuit(&self.document, &self.catalog, area, circuit);
        self.check_panel.focused = None;
        self.check_panel.open = true;
    }

    /// Selects the fault's component or pipe, so the sheet marks what the sentence describes.
    pub(crate) fn select_finding(&mut self, index: usize) {
        let Some(finding) = self.check_panel.findings.get(index).copied() else {
            return;
        };
        if let Some(component) = finding.component {
            self.selected = vec![component];
            self.selected_pipe = None;
        } else if let Some(pipe) = finding.pipe {
            self.selected.clear();
            self.selected_pipe = Some(pipe);
        }
        self.check_panel.focused = Some(index);
    }

    /// Draws the panel; `Esc` or **Cerrar** closes it. **Recomprobar** runs the check again.
    pub(crate) fn show_check_panel(&mut self, context: &egui::Context) {
        if !self.check_panel.open {
            return;
        }
        if context.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.check_panel.open = false;
            return;
        }
        let tokens = self.theme.tokens.clone();
        let language = self.language;
        match check_panel_window(context, &tokens, &self.check_panel, language) {
            Some(Action::Recheck) => self.open_check(),
            Some(Action::Select(index)) => self.select_finding(index),
            Some(Action::Close) => self.check_panel.open = false,
            None => {}
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Recheck,
    Select(usize),
    Close,
}

fn check_panel_window(
    context: &egui::Context,
    tokens: &Tokens,
    panel: &CheckPanel,
    language: Language,
) -> Option<Action> {
    let mut action = None;
    let screen = context.screen_rect();
    egui::Area::new(egui::Id::new("check-panel.blocker"))
        .order(egui::Order::Middle)
        .fixed_pos(screen.min)
        .show(context, |ui| {
            ui.allocate_rect(screen, egui::Sense::click_and_drag());
        });
    egui::Window::new(language.text("check.title"))
        .id(egui::Id::new("window.check-panel"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .fixed_size(egui::vec2(tokens.unit * 40.0, tokens.unit * 28.0))
        .show(context, |ui| {
            if panel.findings.is_empty() {
                ui.label(
                    egui::RichText::new(language.text("check.clean"))
                        .size(tokens.text_body)
                        .color(tokens.ink),
                );
            } else {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .max_height(ui.available_height() - tokens.unit * 2.6)
                    .show(ui, |ui| {
                        for (index, finding) in panel.findings.iter().enumerate() {
                            if ui
                                .selectable_label(panel.focused == Some(index), finding.describe())
                                .clicked()
                            {
                                action = Some(Action::Select(index));
                            }
                        }
                    });
            }
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button(language.text("check.recheck")).clicked() {
                    action = Some(Action::Recheck);
                }
                if ui.button(language.text("dialog.close")).clicked() {
                    action = Some(Action::Close);
                }
            });
        });
    action
}

#[cfg(test)]
mod tests {
    use eframe::egui;
    use egui_kittest::kittest::Queryable;
    use punchi_fluid_doc::{Document, Placement};
    use punchi_fluid_draw::surface::Pt;
    use punchi_fluid_i18n::Language;

    use crate::Window;
    use crate::theme::Theme;

    fn window_with_a_valve() -> Window {
        let mut document = Document::new();
        document.place(Placement::new(
            "valve.3_2.spring_return",
            Pt::new(60.0, 60.0),
            20.0,
            20.0,
        ));
        Window::new(Theme::fallback(), document, None, None)
    }

    fn harness(window: &mut Window) -> egui_kittest::Harness<'_> {
        let mut harness =
            egui_kittest::Harness::new(move |context| window.show_check_panel(context));
        harness.set_size(egui::vec2(1366.0, 768.0));
        if crate::theme::install_font(&harness.ctx, None).is_err() {
            eprintln!("no system font: the check panel test needs one");
        }
        harness.run();
        harness
    }

    #[test]
    fn the_check_panel_follows_the_language() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_check();
        {
            let mut harness = harness(&mut window);
            assert!(harness.query_all_by_label("Check circuit").count() >= 1);
            assert!(harness.query_all_by_label_contains("no faults").count() >= 1);
            harness.get_by_label("Close").click();
            harness.run();
        }
        window.language = Language::Spanish;
        window.open_check();
        {
            let mut harness = harness(&mut window);
            assert!(harness.query_all_by_label("Comprobar circuito").count() >= 1);
            assert!(harness.query_all_by_label_contains("sin faltas").count() >= 1);
            harness.get_by_label("Cerrar").click();
            harness.run();
        }
    }

    #[test]
    fn the_panel_lists_the_faults() {
        let mut window = window_with_a_valve();
        window.open_check();
        assert!(window.check_panel.open);
        assert!(
            !window.check_panel.findings.is_empty(),
            "the valve's open ports are a fault"
        );
    }

    #[test]
    fn a_clean_sheet_says_so() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_check();
        assert!(
            window.check_panel.findings.is_empty(),
            "an empty sheet has no fault"
        );
    }

    #[test]
    fn a_fault_selects_its_component() {
        let mut window = window_with_a_valve();
        window.open_check();
        let index = window
            .check_panel
            .findings
            .iter()
            .position(|finding| finding.component.is_some())
            .expect("a fault names a component");
        let component = window.check_panel.findings[index].component;
        window.select_finding(index);
        assert_eq!(Some(window.selected.clone()), component.map(|c| vec![c]));
    }

    #[test]
    fn escape_closes_the_panel() {
        let mut window = window_with_a_valve();
        window.open_check();
        {
            let mut harness = harness(&mut window);
            harness.key_press(egui::Key::Escape);
            harness.run();
        }
        assert!(!window.check_panel.open);
    }
}
