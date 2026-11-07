// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The edited document: where each component sits, and how to take that back.
//!
//! A document is a list of placements — a catalog id and a box in millimetres — plus an
//! undo history of bounded length. It knows nothing about physics, drawing or the
//! interface: the solver reads the placements it cares about and the drawing paints them,
//! but neither owns them. That separation is what makes undo a list of commands instead of
//! a stack of snapshots copied out of the window.
//!
//! The history is bounded (100 entries) because an unbounded one is a leak with a nice
//! name. When it overflows the oldest entry is dropped, so the oldest states cannot be
//! reached again — the behaviour every editor has.

use punchi_fluid_draw::Orientation;
use punchi_fluid_draw::surface::{Pt, Rect};

/// Most entries the undo history keeps.
pub const HISTORY_LIMIT: usize = 100;

/// What one instance changes about its definition.
///
/// The reference edits the parameters of the component in the schematic, not of the
/// library: a larger cylinder in one circuit leaves the catalog alone. `None` means "as
/// the file says", so an untouched placement is indistinguishable from the definition.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PlacementEdits {
    /// Supply pressure, gauge bar.
    pub pressure_bar: Option<f64>,
    /// Conductance of every internal path of a valve or throttle.
    pub conductance: Option<f64>,
    /// Cylinder stroke, millimetres.
    pub stroke_mm: Option<f64>,
    /// Cylinder bore, millimetres.
    pub bore_mm: Option<f64>,
    /// Cylinder rod diameter, millimetres.
    pub rod_mm: Option<f64>,
    /// Cylinder moving mass, kilograms.
    pub moving_mass_kg: Option<f64>,
}

impl PlacementEdits {
    /// `true` when nothing is overridden.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }
}

/// One component on the sheet.
#[derive(Debug, Clone, PartialEq)]
pub struct Placement {
    id: u64,
    /// Catalog id of the definition it draws.
    pub catalog_id: String,
    /// Centre, in millimetres of the sheet.
    pub centre: Pt,
    /// Symbol box, in millimetres.
    pub width: f32,
    /// Symbol box, in millimetres.
    pub height: f32,
    /// Per-instance values that override the definition file.
    pub edits: PlacementEdits,
    /// How the symbol is turned on the sheet.
    pub orientation: Orientation,
}

impl Placement {
    /// A placement without an identity yet; the document assigns one when it is added.
    #[must_use]
    pub fn new(catalog_id: impl Into<String>, centre: Pt, width: f32, height: f32) -> Self {
        Self {
            id: 0,
            catalog_id: catalog_id.into(),
            centre,
            width,
            height,
            edits: PlacementEdits::default(),
            orientation: Orientation::NONE,
        }
    }

    /// A placement with an identity already, for a document read from a file.
    ///
    /// The identity has to survive the round trip: pipes name their ends by it, so a
    /// loader that renumbered the components would reconnect the wrong ports.
    #[must_use]
    pub fn with_id(
        id: u64,
        catalog_id: impl Into<String>,
        centre: Pt,
        width: f32,
        height: f32,
    ) -> Self {
        Self {
            id,
            catalog_id: catalog_id.into(),
            centre,
            width,
            height,
            edits: PlacementEdits::default(),
            orientation: Orientation::NONE,
        }
    }

    /// The identity connections use to name this component.
    ///
    /// It is assigned once and never reused, so removing a component cannot renumber
    /// anything else — which is what makes undo of a removal trivial.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// The box this placement occupies.
    #[must_use]
    pub fn rect(&self) -> Rect {
        Rect::from_center_size(self.centre, self.width, self.height)
    }

    /// The box this placement occupies once it is turned.
    ///
    /// A symbol standing on its side swaps its width and height, and that is the box a
    /// click has to fall in and the ring has to surround.
    #[must_use]
    pub fn box_rect(&self) -> Rect {
        self.orientation.box_of(self.rect())
    }

    /// `true` when the point, in millimetres, falls inside the box grown by `slack`.
    #[must_use]
    pub fn contains(&self, point: Pt, slack: f32) -> bool {
        self.box_rect().expand(slack).contains(point)
    }
}

/// One end of a connection: a component and one of its ports.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Endpoint {
    /// Id of the placement.
    pub placement: u64,
    /// Port identifier as the definition names it.
    pub port: String,
}

impl Endpoint {
    /// An endpoint of `placement` at `port`.
    #[must_use]
    pub fn new(placement: u64, port: impl Into<String>) -> Self {
        Self {
            placement,
            port: port.into(),
        }
    }
}

