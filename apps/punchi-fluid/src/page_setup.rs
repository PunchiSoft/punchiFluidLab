// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The page setup panel: paper, orientation and the frame in one place.
//!
//! The panel edits a draft and only touches the document when the reader applies; the
//! document's own `set_sheet` validates it, so a draft that leaves no drawing space is
//! refused with a notice instead of a broken sheet.

use eframe::egui;
use punchi_fluid_doc::paper::{Margins, Orientation, PaperSize, Sheet};
use punchi_fluid_i18n::Language;

use crate::Window;
use crate::ribbon::MarginPreset;
use crate::theme::Tokens;

/// The panel's draft. `custom` chooses between the named sizes and the millimetres.
pub(crate) struct PageSetup {
    pub(crate) open: bool,
    pub(crate) paper: PaperSize,
    pub(crate) custom: bool,
    pub(crate) orientation: Orientation,
    pub(crate) margins: Margins,
    pub(crate) custom_width: f32,
    pub(crate) custom_height: f32,
}

impl Default for PageSetup {
    fn default() -> Self {
        Self::from_sheet(Sheet::a4())
    }
}

impl PageSetup {
    /// The draft a sheet opens with.
    fn from_sheet(sheet: Sheet) -> Self {
        let (width, height) = sheet.paper.size_mm();
        Self {
            open: false,
            paper: sheet.paper,
            custom: matches!(sheet.paper, PaperSize::Custom { .. }),
            orientation: sheet.orientation,
            margins: sheet.margins,
            custom_width: width,
            custom_height: height,
        }
    }

    /// Opens the panel with the document's sheet as the starting draft.
    fn load(&mut self, sheet: Sheet) {
        *self = Self::from_sheet(sheet);
        self.open = true;
    }

    /// The sheet the draft describes, or `None` when it leaves no drawing space.
    fn sheet(&self) -> Option<Sheet> {
        let paper = if self.custom {
            PaperSize::custom(self.custom_width, self.custom_height)?
        } else {
            self.paper
        };
        let sheet = Sheet {
            paper,
            orientation: self.orientation,
            margins: self.margins,
        };
        (sheet.margins.fits(paper, self.orientation) && paper.is_valid()).then_some(sheet)
    }
}

impl Window {
    /// **Configurar página**: opens the panel with the document's sheet.
    pub(crate) fn open_page_setup(&mut self) {
        self.page_setup.load(self.document.sheet());
    }

    /// Applies the draft as one undo and closes the panel; an impossible sheet is refused.
    pub(crate) fn apply_page_setup(&mut self) {
        let Some(sheet) = self.page_setup.sheet() else {
            self.notice = Some(self.language.text("page.no_room").to_owned());
            return;
        };
        if !self.document.set_sheet(sheet) {
            self.notice = Some(self.language.text("page.refused").to_owned());
            return;
        }
        self.refresh_page();
        self.page_setup.open = false;
        self.notice = Some(self.language.text("page.updated").to_owned());
    }

    /// Draws the panel; it closes with `Esc` or **Cerrar** without touching the document.
    pub(crate) fn show_page_setup(&mut self, context: &egui::Context) {
        if !self.page_setup.open {
            return;
        }
        if context.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.page_setup.open = false;
            return;
        }
        let tokens = self.theme.tokens.clone();
        let language = self.language;
        let action = page_setup_window(
            context,
            &tokens,
            &mut self.page_setup,
            language,
            &mut self.grid,
        );
        match action {
            Some(Action::Apply) => self.apply_page_setup(),
            Some(Action::Close) => self.page_setup.open = false,
            None => {}
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Apply,
    Close,
}

fn page_setup_window(
    context: &egui::Context,
    tokens: &Tokens,
    draft: &mut PageSetup,
    language: Language,
    grid: &mut crate::grid::GridView,
) -> Option<Action> {
    let mut action = None;
    let screen = context.screen_rect();
    egui::Area::new(egui::Id::new("page-setup.blocker"))
        .order(egui::Order::Middle)
        .fixed_pos(screen.min)
        .show(context, |ui| {
            ui.allocate_rect(screen, egui::Sense::click_and_drag());
        });
    egui::Window::new(language.text("page.title"))
        .id(egui::Id::new("window.page-setup"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .fixed_size(egui::vec2(tokens.unit * 34.0, tokens.unit * 27.0))
        .show(context, |ui| {
            sheet_controls(ui, tokens, draft, language);
            margin_controls(ui, tokens, draft, language);
            grid_controls(ui, tokens, grid, language);
            if draft.sheet().is_none() {
                ui.label(
                    egui::RichText::new(language.text("page.no_room"))
                        .size(tokens.text_body)
                        .color(tokens.ink_muted),
                );
            }
            ui.separator();
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        draft.sheet().is_some(),
                        egui::Button::new(language.text("dialog.apply")),
                    )
                    .clicked()
                {
                    action = Some(Action::Apply);
                }
                if ui.button(language.text("dialog.close")).clicked() {
                    action = Some(Action::Close);
                }
            });
        });
    action
}

