// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The window: one fixed layout over the document's sheet.
//!
//! This is ADR 0035's phase 3: the window opens at the 1366 × 768 floor, draws the sheet of a
//! new document with the ribbon and the view's zoom and pan, and reports the fit, the
//! magnification and the drawing's own time. The sidebar and the document's commands arrive
//! in the following slices. Colour, type and motion come from the desktop through `theme`.

use std::path::{Path, PathBuf};
use std::time::Instant;

use eframe::egui;
use punchi_fluid_catalog::Kinematics;
use punchi_fluid_doc::{Align, Document, Placement, Turn};
use punchi_fluid_draw::symbol::Ink;
use punchi_fluid_draw::{CYLINDER_RATIO, Pt, Rect, SYMBOL_BOX_MM, draw_placed};
use punchi_fluid_export::{Page, TitleBlock, sheet};
use punchi_fluid_file::sheet::Sheet;

mod ribbon;
mod scene;
mod sidebar;
mod surface;
mod theme;
mod view;

use surface::EguiSurface;

fn main() -> eframe::Result<()> {
    let theme = theme::Theme::from_desktop();
    let path = std::env::args().nth(1).map(PathBuf::from);
    let mut notice = None;
    let document = match path.as_deref() {
        Some(path) => match Sheet::read(path)
            .map_err(|cause| cause.to_string())
            .and_then(|sheet| punchi_fluid_session::load(&sheet))
        {
            Ok(document) => document,
            Err(cause) => {
                notice = Some(format!("no se pudo abrir el documento: {cause}"));
                Document::new()
            }
        },
        None => Document::new(),
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1366.0, 768.0])
            .with_min_inner_size([1366.0, 768.0])
            .with_title("Punchi Fluid Lab"),
        ..Default::default()
    };
    eframe::run_native(
        "punchi-fluid",
        options,
        Box::new(move |context| {
            egui_extras::install_image_loaders(&context.egui_ctx);
            theme::install_font(&context.egui_ctx, theme.font_family())?;
            theme.apply(&context.egui_ctx);
            Ok(Box::new(Window::new(theme, document, path, notice)))
        }),
    )
}

/// What the window owns today: the document it was given, the sheet it draws, the reader's
/// view, the ribbon, the desktop's theme and the last frame's numbers.
struct Window {
    document: Document,
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
    /// The catalog palette: the third region, with its search state.
    sidebar: sidebar::Sidebar,
    /// The placements the reader selected, as indices into the document.
    selected: Vec<usize>,
    /// The drag in progress: the pointer's press, the selection's start centres and the
    /// millimetres it has moved. The drawing applies the offset live; the release commits it.
    drag: Option<Drag>,
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
}

/// A primary-button drag of the selection.
struct Drag {
    pointer: egui::Pos2,
    starts: Vec<(usize, Pt)>,
    delta: egui::Vec2,
}

/// How far beside its edge a placement still accepts the click, in millimetres.
const HIT_SLACK: f32 = 1.0;

/// The palette's width: 18 grid units, the width the interface specification asks for.
const SIDEBAR_WIDTH: f32 = 240.0;

