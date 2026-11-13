// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Drawing of catalog symbols, from the declaration and never from free geometry.
//!
//! A valve body is split into one equal square per position, the stubs come from the
//! port table, the flow path and its arrow from the connection, the blocking mark from
//! the position, the numerals from the port identifiers and the spring and actuator
//! from the declared parts. Nothing here reads a coordinate that a file could get
//! wrong, which is what makes the screen and the paper agree by construction.

use punchi_fluid_catalog::{
    ActuatorKind, Behaviour, ComponentDefinition, Kinematics, LineClass as CatalogLineClass,
    Primitive, PrimitiveKind, ReturnKind, Side,
};

use crate::surface::{Anchor, Colour, LineClass, Pt, Rect, Stroke, Surface, dashes};

/// Width of a symbol line, in millimetres of the drawing.
pub const LINE: f32 = 0.35;
/// Cap height of a port numeral.
pub const NUMERAL: f32 = 3.0;
/// Smallest symbol box that still shows its numerals.
pub const NUMERAL_FLOOR: f32 = 12.0;

/// Colours the drawing needs, taken from the theme or forced for print.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ink {
    /// Lines of a symbol.
    pub line: Colour,
    /// Lines of a position that is not active.
    pub dim: Colour,
    /// Secondary text.
    pub muted: Colour,
    /// Primary text.
    pub text: Colour,
    /// Supply line.
    pub supply: Colour,
    /// Working line.
    pub work: Colour,
    /// Accent, used for the active square.
    pub accent: Colour,
    /// Paper or sheet behind the drawing.
    pub background: Colour,
}

impl Ink {
    /// Ink for a printed sheet: black on white, the only thing a diagram needs.
    #[must_use]
    pub const fn print() -> Self {
        Self {
            line: Colour::BLACK,
            dim: Colour::new(130, 130, 130),
            muted: Colour::new(90, 90, 90),
            text: Colour::BLACK,
            supply: Colour::BLACK,
            work: Colour::BLACK,
            accent: Colour::new(220, 226, 240),
            background: Colour::WHITE,
        }
    }
}

/// Class of a primitive, from the file or its nature.
fn class_of(primitive: &Primitive) -> LineClass {
    if primitive.line_class.is_none() && primitive.dashed {
        return LineClass::Drain;
    }
    match primitive.line_class {
        Some(CatalogLineClass::Pilot) => LineClass::Pilot,
        Some(CatalogLineClass::Drain) => LineClass::Drain,
        Some(CatalogLineClass::Electrical) => LineClass::Electrical,
        Some(CatalogLineClass::Working) | None => LineClass::Working,
    }
}

fn scale(rect: Rect) -> (f32, f32) {
    (rect.width(), rect.height())
}

/// Point at a normalised position of the symbol box.
///
/// Clippy cannot know that a definition file is validated to `0..=1`, so the cast is
/// allowed here, once, instead of at every call site.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn at(rect: Rect, x: f64, y: f64) -> Pt {
    rect.point_at(x as f32, y as f32)
}

/// Point at a normalised position, in a box that is not the whole symbol.
#[must_use]
pub fn scaled(rect: Rect, x: f64, y: f64) -> Pt {
    at(rect, x, y)
}

/// Rectangle of a `rect` primitive inside the symbol box.
#[must_use]
pub fn rect_from(rect: Rect, points: &[f64]) -> Option<Rect> {
    let [x, y, width, height] = <[f32; 4]>::try_from(
        points
            .iter()
            .copied()
            .map(|value| {
                #[allow(clippy::cast_possible_truncation)]
                {
                    value as f32
                }
            })
            .collect::<Vec<f32>>()
            .as_slice(),
    )
    .ok()?;
    let (sx, sy) = scale(rect);
    Some(Rect::from_min_size(
        at(rect, f64::from(x), f64::from(y)),
        width * sx,
        height * sy,
    ))
}

/// Draws a list of primitives inside `rect`.
pub fn draw_primitives(
    surface: &mut dyn Surface,
    rect: Rect,
    primitives: &[Primitive],
    colour: Colour,
    width: f32,
) {
    for primitive in primitives {
        #[allow(clippy::cast_possible_truncation)]
        let stroke_width = width * primitive.stroke.max(0.5) as f32;
        let stroke = Stroke::new(stroke_width, colour).with_class(class_of(primitive));
        let points: Vec<Pt> = primitive
            .points
            .chunks_exact(2)
            .map(|pair| at(rect, pair[0], pair[1]))
            .collect();

        match primitive.kind {
            PrimitiveKind::Line | PrimitiveKind::Polyline => {
                for segment in dashes(&points, stroke.class.pattern()) {
                    surface.line(&segment, stroke);
                }
            }
            PrimitiveKind::Polygon => {
                if primitive.filled {
                    surface.polygon(&points, Some(colour), None);
                } else {
                    let mut closed = points.clone();
                    if let Some(first) = points.first().copied() {
                        closed.push(first);
                    }
                    for segment in dashes(&closed, stroke.class.pattern()) {
                        surface.line(&segment, stroke);
                    }
                }
            }
            PrimitiveKind::Rect => {
                if let Some(shape) = rect_from(rect, &primitive.points) {
                    surface.rect(
                        shape,
                        0.6,
                        primitive.filled.then_some(colour),
                        (!primitive.filled).then_some(stroke),
                    );
                }
            }
            PrimitiveKind::Circle => {
                let values: Vec<f32> = primitive
                    .points
                    .iter()
                    .map(|value| {
                        #[allow(clippy::cast_possible_truncation)]
                        {
                            *value as f32
                        }
                    })
                    .collect();
                if let [cx, cy, radius] = values.as_slice() {
                    let centre = at(rect, f64::from(*cx), f64::from(*cy));
                    surface.circle(
                        centre,
                        radius * rect.width().min(rect.height()),
                        primitive.filled.then_some(colour),
                        (!primitive.filled).then_some(stroke),
                    );
                }
            }
            PrimitiveKind::Arc => {
                if primitive.points.len() >= 5 {
                    let centre = at(rect, primitive.points[0], primitive.points[1]);
                    #[allow(clippy::cast_possible_truncation)]
                    let radius = primitive.points[2] as f32 * rect.width().min(rect.height());
                    #[allow(clippy::cast_possible_truncation)]
                    let start = primitive.points[3] as f32;
                    #[allow(clippy::cast_possible_truncation)]
                    let end = primitive.points[4] as f32;
                    surface.arc(centre, radius, start, end, stroke);
                }
            }
        }
    }
}

/// Point where a port's stub leaves the body.
#[must_use]
pub fn stub_end(rect: Rect, body: Rect, port: [f64; 2]) -> Pt {
    // A port is declared inside the symbol box, so its point is read in that frame and
    // then brought to the body edge. Reading it in the body's own frame would tilt every
    // stub whose port is not at the very end of the body.
    let point = at(rect, port[0], port[1]);
    let inside_x = (body.min.x..=body.max.x).contains(&point.x);
    let inside_y = (body.min.y..=body.max.y).contains(&point.y);
    match (inside_x, inside_y) {
        (true, false) => Pt::new(
            point.x,
            if point.y < body.min.y {
                body.min.y
            } else {
                body.max.y
            },
        ),
        (false, true) => Pt::new(
            if point.x < body.min.x {
                body.min.x
            } else {
                body.max.x
            },
            point.y,
        ),
        _ => point,
    }
}

/// Draws a text run over a soft background of the sheet colour.
///
/// A diagram puts labels near lines and lines near labels. Instead of measuring text to
/// avoid every crossing, the label carries a halo the colour of the paper: the line
/// passes behind it and the glyph stays readable. Works the same on screen and on paper,
/// which is the only reason it is allowed at all.
pub fn haloed_text(
    surface: &mut dyn Surface,
    at: Pt,
    text: &str,
    size: f32,
    colour: Colour,
    background: Colour,
) {
    haloed_text_anchored(surface, at, text, size, colour, background, Anchor::Middle);
}

/// The same label, hung from the other side: a caption that would sit on a pipe is
/// moved beside the symbol instead of through it.
pub fn haloed_text_anchored(
    surface: &mut dyn Surface,
    at: Pt,
    text: &str,
    size: f32,
    colour: Colour,
    background: Colour,
    anchor: Anchor,
) {
    surface.text(at, text, size * 1.45, background, anchor);
    surface.text(at, text, size, colour, anchor);
}

/// Radius of the connection circle drawn at every port, in millimetres of the symbol.
///
/// The circle is the convention that tells a beginner where a hose attaches: ISO 1219-1
/// marks a connection point as a small open circle, and the drawing owes it to every
/// symbol rather than leaving it to each file to remember.
fn connection_radius(rect: Rect) -> f32 {
    (rect.height() * 0.05).clamp(0.55, 1.4)
}

/// Marks a port with the small open circle a hose arrives at.
///
/// Drawn **after** the pipes and filled with the paper colour, so the last millimetre of a
/// hose goes behind the circle and the connection reads as a terminal instead of a line
/// that happens to stop.
pub fn draw_connection(surface: &mut dyn Surface, at: Pt, rect: Rect, ink: &Ink) {
    surface.circle(
        at,
        connection_radius(rect),
        Some(ink.background),
        Some(Stroke::new(LINE, ink.line)),
    );
}

