# Packaging

Targets, in order of usefulness for a teaching tool:

1. **AppImage** — one file, no dependencies, works on any distribution and on the
   lab machines where this kind of software usually lives.
2. **Flatpak** — sandboxed and discoverable, once the file format is stable.
3. **Windows installer** — the platform where the reference tool runs, and the
   most likely place a teacher will try this.

Nothing is bundled yet: `cargo build --release -p punchi-fluid-window` produces the
window binary, and a release needs an explicit request before any artifact is published.
The desktop entry below is the first piece every one of those three targets installs.

The window is the Rust/egui binary in `apps/punchi-fluid`; it links the graphics stack
directly and needs no Qt, CMake or QML module. The retired Qt app and its Multimedia
dependency are not part of a bundle.

The release workflow still targets the retired Qt app and must be updated before a tag;
until then the archive command in `.github/workflows/release.yml` is a known debt, not a
working release path.

## The desktop entry

`org.punchisoft.PunchiFluidLab.desktop` is the application's entry, and its **file name is the
window's desktop name**: `branding::APP_ID` in `apps/punchi-fluid/src/branding.rs`, announced
through `ViewportBuilder::with_app_id`. A Wayland compositor matches a window to its entry by
that name and takes the icon from the entry's `Icon=`, so a rename that touches one and not
the other leaves the window without an icon and the entry orphaned. The test
`the_desktop_entry_is_named_after_the_desktop_name_the_window_announces` compiles the path in,
so the pair cannot drift apart silently.

`Icon=punchi-fluid-lab` is the name of the raster set in `assets/branding`, installed into
`hicolor/<size>/apps/`; `Exec=punchi-fluid` is the binary an AppImage, a Flatpak or a package
puts on `PATH`. The strings inside the entry are user-facing, so they are in Spanish like the
rest of the interface until F8 moves them to translation catalogs.

### A local install, to see the icon without a bundle

```bash
install -Dm644 packaging/org.punchisoft.PunchiFluidLab.desktop \
  ~/.local/share/applications/org.punchisoft.PunchiFluidLab.desktop
for size in 16 24 32 48 64 128 256 512; do
  install -Dm644 "assets/branding/punchi-fluid-lab-$size.png" \
    "$HOME/.local/share/icons/hicolor/${size}x${size}/apps/punchi-fluid-lab.png"
done
gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor
```

`Exec=punchi-fluid` assumes the binary is on `PATH`; a developer pointing the entry at a build
tree writes the absolute path there instead. KDE caches desktop entries, so a session that
does not pick it up needs `kbuildsycoca6` (KDE 5: `kbuildsycoca5`) or a new login.

**Why the window cannot do it alone.** On Wayland the client cannot set its window icon —
`winit`'s Wayland `set_window_icon` is an empty function — so `ViewportBuilder::with_icon` is a
no-op there and the icon has to come from the entry. On X11 and Windows both paths work.
