// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The start rail: the brand, the way back, the `Iniciar` group and the `Aprender` group.
//!
//! The rail is the page's own column and not the catalog sidebar: while Inicio is open it
//! takes the catalog's width and tokens, and the catalog is not drawn at all. Its rows are
//! painted controls, reachable with `Tab` and named for the accessibility layer; `Enter` and
//! `Space` activate them, as they do the ribbon's own controls.

use eframe::egui;

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
    label: "Volver a la hoja",
    drawing: ICON_BACK,
    shortcut: None,
    current: false,
};
const NAV_HOME: NavItem = NavItem {
    label: "Inicio",
    drawing: ICON_HOME,
    shortcut: None,
    current: true,
};
const NAV_NEW: NavItem = NavItem {
    label: "Nuevo proyecto",
    drawing: ICON_NEW,
    shortcut: Some("Ctrl+N"),
    current: false,
};
const NAV_OPEN: NavItem = NavItem {
    label: "Abrir…",
    drawing: ICON_OPEN,
    shortcut: Some("Ctrl+O"),
    current: false,
};
const NAV_PREFERENCES: NavItem = NavItem {
    label: "Configuración",
    drawing: ICON_PREFERENCES,
    shortcut: Some("Ctrl+,"),
    current: false,
};
const NAV_ABOUT: NavItem = NavItem {
    label: "Acerca de",
    drawing: ICON_ABOUT,
    shortcut: None,
    current: false,
};
const NAV_LICENSE: NavItem = NavItem {
    label: "Licencia",
    drawing: ICON_LICENSE,
    shortcut: None,
    current: false,
};

/// Draws the rail: the brand, the way back, the `Iniciar` group and the `Aprender` group.
pub(super) fn nav(ui: &mut egui::Ui, tokens: &Tokens) -> Option<Action> {
    let mut action = None;
    let seam = ui.max_rect();
    ui.add_space(tokens.unit * 0.75);
    ui.horizontal(|ui| {
        ui.add_space(tokens.unit * 0.5);
        ui.label(
            egui::RichText::new("Punchi Fluid Lab")
                .size(tokens.text_label)
                .strong()
                .color(tokens.ink),
        );
    });
    ui.add_space(tokens.unit * 0.6);
    ui.separator();
    ui.add_space(tokens.unit * 0.6);
    if nav_row(ui, tokens, &NAV_TO_SHEET) {
        action = Some(Action::Sheet);
    }
    nav_heading(ui, tokens, "Iniciar");
    // Inicio is the page itself: choosing it keeps the reader where it is.
    nav_row(ui, tokens, &NAV_HOME);
    if nav_row(ui, tokens, &NAV_NEW) {
        action = Some(Action::NewProject);
    }
    if nav_row(ui, tokens, &NAV_OPEN) {
        action = Some(Action::OpenDialog);
    }
    if nav_row(ui, tokens, &NAV_PREFERENCES) {
        action = Some(Action::Preferences);
    }
    nav_heading(ui, tokens, "Aprender");
    if nav_row(ui, tokens, &NAV_ABOUT) {
        action = Some(Action::About);
    }
    if nav_row(ui, tokens, &NAV_LICENSE) {
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
fn nav_row(ui: &mut egui::Ui, tokens: &Tokens, item: &NavItem) -> bool {
    let height = (tokens.unit * 2.0).max(32.0);
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::hover(),
    );
    let id = ui.make_persistent_id(("start.nav", item.label));
    let response = focusable(ui, rect, id);
    if !ui.is_rect_visible(rect) {
        return false;
    }
    let ink = if item.current {
        tokens.on_accent
    } else {
        tokens.ink
    };
    if item.current {
        ui.painter()
            .rect_filled(rect, tokens.radius_control, tokens.accent);
    } else if response.hovered() || response.has_focus() {
        ui.painter()
            .rect_filled(rect, tokens.radius_control, tokens.accent_soft);
    }
    if response.has_focus() && !item.current {
        ui.painter().rect_stroke(
            rect,
            tokens.radius_control,
            egui::Stroke::new(2.0_f32, tokens.accent),
            egui::StrokeKind::Inside,
        );
    }
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + tokens.unit * 1.3, rect.center().y),
        egui::vec2(tokens.unit * RAIL_ICON, tokens.unit * RAIL_ICON),
    );
    icon(ui, item.drawing, item.label, ink, icon_rect);
    ui.painter().text(
        egui::pos2(icon_rect.right() + tokens.unit * 0.6, rect.center().y),
        egui::Align2::LEFT_CENTER,
        item.label,
        egui::FontId::proportional(tokens.text_body),
        ink,
    );
    if let Some(shortcut) = item.shortcut {
        ui.painter().text(
            egui::pos2(rect.right() - tokens.unit * 0.6, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            shortcut,
            egui::FontId::proportional(tokens.text_small),
            if item.current {
                tokens.on_accent
            } else {
                tokens.ink_muted
            },
        );
    }
    let current = item.current;
    response.widget_info(|| {
        if current {
            egui::WidgetInfo::selected(egui::WidgetType::Button, true, true, item.label)
        } else {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, item.label)
        }
    });
    let activated = activated(ui, &response);
    match item.shortcut {
        Some(shortcut) => response.on_hover_text(format!("{} ({shortcut})", item.label)),
        None => response.on_hover_text(item.label),
    };
    activated
}

/// The small heading of a rail group.
fn nav_heading(ui: &mut egui::Ui, tokens: &Tokens, text: &str) {
    ui.add_space(tokens.unit * 0.6);
    ui.horizontal(|ui| {
        ui.add_space(tokens.unit * 0.5);
        ui.label(
            egui::RichText::new(text)
                .size(tokens.text_small)
                .color(tokens.ink_dim),
        );
    });
    ui.add_space(tokens.unit * 0.2);
}