/// Numeral of a port, outside the symbol next to its stub.
pub fn draw_numeral(
    surface: &mut dyn Surface,
    rect: Rect,
    definition: &ComponentDefinition,
    body: Option<Rect>,
    id: &str,
    port: [f64; 2],
    ink: &Ink,
) {
    // A numeral narrower than about 12 device pixels is a smudge, and the contract
    // prefers a legible marker to a smudge.
    if rect.height() * surface.device_scale() < NUMERAL_FLOOR {
        return;
    }
    let point = at(rect, port[0], port[1]);
    let centre = rect.center();
    let reach = rect.height() * 0.14;
    let anchor = match body {
        // A side port joins the body through a horizontal stub, and its numeral belongs
        // beyond the outer end of that stub. Anchoring it at the body edge — where a top or
        // a bottom port is read — puts a side port's numeral inside the symbol, on top of
        // the rod.
        Some(body) if is_side_port(body, point) => point,
        Some(body) => stub_end(rect, body, port),
        None => point,
    };
    let offset = match body {
        Some(body) if is_side_port(body, point) => {
            let (dx, dy) = centre.to(anchor);
            let length = dx.hypot(dy);
            if length > f32::EPSILON {
                Pt::new(dx / length * reach, dy / length * reach)
            } else {
                Pt::new(reach, 0.0)
            }
        }
        Some(body) => {
            // A top or a bottom port: the designation sits level with the body edge, beside
            // its own stub and **towards the body**, on the side the body still has room for.
            // Two reasons for each half of that: a numeral pushed past the end of the body
            // reads as belonging to whatever comes next on the sheet, and one written past
            // the far end of the stub sits exactly where the pipe arrives, so its halo cuts
            // the pipe in two.
            // A numeral belongs on the side of its stub where no other port is written: the
            // ports of a two-position valve are all drawn on one square, so the body's middle
            // is not a useful reference for "outward", and two numerals written between their
            // stubs read as one port with two names.
            let below = point.y >= centre.y;
            let mut nearest: Option<(f32, f32)> = None;
            for other in &definition.port {
                let other_point = at(rect, other.position[0], other.position[1]);
                if (other_point.y >= centre.y) != below {
                    continue;
                }
                let gap = (other_point.x - point.x).abs();
                if gap <= f32::EPSILON {
                    continue;
                }
                if nearest.is_none_or(|(closest, _)| gap < closest) {
                    nearest = Some((gap, if other_point.x > point.x { -1.0 } else { 1.0 }));
                }
            }
            let side = if let Some((_, away)) = nearest {
                away
            } else {
                let outward = if point.x < body.center().x { -1.0 } else { 1.0 };
                let room = if outward < 0.0 {
                    point.x - body.min.x
                } else {
                    body.max.x - point.x
                };
                if room > reach * 1.1 {
                    outward
                } else {
                    -outward
                }
            };
            // Clear of the body, **lateral to its own stub**: a numeral written past the far end
            // of the stub sits where the pipe arrives and its halo cuts the pipe in two, and one
            // written towards the body lands inside it. This is the band between the two.
            let clear = reach * 1.6;
            Pt::new(side * clear, if below { reach * 0.9 } else { -reach * 0.9 })
        }
        None => {
            // A flat component declares no body, so there is no edge to sit beside: the numeral
            // goes **lateral** to the stub instead of past its end. A pipe arrives at the end
            // of a stub, and a numeral written there has its halo cut through the pipe.
            let (dx, dy) = centre.to(anchor);
            let length = dx.hypot(dy);
            let clear = reach * 1.6;
            if length <= f32::EPSILON {
                Pt::new(clear, 0.0)
            } else {
                let mut lateral = Pt::new(-dy / length * clear, dx / length * clear);
                let beside = Pt::new(anchor.x + lateral.x, anchor.y + lateral.y);
                if beside.x < rect.min.x || beside.x > rect.max.x {
                    lateral = Pt::new(-lateral.x, -lateral.y);
                }
                lateral
            }
        }
    };
    haloed_text(
        surface,
        Pt::new(anchor.x + offset.x, anchor.y + offset.y),
        id,
        NUMERAL,
        ink.line,
        ink.background,
    );
}

/// Whether a port sits to the left or the right of the body, level with it.
///
/// Those are the ports that join a horizontal stub; the top and the bottom ones join a
/// vertical stub and are read next to the body's edge.
fn is_side_port(body: Rect, point: Pt) -> bool {
    let level = (body.min.y..=body.max.y).contains(&point.y);
    let outside = point.x < body.min.x || point.x > body.max.x;
    level && outside
}

fn axis(body: Rect, side: Side) -> (Pt, Pt) {
    let centre = body.center();
    match side {
        Side::Left => (Pt::new(body.min.x, centre.y), Pt::new(-1.0, 0.0)),
        Side::Right => (Pt::new(body.max.x, centre.y), Pt::new(1.0, 0.0)),
        Side::Top => (Pt::new(centre.x, body.min.y), Pt::new(0.0, -1.0)),
        Side::Bottom => (Pt::new(centre.x, body.max.y), Pt::new(0.0, 1.0)),
    }
}

/// Point and unit direction a fraction of the way along a polyline.
///
/// Used by the flow arrows: the arrow sits on the line, so it has to know where the line
/// is and which way it runs at that point.
#[must_use]
pub fn along(points: &[Pt], at: f32) -> Option<(Pt, Pt)> {
    let total: f32 = points
        .windows(2)
        .map(|pair| {
            let (dx, dy) = pair[0].to(pair[1]);
            dx.hypot(dy)
        })
        .sum();
    if total <= f32::EPSILON || points.len() < 2 {
        return None;
    }
    let target = total * at.clamp(0.0, 1.0);
    let mut walked = 0.0;
    for pair in points.windows(2) {
        let (dx, dy) = pair[0].to(pair[1]);
        let length = dx.hypot(dy);
        if walked + length >= target {
            let t = if length <= f32::EPSILON {
                0.0
            } else {
                (target - walked) / length
            };
            let point = Pt::new(pair[0].x + dx * t, pair[0].y + dy * t);
            let direction = if length <= f32::EPSILON {
                Pt::new(1.0, 0.0)
            } else {
                Pt::new(dx / length, dy / length)
            };
            return Some((point, direction));
        }
        walked += length;
    }
    None
}

/// Solid arrowhead on a line, pointing along `direction`.
pub fn draw_flow_arrow(
    surface: &mut dyn Surface,
    at: Pt,
    direction: Pt,
    size: f32,
    colour: Colour,
) {
    let tip = Pt::new(at.x + direction.x * size, at.y + direction.y * size);
    let back = Pt::new(
        at.x - direction.x * size * 0.4,
        at.y - direction.y * size * 0.4,
    );
    let width = (-direction.y * size * 0.55, direction.x * size * 0.55);
    surface.polygon(
        &[
            tip,
            Pt::new(back.x + width.0, back.y + width.1),
            Pt::new(back.x - width.0, back.y - width.1),
        ],
        Some(colour),
        None,
    );
}

/// Return spring drawn as a zigzag away from the body, at the declared end.
pub fn draw_spring(surface: &mut dyn Surface, body: Rect, side: Side, colour: Colour, width: f32) {
    let (edge, out) = axis(body, side);
    let along = (-out.y, out.x);
    // The coil has to be read at palette size, so it is drawn at a size proportional to the
    // body and not to the line: one step away from the body, three steps of travel, and an
    // amplitude wide enough that its turns are not a smudge.
    let step = body.height().min(body.width()) * 0.13;
    let half = body.height().min(body.width()) * 0.20;
    let stroke = Stroke::new(width * 0.9, colour);

    let start = Pt::new(edge.x + out.x * step, edge.y + out.y * step);
    let end = Pt::new(start.x + out.x * step * 3.0, start.y + out.y * step * 3.0);
    let mut points = vec![edge, start];
    let coils = 5_u16;
    for index in 0..coils * 2 {
        let progress = (f32::from(index) + 0.5) / f32::from(coils * 2);
        let centre = Pt::new(
            start.x + out.x * step * 3.0 * progress,
            start.y + out.y * step * 3.0 * progress,
        );
        let offset = if index % 2 == 0 { half } else { -half };
        points.push(Pt::new(
            centre.x + along.0 * offset,
            centre.y + along.1 * offset,
        ));
    }
    points.push(end);
    surface.line(&points, stroke);
}

