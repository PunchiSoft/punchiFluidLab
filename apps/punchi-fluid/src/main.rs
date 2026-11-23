// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The window: the shell's three regions over the document's sheet.
//!
//! This is ADR 0035's phase 3 and `docs/ui-ux.md` §2.2: the shell chooses wide, medium or
//! compact from the room the window has, the window opens at the screen-aware size and
//! remembers its size and place, and the sheet keeps its floor. The frame itself lives in
//! `shell`; this module owns the window's state and the document's commands. Colour, type and
//! motion come from the desktop through `theme`.

use std::path::PathBuf;

use eframe::egui;
use punchi_fluid_catalog::Kinematics;
use punchi_fluid_doc::{Align, Document, Placement, Turn};
use punchi_fluid_draw::{CYLINDER_RATIO, Pt, SYMBOL_BOX_MM};
#[cfg(test)]
use punchi_fluid_export::sheet;
use punchi_fluid_export::{Page, TitleBlock};
use punchi_fluid_file::sheet::Sheet;
use punchi_fluid_session::recents::Recents;

use crate::ribbon::{Command, Speed};

mod auto_hide;
mod block_editor;
mod branding;
mod check_panel;
mod circuit_dialogs;
mod edit_commands;
mod export;
mod export_options;
mod files;
mod inspector;
mod layout;
mod lifecycle;
mod move_symbol;
mod page_setup;
mod panel;
mod preferences;
mod ribbon;
mod scene;
mod selection;
mod settings;
mod sheet_commands;
mod sheet_menu;
mod shell;
mod shortcuts;
mod sidebar;
mod sim;
mod start;
mod start_commands;
mod state_file;
mod surface;
mod theme;
mod view;
mod window_state;

fn main() -> eframe::Result<()> {
    let path = std::env::args().nth(1).map(PathBuf::from);
    let mut notice = None;
    let mut recents = Recents::load();
    let document = match path.as_deref() {
        Some(path) => match Sheet::read(path)
            .map_err(|cause| cause.to_string())
            .and_then(|sheet| punchi_fluid_session::load(&sheet))
        {
            Ok(document) => {
                // A document named on the command line is a document the page offers to
                // reopen, exactly as if it had been opened from the dialog.
                recents.remember(path);
                document
            }
            Err(cause) => {
                notice = Some(format!("no se pudo abrir el documento: {cause}"));
                Document::new()
            }
        },
        None => Document::new(),
    };
    let safety = punchi_fluid_session::safety::SafetyCopy::load();
    // The screen is only known after the window exists: the first frame sizes and centres it
    // from the monitor the viewport reports. The provisional size is the remembered one when
    // there is a state, so a returning window does not jump; its place is applied on the first
    // frame, which keeps the opening monitor as the anchor for the disconnect fallback.
    let geometry = window_state::Store::load();
    let opening = window_state::provisional(geometry.remembered());
    // The appearance is read before the theme so the window opens in the saved theme,
    // without a frame in the desktop palette first.
    let settings = settings::Store::load();
    let theme = theme::Theme::for_appearance(
        settings.remembered().appearance,
        settings.remembered().theme_day,
        settings.remembered().theme_night,
    );
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size(opening.size)
        .with_min_inner_size([window_state::MIN_WIDTH, window_state::MIN_HEIGHT])
        .with_title("Punchi Fluid Lab")
        .with_app_id(branding::APP_ID)
        .with_icon(branding::window_icon());
    if opening.maximized {
        viewport = viewport.with_maximized(true);
    }
    let options = eframe::NativeOptions {
        viewport,
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "punchi-fluid",
        options,
        Box::new(move |context| {
            egui_extras::install_image_loaders(&context.egui_ctx);
            theme::install_font(&context.egui_ctx, theme.font_family())?;
            theme.apply(&context.egui_ctx);
            Ok(Box::new(
                Window::with_safety(theme, document, path, notice, recents, safety)
                    .with_window_state(geometry)
                    .with_settings(settings),
            ))
        }),
    )
}

