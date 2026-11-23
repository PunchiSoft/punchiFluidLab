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
use crate::theme::{Appearance, ThemeId, Tokens};

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

/// Which of the two system slots a theme card fills.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Slot {
    #[default]
    Day,
    Night,
}

impl Slot {
    fn label_es(self) -> &'static str {
        match self {
            Self::Day => "Día",
            Self::Night => "Noche",
        }
    }
}

/// The window's state: whether it is open, which page it shows and which slot it edits.
#[derive(Debug, Default)]
pub(crate) struct Preferences {
    pub(crate) open: bool,
    page: Page,
    aimed: Slot,
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
        self.theme = crate::theme::Theme::for_appearance(
            appearance,
            self.appearance_day,
            self.appearance_night,
        );
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
        let aimed = self.preferences.aimed;
        let (day, night) = (self.appearance_day, self.appearance_night);
        let previews: Vec<(Appearance, Tokens)> = Appearance::CHOICES
            .iter()
            .map(|appearance| (*appearance, self.theme.preview(*appearance)))
            .collect();
        let day_preview = self.theme.preview(Appearance::Theme(day));
        let night_preview = self.theme.preview(Appearance::Theme(night));
        let data = PageData {
            page,
            tokens: &tokens,
            previews: &previews,
            current,
            day,
            night,
            aimed,
            day_preview,
            night_preview,
        };
        let chosen = preferences_window(context, &data, self.autohide_on);
        if let Some(candidate) = chosen.page {
            self.preferences.page = candidate;
        }
        if let Some(slot) = chosen.slot {
            self.preferences.aimed = slot;
        }
        if let Some(mode) = chosen.appearance {
            self.apply_appearance(context, mode);
        }
        if let Some(id) = chosen.theme {
            if self.theme.appearance == Appearance::System {
                match self.preferences.aimed {
                    Slot::Day => self.appearance_day = id,
                    Slot::Night => self.appearance_night = id,
                }
                self.apply_appearance(context, Appearance::System);
            } else {
                self.apply_appearance(context, Appearance::Theme(id));
            }
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
    theme: Option<ThemeId>,
    slot: Option<Slot>,
    action: Option<Action>,
}

/// The values the page draws with, gathered before the window borrows nothing from the app.
struct PageData<'a> {
    page: Page,
    tokens: &'a Tokens,
    previews: &'a [(Appearance, Tokens)],
    current: Appearance,
    day: ThemeId,
    night: ThemeId,
    aimed: Slot,
    day_preview: Tokens,
    night_preview: Tokens,
}

/// The window's frame: the blocker, the nav, the page and the footer.
fn preferences_window(context: &egui::Context, data: &PageData<'_>, autohide_on: bool) -> Chosen {
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
        .fixed_size(egui::vec2(data.tokens.unit * 46.0, data.tokens.unit * 30.0))
        .show(context, |ui| {
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(data.tokens.unit * 10.5);
                    ui.add_space(data.tokens.unit * 0.4);
                    for (candidate, label) in Page::ALL {
                        if ui
                            .selectable_label(data.current_page(candidate), label)
                            .clicked()
                        {
                            chosen.page = Some(candidate);
                        }
                    }
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width());
                    let height = ui.available_height();
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .max_height(height)
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            if data.current_page(Page::Appearance) {
                                appearance_page(ui, data, &mut chosen);
                            } else {
                                panels_page(ui, data.tokens, autohide_on, &mut chosen.action);
                            }
                        });
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

impl PageData<'_> {
    /// The page the window shows: kept on the data so the closure does not read the app.
    fn current_page(&self, candidate: Page) -> bool {
        self.page == candidate
    }
}