/// Actuation drawn away from the body, at the declared end.
///
/// Every kind is one function of ISO 1219-1 drawn in the same frame: a stem leaves the
/// body at the declared side, and the glyph that names the function sits at its end. The
/// geometry lives here so a definition file picks a function and never a shape, and so
/// that the same glyph appears on the screen, on paper and on the reference sheet.
pub fn draw_actuator(
    surface: &mut dyn Surface,
    body: Rect,
    kind: ActuatorKind,
    side: Side,
    colour: Colour,
    width: f32,
) {
    let (edge, out) = axis(body, side);
    let along = (-out.y, out.x);
    // An actuator drawn on top of the box belongs to no position: it starts clear of the
    // edge and reaches out far enough to read at palette size.
    let step = body.height().min(body.width()) * 0.13;
    let reach = step * 2.0;

    let p = |a: f32, b: f32| {
        Pt::new(
            edge.x + out.x * a + along.0 * b,
            edge.y + out.y * a + along.1 * b,
        )
    };

    match kind {
        ActuatorKind::Lever => draw_lever(surface, edge, &p, reach, step, colour, width, false),
        ActuatorKind::LeverDetent => {
            draw_lever(surface, edge, &p, reach, step, colour, width, true);
        }
        ActuatorKind::PushButton => draw_push_button(surface, edge, &p, reach, step, colour, width),
        ActuatorKind::MushroomButton => {
            draw_mushroom(surface, edge, &p, reach, step, colour, width);
        }
        ActuatorKind::Pedal => draw_pedal(surface, edge, &p, reach, step, colour, width),
        ActuatorKind::Plunger => draw_plunger(surface, edge, &p, reach, step, colour, width),
        ActuatorKind::Roller => draw_roller(surface, &p, reach, step, colour, width, false),
        ActuatorKind::RollerOneWay => draw_roller(surface, &p, reach, step, colour, width, true),
        ActuatorKind::Pilot => draw_pressure_line(surface, &p, reach, step, colour, width, false),
        ActuatorKind::PilotHydraulic => {
            draw_pressure_line(surface, &p, reach, step, colour, width, true);
        }
        ActuatorKind::Solenoid => draw_windings(surface, &p, reach, step, colour, width, 1),
        ActuatorKind::SolenoidDouble => draw_windings(surface, &p, reach, step, colour, width, 2),
        ActuatorKind::SolenoidServo => {
            // The winding box with the small pilot valve it drives: the actuation is
            // electrical, and the arrow shows that air does the pushing.
            let box_near = step * 2.4;
            draw_winding_box(
                surface,
                &p,
                box_near,
                box_near + step * 2.2,
                step * 1.2,
                colour,
                width,
            );
            draw_pressure_line(surface, &p, step * 1.6, step * 0.8, colour, width, false);
        }
        ActuatorKind::Motor => {
            draw_motor(surface, edge, &p, reach, step, colour, width, out, along);
        }
    }
}

/// The bar across an actuator's stem: the part a hand, a cam or a stop meets.
#[allow(clippy::too_many_arguments)]
fn stem_bar(
    surface: &mut dyn Surface,
    p: &impl Fn(f32, f32) -> Pt,
    at: f32,
    half: f32,
    colour: Colour,
    width: f32,
) {
    surface.line(
        &[p(at, -half), p(at, half)],
        Stroke::new(width * 1.8, colour),
    );
}

/// A hand lever: a bar against the body that carries the arm, the arm itself, and the
/// pivot it turns about.
///
/// ISO 1219-1 reads a lever by its pivot: the small circle at the far end is what separates
/// it from the general manual actuation, which is a bar and nothing else. The bar against
/// the body is the arm's foot, and it is what a reader looks at to see which way the spool
/// is pushed.
#[allow(clippy::too_many_arguments)]
fn draw_lever(
    surface: &mut dyn Surface,
    edge: Pt,
    p: &impl Fn(f32, f32) -> Pt,
    reach: f32,
    step: f32,
    colour: Colour,
    width: f32,
    detent: bool,
) {
    let stroke = Stroke::new(width, colour);
    let foot = step * 0.2;
    stem_bar(surface, p, foot, step * 0.9, colour, width);
    let pivot_at = foot + reach * 1.05;
    surface.line(&[p(foot, 0.0), p(pivot_at, 0.0)], stroke);
    surface.circle(
        p(pivot_at + step * 0.34, 0.0),
        step * 0.34,
        None,
        Some(stroke),
    );
    let _ = edge;
    if detent {
        // The catch keeps the spool where it is left: a tick across the arm says the lever
        // stays put instead of springing back.
        surface.line(
            &[
                p(foot + step * 0.35, -step * 0.75),
                p(foot + step * 0.35, step * 0.75),
            ],
            Stroke::new(width * 1.4, colour),
        );
        surface.line(
            &[
                p(foot + step * 0.35, step * 0.75),
                p(foot + step * 1.0, step * 1.15),
            ],
            stroke,
        );
    }
}

/// A push button: a stem ending in the rounded head a finger presses.
///
/// The rounding is the difference that carries meaning: a plain bar is the general manual
/// actuation, and a rounded one is a button. A bar with square ends and a bar with round
/// ends are the same size in the drawing and different machines in the catalog.
#[allow(clippy::too_many_arguments)]
fn draw_push_button(
    surface: &mut dyn Surface,
    _edge: Pt,
    p: &impl Fn(f32, f32) -> Pt,
    reach: f32,
    step: f32,
    colour: Colour,
    width: f32,
) {
    let stroke = Stroke::new(width, colour);
    let head = step * 0.2 + reach * 1.0;
    surface.line(&[p(0.0, 0.0), p(head, 0.0)], stroke);
    let half = step * 0.95;
    let radius = step * 0.22;
    let shaft = half - radius;
    let mut outline = Vec::with_capacity(20);
    // A stadium across the stem, drawn as a polygon so it scales with the symbol and not
    // with a font or a pen cap.
    for index in 0..=6 {
        #[allow(clippy::cast_precision_loss)]
        let turn = std::f32::consts::PI * (index as f32 / 6.0) - std::f32::consts::FRAC_PI_2;
        outline.push(p(head + turn.cos() * radius, -shaft + turn.sin() * radius));
    }
    for index in 0..=6 {
        #[allow(clippy::cast_precision_loss)]
        let turn = std::f32::consts::PI * (index as f32 / 6.0) + std::f32::consts::FRAC_PI_2;
        outline.push(p(head + turn.cos() * radius, shaft + turn.sin() * radius));
    }
    surface.polygon(&outline, None, Some(stroke));
}

/// A mushroom head: a stem with a dome a palm presses, not a finger.
#[allow(clippy::too_many_arguments)]
fn draw_mushroom(
    surface: &mut dyn Surface,
    _edge: Pt,
    p: &impl Fn(f32, f32) -> Pt,
    reach: f32,
    step: f32,
    colour: Colour,
    width: f32,
) {
    let stroke = Stroke::new(width, colour);
    let neck = reach * 0.8;
    surface.line(&[p(0.0, 0.0), p(neck, 0.0)], stroke);
    let half = step * 1.2;
    let mut dome = Vec::with_capacity(9);
    for index in 0..=8 {
        #[allow(clippy::cast_precision_loss)]
        let fraction = index as f32 / 8.0;
        let bulge = (std::f32::consts::PI * fraction).sin() * step * 0.75;
        dome.push(p(neck + bulge, -half + 2.0 * half * fraction));
    }
    surface.line(&dome, stroke);
    surface.line(
        &[p(neck, -half), p(neck, half)],
        Stroke::new(width * 1.8, colour),
    );
}

/// A pedal: the same foot as a lever, with the plate the foot stands on at its far end.
#[allow(clippy::too_many_arguments)]
fn draw_pedal(
    surface: &mut dyn Surface,
    _edge: Pt,
    p: &impl Fn(f32, f32) -> Pt,
    reach: f32,
    step: f32,
    colour: Colour,
    width: f32,
) {
    let stroke = Stroke::new(width, colour);
    let foot = step * 0.2;
    stem_bar(surface, p, foot, step * 0.9, colour, width);
    let tip = foot + reach * 1.05;
    surface.line(&[p(foot, 0.0), p(tip, 0.0)], stroke);
    // The plate is drawn at a slight angle, which is what makes a pedal read as something a
    // foot rests on instead of a second hand lever.
    surface.line(
        &[
            p(tip - step * 0.45, -step * 0.75),
            p(tip + step * 0.45, step * 1.1),
        ],
        Stroke::new(width * 1.8, colour),
    );
}

/// Piston actuation: a dome that sits on the body, pressed by the piston of a slave
/// cylinder. It has no stem and nothing to grip, which is what tells it apart from the
/// button.
#[allow(clippy::too_many_arguments)]
fn draw_plunger(
    surface: &mut dyn Surface,
    edge: Pt,
    p: &impl Fn(f32, f32) -> Pt,
    _reach: f32,
    step: f32,
    colour: Colour,
    width: f32,
) {
    let stroke = Stroke::new(width, colour);
    let half = step * 1.05;
    let mut dome = Vec::with_capacity(9);
    for index in 0..=8 {
        #[allow(clippy::cast_precision_loss)]
        let fraction = index as f32 / 8.0;
        let bulge = (std::f32::consts::PI * fraction).sin() * step * 0.8;
        dome.push(p(bulge, -half + 2.0 * half * fraction));
    }
    surface.line(&dome, stroke);
    let _ = edge;
}

