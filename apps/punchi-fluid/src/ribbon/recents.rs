// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The live documents list: the last ten, newest first. A file that is gone is named and
//! disabled, and two files with one name are told apart by their folder.

use std::path::Path;

use eframe::egui;
use punchi_fluid_file::recent::Recent;
use punchi_fluid_i18n::Language;

use super::Command;

pub(super) fn rows(
    ui: &mut egui::Ui,
    recents: &[Recent],
    picked: &mut Option<Command>,
    language: Language,
) {
    if recents.is_empty() {
        ui.add_enabled(
            false,
            egui::Button::new(language.text("ribbon.recents_empty")),
        );
        return;
    }
    for (index, recent) in recents.iter().enumerate() {
        let missing = !Path::new(&recent.path).is_file();
        // Two files can share a name; the folder is the only honest way to tell them apart,
        // and a missing one says so before the reader clicks.
        let shared = recents
            .iter()
            .filter(|other| other.name == recent.name)
            .count()
            > 1;
        let folder = Path::new(&recent.path)
            .parent()
            .and_then(Path::file_name)
            .and_then(std::ffi::OsStr::to_str);
        let label = match (missing, shared && folder.is_some()) {
            (true, true) => language.format(
                "ribbon.recents_missing_folder",
                &[
                    ("name", &recent.name),
                    ("folder", folder.unwrap_or_default()),
                ],
            ),
            (true, false) => language.format("ribbon.recents_missing", &[("name", &recent.name)]),
            (false, true) => language.format(
                "ribbon.recents_folder",
                &[
                    ("name", &recent.name),
                    ("folder", folder.unwrap_or_default()),
                ],
            ),
            (false, false) => recent.name.clone(),
        };
        if ui.add_enabled(!missing, egui::Button::new(label)).clicked() && !missing {
            *picked = Some(Command::OpenRecent(index));
        }
    }
}