/// The view grid: its toggle and its step, at the right of the page controls. Both are view
/// settings: the document, the print and the undo history stay untouched.
fn grid_controls(
    ui: &mut egui::Ui,
    tokens: &Tokens,
    grid: &mut crate::grid::GridView,
    language: Language,
) {
    ui.separator();
    ui.checkbox(&mut grid.visible, language.text("page.grid"));
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(language.text("page.grid_step"))
                .size(tokens.text_small)
                .color(tokens.ink_muted),
        );
        egui::ComboBox::from_id_salt("page.grid.step")
            .selected_text(step_label(grid.step_mm, language))
            .show_ui(ui, |ui| {
                for step in crate::grid::STEPS {
                    if ui
                        .selectable_label(grid.step_mm == step, step_label(step, language))
                        .clicked()
                    {
                        grid.set_step(step);
                    }
                }
            });
    });
}

/// One grid step, with the unit the reader chose.
fn step_label(step: u16, language: Language) -> String {
    let millimetres = step.to_string();
    language.format("page.grid_step_mm", &[("mm", &millimetres)])
}
/// Paper and orientation: the named sizes, the custom millimetres and the turn.
fn sheet_controls(ui: &mut egui::Ui, tokens: &Tokens, draft: &mut PageSetup, language: Language) {
    ui.label(
        egui::RichText::new(language.text("page.paper"))
            .size(tokens.text_label)
            .strong()
            .color(tokens.ink),
    );
    egui::ComboBox::from_id_salt("page-setup.paper")
        .selected_text(draft.paper_label(language))
        .show_ui(ui, |ui| {
            for paper in PaperSize::ALL {
                if ui
                    .selectable_label(!draft.custom && draft.paper == paper, paper.label())
                    .clicked()
                {
                    draft.paper = paper;
                    draft.custom = false;
                }
            }
            if ui
                .selectable_label(draft.custom, language.text("page.custom"))
                .clicked()
            {
                draft.custom = true;
            }
        });
    if draft.custom {
        ui.horizontal(|ui| {
            ui.add(
                egui::DragValue::new(&mut draft.custom_width)
                    .range(PaperSize::MIN_BOUND_MM..=PaperSize::MAX_BOUND_MM)
                    .suffix(" mm")
                    .speed(1.0),
            );
            ui.label("×");
            ui.add(
                egui::DragValue::new(&mut draft.custom_height)
                    .range(PaperSize::MIN_BOUND_MM..=PaperSize::MAX_BOUND_MM)
                    .suffix(" mm")
                    .speed(1.0),
            );
        });
    }
    ui.add_space(tokens.unit * 0.4);
    ui.label(
        egui::RichText::new(language.text("page.orientation"))
            .size(tokens.text_label)
            .strong()
            .color(tokens.ink),
    );
    ui.horizontal(|ui| {
        if ui
            .selectable_label(
                draft.orientation == Orientation::Portrait,
                language.text("page.portrait"),
            )
            .clicked()
        {
            draft.orientation = Orientation::Portrait;
        }
        if ui
            .selectable_label(
                draft.orientation == Orientation::Landscape,
                language.text("page.landscape"),
            )
            .clicked()
        {
            draft.orientation = Orientation::Landscape;
        }
    });
}

/// The frame: four presets, then the millimetres.
fn margin_controls(ui: &mut egui::Ui, tokens: &Tokens, draft: &mut PageSetup, language: Language) {
    ui.add_space(tokens.unit * 0.4);
    ui.label(
        egui::RichText::new(language.text("page.margins"))
            .size(tokens.text_label)
            .strong()
            .color(tokens.ink),
    );
    for preset in [
        MarginPreset::Normal,
        MarginPreset::Narrow,
        MarginPreset::Moderate,
        MarginPreset::Wide,
    ] {
        if ui.button(preset.label(language)).clicked() {
            draft.margins = preset.margins();
        }
    }
    egui::Grid::new("page-setup.margins")
        .num_columns(2)
        .spacing([tokens.unit * 0.4, tokens.unit * 0.2])
        .show(ui, |ui| {
            for (label, value) in [
                (language.text("page.top"), &mut draft.margins.top),
                (language.text("page.right"), &mut draft.margins.right),
                (language.text("page.bottom"), &mut draft.margins.bottom),
                (language.text("page.left"), &mut draft.margins.left),
            ] {
                ui.label(label);
                ui.add(
                    egui::DragValue::new(value)
                        .range(0.0..=200.0)
                        .suffix(" mm")
                        .speed(1.0),
                );
                ui.end_row();
            }
        });
}

