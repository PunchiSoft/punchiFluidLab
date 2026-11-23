// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The documents the start page offers to reopen.
//!
//! The list itself is [`punchi_fluid_file::recent::RecentList`]; this module owns **where**
//! it lives and when it is written, so the window does not have to know the file layout and
//! a second front end would share the same list. The file is user data, read defensively and
//! written best-effort: a full disk leaves the document saved and the list stale, which is the
//! order the reader cares about.

use std::path::{Path, PathBuf};

use punchi_fluid_file::recent::{Recent, RecentList};

/// Environment variable that points the list at another file, for tests and for a portable
/// copy. It is not documented as a feature: the list is an aid, not a document.
pub const RECENT_FILE_ENV: &str = "PUNCHI_FLUID_RECENT_FILE";

/// The remembered documents, with the file they are read from and written to.
#[derive(Debug, Clone, Default)]
pub struct Recents {
    path: Option<PathBuf>,
    list: RecentList,
}

impl Recents {
    /// Reads the list from the default place, or an empty one when there is none yet.
    #[must_use]
    pub fn load() -> Self {
        let path = default_path();
        Self::at(path)
    }

    /// Reads the list from `path`; `None` keeps it in memory only.
    #[must_use]
    pub fn at(path: Option<PathBuf>) -> Self {
        let list = path
            .as_deref()
            .and_then(|path| RecentList::read(path).ok())
            .unwrap_or_default();
        Self { path, list }
    }

    /// The remembered documents, newest first.
    #[must_use]
    pub fn entries(&self) -> &[Recent] {
        self.list.entries()
    }

    /// The file the list is written to, when it has one.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Puts a document at the front and writes the list.
    ///
    /// Returns `false` when the path cannot be remembered; the write is best-effort and does
    /// not change that answer, because the document is already saved either way.
    pub fn remember(&mut self, path: &Path) -> bool {
        let Some(text) = path.to_str() else {
            return false;
        };
        if !self.list.remember(text) {
            return false;
        }
        self.write();
        true
    }

    /// Drops one entry and writes the list.
    pub fn forget(&mut self, path: &str) -> bool {
        let dropped = self.list.forget(path);
        if dropped {
            self.write();
        }
        dropped
    }

    /// Writes the list, ignoring a failure: the reader's document is what must be safe.
    fn write(&self) {
        if let Some(path) = self.path.as_deref() {
            let _ = self.list.write(path);
        }
    }
}

/// The file the list lives in: the environment override, else the user data folder.
///
/// A session that cannot resolve the folder keeps the list in memory only, which is the
/// honest degraded mode: the start page still shows what this run opened.
#[must_use]
pub fn default_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os(RECENT_FILE_ENV) {
        return Some(PathBuf::from(path));
    }
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
        })?;
    Some(base.join("punchi-fluid").join("recent.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "punchi-fluid-recents-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&dir).expect("the scratch folder");
        dir
    }

    #[test]
    fn the_list_remembers_writes_and_forgets() {
        let dir = scratch();
        let file = dir.join("recent.json");
        let first = dir.join("uno.pff");
        let second = dir.join("dos.pff");

        let mut recents = Recents::at(Some(file.clone()));
        assert!(recents.entries().is_empty());
        assert!(recents.remember(&first));
        assert!(recents.remember(&second));
        assert_eq!(recents.entries().len(), 2);
        assert_eq!(recents.entries()[0].name, "dos.pff");

        // A new read sees what the previous one wrote.
        let reread = Recents::at(Some(file));
        assert_eq!(reread.entries().len(), 2);

        assert!(recents.forget(second.to_str().expect("a path")));
        assert_eq!(recents.entries().len(), 1);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_list_without_a_file_still_works_in_memory() {
        let mut recents = Recents::at(None);
        assert!(recents.remember(Path::new("/tmp/sin-destino.pff")));
        assert_eq!(recents.entries().len(), 1);
    }

    #[test]
    fn a_broken_file_reads_as_an_empty_list() {
        let dir = scratch();
        let file = dir.join("recent.json");
        std::fs::write(&file, "{ esto no es json").expect("the broken file");
        let recents = Recents::at(Some(file));
        assert!(recents.entries().is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }
}