/// What the window owns today: the document it was given, the sheet it draws, the reader's
/// view, the ribbon, the desktop's theme and the last frame's numbers.
struct Window {
    document: Document,
    /// The document's saved revision, safety copy, recovery, title and open panel.
    lifecycle: lifecycle::Lifecycle,
    catalog: punchi_fluid_catalog::Catalog,
    path: Option<PathBuf>,
    notice: Option<String>,
    page: Page,
    block: TitleBlock,
    view: view::View,
    magnification: f32,
    frame_ms: f64,
    theme: theme::Theme,
    ribbon: ribbon::Ribbon,
    /// The simulation's clock speed; the ribbon shows the active one.
    speed: Speed,
    /// The page setup panel's draft; the document is only touched on apply.
    page_setup: page_setup::PageSetup,
    /// The title block editor's draft; the document is only touched on apply.
    block_editor: block_editor::BlockEditor,
    /// The circuit dialog waiting for an answer, if any.
    circuit_dialog: Option<circuit_dialogs::CircuitDialog>,
    /// The export options: format, resolution and the parts list.
    export_options: export_options::ExportOptions,
    /// The faults panel's last run and selection.
    check_panel: check_panel::CheckPanel,
    /// The catalog palette: the third region, with its search state.
    sidebar: sidebar::Sidebar,
    /// The right dock: the properties of the selection, and its own view state.
    inspector: inspector::Inspector,
    /// The placements the reader selected, as indices into the document.
    selected: Vec<usize>,
    /// The selected hose, when the selection is one. A hose and components are never
    /// selected together: `Supr` has to know which one it removes.
    selected_pipe: Option<usize>,
    /// The drag in progress: the pointer's press, the selection's start centres and the
    /// millimetres it has moved. The drawing applies the offset live; the release commits it.
    drag: Option<Drag>,
    /// The armed move: the pointer carries the selection and a click drops it. View state,
    /// not a document edit until the drop.
    moving: Option<move_symbol::Moving>,
    /// The sheet's context menu, its position and the revision it belongs to.
    menu: sheet_menu::SheetMenu,
    /// The empty Configuración and Descripción windows the menu opens.
    mockups: sheet_menu::Mockups,
    /// The window's clipboard: the placements cut or copied, ready to paste.
    clipboard: Vec<Placement>,
    /// The live model of the active circuit, rebuilt when the document changes.
    model: Option<punchi_fluid_session::Model>,
    /// The document revision the model was built from.
    model_revision: u64,
    /// Whether the model's clock is being advanced.
    running: bool,
    /// The valve held under the pointer, so the gesture presses once and releases once.
    pressed_valve: Option<usize>,
    /// Whether the Inicio page is the screen, not the sheet.
    start_open: bool,
    /// The documents Inicio offers to reopen, and where they are written.
    recents: Recents,
    /// The file dialog, while it is open.
    dialog: Option<files::Dialog>,
    /// The shell: the layout mode, the compact drawer and the remembered geometry.
    shell: shell::Shell,
    /// The panel settings store: the file both panels and the auto-hide flag live in.
    settings: settings::Store,
    /// The auto-hide timers: the one place that knows the dwell and the linger.
    auto_hide: auto_hide::AutoHide,
    /// Whether **Ver ▸ Ocultar al salir** is on.
    autohide_on: bool,
    /// The theme the system mode uses when the desktop is light.
    appearance_day: theme::ThemeId,
    /// The theme the system mode uses when the desktop is dark.
    appearance_night: theme::ThemeId,
    /// The Configuración window: whether it is open and which page it shows.
    preferences: preferences::Preferences,
}

/// A primary-button drag of the selection.
struct Drag {
    pointer: egui::Pos2,
    starts: Vec<(usize, Pt)>,
    delta: egui::Vec2,
}

/// How far beside its edge a placement still accepts the click, in millimetres.
const HIT_SLACK: f32 = 1.0;

impl Window {
    /// Places a catalog entry at the centre of the sheet, as one undo entry, and selects
    /// it. The box is the canonical one, or the actuator's wider barrel box.
    fn add_component(&mut self, catalog_id: &str) {
        let Some(definition) = self.catalog.get(catalog_id) else {
            self.notice = Some(format!("no está en el catálogo: {catalog_id}"));
            return;
        };
        let (width, height) = if definition.kinematics == Kinematics::LinearActuator {
            (SYMBOL_BOX_MM * CYLINDER_RATIO, SYMBOL_BOX_MM)
        } else {
            (SYMBOL_BOX_MM, SYMBOL_BOX_MM)
        };
        let centre = self.page.rect().center();
        let index = self
            .document
            .place(Placement::new(catalog_id, centre, width, height));
        self.selected = vec![index];
        let name = definition.name_es.as_deref().unwrap_or(&definition.name);
        self.notice = Some(format!("añadida {name}"));
    }

