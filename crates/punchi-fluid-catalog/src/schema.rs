// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Shape of a component definition file.
//!
//! Every symbol the simulator knows about is described by one TOML file: what it
//! is, which ports it has, how it behaves and how it is drawn. Nothing here is
//! specific to a particular valve, so a new component is a new file, not a new
//! Rust type.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Version of the file format understood by this build.
///
/// 2 added the declared valve body, the declared spring and actuator parts, the
/// blocking lists per position and a line class on every line. 3 added the declared
/// `[return]` part and the whole set of actuation kinds; a file that still declares 2
/// reads, because `[spring]` is the short form of `[return] kind = "spring"`.
pub const SCHEMA_VERSION: u32 = 3;

/// Oldest format version this build still reads.
///
/// The reader never guesses: a file older than this is reported, not upgraded.
pub const SCHEMA_MIN: u32 = 2;

/// Broad family of a component, used to group it in a palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    /// Compressed air source.
    Supply,
    /// Directional or flow control valve.
    Valve,
    /// Cylinder, motor or other actuator.
    Actuator,
    /// Restriction without a spool.
    Throttle,
    /// Silencer, filter, manifold.
    Accessory,
    /// Pressure switch, gauge.
    Sensor,
}

impl Category {
    /// Stable name written in the definition file.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Supply => "supply",
            Self::Valve => "valve",
            Self::Actuator => "actuator",
            Self::Throttle => "throttle",
            Self::Accessory => "accessory",
            Self::Sensor => "sensor",
        }
    }
}

/// Role a port plays in the circuit, independent of its number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PortRole {
    /// Port fed by the compressed air source.
    Supply,
    /// Port connected to the working line.
    Working,
    /// Port open to atmosphere or to a return line.
    Exhaust,
    /// Rod, shaft or other mechanical connection.
    Mechanical,
}

/// How the user changes the position of a valve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Actuation {
    /// Fixed element, there is no position to change.
    #[default]
    None,
    /// Clicking flips to the next position and stays there.
    Toggle,
    /// Held while the pointer is pressed, released afterwards.
    Momentary,
}

/// Moving part the renderer has to draw with the simulation state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kinematics {
    /// Nothing moves.
    #[default]
    None,
    /// Piston travelling along the horizontal axis of the symbol box.
    LinearActuator,
}

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
        }
    }
}

/// End of the symbol box where a part sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    /// Left end.
    Left,
    /// Right end.
    Right,
    /// Top edge.
    Top,
    /// Bottom edge.
    Bottom,
}

/// Actuation drawn outside the valve body.
///
/// The list follows the actuation families of ISO 1219-1: manual, mechanical, by fluid
/// pressure and electrical. The file names the function, and the renderer owns the
/// geometry of each one, so a new valve that is pushed by a roller is a new file and not
/// a new branch here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActuatorKind {
    /// Hand lever: a bar against the body, the arm, and the pivot it turns about.
    Lever,
    /// Lever that stays where it is put: the lever plus the notch of a detent.
    LeverDetent,
    /// Push button: a stem ending in a bar with rounded ends, which is what a finger
    /// presses. The rounding is the difference between a button and the plain manual
    /// actuation, so one is not drawn as the other.
    PushButton,
    /// Mushroom-head button, for a palm: a dome a palm presses.
    MushroomButton,
    /// Foot pedal: the lever's foot with the slanting plate a foot stands on.
    Pedal,
    /// Piston actuation: a dome that sits on the body, pressed by a slave cylinder's
    /// piston. It has no stem and nothing to grip.
    Plunger,
    /// Roller: a circle on a short arm, with its axle marked at the centre. The axle is
    /// what tells a roller from a lever's pivot.
    Roller,
    /// Roller that acts in one direction only, drawn with its one-way tick.
    RollerOneWay,
    /// Pneumatic pilot: a box on the end face with an **open** triangle inside, pointing
    /// away from the body. Air does the pushing.
    Pilot,
    /// Hydraulic pilot: the same box and triangle, **filled**, because the fluid is the
    /// medium. Open for air, filled for oil.
    PilotHydraulic,
    /// Solenoid, one winding: a box with one diagonal through it.
    Solenoid,
    /// Solenoid with two windings: two boxes, one per coil.
    SolenoidDouble,
    /// Solenoid with a servo pilot: the winding box plus the pilot triangle it drives.
    SolenoidServo,
    /// Electric motor: a circle with an `M`.
    Motor,
}

