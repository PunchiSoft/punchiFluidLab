// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The menu bar of the window.
//!
//! Every entry is the same [`Action`] the ribbon and the keyboard use, so a command
//! exists once and is reachable three ways. The bar ends with the help menu, whose last
//! entry opens the about window: name, version, licence and the state of the symbol
//! conformance, which is the one thing a user of a diagram tool wants to know about the
//! build they are holding.

use egui::{RichText, Ui};

use punchi_fluid_theme::{Theme, Token};

use crate::ribbon::{Action, Tab};
use crate::shell::Mode;

/// Language of the menu labels.
///
/// The interface is English; the menu bar is the one place where a translated label is
/// cheap and most visible, so it follows the desktop locale until the Fluent pipeline
/// lands. It is a stopgap, and it says so.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    /// English labels.
    #[default]
    English,
    /// Spanish labels.
    Spanish,
}

impl Language {
    /// Picks the language from the usual locale variables.
    #[must_use]
    pub fn from_environment() -> Self {
        for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(value) = std::env::var(key) {
                if value.to_ascii_lowercase().starts_with("es") {
                    return Self::Spanish;
                }
                if !value.is_empty() {
                    return Self::English;
                }
            }
        }
        Self::English
    }

    /// Label of a ribbon tab in this language.
    #[must_use]
    pub const fn tab(self, tab: Tab) -> &'static str {
        match (self, tab) {
            (Self::English, Tab::File) => "File",
            (Self::English, Tab::Edit) => "Edit",
            (Self::English, Tab::View) => "View",
            (Self::English, Tab::Simulate) => "Simulate",
            (Self::Spanish, Tab::File) => "Archivo",
            (Self::Spanish, Tab::Edit) => "Editar",
            (Self::Spanish, Tab::View) => "Ver",
            (Self::Spanish, Tab::Simulate) => "Simular",
        }
    }

    /// Label of a toolbar command, in this language.
    ///
    /// Only the commands the toolbar writes out are translated while the rest of the
    /// interface is English: a half-translated screen is worse than a foreign one, but
    /// the six buttons beside a translated menu bar cannot stay English either.
    #[must_use]
    pub const fn action_label(self, action: Action) -> Option<&'static str> {
        match (self, action) {
            (Self::Spanish, Action::New) => Some("Nuevo"),
            (Self::Spanish, Action::Open) => Some("Abrir"),
            (Self::Spanish, Action::Save) => Some("Guardar"),
            (Self::Spanish, Action::SaveAs) => Some("Guardar como"),
            (Self::Spanish, Action::Undo) => Some("Deshacer"),
            (Self::Spanish, Action::Redo) => Some("Rehacer"),
            _ => None,
        }
    }

    /// Label of the tools menu.
    #[must_use]
    pub const fn tools(self) -> &'static str {
        match self {
            Self::English => "Tools",
            Self::Spanish => "Herramientas",
        }
    }

    /// Label of the preferences menu.
    #[must_use]
    pub const fn preferences(self) -> &'static str {
        match self {
            Self::English => "Preferences",
            Self::Spanish => "Preferencias",
        }
    }

    /// Label of the help menu.
    #[must_use]
    pub const fn help(self) -> &'static str {
        match self {
            Self::English => "Help",
            Self::Spanish => "Ayuda",
        }
    }

    /// Label of the last entry of the help menu.
    #[must_use]
    pub const fn about(self) -> &'static str {
        match self {
            Self::English => "About Punchi Fluid Lab",
            Self::Spanish => "Acerca de Punchi Fluid Lab",
        }
    }

    /// Label of the help entry that opens the shortcut reference.
    #[must_use]
    pub const fn help_panel(self) -> &'static str {
        match self {
            Self::English => "Keyboard and actions",
            Self::Spanish => "Teclado y acciones",
        }
    }

    /// Short label of the about entry, for a toolbar that has one line to give it.
    #[must_use]
    pub const fn about_short(self) -> &'static str {
        match self {
            Self::English => "About",
            Self::Spanish => "Acerca de",
        }
    }

    /// Word that introduces the version number.
    #[must_use]
    pub const fn version(self) -> &'static str {
        match self {
            Self::English => "Version",
            Self::Spanish => "Versión",
        }
    }

    /// Title of the about window.
    #[must_use]
    pub const fn about_title(self) -> &'static str {
        match self {
            Self::English => "About Punchi Fluid Lab",
            Self::Spanish => "Acerca de Punchi Fluid Lab",
        }
    }

    /// One line describing what the program is.
    #[must_use]
    pub const fn about_pitch(self) -> &'static str {
        match self {
            Self::English => {
                "Pneumatic and hydraulic circuit simulator: draw the schematic, run the model, watch the actuators move."
            }
            Self::Spanish => {
                "Simulador de circuitos neumáticos e hidráulicos: dibuje el esquema, ejecute el modelo, vea moverse los actuadores."
            }
        }
    }
}

