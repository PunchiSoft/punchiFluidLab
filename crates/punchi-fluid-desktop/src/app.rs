// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Application state and layout. The shell owns the window; the crates own the
//! work.

use std::path::PathBuf;

use eframe::egui;
use punchi_fluid_catalog::Catalog;
use punchi_fluid_solver::pneumatic::PneumaticSystem;
use punchi_fluid_theme::{Scheme, SchemeWatcher, Theme};
use punchi_fluid_ui::SchematicCanvas;

use crate::export_panel::{self, ExportState};
use punchi_fluid_ui::menu::{self, MenuState};
use punchi_fluid_ui::ribbon::{self, Action, RibbonInfo, RibbonState};
use punchi_fluid_ui::shell::{self, Mode, StatusInfo};
use punchi_fluid_ui::sidebar::{self, SidebarEvent, SidebarState};

/// Playback speed factors offered by the transport.
const SPEEDS: &[f64] = &[0.05, 0.1, 0.25, 0.5, 1.0];

/// Longest wall-clock step fed to the simulation, so a stalled window does not
/// jump the circuit forward.
const MAX_FRAME_STEP: f64 = 0.02;

/// Where the palette comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThemeChoice {
    /// Follow the desktop portal, then `kdeglobals`.
    System,
    Light,
    Dark,
}

impl ThemeChoice {
    fn scheme(self, portal: Scheme) -> Scheme {
        match self {
            Self::System => portal,
            Self::Light => Scheme::Light,
            Self::Dark => Scheme::Dark,
        }
    }

    /// `PUNCHI_FLUID_THEME=light|dark` pins the palette, for screenshots and demos
    /// on a machine whose desktop cannot be changed. Anything else follows the
    /// desktop.
    fn from_env() -> Self {
        match std::env::var("PUNCHI_FLUID_THEME").as_deref() {
            Ok("light") => Self::Light,
            Ok("dark") => Self::Dark,
            _ => Self::System,
        }
    }
}

/// Which optional regions are visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ViewState {
    grid: bool,
    sidebar: bool,
    status_bar: bool,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            grid: true,
            sidebar: true,
            status_bar: true,
        }
    }
}

/// Top-level application.
#[derive(Debug)]
pub struct FluidLabApp {
    catalog: Catalog,
    user_dir: PathBuf,
    system: Option<PneumaticSystem>,
    canvas: SchematicCanvas,
    playing: bool,
    speed: f64,
    status: String,
    theme: Theme,
    theme_choice: ThemeChoice,
    portal_scheme: Scheme,
    watcher: SchemeWatcher,
    ribbon: RibbonState,
    menu: MenuState,
    sidebar: SidebarState,
    mode: Mode,
    view: ViewState,
    help_open: bool,
    preview: bool,
    export: ExportState,
    actuation: Actuation,
}

/// The valve actuator as a hand would use it.
#[derive(Debug, Default)]
struct Actuation {
    /// The demo holds the actuator on its own.
    demo: bool,
    /// A source asked for the spool during this frame.
    requested: bool,
    /// The spool is applied, so the app owes the valve a release.
    holding: bool,
}

impl FluidLabApp {
    /// Builds the application, applying the desktop theme before the first frame.
    pub fn new(context: &eframe::CreationContext<'_>) -> Self {
        let theme_choice = ThemeChoice::from_env();
        let theme = Theme::detect_with(theme_choice.scheme(Scheme::NoPreference));
        theme.apply(&context.egui_ctx);

        let user_dir = punchi_fluid_catalog::user_catalog_dir().unwrap_or_default();
        let demo = std::env::var_os("PUNCHI_FLUID_DEMO").is_some();
        let mut app = Self {
            catalog: Catalog::builtin(),
            user_dir,
            system: None,
            canvas: SchematicCanvas::default(),
            playing: demo,
            speed: 0.25,
            status: String::new(),
            theme,
            theme_choice,
            portal_scheme: Scheme::NoPreference,
            watcher: SchemeWatcher::spawn(),
            ribbon: RibbonState {
                language: MenuState::detect().language,
                ..RibbonState::default()
            },
            menu: MenuState::detect(),
            sidebar: SidebarState::default(),
            mode: Mode::Simulate,
            view: ViewState::default(),
            help_open: false,
            preview: std::env::var_os("PUNCHI_FLUID_PREVIEW").is_some(),
            export: ExportState::new(),
            actuation: Actuation {
                demo,
                ..Actuation::default()
            },
        };
        app.reload_catalog();
        if demo {
            app.actuate_valve();
        }
        app
    }