impl ActuatorKind {
    /// Stable name written in the definition file.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lever => "lever",
            Self::LeverDetent => "lever_detent",
            Self::PushButton => "push_button",
            Self::MushroomButton => "mushroom_button",
            Self::Pedal => "pedal",
            Self::Plunger => "plunger",
            Self::Roller => "roller",
            Self::RollerOneWay => "roller_one_way",
            Self::Pilot => "pilot",
            Self::PilotHydraulic => "pilot_hydraulic",
            Self::Solenoid => "solenoid",
            Self::SolenoidDouble => "solenoid_double",
            Self::SolenoidServo => "solenoid_servo",
            Self::Motor => "motor",
        }
    }

    /// Every kind, in the order the reference sheet shows them.
    pub const ALL: &'static [Self] = &[
        Self::Lever,
        Self::LeverDetent,
        Self::PushButton,
        Self::MushroomButton,
        Self::Pedal,
        Self::Plunger,
        Self::Roller,
        Self::RollerOneWay,
        Self::Pilot,
        Self::PilotHydraulic,
        Self::Solenoid,
        Self::SolenoidDouble,
        Self::SolenoidServo,
        Self::Motor,
    ];

    /// Spanish name, the one the reference sheet and the file author read.
    #[must_use]
    pub const fn name_es(self) -> &'static str {
        match self {
            Self::Lever => "Palanca",
            Self::LeverDetent => "Palanca con enclavamiento",
            Self::PushButton => "Botón",
            Self::MushroomButton => "Botón tipo hongo",
            Self::Pedal => "Pedal",
            Self::Plunger => "Émbolo",
            Self::Roller => "Rodillo",
            Self::RollerOneWay => "Rodillo escamoteable",
            Self::Pilot => "Pilotaje neumático",
            Self::PilotHydraulic => "Pilotaje hidráulico",
            Self::Solenoid => "Electroimán, una bobina",
            Self::SolenoidDouble => "Electroimán, dos bobinas",
            Self::SolenoidServo => "Electroimán con servopilotaje",
            Self::Motor => "Motor eléctrico",
        }
    }
}

/// How a valve or a cylinder goes back to its rest state.
///
/// The three returns that push the spool back are the spring, the air and the hydraulic
/// fluid; `detent` is the absence of one: the spool stays where it is left, which is why
/// a valve with a detent and no actuator cannot exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReturnKind {
    /// Return spring, drawn as a zigzag at the declared end.
    Spring,
    /// Pneumatic return: pressure at that end pushes the spool back.
    Pneumatic,
    /// Hydraulic return: the same, drawn with the filled triangle.
    Hydraulic,
    /// No return: a detent notch holds the spool where it is left.
    Detent,
}

impl ReturnKind {
    /// Stable name written in the definition file.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Spring => "spring",
            Self::Pneumatic => "pneumatic",
            Self::Hydraulic => "hydraulic",
            Self::Detent => "detent",
        }
    }

    /// Every kind, in the order the reference sheet shows them.
    pub const ALL: &'static [Self] =
        &[Self::Spring, Self::Pneumatic, Self::Hydraulic, Self::Detent];

    /// `true` when the part pushes the spool back on its own.
    #[must_use]
    pub const fn pushes_back(self) -> bool {
        !matches!(self, Self::Detent)
    }

    /// Spanish name.
    #[must_use]
    pub const fn name_es(self) -> &'static str {
        match self {
            Self::Spring => "Retorno por muelle",
            Self::Pneumatic => "Retorno neumático",
            Self::Hydraulic => "Retorno hidráulico",
            Self::Detent => "Sin retorno, con enclavamiento",
        }
    }
}