/// A pipe between two ports.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Connection {
    /// One end; the earlier of the two under the canonical order.
    pub from: Endpoint,
    /// The other end.
    pub to: Endpoint,
    /// How far the middle segment was dragged from the automatic route, in tenths of a
    /// millimetre.
    ///
    /// Tenths keep the offset exact in a type the history can compare, so a drag records
    /// one entry and an undo puts the pipe back where it was instead of re-rounding it.
    offset: i32,
}

impl Connection {
    /// `true` when this connection has `placement` at either end.
    #[must_use]
    pub fn touches(&self, placement: u64) -> bool {
        self.from.placement == placement || self.to.placement == placement
    }

    /// The middle segment's displacement from the automatic route, in tenths of a
    /// millimetre, exactly as the history stores it.
    #[must_use]
    pub const fn offset_tenths(&self) -> i32 {
        self.offset
    }

    /// The middle segment's displacement from the automatic route, in millimetres.
    #[must_use]
    pub fn offset_mm(&self) -> f32 {
        // The stored range is clamped to a thousand millimetres, far inside `f32`.
        #[allow(clippy::cast_precision_loss)]
        let offset = self.offset as f32;
        offset / 10.0
    }
}

/// A change that can be undone.
#[derive(Debug, Clone, PartialEq)]
enum Change {
    Added {
        index: usize,
        placement: Placement,
    },
    Removed {
        index: usize,
        placement: Placement,
        /// Pipes the component carried, with the position they had, so undo puts them
        /// back where they were.
        orphans: Vec<(usize, Connection)>,
    },
    Moved {
        index: usize,
        from: Pt,
        to: Pt,
    },
    Connected {
        index: usize,
        connection: Connection,
    },
    Disconnected {
        index: usize,
        connection: Connection,
    },
    /// The middle segment of a pipe was dragged to another position.
    Routed {
        index: usize,
        from: i32,
        to: i32,
    },
    /// One instance's parameters were edited.
    Edited {
        index: usize,
        from: PlacementEdits,
        to: PlacementEdits,
    },
    /// A placement was turned or reflected.
    Turned {
        index: usize,
        from: Orientation,
        to: Orientation,
    },
    /// Several changes that are one edit to the user.
    ///
    /// Splicing a junction into a pipe removes one, places a component and draws three, and
    /// a user who has to press undo five times for one gesture has been told the program is
    /// fine with losing their work. The batch undoes as one.
    Batch(Vec<Change>),
}

/// The document being edited.
#[derive(Debug, Default, Clone)]
pub struct Document {
    placements: Vec<Placement>,
    connections: Vec<Connection>,
    next_id: u64,
    undo: Vec<Change>,
    redo: Vec<Change>,
}

impl Document {
    /// An empty document.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Every placement, in the order they were placed.
    #[must_use]
    pub fn placements(&self) -> &[Placement] {
        &self.placements
    }

    /// Every pipe, in the order they were drawn.
    #[must_use]
    pub fn connections(&self) -> &[Connection] {
        &self.connections
    }

