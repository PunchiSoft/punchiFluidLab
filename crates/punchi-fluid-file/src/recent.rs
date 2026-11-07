// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The last documents opened, so the user does not have to walk the file dialog again.
//!
//! The list is a file like any other: untrusted input, bounded, versioned and written
//! atomically. It holds paths and nothing else — no content of the documents — so a list
//! that is lost or corrupted costs a few menu entries and never a circuit.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Format version written by this build.
pub const RECENT_VERSION: u32 = 1;

/// Most documents the list remembers.
pub const MAX_RECENT: usize = 10;

/// Longest path the list accepts, in bytes.
pub const MAX_PATH: usize = 4096;

/// Largest list file the reader will load, in bytes.
pub const MAX_BYTES: u64 = 64 * 1024;

/// A remembered document: the file and how it is called in the menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recent {
    /// Absolute path of the document.
    pub path: String,
    /// File name, kept beside the path so the menu does not have to split a string.
    pub name: String,
}

impl Recent {
    /// A remembered document from its path; the name is its last component.
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        let path = path.into();
        let name = Path::new(&path)
            .file_name()
            .map_or_else(|| path.clone(), |name| name.to_string_lossy().into_owned());
        Self { path, name }
    }
}

/// The remembered documents, newest first.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecentList {
    entries: Vec<Recent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Stored {
    version: u32,
    #[serde(default)]
    path: Vec<String>,
}

/// Everything that can go wrong while reading or writing the list.
#[derive(Debug, thiserror::Error)]
pub enum RecentError {
    /// The file is not the JSON this build writes.
    #[error("invalid recent list: {0}")]
    Parse(#[from] serde_json::Error),

    /// The file was written by a newer, unknown version.
    #[error("recent list version {found} is newer than this build understands ({supported})")]
    UnsupportedVersion {
        /// Version found in the file.
        found: u32,
        /// Highest version this build understands.
        supported: u32,
    },

    /// The file is larger than the reader will load.
    #[error("the recent list is larger than {MAX_BYTES} bytes")]
    TooLarge,

    /// The file could not be read or written.
    #[error("recent list I/O failed: {0}")]
    Io(#[from] std::io::Error),
}

impl RecentList {
    /// An empty list.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The entries, newest first.
    #[must_use]
    pub fn entries(&self) -> &[Recent] {
        &self.entries
    }

    /// How many documents are remembered.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `true` when nothing is remembered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Puts a document at the front, dropping an older entry for the same path.
    ///
    /// A path longer than [`MAX_PATH`] is refused instead of stored and truncated: a
    /// truncated path is a menu entry that opens the wrong file or none.
    pub fn remember(&mut self, path: &str) -> bool {
        if path.is_empty() || path.len() > MAX_PATH || path.contains('\0') {
            return false;
        }
        self.entries.retain(|entry| entry.path != path);
        self.entries.insert(0, Recent::new(path));
        self.entries.truncate(MAX_RECENT);
        true
    }

    /// Drops one path, for a document that moved or was deleted.
    pub fn forget(&mut self, path: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|entry| entry.path != path);
        self.entries.len() != before
    }

    /// Drops every entry whose file is no longer there.
    ///
    /// The list exists so its entries can be opened, so an entry that cannot be opened is
    /// noise in the menu. A file that is merely unreadable stays: it may be a permissions
    /// problem the user can fix, and silently forgetting it would be worse.
    pub fn prune_missing(&mut self) -> usize {
        let before = self.entries.len();
        self.entries.retain(|entry| Path::new(&entry.path).exists());
        before - self.entries.len()
    }

    /// The entries as the stored shape.
    fn to_stored(&self) -> Stored {
        Stored {
            version: RECENT_VERSION,
            path: self
                .entries
                .iter()
                .map(|entry| entry.path.clone())
                .collect(),
        }
    }

    /// Reads the list, pruning entries whose file is gone.
    ///
    /// A missing file is an empty list, not an error: the first run has no list, and a
    /// program that refuses to start because its own convenience file is absent is worse
    /// than one without a recent menu.
    ///
    /// # Errors
    ///
    /// Returns [`RecentError::TooLarge`] before reading when the file exceeds
    /// [`MAX_BYTES`], and the parse and version errors of [`Self::parse`].
    pub fn read(path: &Path) -> Result<Self, RecentError> {
        let size = match fs::metadata(path) {
            Ok(metadata) => metadata.len(),
            Err(cause) if cause.kind() == std::io::ErrorKind::NotFound => return Ok(Self::new()),
            Err(cause) => return Err(RecentError::Io(cause)),
        };
        if size > MAX_BYTES {
            return Err(RecentError::TooLarge);
        }
        let mut list = Self::parse(&fs::read_to_string(path)?)?;
        list.prune_missing();
        Ok(list)
    }

    /// Parses a list from text, bounding everything a file could get wrong.
    ///
    /// # Errors
    ///
    /// Returns [`RecentError::Parse`] for invalid JSON and
    /// [`RecentError::UnsupportedVersion`] for a list this build cannot read.
    pub fn parse(text: &str) -> Result<Self, RecentError> {
        let stored: Stored = serde_json::from_str(text)?;
        if stored.version > RECENT_VERSION {
            return Err(RecentError::UnsupportedVersion {
                found: stored.version,
                supported: RECENT_VERSION,
            });
        }
        let mut list = Self::new();
        // Newest first, and the loop reverses so the most recent recorded path ends up on
        // top whatever order the file was written in.
        for path in stored.path.iter().rev() {
            list.remember(path);
        }
        Ok(list)
    }

    /// Writes the list atomically: a temporary file beside it, then a rename.
    ///
    /// # Errors
    ///
    /// Returns the serialisation error or [`RecentError::Io`].
    pub fn write(&self, path: &Path) -> Result<(), RecentError> {
        let text = serde_json::to_string_pretty(&self.to_stored())?;
        write_atomically(path, &text)?;
        Ok(())
    }
}

/// Writes `text` to `path` through a temporary file in the same folder.
fn write_atomically(path: &Path, text: &str) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }
    let temporary = temporary_path(path);
    fs::write(&temporary, text)?;
    fs::rename(&temporary, path)?;
    Ok(())
}

