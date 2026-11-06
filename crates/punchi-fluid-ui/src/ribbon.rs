// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The ribbon: four tabs of document-wide actions and the always-visible transport.
//!
//! Every action is a value of [`Action`], not a closure, so the same command can be
//! triggered by a button, a menu entry and a keyboard shortcut. Actions that the
//! current build cannot perform are shown disabled with the reason in their tooltip,
//! never hidden: a menu that changes shape as milestones land is harder to learn.

use egui::{Key, KeyboardShortcut, Modifiers, Ui};
use punchi_fluid_theme::{Metrics, Theme, Token};

use crate::shell::Mode;

/// Tab that groups actions in the ribbon.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Tab {
    /// Whole-document actions.
    #[default]
    File,
    /// Document editing.
    Edit,
    /// What the interface shows.
    View,
    /// Running the model.
    Simulate,
}

impl Tab {
    /// Tabs in the order they are drawn.
    pub const ALL: [Self; 4] = [Self::File, Self::Edit, Self::View, Self::Simulate];

    /// Title shown on the tab and in the overflow menu.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::File => "File",
            Self::Edit => "Edit",
            Self::View => "View",
            Self::Simulate => "Simulate",
        }
    }
}

/// One command the shell can perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    /// Start an empty document.
    New,
    /// Open a document.
    Open,
    /// Save the document.
    Save,
    /// Save under another name.
    SaveAs,
    /// Write a PNG.
    ExportPng,
    /// Write a PDF.
    ExportPdf,
    /// Close the window.
    Quit,
    /// Undo the last command.
    Undo,
    /// Redo the last undone command.
    Redo,
    /// Cut the selection.
    Cut,
    /// Copy the selection.
    Copy,
    /// Paste the clipboard.
    Paste,
    /// Duplicate the selection.
    Duplicate,
    /// Delete the selection.
    Delete,
    /// Select every component.
    SelectAll,
    /// Rotate the selection a quarter turn.
    Rotate,
    /// Show or hide the sheet grid.
    ToggleGrid,
    /// Snap to the grid.
    SnapGrid,
    /// Snap to ports.
    SnapPorts,
    /// Zoom in.
    ZoomIn,
    /// Zoom out.
    ZoomOut,
    /// Fit the circuit to the window.
    ZoomFit,
    /// Follow the desktop scheme.
    ThemeSystem,
    /// Force the light scheme.
    ThemeLight,
    /// Force the dark scheme.
    ThemeDark,
    /// Re-read the desktop palette now.
    ReloadTheme,
    /// Show or hide the catalog sidebar.
    ToggleSidebar,
    /// Show or hide the status bar.
    ToggleStatusBar,
    /// Run or pause the model.
    PlayPause,
    /// Advance the model one fixed step.
    Step,
    /// Restart the model.
    Reset,
    /// Change the valve position.
    ActuateValve,
    /// Halve the playback speed.
    SpeedSlower,
    /// Double the playback speed, up to real time.
    SpeedFaster,
    /// Switch between editing and simulation.
    ToggleMode,
    /// Re-read the catalog from disk.
    ReloadCatalog,
    /// Focus the catalog search box.
    FocusSearch,
    /// Show the shortcut reference.
    Help,
}

