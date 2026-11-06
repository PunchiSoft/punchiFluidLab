// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The export panel: a window, never a modal, so the sheet stays visible behind it.

use std::path::PathBuf;

use eframe::egui;
use punchi_fluid_export::{Margins, Orientation, Page, PaperSize, TitleBlock};
use punchi_fluid_theme::{Theme, Token};

/// File the panel writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// One vector page.
    Pdf,
    /// One bitmap at a chosen resolution.
    Png,
}

impl Format {
    /// Extension the destination should carry.
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Png => "png",
        }
    }

    /// Name shown on the format button.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Pdf => "PDF",
            Self::Png => "PNG",
        }
    }
}

/// Everything the panel edits and remembers between frames.
#[derive(Debug)]
pub struct ExportState {
    /// The panel is visible.
    pub open: bool,
    /// Which file to write.
    pub format: Format,
    /// Resolution of a PNG export.
    pub dpi: f32,
    /// Paper and margins.
    pub page: Page,
    /// Text of the identification block.
    pub block: TitleBlock,
    /// Destination file, as typed.
    pub destination: String,
    /// Result of the last attempt, shown in the panel and the status bar.
    pub message: Option<String>,
}

impl ExportState {
    /// A4 portrait with the note the review folder demands.
    #[must_use]
    pub fn new() -> Self {
        let block = punchi_fluid_export::block_with_review_state("Pneumatic circuit", today());
        Self {
            open: false,
            format: Format::Pdf,
            dpi: 300.0,
            page: Page::a4(block.clone()),
            destination: default_destination(),
            block,
            message: None,
        }
    }

    /// Destination as a path, with the extension of the chosen format.
    #[must_use]
    pub fn path(&self) -> PathBuf {
        let path = PathBuf::from(self.destination.trim());
        if path.as_os_str().is_empty() {
            return path;
        }
        path.with_extension(self.format.extension())
    }
}

impl Default for ExportState {
    fn default() -> Self {
        Self::new()
    }
}

/// Today in `YYYY-MM-DD`, from the system clock without pulling a time crate.
fn today() -> String {
    // The title block wants a date, not a time zone: seconds since the epoch converted
    // by the calendar rules is enough and keeps the dependency list at zero.
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    let days = i64::try_from(seconds / 86_400).unwrap_or(0);
    civil_from_days(days)
}

fn civil_from_days(days: i64) -> String {
    // Howard Hinnant's algorithm, adjusted to an epoch of 1970-01-01.
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { year + 1 } else { year };
    format!("{year:04}-{month:02}-{day:02}")
}

fn default_destination() -> String {
    let base = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    let documents = base.join("Documents");
    let folder = if documents.is_dir() { documents } else { base };
    folder.join("punchi-fluid-lab.pdf").display().to_string()
}

/// Draws the panel. Returns `true` when the user asked to export.
pub fn show(state: &mut ExportState, context: &egui::Context, theme: &Theme) -> bool {
    if !state.open {
        return false;
    }
    let mut export = false;
    let mut open = state.open;
    egui::Window::new("Export PDF")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(theme.metrics.help_width * 1.2)
        .show(context, |ui| {
            format_row(ui, state);
            paper_rows(ui, state);

            ui.separator();
            egui::Grid::new("title-block-fields")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Title");
                    ui.text_edit_singleline(&mut state.block.title);
                    ui.end_row();
                    ui.label("Project");
                    ui.text_edit_singleline(&mut state.block.project);
                    ui.end_row();
                    ui.label("Revision");
                    ui.text_edit_singleline(&mut state.block.revision);
                    ui.end_row();
                    ui.label("Date");
                    ui.text_edit_singleline(&mut state.block.date);
                    ui.end_row();
                });

            ui.separator();
            ui.label("Destination");
            ui.text_edit_singleline(&mut state.destination);

            let (width_mm, height_mm) = state.page.size_mm();
            let detail = match state.format {
                Format::Pdf => String::from("one page · vector"),
                Format::Png => {
                    let (pixels_x, pixels_y) = state.page.pixel_size(state.dpi);
                    format!("{pixels_x} × {pixels_y} px · {} dpi", state.dpi.round())
                }
            };
            ui.label(
                egui::RichText::new(format!(
                    "{} × {} mm · {detail}",
                    width_mm.round(),
                    height_mm.round()
                ))
                .small()
                .color(theme.colour(Token::TextMuted)),
            );

            if let Some(note) = &state.block.note {
                ui.label(
                    egui::RichText::new(note)
                        .small()
                        .color(theme.colour(Token::Warning)),
                );
            }

            if ui
                .button("Export")
                .on_hover_text(punchi_fluid_ui::Action::ExportPdf.tooltip())
                .clicked()
            {
                export = true;
            }
            if let Some(message) = &state.message {
                let colour = if message.starts_with("Wrote") {
                    theme.colour(Token::Ok)
                } else {
                    theme.colour(Token::Danger)
                };
                ui.label(egui::RichText::new(message).small().color(colour));
            }
        });
    state.open = open;
    export
}