/// Return spring declared as a part, so the renderer draws it and the lint can
/// require it.
///
/// Kept as the short spelling of `[return] kind = "spring"`: a file written before the
/// other returns existed still reads, and the renderer draws the same zigzag either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpringPart {
    /// End of the body the spring works on.
    pub side: Side,
}

/// Return declared as a part: spring, pressure or detent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReturnPart {
    /// What brings the spool back, if anything.
    pub kind: ReturnKind,
    /// End of the body the return works on.
    pub side: Side,
}

/// Actuation declared as a part.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ActuatorPart {
    /// What produces the movement.
    pub kind: ActuatorKind,
    /// End of the body the actuator works on.
    pub side: Side,
}

/// Drawing primitive supported by the symbol renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimitiveKind {
    /// Two points.
    Line,
    /// Open sequence of segments.
    Polyline,
    /// Closed sequence of segments.
    Polygon,
    /// `[x, y, width, height]`.
    Rect,
    /// `[cx, cy, radius]`.
    Circle,
    /// `[cx, cy, radius, start_degrees, end_degrees]`.
    Arc,
}

/// One drawing primitive, in coordinates normalised to the symbol box `0..=1`.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Primitive {
    /// Geometry to draw.
    pub kind: PrimitiveKind,
    /// Flat coordinate list, interpreted according to `kind`.
    #[serde(default)]
    pub points: Vec<f64>,
    /// Fills the inside instead of only stroking the outline.
    #[serde(default)]
    pub filled: bool,
    /// Draws the primitive with a dashed pattern, instead of the pattern of its
    /// line class. Kept for parts that are genuinely dashed, like a spring.
    #[serde(default)]
    pub dashed: bool,
    /// Class of the line. Required on every line and polyline.
    #[serde(default)]
    pub line_class: Option<LineClass>,
    /// Stroke width multiplier relative to the base line width.
    #[serde(default = "default_stroke")]
    pub stroke: f64,
}

fn default_stroke() -> f64 {
    1.0
}

/// Connection point of a component.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Port {
    /// Identifier used by the connections of a valve, typically `"1"`, `"2"`.
    pub id: String,
    /// What the port does.
    pub role: PortRole,
    /// Position inside the symbol box, `[x, y]` in `0..=1`.
    pub position: [f64; 2],
}

/// Internal path that a valve position opens.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Connection {
    /// Port on one side of the open path.
    pub from: String,
    /// Port on the other side.
    pub to: String,
    /// Sonic conductance of that path, `m³ / (s · Pa)`.
    pub conductance: f64,
}

/// One position of a valve: what it connects and how it looks.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ValvePosition {
    /// Human-readable name of the position.
    pub name: String,
    /// Spanish name, when the automatic one is not good enough.
    #[serde(default)]
    pub name_es: Option<String>,
    /// Paths that are open while the valve sits in this position.
    #[serde(default)]
    pub connection: Vec<Connection>,
    /// Symbol overlay drawn on top of the base symbol in this position, in
    /// coordinates relative to that position's own square (`0..=1` per square).
    #[serde(default)]
    pub symbol: Vec<Primitive>,
    /// Ports this position closes. The renderer draws the blocking mark for each
    /// one at the edge of the position's square.
    #[serde(default)]
    pub blocked: Vec<String>,
}

impl ValvePosition {
    /// The name to show in the interface, as [`ComponentDefinition::display_name`] does.
    #[must_use]
    pub fn display_name(&self) -> &str {
        self.name_es.as_deref().unwrap_or(&self.name)
    }
}

