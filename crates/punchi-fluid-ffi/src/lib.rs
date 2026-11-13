// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The C ABI a native front end speaks.
//!
//! The Rust core owns the model and the drawing; this crate turns both into a flat
//! `repr(C)` description that a C++ or QML window replays. Two decisions shape it:
//!
//! - **The drawing crosses as a command buffer, not as callbacks.** One call per frame
//!   fills a [`PfFrame`] with [`PfCall`] records that the front end replays onto a
//!   `QPainter`. The screen then consumes the same primitive list the export tests
//!   already compare, so the window and the paper cannot drift apart.
//! - **Colour comes in from outside.** [`PfInk`] is filled by the front end from the
//!   desktop theme. Nothing here picks a colour, which is the rule the UI already
//!   lives by.
//!
//! Dash patterns are expanded here, in millimetres, by the same function the PDF and
//! PNG backends use. A backend that re-invented them would be a second drawing.
//!
//! Nothing may unwind across the boundary: every entry point catches unwinding and
//! reports a code. Pointers handed out by [`pf_session_build_frame`] point into the
//! session and stay valid until the next rebuild or [`pf_session_free`].

#![allow(
    unsafe_code,
    reason = "a C ABI dereferences caller pointers and is entered with no Rust frame above it; the workspace forbids unsafe everywhere else, and this is the only crate that opts out"
)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "lengths and counts are bounded by one frame of drawing, far below u32::MAX"
)]

use std::ffi::{CStr, CString, c_char};
use std::fmt::Write as _;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};

use punchi_fluid_catalog::{Behaviour, Catalog};
use punchi_fluid_check::check;

/// The junction the splice inserts, and the box it is placed in.
///
/// The box is small because the symbol is a single dot: it exists so the junction can be
/// hit and selected, not to take up room on the sheet.
const JUNCTION_ID: &str = "junction.tee";
const JUNCTION_MM: f32 = 8.0;
use punchi_fluid_doc::{
    Align, Circuit, Document, Endpoint, Placement, PlacementEdits, Quantity, Turn,
};
use punchi_fluid_draw::placed::{
    Pipe, PipeEnd, Placed, Reading, draw_placed, port_edge, port_point, route_between,
};
use punchi_fluid_draw::surface::{Anchor, Colour, LineClass, Pt, Rect, Stroke, Surface};
use punchi_fluid_draw::symbol::Ink;
use punchi_fluid_draw::{Layout, Orientation, dashes};
use punchi_fluid_file::recent::RecentList;
use punchi_fluid_file::sheet::{Sheet, StoredEdits, StoredPipe, StoredPlacement};
use punchi_fluid_solver::network::{Network, NetworkPart, PartBehaviour, PortAddress};
use punchi_fluid_solver::pneumatic::Telemetry;

/// The call succeeded.
pub const PF_OK: i32 = 0;
/// A required pointer was null.
pub const PF_ERR_NULL: i32 = -1;
/// The model refused the request; see [`pf_session_last_error`].
pub const PF_ERR_MODEL: i32 = -2;
/// A panic was caught at the boundary.
pub const PF_ERR_PANIC: i32 = -3;

/// Primitive kinds carried by [`PfCall::kind`].
pub const PF_KIND_LINE: u32 = 0;
/// Closed polygon.
pub const PF_KIND_POLYGON: u32 = 1;
/// Rectangle, optionally rounded.
pub const PF_KIND_RECT: u32 = 2;
/// Circle.
pub const PF_KIND_CIRCLE: u32 = 3;
/// Arc.
pub const PF_KIND_ARC: u32 = 4;
/// Text run.
pub const PF_KIND_TEXT: u32 = 5;

/// Catalog categories, in the order the sidebar groups them.
pub const PF_CATEGORY_SUPPLY: u32 = 0;
/// Directional or flow control valve.
pub const PF_CATEGORY_VALVE: u32 = 1;
/// Cylinder, motor or other actuator.
pub const PF_CATEGORY_ACTUATOR: u32 = 2;
/// Restriction without a spool.
pub const PF_CATEGORY_THROTTLE: u32 = 3;
/// Silencer, filter, manifold.
pub const PF_CATEGORY_ACCESSORY: u32 = 4;
/// Pressure switch, gauge.
pub const PF_CATEGORY_SENSOR: u32 = 5;

/// [`PfCall::flags`] bit: the call carries a fill colour.
pub const PF_HAS_FILL: u32 = 1;
/// [`PfCall::flags`] bit: the call carries a stroke.
pub const PF_HAS_STROKE: u32 = 2;

/// RGB triplet, eight bits per channel, sRGB.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PfRgb {
    /// Red.
    pub r: u8,
    /// Green.
    pub g: u8,
    /// Blue.
    pub b: u8,
}

impl PfRgb {
    const BLACK: Self = Self { r: 0, g: 0, b: 0 };

    const fn from_colour(colour: Colour) -> Self {
        Self {
            r: colour.r,
            g: colour.g,
            b: colour.b,
        }
    }

    const fn to_colour(self) -> Colour {
        Colour {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
}

/// Every colour the drawing needs, in the order the front end fills them.
///
/// The front end reads these from the desktop theme; the Rust side never derives one,
/// so a light desktop and a dark desktop produce different ink from the same drawing.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PfInk {
    /// Lines of a symbol.
    pub line: PfRgb,
    /// Lines of an inactive position.
    pub dim: PfRgb,
    /// Secondary text.
    pub muted: PfRgb,
    /// Primary text.
    pub text: PfRgb,
    /// Supply line.
    pub supply: PfRgb,
    /// Working line.
    pub work: PfRgb,
    /// Active square.
    pub accent: PfRgb,
    /// Paper behind the drawing.
    pub background: PfRgb,
}

impl PfInk {
    fn to_ink(self) -> Ink {
        Ink {
            line: self.line.to_colour(),
            dim: self.dim.to_colour(),
            muted: self.muted.to_colour(),
            text: self.text.to_colour(),
            supply: self.supply.to_colour(),
            work: self.work.to_colour(),
            accent: self.accent.to_colour(),
            background: self.background.to_colour(),
        }
    }
}

/// The state the front end reads every frame, in SI plus the valve position.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PfTelemetry {
    /// Simulated time, `s`.
    pub time_s: f64,
    /// Absolute cylinder pressure, `Pa`.
    pub pressure_pa: f64,
    /// Piston position, `m`.
    pub position_m: f64,
    /// Piston velocity, `m/s`.
    pub velocity_m_s: f64,
    /// Gas mass in, `kg`.
    pub mass_in_kg: f64,
    /// Gas mass out, `kg`.
    pub mass_out_kg: f64,
    /// Active valve position.
    pub valve_position: u32,
    /// `1` when a model is loaded, `0` when the catalog had no usable definition.
    pub has_model: u32,
    /// Usable stroke of the cylinder, `m`.
    ///
    /// Without it a front end can see the piston move but not tell when it has arrived:
    /// "fully extended" is a comparison against this number, and a click at the end of the
    /// stroke is an event worth a sound.
    pub stroke_m: f64,
}

/// One primitive of one frame.
///
/// Fields that the `kind` does not use are zero. `points`/`text` are ranges into the
/// frame's arenas, not pointers, so the whole frame is one allocation to copy.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PfCall {
    /// One of the `PF_KIND_*` values.
    pub kind: u32,
    /// `PF_HAS_FILL`, `PF_HAS_STROKE`, or both.
    pub flags: u32,
    /// Line class: 0 solid, 1 pilot, 2 drain, 3 electrical. Informational.
    pub line_class: u32,
    /// Text anchor: 0 start, 1 middle, 2 end.
    pub anchor: u32,
    /// First index into `PfFrame::points`.
    pub points_at: u32,
    /// Number of points.
    pub points_len: u32,
    /// Byte offset into `PfFrame::text`.
    pub text_at: u32,
    /// Byte length of the UTF-8 run.
    pub text_len: u32,
    /// Rectangle `x0 y0 x1 y1`; for a circle or an arc, `cx cy r 0`.
    pub rect: [f32; 4],
    /// Arc start and end angle, degrees clockwise.
    pub angles: [f32; 2],
    /// Corner radius of a rectangle, millimetres.
    pub corner: f32,
    /// Cap height of a text run, millimetres.
    pub size: f32,
    /// Stroke width, millimetres.
    pub width: f32,
    /// Stroke colour.
    pub stroke: PfRgb,
    /// Fill colour, and the colour of a text run.
    pub fill: PfRgb,
}

/// A point in millimetres of the drawing.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PfPoint {
    /// Horizontal.
    pub x: f32,
    /// Vertical, downwards.
    pub y: f32,
}

/// One frame of drawing, borrowed from the session.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PfFrame {
    /// Primitive list.
    pub calls: *const PfCall,
    /// Number of primitives.
    pub calls_len: u32,
    /// Point arena.
    pub points: *const PfPoint,
    /// Number of points.
    pub points_len: u32,
    /// UTF-8 text arena.
    pub text: *const u8,
    /// Bytes of text.
    pub text_len: u32,
    /// Drawing width in millimetres, as the model laid it out.
    pub width_mm: f32,
    /// Drawing height in millimetres.
    pub height_mm: f32,
}

/// One quantity a recorder can chart, and the placement it belongs to.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PfChartSource {
    /// Identity of the placement being observed.
    pub placement: u32,
    /// `PF_QUANTITY_PRESSURE`, `PF_QUANTITY_FLOW` or `PF_QUANTITY_POSITION`.
    pub quantity: u32,
}

/// Gauge pressure of a port, `bar`.
pub const PF_QUANTITY_PRESSURE: u32 = 0;
/// Flow through a meter, normal litres per minute.
pub const PF_QUANTITY_FLOW: u32 = 1;
/// Extension of a cylinder, millimetres.
pub const PF_QUANTITY_POSITION: u32 = 2;

/// One catalog entry: a component the sidebar shows, by its own symbol.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PfCatalogItem {
    /// Byte offset of the id in the catalog's text arena.
    pub id_at: u32,
    /// Length of the id.
    pub id_len: u32,
    /// Byte offset of the name.
    pub name_at: u32,
    /// Length of the name.
    pub name_len: u32,
    /// One of the `PF_CATEGORY_*` values.
    pub category: u32,
}

/// The catalog in display order, borrowed from the session.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PfCatalog {
    /// Entries, grouped by category and then by id.
    pub items: *const PfCatalogItem,
    /// Number of entries.
    pub items_len: u32,
    /// UTF-8 arena holding the ids and names.
    pub text: *const u8,
    /// Bytes of text.
    pub text_len: u32,
}

/// Sizes of the structures below, so the C side can prove it agrees with Rust.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PfAbi {
    /// Bumped whenever a structure changes shape.
    pub version: u32,
    /// `sizeof(PfCall)`.
    pub call_size: u32,
    /// `sizeof(PfTelemetry)`.
    pub telemetry_size: u32,
    /// `sizeof(PfFrame)`.
    pub frame_size: u32,
    /// `sizeof(PfInk)`.
    pub ink_size: u32,
    /// `sizeof(PfPoint)`.
    pub point_size: u32,
}

/// The ABI layout this build produces.
pub const PF_ABI_VERSION: u32 = 5;

fn rgb(colour: Option<Colour>) -> PfRgb {
    colour.map_or(PfRgb::BLACK, PfRgb::from_colour)
}

fn class_code(class: LineClass) -> u32 {
    match class {
        LineClass::Working => 0,
        LineClass::Pilot => 1,
        LineClass::Drain => 2,
        LineClass::Electrical => 3,
    }
}

fn category_code(category: punchi_fluid_catalog::Category) -> u32 {
    use punchi_fluid_catalog::Category;
    match category {
        Category::Supply => PF_CATEGORY_SUPPLY,
        Category::Valve => PF_CATEGORY_VALVE,
        Category::Actuator => PF_CATEGORY_ACTUATOR,
        Category::Throttle => PF_CATEGORY_THROTTLE,
        Category::Accessory => PF_CATEGORY_ACCESSORY,
        Category::Sensor => PF_CATEGORY_SENSOR,
    }
}

fn anchor_code(anchor: Anchor) -> u32 {
    match anchor {
        Anchor::Start => 0,
        Anchor::Middle => 1,
        Anchor::End => 2,
    }
}

/// Writes the drawing into the flat frame the front end replays.
struct FrameBuffer<'a> {
    calls: &'a mut Vec<PfCall>,
    points: &'a mut Vec<PfPoint>,
    text: &'a mut Vec<u8>,
    scale: f32,
}

impl FrameBuffer<'_> {
    fn push_points(&mut self, source: &[Pt]) -> (u32, u32) {
        let at = self.points.len() as u32;
        self.points.extend(source.iter().map(|point| PfPoint {
            x: point.x,
            y: point.y,
        }));
        (at, source.len() as u32)
    }
}

impl Surface for FrameBuffer<'_> {
    fn line(&mut self, points: &[Pt], stroke: Stroke) {
        // The pattern is expanded here so the window, the PNG and the PDF consume the
        // same segments instead of three dash implementations that can disagree.
        for run in dashes(points, stroke.class.pattern()) {
            let (points_at, points_len) = self.push_points(&run);
            self.calls.push(PfCall {
                kind: PF_KIND_LINE,
                flags: PF_HAS_STROKE,
                line_class: class_code(stroke.class),
                points_at,
                points_len,
                width: stroke.width,
                stroke: PfRgb::from_colour(stroke.colour),
                ..PfCall::default()
            });
        }
    }

    fn polygon(&mut self, points: &[Pt], fill: Option<Colour>, stroke: Option<Stroke>) {
        let (points_at, points_len) = self.push_points(points);
        let mut flags = 0;
        if fill.is_some() {
            flags |= PF_HAS_FILL;
        }
        if stroke.is_some() {
            flags |= PF_HAS_STROKE;
        }
        self.calls.push(PfCall {
            kind: PF_KIND_POLYGON,
            flags,
            points_at,
            points_len,
            width: stroke.map_or(0.0, |stroke| stroke.width),
            line_class: stroke.map_or(0, |stroke| class_code(stroke.class)),
            stroke: rgb(stroke.map(|stroke| stroke.colour)),
            fill: rgb(fill),
            ..PfCall::default()
        });
    }

    fn rect(&mut self, rect: Rect, corner: f32, fill: Option<Colour>, stroke: Option<Stroke>) {
        let mut flags = 0;
        if fill.is_some() {
            flags |= PF_HAS_FILL;
        }
        if stroke.is_some() {
            flags |= PF_HAS_STROKE;
        }
        self.calls.push(PfCall {
            kind: PF_KIND_RECT,
            flags,
            rect: [rect.min.x, rect.min.y, rect.max.x, rect.max.y],
            corner,
            width: stroke.map_or(0.0, |stroke| stroke.width),
            line_class: stroke.map_or(0, |stroke| class_code(stroke.class)),
            stroke: rgb(stroke.map(|stroke| stroke.colour)),
            fill: rgb(fill),
            ..PfCall::default()
        });
    }

    fn circle(&mut self, centre: Pt, radius: f32, fill: Option<Colour>, stroke: Option<Stroke>) {
        let mut flags = 0;
        if fill.is_some() {
            flags |= PF_HAS_FILL;
        }
        if stroke.is_some() {
            flags |= PF_HAS_STROKE;
        }
        self.calls.push(PfCall {
            kind: PF_KIND_CIRCLE,
            flags,
            rect: [centre.x, centre.y, radius, 0.0],
            width: stroke.map_or(0.0, |stroke| stroke.width),
            line_class: stroke.map_or(0, |stroke| class_code(stroke.class)),
            stroke: rgb(stroke.map(|stroke| stroke.colour)),
            fill: rgb(fill),
            ..PfCall::default()
        });
    }

    fn arc(&mut self, centre: Pt, radius: f32, start: f32, end: f32, stroke: Stroke) {
        self.calls.push(PfCall {
            kind: PF_KIND_ARC,
            flags: PF_HAS_STROKE,
            rect: [centre.x, centre.y, radius, 0.0],
            angles: [start, end],
            width: stroke.width,
            line_class: class_code(stroke.class),
            stroke: PfRgb::from_colour(stroke.colour),
            ..PfCall::default()
        });
    }

    fn text(&mut self, at: Pt, text: &str, size: f32, colour: Colour, anchor: Anchor) {
        let text_at = self.text.len() as u32;
        self.text.extend_from_slice(text.as_bytes());
        self.calls.push(PfCall {
            kind: PF_KIND_TEXT,
            flags: PF_HAS_FILL,
            anchor: anchor_code(anchor),
            text_at,
            text_len: text.len() as u32,
            rect: [at.x, at.y, 0.0, 0.0],
            size,
            fill: PfRgb::from_colour(colour),
            ..PfCall::default()
        });
    }

    fn device_scale(&self) -> f32 {
        self.scale
    }
}

/// One simulation and one drawing, owned by the front end through an opaque pointer.
pub struct PfSession {
    catalog: Catalog,
    system: Option<Network>,
    ink: Ink,
    calls: Vec<PfCall>,
    points: Vec<PfPoint>,
    text: Vec<u8>,
    layout: Option<Layout>,
    document: Document,
    /// Part of the network each placement became, parallel to the document. `None` for a
    /// definition the solver cannot model, which the drawing then shows uncoloured.
    placement_part: Vec<Option<usize>>,
    /// Valve of the network each placement became, parallel to the document.
    placement_valve: Vec<Option<usize>>,
    /// The selection, in the order the user added to it; the last one is the primary.
    selected: Vec<usize>,
    /// Placements kept for paste, with their catalog ids and per-instance values.
    clipboard: Vec<Placement>,
    /// The pipe under the last selection, if any. A component and a pipe are never
    /// selected at the same time: Delete has to know which one it removes.
    selected_pipe: Option<usize>,
    /// `true` once the starting circuit has been placed. A flag, not "is the document
    /// empty": a component dropped before the first frame would otherwise suppress it.
    seeded: bool,
    /// Document revision at the moment it was last written to or read from a file, or
    /// `None` when what is on the sheet never came from a file and was never saved.
    ///
    /// The difference against `document.revision()` is what "there is work that is not in
    /// the file" means, and it costs one comparison instead of a comparison of every field
    /// of every placement.
    saved_revision: Option<u64>,
    /// Where the list of remembered documents lives, when the front end told us.
    ///
    /// The core does not choose a directory: a library that writes to a home it guessed is
    /// a library that surprises the user. Without a path the list simply does not exist.
    recent_path: Option<std::path::PathBuf>,
    /// The remembered documents, newest first.
    recent: RecentList,
    error: CString,
    width_mm: f32,
    height_mm: f32,
    scale: f32,
    /// Ids in display order, parallel to `catalog_items`.
    catalog_order: Vec<String>,
    catalog_items: Vec<PfCatalogItem>,
    catalog_text: Vec<u8>,
    preview_calls: Vec<PfCall>,
    preview_points: Vec<PfPoint>,
    preview_text: Vec<u8>,
    /// Samples of every bound recorder, by placement identity. A run artefact: the
    /// document stores the source, never the samples (ADR 0015).
    charts: std::collections::HashMap<u64, Chart>,
    /// Whether each contact is closed, by placement identity. A run state, like a valve
    /// position: the file says how the contact starts, the user clicks it, a reopened
    /// document starts at rest (ADR 0018).
    contacts: std::collections::HashMap<u64, bool>,
    /// Whether each relay contact is closed, by placement identity. Derived from the net
    /// on every control pass, never clicked: a relay moves because its coil is live
    /// (ADR 0019).
    relays: std::collections::HashMap<u64, bool>,
}

impl std::fmt::Debug for PfSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PfSession")
            .field("has_model", &self.system.is_some())
            .field("calls", &self.calls.len())
            .finish_non_exhaustive()
    }
}

/// The bounded window of samples one recorder keeps.
#[derive(Debug, Clone)]
struct Chart {
    /// Most samples kept, from the file's window and rate.
    capacity: usize,
    /// Seconds between samples.
    interval: f64,
    /// Simulation time at which the next sample is due.
    next_s: f64,
    /// Samples, oldest first, in the source's own unit.
    values: Vec<f32>,
}

impl Chart {
    /// A ring that keeps at most `capacity` samples, one every `interval` seconds.
    fn new(capacity: usize, interval: f64) -> Self {
        Self {
            capacity: capacity.max(2),
            interval: interval.max(f64::MIN_POSITIVE),
            next_s: 0.0,
            values: Vec::with_capacity(capacity.max(2)),
        }
    }

    /// Adds one sample, dropping the oldest when the window is full.
    fn push(&mut self, value: f32) {
        if self.values.len() >= self.capacity {
            self.values.remove(0);
        }
        self.values.push(value);
    }

    /// Lowest and highest sample in the window, or `None` when it is empty.
    fn span(&self) -> Option<(f32, f32)> {
        let first = *self.values.first()?;
        let mut min = first;
        let mut max = first;
        for value in &self.values {
            min = min.min(*value);
            max = max.max(*value);
        }
        Some((min, max))
    }
}

