// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Line classes, with their own pattern in the drawing.
//!
//! Split from `schema.rs` to keep both files under the architecture budget; the type is
//! re-exported from the schema module, so callers keep importing
//! `crate::schema::LineClass`.

use serde::{Deserialize, Serialize};

/// Kind of line, with its own pattern in the drawing.
///
/// The class is not decorative: a pilot line drawn solid is a misreading risk, so the
/// renderer picks the pattern and the definition only declares the class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LineClass {
    /// Working line, carries the fluid.
    Working,
    /// Pilot line, carries a control signal of the same fluid.
    Pilot,
    /// Drain or leak line.
    Drain,
    /// Electrical connection.
    Electrical,
    /// Assembly boundary: the frame that groups the functions of one unit.
    ///
    /// It is a chain line, not a drain or a pilot line drawn longer. Only a symbol
    /// primitive may declare it; a hose cannot carry it, because a hose is not an
    /// assembly boundary.
    Boundary,
}

impl LineClass {
    /// Stable name written in the definition file.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Working => "working",
            Self::Pilot => "pilot",
            Self::Drain => "drain",
            Self::Electrical => "electrical",
            Self::Boundary => "boundary",
        }
    }
}
