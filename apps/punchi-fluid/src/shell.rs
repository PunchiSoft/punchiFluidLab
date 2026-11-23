// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The shell's frame: the window's geometry, the layout mode, the regions and the drawer.
//!
//! `docs/ui-ux.md` §2.2: the mode follows from the room the window has and the content
//! measurements in [`crate::layout`]; the sheet is the region that never gives way. The wide
//! and medium modes dock both panels; the compact one folds them to their rails and opens
//! them as a drawer over the drawing area, dismissed by `Esc` or a click outside. The first
//! frame asks the screen for its size and sends the geometry to the viewport, so no resolution
//! is ever guessed or remembered as a fact.

use std::path::Path;
use std::time::Instant;

use eframe::egui;

use crate::Window;
use crate::inspector::Action;
use crate::layout::LayoutMode;
use crate::scene;
use crate::sidebar::Sidebar;
use crate::surface::{EguiSurface, ScreenInk};
use crate::window_state::{self, Geometry};
use punchi_fluid_draw::symbol::Ink;
use punchi_fluid_draw::{Pt, Rect, draw_placed};
use punchi_fluid_export::sheet;

/// Which panel the compact drawer is showing. One at a time: the drawing stays visible
/// beside it and the other rail keeps its control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Drawer {
    Catalog,
    Inspector,
}

impl Drawer {
    /// The salt of the drawer's floating surface, unique per panel.
    fn key(self) -> &'static str {
        match self {
            Self::Catalog => "drawer.catalog",
            Self::Inspector => "drawer.inspector",
        }
    }
}

/// What the drawer's body asked the window to do, after it closes.
enum DrawerAction {
    Add(String),
    Edit(Action),
}

/// The shell's per-session geometry state: the mode, the drawer, the store and the last
/// geometry the viewport reported.
#[derive(Debug)]
pub(crate) struct Shell {
    /// The mode derived from the room the window has; the field is what the tests and the
    /// layout read, never a remembered screen list.
    pub(crate) mode: LayoutMode,
    /// The panel the compact mode opened over the sheet, if any.
    pub(crate) drawer: Option<Drawer>,
    state: window_state::Store,
    restored: Option<Geometry>,
    /// The size and place the window has when it is not maximized, so maximizing keeps them.
    normal: Option<Geometry>,
    /// The last geometry the viewport reported, for the close path.
    live: Option<Geometry>,
    /// Whether the first geometry has been sent; the screen is only known after one frame.
    applied: bool,
    /// Whether the widths a settings file left still have to be clamped to the mode the
    /// window opened in.
    pub(crate) restore_pending: bool,
}

impl Shell {
    /// A shell with no remembered geometry: the tests' window.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::with_store(window_state::Store::at(None))
    }

    /// A shell that carries the geometry the reader's last session left.
    #[must_use]
    pub(crate) fn with_store(state: window_state::Store) -> Self {
        let restored = state.remembered().copied();
        Self {
            mode: LayoutMode::Wide,
            drawer: None,
            state,
            restored,
            normal: None,
            live: None,
            applied: false,
            restore_pending: false,
        }
    }

    /// Writes a pending geometry now: the close path, so the last move survives a window
    /// closed before the settle delay.
    fn flush(&mut self) {
        if let Some(live) = self.live {
            let geometry = window_state::remembered_geometry(live, self.normal.as_ref());
            self.state.remember(geometry);
        }
    }
}

impl Window {
    /// The frame: geometry, keyboard, the document's life, and then the page or the shell.
    pub(crate) fn frame(&mut self, context: &egui::Context) {
        self.update_layout_mode(context);
        self.track_geometry(context);
        self.sync_commands(context);
        self.tick(context);
        self.lifecycle_frame(context);
        // Inicio is the whole window: the ribbon, the status bar and the panels below are the
        // sheet's chrome, and the start page hides them while it is open.
        if self.start_open {
            self.shell.drawer = None;
            // The panels are not on screen: no timer may fire behind the start page.
            self.auto_hide.cancel_all();
            self.show_start(context);
            self.show_files(context);
            self.show_preferences(context);
            self.show_page_setup(context);
            self.show_block_editor(context);
            self.show_circuit_dialogs(context);
            self.show_export_options(context);
            self.show_check_panel(context);
            return;
        }
        self.show_ribbon(context);
        self.show_status(context);
        self.show_regions(context);
        self.show_sheet(context);
        // The auto-hide timers read the pointer and the focus the panels reported this frame,
        // and the settings are noted after both, so a fold or a dwell lands in the file.
        let now = Instant::now();
        self.drive_auto_hide(context, now);
        self.observe_panel_settings(now);
        if self.settings.is_pending() {
            // A fold or a width has stopped moving: wake once more to write it, even if the
            // reader never touches the window again.
            context.request_repaint_after(crate::settings::SETTLE);
        }
        // The dialog is the last surface drawn, so it is in front of the sheet and catches the
        // keyboard before the sheet's shortcuts do.
        self.show_files(context);
        self.show_preferences(context);
        self.show_page_setup(context);
        self.show_block_editor(context);
        self.show_circuit_dialogs(context);
        self.show_export_options(context);
        self.show_check_panel(context);
    }

