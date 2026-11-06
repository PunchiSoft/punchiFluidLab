// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The desktop's own icons.
//!
//! A native application does not ship its own toolbar pictograms: it asks the icon
//! theme for `document-new` and draws whatever the user's desktop answers. This crate
//! does exactly that — a freedesktop icon lookup plus a rasteriser — so the toolbar
//! looks like the rest of the desktop and follows a change of theme without a rebuild.
//!
//! Nothing of the theme is copied into the repository or into the binary: the files are
//! read from disk at run time, and a missing theme is not an error but a reason for the
//! caller to fall back to its own drawing.

use std::path::{Path, PathBuf};

/// A rendered icon: straight RGBA, row-major, `size × size`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Icon {
    /// Side of the image in pixels.
    pub size: u32,
    /// Four bytes per pixel, premultiplied by nothing: plain `RGBA`.
    pub pixels: Vec<u8>,
}

/// The icon theme of the desktop, resolved once.
#[derive(Debug, Clone, Default)]
pub struct IconTheme {
    roots: Vec<PathBuf>,
    /// Themes to try, most specific first, ending in `hicolor`.
    chain: Vec<String>,
}

impl IconTheme {
    /// Resolves the theme named by `kdeglobals` (or `$XDG_CURRENT_DESKTOP` defaults).
    #[must_use]
    pub fn detect() -> Self {
        Self::detect_for(false)
    }

    /// Resolves the theme for a window that is light or dark.
    ///
    /// A desktop that names its theme in `kdeglobals` gets exactly that theme: the user
    /// chose it. A desktop that names nothing — GNOME, a bare session, a container — has
    /// no idea which variant this window is drawing, so the scheme decides between
    /// `breeze` and `breeze-dark`. Without that, the toolbar asks for dark strokes and
    /// paints them on a dark bar, which is the one way an icon can be worse than none.
    #[must_use]
    pub fn detect_for(dark: bool) -> Self {
        let name = kde_icon_theme()
            .unwrap_or_else(|| String::from(if dark { "breeze-dark" } else { "breeze" }));
        Self::named(&name)
    }

    /// Resolves a theme by name, following its `index.theme` inheritance.
    #[must_use]
    pub fn named(name: &str) -> Self {
        let roots = icon_roots();
        let mut chain = Vec::new();
        let mut pending = vec![name.to_owned()];
        while let Some(next) = pending.pop() {
            if chain.iter().any(|seen| seen == &next) {
                continue;
            }
            chain.push(next.clone());
            // A theme declares what it inherits; a missing index.theme is normal for a
            // theme installed by hand, and then only the theme itself is searched.
            for parent in parents(&roots, &next) {
                if !chain.contains(&parent) {
                    pending.push(parent);
                }
            }
        }
        if !chain.iter().any(|seen| seen == "hicolor") {
            chain.push(String::from("hicolor"));
        }
        Self { roots, chain }
    }

    /// Themes that will be searched, in order.
    #[must_use]
    pub fn chain(&self) -> &[String] {
        &self.chain
    }

    /// Finds the file of an icon at about `size` pixels.
    ///
    /// A theme lays its icons out as `<theme>/<context>/<size>/<name>` (Breeze) or
    /// `<theme>/<size>/<context>/<name>` (hicolor), so the search walks down from the
    /// theme root and scores every directory that holds the file by how close its size
    /// is to the one asked for. `scalable` is the last resort.
    #[must_use]
    pub fn find(&self, name: &str, size: u16) -> Option<PathBuf> {
        for theme in &self.chain {
            let mut best: Option<(u32, PathBuf)> = None;
            for root in &self.roots {
                let base = root.join(theme);
                if base.is_dir() {
                    walk(&base, name, u32::from(size), 0, &mut best);
                }
            }
            if best.is_some() {
                return best.map(|(_, path)| path);
            }
        }
        None
    }

    /// Reads and rasterises an icon at `px` pixels square.
    ///
    /// Returns `None` when the theme has no such icon or the file is not readable: the
    /// caller draws its own, which is why this is not an error.
    #[must_use]
    pub fn render(&self, name: &str, px: u16) -> Option<Icon> {
        let path = self.find(name, px)?;
        let text = std::fs::read_to_string(&path).ok()?;
        rasterise(&text, px)
    }
}

/// Parses and rasterises SVG text at `px` pixels square.
#[must_use]
pub fn rasterise(svg: &str, px: u16) -> Option<Icon> {
    let options = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(svg, &options).ok()?;
    let side = u32::from(px).max(1);
    let mut pixmap = tiny_skia::Pixmap::new(side, side)?;
    let source = tree.size();
    // An icon is a few dozen pixels: the cast cannot lose anything, and the lint does not
    // know that.
    #[allow(clippy::cast_precision_loss)]
    let wanted = side as f32;
    let scale = (wanted / source.width()).min(wanted / source.height());
    let transform = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    Some(Icon {
        size: side,
        pixels: pixmap.take(),
    })
}

/// Walks a theme directory looking for one icon, keeping the closest size.
fn walk(dir: &Path, name: &str, wanted: u32, depth: u8, best: &mut Option<(u32, PathBuf)>) {
    if depth > 3 {
        return;
    }
    for file in candidates(dir, name) {
        if file.is_file() {
            let score = score(dir, wanted);
            if best.as_ref().is_none_or(|(seen, _)| score < *seen) {
                *best = Some((score, file));
            }
        }
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, name, wanted, depth + 1, best);
        }
    }
}