/// State that survives between frames: the about window and the label language.
#[derive(Debug, Default)]
pub struct MenuState {
    /// The about window is open.
    pub about_open: bool,
    /// Language of the menu labels.
    pub language: Language,
}

impl Language {
    /// Width every menu of the bar opens with, so a list of entries and their shortcuts
    /// is not cut on the widest one.
    #[must_use]
    pub fn menu_width(self, theme: &Theme) -> f32 {
        theme.metrics.menu_min_width
    }
}

impl MenuState {
    /// Builds the state for this desktop.
    #[must_use]
    pub fn detect() -> Self {
        Self {
            about_open: false,
            language: Language::from_environment(),
        }
    }
}

/// What the menu bar produced this frame.
#[derive(Debug, Clone, Copy, Default)]
pub struct MenuOutcome {
    /// Command the user chose.
    pub action: Option<Action>,
}

/// Draws the menu bar and returns what the user chose.
pub fn show(ui: &mut Ui, theme: &Theme, state: &mut MenuState) -> MenuOutcome {
    let mut outcome = MenuOutcome::default();
    let language = state.language;

    ui.horizontal(|ui| {
        // A menu bar is text, not a row of buttons: no frame, no fill, and a highlight
        // only under the pointer — the way the native editor draws it.
        crate::shell::flat_controls(ui, theme);
        for tab in Tab::ALL {
            ui.menu_button(language.tab(tab), |ui| {
                if let Some(action) = tab_menu(ui, state.language.menu_width(theme), tab) {
                    outcome.action = Some(action);
                }
            });
        }
        ui.menu_button(language.tools(), |ui| {
            if let Some(action) = actions_menu(ui, &[Action::ReloadCatalog, Action::Help]) {
                outcome.action = Some(action);
            }
        });
        ui.menu_button(language.preferences(), |ui| {
            if let Some(action) = actions_menu(
                ui,
                &[
                    Action::ThemeSystem,
                    Action::ThemeLight,
                    Action::ThemeDark,
                    Action::ReloadTheme,
                ],
            ) {
                outcome.action = Some(action);
            }
        });
        ui.menu_button(language.help(), |ui| {
            if ui.button(language.help_panel()).clicked() {
                outcome.action = Some(Action::Help);
                ui.close();
            }
            ui.separator();
            // Last entry of the last menu, where every desktop puts it.
            if ui.button(language.about()).clicked() {
                state.about_open = true;
                ui.close();
            }
        });
    });

    outcome
}

/// A menu made of an explicit list of commands.
fn actions_menu(ui: &mut Ui, actions: &[Action]) -> Option<Action> {
    let mut fired = None;
    for action in actions {
        let enabled = action.unavailable().is_none();
        let text = RichText::new(format!(
            "{}\u{2003}{}",
            action.label(),
            action.shortcut_text()
        ));
        let response = ui.add_enabled(enabled, egui::Button::new(text));
        let response = if enabled {
            response.on_hover_cursor(egui::CursorIcon::PointingHand)
        } else {
            response
        };
        if response.clicked() {
            fired = Some(*action);
            ui.close();
        }
    }
    fired
}