    /// The mode the room the window has calls for. The layout is a consequence of the limits,
    /// so this is the only place that decides it, and the field records the answer.
    pub(crate) fn update_layout_mode(&mut self, context: &egui::Context) {
        self.shell.mode = LayoutMode::from_available(context.screen_rect().width());
        // The remembered widths are clamped once, against the mode the window really opened
        // in: the first frame still holds the provisional size the viewport reports.
        self.restore_panels_to_mode(context);
    }

    /// The mode, for the commands: the keyboard path has to know which presentation a panel
    /// has before it toggles it.
    pub(crate) fn layout_mode(&self) -> LayoutMode {
        self.shell.mode
    }

    /// Opens a panel over the drawing area in the compact mode, or the docked panel in the
    /// wide and medium ones.
    pub(crate) fn open_drawer(&mut self, drawer: Drawer) {
        match drawer {
            Drawer::Catalog => self.sidebar.open(),
            Drawer::Inspector => self.inspector.open(),
        }
        self.shell.drawer = Some(drawer);
    }

    /// `Ctrl+B` and `Ctrl+J` in the compact mode: the drawer replaces the docked panel.
    pub(crate) fn toggle_drawer(&mut self, drawer: Drawer) {
        if self.shell.drawer == Some(drawer) {
            self.shell.drawer = None;
        } else {
            self.open_drawer(drawer);
        }
    }

    /// **Restablecer vista** closes an open drawer with the panels and the view.
    pub(crate) fn close_drawer(&mut self) {
        self.shell.drawer = None;
    }

    /// Writes the window geometry on the close path.
    pub(crate) fn flush_window_state(&mut self) {
        self.shell.flush();
    }

    /// Sends the first geometry once the screen is known, then notes every change. A
    /// maximized window keeps the normal size it had, and a minimized one is not a size.
    fn track_geometry(&mut self, context: &egui::Context) {
        if !self.shell.applied {
            let Some(screen) = monitor_size(context) else {
                // The frame after creation reports the monitor; wait for it instead of
                // guessing a size the desktop would have to correct.
                context.request_repaint();
                return;
            };
            let geometry = window_state::opening_geometry(
                screen,
                self.theme.tokens.unit,
                self.shell.restored.as_ref(),
            );
            // The floor goes first: a screen below 900 × 600 must clamp the window to its own
            // room, never to a minimum the monitor cannot hold.
            window_state::apply_minimum(
                context,
                window_state::minimum(screen, self.theme.tokens.unit),
            );
            window_state::apply(context, &geometry);
            self.shell.normal = Some(Geometry {
                maximized: false,
                ..geometry
            });
            self.shell.applied = true;
            // The commands take effect on the next frame: this frame's viewport still holds
            // the provisional size, which must not overwrite the remembered one.
            return;
        }
        let viewport = context.input(|input| input.viewport().clone());
        if viewport.minimized == Some(true) {
            return;
        }
        let size = viewport
            .inner_rect
            .map_or_else(|| context.screen_rect().size(), |rect| rect.size());
        let live = Geometry {
            size,
            position: viewport.outer_rect.map(|rect| rect.min),
            maximized: viewport.maximized.unwrap_or(false),
        };
        if !live.maximized {
            self.shell.normal = Some(Geometry {
                maximized: false,
                ..live
            });
        }
        self.shell.live = Some(live);
        let normal = self.shell.normal;
        self.shell
            .state
            .observe(live, normal.as_ref(), Instant::now());
    }

    /// The ribbon, its commands and the tab strip.
    fn show_ribbon(&mut self, context: &egui::Context) {
        let palette = crate::ribbon::Palette::from_theme(&self.theme.tokens);
        egui::TopBottomPanel::top("ribbon").show(context, |ui| {
            self.ribbon.show(ui, &palette, self.recents.entries());
        });
        if let Some(command) = self.ribbon.take() {
            self.dispatch(command);
        }
    }

