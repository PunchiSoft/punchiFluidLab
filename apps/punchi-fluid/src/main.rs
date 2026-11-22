// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The window: one fixed layout over the document's sheet.
//!
//! This is the first slice of ADR 0035's phase 3: the window opens at the 1366 × 768 floor,
//! draws the sheet of a new document and reports the fit and the drawing's own time. The
//! ribbon, the sidebar and the document's commands arrive in the following slices. Colour,
//! type and motion come from the desktop through `theme`.

use std::time::Instant;

use eframe::egui;
use punchi_fluid_doc::Document;
use punchi_fluid_export::{Page, TitleBlock, sheet};

mod ribbon;
mod surface;
mod theme;

use surface::EguiSurface;

fn main() -> eframe::Result<()> {
    let theme = theme::Theme::from_desktop();
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
            Ok(Box::new(Window::new(theme)))
        }),
    )
}

/// What the window owns today: the sheet of a new document, the desktop's theme and the last
/// frame's numbers.
struct Window {
    page: Page,
    block: TitleBlock,
    scale: f32,
    frame_ms: f64,
    theme: theme::Theme,
    ribbon: ribbon::Ribbon,
}

impl Window {
    fn new(theme: theme::Theme) -> Self {
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
            theme,
            ribbon: ribbon::Ribbon::new(),
        }
    }
}

impl eframe::App for Window {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        let palette = ribbon::Palette::from_theme(&self.theme.tokens);
        egui::TopBottomPanel::top("ribbon").show(context, |ui| {
            self.ribbon.show(ui, &palette);
        });

        let (width, height) = self.page.size_mm();
        let ink = self.theme.tokens.ink;
        let last = self.ribbon.last.clone();
        egui::TopBottomPanel::bottom("status").show(context, |ui| {
            ui.label(
                egui::RichText::new(format!(
                    "hoja {width:.0}×{height:.0} mm · {:.2} px/mm · {:.2} ms de dibujo · último: {last}",
                    self.scale, self.frame_ms
                ))
                .color(ink),
            );
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
    use punchi_fluid_draw::surface::Recording;

    use super::{Window, sheet, theme};

    #[test]
    fn the_sheet_of_a_new_document_draws_into_a_recording() {
        let window = Window::new(theme::Theme::fallback());
        let mut recording = Recording::default();
        sheet::draw(&mut recording, &window.page, &window.block);
        assert!(
            !recording.is_empty(),
            "the sheet must produce calls: {}",
            recording.len()
        );
    }
}