impl PfSession {
    fn new(user_dir: Option<&Path>) -> Self {
        let catalog = match user_dir {
            Some(dir) => Catalog::builtin().with_user_dir(dir),
            None => Catalog::builtin(),
        };
        let mut session = Self {
            catalog,
            system: None,
            ink: Ink::print(),
            calls: Vec::new(),
            points: Vec::new(),
            text: Vec::new(),
            layout: None,
            document: Document::new(),
            placement_part: Vec::new(),
            placement_valve: Vec::new(),
            selected: Vec::new(),
            clipboard: Vec::new(),
            selected_pipe: None,
            seeded: false,
            saved_revision: Some(0),
            recent_path: None,
            recent: RecentList::new(),
            error: CString::default(),
            width_mm: 200.0,
            height_mm: 130.0,
            scale: 1.0,
            catalog_order: Vec::new(),
            catalog_items: Vec::new(),
            catalog_text: Vec::new(),
            preview_calls: Vec::new(),
            preview_points: Vec::new(),
            preview_text: Vec::new(),
            charts: std::collections::HashMap::new(),
            contacts: std::collections::HashMap::new(),
            relays: std::collections::HashMap::new(),
        };
        session.build_catalog();
        // Seeded now, not on the first frame: a front end may place or connect before it
        // ever paints, and a document that is empty until the paint would refuse it.
        session.ensure_seeded();
        session
    }

    /// Places the starting circuit once, at the positions the fixed layout uses.
    fn ensure_seeded(&mut self) {
        if self.seeded {
            return;
        }
        self.seeded = true;
        let area = Rect::from_min_size(Pt::new(0.0, 0.0), self.width_mm, self.height_mm);
        self.seed_document(area);
        // The document is now the circuit: the model is built from it, never from a
        // hardcoded three-component list.
        self.rebuild_model();
    }

    /// The sidebar mirrors the catalog, so the list it shows comes from here.
    ///
    /// `Catalog::iter` walks a map, so the entries are sorted: a sidebar that reorders
    /// itself between launches is a bug the user sees.
    fn build_catalog(&mut self) {
        let mut entries: Vec<(u32, String, String)> = self
            .catalog
            .iter()
            .filter(|definition| !definition.disabled)
            .map(|definition| {
                (
                    category_code(definition.category),
                    definition.id.clone(),
                    definition.display_name().to_owned(),
                )
            })
            .collect();
        entries.sort();
        self.catalog_order = entries.iter().map(|(_, id, _)| id.clone()).collect();
        self.catalog_items.clear();
        self.catalog_text.clear();
        for (category, id, name) in entries {
            let id_at = self.catalog_text.len() as u32;
            self.catalog_text.extend_from_slice(id.as_bytes());
            let name_at = self.catalog_text.len() as u32;
            self.catalog_text.extend_from_slice(name.as_bytes());
            self.catalog_items.push(PfCatalogItem {
                id_at,
                id_len: id.len() as u32,
                name_at,
                name_len: name.len() as u32,
                category,
            });
        }
    }

    /// Draws one catalog entry alone, for the sidebar thumbnail.
    ///
    /// The symbol is the definition's own, drawn by the same code as the sheet: a
    /// separate icon set would be a second drawing that drifts.
    fn build_preview(&mut self, index: u32, width_mm: f32, height_mm: f32, scale: f32) -> i32 {
        let Some(id) = self.catalog_order.get(index as usize).cloned() else {
            return self.fail("that catalog entry does not exist");
        };
        if self.catalog.get(&id).is_none() {
            return self.fail("the catalog entry disappeared");
        }
        self.preview_calls.clear();
        self.preview_points.clear();
        self.preview_text.clear();
        let side = width_mm.min(height_mm) * 0.82;
        let rect = Rect::from_center_size(Pt::new(width_mm * 0.5, height_mm * 0.5), side, side);
        let Self {
            catalog,
            preview_calls,
            preview_points,
            preview_text,
            ink,
            ..
        } = self;
        let Some(definition) = catalog.get(&id) else {
            return PF_ERR_MODEL;
        };
        let mut buffer = FrameBuffer {
            calls: preview_calls,
            points: preview_points,
            text: preview_text,
            scale: scale.max(0.01),
        };
        punchi_fluid_draw::draw_component(&mut buffer, rect, definition, 0, 0.0, ink);
        PF_OK
    }

    fn fail(&mut self, message: &str) -> i32 {
        self.error = cstring(message);
        PF_ERR_MODEL
    }

    fn telemetry(&self) -> PfTelemetry {
        let Some(system) = self.system.as_ref() else {
            return PfTelemetry::default();
        };
        if system.cylinder().is_none() {
            return PfTelemetry::default();
        }
        let Telemetry {
            time,
            pressure_pa,
            position,
            velocity,
            valve_position,
            mass_in,
            mass_out,
        } = system.telemetry();
        PfTelemetry {
            time_s: time,
            pressure_pa,
            position_m: position,
            velocity_m_s: velocity,
            mass_in_kg: mass_in,
            mass_out_kg: mass_out,
            valve_position: valve_position as u32,
            has_model: 1,
            stroke_m: system.cylinder().map_or(0.0, |cylinder| cylinder.stroke),
        }
    }

    fn advance(&mut self, dt: f64) -> i32 {
        // The borrow of the model ends before the recorders are sampled: they read the
        // same network through `self`, and holding one mutable borrow across both would
        // be two borrows of one field.
        let stepped = match self.system.as_mut() {
            Some(system) => system
                .advance(dt)
                .map(|_| ())
                .map_err(|cause| cause.to_string()),
            None => return self.fail("no model is loaded"),
        };
        match stepped {
            Ok(()) => {
                self.sample_charts();
                PF_OK
            }
            Err(cause) => self.fail(&cause),
        }
    }