impl PageSetup {
    /// The name the combo shows: the named size or **Personalizado**.
    fn paper_label(&self, language: Language) -> &'static str {
        if self.custom {
            language.text("page.custom")
        } else {
            self.paper.label()
        }
    }
}

impl MarginPreset {
    /// The name the panel and the gallery show; the four live once, in the ribbon gallery.
    fn label(self, language: Language) -> &'static str {
        language.text(match self {
            Self::Narrow => "ribbon.gallery.margin.narrow",
            Self::Normal => "ribbon.gallery.margin.normal",
            Self::Moderate => "ribbon.gallery.margin.moderate",
            Self::Wide => "ribbon.gallery.margin.wide",
        })
    }
}

#[cfg(test)]
mod tests {
    use eframe::egui;
    use egui_kittest::kittest::Queryable;
    use punchi_fluid_doc::Document;
    use punchi_fluid_doc::paper::{PaperSize, Sheet};
    use punchi_fluid_i18n::Language;

    use crate::Window;
    use crate::theme::Theme;

    fn harness(window: &mut Window) -> egui_kittest::Harness<'_> {
        let mut harness =
            egui_kittest::Harness::new(move |context| window.show_page_setup(context));
        harness.set_size(egui::vec2(1366.0, 768.0));
        if crate::theme::install_font(&harness.ctx, None).is_err() {
            eprintln!("no system font: the page setup test needs one");
        }
        harness.run();
        harness
    }

    #[test]
    fn the_page_setup_follows_the_language() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_page_setup();
        {
            let harness = harness(&mut window);
            assert!(harness.query_all_by_label("Page setup").count() >= 1);
            assert!(
                harness
                    .query_by_role_and_label(egui::accesskit::Role::Button, "Apply")
                    .is_some()
            );
            assert!(
                harness
                    .query_by_role_and_label(egui::accesskit::Role::Button, "Close")
                    .is_some()
            );
            assert!(harness.query_all_by_label("Portrait").count() >= 1);
        }
        window.page_setup.open = false;
        window.language = Language::Spanish;
        window.open_page_setup();
        {
            let harness = harness(&mut window);
            assert!(harness.query_all_by_label("Configurar página").count() >= 1);
            assert!(
                harness
                    .query_by_role_and_label(egui::accesskit::Role::Button, "Aplicar")
                    .is_some()
            );
            assert!(harness.query_all_by_label("Vertical").count() >= 1);
        }
    }

    #[test]
    fn escape_closes_the_panel() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_page_setup();
        {
            let mut harness = harness(&mut window);
            harness.key_press(egui::Key::Escape);
            harness.run();
        }
        assert!(!window.page_setup.open);
    }

    #[test]
    fn applying_a_named_paper_changes_the_sheet() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_page_setup();
        window.page_setup.paper = PaperSize::A3;
        window.apply_page_setup();
        assert_eq!(window.document.sheet().paper, PaperSize::A3);
        assert!(window.document.can_undo(), "the change is one undo");
        assert!(!window.page_setup.open, "the panel closes on apply");
    }

    #[test]
    fn a_custom_size_is_validated_and_applied() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_page_setup();
        window.page_setup.custom = true;
        window.page_setup.custom_width = 300.0;
        window.page_setup.custom_height = 200.0;
        window.apply_page_setup();
        assert_eq!(
            window.document.sheet().paper,
            PaperSize::Custom {
                width_mm: 300.0,
                height_mm: 200.0
            }
        );
    }

    #[test]
    fn impossible_margins_are_refused() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_page_setup();
        let before = window.document.sheet();
        window.page_setup.margins = punchi_fluid_doc::paper::Margins {
            top: 500.0,
            right: 500.0,
            bottom: 500.0,
            left: 500.0,
        };
        window.apply_page_setup();
        assert_eq!(window.document.sheet(), before, "the sheet is untouched");
        assert!(
            window
                .notice
                .as_deref()
                .is_some_and(|notice| notice.contains("no room for the drawing")),
            "the refusal is explained: {:?}",
            window.notice
        );
    }

    #[test]
    fn the_draft_starts_with_the_document_sheet() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_page_setup();
        assert_eq!(window.page_setup.margins, Sheet::a4().margins);
        assert_eq!(window.page_setup.orientation, Sheet::a4().orientation);
    }
}
