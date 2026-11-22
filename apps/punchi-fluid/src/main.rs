// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The window: one fixed layout over the document's sheet.
//!
//! This is the first slice of ADR 0035's phase 3: the window opens at the 1366 × 768 floor,
//! draws the sheet of a new document and reports the fit and the drawing's own time. The
//! ribbon, the sidebar and the document's commands arrive in the following slices.

use std::time::Instant;

use eframe::egui;
use punchi_fluid_doc::Document;
use punchi_fluid_export::{Page, TitleBlock, sheet};

mod surface;

use surface::EguiSurface;

fn main() -> eframe::Result<()> {
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
        Box::new(|context| {
            context.egui_ctx.set_theme(egui::ThemePreference::System);
            install_system_font(&context.egui_ctx)?;
            Ok(Box::new(Window::new()))
        }),
    )
}

/// Installs the desktop's sans-serif face in egui.
///
/// The interface rule says colour, size and font come from the desktop, and the toolkit
/// ships none of them: egui's default fonts are disabled so no font licence travels with the
/// window. The export crate picks its face the same way, so the window's text and the
/// printed page agree.
///
/// # Errors
///
/// Returns an I/O error when the machine has no font database entry that can be read.
fn install_system_font(context: &egui::Context) -> Result<(), std::io::Error> {
    let mut database = fontdb::Database::new();
    database.load_system_fonts();
    let named = [
        fontdb::Family::SansSerif,
        fontdb::Family::Name("DejaVu Sans"),
        fontdb::Family::Name("Noto Sans"),
        fontdb::Family::Name("Liberation Sans"),
    ];
    let id = database
        .query(&fontdb::Query {
            families: &named,
            weight: fontdb::Weight::NORMAL,
            style: fontdb::Style::Normal,
            stretch: fontdb::Stretch::Normal,
        })
        .or_else(|| database.faces().next().map(|face| face.id))
        .ok_or_else(|| std::io::Error::other("no system font is installed"))?;
    let face = database
        .face(id)
        .ok_or_else(|| std::io::Error::other("the system font database has no face"))?;
    let data = match &face.source {
        fontdb::Source::Binary(binary) => binary.as_ref().as_ref().to_vec(),
        fontdb::Source::File(path) => std::fs::read(path)?,
        fontdb::Source::SharedFile(_path, binary) => binary.as_ref().as_ref().to_vec(),
    };
    let name = String::from("system");
    let mut definitions = egui::FontDefinitions::default();
    definitions.font_data.insert(
        name.clone(),
        std::sync::Arc::new(egui::FontData {
            font: std::borrow::Cow::Owned(data),
            index: face.index,
            tweak: egui::FontTweak::default(),
        }),
    );
    definitions
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, name.clone());
    definitions
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, name);
    context.set_fonts(definitions);
    Ok(())
}

/// What the window owns today: the sheet of a new document and the last frame's numbers.
struct Window {
    page: Page,
    block: TitleBlock,
    scale: f32,
    frame_ms: f64,
}

impl Window {
    fn new() -> Self {
        let document = Document::new();
        let sheet = document.sheet();
        let block = document
            .circuits()
            .first()
            .map_or_else(TitleBlock::default, |circuit| circuit.block.clone());
        let page = Page {
            paper: sheet.paper,
            orientation: sheet.orientation,
            margins: sheet.margins,
            title_block: Some(block.clone()),
            parts: None,
        };
        Self {
            page,
            block,
            scale: 1.0,
            frame_ms: 0.0,
        }
    }
}

impl eframe::App for Window {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        let (width, height) = self.page.size_mm();
        egui::TopBottomPanel::bottom("status").show(context, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "hoja {width:.0}×{height:.0} mm · {:.2} px/mm · {:.2} ms de dibujo",
                    self.scale, self.frame_ms
                ));
            });
        });

        egui::CentralPanel::default().show(context, |ui| {
            let page_rect = self.page.rect();
            let area = ui.available_rect_before_wrap();
            let scale = (area.width() / page_rect.width()).min(area.height() / page_rect.height());
            let origin = area.min - egui::vec2(page_rect.min.x * scale, page_rect.min.y * scale);
            let sheet = ui.allocate_rect(area, egui::Sense::hover());
            sheet.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Image, true, "hoja de dibujo")
            });
            let mut drawing = EguiSurface::new(context, origin, scale);
            let started = Instant::now();
            sheet::draw(&mut drawing, &self.page, &self.block);
            self.scale = scale;
            self.frame_ms = started.elapsed().as_secs_f64() * 1000.0;
            let painter = ui.painter().with_clip_rect(ui.max_rect());
            painter.extend(drawing.shapes);
        });
    }
}

#[cfg(test)]
mod tests {
    use eframe::egui;
    use punchi_fluid_draw::surface::Recording;

    use super::{Window, install_system_font, sheet};

    #[test]
    fn the_desktop_font_lays_out_spanish_text() {
        let context = egui::Context::default();
        if install_system_font(&context).is_err() {
            eprintln!("no system font on this machine: the window cannot start here");
            return;
        }
        let mut width = 0.0;
        let _ = context.run(egui::RawInput::default(), |context| {
            let galley = context.fonts(|fonts| {
                fonts.layout_no_wrap(
                    "áñ".to_owned(),
                    egui::FontId::proportional(12.0),
                    egui::Color32::BLACK,
                )
            });
            width = galley.size().x;
        });
        assert!(width > 0.0, "the accents must have glyphs");
    }

    #[test]
    fn the_sheet_of_a_new_document_draws_into_a_recording() {
        let window = Window::new();
        let mut recording = Recording::default();
        sheet::draw(&mut recording, &window.page, &window.block);
        assert!(
            !recording.is_empty(),
            "the sheet must produce calls: {}",
            recording.len()
        );
    }
}
