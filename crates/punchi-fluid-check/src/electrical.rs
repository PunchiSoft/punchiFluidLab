// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The electrical faults: what the control circuit gets wrong before it is run.
//!
//! The control circuit is a boolean net ([ADR 0018]): terminals are nodes, wires and closed
//! contacts are edges. That makes one fault decidable from the document alone, without a
//! simulation: the live rail joined to its own return. It is the ninth fault of the reference,
//! whose own ninth belongs to a model that couples a coil and its contact by name; ours
//! couples them by wire, so this is the equivalent mistake under our rules.
//!
//! [ADR 0018]: ../docs/adr/0018-the-control-circuit-is-a-boolean-net.md

use punchi_fluid_catalog::{Behaviour, Catalog, PortRole};
use punchi_fluid_doc::Document;

use crate::{Finding, FindingKind};

/// The ninth fault: the live rail joined to its own return.
///
/// A wire or a contact that joins the positive terminal of a supply to its return is a short,
/// and it is the electrical fault this model can decide on its own. The reference's ninth fault
/// — a label that does not match — belongs to a model that couples a coil and its contact by
/// name; ours couples them by wire, so the equivalent mistake is this one (the ledger and
/// [ADR 0018] say why).
///
/// Contacts start at the state their file declares, because the check runs before the
/// simulation and has no clicked state to read.
///
/// [ADR 0018]: ../docs/adr/0018-the-control-circuit-is-a-boolean-net.md
pub(crate) fn short_circuits(document: &Document, catalog: &Catalog, circuit: u32) -> Vec<Finding> {
    let mut findings = Vec::new();
    let net = ControlNet::of(document, catalog, circuit);
    for (index, placement) in document.placements().iter().enumerate() {
        if placement.circuit != circuit {
            continue;
        }
        let Some(definition) = catalog.get(&placement.catalog_id) else {
            continue;
        };
        if !matches!(definition.behaviour, Behaviour::ElectricalSupply { .. }) {
            continue;
        }
        let (Some(live), Some(ret)) = (
            definition.port.iter().find(|port| port.id == "L"),
            definition.port.iter().find(|port| port.id == "N"),
        ) else {
            continue;
        };
        let (Some(start), Some(goal)) = (
            net.index_of(placement.id(), &live.id),
            net.index_of(placement.id(), &ret.id),
        ) else {
            continue;
        };
        if !net.reaches(start, goal) {
            continue;
        }
        findings.push(Finding {
            kind: FindingKind::ShortCircuit,
            at: placement.rect().center(),
            component: Some(index),
            pipe: None,
        });
    }
    findings
}

/// A terminal of the control net: the placement it belongs to and its port.
#[derive(Debug, PartialEq, Eq)]
struct Terminal {
    placement: u64,
    port: String,
}

/// The control net of one circuit: terminals joined by wires and closed contacts.
///
/// The graph is small and short-lived — one per checked circuit — so it is built on demand
/// rather than kept as a cache. A contact that starts open is a gap: a button held down at
/// the moment of the check does not count as a short until the run says so.
#[derive(Debug, Default)]
struct ControlNet {
    terminals: Vec<Terminal>,
    edges: Vec<Vec<usize>>,
}

impl ControlNet {
    /// Builds the net from the document: every wire, and every normally-closed contact.
    fn of(document: &Document, catalog: &Catalog, circuit: u32) -> Self {
        let mut net = Self::default();
        for connection in document.connections() {
            if connection.circuit != circuit {
                continue;
            }
            let a = net.node(connection.from.placement, &connection.from.port);
            let b = net.node(connection.to.placement, &connection.to.port);
            net.join(a, b);
        }
        for placement in document.placements() {
            if placement.circuit != circuit {
                continue;
            }
            let Some(definition) = catalog.get(&placement.catalog_id) else {
                continue;
            };
            let Behaviour::Contact { normally_closed } = definition.behaviour else {
                continue;
            };
            if !normally_closed {
                continue;
            }
            let mut terminals = definition
                .port
                .iter()
                .filter(|port| port.role == PortRole::Electrical)
                .map(|port| port.id.as_str());
            if let (Some(first), Some(second)) = (terminals.next(), terminals.next()) {
                let a = net.node(placement.id(), first);
                let b = net.node(placement.id(), second);
                net.join(a, b);
            }
        }
        net
    }

    /// The node of a terminal, adding it the first time it is seen.
    fn node(&mut self, placement: u64, port: &str) -> usize {
        if let Some(index) = self.index_of(placement, port) {
            return index;
        }
        self.terminals.push(Terminal {
            placement,
            port: port.to_owned(),
        });
        self.edges.push(Vec::new());
        self.terminals.len() - 1
    }

    /// Adds an edge in both directions: a wire conducts either way.
    fn join(&mut self, a: usize, b: usize) {
        self.edges[a].push(b);
        self.edges[b].push(a);
    }

    /// The node of a terminal that the net already knows.
    fn index_of(&self, placement: u64, port: &str) -> Option<usize> {
        self.terminals
            .iter()
            .position(|terminal| terminal.placement == placement && terminal.port == port)
    }

    /// `true` when `start` and `goal` are joined by wires and closed contacts.
    fn reaches(&self, start: usize, goal: usize) -> bool {
        let mut seen = vec![false; self.terminals.len()];
        let mut pending = vec![start];
        seen[start] = true;
        while let Some(node) = pending.pop() {
            if node == goal {
                return true;
            }
            for next in &self.edges[node] {
                if !seen[*next] {
                    seen[*next] = true;
                    pending.push(*next);
                }
            }
        }
        false
    }
}
