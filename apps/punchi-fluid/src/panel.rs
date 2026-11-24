// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The chrome every panel shares: the fold, the header and the rail.
//!
//! A panel is chrome, never a document: hiding it, widening it and its keyboard path are
//! view state the window remembers for the session, and **Ver ▸ Restablecer vista** puts it
//! back. The fold lasts the theme's `motion_ms` and is instant when the theme reports
//! reduced motion, which it does by carrying a zero duration. The catalog and the right dock
//! share this module and keep their own content and width bounds.

use eframe::egui;
use punchi_fluid_i18n::Language;

use crate::theme::Tokens;

/// The rail is one unit wide, never below the 24 px the interface specification gives it as
/// a target.
pub const MIN_RAIL: f32 = 24.0;

/// Which edge the panel is docked to: it decides the side the panel folds towards and the
/// pointer direction that widens it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Left,
    Right,
}

impl Edge {
    /// A horizontal drag that pulls the panel away from its edge widens it.
    fn widen(self, delta_x: f32) -> f32 {
        match self {
            Self::Left => delta_x,
            Self::Right => -delta_x,
        }
    }
}

/// What the last frame told the auto-hide timers about one panel: the pointer rested on the
/// rail while the panel was closed, the pointer was inside its surface, or the keyboard focus
/// was. The shell reads it through [`Panel::probe`] and owns the timers.
#[derive(Debug, Clone, Copy, Default)]
pub struct Probe {
    pub rail_hovered: bool,
    pub pointer_inside: bool,
    pub focus_inside: bool,
}

/// What one panel's chrome says and the bounds it keeps, so the two panels differ in data
/// instead of in code.
#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub edge: Edge,
    /// The panel's name in egui and the salt of its animation, unique in the window.
    pub id: &'static str,
    pub default_width: f32,
    pub min_width: f32,
    pub max_width: f32,
    /// The name the accessibility layer reads for the width grip.
    pub width_label: &'static str,
    pub show_label: &'static str,
    pub hide_label: &'static str,
    /// The keyboard path shown in both controls' hints.
    pub shortcut: &'static str,
    /// The panel frame's horizontal inner margin, in points, as each panel had it.
    pub margin: i8,
}

/// One panel's view state: whether it is open, how wide it is and how far the fold has gone.
#[derive(Debug, Clone)]
pub struct Panel {
    edge: Edge,
    id: &'static str,
    open: bool,
    width: f32,
    /// How far the fold has gone, `0` closed and `1` open, kept so the tests can see the
    /// animation without a screenshot.
    shade: f32,
    default_width: f32,
    min_width: f32,
    max_width: f32,
    width_label: &'static str,
    show_label: &'static str,
    hide_label: &'static str,
    shortcut: &'static str,
    /// The visible language the panel's own labels resolve with.
    language: Language,
    margin: i8,
    /// What the last frame told the auto-hide timers, refreshed on every draw.
    probe: Probe,
}

impl Panel {
    #[must_use]
    pub fn new(settings: Settings) -> Self {
        Self {
            edge: settings.edge,
            id: settings.id,
            open: true,
            width: settings.default_width,
            shade: 1.0,
            default_width: settings.default_width,
            min_width: settings.min_width,
            max_width: settings.max_width,
            width_label: settings.width_label,
            show_label: settings.show_label,
            hide_label: settings.hide_label,
            shortcut: settings.shortcut,
            language: Language::English,
            margin: settings.margin,
            probe: Probe::default(),
        }
    }

    /// Sets the language the panel's own labels resolve with.
    pub fn set_language(&mut self, language: Language) {
        self.language = language;
    }

