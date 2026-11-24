// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The card images: one normalized render per catalog id, compiled in for the
//! **Descripción del componente** window.
//!
//! The picture is descriptive, not the schematic: the sheet keeps drawing the ISO 1219
//! symbol from the catalog file. A card is added only for a definition that exists, and
//! its origin and licence live in `assets/catalog/cards/README.md`.

/// One card image and the catalog id it belongs to.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Card {
    pub(crate) catalog_id: &'static str,
    pub(crate) uri: &'static str,
    pub(crate) bytes: &'static [u8],
}

/// The cards compiled into the binary.
pub(crate) const CARDS: &[Card] = &[Card {
    catalog_id: "cylinder.single_acting_20_100",
    uri: "bytes://cards/cylinder.single_acting_20_100.png",
    bytes: include_bytes!("../../../assets/catalog/cards/cylinder.single_acting_20_100.png"),
}];

/// The card of one definition, when it has one.
#[must_use]
pub(crate) fn image(catalog_id: &str) -> Option<&'static Card> {
    CARDS.iter().find(|card| card.catalog_id == catalog_id)
}

#[cfg(test)]
mod tests;
