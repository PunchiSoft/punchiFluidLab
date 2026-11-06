// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The valve is a momentary actuator: it is applied while pressed and released when
//! the button comes up. Nothing else on the sheet reacts.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

use egui::{Event, Modifiers, PointerButton, Pos2, Rect, Vec2};
use punchi_fluid_catalog::Catalog;
use punchi_fluid_solver::pneumatic::PneumaticSystem;
use punchi_fluid_ui::SchematicCanvas;
use punchi_fluid_ui::canvas::layout;

const SCREEN: Vec2 = Vec2::new(900.0, 640.0);

/// One frame's report, as `(applied, released)`.
type Frame = (bool, bool);

/// Three frames: idle, pointer down, pointer up.
fn press_at(canvas: &mut SchematicCanvas, pointer: Pos2) -> [Frame; 3] {
    let context = egui::Context::default();
    let rect = Rect::from_min_size(Pos2::ZERO, SCREEN);
    let catalog = Catalog::builtin();
    let system = PneumaticSystem::from_catalog(&catalog).expect("the built-in circuit starts");

    let down = Event::PointerButton {
        pos: pointer,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::default(),
    };
    let up = Event::PointerButton {
        pos: pointer,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::default(),
    };

    let mut frames = [(false, false); 3];
    for (frame, events) in [vec![], vec![down], vec![up]].into_iter().enumerate() {
        let input = egui::RawInput {
            screen_rect: Some(rect),
            events,
            ..egui::RawInput::default()
        };
        let mut output = context.run_ui(input, |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                let result = canvas.show(ui, &system, &catalog);
                frames[frame] = (result.valve_held, result.valve_released);
            });
        });
        // Headless: nothing renders the texture atlas, so drop the deltas.
        output.textures_delta.clear();
    }
    frames
}

#[test]
fn holding_the_valve_applies_it() {
    let area = Rect::from_min_size(Pos2::ZERO, SCREEN);
    let placed = layout(area).expect("the test screen is big enough");
    let mut canvas = SchematicCanvas::default();
    assert_eq!(
        press_at(&mut canvas, placed.valve.center()),
        [(false, false), (true, false), (false, true),],
        "the valve applies while the button is held and releases when it comes up"
    );
}

#[test]
fn holding_elsewhere_is_ignored() {
    let area = Rect::from_min_size(Pos2::ZERO, SCREEN);
    let placed = layout(area).expect("the test screen is big enough");
    for spot in [placed.cylinder.center(), placed.supply.center()] {
        let mut canvas = SchematicCanvas::default();
        assert_eq!(
            press_at(&mut canvas, spot),
            [(false, false); 3],
            "only the valve may actuate"
        );
    }
}
