// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Nominal outside diameters of pneumatic tubing.
//!
//! A tube is a property of a connection, not a component and not a catalog file. The value
//! travels as **integer hundredths of a millimetre** so the document and the sheet keep
//! `Eq`/`Hash` and the history can compare two pipes exactly, the way the middle-segment
//! offset already does.
//!
//! The entries below are this project's own nominal practice list: the metric series a
//! workshop stocks and the inch sizes that are actually sold as tubing. They are not a
//! normative ISO table — ISO 1219 defines symbols, not tube diameters — and the reference
//! record says so in [docs/reference/tube.md](../../../../docs/reference/tube.md).

/// Smallest accepted nominal outside diameter, in hundredths of a millimetre (1 mm).
pub const MIN_TUBE_HUNDREDTHS: u16 = 100;

/// Largest accepted nominal outside diameter, in hundredths of a millimetre (50 mm).
pub const MAX_TUBE_HUNDREDTHS: u16 = 5000;

/// The diameter a new pneumatic connection starts with, in hundredths (10 mm).
///
/// It is a documented constant, not a page setting: the gallery changes the pipe it is
/// applied to, and a second global default would be a second source of truth.
pub const DEFAULT_TUBE_HUNDREDTHS: u16 = 1000;

/// Which practice list an entry belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nominal {
    /// The metric series, named in millimetres.
    Metric,
    /// The inch series, named in inches and stored as the exact converted value.
    Inch,
}

/// One entry of the nominal series.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TubeSize {
    /// Nominal outside diameter, hundredths of a millimetre.
    pub hundredths: u16,
    /// The name the interface shows for this entry.
    pub label: &'static str,
    /// Which series the entry belongs to.
    pub nominal: Nominal,
}

/// The metric entries: 4 to 50 mm.
const METRIC: &[TubeSize] = &[
    TubeSize {
        hundredths: 400,
        label: "4 mm",
        nominal: Nominal::Metric,
    },
    TubeSize {
        hundredths: 500,
        label: "5 mm",
        nominal: Nominal::Metric,
    },
    TubeSize {
        hundredths: 600,
        label: "6 mm",
        nominal: Nominal::Metric,
    },
    TubeSize {
        hundredths: 800,
        label: "8 mm",
        nominal: Nominal::Metric,
    },
    TubeSize {
        hundredths: 1000,
        label: "10 mm",
        nominal: Nominal::Metric,
    },
    TubeSize {
        hundredths: 1200,
        label: "12 mm",
        nominal: Nominal::Metric,
    },
    TubeSize {
        hundredths: 1400,
        label: "14 mm",
        nominal: Nominal::Metric,
    },
    TubeSize {
        hundredths: 1600,
        label: "16 mm",
        nominal: Nominal::Metric,
    },
    TubeSize {
        hundredths: 2000,
        label: "20 mm",
        nominal: Nominal::Metric,
    },
    TubeSize {
        hundredths: 2500,
        label: "25 mm",
        nominal: Nominal::Metric,
    },
    TubeSize {
        hundredths: 3200,
        label: "32 mm",
        nominal: Nominal::Metric,
    },
    TubeSize {
        hundredths: 4000,
        label: "40 mm",
        nominal: Nominal::Metric,
    },
    TubeSize {
        hundredths: 5000,
        label: "50 mm",
        nominal: Nominal::Metric,
    },
];

/// The inch entries, converted exactly and rounded to the hundredth of a millimetre.
const INCH: &[TubeSize] = &[
    TubeSize {
        hundredths: 318,
        label: "1/8\"",
        nominal: Nominal::Inch,
    },
    TubeSize {
        hundredths: 397,
        label: "5/32\"",
        nominal: Nominal::Inch,
    },
    TubeSize {
        hundredths: 476,
        label: "3/16\"",
        nominal: Nominal::Inch,
    },
    TubeSize {
        hundredths: 635,
        label: "1/4\"",
        nominal: Nominal::Inch,
    },
    TubeSize {
        hundredths: 794,
        label: "5/16\"",
        nominal: Nominal::Inch,
    },
    TubeSize {
        hundredths: 953,
        label: "3/8\"",
        nominal: Nominal::Inch,
    },
    TubeSize {
        hundredths: 1270,
        label: "1/2\"",
        nominal: Nominal::Inch,
    },
    TubeSize {
        hundredths: 1588,
        label: "5/8\"",
        nominal: Nominal::Inch,
    },
    TubeSize {
        hundredths: 1905,
        label: "3/4\"",
        nominal: Nominal::Inch,
    },
    TubeSize {
        hundredths: 2540,
        label: "1\"",
        nominal: Nominal::Inch,
    },
];

