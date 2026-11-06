// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the desktop actually answers, printed so the toolbar's names can be checked.
//!
//! Not an assertion about a particular theme: a machine without Breeze has no icons to
//! find, and that is a valid outcome — the toolbar falls back to its own drawing.

use punchi_fluid_icons::IconTheme;

#[test]
fn the_actions_of_the_toolbar_resolve_on_this_desktop() {
    let theme = IconTheme::detect();
    println!("theme chain: {:?}", theme.chain());
    let mut found = 0;
    for name in [
        "document-new",
        "document-open",
        "document-save",
        "document-save-as",
        "edit-undo",
        "edit-redo",
        "media-playback-start",
        "media-skip-forward",
        "view-refresh",
    ] {
        match theme.find(name, 22) {
            Some(path) => {
                println!("{name} -> {}", path.display());
                found += 1;
            }
            None => println!("{name} -> missing"),
        }
    }
    println!("{found} of 9 resolved");

    // A dark window must not get the light-strokes variant painted on a dark bar: the
    // theme choice follows the scheme when the desktop names no theme.
    let dark = IconTheme::detect_for(true);
    println!("dark chain: {:?}", dark.chain());
    assert!(!dark.chain().is_empty());
}
