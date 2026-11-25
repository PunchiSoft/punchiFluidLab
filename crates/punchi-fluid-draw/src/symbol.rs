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
    MovingPart, Primitive, PrimitiveKind, ReturnKind, Side,
};
use punchi_fluid_solver::network::BallSeat;

use crate::surface::{Anchor, Colour, LineClass, Pt, Rect, Stroke, Surface, dashes};

/// Width of a symbol line, in millimetres of the drawing.
pub const LINE: f32 = 0.35;
/// Cap height of a port numeral.
pub const NUMERAL: f32 = 3.0;
/// Smallest symbol box that still shows its numerals.
pub const NUMERAL_FLOOR: f32 = 12.0;

/// Box a definition occupies on the sheet, in millimetres.
///
/// The file declares it with `box = [width, height]`; a file without the key keeps the
/// canonical [`crate::schematic::SYMBOL_BOX_MM`] square. The palette, the drop and the
/// templates all read this one function, so a symbol cannot take one size when it is
/// dragged and another when a template places it.
#[must_use]
pub fn symbol_box_mm(definition: &ComponentDefinition) -> (f32, f32) {
    definition.declared_box_mm().map_or(
        (
            crate::schematic::SYMBOL_BOX_MM,
            crate::schematic::SYMBOL_BOX_MM,
        ),
        |[width, height]| (width, height),
    )
}
/// Share of a cylinder barrel's length taken by each end cap, as a fraction of its width.
///
/// A cylinder's working port is drilled in an end cap, so the chamber the piston crosses
/// starts at the cap's inner face and the port sits between the end wall and that face.
/// The figure `X11440`/`X11450` puts the face about a tenth of the barrel's length in and
/// the port halfway between the two; the previous one-twenty-fifth inset was too thin to
/// hold a port and left it inside the swept chamber.
pub const CYLINDER_CAP_SHARE: f32 = 0.11;

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
        Some(CatalogLineClass::Boundary) => LineClass::Boundary,
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
    draw_primitives_seated(surface, rect, primitives, None, colour, width);
}