/// Every entry of the series, metric first and inch second.
///
/// The order is the menu's: the two groups are separate entries, because 10 mm and 3/8"
/// are different tubes that a shop stocks for different reasons.
pub const TUBE_SIZES: &[TubeSize] = &[
    METRIC[0], METRIC[1], METRIC[2], METRIC[3], METRIC[4], METRIC[5], METRIC[6], METRIC[7],
    METRIC[8], METRIC[9], METRIC[10], METRIC[11], METRIC[12], INCH[0], INCH[1], INCH[2], INCH[3],
    INCH[4], INCH[5], INCH[6], INCH[7], INCH[8], INCH[9],
];

/// The entry whose stored value is `hundredths`, if this is a named size.
#[must_use]
pub fn tube_size(hundredths: u16) -> Option<&'static TubeSize> {
    TUBE_SIZES.iter().find(|size| size.hundredths == hundredths)
}

/// `true` when `hundredths` is inside the accepted range.
#[must_use]
pub const fn is_valid_tube(hundredths: u16) -> bool {
    hundredths >= MIN_TUBE_HUNDREDTHS && hundredths <= MAX_TUBE_HUNDREDTHS
}

/// What the interface prints for a stored value.
///
/// A named entry keeps its name (`10 mm`, `3/8"`); any other accepted value is shown as a
/// measurement with the decimal comma the interface uses (`10,50 mm`).
#[must_use]
pub fn tube_label(hundredths: u16) -> String {
    if let Some(size) = tube_size(hundredths) {
        return size.label.to_owned();
    }
    let whole = hundredths / 100;
    let fraction = hundredths % 100;
    if fraction == 0 {
        format!("{whole} mm")
    } else {
        format!("{whole},{fraction:02} mm")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_series_is_metric_then_inch_with_the_documented_values() {
        assert_eq!(TUBE_SIZES.len(), 23);
        let metric = TUBE_SIZES
            .iter()
            .take_while(|size| size.nominal == Nominal::Metric)
            .count();
        assert_eq!(metric, 13);
        assert!(
            TUBE_SIZES[metric..]
                .iter()
                .all(|size| size.nominal == Nominal::Inch)
        );
        let stored: Vec<u16> = TUBE_SIZES.iter().map(|size| size.hundredths).collect();
        for (index, value) in stored.iter().enumerate() {
            assert!(
                !stored[index + 1..].contains(value),
                "{value} appears twice in the series"
            );
        }
        for size in TUBE_SIZES {
            assert!(is_valid_tube(size.hundredths), "{size:?} is out of bounds");
        }
        assert_eq!(tube_size(1000).map(|size| size.label), Some("10 mm"));
        assert_eq!(tube_size(953).map(|size| size.label), Some("3/8\""));
        assert_eq!(tube_size(1050), None);
    }

    #[test]
    fn a_named_entry_keeps_its_name_and_a_custom_value_is_a_measurement() {
        assert_eq!(tube_label(1000), "10 mm");
        assert_eq!(tube_label(318), "1/8\"");
        assert_eq!(tube_label(1050), "10,50 mm");
        assert_eq!(tube_label(1005), "10,05 mm");
        assert_eq!(tube_label(1100), "11 mm");
    }

    #[test]
    fn the_bounds_are_one_and_fifty_millimetres() {
        assert_eq!(MIN_TUBE_HUNDREDTHS, 100);
        assert_eq!(MAX_TUBE_HUNDREDTHS, 5000);
        assert!(!is_valid_tube(99));
        assert!(is_valid_tube(100));
        assert!(is_valid_tube(5000));
        assert!(!is_valid_tube(5001));
    }
}