    /// The status bar: the sheet, the magnification, the frame's time and the document.
    fn show_status(&mut self, context: &egui::Context) {
        let (width, height) = self.page.size_mm();
        let ink = self.theme.tokens.ink;
        let last = self.ribbon.last.clone();
        let zoom = self.view.zoom();
        let magnification = self.magnification;
        let source = self
            .path
            .as_deref()
            .and_then(Path::file_name)
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("sin título");
        let notice = self
            .notice
            .as_deref()
            .map(|notice| format!(" · {notice}"))
            .unwrap_or_default();
        let simulation = self.simulation_line();
        egui::TopBottomPanel::bottom("status").show(context, |ui| {
            ui.label(
                egui::RichText::new(format!(
                    "hoja {width:.0}×{height:.0} mm · {:.0} % · {magnification:.2} px/mm · {:.2} ms de dibujo · último: {last} · {source}{notice}{simulation}",
                    zoom * 100.0, self.frame_ms
                ))
                .color(ink),
            );
        });
    }

    /// Both regions below the ribbon, in the presentation the mode asks for.
    fn show_regions(&mut self, context: &egui::Context) {
        if self.shell.mode == LayoutMode::Compact {
            let tokens = &self.theme.tokens;
            let catalog = self.sidebar.show_rail(context, tokens);
            let inspector = self.inspector.show_rail(context, tokens);
            if catalog {
                self.open_drawer(Drawer::Catalog);
            }
            if inspector {
                self.open_drawer(Drawer::Inspector);
            }
            if let Some(drawer) = self.shell.drawer {
                self.show_drawer(context, drawer);
            }
            return;
        }
        self.show_catalog(context);
        self.show_inspector(context);
    }

