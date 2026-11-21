// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a component sits: off the sheet, on top of another one, or under the title block.

use punchi_fluid_doc::Document;

use punchi_fluid_draw::surface::{Pt, Rect};

use crate::{Finding, FindingKind};

/// Faults about where a component sits: off the sheet, or on top of another one.
pub(crate) fn placement_faults(
    document: &Document,
    sheet: Rect,
    circuit: u32,
    reserved: &[Rect],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (index, placement) in document.placements().iter().enumerate() {
        if placement.circuit != circuit {
            continue;
        }
        let rect = placement.rect();
        if !contains_rect(sheet, rect) {
            findings.push(Finding {
                kind: FindingKind::OutsideSheet,
                at: rect.center(),
                component: Some(index),
                pipe: None,
            });
        }
        if reserved.iter().any(|area| intersect(rect, *area).is_some()) {
            findings.push(Finding {
                kind: FindingKind::UnderTitleBlock,
                at: rect.center(),
                component: Some(index),
                pipe: None,
            });
        }
    }

    for (index, placement) in document.placements().iter().enumerate() {
        if placement.circuit != circuit {
            continue;
        }
        for (other, candidate) in document.placements().iter().enumerate().skip(index + 1) {
            if candidate.circuit != circuit {
                continue;
            }
            let overlap = intersect(placement.rect(), candidate.rect());
            if let Some(area) = overlap {
                findings.push(Finding {
                    kind: FindingKind::OverlappingComponents,
                    at: area.center(),
                    component: Some(other),
                    pipe: None,
                });
            }
        }
    }

    findings
}

pub(crate) fn contains_rect(outer: Rect, inner: Rect) -> bool {
    inner.min.x >= outer.min.x
        && inner.min.y >= outer.min.y
        && inner.max.x <= outer.max.x
        && inner.max.y <= outer.max.y
}

/// The overlap of two rectangles, when it has area.
///
/// The bounds are compared before they are turned into a `Rect`, because building one from
/// two corners normalises them: two boxes far apart would produce a rectangle that spans
/// the gap between them and a positive area out of nothing.
pub(crate) fn intersect(a: Rect, b: Rect) -> Option<Rect> {
    let min_x = a.min.x.max(b.min.x);
    let min_y = a.min.y.max(b.min.y);
    let max_x = a.max.x.min(b.max.x);
    let max_y = a.max.y.min(b.max.y);
    // Touching edges is not an overlap: two components may sit side by side.
    if max_x - min_x > 0.5 && max_y - min_y > 0.5 {
        Some(Rect::from_corners(
            Pt::new(min_x, min_y),
            Pt::new(max_x, max_y),
        ))
    } else {
        None
    }
}
