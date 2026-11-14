// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The rows of the title block, chosen by content model and per-field visibility.
//!
//! One layout feeds the export and, through the ABI, the canvas. The widths are written on
//! the guide's 180 mm A3 template and scaled to the paper's block width; a hidden field
//! collapses its cell and the rest of the row shares the freed width. A row with no visible
//! field disappears, so a school block is shorter than an industrial one.

use punchi_fluid_core::{ContentModel, TitleBlock, TitleField};

use crate::page::{PaperSize, is_large_sheet};

/// One cell of a template row. The sheet number is derived, not a [`TitleField`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockField {
    /// One of the stored fields.
    Stored(TitleField),
    /// The derived `n/N`.
    Sheet,
}

impl BlockField {
    /// `true` when the value is printed at the leading 5 mm height.
    #[must_use]
    pub fn is_heading(self) -> bool {
        matches!(self, Self::Stored(TitleField::Owner | TitleField::Title))
    }
}

/// One cell of the resolved layout, in millimetres of the block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockCell {
    /// Field the cell prints.
    pub field: BlockField,
    /// Width in millimetres.
    pub width: f32,
}

/// One row of the resolved layout.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockRow {
    /// Height in millimetres.
    pub height: f32,
    /// Cells, left to right.
    pub cells: Vec<BlockCell>,
}

/// The resolved block: its size and its rows.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockLayout {
    /// Width in millimetres.
    pub width: f32,
    /// Height in millimetres, after empty rows collapse.
    pub height: f32,
    /// Rows, top to bottom.
    pub rows: Vec<BlockRow>,
}

struct CellSpec {
    field: BlockField,
    width: f32,
}

struct RowSpec {
    height: f32,
    cells: Vec<CellSpec>,
}

/// A cell of the 180 mm template.
const fn cell(field: BlockField, width: f32) -> CellSpec {
    CellSpec { field, width }
}

const fn stored(field: TitleField, width: f32) -> CellSpec {
    cell(BlockField::Stored(field), width)
}

/// The rows of one content model on the 180 mm template, at the A3 height.
fn model_rows(model: ContentModel) -> (Vec<RowSpec>, f32) {
    match model {
        ContentModel::School => (
            vec![
                RowSpec {
                    height: 14.0,
                    cells: vec![
                        stored(TitleField::Owner, 70.0),
                        stored(TitleField::Title, 110.0),
                    ],
                },
                RowSpec {
                    height: 21.0,
                    cells: vec![
                        stored(TitleField::Prepared, 60.0),
                        stored(TitleField::Date, 55.0),
                        cell(BlockField::Sheet, 65.0),
                    ],
                },
            ],
            35.0,
        ),
        ContentModel::Technical => (
            vec![
                RowSpec {
                    height: 12.0,
                    cells: vec![
                        stored(TitleField::Owner, 50.0),
                        stored(TitleField::Title, 130.0),
                    ],
                },
                RowSpec {
                    height: 14.0,
                    cells: vec![
                        stored(TitleField::Number, 50.0),
                        stored(TitleField::Revision, 20.0),
                        cell(BlockField::Sheet, 25.0),
                        stored(TitleField::Date, 35.0),
                        stored(TitleField::Norms, 50.0),
                    ],
                },
                RowSpec {
                    height: 14.0,
                    cells: vec![
                        stored(TitleField::Prepared, 60.0),
                        stored(TitleField::Checked, 60.0),
                        stored(TitleField::Approved, 60.0),
                    ],
                },
            ],
            40.0,
        ),
        ContentModel::Industrial => (
            vec![
                RowSpec {
                    height: 14.0,
                    cells: vec![
                        stored(TitleField::Owner, 80.0),
                        stored(TitleField::Project, 60.0),
                        stored(TitleField::Equipment, 40.0),
                    ],
                },
                RowSpec {
                    height: 15.0,
                    cells: vec![stored(TitleField::Title, 180.0)],
                },
                RowSpec {
                    height: 12.0,
                    cells: vec![
                        stored(TitleField::Number, 70.0),
                        stored(TitleField::Revision, 20.0),
                        cell(BlockField::Sheet, 25.0),
                        stored(TitleField::Date, 35.0),
                        stored(TitleField::Scale, 30.0),
                    ],
                },
                RowSpec {
                    height: 12.0,
                    cells: vec![
                        stored(TitleField::Prepared, 60.0),
                        stored(TitleField::Checked, 60.0),
                        stored(TitleField::Approved, 60.0),
                    ],
                },
                RowSpec {
                    height: 12.0,
                    cells: vec![stored(TitleField::Norms, 180.0)],
                },
            ],
            65.0,
        ),
    }
}

