// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The run until a state change: the one command that needs the clock and the model
//! together and nothing from the layout.

use std::time::Duration;

use eframe::egui;

use crate::Window;

impl Window {
    /// Rebuilds stale model state and advances the running clock at the sheet's frame pace.
    pub(crate) fn tick(&mut self, context: &egui::Context) {
        self.rebuild_if_stale();
        if self.running {
            let dt = f64::from(context.input(|input| input.stable_dt).min(0.1));
            self.advance(dt * f64::from(self.speed.factor()));
            if self.running {
                context.request_repaint_after(Duration::from_millis(16));
            }
        }
    }

    /// Enters the simulation with the model the sheet builds, or leaves the mode alone when
    /// the circuit has no model. Returns whether there is a model to advance.
    pub(crate) fn enter_simulation(&mut self) -> bool {
        // A click actuates while the model runs, so entering the simulation leaves the hand.
        self.hand_mode = false;
        self.rebuild_if_stale();
        if self.model.is_some() {
            self.simulation = true;
            true
        } else {
            false
        }
    }

    /// Leaves simulation input inactive and releases a valve held by the pointer.
    pub(crate) fn pause(&mut self) {
        self.running = false;
        self.release_pressed_valve();
    }

    /// Releases the current momentary valve, if the pointer was holding one.
    pub(crate) fn release_pressed_valve(&mut self) {
        let Some(index) = self.pressed_valve.take() else {
            return;
        };
        if let Some(model) = self.model.as_mut() {
            let _ = model.release_valve(index);
        }
    }

    /// Position of the model's cylinder, in metres, or `None` when no model is loaded. The
    /// acceptance test reads the piston through this accessor instead of the solver's own
    /// types, so the window stays the only thing it has to know.
    #[allow(dead_code, reason = "the acceptance test reads the piston through it")]
    pub(crate) fn cylinder_position(&self) -> Option<f64> {
        let network = self.model.as_ref()?.network();
        network.cylinder().map(|_| network.position())
    }

    /// Runs until the piston reaches an end or the valve position changes, with a cap:
    /// a circuit that never changes stops after sixty simulated seconds.
    pub(crate) fn until_change(&mut self) {
        if !self.enter_simulation() {
            return;
        }
        self.pause();
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
                self.notice = Some(
                    self.language
                        .format("notice.sim_stopped", &[("cause", &cause)]),
                );
                return;
            }
            let now = model.network().telemetry();
            if now.valve_position != start.valve_position {
                self.notice = Some(self.language.text("notice.valve_changed").to_owned());
                return;
            }
            let reached = (far(now.position) && !far(start.position))
                || (near(now.position) && !near(start.position));
            if reached {
                self.notice = Some(self.language.text("notice.piston_end").to_owned());
                return;
            }
        }
        self.notice = Some(self.language.text("notice.no_change").to_owned());
    }
}
