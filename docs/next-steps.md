# Continuation: the work order

Read this before opening an editor. It says exactly where the project stands, what is
done next, which files it touches, which tests prove it and which questions belong to
the project owner. The reasoning behind every rule is in the linked documents; this
file is the execution order.

> **Read the toolkit note first.** [ADR 0005](adr/0005-qt-quick-over-a-c-abi.md) replaced
> the egui front end with the Qt window over `punchi-fluid-ffi` and retired
> `punchi-fluid-theme`, `punchi-fluid-ui`, `punchi-fluid-desktop` and
> `punchi-fluid-icons`. The milestone sections below are kept as the record of the
> reasoning; the source of truth for what comes next is
> [reference-parity.md](reference-parity.md) and the order at the top of this file.

## Where the project stands

`0.2` — a working vertical slice, and a specification for everything after it.

| Area | State | Evidence |
|---|---|---|
| Domain, file format, CLI | **implemented** | `cargo test --workspace`, CLI smoke test in `check.sh` |
| Component catalog (TOML files, override, `disabled`) | **implemented** | 7 catalog tests; verified in the real session with a user override and a broken file |
| Gas model, valve positions, cylinder with spring and friction | **implemented** | ISO 6358 against a hand calculation, mass balance 1e-9, stroke time bracketed, 10⁵ steps without NaN |
| Canvas: catalog-driven symbols, click on the valve, live readouts | **implemented** | headless click tests, screenshots |
| Shell: ribbon, sidebar, status bar, theme | **implemented** — [ui-ux.md](ui-ux.md) | 17 theme tests, 4 contrast tests, colour-literal scan, 10 UI tests, light/dark screenshots |
| Export PNG/PDF | **implemented** — [plan.md](plan.md) M2 | 8 PDF acceptance tests + 5 raster tests: pixel sizes from the IHDR, empty margin band, determinism, refusal before allocating |
| ISO 1219 conformance | **structural half implemented, visual half reviewed** — [symbology.md](symbology.md) | 8 lint rules with a test each, 4 review files with light/dark screenshots, review-folder test; 300 dpi pending M2 |
| Editor: document, undo, pipes, T, re-route, per-instance properties | **implemented** — [reference-parity.md](reference-parity.md) F1 | doc/ABI/`--selftest`, per row in the ledger |
| The sheet is the network the solver runs | **implemented** — [reference-parity.md](reference-parity.md) F2 | network tests, the 5/2 + double-acting ABI circuit, pipe colour |
| Save and open a sheet | **implemented** — [adr/0007](adr/0007-the-sheet-format-in-the-abi.md) | six format tests, ABI round trip, `--selftest` |

The repository is initialised and the baseline is committed; the working tree is on
`main` with the Qt front end merged in.

## Order after 2026-11-06

The milestone list below was written for the vertical slice; the capabilities are now
tracked in [reference-parity.md](reference-parity.md), which is the source of truth. The
order that follows from it:

1. **F3 — instruments and runs**: a manometer, a flow meter, a force meter and a strip
   chart as catalog entries with readings, plus *run until state change* (manual §3.2:
   piston at an end, valve actuated, relay, switch).
2. **F1 — the editing functions of §4.2** that are still missing: rotate/flip,
   copy/paste/duplicate, multi-selection with align and distribute, pipe classes, blind
   plugs and silencers.
3. **F4 — documents**: autosave, recent documents, a warning when closing an unsaved
   circuit, a project as a folder, and the designations that unlock three of the four
   remaining check faults.
4. **F5 — output**: parts list and title-block legend, then SVG.
5. **F6 — libraries** beyond pneumatics, each with a `behaviour` variant and an ADR.

## Before the first line of code

```bash
./scripts-dev/check.sh                 # must be green before you touch anything
cmake --build apps/punchi-fluid-qt/build   # the window (see apps/punchi-fluid-qt/CMakeLists.txt)
apps/punchi-fluid-qt/build/punchi-fluid --toggle   # starts actuated
```

The first commit is `5d47854`, `chore: import the vertical slice as the baseline`; the
toolkit decision is [ADR 0005](adr/0005-qt-quick-over-a-c-abi.md), accepted on 2026-11-06.
Builds run with two cargo jobs and each test binary under a memory ceiling — see
[.agents/skills/machine-resource-budget](../.agents/skills/machine-resource-budget/SKILL.md).

Environment verified on 2026-11-06: Fedora 44, Plasma 6.7.5, Wayland, KDE, Rust 1.98.1,
`~/.config/kdeglobals` present with `[Colors:Window] BackgroundNormal=36,36,36` and
`[Colors:Selection] BackgroundNormal=49,91,239`, portal
`org.freedesktop.appearance color-scheme` answering `1` (dark), Breeze symbolic icons
under `/usr/share/icons/breeze/actions/22/`.