    /// The quantities a definition can report, in the order the interface lists them.
    fn quantities_of(
        definition: &punchi_fluid_catalog::ComponentDefinition,
    ) -> &'static [Quantity] {
        const PRESSURE: [Quantity; 1] = [Quantity::Pressure];
        const PRESSURE_FLOW: [Quantity; 2] = [Quantity::Pressure, Quantity::Flow];
        const PRESSURE_POSITION: [Quantity; 2] = [Quantity::Pressure, Quantity::Position];
        match definition.behaviour {
            punchi_fluid_catalog::Behaviour::FlowMeter { .. } => &PRESSURE_FLOW,
            punchi_fluid_catalog::Behaviour::Cylinder { .. } => &PRESSURE_POSITION,
            _ if !definition.port.is_empty() => &PRESSURE,
            _ => &[],
        }
    }

    /// The placements, and the quantities of each, a recorder in `circuit` can chart.
    ///
    /// The recorder itself is left out: it observes, it does not report.
    fn chart_candidates(&self, recorder: usize, circuit: u32) -> Vec<PfChartSource> {
        let mut candidates = Vec::new();
        for (index, placement) in self.document.placements().iter().enumerate() {
            if index == recorder || placement.circuit != circuit {
                continue;
            }
            let Some(definition) = self.catalog.get(&placement.catalog_id) else {
                continue;
            };
            #[allow(clippy::cast_possible_truncation)]
            let placement_id = placement.id() as u32;
            for quantity in Self::quantities_of(definition) {
                candidates.push(PfChartSource {
                    placement: placement_id,
                    quantity: quantity_index(*quantity),
                });
            }
        }
        candidates
    }

    /// The valve's definition as text, for the edit panel.
    ///
    /// The front end shows what the file says instead of a form it invented: name, id,
    /// ports, positions and how the return works all come from the catalog.
    fn describe_valve(&self) -> Option<String> {
        let system = self.system.as_ref()?;
        let valve = system.valve()?;
        let definition = self.catalog.get(&valve.definition_id)?;
        let mut out = String::new();
        out.push_str(&definition.name);
        out.push('\n');
        out.push_str(&definition.id);
        out.push('\n');
        if let Some(summary) = definition.summary.as_deref() {
            out.push_str(summary);
            out.push('\n');
        }
        out.push('\n');
        let _ = writeln!(out, "Categoría: {}", definition.category.as_str());
        let ports: Vec<&str> = definition
            .port
            .iter()
            .map(|port| port.id.as_str())
            .collect();
        let _ = writeln!(out, "Puertos: {}", ports.join(", "));
        let _ = writeln!(out, "Posiciones: {}", valve.position_names.join(", "));
        out.push_str(if valve.spring_return {
            "Retorno: muelle — solo queda accionada mientras se mantiene\n"
        } else {
            "Retorno: sin muelle — queda donde se deja\n"
        });
        Some(out)
    }

    /// Back to `t = 0`, with the same catalog. The ink survives, because it is the
    /// desktop's and not the model's.
    fn restart(&mut self) -> i32 {
        self.rebuild_model();
        if self.system.is_some() {
            self.error = CString::default();
            PF_OK
        } else {
            self.fail("no model can be built from this circuit")
        }
    }

    /// Places the clipboard at a centre and selects what arrived.
    ///
    /// The copies keep the arrangement they were copied with, so pasting a group of three
    /// gives back the group and not three corners of the sheet.
    fn paste_at(&mut self, copies: &[Placement], centre: Pt) -> i32 {
        let Some(anchor) = copies.first() else {
            return PF_ERR_MODEL;
        };
        let (dx, dy) = (centre.x - anchor.centre.x, centre.y - anchor.centre.y);
        let indices: Vec<usize> = copies
            .iter()
            .map(|copy| {
                let moved = Pt::new(copy.centre.x + dx, copy.centre.y + dy);
                self.document.place_copy(copy.clone(), Some(moved))
            })
            .collect();
        let primary = indices.last().copied();
        self.selected = indices;
        self.selected_pipe = None;
        self.rebuild_model();
        primary.map_or(PF_ERR_MODEL, |index| {
            i32::try_from(index).unwrap_or(PF_ERR_MODEL)
        })
    }

    /// Rebuilds the network from the document, which is the only thing that decides what
    /// the simulation runs. Called after every edit that changes the graph, never after a
    /// move: dragging a component leaves a model that is still running.
    fn rebuild_model(&mut self) {
        let mut parts: Vec<NetworkPart> = Vec::new();
        let mut part_of: Vec<Option<usize>> = vec![None; self.document.placements().len()];
        let mut valve_of: Vec<Option<usize>> = vec![None; self.document.placements().len()];
        let mut valves = 0;
        let circuit = self.document.active_circuit();
        for (index, placement) in self.document.placements().iter().enumerate() {
            if placement.circuit != circuit {
                continue;
            }
            let Some(definition) = self.catalog.get(&placement.catalog_id) else {
                continue;
            };
            let mut definition = definition.clone();
            apply_edits(&mut definition, &placement.edits);
            if let Ok(part) = NetworkPart::from_definition(&definition) {
                if matches!(part.behaviour, PartBehaviour::Valve { .. }) {
                    valve_of[index] = Some(valves);
                    valves += 1;
                }
                part_of[index] = Some(parts.len());
                parts.push(part);
            }
            // A definition the solver cannot model is left out; the circuit check is what
            // tells the user about it, not a silent half-model.
        }
        let mut pipes = Vec::new();
        for connection in self.document.connections() {
            if connection.circuit != circuit {
                continue;
            }
            let (Some(from), Some(to)) = (
                self.document.index_of(connection.from.placement),
                self.document.index_of(connection.to.placement),
            ) else {
                continue;
            };
            let (Some(from), Some(to)) = (part_of[from], part_of[to]) else {
                continue;
            };
            pipes.push((
                PortAddress::new(from, connection.from.port.clone()),
                PortAddress::new(to, connection.to.port.clone()),
            ));
        }
        self.placement_part = part_of;
        self.placement_valve = valve_of;
        match Network::build(&parts, &pipes) {
            Ok(network) if network.cylinder().is_some() => {
                self.system = Some(network);
                self.error = CString::default();
            }
            Ok(_) => {
                self.system = None;
                self.error = cstring("the circuit has no cylinder to move");
            }
            Err(cause) => {
                self.system = None;
                self.error = cstring(&cause.to_string());
            }
        }
        self.ensure_charts();
        self.apply_control();
    }

    /// Builds the sample ring of every bound recorder, keeping what a rebuild can keep.
    ///
    /// Called after an edit, not per frame: a source that did not change keeps its window,
    /// a recorder that was unbound or rebound starts empty, because its old samples belong
    /// to a quantity nobody is looking at any more.
    fn ensure_charts(&mut self) {
        let mut charts = std::collections::HashMap::with_capacity(self.charts.len());
        for placement in self.document.placements() {
            let Some(source) = placement.source else {
                continue;
            };
            let Some(definition) = self.catalog.get(&placement.catalog_id) else {
                continue;
            };
            let punchi_fluid_catalog::Behaviour::Recorder { span_s, rate_hz } =
                definition.behaviour
            else {
                continue;
            };
            // The catalog bounded the product already; the cast is inside `usize` by
            // construction and the clamp keeps the ring usable even for a hand-made file.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let capacity = ((span_s * rate_hz).round() as usize).clamp(2, 30_000);
            let interval = 1.0 / rate_hz.max(f64::MIN_POSITIVE);
            let existing = self.charts.remove(&placement.id());
            let mut chart = existing.unwrap_or_else(|| Chart::new(capacity, interval));
            chart.capacity = capacity;
            chart.interval = interval;
            if chart.values.len() > capacity {
                let extra = chart.values.len() - capacity;
                chart.values.drain(..extra);
            }
            let _ = source;
            charts.insert(placement.id(), chart);
        }
        self.charts = charts;
    }

    /// The value one recorder charts right now, in the source's own unit.
    fn chart_value(&self, recorder: u64) -> Option<f32> {
        let placement = self
            .document
            .placements()
            .iter()
            .find(|placement| placement.id() == recorder)?;
        let source = placement.source?;
        let index = self.document.index_of(source.placement)?;
        let target = self.document.placements().get(index)?;
        let definition = self.catalog.get(&target.catalog_id)?;
        let part = self.placement_part.get(index).copied().flatten()?;
        let network = self.system.as_ref()?;
        match source.quantity {
            punchi_fluid_doc::Quantity::Pressure => {
                let tap = definition.port.first()?;
                let pressure = network.port_pressure(part, &tap.id)?;
                #[allow(clippy::cast_possible_truncation)]
                let reading = ((pressure - network.atmosphere_pa()) / 1e5) as f32;
                Some(reading)
            }
            punchi_fluid_doc::Quantity::Flow => {
                #[allow(clippy::cast_possible_truncation)]
                let reading = network.part_flow_l_min(part)? as f32;
                Some(reading)
            }
            punchi_fluid_doc::Quantity::Position => {
                let punchi_fluid_catalog::Behaviour::Cylinder { stroke_mm, .. } =
                    definition.behaviour
                else {
                    return None;
                };
                #[allow(clippy::cast_possible_truncation)]
                let reading = (network.extension_of(part)? * stroke_mm) as f32;
                Some(reading)
            }
        }
    }

    /// Takes one sample for every recorder whose next sample is due.
    ///
    /// The map is moved out of `self` so the value can be read from the model without
    /// holding two borrows; nothing is allocated here, which is what keeps the advance
    /// step free of allocations.
    fn sample_charts(&mut self) {
        if self.charts.is_empty() {
            return;
        }
        let Some(now) = self.system.as_ref().map(Network::time) else {
            return;
        };
        let mut charts = std::mem::take(&mut self.charts);
        for (id, chart) in &mut charts {
            if now + f64::EPSILON < chart.next_s {
                continue;
            }
            if let Some(value) = self.chart_value(*id) {
                chart.push(value);
                chart.next_s = now + chart.interval;
            }
        }
        self.charts = charts;
    }

    /// Whether the contact at `index` is closed, starting from what its file declares.
    fn contact_closed_at(&mut self, index: usize) -> Option<bool> {
        let placement = self.document.placements().get(index)?;
        let id = placement.id();
        if let Some(closed) = self.contacts.get(&id) {
            return Some(*closed);
        }
        let definition = self.catalog.get(&placement.catalog_id)?;
        let punchi_fluid_catalog::Behaviour::Contact { normally_closed } = definition.behaviour
        else {
            return None;
        };
        self.contacts.insert(id, normally_closed);
        Some(normally_closed)
    }

    /// Closes or opens a contact, and lets the control circuit follow.
    fn toggle_contact(&mut self, index: usize) -> i32 {
        let Some(closed) = self.contact_closed_at(index) else {
            return self.fail("that component is not a contact");
        };
        let Some(placement) = self.document.placements().get(index) else {
            return self.fail("that component is not in the document");
        };
        let id = placement.id();
        self.contacts.insert(id, !closed);
        self.apply_control();
        PF_OK
    }

    /// Energises the control circuit and positions the solenoid valves.
    ///
    /// The net is boolean: a rail marks its positive terminal live, liveness travels through
    /// wires and closed contacts, and a solenoid valve whose terminal is live sits in its
    /// actuated position. No voltage, no current, no short circuit (ADR 0018).
    fn apply_control(&mut self) {
        let circuit = self.document.active_circuit();
        let user_closed = self.closed_user_contacts(circuit);
        let relays = self.relay_bindings(circuit);
        let state = settle_relays(
            &self.document,
            &self.catalog,
            circuit,
            &user_closed,
            &relays,
        );
        self.publish_relay_state(&relays, &state);
        self.position_solenoids(circuit, &user_closed, &relays, &state);
    }

    /// The clicked contacts that are closed, as the terminal pairs the net joins.
    fn closed_user_contacts(&mut self, circuit: u32) -> Vec<(usize, String, String)> {
        // The states are read first: reading one may write the map, which cannot happen
        // while the document is borrowed by a loop.
        let candidates: Vec<usize> = self
            .document
            .placements()
            .iter()
            .enumerate()
            .filter(|(_, placement)| placement.circuit == circuit)
            .map(|(index, _)| index)
            .collect();
        let mut closed = Vec::new();
        for index in candidates {
            if !self.contact_closed_at(index).unwrap_or(false) {
                continue;
            }
            let Some(placement) = self.document.placements().get(index) else {
                continue;
            };
            let Some(definition) = self.catalog.get(&placement.catalog_id) else {
                continue;
            };
            if let Some((a, b)) = contact_pair(definition) {
                closed.push((index, a, b));
            }
        }
        closed
    }

    /// The relays of one circuit, in document order.
    fn relay_bindings(&self, circuit: u32) -> Vec<RelayBinding> {
        let mut relays = Vec::new();
        for (index, placement) in self.document.placements().iter().enumerate() {
            if placement.circuit != circuit {
                continue;
            }
            let Some(definition) = self.catalog.get(&placement.catalog_id) else {
                continue;
            };
            if let Behaviour::Relay {
                coil,
                contact,
                normally_closed,
            } = &definition.behaviour
            {
                relays.push((index, coil.clone(), contact.clone(), *normally_closed));
            }
        }
        relays
    }

    /// Keeps the settled state for the frame; a relay is never clicked by hand.
    fn publish_relay_state(&mut self, relays: &[RelayBinding], state: &[bool]) {
        let mut settled = Vec::with_capacity(relays.len());
        for ((index, _, _, _), closed) in relays.iter().zip(state) {
            if let Some(placement) = self.document.placements().get(*index) {
                settled.push((placement.id(), *closed));
            }
        }
        self.relays = settled.into_iter().collect();
    }

    /// A solenoid valve follows its coil in the net the relays settled on.
    fn position_solenoids(
        &mut self,
        circuit: u32,
        user_closed: &[(usize, String, String)],
        relays: &[RelayBinding],
        state: &[bool],
    ) {
        let net = control_net(
            &self.document,
            &self.catalog,
            circuit,
            &closed_pairs(user_closed, relays, state),
        );
        let live = net.energised();
        for (index, placement) in self.document.placements().iter().enumerate() {
            if placement.circuit != circuit {
                continue;
            }
            let Some(definition) = self.catalog.get(&placement.catalog_id) else {
                continue;
            };
            let punchi_fluid_catalog::Behaviour::Valve {
                default_position, ..
            } = &definition.behaviour
            else {
                continue;
            };
            if definition.actuator.map(|actuator| actuator.kind)
                != Some(punchi_fluid_catalog::ActuatorKind::Solenoid)
            {
                continue;
            }
            let Some(coil) = definition
                .port
                .iter()
                .find(|port| port.role == punchi_fluid_catalog::PortRole::Electrical)
            else {
                continue;
            };
            let Some(node) = net.node_of(index, &coil.id) else {
                continue;
            };
            let position = if live[node] {
                1 - *default_position
            } else {
                *default_position
            };
            // The setter names the valve; the getter names the part it belongs to.
            let Some(valve) = self.placement_valve.get(index).copied().flatten() else {
                continue;
            };
            if let Some(system) = self.system.as_mut() {
                system.set_valve_position_at(valve, position);
            }
        }
    }

    /// Builds the frame the front end draws.    /// Builds the frame the front end draws.
    fn build_frame(&mut self, width_mm: f32, height_mm: f32, scale: f32) {
        self.width_mm = width_mm.max(1.0);
        self.height_mm = height_mm.max(1.0);
        self.scale = scale.max(0.01);
        self.calls.clear();
        self.points.clear();
        self.text.clear();
        self.ensure_seeded();
        let Self {
            catalog,
            document,
            placement_part,
            system,
            ink,
            calls,
            points,
            text,
            layout,
            scale,
            selected,
            selected_pipe,
            charts,
            contacts,
            relays,
            ..
        } = self;

        let circuit = document.active_circuit();
        let (placed, pipes) = scene(
            document,
            catalog,
            system.as_ref(),
            placement_part,
            circuit,
            charts,
            ContactStates { contacts, relays },
        );
        // The boxes the hit test and the pipes use are the document's, not a layout the
        // front end guessed.
        *layout = layout_of(&placed);

        let mut buffer = FrameBuffer {
            calls,
            points,
            text,
            scale: *scale,
        };
        draw_placed(
            &mut buffer,
            &placed,
            &pipes,
            catalog,
            ink,
            selected.as_slice(),
        );
        if let Some(pipe) = *selected_pipe {
            draw_selected_pipe(&mut buffer, document, catalog, ink, pipe);
        }
    }

    /// The document as the sheet format stores it.
    fn to_sheet(&self) -> Sheet {
        let placement = self
            .document
            .placements()
            .iter()
            .map(|placement| {
                let edits = &placement.edits;
                let edit = if edits.is_empty() {
                    None
                } else {
                    Some(StoredEdits {
                        pressure_bar: edits.pressure_bar,
                        conductance: edits.conductance,
                        stroke_mm: edits.stroke_mm,
                        bore_mm: edits.bore_mm,
                        rod_mm: edits.rod_mm,
                        moving_mass_kg: edits.moving_mass_kg,
                    })
                };
                // Tenths of a millimetre are the precision the sheet draws at; storing
                // the raw `f32` only writes the layout's arithmetic noise into the file.
                let round = |value: f32| (value * 10.0).round() / 10.0;
                StoredPlacement {
                    id: placement.id(),
                    circuit: placement.circuit,
                    catalog: placement.catalog_id.clone(),
                    x: round(placement.centre.x),
                    y: round(placement.centre.y),
                    width: round(placement.width),
                    height: round(placement.height),
                    edit,
                    quarter_turns: placement.orientation.quarter_turns(),
                    mirrored: placement.orientation.mirrored(),
                    designation: placement.designation.clone(),
                    source: placement
                        .source
                        .map(|source| punchi_fluid_file::sheet::StoredSource {
                            placement: source.placement,
                            quantity: source.quantity.as_str().to_owned(),
                        }),
                }
            })
            .collect();
        let pipe = self
            .document
            .connections()
            .iter()
            .map(|connection| StoredPipe {
                circuit: connection.circuit,
                from: connection.from.placement,
                from_port: connection.from.port.clone(),
                to: connection.to.placement,
                to_port: connection.to.port.clone(),
                offset_tenths: connection.offset_tenths(),
                class: match connection.class() {
                    LineClass::Pilot => punchi_fluid_file::sheet::StoredClass::Pilot,
                    LineClass::Drain => punchi_fluid_file::sheet::StoredClass::Drain,
                    // The file has no electrical class yet, so a line the format cannot
                    // name is written as working until F6 gives it one.
                    LineClass::Working | LineClass::Electrical => {
                        punchi_fluid_file::sheet::StoredClass::Working
                    }
                },
            })
            .collect();
        let circuits = self
            .document
            .circuits()
            .iter()
            .map(|circuit| punchi_fluid_file::sheet::StoredCircuit {
                id: circuit.id,
                title: circuit.title.clone(),
            })
            .collect();
        Sheet {
            format: punchi_fluid_file::sheet::SHEET_VERSION,
            title: String::from("Circuito"),
            circuits,
            placement,
            pipe,
            active: self.document.active_circuit(),
        }
    }

    /// Replaces the document with a parsed sheet, leaving the old one untouched on error.
    fn load(&mut self, sheet: &Sheet) -> Result<(), String> {
        let placements: Vec<Placement> = sheet
            .placement
            .iter()
            .map(|stored| {
                let mut placement = Placement::with_id(
                    stored.id,
                    stored.catalog.clone(),
                    Pt::new(stored.x, stored.y),
                    stored.width,
                    stored.height,
                );
                placement.circuit = stored.circuit;
                if let Some(edit) = stored.edit {
                    placement.edits = PlacementEdits {
                        pressure_bar: edit.pressure_bar,
                        conductance: edit.conductance,
                        stroke_mm: edit.stroke_mm,
                        bore_mm: edit.bore_mm,
                        rod_mm: edit.rod_mm,
                        moving_mass_kg: edit.moving_mass_kg,
                    };
                }
                placement.orientation = Orientation::new(stored.quarter_turns, stored.mirrored);
                placement.source = stored.source.as_ref().and_then(|source| {
                    let quantity = punchi_fluid_doc::Quantity::from_name(&source.quantity)?;
                    Some(punchi_fluid_doc::Source::new(source.placement, quantity))
                });
                placement.designation.clone_from(&stored.designation);
                placement
            })
            .collect();
        let mut circuits: Vec<Circuit> = sheet
            .circuits
            .iter()
            .map(|circuit| Circuit::new(circuit.id, circuit.title.clone()))
            .collect();
        if circuits.is_empty() {
            circuits.push(Circuit::new(0, "Circuito 1"));
        }
        let active = if circuits.iter().any(|circuit| circuit.id == sheet.active) {
            sheet.active
        } else {
            circuits[0].id
        };
        let mut document = Document::new();
        if !document.set_project(circuits, active) {
            return Err(String::from("the project has no usable circuit"));
        }
        document.seed(placements);
        for pipe in &sheet.pipe {
            if !document.seed_connect(
                Endpoint::new(pipe.from, pipe.from_port.clone()),
                Endpoint::new(pipe.to, pipe.to_port.clone()),
            ) {
                return Err(format!(
                    "the pipe {}:{} -> {}:{} cannot be connected",
                    pipe.from, pipe.from_port, pipe.to, pipe.to_port
                ));
            }
            let index = document.connections().len() - 1;
            document.seed_pipe_circuit(index, pipe.circuit);
            document.seed_route(index, pipe.offset_tenths);
            document.seed_pipe_class(
                index,
                match pipe.class {
                    punchi_fluid_file::sheet::StoredClass::Working => LineClass::Working,
                    punchi_fluid_file::sheet::StoredClass::Pilot => LineClass::Pilot,
                    punchi_fluid_file::sheet::StoredClass::Drain => LineClass::Drain,
                },
            );
        }
        self.document = document;
        self.seeded = true;
        self.saved_revision = Some(self.document.revision());
        self.select_only(None);
        self.clipboard.clear();
        self.rebuild_model();
        Ok(())
    }

    /// Every fault the design check reports, against the sheet as it is now.
    ///
    /// Recomputed on demand rather than cached: a check is a handful of comparisons, and a
    /// cached list is a list that goes stale behind an edit.
    fn findings(&self) -> Vec<punchi_fluid_check::Finding> {
        let sheet = Rect::from_min_size(Pt::new(0.0, 0.0), self.width_mm, self.height_mm);
        check(
            &self.document,
            &self.catalog,
            sheet,
            self.document.active_circuit(),
        )
    }

    /// The pipe nearest to `point`, if one runs close enough.
    ///
    /// The distance is measured to the routed polyline, not to the straight line between
    /// the ports, so what the user sees is what answers.
    fn pipe_at(&self, point: Pt, tolerance: f32) -> Option<usize> {
        let mut best: Option<(f32, usize)> = None;
        let circuit = self.document.active_circuit();
        for (index, connection) in self.document.connections().iter().enumerate() {
            if connection.circuit != circuit {
                continue;
            }
            let Some(route) = pipe_route(&self.document, &self.catalog, index) else {
                continue;
            };
            let distance = distance_to_path(point, &route);
            if distance <= tolerance && best.is_none_or(|(closest, _)| distance < closest) {
                best = Some((distance, index));
            }
        }
        best.map(|(_, index)| index)
    }

    /// Moves the middle segment of a pipe so it passes through `point`.
    ///
    /// The gesture is the reference's: drag a hose until it reads clearly. The offset is
    /// measured from the automatic route along the axis the elbow jogs in, so a drag
    /// perpendicular to the middle segment does nothing surprising when the components
    /// later move.
    fn route_pipe(&mut self, index: usize, point: Pt) -> i32 {
        let Some((start, end)) = pipe_points(&self.document, &self.catalog, index) else {
            return self.fail("that pipe does not exist");
        };
        if pipe_is_elbow(&self.document, &self.catalog, index) {
            // One bend has no middle segment: the route is already the shortest orthogonal
            // path between the two edges, and a drag has nothing to move.
            return self.fail("this hose has one bend and no segment to drag");
        }
        let horizontal = (end.x - start.x).abs() >= (end.y - start.y).abs();
        let offset = if horizontal {
            point.x - f32::midpoint(start.x, end.x)
        } else {
            point.y - f32::midpoint(start.y, end.y)
        };
        if self.document.route_to(index, offset) {
            PF_OK
        } else {
            // The point is already where the segment is: not an error, and not an edit.
            PF_OK
        }
    }

    /// The port nearest to `point`, if any is close enough to be meant.
    ///
    /// The tolerance is in millimetres of the sheet, so it shrinks with the zoom in
    /// screen terms and stays a physical distance in the drawing.
    fn port_at(&self, point: Pt, tolerance: f32) -> Option<(usize, &str)> {
        let mut best: Option<(f32, usize, &str)> = None;
        for (index, placement) in self.document.placements().iter().enumerate() {
            let Some(definition) = self.catalog.get(&placement.catalog_id) else {
                continue;
            };
            for port in &definition.port {
                let Some(at) = port_point(
                    placement.rect(),
                    placement.orientation,
                    definition,
                    &port.id,
                ) else {
                    continue;
                };
                let distance = Pt::new(point.x - at.x, point.y - at.y).length();
                if distance > tolerance {
                    continue;
                }
                if best.is_none_or(|(closest, _, _)| distance < closest) {
                    best = Some((distance, index, port.id.as_str()));
                }
            }
        }
        best.map(|(_, index, port)| (index, port))
    }

    /// Adds a document to the list of remembered ones and writes the list.
    ///
    /// The write is best effort on purpose: failing to remember a file must not fail the
    /// save the user asked for.
    fn remember_recent(&mut self, path: &str) {
        let Some(list_path) = self.recent_path.clone() else {
            return;
        };
        if !self.recent.remember(path) {
            return;
        }
        let _ = self.recent.write(&list_path);
    }

    /// `true` when the document holds work that is not in the file it came from.
    fn is_modified(&self) -> bool {
        self.saved_revision != Some(self.document.revision())
    }

    /// The component the properties form and the transport talk about: the last touched.
    fn primary(&self) -> Option<usize> {
        self.selected.last().copied()
    }

    /// Replaces the selection with one component, or with nothing.
    fn select_only(&mut self, index: Option<usize>) {
        self.selected.clear();
        if let Some(index) = index {
            self.selected.push(index);
        }
        self.selected_pipe = None;
    }

    /// Adds the component to the selection, or takes it out: `Mayús` and a click.
    fn toggle_selected(&mut self, index: usize) {
        if let Some(position) = self
            .selected
            .iter()
            .position(|candidate| *candidate == index)
        {
            self.selected.remove(position);
        } else if index < self.document.len() {
            self.selected.push(index);
        }
        self.selected_pipe = None;
    }

    /// Keeps the selection inside the document after an undo removed its component.
    fn clamp_selection(&mut self) {
        let len = self.document.len();
        self.selected.retain(|index| *index < len);
        if self
            .selected_pipe
            .is_some_and(|index| index >= self.document.connections().len())
        {
            self.selected_pipe = None;
        }
    }

    /// The sheet starts with the circuit the model simulates, at the positions the fixed
    /// layout uses, so the picture the user already knows is the starting document.
    fn seed_document(&mut self, area: Rect) {
        let Some(layout) = punchi_fluid_draw::layout(area) else {
            return;
        };
        // The unit is seeded last on purpose: the order of this list is the index the ABI
        // hands out, and the worked example's first three are the ones the front end has
        // always addressed. Where a component sits comes from the layout, not from here.
        let seeded: Vec<punchi_fluid_doc::Placement> = [
            (punchi_fluid_draw::schematic::SUPPLY_ID, layout.supply),
            (punchi_fluid_draw::schematic::VALVE_ID, layout.valve),
            (punchi_fluid_draw::schematic::CYLINDER_ID, layout.cylinder),
            (
                punchi_fluid_draw::schematic::PREPARATION_ID,
                layout.preparation,
            ),
        ]
        .into_iter()
        .map(|(id, rect)| {
            punchi_fluid_doc::Placement::new(id, rect.center(), rect.width(), rect.height())
        })
        .collect();
        // A component dropped before the first frame stays; the starting circuit is added
        // around it rather than replacing it.
        let mut all = self.document.placements().to_vec();
        all.extend(seeded);
        self.select_only(None);
        self.document.seed(all);
        // The starting circuit is drawn with its hoses. Three pieces and no pipe does not
        // read as a circuit, and the checker would report two open pneumatic connections on
        // a sheet that is meant to be the worked example.
        // The compressor feeds the service unit and the unit feeds the valve: an air line
        // without the unit is wet air at a pressure nobody set.
        self.connect_seeded(
            (punchi_fluid_draw::schematic::SUPPLY_ID, "1"),
            (punchi_fluid_draw::schematic::PREPARATION_ID, "1"),
        );
        self.connect_seeded(
            (punchi_fluid_draw::schematic::PREPARATION_ID, "2"),
            (punchi_fluid_draw::schematic::VALVE_ID, "1"),
        );
        self.connect_seeded(
            (punchi_fluid_draw::schematic::VALVE_ID, "2"),
            (punchi_fluid_draw::schematic::CYLINDER_ID, "1"),
        );
        // The worked example is what the window opens with, not work the user typed: the
        // title has nothing to mark and closing asks nothing.
        self.saved_revision = Some(self.document.revision());
    }

    /// Joins two seeded ports by definition id, doing nothing when either is absent: the
    /// starting circuit must survive a catalog that lost a definition, not abort.
    fn connect_seeded(&mut self, from: (&str, &str), to: (&str, &str)) {
        let find = |document: &punchi_fluid_doc::Document, id: &str| {
            document
                .placements()
                .iter()
                .find(|placement| placement.catalog_id == id)
                .map(punchi_fluid_doc::Placement::id)
        };
        let (Some(from_id), Some(to_id)) =
            (find(&self.document, from.0), find(&self.document, to.0))
        else {
            return;
        };
        let _ = self.document.seed_connect(
            punchi_fluid_doc::Endpoint::new(from_id, from.1),
            punchi_fluid_doc::Endpoint::new(to_id, to.1),
        );
    }

    /// Hit test against the layout of the last built frame, so the click target is
    /// exactly what the user sees rather than a second recomputation.
    fn hit_valve(&self, x: f32, y: f32) -> bool {
        self.layout
            .is_some_and(|layout| layout.valve.expand(2.0).contains(Pt::new(x, y)))
    }
}

fn cstring(message: &str) -> CString {
    CString::new(message.replace('\0', " ")).unwrap_or_default()
}

/// The ABI number of a quantity.
fn quantity_index(quantity: Quantity) -> u32 {
    match quantity {
        Quantity::Pressure => PF_QUANTITY_PRESSURE,
        Quantity::Flow => PF_QUANTITY_FLOW,
        Quantity::Position => PF_QUANTITY_POSITION,
    }
}

/// The quantity an ABI number stands for, or `None` when the number is not one.
fn quantity_from_index(index: u32) -> Option<Quantity> {
    match index {
        PF_QUANTITY_PRESSURE => Some(Quantity::Pressure),
        PF_QUANTITY_FLOW => Some(Quantity::Flow),
        PF_QUANTITY_POSITION => Some(Quantity::Position),
        _ => None,
    }
}

/// A caller-supplied circuit title, trimmed and bounded, or `None` when it is unusable.
fn valid_circuit_title(title: *const c_char) -> Option<String> {
    if title.is_null() {
        return None;
    }
    // SAFETY: the caller promised a NUL-terminated string.
    let text = unsafe { CStr::from_ptr(title) }.to_str().ok()?;
    if text.is_empty()
        || text.trim() != text
        || text.chars().count() > punchi_fluid_file::sheet::MAX_CIRCUIT_TITLE
        || text.chars().any(char::is_control)
    {
        return None;
    }
    Some(text.to_owned())
}

fn guard(body: impl FnOnce() -> i32) -> i32 {
    catch_unwind(AssertUnwindSafe(body)).unwrap_or(PF_ERR_PANIC)
}

/// # Safety
///
/// `session` must be null or a pointer returned by [`pf_session_new`] that has not been
/// freed. The returned reference may not outlive that session.
unsafe fn borrow<'a>(session: *mut PfSession) -> Option<&'a mut PfSession> {
    // SAFETY: the caller upholds the contract above.
    unsafe { session.as_mut() }
}

/// # Safety
///
/// `session` must be null or a pointer returned by [`pf_session_new`] that has not been
/// freed. The returned reference may not outlive that session.
unsafe fn borrow_shared<'a>(session: *const PfSession) -> Option<&'a PfSession> {
    // SAFETY: the caller upholds the contract above.
    unsafe { session.as_ref() }
}

/// Library version as a NUL-terminated string. Never null, never freed.
#[unsafe(no_mangle)]
pub extern "C" fn pf_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr().cast()
}

/// Fills `out` with the layout of the structures this build produces.
///
/// # Safety
///
/// `out` must be null or point to a writable [`PfAbi`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_abi(out: *mut PfAbi) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a writable structure.
        let Some(out) = (unsafe { out.as_mut() }) else {
            return PF_ERR_NULL;
        };
        *out = PfAbi {
            version: PF_ABI_VERSION,
            call_size: std::mem::size_of::<PfCall>() as u32,
            telemetry_size: std::mem::size_of::<PfTelemetry>() as u32,
            frame_size: std::mem::size_of::<PfFrame>() as u32,
            ink_size: std::mem::size_of::<PfInk>() as u32,
            point_size: std::mem::size_of::<PfPoint>() as u32,
        };
        PF_OK
    })
}

/// Creates a session, optionally over a directory of user catalog files.
///
/// Returns null on failure. Never null for a valid call: a catalog that yields no model
/// still produces a session whose `has_model` is `0` and whose error can be read.
///
/// # Safety
///
/// `user_catalog_dir` must be null or a valid NUL-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_new(user_catalog_dir: *const c_char) -> *mut PfSession {
    catch_unwind(AssertUnwindSafe(|| {
        let dir = if user_catalog_dir.is_null() {
            None
        } else {
            // SAFETY: the caller promises a NUL-terminated string.
            let text = unsafe { CStr::from_ptr(user_catalog_dir) };
            text.to_str()
                .ok()
                .filter(|text| !text.is_empty())
                .map(PathBuf::from)
        };
        Box::into_raw(Box::new(PfSession::new(dir.as_deref())))
    }))
    .unwrap_or(std::ptr::null_mut())
}

/// Frees a session. A null pointer is ignored.
///
/// # Safety
///
/// `session` must be null or a pointer returned by [`pf_session_new`], and it must not
/// be used again afterwards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_free(session: *mut PfSession) {
    if session.is_null() {
        return;
    }
    // SAFETY: the caller promises ownership of a pointer from `pf_session_new`.
    drop(unsafe { Box::from_raw(session) });
}

/// Last error message of the session, NUL-terminated. Valid until the next failing call.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_last_error(session: *const PfSession) -> *const c_char {
    // SAFETY: the caller promises a live session pointer.
    unsafe { borrow_shared(session) }.map_or(std::ptr::null(), |session| session.error.as_ptr())
}

/// Reads the state of the model.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_telemetry(
    session: *const PfSession,
    out: *mut PfTelemetry,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds both contracts.
        let (Some(session), Some(out)) =
            (unsafe { borrow_shared(session) }, unsafe { out.as_mut() })
        else {
            return PF_ERR_NULL;
        };
        *out = session.telemetry();
        PF_OK
    })
}

/// Replaces the ink. The front end fills it from the desktop theme.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `ink` must be null or readable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_set_ink(session: *mut PfSession, ink: *const PfInk) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds both contracts.
        let (Some(session), Some(ink)) = (unsafe { borrow(session) }, unsafe { ink.as_ref() })
        else {
            return PF_ERR_NULL;
        };
        session.ink = ink.to_ink();
        PF_OK
    })
}

/// Advances the model by `dt` seconds.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_advance(session: *mut PfSession, dt: f64) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        session.advance(dt)
    })
}

