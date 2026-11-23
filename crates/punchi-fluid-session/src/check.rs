// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The design check, reached through the session.
//!
//! The rule lives in `punchi-fluid-check`; the window must not depend on that crate, so the
//! session forwards the document's check in one function (ADR 0044). It adds no policy of
//! its own: it takes the document, the catalog, the drawing space the caller measures
//! against and the circuit, and returns the check's own answer.

use punchi_fluid_catalog::Catalog;
use punchi_fluid_doc::Document;
use punchi_fluid_draw::surface::Rect;

pub use punchi_fluid_check::{Finding, FindingKind};

/// Every fault the design check reports for `circuit` against `sheet`, in the check's order.
///
/// `sheet` is the drawing space in millimetres, the frame the window draws and prints; the
/// caller computes it from the document's paper so the headless and the window paths measure
/// the same area.
#[must_use]
pub fn check_circuit(
    document: &Document,
    catalog: &Catalog,
    sheet: Rect,
    circuit: u32,
) -> Vec<punchi_fluid_check::Finding> {
    punchi_fluid_check::check(document, catalog, sheet, circuit)
}

#[cfg(test)]
mod tests {
    use punchi_fluid_catalog::default_catalog;
    use punchi_fluid_doc::{Document, Placement};
    use punchi_fluid_draw::surface::{Pt, Rect};

    use super::check_circuit;

    fn area() -> Rect {
        Rect::from_min_size(Pt::new(20.0, 10.0), 257.0, 190.0)
    }

    #[test]
    fn the_check_forwards_the_documents_findings() {
        let mut document = Document::new();
        document.place(Placement::new(
            "valve.3_2.spring_return",
            Pt::new(60.0, 60.0),
            20.0,
            20.0,
        ));
        let catalog = default_catalog();
        let circuit = document.active_circuit();
        let findings = check_circuit(&document, &catalog, area(), circuit);
        assert!(
            findings
                .iter()
                .any(|finding| finding.kind == punchi_fluid_check::FindingKind::OpenConnection),
            "the open ports are reported: {findings:?}"
        );
        assert_eq!(
            findings,
            punchi_fluid_check::check(&document, &catalog, area(), circuit),
            "the session forwards the check unchanged"
        );
    }

    #[test]
    fn an_empty_sheet_has_no_faults() {
        let document = Document::new();
        let catalog = default_catalog();
        let findings = check_circuit(&document, &catalog, area(), document.active_circuit());
        assert!(findings.is_empty(), "nothing placed, nothing to fault");
    }
}