impl Action {
    /// Every action, in tab order. The shortcut test walks this list.
    pub const ALL: &'static [Self] = &[
        Self::New,
        Self::Open,
        Self::Save,
        Self::SaveAs,
        Self::ExportPng,
        Self::ExportPdf,
        Self::Quit,
        Self::Undo,
        Self::Redo,
        Self::Cut,
        Self::Copy,
        Self::Paste,
        Self::Duplicate,
        Self::Delete,
        Self::SelectAll,
        Self::Rotate,
        Self::ToggleGrid,
        Self::SnapGrid,
        Self::SnapPorts,
        Self::ZoomIn,
        Self::ZoomOut,
        Self::ZoomFit,
        Self::ThemeSystem,
        Self::ThemeLight,
        Self::ThemeDark,
        Self::ReloadTheme,
        Self::ToggleSidebar,
        Self::ToggleStatusBar,
        Self::PlayPause,
        Self::Step,
        Self::Reset,
        Self::ActuateValve,
        Self::SpeedSlower,
        Self::SpeedFaster,
        Self::ToggleMode,
        Self::ReloadCatalog,
        Self::FocusSearch,
        Self::Help,
    ];

    /// Tab the action belongs to.
    #[must_use]
    pub const fn tab(self) -> Tab {
        match self {
            Self::New
            | Self::Open
            | Self::Save
            | Self::SaveAs
            | Self::ExportPng
            | Self::ExportPdf
            | Self::Quit => Tab::File,
            Self::Undo
            | Self::Redo
            | Self::Cut
            | Self::Copy
            | Self::Paste
            | Self::Duplicate
            | Self::Delete
            | Self::SelectAll
            | Self::Rotate => Tab::Edit,
            Self::ToggleGrid
            | Self::SnapGrid
            | Self::SnapPorts
            | Self::ZoomIn
            | Self::ZoomOut
            | Self::ZoomFit
            | Self::ThemeSystem
            | Self::ThemeLight
            | Self::ThemeDark
            | Self::ReloadTheme
            | Self::ToggleSidebar
            | Self::ToggleStatusBar
            | Self::FocusSearch
            | Self::ReloadCatalog
            | Self::Help => Tab::View,
            Self::PlayPause
            | Self::Step
            | Self::Reset
            | Self::ActuateValve
            | Self::SpeedSlower
            | Self::SpeedFaster
            | Self::ToggleMode => Tab::Simulate,
        }
    }

    /// Short name used on buttons and in the help panel.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::New => "New",
            Self::Open => "Open",
            Self::Save => "Save",
            Self::SaveAs => "Save as",
            Self::ExportPng => "Export PNG",
            Self::ExportPdf => "Export PDF",
            Self::Quit => "Quit",
            Self::Undo => "Undo",
            Self::Redo => "Redo",
            Self::Cut => "Cut",
            Self::Copy => "Copy",
            Self::Paste => "Paste",
            Self::Duplicate => "Duplicate",
            Self::Delete => "Delete",
            Self::SelectAll => "Select all",
            Self::Rotate => "Rotate 90 degrees",
            Self::ToggleGrid => "Grid",
            Self::SnapGrid => "Snap to grid",
            Self::SnapPorts => "Snap to ports",
            Self::ZoomIn => "Zoom in",
            Self::ZoomOut => "Zoom out",
            Self::ZoomFit => "Fit to window",
            Self::ThemeSystem => "System theme",
            Self::ThemeLight => "Light theme",
            Self::ThemeDark => "Dark theme",
            Self::ReloadTheme => "Reload theme",
            Self::ToggleSidebar => "Sidebar",
            Self::ToggleStatusBar => "Status bar",
            Self::PlayPause => "Play or pause",
            Self::Step => "Step 10 ms",
            Self::Reset => "Reset",
            Self::ActuateValve => "Hold valve",
            Self::SpeedSlower => "Slower",
            Self::SpeedFaster => "Faster",
            Self::ToggleMode => "Edit or simulation",
            Self::ReloadCatalog => "Reload catalog",
            Self::FocusSearch => "Search the catalog",
            Self::Help => "Keyboard shortcuts",
        }
    }

    /// Keyboard shortcut. Every action has one, which the test enforces.
    #[must_use]
    pub const fn shortcut(self) -> KeyboardShortcut {
        const COMMAND_SHIFT: Modifiers = Modifiers {
            alt: false,
            ctrl: false,
            shift: true,
            mac_cmd: false,
            command: true,
        };
        const ALT: Modifiers = Modifiers::ALT;
        let key = match self {
            Self::New => Key::N,
            Self::Open => Key::O,
            Self::Save => Key::S,
            Self::SaveAs => return KeyboardShortcut::new(COMMAND_SHIFT, Key::S),
            Self::ExportPng => return KeyboardShortcut::new(COMMAND_SHIFT, Key::P),
            Self::ExportPdf => return KeyboardShortcut::new(COMMAND_SHIFT, Key::E),
            Self::Quit => Key::Q,
            Self::Undo => Key::Z,
            Self::Redo => return KeyboardShortcut::new(COMMAND_SHIFT, Key::Z),
            Self::Cut => Key::X,
            Self::Copy => Key::C,
            Self::Paste => Key::V,
            Self::Duplicate => Key::D,
            Self::Delete => Key::Delete,
            Self::SelectAll => Key::A,
            Self::Rotate => Key::R,
            Self::ToggleGrid => Key::G,
            Self::SnapGrid => return KeyboardShortcut::new(Modifiers::SHIFT, Key::G),
            Self::SnapPorts => return KeyboardShortcut::new(Modifiers::SHIFT, Key::P),
            Self::ZoomIn => Key::Plus,
            Self::ZoomOut => Key::Minus,
            Self::ZoomFit => Key::Num0,
            Self::ThemeSystem => return KeyboardShortcut::new(ALT, Key::Num0),
            Self::ThemeLight => return KeyboardShortcut::new(ALT, Key::Num1),
            Self::ThemeDark => return KeyboardShortcut::new(ALT, Key::Num2),
            Self::ReloadTheme => return KeyboardShortcut::new(COMMAND_SHIFT, Key::T),
            Self::ToggleSidebar => Key::F4,
            Self::ToggleStatusBar => Key::F9,
            Self::PlayPause => Key::Space,
            Self::Step => Key::F6,
            Self::Reset => return KeyboardShortcut::new(COMMAND_SHIFT, Key::R),
            Self::ActuateValve => return KeyboardShortcut::new(COMMAND_SHIFT, Key::A),
            Self::SpeedSlower => Key::OpenBracket,
            Self::SpeedFaster => Key::CloseBracket,
            Self::ToggleMode => Key::M,
            Self::ReloadCatalog => Key::F5,
            Self::FocusSearch => Key::F,
            Self::Help => Key::F1,
        };
        KeyboardShortcut::new(Modifiers::COMMAND, key)
    }

    /// Why the action cannot be used in this build, if it cannot.
    #[must_use]
    pub const fn unavailable(self) -> Option<&'static str> {
        match self {
            Self::New
            | Self::Open
            | Self::Save
            | Self::SaveAs
            | Self::Undo
            | Self::Redo
            | Self::Cut
            | Self::Copy
            | Self::Paste
            | Self::Duplicate
            | Self::Delete
            | Self::SelectAll
            | Self::Rotate
            | Self::SnapGrid
            | Self::SnapPorts
            | Self::ZoomIn
            | Self::ZoomOut
            | Self::ZoomFit => Some("Arrives with the editor (M3)"),

            _ => None,
        }
    }

    /// Shortcut written the way a tooltip shows it.
    #[must_use]
    pub fn shortcut_text(self) -> String {
        let shortcut = self.shortcut();
        let mut parts = Vec::new();
        if shortcut.modifiers.command {
            parts.push(String::from("Ctrl"));
        }
        if shortcut.modifiers.shift {
            parts.push(String::from("Shift"));
        }
        if shortcut.modifiers.alt {
            parts.push(String::from("Alt"));
        }
        parts.push(String::from(shortcut.logical_key.name()));
        parts.join("+")
    }

    /// Tooltip: what the action does and how to reach it without a pointer.
    #[must_use]
    pub fn tooltip(self) -> String {
        let mut text = match self.unavailable() {
            Some(reason) => format!("{} ({}) - {reason}", self.label(), self.shortcut_text()),
            None => format!("{} ({})", self.label(), self.shortcut_text()),
        };
        if self == Self::ActuateValve {
            text.push_str(" - keep it down to keep the spool shifted");
        }
        text
    }
}