    /// Rebuilds the catalog from the binary and the user directory, then the
    /// circuit on top of it.
    fn reload_catalog(&mut self) {
        self.catalog = Catalog::builtin().with_user_dir(&self.user_dir);
        self.restart_circuit();
    }

    /// Throws the simulation away and starts again from the catalog.
    fn restart_circuit(&mut self) {
        let problems = self.catalog.problems().len();
        let loaded = self.catalog.len();
        self.playing = false;
        match PneumaticSystem::from_catalog(&self.catalog) {
            Ok(system) => {
                self.system = Some(system);
                self.status = if problems == 0 {
                    format!("{loaded} definitions loaded")
                } else {
                    format!("{loaded} definitions loaded, {problems} file(s) skipped")
                };
            }
            Err(error) => {
                self.system = None;
                self.status = format!("Cannot build the circuit: {error}");
            }
        }
    }

    fn advance(&mut self, seconds: f64) {
        let Some(system) = self.system.as_mut() else {
            return;
        };
        if let Err(error) = system.advance(seconds) {
            self.status = format!("Simulation stopped: {error}");
            self.playing = false;
        }
    }

    /// Applies the actuator: a spring-return valve moves while it is held.
    fn actuate_valve(&mut self) {
        let Some(system) = self.system.as_mut() else {
            return;
        };
        if system.press_valve() {
            self.status = format!(
                "Valve at `{}`, held by the actuator",
                system
                    .valve()
                    .position_names
                    .get(system.valve_position())
                    .cloned()
                    .unwrap_or_default()
            );
        }
    }

    /// Lets go: the spring puts the spool back where it belongs.
    fn release_valve(&mut self) {
        let Some(system) = self.system.as_mut() else {
            return;
        };
        if system.release_valve() {
            self.status = format!(
                "Valve back at `{}` by the return spring",
                system
                    .valve()
                    .position_names
                    .get(system.valve_position())
                    .cloned()
                    .unwrap_or_default()
            );
        }
    }

    fn change_speed(&mut self, direction: isize) {
        let index = SPEEDS
            .iter()
            .position(|speed| (*speed - self.speed).abs() < f64::EPSILON)
            .unwrap_or(2);
        let last = SPEEDS.len().saturating_sub(1);
        let next = index.saturating_add_signed(direction).min(last);
        self.speed = SPEEDS.get(next).copied().unwrap_or(1.0);
        self.status = format!("Speed {:.2}x", self.speed);
    }

    /// Re-reads the desktop palette and applies it.
    fn reload_theme(&mut self, context: &egui::Context) {
        self.theme = Theme::detect_with(self.theme_choice.scheme(self.portal_scheme));
        self.theme.apply(context);
        // The whole window changes colour, so nothing may be left holding pixels of the
        // previous palette: a panel that is not repainted keeps the old theme on screen and
        // the window ends up half light and half dark.
        context.request_repaint();
        self.status = format!(
            "{} theme from the desktop",
            if self.theme.scheme.is_dark() {
                "Dark"
            } else {
                "Light"
            }
        );
    }

    /// Drains the portal watcher, reloading the theme when the desktop changes.
    fn poll_portal(&mut self, context: &egui::Context) {
        let mut changed = false;
        while let Some(scheme) = self.watcher.received() {
            if scheme != self.portal_scheme {
                self.portal_scheme = scheme;
                changed = true;
            }
        }
        if changed && self.theme_choice == ThemeChoice::System {
            self.reload_theme(context);
        }
    }