    /// How many components the sheet holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.placements.len()
    }

    /// `true` when nothing has been placed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.placements.is_empty()
    }

    /// The index of the placement with this id.
    #[must_use]
    pub fn index_of(&self, id: u64) -> Option<usize> {
        self.placements
            .iter()
            .position(|placement| placement.id == id)
    }

    /// The index of the placement under `point`, if any. The one on top wins.
    #[must_use]
    pub fn hit(&self, point: Pt, slack: f32) -> Option<usize> {
        self.placements
            .iter()
            .rposition(|placement| placement.contains(point, slack))
    }

    /// Replaces every placement as the starting state, with no history.
    ///
    /// Seeding is not an edit the user made, so it must not be undoable — and clearing the
    /// history by undoing it, which is the obvious wrong way, also removes what was just
    /// seeded.
    pub fn seed(&mut self, placements: Vec<Placement>) {
        self.placements.clear();
        self.connections.clear();
        self.undo.clear();
        self.redo.clear();
        for mut placement in placements {
            if placement.id == 0 {
                placement.id = self.take_id();
            }
            self.placements.push(placement);
        }
    }

    /// Adds a component and returns its index.
    pub fn place(&mut self, placement: Placement) -> usize {
        let mut placement = placement;
        if placement.id == 0 {
            placement.id = self.take_id();
        }
        self.next_id = self.next_id.max(placement.id + 1);
        let index = self.placements.len();
        self.placements.push(placement.clone());
        self.record(Change::Added { index, placement });
        index
    }

    /// Draws a pipe between two ports.
    ///
    /// Refuses the three things that are not pipes: an end joined to itself, a duplicate
    /// in either direction, and a component that is not on the sheet. A refusal is
    /// `false`, never a silent success.
    pub fn connect(&mut self, from: Endpoint, to: Endpoint) -> bool {
        let Some(index) = self.add_connection(from, to) else {
            return false;
        };
        let connection = self.connections[index].clone();
        self.record(Change::Connected { index, connection });
        true
    }

    /// Adds a pipe that belongs to the starting state, with no history entry.
    ///
    /// The seed is not something the user did, so undo must not take it apart. A pipe
    /// added here also survives the first undo of an edit made on top of it.
    pub fn seed_connect(&mut self, from: Endpoint, to: Endpoint) -> bool {
        self.add_connection(from, to).is_some()
    }

    /// Gives a seeded pipe its route without recording an edit.
    ///
    /// A route read from a file belongs to the starting state: opening a document must not
    /// fill the history with the routes it already had.
    pub fn seed_route(&mut self, index: usize, offset_tenths: i32) -> bool {
        let Some(connection) = self.connections.get_mut(index) else {
            return false;
        };
        connection.offset = offset_tenths.clamp(-10_000, 10_000);
        true
    }

    fn add_connection(&mut self, from: Endpoint, to: Endpoint) -> Option<usize> {
        if from == to
            || self.index_of(from.placement).is_none()
            || self.index_of(to.placement).is_none()
        {
            return None;
        }
        // Canonical order, so the same pipe drawn from either end is the same pipe.
        let (from, to) = if (&from.placement, &from.port) < (&to.placement, &to.port) {
            (from, to)
        } else {
            (to, from)
        };
        if self
            .connections
            .iter()
            .any(|existing| existing.from == from && existing.to == to)
        {
            return None;
        }
        let connection = Connection {
            from,
            to,
            offset: 0,
        };
        let index = self.connections.len();
        self.connections.push(connection);
        Some(index)
    }

    /// Removes a pipe.
    pub fn disconnect(&mut self, index: usize) -> bool {
        if index >= self.connections.len() {
            return false;
        }
        let connection = self.connections.remove(index);
        self.record(Change::Disconnected { index, connection });
        true
    }

    /// Splits a pipe with a junction and joins a third pipe to it.
    ///
    /// This is the gesture the reference documents: a hose end dropped on an existing hose
    /// leaves a T where it landed, not a crossing. The junction is a component with three
    /// ports on one point (ADR 0006), so the document stays port-to-port and this operation
    /// is written in terms of the operations it already has.
    ///
    /// The whole splice is one entry in the history. It returns `false` and changes nothing
    /// when any of the joins is refused, so a half-spliced pipe can never be left behind.
    pub fn splice(&mut self, pipe: usize, junction: Placement, attach: Endpoint) -> bool {
        let Some(connection) = self.connections.get(pipe).cloned() else {
            return false;
        };
        let mark = self.undo.len();
        if !self.disconnect(pipe) {
            return false;
        }
        let index = self.place(junction);
        let junction_id = self.placements[index].id();
        let port = |id: &str| Endpoint::new(junction_id, id);
        let spliced = self.connect(connection.from.clone(), port("1"))
            && self.connect(port("2"), connection.to.clone())
            && self.connect(port("3"), attach);
        if !spliced {
            let taken: Vec<Change> = self.undo.split_off(mark);
            for one in taken.iter().rev() {
                self.apply_inverse(one);
            }
            self.redo.clear();
            self.next_id = self.next_id.max(junction_id + 1);
            return false;
        }
        let taken: Vec<Change> = self.undo.split_off(mark);
        self.undo.push(Change::Batch(taken));
        if self.undo.len() > HISTORY_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
        true
    }

    /// Moves a placement. A move that changes nothing is not recorded.
    pub fn move_to(&mut self, index: usize, centre: Pt) -> bool {
        let Some(placement) = self.placements.get_mut(index) else {
            return false;
        };
        if placement.centre == centre {
            return false;
        }
        let from = placement.centre;
        placement.centre = centre;
        self.record(Change::Moved {
            index,
            from,
            to: centre,
        });
        true
    }

    /// Displaces the middle segment of a pipe by `offset_mm`.
    ///
    /// The reference lets the reader drag a hose segment until the circuit reads clearly;
    /// this is that gesture as one number, because the route is an elbow and the segment
    /// that moves is always the middle one. A value that does not change the stored
    /// tenths is not recorded, so a click that selects a pipe adds nothing to the
    /// history.
    pub fn route_to(&mut self, index: usize, offset_mm: f32) -> bool {
        #[allow(clippy::cast_possible_truncation)]
        let tenths = (offset_mm * 10.0).round().clamp(-10_000.0, 10_000.0) as i32;
        let Some(connection) = self.connections.get_mut(index) else {
            return false;
        };
        if connection.offset == tenths {
            return false;
        }
        let from = connection.offset;
        connection.offset = tenths;
        self.record(Change::Routed {
            index,
            from,
            to: tenths,
        });
        true
    }

    /// Copies a placement, optionally with another centre, and returns the copy's index.
    ///
    /// The copy keeps the catalog id, the box and the per-instance values and gets a new
    /// identity, so the pipes of the original stay where they are. One undo removes the
    /// copy and nothing else.
    pub fn duplicate(&mut self, index: usize, centre: Option<Pt>) -> Option<usize> {
        let copy = self.placements.get(index)?.clone();
        Some(self.place_copy(copy, centre))
    }

    /// Adds a copy of a placement, clearing the identity it came with.
    ///
    /// A pasted component is the same component with a new identity, so its pipes —which
    /// name the old identity— do not follow it.
    pub fn place_copy(&mut self, mut placement: Placement, centre: Option<Pt>) -> usize {
        placement.id = 0;
        if let Some(centre) = centre {
            placement.centre = centre;
        }
        self.place(placement)
    }

    /// Sets the per-instance values of a placement, as one undoable edit.
    ///
    /// An edit that changes nothing is not recorded; a form that is opened and applied
    /// without touching a field must not put an entry in the history.
    pub fn edit(&mut self, index: usize, edits: PlacementEdits) -> bool {
        let Some(placement) = self.placements.get_mut(index) else {
            return false;
        };
        if placement.edits == edits {
            return false;
        }
        let from = std::mem::replace(&mut placement.edits, edits.clone());
        self.record(Change::Edited {
            index,
            from,
            to: edits,
        });
        true
    }

    /// Turns a placement a quarter clockwise: `Edición ▸ Girar`.
    pub fn turn_clockwise(&mut self, index: usize) -> bool {
        self.turn(index, Orientation::turned_clockwise)
    }

    /// Turns a placement a quarter anticlockwise: `Ctrl` and a double click.
    pub fn turn_anticlockwise(&mut self, index: usize) -> bool {
        self.turn(index, Orientation::turned_anticlockwise)
    }

    /// Reflects a placement about its own vertical axis: `Edición ▸ Reflejar`.
    pub fn reflect(&mut self, index: usize) -> bool {
        self.turn(index, Orientation::reflected)
    }

    /// How a placement is turned, or `None` when there is no such placement.
    #[must_use]
    pub fn orientation_of(&self, index: usize) -> Option<Orientation> {
        self.placements
            .get(index)
            .map(|placement| placement.orientation)
    }

    fn turn(&mut self, index: usize, turn: fn(Orientation) -> Orientation) -> bool {
        let Some(placement) = self.placements.get_mut(index) else {
            return false;
        };
        let from = placement.orientation;
        let to = turn(from);
        placement.orientation = to;
        self.record(Change::Turned { index, from, to });
        true
    }

    /// Removes a placement, and with it every pipe it carried.
    pub fn remove(&mut self, index: usize) -> Option<Placement> {
        if index >= self.placements.len() {
            return None;
        }
        let placement = self.placements.remove(index);
        let mut orphans = Vec::new();
        let mut kept = Vec::with_capacity(self.connections.len());
        for (position, connection) in std::mem::take(&mut self.connections)
            .into_iter()
            .enumerate()
        {
            if connection.touches(placement.id) {
                orphans.push((position, connection));
            } else {
                kept.push(connection);
            }
        }
        self.connections = kept;
        self.record(Change::Removed {
            index,
            placement: placement.clone(),
            orphans,
        });
        Some(placement)
    }

    /// `true` when there is a change to take back.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// `true` when an undone change can be applied again.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Takes back the last change.
    pub fn undo(&mut self) -> bool {
        let Some(change) = self.undo.pop() else {
            return false;
        };
        self.apply_inverse(&change);
        self.redo.push(change);
        true
    }

    /// Applies the last undone change again.
    pub fn redo(&mut self) -> bool {
        let Some(change) = self.redo.pop() else {
            return false;
        };
        self.apply(&change);
        self.undo.push(change);
        true
    }

    fn take_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    fn record(&mut self, change: Change) {
        // A drag is one edit, not one per mouse move: consecutive moves of the same
        // component extend the entry already at the top of the history.
        if let (
            Some(Change::Moved {
                index: last_index,
                to: last_to,
                ..
            }),
            Change::Moved { index, to, .. },
        ) = (self.undo.last_mut(), &change)
        {
            if last_index == index {
                *last_to = *to;
                self.redo.clear();
                return;
            }
        }
        // Same rule for a routed pipe: one drag, one entry.
        if let (
            Some(Change::Routed {
                index: last_index,
                to: last_to,
                ..
            }),
            Change::Routed { index, to, .. },
        ) = (self.undo.last_mut(), &change)
        {
            if last_index == index {
                *last_to = *to;
                self.redo.clear();
                return;
            }
        }
        self.undo.push(change);
        if self.undo.len() > HISTORY_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    fn apply(&mut self, change: &Change) {
        match change {
            Change::Batch(changes) => {
                for one in changes {
                    self.apply(one);
                }
            }
            Change::Added { index, placement } => {
                insert_at(&mut self.placements, *index, placement.clone());
            }
            Change::Removed {
                index,
                orphans,
                placement,
            } => {
                if *index < self.placements.len() {
                    self.placements.remove(*index);
                }
                self.connections
                    .retain(|connection| !orphans.iter().any(|(_, orphan)| orphan == connection));
                let _ = placement;
            }
            Change::Moved { index, to, .. } => {
                if let Some(placement) = self.placements.get_mut(*index) {
                    placement.centre = *to;
                }
            }
            Change::Connected { index, connection } => {
                insert_at(&mut self.connections, *index, connection.clone());
            }
            Change::Disconnected { index, .. } => {
                if *index < self.connections.len() {
                    self.connections.remove(*index);
                }
            }
            Change::Routed { index, to, .. } => {
                if let Some(connection) = self.connections.get_mut(*index) {
                    connection.offset = *to;
                }
            }
            Change::Edited { index, to, .. } => {
                if let Some(placement) = self.placements.get_mut(*index) {
                    placement.edits = to.clone();
                }
            }
            Change::Turned { index, to, .. } => {
                if let Some(placement) = self.placements.get_mut(*index) {
                    placement.orientation = *to;
                }
            }
        }
    }

    fn apply_inverse(&mut self, change: &Change) {
        match change {
            Change::Batch(changes) => {
                for one in changes.iter().rev() {
                    self.apply_inverse(one);
                }
            }
            Change::Added { index, .. } => {
                if *index < self.placements.len() {
                    self.placements.remove(*index);
                }
            }
            Change::Removed {
                index,
                placement,
                orphans,
            } => {
                insert_at(&mut self.placements, *index, placement.clone());
                for (position, connection) in orphans {
                    insert_at(&mut self.connections, *position, connection.clone());
                }
            }
            Change::Moved { index, from, .. } => {
                if let Some(placement) = self.placements.get_mut(*index) {
                    placement.centre = *from;
                }
            }
            Change::Connected { index, .. } => {
                if *index < self.connections.len() {
                    self.connections.remove(*index);
                }
            }
            Change::Disconnected { index, connection } => {
                insert_at(&mut self.connections, *index, connection.clone());
            }
            Change::Routed { index, from, .. } => {
                if let Some(connection) = self.connections.get_mut(*index) {
                    connection.offset = *from;
                }
            }
            Change::Edited { index, from, .. } => {
                if let Some(placement) = self.placements.get_mut(*index) {
                    placement.edits = from.clone();
                }
            }
            Change::Turned { index, from, .. } => {
                if let Some(placement) = self.placements.get_mut(*index) {
                    placement.orientation = *from;
                }
            }
        }
    }
}

