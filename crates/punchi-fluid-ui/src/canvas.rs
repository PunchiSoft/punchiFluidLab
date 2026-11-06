// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The sheet: it shows the drawing and reports what the user clicked.
//!
//! The drawing itself is in `punchi-fluid-draw`, the same one the PDF export uses. This
//! module owns the part a file cannot own: where the pointer is, what it hit, and the
//! cursor to show while it hovers the valve.

use egui::{Pos2, Rect, Sense, Ui, Vec2};
use punchi_fluid_catalog::Catalog;
use punchi_fluid_draw::{Pt, Rect as MmRect};
use punchi_fluid_solver::pneumatic::PneumaticSystem;

use crate::surface::{self, EguiSurface, to_screen};

/// Identifier of the supply drawn on the canvas.
pub const SUPPLY_ID: &str = punchi_fluid_draw::schematic::SUPPLY_ID;
/// Identifier of the valve drawn on the canvas.
pub const VALVE_ID: &str = punchi_fluid_draw::schematic::VALVE_ID;
/// Identifier of the cylinder drawn on the canvas.
pub const CYLINDER_ID: &str = punchi_fluid_draw::schematic::CYLINDER_ID;

/// What the user did on the canvas during a frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CanvasOutcome {
    /// The valve is being held down, so a momentary valve is actuated.
    pub valve_held: bool,
    /// The pointer was released over the valve, so a spring returns it.
    pub valve_released: bool,
}

/// Drawing surface for the circuit.
#[derive(Debug, Default)]
pub struct SchematicCanvas {
    scene_size: Vec2,
    /// The pointer is down on the valve, so the release still belongs to it.
    holding: bool,
}

/// Placement of the three components of the fixed circuit, in screen points.
#[derive(Debug, Clone, Copy)]
pub struct Layout {
    /// Where the supply symbol is drawn.
    pub supply: Rect,
    /// Where the valve symbol is drawn, and the area that reacts to clicks.
    pub valve: Rect,
    /// Where the cylinder symbol is drawn.
    pub cylinder: Rect,
}

/// Placement of the circuit inside `area`, in screen coordinates.
///
/// Returns `None` when there is no room to draw anything readable.
#[must_use]
pub fn layout(area: Rect) -> Option<Layout> {
    let fitted = surface::fit(area);
    let placed = punchi_fluid_draw::layout(surface::logical_area())?;
    Some(Layout {
        supply: to_screen(fitted, placed.supply),
        valve: to_screen(fitted, placed.valve),
        cylinder: to_screen(fitted, placed.cylinder),
    })
}

impl SchematicCanvas {
    /// Draws the circuit and reports the interaction.
    pub fn show(
        &mut self,
        ui: &mut Ui,
        system: &PneumaticSystem,
        catalog: &Catalog,
    ) -> CanvasOutcome {
        self.scene_size = ui.available_size();
        let (response, painter) = ui.allocate_painter(self.scene_size, Sense::click());
        let area = response.rect;
        let theme = punchi_fluid_theme::current(ui.ctx());
        let ink = surface::ink(&theme);

        let fitted = surface::fit(area);
        let mut surface = EguiSurface::new(&painter, fitted.origin).with_scale(fitted.scale);
        let placed =
            punchi_fluid_draw::draw(&mut surface, surface::logical_area(), system, catalog, &ink);

        let Some(placed) = placed else {
            return CanvasOutcome::default();
        };
        let valve = to_screen(fitted, placed.valve);
        let hit_area = valve.expand(theme.metrics.grid_unit * 0.6);
        let over = response
            .hover_pos()
            .is_some_and(|pointer| hit_area.contains(pointer));
        if over {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        // A spring-return valve is actuated only while the actuator is applied, so the
        // canvas reports the hold and the release, not a click.
        let pointer_down = ui.input(|input| input.pointer.any_down());
        let pressed = response.is_pointer_button_down_on()
            && response
                .interact_pointer_pos()
                .is_some_and(|pointer| hit_area.contains(pointer));
        if pressed {
            self.holding = true;
        }
        let released = self.holding && !pointer_down;
        if released {
            self.holding = false;
        }

        CanvasOutcome {
            valve_held: self.holding && pointer_down,
            valve_released: released,
        }
    }

    /// Size the canvas was given on the last frame.
    #[must_use]
    pub fn scene_size(&self) -> Vec2 {
        self.scene_size
    }
}

/// Point of the drawing under a screen position, for future editing.
#[must_use]
pub fn drawing_point(area: Rect, position: Pos2) -> Pt {
    surface::to_drawing(surface::fit(area), position)
}

/// Logical drawing rectangle the canvas lays the circuit out in.
#[must_use]
pub fn drawing_area() -> MmRect {
    surface::logical_area()
}