/// What the component does during the simulation.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Behaviour {
    /// Constant pressure source.
    Supply {
        /// Gauge pressure of the source, `bar`.
        pressure_bar: f64,
    },
    /// Switchable manifold.
    Valve {
        /// How the position changes.
        actuation: Actuation,
        /// Whether releasing the actuator returns to the default position.
        #[serde(default)]
        spring_return: bool,
        /// Position selected before the simulation starts.
        #[serde(default)]
        default_position: usize,
        /// Available positions, in drawing order.
        positions: Vec<ValvePosition>,
    },
    /// Single-acting cylinder with a return spring.
    Cylinder {
        /// Piston bore, `mm`.
        bore_mm: f64,
        /// Rod diameter, `mm`.
        ///
        /// The rod reduces the area of the chamber it crosses, which is the return
        /// chamber of a double-acting cylinder. A single-acting cylinder leaves it at
        /// zero and both areas are the bore's.
        #[serde(default)]
        rod_mm: f64,
        /// Usable stroke, `mm`.
        stroke_mm: f64,
        /// Length standing for the residual volume, `mm`.
        dead_length_mm: f64,
        /// Moving mass, `kg`.
        moving_mass_kg: f64,
        /// Return-spring stiffness, `N / m`.
        spring_rate: f64,
        /// Return-spring preload at rest, `N`.
        spring_preload: f64,
        /// Viscous damping, `N · s / m`.
        damping: f64,
        /// Coulomb friction, `N`.
        friction: f64,
    },
    /// Pressure gauge: a passive port that shows the pressure at that point.
    Gauge {
        /// Lowest pressure on the dial, gauge bar.
        #[serde(default)]
        min_bar: f64,
        /// Highest pressure on the dial, gauge bar.
        max_bar: f64,
    },
    /// Fixed restriction.
    Throttle {
        /// Sonic conductance, `m³ / (s · Pa)`.
        conductance: f64,
    },
    /// Flow meter: an inline restriction that reports the gas that goes through it.
    ///
    /// The conductance is the physics — a meter does obstruct the line it measures — and
    /// the nominal flow is the declared full scale, so a larger meter is another file.
    FlowMeter {
        /// Sonic conductance, `m³ / (s · Pa)`.
        conductance: f64,
        /// Full scale of the dial, normal litres per minute.
        nominal_l_min: f64,
    },
    /// A junction: hoses meet here and the component adds nothing of its own.
    ///
    /// It has no volume, no restriction and no force, so the solver treats every port as
    /// the same point. Its three ports sit on one another for exactly that reason; see
    /// ADR 0006 for why a T junction is a component and not a document node.
    Junction,
}

impl Behaviour {
    /// Short name used in listings and error messages.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Supply { .. } => "supply",
            Self::Valve { .. } => "valve",
            Self::Cylinder { .. } => "cylinder",
            Self::Gauge { .. } => "gauge",
            Self::Throttle { .. } => "throttle",
            Self::FlowMeter { .. } => "flow_meter",
            Self::Junction => "junction",
        }
    }
}

/// Where a definition came from, which decides what wins on a conflict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    /// Shipped inside the binary.
    #[default]
    Builtin,
    /// Found in the user catalog directory.
    User,
}

