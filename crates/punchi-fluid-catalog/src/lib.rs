// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Component catalog.
//!
//! Every valve, cylinder or supply the simulator understands is a definition
//! file, not a Rust type. This crate owns the file format, the validation and the
//! directory layout, so adding a component is a matter of dropping a `.toml` file
//! next to the others, and removing one is a matter of deleting or disabling it.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

pub mod builtin;
pub mod catalog;
pub mod error;
pub mod lint;
pub mod reviews;
pub mod schema;

use std::path::PathBuf;

pub use catalog::Catalog;
pub use error::CatalogError;
pub use lint::LintError;
pub use schema::{
    Actuation, ActuatorKind, ActuatorPart, Behaviour, Category, ComponentDefinition, Connection,
    Kinematics, LineClass, Origin, Port, PortRole, Primitive, PrimitiveKind, ReturnKind, ReturnPart,
    SCHEMA_MIN, SCHEMA_VERSION, Side, SpringPart, ValvePosition,
};

/// Environment variable that points the application at another catalog folder.
pub const CATALOG_DIR_ENV: &str = "PUNCHI_FLUID_CATALOG_DIR";

/// Folder where the user keeps custom definitions.
///
/// Honours [`CATALOG_DIR_ENV`], then `XDG_DATA_HOME`, then `~/.local/share`.
#[must_use]
pub fn user_catalog_dir() -> Option<PathBuf> {
    if let Some(value) = std::env::var_os(CATALOG_DIR_ENV)
        && !value.is_empty()
    {
        return Some(PathBuf::from(value));
    }

    let base = std::env::var_os("XDG_DATA_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .filter(|value| !value.is_empty())
                .map(|home| PathBuf::from(home).join(".local").join("share"))
        })?;

    Some(base.join("punchi-fluid-lab").join("catalog"))
}

/// Built-in definitions plus whatever the user has in [`user_catalog_dir`].
#[must_use]
pub fn default_catalog() -> Catalog {
    let catalog = Catalog::builtin();
    match user_catalog_dir() {
        Some(dir) => catalog.with_user_dir(&dir),
        None => catalog,
    }
}
