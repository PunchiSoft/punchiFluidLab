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

/// Longest designation a component may carry.
///
/// A designation is a short name a person reads on the sheet and in the parts list, not a
/// note: 40 characters fit beside a symbol and stop a pasted paragraph from covering the
/// drawing.
pub const MAX_DESIGNATION: usize = 40;

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

/// What one command does to the orientation of a placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Turn {
    /// A quarter turn clockwise: `Edición ▸ Girar`.
    Clockwise,
    /// A quarter turn anticlockwise: `Ctrl` and a double click.
    Anticlockwise,
    /// A reflection on the symbol's own vertical axis: `Edición ▸ Reflejar`.
    Reflect,
}

impl Turn {
    /// The orientation this command produces.
    #[must_use]
    pub const fn apply(self, orientation: Orientation) -> Orientation {
        match self {
            Self::Clockwise => orientation.turned_clockwise(),
            Self::Anticlockwise => orientation.turned_anticlockwise(),
            Self::Reflect => orientation.reflected(),
        }
    }
}

/// The edge a selection is lined up on: `Edición ▸ Alinear`.
///
/// The reference always lines the marked objects up on the one that already sits furthest
/// in that direction, so the command never needs an anchor the user has to choose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    /// Line the left edges up on the leftmost component.
    Left,
    /// Line the right edges up on the rightmost component.
    Right,
    /// Line the top edges up on the highest component.
    Top,
    /// Line the bottom edges up on the lowest component.
    Bottom,
}

/// Axis along which a selection is spread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// Left to right.
    Horizontal,
    /// Top to bottom.
    Vertical,
}

impl Axis {
    /// The near edge of a box on this axis.
    fn min(self, rect: Rect) -> f32 {
        match self {
            Self::Horizontal => rect.min.x,
            Self::Vertical => rect.min.y,
        }
    }

    /// The far edge of a box on this axis.
    fn max(self, rect: Rect) -> f32 {
        match self {
            Self::Horizontal => rect.max.x,
            Self::Vertical => rect.max.y,
        }
    }

    /// The box's size along this axis.
    fn size(self, rect: Rect) -> f32 {
        match self {
            Self::Horizontal => rect.width(),
            Self::Vertical => rect.height(),
        }
    }

    /// A centre moved by `delta` along this axis.
    fn moved(self, centre: Pt, delta: f32) -> Pt {
        match self {
            Self::Horizontal => Pt::new(centre.x + delta, centre.y),
            Self::Vertical => Pt::new(centre.x, centre.y + delta),
        }
    }
}

impl Align {
    /// The edge of a box this alignment lines the others up on.
    fn edge(self, rect: Rect) -> f32 {
        match self {
            Self::Left => rect.min.x,
            Self::Right => rect.max.x,
            Self::Top => rect.min.y,
            Self::Bottom => rect.max.y,
        }
    }

    /// `true` when the alignment moves the box sideways.
    const fn horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }

    /// `true` when `edge` reaches further out than the reference, so it becomes the new one.
    fn further_out(self, edge: f32, reference: f32) -> bool {
        match self {
            Self::Left | Self::Top => edge < reference,
            Self::Right | Self::Bottom => edge > reference,
        }
    }
}

/// One circuit of a project.
///
/// A circuit is a sheet: it owns a subset of the project's placements and pipes, and the
/// active circuit is the one the solver, the drawing and the export see.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Circuit {
    /// Stable identity a placement or a pipe names.
    pub id: u32,
    /// Name shown in the selector and in the printed title block.
    pub title: String,
}

impl Circuit {
    /// A circuit with an identity and a title.
    #[must_use]
    pub fn new(id: u32, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
        }
    }
}

/// A quantity a recorder can chart.
///
/// The three the model can already answer; a definition that cannot report one refuses the
/// source instead of charting a zero (ADR 0015).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Quantity {
    /// Pressure at the first port, gauge bar.
    Pressure,
    /// Flow through a flow meter, normal litres per minute.
    Flow,
    /// Extension of a cylinder, millimetres.
    Position,
}

impl Quantity {
    /// Stable name the interface and the file use.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pressure => "pressure",
            Self::Flow => "flow",
            Self::Position => "position",
        }
    }

    /// The quantity a name stands for, or `None` when the name is not one.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "pressure" => Some(Self::Pressure),
            "flow" => Some(Self::Flow),
            "position" => Some(Self::Position),
            _ => None,
        }
    }
}

/// What a recorder charts: another placement and one of its quantities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Source {
    /// Identity of the placement being observed.
    pub placement: u64,
    /// Which of its quantities is charted.
    pub quantity: Quantity,
}

impl Source {
    /// A source naming `placement` and `quantity`.
    #[must_use]
    pub const fn new(placement: u64, quantity: Quantity) -> Self {
        Self {
            placement,
            quantity,
        }
    }
}

