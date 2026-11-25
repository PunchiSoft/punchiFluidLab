// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The export options: the format the name defaults to, the PNG resolution and whether the
//! drawing carries a parts list.
//!
//! The options are view state: an export prints the document and never edits it, so a change
//! here is neither an undo nor a modified mark.

use eframe::egui;
use punchi_fluid_i18n::Language;

use crate::Window;
use crate::dialog::{Action, ButtonRole, Dialog, DialogResult, DialogStyle};
use crate::theme::Tokens;

/// The format an export defaults its name to. The writer still follows the typed extension;
/// this is the default, not a hidden substitution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExportFormat {
    Pdf,
    Png,
    Svg,
}

impl ExportFormat {
    pub(crate) const ALL: [Self; 3] = [Self::Pdf, Self::Png, Self::Svg];

    /// The label the panel shows.
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Pdf => "PDF",
            Self::Png => "PNG",
            Self::Svg => "SVG",
        }
    }

    /// The extension the file dialog suggests when the reader types none.
    pub(crate) const fn extension(self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Png => "png",
            Self::Svg => "svg",
        }
    }
}

/// The export options, as the window keeps them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ExportOptions {
    pub(crate) open: bool,
    pub(crate) format: ExportFormat,
    pub(crate) dpi: u16,
    pub(crate) parts: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            open: false,
            format: ExportFormat::Pdf,
            dpi: 300,
            parts: false,
        }
    }
}

impl ExportOptions {
    /// The resolutions the panel offers for a raster export.
    pub(crate) const DPI: [u16; 4] = [72, 150, 300, 600];
}

impl Window {
    /// **Opciones…**: opens a draft with the committed options.
    pub(crate) fn open_export_options(&mut self) {
        self.export_options_draft = self.export_options;
        self.export_options_draft.open = false;
        self.export_options.open = true;
    }

    /// Commits the draft and closes the panel; an export does not touch the document.
    pub(crate) fn apply_export_options(&mut self) {
        self.export_options = self.export_options_draft;
        self.export_options.open = false;
        self.notice = Some(self.language.text("export.applied").to_owned());
    }

    /// Draws the panel; the ×, `Esc` or **Cancelar** discards the draft.
    pub(crate) fn show_export_options(&mut self, context: &egui::Context) {
        if !self.export_options.open {
            return;
        }
        let tokens = self.theme.tokens.clone();
        let style = DialogStyle::new(&self.theme.tokens);
        let language = self.language;
        let dialog = Dialog {
            id: egui::Id::new("window.export-options"),
            title: language.text("export.title"),
            close_label: language.text("dialog.close"),
            preferred_width: (tokens.unit * 30.0).max(480.0),
            style,
        };
        let draft = &mut self.export_options_draft;
        let response = dialog.show(
            context,
            draft,
            |ui, draft| export_options_body(ui, style, &tokens, draft, language),
            |_| {
                vec![
                    Action {
                        label: language.text("dialog.apply"),
                        value: ExportAction::Apply,
                        role: ButtonRole::Primary,
                        enabled: true,
                        focus: false,
                    },
                    Action {
                        label: language.text("dialog.cancel"),
                        value: ExportAction::Cancel,
                        role: ButtonRole::Secondary,
                        enabled: true,
                        focus: false,
                    },
                ]
            },
        );
        match response.inner {
            Some(DialogResult::Action(ExportAction::Apply)) => self.apply_export_options(),
            Some(DialogResult::Action(ExportAction::Cancel) | DialogResult::Close) => {
                self.export_options.open = false;
            }
            None if response.should_close() => self.export_options.open = false,
            None => {}
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExportAction {
    Apply,
    Cancel,
}

fn export_options_body(
    ui: &mut egui::Ui,
    style: DialogStyle,
    tokens: &Tokens,
    options: &mut ExportOptions,
    language: Language,
) {
    style.card(style.surface).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(
            egui::RichText::new(language.text("export.format"))
                .size(style.text_size)
                .strong()
                .color(style.text),
        );
        ui.horizontal_wrapped(|ui| {
            for format in ExportFormat::ALL {
                if ui
                    .selectable_label(options.format == format, format.label())
                    .clicked()
                {
                    options.format = format;
                }
            }
        });
    });
    ui.add_space(tokens.unit * 0.5);
    style.card(style.surface).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(
            egui::RichText::new(language.text("export.resolution"))
                .size(style.text_size)
                .strong()
                .color(style.text),
        );
        ui.horizontal_wrapped(|ui| {
            for dpi in ExportOptions::DPI {
                let chosen = options.dpi == dpi;
                let label = language.format("export.dpi", &[("dpi", &dpi.to_string())]);
                if ui
                    .add_enabled(
                        options.format == ExportFormat::Png,
                        egui::Button::new(label).selected(chosen),
                    )
                    .clicked()
                {
                    options.dpi = dpi;
                }
            }
        });
    });
    ui.add_space(tokens.unit * 0.5);
    ui.checkbox(&mut options.parts, language.text("export.parts"));
}

