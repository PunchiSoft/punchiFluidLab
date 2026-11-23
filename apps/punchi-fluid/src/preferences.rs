// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Configuración: the window's appearance and its panel settings.
//!
//! `docs/ui-ux.md` §2.4: a nav, a page and a footer, opened from the Inicio rail, `Ctrl+,`
//! and **Ver ▸ Tema**. The appearance applies live and is view state: nothing here touches
//! the document, its history or its modified mark. A built-in theme is a `Palette` seed
//! through the same theme adapter (ADR 0042); the card shows the measured result.

use eframe::egui;

use crate::Window;
use crate::theme::{Appearance, Tokens};

/// The page the window shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Page {
    #[default]
    Appearance,
    Panels,
}

impl Page {
    const ALL: [(Self, &'static str); 2] =
        [(Self::Appearance, "Apariencia"), (Self::Panels, "Paneles")];
}

/// The window's state: whether it is open and which page it shows.
#[derive(Debug, Default)]
pub(crate) struct Preferences {
    pub(crate) open: bool,
    page: Page,
}

impl Preferences {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

impl Window {
    /// Opens the window on the appearance page: what **Ver ▸ Tema**, `Ctrl+,` and the Inicio
    /// rail do.
    pub(crate) fn open_preferences(&mut self) {
        self.preferences.page = Page::Appearance;
        self.preferences.open = true;
    }

    /// Applies an appearance and remembers it at once: the theme is view state, and a close
    /// before the settle delay must not lose it.
    pub(crate) fn apply_appearance(&mut self, context: &egui::Context, appearance: Appearance) {
        if self.theme.appearance == appearance {
            return;
        }
        self.theme = crate::theme::Theme::for_appearance(appearance);
        self.theme.apply(context);
        let settings = self.panel_settings();
        self.settings.remember(settings);
        context.request_repaint();
    }

    /// Draws the window over the sheet or the start page, last of all surfaces.
    pub(crate) fn show_preferences(&mut self, context: &egui::Context) {
        if !self.preferences.open {
            return;
        }
        if context.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.preferences.open = false;
            return;
        }
        let tokens = self.theme.tokens.clone();
        let current = self.theme.appearance;
        let page = self.preferences.page;
        let autohide_on = self.autohide_on;
        let previews: Vec<(Appearance, Tokens)> = Appearance::CHOICES
            .iter()
            .map(|appearance| (*appearance, self.theme.preview(*appearance)))
            .collect();
        let chosen = preferences_window(context, &tokens, page, current, autohide_on, &previews);
        if let Some(candidate) = chosen.page {
            self.preferences.page = candidate;
        }
        if let Some(appearance) = chosen.appearance {
            self.apply_appearance(context, appearance);
        }
        match chosen.action {
            Some(Action::Close) => self.preferences.open = false,
            Some(Action::ResetPage) => match page {
                Page::Appearance => self.apply_appearance(context, Appearance::Desktop),
                Page::Panels => self.view_command("Restablecer vista"),
            },
            Some(Action::ToggleAutohide) => self.toggle_autohide(),
            Some(Action::ResetView) => self.view_command("Restablecer vista"),
            None => {}
        }
    }
}

/// One command the window asked for on this frame. A click can run one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Close,
    ResetPage,
    ToggleAutohide,
    ResetView,
}

/// What the window asked for, applied once it has closed on this frame.
#[derive(Default)]
struct Chosen {
    page: Option<Page>,
    appearance: Option<Appearance>,
    action: Option<Action>,
}

/// The window's frame: the blocker, the nav, the page and the footer.
fn preferences_window(
    context: &egui::Context,
    tokens: &Tokens,
    page: Page,
    current: Appearance,
    autohide_on: bool,
    previews: &[(Appearance, Tokens)],
) -> Chosen {
    let mut chosen = Chosen::default();
    // The layer under the window keeps the pointer out of the sheet while it is open.
    let screen = context.screen_rect();
    egui::Area::new(egui::Id::new("preferences.blocker"))
        .order(egui::Order::Middle)
        .fixed_pos(screen.min)
        .show(context, |ui| {
            ui.allocate_rect(screen, egui::Sense::click_and_drag());
        });
    egui::Window::new("Configuración")
        .id(egui::Id::new("window.preferences"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .fixed_size(egui::vec2(tokens.unit * 46.0, tokens.unit * 30.0))
        .show(context, |ui| {
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(tokens.unit * 10.5);
                    ui.add_space(tokens.unit * 0.4);
                    for (candidate, label) in Page::ALL {
                        if ui.selectable_label(page == candidate, label).clicked() {
                            chosen.page = Some(candidate);
                        }
                    }
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width());
                    match page {
                        Page::Appearance => appearance_page(
                            ui,
                            tokens,
                            previews,
                            current,
                            &mut chosen.appearance,
                            &mut chosen.action,
                        ),
                        Page::Panels => panels_page(ui, tokens, autohide_on, &mut chosen.action),
                    }
                });
            });
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Restaurar página").clicked() {
                    chosen.action = Some(Action::ResetPage);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Cerrar").clicked() {
                        chosen.action = Some(Action::Close);
                    }
                });
            });
        });
    chosen
}

