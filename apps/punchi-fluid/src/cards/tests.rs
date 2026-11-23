// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The card table and its normalized images, checked without a window.

use punchi_fluid_catalog::default_catalog;

use super::{CARDS, image};

#[test]
fn every_card_names_a_builtin_definition() {
    let catalog = default_catalog();
    for card in CARDS {
        assert!(
            catalog.get(card.catalog_id).is_some(),
            "the card `{}` has no definition",
            card.catalog_id
        );
    }
}

#[test]
fn every_card_is_normalized_to_800_by_450() {
    for card in CARDS {
        let bytes = card.bytes;
        assert_eq!(&bytes[1..4], b"PNG", "{} is not a PNG", card.catalog_id);
        let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        assert_eq!(
            (width, height),
            (800, 450),
            "{} is {width}x{height}",
            card.catalog_id
        );
        assert!(
            std::path::Path::new(card.uri)
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("png")),
            "{} has no PNG image URI",
            card.catalog_id
        );
    }
}

#[test]
fn the_card_folder_records_its_licence() {
    let readme = include_str!("../../../../assets/catalog/cards/README.md");
    for card in CARDS {
        assert!(
            readme.contains(card.catalog_id),
            "{} has no row in the card README",
            card.catalog_id
        );
        assert!(
            readme.contains("GPL-3.0-or-later"),
            "the card README has no licence"
        );
    }
}

#[test]
fn the_simple_cylinder_card_is_the_one_the_window_uses() {
    let card = image("cylinder.single_acting_20_100").expect("the example card");
    assert_eq!(card.catalog_id, "cylinder.single_acting_20_100");
    assert!(image("valve.3_2.spring_return").is_none());
}
