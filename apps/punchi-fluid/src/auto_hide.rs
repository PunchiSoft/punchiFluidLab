// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Auto-hide: the dwell timers that open a rail and close an abandoned panel.
//!
//! `docs/ui-ux.md` §2.1: with auto-hide on, a dwell of about 350 ms on a closed panel's rail
//! opens it and about 800 ms without the pointer closes it again; a return cancels the timer,
//! and nothing fires while a pointer drag is in progress, while the keyboard focus is inside
//! the panel or while a dialog is open. The timers live here and nowhere else, they read
//! `Instant`, and the window only reports what the pointer and the keyboard did — so a
//! headless test drives the dwell with a made-up clock instead of sleeping.

use std::time::{Duration, Instant};

use eframe::egui;

use crate::Window;
use crate::layout::LayoutMode;

/// How long the pointer rests on a closed panel's rail before the panel opens.
pub const DWELL: Duration = Duration::from_millis(350);

/// How long the pointer stays away from an open panel before it closes.
pub const LINGER: Duration = Duration::from_millis(800);

/// Which panel a pair of timers belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Catalog,
    Inspector,
}

impl Slot {
    const ALL: [Self; 2] = [Self::Catalog, Self::Inspector];

    fn index(self) -> usize {
        match self {
            Self::Catalog => 0,
            Self::Inspector => 1,
        }
    }
}

/// One panel's two timers: the dwell on its rail while it is closed, and the wait away from it
/// while it is open. Only one of them is ever running for a given panel.
#[derive(Debug, Clone, Copy, Default)]
struct Timer {
    rail_since: Option<Instant>,
    leave_since: Option<Instant>,
}

/// The auto-hide timers of both panels, the one place that knows the dwell and the linger.
#[derive(Debug, Default)]
pub struct AutoHide {
    timers: [Timer; 2],
}

impl AutoHide {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The dwell on a closed panel's rail: `true` once the pointer has rested there for
    /// [`DWELL`]. A blocked frame — a drag, the keyboard inside the panel, a dialog — forgets
    /// the dwell, so the panel never opens behind a gesture the reader is making.
    pub fn rail(&mut self, slot: Slot, hovering: bool, blocked: bool, now: Instant) -> bool {
        let timer = &mut self.timers[slot.index()];
        if blocked || !hovering {
            timer.rail_since = None;
            return false;
        }
        let since = *timer.rail_since.get_or_insert(now);
        now.saturating_duration_since(since) >= DWELL
    }

    /// The wait away from an open panel: `true` once the pointer has been outside it for
    /// [`LINGER`]. A return to the panel, or a blocked frame, cancels the pending close.
    pub fn away(&mut self, slot: Slot, pointer_inside: bool, blocked: bool, now: Instant) -> bool {
        let timer = &mut self.timers[slot.index()];
        if blocked || pointer_inside {
            timer.leave_since = None;
            return false;
        }
        let since = *timer.leave_since.get_or_insert(now);
        now.saturating_duration_since(since) >= LINGER
    }

    /// Forgets one panel's timers, after the window changed its state by another path.
    pub fn cancel(&mut self, slot: Slot) {
        self.timers[slot.index()] = Timer::default();
    }

    /// Forgets every timer: auto-hide is off, or the panels are not on screen.
    pub fn cancel_all(&mut self) {
        self.timers = [Timer::default(); 2];
    }

    /// The time until a running timer fires, so the frame loop wakes for it even when the
    /// pointer and the keyboard are still. `None` when nothing is pending.
    #[must_use]
    pub fn wait(&self, now: Instant) -> Option<Duration> {
        let mut wait = None;
        for timer in &self.timers {
            for (since, limit) in [(timer.rail_since, DWELL), (timer.leave_since, LINGER)] {
                let Some(since) = since else {
                    continue;
                };
                let remaining = limit.saturating_sub(now.saturating_duration_since(since));
                wait = Some(wait.map_or(remaining, |wait: Duration| wait.min(remaining)));
            }
        }
        wait
    }
}