/// Presses the valve of one placement, as the pointer does.
///
/// A circuit can hold more than one valve, and the reference actuates the one under the
/// pointer. This is that valve, by placement, not "the first one in the model".
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_press_component_valve(
    session: *mut PfSession,
    index: u32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        let Some(Some(valve)) = session.placement_valve.get(index as usize).copied() else {
            return session.fail("that component is not a valve");
        };
        let Some(system) = session.system.as_mut() else {
            return session.fail("no model is loaded");
        };
        system.press_valve_at(valve);
        PF_OK
    })
}

/// Releases the valve of one placement, letting a spring return it.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_release_component_valve(
    session: *mut PfSession,
    index: u32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        let Some(Some(valve)) = session.placement_valve.get(index as usize).copied() else {
            return session.fail("that component is not a valve");
        };
        let Some(system) = session.system.as_mut() else {
            return session.fail("no model is loaded");
        };
        system.release_valve_at(valve);
        PF_OK
    })
}

/// Active position of the valve of one placement, or `-1` when it is not a valve.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_component_valve_position(
    session: *const PfSession,
    index: u32,
) -> i32 {
    // SAFETY: the caller promises a live session pointer.
    let Some(session) = (unsafe { borrow_shared(session) }) else {
        return PF_ERR_NULL;
    };
    let Some(Some(part)) = session.placement_part.get(index as usize).copied() else {
        return -1;
    };
    let Some(system) = session.system.as_ref() else {
        return -1;
    };
    system
        .valve_position_at(part)
        .map_or(-1, |position| i32::try_from(position).unwrap_or(-1))
}

/// `1` when the valve of that placement returns on release, `0` when it latches, `-1`
/// when the placement is not a valve.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_component_valve_spring_return(
    session: *const PfSession,
    index: u32,
) -> i32 {
    // SAFETY: the caller promises a live session pointer.
    let Some(session) = (unsafe { borrow_shared(session) }) else {
        return PF_ERR_NULL;
    };
    let Some(Some(part)) = session.placement_part.get(index as usize).copied() else {
        return -1;
    };
    let Some(system) = session.system.as_ref() else {
        return -1;
    };
    system.valve_spring_return_at(part).map_or(-1, i32::from)
}

/// Moves the valve to `position`. Out-of-range positions are refused, not clamped.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_set_valve(session: *mut PfSession, position: u32) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if session.system.is_none() {
            return session.fail("no model is loaded");
        }
        let Some(system) = session.system.as_mut() else {
            return PF_ERR_MODEL;
        };
        if system.set_valve_position(position as usize) {
            PF_OK
        } else {
            session.fail("that valve position does not exist")
        }
    })
}

/// Toggles the valve between its positions.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_toggle_valve(session: *mut PfSession) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if session.system.is_none() {
            return session.fail("no model is loaded");
        }
        let Some(system) = session.system.as_mut() else {
            return PF_ERR_MODEL;
        };
        if system.toggle_valve() {
            PF_OK
        } else {
            session.fail("the valve has a single position")
        }
    })
}

/// Holds the valve actuated, the press half of a lever held down.
///
/// A spring-return valve is only actuated while the gesture lasts, so the front end holds
/// this until the pointer or key is released and then calls
/// [`pf_session_release_valve`].
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_press_valve(session: *mut PfSession) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if session.system.is_none() {
            return session.fail("no model is loaded");
        }
        let Some(system) = session.system.as_mut() else {
            return PF_ERR_MODEL;
        };
        system.press_valve();
        PF_OK
    })
}

/// The release half: the spring returns the spool, a latched valve stays where it is.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_release_valve(session: *mut PfSession) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if session.system.is_none() {
            return session.fail("no model is loaded");
        }
        let Some(system) = session.system.as_mut() else {
            return PF_ERR_MODEL;
        };
        system.release_valve();
        PF_OK
    })
}

/// `1` when the valve is held actuated by the gesture and returns on release.
///
/// The front end needs this to choose between press-and-hold and click-to-latch; guessing
/// would make a lever valve stay actuated after a click, which it does not.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_valve_spring_return(session: *const PfSession) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return 0;
        };
        let Some(system) = session.system.as_ref() else {
            return 0;
        };
        i32::from(system.valve().is_some_and(|valve| valve.spring_return))
    })
}

/// One placement's definition and effective parameters, as text for the inspector.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `out` must be null or point to
/// `capacity` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_describe_component(
    session: *const PfSession,
    index: u32,
    out: *mut c_char,
    capacity: u32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds both contracts.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return PF_ERR_NULL;
        };
        if out.is_null() || capacity == 0 {
            return PF_ERR_NULL;
        }
        let Some(placement) = session.document.placements().get(index as usize) else {
            return PF_ERR_MODEL;
        };
        let Some(definition) = session.catalog.get(&placement.catalog_id) else {
            return PF_ERR_MODEL;
        };
        let mut definition = definition.clone();
        apply_edits(&mut definition, &placement.edits);
        let mut text = String::new();
        text.push_str(definition.display_name());
        text.push('\n');
        text.push_str(&definition.id);
        text.push('\n');
        text.push('\n');
        let _ = writeln!(text, "Categoría: {}", category_es(definition.category));
        let ports: Vec<&str> = definition
            .port
            .iter()
            .map(|port| port.id.as_str())
            .collect();
        let _ = writeln!(text, "Puertos: {}", ports.join(", "));
        describe_behaviour(&mut text, &definition.behaviour);

        let bytes = text.as_bytes();
        let capacity = capacity as usize;
        if bytes.len() >= capacity {
            return PF_ERR_MODEL;
        }
        // SAFETY: the caller promised `capacity` writable bytes and the buffer is large
        // enough for the text plus the terminator.
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr().cast::<c_char>(), out, bytes.len());
            out.add(bytes.len()).write(0);
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let written = bytes.len() as i32;
        written
    })
}

/// Writes what the behaviour means, for the inspector.
fn describe_behaviour(text: &mut String, behaviour: &punchi_fluid_catalog::Behaviour) {
    match behaviour {
        Behaviour::Supply { pressure_bar } => {
            let _ = writeln!(text, "Presión: {pressure_bar:.2} bar");
        }
        Behaviour::Valve {
            positions,
            spring_return,
            actuation,
            ..
        } => {
            let names: Vec<&str> = positions
                .iter()
                .map(punchi_fluid_catalog::ValvePosition::display_name)
                .collect();
            let _ = writeln!(text, "Posiciones: {}", names.join(", "));
            let _ = writeln!(text, "Accionamiento: {}", actuation_es(*actuation));
            text.push_str(if *spring_return {
                "Retorno: muelle — solo queda accionada mientras se mantiene\n"
            } else {
                "Retorno: sin muelle — queda donde se deja\n"
            });
            if let Some(connection) = positions
                .first()
                .and_then(|position| position.connection.first())
            {
                let _ = writeln!(
                    text,
                    "Conductancia: {:.2e} m³/(s·Pa)",
                    connection.conductance
                );
            }
        }
        Behaviour::Cylinder {
            bore_mm,
            rod_mm,
            stroke_mm,
            moving_mass_kg,
            ..
        } => {
            let _ = writeln!(text, "Carrera: {stroke_mm:.1} mm");
            let _ = writeln!(text, "Émbolo: {bore_mm:.1} mm");
            let _ = writeln!(text, "Vástago: {rod_mm:.1} mm");
            let _ = writeln!(text, "Masa móvil: {moving_mass_kg:.2} kg");
        }
        Behaviour::Gauge { min_bar, max_bar } => {
            let _ = writeln!(text, "Escala: {min_bar:.1} … {max_bar:.1} bar");
            text.push_str("Lee la presión del puerto al que se conecta\n");
        }
        Behaviour::Throttle { conductance } => {
            let _ = writeln!(text, "Conductancia: {conductance:.2e} m³/(s·Pa)");
        }
        Behaviour::FlowMeter {
            conductance,
            nominal_l_min,
        } => {
            let _ = writeln!(text, "Escala: 0 … {nominal_l_min:.0} l/min");
            let _ = writeln!(text, "Conductancia: {conductance:.2e} m³/(s·Pa)");
            text.push_str("Lee el caudal que lo atraviesa\n");
        }
        Behaviour::Silencer { conductance } => {
            let _ = writeln!(
                text,
                "Conductancia al ambiente: {conductance:.2e} m³/(s·Pa)"
            );
            text.push_str("Ventila el escape sin modelar el ruido\n");
        }
        Behaviour::Plug => text.push_str("Cierra el puerto al que se conecta\n"),
        Behaviour::Recorder { span_s, rate_hz } => {
            let _ = writeln!(text, "Ventana: {span_s:.0} s a {rate_hz:.0} muestras/s");
            text.push_str("Elige qué registra en la lista de fuentes\n");
        }
        Behaviour::ElectricalSupply { voltage_v } => {
            let _ = writeln!(text, "Tensión: {voltage_v:.0} V");
            text.push_str("Circuito eléctrico: se dibuja, todavía no se simula\n");
        }
        Behaviour::Contact { normally_closed } => {
            text.push_str(if *normally_closed {
                "Contacto cerrado en reposo\n"
            } else {
                "Contacto abierto en reposo\n"
            });
            text.push_str("Circuito eléctrico: se dibuja, todavía no se simula\n");
        }
        Behaviour::Relay {
            coil,
            contact,
            normally_closed,
        } => {
            let _ = writeln!(text, "Bobina: {}–{}", coil[0], coil[1]);
            let _ = writeln!(text, "Contacto: {}–{}", contact[0], contact[1]);
            text.push_str(if *normally_closed {
                "Contacto cerrado en reposo\n"
            } else {
                "Contacto abierto en reposo\n"
            });
        }
        Behaviour::Junction => text.push_str("Tres puertos en un punto\n"),
    }
}

/// Writes the valve's definition into `out` as NUL-terminated UTF-8.
///
/// Returns the bytes written without the terminator, or a negative code when there is no
/// model or the buffer is too small. Nothing is truncated silently: a short buffer is an
/// error the caller can see.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `out` must be null or point to
/// `capacity` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_describe_valve(
    session: *const PfSession,
    out: *mut c_char,
    capacity: u32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds both contracts.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return PF_ERR_NULL;
        };
        if out.is_null() {
            return PF_ERR_NULL;
        }
        let Some(text) = session.describe_valve() else {
            return PF_ERR_MODEL;
        };
        let bytes = text.as_bytes();
        let capacity = capacity as usize;
        // A description longer than `i32::MAX` is not a description; the bound also makes
        // the return value provably non-negative.
        if capacity == 0 || bytes.len() + 1 > capacity || bytes.len() > i32::MAX as usize {
            return PF_ERR_MODEL;
        }
        // SAFETY: the caller promised `capacity` writable bytes and the length was checked.
        let target = unsafe { std::slice::from_raw_parts_mut(out.cast::<u8>(), capacity) };
        target[..bytes.len()].copy_from_slice(bytes);
        target[bytes.len()] = 0;
        i32::try_from(bytes.len()).unwrap_or(PF_ERR_MODEL)
    })
}

/// Rebuilds the frame for an area of `width_mm` × `height_mm`, at `pixels_per_mm`.
///
/// The pointers in `out` borrow the session: they stay valid until the next call to this
/// function or to [`pf_session_free`]. The function allocates only when the drawing grew.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_build_frame(
    session: *mut PfSession,
    width_mm: f32,
    height_mm: f32,
    pixels_per_mm: f32,
    out: *mut PfFrame,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds both contracts.
        let (Some(session), Some(out)) = (unsafe { borrow(session) }, unsafe { out.as_mut() })
        else {
            return PF_ERR_NULL;
        };
        session.build_frame(width_mm, height_mm, pixels_per_mm);
        *out = PfFrame {
            calls: session.calls.as_ptr(),
            calls_len: session.calls.len() as u32,
            points: session.points.as_ptr(),
            points_len: session.points.len() as u32,
            text: session.text.as_ptr(),
            text_len: session.text.len() as u32,
            width_mm: session.width_mm,
            height_mm: session.height_mm,
        };
        PF_OK
    })
}

/// `1` when the point, in millimetres of the last built frame, falls on the valve.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_hit_valve(
    session: *const PfSession,
    x_mm: f32,
    y_mm: f32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return 0;
        };
        i32::from(session.hit_valve(x_mm, y_mm))
    })
}

/// The valve's box in millimetres of the last built frame, as `x0 y0 x1 y1`.
///
/// The front end draws its hover ring around this, so the affordance marks the real hit
/// area instead of a rectangle the C++ side guessed.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `out` must be null or point to
/// four writable floats.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_valve_rect(session: *const PfSession, out: *mut f32) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds both contracts.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return PF_ERR_NULL;
        };
        if out.is_null() {
            return PF_ERR_NULL;
        }
        let Some(layout) = session.layout else {
            return PF_ERR_MODEL;
        };
        // SAFETY: the caller promised four writable floats.
        let target = unsafe { std::slice::from_raw_parts_mut(out, 4) };
        target[0] = layout.valve.min.x;
        target[1] = layout.valve.min.y;
        target[2] = layout.valve.max.x;
        target[3] = layout.valve.max.y;
        PF_OK
    })
}

/// `1` when the drawing has a layout, `0` when the area is too small to be readable.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_has_layout(session: *const PfSession) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return 0;
        };
        i32::from(session.layout.is_some())
    })
}

/// The catalog, in display order. The pointers borrow the session.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_catalog(session: *const PfSession, out: *mut PfCatalog) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds both contracts.
        let (Some(session), Some(out)) =
            (unsafe { borrow_shared(session) }, unsafe { out.as_mut() })
        else {
            return PF_ERR_NULL;
        };
        *out = PfCatalog {
            items: session.catalog_items.as_ptr(),
            items_len: session.catalog_items.len() as u32,
            text: session.catalog_text.as_ptr(),
            text_len: session.catalog_text.len() as u32,
        };
        PF_OK
    })
}

/// Draws one catalog entry alone into `out`, for a sidebar thumbnail.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_preview(
    session: *mut PfSession,
    index: u32,
    width_mm: f32,
    height_mm: f32,
    pixels_per_mm: f32,
    out: *mut PfFrame,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds both contracts.
        let (Some(session), Some(out)) = (unsafe { borrow(session) }, unsafe { out.as_mut() })
        else {
            return PF_ERR_NULL;
        };
        let code =
            session.build_preview(index, width_mm.max(1.0), height_mm.max(1.0), pixels_per_mm);
        if code != PF_OK {
            return code;
        }
        *out = PfFrame {
            calls: session.preview_calls.as_ptr(),
            calls_len: session.preview_calls.len() as u32,
            points: session.preview_points.as_ptr(),
            points_len: session.preview_points.len() as u32,
            text: session.preview_text.as_ptr(),
            text_len: session.preview_text.len() as u32,
            width_mm,
            height_mm,
        };
        PF_OK
    })
}

/// Returns the model to `t = 0` without touching the catalog or the ink.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_restart(session: *mut PfSession) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        session.restart()
    })
}

/// The box a component of this category takes when it is dropped on the sheet.
///
/// A valve is wide and short, a cylinder is long: the drop then looks like the symbol it
/// brings, instead of every component arriving the same size.
fn default_size(category: u32) -> (f32, f32) {
    match category {
        PF_CATEGORY_SUPPLY => (26.0, 26.0),
        PF_CATEGORY_VALVE => (34.0, 17.0),
        PF_CATEGORY_ACTUATOR => (62.0, 26.0),
        PF_CATEGORY_THROTTLE => (20.0, 20.0),
        PF_CATEGORY_ACCESSORY => (24.0, 24.0),
        _ => (22.0, 22.0),
    }
}

/// Snaps a point to the grid the sheet draws, so components line up instead of floating.
fn snap(value: f32, grid: f32) -> f32 {
    (value / grid).round() * grid
}

/// Places a catalog entry at `(x_mm, y_mm)`, snapped to the 10 mm grid.
///
/// Returns the index of the new placement, or a negative code.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_place_component(
    session: *mut PfSession,
    catalog_index: u32,
    x_mm: f32,
    y_mm: f32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        let Some(id) = session.catalog_order.get(catalog_index as usize).cloned() else {
            return session.fail("that catalog entry does not exist");
        };
        let Some(definition) = session.catalog.get(&id) else {
            return session.fail("the catalog entry disappeared");
        };
        let (width, height) = default_size(category_code(definition.category));
        let index = session.document.place(punchi_fluid_doc::Placement::new(
            id,
            Pt::new(snap(x_mm, 10.0), snap(y_mm, 10.0)),
            width,
            height,
        ));
        session.select_only(Some(index));
        session.rebuild_model();
        i32::try_from(index).unwrap_or(PF_ERR_MODEL)
    })
}

/// Starts a new document with the worked example and no history.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_new_document(session: *mut PfSession) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        session.document = Document::new();
        session.seeded = false;
        session.select_only(None);
        session.ensure_seeded();
        session.error = CString::default();
        PF_OK
    })
}

/// How many circuits the project holds.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_circuit_count(session: *const PfSession) -> u32 {
    // SAFETY: the caller upholds the contract.
    let Some(session) = (unsafe { borrow_shared(session) }) else {
        return 0;
    };
    u32::try_from(session.document.circuits().len()).unwrap_or(0)
}

/// Identity of the circuit at `index`, or a negative error.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_circuit_id(session: *const PfSession, index: u32) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds the contract.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return PF_ERR_NULL;
        };
        let Some(circuit) = session.document.circuits().get(index as usize) else {
            return PF_ERR_MODEL;
        };
        i32::try_from(circuit.id).unwrap_or(PF_ERR_MODEL)
    })
}

/// Name of the circuit at `index`, NUL-terminated UTF-8 in `out`.
///
/// # Safety
///
/// `session` must be null or a live session pointer; `out` must be null or point to
/// `capacity` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_circuit_name(
    session: *const PfSession,
    index: u32,
    out: *mut c_char,
    capacity: u32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds both contracts.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return PF_ERR_NULL;
        };
        if out.is_null() {
            return PF_ERR_NULL;
        }
        let Some(circuit) = session.document.circuits().get(index as usize) else {
            return PF_ERR_MODEL;
        };
        let bytes = circuit.title.as_bytes();
        let capacity = capacity as usize;
        if capacity == 0 || bytes.len() + 1 > capacity {
            return PF_ERR_MODEL;
        }
        // SAFETY: the caller promised `capacity` writable bytes and the length was checked.
        let target = unsafe { std::slice::from_raw_parts_mut(out.cast::<u8>(), capacity) };
        target[..bytes.len()].copy_from_slice(bytes);
        target[bytes.len()] = 0;
        i32::try_from(bytes.len()).unwrap_or(PF_ERR_MODEL)
    })
}

/// Identity of the circuit the sheet shows, or a negative error.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_active_circuit(session: *const PfSession) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds the contract.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return PF_ERR_NULL;
        };
        i32::try_from(session.document.active_circuit()).unwrap_or(PF_ERR_MODEL)
    })
}

/// Switches the sheet to another circuit. A view change, not an edit.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_select_circuit(session: *mut PfSession, id: u32) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds the contract.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if !session.document.select_circuit(id) {
            return session.fail("no such circuit");
        }
        session.select_only(None);
        session.clipboard.clear();
        session.rebuild_model();
        session.error = CString::default();
        PF_OK
    })
}

/// Adds an empty circuit, makes it active and returns its identity.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `title` a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_add_circuit(
    session: *mut PfSession,
    title: *const c_char,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds the contract.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        let Some(title) = valid_circuit_title(title) else {
            return session.fail("the circuit title is empty or invalid");
        };
        let id = session.document.add_circuit(title);
        session.select_only(None);
        session.clipboard.clear();
        session.rebuild_model();
        session.error = CString::default();
        i32::try_from(id).unwrap_or(PF_ERR_MODEL)
    })
}

/// Renames a circuit.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `title` a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_rename_circuit(
    session: *mut PfSession,
    id: u32,
    title: *const c_char,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds the contract.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        let Some(title) = valid_circuit_title(title) else {
            return session.fail("the circuit title is empty or invalid");
        };
        if !session.document.rename_circuit(id, title) {
            return session.fail("no such circuit");
        }
        session.error = CString::default();
        PF_OK
    })
}

/// Keeps the selected component for a later paste.
///
/// The clipboard is one placement, because the editor selects one at a time; the
/// reference copies every selected component, and multi-selection will extend this.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_copy_selected(session: *mut PfSession) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if session.selected.is_empty() {
            return session.fail("nothing is selected");
        }
        session.clipboard = session
            .selected
            .iter()
            .filter_map(|index| session.document.placements().get(*index).cloned())
            .collect();
        PF_OK
    })
}

/// Pastes the copied component at a point of the sheet, snapped to the grid.
///
/// Returns the index of the new placement, or a negative code.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_paste(session: *mut PfSession, x_mm: f32, y_mm: f32) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if session.clipboard.is_empty() {
            return session.fail("the clipboard is empty");
        }
        let copies = session.clipboard.clone();
        let centre = Pt::new(snap(x_mm, 10.0), snap(y_mm, 10.0));
        session.paste_at(&copies, centre)
    })
}

