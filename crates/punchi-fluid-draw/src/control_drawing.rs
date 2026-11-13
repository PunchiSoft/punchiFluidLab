// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Drawing of the control parts whose state comes from the run.
//!
//! The symbol file holds one drawing; these helpers add the moving bar of a contact or a
//! relay and the rays of a lit lamp. Keeping them here leaves `placed.rs` inside its line
//! budget and puts the stateful drawing in one place, beside the tests that exercise it.

use punchi_fluid_catalog::ComponentDefinition;

use crate::placed::Placed;
use crate::surface::{Pt, Stroke, Surface};
use crate::symbol::{self, Ink};

/// Draws the moving bar of a contact or a relay from the state the run reports.
///
/// The bar is the same idea for both parts: the file holds one symbol and the sheet shows
/// two, because the state belongs to the run and not to the document. A relay's bar is drawn
/// between the gap its file leaves at x = 0.72, and tilts away from the moving terminal when
/// it rests (ADR 0019).
pub(crate) fn moving_bar(
    surface: &mut dyn Surface,
    placed: &Placed<'_>,
    definition: &ComponentDefinition,
    ink: &Ink,
) {
    match &definition.behaviour {
        punchi_fluid_catalog::Behaviour::Contact { normally_closed }
        | punchi_fluid_catalog::Behaviour::LimitSwitch {
            normally_closed, ..
        }
        | punchi_fluid_catalog::Behaviour::PressureSwitch {
            normally_closed, ..
        } => {
            let closed = placed.contact_closed.unwrap_or(*normally_closed);
            let inner = placed.box_rect();
            let y = if closed { 0.5 } else { 0.32 };
            surface.line(
                &[symbol::at(inner, 0.30, y), symbol::at(inner, 0.70, y)],
                Stroke::new(symbol::LINE, ink.line),
            );
        }
        punchi_fluid_catalog::Behaviour::Relay {
            normally_closed, ..
        } => {
            let closed = placed.contact_closed.unwrap_or(*normally_closed);
            let inner = placed.box_rect();
            let fixed = symbol::at(inner, 0.72, 0.30);
            let moving = symbol::at(inner, 0.72, 0.70);
            let end = if closed {
                moving
            } else {
                Pt::new(
                    moving.x + inner.width() * 0.10,
                    moving.y - inner.height() * 0.12,
                )
            };
            surface.line(&[fixed, end], Stroke::new(symbol::LINE, ink.line));
        }
        _ => {}
    }
}

/// Draws the rays of a lit indicator around the circle its file declares.
///
/// The state is never colour alone: a lit lamp gains rays and the word `encendida` beside
/// it, and an unlit one keeps both away. The centre and radius come from the file, so a
/// larger lamp is another file and not another branch.
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn indicator_rays(
    surface: &mut dyn Surface,
    placed: &Placed<'_>,
    definition: &ComponentDefinition,
    ink: &Ink,
) {
    let Some(circle) = definition.symbol.iter().find(|primitive| {
        primitive.kind == punchi_fluid_catalog::PrimitiveKind::Circle && primitive.points.len() >= 3
    }) else {
        return;
    };
    let inner = placed.box_rect();
    let unit = inner.width().min(inner.height());
    let centre = symbol::at(inner, circle.points[0], circle.points[1]);
    let radius = circle.points[2] as f32 * unit;
    let from = radius * 1.2;
    let to = radius * 1.45;
    for step in 0..8_u16 {
        let angle = f32::from(step) * std::f32::consts::FRAC_PI_4;
        let (sin, cos) = angle.sin_cos();
        surface.line(
            &[
                Pt::new(centre.x + from * cos, centre.y + from * sin),
                Pt::new(centre.x + to * cos, centre.y + to * sin),
            ],
            Stroke::new(symbol::LINE, ink.line),
        );
    }
}