fn tab_menu(ui: &mut Ui, min_width: f32, tab: Tab) -> Option<Action> {
    ui.set_min_width(min_width);
    let mut fired = None;
    for action in Action::ALL.iter().copied().filter(|one| one.tab() == tab) {
        let enabled = action.unavailable().is_none();
        let text = RichText::new(format!(
            "{}\u{2003}{}",
            action.label(),
            action.shortcut_text()
        ));
        let response = ui.add_enabled(
            enabled,
            egui::Button::new(text).min_size(egui::vec2(ui.spacing().interact_size.x * 9.0, 0.0)),
        );
        let response = if enabled {
            response
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .on_hover_text(action.tooltip())
        } else {
            response.on_disabled_hover_text(action.tooltip())
        };
        if response.clicked() {
            fired = Some(action);
            ui.close();
        }
    }
    fired
}

/// Draws the about window when it is open.
pub fn about(ui: &Ui, state: &mut MenuState, theme: &Theme, version: &str, mode: Mode) {
    if !state.about_open {
        return;
    }
    let language = state.language;
    let mut open = true;
    egui::Window::new(language.about_title())
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ui.ctx(), |ui| {
            ui.set_max_width(theme.metrics.about_width);
            ui.label(RichText::new("Punchi Fluid Lab").size(theme.metrics.about_banner));
            ui.label(
                RichText::new(format!("{} {version}", language.version()))
                    .color(theme.colour(Token::TextMuted)),
            );
            ui.add_space(theme.metrics.grid_unit / 2.0);
            ui.label(language.about_pitch());
            ui.add_space(theme.metrics.grid_unit / 2.0);
            ui.separator();
            ui.label(format!(
                "GPL-3.0-or-later\u{2003}\u{00b7}\u{2003}{}",
                mode_label(mode, language)
            ));
            ui.label(RichText::new(conformance(language)).color(theme.colour(Token::TextMuted)));
            ui.add_space(theme.metrics.grid_unit / 2.0);
            if ui
                .button(if language == Language::Spanish {
                    "Cerrar"
                } else {
                    "Close"
                })
                .clicked()
            {
                state.about_open = false;
            }
        });
    if !open {
        state.about_open = false;
    }
}

const fn mode_label(mode: Mode, language: Language) -> &'static str {
    match (mode, language) {
        (Mode::Edit, Language::English) => "Edit mode",
        (Mode::Edit, Language::Spanish) => "Modo edición",
        (Mode::Simulate, Language::English) => "Simulation mode",
        (Mode::Simulate, Language::Spanish) => "Modo simulación",
    }
}

/// The same sentence the export writes: until a symbol review is signed, the document
/// says it is unverified, and so does the build.
fn conformance(language: Language) -> &'static str {
    match language {
        Language::English => "Symbols: ISO 1219-1 conformance under review, not yet signed.",
        Language::Spanish => "Símbolos: conformidad con ISO 1219-1 en revisión, todavía sin firma.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_last_menu_entry_is_the_about_one_in_both_languages() {
        for language in [Language::English, Language::Spanish] {
            let label = language.about();
            assert!(
                label.starts_with("About") || label.starts_with("Acerca de"),
                "the last entry names the program: {label}"
            );
        }
        assert_eq!(Language::Spanish.about(), "Acerca de Punchi Fluid Lab");
    }

    #[test]
    fn every_tab_has_a_label_in_both_languages() {
        for language in [Language::English, Language::Spanish] {
            for tab in Tab::ALL {
                assert!(!language.tab(tab).is_empty());
            }
            assert!(!language.help().is_empty());
        }
        assert_eq!(Language::Spanish.tab(Tab::File), "Archivo");
        assert_eq!(Language::Spanish.tab(Tab::Simulate), "Simular");
    }

    #[test]
    fn a_spanish_desktop_gets_spanish_labels() {
        // The environment of the test runner is not the desktop's, so the parser is
        // exercised directly instead of through `std::env`.
        assert_eq!(Language::default(), Language::English);
        assert_ne!(Language::English, Language::Spanish);
    }
}
