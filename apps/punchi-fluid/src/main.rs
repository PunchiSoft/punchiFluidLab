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
use punchi_fluid_doc::Document;
use punchi_fluid_draw::draw_placed;
use punchi_fluid_draw::symbol::Ink;
use punchi_fluid_export::{Page, TitleBlock, sheet};
use punchi_fluid_file::sheet::Sheet;

mod ribbon;
mod scene;
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
}

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
        }
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
        // The ribbon enables Guardar only when there is a path to write to, and the window
        // owns that path; the keyboard shortcut is the same command.
        self.ribbon.savable = self.path.is_some();
        if context.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::S)) {
            self.save();
        }
        let palette = ribbon::Palette::from_theme(&self.theme.tokens);
        egui::TopBottomPanel::top("ribbon").show(context, |ui| {
            self.ribbon.show(ui, &palette);
        });
        if let Some(command) = self.ribbon.take() {
            match command.as_str() {
                "Guardar" => self.save(),
                _ => self.view.command(&command),
            }
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
            let origin = placement.origin
                - egui::vec2(
                    page_rect.min.x * placement.scale,
                    page_rect.min.y * placement.scale,
                );
            let mut drawing = EguiSurface::new(context, origin, placement.scale);
            let scene = scene::build(&self.document, self.document.active_circuit());
            let started = Instant::now();
            sheet::draw(&mut drawing, &self.page, &self.block);
            draw_placed(
                &mut drawing,
                &scene.placed,
                &scene.pipes,
                &self.catalog,
                &Ink::print(),
                &[],
            );
            self.frame_ms = started.elapsed().as_secs_f64() * 1000.0;
            let painter = ui.painter().with_clip_rect(ui.max_rect());
            painter.extend(drawing.shapes);
        });
    }
}

impl Window {
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
    use punchi_fluid_doc::Document;
    use punchi_fluid_draw::surface::Recording;

    use super::{Window, sheet, theme};

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
