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
use punchi_fluid_doc::{Document, Placement, Turn};
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

    /// The placement under `point` in the active circuit, the one on top first.
    fn hit_placement(&self, point: Pt) -> Option<usize> {
        let circuit = self.document.active_circuit();
        self.document.placements().iter().rposition(|placement| {
            placement.circuit == circuit && placement.contains(point, HIT_SLACK)
        })
    }

    /// Selects what is under `point`, or clears the selection when the sheet is empty there.
    fn select_at(&mut self, point: Pt) -> Option<usize> {
        let hit = self.hit_placement(point);
        self.selected = hit.map(|index| vec![index]).unwrap_or_default();
        self.notice = hit.and_then(|index| {
            self.document
                .placements()
                .get(index)
                .map(|placement| format!("seleccionado {}", placement.catalog_id))
        });
        hit
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
            "Deshacer" => self.undo(),
            "Rehacer" => self.redo(),
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
        self.ribbon.savable = self.path.is_some();
        self.ribbon.selection = !self.selected.is_empty();
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
        egui::TopBottomPanel::bottom("status").show(context, |ui| {
            ui.label(
                egui::RichText::new(format!(
                    "hoja {width:.0}×{height:.0} mm · {:.0} % · {magnification:.2} px/mm · {:.2} ms de dibujo · último: {last} · {source}{notice}",
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
            let mut scene = scene::build(&self.document, self.document.active_circuit());
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
        if response.clicked() {
            if let Some(position) = response.interact_pointer_pos() {
                self.select_at(to_mm(position));
            }
        }
        if response.drag_started_by(egui::PointerButton::Primary) {
            if let Some(position) = response.interact_pointer_pos() {
                let index = self.select_at(to_mm(position));
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
mod tests {
    use eframe::egui;
    use punchi_fluid_doc::{Document, Placement};
    use punchi_fluid_draw::surface::Recording;

    use super::{Drag, Pt, Window, sheet, theme};

    #[test]
    fn a_click_selects_the_placement_under_it_and_clears_the_rest() {
        let mut document = Document::new();
        document.place(Placement::new(
            "valve.5_2",
            Pt::new(100.0, 80.0),
            24.0,
            24.0,
        ));
        let mut window = Window::new(theme::Theme::fallback(), document, None, None);
        assert_eq!(window.select_at(Pt::new(101.0, 81.0)), Some(0));
        assert_eq!(window.selected, vec![0]);
        assert_eq!(window.select_at(Pt::new(10.0, 10.0)), None);
        assert!(window.selected.is_empty());
    }

    #[test]
    fn a_drag_commit_moves_the_selection_and_leaves_one_undo_step() {
        let mut document = Document::new();
        document.place(Placement::new(
            "valve.5_2",
            Pt::new(100.0, 80.0),
            24.0,
            24.0,
        ));
        let mut window = Window::new(theme::Theme::fallback(), document, None, None);
        let _ = window.select_at(Pt::new(100.0, 80.0));
        window.drag = Some(Drag {
            pointer: egui::pos2(0.0, 0.0),
            starts: vec![(0, Pt::new(100.0, 80.0))],
            delta: egui::Vec2::ZERO,
        });
        window.commit_drag(1.0, egui::pos2(20.0, 10.0));
        let moved = window.document.placements()[0].centre;
        assert!((moved.x - 120.0).abs() < 0.01);
        assert!((moved.y - 90.0).abs() < 0.01);
        assert!(window.document.can_undo(), "one drag, one undo entry");
        assert!(window.drag.is_none());
    }

    #[test]
    fn adding_a_component_places_it_at_the_sheet_centre_as_one_undo_step() {
        let mut window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
        let centre = window.page.rect().center();
        window.add_component("valve.3_2.spring_return");
        assert_eq!(window.document.placements().len(), 1);
        let placed = window.document.placements()[0].centre;
        assert!((placed.x - centre.x).abs() < 0.01);
        assert!((placed.y - centre.y).abs() < 0.01);
        assert_eq!(window.selected, vec![0]);
        window.undo();
        assert!(window.document.placements().is_empty());
    }

    #[test]
    fn deleting_the_selection_is_one_undo_step() {
        let mut document = Document::new();
        document.place(Placement::new(
            "valve.5_2",
            Pt::new(100.0, 80.0),
            24.0,
            24.0,
        ));
        let mut window = Window::new(theme::Theme::fallback(), document, None, None);
        let _ = window.select_at(Pt::new(100.0, 80.0));
        window.remove_selected();
        assert!(window.document.placements().is_empty());
        assert!(window.selected.is_empty());
        window.undo();
        assert_eq!(
            window.document.placements().len(),
            1,
            "one undo brings it back"
        );
    }

    #[test]
    fn turning_and_reflecting_the_selection_both_undo() {
        let mut document = Document::new();
        document.place(Placement::new(
            "valve.5_2",
            Pt::new(100.0, 80.0),
            24.0,
            24.0,
        ));
        let before = document.placements()[0].orientation;
        let mut window = Window::new(theme::Theme::fallback(), document, None, None);
        let _ = window.select_at(Pt::new(100.0, 80.0));
        window.rotate_selected();
        assert_ne!(window.document.placements()[0].orientation, before);
        window.undo();
        assert_eq!(window.document.placements()[0].orientation, before);
        let _ = window.select_at(Pt::new(100.0, 80.0));
        window.reflect_selected();
        assert_ne!(window.document.placements()[0].orientation, before);
        window.undo();
        assert_eq!(window.document.placements()[0].orientation, before);
    }

    #[test]
    fn the_window_saves_the_document_it_was_given() {
        let path = std::env::temp_dir().join(format!(
            "punchi-fluid-window-save-{}.pfluid",
            std::process::id()
        ));
        let mut window = Window::new(
            theme::Theme::fallback(),
            Document::new(),
            Some(path.clone()),
            None,
        );
        window.save();
        let text = std::fs::read_to_string(&path).expect("the window wrote the file");
        assert!(
            punchi_fluid_file::sheet::Sheet::parse(&text).is_ok(),
            "the saved text is a sheet"
        );
        assert!(
            window
                .notice
                .as_deref()
                .is_some_and(|notice| notice.starts_with("guardado")),
            "the status bar says the save happened"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn the_sheet_of_a_new_document_draws_into_a_recording() {
        let window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
        let mut recording = Recording::default();
        sheet::draw(&mut recording, &window.page, &window.block);
        assert!(
            !recording.is_empty(),
            "the sheet must produce calls: {}",
            recording.len()
        );
    }
}