    fn label(&self, key: &'static str) -> &'static str {
        self.language.text(key)
    }

    #[must_use]
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// What the last frame told the auto-hide timers: the shell reads it after the panels drew.
    #[must_use]
    pub fn probe(&self) -> Probe {
        self.probe
    }

    #[cfg(test)]
    pub fn probe_for_test(&mut self, rail_hovered: bool, pointer_inside: bool, focus_inside: bool) {
        self.probe = Probe {
            rail_hovered,
            pointer_inside,
            focus_inside,
        };
    }

    #[must_use]
    pub fn width(&self) -> f32 {
        self.width
    }

    /// The width the panel asks for when it floats over the sheet: the width the reader gave
    /// it, inside the drag bounds.
    #[must_use]
    pub fn floating_width(&self) -> f32 {
        self.width.clamp(self.min_width, self.max_width)
    }

    /// The panel frame's horizontal inner margin, in points, so the floating surface keeps
    /// the content aligned with the docked one.
    #[must_use]
    pub fn margin(&self) -> i8 {
        self.margin
    }

    #[cfg(test)]
    #[must_use]
    pub fn shade(&self) -> f32 {
        self.shade
    }

    /// Hides the panel, or brings it back at the width it had.
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    /// **Restablecer vista**: open again at the preferred width.
    pub fn reset(&mut self) {
        self.open = true;
        self.width = self.default_width;
    }

    /// Puts the panel back where a settings file left it: open or folded at the remembered
    /// width, already folded instead of animating from the default on the first frame.
    pub fn restore(&mut self, open: bool, width: f32) {
        self.open = open;
        self.set_width(width);
        self.shade = if open { 1.0 } else { 0.0 };
    }

    /// The width the settings and the layout ask for, clamped to the panel's own bounds: an
    /// out-of-range number is a bug or a corrupt file, never a panel wider than its drag bound.
    pub fn set_width(&mut self, width: f32) {
        if width.is_finite() {
            self.width = width.clamp(self.min_width, self.max_width);
        }
    }

    /// A width drag; clamped so the panel never takes the sheet's floor.
    pub fn resize_by(&mut self, delta: f32) {
        self.width = (self.width + delta).clamp(self.min_width, self.max_width);
    }

    /// Draws the panel (or its rail) at `docked_width` and returns what its content asked
    /// for, if anything. The layout decides that width: the wide mode leaves the reader's
    /// own width, the medium one gives the minimum.
    ///
    /// The panel is always one `SidePanel`: its width is the fold's interpolation, so the
    /// sheet takes the room in the same gesture instead of after it.
    pub fn show<R>(
        &mut self,
        context: &egui::Context,
        tokens: &Tokens,
        title: &str,
        subtitle: Option<&str>,
        docked_width: f32,
        body: impl FnOnce(&mut egui::Ui) -> R,
    ) -> Option<R> {
        let rail = tokens.unit.max(MIN_RAIL);
        #[allow(
            clippy::cast_precision_loss,
            reason = "a motion token in milliseconds is far below the f32 integer range"
        )]
        let motion = tokens.motion_ms as f32 / 1_000.0;
        self.probe = Probe::default();
        let animation = egui::Id::new(("panel.fold", self.id));
        self.shade = context.animate_bool_with_time(animation, self.open, motion);
        let target = docked_width.max(rail);
        let width = rail + (target - rail) * self.shade;
        let panel = match self.edge {
            Edge::Left => egui::SidePanel::left(self.id),
            Edge::Right => egui::SidePanel::right(self.id),
        };
        let mut result = None;
        let shown = panel
            .exact_width(width)
            .resizable(false)
            .frame(
                egui::Frame::default()
                    .fill(tokens.panel)
                    .inner_margin(egui::Margin::symmetric(self.margin, 0)),
            )
            .show(context, |ui| {
                if self.shade <= 0.001 && !self.open {
                    if self.rail(ui, tokens) {
                        self.open();
                    }
                    return;
                }
                let hide = self.header(ui, tokens, title, subtitle);
                if hide {
                    self.close();
                }
                if self.shade < 0.999 {
                    return;
                }
                result = Some(body(ui));
            });
        self.observe(context, shown.response.rect);
        result
    }

    /// Draws only the rail, for the compact shell, and reports whether the reader asked for
    /// the panel. The rail does not change the open flag: the drawer is the shell's state.
    pub fn show_rail(&mut self, context: &egui::Context, tokens: &Tokens) -> bool {
        let rail = tokens.unit.max(MIN_RAIL);
        let panel = match self.edge {
            Edge::Left => egui::SidePanel::left(self.id),
            Edge::Right => egui::SidePanel::right(self.id),
        };
        self.probe = Probe::default();
        let mut opened = false;
        let shown = panel
            .exact_width(rail)
            .resizable(false)
            .frame(
                egui::Frame::default()
                    .fill(tokens.panel)
                    .inner_margin(egui::Margin::symmetric(self.margin, 0)),
            )
            .show(context, |ui| {
                opened = self.rail(ui, tokens);
            });
        self.observe(context, shown.response.rect);
        opened
    }

    /// Draws the header, the rule and the body in a floating surface (the compact drawer).
    /// Reports the body's answer and whether the hide control asked to close the panel.
    pub fn show_body<R>(
        &mut self,
        ui: &mut egui::Ui,
        tokens: &Tokens,
        title: &str,
        subtitle: Option<&str>,
        body: impl FnOnce(&mut egui::Ui) -> R,
    ) -> (R, bool) {
        self.probe = Probe::default();
        let hide = self.header(ui, tokens, title, subtitle);
        let result = body(ui);
        self.observe(ui.ctx(), ui.max_rect());
        (result, hide)
    }

    /// Records what the pointer and the keyboard did with a surface the shell draws itself
    /// (the compact drawer): the auto-hide timers read it like a docked panel's.
    pub fn observe_surface(&mut self, context: &egui::Context, rect: egui::Rect) {
        self.observe(context, rect);
    }

    /// Records what the pointer and the keyboard did with the panel this frame. The rail flag
    /// is set by [`Panel::rail`] itself; this fills the surface and the focus in.
    fn observe(&mut self, context: &egui::Context, rect: egui::Rect) {
        self.probe.pointer_inside = context
            .pointer_hover_pos()
            .is_some_and(|position| rect.contains(position));
        self.probe.focus_inside = focus_inside(context, rect);
    }

    /// The rail the closed panel leaves at the edge: one unit wide, and a button that brings
    /// the panel back. The width is the panel's, so the control is a target not a sliver.
    /// Returns whether the reader asked for the panel.
    fn rail(&mut self, ui: &mut egui::Ui, tokens: &Tokens) -> bool {
        ui.add_space(tokens.unit * 0.5);
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), (tokens.unit * 1.6).max(30.0)),
            egui::Sense::hover(),
        );
        let response = focusable(ui, rect, ui.make_persistent_id((self.id, "rail")));
        self.probe.rail_hovered = response.hovered();
        if ui.is_rect_visible(rect) {
            if response.hovered() || response.has_focus() {
                ui.painter()
                    .rect_filled(rect, tokens.radius_small, tokens.accent_soft);
            }
            draw_panel_glyph(ui, rect, tokens.ink, tokens.accent);
        }
        let show = self.label(self.show_label);
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, show));
        let activated = activated(ui, &response);
        response.on_hover_text(format!("{} ({})", show, self.shortcut));
        activated
    }

    /// The header: the title, the control that hides the panel and, on the inner edge, the
    /// grip that widens it. Returns whether the hide control was used.
    fn header(
        &mut self,
        ui: &mut egui::Ui,
        tokens: &Tokens,
        title: &str,
        subtitle: Option<&str>,
    ) -> bool {
        ui.add_space(tokens.unit * 0.35);
        let mut hide = false;
        ui.horizontal(|ui| {
            if self.edge == Edge::Right {
                self.grip(ui, tokens);
            }
            ui.label(
                egui::RichText::new(title)
                    .size(tokens.text_label)
                    .strong()
                    .color(tokens.ink),
            );
            if let Some(subtitle) = subtitle {
                ui.add_space(tokens.unit * 0.35);
                ui.label(
                    egui::RichText::new(subtitle)
                        .size(tokens.text_small)
                        .color(tokens.ink_muted),
                );
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                hide = self.hide_control(ui, tokens);
                if self.edge == Edge::Left {
                    self.grip(ui, tokens);
                }
            });
        });
        ui.add_space(tokens.unit * 0.25);
        let line = ui.max_rect();
        ui.painter().line_segment(
            [
                egui::pos2(line.left(), ui.cursor().top()),
                egui::pos2(line.right(), ui.cursor().top()),
            ],
            egui::Stroke::new(1.0_f32, tokens.line),
        );
        ui.add_space(tokens.unit * 0.25);
        hide
    }

    /// The control that hides the panel: the project glyph, its name and its keyboard hint.
    fn hide_control(&mut self, ui: &mut egui::Ui, tokens: &Tokens) -> bool {
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(tokens.unit, tokens.unit), egui::Sense::hover());
        let button = focusable(ui, rect, ui.make_persistent_id((self.id, "hide")));
        if ui.is_rect_visible(rect) {
            if button.hovered() || button.has_focus() {
                ui.painter()
                    .rect_filled(rect, tokens.radius_small, tokens.accent_soft);
            }
            draw_panel_glyph(ui, rect, tokens.ink, tokens.accent);
        }
        let hide = self.label(self.hide_label);
        button.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, hide));
        let activated = activated(ui, &button);
        button.on_hover_text(format!("{} ({})", hide, self.shortcut));
        activated
    }

    /// The grip on the panel's inner edge: a horizontal drag changes the remembered width.
    fn grip(&mut self, ui: &mut egui::Ui, tokens: &Tokens) {
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(9.0, (tokens.unit * 1.4).max(24.0)),
            egui::Sense::hover(),
        );
        let grip = focusable(ui, rect, ui.make_persistent_id((self.id, "grip")));
        let width = self.label(self.width_label);
        grip.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Other, true, width));
        if grip.hovered() || grip.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        }
        if grip.dragged() {
            self.resize_by(self.edge.widen(grip.drag_delta().x));
        }
        grip.on_hover_text(self.label("panel.resize"));
    }
}

