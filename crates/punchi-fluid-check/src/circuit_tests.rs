// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests for the ninth fault, the pipe that joins two circuits.

use super::*;
use punchi_fluid_doc::Placement;

fn catalog() -> Catalog {
    Catalog::builtin()
}

fn sheet() -> Rect {
    Rect::from_min_size(Pt::new(0.0, 0.0), 200.0, 130.0)
}

#[test]
fn a_pipe_between_two_circuits_is_the_ninth_fault() {
    let mut document = Document::new();
    let supply = document.place(Placement::new(
        "supply.compressor",
        Pt::new(20.0, 20.0),
        30.0,
        16.0,
    ));
    let supply = document.placements()[supply].id();
    document.add_circuit("Control");
    let valve = document.place(Placement::new(
        "valve.3_2.spring_return",
        Pt::new(100.0, 80.0),
        30.0,
        16.0,
    ));
    let valve = document.placements()[valve].id();
    assert!(document.connect(
        punchi_fluid_doc::Endpoint::new(supply, "1"),
        punchi_fluid_doc::Endpoint::new(valve, "1"),
    ));

    let findings = check(&document, &catalog(), sheet(), 1);
    let joined = findings
        .iter()
        .find(|finding| finding.kind == FindingKind::PipeAcrossCircuits)
        .expect("the cross-circuit pipe is reported");
    assert!(joined.pipe.is_some());
}