/// Ribbon state that survives between frames.
#[derive(Debug, Default)]
pub struct RibbonState {
    /// Tab currently visible.
    pub tab: Tab,
    /// Language of the tab strip, the same one the menu bar uses.
    pub language: crate::menu::Language,
    /// Icons already rasterised from the desktop's theme.
    pub icons: crate::icons::ThemeIcons,
}

/// What the ribbon needs to know about the application.
#[derive(Debug, Clone, Copy)]
pub struct RibbonInfo {
    /// The model is advancing.
    pub playing: bool,
    /// Playback speed factor.
    pub speed: f64,
    /// Editing or simulating.
    pub mode: Mode,
}

/// Draws the ribbon and returns the action the user triggered.
pub fn show(
    ui: &mut Ui,
    theme: &Theme,
    state: &mut RibbonState,
    info: &RibbonInfo,
) -> Option<Action> {
    let metrics = theme.metrics;
    let collapsed = ui.ctx().content_rect().width() < metrics.ribbon_collapse_width;
    let mut fired = None;

    ui.horizontal(|ui| {
        ui.set_min_height(metrics.ribbon_height);
        // A native toolbar: the frequent commands as icons, the rest in the menus. It
        // collapses to a menu button when the window is too narrow to hold it.
        if collapsed {
            let language = state.language;
            ui.menu_button("Menu", |ui| {
                for tab in Tab::ALL {
                    ui.menu_button(language.tab(tab), |ui| {
                        fired = fired.or(tab_menu(ui, tab));
                    });
                }
            });
        } else {
            fired = toolbar_bar(ui, theme, info, state.language, &mut state.icons);
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // The right end of the toolbar only says how the model is running.
            fired = fired.or(mode_switch(ui, info.mode));
            ui.separator();
            ui.label(
                egui::RichText::new(format!("{}x", info.speed))
                    .color(theme.colour(Token::TextMuted)),
            );
        });
    });

    fired
}

