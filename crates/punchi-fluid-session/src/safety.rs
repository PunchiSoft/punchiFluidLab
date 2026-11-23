// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The safety copy: the work a crash would otherwise take.
//!
//! The window writes a copy of the document while it has unsaved changes, and reads it back
//! on the next start. It is **not** a save: the document stays unsaved, the path the reader
//! chose does not change, and recovering leaves the document with no path so the next save
//! asks for one (the ABI settled this in `pf_session_autosave`/`pf_session_recover`, and the
//! Rust window follows the same rule).
//!
//! The copy lives beside the recent list, in the user's data folder, and is removed when the
//! document is saved, opened or discarded. A crash leaves it there; that is its purpose.

use std::path::{Path, PathBuf};

use punchi_fluid_doc::Document;
use punchi_fluid_file::sheet::Sheet;

/// Environment variable that points the copy at another file, for tests and for a portable
/// copy.
pub const SAFETY_FILE_ENV: &str = "PUNCHI_FLUID_RECOVERY_FILE";

/// The safety copy, with the file it is read from and written to.
#[derive(Debug, Clone, Default)]
pub struct SafetyCopy {
    path: Option<PathBuf>,
}

impl SafetyCopy {
    /// The copy in the default place.
    #[must_use]
    pub fn load() -> Self {
        Self::at(default_path())
    }

    /// The copy at `path`; `None` keeps it disabled, which is what a test that does not want
    /// to touch the reader's folder uses.
    #[must_use]
    pub fn at(path: Option<PathBuf>) -> Self {
        Self { path }
    }

    /// The file the copy lives in, when it has one.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// `true` when a copy is waiting to be recovered.
    #[must_use]
    pub fn exists(&self) -> bool {
        self.path.as_deref().is_some_and(Path::exists)
    }

    /// Writes the document as the safety copy.
    ///
    /// Best-effort by contract: a full disk costs the copy, not the reader's work, and the
    /// next tick tries again. Returns whether the copy was written.
    #[must_use]
    pub fn write(&self, document: &Document) -> bool {
        let Some(path) = self.path.as_deref() else {
            return false;
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        punchi_fluid_session_save(document, path).is_ok()
    }

    /// Reads the copy back as a document.
    ///
    /// # Errors
    ///
    /// Returns the sheet reader's or the session's message when the copy cannot be read.
    pub fn read(&self) -> Result<Document, String> {
        let Some(path) = self.path.as_deref() else {
            return Err(String::from("no hay copia de seguridad"));
        };
        let sheet = Sheet::read(path).map_err(|cause| cause.to_string())?;
        punchi_fluid_session_load(&sheet)
    }

    /// Removes the copy, for a document that was saved, opened or discarded.
    ///
    /// Returns whether a file was removed. An absent copy is not an error: the window calls
    /// this on every save, and most saves have nothing to remove.
    #[must_use]
    pub fn discard(&self) -> bool {
        let Some(path) = self.path.as_deref() else {
            return false;
        };
        std::fs::remove_file(path).is_ok()
    }
}

/// The file the copy lives in: the environment override, else the user data folder.
#[must_use]
pub fn default_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os(SAFETY_FILE_ENV) {
        return Some(PathBuf::from(path));
    }
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
        })?;
    Some(base.join("punchi-fluid").join("recovery.pfluid"))
}

use crate::{load as punchi_fluid_session_load, save as punchi_fluid_session_save};

#[cfg(test)]
mod tests {
    use super::*;
    use punchi_fluid_doc::Document;
    use punchi_fluid_draw::surface::Pt;

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("punchi-fluid-safety-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("the scratch folder");
        dir
    }

    #[test]
    fn a_copy_is_written_read_and_discarded() {
        let dir = scratch("round-trip");
        let file = dir.join("recovery.pfluid");
        let copy = SafetyCopy::at(Some(file.clone()));
        assert!(!copy.exists());

        let document = Document::new();
        assert!(copy.write(&document));
        assert!(copy.exists());

        // Recovering reads a document, and leaves nothing behind until it is discarded.
        let recovered = copy.read().expect("the copy reads back");
        assert_eq!(recovered.circuits().len(), document.circuits().len());
        assert!(copy.discard());
        assert!(!copy.exists());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn an_unreadable_copy_is_an_error_and_a_discard_that_finds_nothing_is_not() {
        let dir = scratch("broken");
        let file = dir.join("recovery.pfluid");
        std::fs::write(&file, "no es un documento").expect("the broken copy");
        let copy = SafetyCopy::at(Some(file));
        assert!(copy.read().is_err());
        assert!(copy.discard());
        assert!(!copy.discard(), "nothing left to remove");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_copy_without_a_file_does_nothing_at_all() {
        let copy = SafetyCopy::at(None);
        assert!(!copy.exists());
        assert!(!copy.write(&Document::new()));
        assert!(copy.read().is_err());
        assert!(!copy.discard());
    }

    #[test]
    fn a_copy_keeps_the_document_it_was_written_from() {
        let dir = scratch("content");
        let file = dir.join("recovery.pfluid");
        let copy = SafetyCopy::at(Some(file));
        let mut document = Document::new();
        let placement = punchi_fluid_doc::Placement::new(
            "valve.3_2.spring_return",
            Pt::new(50.0, 50.0),
            20.0,
            20.0,
        );
        document.place(placement);
        assert!(copy.write(&document));
        let recovered = copy.read().expect("the copy reads back");
        assert_eq!(recovered.placements().len(), 1);
        std::fs::remove_dir_all(&dir).ok();
    }
}