    /// Rebuilds the model when the document changed, and says why when it cannot.
    fn rebuild_if_stale(&mut self) {
        if self.model_revision == self.document.revision() {
            return;
        }
        self.model = match punchi_fluid_session::Model::build(&self.document, &self.catalog) {
            Ok(model) => Some(model),
            Err(cause) => {
                self.notice = Some(cause);
                None
            }
        };
        self.model_revision = self.document.revision();
    }

    /// Starts the clock if the sheet has a model.
    fn start(&mut self) {
        self.rebuild_if_stale();
        if self.model.is_some() {
            self.running = true;
        }
    }

    /// Advances the model by `dt` seconds, stopping with the reason when the step fails.
    fn advance(&mut self, dt: f64) {
        let Some(model) = self.model.as_mut() else {
            self.running = false;
            return;
        };
        if let Err(cause) = model.advance(dt) {
            self.running = false;
            self.notice = Some(format!("la simulación se detuvo: {cause}"));
        }
    }

    /// Stops the clock and throws the model away, so the next start builds it at `t = 0`.
    fn stop(&mut self) {
        self.running = false;
        self.model = None;
        self.model_revision = 0;
    }

    /// The live frame's work: rebuild the model when the document moved and advance the
    /// clock when it is running.
    fn tick(&mut self, context: &egui::Context) {
        self.rebuild_if_stale();
        if self.running {
            let dt = f64::from(context.input(|input| input.stable_dt).min(0.1));
            self.advance(dt * f64::from(self.speed.factor()));
        }
    }

    /// The status bar's live line: nothing until a model is loaded.
    fn simulation_line(&self) -> String {
        match self.model.as_ref().map(|model| model.network().telemetry()) {
            Some(telemetry) => format!(
                " · t {:.2} s · {:.2} bar · {:.1} mm · {:.1} mm/s",
                telemetry.time,
                telemetry.pressure_pa / 1.0e5,
                telemetry.position * 1000.0,
                telemetry.velocity * 1000.0,
            ),
            None => String::new(),
        }
    }

    /// The placement under `point` in the active circuit, the one on top first.
    fn hit_placement(&self, point: Pt) -> Option<usize> {
        let circuit = self.document.active_circuit();
        self.document.placements().iter().rposition(|placement| {
            placement.circuit == circuit && placement.contains(point, HIT_SLACK)
        })
    }

    /// Selects what is under `point`, with Shift adding or removing it from the selection,
    /// or clears the selection when the sheet is empty there. Returns the hit, if any.
    fn select_at(&mut self, point: Pt, additive: bool) -> Option<usize> {
        let hit = self.hit_placement(point);
        if let Some(index) = hit {
            self.selected_pipe = None;
            if additive {
                if self.selected.contains(&index) {
                    self.selected.retain(|selected| *selected != index);
                } else {
                    self.selected.push(index);
                }
            } else {
                self.selected = vec![index];
            }
        } else if !additive {
            self.selected.clear();
            self.select_pipe_at(point);
        }
        self.sync_selection_notice();
        hit
    }

    /// Selects every placement of the active circuit.
    fn select_all(&mut self) {
        self.selected_pipe = None;
        let circuit = self.document.active_circuit();
        self.selected = self
            .document
            .placements()
            .iter()
            .enumerate()
            .filter(|(_, placement)| placement.circuit == circuit)
            .map(|(index, _)| index)
            .collect();
        let count = self.selected.len();
        self.notice = (count > 0).then(|| format!("{count} seleccionadas"));
    }

    /// Copies the selection a little displaced, as one undo entry, and selects the copies.
    fn duplicate_selected(&mut self) {
        if self.selected.is_empty() {
            return;
        }
        let copies = self.document.duplicate_many(&self.selected, (10.0, 10.0));
        if copies.is_empty() {
            return;
        }
        self.selected = copies;
        let count = self.selected.len();
        self.notice = Some(format!("duplicadas {count}"));
    }

    /// Aligns the selection on its furthest edge, as one undo entry.
    fn align_selected(&mut self, align: Align) {
        if self.selected.len() < 2 {
            return;
        }
        if self.document.align_many(&self.selected, align) {
            self.notice = Some(String::from("alineado"));
        }
    }