/// Pastes the copied component displaced from where it was copied.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_paste_offset(
    session: *mut PfSession,
    right_mm: f32,
    down_mm: f32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if session.clipboard.is_empty() {
            return session.fail("the clipboard is empty");
        }
        let copies = session.clipboard.clone();
        let Some(anchor) = copies.first() else {
            return session.fail("the clipboard is empty");
        };
        let centre = Pt::new(
            snap(anchor.centre.x + right_mm, 10.0),
            snap(anchor.centre.y + down_mm, 10.0),
        );
        session.paste_at(&copies, centre)
    })
}

/// Copies a component in place, displaced by a delta, and returns the new index.
///
/// This is the Shift+drag gesture: the copy is under the pointer and the original stays.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_duplicate_component(
    session: *mut PfSession,
    index: u32,
    right_mm: f32,
    down_mm: f32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        let Some(source) = session.document.placements().get(index as usize) else {
            return session.fail("that component does not exist");
        };
        let centre = Pt::new(
            snap(source.centre.x + right_mm, 10.0),
            snap(source.centre.y + down_mm, 10.0),
        );
        let Some(copy) = session.document.duplicate(index as usize, Some(centre)) else {
            return session.fail("that component does not exist");
        };
        session.select_only(Some(copy));
        session.rebuild_model();
        i32::try_from(copy).unwrap_or(PF_ERR_MODEL)
    })
}

/// Saves the sheet to `path`, atomically.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `path` must be null or a
/// NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_save(session: *mut PfSession, path: *const c_char) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if path.is_null() {
            return PF_ERR_NULL;
        }
        // SAFETY: the caller promised a NUL-terminated string.
        let Ok(path) = unsafe { CStr::from_ptr(path) }.to_str() else {
            return PF_ERR_MODEL;
        };
        if path.is_empty() || path.len() > 4096 {
            return PF_ERR_MODEL;
        }
        match session.to_sheet().write(std::path::Path::new(path)) {
            Ok(()) => {
                session.saved_revision = Some(session.document.revision());
                session.remember_recent(path);
                PF_OK
            }
            // A save is not a model error, but the interface reads one channel; the
            // message lands there.
            Err(cause) => session.fail(&cause.to_string()),
        }
    })
}

/// What the export writes: a raster or a vector page.
pub const PF_EXPORT_PNG: u32 = 0;
/// A PDF page.
pub const PF_EXPORT_PDF: u32 = 1;

/// The page as an SVG document, in millimetres.
pub const PF_EXPORT_SVG: u32 = 2;

/// How many papers the export offers, so a front end can build its list from here.
#[unsafe(no_mangle)]
pub const extern "C" fn pf_export_paper_count() -> u32 {
    punchi_fluid_export::PaperSize::ALL.len() as u32
}

/// Name of the paper at `index`, written into `out` as a NUL-terminated string.
///
/// # Safety
///
/// `out` must point to `capacity` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_export_paper_name(index: u32, out: *mut c_char, capacity: u32) -> i32 {
    guard(|| {
        if out.is_null() || capacity == 0 {
            return PF_ERR_NULL;
        }
        let Some(paper) = punchi_fluid_export::PaperSize::ALL.get(index as usize) else {
            return PF_ERR_MODEL;
        };
        let bytes = paper.label().as_bytes();
        if bytes.len() + 1 > capacity as usize {
            return PF_ERR_MODEL;
        }
        // SAFETY: the caller promised `capacity` writable bytes and the length was checked.
        let buffer = unsafe { std::slice::from_raw_parts_mut(out.cast::<u8>(), capacity as usize) };
        buffer[..bytes.len()].copy_from_slice(bytes);
        buffer[bytes.len()] = 0;
        PF_OK
    })
}

/// Export settings, as the window's panel provides them.
///
/// One structure instead of a longer argument list: a sheet has a paper, an orientation, a
/// margin on each side, a resolution and a list, and a caller that wanted to leave one of
/// them out would otherwise have to remember its place.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PfExportOptions {
    /// `PF_EXPORT_PNG`, `PF_EXPORT_PDF` or `PF_EXPORT_SVG`.
    pub kind: u32,
    /// Index into the paper list, or [`PF_EXPORT_CUSTOM_PAPER`].
    pub paper: u32,
    /// `PF_EXPORT_LANDSCAPE` and `PF_EXPORT_PARTS_LIST`, OR-ed together.
    pub flags: u32,
    /// Resolution for a PNG, ignored by the vector writers.
    pub dpi: f32,
    /// Margin above the frame, millimetres.
    pub margin_top: f32,
    /// Margin right of the frame, millimetres.
    pub margin_right: f32,
    /// Margin below the frame, millimetres.
    pub margin_bottom: f32,
    /// Margin left of the frame, millimetres.
    pub margin_left: f32,
    /// Width of a custom sheet, millimetres; ignored for a named paper.
    pub width_mm: f32,
    /// Height of a custom sheet, millimetres.
    pub height_mm: f32,
}

/// The `paper` value that means "the size in `width_mm` and `height_mm`".
pub const PF_EXPORT_CUSTOM_PAPER: u32 = u32::MAX;

/// Turns the sheet on its side.
pub const PF_EXPORT_LANDSCAPE: u32 = 1;

/// Prints the parts list above the block.
pub const PF_EXPORT_PARTS_LIST: u32 = 2;

/// The settings a caller gets when it passes nothing: A4 portrait, 18 mm margins, the list.
impl Default for PfExportOptions {
    fn default() -> Self {
        Self {
            kind: PF_EXPORT_PDF,
            paper: 1,
            flags: PF_EXPORT_PARTS_LIST,
            dpi: 300.0,
            margin_top: 18.0,
            margin_right: 18.0,
            margin_bottom: 18.0,
            margin_left: 18.0,
            width_mm: 210.0,
            height_mm: 297.0,
        }
    }
}

/// Writes the sheet, as it is drawn, to `path`.
///
/// The page is composed by the export crate and drawn by the session, so what is printed is
/// what is on the sheet: the same placements, the same routes, the same designations.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `path` and `date` NUL-terminated
/// strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_export(
    session: *mut PfSession,
    path: *const c_char,
    kind: u32,
    paper: u32,
    landscape: u32,
    dpi: f32,
    date: *const c_char,
) -> i32 {
    let options = PfExportOptions {
        kind,
        paper,
        flags: if landscape == 0 {
            PF_EXPORT_PARTS_LIST
        } else {
            PF_EXPORT_LANDSCAPE | PF_EXPORT_PARTS_LIST
        },
        dpi,
        ..PfExportOptions::default()
    };
    // SAFETY: the caller upholds the contract of the same three pointers.
    unsafe { pf_session_export_with(session, path, date, &raw const options) }
}

/// Writes the sheet with the panel's settings: paper, margins, resolution and list.
///
/// # Safety
///
/// `session` must be null or a live session pointer, `path` and `date` NUL-terminated
/// strings, and `options` null or pointing to a readable [`PfExportOptions`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_export_with(
    session: *mut PfSession,
    path: *const c_char,
    date: *const c_char,
    options: *const PfExportOptions,
) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if path.is_null() || date.is_null() || options.is_null() {
            return PF_ERR_NULL;
        }
        // SAFETY: the caller promised a readable structure.
        let options = unsafe { options.read() };
        // SAFETY: the caller promised NUL-terminated strings.
        let (Ok(target), Ok(date)) = (
            unsafe { CStr::from_ptr(path) }.to_str(),
            unsafe { CStr::from_ptr(date) }.to_str(),
        ) else {
            return PF_ERR_MODEL;
        };
        session.export_page(target, date, &options)
    })
}

impl PfSession {
    /// Composes and writes one page, from the panel's settings.
    fn export_page(&mut self, target: &str, date: &str, options: &PfExportOptions) -> i32 {
        let kind = options.kind;
        if target.is_empty() || target.len() > 4096 {
            return self.fail("the export path is empty or too long");
        }
        if date.is_empty() || date.len() > 32 || date.chars().any(char::is_control) {
            return self.fail("the export date is empty or invalid");
        }
        let paper = if options.paper == PF_EXPORT_CUSTOM_PAPER {
            match punchi_fluid_export::PaperSize::custom(options.width_mm, options.height_mm) {
                Some(paper) => paper,
                None => {
                    return self.fail("the custom paper is outside the bounds the export accepts");
                }
            }
        } else {
            match punchi_fluid_export::PaperSize::ALL.get(options.paper as usize) {
                Some(paper) => *paper,
                None => return self.fail("no such paper"),
            }
        };
        let margins = punchi_fluid_export::Margins {
            top: options.margin_top,
            right: options.margin_right,
            bottom: options.margin_bottom,
            left: options.margin_left,
        };
        if !margins.is_valid() {
            return self.fail("the margins must be zero or positive");
        }
        // A printed page has to be identifiable after the fact, and the sheet has no name of
        // its own: the block takes the name of the file being written, which is what the user
        // called the drawing, and the name of the circuit in view.
        let mut block = export_block(&self.document, target, date);
        self.ensure_seeded();
        let circuit = self.document.active_circuit();
        let (placed, pipes) = scene(
            &self.document,
            &self.catalog,
            self.system.as_ref(),
            &self.placement_part,
            circuit,
            &self.charts,
            ContactStates {
                contacts: &self.contacts,
                relays: &self.relays,
            },
        );
        let bounds = punchi_fluid_draw::placed::bounds_placed(&placed, &pipes, &self.catalog);
        let mut page = punchi_fluid_export::Page {
            paper,
            orientation: if options.flags & PF_EXPORT_LANDSCAPE == 0 {
                punchi_fluid_export::Orientation::Portrait
            } else {
                punchi_fluid_export::Orientation::Landscape
            },
            margins,
            parts: if options.flags & PF_EXPORT_PARTS_LIST == 0 {
                None
            } else {
                Some(parts_list(&self.document, &self.catalog, circuit))
            },
            ..punchi_fluid_export::Page::a4(block.clone())
        };
        let Ok(area) = punchi_fluid_export::drawing_area(&page, &block) else {
            return self.fail("the page leaves no room for the drawing");
        };
        let scale = bounds.map_or(1.0, |bounds| {
            punchi_fluid_draw::surface::fit_scale(bounds, area)
        });
        block.scale = Some(if scale < 0.999 {
            format!("1:{:.0}", 1.0 / scale)
        } else {
            String::from("1:1")
        });
        // Keep the page's own copy in step; the renderer takes the block explicitly, but a
        // caller that reads the page must see the same scale.
        page.title_block = Some(block.clone());
        let draw = |surface: &mut dyn punchi_fluid_draw::Surface, area: punchi_fluid_draw::Rect| {
            let Some(bounds) = bounds else {
                return;
            };
            // The page hands the drawing the area the frame leaves free; the fit maps the
            // drawing's own bounds onto it, so a large circuit shrinks instead of being
            // cropped and a small one stays at its size in the middle of the page.
            let mut fitted = punchi_fluid_draw::surface::Scaled::fit(surface, bounds, area);
            punchi_fluid_draw::draw_placed(
                &mut fitted,
                &placed,
                &pipes,
                &self.catalog,
                &punchi_fluid_export::print_ink(),
                &[],
            );
        };
        let path = std::path::Path::new(target);
        let written = match kind {
            PF_EXPORT_PDF => punchi_fluid_export::render_pdf_with(&page, &block, draw)
                .and_then(|bytes| write_atomically(path, "pdf", &bytes)),
            PF_EXPORT_SVG => punchi_fluid_export::render_svg_with(&page, &block, draw)
                .and_then(|text| write_atomically(path, "svg", text.as_bytes())),
            _ => punchi_fluid_export::render_png_with(&page, &block, options.dpi.max(72.0), draw)
                .and_then(|bytes| write_atomically(path, "png", &bytes)),
        };
        match written {
            Ok(()) => PF_OK,
            Err(cause) => self.fail(&cause.to_string()),
        }
    }
}

/// The parts list of one circuit: what it holds, grouped by definition.
fn parts_list(
    document: &Document,
    catalog: &Catalog,
    circuit: u32,
) -> punchi_fluid_export::PartsList {
    let placements = document
        .placements()
        .iter()
        .filter(|placement| placement.circuit == circuit)
        .map(|placement| {
            (
                placement.catalog_id.as_str(),
                placement.designation.as_deref(),
            )
        });
    punchi_fluid_export::PartsList::group(catalog, placements)
}

/// The title block for an export: the file name, the date and the circuit in view.
fn export_block(document: &Document, target: &str, date: &str) -> punchi_fluid_export::TitleBlock {
    // The sheet has no name of its own, so the block takes the name of the file being
    // written, which is what the user called the drawing.
    let title = std::path::Path::new(target).file_stem().map_or_else(
        || String::from("Circuit"),
        |stem| stem.to_string_lossy().into_owned(),
    );
    let mut block = punchi_fluid_export::block_with_review_state(&title, date);
    block.project = document
        .circuits()
        .iter()
        .find(|circuit| circuit.id == document.active_circuit())
        .map_or_else(String::new, |circuit| circuit.title.clone());
    block
}

/// Writes `bytes` to `path` through a temporary file, so a crash leaves no half page.
fn write_atomically(
    path: &std::path::Path,
    extension: &str,
    bytes: &[u8],
) -> Result<(), punchi_fluid_export::ExportError> {
    let temporary = path.with_extension(format!("{extension}.part"));
    std::fs::write(&temporary, bytes).map_err(|source| {
        punchi_fluid_export::ExportError::Write {
            path: temporary.display().to_string(),
            source,
        }
    })?;
    std::fs::rename(&temporary, path).map_err(|source| punchi_fluid_export::ExportError::Write {
        path: path.display().to_string(),
        source,
    })
}

/// Tells the core where to keep the list of remembered documents.
///
/// Called once, at start, with a path the front end derived from the desktop's data
/// directory. Passing an empty string turns the list off, which is what a test or a
/// throwaway session wants: nothing is read and nothing is written.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `path` must be null or a
/// NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_set_recent_file(
    session: *mut PfSession,
    path: *const c_char,
) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if path.is_null() {
            return PF_ERR_NULL;
        }
        // SAFETY: the caller promised a NUL-terminated string.
        let Ok(path) = unsafe { CStr::from_ptr(path) }.to_str() else {
            return PF_ERR_MODEL;
        };
        if path.is_empty() {
            session.recent_path = None;
            session.recent = RecentList::new();
            return PF_OK;
        }
        if path.len() > 4096 {
            return PF_ERR_MODEL;
        }
        session.recent_path = Some(std::path::PathBuf::from(path));
        // A list that cannot be read is an empty list: the menu loses its entries, the
        // session starts.
        session.recent = RecentList::read(std::path::Path::new(path)).unwrap_or_default();
        PF_OK
    })
}

/// How many documents the list remembers.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_recent_count(session: *const PfSession) -> u32 {
    // SAFETY: the caller promises a live session pointer.
    unsafe { borrow_shared(session) }.map_or(0, |session| {
        u32::try_from(session.recent.len()).unwrap_or(0)
    })
}

/// The path of one remembered document, as NUL-terminated UTF-8.
///
/// Returns the length written, or a negative code. The front end derives the menu label
/// from the path, so the two can never disagree.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `out` must be null or point to
/// `capacity` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_recent_path(
    session: *const PfSession,
    slot: u32,
    out: *mut c_char,
    capacity: u32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds both contracts.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return PF_ERR_NULL;
        };
        if out.is_null() {
            return PF_ERR_NULL;
        }
        let Some(entry) = session.recent.entries().get(slot as usize) else {
            return PF_ERR_MODEL;
        };
        let bytes = entry.path.as_bytes();
        let capacity = capacity as usize;
        if capacity == 0 || bytes.len() + 1 > capacity {
            return PF_ERR_MODEL;
        }
        // SAFETY: the caller promised `capacity` writable bytes and the length was checked.
        let target = unsafe { std::slice::from_raw_parts_mut(out.cast::<u8>(), capacity) };
        target[..bytes.len()].copy_from_slice(bytes);
        target[bytes.len()] = 0;
        i32::try_from(bytes.len()).unwrap_or(PF_ERR_MODEL)
    })
}

/// Opens one remembered document by its place in the list.
///
/// A document that vanished since the list was written is forgotten and reported, which
/// also cleans the menu: the entries are the files that can still be opened.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_open_recent(session: *mut PfSession, slot: u32) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        let Some(entry) = session.recent.entries().get(slot as usize).cloned() else {
            return session.fail("that document is not in the list");
        };
        let sheet = match Sheet::read(std::path::Path::new(&entry.path)) {
            Ok(sheet) => sheet,
            Err(cause) => {
                if std::path::Path::new(&entry.path).exists() {
                    return session.fail(&cause.to_string());
                }
                session.recent.forget(&entry.path);
                if let Some(list_path) = session.recent_path.clone() {
                    let _ = session.recent.write(&list_path);
                }
                return session.fail("that document is no longer there");
            }
        };
        match session.load(&sheet) {
            Ok(()) => {
                session.remember_recent(&entry.path);
                session.error = CString::default();
                PF_OK
            }
            Err(cause) => session.fail(&cause),
        }
    })
}

/// Forgets every remembered document.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_forget_recent(session: *mut PfSession) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        session.recent = RecentList::new();
        if let Some(list_path) = session.recent_path.clone() {
            if let Err(cause) = session.recent.write(&list_path) {
                return session.fail(&cause.to_string());
            }
        }
        PF_OK
    })
}

/// `1` when the sheet holds work that is not in the file it came from, `0` otherwise.
///
/// What the title bar marks and what a warning before closing asks about. Marking a
/// component, selecting, zooming and the check are not changes: the document is the same.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_is_modified(session: *const PfSession) -> i32 {
    // SAFETY: the caller promises a live session pointer.
    unsafe { borrow_shared(session) }.map_or(0, |session| i32::from(session.is_modified()))
}

/// Writes a copy of the sheet for safety, without counting it as the saved document.
///
/// The autosave is a copy, not a save: the document is still modified afterwards, the path
/// the user chose is unchanged, and a crash costs the work since the last copy instead of
/// all of it.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `path` must be null or a
/// NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_autosave(session: *mut PfSession, path: *const c_char) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if path.is_null() {
            return PF_ERR_NULL;
        }
        // SAFETY: the caller promised a NUL-terminated string.
        let Ok(path) = unsafe { CStr::from_ptr(path) }.to_str() else {
            return PF_ERR_MODEL;
        };
        if path.is_empty() || path.len() > 4096 {
            return PF_ERR_MODEL;
        }
        match session.to_sheet().write(std::path::Path::new(path)) {
            Ok(()) => PF_OK,
            Err(cause) => session.fail(&cause.to_string()),
        }
    })
}

/// Opens a sheet that came from an autosave copy: the document is modified afterwards.
///
/// Recovering is not opening. The file is a copy taken to survive a crash, so the work in
/// it has not been saved anywhere the user chose, and the next save must go to a path of
/// their own. The document therefore stays modified, and the front end leaves the current
/// path empty.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `path` must be null or a
/// NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_recover(session: *mut PfSession, path: *const c_char) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if path.is_null() {
            return PF_ERR_NULL;
        }
        // SAFETY: the caller promised a NUL-terminated string.
        let Ok(path) = unsafe { CStr::from_ptr(path) }.to_str() else {
            return PF_ERR_MODEL;
        };
        if path.is_empty() || path.len() > 4096 {
            return PF_ERR_MODEL;
        }
        let sheet = match Sheet::read(std::path::Path::new(path)) {
            Ok(sheet) => sheet,
            Err(cause) => return session.fail(&cause.to_string()),
        };
        match session.load(&sheet) {
            Ok(()) => {
                session.saved_revision = None;
                session.error = CString::default();
                PF_OK
            }
            Err(cause) => session.fail(&cause),
        }
    })
}

/// Opens a sheet from `path`, replacing the document and rebuilding the model.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `path` must be null or a
/// NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_open(session: *mut PfSession, path: *const c_char) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if path.is_null() {
            return PF_ERR_NULL;
        }
        // SAFETY: the caller promised a NUL-terminated string.
        let Ok(path) = unsafe { CStr::from_ptr(path) }.to_str() else {
            return PF_ERR_MODEL;
        };
        if path.is_empty() || path.len() > 4096 {
            return PF_ERR_MODEL;
        }
        let sheet = match Sheet::read(std::path::Path::new(path)) {
            Ok(sheet) => sheet,
            Err(cause) => return session.fail(&cause.to_string()),
        };
        match session.load(&sheet) {
            Ok(()) => {
                session.error = CString::default();
                session.remember_recent(path);
                PF_OK
            }
            Err(cause) => session.fail(&cause),
        }
    })
}