fn insert_at<T>(items: &mut Vec<T>, index: usize, item: T) {
    let index = index.min(items.len());
    items.insert(index, item);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-6
    }

    fn placement(id: &str, x: f32, y: f32) -> Placement {
        Placement::new(id, Pt::new(x, y), 20.0, 20.0)
    }

    #[test]
    fn a_junction_splits_a_pipe_and_the_whole_splice_undoes_as_one() {
        let mut document = Document::new();
        let a = document.place(placement("a", 10.0, 10.0));
        let b = document.place(placement("b", 90.0, 10.0));
        let c = document.place(placement("c", 10.0, 90.0));
        let (a, b, c) = (
            document.placements()[a].id(),
            document.placements()[b].id(),
            document.placements()[c].id(),
        );
        assert!(document.connect(Endpoint::new(a, "1"), Endpoint::new(b, "1")));
        let pipes = document.connections().len();

        // One gesture: the pipe a-b becomes a-to-junction and junction-to-b, and c arrives.
        let spliced = document.splice(
            0,
            placement("junction.tee", 50.0, 10.0),
            Endpoint::new(c, "1"),
        );
        assert!(spliced);
        assert_eq!(document.connections().len(), pipes + 2);
        assert_eq!(document.len(), 4);
        // Every pipe from the splice reaches the junction, and the old pipe is gone.
        assert!(
            !document
                .connections()
                .iter()
                .any(|pipe| pipe.from == Endpoint::new(a, "1") && pipe.to == Endpoint::new(b, "1"))
        );

        // One undo puts all of it back: a user who has to press five times for one gesture
        // has been told the program does not mind losing their work.
        assert!(document.undo());
        assert_eq!(document.connections().len(), pipes);
        assert_eq!(document.len(), 3);
        assert!(
            document
                .connections()
                .iter()
                .any(|pipe| pipe.to == Endpoint::new(b, "1"))
        );
        // And one redo brings it back.
        assert!(document.redo());
        assert_eq!(document.connections().len(), pipes + 2);
    }

    #[test]
    fn a_splice_that_cannot_finish_leaves_the_pipe_untouched() {
        let mut document = Document::new();
        let a = document.place(placement("a", 10.0, 10.0));
        let b = document.place(placement("b", 90.0, 10.0));
        let (a, b) = (document.placements()[a].id(), document.placements()[b].id());
        assert!(document.connect(Endpoint::new(a, "1"), Endpoint::new(b, "1")));

        // An attachment on a component that is not on the sheet is refused, and the splice
        // must not leave a junction placed and a pipe cut in half behind it.
        let refused = document.splice(
            0,
            placement("junction.tee", 50.0, 10.0),
            Endpoint::new(9999, "1"),
        );
        assert!(!refused);
        assert_eq!(document.len(), 2, "the junction was not left behind");
        assert_eq!(document.connections().len(), 1, "the pipe is whole");
        assert_eq!(document.connections()[0].from, Endpoint::new(a, "1"));
        assert_eq!(document.connections()[0].to, Endpoint::new(b, "1"));
    }

    #[test]
    fn placing_grows_the_document_and_can_be_undone() {
        let mut document = Document::new();
        document.place(placement("a", 10.0, 10.0));
        document.place(placement("b", 40.0, 40.0));
        assert_eq!(document.len(), 2);
        assert!(document.undo());
        assert_eq!(document.len(), 1);
        assert!(document.undo());
        assert!(document.is_empty());
        assert!(!document.undo());
    }

    #[test]
    fn every_placement_gets_an_identity_of_its_own() {
        let mut document = Document::new();
        document.place(placement("a", 0.0, 0.0));
        document.place(placement("a", 10.0, 0.0));
        let ids: Vec<u64> = document.placements().iter().map(Placement::id).collect();
        assert_ne!(ids[0], ids[1], "two components of one kind are still two");
        assert_eq!(document.index_of(ids[1]), Some(1));
    }

    #[test]
    fn seeding_is_not_an_edit() {
        let mut document = Document::new();
        document.seed(vec![placement("a", 1.0, 1.0), placement("b", 2.0, 2.0)]);
        assert_eq!(document.len(), 2);
        assert!(!document.can_undo(), "the starting state is not undoable");
        document.place(placement("c", 3.0, 3.0));
        assert!(document.undo());
        assert_eq!(
            document.len(),
            2,
            "undo takes back the placement, not the seed"
        );
    }

    #[test]
    fn a_connection_joins_two_ports_and_can_be_undone() {
        let mut document = Document::new();
        document.place(placement("supply", 0.0, 0.0));
        document.place(placement("valve", 40.0, 0.0));
        let (a, b) = (document.placements()[0].id(), document.placements()[1].id);

        assert!(document.connect(Endpoint::new(a, "1"), Endpoint::new(b, "1")));
        assert_eq!(document.connections().len(), 1);
        assert!(document.undo());
        assert!(document.connections().is_empty());
        assert!(document.redo());
        assert_eq!(document.connections().len(), 1);
    }

    #[test]
    fn the_same_pipe_drawn_from_either_end_is_one_pipe() {
        let mut document = Document::new();
        document.place(placement("a", 0.0, 0.0));
        document.place(placement("b", 40.0, 0.0));
        let (a, b) = (document.placements()[0].id(), document.placements()[1].id());
        assert!(document.connect(Endpoint::new(a, "1"), Endpoint::new(b, "2")));
        assert!(
            !document.connect(Endpoint::new(b, "2"), Endpoint::new(a, "1")),
            "the reverse of an existing pipe is the same pipe"
        );
        assert_eq!(document.connections().len(), 1);
    }

    #[test]
    fn a_port_cannot_be_joined_to_itself_or_to_a_missing_component() {
        let mut document = Document::new();
        document.place(placement("a", 0.0, 0.0));
        let a = document.placements()[0].id();
        assert!(!document.connect(Endpoint::new(a, "1"), Endpoint::new(a, "1")));
        assert!(!document.connect(Endpoint::new(a, "1"), Endpoint::new(9999, "1")));
        assert!(document.connections().is_empty());
    }

    #[test]
    fn removing_a_component_takes_its_pipes_and_undo_brings_them_back() {
        let mut document = Document::new();
        document.place(placement("a", 0.0, 0.0));
        document.place(placement("b", 40.0, 0.0));
        document.place(placement("c", 80.0, 0.0));
        let ids: Vec<u64> = document.placements().iter().map(Placement::id).collect();
        document.connect(Endpoint::new(ids[0], "1"), Endpoint::new(ids[1], "1"));
        document.connect(Endpoint::new(ids[1], "2"), Endpoint::new(ids[2], "1"));
        assert_eq!(document.connections().len(), 2);

        document.remove(1);
        assert_eq!(document.len(), 2);
        assert!(
            document.connections().is_empty(),
            "a pipe to a component that is gone is not a pipe"
        );

        assert!(document.undo());
        assert_eq!(document.len(), 3);
        assert_eq!(document.connections().len(), 2);
        let ids_again: Vec<u64> = document.placements().iter().map(Placement::id).collect();
        assert_eq!(ids, ids_again, "the identities survive the round trip");
    }

    #[test]
    fn redo_puts_the_change_back_exactly_once() {
        let mut document = Document::new();
        document.place(placement("a", 10.0, 10.0));
        assert!(document.undo());
        assert!(document.redo());
        assert_eq!(document.len(), 1);
        assert!(!document.redo());
    }

    #[test]
    fn a_new_change_drops_the_redo_line() {
        let mut document = Document::new();
        document.place(placement("a", 10.0, 10.0));
        assert!(document.undo());
        assert!(document.can_redo());
        document.place(placement("b", 5.0, 5.0));
        assert!(!document.can_redo());
    }

    #[test]
    fn undoing_a_move_restores_the_previous_centre() {
        let mut document = Document::new();
        let index = document.place(placement("a", 10.0, 10.0));
        assert!(document.move_to(index, Pt::new(30.0, 30.0)));
        assert!(document.undo());
        assert_eq!(document.placements()[0].centre, Pt::new(10.0, 10.0));
    }

    #[test]
    fn a_duplicate_keeps_the_values_gets_a_new_identity_and_one_undo_removes_it() {
        let mut document = Document::new();
        let source = document.place(placement("valve.5_2.spring_return", 10.0, 10.0));
        let edits = PlacementEdits {
            conductance: Some(2.0e-9),
            ..PlacementEdits::default()
        };
        assert!(document.edit(source, edits.clone()));
        let original_id = document.placements()[source].id();
        let copy = document
            .duplicate(source, Some(Pt::new(30.0, 40.0)))
            .expect("the source exists");
        assert_eq!(document.len(), 2);
        assert_ne!(document.placements()[copy].id(), original_id);
        assert_eq!(
            document.placements()[copy].catalog_id,
            "valve.5_2.spring_return"
        );
        assert_eq!(document.placements()[copy].centre, Pt::new(30.0, 40.0));
        assert_eq!(document.placements()[copy].edits, edits);
        assert!(document.undo());
        assert_eq!(document.len(), 1);
        assert_eq!(document.placements()[0].id(), original_id);
        assert!(document.duplicate(9, None).is_none());
    }

    #[test]
    fn editing_a_placement_is_one_undoable_change() {
        let mut document = Document::new();
        let index = document.place(placement("cylinder.single_acting_20_100", 10.0, 10.0));
        assert!(PlacementEdits::default().is_empty());
        let edits = PlacementEdits {
            stroke_mm: Some(50.0),
            bore_mm: Some(32.0),
            ..PlacementEdits::default()
        };
        assert!(document.edit(index, edits.clone()));
        assert_eq!(document.placements()[0].edits, edits);

        // Applying the same form twice is not an edit.
        assert!(!document.edit(index, edits.clone()));
        assert!(document.undo());
        assert!(document.placements()[0].edits.is_empty());
        assert!(document.redo());
        assert_eq!(document.placements()[0].edits.stroke_mm, Some(50.0));
    }

    #[test]
    fn a_seeded_pipe_is_part_of_the_starting_state_and_not_of_the_history() {
        let mut document = Document::new();
        document.seed(vec![placement("a", 10.0, 10.0), placement("b", 90.0, 10.0)]);
        let (a, b) = (document.placements()[0].id(), document.placements()[1].id());
        assert!(document.seed_connect(Endpoint::new(a, "1"), Endpoint::new(b, "1")));
        assert_eq!(document.connections().len(), 1);
        assert!(
            !document.can_undo(),
            "undo must not take the starting circuit apart"
        );
        // The first edit on top is the first thing undo takes back.
        assert!(document.route_to(0, 12.0));
        assert!(document.undo());
        assert_eq!(document.connections().len(), 1);
        assert!(close(document.connections()[0].offset_mm(), 0.0));
    }

    #[test]
    fn a_drag_is_one_undo_entry_not_one_per_move() {
        let mut document = Document::new();
        let index = document.place(placement("a", 10.0, 10.0));
        for step in 1..=20 {
            #[allow(clippy::cast_precision_loss)]
            let offset = step as f32;
            document.move_to(index, Pt::new(10.0 + offset, 10.0));
        }
        assert!(document.undo());
        assert_eq!(document.placements()[0].centre, Pt::new(10.0, 10.0));
    }

    #[test]
    fn dragging_a_pipe_segment_is_one_undo_entry_and_comes_back() {
        let mut document = Document::new();
        let a = document.place(placement("a", 10.0, 10.0));
        let b = document.place(placement("b", 90.0, 10.0));
        let (a, b) = (document.placements()[a].id(), document.placements()[b].id());
        assert!(document.connect(Endpoint::new(a, "1"), Endpoint::new(b, "1")));
        assert!(close(document.connections()[0].offset_mm(), 0.0));

        // The whole drag is one number, and the numbers are tenths of a millimetre.
        assert!(document.route_to(0, 12.0));
        assert!(document.route_to(0, 25.4));
        assert!(close(document.connections()[0].offset_mm(), 25.4));
        assert!(document.undo());
        assert!(close(document.connections()[0].offset_mm(), 0.0));
        assert!(document.redo());
        assert!(close(document.connections()[0].offset_mm(), 25.4));

        // A click that changes nothing is not an edit.
        assert!(!document.route_to(0, 25.4));
        assert!(!document.route_to(9, 1.0));
    }

    #[test]
    fn a_routed_pipe_survives_the_undo_of_an_unrelated_move() {
        let mut document = Document::new();
        let a = document.place(placement("a", 10.0, 10.0));
        let b = document.place(placement("b", 90.0, 10.0));
        let (a, b) = (document.placements()[a].id(), document.placements()[b].id());
        assert!(document.connect(Endpoint::new(a, "1"), Endpoint::new(b, "1")));
        assert!(document.route_to(0, 30.0));
        assert!(document.move_to(1, Pt::new(90.0, 60.0)));
        assert!(document.undo());
        assert!(close(document.connections()[0].offset_mm(), 30.0));
    }

    #[test]
    fn the_history_never_grows_past_its_limit() {
        let mut document = Document::new();
        for step in 0..(HISTORY_LIMIT + 40) {
            #[allow(clippy::cast_precision_loss)]
            let x = step as f32;
            document.place(placement("a", x, 0.0));
        }
        let mut taken_back = 0;
        while document.undo() {
            taken_back += 1;
        }
        assert_eq!(taken_back, HISTORY_LIMIT);
    }

    #[test]
    fn a_turn_is_one_undo_entry_and_four_of_them_are_the_symbol_as_it_was() {
        let mut document = Document::new();
        let index = document.place(Placement::new(
            "valve.3_2.spring_return",
            Pt::new(50.0, 50.0),
            30.0,
            15.0,
        ));
        assert!(document.turn_clockwise(index));
        assert_eq!(
            document.orientation_of(index),
            Some(Orientation::new(1, false))
        );
        assert!(document.undo());
        assert_eq!(document.orientation_of(index), Some(Orientation::NONE));
        assert!(document.redo());
        for _ in 0..3 {
            assert!(document.turn_clockwise(index));
        }
        assert_eq!(document.orientation_of(index), Some(Orientation::NONE));
    }

    #[test]
    fn a_turned_symbol_occupies_the_box_it_is_drawn_in() {
        let mut document = Document::new();
        let index = document.place(Placement::new(
            "valve.3_2.spring_return",
            Pt::new(50.0, 50.0),
            30.0,
            15.0,
        ));
        // Side on, the box is 15 wide and 30 tall, so a point above the centre is inside
        // it and the same point to the right is not.
        assert!(document.turn_clockwise(index));
        assert!(document.placements()[index].contains(Pt::new(50.0, 62.0), 0.0));
        assert!(!document.placements()[index].contains(Pt::new(62.0, 50.0), 0.0));
    }

    #[test]
    fn a_mirror_twice_leaves_the_symbol_alone_and_the_copy_keeps_it() {
        let mut document = Document::new();
        let index = document.place(placement("valve.3_2.spring_return", 10.0, 10.0));
        assert!(document.reflect(index));
        assert_eq!(
            document.orientation_of(index),
            Some(Orientation::new(0, true))
        );
        assert!(document.reflect(index));
        assert_eq!(document.orientation_of(index), Some(Orientation::NONE));
        assert!(document.reflect(index));
        let copy = document
            .duplicate(index, None)
            .map(|copy| document.placements()[copy].orientation);
        assert_eq!(copy, Some(Orientation::new(0, true)));
    }

    #[test]
    fn the_hit_test_prefers_the_component_on_top() {
        let mut document = Document::new();
        document.place(placement("below", 10.0, 10.0));
        document.place(placement("above", 10.0, 10.0));
        let hit = document.hit(Pt::new(10.0, 10.0), 0.0);
        assert_eq!(
            hit.map(|index| document.placements()[index].catalog_id.as_str()),
            Some("above")
        );
        assert_eq!(document.hit(Pt::new(200.0, 200.0), 0.0), None);
    }
}