/// The appearance page: the mode's note, the two cards and the reset.
fn appearance_page(
    ui: &mut egui::Ui,
    tokens: &Tokens,
    previews: &[(Appearance, Tokens)],
    current: Appearance,
    choose: &mut Option<Appearance>,
    action: &mut Option<Action>,
) {
    ui.heading(
        egui::RichText::new("Apariencia")
            .size(tokens.text_label)
            .color(tokens.ink),
    );
    ui.label(
        egui::RichText::new("Colores, tema y contraste.")
            .size(tokens.text_body)
            .color(tokens.ink_dim),
    );
    ui.add_space(tokens.unit * 0.8);
    ui.horizontal(|ui| {
        for (appearance, preview) in previews {
            ui.vertical(|ui| {
                let selected = *appearance == current;
                if theme_card(ui, tokens, preview, selected, appearance.label_es()) {
                    *choose = Some(*appearance);
                }
                ui.label(
                    egui::RichText::new(appearance.label_es())
                        .size(tokens.text_body)
                        .color(tokens.ink),
                );
                let note = match appearance {
                    Appearance::Desktop => "Sistema",
                    Appearance::Theme(_) => "Integrado",
                };
                let state = if selected { "En uso" } else { note };
                ui.label(
                    egui::RichText::new(state)
                        .size(tokens.text_small)
                        .color(if selected {
                            tokens.accent
                        } else {
                            tokens.ink_muted
                        }),
                );
                ui.add_space(tokens.unit);
            });
        }
    });
    if ui.button("Restaurar apariencia").clicked() {
        *action = Some(Action::ResetPage);
    }
}

/// The panel page: the auto-hide flag and the view reset, the same settings as **Ver**.
fn panels_page(ui: &mut egui::Ui, tokens: &Tokens, autohide_on: bool, action: &mut Option<Action>) {
    ui.heading(
        egui::RichText::new("Paneles")
            .size(tokens.text_label)
            .color(tokens.ink),
    );
    ui.label(
        egui::RichText::new("Qué se recuerda de la ventana; nada de esto es del documento.")
            .size(tokens.text_body)
            .color(tokens.ink_dim),
    );
    ui.add_space(tokens.unit * 0.8);
    let mut on = autohide_on;
    if ui.checkbox(&mut on, "Ocultar al salir").changed() {
        *action = Some(Action::ToggleAutohide);
    }
    ui.add_space(tokens.unit * 0.6);
    if ui.button("Restablecer vista").clicked() {
        *action = Some(Action::ResetView);
    }
}

/// A mini window drawn with the theme's own tokens: ribbon, panel, canvas, paper and accent.
fn theme_card(
    ui: &mut egui::Ui,
    tokens: &Tokens,
    preview: &Tokens,
    selected: bool,
    label: &str,
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
            egui::Stroke::new(2.0_f32, tokens.accent),
            egui::StrokeKind::Inside,
        );
    }
    response.clicked()
}

#[cfg(test)]
mod tests {
    use eframe::egui;
    use egui_kittest::kittest::Queryable;

    use crate::theme::{Appearance, Theme, ThemeId};
    use crate::{Document, Window};

    /// The window with the settings window open, drawn headless at the floor size.
    fn harness(window: &mut Window) -> egui_kittest::Harness<'_> {
        let mut harness =
            egui_kittest::Harness::new(move |context| window.show_preferences(context));
        harness.set_size(egui::vec2(1366.0, 768.0));
        if crate::theme::install_font(&harness.ctx, None).is_err() {
            eprintln!("no system font: the preferences test needs one");
        }
        harness.run();
        harness
    }

    #[test]
    fn the_window_chooses_pizarra_and_closes() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_preferences();
        {
            let mut harness = harness(&mut window);
            harness
                .get_by_role_and_label(egui::accesskit::Role::Button, "Pizarra")
                .click();
            harness.run();
            harness.get_by_label("Cerrar").click();
            harness.run();
        }
        assert_eq!(
            window.theme.appearance,
            Appearance::Theme(ThemeId::Slate),
            "the card applies the theme live"
        );
        assert!(!window.preferences.open, "Cerrar closes the window");
    }

    #[test]
    fn escape_closes_the_window() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_preferences();
        {
            let mut harness = harness(&mut window);
            harness.key_press(egui::Key::Escape);
            harness.run();
        }
        assert!(!window.preferences.open);
    }
}