/// The temporary file used for the atomic write, beside the target.
fn temporary_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    path.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list() -> RecentList {
        let mut list = RecentList::new();
        list.remember("/home/alguien/uno.pfluid");
        list.remember("/home/alguien/dos.pfluid");
        list
    }

    #[test]
    fn the_newest_document_is_first_and_a_repeat_moves_up() {
        let mut list = list();
        assert_eq!(list.entries()[0].name, "dos.pfluid");
        assert_eq!(list.entries()[1].name, "uno.pfluid");
        assert!(list.remember("/home/alguien/uno.pfluid"));
        assert_eq!(list.len(), 2, "the repeat replaced the old entry");
        assert_eq!(list.entries()[0].name, "uno.pfluid");
    }

    #[test]
    fn the_list_stops_at_the_limit() {
        let mut list = RecentList::new();
        for index in 0..(MAX_RECENT + 5) {
            assert!(list.remember(&format!("/tmp/{index}.pfluid")));
        }
        assert_eq!(list.len(), MAX_RECENT);
        assert_eq!(list.entries()[0].path, "/tmp/14.pfluid");
        assert!(
            !list
                .entries()
                .iter()
                .any(|entry| entry.path == "/tmp/0.pfluid"),
            "the oldest entry made room"
        );
    }

    #[test]
    fn a_list_round_trips_and_never_writes_a_temporary_file() {
        let directory = tempfile::tempdir().expect("a temporary folder");
        let path = directory.path().join("recent.json");
        // Reading prunes entries whose file is gone, so the documents are real files: the
        // list promise is "documents you can open", not "paths somebody typed".
        let documents: Vec<String> = ["uno", "dos"]
            .iter()
            .map(|name| {
                let file = directory.path().join(format!("{name}.pfluid"));
                std::fs::write(&file, "{}").expect("writes");
                file.to_string_lossy().into_owned()
            })
            .collect();
        let mut list = RecentList::new();
        for document in &documents {
            assert!(list.remember(document));
        }
        list.write(&path).expect("writes");
        assert!(
            !temporary_path(&path).exists(),
            "the temporary file is renamed"
        );
        let read = RecentList::read(&path).expect("reads");
        assert_eq!(read, list);
        assert_eq!(read.entries()[0].name, "dos.pfluid");
    }

    #[test]
    fn a_missing_file_is_an_empty_list_and_a_broken_one_is_an_error() {
        let directory = tempfile::tempdir().expect("a temporary folder");
        let missing = directory.path().join("nada.json");
        assert!(RecentList::read(&missing).expect("reads").is_empty());
        let broken = directory.path().join("roto.json");
        std::fs::write(&broken, "{ no es json").expect("writes");
        assert!(matches!(
            RecentList::read(&broken),
            Err(RecentError::Parse(_))
        ));
    }

    #[test]
    fn a_future_list_is_refused_and_an_unknown_key_is_not_ignored() {
        assert!(matches!(
            RecentList::parse(r#"{"version":99,"path":[]}"#),
            Err(RecentError::UnsupportedVersion { found: 99, .. })
        ));
        assert!(matches!(
            RecentList::parse(r#"{"version":1,"path":[],"sorpresa":true}"#),
            Err(RecentError::Parse(_))
        ));
    }

    #[test]
    fn an_entry_whose_file_is_gone_is_dropped_on_read() {
        let directory = tempfile::tempdir().expect("a temporary folder");
        let document = directory.path().join("vivo.pfluid");
        std::fs::write(&document, "{}").expect("writes");
        let gone = directory.path().join("borrado.pfluid");
        let mut list = RecentList::new();
        assert!(list.remember(&gone.to_string_lossy()));
        assert!(list.remember(&document.to_string_lossy()));
        let path = directory.path().join("recent.json");
        list.write(&path).expect("writes");
        let read = RecentList::read(&path).expect("reads");
        assert_eq!(read.len(), 1, "only the file that exists is offered");
        assert!(read.entries()[0].path.ends_with("vivo.pfluid"));
    }

    #[test]
    fn a_path_too_long_or_with_a_nul_is_refused_instead_of_stored() {
        let mut list = RecentList::new();
        assert!(!list.remember(""));
        assert!(!list.remember(&"a".repeat(MAX_PATH + 1)));
        assert!(!list.remember("/tmp/con\0nul.pfluid"));
        assert!(list.is_empty());
    }
}
