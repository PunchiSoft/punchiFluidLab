// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The run until a state change: the one command that needs the clock and the model
//! together and nothing from the layout.

use crate::Window;

impl Window {
    /// Runs until the piston reaches an end or the valve position changes, with a cap:
    /// a circuit that never changes stops after sixty simulated seconds.
    pub(crate) fn until_change(&mut self) {
        self.running = false;
        let Some(model) = self.model.as_mut() else {
            return;
        };
        let stroke = model
            .network()
            .cylinder()
            .map_or(0.0, |cylinder| cylinder.stroke);
        let start = model.network().telemetry();
        // The far end is the one the piston moves towards; the near end is the rest one, so
        // a start at rest does not stop the run on its first step.
        let far = |position: f64| stroke > 0.0 && position >= stroke - 1.0e-4;
        let near = |position: f64| position <= 1.0e-4;
        for _ in 0..6000 {
            if let Err(cause) = model.advance(0.01) {
                self.notice = Some(format!("la simulación se detuvo: {cause}"));
                return;
            }
            let now = model.network().telemetry();
            if now.valve_position != start.valve_position {
                self.notice = Some(String::from("la válvula ha cambiado"));
                return;
            }
            let reached = (far(now.position) && !far(start.position))
                || (near(now.position) && !near(start.position));
            if reached {
                self.notice = Some(String::from("el pistón ha llegado a un tope"));
                return;
            }
        }
        self.notice = Some(String::from("sin cambio en 60 s"));
    }
}
