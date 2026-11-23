// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The export command: the drawing on a standard sheet, written as PDF, SVG or PNG.
//!
//! An export prints the document; it is not a save. The file the document came from, the
//! modified state and the safety copy stay exactly where they were, so a reader can print a
//! draft and keep editing it.
//!
//! The drawing itself is the ABI's own shape (`crates/punchi-fluid-ffi/src/lib.rs`): the
//! scene of the active circuit, its bounds measured by the same code the page draws with,
//! and a fitted surface so a large circuit shrinks instead of being cropped. The page, the
//! frame and the identification block come from `punchi-fluid-export`; this module only
//! chooses the writer by the extension the reader typed.

use std::path::Path;

use crate::Window;

/// Resolution of the PNG export: the print resolution the export offers.
const PNG_DPI: f32 = 300.0;

impl Window {
    /// `Exportar`: opens the dialog in export mode, starting where the document or the
    /// reader is, exactly like `Guardar como`.
    pub(crate) fn export_dialog(&mut self) {
        let start = crate::files::start_directory(self.path.as_deref());
        self.dialog = Some(crate::files::Dialog::open(
            crate::files::Mode::Export,
            start,
        ));
    }

    /// Prints the active circuit of the document to `path`, choosing the writer by the
    /// path's extension. The page carries the drawing's own title block and no parts list.
    ///
    /// # Errors
    ///
    /// Returns a Spanish message the dialog can show when the path asks for a format this
    /// build does not render, when the drawing has nothing to print, or when the writer
    /// cannot render or write the file.
    pub(crate) fn export_to(&mut self, path: &Path) -> Result<(), String> {
        let extension = path
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .map(str::to_ascii_lowercase)
            .ok_or_else(|| {
                String::from("Para exportar, escribe un nombre con extensión pdf, png o svg.")
            })?;
        if !crate::files::is_export_extension(&extension) {
            return Err(format!(
                "«{extension}» no se puede exportar: elige pdf, png o svg."
            ));
        }
        let scene = crate::scene::build(
            &self.document,
            &self.catalog,
            self.document.active_circuit(),
            self.model.as_ref(),
        );
        let Some(bounds) =
            punchi_fluid_draw::bounds_placed(&scene.placed, &scene.pipes, &self.catalog)
        else {
            return Err(String::from("El dibujo no tiene nada que exportar."));
        };
        // The page the window draws, with the block this export prints. The parts list is
        // not part of the window's page and an export does not add one.
        let page = punchi_fluid_export::Page {
            title_block: Some(self.block.clone()),
            parts: None,
            ..self.page.clone()
        };
        let block = &self.block;
        let catalog = &self.catalog;
        let draw = |surface: &mut dyn punchi_fluid_draw::Surface, area: punchi_fluid_draw::Rect| {
            let mut fitted = punchi_fluid_draw::surface::Scaled::fit(surface, bounds, area);
            punchi_fluid_draw::draw_placed(
                &mut fitted,
                &scene.placed,
                &scene.pipes,
                catalog,
                &punchi_fluid_export::print_ink(),
                &[],
            );
        };
        match extension.as_str() {
            "pdf" => {
                let bytes = punchi_fluid_export::render_pdf_with(&page, block, draw)
                    .map_err(|cause| format!("no se pudo escribir el PDF: {cause}"))?;
                write_atomically(path, "pdf", &bytes)
            }
            "png" => {
                let bytes = punchi_fluid_export::render_png_with(&page, block, PNG_DPI, draw)
                    .map_err(|cause| format!("no se pudo escribir el PNG: {cause}"))?;
                write_atomically(path, "png", &bytes)
            }
            "svg" => {
                let text = punchi_fluid_export::render_svg_with(&page, block, draw)
                    .map_err(|cause| format!("no se pudo escribir el SVG: {cause}"))?;
                write_atomically(path, "svg", text.as_bytes())
            }
            _ => Err(format!(
                "«{extension}» no se puede exportar: elige pdf, png o svg."
            )),
        }
    }
}

/// Writes `bytes` to `path` through a sibling `.part` file, so a crash leaves no half file:
/// the same rule `punchi_fluid_export::write_pdf` follows.
fn write_atomically(path: &Path, extension: &str, bytes: &[u8]) -> Result<(), String> {
    let temporary = path.with_extension(format!("{extension}.part"));
    std::fs::write(&temporary, bytes)
        .map_err(|cause| format!("no se pudo escribir {}: {cause}", temporary.display()))?;
    std::fs::rename(&temporary, path)
        .map_err(|cause| format!("no se pudo reemplazar {}: {cause}", path.display()))
}