/// Which fields a [`PfEditSet`] carries.
pub const PF_EDIT_PRESSURE: u32 = 1;
/// Valve or throttle conductance.
pub const PF_EDIT_CONDUCTANCE: u32 = 2;
/// Cylinder stroke.
pub const PF_EDIT_STROKE: u32 = 4;
/// Cylinder bore.
pub const PF_EDIT_BORE: u32 = 8;
/// Cylinder rod diameter.
pub const PF_EDIT_ROD: u32 = 16;
/// Cylinder moving mass.
pub const PF_EDIT_MASS: u32 = 32;

/// The editable parameters of one placement.
///
/// `mask` says which values are meaningful: a field the behaviour does not have is not
/// shown by the interface and is ignored on write. The values are in the units the
/// reference's dialog uses: gauge bar, millimetres and kilograms.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PfEditSet {
    /// Bit per field, `PF_EDIT_*`.
    pub mask: u32,
    /// Supply pressure, gauge bar.
    pub pressure_bar: f64,
    /// Valve or throttle conductance, `m³ / (s · Pa)`.
    pub conductance: f64,
    /// Cylinder stroke, millimetres.
    pub stroke_mm: f64,
    /// Cylinder bore, millimetres.
    pub bore_mm: f64,
    /// Cylinder rod diameter, millimetres.
    pub rod_mm: f64,
    /// Cylinder moving mass, kilograms.
    pub moving_mass_kg: f64,
}

/// How many components the sheet holds.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_component_count(session: *const PfSession) -> u32 {
    // SAFETY: the caller promises a live session pointer.
    unsafe { borrow_shared(session) }.map_or(0, |session| session.document.len() as u32)
}

/// The box of one placement, as `x0 y0 x1 y1` in millimetres.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `out` must be null or point to
/// four writable floats.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_component_rect(
    session: *const PfSession,
    index: u32,
    out: *mut f32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds both contracts.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return PF_ERR_NULL;
        };
        if out.is_null() {
            return PF_ERR_NULL;
        }
        let Some(placement) = session.document.placements().get(index as usize) else {
            return PF_ERR_MODEL;
        };
        let rect = placement.box_rect();
        // SAFETY: the caller promised four writable floats.
        let target = unsafe { std::slice::from_raw_parts_mut(out, 4) };
        target[0] = rect.min.x;
        target[1] = rect.min.y;
        target[2] = rect.max.x;
        target[3] = rect.max.y;
        PF_OK
    })
}

/// The catalog id of one placement, as NUL-terminated UTF-8.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `out` must be null or point to
/// `capacity` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_component_id(
    session: *const PfSession,
    index: u32,
    out: *mut c_char,
    capacity: u32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds both contracts.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return PF_ERR_NULL;
        };
        if out.is_null() {
            return PF_ERR_NULL;
        }
        let Some(placement) = session.document.placements().get(index as usize) else {
            return PF_ERR_MODEL;
        };
        let bytes = placement.catalog_id.as_bytes();
        let capacity = capacity as usize;
        if capacity == 0 || bytes.len() + 1 > capacity {
            return PF_ERR_MODEL;
        }
        // SAFETY: the caller promised `capacity` writable bytes and the length was checked.
        let target = unsafe { std::slice::from_raw_parts_mut(out.cast::<u8>(), capacity) };
        target[..bytes.len()].copy_from_slice(bytes);
        target[bytes.len()] = 0;
        i32::try_from(bytes.len()).unwrap_or(PF_ERR_MODEL)
    })
}

/// Which placement is under the point, or `-1`.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_hit_component(
    session: *const PfSession,
    x_mm: f32,
    y_mm: f32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return -1;
        };
        let circuit = session.document.active_circuit();
        session
            .document
            .placements()
            .iter()
            .rposition(|placement| {
                placement.circuit == circuit && placement.contains(Pt::new(x_mm, y_mm), 1.0)
            })
            .map_or(-1, |index| i32::try_from(index).unwrap_or(-1))
    })
}

/// Selects a placement, or clears the selection with `-1`.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_select(session: *mut PfSession, index: i32) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        session.select_only(usize::try_from(index).ok());
        PF_OK
    })
}

/// The index of the selected placement, or `-1`.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_selected(session: *const PfSession) -> i32 {
    // SAFETY: the caller promises a live session pointer.
    unsafe { borrow_shared(session) }
        .and_then(PfSession::primary)
        .map_or(-1, |index| i32::try_from(index).unwrap_or(-1))
}

/// The designation of one placement, as NUL-terminated UTF-8.
///
/// Returns the length written (`0` when the component has no name) or a negative code.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `out` must be null or point to
/// `capacity` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_component_designation(
    session: *const PfSession,
    index: u32,
    out: *mut c_char,
    capacity: u32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds both contracts.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return PF_ERR_NULL;
        };
        if out.is_null() {
            return PF_ERR_NULL;
        }
        let Some(placement) = session.document.placements().get(index as usize) else {
            return PF_ERR_MODEL;
        };
        let designation = placement.designation.as_deref().unwrap_or("");
        let bytes = designation.as_bytes();
        let capacity = capacity as usize;
        if capacity == 0 || bytes.len() + 1 > capacity {
            return PF_ERR_MODEL;
        }
        // SAFETY: the caller promised `capacity` writable bytes and the length was checked.
        let target = unsafe { std::slice::from_raw_parts_mut(out.cast::<u8>(), capacity) };
        target[..bytes.len()].copy_from_slice(bytes);
        target[bytes.len()] = 0;
        i32::try_from(bytes.len()).unwrap_or(PF_ERR_MODEL)
    })
}

/// How many sources a recorder can chart, in the circuit of the recorder.
///
/// Every placement that reports at least one quantity is a candidate, with one entry per
/// quantity it reports: pressure at its first port, flow through a meter, extension of a
/// cylinder. A recorder observes, so it is never a candidate for itself.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_chart_candidate_count(
    session: *const PfSession,
    index: u32,
) -> u32 {
    // SAFETY: the caller upholds the contract.
    let Some(session) = (unsafe { borrow_shared(session) }) else {
        return 0;
    };
    let Some(placement) = session.document.placements().get(index as usize) else {
        return 0;
    };
    let Some(definition) = session.catalog.get(&placement.catalog_id) else {
        return 0;
    };
    if !matches!(
        definition.behaviour,
        punchi_fluid_catalog::Behaviour::Recorder { .. }
    ) {
        return 0;
    }
    u32::try_from(
        session
            .chart_candidates(index as usize, placement.circuit)
            .len(),
    )
    .unwrap_or(0)
}

/// One source a recorder can chart, by slot, in the order the interface lists it.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `out` must be null or point to a
/// writable [`PfChartSource`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_chart_candidate(
    session: *const PfSession,
    index: u32,
    slot: u32,
    out: *mut PfChartSource,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds both contracts.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return PF_ERR_NULL;
        };
        if out.is_null() {
            return PF_ERR_NULL;
        }
        let Some(placement) = session.document.placements().get(index as usize) else {
            return PF_ERR_MODEL;
        };
        let Some(candidate) = session
            .chart_candidates(index as usize, placement.circuit)
            .get(slot as usize)
            .copied()
        else {
            return PF_ERR_MODEL;
        };
        // SAFETY: the caller promised a writable record.
        unsafe { out.write(candidate) };
        PF_OK
    })
}

/// The label of one chart source, in Spanish, as NUL-terminated UTF-8.
///
/// The label names the component — its designation when it has one — and the quantity, so
/// the inspector lists "1A1 (Cilindro) · posición" instead of an identity it cannot read.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `out` must be null or point to
/// `capacity` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_chart_candidate_label(
    session: *const PfSession,
    index: u32,
    slot: u32,
    out: *mut c_char,
    capacity: u32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds both contracts.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return PF_ERR_NULL;
        };
        if out.is_null() {
            return PF_ERR_NULL;
        }
        let Some(placement) = session.document.placements().get(index as usize) else {
            return PF_ERR_MODEL;
        };
        let Some(candidate) = session
            .chart_candidates(index as usize, placement.circuit)
            .get(slot as usize)
            .copied()
        else {
            return PF_ERR_MODEL;
        };
        let Some(target_index) = session.document.index_of(u64::from(candidate.placement)) else {
            return PF_ERR_MODEL;
        };
        let Some(target) = session.document.placements().get(target_index) else {
            return PF_ERR_MODEL;
        };
        let Some(definition) = session.catalog.get(&target.catalog_id) else {
            return PF_ERR_MODEL;
        };
        let quantity = match candidate.quantity {
            PF_QUANTITY_PRESSURE => "presión",
            PF_QUANTITY_FLOW => "caudal",
            PF_QUANTITY_POSITION => "posición",
            _ => return PF_ERR_MODEL,
        };
        let name = definition.display_name();
        let label = match target.designation.as_deref() {
            Some(designation) => format!("{designation} ({name}) · {quantity}"),
            None => format!("{name} · {quantity}"),
        };
        let bytes = label.as_bytes();
        let capacity = capacity as usize;
        if capacity == 0 || bytes.len() + 1 > capacity {
            return PF_ERR_MODEL;
        }
        // SAFETY: the caller promised `capacity` writable bytes and the length was checked.
        let target = unsafe { std::slice::from_raw_parts_mut(out.cast::<u8>(), capacity) };
        target[..bytes.len()].copy_from_slice(bytes);
        target[bytes.len()] = 0;
        i32::try_from(bytes.len()).unwrap_or(PF_ERR_MODEL)
    })
}

/// Binds a recorder to a source, as one undoable edit.
///
/// The document refuses a source that is missing, is the recorder itself or lives in
/// another circuit; this refuses a quantity the component cannot report, which only the
/// catalog knows.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_set_chart_source(
    session: *mut PfSession,
    index: u32,
    placement: u32,
    quantity: u32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds the contract.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        let Some(recorder) = session.document.placements().get(index as usize) else {
            return session.fail("no such recorder");
        };
        let Some(definition) = session.catalog.get(&recorder.catalog_id) else {
            return session.fail("the recorder has no definition");
        };
        if !matches!(
            definition.behaviour,
            punchi_fluid_catalog::Behaviour::Recorder { .. }
        ) {
            return session.fail("the placement is not a recorder");
        }
        let Some(quantity) = quantity_from_index(quantity) else {
            return session.fail("no such quantity");
        };
        let Some(target) = session.document.index_of(u64::from(placement)) else {
            return session.fail("the source component is not in the document");
        };
        let Some(target_definition) = session
            .document
            .placements()
            .get(target)
            .and_then(|placement| session.catalog.get(&placement.catalog_id))
        else {
            return session.fail("the source component has no definition");
        };
        if !PfSession::quantities_of(target_definition).contains(&quantity) {
            return session.fail("that component does not report that quantity");
        }
        let source = punchi_fluid_doc::Source::new(u64::from(placement), quantity);
        if !session.document.bind_source(index as usize, Some(source)) {
            return session.fail("the source was refused");
        }
        // A new source is a new window: the old samples describe a quantity nobody is
        // looking at any more.
        let recorder_id = session.document.placements()[index as usize].id();
        session.charts.remove(&recorder_id);
        session.rebuild_model();
        session.error = CString::default();
        PF_OK
    })
}

/// Unbinds a recorder, as one undoable edit.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_clear_chart_source(session: *mut PfSession, index: u32) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds the contract.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        let Some(placement) = session.document.placements().get(index as usize) else {
            return session.fail("no such recorder");
        };
        let id = placement.id();
        if !session.document.bind_source(index as usize, None) {
            return session.fail("the recorder is not bound");
        }
        session.charts.remove(&id);
        session.rebuild_model();
        session.error = CString::default();
        PF_OK
    })
}

/// The source of a recorder: `1` when it is bound, `0` when it is not, or a negative error.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `out` must be null or point to a
/// writable [`PfChartSource`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_chart_source(
    session: *const PfSession,
    index: u32,
    out: *mut PfChartSource,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds both contracts.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return PF_ERR_NULL;
        };
        if out.is_null() {
            return PF_ERR_NULL;
        }
        let Some(placement) = session.document.placements().get(index as usize) else {
            return PF_ERR_MODEL;
        };
        let Some(source) = placement.source else {
            return 0;
        };
        #[allow(clippy::cast_possible_truncation)]
        let record = PfChartSource {
            placement: source.placement as u32,
            quantity: quantity_index(source.quantity),
        };
        // SAFETY: the caller promised a writable record.
        unsafe { out.write(record) };
        1
    })
}

/// The Spanish name of a quantity, for the inspector, as NUL-terminated UTF-8.
///
/// # Safety
///
/// `out` must be null or point to `capacity` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_chart_quantity_name(
    quantity: u32,
    out: *mut c_char,
    capacity: u32,
) -> i32 {
    guard(|| {
        if out.is_null() {
            return PF_ERR_NULL;
        }
        let name = match quantity {
            PF_QUANTITY_PRESSURE => "presión",
            PF_QUANTITY_FLOW => "caudal",
            PF_QUANTITY_POSITION => "posición",
            _ => return PF_ERR_MODEL,
        };
        let bytes = name.as_bytes();
        let capacity = capacity as usize;
        if capacity == 0 || bytes.len() + 1 > capacity {
            return PF_ERR_MODEL;
        }
        // SAFETY: the caller promised `capacity` writable bytes and the length was checked.
        let target = unsafe { std::slice::from_raw_parts_mut(out.cast::<u8>(), capacity) };
        target[..bytes.len()].copy_from_slice(bytes);
        target[bytes.len()] = 0;
        i32::try_from(bytes.len()).unwrap_or(PF_ERR_MODEL)
    })
}

/// Names one placement, or takes the name away with an empty string.
///
/// The name is what the sheet draws under the symbol and what the parts list prints; an
/// empty one is no name at all, so two unnamed cylinders are not duplicates.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `text` must be null or a
/// NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_set_component_designation(
    session: *mut PfSession,
    index: u32,
    text: *const c_char,
) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if text.is_null() {
            return PF_ERR_NULL;
        }
        // SAFETY: the caller promised a NUL-terminated string.
        let Ok(text) = unsafe { CStr::from_ptr(text) }.to_str() else {
            return PF_ERR_MODEL;
        };
        // Naming it with the name it already has is not a change, and not an error: the
        // form applies every field it shows, so a user who opens the panel and presses
        // Aplicar must not be told the name is invalid.
        let trimmed = text.trim();
        let unchanged = if trimmed.is_empty() {
            session.document.designation_of(index as usize).is_none()
        } else {
            session.document.designation_of(index as usize) == Some(trimmed)
        };
        if unchanged || session.document.designate(index as usize, Some(text)) {
            PF_OK
        } else {
            session.fail("that designation cannot be used")
        }
    })
}

/// The editable parameters of one placement, as the reference's dialog shows them.
///
/// The values are the effective ones: the definition with this instance's overrides
/// already applied, so a form opens with what the model is using, not with the file.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `out` must be null or point to a
/// writable [`PfEditSet`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_component_edits(
    session: *const PfSession,
    index: u32,
    out: *mut PfEditSet,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds both contracts.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return PF_ERR_NULL;
        };
        if out.is_null() {
            return PF_ERR_NULL;
        }
        let Some(placement) = session.document.placements().get(index as usize) else {
            return PF_ERR_MODEL;
        };
        let Some(definition) = session.catalog.get(&placement.catalog_id) else {
            return PF_ERR_MODEL;
        };
        let mut definition = definition.clone();
        apply_edits(&mut definition, &placement.edits);
        let mut set = PfEditSet::default();
        match &definition.behaviour {
            Behaviour::Supply { pressure_bar } => {
                set.mask = PF_EDIT_PRESSURE;
                set.pressure_bar = *pressure_bar;
            }
            Behaviour::Valve { positions, .. } => {
                set.mask = PF_EDIT_CONDUCTANCE;
                set.conductance = positions
                    .first()
                    .and_then(|position| position.connection.first())
                    .map_or(0.0, |connection| connection.conductance);
            }
            // A throttle and a silencer are both restrictions: one field, one meaning.
            Behaviour::Throttle { conductance } | Behaviour::Silencer { conductance } => {
                set.mask = PF_EDIT_CONDUCTANCE;
                set.conductance = *conductance;
            }
            Behaviour::Cylinder {
                bore_mm,
                rod_mm,
                stroke_mm,
                moving_mass_kg,
                ..
            } => {
                set.mask = PF_EDIT_STROKE | PF_EDIT_BORE | PF_EDIT_ROD | PF_EDIT_MASS;
                set.bore_mm = *bore_mm;
                set.rod_mm = *rod_mm;
                set.stroke_mm = *stroke_mm;
                set.moving_mass_kg = *moving_mass_kg;
            }
            // Neither an instrument nor a termination nor a terminal has a pneumatic
            // parameter of its own to edit here.
            Behaviour::Gauge { .. }
            | Behaviour::FlowMeter { .. }
            | Behaviour::Recorder { .. }
            | Behaviour::ElectricalSupply { .. }
            | Behaviour::Contact { .. }
            | Behaviour::Relay { .. }
            | Behaviour::Plug
            | Behaviour::Junction => {}
        }
        // SAFETY: the caller promised a writable structure.
        unsafe { out.write(set) };
        PF_OK
    })
}

/// Writes the masked parameters of one placement, as one undoable edit.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `set` must be null or point to a
/// readable [`PfEditSet`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_set_component_edits(
    session: *mut PfSession,
    index: u32,
    set: *const PfEditSet,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds both contracts.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if set.is_null() {
            return PF_ERR_NULL;
        }
        // SAFETY: the caller promised a readable structure.
        let set = unsafe { *set };
        let Some(placement) = session.document.placements().get(index as usize) else {
            return session.fail("that component does not exist");
        };
        let Some(definition) = session.catalog.get(&placement.catalog_id) else {
            return session.fail("that component has no definition");
        };
        let mut edits = placement.edits.clone();
        if set.mask & PF_EDIT_PRESSURE != 0 {
            if !set.pressure_bar.is_finite() || set.pressure_bar < 0.0 {
                return session.fail("the supply pressure must be zero or positive");
            }
            edits.pressure_bar = Some(set.pressure_bar);
        }
        if set.mask & PF_EDIT_CONDUCTANCE != 0 {
            if !set.conductance.is_finite() || set.conductance <= 0.0 {
                return session.fail("the conductance must be positive");
            }
            edits.conductance = Some(set.conductance);
        }
        if set.mask & PF_EDIT_STROKE != 0 {
            if !set.stroke_mm.is_finite() || set.stroke_mm <= 0.0 {
                return session.fail("the stroke must be positive");
            }
            edits.stroke_mm = Some(set.stroke_mm);
        }
        if set.mask & PF_EDIT_MASS != 0 {
            if !set.moving_mass_kg.is_finite() || set.moving_mass_kg <= 0.0 {
                return session.fail("the moving mass must be positive");
            }
            edits.moving_mass_kg = Some(set.moving_mass_kg);
        }
        let bore = if set.mask & PF_EDIT_BORE != 0 {
            if !set.bore_mm.is_finite() || set.bore_mm <= 0.0 {
                return session.fail("the bore must be positive");
            }
            edits.bore_mm = Some(set.bore_mm);
            set.bore_mm
        } else {
            edits.bore_mm.unwrap_or(match &definition.behaviour {
                Behaviour::Cylinder { bore_mm, .. } => *bore_mm,
                _ => 0.0,
            })
        };
        if set.mask & PF_EDIT_ROD != 0 {
            if !set.rod_mm.is_finite() || set.rod_mm < 0.0 || set.rod_mm >= bore {
                return session.fail("the rod must be smaller than the bore");
            }
            edits.rod_mm = Some(set.rod_mm);
        }
        if !session.document.edit(index as usize, edits) {
            // The form was applied without changing anything: not an error, not an edit.
            return PF_OK;
        }
        session.rebuild_model();
        PF_OK
    })
}

/// Moves a placement, snapping to the grid.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_move_component(
    session: *mut PfSession,
    index: u32,
    x_mm: f32,
    y_mm: f32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if session
            .document
            .move_to(index as usize, Pt::new(snap(x_mm, 10.0), snap(y_mm, 10.0)))
        {
            PF_OK
        } else {
            PF_ERR_MODEL
        }
    })
}

/// Turns the selected placement a quarter, clockwise or the other way.
///
/// `anticlockwise` is what `Ctrl` and a double click does in the reference; the menu
/// command and the ribbon button are the clockwise turn.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_rotate_selected(
    session: *mut PfSession,
    anticlockwise: i32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if session.selected.is_empty() {
            return session.fail("select a component to turn it");
        }
        let turn = if anticlockwise == 0 {
            Turn::Clockwise
        } else {
            Turn::Anticlockwise
        };
        let indices = session.selected.clone();
        if session.document.turn_many(&indices, turn) {
            PF_OK
        } else {
            PF_ERR_MODEL
        }
    })
}

/// Reflects the selected placement on its own vertical axis.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_mirror_selected(session: *mut PfSession) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if session.selected.is_empty() {
            return session.fail("select a component to reflect it");
        }
        let indices = session.selected.clone();
        if session.document.turn_many(&indices, Turn::Reflect) {
            PF_OK
        } else {
            PF_ERR_MODEL
        }
    })
}