    /// The compact drawer: one panel over the drawing area, with the sheet still behind it.
    /// It closes with `Esc`, with a click outside and with its own hide control; opening one
    /// panel replaces the other.
    fn show_drawer(&mut self, context: &egui::Context, drawer: Drawer) {
        let tokens = self.theme.tokens.clone();
        let area = context.available_rect();
        if area.width() <= 0.0 || area.height() <= 0.0 {
            return;
        }
        let (width, margin) = match drawer {
            Drawer::Catalog => (self.sidebar.floating_width(), self.sidebar.margin()),
            Drawer::Inspector => (self.inspector.floating_width(), self.inspector.margin()),
        };
        let width = width.clamp(0.0, area.width());
        let position = match drawer {
            Drawer::Catalog => egui::pos2(area.left(), area.top()),
            Drawer::Inspector => egui::pos2(area.right() - width, area.top()),
        };
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "the radius token is 3-8 points, far inside the shadow's 0-255 blur"
        )]
        let blur = tokens.radius.round() as u8;
        let frame = egui::Frame::default()
            .fill(tokens.panel)
            .inner_margin(egui::Margin::symmetric(margin, 0))
            .stroke(egui::Stroke::new(1.0_f32, tokens.line))
            .shadow(egui::epaint::Shadow {
                offset: [1, 2],
                blur,
                spread: 0,
                color: tokens.shadow,
            });
        let id = egui::Id::new(drawer.key());
        let surface = egui::Area::new(id)
            .kind(egui::UiKind::Modal)
            .sense(egui::Sense::hover())
            .fixed_pos(position)
            .order(egui::Order::Foreground)
            .interactable(true);
        let height = area.height();
        let response = egui::Modal::new(id)
            .area(surface)
            .backdrop_color(egui::Color32::TRANSPARENT)
            .frame(frame)
            .show(context, |ui| {
                ui.set_min_width(width);
                ui.set_min_height(height);
                match drawer {
                    Drawer::Catalog => {
                        let subtitle =
                            format!("{} componentes", Sidebar::entry_count(&self.catalog));
                        let (chosen, hide) =
                            self.sidebar
                                .show_drawer(ui, &tokens, &subtitle, &self.catalog);
                        (chosen.map(DrawerAction::Add), hide)
                    }
                    Drawer::Inspector => {
                        let (action, hide) = self.inspector.show_drawer(
                            ui,
                            &tokens,
                            &self.document,
                            &self.catalog,
                            self.model.as_ref(),
                            &self.selected,
                            self.selected_pipe,
                        );
                        (action.map(DrawerAction::Edit), hide)
                    }
                }
            });
        let surface = response.response.rect;
        match drawer {
            Drawer::Catalog => self.sidebar.observe_surface(context, surface),
            Drawer::Inspector => self.inspector.observe_surface(context, surface),
        }
        let should_close = response.should_close();
        let (action, hide) = response.inner;
        if hide {
            match drawer {
                Drawer::Catalog => self.sidebar.close(),
                Drawer::Inspector => self.inspector.close(),
            }
            self.shell.drawer = None;
        } else if should_close {
            self.shell.drawer = None;
        } else if let Some(action) = action {
            self.shell.drawer = None;
            match action {
                DrawerAction::Add(catalog_id) => self.add_component(&catalog_id),
                DrawerAction::Edit(action) => self.apply_inspector(action),
            }
        }
    }

    /// The sheet, its gestures and its drawing: the last region, the one that never gives way.
    fn show_sheet(&mut self, context: &egui::Context) {
        let (width, height) = self.page.size_mm();
        let page_mm = egui::vec2(width, height);
        egui::CentralPanel::default().show(context, |ui| {
            let area = ui.available_rect_before_wrap();
            let sheet = ui.allocate_rect(area, egui::Sense::click_and_drag());
            sheet.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Image, true, "hoja de dibujo")
            });
            self.handle_view(ui, &sheet, area, page_mm);
            let placement = self.view.placement(area, page_mm);
            self.magnification = placement.scale;
            let page_rect = self.page.rect();
            let page_min = egui::vec2(page_rect.min.x, page_rect.min.y);
            self.handle_edit(&sheet, &placement, page_min);
            let origin = placement.origin
                - egui::vec2(
                    page_rect.min.x * placement.scale,
                    page_rect.min.y * placement.scale,
                );
            let mut drawing = EguiSurface::new(context, origin, placement.scale)
                .with_screen_ink(ScreenInk::from_tokens(&self.theme.tokens));
            let mut scene = scene::build(
                &self.document,
                &self.catalog,
                self.document.active_circuit(),
                self.model.as_ref(),
            );
            // The drag is previewed on the drawing, not written to the document: the document
            // only changes once, when the button comes up.
            if let Some(drag) = &self.drag {
                let delta = drag.delta;
                for placed in &mut scene.placed {
                    if drag.starts.iter().any(|(index, _)| *index == placed.index) {
                        placed.rect = Rect::from_min_size(
                            Pt::new(placed.rect.min.x + delta.x, placed.rect.min.y + delta.y),
                            placed.rect.max.x - placed.rect.min.x,
                            placed.rect.max.y - placed.rect.min.y,
                        );
                    }
                }
            }
            let started = Instant::now();
            sheet::draw(&mut drawing, &self.page, &self.block);
            draw_placed(
                &mut drawing,
                &scene.placed,
                &scene.pipes,
                &self.catalog,
                &Ink::print(),
                &self.selected,
            );
            self.frame_ms = started.elapsed().as_secs_f64() * 1000.0;
            let tokens = &self.theme.tokens;
            let painter = ui.painter().with_clip_rect(ui.max_rect());
            // Three planes tell the sheet from the chrome: the canvas is the sunken
            // workspace, the paper is the raised card, and one shadow keeps the card off
            // the canvas. The paper itself is the drawing's first call, so the print
            // surface paints white and the screen maps that white to `paper`.
            painter.rect_filled(area.shrink(tokens.unit * 0.5), 0.0, tokens.surface_sunken);
            let paper = egui::Rect::from_min_size(
                placement.origin,
                egui::vec2(
                    page_rect.width() * placement.scale,
                    page_rect.height() * placement.scale,
                ),
            );
            painter.rect_filled(paper.translate(egui::vec2(0.0, 3.0)), 0.0, tokens.shadow);
            painter.rect_stroke(
                paper,
                0.0,
                egui::Stroke::new(1.0_f32, tokens.line_strong),
                egui::StrokeKind::Inside,
            );
            painter.extend(drawing.shapes);
            self.highlight_selected_pipe(ui, origin, placement.scale);
        });
    }
}

/// The monitor's size in points, when the desktop reports it. A window created on Wayland
/// knows the size but not its place; both are asked, never assumed.
fn monitor_size(context: &egui::Context) -> Option<egui::Vec2> {
    context
        .input(|input| input.viewport().monitor_size)
        .filter(|size| size.is_finite() && size.x > 0.0 && size.y > 0.0)
}

#[cfg(test)]
mod tests;
