# Packaging

Targets, in order of usefulness for a teaching tool:

1. **AppImage** — one file, no dependencies, works on any distribution and on the
   lab machines where this kind of software usually lives.
2. **Flatpak** — sandboxed and discoverable, once the file format is stable.
3. **Windows installer** — the platform where the reference tool runs, and the
   most likely place a teacher will try this.

Nothing is packaged yet: `cargo build --release -p punchi-fluid-window` produces the
window binary, and a release needs an explicit request before any artifact is published.

The window is the Rust/egui binary in `apps/punchi-fluid`; it links the graphics stack
directly and needs no Qt, CMake or QML module. The retired Qt app and its Multimedia
dependency are not part of a bundle.

The release workflow still targets the retired Qt app and must be updated before a tag;
until then the archive command in `.github/workflows/release.yml` is a known debt, not a
working release path.
