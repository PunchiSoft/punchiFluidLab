// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Failures that can appear while building the catalog.

use std::path::PathBuf;

use thiserror::Error;

/// Why a definition file was not accepted.
#[derive(Debug, Error)]
pub enum CatalogError {
    /// The file could not be read.
    #[error("cannot read {path}: {source}")]
    Read {
        /// File that failed.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// The file is not valid TOML for this schema.
    #[error("cannot parse {path}: {source}")]
    Parse {
        /// File that failed.
        path: PathBuf,
        /// Parser error with line and column.
        #[source]
        source: Box<toml::de::Error>,
    },

    /// The file parses but says something impossible.
    #[error("{path}: {reason}")]
    Invalid {
        /// File that failed.
        path: PathBuf,
        /// What is wrong, in one sentence.
        reason: String,
    },

    /// The file was written for another version of the format.
    #[error(
        "{path}: schema {found} is not supported, this build reads schema {} to {}",
        super::schema::SCHEMA_MIN,
        super::schema::SCHEMA_VERSION
    )]
    UnsupportedSchema {
        /// File that failed.
        path: PathBuf,
        /// Version declared by the file.
        found: u32,
    },
}