#[cfg(test)]
mod tests {
    use eframe::egui;
    use egui_kittest::kittest::Queryable;
    use punchi_fluid_doc::{Document, Placement};
    use punchi_fluid_draw::Pt;

    use crate::Window;
    use crate::theme::Theme;

    #[test]
    fn the_export_options_follow_the_language() {
        let mut window = window_with_a_valve();
        window.open_export_options();
        {
            let harness = harness(&mut window);
            assert!(harness.query_all_by_label("Export options").count() >= 1);
            assert!(harness.query_all_by_label("300 dpi").count() >= 1);
            assert!(
                harness
                    .query_all_by_label_contains("Include parts list")
                    .count()
                    >= 1
            );
            assert!(
                harness
                    .query_by_role_and_label(egui::accesskit::Role::Button, "Apply")
                    .is_some()
            );
            assert!(
                harness
                    .query_by_role_and_label(egui::accesskit::Role::Button, "Cancel")
                    .is_some()
            );
            assert!(harness.query_by_label("Close").is_some());
        }
        window.export_options.open = false;
        window.language = punchi_fluid_i18n::Language::Spanish;
        window.open_export_options();
        {
            let harness = harness(&mut window);
            assert!(
                harness
                    .query_all_by_label("Opciones de exportación")
                    .count()
                    >= 1
            );
            assert!(harness.query_all_by_label("300 ppp").count() >= 1);
            assert!(
                harness
                    .query_all_by_label_contains("Incluir lista de piezas")
                    .count()
                    >= 1
            );
            assert!(
                harness
                    .query_by_role_and_label(egui::accesskit::Role::Button, "Aplicar")
                    .is_some()
            );
            assert!(
                harness
                    .query_by_role_and_label(egui::accesskit::Role::Button, "Cancelar")
                    .is_some()
            );
            assert!(harness.query_by_label("Cerrar").is_some());
        }
    }

    fn window_with_a_valve() -> Window {
        let mut document = Document::new();
        document.place(Placement::new(
            "valve.3_2.spring_return",
            Pt::new(60.0, 60.0),
            24.0,
            24.0,
        ));
        Window::new(Theme::fallback(), document, None, None)
    }

