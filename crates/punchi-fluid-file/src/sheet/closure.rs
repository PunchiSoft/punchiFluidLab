// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading and checking the port-closure map of a stored placement.

use std::collections::BTreeMap;

use super::{MAX_ID, StoredPlacement};

/// Reads the port-closure map, refusing a port that appears twice.
///
/// A `BTreeMap` alone would silently keep the last value of a duplicate JSON key; a file
/// is untrusted input, so the reader reports the repetition instead of choosing for it.
pub(super) fn deserialize_port_closures<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<String, String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{Error, MapAccess, Visitor};
    use std::fmt;

    struct PortClosures;

    impl<'de> Visitor<'de> for PortClosures {
        type Value = BTreeMap<String, String>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a map of port id to closure id")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut closures = BTreeMap::new();
            while let Some((port, closure)) = map.next_entry::<String, String>()? {
                if closures.insert(port, closure).is_some() {
                    return Err(A::Error::custom("a port is closed twice"));
                }
            }
            Ok(closures)
        }
    }

    deserializer.deserialize_map(PortClosures)
}

/// `true` when a port or closure id is non-empty, ASCII and at most [`MAX_ID`] bytes.
pub(super) fn identifier_is_valid(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_ID && value.is_ascii()
}

/// `true` when the named port of the component carries a closure.
pub(super) fn is_closed(placements: &[StoredPlacement], placement: u64, port: &str) -> bool {
    placements
        .iter()
        .find(|candidate| candidate.id == placement)
        .is_some_and(|candidate| candidate.port_closures.contains_key(port))
}
