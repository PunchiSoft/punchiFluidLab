// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! A simulated circuit on a standard sheet.
//!
//! The drawing is not redone here: [`punchi_fluid_draw`] draws it once against a
//! [`Surface`], the window implements that trait over `egui` and this crate implements
//! it over a PDF content stream. The page, the margins and the identification block are
//! the only things this crate adds.

pub mod page;
pub mod pdf;
pub mod png;
pub mod sheet;

use std::path::Path;

use punchi_fluid_catalog::Catalog;
use punchi_fluid_draw::{Ink, Pt, Rect};
use punchi_fluid_solver::pneumatic::PneumaticSystem;
use thiserror::Error;

pub use page::{Margins, Orientation, Page, PaperSize, TitleBlock};
pub use pdf::{PdfSurface, assemble};

/// Why an export did not happen.
#[derive(Debug, Error)]
pub enum ExportError {
    /// The sheet is larger than the writer accepts.
    #[error("page {width_mm:.0} × {height_mm:.0} mm is larger than the writer accepts")]
    PageTooLarge {
        /// Width in millimetres.
        width_mm: f32,
        /// Height in millimetres.
        height_mm: f32,
    },

    /// The margins leave no room for the drawing.
    #[error("the margins leave no content area on {0}")]
    EmptyContent(&'static str),

    /// The request would produce something the writer will not build.
    #[error("refused: {0}")]
    Refused(String),

    /// The file could not be written.
    #[error("cannot write {path}: {source}")]
    Write {
        /// Destination.
        path: String,
        /// Underlying error.
        #[source]
        source: std::io::Error,
    },
}

/// Renders the circuit as a one-page PDF.
///
/// The bytes are a pure function of `page`, `block` and the circuit state: two calls
/// with the same inputs produce identical files, which is what makes an export testable.
///
/// # Errors
///
/// Returns [`ExportError::PageTooLarge`] when the sheet exceeds the writer's limit and
/// [`ExportError::EmptyContent`] when the margins leave no room.
pub fn render_pdf(
    page: &Page,
    block: &TitleBlock,
    system: &PneumaticSystem,
    catalog: &Catalog,
) -> Result<Vec<u8>, ExportError> {
    let (width_mm, height_mm) = page.size_mm();
    if width_mm * height_mm > pdf::MAX_PAGE_MM2 {
        return Err(ExportError::PageTooLarge {
            width_mm,
            height_mm,
        });
    }
    let inner = drawing_area(page, block)?;

    let mut surface = PdfSurface::new(page.rect());
    sheet::draw(&mut surface, page, block);
    punchi_fluid_draw::draw(&mut surface, inner, system, catalog, &print_ink());

    Ok(assemble(&surface))
}

/// Renders the circuit as a PNG at `dpi`.
///
/// # Errors
///
/// [`ExportError::Refused`] when the resolution or the bitmap size is outside the
/// budget, [`ExportError::EmptyContent`] when the margins leave no room.
pub fn render_png(
    page: &Page,
    block: &TitleBlock,
    system: &PneumaticSystem,
    catalog: &Catalog,
    dpi: f32,
) -> Result<Vec<u8>, ExportError> {
    let inner = drawing_area(page, block)?;
    let mut surface = png::PngSurface::new(page.rect(), dpi)?;
    sheet::draw(&mut surface, page, block);
    punchi_fluid_draw::draw(&mut surface, inner, system, catalog, &print_ink());
    surface.encode()
}

/// Area the drawing may use: the content, without the block and without the note band.
fn drawing_area(page: &Page, block: &TitleBlock) -> Result<Rect, ExportError> {
    let drawing = page.drawing_rect();
    if drawing.is_empty() {
        return Err(ExportError::EmptyContent(page.paper.label()));
    }
    let top = if block.note.is_some() { 6.0 } else { 0.0 };
    Ok(Rect::from_corners(
        Pt::new(drawing.min.x + 4.0, drawing.min.y + 4.0 + top),
        Pt::new(drawing.max.x - 4.0, drawing.max.y - 4.0),
    ))
}

/// Ink for paper: black on white, whatever the screen is wearing.
fn print_ink() -> Ink {
    let mut ink = Ink::print();
    ink.background = punchi_fluid_draw::Colour::WHITE;
    ink
}

/// Renders and writes the PDF, atomically: a crash leaves no half file.
///
/// # Errors
///
/// Returns the render error, or [`ExportError::Write`] when the destination cannot be
/// replaced.
pub fn write_pdf(
    path: &Path,
    page: &Page,
    block: &TitleBlock,
    system: &PneumaticSystem,
    catalog: &Catalog,
) -> Result<(), ExportError> {
    let bytes = render_pdf(page, block, system, catalog)?;
    let temporary = path.with_extension("pdf.part");
    std::fs::write(&temporary, &bytes).map_err(|source| ExportError::Write {
        path: temporary.display().to_string(),
        source,
    })?;
    std::fs::rename(&temporary, path).map_err(|source| ExportError::Write {
        path: path.display().to_string(),
        source,
    })?;
    Ok(())
}

/// Renders and writes a PNG, atomically like the PDF.
///
/// # Errors
///
/// Returns the render or the write error, never a half file.
pub fn write_png(
    path: &Path,
    page: &Page,
    block: &TitleBlock,
    system: &PneumaticSystem,
    catalog: &Catalog,
    dpi: f32,
) -> Result<(), ExportError> {
    let bytes = render_png(page, block, system, catalog, dpi)?;
    let temporary = path.with_extension("png.part");
    std::fs::write(&temporary, &bytes).map_err(|source| ExportError::Write {
        path: temporary.display().to_string(),
        source,
    })?;
    std::fs::rename(&temporary, path).map_err(|source| ExportError::Write {
        path: path.display().to_string(),
        source,
    })?;
    Ok(())
}

/// Note the export must carry while a symbol review is open.
#[must_use]
pub fn unverified_note() -> Option<String> {
    if punchi_fluid_catalog::reviews::all_complete() {
        return None;
    }
    Some(String::from("symbols not yet verified against ISO 1219-1"))
}

/// Title block with the note the review folder demands, no more and no less.
#[must_use]
pub fn block_with_review_state(title: &str, date: impl Into<String>) -> TitleBlock {
    let block = TitleBlock::new(title, date);
    match unverified_note() {
        Some(note) => TitleBlock {
            note: Some(note),
            ..block
        },
        None => block,
    }
}
