// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Colour scheme from the desktop portal.
//!
//! `org.freedesktop.appearance color-scheme` answers `0` (no preference), `1`
//! (prefer dark) or `2` (prefer light), and `SettingChanged` reports a live change.
//! The read and the subscription happen on a background thread: a missing or slow
//! bus must never delay the first frame, and a missing portal is a normal case.

use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

use crate::Scheme;

use zbus::zvariant::OwnedValue;

const NAMESPACE: &str = "org.freedesktop.appearance";
const KEY: &str = "color-scheme";

/// Live subscription to the desktop's colour-scheme preference.
#[derive(Debug)]
pub struct SchemeWatcher {
    receiver: Receiver<Scheme>,
}

impl SchemeWatcher {
    /// Starts the background reader.
    ///
    /// The first value arrives as soon as the portal answers; without a session bus
    /// the watcher reports `NoPreference` once and stays quiet.
    #[must_use]
    pub fn spawn() -> Self {
        let (sender, receiver) = channel();
        let spawned = thread::Builder::new()
            .name(String::from("punchi-fluid-portal"))
            .spawn(move || watch(&sender));
        if let Err(error) = &spawned {
            // Not fatal: the theme stays on the value read from kdeglobals.
            eprintln!("theme: could not start the portal watcher: {error}");
        }
        Self { receiver }
    }

    /// The newest scheme the portal reported, if any.
    #[must_use]
    pub fn received(&self) -> Option<Scheme> {
        self.receiver.try_recv().ok()
    }
}

fn watch(sender: &Sender<Scheme>) {
    let Ok(connection) = zbus::blocking::Connection::session() else {
        let _ = sender.send(Scheme::NoPreference);
        return;
    };
    let Ok(proxy) = SettingsProxyBlocking::new(&connection) else {
        let _ = sender.send(Scheme::NoPreference);
        return;
    };

    let initial = proxy
        .read_one(NAMESPACE, KEY)
        .map_or(Scheme::NoPreference, |value| scheme_of(&value));
    if sender.send(initial).is_err() {
        return;
    }

    let Ok(signals) = proxy.receive_setting_changed() else {
        return;
    };
    for signal in signals {
        let Ok(arguments) = signal.args() else {
            continue;
        };
        if arguments.namespace == NAMESPACE
            && arguments.key == KEY
            && sender.send(scheme_of(&arguments.value)).is_err()
        {
            return;
        }
    }
}

fn scheme_of(value: &OwnedValue) -> Scheme {
    u32::try_from(value.clone()).map_or(Scheme::NoPreference, Scheme::from_portal)
}

#[zbus::proxy(
    interface = "org.freedesktop.portal.Settings",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait Settings {
    fn read_one(&self, namespace: &str, key: &str) -> zbus::Result<zbus::zvariant::OwnedValue>;

    #[zbus(signal)]
    fn setting_changed(&self, namespace: &str, key: &str, value: zbus::zvariant::OwnedValue);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portal_numbers_map_to_schemes() {
        assert_eq!(Scheme::from_portal(0), Scheme::NoPreference);
        assert_eq!(Scheme::from_portal(1), Scheme::Dark);
        assert_eq!(Scheme::from_portal(2), Scheme::Light);
        assert_eq!(Scheme::from_portal(7), Scheme::NoPreference);
    }

    #[test]
    fn the_watcher_never_panics_without_a_bus() {
        let watcher = SchemeWatcher::spawn();
        // Either the portal answered or it did not; both are valid on a build machine.
        let _ = watcher.received();
    }
}
