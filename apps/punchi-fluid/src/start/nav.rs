// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The start rail: the brand, the way back, the `Iniciar` group and the `Aprender` group.
//!
//! The rail is the page's own column and not the catalog sidebar: while Inicio is open it
//! takes the catalog's width and tokens, and the catalog is not drawn at all. Its rows are
//! painted controls, reachable with `Tab` and named for the accessibility layer; `Enter` and
//! `Space` activate them, as they do the ribbon's own controls.

use eframe::egui;
use punchi_fluid_i18n::Language;

use super::{
    Action, Drawing, ICON_ABOUT, ICON_BACK, ICON_HOME, ICON_LICENSE, ICON_NEW, ICON_OPEN,
    ICON_PREFERENCES, activated, focusable, icon,
};
use crate::theme::Tokens;

/// The rail's icon size, in grid units: the proposal's `unit * 1.1` raster rule.
const RAIL_ICON: f32 = 1.1;

/// One row of the start rail: its drawing, its label and the shortcut at the right.
struct NavItem {
    label: &'static str,
    drawing: Drawing,
    shortcut: Option<&'static str>,
    current: bool,
}

const NAV_TO_SHEET: NavItem = NavItem {
    label: "start.nav.back",
    drawing: ICON_BACK,
    shortcut: None,
    current: false,
};
const NAV_HOME: NavItem = NavItem {
    label: "start.nav.home",
    drawing: ICON_HOME,
    shortcut: None,
    current: true,
};
const NAV_NEW: NavItem = NavItem {
    label: "start.nav.new",
    drawing: ICON_NEW,
    shortcut: Some("Ctrl+N"),
    current: false,
};
const NAV_OPEN: NavItem = NavItem {
    label: "start.nav.open",
    drawing: ICON_OPEN,
    shortcut: Some("Ctrl+O"),
    current: false,
};
const NAV_PREFERENCES: NavItem = NavItem {
    label: "start.nav.preferences",
    drawing: ICON_PREFERENCES,
    shortcut: Some("Ctrl+,"),
    current: false,
};
const NAV_ABOUT: NavItem = NavItem {
    label: "start.nav.about",
    drawing: ICON_ABOUT,
    shortcut: None,
    current: false,
};
const NAV_LICENSE: NavItem = NavItem {
    label: "start.nav.licence",
    drawing: ICON_LICENSE,
    shortcut: None,
    current: false,
};

/// Draws the rail the reference shows: Inicio chosen, the two ways in, the settings gear and,
/// at the foot, Acerca de and Licencia. The way back to the sheet appears only when a document
/// is open; the title and the group headings of the retired layout are gone.
pub(super) fn nav(
    ui: &mut egui::Ui,
    tokens: &Tokens,
    language: Language,
    has_document: bool,
) -> Option<Action> {
    let mut action = None;
    let seam = ui.max_rect();
    ui.add_space(tokens.unit * 0.75);
    if has_document && nav_row(ui, tokens, &NAV_TO_SHEET, language) {
        action = Some(Action::Sheet);
    }
    // Inicio is the page itself: choosing it keeps the reader where it is.
    nav_row(ui, tokens, &NAV_HOME, language);
    if nav_row(ui, tokens, &NAV_NEW, language) {
        action = Some(Action::NewProject);
    }
    if nav_row(ui, tokens, &NAV_OPEN, language) {
        action = Some(Action::OpenDialog);
    }
    ui.separator();
    if nav_row(ui, tokens, &NAV_PREFERENCES, language) {
        action = Some(Action::Preferences);
    }
    // Acerca de and Licencia keep the foot of the rail: the space between them and the
    // settings row is what the capture calls the rail's quiet end.
    let height = tokens.row_height;
    let foot = height * 2.0 + ui.spacing().item_spacing.y;
    ui.add_space((ui.available_height() - foot).max(0.0));
    if nav_row(ui, tokens, &NAV_ABOUT, language) {
        action = Some(Action::About);
    }
    if nav_row(ui, tokens, &NAV_LICENSE, language) {
        action = Some(Action::Licence);
    }
    // The seam between the two planes of the page: the rail's edge, not a colour.
    ui.painter().line_segment(
        [
            egui::pos2(seam.right() - 0.5, seam.top()),
            egui::pos2(seam.right() - 0.5, seam.bottom()),
        ],
        egui::Stroke::new(1.0_f32, tokens.line),
    );
    action
}

/// One rail row: its drawing, its label, the shortcut hint and the accent mark when it is
/// the current view. Reachable by `Tab`, named for the accessibility layer, activated with
/// `Enter` or `Space` like the ribbon's own controls.
fn nav_row(ui: &mut egui::Ui, tokens: &Tokens, item: &NavItem, language: Language) -> bool {
    let label = language.text(item.label);
    let height = tokens.row_height;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::hover(),
    );
    let id = ui.make_persistent_id(("start.nav", item.label));
    let response = focusable(ui, rect, id);
    if !ui.is_rect_visible(rect) {
        return false;
    }
    let ink = tokens.ink;
    if item.current {
        // The capture's chosen row is a light pill, not the accent: the ink stays readable
        // and the mark is not colour-only, the accessibility layer says selected.
        ui.painter()
            .rect_filled(rect, tokens.radius_control, tokens.surface_raised);
    } else if response.hovered() || response.has_focus() {
        ui.painter()
            .rect_filled(rect, tokens.radius_control, tokens.accent_soft);
    }
    if response.has_focus() && !item.current {
        ui.painter().rect_stroke(
            rect,
            tokens.radius_control,
            egui::Stroke::new(tokens.focus_width, tokens.accent),
            egui::StrokeKind::Inside,
        );
    }
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + tokens.unit * 1.3, rect.center().y),
        egui::vec2(tokens.unit * RAIL_ICON, tokens.unit * RAIL_ICON),
    );
    icon(ui, item.drawing, label, ink, icon_rect);
    ui.painter().text(
        egui::pos2(icon_rect.right() + tokens.unit * 0.6, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(tokens.text_action),
        ink,
    );
    let current = item.current;
    response.widget_info(|| {
        if current {
            egui::WidgetInfo::selected(egui::WidgetType::Button, true, true, label)
        } else {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label)
        }
    });
    let activated = activated(ui, &response);
    match item.shortcut {
        Some(shortcut) => response.on_hover_text(format!("{label} ({shortcut})")),
        None => response.on_hover_text(label),
    };
    activated
}

#[cfg(test)]
mod tests {
    use super::NAV_PREFERENCES;

    #[test]
    fn the_settings_row_uses_the_gear() {
        assert_eq!(
            NAV_PREFERENCES.drawing.0, "preferences-system",
            "Settings wears the gear, not the theme brush"
        );
    }
}