To try a component change without recompiling:

```bash
mkdir -p ~/.local/share/punchi-fluid-lab/catalog
cp assets/catalog/valve.3_2.spring_return.toml ~/.local/share/punchi-fluid-lab/catalog/
# edit the copy: same `id` replaces the built-in one
```

The four rules that do not bend: the gate stays green without silencing warnings, a claim
names its evidence level, a symbol is not conformant until it is reviewed, and a capability
is not started until its observation of the reference is written in
[reference-parity.md](reference-parity.md) — see the
[skill](../.agents/skills/reference-parity/SKILL.md). The reference is the base: replicate
it first, then improve it, and finish each capability with that decision written down.

## Order of work

**M1 → M2 → M1.5**, then M3 onwards as described in [plan.md](plan.md).

Why this order: M1 is independent and makes everything after it visible. M2 creates the
`Surface` layer that the conformance renderer needs, so M2 must already implement the
renderer changes (port numerals, line classes, blocking marks) and leave only the lint,
the redraw and the review for M1.5. If the owner prefers, M1.5's lint can run right
after M2's first step, before the export backends; nothing else changes.

---

> **Toolkit note (2026-11-06).** [ADR 0005](adr/0005-qt-quick-over-a-c-abi.md) is accepted:
> the window is Qt Quick over the C ABI in `punchi-fluid-ffi`, and `punchi-fluid-theme`,
> `punchi-fluid-ui`, `punchi-fluid-desktop` and `punchi-fluid-icons` have been retired.
> The milestone *goals* below still describe the product correctly; the file paths and the
> `egui` types in them no longer exist. Where this document says `crates/punchi-fluid-ui`,
> read `apps/punchi-fluid-qt`. The theme work of M1 is done, in
> `apps/punchi-fluid-qt/src/theme.cpp` — measured by `--check-theme`, not by eye.

## M1 — Application shell

Goal: the window looks and behaves like the design, using the desktop's own colours.

### 1.1 the theme adapter — **done, in the Qt front end**

`QApplication::palette()` already carries `kdeglobals` and the colour-scheme portal, so
the adapter reads it and derives `ThemeTokens`; `ensureContrast` shifts any colour that
cannot reach its criterion. The historical plan this section replaced is below, kept only
so the reasoning is not lost.

```
crates/punchi-fluid-theme/          # retired by ADR 0005
  src/kdeglobals.rs     # INI parsing, [Colors:*], [General] font, LookAndFeelPackage
  src/portal.rs         # org.freedesktop.appearance color-scheme, + SettingChanged
  tests/contrast.rs     # text >= 4.5:1, lines >= 3:1, both schemes
```

Public shape (keep it this small):

```rust
pub enum Scheme { Light, Dark, NoPreference }
pub enum Token { Surface, SurfaceAlt, SurfaceRaised, Text, TextMuted, Accent, Danger, Warning, Ok, GridMinor, GridMajor, WorkLine }
pub struct Palette { /* colours per token, built from kdeglobals or a fallback */ }
pub struct Theme { pub palette: Palette, pub font_family: String, pub font_size: f32, pub animation_factor: f32 }
impl Theme { pub fn detect() -> Self; pub fn visuals(&self) -> egui::Visuals; pub fn colour(&self, token: Token) -> egui::Color32; }
```

Dependencies: `egui`, `thiserror`, and a DBus client. Prefer `zbus`, not
`dbus`/`libdbus` (no C library, no `openssl-sys`, keeps `deny.toml` happy). A missing
portal is a normal case: fall back to `Visuals::dark()`/`light()`.

Traps: do not read `kdeglobals` per frame; do not assume the file exists; do not parse
`[General] font` naively (it is a Qt font string, `family,pointsize,…` — take the first
two fields); `AnimationDurationFactor = 0` must disable easing, not the simulation.

### 1.2 Ribbon

`crates/punchi-fluid-ui/src/ribbon.rs`: `RibbonState { tab: Tab }`, four tabs, the
transport always visible on the right, overflow `⋯` for actions that do not fit,
collapse to a menu button under 1100 px. Each action is an enum value dispatched to the
application, not a closure per button, so the same action can be triggered by keyboard,
menu and toolbar.

### 1.3 Sidebar

`crates/punchi-fluid-ui/src/sidebar.rs`: groups by `Category`, search box, drag payload
equal to the definition id, thumbnail drawn through the same symbol renderer, footer
with the catalog folder and the skipped-file count. Reuses `Catalog::iter`.

### 1.4 Frame and status bar