/// One component definition, as stored in a catalog file.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentDefinition {
    /// Format version, must equal [`SCHEMA_VERSION`].
    pub schema: u32,
    /// Stable identifier, `family.variant` in `snake_case`.
    pub id: String,
    /// Name shown in the interface.
    pub name: String,
    /// Spanish name, when the automatic one is not good enough.
    #[serde(default)]
    pub name_es: Option<String>,
    /// Family of the component.
    pub category: Category,
    /// One-line description.
    #[serde(default)]
    pub summary: Option<String>,
    /// Hides this definition, including a built-in one with the same id.
    #[serde(default)]
    pub disabled: bool,
    /// Connection points.
    #[serde(default)]
    pub port: Vec<Port>,
    /// Parts drawn in every position, in coordinates of the whole symbol box.
    ///
    /// A valve body is not listed here: it follows from `body` and the number of
    /// positions, so it can never come out ragged.
    #[serde(default)]
    pub symbol: Vec<Primitive>,
    /// Body of a directional valve, `[x, y, width, height]`, split into one square
    /// per position.
    #[serde(default)]
    pub body: Option<[f64; 4]>,
    /// Return spring, drawn by the renderer at the declared end.
    ///
    /// The short spelling of `[return] kind = "spring"`; a file declares one or the
    /// other, never both.
    #[serde(default)]
    pub spring: Option<SpringPart>,
    /// Return declared as a part: spring, air, hydraulic fluid or a detent.
    #[serde(rename = "return", default)]
    pub return_part: Option<ReturnPart>,
    /// Actuation, drawn by the renderer at the declared end.
    #[serde(default)]
    pub actuator: Option<ActuatorPart>,
    /// Simulation behaviour.
    pub behaviour: Behaviour,
    /// Moving parts the renderer must animate.
    #[serde(default)]
    pub kinematics: Kinematics,
    /// File this definition was read from.
    #[serde(skip)]
    pub source: Option<PathBuf>,
    /// Whether it was read from the binary or from the user directory.
    #[serde(skip)]
    pub origin: Origin,
}

impl ComponentDefinition {
    /// Finds a port by identifier.
    #[must_use]
    pub fn port(&self, id: &str) -> Option<&Port> {
        self.port.iter().find(|port| port.id == id)
    }

    /// Return the file declares, in either of the two spellings it accepts.
    ///
    /// `[spring]` is the short form of `[return] kind = "spring"`, so the renderer asks
    /// this and never reads the two fields itself. A file that declares both has one
    /// meaning too many, which the lint rejects; `[return]` wins here only so that the
    /// drawing never depends on the order the parser saw them.
    #[must_use]
    pub fn resolved_return(&self) -> Option<ReturnPart> {
        self.return_part.or(self.spring.map(|spring| ReturnPart {
            kind: ReturnKind::Spring,
            side: spring.side,
        }))
    }

    /// `true` when the file declares a return twice, in both spellings.
    #[must_use]
    pub fn declares_two_returns(&self) -> bool {
        self.return_part.is_some() && self.spring.is_some()
    }

    /// The name to show in the interface.
    ///
    /// The interface speaks Spanish and every built-in file carries `name_es`, so that is
    /// the one a reader sees; `name` is the fallback and the identifier a user's own file
    /// can leave alone. When the interface grows a real language setting this becomes the
    /// lookup keyed by it, and no file has to change.
    #[must_use]
    pub fn display_name(&self) -> &str {
        self.name_es.as_deref().unwrap_or(&self.name)
    }

    /// Finds the first port with `role`.
    #[must_use]
    pub fn port_with_role(&self, role: PortRole) -> Option<&Port> {
        self.port.iter().find(|port| port.role == role)
    }

    /// Squares of a valve body, one per position, in symbol-box coordinates.
    ///
    /// The renderer and the export draw the body from here, so the squares are equal
    /// and share edges no matter what a definition file says. Empty when the body is
    /// missing or degenerate, which the lint rejects.
    #[must_use]
    pub fn body_squares(&self) -> Vec<[f64; 4]> {
        let count = self.position_count();
        let Some([x, y, width, height]) = self.body else {
            return Vec::new();
        };
        if count == 0 || !(width.is_finite() && height.is_finite()) || height <= 0.0 {
            return Vec::new();
        }
        let side = height;
        (0..count)
            .map(|index| {
                #[allow(clippy::cast_precision_loss)]
                let offset = side * index as f64;
                [x + offset, y, side, height]
            })
            .filter(|square| square[0] + square[2] <= x + width + 1.0e-6)
            .collect()
    }

    /// Number of positions, `1` for anything that is not a valve.
    #[must_use]
    pub fn position_count(&self) -> usize {
        match &self.behaviour {
            Behaviour::Valve { positions, .. } => positions.len(),
            _ => 1,
        }
    }
}
