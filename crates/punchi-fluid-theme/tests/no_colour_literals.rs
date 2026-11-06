// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! A colour literal outside this crate is a bug, and this test says where.

use std::path::{Path, PathBuf};

const ALLOWED_CRATE: &str = "punchi-fluid-theme";
const FORBIDDEN: &str = "Color32::";

#[test]
fn colours_are_only_named_in_the_theme_crate() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the theme crate lives inside crates/");
    let mut files = Vec::new();
    collect(crates, &mut files);

    let mut offenders = Vec::new();
    for file in files {
        if file
            .components()
            .any(|part| part.as_os_str() == ALLOWED_CRATE)
        {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        for (number, line) in text.lines().enumerate() {
            if line.contains(FORBIDDEN) {
                offenders.push(format!(
                    "{}:{}: {}",
                    file.display(),
                    number + 1,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "colour literals belong in `{ALLOWED_CRATE}`:\n{}",
        offenders.join("\n")
    );
}

fn collect(directory: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}