impl Window {
    /// Runs the auto-hide timers on the panels as they drew this frame. The frame's input state
    /// is read here and nowhere else: a pointer button held down is a drag in progress, and a
    /// dialog in front of the window suspends the whole behaviour.
    pub(crate) fn drive_auto_hide(&mut self, context: &egui::Context, now: Instant) {
        if !self.autohide_on || self.start_open {
            self.auto_hide.cancel_all();
            return;
        }
        let blocked = self.dialog_open() || context.input(|input| input.pointer.any_down());
        if self.apply_auto_hide(blocked, now) {
            // A panel opened or closed after the frame drew: draw the new state next frame.
            context.request_repaint();
        }
        if let Some(wait) = self.auto_hide.wait(now) {
            context.request_repaint_after(wait);
        }
    }

    /// The decision without the frame's input state, so the window tests can drive the timers
    /// with a made-up clock instead of sleeping. Returns whether the panels changed.
    pub(crate) fn apply_auto_hide(&mut self, blocked: bool, now: Instant) -> bool {
        if !self.autohide_on || self.start_open {
            self.auto_hide.cancel_all();
            return false;
        }
        match self.shell.mode {
            LayoutMode::Compact => self.auto_hide_compact(blocked, now),
            LayoutMode::Wide | LayoutMode::Medium => self.auto_hide_docked(blocked, now),
        }
    }

    /// The probe the panels left in the frame they last drew.
    fn probe(&self, slot: Slot) -> crate::panel::Probe {
        match slot {
            Slot::Catalog => self.sidebar.probe(),
            Slot::Inspector => self.inspector.probe(),
        }
    }

    /// The docked panels: each one opens from its rail after the dwell and closes after the
    /// pointer has been away for the linger. The keyboard inside a panel suspends its close.
    fn auto_hide_docked(&mut self, blocked: bool, now: Instant) -> bool {
        let mut changed = false;
        for slot in Slot::ALL {
            let probe = self.probe(slot);
            let open = match slot {
                Slot::Catalog => self.sidebar.is_open(),
                Slot::Inspector => self.inspector.is_open(),
            };
            let blocked = blocked || probe.focus_inside;
            if open {
                if self
                    .auto_hide
                    .away(slot, probe.pointer_inside, blocked, now)
                {
                    match slot {
                        Slot::Catalog => self.sidebar.close(),
                        Slot::Inspector => self.inspector.close(),
                    }
                    self.auto_hide.cancel(slot);
                    changed = true;
                }
            } else if self.auto_hide.rail(slot, probe.rail_hovered, blocked, now) {
                match slot {
                    Slot::Catalog => self.sidebar.open(),
                    Slot::Inspector => self.inspector.open(),
                }
                self.auto_hide.cancel(slot);
                changed = true;
            }
        }
        changed
    }

    /// The compact drawers: the rail opens the panel's drawer after the dwell, and the drawer
    /// closes after the pointer has been away for the linger. Only one drawer is ever open.
    fn auto_hide_compact(&mut self, blocked: bool, now: Instant) -> bool {
        if let Some(drawer) = self.shell.drawer {
            let slot = match drawer {
                crate::shell::Drawer::Catalog => Slot::Catalog,
                crate::shell::Drawer::Inspector => Slot::Inspector,
            };
            let probe = self.probe(slot);
            if self.auto_hide.away(
                slot,
                probe.pointer_inside,
                blocked || probe.focus_inside,
                now,
            ) {
                self.shell.drawer = None;
                self.auto_hide.cancel(slot);
                return true;
            }
            return false;
        }
        for slot in Slot::ALL {
            let probe = self.probe(slot);
            if self.auto_hide.rail(slot, probe.rail_hovered, blocked, now) {
                match slot {
                    Slot::Catalog => self.open_drawer(crate::shell::Drawer::Catalog),
                    Slot::Inspector => self.open_drawer(crate::shell::Drawer::Inspector),
                }
                self.auto_hide.cancel(slot);
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests;