    /// Copies the selection into the window's clipboard.
    fn copy_selected(&mut self) {
        self.clipboard = self
            .selected
            .iter()
            .filter_map(|index| self.document.placements().get(*index).cloned())
            .collect();
        let count = self.clipboard.len();
        if count > 0 {
            self.notice = Some(format!("copiadas {count}"));
        }
    }

    /// Pastes the clipboard a little displaced, as one undo entry, and selects the copies.
    fn paste(&mut self) {
        if self.clipboard.is_empty() {
            return;
        }
        let copies: Vec<Placement> = self
            .clipboard
            .iter_mut()
            .map(|placement| {
                placement.centre = Pt::new(placement.centre.x + 10.0, placement.centre.y + 10.0);
                placement.clone()
            })
            .collect();
        self.selected = self.document.place_many(copies);
        let count = self.selected.len();
        self.notice = Some(format!("pegadas {count}"));
    }

    /// Cuts the selection: copies it and removes it as one undo entry.
    fn cut_selected(&mut self) {
        self.copy_selected();
        self.remove_selected();
    }

    /// Ends a drag: one undo entry for the whole gesture, never one per frame.
    fn commit_drag(&mut self, scale: f32, now: egui::Pos2) {
        let Some(drag) = self.drag.take() else {
            return;
        };
        let delta = egui::vec2(now.x - drag.pointer.x, now.y - drag.pointer.y) / scale;
        if delta.length() < 0.5 {
            return;
        }
        let moves: Vec<(usize, Pt)> = drag
            .starts
            .iter()
            .map(|(index, start)| (*index, Pt::new(start.x + delta.x, start.y + delta.y)))
            .collect();
        if self.document.move_many(&moves) {
            self.notice = Some(String::from("movido"));
        }
    }

    /// Deletes the selection as one undo entry: the selected hose when there is one, the
    /// selected components otherwise.
    fn remove_selected(&mut self) {
        if self.remove_selected_pipe() {
            return;
        }
        if self.selected.is_empty() {
            return;
        }
        let removed = self.document.remove_many(&self.selected);
        self.selected.clear();
        if !removed.is_empty() {
            self.notice = Some(format!("borradas {}", removed.len()));
        }
    }

    /// Turns the selection a quarter clockwise, as one undo entry.
    fn rotate_selected(&mut self) {
        if self.selected.is_empty() {
            return;
        }
        if self.document.turn_many(&self.selected, Turn::Clockwise) {
            self.notice = Some(String::from("girado"));
        }
    }

    /// Reflects the selection on its own vertical axis, as one undo entry.
    fn reflect_selected(&mut self) {
        if self.selected.is_empty() {
            return;
        }
        if self.document.turn_many(&self.selected, Turn::Reflect) {
            self.notice = Some(String::from("reflejado"));
        }
    }

    /// Steps the document's history back. The selection is dropped: a restored placement may
    /// have taken an index the selection still names.
    fn undo(&mut self) {
        if self.document.undo() {
            self.selected.clear();
            self.selected_pipe = None;
            self.notice = Some(String::from("deshecho"));
        }
    }

    /// Steps the document's history forward, dropping the selection for the same reason.
    fn redo(&mut self) {
        if self.document.redo() {
            self.selected.clear();
            self.selected_pipe = None;
            self.notice = Some(String::from("rehecho"));
        }
    }