/// Draws the primitives and places the declared moving part on its seat.
///
/// The part is the ball of a shuttle: the file names the primitive and the three seats, and
/// the live state chooses one. The radius stays the file's; only the centre moves.
pub fn draw_primitives_seated(
    surface: &mut dyn Surface,
    rect: Rect,
    primitives: &[Primitive],
    moving: Option<(&MovingPart, BallSeat)>,
    colour: Colour,
    width: f32,
) {
    for (index, primitive) in primitives.iter().enumerate() {
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
                if let [cx, cy, radius] = primitive.points.as_slice() {
                    let (cx, cy) = match moving {
                        Some((part, seat)) if part.primitive == index => {
                            let seat = match seat {
                                BallSeat::Left => 0,
                                BallSeat::Centre => 1,
                                BallSeat::Right => 2,
                            };
                            (part.seats[seat][0], part.seats[seat][1])
                        }
                        _ => (*cx, *cy),
                    };
                    let centre = at(rect, cx, cy);
                    #[allow(clippy::cast_possible_truncation)]
                    let radius = *radius as f32 * rect.width().min(rect.height());
                    surface.circle(
                        centre,
                        radius,
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

/// Numeral of a port, clear of the symbol beside its connection.
/// The geometry shared by the three numeral placements.
struct NumeralGeometry {
    rect: Rect,
    boundary: Rect,
    centre: Pt,
    point: Pt,
    reach: f32,
    clear: f32,
    normal: f32,
    gap: f32,
    epsilon: f32,
}

///
/// The numeral is placed against the symbol's **outer boundary**: the declared
/// [`ComponentDefinition::envelope`] when there is one, else the body, else the whole
/// box. It sits on the clear side of that boundary — outside when the sheet leaves room,
/// inside when it does not — and never on the line: the connection may cross the
/// boundary, the designation may not.
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
    let boundary = outer_boundary(rect, definition, body);
    let centre = boundary.center();
    let reach = rect.height() * 0.14;
    let clear = reach * 1.6;
    let gap = (rect.height() * 0.06).max(1.0);
    // The boundary-to-centre distance: the glyph keeps `gap` from the line and half its
    // cap is on each side of the anchor.
    let normal = gap + NUMERAL * 0.5;
    let epsilon = 1.0e-3;

    let level_y = point.y >= boundary.min.y - epsilon && point.y <= boundary.max.y + epsilon;
    let level_x = point.x >= boundary.min.x - epsilon && point.x <= boundary.max.x + epsilon;
    let on_side =
        level_y && (point.x <= boundary.min.x + epsilon || point.x >= boundary.max.x - epsilon);
    let on_top_or_bottom =
        level_x && (point.y <= boundary.min.y + epsilon || point.y >= boundary.max.y - epsilon);

    let geometry = NumeralGeometry {
        rect,
        boundary,
        centre,
        point,
        reach,
        clear,
        normal,
        gap,
        epsilon,
    };
    let placed = if on_side {
        side_numeral(&geometry)
    } else if on_top_or_bottom {
        top_bottom_numeral(&geometry, definition, port)
    } else {
        inside_numeral(&geometry)
    };
    haloed_text(surface, placed, id, NUMERAL, ink.line, ink.background);
}

/// A side port joins the symbol through a horizontal stub; place the numeral on the
/// clear side of the boundary, perpendicular to it.
///
/// A port drawn **outside** the functional boundary already has a margin: the numeral
/// goes in that margin, beside its port and above or below the stub, instead of being
/// pushed a whole clearance away. That is the service unit's case: the ports sit outside
/// the frame and the designation belongs next to its own port (`RF053`).
fn side_numeral(geometry: &NumeralGeometry) -> Pt {
    let outside = if geometry.point.x < geometry.centre.x {
        -1.0
    } else {
        1.0
    };
    let edge = if outside < 0.0 {
        geometry.boundary.min.x
    } else {
        geometry.boundary.max.x
    };
    let port_outside = if outside < 0.0 {
        geometry.point.x < geometry.boundary.min.x - geometry.epsilon
    } else {
        geometry.point.x > geometry.boundary.max.x + geometry.epsilon
    };
    if port_outside {
        let margin = if outside < 0.0 {
            edge - geometry.rect.min.x
        } else {
            geometry.rect.max.x - edge
        };
        // The glyph whole, with its gap, has to fit in the margin. The service unit's
        // 4.6 mm margin does; a narrower one keeps the old rule and places it inside.
        if margin + geometry.epsilon >= NUMERAL + geometry.gap {
            let half = NUMERAL * 0.5;
            let x = if outside < 0.0 {
                geometry.rect.min.x + half
            } else {
                geometry.rect.max.x - half
            };
            // Port 1 (left) reads above its stub and port 2 (right) below it, the way the
            // reference sheet places the pair; a box too short flips it to the other side.
            let prefer_below = geometry.point.x > geometry.centre.x;
            let first = if prefer_below {
                geometry.point.y + geometry.normal
            } else {
                geometry.point.y - geometry.normal
            };
            let second = if prefer_below {
                geometry.point.y - geometry.normal
            } else {
                geometry.point.y + geometry.normal
            };
            let fits = |y: f32| {
                y - half >= geometry.rect.min.y - geometry.epsilon
                    && y + half <= geometry.rect.max.y + geometry.epsilon
            };
            let y = if fits(first) { first } else { second };
            return Pt::new(x, y);
        }
    }
    let outside_room = if outside < 0.0 {
        geometry.boundary.min.x - geometry.rect.min.x
    } else {
        geometry.rect.max.x - geometry.boundary.max.x
    };
    // The whole glyph has to fit outside: `gap` plus the cap. If it does not, the
    // inside is the only clear space (the service-unit frame leaves 1.2 mm).
    let direction = if outside_room + geometry.epsilon < geometry.gap + NUMERAL {
        -outside
    } else {
        outside
    };
    let (dx, dy) = geometry.centre.to(geometry.point);
    let length = dx.hypot(dy);
    let lateral = if length > f32::EPSILON {
        Pt::new(-dy / length * geometry.clear, dx / length * geometry.clear)
    } else {
        Pt::new(geometry.clear, 0.0)
    };
    let mut at = Pt::new(
        edge + direction * geometry.normal + lateral.x,
        geometry.point.y + lateral.y,
    );
    if at.y - NUMERAL * 0.5 < geometry.rect.min.y || at.y + NUMERAL * 0.5 > geometry.rect.max.y {
        at.y = geometry.point.y - lateral.y;
    }
    at
}

/// A top or bottom port is read beside its stub, on the side of the boundary that has
/// room; the lateral shift stays off the neighbouring stubs.
fn top_bottom_numeral(
    geometry: &NumeralGeometry,
    definition: &ComponentDefinition,
    port: [f64; 2],
) -> Pt {
    let below = geometry.point.y >= geometry.centre.y;
    let outward = if below { 1.0 } else { -1.0 };
    let outside_room = if below {
        geometry.rect.max.y - geometry.boundary.max.y
    } else {
        geometry.boundary.min.y - geometry.rect.min.y
    };
    let direction = if outside_room + geometry.epsilon < geometry.gap + NUMERAL {
        -outward
    } else {
        outward
    };
    let edge = if below {
        geometry.boundary.max.y
    } else {
        geometry.boundary.min.y
    };
    let anchor = stub_end(geometry.rect, geometry.boundary, port);
    let mut nearest: Option<(f32, f32)> = None;
    for other in &definition.port {
        let other_point = at(geometry.rect, other.position[0], other.position[1]);
        if (other_point.y >= geometry.centre.y) != below {
            continue;
        }
        let distance = (other_point.x - geometry.point.x).abs();
        if distance <= f32::EPSILON {
            continue;
        }
        if nearest.is_none_or(|(closest, _)| distance < closest) {
            nearest = Some((
                distance,
                if other_point.x > geometry.point.x {
                    -1.0
                } else {
                    1.0
                },
            ));
        }
    }
    let side = if let Some((_, away)) = nearest {
        away
    } else {
        let outward = if geometry.point.x < geometry.boundary.center().x {
            -1.0
        } else {
            1.0
        };
        let room = if outward < 0.0 {
            geometry.point.x - geometry.boundary.min.x
        } else {
            geometry.boundary.max.x - geometry.point.x
        };
        if room > geometry.reach * 1.1 {
            outward
        } else {
            -outward
        }
    };
    Pt::new(
        anchor.x + side * geometry.clear,
        edge + direction * geometry.normal,
    )
}

/// The port is already inside the boundary: keep the numeral lateral to the stub and off
/// the line, flipping it when the box would clip it.
fn inside_numeral(geometry: &NumeralGeometry) -> Pt {
    let (dx, dy) = geometry.centre.to(geometry.point);
    let length = dx.hypot(dy);
    let lateral = if length <= f32::EPSILON {
        Pt::new(geometry.clear, 0.0)
    } else {
        let mut lateral = Pt::new(-dy / length * geometry.clear, dx / length * geometry.clear);
        let beside = Pt::new(geometry.point.x + lateral.x, geometry.point.y + lateral.y);
        if beside.x < geometry.rect.min.x || beside.x > geometry.rect.max.x {
            lateral = Pt::new(-lateral.x, -lateral.y);
        }
        if beside.y < geometry.rect.min.y || beside.y > geometry.rect.max.y {
            lateral.y = -lateral.y;
        }
        lateral
    };
    Pt::new(geometry.point.x + lateral.x, geometry.point.y + lateral.y)
}

/// The outer boundary a port numeral is placed against.
///
/// The assembly frame the file draws with the `boundary` class is the boundary when it
/// exists: a port may sit outside it, in the box's margin, and the designation belongs
/// outside the frame. Otherwise the declared envelope is the functional edge, then the
/// body is the renderer's box for a valve, and the whole box is the fallback for a flat
/// symbol with neither.
fn outer_boundary(rect: Rect, definition: &ComponentDefinition, body: Option<Rect>) -> Rect {
    definition
        .frame
        .map(|[x1, y1, x2, y2]| Rect::from_corners(at(rect, x1, y1), at(rect, x2, y2)))
        .or_else(|| frame_boundary(rect, definition))
        .or_else(|| {
            definition
                .envelope
                .map(|[x1, y1, x2, y2]| Rect::from_corners(at(rect, x1, y1), at(rect, x2, y2)))
        })
        .or(body)
        .unwrap_or(rect)
}

/// Bounding box of the `boundary`-class primitives, in millimetres of `rect`.
///
/// A chain line is the frame that groups the functions of one unit; it is not a second
/// body and the renderer does not invent one. The function is total: a primitive that
/// cannot be bounded (an arc used as a frame, which no built-in does) is skipped.
fn frame_boundary(rect: Rect, definition: &ComponentDefinition) -> Option<Rect> {
    let mut bounds: Option<Rect> = None;
    for primitive in &definition.symbol {
        if primitive.line_class != Some(CatalogLineClass::Boundary) {
            continue;
        }
        let points: Vec<Pt> = match primitive.kind {
            PrimitiveKind::Rect => {
                let Some(shape) = rect_from(rect, &primitive.points) else {
                    continue;
                };
                vec![shape.min, shape.max]
            }
            PrimitiveKind::Line | PrimitiveKind::Polyline | PrimitiveKind::Polygon => primitive
                .points
                .chunks_exact(2)
                .map(|pair| at(rect, pair[0], pair[1]))
                .collect(),
            PrimitiveKind::Circle | PrimitiveKind::Arc => continue,
        };
        let Some(first) = points.first().copied() else {
            continue;
        };
        let mut frame = Rect::from_corners(first, first);
        for point in points.into_iter().skip(1) {
            frame = Rect::from_corners(
                Pt::new(frame.min.x.min(point.x), frame.min.y.min(point.y)),
                Pt::new(frame.max.x.max(point.x), frame.max.y.max(point.y)),
            );
        }
        bounds = Some(match bounds {
            Some(existing) => Rect::from_corners(
                Pt::new(
                    existing.min.x.min(frame.min.x),
                    existing.min.y.min(frame.min.y),
                ),
                Pt::new(
                    existing.max.x.max(frame.max.x),
                    existing.max.y.max(frame.max.y),
                ),
            ),
            None => frame,
        });
    }
    bounds
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

/// A hand lever: the pivot circle where the arm meets the body, the arm, and the bar of
/// the general manual actuation as its grip at the far end.
///
/// ISO 1219-1 reads a lever by its pivot: the small circle sits at the **body end**, at
/// the point the arm turns about, and the bar a hand pushes is at the far end of the arm
/// (8.5.16, `688V1`; the roller lever of 6.1.2.5, `X10270`, puts its pivot against the
/// body and its roller at the far end for the same reason). A circle at the far end reads
/// as a roller and a bar against the body reads as the general manual actuation, which is
/// why the two ends are not interchangeable.
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
    let foot = step * 0.5;
    let pivot = p(foot, 0.0);
    surface.circle(pivot, step * 0.34, None, Some(stroke));
    // The arm leaves the pivot at 20° from the actuation axis, which is the angle the
    // standard's lever module carries.
    let angle = 20.0_f32.to_radians();
    let arm = reach * 1.05;
    let along = (angle.cos(), angle.sin());
    let across = (-along.1, along.0);
    let at = |fraction: f32, lateral: f32| {
        p(
            foot + arm * along.0 * fraction + across.0 * lateral,
            arm * along.1 * fraction + across.1 * lateral,
        )
    };
    let tip = at(1.0, 0.0);
    surface.line(&[pivot, tip], stroke);
    // The grip is the bar of the general manual actuation, across the far end.
    let half = step * 0.9;
    surface.line(
        &[at(1.0, -half), at(1.0, half)],
        Stroke::new(width * 1.8, colour),
    );
    let _ = edge;
    if detent {
        // The catch keeps the spool where it is left: a V notch across the arm says the
        // lever stays put instead of springing back.
        let notch = step * 0.5;
        let contact = |sign: f32| at(0.55, notch * sign);
        let apex = at(0.55 - notch / arm, 0.0);
        surface.line(
            &[contact(-1.0), apex, contact(1.0)],
            Stroke::new(width * 1.2, colour),
        );
    }
}

/// A push button: a stem ending in the bracket a finger pushes.
///
/// The standard's element for a control mechanism operated by pushing (`682V1`, and the
/// 2/2 valve `X10210`) is a bar across the end of the stem with two short prongs back
/// toward the body, not a rounded cap: the bracket is what a reader sees, and a rounded
/// bar is a bar with no bracket, which reads as the general manual actuation.
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
    let plate = reach * 0.85;
    surface.line(&[p(0.0, 0.0), p(plate, 0.0)], stroke);
    let half = step * 1.05;
    let prong = step * 0.75;
    surface.line(
        &[p(plate, -half), p(plate, half)],
        Stroke::new(width * 1.8, colour),
    );
    surface.line(&[p(plate, -half), p(plate - prong, -half)], stroke);
    surface.line(&[p(plate, half), p(plate - prong, half)], stroke);
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

/// A pedal: the lever's pivot and arm, with the plate a foot stands on at the far end.
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
    let foot = step * 0.5;
    let pivot = p(foot, 0.0);
    surface.circle(pivot, step * 0.34, None, Some(stroke));
    let tip = foot + reach * 1.05;
    surface.line(&[pivot, p(tip, 0.0)], stroke);
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

/// Pilot actuation: a box on the end face with a triangle inside it, pointing **toward the
/// body**, the direction the pressure pushes the valving element. The triangle is open for
/// a pneumatic pilot and filled for a hydraulic one.
///
/// The box is not decoration: without it a pilot cannot be told from a plain arrow marking a
/// flow, and the difference between the two media would be invisible in a photocopy. The
/// direction is the standard's: `F044V1` and `F043V1` draw the triangle with its apex at the
/// valve, and the complete valves `X10410` and `X10460` repeat it.
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
    // The apex sits at the body end: the pressure acts on the valving element, so the
    // triangle points that way. A triangle pointing away reads as a flow arrow.
    let triangle = [
        p(near + inset, 0.0),
        p(far - inset, -half + inset),
        p(far - inset, half - inset),
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
    draw_valve_with(surface, rect, definition, active, ink, true, true);
}

/// The valve, with or without the numerals its ports declare, and with or without the
/// connection points the sheet draws at each port.
fn draw_valve_with(
    surface: &mut dyn Surface,
    rect: Rect,
    definition: &ComponentDefinition,
    active: usize,
    ink: &Ink,
    numerals: bool,
    connections: bool,
) {
    let Some([bx, by, bw, bh]) = definition.body else {
        draw_flat_with(surface, rect, definition, None, ink, numerals, connections);
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
        if connections {
            draw_connection(surface, from, rect, ink);
        }
        if numerals && port.show_numeral {
            draw_numeral(
                surface,
                rect,
                definition,
                Some(body),
                port.numeral(),
                port.position,
                ink,
            );
        }
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
    draw_flat_with(surface, rect, definition, None, ink, true, true);
}

/// The flat symbol, with or without the numerals its ports declare, and with or without the
/// connection points the sheet draws at each port.
fn draw_flat_with(
    surface: &mut dyn Surface,
    rect: Rect,
    definition: &ComponentDefinition,
    moving: Option<BallSeat>,
    ink: &Ink,
    numerals: bool,
    connections: bool,
) {
    draw_primitives_seated(
        surface,
        rect,
        &definition.symbol,
        definition.moving.first().zip(moving),
        ink.line,
        LINE,
    );
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
        if connections {
            draw_connection(
                surface,
                at(rect, port.position[0], port.position[1]),
                rect,
                ink,
            );
        }
        if numerals && port.show_numeral {
            draw_numeral(
                surface,
                rect,
                definition,
                body,
                port.numeral(),
                port.position,
                ink,
            );
        }
    }
}

/// Share of the barrel's internal length the piston crosses at full extension.
///
/// The file declares the useful stroke and the residual length standing for the dead
/// volume; together they are the internal length a reader sees between the two cap faces.
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
    let inner_left = barrel.min.x + barrel.width() * CYLINDER_CAP_SHARE;
    let inner_right = barrel.max.x - barrel.width() * CYLINDER_CAP_SHARE;
    // The chamber is the inner length between the two end caps, and the piston crosses the
    // stroke's share of it: at full extension it stops short of the rod-side cap face by
    // exactly the dead length, which is where the compressed spring sits. The travel is the
    // internal length the file gives the piston, not a fraction chosen by the renderer. The
    // cap is also the room the port needs: a port drilled inside this chamber would be
    // swept by the piston, and the figure puts it between the end wall and this face.
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

/// Return spring between the piston and the rod-side cap face, compressed by the stroke.
///
/// The spring is drawn **across the chamber between the piston and the rod-side cap**, and
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
    moving: Option<BallSeat>,
    ink: &Ink,
) {
    draw_component_with(
        surface, rect, definition, active, extension, moving, ink, true, true,
    );
}

/// The same symbol as the panel shows it: without the connection points the sheet draws at
/// each port. The panel lists symbols; the sheet draws a diagram and marks where a hose
/// arrives.
pub fn draw_component_preview(
    surface: &mut dyn Surface,
    rect: Rect,
    definition: &ComponentDefinition,
    active: usize,
    extension: f32,
    moving: Option<BallSeat>,
    ink: &Ink,
) {
    draw_component_with(
        surface, rect, definition, active, extension, moving, ink, true, false,
    );
}

/// The same symbol without the numerals of its own ports.
///
/// A closure fitted to a host port is an accessory of that port, not a component with
/// labels of its own: its `1` would sit beside the host's numeral and read as a second
/// component. The host keeps its numerals; only the closure's are silent.
pub fn draw_component_without_numerals(
    surface: &mut dyn Surface,
    rect: Rect,
    definition: &ComponentDefinition,
    active: usize,
    extension: f32,
    moving: Option<BallSeat>,
    ink: &Ink,
) {
    draw_component_with(
        surface, rect, definition, active, extension, moving, ink, false, true,
    );
}

/// The one dispatcher, with the numeral and connection-point decisions explicit.
#[allow(clippy::too_many_arguments)]
fn draw_component_with(
    surface: &mut dyn Surface,
    rect: Rect,
    definition: &ComponentDefinition,
    active: usize,
    extension: f32,
    moving: Option<BallSeat>,
    ink: &Ink,
    numerals: bool,
    connections: bool,
) {
    if definition.body.is_some() {
        draw_valve_with(
            surface,
            rect,
            definition,
            active,
            ink,
            numerals,
            connections,
        );
        return;
    }
    draw_flat_with(
        surface,
        rect,
        definition,
        moving,
        ink,
        numerals,
        connections,
    );
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
    fn the_panel_preview_skips_the_connection_points() {
        let definition = valve();
        let rect = Rect::from_min_size(Pt::new(0.0, 0.0), 20.0, 20.0);
        let mut sheet = Recording::default();
        draw_component(&mut sheet, rect, &definition, 0, 0.0, None, &Ink::print());
        let mut panel = Recording::default();
        draw_component_preview(&mut panel, rect, &definition, 0, 0.0, None, &Ink::print());
        let circles = |recording: &Recording| {
            recording
                .calls()
                .iter()
                .filter(|call| matches!(call, crate::surface::Call::Circle { .. }))
                .count()
        };
        assert_eq!(
            circles(&sheet) - circles(&panel),
            definition.port.len(),
            "one connection point per port on the sheet, none in the panel"
        );
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
    fn every_cylinder_port_is_drilled_in_a_cap_and_clear_of_the_chamber() {
        let catalog = punchi_fluid_catalog::Catalog::builtin();
        // The sheet's box: a cylinder is 44 × 20 mm, so the millimetres below are the ones
        // the review measures.
        let rect = Rect::from_min_size(Pt::new(0.0, 0.0), 44.0, 20.0);
        for id in [
            "cylinder.single_acting_20_100",
            "cylinder.double_acting_32_200",
        ] {
            let definition = catalog
                .get(id)
                .expect("the built-in cylinder exists")
                .clone();
            let barrel = barrel_of(&definition, rect).expect("a barrel");
            let cap = barrel.width() * CYLINDER_CAP_SHARE;
            let chamber_left = barrel.min.x + cap;
            let chamber_right = barrel.max.x - cap;
            let mut depths = Vec::new();
            for port in &definition.port {
                let point = at(rect, port.position[0], port.position[1]);
                // A port inside the chamber would be swept by the piston: a single-acting
                // cylinder would pressurise the spring chamber at rest, and a double-acting
                // one would lose the connection when its piston passed over it.
                assert!(
                    point.x < chamber_left || point.x > chamber_right,
                    "{id}: port {} at {:.2} mm is inside the chamber {:.2}..{:.2} mm",
                    port.id,
                    point.x,
                    chamber_left,
                    chamber_right
                );
                // It is drilled in its cap: between the end wall and the cap's inner face.
                let depth = if point.x < chamber_left {
                    point.x - barrel.min.x
                } else {
                    barrel.max.x - point.x
                };
                assert!(
                    depth > LINE && depth < cap,
                    "{id}: port {} sits {:.2} mm from its end wall, the cap is {:.2} mm",
                    port.id,
                    depth,
                    cap
                );
                depths.push(depth);
            }
            // Both ports of a double-acting cylinder sit at the same depth, one per end.
            if definition.port.len() > 1 {
                let first = depths.first().copied().unwrap_or_default();
                assert!(
                    depths.iter().all(|depth| (depth - first).abs() < 1.0e-3),
                    "{id}: the ports are not at the same depth from their caps: {depths:?}"
                );
                assert!(
                    definition
                        .port
                        .iter()
                        .map(|port| at(rect, port.position[0], port.position[1]).x)
                        .fold((false, false), |(left, right), x| {
                            (left || (x < chamber_left), right || (x > chamber_right))
                        })
                        == (true, true),
                    "{id}: a port is missing from one of the two caps"
                );
            }
        }

        // The single-acting cylinder's only port is in the cap end (the left one), so the
        // rod side carries the spring and the rod and no working connection.
        let single = catalog
            .get("cylinder.single_acting_20_100")
            .expect("the built-in cylinder exists")
            .clone();
        let barrel = barrel_of(&single, rect).expect("a barrel");
        let cap = barrel.width() * CYLINDER_CAP_SHARE;
        let ports: Vec<Pt> = single
            .port
            .iter()
            .map(|port| at(rect, port.position[0], port.position[1]))
            .collect();
        assert_eq!(
            ports.len(),
            1,
            "a single-acting cylinder has one working port"
        );
        assert!(
            ports[0].x < barrel.min.x + cap,
            "the only port must be in the cap end, at {:.2} mm",
            ports[0].x
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
            let inset = barrel.width() * CYLINDER_CAP_SHARE;
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
        let wall = barrel.max.x - barrel.width() * CYLINDER_CAP_SHARE;
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

    /// A numeral a recording drew, as its identifier, anchor and cap height.
    ///
    /// The halo is the same text in a larger size; the glyph is the run at [`NUMERAL`],
    /// which is the one the reader sees and the only one the clearance rule is about.
    fn numerals(calls: &[crate::surface::Call]) -> Vec<(String, Pt, f32)> {
        calls
            .iter()
            .filter_map(|call| {
                let crate::surface::Call::Text { at, text, size, .. } = call else {
                    return None;
                };
                ((*size - NUMERAL).abs() < 1.0e-3).then(|| (text.clone(), *at, *size))
            })
            .collect()
    }

    /// The box a numeral occupies, from its anchor and cap height.
    ///
    /// `Surface::text` returns no extents and `Call::Text` stores only the anchor and the
    /// cap. A digit is narrower than it is tall, so one cap each way is conservative: a
    /// box this size that stays clear is a glyph that stays clear.
    fn numeral_box(at: Pt, cap: f32) -> Rect {
        Rect::from_center_size(at, cap, cap)
    }

    /// `true` when a segment touches a rectangle (Liang–Barsky clip, edges included).
    fn segment_hits_rect(a: Pt, b: Pt, rect: Rect) -> bool {
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let p = [-dx, dx, -dy, dy];
        let q = [
            a.x - rect.min.x,
            rect.max.x - a.x,
            a.y - rect.min.y,
            rect.max.y - a.y,
        ];
        let mut enter = 0.0_f32;
        let mut leave = 1.0_f32;
        for index in 0..4 {
            if p[index].abs() < 1.0e-9 {
                if q[index] < 0.0 {
                    return false;
                }
                continue;
            }
            let ratio = q[index] / p[index];
            if p[index] < 0.0 {
                if ratio > leave {
                    return false;
                }
                enter = enter.max(ratio);
            } else {
                if ratio < enter {
                    return false;
                }
                leave = leave.min(ratio);
            }
        }
        true
    }

    /// `true` when a circle touches a rectangle.
    fn circle_hits_rect(centre: Pt, radius: f32, rect: Rect) -> bool {
        let nearest = Pt::new(
            centre.x.clamp(rect.min.x, rect.max.x),
            centre.y.clamp(rect.min.y, rect.max.y),
        );
        centre.to(nearest).0.hypot(centre.to(nearest).1) <= radius
    }

    type DrawnStrokes = (Vec<(Pt, Pt)>, Vec<(Pt, f32)>);

    /// Segments and circles a base symbol draws, in millimetres of its box.
    fn drawn_strokes(rect: Rect, definition: &ComponentDefinition) -> DrawnStrokes {
        let mut strokes = Vec::new();
        let mut circles = Vec::new();
        for primitive in &definition.symbol {
            let points: Vec<Pt> = primitive
                .points
                .chunks_exact(2)
                .map(|pair| at(rect, pair[0], pair[1]))
                .collect();
            match primitive.kind {
                PrimitiveKind::Line | PrimitiveKind::Polyline => {
                    strokes.extend(points.windows(2).map(|pair| (pair[0], pair[1])));
                }
                PrimitiveKind::Polygon => {
                    strokes.extend(points.windows(2).map(|pair| (pair[0], pair[1])));
                    if let (Some(first), Some(last)) = (points.first(), points.last()) {
                        strokes.push((*last, *first));
                    }
                }
                PrimitiveKind::Rect => {
                    if let Some(rect) = rect_from(rect, &primitive.points) {
                        let corners = [
                            Pt::new(rect.min.x, rect.min.y),
                            Pt::new(rect.max.x, rect.min.y),
                            Pt::new(rect.max.x, rect.max.y),
                            Pt::new(rect.min.x, rect.max.y),
                        ];
                        for index in 0..4 {
                            strokes.push((corners[index], corners[(index + 1) % 4]));
                        }
                    }
                }
                PrimitiveKind::Circle => {
                    if let [cx, cy, radius] = primitive.points.as_slice() {
                        #[allow(clippy::cast_possible_truncation)]
                        let centre = at(rect, *cx, *cy);
                        #[allow(clippy::cast_possible_truncation)]
                        let radius = *radius as f32 * rect.width().min(rect.height());
                        circles.push((centre, radius));
                    }
                }
                PrimitiveKind::Arc => {
                    // No built-in envelope carrier uses an arc. Keep the helper total by
                    // walking the arc as short chords instead of ignoring it.
                    if primitive.points.len() >= 5 {
                        #[allow(clippy::cast_possible_truncation)]
                        let centre = at(rect, primitive.points[0], primitive.points[1]);
                        #[allow(clippy::cast_possible_truncation)]
                        let radius = primitive.points[2] as f32 * rect.width().min(rect.height());
                        #[allow(clippy::cast_possible_truncation)]
                        let start = primitive.points[3] as f32;
                        #[allow(clippy::cast_possible_truncation)]
                        let end = primitive.points[4] as f32;
                        let steps = 24;
                        let mut previous = None;
                        for step in 0..=steps {
                            #[allow(clippy::cast_precision_loss)]
                            let angle =
                                (start + (end - start) * step as f32 / steps as f32).to_radians();
                            let point = Pt::new(
                                centre.x + radius * angle.cos(),
                                centre.y + radius * angle.sin(),
                            );
                            if let Some(from) = previous {
                                strokes.push((from, point));
                            }
                            previous = Some(point);
                        }
                    }
                }
            }
        }
        (strokes, circles)
    }

    /// Edges of a rectangle as segments, so a boundary can be checked like a drawn line.
    fn rect_edges(rect: Rect) -> Vec<(Pt, Pt)> {
        let corners = [
            Pt::new(rect.min.x, rect.min.y),
            Pt::new(rect.max.x, rect.min.y),
            Pt::new(rect.max.x, rect.max.y),
            Pt::new(rect.min.x, rect.max.y),
        ];
        (0..4)
            .map(|index| (corners[index], corners[(index + 1) % 4]))
            .collect()
    }

    /// The two service units carry their designations outside their frame, in the box's
    /// margin, beside their own ports, in both themes. A numeral that goes back inside the
    /// frame is the defect the owner reported on the reference capture.
    fn assert_conditioning_numerals_are_outside_the_frame(
        id: &str,
        frame_width: f32,
        frame_height: f32,
    ) {
        let catalog = punchi_fluid_catalog::Catalog::builtin();
        let definition = catalog.get(id).expect("the built-in exists").clone();
        let [width, height] = definition
            .declared_box_mm()
            .expect("the unit declares its box");
        let rect = Rect::from_min_size(Pt::new(0.0, 0.0), width, height);
        let boundary = outer_boundary(rect, &definition, None);
        assert!(
            (boundary.width() - frame_width).abs() < 1.0e-4
                && (boundary.height() - frame_height).abs() < 1.0e-4,
            "{id}: the frame is not {frame_width} x {frame_height} mm: {boundary:?}"
        );
        let gap = (rect.height() * 0.06).max(1.0);
        let (strokes, circles) = drawn_strokes(rect, &definition);
        let dark = Ink {
            line: Colour::WHITE,
            dim: Colour::new(150, 150, 150),
            muted: Colour::new(180, 180, 180),
            text: Colour::WHITE,
            supply: Colour::WHITE,
            work: Colour::WHITE,
            accent: Colour::new(40, 46, 60),
            background: Colour::new(20, 20, 20),
        };
        for ink in [Ink::print(), dark] {
            let mut recording = Recording::default();
            draw_component(&mut recording, rect, &definition, 0, 0.0, None, &ink);
            let numerals = numerals(recording.calls());
            assert_eq!(
                numerals.len(),
                2,
                "{id}: the two ports carry one numeral each"
            );
            for (numeral, anchor, cap) in &numerals {
                let glyph = numeral_box(*anchor, *cap);
                // The designation is outside the frame, in the box's margin, beside its
                // own port: outside on the side the port leaves, never on the frame.
                let port = definition
                    .port(numeral)
                    .expect("the numeral names a declared port");
                let port_at = at(rect, port.position[0], port.position[1]);
                let outside = if port_at.x < boundary.center().x {
                    glyph.max.x < boundary.min.x - 1.0e-3
                } else {
                    glyph.min.x > boundary.max.x + 1.0e-3
                };
                assert!(
                    outside,
                    "{id}: numeral {numeral} is not outside the frame {boundary:?}: {glyph:?}"
                );
                // It stays in the margin, close to its port, not across the drawing.
                assert!(
                    (glyph.center().x - port_at.x).abs() <= NUMERAL
                        && (glyph.center().y - port_at.y).abs() <= NUMERAL * 1.6,
                    "{id}: numeral {numeral} is not beside its port {port_at:?}: {glyph:?}"
                );
                // The glyph keeps the gap from the frame the placement asked for.
                let near = glyph.expand(gap - 1.0e-3);
                for (from, to) in rect_edges(boundary) {
                    assert!(
                        !segment_hits_rect(from, to, near),
                        "{id}: numeral {numeral} is not clear of the frame edge {from:?}–{to:?}: {near:?}"
                    );
                }
                assert!(
                    rect.contains(glyph.min) && rect.contains(glyph.max),
                    "{id}: numeral {numeral} leaves the {width} x {height} mm box: {glyph:?}"
                );
                for (from, to) in &strokes {
                    assert!(
                        !segment_hits_rect(*from, *to, glyph),
                        "{id}: numeral {numeral} touches a primitive {from:?}–{to:?}: {glyph:?}"
                    );
                }
                for (centre, radius) in &circles {
                    assert!(
                        !circle_hits_rect(*centre, *radius, glyph),
                        "{id}: numeral {numeral} touches the dial at {centre:?} r {radius}: {glyph:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_service_unit_numerals_stay_clear_of_the_frame_and_the_drawing() {
        assert_conditioning_numerals_are_outside_the_frame("preparation.service_unit", 36.8, 20.0);
    }

    #[test]
    fn the_simplified_service_unit_numerals_sit_outside_its_body() {
        assert_conditioning_numerals_are_outside_the_frame(
            "preparation.service_unit_simple",
            31.2,
            15.2,
        );
    }

    #[test]
    fn the_filter_regulator_numerals_sit_outside_its_frame() {
        assert_conditioning_numerals_are_outside_the_frame(
            "preparation.filter_regulator",
            36.8,
            20.0,
        );
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
    fn the_push_button_is_a_bracket_open_toward_the_body() {
        // 8.5.10 (`682V1`) and the 2/2 valve `X10210` draw the pushing element as a bar
        // across the end of the stem with two prongs back toward the body: a bracket, not
        // a rounded cap. The bracket is the shape a reader recognises as "push".
        let body = Rect::from_min_size(Pt::new(50.0, 20.0), 40.0, 20.0);
        let mut recording = Recording::default();
        draw_actuator(
            &mut recording,
            body,
            ActuatorKind::PushButton,
            Side::Left,
            Colour::BLACK,
            LINE,
        );
        let calls = recording.calls();
        let lines: Vec<&Vec<Pt>> = calls
            .iter()
            .filter_map(|call| match call {
                crate::surface::Call::Line { points, .. } => Some(points),
                _ => None,
            })
            .collect();
        let min_x = touched(calls)
            .iter()
            .map(|point| point.x)
            .fold(f32::INFINITY, f32::min);
        let bar = lines
            .iter()
            .find(|points| {
                points.len() == 2
                    && (points[0].x - min_x).abs() < 1.0e-3
                    && (points[1].x - min_x).abs() < 1.0e-3
            })
            .expect("the bracket bar across the far end");
        assert!(
            (bar[0].y - bar[1].y).abs() > 1.0e-3,
            "the bar lies along the stem instead of across it: {bar:?}"
        );
        for end in [bar[0], bar[1]] {
            let meets =
                |point: &Pt| (point.x - end.x).abs() < 1.0e-3 && (point.y - end.y).abs() < 1.0e-3;
            let prong = lines.iter().any(|points| {
                points.len() == 2
                    && points.iter().any(meets)
                    && points.iter().any(|point| point.x > end.x + 1.0e-3)
            });
            assert!(
                prong,
                "the bar end {end:?} has no prong back toward the body: {bar:?}"
            );
        }
    }

    #[test]
    fn the_pilot_triangle_points_at_the_body() {
        // `F044V1`/`F043V1` and the complete valves `X10410`/`X10460` draw the triangle
        // with its apex at the valve: the pressure pushes the valving element. On a left
        // actuator the body is at the larger x, so the apex must be the rightmost vertex.
        let body = Rect::from_min_size(Pt::new(50.0, 20.0), 40.0, 20.0);
        for kind in [ActuatorKind::Pilot, ActuatorKind::PilotHydraulic] {
            let mut recording = Recording::default();
            draw_actuator(&mut recording, body, kind, Side::Left, Colour::BLACK, LINE);
            let triangle = recording
                .calls()
                .iter()
                .find_map(|call| match call {
                    crate::surface::Call::Polygon { points, .. } if points.len() == 3 => {
                        Some(points.clone())
                    }
                    _ => None,
                })
                .expect("the pilot triangle is a polygon of three points");
            let min_x = triangle
                .iter()
                .map(|point| point.x)
                .fold(f32::INFINITY, f32::min);
            let max_x = triangle
                .iter()
                .map(|point| point.x)
                .fold(f32::NEG_INFINITY, f32::max);
            assert!(
                max_x > min_x + 1.0e-3,
                "{kind:?}: the triangle has no side to point along: {triangle:?}"
            );
            let apexes = triangle
                .iter()
                .filter(|point| (point.x - max_x).abs() < 1.0e-3)
                .count();
            assert_eq!(
                apexes, 1,
                "{kind:?}: exactly one apex sits at the body end: {triangle:?}"
            );
        }
    }

    #[test]
    fn the_lever_pivots_against_the_body_and_grips_at_the_far_end() {
        // 8.5.16 (`688V1`) and the roller lever `X10270` put the pivot circle at the body
        // end and the part a hand pushes at the far end; the other way round reads as a
        // roller on the body and a bar on the far end, which is a different machine.
        let body = Rect::from_min_size(Pt::new(50.0, 20.0), 40.0, 20.0);
        let mut recording = Recording::default();
        draw_actuator(
            &mut recording,
            body,
            ActuatorKind::Lever,
            Side::Left,
            Colour::BLACK,
            LINE,
        );
        let calls = recording.calls();
        let pivot = calls
            .iter()
            .find_map(|call| match call {
                crate::surface::Call::Circle { centre, .. } => Some(*centre),
                _ => None,
            })
            .expect("the lever draws its pivot circle");
        let far = touched(calls)
            .iter()
            .map(|point| point.x)
            .fold(f32::INFINITY, f32::min);
        assert!(
            pivot.x < body.min.x,
            "the pivot is outside the body: {pivot:?}"
        );
        assert!(
            pivot.x > f32::midpoint(body.min.x, far),
            "the pivot must sit on the body half of the glyph: pivot {pivot:?}, far end {far}"
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

    #[test]
    fn the_moving_ball_takes_the_left_centre_and_right_seats() {
        let catalog = punchi_fluid_catalog::Catalog::builtin();
        let definition = catalog.get("valve.shuttle").expect("the shuttle").clone();
        let rect = Rect::from_min_size(Pt::new(0.0, 0.0), 20.0, 20.0);
        let mut centres = Vec::new();
        for seat in [BallSeat::Left, BallSeat::Centre, BallSeat::Right] {
            let mut recording = Recording::default();
            draw_primitives_seated(
                &mut recording,
                rect,
                &definition.symbol,
                definition.moving.first().zip(Some(seat)),
                Ink::print().line,
                LINE,
            );
            let centre = recording
                .calls()
                .iter()
                .find_map(|call| match call {
                    crate::surface::Call::Circle { centre, radius, .. }
                        if (radius - 1.0).abs() < 0.2 =>
                    {
                        Some(centre.x)
                    }
                    _ => None,
                })
                .expect("the ball");
            centres.push(centre);
        }
        assert!(
            centres[0] < centres[1] && centres[1] < centres[2],
            "the ball stayed put: {centres:?}"
        );
    }
}