/// The project's panel glyph: two rectangles with the side that folds marked. The desktop
/// icon themes have no panel glyph, and a label alone is not a control.
fn draw_panel_glyph(ui: &egui::Ui, rect: egui::Rect, ink: egui::Color32, accent: egui::Color32) {
    // Every measure is a fraction of the control, so the glyph follows the theme's unit
    // instead of carrying a size of its own.
    let size = rect.size().min_elem() * 0.72;
    let body = egui::Rect::from_center_size(rect.center(), egui::vec2(size, size * 0.8));
    ui.painter().rect_stroke(
        body,
        size * 0.14,
        egui::Stroke::new(1.0_f32, ink),
        egui::StrokeKind::Inside,
    );
    let folded = egui::Rect::from_min_max(
        egui::pos2(body.left() + size * 0.12, body.top() + size * 0.14),
        egui::pos2(body.left() + size * 0.32, body.bottom() - size * 0.14),
    );
    ui.painter().rect_filled(folded, size * 0.06, accent);
}

/// A painted control gets the keyboard focus: `Tab` reaches it and the accessibility layer
/// finds it by its name.
fn focusable(ui: &mut egui::Ui, rect: egui::Rect, id: egui::Id) -> egui::Response {
    let response = ui.interact(rect, id, egui::Sense::click_and_drag());
    let layer = ui.layer_id();
    ui.memory_mut(|memory| memory.interested_in_focus(id, layer));
    response
}

/// Whether the keyboard focus sits on a widget inside `rect`: the auto-hide timers suspend
/// while the reader is typing in the panel.
fn focus_inside(context: &egui::Context, rect: egui::Rect) -> bool {
    context
        .memory(egui::Memory::focused)
        .and_then(|id| context.read_response(id))
        .is_some_and(|focused| rect.contains(focused.rect.center()))
}

/// Whether the reader activated the control with the pointer or with `Enter`/`Space` while
/// it has the keyboard.
fn activated(ui: &egui::Ui, response: &egui::Response) -> bool {
    response.clicked()
        || (response.has_focus()
            && ui.input(|input| {
                input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Space)
            }))
}

#[cfg(test)]
mod tests;
