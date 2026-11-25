// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The one shortcut table: the key, the command it dispatches and the label the hint shows.
//!
//! A shortcut exists only next to the command it runs, so a hint cannot announce a key no
//! handler answers: the tooltip and the keyboard read the same row. A command with no row has
//! no hint.

use eframe::egui;

use crate::ribbon::Command;

/// One key with the modifiers it needs and the label the interface prints.
#[allow(
    clippy::struct_excessive_bools,
    reason = "the modifiers are independent flags, not a bit set to encode"
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Shortcut {
    key: egui::Key,
    ctrl: bool,
    shift: bool,
    alt: bool,
    /// The text the hint shows, in the interface's language.
    pub(crate) label: &'static str,
    /// `true` when the shortcut must not fire while a text field has the keyboard.
    pub(crate) editing: bool,
}

impl Shortcut {
    const fn new(key: egui::Key, ctrl: bool, shift: bool, alt: bool, label: &'static str) -> Self {
        Self {
            key,
            ctrl,
            shift,
            alt,
            label,
            editing: false,
        }
    }

    /// A shortcut that edits the document and waits for no text field to have the keyboard.
    const fn editing(
        key: egui::Key,
        ctrl: bool,
        shift: bool,
        alt: bool,
        label: &'static str,
    ) -> Self {
        Self {
            key,
            ctrl,
            shift,
            alt,
            label,
            editing: true,
        }
    }

    /// `true` when the reader pressed this key with exactly these modifiers on this frame.
    pub(crate) fn pressed(self, context: &egui::Context) -> bool {
        context.input(|input| {
            input.key_pressed(self.key)
                && input.modifiers.ctrl == self.ctrl
                && input.modifiers.shift == self.shift
                && input.modifiers.alt == self.alt
        })
    }
}

/// Every shortcut the window answers, in the order the hints read them.
pub(crate) const ALL: &[(Command, Shortcut)] = &[
    (
        Command::ProjectNew,
        Shortcut::new(egui::Key::N, true, false, false, "Ctrl+N"),
    ),
    (
        Command::ProjectOpen,
        Shortcut::new(egui::Key::O, true, false, false, "Ctrl+O"),
    ),
    (
        Command::ProjectSave,
        Shortcut::new(egui::Key::S, true, false, false, "Ctrl+S"),
    ),
    (
        Command::ProjectSaveAs,
        Shortcut::new(egui::Key::S, true, true, false, "Ctrl+Shift+S"),
    ),
    (
        Command::ProjectExport,
        Shortcut::new(egui::Key::E, true, true, false, "Ctrl+Shift+E"),
    ),
    (
        Command::Undo,
        Shortcut::editing(egui::Key::Z, true, false, false, "Ctrl+Z"),
    ),
    (
        Command::Redo,
        Shortcut::editing(egui::Key::Z, true, true, false, "Ctrl+Shift+Z"),
    ),
    // The second way to redo the reference offered; the hint shows the first row.
    (
        Command::Redo,
        Shortcut::editing(egui::Key::Y, true, false, false, "Ctrl+Y"),
    ),
    (
        Command::Copy,
        Shortcut::editing(egui::Key::C, true, false, false, "Ctrl+C"),
    ),
    (
        Command::Paste,
        Shortcut::editing(egui::Key::V, true, false, false, "Ctrl+V"),
    ),
    (
        Command::Duplicate,
        Shortcut::editing(egui::Key::D, true, false, false, "Ctrl+D"),
    ),
    (
        Command::Delete,
        Shortcut::editing(egui::Key::Delete, false, false, false, "Supr"),
    ),
    (
        Command::SelectAll,
        Shortcut::editing(egui::Key::A, true, false, false, "Ctrl+A"),
    ),
    (
        Command::Rotate,
        Shortcut::new(egui::Key::R, false, false, true, "Alt+R"),
    ),
    (
        Command::Mirror,
        Shortcut::new(egui::Key::M, false, false, true, "Alt+M"),
    ),
    (
        Command::SimReset,
        Shortcut::new(egui::Key::R, true, false, false, "Ctrl+R"),
    ),
    (
        Command::Tube,
        Shortcut::new(egui::Key::D, false, false, true, "Alt+D"),
    ),
    (
        Command::Theme,
        Shortcut::new(egui::Key::T, true, false, false, "Ctrl+T"),
    ),
    (
        Command::CircuitRename,
        Shortcut::new(egui::Key::F2, false, false, false, "F2"),
    ),
    (
        Command::CircuitClose,
        Shortcut::new(egui::Key::W, true, false, false, "Ctrl+W"),
    ),
    (
        Command::Catalog,
        Shortcut::new(egui::Key::B, true, false, false, "Ctrl+B"),
    ),
    (
        Command::Properties,
        Shortcut::new(egui::Key::J, true, false, false, "Ctrl+J"),
    ),
    (
        Command::ResetView,
        Shortcut::new(egui::Key::Num0, true, true, false, "Ctrl+Shift+0"),
    ),
    (
        Command::FitSheet,
        Shortcut::new(egui::Key::Num0, true, false, false, "Ctrl+0"),
    ),
    (
        Command::ZoomReset,
        Shortcut::new(egui::Key::Num1, true, false, false, "Ctrl+1"),
    ),
    (
        Command::Start,
        Shortcut::new(egui::Key::F9, false, false, false, "F9"),
    ),
    (
        Command::Pause,
        Shortcut::new(egui::Key::F8, false, false, false, "F8"),
    ),
    (
        Command::Step,
        Shortcut::new(egui::Key::F10, false, false, false, "F10"),
    ),
    (
        Command::Stop,
        Shortcut::new(egui::Key::F5, false, false, false, "F5"),
    ),
];

