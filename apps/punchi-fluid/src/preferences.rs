// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Configuración: the window's appearance and its panel settings.
//!
//! `docs/ui-ux.md` §2.4: a nav, a page and a footer, opened from the Inicio rail, `Ctrl+,`
//! and **Ver ▸ Tema**. The appearance applies live and is view state: nothing here touches
//! the document, its history or its modified mark. A built-in theme is a `Palette` seed
//! through the same theme adapter (ADR 0042); the card shows the measured result.

use eframe::egui;
use punchi_fluid_i18n::Language;

use crate::Window;
use crate::dialog::{Action as DialogAction, ButtonRole, Dialog, DialogResult, DialogStyle};
use crate::theme::{Appearance, ThemeId, Tokens};

mod pages;

/// The page the window shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Page {
    #[default]
    Appearance,
    Panels,
}

impl Page {
    const ALL: [(Self, &'static str); 2] = [
        (Self::Appearance, "preferences.page.appearance"),
        (Self::Panels, "preferences.page.panels"),
    ];
}

/// Which of the two system slots a theme card fills.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Slot {
    #[default]
    Day,
    Night,
}

impl Slot {
    fn label_key(self) -> &'static str {
        match self {
            Self::Day => "preferences.slot.day",
            Self::Night => "preferences.slot.night",
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

    /// Applies the language and remembers it at once: it is view state, and a close before
    /// the settle delay must not lose it.
    pub(crate) fn apply_language(&mut self, context: &egui::Context, language: Language) {
        if self.language == language {
            return;
        }
        self.language = language;
        let settings = self.panel_settings();
        self.settings.remember(settings);
        context.request_repaint();
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
        let tokens = self.theme.tokens.clone();
        let style = DialogStyle::new(&self.theme.tokens);
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
        let language = self.language;
        let data = PageData {
            page,
            tokens: &tokens,
            style,
            previews: &previews,
            current,
            day,
            night,
            aimed,
            day_preview,
            night_preview,
            language,
            autohide_on: self.autohide_on,
        };
        let mut chosen = Chosen::default();
        let dialog = Dialog {
            id: egui::Id::new("window.preferences"),
            title: language.text("preferences.title"),
            close_label: language.text("dialog.close"),
            preferred_width: (tokens.unit * 46.0).max(736.0),
            style,
        };
        let response = dialog.show(
            context,
            &mut chosen,
            |ui, chosen| pages::preferences_body(ui, &data, chosen),
            |_chosen| {
                vec![
                    DialogAction {
                        label: language.text("preferences.reset_page"),
                        value: Action::ResetPage,
                        role: ButtonRole::Secondary,
                        enabled: true,
                        focus: false,
                    },
                    DialogAction {
                        label: language.text("dialog.done"),
                        value: Action::Close,
                        role: ButtonRole::Secondary,
                        enabled: true,
                        focus: false,
                    },
                ]
            },
        );
        let action = match response.inner {
            Some(DialogResult::Action(action)) => Some(action),
            Some(DialogResult::Close) => Some(Action::Close),
            None if response.should_close() => Some(Action::Close),
            None => chosen.action,
        };
        if let Some(candidate) = chosen.page {
            self.preferences.page = candidate;
        }
        if let Some(language) = chosen.language {
            self.apply_language(context, language);
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
        match action {
            Some(Action::Close) => self.preferences.open = false,
            Some(Action::ResetPage) => match page {
                Page::Appearance => self.apply_appearance(context, Appearance::Desktop),
                Page::Panels => self.reset_view(),
            },
            Some(Action::ToggleAutohide) => self.toggle_autohide(),
            Some(Action::ResetView) => self.reset_view(),
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
    language: Option<Language>,
    action: Option<Action>,
}

/// The values the page draws with, gathered before the window borrows nothing from the app.
struct PageData<'a> {
    page: Page,
    tokens: &'a Tokens,
    style: DialogStyle,
    previews: &'a [(Appearance, Tokens)],
    current: Appearance,
    day: ThemeId,
    night: ThemeId,
    aimed: Slot,
    day_preview: Tokens,
    night_preview: Tokens,
    language: Language,
    autohide_on: bool,
}

impl PageData<'_> {
    /// The page the window shows: kept on the data so the closure does not read the app.
    fn current_page(&self, candidate: Page) -> bool {
        self.page == candidate
    }
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
    fn the_settings_window_follows_the_language() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_preferences();
        {
            let mut harness = harness(&mut window);
            assert!(harness.query_all_by_label("Settings").count() >= 1);
            assert!(harness.query_all_by_label("Appearance").count() >= 1);
            assert!(harness.query_all_by_label("Slate").count() >= 1);
            assert!(
                harness
                    .query_by_role_and_label(egui::accesskit::Role::Button, "Done")
                    .is_some()
            );
            harness.get_by_label("Close").click();
            harness.run();
        }
        window.language = punchi_fluid_i18n::Language::Spanish;
        window.open_preferences();
        {
            let mut harness = harness(&mut window);
            assert!(harness.query_all_by_label("Configuración").count() >= 1);
            assert!(harness.query_all_by_label("Apariencia").count() >= 1);
            assert!(harness.query_all_by_label("Pizarra").count() >= 1);
            assert!(
                harness
                    .query_by_role_and_label(egui::accesskit::Role::Button, "Listo")
                    .is_some()
            );
            harness.get_by_label("Cerrar").click();
            harness.run();
        }
    }

    #[test]
    fn the_window_chooses_pizarra_and_closes() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_preferences();
        {
            let mut harness = harness(&mut window);
            harness
                .get_by_role_and_label(egui::accesskit::Role::Button, "Slate")
                .click();
            harness.run();
            harness.get_by_label("Close").click();
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
    fn the_language_selector_switches_the_interface() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_preferences();
        {
            let mut harness = harness(&mut window);
            harness
                .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Language")
                .hover();
            harness.run();
            harness
                .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Language")
                .click();
            harness.run();
            harness.get_by_label("Español").hover();
            harness.run();
            harness.get_by_label("Español").click();
            harness.run();
        }
        assert_eq!(
            window.language,
            punchi_fluid_i18n::Language::Spanish,
            "the selector switches the language"
        );
        assert_eq!(
            window.panel_settings().language,
            punchi_fluid_i18n::Language::Spanish,
            "the language is view state"
        );
    }

    #[test]
    fn the_window_offers_the_light_theme_too() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_preferences();
        {
            let mut harness = harness(&mut window);
            harness
                .get_by_role_and_label(egui::accesskit::Role::Button, "Light CAD")
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
                .get_by_role_and_label(egui::accesskit::Role::Button, "Nordic")
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
                .get_by_role_and_label(egui::accesskit::Role::Button, "Deep blue")
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
                .get_by_role_and_label(egui::accesskit::Role::Button, "Graphite")
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
                .get_by_role_and_label(egui::accesskit::Role::Button, "System")
                .hover();
            harness.run();
            harness
                .get_by_role_and_label(egui::accesskit::Role::Button, "System")
                .click();
            harness.run();
            harness
                .get_by_role_and_label(egui::accesskit::Role::Button, "Night")
                .hover();
            harness.run();
            harness
                .get_by_role_and_label(egui::accesskit::Role::Button, "Night")
                .click();
            harness.run();
            harness
                .get_by_role_and_label(egui::accesskit::Role::Button, "Light CAD")
                .hover();
            harness.run();
            harness
                .get_by_role_and_label(egui::accesskit::Role::Button, "Light CAD")
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
    fn the_footer_reset_returns_to_the_desktop() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_preferences();
        {
            let mut harness = harness(&mut window);
            harness
                .get_by_role_and_label(egui::accesskit::Role::Button, "Slate")
                .hover();
            harness.run();
            harness
                .get_by_role_and_label(egui::accesskit::Role::Button, "Slate")
                .click();
            harness.run();
            harness
                .get_by_role_and_label(egui::accesskit::Role::Button, "Reset page")
                .hover();
            harness.run();
            harness
                .get_by_role_and_label(egui::accesskit::Role::Button, "Reset page")
                .click();
            harness.run();
        }
        assert_eq!(
            window.theme.appearance,
            Appearance::Desktop,
            "Reset page returns the appearance page to the desktop"
        );
    }

    #[test]
    fn the_preferences_fit_a_narrow_window() {
        let mut window = Window::new(Theme::fallback(), Document::new(), None, None);
        window.open_preferences();
        {
            let mut harness = harness(&mut window);
            harness.set_size(egui::vec2(420.0, 420.0));
            harness.run_steps(5);
            let close = harness.get_by_label("Close");
            assert!(close.rect().size().min_elem() >= 32.0);
            assert!(close.rect().right() <= 420.0);
            for label in ["Reset page", "Done"] {
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
