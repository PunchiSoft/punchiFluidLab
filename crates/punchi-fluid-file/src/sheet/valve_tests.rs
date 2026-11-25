// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{SHEET_VERSION, Sheet, StoredEdits, sheet};

#[test]
fn format_fifteen_uses_the_catalog_position_and_the_new_field_is_bounded() {
    let mut old = sheet();
    old.format = 15;
    let current = Sheet::parse(&old.to_json().expect("old sheet writes")).expect("old sheet reads");
    assert_eq!(current.format, SHEET_VERSION);
    assert_eq!(
        current.placement[0]
            .edit
            .and_then(|edit| edit.valve_initial_position),
        None
    );

    let mut current = current;
    current.placement[0].edit = Some(StoredEdits {
        valve_initial_position: Some(2),
        ..StoredEdits::default()
    });
    assert!(current.validate().is_err());
}