    fn dispatch(&mut self, context: &egui::Context, action: Action) {
        if let Some(reason) = action.unavailable() {
            self.status = format!("{}: {reason}", action.label());
            return;
        }
        match action {
            Action::Quit => context.send_viewport_cmd(egui::ViewportCommand::Close),
            Action::PlayPause => {
                self.mode = Mode::Simulate;
                self.playing = !self.playing;
                self.status = if self.playing {
                    String::from("Running")
                } else {
                    String::from("Paused")
                };
            }
            Action::Step => {
                self.mode = Mode::Simulate;
                self.playing = false;
                self.advance(0.010);
            }
            Action::Reset => self.restart_circuit(),
            Action::ActuateValve => {
                self.mode = Mode::Simulate;
                self.actuation.requested = true;
            }
            Action::SpeedSlower => self.change_speed(-1),
            Action::SpeedFaster => self.change_speed(1),
            Action::ToggleMode => {
                self.mode = self.mode.toggled();
                self.status = format!("{} mode", self.mode.label());
            }
            Action::ToggleGrid => {
                self.view.grid = !self.view.grid;
                self.status = if self.view.grid {
                    String::from("Grid on")
                } else {
                    String::from("Grid off")
                };
            }
            Action::ToggleSidebar => self.view.sidebar = !self.view.sidebar,
            Action::ToggleStatusBar => self.view.status_bar = !self.view.status_bar,
            Action::ThemeSystem => {
                self.theme_choice = ThemeChoice::System;
                self.reload_theme(context);
            }
            Action::ThemeLight => {
                self.theme_choice = ThemeChoice::Light;
                self.reload_theme(context);
            }
            Action::ThemeDark => {
                self.theme_choice = ThemeChoice::Dark;
                self.reload_theme(context);
            }
            Action::ReloadTheme => self.reload_theme(context),
            Action::ReloadCatalog => self.reload_catalog(),
            Action::FocusSearch => {
                self.view.sidebar = true;
                self.sidebar.focus_search = true;
            }
            Action::Help => self.help_open = true,
            Action::ExportPdf => {
                self.export.format = export_panel::Format::Pdf;
                self.export.open = true;
                self.playing = false;
                self.status = String::from("Export panel open");
            }
            Action::ExportPng => {
                self.export.format = export_panel::Format::Png;
                self.export.open = true;
                self.playing = false;
                self.status = String::from("Export panel open");
            }
            Action::New
            | Action::Open
            | Action::Save
            | Action::SaveAs
            | Action::Undo
            | Action::Redo
            | Action::Cut
            | Action::Copy
            | Action::Paste
            | Action::Duplicate
            | Action::Delete
            | Action::SelectAll
            | Action::Rotate
            | Action::SnapGrid
            | Action::SnapPorts
            | Action::ZoomIn
            | Action::ZoomOut
            | Action::ZoomFit => {
                // Unavailable actions return above, so this arm is unreachable in
                // practice; it keeps the match total without a wildcard.
            }
        }
    }

    /// Draws the export panel and performs the export when asked.
    fn show_export(&mut self, context: &egui::Context) {
        if !export_panel::show(&mut self.export, context, &self.theme) {
            return;
        }
        let result = export_panel::check_destination(&self.export).and_then(|path| {
            let Some(system) = self.system.as_ref() else {
                return Err(String::from("There is no circuit to export"));
            };
            let written = match self.export.format {
                export_panel::Format::Pdf => punchi_fluid_export::write_pdf(
                    &path,
                    &self.export.page,
                    &self.export.block,
                    system,
                    &self.catalog,
                ),
                export_panel::Format::Png => punchi_fluid_export::write_png(
                    &path,
                    &self.export.page,
                    &self.export.block,
                    system,
                    &self.catalog,
                    self.export.dpi,
                ),
            };
            written.map(|()| path).map_err(|error| error.to_string())
        });
        self.export.message = Some(match result {
            Ok(path) => format!("Wrote {}", path.display()),
            Err(error) => error,
        });
        self.status = self.export.message.clone().unwrap_or_default();
    }

    fn status_info(&self) -> StatusInfo<'_> {
        let telemetry = self.system.as_ref().map(PneumaticSystem::telemetry);
        StatusInfo {
            time: telemetry.map_or(0.0, |value| value.time),
            pressure_bar: self.system.as_ref().map_or(0.0, |system| {
                (system.pressure_pa() - system.atmosphere_pa()) / 100_000.0
            }),
            position_mm: telemetry.map_or(0.0, |value| value.position * 1000.0),
            velocity: telemetry.map_or(0.0, |value| value.velocity),
            mass_in_g: telemetry.map_or(0.0, |value| value.mass_in * 1000.0),
            definitions: self.catalog.len(),
            skipped: self.catalog.problems().len(),
            mode: self.mode,
            message: &self.status,
        }
    }
}

impl FluidLabApp {
    /// One gesture, three sources: the valve on the sheet, the ribbon button and the
    /// keyboard. While any of them is applied the spool is held; when all are gone the
    /// spring puts it back.
    fn apply_actuation(&mut self, context: &egui::Context, held: bool, released: bool) {
        let keyboard_held = if self.mode == Mode::Simulate {
            context.input(|input| {
                input.key_down(egui::Key::A) && input.modifiers.command && input.modifiers.shift
            })
        } else {
            false
        };
        let applied = held || keyboard_held || self.actuation.requested;
        if applied {
            self.actuate_valve();
        } else if self.actuation.holding || released {
            self.release_valve();
        }
        self.actuation.holding = applied;
    }
}