/// One component on the sheet.
#[derive(Debug, Clone, PartialEq)]
pub struct Placement {
    id: u64,
    /// Circuit this placement belongs to.
    pub circuit: u32,
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
    /// Name the user gave this component, shown on the sheet and in the parts list.
    ///
    /// `None` is "no designation", which is different from an empty one: a valve may carry
    /// no name at all, and a circuit may have two unnamed cylinders without either of them
    /// being a duplicate.
    pub designation: Option<String>,
    /// What a recorder charts, when this placement is one.
    ///
    /// `None` is "not bound yet", which the inspector shows as an incomplete instrument;
    /// the drawing says so instead of charting a flat zero.
    pub source: Option<Source>,
}

impl Placement {
    /// A placement without an identity yet; the document assigns one when it is added.
    #[must_use]
    pub fn new(catalog_id: impl Into<String>, centre: Pt, width: f32, height: f32) -> Self {
        Self {
            id: 0,
            circuit: 0,
            catalog_id: catalog_id.into(),
            centre,
            width,
            height,
            edits: PlacementEdits::default(),
            orientation: Orientation::NONE,
            designation: None,
            source: None,
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
            circuit: 0,
            catalog_id: catalog_id.into(),
            centre,
            width,
            height,
            edits: PlacementEdits::default(),
            orientation: Orientation::NONE,
            designation: None,
            source: None,
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
    /// Circuit this pipe belongs to.
    pub circuit: u32,
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
    /// One instance was given a designation, or had it taken away.
    Designated {
        index: usize,
        from: Option<String>,
        to: Option<String>,
    },
    /// A recorder was bound to a source, or unbound.
    Sourced {
        index: usize,
        from: Option<Source>,
        to: Option<Source>,
    },
    /// A circuit was added to the project.
    CircuitAdded {
        index: usize,
        circuit: Circuit,
    },
    /// A circuit was renamed.
    CircuitRenamed {
        id: u32,
        from: String,
        to: String,
    },
    /// Several changes that are one edit to the user.
    ///
    /// Splicing a junction into a pipe removes one, places a component and draws three, and
    /// a user who has to press undo five times for one gesture has been told the program is
    /// fine with losing their work. The batch undoes as one.
    Batch(Vec<Change>),
}

/// The document being edited.
#[derive(Debug, Clone)]
pub struct Document {
    placements: Vec<Placement>,
    connections: Vec<Connection>,
    circuits: Vec<Circuit>,
    active: u32,
    next_id: u64,
    undo: Vec<Change>,
    redo: Vec<Change>,
    /// Counts every change of state, so a caller can tell whether the document differs
    /// from the copy it last saved without comparing every field.
    revision: u64,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            placements: Vec::new(),
            connections: Vec::new(),
            circuits: vec![Circuit::new(0, "Circuito 1")],
            active: 0,
            next_id: 0,
            undo: Vec::new(),
            redo: Vec::new(),
            revision: 0,
        }
    }
}

impl Document {
    /// An empty document with one circuit.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Every circuit, in the order the project lists them.
    #[must_use]
    pub fn circuits(&self) -> &[Circuit] {
        &self.circuits
    }

    /// The circuit the solver and the drawing use.
    #[must_use]
    pub const fn active_circuit(&self) -> u32 {
        self.active
    }

    /// Replaces the circuit list and the active circuit, for a document read from a file.
    ///
    /// This is the starting state, not an edit: it does not enter the history.
    pub fn set_project(&mut self, circuits: Vec<Circuit>, active: u32) -> bool {
        if circuits.is_empty() || !circuits.iter().any(|circuit| circuit.id == active) {
            return false;
        }
        self.circuits = circuits;
        self.active = active;
        true
    }

    /// Switches the active circuit. A view change, not an edit, so it is not undoable.
    pub fn select_circuit(&mut self, id: u32) -> bool {
        if !self.circuits.iter().any(|circuit| circuit.id == id) {
            return false;
        }
        self.active = id;
        true
    }

    /// Adds an empty circuit and makes it active, as one undoable change.
    pub fn add_circuit(&mut self, title: impl Into<String>) -> u32 {
        let id = self
            .circuits
            .iter()
            .map(|circuit| circuit.id)
            .max()
            .map_or(1, |id| id + 1);
        let circuit = Circuit::new(id, title);
        let index = self.circuits.len();
        self.circuits.push(circuit.clone());
        self.active = id;
        self.record(Change::CircuitAdded { index, circuit });
        id
    }

    /// Renames a circuit, as one undoable change.
    pub fn rename_circuit(&mut self, id: u32, title: impl Into<String>) -> bool {
        let title = title.into();
        let Some(circuit) = self.circuits.iter_mut().find(|circuit| circuit.id == id) else {
            return false;
        };
        let from = circuit.title.clone();
        if from == title {
            return true;
        }
        circuit.title.clone_from(&title);
        self.record(Change::CircuitRenamed {
            id,
            from,
            to: title,
        });
        true
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
        // The starting state is the baseline a save is measured against, not a change.
        self.revision = self.revision.wrapping_add(1);
        for mut placement in placements {
            if placement.id == 0 {
                placement.id = self.take_id();
            }
            self.next_id = self.next_id.max(placement.id + 1);
            self.placements.push(placement);
        }
    }