/// How far a directory is from the size asked for, from its own name and its parents.
///
/// The closest numeric segment decides; a `scalable` directory counts as far, so a
/// raster size is always preferred to it; a symbolic variant is never chosen, because a
/// toolbar wants the full-colour icon.
fn score(dir: &Path, wanted: u32) -> u32 {
    let mut best: Option<u32> = None;
    for part in dir.components() {
        let Some(text) = part.as_os_str().to_str() else {
            continue;
        };
        let lowered = text.to_ascii_lowercase();
        if lowered.contains("symbolic") {
            return u32::MAX / 2;
        }
        let distance = if lowered.starts_with("scalable") {
            512
        } else {
            match declared_size(text) {
                1024 => continue,
                declared => declared.abs_diff(wanted),
            }
        };
        best = Some(best.map_or(distance, |seen: u32| seen.min(distance)));
    }
    best.unwrap_or(1024)
}

/// Declared size of an icon directory: `22`, `22x22`, `48@2x`, `scalable`.
fn declared_size(folder: &str) -> u32 {
    let mut numbers = folder
        .split(|character: char| !character.is_ascii_digit())
        .filter_map(|part| part.parse::<u32>().ok());
    numbers.next().unwrap_or_else(|| {
        if folder.starts_with("scalable") {
            512
        } else {
            1024
        }
    })
}

/// The files an icon name may live in, preferred first.
fn candidates(dir: &Path, name: &str) -> Vec<PathBuf> {
    vec![
        dir.join(format!("{name}.svg")),
        dir.join(format!("{name}.png")),
        dir.join(format!("{name}.svgz")),
    ]
}

/// Every directory an icon theme may be installed in, in freedesktop order.
fn icon_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(home) = std::env::var_os("XDG_DATA_HOME") {
        roots.push(PathBuf::from(home).join("icons"));
    } else if let Some(home) = std::env::var_os("HOME") {
        roots.push(PathBuf::from(home).join(".local/share/icons"));
    }
    let dirs = std::env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| String::from("/usr/local/share:/usr/share"));
    for dir in dirs.split(':').filter(|dir| !dir.is_empty()) {
        roots.push(Path::new(dir).join("icons"));
    }
    roots.dedup();
    roots
}

/// Themes a theme inherits from, read from its `index.theme`.
fn parents(roots: &[PathBuf], theme: &str) -> Vec<String> {
    for root in roots {
        let index = root.join(theme).join("index.theme");
        let Ok(text) = std::fs::read_to_string(index) else {
            continue;
        };
        if let Some(value) = index_value(&text, "Inherits") {
            return value
                .split(',')
                .map(|name| name.trim().to_owned())
                .filter(|name| !name.is_empty())
                .collect();
        }
        return Vec::new();
    }
    Vec::new()
}

/// Value of a key in the `[Icon Theme]` section of an `index.theme`.
fn index_value(text: &str, key: &str) -> Option<String> {
    let mut in_section = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('[') {
            in_section = line.eq_ignore_ascii_case("[Icon Theme]");
            continue;
        }
        if !in_section {
            continue;
        }
        if let Some((name, value)) = line.split_once('=')
            && name.trim().eq_ignore_ascii_case(key)
        {
            return Some(value.trim().to_owned());
        }
    }
    None
}

/// Icon theme named by `kdeglobals`.
fn kde_icon_theme() -> Option<String> {
    let home = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    let text = std::fs::read_to_string(home.join("kdeglobals")).ok()?;
    index_value(&text, "Theme")
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn a_directory_size_is_read_from_its_name() {
        assert_eq!(declared_size("22"), 22);
        assert_eq!(declared_size("24x24"), 24);
        assert_eq!(declared_size("48@2x"), 48);
        assert!(declared_size("scalable") > 400);
    }

    #[test]
    fn the_inheritance_is_read_from_the_index_theme() {
        let text = "[Icon Theme]\nName=Breeze\nInherits=hicolor\n";
        assert_eq!(index_value(text, "Inherits"), Some(String::from("hicolor")));
        assert_eq!(
            index_value("[Desktop Entry]\nInherits=x\n", "Inherits"),
            None
        );
    }

    #[test]
    fn a_plain_svg_rasterises_to_the_size_asked_for() {
        let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"22\" height=\"22\">\
            <rect x=\"4\" y=\"4\" width=\"14\" height=\"14\" fill=\"#ff0000\"/>\
        </svg>";
        let icon = rasterise(svg, 32).expect("a plain rectangle rasterises");
        assert_eq!(icon.size, 32);
        assert_eq!(icon.pixels.len(), 32 * 32 * 4);
        // The middle of the square is the red we asked for.
        let middle = ((16 * 32) + 16) * 4;
        assert!(
            icon.pixels[middle] > 200,
            "red channel: {:?}",
            &icon.pixels[middle..middle + 4]
        );
    }

    #[test]
    fn a_file_that_is_not_an_svg_is_not_an_error() {
        assert!(rasterise("not an svg at all", 22).is_none());
    }

    #[test]
    fn an_unknown_icon_is_reported_as_missing_not_as_a_failure() {
        let theme = IconTheme::named("punchi-fluid-does-not-exist");
        assert!(theme.render("document-new", 22).is_none());
        assert!(
            theme.chain().iter().any(|name| name == "hicolor"),
            "the search always ends in hicolor: {:?}",
            theme.chain()
        );
    }
}