/// The first shortcut of `command`, if it has one.
pub(crate) fn shortcut(command: Command) -> Option<Shortcut> {
    ALL.iter()
        .find(|(carried, _)| *carried == command)
        .map(|(_, shortcut)| *shortcut)
}

/// The label the hint prints for `command`, if it has a shortcut.
pub(crate) fn label(command: Command) -> Option<&'static str> {
    shortcut(command).map(|shortcut| shortcut.label)
}

#[cfg(test)]
mod tests {
    use super::{ALL, Command};

    #[test]
    fn every_shortcut_has_a_handler() {
        for (command, _) in ALL {
            assert!(
                Command::ALL.contains(command),
                "{command:?} has a shortcut and no dispatch arm"
            );
        }
    }

    #[test]
    fn the_fit_and_the_actual_size_keep_their_own_keys() {
        assert_eq!(super::label(Command::FitSheet), Some("Ctrl+0"));
        assert_eq!(super::label(Command::ZoomReset), Some("Ctrl+1"));
        assert_eq!(super::label(Command::ResetView), Some("Ctrl+Shift+0"));
    }

    #[test]
    fn no_two_shortcuts_share_a_key() {
        for (index, (_, left)) in ALL.iter().enumerate() {
            for (_, right) in ALL.iter().skip(index + 1) {
                assert_ne!(
                    (left.key, left.ctrl, left.shift, left.alt),
                    (right.key, right.ctrl, right.shift, right.alt),
                    "two commands answer the same key: {} and {}",
                    left.label,
                    right.label
                );
            }
        }
    }
}

#[cfg(test)]
mod press_tests {
    use eframe::egui;

    use super::{ALL, shortcut};
    use crate::ribbon::Command;

    #[test]
    fn a_pressed_shortcut_is_seen() {
        let context = egui::Context::default();
        let mut input = egui::RawInput {
            modifiers: egui::Modifiers::CTRL,
            ..Default::default()
        };
        input.events.push(egui::Event::Key {
            key: egui::Key::Z,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::CTRL,
        });
        let mut seen = false;
        let _ = context.run(input, |context| {
            seen = shortcut(Command::Undo).is_some_and(|shortcut| shortcut.pressed(context));
            let _ = ALL;
        });
        assert!(seen, "Ctrl+Z is a shortcut and the frame sees it");
    }
}