/// How a placement is turned: quarter turns clockwise and whether it is mirrored.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `out` must be null or point to
/// two writable integers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_component_orientation(
    session: *const PfSession,
    index: u32,
    out: *mut u32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds both contracts.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return PF_ERR_NULL;
        };
        if out.is_null() {
            return PF_ERR_NULL;
        }
        let Some(orientation) = session.document.orientation_of(index as usize) else {
            return PF_ERR_MODEL;
        };
        // SAFETY: the caller promised two writable integers.
        let target = unsafe { std::slice::from_raw_parts_mut(out, 2) };
        target[0] = u32::from(orientation.quarter_turns());
        target[1] = u32::from(orientation.mirrored());
        PF_OK
    })
}

/// Removes every selected placement, with their pipes, as one change.
///
/// A component and a pipe are never selected together, so this is the `Delete` of a
/// selection of components; the pipe is removed by its own command.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_remove_selection(session: *mut PfSession) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if session.selected.is_empty() {
            return session.fail("nothing is selected");
        }
        let indices = session.selected.clone();
        let removed = session.document.remove_many(&indices);
        // A removal renumbers everything after it, so an index kept from before it names
        // another component: the selection goes with what it selected.
        session.select_only(None);
        session.rebuild_model();
        i32::try_from(removed.len()).unwrap_or(PF_ERR_MODEL)
    })
}

/// Adds or removes one placement in the selection: `Mayús` and a click.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_toggle_select(session: *mut PfSession, index: i32) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        let Some(index) = usize::try_from(index).ok() else {
            return PF_ERR_MODEL;
        };
        if index >= session.document.len() {
            return session.fail("that component does not exist");
        }
        session.toggle_selected(index);
        PF_OK
    })
}

/// Selects every component of the sheet: `Edición ▸ Seleccionar todo`, `Ctrl+E`.
///
/// Returns how many components are selected.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_select_all(session: *mut PfSession) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        session.selected = (0..session.document.len()).collect();
        session.selected_pipe = None;
        i32::try_from(session.selected.len()).unwrap_or(PF_ERR_MODEL)
    })
}

/// Selects every component inside a rectangle of the sheet: the rubber band.
///
/// Returns how many components are selected; an empty rectangle clears the selection,
/// which is what letting go of a band over nothing means.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_select_rect(
    session: *mut PfSession,
    x0_mm: f32,
    y0_mm: f32,
    x1_mm: f32,
    y1_mm: f32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        let band = Rect::from_corners(Pt::new(x0_mm, y0_mm), Pt::new(x1_mm, y1_mm));
        session.selected = session
            .document
            .placements()
            .iter()
            .enumerate()
            .filter(|(_, placement)| {
                band.expand(0.5).contains(placement.box_rect().min)
                    && band.expand(0.5).contains(placement.box_rect().max)
            })
            .map(|(index, _)| index)
            .collect();
        session.selected_pipe = None;
        i32::try_from(session.selected.len()).unwrap_or(PF_ERR_MODEL)
    })
}

/// How many components are selected.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_selection_count(session: *const PfSession) -> u32 {
    // SAFETY: the caller promises a live session pointer.
    unsafe { borrow_shared(session) }.map_or(0, |session| {
        u32::try_from(session.selected.len()).unwrap_or(0)
    })
}

/// One selected placement by position in the selection, or `-1`.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_selection_at(session: *const PfSession, slot: u32) -> i32 {
    // SAFETY: the caller promises a live session pointer.
    unsafe { borrow_shared(session) }
        .and_then(|session| session.selected.get(slot as usize).copied())
        .map_or(-1, |index| i32::try_from(index).unwrap_or(-1))
}

/// Moves the whole selection by a delta, snapped to the grid, as one change.
///
/// This is the drag of several components: the pointer moves the primary one and the rest
/// follow by the same step, so the group keeps its shape.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_move_selection(
    session: *mut PfSession,
    right_mm: f32,
    down_mm: f32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if session.selected.is_empty() {
            return session.fail("nothing is selected");
        }
        let moves: Vec<(usize, Pt)> = session
            .selected
            .iter()
            .filter_map(|index| {
                let centre = session.document.placements().get(*index)?.centre;
                Some((*index, Pt::new(centre.x + right_mm, centre.y + down_mm)))
            })
            .collect();
        if session.document.move_many(&moves) {
            session.rebuild_model();
            PF_OK
        } else {
            PF_ERR_MODEL
        }
    })
}

/// Lines the selection up on the component that is already furthest out in one direction.
///
/// `edge` is `0` left, `1` right, `2` top, `3` bottom, which is the order of the buttons in
/// the ribbon. The whole alignment is one change, so one undo puts the circuit back.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_align_selection(session: *mut PfSession, edge: u32) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        let align = match edge {
            0 => Align::Left,
            1 => Align::Right,
            2 => Align::Top,
            3 => Align::Bottom,
            _ => return session.fail("that is not one of the four edges"),
        };
        if session.selected.len() < 2 {
            return session.fail("select at least two components to line them up");
        }
        let indices = session.selected.clone();
        if session.document.align_many(&indices, align) {
            session.rebuild_model();
            PF_OK
        } else {
            PF_ERR_MODEL
        }
    })
}

/// Spreads the selection so the gaps between the symbols are equal.
///
/// `axis` is `0` across (left to right) and `1` down (top to bottom), the order of the two
/// buttons in the ribbon. The two outermost keep their places and the whole spread is one
/// change, so one undo puts every component back.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_space_selection(session: *mut PfSession, axis: u32) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        let axis = match axis {
            0 => punchi_fluid_doc::Axis::Horizontal,
            1 => punchi_fluid_doc::Axis::Vertical,
            _ => return session.fail("that is not one of the two axes"),
        };
        if session.selected.len() < 3 {
            return session.fail("select at least three components to spread them");
        }
        let indices = session.selected.clone();
        if session.document.space_evenly(&indices, axis) {
            session.rebuild_model();
            PF_OK
        } else {
            session.fail("the selected symbols do not fit between their own ends")
        }
    })
}

/// Closes or opens the contact at one placement, as the click does.
///
/// The state belongs to the run: a reopened document starts with its contacts at rest
/// (ADR 0018). The control circuit follows at once, so a solenoid valve moves with the
/// click and not on the next frame.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_toggle_contact(session: *mut PfSession, index: u32) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds the contract.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        session.toggle_contact(index as usize)
    })
}

/// Whether the contact at one placement is closed: `1`, `0`, or a negative error when the
/// component is not a contact.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_contact_closed(session: *mut PfSession, index: u32) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds the contract.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        match session.contact_closed_at(index as usize) {
            Some(closed) => i32::from(closed),
            None => PF_ERR_MODEL,
        }
    })
}

/// The line class of one pipe: `0` working, `1` pilot, `2` drain, or a negative error.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_pipe_class(session: *const PfSession, index: u32) -> i32 {
    // SAFETY: the caller upholds the contract.
    let Some(session) = (unsafe { borrow_shared(session) }) else {
        return PF_ERR_NULL;
    };
    let Some(connection) = session.document.connections().get(index as usize) else {
        return PF_ERR_MODEL;
    };
    match connection.class() {
        LineClass::Working => 0,
        LineClass::Pilot => 1,
        LineClass::Drain => 2,
        // The electrical class arrives with F6; the ABI does not offer it yet.
        LineClass::Electrical => PF_ERR_MODEL,
    }
}

/// Gives one pipe another line class, as one undoable change.
///
/// `class` is `0` working, `1` pilot, `2` drain. The pattern is the class's, so the sheet
/// draws a pilot or drain line as itself, and the value travels with the pipe in the file.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_set_pipe_class(
    session: *mut PfSession,
    index: u32,
    class: u32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds the contract.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        let class = match class {
            0 => LineClass::Working,
            1 => LineClass::Pilot,
            2 => LineClass::Drain,
            _ => return session.fail("that is not a line class this build knows"),
        };
        if !session.document.set_pipe_class(index as usize, class) {
            return session.fail("the pipe is not there or already has that class");
        }
        session.error = CString::default();
        PF_OK
    })
}

/// Copies the whole selection in place, displaced by a delta, and selects the copies.
///
/// This is the Shift+drag gesture when more than one component is marked.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_duplicate_selection(
    session: *mut PfSession,
    right_mm: f32,
    down_mm: f32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if session.selected.is_empty() {
            return session.fail("nothing is selected");
        }
        let indices = session.selected.clone();
        let copies = session
            .document
            .duplicate_many(&indices, (snap(right_mm, 10.0), snap(down_mm, 10.0)));
        if copies.is_empty() {
            return session.fail("nothing could be copied");
        }
        session.selected = copies;
        session.rebuild_model();
        i32::try_from(session.selected.len()).unwrap_or(PF_ERR_MODEL)
    })
}

/// Removes a placement. The selection is cleared when it was the one removed.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_remove_component(session: *mut PfSession, index: u32) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if session.document.remove(index as usize).is_none() {
            return session.fail("that component does not exist");
        }
        session
            .selected
            .retain(|selected| *selected != index as usize);
        session.rebuild_model();
        PF_OK
    })
}

/// Takes back the last document change.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_undo(session: *mut PfSession) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if session.document.undo() {
            session.clamp_selection();
            session.rebuild_model();
            PF_OK
        } else {
            PF_ERR_MODEL
        }
    })
}

/// Applies the last undone document change again.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_redo(session: *mut PfSession) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if session.document.redo() {
            session.clamp_selection();
            session.rebuild_model();
            PF_OK
        } else {
            PF_ERR_MODEL
        }
    })
}

/// `1` when there is a document change to take back.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_can_undo(session: *const PfSession) -> i32 {
    // SAFETY: the caller promises a live session pointer.
    unsafe { borrow_shared(session) }.map_or(0, |session| i32::from(session.document.can_undo()))
}

/// `1` when an undone change can be applied again.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_can_redo(session: *const PfSession) -> i32 {
    // SAFETY: the caller promises a live session pointer.
    unsafe { borrow_shared(session) }.map_or(0, |session| i32::from(session.document.can_redo()))
}

/// Repaints one pipe in the accent colour, over the black one, so the selection is visible.
fn draw_selected_pipe(
    surface: &mut FrameBuffer<'_>,
    document: &Document,
    catalog: &Catalog,
    ink: &Ink,
    index: usize,
) {
    if document.connections().get(index).is_none() {
        return;
    }
    let Some(route) = pipe_route(document, catalog, index) else {
        return;
    };
    surface.line(&route, Stroke::new(2.4, ink.accent));
}

/// Spanish name of a category, because the inspector is interface copy.
fn category_es(category: punchi_fluid_catalog::Category) -> &'static str {
    use punchi_fluid_catalog::Category;
    match category {
        Category::Supply => "suministro",
        Category::Valve => "válvula",
        Category::Actuator => "actuador",
        Category::Throttle => "estrangulador",
        Category::Accessory => "accesorio",
        Category::Sensor => "sensor",
    }
}

/// Spanish name of an actuation, for the same reason.
fn actuation_es(actuation: punchi_fluid_catalog::Actuation) -> &'static str {
    use punchi_fluid_catalog::Actuation;
    match actuation {
        Actuation::None => "ninguno",
        Actuation::Toggle => "conmutar",
        Actuation::Momentary => "mantener",
    }
}

/// Applies the per-instance values of a placement to a copy of its definition.
///
/// Only fields the behaviour actually has are touched, so an override left over from a
/// previous component cannot leak into a different one.
fn apply_edits(definition: &mut punchi_fluid_catalog::ComponentDefinition, edits: &PlacementEdits) {
    match &mut definition.behaviour {
        Behaviour::Supply { pressure_bar } => {
            if let Some(value) = edits.pressure_bar {
                *pressure_bar = value;
            }
        }
        Behaviour::Valve { positions, .. } => {
            if let Some(value) = edits.conductance {
                for position in positions {
                    for connection in &mut position.connection {
                        connection.conductance = value;
                    }
                }
            }
        }
        // A throttle and a silencer are both restrictions, so the same override feeds both.
        Behaviour::Throttle { conductance } | Behaviour::Silencer { conductance } => {
            if let Some(value) = edits.conductance {
                *conductance = value;
            }
        }
        Behaviour::Cylinder {
            bore_mm,
            rod_mm,
            stroke_mm,
            moving_mass_kg,
            ..
        } => {
            if let Some(value) = edits.bore_mm {
                *bore_mm = value;
            }
            if let Some(value) = edits.rod_mm {
                *rod_mm = value;
            }
            if let Some(value) = edits.stroke_mm {
                *stroke_mm = value;
            }
            if let Some(value) = edits.moving_mass_kg {
                *moving_mass_kg = value;
            }
        }
        Behaviour::Gauge { .. }
        | Behaviour::FlowMeter { .. }
        | Behaviour::Recorder { .. }
        | Behaviour::ElectricalSupply { .. }
        | Behaviour::Contact { .. }
        | Behaviour::Relay { .. }
        | Behaviour::Plug
        | Behaviour::Junction => {}
    }
}

/// The run state a frame reads for its moving contacts: the clicked buttons and the relays
/// the net settled on.
#[derive(Clone, Copy)]
struct ContactStates<'a> {
    contacts: &'a std::collections::HashMap<u64, bool>,
    relays: &'a std::collections::HashMap<u64, bool>,
}

/// What the sheet holds, ready to be drawn: the placements with the state the model reports for
/// each, and the hoses with their route offsets.
///
/// The screen and the exported page both call this, so what is printed is what is on the sheet
/// and not a second construction that could drift from it.
fn scene<'a>(
    document: &'a Document,
    catalog: &'a Catalog,
    system: Option<&'a Network>,
    placement_part: &[Option<usize>],
    circuit: u32,
    charts: &'a std::collections::HashMap<u64, Chart>,
    states: ContactStates<'_>,
) -> (Vec<Placed<'a>>, Vec<Pipe<'a>>) {
    // Scene index of every document placement, so a pipe names the end it draws while the
    // selection and the hit test keep using the document index.
    let mut scene_of = vec![None; document.placements().len()];
    let mut placed: Vec<Placed<'a>> = Vec::new();
    for (index, placement) in document.placements().iter().enumerate() {
        if placement.circuit != circuit {
            continue;
        }
        scene_of[index] = Some(placed.len());
        placed.push(Placed {
            index,
            catalog_id: placement.catalog_id.as_str(),
            rect: placement.rect(),
            extension: placement_part
                .get(index)
                .copied()
                .flatten()
                .zip(system)
                .and_then(|(part, network)| network.extension_of(part))
                .map_or(0.0, |extension| extension as f32),
            valve_position: placement_part
                .get(index)
                .copied()
                .flatten()
                .zip(system)
                .and_then(|(part, network)| network.valve_position_at(part))
                .unwrap_or(0),
            reading: recorder_reading(placement, charts).or_else(|| {
                instrument_reading(
                    catalog.get(placement.catalog_id.as_str()),
                    placement_part.get(index).copied().flatten(),
                    system,
                )
            }),
            series: chart_series(placement.id(), charts),
            contact_closed: contact_closed(placement, states.contacts, states.relays),
            orientation: placement.orientation,
            designation: placement.designation.as_deref(),
        });
    }
    let pipes: Vec<Pipe<'a>> = document
        .connections()
        .iter()
        .filter(|connection| connection.circuit == circuit)
        .filter_map(|connection| {
            let from_document = document.index_of(connection.from.placement)?;
            let to_document = document.index_of(connection.to.placement)?;
            if document.placements().get(from_document)?.circuit != circuit
                || document.placements().get(to_document)?.circuit != circuit
            {
                return None;
            }
            let from_index = scene_of[from_document]?;
            let to_index = scene_of[to_document]?;
            Some(Pipe {
                from: PipeEnd {
                    placement: from_index,
                    port: connection.from.port.as_str(),
                },
                to: PipeEnd {
                    placement: to_index,
                    port: connection.to.port.as_str(),
                },
                offset_mm: connection.offset_mm(),
                class: connection.class(),
                pressure_ratio: hose_pressure(
                    system,
                    placement_part.get(from_document).copied().flatten(),
                    &connection.from.port,
                ),
            })
        })
        .collect();
    (placed, pipes)
}

/// The fixed-composition boxes, when the document still holds those four components.
fn layout_of(placed: &[Placed<'_>]) -> Option<Layout> {
    let find = |wanted: &str| {
        placed
            .iter()
            .find(|candidate| candidate.catalog_id == wanted)
            .map(|candidate| candidate.rect)
    };
    match (
        find(punchi_fluid_draw::schematic::SUPPLY_ID),
        find(punchi_fluid_draw::schematic::PREPARATION_ID),
        find(punchi_fluid_draw::schematic::VALVE_ID),
        find(punchi_fluid_draw::schematic::CYLINDER_ID),
    ) {
        (Some(supply), Some(preparation), Some(valve), Some(cylinder)) => Some(Layout {
            supply,
            preparation,
            valve,
            cylinder,
        }),
        _ => None,
    }
}

/// A relay in the control pass: placement index, coil pair, contact pair and rest state.
type RelayBinding = (usize, [String; 2], [String; 2], bool);

/// The electrical terminals of one circuit and the edges between them.
///
/// The net is what makes the control circuit a model instead of a drawing: terminals are
/// nodes, a wire and a closed contact are edges, and a rail marks its positive terminal as a
/// source (ADR 0018).
struct ControlNet {
    /// Terminal of each node: the placement index and the port id.
    nodes: Vec<(usize, String)>,
    /// Neighbours of each node.
    adjacency: Vec<Vec<usize>>,
    /// Nodes a rail energises.
    seeds: Vec<usize>,
}

impl ControlNet {
    fn node_of(&self, index: usize, port: &str) -> Option<usize> {
        self.nodes
            .iter()
            .position(|(candidate, id)| *candidate == index && id == port)
    }

    /// Which nodes are live: the seeds, then everything they reach.
    fn energised(&self) -> Vec<bool> {
        let mut live = vec![false; self.nodes.len()];
        let mut queue = self.seeds.clone();
        for seed in &self.seeds {
            live[*seed] = true;
        }
        while let Some(node) = queue.pop() {
            for next in &self.adjacency[node] {
                if !live[*next] {
                    live[*next] = true;
                    queue.push(*next);
                }
            }
        }
        live
    }
}

/// Builds the net of one circuit, with the contacts that are closed.
fn control_net(
    document: &Document,
    catalog: &Catalog,
    circuit: u32,
    closed: &[(usize, String, String)],
) -> ControlNet {
    let mut nodes: Vec<(usize, String)> = Vec::new();
    let mut seeds: Vec<usize> = Vec::new();
    for (index, placement) in document.placements().iter().enumerate() {
        if placement.circuit != circuit {
            continue;
        }
        let Some(definition) = catalog.get(&placement.catalog_id) else {
            continue;
        };
        for port in &definition.port {
            if port.role != punchi_fluid_catalog::PortRole::Electrical {
                continue;
            }
            let node = nodes.len();
            nodes.push((index, port.id.clone()));
            if matches!(
                definition.behaviour,
                punchi_fluid_catalog::Behaviour::ElectricalSupply { .. }
            ) && port.id == "L"
            {
                seeds.push(node);
            }
        }
    }
    let count = nodes.len();
    let mut net = ControlNet {
        nodes,
        adjacency: vec![Vec::new(); count],
        seeds,
    };
    let node_of = |index: usize, port: &str| {
        net.nodes
            .iter()
            .position(|(candidate, id)| *candidate == index && id == port)
    };
    // Wires join the terminals they connect.
    for connection in document.connections() {
        if connection.circuit != circuit {
            continue;
        }
        let (Some(from), Some(to)) = (
            document.index_of(connection.from.placement),
            document.index_of(connection.to.placement),
        ) else {
            continue;
        };
        if let (Some(a), Some(b)) = (
            node_of(from, &connection.from.port),
            node_of(to, &connection.to.port),
        ) {
            net.adjacency[a].push(b);
            net.adjacency[b].push(a);
        }
    }
    // Every closed contact joins its own two terminals.
    for (index, from, to) in closed {
        if let (Some(a), Some(b)) = (node_of(*index, from), node_of(*index, to)) {
            net.adjacency[a].push(b);
            net.adjacency[b].push(a);
        }
    }
    net
}

/// Rebuilds the net until every relay agrees with the coil that feeds it.
///
/// A relay contact is not clicked: it follows its coil, and a coil that follows a contact
/// can take more than one pass to settle. The loop is bounded by the number of relays, so a
/// circuit that oscillates still stops.
fn settle_relays(
    document: &Document,
    catalog: &Catalog,
    circuit: u32,
    user_closed: &[(usize, String, String)],
    relays: &[RelayBinding],
) -> Vec<bool> {
    let mut state: Vec<bool> = relays
        .iter()
        .map(|(_, _, _, normally_closed)| *normally_closed)
        .collect();
    let bound = relays.len().saturating_mul(2).saturating_add(2);
    for _ in 0..=bound {
        let net = control_net(
            document,
            catalog,
            circuit,
            &closed_pairs(user_closed, relays, &state),
        );
        let live = net.energised();
        let settled: Vec<bool> = relays
            .iter()
            .map(|(index, coil, _, normally_closed)| {
                let (Some(a), Some(b)) =
                    (net.node_of(*index, &coil[0]), net.node_of(*index, &coil[1]))
                else {
                    return *normally_closed;
                };
                let energised = live[a] != live[b];
                if *normally_closed {
                    !energised
                } else {
                    energised
                }
            })
            .collect();
        if settled == state {
            break;
        }
        state = settled;
    }
    state
}

/// The two terminals a contact or a relay contact joins, when the file says so.
///
/// A push button has exactly two electrical terminals; a relay has four and names its
/// contact pair in the behaviour, because its coil terminals share the same role.
fn contact_pair(
    definition: &punchi_fluid_catalog::ComponentDefinition,
) -> Option<(String, String)> {
    match &definition.behaviour {
        Behaviour::Contact { .. } => {
            let mut terminals = definition
                .port
                .iter()
                .filter(|port| port.role == punchi_fluid_catalog::PortRole::Electrical)
                .map(|port| port.id.clone());
            Some((terminals.next()?, terminals.next()?))
        }
        Behaviour::Relay { contact, .. } => Some((contact[0].clone(), contact[1].clone())),
        _ => None,
    }
}

/// The pairs the net treats as closed: the clicked contacts and the relays that settled.
fn closed_pairs(
    user_closed: &[(usize, String, String)],
    relays: &[(usize, [String; 2], [String; 2], bool)],
    state: &[bool],
) -> Vec<(usize, String, String)> {
    let mut closed = user_closed.to_vec();
    for ((index, _, contact, _), on) in relays.iter().zip(state) {
        if *on {
            closed.push((*index, contact[0].clone(), contact[1].clone()));
        }
    }
    closed
}

/// Whether a contact on the sheet is closed, when the definition is one.
///
/// A state nobody has clicked yet starts at what the file declares, so the sheet opens with
/// the contact in its rest position and the drawing needs no default of its own. A relay's
/// state is the net's, not the user's, and it lives in its own map.
fn contact_closed(
    placement: &punchi_fluid_doc::Placement,
    contacts: &std::collections::HashMap<u64, bool>,
    relays: &std::collections::HashMap<u64, bool>,
) -> Option<bool> {
    contacts
        .get(&placement.id())
        .or_else(|| relays.get(&placement.id()))
        .copied()
}

/// What a bound recorder charts, as its newest sample and its unit.
///
/// `None` when the recorder is unbound or has not sampled yet: an empty chart is not a
/// reading of zero.
fn recorder_reading(
    placement: &punchi_fluid_doc::Placement,
    charts: &std::collections::HashMap<u64, Chart>,
) -> Option<Reading> {
    let source = placement.source?;
    let chart = charts.get(&placement.id())?;
    let value = *chart.values.last()?;
    Some(match source.quantity {
        punchi_fluid_doc::Quantity::Pressure => Reading::Pressure(value),
        punchi_fluid_doc::Quantity::Flow => Reading::Flow(value),
        punchi_fluid_doc::Quantity::Position => Reading::Position(value),
    })
}

/// The window of samples of one recorder, when it has at least two points to draw.
fn chart_series(
    id: u64,
    charts: &std::collections::HashMap<u64, Chart>,
) -> Option<punchi_fluid_draw::placed::Series<'_>> {
    let chart = charts.get(&id)?;
    if chart.values.len() < 2 {
        return None;
    }
    let (min, max) = chart.span()?;
    Some(punchi_fluid_draw::placed::Series {
        values: &chart.values,
        min,
        max,
    })
}

