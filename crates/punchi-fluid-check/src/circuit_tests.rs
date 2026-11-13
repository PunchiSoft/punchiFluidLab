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

#[cfg(test)]
mod termination_tests {
    use super::*;
    use punchi_fluid_doc::Placement;

    fn catalog() -> Catalog {
        Catalog::builtin()
    }

    fn sheet() -> Rect {
        Rect::from_min_size(Pt::new(0.0, 0.0), 200.0, 130.0)
    }

    #[test]
    fn a_capped_port_is_not_an_open_connection() {
        // A working port with nothing on it is a fault; the same port with a blind plug is
        // a decision the drawing states, and the check has to read it that way.
        let mut document = Document::new();
        let valve = document.place(Placement::new(
            "valve.3_2.spring_return",
            Pt::new(60.0, 40.0),
            30.0,
            16.0,
        ));
        let valve_id = document.placements()[valve].id();
        let supply = document.place(Placement::new(
            "supply.compressor",
            Pt::new(20.0, 40.0),
            30.0,
            16.0,
        ));
        let supply_id = document.placements()[supply].id();
        assert!(document.connect(
            punchi_fluid_doc::Endpoint::new(supply_id, "1"),
            punchi_fluid_doc::Endpoint::new(valve_id, "1"),
        ));

        let open = check(&document, &catalog(), sheet(), 0);
        assert!(
            open.iter()
                .any(|finding| finding.kind == FindingKind::OpenConnection),
            "the unconnected working port is reported"
        );

        let plug = document.place(Placement::new(
            "plug.blind",
            Pt::new(100.0, 40.0),
            20.0,
            10.0,
        ));
        let plug_id = document.placements()[plug].id();
        assert!(document.connect(
            punchi_fluid_doc::Endpoint::new(valve_id, "2"),
            punchi_fluid_doc::Endpoint::new(plug_id, "1"),
        ));

        let capped = check(&document, &catalog(), sheet(), 0);
        assert!(
            !capped
                .iter()
                .any(|finding| finding.kind == FindingKind::OpenConnection),
            "the capped port is still reported: {:?}",
            capped
                .iter()
                .map(|finding| finding.kind)
                .collect::<Vec<_>>()
        );
    }
}

#[cfg(test)]
mod electrical_tests {
    use super::*;
    use punchi_fluid_doc::Placement;

    fn sheet() -> Rect {
        Rect::from_min_size(Pt::new(0.0, 0.0), 200.0, 130.0)
    }

    #[test]
    fn an_open_electrical_terminal_is_not_a_pneumatic_fault() {
        // A terminal with no wire is an unfinished drawing, not escaping air: the check
        // reports the hazards of the pneumatic circuit and leaves the control circuit alone.
        let mut document = Document::new();
        document.place(Placement::new(
            "electrical.supply",
            Pt::new(40.0, 40.0),
            30.0,
            16.0,
        ));
        let findings = check(&document, &Catalog::builtin(), sheet(), 0);
        assert!(
            !findings
                .iter()
                .any(|finding| finding.kind == FindingKind::OpenConnection),
            "an electrical terminal was reported as an open pneumatic port"
        );
    }
}

#[cfg(test)]
mod connection_tests {
    use super::*;
    use punchi_fluid_doc::Placement;

    fn catalog() -> Catalog {
        Catalog::builtin()
    }

    fn sheet() -> Rect {
        Rect::from_min_size(Pt::new(0.0, 0.0), 200.0, 130.0)
    }

    #[test]
    fn a_hose_on_an_electrical_terminal_is_the_eighth_fault() {
        // Fault 5's second half: the two ends do not fit. The network would treat the
        // terminal as a volume, so the check has to point at the pipe first.
        let mut document = Document::new();
        let supply = document.place(Placement::new(
            "supply.compressor",
            Pt::new(30.0, 30.0),
            30.0,
            16.0,
        ));
        let supply = document.placements()[supply].id();
        let contact = document.place(Placement::new(
            "electrical.contact",
            Pt::new(90.0, 30.0),
            20.0,
            12.0,
        ));
        let contact_id = document.placements()[contact].id();
        assert!(document.connect(
            punchi_fluid_doc::Endpoint::new(supply, "1"),
            punchi_fluid_doc::Endpoint::new(contact_id, "1"),
        ));

        let findings = check(&document, &catalog(), sheet(), 0);
        let mismatched = findings
            .iter()
            .find(|finding| finding.kind == FindingKind::MismatchedConnection)
            .expect("the mixed connection is reported");
        assert!(mismatched.pipe.is_some());
    }

    #[test]
    fn a_wire_between_two_terminals_and_a_hose_between_ports_are_not_faults() {
        let mut document = Document::new();
        let rail = document.place(Placement::new(
            "electrical.supply",
            Pt::new(30.0, 30.0),
            30.0,
            16.0,
        ));
        let rail = document.placements()[rail].id();
        let contact = document.place(Placement::new(
            "electrical.contact",
            Pt::new(90.0, 30.0),
            20.0,
            12.0,
        ));
        let contact_id = document.placements()[contact].id();
        assert!(document.connect(
            punchi_fluid_doc::Endpoint::new(rail, "L"),
            punchi_fluid_doc::Endpoint::new(contact_id, "1"),
        ));
        let supply = document.place(Placement::new(
            "supply.compressor",
            Pt::new(30.0, 80.0),
            30.0,
            16.0,
        ));
        let supply = document.placements()[supply].id();
        let valve = document.place(Placement::new(
            "valve.3_2.spring_return",
            Pt::new(90.0, 80.0),
            30.0,
            16.0,
        ));
        let valve = document.placements()[valve].id();
        assert!(document.connect(
            punchi_fluid_doc::Endpoint::new(supply, "1"),
            punchi_fluid_doc::Endpoint::new(valve, "1"),
        ));

        let findings = check(&document, &catalog(), sheet(), 0);
        assert!(
            !findings
                .iter()
                .any(|finding| finding.kind == FindingKind::MismatchedConnection),
            "a wire and a hose were called mismatched: {:?}",
            findings
                .iter()
                .map(|finding| finding.kind)
                .collect::<Vec<_>>()
        );
    }
}
