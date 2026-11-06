// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The review folder is part of the contract, not a courtesy.
//!
//! A symbol is *unverified* until `assets/symbols/review/<id>.md` exists, names the
//! symbol, states a status and is dated. The export reads this folder to decide whether
//! it may drop the "symbols not yet verified against ISO 1219-1" note, so a built-in
//! added without a review would otherwise ship looking conformant.

#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used, clippy::panic))]

use std::path::{Path, PathBuf};

use punchi_fluid_catalog::Catalog;

fn review_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/symbols/review")
}

#[test]
fn every_builtin_symbol_has_a_review_file() {
    let catalog = Catalog::builtin();
    let dir = review_dir();
    let mut missing = Vec::new();

    for definition in catalog.iter() {
        let path = dir.join(format!("{}.md", definition.id));
        if !path.is_file() {
            missing.push(definition.id.clone());
        }
    }

    assert!(
        missing.is_empty(),
        "these built-in symbols have no review file in {}: {}",
        dir.display(),
        missing.join(", ")
    );
}

#[test]
fn every_review_states_its_status_and_its_date() {
    let catalog = Catalog::builtin();
    let dir = review_dir();

    for definition in catalog.iter() {
        let path = dir.join(format!("{}.md", definition.id));
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));

        let status = text
            .lines()
            .find_map(|line| line.trim().strip_prefix("status:"))
            .map(str::trim)
            .unwrap_or_default();
        assert!(
            matches!(status, "complete" | "pending-300dpi"),
            "{}: status must be `complete` or `pending-300dpi`, found `{status}`",
            path.display()
        );

        assert!(
            text.lines().any(|line| line.trim().starts_with("date: 20")),
            "{}: a review needs a date",
            path.display()
        );
        assert!(
            text.contains(&definition.id),
            "{}: the review must name the symbol it covers",
            path.display()
        );
    }
}

#[test]
fn a_review_without_a_screenshot_is_not_a_review() {
    let catalog = Catalog::builtin();
    let dir = review_dir();

    for definition in catalog.iter() {
        for theme in ["light", "dark"] {
            let image = dir.join(format!("{}-{theme}.png", definition.id));
            assert!(
                image.is_file(),
                "{}: the review needs a {theme} screenshot next to it",
                image.display()
            );
        }
    }
}
