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

use punchi_fluid_catalog::Catalog;
use punchi_fluid_doc::Document;
use punchi_fluid_draw::placed::{Pipe, Placed, route_length_mm};

use crate::Window;
use punchi_fluid_export::Labels;
use punchi_fluid_i18n::Language;

impl Window {
    /// `Exportar`: opens the dialog in export mode, starting where the document or the
    /// reader is, exactly like `Guardar como`.
    pub(crate) fn export_dialog(&mut self) {
        let start = crate::files::start_directory(self.path.as_deref());
        self.dialog = Some(crate::files::Dialog::open_export(
            start,
            self.export_options.format.extension(),
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
            .ok_or_else(|| String::from(self.language.text("notice.export_hint")))?;
        if !crate::files::is_export_extension(&extension) {
            return Err(self
                .language
                .format("notice.export_extension", &[("extension", &extension)]));
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
            return Err(String::from(self.language.text("notice.nothing_to_export")));
        };
        // The page the window draws, with the block this export prints and the parts list
        // when the reader asked for one.
        let labels = export_labels(self.language);
        let parts = self
            .export_options
            .parts
            .then(|| {
                parts_list(
                    &self.document,
                    &self.catalog,
                    &scene.placed,
                    &scene.pipes,
                    self.document.active_circuit(),
                    &labels,
                )
            })
            .filter(|parts| !parts.is_empty());
        let page = punchi_fluid_export::Page {
            title_block: Some(self.block.clone()),
            parts,
            labels,
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
                let bytes =
                    punchi_fluid_export::render_pdf_with(&page, block, draw).map_err(|cause| {
                        self.language
                            .format("notice.pdf_failed", &[("cause", &cause.to_string())])
                    })?;
                write_atomically(path, "pdf", &bytes, self.language)
            }
            "png" => {
                let bytes = punchi_fluid_export::render_png_with(
                    &page,
                    block,
                    f32::from(self.export_options.dpi),
                    draw,
                )
                .map_err(|cause| {
                    self.language
                        .format("notice.png_failed", &[("cause", &cause.to_string())])
                })?;
                write_atomically(path, "png", &bytes, self.language)
            }
            "svg" => {
                let text =
                    punchi_fluid_export::render_svg_with(&page, block, draw).map_err(|cause| {
                        self.language
                            .format("notice.svg_failed", &[("cause", &cause.to_string())])
                    })?;
                write_atomically(path, "svg", text.as_bytes(), self.language)
            }
            _ => Err(format!(
                "«{extension}» no se puede exportar: elige pdf, png o svg."
            )),
        }
    }
}

/// The parts list of the active circuit: what the drawing holds, its port closures and its
/// drawn tubing, grouped by definition.
fn parts_list(
    document: &Document,
    catalog: &Catalog,
    placements: &[Placed<'_>],
    pipes: &[Pipe<'_>],
    circuit: u32,
    labels: &Labels,
) -> punchi_fluid_export::PartsList {
    let placement_rows = document
        .placements()
        .iter()
        .filter(|placement| placement.circuit == circuit)
        .map(|placement| {
            (
                placement.catalog_id.as_str(),
                placement.designation.as_deref(),
            )
        });
    let closures = document
        .placements()
        .iter()
        .filter(|placement| placement.circuit == circuit)
        .flat_map(|placement| {
            placement.port_closures.iter().map(move |(port, closure)| {
                (
                    placement.catalog_id.as_str(),
                    placement.designation.as_deref(),
                    port.as_str(),
                    closure.as_str(),
                )
            })
        });
    let tubes = pipes.iter().enumerate().filter_map(|(index, pipe)| {
        let length_mm = route_length_mm(placements, pipes, catalog, index)?;
        Some(punchi_fluid_export::parts::TubeRowInput {
            od: pipe.tube_od_hundredths,
            class: pipe.class,
            length_mm,
        })
    });
    punchi_fluid_export::PartsList::group(catalog, placement_rows, closures, tubes, labels)
}

/// The words the printed page carries in the reader's language.
fn export_labels(language: Language) -> Labels {
    match language {
        Language::English => Labels::english(),
        Language::Spanish => Labels::spanish(),
    }
}

/// Writes `bytes` to `path` through a sibling `.part` file, so a crash leaves no half file:
/// the same rule `punchi_fluid_export::write_pdf` follows.
fn write_atomically(
    path: &Path,
    extension: &str,
    bytes: &[u8],
    language: Language,
) -> Result<(), String> {
    let temporary = path.with_extension(format!("{extension}.part"));
    std::fs::write(&temporary, bytes).map_err(|cause| {
        language.format(
            "notice.write_failed",
            &[
                ("path", &temporary.display().to_string()),
                ("cause", &cause.to_string()),
            ],
        )
    })?;
    std::fs::rename(&temporary, path).map_err(|cause| {
        language.format(
            "notice.replace_failed",
            &[
                ("path", &path.display().to_string()),
                ("cause", &cause.to_string()),
            ],
        )
    })
}

#[cfg(test)]
mod tests {
    use punchi_fluid_catalog::default_catalog;
    use punchi_fluid_doc::{Document, Placement};
    use punchi_fluid_draw::Pt;

    use super::parts_list;

    #[test]
    fn the_parts_list_groups_the_placed_component() {
        let mut document = Document::new();
        document.place(Placement::new(
            "valve.3_2.spring_return",
            Pt::new(60.0, 60.0),
            24.0,
            24.0,
        ));
        let catalog = default_catalog();
        let circuit = document.active_circuit();
        let scene = crate::scene::build(&document, &catalog, circuit, None);
        let parts = parts_list(
            &document,
            &catalog,
            &scene.placed,
            &scene.pipes,
            circuit,
            &punchi_fluid_export::Labels::english(),
        );
        assert_eq!(parts.rows.len(), 1, "one definition on the sheet");
        assert_eq!(parts.rows[0].quantity, 1);
        assert!(parts.tubes.is_empty(), "no pipes on the sheet");
    }
}
