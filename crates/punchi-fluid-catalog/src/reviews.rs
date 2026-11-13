// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The signed-off reviews of the built-in symbols, compiled into the binary.
//!
//! A definition and its review travel together: the export has to know, at runtime and
//! without reading a folder that an installed copy does not have, whether it may drop
//! the "not verified" note. The same files are checked on disk by
//! `tests/review.rs`, so the binary and the repository cannot disagree.

/// Review file of every built-in definition, as `(id, text)`.
pub const REVIEWS: &[(&str, &str)] = &[
    (
        "cylinder.single_acting_20_100",
        include_str!("../../../assets/symbols/review/cylinder.single_acting_20_100.md"),
    ),
    (
        "cylinder.double_acting_32_200",
        include_str!("../../../assets/symbols/review/cylinder.double_acting_32_200.md"),
    ),
    (
        "supply.compressor",
        include_str!("../../../assets/symbols/review/supply.compressor.md"),
    ),
    (
        "supply.network",
        include_str!("../../../assets/symbols/review/supply.network.md"),
    ),
    (
        "preparation.service_unit",
        include_str!("../../../assets/symbols/review/preparation.service_unit.md"),
    ),
    (
        "preparation.service_unit_simple",
        include_str!("../../../assets/symbols/review/preparation.service_unit_simple.md"),
    ),
    (
        "filter.air",
        include_str!("../../../assets/symbols/review/filter.air.md"),
    ),
    (
        "filter.air_manual_drain",
        include_str!("../../../assets/symbols/review/filter.air_manual_drain.md"),
    ),
    (
        "lubricator.oil",
        include_str!("../../../assets/symbols/review/lubricator.oil.md"),
    ),
    (
        "throttle.fixed",
        include_str!("../../../assets/symbols/review/throttle.fixed.md"),
    ),
    (
        "valve.3_2.spring_return",
        include_str!("../../../assets/symbols/review/valve.3_2.spring_return.md"),
    ),
    (
        "valve.5_2.spring_return",
        include_str!("../../../assets/symbols/review/valve.5_2.spring_return.md"),
    ),
    (
        "junction.tee",
        include_str!("../../../assets/symbols/review/junction.tee.md"),
    ),
    (
        "gauge.pressure",
        include_str!("../../../assets/symbols/review/gauge.pressure.md"),
    ),
    (
        "flow.meter",
        include_str!("../../../assets/symbols/review/flow.meter.md"),
    ),
    (
        "chart.recorder",
        include_str!("../../../assets/symbols/review/chart.recorder.md"),
    ),
    (
        "plug.blind",
        include_str!("../../../assets/symbols/review/plug.blind.md"),
    ),
    (
        "silencer.pneumatic",
        include_str!("../../../assets/symbols/review/silencer.pneumatic.md"),
    ),
    (
        "electrical.supply",
        include_str!("../../../assets/symbols/review/electrical.supply.md"),
    ),
    (
        "electrical.contact",
        include_str!("../../../assets/symbols/review/electrical.contact.md"),
    ),
    (
        "electrical.relay",
        include_str!("../../../assets/symbols/review/electrical.relay.md"),
    ),
    (
        "electrical.timer.ton",
        include_str!("../../../assets/symbols/review/electrical.timer.ton.md"),
    ),
    (
        "electrical.timer.tof",
        include_str!("../../../assets/symbols/review/electrical.timer.tof.md"),
    ),
    (
        "valve.3_2.solenoid",
        include_str!("../../../assets/symbols/review/valve.3_2.solenoid.md"),
    ),
];

/// Review text of one definition.
#[must_use]
pub fn of(id: &str) -> Option<&'static str> {
    REVIEWS
        .iter()
        .find(|(name, _)| *name == id)
        .map(|(_, text)| *text)
}

/// Status written in a review file, or `None` when it has none.
#[must_use]
pub fn status(text: &str) -> Option<&str> {
    text.lines()
        .find_map(|line| line.trim().strip_prefix("status:"))
        .map(str::trim)
}

/// `true` when every built-in symbol carries a complete review.
///
/// Only then may a document stop calling its own symbols unverified.
#[must_use]
pub fn all_complete() -> bool {
    REVIEWS
        .iter()
        .all(|(_, text)| status(text) == Some("complete"))
}

/// Every symptom of a review that is still open, for the export note.
#[must_use]
pub fn pending() -> Vec<&'static str> {
    REVIEWS
        .iter()
        .filter(|(_, text)| status(text) != Some("complete"))
        .map(|(id, _)| *id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_builtin_definition_has_a_compiled_review() {
        for (name, _) in crate::builtin::FILES {
            let id = name.trim_end_matches(".toml");
            assert!(of(id).is_some(), "{id} has no compiled review");
        }
    }

    #[test]
    fn every_review_declares_a_known_status() {
        for (id, text) in REVIEWS {
            let status = status(text).unwrap_or_default();
            assert!(
                matches!(status, "complete" | "pending-300dpi"),
                "{id} has status `{status}`"
            );
        }
    }

    #[test]
    fn the_export_note_is_on_while_any_review_is_open() {
        if all_complete() {
            assert!(pending().is_empty());
        } else {
            assert!(!pending().is_empty());
        }
    }
}