/// The buttons of the toolbar: the frequent commands, the way a native application puts
/// them within reach and leaves the rest in its menus.
///
/// Each inner slice is a group, separated from the next by a rule.
fn toolbar() -> &'static [&'static [Action]] {
    &[
        // The order and grouping of the native editor these buttons follow: the file
        // commands together, a rule, then history, then this program's own work.
        &[Action::New, Action::Open, Action::Save, Action::SaveAs],
        &[Action::Undo, Action::Redo],
        &[Action::ExportPng, Action::ExportPdf],
        &[Action::PlayPause, Action::Step, Action::Reset],
        &[Action::ActuateValve],
    ]
}

/// The toolbar itself, laid out as a row of native buttons.
fn toolbar_bar(
    ui: &mut Ui,
    theme: &Theme,
    info: &RibbonInfo,
    language: crate::menu::Language,
    icons: &mut crate::icons::ThemeIcons,
) -> Option<Action> {
    let mut fired = None;
    ui.horizontal(|ui| {
        ui.set_min_height(theme.metrics.ribbon_height);
        crate::shell::flat_controls(ui, theme);
        ui.spacing_mut().item_spacing.x = theme.metrics.toolbar_gap;
        ui.spacing_mut().button_padding = egui::vec2(
            theme.metrics.toolbar_padding,
            theme.metrics.toolbar_padding * 0.4,
        );
        for (index, group) in toolbar().iter().enumerate() {
            if index > 0 {
                ui.separator();
            }
            for action in *group {
                if let Some(action) = toolbar_button(ui, theme, *action, info, language, icons) {
                    fired = Some(action);
                }
            }
        }
    });
    fired
}

/// One toolbar button.
///
/// Built with `egui::Button`, which measures its own content: the previous hand-laid
/// version reserved a fixed width for the label and the text ran over the next button.
fn toolbar_button(
    ui: &mut Ui,
    theme: &Theme,
    action: Action,
    info: &RibbonInfo,
    language: crate::menu::Language,
    icons: &mut crate::icons::ThemeIcons,
) -> Option<Action> {
    let metrics = theme.metrics;
    let enabled = action.unavailable().is_none();
    let label = if action == Action::PlayPause && info.playing {
        "Pause"
    } else {
        language
            .action_label(action)
            .unwrap_or_else(|| action.label())
    };
    let labelled = matches!(
        action,
        Action::New | Action::Open | Action::Save | Action::SaveAs | Action::Undo | Action::Redo
    );
    let image = crate::icons::texture_image(ui, action, theme, icons);
    let text = egui::RichText::new(label).size(metrics.toolbar_label);
    let button = match (&image, labelled) {
        (Some(image), true) => egui::Button::image_and_text(image.clone(), text),
        (Some(image), false) => egui::Button::image(image.clone()),
        (None, _) => egui::Button::new(text),
    }
    .min_size(egui::vec2(0.0, metrics.toolbar_button_height));

    let response = ui.add_enabled(enabled, button);
    let response = if enabled {
        response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text(action.tooltip())
    } else {
        response.on_disabled_hover_text(action.tooltip())
    };
    let commanded = if action == Action::ActuateValve {
        response.is_pointer_button_down_on()
    } else {
        response.clicked()
    };
    commanded.then_some(action)
}

