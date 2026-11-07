# Packaging

Targets, in order of usefulness for a teaching tool:

1. **AppImage** — one file, no dependencies, works on any distribution and on the
   lab machines where this kind of software usually lives.
2. **Flatpak** — sandboxed and discoverable, once the file format is stable.
3. **Windows installer** — the platform where the reference tool runs, and the
   most likely place a teacher will try this.

Nothing is packaged yet: `cargo build --release` produces a working binary, and a
release needs an explicit request before any artifact is published.

The window is Qt Quick and imports **Qt Multimedia** from QML for the event sounds, so
any bundle has to carry that module and its audio backend: it is a run-time dependency
only, and building needs just the QML module present in the Qt installation.
