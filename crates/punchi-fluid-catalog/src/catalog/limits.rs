// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Bounds a definition declares for its own drawing: the symbol box and the frame.
//!
//! Split from `catalog.rs` to keep both files under the architecture budget; the checks are
//! called once by the loader and their constants are re-exported from the catalog module.

use std::path::Path;

use crate::error::CatalogError;
use crate::schema::ComponentDefinition;

use super::invalid;

/// Smallest symbol box a definition may declare, in millimetres.
///
/// A box below this cannot hold a legible numeral, so a file asking for one is malformed
/// rather than a small symbol.
pub const MIN_BOX_MM: f64 = 1.0;

/// Largest symbol box a definition may declare, in millimetres.
///
/// A symbol is read on a sheet, and the widest paper here is A4: a box beyond 200 mm would
/// not fit beside the title block and the margins, so the loader refuses it instead of
/// letting it draw off the page.
pub const MAX_BOX_MM: f64 = 200.0;

/// Checks the optional symbol box a file declares, in millimetres.
///
/// The box is a datum the palette, the drop and the templates all read, so it is checked
/// once here: finite, positive and inside [`MIN_BOX_MM`]..=[`MAX_BOX_MM`]. An absent box
/// is the canonical square and is not an error.
pub(super) fn check_box(definition: &ComponentDefinition, path: &Path) -> Result<(), CatalogError> {
    let Some([width, height]) = definition.box_mm else {
        return Ok(());
    };
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return Err(invalid(
            path,
            "a `box` needs a finite, positive width and height",
        ));
    }
    if !(MIN_BOX_MM..=MAX_BOX_MM).contains(&width) || !(MIN_BOX_MM..=MAX_BOX_MM).contains(&height) {
        return Err(invalid(
            path,
            &format!(
                "a `box` must be between {MIN_BOX_MM} and {MAX_BOX_MM} mm on each side, found {width} x {height}"
            ),
        ));
    }
    Ok(())
}

/// Checks the optional frame a file declares, in coordinates of the symbol box.
///
/// The frame is the line the port numerals are placed outside of, so it has to be a real
/// rectangle inside the box. The ports are not required to be inside it: a port outside the
/// frame is the case the field exists for, and the envelope is what contains both.
pub(super) fn check_frame(
    definition: &ComponentDefinition,
    path: &Path,
) -> Result<(), CatalogError> {
    let Some([x1, y1, x2, y2]) = definition.frame else {
        return Ok(());
    };
    if [x1, y1, x2, y2].iter().any(|value| !value.is_finite()) {
        return Err(invalid(path, "a `frame` must be finite"));
    }
    if !(0.0..=1.0).contains(&x1)
        || !(0.0..=1.0).contains(&y1)
        || !(0.0..=1.0).contains(&x2)
        || !(0.0..=1.0).contains(&y2)
    {
        return Err(invalid(
            path,
            "a `frame` must lie inside the symbol box, 0..=1",
        ));
    }
    if x2 <= x1 || y2 <= y1 {
        return Err(invalid(
            path,
            "a `frame` needs two corners in increasing order",
        ));
    }
    Ok(())
}