/// Format and resolution.
fn format_row(ui: &mut egui::Ui, state: &mut ExportState) {
    ui.horizontal(|ui| {
        ui.label("Format");
        for format in [Format::Pdf, Format::Png] {
            if ui
                .selectable_label(state.format == format, format.label())
                .clicked()
            {
                state.format = format;
            }
        }
        if state.format == Format::Png {
            ui.label("DPI");
            ui.add(egui::DragValue::new(&mut state.dpi).range(72.0..=600.0));
        }
    });
}

/// Paper, orientation and margins.
fn paper_rows(ui: &mut egui::Ui, state: &mut ExportState) {
    ui.horizontal(|ui| {
        ui.label("Paper");
        for paper in PaperSize::ALL {
            let selected = state.page.paper == paper;
            if ui.selectable_label(selected, paper.label()).clicked() {
                state.page.paper = paper;
            }
        }
    });
    ui.horizontal(|ui| {
        ui.label("Orientation");
        for (orientation, label) in [
            (Orientation::Portrait, "Portrait"),
            (Orientation::Landscape, "Landscape"),
        ] {
            let selected = state.page.orientation == orientation;
            if ui.selectable_label(selected, label).clicked() {
                state.page.orientation = orientation;
            }
        }
    });
    ui.horizontal(|ui| {
        ui.label("Margin (mm)");
        ui.add(egui::DragValue::new(&mut state.page.margins.top).range(0.0..=60.0));
        state.page.margins.bottom = state.page.margins.top;
        state.page.margins.left = state.page.margins.top;
        state.page.margins.right = state.page.margins.top;
    });
}

/// Refuses a destination that is a folder or empty, with a message a user can act on.
pub fn check_destination(state: &ExportState) -> Result<PathBuf, String> {
    let path = state.path();
    if path.as_os_str().is_empty() {
        return Err(String::from("Choose a destination file"));
    }
    if path.is_dir() {
        return Err(String::from("The destination is a folder, not a file"));
    }
    let margins: Margins = state.page.margins;
    if !margins.is_valid() {
        return Err(String::from("The margins must be zero or positive"));
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_date_is_written_the_way_a_title_block_wants_it() {
        assert_eq!(civil_from_days(0), "1970-01-01");
        assert_eq!(civil_from_days(19_000), "2022-01-08");
        assert_eq!(civil_from_days(20_000), "2024-10-04");
    }

    #[test]
    fn an_empty_destination_is_refused_with_a_reason() {
        let mut state = ExportState::new();
        state.destination = String::new();
        assert!(check_destination(&state).is_err());
    }

    #[test]
    fn the_panel_starts_with_the_note_if_a_review_is_open() {
        let state = ExportState::new();
        if punchi_fluid_catalog::reviews::all_complete() {
            assert!(state.block.note.is_none());
        } else {
            assert!(state.block.note.is_some());
        }
    }
}
