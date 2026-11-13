// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The small rules a switch can carry: when a relay follows its coil, and which end of a
//! cylinder a limit switch watches.
//!
//! They live apart from the rest of the schema because they are behaviour vocabulary used
//! by more than one definition, and because the schema file has a line budget like every
//! other source.

use serde::{Deserialize, Serialize};

/// The end of a cylinder's stroke a limit switch watches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LimitEnd {
    /// The rod is out: the roller is pressed at the far end.
    #[default]
    Extended,
    /// The rod is home: the roller is pressed at the near end.
    Retracted,
}

impl LimitEnd {
    /// Stable name written in the definition file.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Extended => "extended",
            Self::Retracted => "retracted",
        }
    }
}

/// When a relay's contact follows its coil.
///
/// The names and the behaviour follow IEC 61131-3: `ton` is an on-delay and `tof` an
/// off-delay. An `instant` relay is the part with no timer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelayKind {
    /// The contact follows the coil at once.
    #[default]
    Instant,
    /// On-delay: the contact closes after the coil has been live for the delay.
    Ton,
    /// Off-delay: the contact opens after the coil has been dead for the delay.
    Tof,
}

impl RelayKind {
    /// Stable name written in the definition file.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Instant => "instant",
            Self::Ton => "ton",
            Self::Tof => "tof",
        }
    }

    /// `true` when the part is a timer rather than an instantaneous relay.
    #[must_use]
    pub const fn is_timer(self) -> bool {
        !matches!(self, Self::Instant)
    }
}