`crates/punchi-fluid-ui/src/shell.rs`: region layout with sizes from the theme tokens,
focus order, `Tab` cycling between regions. Status bar prints the values the current
code already computes in `FluidLabApp::status_bar`.

### 1.5 Desktop wiring

`crates/punchi-fluid-desktop/src/app.rs`: build the `Theme` once, apply it to the
`egui::Context`, subscribe to theme changes, and add `accesskit` to the `eframe`
features (exists in 0.37 as `accesskit`, currently not enabled in our
`default-features = false, features = [...]` list). Add `punchi-fluid-theme` to the
workspace members and to `Cargo.toml`.

### Acceptance for M1

1. Light and dark, switched live from Plasma's settings, no restart.
2. A test scans the sources and fails on `Color32::` outside `punchi-fluid-theme`.
3. Keyboard reachability: every action has a shortcut, listed in a help panel.
4. `tests/contrast.rs` green for both palettes.
5. Screenshots in both schemes, attached to the pull request.
6. `check.sh` green; no new warnings.

---

## M2 — One drawing, two exports

Goal: the same drawing that is on screen becomes a PNG or a PDF on a standard sheet.
This is the feature the owner asked for; it is only cheap if the drawing is not
duplicated.

### 2.1 `punchi-fluid-draw` (new crate)

Move the drawing out of `crates/punchi-fluid-ui/src/{canvas,symbol}.rs` into
`punchi-fluid-draw`, behind a trait. Units are **millimetres** at the API boundary; the
screen multiplies by `pixels_per_point` and the paper by 25.4/72.

```rust
pub struct Pt { pub x: f32, pub y: f32 }         // mm
pub struct Rect { pub min: Pt, pub max: Pt }     // mm
pub enum LineClass { Working, Pilot, Drain, Electrical }
pub struct Stroke { pub width_mm: f32, pub colour: Colour, pub class: LineClass }
pub enum Anchor { Start, Middle, End }           // horizontal text anchor

pub trait Surface {
    fn line(&mut self, points: &[Pt], stroke: Stroke);
    fn polygon(&mut self, points: &[Pt], fill: Option<Colour>, stroke: Option<Stroke>);
    fn rect(&mut self, rect: Rect, corner_mm: f32, fill: Option<Colour>, stroke: Option<Stroke>);
    fn circle(&mut self, centre: Pt, radius_mm: f32, fill: Option<Colour>, stroke: Option<Stroke>);
    fn arc(&mut self, centre: Pt, radius_mm: f32, start_deg: f32, end_deg: f32, stroke: Stroke);
    fn text(&mut self, at: Pt, text: &str, size_mm: f32, colour: Colour, anchor: Anchor);
    fn recording(&mut self) -> Option<&mut Vec<Call>> { None }   // for the equivalence test
}

pub fn draw_schematic(surface: &mut dyn Surface, area: Rect, system: &PneumaticSystem, catalog: &Catalog);
```

Conformance work that belongs here, not later: port numerals from the `port` table,
`LineClass` on every polyline, blocking mark at closed ports, and a filled dot at
three-way junctions.

### 2.2 the window implements `Surface`

Done: `apps/punchi-fluid-qt/src/replay.cpp` replays the command buffer with `QPainter`,
and `canvasitem.cpp` keeps only hit-testing and the click outcome. The equivalent on the
egui side was `punchi-fluid-ui::surface::EguiSurface`, retired by ADR 0005.

### 2.3 Page geometry (`punchi-fluid-export`)

```rust
pub enum PaperSize { A5, A4, A3, A2, Letter, Legal }
pub enum Orientation { Portrait, Landscape }
pub struct Margins { pub top_mm: f32, pub right_mm: f32, pub bottom_mm: f32, pub left_mm: f32 }
pub struct TitleBlock { pub title: String, pub project: String, pub revision: String, pub date: String, pub note: Option<String> }
pub struct Page { pub paper: PaperSize, pub orientation: Orientation, pub margins: Margins, pub title_block: Option<TitleBlock> }
impl Page { pub fn size_mm(&self) -> (f32, f32); pub fn content_rect(&self) -> Rect; pub fn title_rect(&self) -> Rect; }
```

A4 is 210 × 297 mm, A3 297 × 420, A5 148 × 210, Letter 215.9 × 279.4, Legal 215.9 × 355.6.
Landscape swaps them. The drawing goes inside `content_rect()`; the title block sits in
the bottom-right of the content area and reserves its height.

### 2.4 PNG backend

`tiny-skia` 0.12 (`Pixmap`, `PathBuilder`, `Stroke`, `Fill`) + `ab_glyph` 0.2 with fonts
from `fontdb` 0.24. 300 dpi default, 72–600 allowed. Refuse before allocating when the
bitmap would exceed 80 Mpx. Deterministic: no timestamps except the title-block date,
fixed PNG encoder settings.

