// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The export options: the format the name defaults to, the PNG resolution and whether the
//! drawing carries a parts list.
//!
//! The options are view state: an export prints the document and never edits it, so a change
//! here is neither an undo nor a modified mark.

use crate::dialog;
use eframe::egui;
use punchi_fluid_i18n::Language;

use crate::Window;
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
    /// **Opciones…**: opens the export options.
    pub(crate) fn open_export_options(&mut self) {
        self.export_options.open = true;
    }

    /// Keeps the choices and closes the panel; an export does not touch the document.
    pub(crate) fn apply_export_options(&mut self) {
        self.export_options.open = false;
        self.notice = Some(self.language.text("export.applied").to_owned());
    }

    /// Draws the panel; `Esc` or **Cerrar** discards the changes.
    pub(crate) fn show_export_options(&mut self, context: &egui::Context) {
        if !self.export_options.open {
            return;
        }
        if context.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.export_options.open = false;
            return;
        }
        let tokens = self.theme.tokens.clone();
        let language = self.language;
        let action = export_options_window(context, &tokens, &mut self.export_options, language);
        match action {
            Some(Action::Apply) => self.apply_export_options(),
            Some(Action::Close) => self.export_options.open = false,
            None => {}
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Apply,
    Close,
}

fn export_options_window(
    context: &egui::Context,
    tokens: &Tokens,
    options: &mut ExportOptions,
    language: Language,
) -> Option<Action> {
    let mut action = None;
    let screen = context.screen_rect();
    egui::Area::new(egui::Id::new("export-options.blocker"))
        .order(egui::Order::Middle)
        .fixed_pos(screen.min)
        .show(context, |ui| {
            ui.allocate_rect(screen, egui::Sense::click_and_drag());
        });
    egui::Window::new(dialog::title(tokens, language.text("export.title")))
        .id(egui::Id::new("window.export-options"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .fixed_size(egui::vec2(tokens.unit * 30.0, tokens.unit * 20.0))
        .show(context, |ui| {
            ui.label(
                egui::RichText::new(language.text("export.format"))
                    .size(tokens.text_label)
                    .strong()
                    .color(tokens.ink),
            );
            ui.horizontal(|ui| {
                for format in ExportFormat::ALL {
                    if ui
                        .selectable_label(options.format == format, format.label())
                        .clicked()
                    {
                        options.format = format;
                    }
                }
            });
            ui.add_space(tokens.unit * 0.4);
            ui.label(
                egui::RichText::new(language.text("export.resolution"))
                    .size(tokens.text_label)
                    .strong()
                    .color(tokens.ink),
            );
            ui.horizontal(|ui| {
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
            ui.add_space(tokens.unit * 0.4);
            ui.checkbox(&mut options.parts, language.text("export.parts"));
            let actions = [
                dialog::Action::default(language.text("dialog.apply")),
                dialog::Action::new(language.text("dialog.close")),
            ];
            if let Some(index) = dialog::actions(ui, tokens, &actions) {
                action = Some(if index == 0 {
                    Action::Apply
                } else {
                    Action::Close
                });
            }
        });
    action
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
    fn escape_closes_the_options() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_export_options();
        {
            let mut harness = harness(&mut window);
            harness.key_press(egui::Key::Escape);
            harness.run();
        }
        assert!(!window.export_options.open);
    }

    #[test]
    fn applying_keeps_the_choices() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_export_options();
        window.export_options.format = super::ExportFormat::Png;
        window.export_options.dpi = 150;
        window.export_options.parts = true;
        window.apply_export_options();
        assert_eq!(window.export_options.format, super::ExportFormat::Png);
        assert_eq!(window.export_options.dpi, 150);
        assert!(window.export_options.parts);
        assert!(!window.export_options.open);
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