    /// Adds a component and returns its index.
    pub fn place(&mut self, placement: Placement) -> usize {
        let mut placement = placement;
        placement.circuit = self.active;
        if placement.id == 0 {
            placement.id = self.take_id();
        }
        self.next_id = self.next_id.max(placement.id + 1);
        let index = self.placements.len();
        self.placements.push(placement.clone());
        self.record(Change::Added { index, placement });
        index
    }

    /// Copies several placements displaced by a delta, as one entry in the history.
    ///
    /// The copies keep their catalog ids, boxes, values and orientations and get new
    /// identities, so the pipes of the originals stay where they are.
    pub fn duplicate_many(&mut self, indices: &[usize], offset: (f32, f32)) -> Vec<usize> {
        let copies: Vec<Placement> = indices
            .iter()
            .filter_map(|index| self.placements.get(*index).cloned())
            .collect();
        let mut added = Vec::with_capacity(copies.len());
        let mut changes = Vec::with_capacity(copies.len());
        for copy in copies {
            let mut copy = copy;
            copy.id = self.take_id();
            copy.circuit = self.active;
            copy.centre = Pt::new(copy.centre.x + offset.0, copy.centre.y + offset.1);
            let index = self.placements.len();
            self.placements.push(copy.clone());
            changes.push(Change::Added {
                index,
                placement: copy,
            });
            added.push(index);
        }
        self.record_group(changes);
        added
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

    /// Gives a seeded pipe its circuit without recording an edit.
    pub fn seed_pipe_circuit(&mut self, index: usize, circuit: u32) -> bool {
        let Some(connection) = self.connections.get_mut(index) else {
            return false;
        };
        connection.circuit = circuit;
        true
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
            circuit: self.active,
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

    /// Moves several placements, as one entry in the history: one drag, one undo.
    pub fn move_many(&mut self, moves: &[(usize, Pt)]) -> bool {
        let mut changes = Vec::with_capacity(moves.len());
        for (index, centre) in moves {
            let Some(placement) = self.placements.get_mut(*index) else {
                continue;
            };
            if placement.centre == *centre {
                continue;
            }
            let from = placement.centre;
            placement.centre = *centre;
            changes.push(Change::Moved {
                index: *index,
                from,
                to: *centre,
            });
        }
        if changes.is_empty() {
            return false;
        }
        self.record_group(changes);
        true
    }

    /// Removes several placements, with their pipes, as one entry in the history.
    pub fn remove_many(&mut self, indices: &[usize]) -> Vec<Placement> {
        let mut changes = Vec::with_capacity(indices.len());
        let mut removed = Vec::with_capacity(indices.len());
        // Highest index first, so every index is still valid when it is used.
        let mut ordered: Vec<usize> = indices.to_vec();
        ordered.sort_unstable_by(|a, b| b.cmp(a));
        ordered.dedup();
        for index in ordered {
            let Some((placement, orphans)) = self.take_placement(index) else {
                continue;
            };
            changes.push(Change::Removed {
                index,
                placement: placement.clone(),
                orphans,
            });
            removed.push(placement);
        }
        if !changes.is_empty() {
            self.record_group(changes);
        }
        removed
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

    /// Gives a placement a designation, or takes it away with `None`.
    ///
    /// Whitespace is trimmed, and an empty text means no designation: a name of spaces is
    /// a name nobody can read, and keeping it would let two components look unnamed and be
    /// reported as duplicates. A name longer than [`MAX_DESIGNATION`] characters is refused
    /// instead of truncated, because half a name is worse than a clear refusal.
    pub fn designate(&mut self, index: usize, designation: Option<&str>) -> bool {
        let wanted = designation
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_owned);
        if wanted.as_ref().is_some_and(|text| {
            text.chars().count() > MAX_DESIGNATION || text.chars().any(char::is_control)
        }) {
            return false;
        }
        let Some(placement) = self.placements.get_mut(index) else {
            return false;
        };
        if placement.designation == wanted {
            return false;
        }
        let from = std::mem::replace(&mut placement.designation, wanted.clone());
        self.record(Change::Designated {
            index,
            from,
            to: wanted,
        });
        true
    }

    /// The designation of one placement, if it has one.
    #[must_use]
    pub fn designation_of(&self, index: usize) -> Option<&str> {
        self.placements.get(index)?.designation.as_deref()
    }

    /// Binds a recorder to a source, or unbinds it with `None`.
    ///
    /// The document refuses what it can decide alone: a source that is not in the document,
    /// that names the recorder itself, or that lives in another circuit. Whether the chosen
    /// component can report the quantity is the catalog's business, so the caller checks it
    /// before asking — the document never guesses a definition.
    pub fn bind_source(&mut self, index: usize, source: Option<Source>) -> bool {
        let Some(recorder) = self.placements.get(index) else {
            return false;
        };
        let recorder_id = recorder.id;
        let circuit = recorder.circuit;
        if let Some(source) = source {
            if source.placement == recorder_id {
                return false;
            }
            let Some(target) = self
                .placements
                .iter()
                .find(|placement| placement.id == source.placement)
            else {
                return false;
            };
            if target.circuit != circuit {
                return false;
            }
        }
        let Some(recorder) = self.placements.get_mut(index) else {
            return false;
        };
        if recorder.source == source {
            return false;
        }
        let from = recorder.source;
        recorder.source = source;
        self.record(Change::Sourced {
            index,
            from,
            to: source,
        });
        true
    }

    /// The source of one placement, when it is bound.
    #[must_use]
    pub fn source_of(&self, index: usize) -> Option<Source> {
        self.placements.get(index)?.source
    }

    /// Turns a placement a quarter clockwise: `Edición ▸ Girar`.
    pub fn turn_clockwise(&mut self, index: usize) -> bool {
        self.turn_many(&[index], Turn::Clockwise)
    }

    /// Turns a placement a quarter anticlockwise: `Ctrl` and a double click.
    pub fn turn_anticlockwise(&mut self, index: usize) -> bool {
        self.turn_many(&[index], Turn::Anticlockwise)
    }

    /// Reflects a placement about its own vertical axis: `Edición ▸ Reflejar`.
    pub fn reflect(&mut self, index: usize) -> bool {
        self.turn_many(&[index], Turn::Reflect)
    }

    /// Lines several placements up on the one that is furthest out, as one change.
    ///
    /// The reference moves the marked objects until they reach the one already at that
    /// edge, and it says why the result may look a fraction off: a pneumatic symbol sits on
    /// the circuit grid, not inside its own drawing box.
    pub fn align_many(&mut self, indices: &[usize], align: Align) -> bool {
        let boxes: Vec<(usize, Rect)> = indices
            .iter()
            .filter_map(|index| {
                let placement = self.placements.get(*index)?;
                Some((*index, placement.box_rect()))
            })
            .collect();
        if boxes.len() < 2 {
            return false;
        }
        let mut reference = align.edge(boxes[0].1);
        for (_, rect) in &boxes {
            let edge = align.edge(*rect);
            if align.further_out(edge, reference) {
                reference = edge;
            }
        }
        let mut moves: Vec<(usize, Pt)> = Vec::with_capacity(boxes.len());
        for (index, rect) in boxes {
            let delta = reference - align.edge(rect);
            if delta.abs() <= f32::EPSILON {
                continue;
            }
            let centre = self.placements[index].centre;
            moves.push((
                index,
                if align.horizontal() {
                    Pt::new(centre.x + delta, centre.y)
                } else {
                    Pt::new(centre.x, centre.y + delta)
                },
            ));
        }
        if moves.is_empty() {
            return false;
        }
        self.move_many(&moves)
    }

    /// Spreads several placements so the gaps between their boxes are equal.
    ///
    /// The two outermost keep their places, so the drawing keeps its extent and only the
    /// middle moves — which is what a student means by "equal spacing" after dragging two
    /// objects apart and wanting the rest to follow. The gaps are measured between symbol
    /// **boxes**, not centres, because a wide cylinder and a narrow valve on centres look
    /// evenly spaced and are not.
    ///
    /// Refuses fewer than three placements, where there is only one gap and nothing to
    /// make equal, and a selection that does not fit between its own ends.
    pub fn space_evenly(&mut self, indices: &[usize], axis: Axis) -> bool {
        let mut boxes: Vec<(usize, Rect)> = indices
            .iter()
            .filter_map(|index| {
                let placement = self.placements.get(*index)?;
                Some((*index, placement.box_rect()))
            })
            .collect();
        if boxes.len() < 3 {
            return false;
        }
        boxes.sort_by(|left, right| {
            axis.min(left.1)
                .partial_cmp(&axis.min(right.1))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let Some((_, first)) = boxes.first().copied() else {
            return false;
        };
        let Some((_, last)) = boxes.last().copied() else {
            return false;
        };
        let occupied: f32 = boxes.iter().map(|(_, rect)| axis.size(*rect)).sum();
        let span = axis.max(last) - axis.min(first);
        #[allow(clippy::cast_precision_loss)]
        let gap = (span - occupied) / (boxes.len() - 1) as f32;
        if gap < 0.0 {
            return false;
        }
        let mut moves: Vec<(usize, Pt)> = Vec::with_capacity(boxes.len() - 2);
        let mut cursor = axis.max(first) + gap;
        for (index, rect) in boxes.iter().skip(1).take(boxes.len().saturating_sub(2)) {
            let delta = cursor - axis.min(*rect);
            if delta.abs() > f32::EPSILON {
                let centre = self.placements[*index].centre;
                moves.push((*index, axis.moved(centre, delta)));
            }
            cursor += axis.size(*rect) + gap;
        }
        if moves.is_empty() {
            return false;
        }
        self.move_many(&moves)
    }

    /// Turns every listed placement, as one entry in the history.
    ///
    /// A selection of six valves turned in one gesture is one change to the user, and a
    /// user who has to press undo six times for it has been told the program does not mind
    /// losing their work.
    pub fn turn_many(&mut self, indices: &[usize], turn: Turn) -> bool {
        let mut changes = Vec::with_capacity(indices.len());
        for index in indices {
            let Some(placement) = self.placements.get_mut(*index) else {
                continue;
            };
            let from = placement.orientation;
            let to = turn.apply(from);
            if to == from {
                continue;
            }
            placement.orientation = to;
            changes.push(Change::Turned {
                index: *index,
                from,
                to,
            });
        }
        if changes.is_empty() {
            return false;
        }
        self.record_group(changes);
        true
    }

    /// How a placement is turned, or `None` when there is no such placement.
    #[must_use]
    pub fn orientation_of(&self, index: usize) -> Option<Orientation> {
        self.placements
            .get(index)
            .map(|placement| placement.orientation)
    }

    /// Removes a placement, and with it every pipe it carried.
    pub fn remove(&mut self, index: usize) -> Option<Placement> {
        let (placement, orphans) = self.take_placement(index)?;
        self.record(Change::Removed {
            index,
            placement: placement.clone(),
            orphans,
        });
        Some(placement)
    }

    /// Takes a placement and its pipes out of the document, without recording anything.
    fn take_placement(&mut self, index: usize) -> Option<(Placement, Vec<(usize, Connection)>)> {
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
        Some((placement, orphans))
    }

    /// How many times the document has changed since it was created.
    ///
    /// It counts the changes the history records and the ones an undo or a redo puts
    /// back, so a caller that remembers the number at the moment it saved knows whether
    /// there is work that is not in the file. Two things it does not do: it does not fall
    /// when an undo returns the document to its saved shape — a document that was changed
    /// and changed back has still been edited — and it does not survive a round trip
    /// through a file, because a loaded document is a new document.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
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
        self.revision = self.revision.wrapping_add(1);
        true
    }

    /// Applies the last undone change again.
    pub fn redo(&mut self) -> bool {
        let Some(change) = self.redo.pop() else {
            return false;
        };
        self.apply(&change);
        self.undo.push(change);
        self.revision = self.revision.wrapping_add(1);
        true
    }

    fn take_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    /// Records several changes as one entry, or the change itself when there is one.
    fn record_group(&mut self, mut changes: Vec<Change>) {
        match changes.len() {
            0 => {}
            1 => {
                if let Some(only) = changes.pop() {
                    self.record(only);
                }
            }
            _ => self.record(Change::Batch(changes)),
        }
    }

    fn record(&mut self, change: Change) {
        self.revision = self.revision.wrapping_add(1);
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
        // A drag of several components is a batch of moves, and it is still one drag:
        // consecutive batches that move the same components extend the top entry.
        if let (Some(Change::Batch(last)), Change::Batch(current)) = (self.undo.last_mut(), &change)
            && extends_the_same_moves(last, current)
        {
            for (existing, fresh) in last.iter_mut().zip(current) {
                if let (Change::Moved { to: last_to, .. }, Change::Moved { to, .. }) =
                    (existing, fresh)
                {
                    *last_to = *to;
                }
            }
            self.redo.clear();
            return;
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
            Change::Designated { index, to, .. } => {
                if let Some(placement) = self.placements.get_mut(*index) {
                    placement.designation.clone_from(to);
                }
            }
            Change::Sourced { index, to, .. } => {
                if let Some(placement) = self.placements.get_mut(*index) {
                    placement.source = *to;
                }
            }
            Change::CircuitAdded { index, circuit } => {
                insert_at(&mut self.circuits, *index, circuit.clone());
                self.active = circuit.id;
            }
            Change::CircuitRenamed { id, to, .. } => {
                if let Some(circuit) = self.circuits.iter_mut().find(|circuit| circuit.id == *id) {
                    circuit.title.clone_from(to);
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
            Change::Designated { index, from, .. } => {
                if let Some(placement) = self.placements.get_mut(*index) {
                    placement.designation.clone_from(from);
                }
            }
            Change::Sourced { index, from, .. } => {
                if let Some(placement) = self.placements.get_mut(*index) {
                    placement.source = *from;
                }
            }
            Change::CircuitAdded { circuit, .. } => {
                self.circuits.retain(|candidate| candidate.id != circuit.id);
                self.placements
                    .retain(|placement| placement.circuit != circuit.id);
                self.connections
                    .retain(|connection| connection.circuit != circuit.id);
                if self.active == circuit.id {
                    self.active = self.circuits.first().map_or(0, |circuit| circuit.id);
                }
            }
            Change::CircuitRenamed { id, from, .. } => {
                if let Some(circuit) = self.circuits.iter_mut().find(|circuit| circuit.id == *id) {
                    circuit.title.clone_from(from);
                }
            }
        }
    }
}

/// `true` when two batches move exactly the same placements, in the same order.
fn extends_the_same_moves(last: &[Change], current: &[Change]) -> bool {
    last.len() == current.len()
        && !last.is_empty()
        && last.iter().zip(current).all(|(old, new)| {
            matches!(
                (old, new),
                (Change::Moved { index: a, .. }, Change::Moved { index: b, .. }) if a == b
            )
        })
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
    fn moving_a_selection_is_one_drag_and_one_undo() {
        let mut document = Document::new();
        let a = document.place(placement("a", 10.0, 10.0));
        let b = document.place(placement("b", 40.0, 10.0));
        let watchers = document.can_undo();
        assert!(watchers);
        // Three mouse moves of a drag, the same two components each time.
        for step in 1..=3 {
            #[allow(clippy::cast_precision_loss)]
            let dx = step as f32;
            assert!(
                document.move_many(&[(a, Pt::new(10.0 + dx, 10.0)), (b, Pt::new(40.0 + dx, 10.0))])
            );
        }
        assert!(close(document.placements()[a].centre.x, 13.0));
        assert!(close(document.placements()[b].centre.x, 43.0));
        // One undo puts both back: the drag extended the entry, it did not add two more.
        assert!(document.undo());
        assert!(close(document.placements()[a].centre.x, 10.0));
        assert!(close(document.placements()[b].centre.x, 40.0));
        // What is left in the history is the two placements and nothing else.
        let mut earlier = 0;
        while document.undo() {
            earlier += 1;
        }
        assert_eq!(earlier, 2, "the whole drag was a single entry");
    }

    #[test]
    fn removing_and_copying_a_selection_are_one_change_each() {
        let mut document = Document::new();
        let a = document.place(placement("a", 10.0, 10.0));
        let b = document.place(placement("b", 40.0, 10.0));
        let c = document.place(placement("c", 70.0, 10.0));
        assert!(document.connect(
            Endpoint::new(document.placements()[a].id(), "1"),
            Endpoint::new(document.placements()[b].id(), "1")
        ));
        let copies = document.duplicate_many(&[a, b, c], (10.0, 10.0));
        assert_eq!(copies.len(), 3);
        assert_eq!(document.len(), 6);
        assert!(document.undo());
        assert_eq!(document.len(), 3, "one undo puts the three copies away");

        let removed = document.remove_many(&[a, b]);
        assert_eq!(removed.len(), 2);
        assert_eq!(document.len(), 1);
        assert_eq!(
            document.connections().len(),
            0,
            "the pipe went with its ends"
        );
        assert!(document.undo());
        assert_eq!(document.len(), 3);
        assert_eq!(document.connections().len(), 1, "and came back with it");
    }

    #[test]
    fn aligning_moves_everything_onto_the_object_already_at_that_edge() {
        let mut document = Document::new();
        let a = document.place(Placement::new("a", Pt::new(20.0, 20.0), 20.0, 10.0));
        let b = document.place(Placement::new("b", Pt::new(50.0, 60.0), 10.0, 10.0));
        let c = document.place(Placement::new("c", Pt::new(80.0, 90.0), 20.0, 10.0));
        assert!(document.align_many(&[a, b, c], Align::Left));
        let lefts: Vec<f32> = [a, b, c]
            .iter()
            .map(|index| document.placements()[*index].box_rect().min.x)
            .collect();
        assert!(
            lefts.iter().all(|left| close(*left, 10.0)),
            "every left edge is the leftmost: {lefts:?}"
        );
        // Nothing moved vertically, and the whole alignment is a single undo.
        assert!(close(document.placements()[b].centre.y, 60.0));
        assert!(document.undo());
        assert!(close(document.placements()[a].centre.x, 20.0));
        assert!(close(document.placements()[c].centre.x, 80.0));
        // Right edges line up on the rightmost one, which is `c` at 90.
        assert!(document.align_many(&[a, b, c], Align::Right));
        let rights: Vec<f32> = [a, b, c]
            .iter()
            .map(|index| document.placements()[*index].box_rect().max.x)
            .collect();
        assert!(
            rights.iter().all(|right| close(*right, 90.0)),
            "every right edge is the rightmost: {rights:?}"
        );
        // One component on its own has nothing to line up with.
        assert!(!document.align_many(&[a], Align::Top));
    }

    #[test]
    fn turning_a_selection_keeps_the_mirror_of_each_component() {
        let mut document = Document::new();
        let a = document.place(placement("a", 10.0, 10.0));
        let b = document.place(placement("b", 40.0, 10.0));
        assert!(document.reflect(b));
        assert!(document.turn_many(&[a, b], Turn::Clockwise));
        assert_eq!(document.orientation_of(a), Some(Orientation::new(1, false)));
        assert_eq!(document.orientation_of(b), Some(Orientation::new(1, true)));
        assert!(document.undo());
        assert_eq!(document.orientation_of(a), Some(Orientation::NONE));
        assert_eq!(document.orientation_of(b), Some(Orientation::new(0, true)));
    }

    #[test]
    fn a_designation_is_one_undo_and_an_empty_name_is_no_name() {
        let mut document = Document::new();
        let index = document.place(placement("cylinder.single_acting_20_100", 10.0, 10.0));
        assert!(document.designate(index, Some("  1A1  ")));
        assert_eq!(document.designation_of(index), Some("1A1"), "trimmed");
        assert!(document.undo());
        assert_eq!(document.designation_of(index), None);
        assert!(document.redo());
        assert_eq!(document.designation_of(index), Some("1A1"));
        // A name of spaces is not a name, and clearing one leaves nothing behind.
        assert!(document.designate(index, Some("   ")));
        assert_eq!(document.designation_of(index), None);
        // Renaming the same component to the same text is not a change.
        assert!(document.designate(index, Some("2A1")));
        assert!(!document.designate(index, Some("2A1")));
        // A name too long to fit beside a symbol is refused instead of truncated.
        let long = "x".repeat(MAX_DESIGNATION + 1);
        assert!(!document.designate(index, Some(&long)));
        assert!(!document.designate(index, Some("con\nsalto")));
        assert_eq!(document.designation_of(index), Some("2A1"));
        assert_eq!(document.designation_of(99), None);
    }

    #[test]
    fn the_revision_counts_edits_undo_and_redo() {
        let mut document = Document::new();
        let start = document.revision();
        let index = document.place(placement("a", 10.0, 10.0));
        assert!(document.revision() > start, "a placement is a change");
        let placed = document.revision();
        assert!(document.move_to(index, Pt::new(20.0, 10.0)));
        assert!(document.revision() > placed, "a move is a change");
        let moved = document.revision();
        assert!(document.undo());
        assert_ne!(document.revision(), moved, "an undo is a change too");
        assert!(document.redo());
        // Marking is not a change: nothing about the document moves.
        let before = document.revision();
        let _ = document.orientation_of(index);
        assert_eq!(document.revision(), before);
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

#[cfg(test)]
mod circuit_tests {
    use super::*;

    #[test]
    fn adding_a_circuit_switches_to_it_and_one_undo_takes_it_back() {
        let mut document = Document::new();
        let first = document.place(Placement::new(
            "supply.compressor",
            Pt::new(10.0, 10.0),
            20.0,
            20.0,
        ));
        assert_eq!(document.placements()[first].circuit, 0);

        let control = document.add_circuit("Control");
        assert_eq!(control, 1);
        assert_eq!(document.active_circuit(), 1);
        let second = document.place(Placement::new(
            "valve.3_2.spring_return",
            Pt::new(30.0, 30.0),
            20.0,
            20.0,
        ));
        assert_eq!(document.placements()[second].circuit, 1);

        assert!(document.undo());
        assert_eq!(
            document.placements().len(),
            1,
            "the placement in the new circuit goes"
        );
        assert!(document.undo());
        assert!(
            document
                .circuits()
                .iter()
                .all(|circuit| circuit.id != control)
        );
        assert_eq!(
            document.active_circuit(),
            0,
            "the first circuit becomes active again"
        );
        assert_eq!(
            document.placements().len(),
            1,
            "the first circuit keeps its placement"
        );
    }

    #[test]
    fn renaming_a_circuit_is_one_undo() {
        let mut document = Document::new();
        assert!(document.rename_circuit(0, "Power"));
        assert_eq!(document.circuits()[0].title, "Power");
        assert!(document.undo());
        assert_eq!(document.circuits()[0].title, "Circuito 1");
    }

    #[test]
    fn an_unknown_circuit_cannot_be_selected() {
        let mut document = Document::new();
        assert!(!document.select_circuit(99));
        assert_eq!(document.active_circuit(), 0);
    }
}

#[cfg(test)]
mod source_tests {
    use super::*;

    fn place(document: &mut Document, id: &str, x: f32) -> usize {
        document.place(Placement::new(id, Pt::new(x, 20.0), 20.0, 20.0))
    }

    #[test]
    fn a_recorder_is_bound_to_a_source_and_one_undo_takes_it_back() {
        let mut document = Document::new();
        let gauge = place(&mut document, "gauge.pressure", 30.0);
        let recorder = place(&mut document, "chart.recorder", 80.0);
        let source = Source::new(document.placements()[gauge].id(), Quantity::Pressure);
        assert!(document.bind_source(recorder, Some(source)));
        assert_eq!(document.source_of(recorder), Some(source));

        assert!(document.undo());
        assert_eq!(document.source_of(recorder), None, "one undo unbinds it");
        assert!(document.redo());
        assert_eq!(
            document.source_of(recorder),
            Some(source),
            "and redo returns it"
        );
    }

    #[test]
    fn a_source_must_be_another_component_of_the_same_circuit() {
        let mut document = Document::new();
        let gauge = place(&mut document, "gauge.pressure", 30.0);
        let recorder = place(&mut document, "chart.recorder", 80.0);
        let gauge_id = document.placements()[gauge].id();
        let recorder_id = document.placements()[recorder].id();

        assert!(
            !document.bind_source(recorder, Some(Source::new(recorder_id, Quantity::Pressure))),
            "a recorder cannot chart itself"
        );
        assert!(
            !document.bind_source(recorder, Some(Source::new(9_999, Quantity::Pressure))),
            "a source has to exist"
        );

        document.add_circuit("Control");
        let other = place(&mut document, "gauge.pressure", 30.0);
        let other_id = document.placements()[other].id();
        assert!(
            !document.bind_source(recorder, Some(Source::new(other_id, Quantity::Pressure))),
            "a source in another circuit is not on the same sheet"
        );
        // And the gauge of the first circuit still binds.
        assert!(document.bind_source(recorder, Some(Source::new(gauge_id, Quantity::Pressure))));
    }

    #[test]
    fn a_quantity_round_trips_through_its_name() {
        for quantity in [Quantity::Pressure, Quantity::Flow, Quantity::Position] {
            assert_eq!(Quantity::from_name(quantity.as_str()), Some(quantity));
        }
        assert_eq!(Quantity::from_name("temperature"), None);
    }
}

#[cfg(test)]
mod spacing_tests {
    use super::*;

    fn place(document: &mut Document, width: f32, x: f32, y: f32) -> usize {
        document.place(Placement::new(
            "valve.3_2.spring_return",
            Pt::new(x, y),
            width,
            20.0,
        ))
    }

    /// The gaps between the boxes of `indices`, left to right.
    fn gaps(document: &Document, indices: &[usize]) -> Vec<f32> {
        let mut boxes: Vec<Rect> = indices
            .iter()
            .map(|index| document.placements()[*index].box_rect())
            .collect();
        boxes.sort_by(|left, right| {
            left.min
                .x
                .partial_cmp(&right.min.x)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        boxes
            .windows(2)
            .map(|pair| pair[1].min.x - pair[0].max.x)
            .collect()
    }

    #[test]
    fn even_spacing_equalises_the_gaps_and_keeps_the_ends() {
        let mut document = Document::new();
        // Three valves of different widths, unevenly spread.
        let first = place(&mut document, 20.0, 20.0, 40.0);
        let middle = place(&mut document, 40.0, 60.0, 40.0);
        let last = place(&mut document, 20.0, 150.0, 40.0);
        let before: Vec<f32> = document.placements().iter().map(|p| p.centre.x).collect();
        assert!(document.space_evenly(&[first, middle, last], Axis::Horizontal));
        let gaps = gaps(&document, &[first, middle, last]);
        assert_eq!(gaps.len(), 2);
        assert!(
            (gaps[0] - gaps[1]).abs() < 0.01,
            "the gaps are {gaps:?}, not equal"
        );
        assert!(
            (document.placements()[first].centre.x - before[0]).abs() < f32::EPSILON
                && (document.placements()[last].centre.x - before[2]).abs() < f32::EPSILON,
            "the outermost components moved"
        );
        assert!(
            (document.placements()[middle].centre.x - before[1]).abs() > 1.0,
            "the middle component did not move"
        );
    }

    #[test]
    fn even_spacing_is_one_undo() {
        let mut document = Document::new();
        let first = place(&mut document, 20.0, 20.0, 20.0);
        let middle = place(&mut document, 20.0, 40.0, 55.0);
        let last = place(&mut document, 20.0, 60.0, 140.0);
        let before: Vec<Pt> = document.placements().iter().map(|p| p.centre).collect();
        assert!(document.space_evenly(&[first, middle, last], Axis::Vertical));
        assert!(document.undo());
        let after: Vec<Pt> = document.placements().iter().map(|p| p.centre).collect();
        assert_eq!(after, before, "one undo puts every centre back");
    }

    #[test]
    fn even_spacing_refuses_what_it_cannot_do() {
        let mut document = Document::new();
        let first = place(&mut document, 20.0, 20.0, 40.0);
        let second = place(&mut document, 20.0, 100.0, 40.0);
        assert!(
            !document.space_evenly(&[first, second], Axis::Horizontal),
            "two components have a single gap and nothing to equalise"
        );
        // Three boxes wider than the span between their ends cannot be spread.
        let tight = Document::new();
        let mut tight = tight;
        let a = place(&mut tight, 40.0, 30.0, 40.0);
        let b = place(&mut tight, 40.0, 50.0, 40.0);
        let c = place(&mut tight, 40.0, 70.0, 40.0);
        assert!(
            !tight.space_evenly(&[a, b, c], Axis::Horizontal),
            "overlapping boxes that do not fit are left alone"
        );
    }
}