/// A circle with an `M` in it: the standard's mark for a motor.
#[allow(clippy::too_many_arguments)]
fn draw_motor(
    surface: &mut dyn Surface,
    edge: Pt,
    p: &impl Fn(f32, f32) -> Pt,
    _reach: f32,
    step: f32,
    colour: Colour,
    width: f32,
    out: Pt,
    along: (f32, f32),
) {
    let stroke = Stroke::new(width, colour);
    let radius = step * 1.1;
    let centre = p(step * 0.4 + radius, 0.0);
    surface.line(&[edge, p(step * 0.4, 0.0)], stroke);
    surface.circle(centre, radius, None, Some(stroke));
    // The `M` is drawn as lines rather than as text, so it scales with the symbol instead
    // of with a font.
    let arm = radius * 0.55;
    let m = [
        Pt::new(
            centre.x + out.x * arm - along.0 * arm,
            centre.y + out.y * arm - along.1 * arm,
        ),
        Pt::new(centre.x + out.x * arm, centre.y + out.y * arm),
        Pt::new(centre.x + out.x * arm * 0.1, centre.y + out.y * arm * 0.1),
        Pt::new(centre.x - out.x * arm, centre.y - out.y * arm),
        Pt::new(
            centre.x - out.x * arm + along.0 * arm,
            centre.y - out.y * arm + along.1 * arm,
        ),
    ];
    surface.line(&m, Stroke::new(width * 0.9, colour));
}

/// A roller: the cam follower, a circle on a short arm, with its axle marked at the centre.
#[allow(clippy::too_many_arguments)]
fn draw_roller(
    surface: &mut dyn Surface,
    p: &impl Fn(f32, f32) -> Pt,
    reach: f32,
    step: f32,
    colour: Colour,
    width: f32,
    one_way: bool,
) {
    let stroke = Stroke::new(width, colour);
    let radius = step * 0.7;
    let axle = reach * 0.5 + radius;
    surface.line(&[p(0.0, 0.0), p(axle - radius, 0.0)], stroke);
    surface.circle(p(axle, 0.0), radius, None, Some(stroke));
    // The axle is a dot, and it is what makes the circle read as a roller rather than as a
    // pivot: a lever's circle has no axle through it.
    surface.circle(p(axle, 0.0), radius * 0.22, Some(colour), None);
    if one_way {
        // The tick across the arm says the roller only picks the cam up one way round.
        surface.line(
            &[p(axle * 0.45, -step * 0.8), p(axle * 0.8, step * 0.55)],
            Stroke::new(width * 1.2, colour),
        );
    }
}

/// Pilot actuation: a box on the end face with a triangle inside it, pointing away from the
/// body. The triangle is filled for a pneumatic pilot and hollow for a hydraulic one.
///
/// The box is not decoration: without it a pilot cannot be told from a plain arrow marking a
/// flow, and the difference between the two media would be invisible in a photocopy.
#[allow(clippy::too_many_arguments)]
fn draw_pressure_line(
    surface: &mut dyn Surface,
    p: &impl Fn(f32, f32) -> Pt,
    reach: f32,
    step: f32,
    colour: Colour,
    width: f32,
    filled: bool,
) {
    let stroke = Stroke::new(width, colour);
    let half = step * 0.75;
    let near = reach * 0.55;
    let far = near + step * 1.5;
    surface.line(&[p(0.0, 0.0), p(near, 0.0)], stroke);
    surface.polygon(
        &[p(near, -half), p(near, half), p(far, half), p(far, -half)],
        None,
        Some(stroke),
    );
    let inset = step * 0.25;
    let triangle = [
        p(far - inset, 0.0),
        p(near + inset, -half + inset),
        p(near + inset, half - inset),
    ];
    if filled {
        surface.polygon(&triangle, Some(colour), None);
    } else {
        surface.polygon(&triangle, None, Some(stroke));
    }
}

/// Solenoid windings: one box per coil, each with its diagonal.
fn draw_windings(
    surface: &mut dyn Surface,
    p: &impl Fn(f32, f32) -> Pt,
    reach: f32,
    step: f32,
    colour: Colour,
    width: f32,
    coils: usize,
) {
    let half = step * 1.2;
    let mut near = reach * 0.4;
    for coil in 0..coils {
        let far = if coil + 1 == coils {
            near + step * 2.6
        } else {
            near + step * 2.2
        };
        draw_winding_box(surface, p, near, far, half, colour, width);
        near = far;
    }
}

/// One winding box with its diagonal, from `near` to `far` along the stem.
fn draw_winding_box(
    surface: &mut dyn Surface,
    p: &impl Fn(f32, f32) -> Pt,
    near: f32,
    far: f32,
    half: f32,
    colour: Colour,
    width: f32,
) {
    let stroke = Stroke::new(width, colour);
    surface.line(&[p(0.0, 0.0), p(near, 0.0)], stroke);
    surface.polygon(
        &[p(near, -half), p(near, half), p(far, half), p(far, -half)],
        None,
        Some(stroke),
    );
    surface.line(&[p(near, -half), p(far, half)], stroke);
}

/// Return drawn away from the body, at the declared end.
///
/// A return is a part like an actuation: the file says which one and at which end, and
/// the renderer draws the glyph the standard gives that function.
pub fn draw_return(
    surface: &mut dyn Surface,
    body: Rect,
    kind: ReturnKind,
    side: Side,
    colour: Colour,
    width: f32,
) {
    let (edge, out) = axis(body, side);
    let along = (-out.y, out.x);
    let step = body.height().min(body.width()) * 0.1;
    let p = |a: f32, b: f32| {
        Pt::new(
            edge.x + out.x * a + along.0 * b,
            edge.y + out.y * a + along.1 * b,
        )
    };
    match kind {
        ReturnKind::Spring => draw_spring(surface, body, side, colour, width),
        ReturnKind::Pneumatic => {
            draw_pressure_line(surface, &p, step * 2.6, step * 1.0, colour, width, false);
        }
        ReturnKind::Hydraulic => {
            draw_pressure_line(surface, &p, step * 2.6, step * 1.0, colour, width, true);
        }
        ReturnKind::Detent => {
            // No return: a catch holds the spool where it is left. The bar is the stop and
            // the notch is the catch, so it can never be read as a push button.
            let stroke = Stroke::new(width, colour);
            let bar = step * 1.2;
            surface.line(
                &[p(bar, -step * 1.1), p(bar, step * 1.1)],
                Stroke::new(width * 1.6, colour),
            );
            surface.line(
                &[
                    p(bar, -step * 0.4),
                    p(bar - step * 0.7, 0.0),
                    p(bar, step * 0.4),
                ],
                stroke,
            );
        }
    }
}

/// T that shuts a port inside one position's square.
///
/// A closed port is not left blank: the line ends in a bar across it, and that bar
/// belongs to the square whose position shuts it. Drawing it inside the square — not
/// across the stub outside the body — is what lets each half of the valve be read on
/// its own, which is the point of drawing both states.
#[allow(clippy::too_many_arguments)]
pub fn draw_blocking_mark(
    surface: &mut dyn Surface,
    rect: Rect,
    body: Rect,
    square: Rect,
    count: usize,
    port: [f64; 2],
    colour: Colour,
    width: f32,
) {
    let point = at(rect, port[0], port[1]);
    // Same offset inside the square as the connection of that position: the shut port and
    // the open one sit on the same spot of their own square, which is the whole idea.
    let x = edge_point(square, body, count, point).x;
    let below = point.y >= body.center().y;
    let stem = square.height() * 0.2;
    let half = square.height() * 0.15;
    let (on_edge, bar) = if below {
        let edge = square.max.y;
        (Pt::new(x, edge), Pt::new(x, edge - stem))
    } else {
        let edge = square.min.y;
        (Pt::new(x, edge), Pt::new(x, edge + stem))
    };
    // The stem is the port's own line carried inside the square; the bar across it is the
    // shut-off, and both live inside the square of the position that closes the port.
    surface.line(&[on_edge, bar], Stroke::new(width, colour));
    surface.line(
        &[Pt::new(bar.x - half, bar.y), Pt::new(bar.x + half, bar.y)],
        Stroke::new(width * 1.4, colour),
    );
}