    /// Runs the command the ribbon picked, whether it edits the document or the view.
    /// Runs a ribbon command. The match is exhaustive, so a command without a handler does
    /// not compile; the table and this function are the two halves of the same contract.
    fn dispatch(&mut self, command: Command) {
        self.ribbon.last = command.label_es().to_string();
        match command {
            Command::ProjectNew => self.new_document(),
            Command::ProjectOpen => self.open_dialog(files::Mode::Open),
            Command::ProjectSave => self.save(),
            Command::ProjectSaveAs => self.open_dialog(files::Mode::SaveAs),
            Command::ProjectExport => self.export_dialog(),
            Command::ExportOptions => self.open_export_options(),
            Command::Check => self.open_check(),
            Command::PageMargin(preset) => self.set_sheet_margins(preset),
            Command::PageOrientation(orientation) => self.set_sheet_orientation(orientation),
            Command::PagePaper(paper) => self.set_sheet_paper(paper),
            Command::BlockModel(model) => self.set_block_model(model),
            Command::BlockPosition(position) => self.set_block_position(position),
            Command::BlockReset => self.reset_block(),
            Command::PipeClass(class) => self.set_pipe_class(class),
            Command::Tube => self.open_pipe_control(),
            Command::Distribute(axis) => self.distribute(axis),
            Command::SelectAll => self.select_all(),
            Command::SimReset => self.reset_simulation(),
            Command::Speed(speed) => {
                self.speed = speed;
                self.ribbon.active_speed = speed.label();
            }
            Command::PageSetup => self.open_page_setup(),
            Command::BlockEdit => self.open_block_editor(),
            Command::CircuitRename => self.open_circuit_rename(),
            Command::CircuitClose => self.open_circuit_close(),
            Command::Home => self.open_start(),
            Command::OpenRecent(index) => self.open_recent(index),
            Command::CircuitNew => self.new_circuit(),
            Command::Undo => self.undo(),
            Command::Redo => self.redo(),
            Command::Copy => self.copy_selected(),
            Command::Paste => self.paste(),
            Command::Duplicate => self.duplicate_selected(),
            Command::Delete => self.remove_selected(),
            Command::Rotate => self.rotate_selected(),
            Command::Mirror => self.reflect_selected(),
            Command::AlignLeft => self.align_selected(Align::Left),
            Command::AlignRight => self.align_selected(Align::Right),
            Command::AlignTop => self.align_selected(Align::Top),
            Command::AlignBottom => self.align_selected(Align::Bottom),
            Command::ZoomIn => self.view.zoom_in(),
            Command::ZoomOut => self.view.zoom_out(),
            Command::ZoomReset => self.view.reset(),
            Command::Catalog | Command::Properties | Command::Autohide | Command::ResetView => {
                self.view_command(command);
            }
            Command::Theme => self.open_preferences(),
            Command::Start => self.start(),
            Command::Pause => self.running = false,
            Command::Step => {
                self.running = false;
                self.advance(0.01);
            }
            Command::UntilChange => self.until_change(),
            Command::Stop => self.stop(),
        }
    }

    /// **Circuito ▸ Nuevo circuito**: an empty circuit, made active, as one undo. The project
    /// always keeps at least one circuit, so the document can refuse it; the notice says so.
    fn new_circuit(&mut self) {
        let title = format!("Circuito {}", self.document.circuits().len() + 1);
        let id = self.document.add_circuit(title);
        if id == 0 {
            self.notice = Some(String::from("no caben más circuitos"));
            return;
        }
        let (page, block) = files::page_and_block(&self.document);
        self.page = page;
        self.block = block;
        self.selected.clear();
        self.selected_pipe = None;
        self.model = None;
        self.model_revision = 0;
        self.notice = Some(String::from("circuito nuevo"));
    }

    /// The keyboard commands and the ribbon's enabled state, so `update` stays the layout.
    fn sync_commands(&mut self, context: &egui::Context) {
        // The dialog owns the keyboard while it is open: no document command may run behind
        // it, and `Enter`/`Esc` belong to it until it closes.
        if self.dialog.is_some() {
            return;
        }
        if self.preferences.open {
            return;
        }
        // The sheet's menu and the mockups own the keyboard while they are open, as a
        // dialog does: no document command may run behind them.
        if self.menu.is_open() || self.mockups.is_open() {
            return;
        }
        if context.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::Comma)) {
            self.open_preferences();
            return;
        }
        // Every shortcut comes from the one table, so the hint and the handler cannot
        // disagree; an editing command waits until no text field has the keyboard.
        let typing = context.wants_keyboard_input();
        for (command, shortcut) in crate::shortcuts::ALL {
            if shortcut.pressed(context) && (!shortcut.editing || !typing) {
                self.dispatch(*command);
            }
        }
        // Cut is a document edit with no ribbon command of its own yet.
        if !typing && context.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::X))
        {
            self.cut_selected();
        }
        if context.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::F)) {
            self.sidebar.focus_search();
        }
        self.ribbon.savable = self.path.is_some();
        self.ribbon.can_step = self.model.is_some();
        self.ribbon.has_clipboard = !self.clipboard.is_empty();
        self.ribbon.selected = self.selected.len() + usize::from(self.selected_pipe.is_some());
        self.ribbon.can_undo = self.document.can_undo();
        self.ribbon.can_redo = self.document.can_redo();
        self.ribbon.can_reset_block = self.block_is_customized();
        self.ribbon.can_close_circuit = self.document.circuits().len() > 1;
        self.ribbon.has_pipe = self.selected_pipe.is_some();
        self.ribbon.running = self.running;
    }
}

