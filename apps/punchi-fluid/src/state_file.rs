// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The defensive file the window's two state stores share: a bounded read and an atomic write.
//!
//! `window_state` and `settings` each own a `key=value` format and its parser, and differ in
//! what they remember; what they must not differ in is how a file comes off the disk or goes
//! back on it. A read is bounded to the bytes the caller declares, so a corrupt or hostile
//! file cannot make the window allocate without a limit; a write goes through a temporary file
//! in the same folder and a rename, so a half-written file never replaces a good one, and a
//! failure leaves the previous file untouched.

use std::io::{Read, Write as _};
use std::path::{Path, PathBuf};

/// Reads at most `max_bytes` bytes of `path` as UTF-8; `None` when the file is absent,
/// unreadable, not text, or bigger than the bound (one byte past it is read to tell).
#[must_use]
pub fn read_bounded(path: &Path, max_bytes: usize) -> Option<String> {
    let file = std::fs::File::open(path).ok()?;
    let mut text = String::new();
    file.take((max_bytes as u64) + 1)
        .read_to_string(&mut text)
        .ok()?;
    (text.len() <= max_bytes).then_some(text)
}

/// Writes `text` through a temporary created with `create_new`, flushed, and renamed over the
/// target: a planted symlink at the predictable temporary name is removed first, and a
/// failure never leaves the previous file truncated.
pub fn write_atomically(path: &Path, text: &str) -> bool {
    let temporary = temporary_path(path);
    let _ = std::fs::remove_file(&temporary);
    let Ok(mut file) = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
    else {
        return false;
    };
    let written = file
        .write_all(text.as_bytes())
        .and_then(|()| file.sync_all())
        .is_ok();
    drop(file);
    if !written {
        let _ = std::fs::remove_file(&temporary);
        return false;
    }
    if std::fs::rename(&temporary, path).is_err() {
        let _ = std::fs::remove_file(&temporary);
        return false;
    }
    true
}

/// The temporary file beside the target, in the same folder so the rename stays on one
/// filesystem.
#[must_use]
pub fn temporary_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    path.with_file_name(name)
}