/// Nominal size of a model on `paper`: 170 × 35/40/45 up to A4, 180 × 40/50/65 from A3.
#[must_use]
pub fn model_size_mm(paper: PaperSize, model: ContentModel) -> (f32, f32) {
    let large = is_large_sheet(paper);
    let width = if large { 180.0 } else { 170.0 };
    let height = match (model, large) {
        (ContentModel::School, false) => 35.0,
        (ContentModel::School, true) | (ContentModel::Technical, false) => 40.0,
        (ContentModel::Technical, true) => 50.0,
        (ContentModel::Industrial, true) => 65.0,
        (ContentModel::Industrial, false) => 45.0,
    };
    (width, height)
}

/// Resolves the block of `block` on `paper`, collapsing hidden fields and empty rows.
#[must_use]
pub fn layout(block: &TitleBlock, paper: PaperSize) -> BlockLayout {
    let (width, nominal_height) = model_size_mm(paper, block.model);
    let (specs, total_height) = model_rows(block.model);
    let scale = nominal_height / total_height;
    let mut rows = Vec::with_capacity(specs.len());
    let mut height = 0.0;
    for spec in specs {
        let mut cells: Vec<CellSpec> = spec
            .cells
            .into_iter()
            .filter(|cell| match cell.field {
                BlockField::Stored(field) => block.visible.contains(field),
                BlockField::Sheet => true,
            })
            .collect();
        if cells.is_empty() {
            continue;
        }
        let visible_width: f32 = cells.iter().map(|cell| cell.width).sum();
        let row_height = spec.height * scale;
        let factor = width / visible_width;
        let cells = cells
            .drain(..)
            .map(|cell| BlockCell {
                field: cell.field,
                width: cell.width * factor,
            })
            .collect();
        rows.push(BlockRow {
            height: row_height,
            cells,
        });
        height += row_height;
    }
    BlockLayout {
        width,
        height,
        rows,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use punchi_fluid_core::TitleBlock;

    fn size(paper: PaperSize, model: ContentModel) -> (f32, f32) {
        let mut block = TitleBlock::default();
        block.set_model(model);
        let layout = layout(&block, paper);
        (layout.width, layout.height)
    }

    #[test]
    fn each_model_has_its_paper_size() {
        for (model, a4, a3) in [
            (ContentModel::School, (170.0, 35.0), (180.0, 40.0)),
            (ContentModel::Technical, (170.0, 40.0), (180.0, 50.0)),
            (ContentModel::Industrial, (170.0, 45.0), (180.0, 65.0)),
        ] {
            let (w4, h4) = size(PaperSize::A4, model);
            assert!(
                (w4 - a4.0).abs() < 0.01 && (h4 - a4.1).abs() < 0.01,
                "{model:?} A4"
            );
            let (w3, h3) = size(PaperSize::A3, model);
            assert!(
                (w3 - a3.0).abs() < 0.01 && (h3 - a3.1).abs() < 0.01,
                "{model:?} A3"
            );
        }
    }

    #[test]
    fn a_hidden_field_collapses_its_cell_and_an_empty_row_shortens_the_block() {
        let mut block = TitleBlock::default();
        let before = layout(&block, PaperSize::A4);
        assert_eq!(before.rows[0].cells.len(), 3);

        block.set_visible(TitleField::Project, false);
        let after = layout(&block, PaperSize::A4);
        assert_eq!(after.rows[0].cells.len(), 2);
        let width: f32 = after.rows[0].cells.iter().map(|cell| cell.width).sum();
        assert!(
            (width - after.width).abs() < 0.01,
            "the row fills its width"
        );

        block.set_visible(TitleField::Owner, false);
        block.set_visible(TitleField::Equipment, false);
        let collapsed = layout(&block, PaperSize::A4);
        assert_eq!(collapsed.rows.len(), 4, "the empty first row disappears");
        assert!(collapsed.height < before.height);
    }

    #[test]
    fn a_school_block_carries_the_sheet_and_only_its_fields() {
        let mut block = TitleBlock::default();
        block.set_model(ContentModel::School);
        let layout = layout(&block, PaperSize::A4);
        let fields: Vec<BlockField> = layout
            .rows
            .iter()
            .flat_map(|row| row.cells.iter().map(|cell| cell.field))
            .collect();
        assert!(fields.contains(&BlockField::Sheet));
        assert!(fields.contains(&BlockField::Stored(TitleField::Owner)));
        assert!(!fields.contains(&BlockField::Stored(TitleField::Number)));
    }
}
