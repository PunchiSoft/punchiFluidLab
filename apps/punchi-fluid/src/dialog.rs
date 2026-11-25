// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The anatomy every dialog shares: the title at body size and the actions against the
//! right edge.
//!
//! The window is egui ([ADR 0036](../../../docs/adr/0036-the-toolkit-is-egui.md)), so the title
//! bar is the toolkit's; what this module owns is what the toolkit does not decide. The title
//! must never take the page's display size, which belongs to a view that is not a document,
//! and the actions must end at the same edge in every window, with the safe one last and the
//! default carrying the accent and the keyboard. A dialog that draws its own row drifts from
//! the others; every one asks here instead.

use eframe::egui;

use crate::theme::Tokens;

/// The title every dialog hands to its window: body size and strong, never the page's
/// display size.
pub(crate) fn title(tokens: &Tokens, text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(tokens.text_body)
        .strong()
        .color(tokens.ink)
}

/// One action of a dialog's footer.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Action<'a> {
    label: &'a str,
    default: bool,
    enabled: bool,
}

impl<'a> Action<'a> {
    /// An action the pointer and the keyboard both reach.
    pub(crate) const fn new(label: &'a str) -> Self {
        Self {
            label,
            default: false,
            enabled: true,
        }
    }

    /// The action `Enter` runs when no field owns the keyboard: the one the reader is
    /// expected to take. It also takes the focus when the dialog opens.
    pub(crate) const fn default(label: &'a str) -> Self {
        Self {
            label,
            default: true,
            enabled: true,
        }
    }

    /// Whether the action may run now; a disabled one never takes the focus nor `Enter`.
    #[must_use]
    pub(crate) const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Draws the footer: a separator, then the actions against the right edge in reading order,
/// and reports the index the reader chose.
///
/// The last action in reading order sits at the right edge, so the caller puts the safe one
/// there. Only the default carries the accent; it takes the focus when the dialog opens and
/// no field owns the keyboard. egui turns `Enter` on a focused button into a click, so the
/// keyboard answers with the default without a second handler that could disagree with the
/// widget — and a text field keeps its own `Enter`.
pub(crate) fn actions(ui: &mut egui::Ui, tokens: &Tokens, actions: &[Action<'_>]) -> Option<usize> {
    ui.separator();
    let mut chosen = None;
    let keyboard_free = !ui.ctx().wants_keyboard_input();
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        for (index, action) in actions.iter().enumerate().rev() {
            let ink = if action.default {
                tokens.on_accent
            } else {
                tokens.ink
            };
            let mut button = egui::Button::new(egui::RichText::new(action.label).color(ink));
            if action.default {
                button = button.fill(tokens.accent);
            }
            let response = ui.add_enabled(action.enabled, button);
            if response.clicked() {
                chosen = Some(index);
            }
            if action.default
                && action.enabled
                && keyboard_free
                && ui.ctx().memory(|memory| memory.focused().is_none())
            {
                response.request_focus();
            }
        }
    });
    chosen
}

#[cfg(test)]
mod tests;