impl Window {
    fn new(
        theme: theme::Theme,
        document: Document,
        path: Option<PathBuf>,
        notice: Option<String>,
    ) -> Self {
        let sheet = document.sheet();
        let circuit = document.active_circuit();
        let block = document.block(circuit).cloned().unwrap_or_default();
        let page = Page {
            paper: sheet.paper,
            orientation: sheet.orientation,
            margins: sheet.margins,
            title_block: Some(block.clone()),
            parts: None,
        };
        Self {
            document,
            catalog: punchi_fluid_catalog::default_catalog(),
            path,
            notice,
            page,
            block,
            view: view::View::default(),
            magnification: 1.0,
            frame_ms: 0.0,
            theme,
            ribbon: ribbon::Ribbon::new(),
            sidebar: sidebar::Sidebar::new(),
            selected: Vec::new(),
            drag: None,
            clipboard: Vec::new(),
            model: None,
            model_revision: 0,
            running: false,
            pressed_valve: None,
        }
    }

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
            self.advance(dt);
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
        }
        self.notice = match self.selected.len() {
            0 => None,
            1 => Some(String::from("1 seleccionada")),
            count => Some(format!("{count} seleccionadas")),
        };
        hit
    }

    /// Selects every placement of the active circuit.
    fn select_all(&mut self) {
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

    /// Deletes the selection as one undo entry.
    fn remove_selected(&mut self) {
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
            self.notice = Some(String::from("deshecho"));
        }
    }

    /// Steps the document's history forward, dropping the selection for the same reason.
    fn redo(&mut self) {
        if self.document.redo() {
            self.selected.clear();
            self.notice = Some(String::from("rehecho"));
        }
    }

    /// Runs the command the ribbon picked, whether it edits the document or the view.
    fn run_command(&mut self, command: &str) {
        match command {
            "Guardar" => self.save(),
            "Borrar" => self.remove_selected(),
            "Girar" => self.rotate_selected(),
            "Reflejar" => self.reflect_selected(),
            "Duplicar" => self.duplicate_selected(),
            "Copiar" => self.copy_selected(),
            "Pegar" => self.paste(),
            "Izquierda" => self.align_selected(Align::Left),
            "Derecha" => self.align_selected(Align::Right),
            "Arriba" => self.align_selected(Align::Top),
            "Abajo" => self.align_selected(Align::Bottom),
            "Deshacer" => self.undo(),
            "Rehacer" => self.redo(),
            "Iniciar" => self.start(),
            "Pausa" => self.running = false,
            "Paso" => {
                self.running = false;
                self.advance(0.01);
            }
            "Detener" => self.stop(),
            _ => self.view.command(command),
        }
    }

    /// The keyboard commands and the ribbon's enabled state, so `update` stays the layout.
    fn sync_commands(&mut self, context: &egui::Context) {
        if context.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::S)) {
            self.save();
        }
        if context.input(|input| input.key_pressed(egui::Key::Delete)) {
            self.remove_selected();
        }
        if context.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::Z)) {
            self.undo();
        }
        if context.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::Y)) {
            self.redo();
        }
        if context.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::F)) {
            self.sidebar.focus_search = true;
        }
        // Select all and duplicate only when no text field has the keyboard: Ctrl+A in the
        // search selects the search text, not the sheet.
        let idle = context.memory(|memory| memory.focused().is_none());
        if idle && context.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::A)) {
            self.select_all();
        }
        if idle && context.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::D)) {
            self.duplicate_selected();
        }
        if idle && context.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::C)) {
            self.copy_selected();
        }
        if idle && context.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::V)) {
            self.paste();
        }
        if idle && context.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::X)) {
            self.cut_selected();
        }
        self.ribbon.savable = self.path.is_some();
        self.ribbon.has_clipboard = !self.clipboard.is_empty();
        self.ribbon.selected = self.selected.len();
        self.ribbon.can_undo = self.document.can_undo();
        self.ribbon.can_redo = self.document.can_redo();
    }

    /// Writes the document to the path it was opened from, or says why it cannot.
    fn save(&mut self) {
        let Some(path) = self.path.clone() else {
            self.notice = Some(String::from("sin ruta: abre un documento para guardarlo"));
            return;
        };
        match punchi_fluid_session::save(&self.document, &path) {
            Ok(()) => {
                let name = path
                    .file_name()
                    .and_then(std::ffi::OsStr::to_str)
                    .unwrap_or("el documento");
                self.notice = Some(format!("guardado {name}"));
            }
            Err(cause) => self.notice = Some(format!("no se pudo guardar: {cause}")),
        }
    }
}

impl eframe::App for Window {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        self.sync_commands(context);
        self.tick(context);
        let palette = ribbon::Palette::from_theme(&self.theme.tokens);
        egui::TopBottomPanel::top("ribbon").show(context, |ui| {
            self.ribbon.show(ui, &palette);
        });
        if let Some(command) = self.ribbon.take() {
            self.run_command(&command);
        }

        let (width, height) = self.page.size_mm();
        let page_mm = egui::vec2(width, height);
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

        egui::SidePanel::left("catalog")
            .exact_width(SIDEBAR_WIDTH)
            .resizable(false)
            .frame(
                egui::Frame::default()
                    .fill(self.theme.tokens.panel)
                    .inner_margin(egui::Margin::symmetric(6, 0)),
            )
            .show(context, |ui| {
                if let Some(catalog_id) = self.sidebar.show(ui, &self.theme.tokens, &self.catalog) {
                    self.add_component(&catalog_id);
                }
            });

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
            let mut drawing = EguiSurface::new(context, origin, placement.scale);
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
            let painter = ui.painter().with_clip_rect(ui.max_rect());
            painter.extend(drawing.shapes);
        });
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
        // The reference presses a valve while the button is down: the momentary returns, the
        // maintained stays. A press on a valve is not a drag of the selection.
        if response.is_pointer_button_down_on() {
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
        let (scroll, ctrl, pointer) = ui.input(|input| {
            (
                input.raw_scroll_delta,
                input.modifiers.ctrl,
                input.pointer.hover_pos(),
            )
        });
        if response.hovered() && scroll != egui::Vec2::ZERO {
            if ctrl {
                if let Some(pointer) = pointer {
                    self.view
                        .zoom_about((scroll.y * 0.005).exp(), pointer, area, page_mm);
                }
            } else {
                self.view.pan_by(scroll);
            }
        }
        if response.dragged_by(egui::PointerButton::Middle) {
            let delta = ui.input(|input| input.pointer.delta());
            self.view.pan_by(delta);
        }
        let (left, right, up, down, reset) = ui.input(|input| {
            (
                input.key_pressed(egui::Key::ArrowLeft),
                input.key_pressed(egui::Key::ArrowRight),
                input.key_pressed(egui::Key::ArrowUp),
                input.key_pressed(egui::Key::ArrowDown),
                input.modifiers.ctrl && input.key_pressed(egui::Key::Num0),
            )
        });
        if reset {
            self.view.reset();
        }
        let mut x = 0.0;
        let mut y = 0.0;
        if left {
            x += 1.0;
        }
        if right {
            x -= 1.0;
        }
        if up {
            y += 1.0;
        }
        if down {
            y -= 1.0;
        }
        if x != 0.0 || y != 0.0 {
            self.view.pan_arrow(x, y);
        }
    }
}

#[cfg(test)]
mod tests;