### 2.5 PDF backend

`printpdf` 0.12, vector, one page, built-in Helvetica, page in millimetres converted to
points. No path, user name or host name in metadata; title sanitised and bounded to 200
characters. While M1.5 is open, the title block carries `symbols not yet verified
against ISO 1219-1`.

### 2.6 Export panel

`crates/punchi-fluid-ui/src/export_panel.rs`, opened from the ribbon: paper,
orientation, margins, title block fields, DPI, destination path, and a live line with
the resulting pixel size and estimated file size. No modal, no file dialog dependency:
a text field with a default in the user's documents folder, plus a warning when the
target exists.

### Acceptance for M2

1. A4 at 300 dpi → 2480 × 3508 px; A3 landscape at 300 dpi → 4961 × 3508 px. Tested.
2. The margin band contains no ink; the drawing is inside the content rectangle. Pixel
   test in both regions.
3. The PDF has one page, its MediaBox matches the paper within 1 pt, and it contains no
   path, user name or host name. Tested.
4. Two exports of the same input are byte-identical. Tested.
5. Screen and export produce the same call list through a recording `Surface`. Tested.
6. A refused export (DPI 1200, or a bitmap above the budget) fails with a message and
   allocates nothing.
7. The existing click tests still pass, unchanged.

---

## M1.5 — Symbol conformance

Goal: the built-in symbols become conformant, and a lint keeps them that way. Contract:
[symbology.md](symbology.md).

### Lint (in `punchi-fluid-catalog`, `symbol lint`)

One test per rule, each failing on a purpose-built bad definition:

- number of `rect` boxes equals the number of valve positions;
- every position has a path or a blocking mark;
- every port has a numeral drawn and the numerals are unique;
- `spring_return = true` implies a declared spring part;
- `actuation != none` implies a declared actuator part;
- no primitive outside `0..=1`, no zero-length line, no zero radius;
- every polyline has a line class;
- minimum legibility at palette size.

### Redraw

`valve.3_2.spring_return`, `supply.workshop`, `cylinder.single_acting_20_100`,
`throttle.fixed`: add port numerals, structured paths and blocking marks, declared
spring and actuator, line classes, and review the cylinder's spring and rod proportions
against Part 1. The cylinder's rod and spring were redrawn and reviewed on 2026-11-06:
the piston now crosses the whole internal length the definition declares and the return
spring is compressed into the dead length it leaves.

### Review

`assets/symbols/review/<id>.md` plus `<id>-light.png` and `<id>-dark.png` at palette size
and at 300 dpi. The export drops the "unverified" note only when a test finds a review
file for every built-in definition.

### Acceptance for M1.5

1. Lint fails on each of the eight bad definitions and passes on all four built-ins.
2. Every built-in symbol has a review file; the note disappears because of that test.
3. Visual review of the four symbols against Part 1, recorded in the pull request.
4. A legend of line classes and port numerals appears in the title block.

---

## Questions for the project owner

Answer these before the milestone that needs them; each one changes what gets built.

| # | Question | Needed for | Default if unanswered |
|---|---|---|---|
| 1 | `git init` and first commit in this folder? | before M1 | keep working without git |
| 2 | Confirm the current edition of the fluid-power port-identification standard (only ISO 1219-1/2/3 could be verified from this machine) | M1.5 | use 1/2/4/3/5/12/14 and say so in the review |
| 3 | Ribbon at 1100 px, or collapse at 1280? | M1 | 1100 |
| 4 | Default export: A4 portrait, 300 dpi, title block with title/project/revision/date? | M2 | yes to all |
| 5 | Should the export include a symbol legend and the pressure/position readouts, or a clean diagram only? | M2 | legend yes, readouts no |
| 6 | Left sidebar fixed width or user-resizable with a saved preference? | M1 | resizable, saved in the session state |
| 7 | Icon language: Breeze symbolic only, or allow the user's icon theme? | M1 | Breeze symbolic, user theme as fallback |

## Session checklist

- [ ] `./scripts-dev/check.sh` green before touching anything
- [ ] Read [AGENTS.md](../AGENTS.md) and, for the milestone at hand, its spec
- [ ] One milestone, one branch, one pull request
- [ ] New file? SPDX header, English comments, tests next to it
- [ ] Behaviour changed? Test that fails without the change
- [ ] `CHANGELOG.md` updated under `Unreleased`
- [ ] Budgets checked: memory test, frame timing, and the screenshot review
- [ ] State the evidence level in the pull request, and what stays unverified