fn tab_menu(ui: &mut Ui, tab: Tab) -> Option<Action> {
    let mut fired = None;
    for action in Action::ALL
        .iter()
        .copied()
        .filter(|action| action.tab() == tab)
    {
        let enabled = action.unavailable().is_none();
        let response = ui.add_enabled(
            enabled,
            egui::Button::new(format!("{}    {}", action.label(), action.shortcut_text())),
        );
        let response = if enabled {
            response.on_hover_text(action.tooltip())
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

fn mode_switch(ui: &mut Ui, mode: Mode) -> Option<Action> {
    let label = mode.label();
    if ui
        .selectable_label(true, label)
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(Action::ToggleMode.tooltip())
        .clicked()
    {
        return Some(Action::ToggleMode);
    }
    None
}

/// Action triggered by a keyboard shortcut this frame.
///
/// Typing in a search field must not actuate a valve, so shortcuts are ignored while
/// a text field has focus; the help window keeps working.
#[must_use]
pub fn shortcut(context: &egui::Context) -> Option<Action> {
    if context.input_mut(|input| input.consume_shortcut(&Action::Help.shortcut())) {
        return Some(Action::Help);
    }
    if context.egui_wants_keyboard_input() {
        return None;
    }
    context.input_mut(|input| {
        Action::ALL
            .iter()
            .copied()
            .find(|action| *action != Action::Help && input.consume_shortcut(&action.shortcut()))
    })
}

/// `true` when the pointer is over the ribbon, so a click never reaches the sheet.
#[must_use]
pub fn contains(rect: egui::Rect, metrics: &Metrics) -> bool {
    rect.height() <= metrics.ribbon_height * 1.5
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_action_has_a_label_and_a_shortcut() {
        for action in Action::ALL {
            assert!(!action.label().is_empty(), "{action:?} has no label");
            assert!(
                !action.shortcut_text().is_empty(),
                "{action:?} has no shortcut"
            );
        }
    }

    #[test]
    fn no_two_actions_share_a_shortcut() {
        let mut seen: HashSet<KeyboardShortcut> = HashSet::new();
        for action in Action::ALL {
            assert!(
                seen.insert(action.shortcut()),
                "{action:?} reuses {}",
                action.shortcut_text()
            );
        }
    }

    #[test]
    fn every_action_belongs_to_exactly_one_tab() {
        for action in Action::ALL {
            assert!(Tab::ALL.contains(&action.tab()), "{action:?} has no tab");
        }
    }

    #[test]
    fn an_unavailable_action_explains_itself() {
        for action in Action::ALL {
            if let Some(reason) = action.unavailable() {
                assert!(reason.contains('M'), "{action:?} has a vague reason");
                assert!(
                    action.tooltip().contains(reason),
                    "{action:?} loses its reason in the tooltip"
                );
            }
        }
    }

    #[test]
    fn shortcuts_are_unique_even_between_letters_and_named_keys() {
        let redo = Action::Redo.shortcut();
        let undo = Action::Undo.shortcut();
        assert_ne!(redo, undo);
        assert!(redo.modifiers.shift);
        assert!(!undo.modifiers.shift);
    }

    #[test]
    fn the_toolbar_holds_every_command_once_and_never_a_missing_one() {
        let mut seen = Vec::new();
        for group in toolbar() {
            for action in *group {
                assert!(
                    !seen.contains(action),
                    "{} appears twice in the toolbar",
                    action.label()
                );
                seen.push(*action);
            }
        }
        // The commands a native toolbar puts within reach without opening a menu.
        for action in [
            Action::New,
            Action::Open,
            Action::Save,
            Action::ExportPng,
            Action::ExportPdf,
            Action::PlayPause,
            Action::Step,
            Action::Reset,
        ] {
            assert!(
                seen.contains(&action),
                "{} must be one click away",
                action.label()
            );
        }
        // Everything else is still reachable: the menus are built from `Action::ALL`.
        assert!(
            seen.len() < Action::ALL.len(),
            "the toolbar is a selection, not the whole menu"
        );
    }

    #[test]
    fn the_tab_strip_and_the_menu_bar_speak_the_same_language() {
        let state = RibbonState::default();
        assert_eq!(state.language, crate::menu::Language::English);
        assert_eq!(
            crate::menu::Language::Spanish.tab(Tab::Edit),
            "Editar",
            "the strip must not say Edit while the menu says Editar"
        );
    }
}
