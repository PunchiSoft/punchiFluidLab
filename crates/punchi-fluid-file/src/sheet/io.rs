// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading and writing a sheet, the one place that touches the filesystem.
//!
//! The path is untrusted input and so is what a filesystem says about it: a FIFO declares
//! zero bytes, a symbolic link redirects a write, and a permission bit the document had is
//! a promise the next save must keep. The controls live here, next to the call that needs
//! them, and every one of them has a test in `sheet_tests.rs`.

use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use super::{MAX_BYTES, Sheet, SheetError};

impl Sheet {
    /// Reads a sheet from a file.
    ///
    /// The path is untrusted input: it can name a pipe, a device or a file that grows
    /// while it is read, and the length `metadata` reports is a claim, not a guarantee.
    /// The file is therefore checked to be regular through the handle that will be read,
    /// and the read itself is bounded to one byte past [`MAX_BYTES`] so a file that lies
    /// about its size cannot exhaust memory.
    ///
    /// # Errors
    ///
    /// Returns [`SheetError::NotRegular`] for anything that is not a regular file,
    /// [`SheetError::TooLarge`] when the file exceeds [`MAX_BYTES`], plus the errors of
    /// [`Self::parse`].
    pub fn read(path: &Path) -> Result<Self, SheetError> {
        // Checked twice on purpose: before opening, so a pipe is not even opened (opening
        // a FIFO for reading waits for a writer), and again through the handle, so the
        // decision is made about the file that is actually read.
        if !fs::metadata(path)?.is_file() {
            return Err(SheetError::NotRegular);
        }
        let file = fs::File::open(path)?;
        if !file.metadata()?.is_file() {
            return Err(SheetError::NotRegular);
        }
        let mut text = String::new();
        let mut bounded = file.take(MAX_BYTES + 1);
        bounded.read_to_string(&mut text)?;
        drop(bounded);
        if u64::try_from(text.len()).unwrap_or(u64::MAX) > MAX_BYTES {
            return Err(SheetError::TooLarge);
        }
        Self::parse(&text)
    }

    /// Writes the sheet atomically: a temporary file in the same folder, then a rename.
    ///
    /// The temporary is created with `create_new`, which refuses a name that already
    /// exists — including one an attacker planted as a symbolic link — after unlinking a
    /// stale temporary left by a crashed save. It inherits the permissions of the file it
    /// replaces, is flushed before the rename, and is removed when either step fails, so
    /// a failed save leaves neither a truncated document nor a copy of it behind.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::to_json`] and [`SheetError::Io`].
    pub fn write(&self, path: &Path) -> Result<(), SheetError> {
        let text = self.to_json()?;
        let temporary = path.with_extension("pfluid.tmp");
        clear_temporary(&temporary)?;
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(temporary_mode(path));
        }
        let mut file = options.open(&temporary)?;
        let written = file
            .write_all(text.as_bytes())
            .and_then(|()| file.sync_all());
        drop(file);
        if let Err(error) = written {
            let _ = fs::remove_file(&temporary);
            return Err(SheetError::Io(error));
        }
        if let Err(error) = fs::rename(&temporary, path) {
            let _ = fs::remove_file(&temporary);
            return Err(SheetError::Io(error));
        }
        sync_parent(path);
        Ok(())
    }
}

/// Removes whatever sits at the temporary name, without following it.
///
/// A save that crashed leaves a stale temporary; an attacker can plant a symbolic link
/// there instead. Unlinking removes the entry itself, never what it points at, so the
/// next `create_new` is safe either way. A directory is an error: a save must not delete
/// a folder it did not create.
fn clear_temporary(temporary: &Path) -> Result<(), std::io::Error> {
    match fs::symlink_metadata(temporary) {
        Ok(metadata) if metadata.is_dir() => Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "a directory is in the way of the sheet's temporary file",
        )),
        Ok(_) => fs::remove_file(temporary),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

/// The permission bits a new temporary inherits from the document it replaces.
///
/// A `0600` project must not become `0644` because it was saved once, and a new document
/// starts private rather than world-writable.
#[cfg(unix)]
fn temporary_mode(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path).map_or(0o600, |metadata| metadata.permissions().mode() & 0o777)
}

/// Flushes the directory entry so the rename survives a power loss.
///
/// A filesystem that cannot open its folder is not a reason to fail a save that was
/// already published: the document is on disk and only the durability of its name is
/// best effort.
fn sync_parent(path: &Path) {
    let parent = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    if let Ok(directory) = fs::File::open(parent) {
        let _ = directory.sync_all();
    }
}