impl FluidLabApp {
    /// Draws the menu bar of the window and returns the command, if any.
    ///
    /// It belongs to the window, above the ribbon: it is where a desktop user looks for
    /// File, Edit, View and the about entry. It invents nothing — every entry is an
    /// `Action` the ribbon and the keyboard already know.
    fn show_menu_bar(
        &mut self,
        ui: &mut egui::Ui,
        metrics: &punchi_fluid_theme::Metrics,
    ) -> Option<Action> {
        let mut action = None;
        // No border and no rule: in a native window the menu bar and the toolbar are the
        // same surface as the rest of the window, and only the content moves. The fill is
        // painted explicitly with the token the contrast repair targets, so the strip can
        // never fall back to the toolkit's black.
        let chrome = egui::Frame::NONE.fill(self.theme.colour(punchi_fluid_theme::Token::Surface));
        egui::Panel::top("menu")
            .frame(chrome)
            .exact_size(metrics.row_height + metrics.grid_unit * 0.6)
            .show(ui, |ui| {
                let outcome = menu::show(ui, &self.theme, &mut self.menu);
                action = outcome.action;
            });
        action
    }
}

impl eframe::App for FluidLabApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_portal(ui.ctx());
        self.actuation.requested = self.actuation.demo;
        if let Some(action) = ribbon::shortcut(ui.ctx()) {
            self.dispatch(ui.ctx(), action);
        }

        let frame_step = f64::from(ui.input(|input| input.stable_dt)).min(MAX_FRAME_STEP);
        if self.playing {
            self.advance(frame_step * self.speed);
            ui.ctx().request_repaint();
        }

        let metrics = self.theme.metrics;
        let info = RibbonInfo {
            playing: self.playing,
            speed: self.speed,
            mode: self.mode,
        };
        let mut action = self.show_menu_bar(ui, &metrics);
        let chrome = egui::Frame::NONE.fill(self.theme.colour(punchi_fluid_theme::Token::Surface));
        egui::Panel::top("ribbon")
            .frame(chrome)
            .exact_size(shell::ribbon_height(&metrics))
            .show(ui, |ui| {
                action = ribbon::show(ui, &self.theme, &mut self.ribbon, &info);
            });
        if let Some(action) = action {
            self.dispatch(ui.ctx(), action);
        }
        menu::about(
            ui,
            &mut self.menu,
            &self.theme,
            env!("CARGO_PKG_VERSION"),
            self.mode,
        );

        if self.view.status_bar && !self.preview {
            egui::Panel::bottom("status")
                .exact_size(metrics.status_height + metrics.grid_unit * 0.4)
                .show(ui, |ui| {
                    shell::show_status(ui, &self.theme, &self.status_info());
                });
        }

        if self.view.sidebar && !self.preview {
            let mut event = SidebarEvent::default();
            egui::Panel::left("catalog")
                .resizable(true)
                .default_size(metrics.sidebar_width)
                .min_size(metrics.sidebar_min)
                .show(ui, |ui| {
                    event = sidebar::show(
                        ui,
                        &self.theme,
                        &self.catalog,
                        &mut self.sidebar,
                        Some(&self.user_dir),
                    );
                });
            if event.reload {
                self.reload_catalog();
            }
            if let Some(id) = event.place {
                self.status = format!("{id} is ready to place; the editor arrives in M3");
            }
        }

        if self.preview {
            egui::CentralPanel::default().show(ui, |ui| {
                shell::paint_sheet(ui, &self.theme, false);
                punchi_fluid_ui::preview::show(ui, &self.theme, &self.catalog);
            });
            shell::show_help(ui.ctx(), &self.theme, &mut self.help_open);
            return;
        }

        let mut held = false;
        let mut released = false;
        egui::CentralPanel::default().show(ui, |ui| {
            shell::paint_sheet(ui, &self.theme, self.view.grid);
            match self.system.as_ref() {
                Some(system) => {
                    let outcome = self.canvas.show(ui, system, &self.catalog);
                    if self.mode == Mode::Simulate {
                        held = outcome.valve_held;
                        released = outcome.valve_released;
                    }
                }
                None => {
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            egui::RichText::new(
                                "The circuit cannot start, see the catalog sidebar",
                            )
                            .color(self.theme.colour(punchi_fluid_theme::Token::Danger)),
                        );
                    });
                }
            }
        });
        self.apply_actuation(ui.ctx(), held, released);

        shell::show_help(ui.ctx(), &self.theme, &mut self.help_open);
        self.show_export(ui.ctx());
    }
}