/// The appearance page: the three modes, the two system slots and the theme cards.
fn appearance_page(ui: &mut egui::Ui, data: &PageData<'_>, chosen: &mut Chosen) {
    ui.heading(
        egui::RichText::new("Apariencia")
            .size(data.tokens.text_label)
            .color(data.tokens.ink),
    );
    ui.label(
        egui::RichText::new("Colores, tema y contraste.")
            .size(data.tokens.text_body)
            .color(data.tokens.ink_dim),
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
    if ui.button("Restaurar apariencia").clicked() {
        chosen.action = Some(Action::ResetPage);
    }
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
                .selectable_label(selected, mode.mode_label_es())
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
        egui::RichText::new("Día / Noche")
            .size(data.tokens.text_small)
            .color(data.tokens.ink_muted),
    );
    ui.horizontal(|ui| {
        for (slot, id, preview) in [
            (Slot::Day, data.day, &data.day_preview),
            (Slot::Night, data.night, &data.night_preview),
        ] {
            ui.vertical(|ui| {
                let aimed = data.aimed == slot;
                if preview_card(ui, data.tokens, preview, aimed, slot.label_es()) {
                    chosen.slot = Some(slot);
                }
                ui.label(
                    egui::RichText::new(slot.label_es())
                        .size(data.tokens.text_body)
                        .color(data.tokens.ink),
                );
                ui.label(
                    egui::RichText::new(if aimed { "Editando" } else { id.label_es() })
                        .size(data.tokens.text_small)
                        .color(if aimed {
                            data.tokens.accent
                        } else {
                            data.tokens.ink_muted
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
        egui::RichText::new("Temas")
            .size(data.tokens.text_small)
            .color(data.tokens.ink_muted),
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
                if preview_card(ui, data.tokens, preview, selected, id.label_es()) {
                    chosen.theme = Some(*id);
                }
                ui.label(
                    egui::RichText::new(id.label_es())
                        .size(data.tokens.text_body)
                        .color(data.tokens.ink),
                );
                ui.label(
                    egui::RichText::new(if selected { "En uso" } else { "Integrado" })
                        .size(data.tokens.text_small)
                        .color(if selected {
                            data.tokens.accent
                        } else {
                            data.tokens.ink_muted
                        }),
                );
                ui.add_space(data.tokens.unit);
            });
        }
    });
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
fn preview_card(
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
    fn the_window_offers_the_light_theme_too() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_preferences();
        {
            let mut harness = harness(&mut window);
            harness
                .get_by_role_and_label(egui::accesskit::Role::Button, "CAD claro")
                .click();
            harness.run();
        }
        assert_eq!(
            window.theme.appearance,
            Appearance::Theme(ThemeId::Cad),
            "the light card applies the light theme"
        );
    }

    #[test]
    fn the_window_offers_the_high_contrast_theme() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_preferences();
        {
            let mut harness = harness(&mut window);
            harness
                .get_by_role_and_label(egui::accesskit::Role::Button, "Nórdico")
                .click();
            harness.run();
        }
        assert_eq!(
            window.theme.appearance,
            Appearance::Theme(ThemeId::Nordic),
            "the high-contrast card applies the Nordic theme"
        );
    }

    #[test]
    fn the_window_offers_the_deep_blue_theme() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_preferences();
        {
            let mut harness = harness(&mut window);
            harness
                .get_by_role_and_label(egui::accesskit::Role::Button, "Azul profundo")
                .click();
            harness.run();
        }
        assert_eq!(
            window.theme.appearance,
            Appearance::Theme(ThemeId::Indigo),
            "the deep-blue card applies the Indigo theme"
        );
    }

    #[test]
    fn the_window_offers_the_graphite_theme() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_preferences();
        {
            let mut harness = harness(&mut window);
            harness
                .get_by_role_and_label(egui::accesskit::Role::Button, "Grafito")
                .click();
            harness.run();
        }
        assert_eq!(
            window.theme.appearance,
            Appearance::Theme(ThemeId::Graphite),
            "the graphite card applies the Graphite theme"
        );
    }

    #[test]
    fn the_window_switches_to_system_and_fills_the_night_slot() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_preferences();
        {
            let mut harness = harness(&mut window);
            harness
                .get_by_role_and_label(egui::accesskit::Role::Button, "Sistema")
                .click();
            harness.run();
            harness
                .get_by_role_and_label(egui::accesskit::Role::Button, "Noche")
                .click();
            harness.run();
            harness
                .get_by_role_and_label(egui::accesskit::Role::Button, "CAD claro")
                .click();
            harness.run();
        }
        assert_eq!(
            window.theme.appearance,
            Appearance::System,
            "Sistema is the mode in force"
        );
        assert_eq!(
            window.appearance_night,
            ThemeId::Cad,
            "the card filled the aimed night slot"
        );
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