/// What an instrument at this placement reads, in the unit its definition declares.
///
/// A component that is not an instrument, or one the solver could not build, reads nothing:
/// an absent reading and a reading of zero are different claims.
fn instrument_reading(
    definition: Option<&punchi_fluid_catalog::ComponentDefinition>,
    part: Option<usize>,
    network: Option<&Network>,
) -> Option<Reading> {
    let definition = definition?;
    let network = network?;
    let part = part?;
    match definition.behaviour {
        punchi_fluid_catalog::Behaviour::Gauge { .. } => {
            let id = definition.port.first()?;
            let pressure = network.port_pressure(part, &id.id)?;
            #[allow(clippy::cast_possible_truncation)]
            let reading = ((pressure - network.atmosphere_pa()) / 1e5) as f32;
            Some(Reading::Pressure(reading))
        }
        punchi_fluid_catalog::Behaviour::FlowMeter { .. } => {
            #[allow(clippy::cast_possible_truncation)]
            let reading = network.part_flow_l_min(part)? as f32;
            Some(Reading::Flow(reading))
        }
        _ => None,
    }
}

/// Pressure in a hose as a fraction of the supply span, for the drawing.
///
/// A hose of a definition the solver could not model stays neutral instead of guessing.
fn hose_pressure(network: Option<&Network>, component: Option<usize>, id: &str) -> f32 {
    let (Some(network), Some(component)) = (network, component) else {
        return 0.0;
    };
    let Some(pressure) = network.port_pressure(component, id) else {
        return 0.0;
    };
    let span = network.supply_pa() - network.atmosphere_pa();
    if span <= 0.0 {
        return 0.0;
    }
    #[allow(clippy::cast_possible_truncation)]
    let ratio = ((pressure - network.atmosphere_pa()) / span) as f32;
    ratio.clamp(0.0, 1.0)
}

/// Where a pipe starts and ends on the sheet, in millimetres.
///
/// A pipe whose components or ports cannot be found is not drawable and has no geometry:
/// the caller skips it rather than drawing a line to nowhere.
/// `true` when the two ports leave on perpendicular edges, so the route has one bend.
fn pipe_is_elbow(document: &Document, catalog: &Catalog, index: usize) -> bool {
    let Some(connection) = document.connections().get(index) else {
        return false;
    };
    let edges = (
        document
            .index_of(connection.from.placement)
            .and_then(|slot| document.placements().get(slot))
            .and_then(|placement| catalog.get(&placement.catalog_id))
            .map(|definition| port_edge(definition, &connection.from.port)),
        document
            .index_of(connection.to.placement)
            .and_then(|slot| document.placements().get(slot))
            .and_then(|placement| catalog.get(&placement.catalog_id))
            .map(|definition| port_edge(definition, &connection.to.port)),
    );
    matches!(edges, (Some(from), Some(to)) if from.horizontal() != to.horizontal())
}

/// The route of one pipe, from the edges its two ports sit on.
fn pipe_route(document: &Document, catalog: &Catalog, index: usize) -> Option<Vec<Pt>> {
    let connection = document.connections().get(index)?;
    let from = document
        .placements()
        .get(document.index_of(connection.from.placement)?)?;
    let to = document
        .placements()
        .get(document.index_of(connection.to.placement)?)?;
    let from_definition = catalog.get(&from.catalog_id)?;
    let to_definition = catalog.get(&to.catalog_id)?;
    let start = port_point(
        from.rect(),
        from.orientation,
        from_definition,
        &connection.from.port,
    )?;
    let end = port_point(
        to.rect(),
        to.orientation,
        to_definition,
        &connection.to.port,
    )?;
    Some(route_between(
        start,
        port_edge(from_definition, &connection.from.port),
        end,
        port_edge(to_definition, &connection.to.port),
        connection.offset_mm(),
    ))
}

fn pipe_points(document: &Document, catalog: &Catalog, index: usize) -> Option<(Pt, Pt)> {
    let connection = document.connections().get(index)?;
    let from = document
        .placements()
        .get(document.index_of(connection.from.placement)?)?;
    let to = document
        .placements()
        .get(document.index_of(connection.to.placement)?)?;
    let start = port_point(
        from.rect(),
        from.orientation,
        catalog.get(&from.catalog_id)?,
        &connection.from.port,
    )?;
    let end = port_point(
        to.rect(),
        to.orientation,
        catalog.get(&to.catalog_id)?,
        &connection.to.port,
    )?;
    Some((start, end))
}

/// Shortest distance from a point to a polyline, in the drawing's millimetres.
fn distance_to_path(point: Pt, path: &[Pt]) -> f32 {
    let mut best = f32::MAX;
    for pair in path.windows(2) {
        best = best.min(distance_to_segment(point, pair[0], pair[1]));
    }
    best
}

/// Shortest distance from a point to one segment.
fn distance_to_segment(point: Pt, start: Pt, end: Pt) -> f32 {
    let (dx, dy) = start.to(end);
    let length_squared = dx.mul_add(dx, dy * dy);
    if length_squared <= f32::EPSILON {
        return Pt::new(point.x - start.x, point.y - start.y).length();
    }
    let (px, py) = start.to(point);
    let t = ((px.mul_add(dx, py * dy)) / length_squared).clamp(0.0, 1.0);
    Pt::new(
        point.x - dx.mul_add(t, start.x),
        point.y - dy.mul_add(t, start.y),
    )
    .length()
}

/// The port nearest to a point, if one is close enough to be meant.
///
/// Writes the placement index into `index` and the port id into `port` (NUL-terminated
/// UTF-8). Returns `PF_OK`, or `PF_ERR_MODEL` when no port is within `tolerance_mm`.
///
/// # Safety
///
/// `session` must be null or a live session pointer; `index` must be null or writable;
/// `port` must be null or point to `capacity` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_port_at(
    session: *const PfSession,
    x_mm: f32,
    y_mm: f32,
    tolerance_mm: f32,
    index: *mut u32,
    port: *mut c_char,
    capacity: u32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds every contract above.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return PF_ERR_NULL;
        };
        let Some((found, id)) = session.port_at(Pt::new(x_mm, y_mm), tolerance_mm.max(0.5)) else {
            return PF_ERR_MODEL;
        };
        let bytes = id.as_bytes();
        if index.is_null() || port.is_null() {
            return PF_ERR_NULL;
        }
        let capacity = capacity as usize;
        if capacity == 0 || bytes.len() + 1 > capacity {
            return PF_ERR_MODEL;
        }
        // SAFETY: the caller promised one writable index and `capacity` writable bytes.
        unsafe { *index = found as u32 };
        // SAFETY: the caller promised `capacity` writable bytes and the length was checked.
        let target = unsafe { std::slice::from_raw_parts_mut(port.cast::<u8>(), capacity) };
        target[..bytes.len()].copy_from_slice(bytes);
        target[bytes.len()] = 0;
        PF_OK
    })
}

/// Draws a pipe between two ports. Returns `PF_OK`, or `PF_ERR_MODEL` when refused.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and both port ids must be
/// NUL-terminated UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_connect(
    session: *mut PfSession,
    from_index: u32,
    from_port: *const c_char,
    to_index: u32,
    to_port: *const c_char,
) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session and two strings.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if from_port.is_null() || to_port.is_null() {
            return PF_ERR_NULL;
        }
        // SAFETY: both were promised NUL-terminated by the caller.
        let (Ok(from_port), Ok(to_port)) = (
            unsafe { CStr::from_ptr(from_port) }.to_str(),
            unsafe { CStr::from_ptr(to_port) }.to_str(),
        ) else {
            return PF_ERR_MODEL;
        };
        let Some(from) = session
            .document
            .placements()
            .get(from_index as usize)
            .map(punchi_fluid_doc::Placement::id)
        else {
            return session.fail("that component does not exist");
        };
        let Some(to) = session
            .document
            .placements()
            .get(to_index as usize)
            .map(punchi_fluid_doc::Placement::id)
        else {
            return session.fail("that component does not exist");
        };
        if session.document.connect(
            punchi_fluid_doc::Endpoint::new(from, from_port),
            punchi_fluid_doc::Endpoint::new(to, to_port),
        ) {
            session.rebuild_model();
            PF_OK
        } else {
            session.fail("esas conexiones no se pueden unir")
        }
    })
}

/// Splits a pipe with a T junction and joins one more pipe end to it.
///
/// This is the gesture the reference documents: a hose end dropped on an existing hose
/// leaves a T where it landed. The junction is a component, so the whole thing is one entry
/// in the undo history. Returns `PF_OK` or an error code; on an error nothing changed.
///
/// # Safety
///
/// `session` must be null or a live session pointer; `attach_port` must be null or a
/// NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_splice_pipe(
    session: *mut PfSession,
    pipe_index: u32,
    x_mm: f32,
    y_mm: f32,
    attach_index: u32,
    attach_port: *const c_char,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds every contract above.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if attach_port.is_null() {
            return PF_ERR_NULL;
        }
        // SAFETY: the caller promised a NUL-terminated string.
        let Ok(attach_port) = unsafe { CStr::from_ptr(attach_port) }.to_str() else {
            return PF_ERR_MODEL;
        };
        let Some(attach) = session
            .document
            .placements()
            .get(attach_index as usize)
            .map(punchi_fluid_doc::Placement::id)
        else {
            return session.fail("ese componente no existe");
        };
        if session
            .document
            .connections()
            .get(pipe_index as usize)
            .is_none()
        {
            return session.fail("ese tubo no existe");
        }
        if !x_mm.is_finite() || !y_mm.is_finite() {
            return session.fail("la unión necesita un punto real");
        }
        let junction = punchi_fluid_doc::Placement::new(
            JUNCTION_ID,
            Pt::new(x_mm, y_mm),
            JUNCTION_MM,
            JUNCTION_MM,
        );
        if session.document.splice(
            pipe_index as usize,
            junction,
            punchi_fluid_doc::Endpoint::new(attach, attach_port),
        ) {
            session.rebuild_model();
            PF_OK
        } else {
            session.fail("esa unión no se puede insertar")
        }
    })
}

/// Where the `index`-th fault of the design check is, as `x y` in millimetres.
///
/// The interface marks the drawing with these, so a fault is pointed at rather than
/// described and left to the user to find. Call `pf_session_check` first: it returns the
/// count, and this indexes into the same list.
///
/// # Safety
///
/// `session` must be null or a live session pointer; `out` must be null or point to two
/// writable floats.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_finding_at(
    session: *const PfSession,
    index: u32,
    out: *mut f32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds both contracts.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return PF_ERR_NULL;
        };
        if out.is_null() {
            return PF_ERR_NULL;
        }
        let findings = session.findings();
        let Some(finding) = findings.get(index as usize) else {
            return PF_ERR_MODEL;
        };
        // SAFETY: the caller promised two writable floats.
        let target = unsafe { std::slice::from_raw_parts_mut(out, 2) };
        target[0] = finding.at.x;
        target[1] = finding.at.y;
        PF_OK
    })
}

/// How many pipes the sheet holds.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_connection_count(session: *const PfSession) -> u32 {
    // SAFETY: the caller promises a live session pointer.
    unsafe { borrow_shared(session) }
        .map_or(0, |session| session.document.connections().len() as u32)
}

/// The port of a placement, in millimetres of the sheet, as `x y`.
///
/// # Safety
///
/// `session` must be null or a live session pointer; `port` must be null or a
/// NUL-terminated string; `out` must be null or point to two writable floats.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_port_point(
    session: *const PfSession,
    index: u32,
    port: *const c_char,
    out: *mut f32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds every contract above.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return PF_ERR_NULL;
        };
        if port.is_null() || out.is_null() {
            return PF_ERR_NULL;
        }
        // SAFETY: the caller promised a NUL-terminated string.
        let Ok(port) = unsafe { CStr::from_ptr(port) }.to_str() else {
            return PF_ERR_MODEL;
        };
        let Some(placement) = session.document.placements().get(index as usize) else {
            return PF_ERR_MODEL;
        };
        let Some(definition) = session.catalog.get(&placement.catalog_id) else {
            return PF_ERR_MODEL;
        };
        let Some(point) = port_point(placement.rect(), placement.orientation, definition, port)
        else {
            return PF_ERR_MODEL;
        };
        // SAFETY: the caller promised two writable floats.
        let target = unsafe { std::slice::from_raw_parts_mut(out, 2) };
        target[0] = point.x;
        target[1] = point.y;
        PF_OK
    })
}

/// The pipe nearest to a point, or `-1`.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_pipe_at(
    session: *const PfSession,
    x_mm: f32,
    y_mm: f32,
    tolerance_mm: f32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return -1;
        };
        session
            .pipe_at(Pt::new(x_mm, y_mm), tolerance_mm.max(0.5))
            .map_or(-1, |index| i32::try_from(index).unwrap_or(-1))
    })
}

/// Selects a pipe by index, or clears every selection with `-1`.
///
/// A pipe and a component are never selected together: `Delete` has to know which one it
/// removes.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_select_pipe(session: *mut PfSession, index: i32) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        session.selected_pipe = usize::try_from(index).ok();
        session.selected.clear();
        PF_OK
    })
}

/// The selected pipe, or `-1`.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_selected_pipe(session: *const PfSession) -> i32 {
    // SAFETY: the caller promises a live session pointer.
    unsafe { borrow_shared(session) }
        .and_then(|session| session.selected_pipe)
        .map_or(-1, |index| i32::try_from(index).unwrap_or(-1))
}

/// Removes the selected pipe, if there is one.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_remove_selected_pipe(session: *mut PfSession) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        let Some(index) = session.selected_pipe else {
            return PF_ERR_MODEL;
        };
        if session.document.disconnect(index) {
            session.selected_pipe = None;
            session.rebuild_model();
            PF_OK
        } else {
            session.fail("that pipe does not exist")
        }
    })
}

/// Moves the middle segment of a pipe so it passes through a point of the sheet.
///
/// This is the drag that re-routes a hose: the route is an elbow and the segment between
/// its two bends follows the pointer. Every call of one drag extends the same history
/// entry, so the whole gesture is one undo.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_route_pipe(
    session: *mut PfSession,
    index: u32,
    x_mm: f32,
    y_mm: f32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        session.route_pipe(index as usize, Pt::new(x_mm, y_mm))
    })
}

/// How far the middle segment of a pipe sits from the automatic route, in millimetres.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `out` must be null or point to
/// a writable `float`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_pipe_offset(
    session: *const PfSession,
    index: u32,
    out: *mut f32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return PF_ERR_NULL;
        };
        if out.is_null() {
            return PF_ERR_NULL;
        }
        let Some(connection) = session.document.connections().get(index as usize) else {
            return PF_ERR_MODEL;
        };
        // SAFETY: the caller promised a writable float.
        unsafe { out.write(connection.offset_mm()) };
        PF_OK
    })
}

/// Checks the document and writes the findings, one per line, as NUL-terminated UTF-8.
///
/// Returns how many faults were found, or a negative code when there is no model or the
/// buffer is too small. A clean circuit writes an empty string and returns `0`.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `out` must be null or point to
/// `capacity` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_check(
    session: *const PfSession,
    out: *mut c_char,
    capacity: u32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds both contracts.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return PF_ERR_NULL;
        };
        if out.is_null() {
            return PF_ERR_NULL;
        }
        let findings = session.findings();
        let text = findings
            .iter()
            .map(punchi_fluid_check::Finding::describe)
            .collect::<Vec<_>>()
            .join("\n");
        let bytes = text.as_bytes();
        let capacity = capacity as usize;
        if capacity == 0 || bytes.len() + 1 > capacity {
            return PF_ERR_MODEL;
        }
        // SAFETY: the caller promised `capacity` writable bytes and the length was checked.
        let target = unsafe { std::slice::from_raw_parts_mut(out.cast::<u8>(), capacity) };
        target[..bytes.len()].copy_from_slice(bytes);
        target[bytes.len()] = 0;
        i32::try_from(findings.len()).unwrap_or(PF_ERR_MODEL)
    })
}

#[cfg(test)]
mod chart_tests {
    use super::Chart;

    #[test]
    fn a_recorder_keeps_its_window_and_drops_the_oldest() {
        // The window is what the file declares — sixty seconds at ten samples per second —
        // and a sample that no longer fits takes the oldest with it. A buffer that grew
        // would be a leak, and one that dropped the newest would hide what just happened.
        let mut chart = Chart::new(600, 0.1);
        for sample in 0..600 {
            chart.push(sample as f32);
        }
        assert_eq!(chart.values.len(), 600);
        assert_eq!(chart.values.first(), Some(&0.0));
        assert_eq!(chart.values.last(), Some(&599.0));

        chart.push(600.0);
        assert_eq!(chart.values.len(), 600, "the window keeps its size");
        assert_eq!(chart.values.first(), Some(&1.0), "the oldest sample goes");
        assert_eq!(chart.values.last(), Some(&600.0), "the newest stays");

        let (min, max) = chart.span().expect("a full window has a span");
        assert!((min - 1.0).abs() < f32::EPSILON && (max - 600.0).abs() < f32::EPSILON);
        assert_eq!(
            Chart::new(0, 0.0).capacity,
            2,
            "an empty ring is still usable"
        );
    }
}