impl eframe::App for Window {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame(context);
    }

    /// The last geometry the viewport reported is written even when the reader closes the
    /// window before the settle delay: the next start returns to the size and place they left.
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.flush_window_state();
        self.flush_panel_settings();
    }
}

impl Window {
    /// The sheet's editing gestures: a click selects the placement under the pointer, a
    /// primary drag moves the selection with a live preview, and the release commits one
    /// undo entry.
    fn handle_edit(
        &mut self,
        response: &egui::Response,
        placement: &view::Placement,
        page_min: egui::Vec2,
    ) {
        let to_mm = |screen: egui::Pos2| {
            Pt::new(
                page_min.x + (screen.x - placement.origin.x) / placement.scale,
                page_min.y + (screen.y - placement.origin.y) / placement.scale,
            )
        };
        // The armed move, the keyboard menu and the right-click menu own the frame when the
        // sheet is under them: no selection, valve press or drag happens underneath.
        if self.handle_menu_gestures(response, placement, page_min) {
            return;
        }
        // The reference presses a valve while the button is down: the momentary returns, the
        // maintained stays. A press on a valve is not a drag of the selection. Only the
        // primary button presses it; the right button belongs to the menu.
        if response.is_pointer_button_down_on()
            && response
                .ctx
                .input(|input| input.pointer.button_down(egui::PointerButton::Primary))
        {
            if let Some(position) = response.interact_pointer_pos() {
                let hit = self.hit_placement(to_mm(position));
                if hit != self.pressed_valve {
                    if let Some(previous) = self.pressed_valve.take() {
                        if let Some(model) = self.model.as_mut() {
                            let _ = model.release_valve(previous);
                        }
                    }
                    if let Some(index) = hit {
                        let pressed = self
                            .model
                            .as_mut()
                            .is_some_and(|model| model.press_valve(index));
                        if pressed {
                            self.pressed_valve = Some(index);
                        }
                    }
                }
            }
        } else if let Some(index) = self.pressed_valve.take() {
            if let Some(model) = self.model.as_mut() {
                let _ = model.release_valve(index);
            }
        }
        if response.clicked() {
            if let Some(position) = response.interact_pointer_pos() {
                let additive = response.ctx.input(|input| input.modifiers.shift);
                self.select_at(to_mm(position), additive);
            }
        }
        if response.drag_started_by(egui::PointerButton::Primary) {
            if let Some(position) = response.interact_pointer_pos() {
                let additive = response.ctx.input(|input| input.modifiers.shift);
                if additive || self.pressed_valve.is_some() {
                    return;
                }
                let index = self.select_at(to_mm(position), false);
                if index.is_some() {
                    let starts: Vec<(usize, Pt)> = self
                        .selected
                        .iter()
                        .filter_map(|index| {
                            self.document
                                .placements()
                                .get(*index)
                                .map(|placement| (*index, placement.centre))
                        })
                        .collect();
                    self.drag = Some(Drag {
                        pointer: position,
                        starts,
                        delta: egui::Vec2::ZERO,
                    });
                }
            }
        }
        if response.dragged_by(egui::PointerButton::Primary) {
            if let Some(drag) = &mut self.drag {
                if let Some(position) = response.interact_pointer_pos() {
                    drag.delta = egui::vec2(
                        (position.x - drag.pointer.x) / placement.scale,
                        (position.y - drag.pointer.y) / placement.scale,
                    );
                }
            }
        }
        if response.drag_stopped_by(egui::PointerButton::Primary) {
            match response.interact_pointer_pos() {
                Some(position) => self.commit_drag(placement.scale, position),
                None => self.drag = None,
            }
        }
    }

    /// The sheet's gestures: Ctrl+wheel zooms about the pointer, the wheel and the middle
    /// button pan, the arrows pan from the keyboard and Ctrl+0 returns to the fit.
    fn handle_view(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        area: egui::Rect,
        page_mm: egui::Vec2,
    ) {
        // The armed move pins the view: the pointer carries the symbol, so the sheet must not
        // slide under it with a wheel or an arrow.
        if self.moving.is_some() {
            return;
        }
        // The arrow keys belong to the open menu while it is up; panning the sheet under it
        // would both move the anchor and steal the key.
        if self.menu.is_open() {
            return;
        }
        self.view.handle_input(ui, response, area, page_mm);
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_acceptance;