    fn harness(window: &mut Window) -> egui_kittest::Harness<'_> {
        let mut harness =
            egui_kittest::Harness::new(move |context| window.show_export_options(context));
        harness.set_size(egui::vec2(1366.0, 768.0));
        if crate::theme::install_font(&harness.ctx, None).is_err() {
            eprintln!("no system font: the export options test needs one");
        }
        harness.run();
        harness
    }

    #[test]
    fn escape_closes_the_options_and_discards_the_draft() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_export_options();
        window.export_options_draft.format = super::ExportFormat::Png;
        {
            let mut harness = harness(&mut window);
            harness.key_press(egui::Key::Escape);
            harness.run();
        }
        assert!(!window.export_options.open);
        assert_eq!(window.export_options.format, super::ExportFormat::Pdf);
    }

    #[test]
    fn applying_keeps_the_choices() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_export_options();
        window.export_options_draft.format = super::ExportFormat::Png;
        window.export_options_draft.dpi = 150;
        window.export_options_draft.parts = true;
        window.apply_export_options();
        assert_eq!(window.export_options.format, super::ExportFormat::Png);
        assert_eq!(window.export_options.dpi, 150);
        assert!(window.export_options.parts);
        assert!(!window.export_options.open);
    }

    #[test]
    fn cancel_discards_the_draft() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_export_options();
        window.export_options_draft.format = super::ExportFormat::Png;
        window.export_options_draft.dpi = 150;
        window.export_options_draft.parts = true;
        {
            let mut harness = harness(&mut window);
            harness.get_by_label("Cancel").hover();
            harness.run();
            harness.get_by_label("Cancel").click();
            harness.run();
        }
        assert!(!window.export_options.open);
        assert_eq!(window.export_options.format, super::ExportFormat::Pdf);
        assert_eq!(window.export_options.dpi, 300);
        assert!(!window.export_options.parts);
    }

    #[test]
    fn the_close_button_discards_the_draft() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_export_options();
        window.export_options_draft.format = super::ExportFormat::Png;
        {
            let mut harness = harness(&mut window);
            harness.get_by_label("Close").hover();
            harness.run();
            harness.get_by_label("Close").click();
            harness.run();
        }
        assert!(!window.export_options.open);
        assert_eq!(window.export_options.format, super::ExportFormat::Pdf);
    }

    #[test]
    fn the_options_fit_a_narrow_window() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_export_options();
        {
            let mut harness = harness(&mut window);
            harness.set_size(egui::vec2(420.0, 420.0));
            harness.run_steps(5);
            let close = harness.get_by_label("Close");
            assert!(close.rect().size().min_elem() >= 32.0);
            assert!(close.rect().right() <= 420.0);
            for label in ["Apply", "Cancel"] {
                let rect = harness.get_by_label(label).rect();
                assert!(rect.height() >= 32.0, "{label} has a full height target");
                assert!(
                    rect.left() >= 0.0
                        && rect.right() <= 420.0
                        && rect.top() >= 0.0
                        && rect.bottom() <= 420.0,
                    "{label} fits the narrow window: {rect:?}"
                );
            }
        }
    }

    #[test]
    fn the_png_resolution_changes_the_file() {
        let mut window = window_with_a_valve();
        window.export_options.format = super::ExportFormat::Png;
        let directory = std::env::temp_dir();
        let small = directory.join(format!("punchi-export-{}.png", std::process::id()));
        window.export_options.dpi = 72;
        window.export_to(&small).expect("the PNG writes");
        window.export_options.dpi = 300;
        let big = directory.join(format!("punchi-export-big-{}.png", std::process::id()));
        window.export_to(&big).expect("the PNG writes");
        let small_width = png_width(&std::fs::read(&small).expect("the file"));
        let big_width = png_width(&std::fs::read(&big).expect("the file"));
        assert!(big_width > small_width, "{big_width} > {small_width}");
        std::fs::remove_file(small).ok();
        std::fs::remove_file(big).ok();
    }

    #[test]
    fn the_parts_list_reaches_the_drawing() {
        let mut window = window_with_a_valve();
        let directory = std::env::temp_dir();
        let with_parts = directory.join(format!("punchi-parts-{}.svg", std::process::id()));
        window.export_options.parts = true;
        window.export_to(&with_parts).expect("the SVG writes");
        let text = std::fs::read_to_string(&with_parts).expect("the file");
        assert!(
            text.to_lowercase().contains("parts list"),
            "the drawing carries the parts list"
        );
        let without = directory.join(format!("punchi-no-parts-{}.svg", std::process::id()));
        window.export_options.parts = false;
        window.export_to(&without).expect("the SVG writes");
        let text = std::fs::read_to_string(&without).expect("the file");
        assert!(
            !text.to_lowercase().contains("parts list"),
            "no parts list when it is off"
        );
        window.export_options.parts = true;
        window.language = punchi_fluid_i18n::Language::Spanish;
        let spanish = directory.join(format!("punchi-parts-es-{}.svg", std::process::id()));
        window.export_to(&spanish).expect("the SVG writes");
        let text = std::fs::read_to_string(&spanish).expect("the file");
        assert!(
            text.to_lowercase().contains("lista de piezas"),
            "the Spanish drawing carries its parts list"
        );
        std::fs::remove_file(with_parts).ok();
        std::fs::remove_file(without).ok();
        std::fs::remove_file(spanish).ok();
    }

    /// Width of a PNG from its IHDR chunk.
    fn png_width(bytes: &[u8]) -> u32 {
        u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]])
    }
}