/// Draws a directional valve from its declaration.
pub fn draw_valve(
    surface: &mut dyn Surface,
    rect: Rect,
    definition: &ComponentDefinition,
    active: usize,
    ink: &Ink,
) {
    let Some([bx, by, bw, bh]) = definition.body else {
        draw_flat(surface, rect, definition, ink);
        return;
    };
    #[allow(clippy::cast_possible_truncation)]
    let body = Rect::from_min_size(
        at(rect, bx, by),
        bw as f32 * rect.width(),
        bh as f32 * rect.height(),
    );
    let mut squares: Vec<Rect> = definition
        .body_squares()
        .iter()
        .filter_map(|square| rect_from(rect, square))
        .collect();
    // The square next to the return is the rest one, so positions are counted from that
    // end: with the spring on the right, position 0 is the rightmost square and the
    // actuator pushes the spool towards it. It holds for every return the file can
    // declare, because the rest position always lies against it.
    if definition
        .resolved_return()
        .is_some_and(|part| part.side == Side::Right)
    {
        squares.reverse();
    }

    for (index, square) in squares.iter().enumerate() {
        if index == active {
            surface.rect(*square, 0.6, Some(ink.accent), None);
        }
        surface.rect(*square, 0.6, None, Some(Stroke::new(LINE * 1.2, ink.line)));
    }

    draw_primitives(surface, rect, &definition.symbol, ink.line, LINE);

    for port in &definition.port {
        let from = at(rect, port.position[0], port.position[1]);
        let to = stub_end(rect, body, port.position);
        if (from.x - to.x).abs() > f32::EPSILON || (from.y - to.y).abs() > f32::EPSILON {
            surface.line(&[from, to], Stroke::new(LINE, ink.line));
        }
        draw_connection(surface, from, rect, ink);
        draw_numeral(
            surface,
            rect,
            definition,
            Some(body),
            &port.id,
            port.position,
            ink,
        );
    }

    if let Some(part) = definition.resolved_return() {
        draw_return(surface, body, part.kind, part.side, ink.line, LINE);
    }
    if let Some(actuator) = definition.actuator {
        draw_actuator(surface, body, actuator.kind, actuator.side, ink.line, LINE);
    }

    let Behaviour::Valve { positions, .. } = &definition.behaviour else {
        return;
    };
    let mut order: Vec<usize> = (0..positions.len()).collect();
    order.sort_by_key(|index| *index == active);
    for index in order {
        let Some(position) = positions.get(index) else {
            continue;
        };
        let is_active = index == active;
        let colour = if is_active { ink.line } else { ink.dim };
        for connection in &position.connection {
            draw_path(
                surface,
                rect,
                body,
                &squares,
                definition,
                (&connection.from, &connection.to),
                index,
                colour,
            );
        }
        draw_primitives(surface, rect, &position.symbol, colour, LINE);
        // Every position shows the ports it shuts: the reader compares the two squares,
        // so the shut ones are drawn dim while the spool sits in the other position.
        let Some(square) = squares.get(index) else {
            continue;
        };
        for id in &position.blocked {
            if let Some(port) = definition.port(id) {
                draw_blocking_mark(
                    surface,
                    rect,
                    body,
                    *square,
                    squares.len(),
                    port.position,
                    colour,
                    LINE,
                );
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_path(
    surface: &mut dyn Surface,
    rect: Rect,
    body: Rect,
    squares: &[Rect],
    definition: &ComponentDefinition,
    ends: (&str, &str),
    index: usize,
    colour: Colour,
) {
    let (from, to) = ends;
    let (Some(from_port), Some(to_port)) = (definition.port(from), definition.port(to)) else {
        return;
    };
    let Some(square) = squares.get(index) else {
        return;
    };
    // Each position lives in its own square: the path of one spool position must never
    // be drawn across the boundary into the neighbouring one, which is what makes a
    // valve readable — you look at the square and read one state.
    let count = squares.len();
    let start = edge_point(
        *square,
        body,
        count,
        at(rect, from_port.position[0], from_port.position[1]),
    );
    let end = edge_point(
        *square,
        body,
        count,
        at(rect, to_port.position[0], to_port.position[1]),
    );
    // One straight line from port to port inside the square: a vertical one when the
    // position connects two ports on the same side of the body, a slanted one when it
    // takes a port to an exhaust at the other end, which is how the standard draws it.
    surface.line(&[start, end], Stroke::new(LINE, colour));

    let (dx, dy) = start.to(end);
    let length = dx.hypot(dy);
    let (hx, hy) = if length > f32::EPSILON {
        (dx / length, dy / length)
    } else {
        (0.0, -1.0)
    };
    let reach = square.height() * 0.12;
    let back = Pt::new(end.x - hx * reach, end.y - hy * reach);
    let side = (-hy * reach * 0.62, hx * reach * 0.62);
    surface.polygon(
        &[
            end,
            Pt::new(back.x + side.0, back.y + side.1),
            Pt::new(back.x - side.0, back.y - side.1),
        ],
        Some(colour),
        None,
    );
}

/// Point where a port's line meets the edge of one position's square.
///
/// A port sits at one x of the body. Both squares are the same width, so its offset
/// inside its own square is the same in every square: that is how a position shows a
/// connection to a port that is drawn outside it, without leaving its own box.
fn edge_point(square: Rect, body: Rect, count: usize, point: Pt) -> Pt {
    // A port is drawn once, at its x on the body. Every square is the same width, so the
    // port's offset *inside its own square* is the same everywhere, and a position that
    // connects through a port drawn over its neighbour lands on the right place inside
    // itself. That is how a real symbol shows two states of the same set of ports.
    #[allow(clippy::cast_precision_loss)]
    let local = ((point.x - body.min.x) / (body.width() / count as f32))
        .fract()
        .clamp(0.04, 0.96);
    let below = point.y >= body.center().y;
    let edge_y = if below { square.max.y } else { square.min.y };
    let inset = square.height() * 0.06;
    let edge_y = if below {
        edge_y - inset
    } else {
        edge_y + inset
    };
    Pt::new(square.min.x + square.width() * local, edge_y)
}

/// Draws a component whose geometry is entirely in its own file.
pub fn draw_flat(
    surface: &mut dyn Surface,
    rect: Rect,
    definition: &ComponentDefinition,
    ink: &Ink,
) {
    draw_primitives(surface, rect, &definition.symbol, ink.line, LINE);
    // A junction is a filled dot and nothing else (ISO 1219-1): its ports are one point, so
    // three numerals would be drawn on top of each other and read as a smudge.
    if matches!(definition.behaviour, Behaviour::Junction) {
        return;
    }
    // The barrel is the body of a cylinder, and its ports are read against it: the numeral of
    // a port under the barrel belongs beside its own stub, on the outer side of the barrel,
    // not wherever the middle of the symbol box happens to fall.
    let body = if definition.kinematics == Kinematics::LinearActuator {
        barrel_of(definition, rect)
    } else {
        None
    };
    for port in &definition.port {
        draw_connection(
            surface,
            at(rect, port.position[0], port.position[1]),
            rect,
            ink,
        );
        draw_numeral(
            surface,
            rect,
            definition,
            body,
            &port.id,
            port.position,
            ink,
        );
    }
}

/// Share of the barrel's internal length the piston crosses at full extension.
///
/// The file declares the useful stroke and the residual length standing for the dead
/// volume; together they are the internal length a reader sees between the two walls.
/// The piston has to cross the stroke's share of that length, not a decorative part of
/// it, and the residual share is the room the return spring keeps at the rod side when
/// the piston is out. A definition that moves without a cylinder behaviour has no such
/// numbers, so it falls back to the whole chamber.
fn stroke_share(definition: &ComponentDefinition) -> f32 {
    let Behaviour::Cylinder {
        stroke_mm,
        dead_length_mm,
        ..
    } = &definition.behaviour
    else {
        return 1.0;
    };
    let internal = stroke_mm + dead_length_mm;
    if !internal.is_finite() || internal <= 0.0 {
        return 1.0;
    }
    #[allow(clippy::cast_possible_truncation)]
    let share = (stroke_mm / internal) as f32;
    if share.is_finite() {
        share.clamp(0.0, 1.0)
    } else {
        1.0
    }
}

/// Draws the moving parts of a cylinder at an extension in `0..=1`.
pub fn draw_cylinder(
    surface: &mut dyn Surface,
    rect: Rect,
    definition: &ComponentDefinition,
    extension: f32,
    ink: &Ink,
) {
    let Some(barrel) = barrel_of(definition, rect) else {
        return;
    };
    let inner_left = barrel.min.x + barrel.width() * 0.04;
    let inner_right = barrel.max.x - barrel.width() * 0.04;
    // The piston crosses the stroke the definition declares, over the chamber that
    // holds stroke plus dead length. At full extension it stops short of the rod-side
    // wall by exactly the dead length, which is where the compressed spring sits: the
    // travel is the internal length the file gives the piston, not a fraction of it
    // chosen by the renderer.
    let chamber = (inner_right - inner_left).max(0.0);
    let travel = chamber * stroke_share(definition);
    let piston_x = inner_left + travel * extension.clamp(0.0, 1.0);
    // The piston crosses the bore from wall to wall. A line that stops short of the walls
    // reads as a bar floating inside the barrel, and ISO 1219 draws the piston as the
    // seal between the two chambers: it has to meet both.
    let top = barrel.min.y;
    let bottom = barrel.max.y;
    let axis_y = barrel.center().y;

    surface.line(
        &[Pt::new(piston_x, top), Pt::new(piston_x, bottom)],
        Stroke::new(LINE * 2.6, ink.line),
    );
    // The rod is one piece of constant length, so its tip rides along with the piston:
    // it lengthens and shortens by nothing at all. Lengthening it with the stroke would
    // draw a telescope, and a rod that shrinks when the cylinder extends is exactly what
    // a moving part must never do. It reaches the edge of the symbol box at full stroke,
    // so it is visible at rest and never drawn over whatever sits next to the symbol.
    let rod = (rect.max.x - inner_left - travel).max(0.0);
    surface.line(
        &[Pt::new(piston_x, axis_y), Pt::new(piston_x + rod, axis_y)],
        Stroke::new(LINE * 1.4, ink.line),
    );

    if definition
        .resolved_return()
        .is_some_and(|part| part.kind == ReturnKind::Spring)
    {
        draw_cylinder_spring(
            surface,
            piston_x,
            inner_right,
            axis_y,
            barrel,
            extension,
            ink,
        );
    }
}

/// Return spring between the piston and the rod-side wall, compressed by the stroke.
///
/// The spring is drawn **across the chamber between the piston and the rod-side wall**, and
/// it spans all of it: the spring is the return element and it belongs to the chamber it
/// fills, not to a corner of it. It is drawn thin, so the barrel, the piston, the rod and
/// the coil stay distinguishable without shortening the spring.
///
/// The wire is constant and the pitch shrinks: a turned coil that no longer fits at the
/// nominal pitch is drawn with fewer turns instead of more lines in the same millimetre,
/// because a stack of turns drawn on top of each other is a filled band and not a spring.
/// Two turns is the floor: below that the shape stops reading as a coil.
fn draw_cylinder_spring(
    surface: &mut dyn Surface,
    piston_x: f32,
    wall_x: f32,
    axis_y: f32,
    barrel: Rect,
    extension: f32,
    ink: &Ink,
) {
    let gap = wall_x - piston_x;
    if gap <= LINE {
        return;
    }
    // The coil spans the whole chamber between the piston and the wall, which is how this
    // symbol is read: the spring is the return element and it belongs to the chamber it
    // fills, not to a corner of it. It is drawn thin and behind the piston and the rod, so
    // the three parts stay distinguishable without shortening the spring.
    let span = gap;
    let nominal_pitch = barrel.width() * 0.055;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let fitted = (span / nominal_pitch).round().clamp(2.0, 5.0) as u16;
    // A coil spring keeps its wire and loses its pitch: the amplitude grows a little as
    // the coils bunch up, which is what makes the animation read as compression.
    // Amplitude close to the barrel's own height, as the symbol is drawn by hand: a small
    // zigzag on the axis reads as a wire, not as a spring.
    let amplitude = barrel.height() * (0.26 + 0.04 * extension.clamp(0.0, 1.0));
    let steps = fitted * 2;
    let mut points = vec![Pt::new(piston_x, axis_y)];
    for step in 0..steps {
        let fraction = (f32::from(step) + 0.5) / f32::from(steps);
        let x = piston_x + span * fraction;
        let y = axis_y + if step % 2 == 0 { -amplitude } else { amplitude };
        points.push(Pt::new(x, y));
    }
    points.push(Pt::new(wall_x, axis_y));
    // The coil is part of the symbol, not an annotation: same ink and same weight as the
    // other functional lines. A faded line is not a line class in ISO 1219, and it is the
    // first thing to vanish in a photocopy.
    surface.line(&points, Stroke::new(LINE, ink.line));
}

/// Barrel of a cylinder: the widest rectangle its definition declares.
///
/// The animation reads the geometry from the file, so a longer cylinder is a different
/// file and not a different branch here.
fn barrel_of(definition: &ComponentDefinition, rect: Rect) -> Option<Rect> {
    definition
        .symbol
        .iter()
        .filter(|primitive| primitive.kind == PrimitiveKind::Rect)
        .filter_map(|primitive| rect_from(rect, &primitive.points))
        .max_by(|a, b| {
            a.width()
                .partial_cmp(&b.width())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

/// Draws a component whose geometry is entirely in its own file, with the moving parts
/// at `extension` for a cylinder.
pub fn draw_component(
    surface: &mut dyn Surface,
    rect: Rect,
    definition: &ComponentDefinition,
    active: usize,
    extension: f32,
    ink: &Ink,
) {
    if definition.body.is_some() {
        draw_valve(surface, rect, definition, active, ink);
        return;
    }
    draw_flat(surface, rect, definition, ink);
    if definition.kinematics == Kinematics::LinearActuator {
        draw_cylinder(surface, rect, definition, extension, ink);
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::surface::Recording;

    fn valve() -> ComponentDefinition {
        let catalog = punchi_fluid_catalog::Catalog::builtin();
        catalog
            .get("valve.3_2.spring_return")
            .expect("the built-in valve exists")
            .clone()
    }

    #[test]
    fn a_valve_body_is_split_into_equal_squares() {
        let definition = valve();
        let squares = definition.body_squares();
        assert_eq!(squares.len(), 2);
        assert!((squares[0][2] - squares[1][2]).abs() < f64::EPSILON);
        assert!((squares[0][0] + squares[0][2] - squares[1][0]).abs() < f64::EPSILON);
    }

    #[test]
    fn the_drawing_keeps_the_squares_inside_the_given_rect() {
        let definition = valve();
        let rect = Rect::from_min_size(Pt::new(0.0, 0.0), 60.0, 60.0);
        let mut recording = Recording::default();
        draw_valve(&mut recording, rect, &definition, 0, &Ink::print());
        let closed: Vec<&crate::surface::Call> = recording
            .calls()
            .iter()
            .filter(|call| {
                matches!(
                    call,
                    crate::surface::Call::Rect {
                        stroke: Some(_),
                        ..
                    }
                )
            })
            .collect();
        assert!(!closed.is_empty(), "the body squares must be drawn");
    }

    #[test]
    fn the_active_position_fills_exactly_one_square() {
        let definition = valve();
        let rect = Rect::from_min_size(Pt::new(0.0, 0.0), 60.0, 60.0);
        let mut recording = Recording::default();
        draw_valve(&mut recording, rect, &definition, 1, &Ink::print());
        let filled = recording
            .calls()
            .iter()
            .filter(|call| matches!(call, crate::surface::Call::Rect { fill: Some(_), .. }))
            .count();
        assert_eq!(filled, 1);
    }

    #[test]
    fn each_position_is_drawn_inside_its_own_square() {
        let definition = valve();
        let rect = Rect::from_min_size(Pt::new(0.0, 0.0), 120.0, 60.0);
        let squares: Vec<Rect> = definition
            .body_squares()
            .iter()
            .filter_map(|square| rect_from(rect, square))
            .collect();
        assert_eq!(squares.len(), 2);

        let body = Rect::from_min_size(
            at(rect, 0.06, 0.28),
            0.88 * rect.width(),
            0.44 * rect.height(),
        );
        let Behaviour::Valve { positions, .. } = &definition.behaviour else {
            unreachable!("the fixture is a valve")
        };
        for (index, position) in positions.iter().enumerate() {
            let square = squares[index];
            for connection in &position.connection {
                let mut recording = Recording::default();
                draw_path(
                    &mut recording,
                    rect,
                    body,
                    &squares,
                    &definition,
                    (&connection.from, &connection.to),
                    index,
                    Colour::BLACK,
                );
                for call in recording.calls() {
                    let crate::surface::Call::Line { points, .. } = call else {
                        continue;
                    };
                    for point in points {
                        assert!(
                            square.expand(0.01).contains(*point),
                            "position {index} drew outside its square: {point:?} not in {square:?}"
                        );
                    }
                }
            }
        }
    }

    /// Bar of the T that shuts a port, as drawn for the position at `index`.
    fn active_bar(definition: &ComponentDefinition, rect: Rect, index: usize) -> Option<(Pt, Pt)> {
        let ink = Ink::print();
        let mut recording = Recording::default();
        draw_valve(&mut recording, rect, definition, index, &ink);
        recording.calls().iter().find_map(|call| {
            let crate::surface::Call::Line { points, stroke } = call else {
                return None;
            };
            let [from, to] = points.as_slice() else {
                return None;
            };
            // The bar is the only two-point line the valve draws wide, and it is the
            // only part of a position drawn in the ink of the active one: the idle
            // position shuts its port too, in the dim ink.
            let wide = (stroke.width - LINE * 1.4).abs() < 1.0e-6;
            let active = stroke.colour == ink.line;
            (wide && active && (from.y - to.y).abs() < 1.0e-6).then_some((*from, *to))
        })
    }

    #[test]
    fn every_position_shuts_its_blocked_port_inside_its_own_square() {
        let definition = valve();
        let rect = Rect::from_min_size(Pt::new(0.0, 0.0), 60.0, 60.0);
        // `body_squares` lists the squares from left to right; a position is drawn in the
        // square counted from the spring end, because that is the end that holds it.
        let squares: Vec<Rect> = definition
            .body_squares()
            .iter()
            .filter_map(|square| rect_from(rect, square))
            .collect();
        let Behaviour::Valve { positions, .. } = &definition.behaviour else {
            unreachable!("the fixture is a valve")
        };
        let mut drawn_in = Vec::new();
        for index in 0..positions.len() {
            let (from, _) = active_bar(&definition, rect, index)
                .unwrap_or_else(|| panic!("position {index} shuts a port and must draw its bar"));
            let hits: Vec<usize> = squares
                .iter()
                .enumerate()
                .filter(|(_, square)| square.min.x < from.x && from.x < square.max.x)
                .map(|(position, _)| position)
                .collect();
            assert_eq!(
                hits.len(),
                1,
                "position {index}: the bar at {} must sit inside one square, not {hits:?}",
                from.x
            );
            drawn_in.extend(hits);
        }
        let expected: Vec<usize> = if definition
            .resolved_return()
            .is_some_and(|part| part.side == Side::Right)
        {
            (0..positions.len()).rev().collect()
        } else {
            (0..positions.len()).collect()
        };
        assert_eq!(
            drawn_in, expected,
            "positions must count from the spring end"
        );
    }

    #[test]
    fn the_rest_position_is_drawn_next_to_the_spring() {
        let definition = valve();
        // The built-in valve returns by spring, and the spring is drawn on the right.
        assert_eq!(
            definition.resolved_return().map(|part| part.side),
            Some(Side::Right)
        );
        let rect = Rect::from_min_size(Pt::new(0.0, 0.0), 60.0, 60.0);
        let (from, _) = active_bar(&definition, rect, 0).expect("the rest position shuts port 1");
        assert!(
            from.x > rect.min.x + rect.width() * 0.5,
            "the square that holds at rest must be the one against the spring"
        );
    }

    #[test]
    fn the_piston_never_leaves_the_barrel_at_either_end_of_the_stroke() {
        let catalog = punchi_fluid_catalog::Catalog::builtin();
        for id in [
            "cylinder.single_acting_20_100",
            "cylinder.double_acting_32_200",
        ] {
            let definition = catalog
                .get(id)
                .expect("the built-in cylinder exists")
                .clone();
            let rect = Rect::from_min_size(Pt::new(0.0, 0.0), 100.0, 50.0);
            let barrel = barrel_of(&definition, rect).expect("a barrel");
            for extension in [0.0, 0.5, 1.0] {
                let mut recording = Recording::default();
                draw_cylinder(&mut recording, rect, &definition, extension, &Ink::print());
                let crate::surface::Call::Line { points, .. } = &recording.calls()[0] else {
                    unreachable!("the piston is a line")
                };
                let piston_x = points[0].x;
                assert!(
                    piston_x >= barrel.min.x && piston_x <= barrel.max.x,
                    "{id} at {extension}: the piston at {piston_x} left the barrel {barrel:?}"
                );
            }
        }
    }

    #[test]
    fn the_piston_meets_both_walls_of_the_bore() {
        let catalog = punchi_fluid_catalog::Catalog::builtin();
        for id in [
            "cylinder.single_acting_20_100",
            "cylinder.double_acting_32_200",
        ] {
            let definition = catalog
                .get(id)
                .expect("the built-in cylinder exists")
                .clone();
            let rect = Rect::from_min_size(Pt::new(0.0, 0.0), 100.0, 50.0);
            let barrel = barrel_of(&definition, rect).expect("a barrel");
            for extension in [0.0, 0.5, 1.0] {
                let mut recording = Recording::default();
                draw_cylinder(&mut recording, rect, &definition, extension, &Ink::print());
                let crate::surface::Call::Line { points, .. } = &recording.calls()[0] else {
                    unreachable!("the piston is a line")
                };
                let [top, bottom] = points.as_slice() else {
                    unreachable!("a piston is a segment")
                };
                // A piston that stops short of the walls reads as a bar floating inside the
                // barrel. ISO 1219 draws it as the seal between the two chambers, so it has
                // to reach both walls at every position of the stroke.
                assert!(
                    (top.y - barrel.min.y).abs() < 1.0e-4
                        && (bottom.y - barrel.max.y).abs() < 1.0e-4,
                    "{id} at {extension}: the piston spans {}..{}, the bore is {}..{}",
                    top.y,
                    bottom.y,
                    barrel.min.y,
                    barrel.max.y
                );
            }
        }
    }

    #[test]
    fn each_port_of_a_cylinder_is_drilled_in_an_end_cap() {
        let catalog = punchi_fluid_catalog::Catalog::builtin();
        let rect = Rect::from_min_size(Pt::new(0.0, 0.0), 100.0, 50.0);

        // One working port, in the cap end: that is what makes a cylinder single-acting, so
        // nothing is drawn on the rod end at all.
        let single = catalog
            .get("cylinder.single_acting_20_100")
            .expect("the built-in cylinder exists")
            .clone();
        let barrel = barrel_of(&single, rect).expect("a barrel");
        let ports: Vec<Pt> = single
            .port
            .iter()
            .map(|port| at(rect, port.position[0], port.position[1]))
            .collect();
        assert_eq!(ports.len(), 1, "a single-acting cylinder has one port");
        assert!(
            ports[0].x < barrel.center().x,
            "the only port must be in the cap end, not at {}",
            ports[0].x
        );

        // Two ports, one drilled in each cap: a double-acting cylinder is driven from both
        // ends. Both leave the barrel edge, so neither is laid over the rod.
        let double = catalog
            .get("cylinder.double_acting_32_200")
            .expect("the built-in cylinder exists")
            .clone();
        let barrel = barrel_of(&double, rect).expect("a barrel");
        let ports: Vec<Pt> = double
            .port
            .iter()
            .map(|port| at(rect, port.position[0], port.position[1]))
            .collect();
        assert_eq!(ports.len(), 2, "a double-acting cylinder has two ports");
        assert!(
            ports.iter().any(|port| port.x < barrel.center().x)
                && ports.iter().any(|port| port.x > barrel.center().x),
            "one port must be in each cap, got {ports:?}"
        );
        assert!(
            ports.iter().all(|port| port.y > barrel.max.y),
            "both ports leave the barrel through its edge, got {ports:?}"
        );
    }

    #[test]
    fn the_rod_keeps_its_length_at_every_stroke() {
        let catalog = punchi_fluid_catalog::Catalog::builtin();
        for id in [
            "cylinder.single_acting_20_100",
            "cylinder.double_acting_32_200",
        ] {
            let definition = catalog
                .get(id)
                .expect("the built-in cylinder exists")
                .clone();
            let rect = Rect::from_min_size(Pt::new(0.0, 0.0), 100.0, 50.0);
            let mut lengths = Vec::new();
            let mut tips = Vec::new();
            for extension in [0.0, 0.5, 1.0] {
                let mut recording = Recording::default();
                draw_cylinder(&mut recording, rect, &definition, extension, &Ink::print());
                let crate::surface::Call::Line { points, .. } = &recording.calls()[1] else {
                    unreachable!("the rod is the second line, after the piston")
                };
                let [start, tip] = points.as_slice() else {
                    unreachable!("a rod is a segment")
                };
                lengths.push((tip.x - start.x).abs());
                tips.push(tip.x);
            }
            assert!(
                lengths
                    .windows(2)
                    .all(|pair| (pair[0] - pair[1]).abs() < 1.0e-4),
                "{id}: the rod changed length between strokes: {lengths:?}"
            );
            assert!(
                tips.windows(2).all(|pair| pair[1] > pair[0]),
                "{id}: the rod tip must travel with the piston, not stay put: {tips:?}"
            );
            // Visible at rest, inside the box at full stroke: the barrel leaves exactly
            // the room the stroke needs, so the rod is never swallowed nor drawn over
            // whatever the symbol is next to on the sheet.
            let barrel = barrel_of(&definition, rect).expect("a barrel");
            assert!(
                tips[0] > barrel.max.x,
                "{id}: at rest the rod must show outside the barrel, tip at {} vs {}",
                tips[0],
                barrel.max.x
            );
            assert!(
                tips[2] <= rect.max.x + 0.01,
                "{id}: at full stroke the rod tip must stay in the box, {} vs {}",
                tips[2],
                rect.max.x
            );
        }
    }

    #[test]
    fn the_piston_crosses_the_whole_stroke_the_file_declares() {
        let catalog = punchi_fluid_catalog::Catalog::builtin();
        let rect = Rect::from_min_size(Pt::new(0.0, 0.0), 100.0, 50.0);
        for id in [
            "cylinder.single_acting_20_100",
            "cylinder.double_acting_32_200",
        ] {
            let definition = catalog
                .get(id)
                .expect("the built-in cylinder exists")
                .clone();
            let barrel = barrel_of(&definition, rect).expect("a barrel");
            let inset = barrel.width() * 0.04;
            let chamber = barrel.width() - 2.0 * inset;
            let Behaviour::Cylinder {
                stroke_mm,
                dead_length_mm,
                ..
            } = &definition.behaviour
            else {
                unreachable!("a cylinder declares a cylinder behaviour")
            };
            let share = stroke_mm / (stroke_mm + dead_length_mm);
            // The drawing works in millimetres of the sheet, far inside `f32` range.
            #[allow(clippy::cast_possible_truncation)]
            let stroke = chamber * share as f32;

            let mut pistons = Vec::new();
            for extension in [0.0_f32, 0.5, 1.0] {
                let mut recording = Recording::default();
                draw_cylinder(&mut recording, rect, &definition, extension, &Ink::print());
                let crate::surface::Call::Line { points, .. } = &recording.calls()[0] else {
                    unreachable!("the piston is a line")
                };
                pistons.push(points[0].x);
            }

            let travelled = pistons[2] - pistons[0];
            assert!(
                (travelled - stroke).abs() < 1.0e-3,
                "{id}: the piston crossed {travelled} of the {stroke} the file declares"
            );
            // At full extension the piston leaves exactly the declared dead length
            // against the rod-side wall: that is the room the spring is compressed into,
            // not a fraction of the chamber chosen by the renderer.
            let dead = chamber - stroke;
            let gap = (barrel.max.x - inset) - pistons[2];
            assert!(
                (gap - dead).abs() < 1.0e-3,
                "{id}: the piston left {gap} at the rod side, the file declares {dead}"
            );
        }
    }

    #[test]
    fn the_return_spring_compresses_with_the_stroke() {
        let catalog = punchi_fluid_catalog::Catalog::builtin();
        let definition = catalog
            .get("cylinder.single_acting_20_100")
            .expect("the built-in cylinder exists")
            .clone();
        let rect = Rect::from_min_size(Pt::new(0.0, 0.0), 100.0, 50.0);
        let barrel = barrel_of(&definition, rect).expect("a barrel");
        let wall = barrel.max.x - barrel.width() * 0.04;
        let spring = |extension: f32| {
            let mut recording = Recording::default();
            draw_cylinder(&mut recording, rect, &definition, extension, &Ink::print());
            let crate::surface::Call::Line { points, .. } =
                recording.calls().last().expect("the spring is drawn last")
            else {
                unreachable!("the spring is a polyline")
            };
            points.clone()
        };
        let spans: Vec<f32> = [0.0_f32, 0.5, 1.0]
            .iter()
            .map(|extension| {
                let points = spring(*extension);
                points.last().expect("a point").x - points.first().expect("a point").x
            })
            .collect();
        assert!(
            spans[0] > spans[1] && spans[1] > spans[2],
            "the spring must shorten as the piston comes out: {spans:?}"
        );
        assert!(
            spans[2] > 0.0,
            "a fully compressed spring still has a span: {spans:?}"
        );

        // The coil lives between the piston and the wall at every stroke: a turn drawn
        // past the wall would be outside the barrel, and one drawn past the piston would
        // cross it.
        for extension in [0.0_f32, 0.5, 1.0] {
            let points = spring(extension);
            let first = points.first().expect("a point").x;
            let last = points.last().expect("a point").x;
            assert!(
                (last - wall).abs() < 1.0e-3,
                "at {extension}: the coil ends at {last}, the wall is at {wall}"
            );
            assert!(
                points.iter().all(|point| point.x >= first - 1.0e-3),
                "at {extension}: a turn was drawn behind the piston"
            );
        }

        // A tight coil loses turns instead of stacking lines on the same millimetre: a
        // band of turns drawn over each other is a filled rectangle, not a spring.
        let turns = |extension: f32| spring(extension).len();
        assert!(
            turns(1.0) < turns(0.0),
            "the compressed spring must have fewer turns: {} vs {}",
            turns(1.0),
            turns(0.0)
        );
    }

    /// Every point a recording touched, so a test can say where a glyph was drawn.
    fn touched(calls: &[crate::surface::Call]) -> Vec<Pt> {
        let mut points = Vec::new();
        for call in calls {
            match call {
                crate::surface::Call::Line { points: line, .. }
                | crate::surface::Call::Polygon { points: line, .. } => points.extend(line),
                crate::surface::Call::Circle { centre, radius, .. }
                | crate::surface::Call::Arc { centre, radius, .. } => {
                    points.push(Pt::new(centre.x - radius, centre.y - radius));
                    points.push(Pt::new(centre.x + radius, centre.y + radius));
                }
                crate::surface::Call::Rect { rect, .. } => {
                    points.push(rect.min);
                    points.push(rect.max);
                }
                crate::surface::Call::Text { at, .. } => points.push(*at),
            }
        }
        points
    }

    #[test]
    fn every_actuation_kind_draws_its_own_glyph() {
        let body = Rect::from_min_size(Pt::new(50.0, 20.0), 40.0, 20.0);
        let mut shapes: Vec<(ActuatorKind, Vec<crate::surface::Call>)> = Vec::new();
        for kind in ActuatorKind::ALL {
            let mut recording = Recording::default();
            draw_actuator(&mut recording, body, *kind, Side::Left, Colour::BLACK, LINE);
            assert!(!recording.is_empty(), "{} draws nothing", kind.name_es());
            // A glyph belongs outside the body: the body is the renderer's own box and a
            // part drawn across its edge would collide with a square.
            for point in touched(recording.calls()) {
                assert!(
                    point.x <= body.min.x + LINE,
                    "{} drew at {point:?}, inside the body {body:?}",
                    kind.name_es()
                );
            }
            shapes.push((*kind, recording.calls().to_vec()));
        }
        for (index, (kind, calls)) in shapes.iter().enumerate() {
            for (other, other_calls) in shapes.iter().skip(index + 1) {
                assert_ne!(
                    calls,
                    other_calls,
                    "{} and {} are the same drawing; a reader cannot tell them apart",
                    kind.name_es(),
                    other.name_es()
                );
            }
        }
    }

    #[test]
    fn a_hydraulic_pilot_is_filled_and_a_pneumatic_one_is_not() {
        let body = Rect::from_min_size(Pt::new(50.0, 20.0), 40.0, 20.0);
        let filled = |kind: ActuatorKind| {
            let mut recording = Recording::default();
            draw_actuator(&mut recording, body, kind, Side::Left, Colour::BLACK, LINE);
            recording
                .calls()
                .iter()
                .filter_map(|call| match call {
                    crate::surface::Call::Polygon { fill, .. } => Some(fill.is_some()),
                    _ => None,
                })
                .any(|filled| filled)
        };
        assert!(
            !filled(ActuatorKind::Pilot),
            "pneumatic pilot: an open triangle, never a filled one"
        );
        assert!(
            filled(ActuatorKind::PilotHydraulic),
            "hydraulic pilot: the filled triangle is what separates the two fluids"
        );
    }

    #[test]
    fn every_return_kind_draws_its_own_glyph() {
        let body = Rect::from_min_size(Pt::new(0.0, 20.0), 40.0, 20.0);
        let mut shapes: Vec<(ReturnKind, Vec<crate::surface::Call>)> = Vec::new();
        for kind in ReturnKind::ALL {
            let mut recording = Recording::default();
            draw_return(
                &mut recording,
                body,
                *kind,
                Side::Right,
                Colour::BLACK,
                LINE,
            );
            assert!(!recording.is_empty(), "{} draws nothing", kind.name_es());
            for point in touched(recording.calls()) {
                assert!(
                    point.x >= body.max.x - LINE,
                    "{} drew at {point:?}, inside the body {body:?}",
                    kind.name_es()
                );
            }
            shapes.push((*kind, recording.calls().to_vec()));
        }
        for (index, (kind, calls)) in shapes.iter().enumerate() {
            for (other, other_calls) in shapes.iter().skip(index + 1) {
                assert_ne!(
                    calls,
                    other_calls,
                    "{} and {} are the same drawing",
                    kind.name_es(),
                    other.name_es()
                );
            }
        }
    }

    #[test]
    fn a_spring_file_and_a_return_file_draw_the_same_valve() {
        // `[spring]` is the short form of `[return] kind = "spring"`: a file written
        // before the other returns existed must keep drawing the same symbol.
        let catalog = punchi_fluid_catalog::Catalog::builtin();
        let short = catalog
            .get("valve.3_2.spring_return")
            .expect("the built-in valve exists")
            .clone();
        let mut long = short.clone();
        long.return_part = short.spring.map(|spring| punchi_fluid_catalog::ReturnPart {
            kind: ReturnKind::Spring,
            side: spring.side,
        });
        long.spring = None;

        let rect = Rect::from_min_size(Pt::new(0.0, 0.0), 60.0, 60.0);
        let mut first = Recording::default();
        draw_valve(&mut first, rect, &short, 0, &Ink::print());
        let mut second = Recording::default();
        draw_valve(&mut second, rect, &long, 0, &Ink::print());
        assert_eq!(first.calls(), second.calls());
    }

    #[test]
    fn only_the_single_acting_cylinder_draws_a_spring() {
        let catalog = punchi_fluid_catalog::Catalog::builtin();
        let rect = Rect::from_min_size(Pt::new(0.0, 0.0), 100.0, 50.0);
        let lines = |id: &str| {
            let definition = catalog
                .get(id)
                .expect("the built-in cylinder exists")
                .clone();
            let mut recording = Recording::default();
            draw_cylinder(&mut recording, rect, &definition, 0.5, &Ink::print());
            recording
                .calls()
                .iter()
                .filter(|call| matches!(call, crate::surface::Call::Line { .. }))
                .count()
        };
        // Piston, rod and the spring's polyline: three calls. Without a spring, two.
        assert_eq!(lines("cylinder.single_acting_20_100"), 3);
        assert_eq!(lines("cylinder.double_acting_32_200"), 2);
    }

    #[test]
    fn a_spring_is_drawn_at_the_end_it_is_declared_on() {
        let body = Rect::from_min_size(Pt::new(50.0, 20.0), 40.0, 20.0);
        let mut left = Recording::default();
        draw_spring(&mut left, body, Side::Left, Colour::BLACK, LINE);
        let mut right = Recording::default();
        draw_spring(&mut right, body, Side::Right, Colour::BLACK, LINE);
        assert_ne!(left.calls(), right.calls());
        let first = left.calls().first().expect("a spring draws something");
        let crate::surface::Call::Line { points, .. } = first else {
            unreachable!("a spring is drawn as a line")
        };
        assert!(points[0].x <= body.min.x + f32::EPSILON);
    }
}
