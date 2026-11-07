// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Definitions compiled into the binary.
//!
//! The source files live in `assets/catalog`, so the same file can be edited in
//! the repository or copied to the user catalog directory to override it.

/// Name and contents of every built-in definition file.
pub const FILES: &[(&str, &str)] = &[
    (
        "supply.workshop.toml",
        include_str!("../../../assets/catalog/supply.workshop.toml"),
    ),
    (
        "valve.3_2.spring_return.toml",
        include_str!("../../../assets/catalog/valve.3_2.spring_return.toml"),
    ),
    (
        "valve.5_2.spring_return.toml",
        include_str!("../../../assets/catalog/valve.5_2.spring_return.toml"),
    ),
    (
        "cylinder.single_acting_20_100.toml",
        include_str!("../../../assets/catalog/cylinder.single_acting_20_100.toml"),
    ),
    (
        "cylinder.double_acting_32_200.toml",
        include_str!("../../../assets/catalog/cylinder.double_acting_32_200.toml"),
    ),
    (
        "gauge.pressure.toml",
        include_str!("../../../assets/catalog/gauge.pressure.toml"),
    ),
    (
        "throttle.fixed.toml",
        include_str!("../../../assets/catalog/throttle.fixed.toml"),
    ),
    (
        "junction.tee.toml",
        include_str!("../../../assets/catalog/junction.tee.toml"),
    ),
];
