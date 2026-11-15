<!-- SPDX-FileCopyrightText: 2026 PunchiSoft -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Symbol geometry fixtures

One text file per built-in component, holding the drawing calls the renderer produces for
that definition in a fixed 40 × 40 mm box. They are the machine half of the review: the
drawing of a symbol cannot move, change width or lose a mark without this test failing.

`punchi-fluid-draw` has the test (`tests/symbol_geometry.rs`), and `cargo test --workspace`
runs it. A missing fixture is a failure, not a warning, so a new symbol arrives with its file.

## Changing a symbol

1. Change the definition, then run
   `UPDATE_SYMBOL_FIXTURES=1 cargo test -p punchi-fluid-draw --test symbol_geometry`.
2. **Read the diff** of the affected fixture. It is the drawing, in numbers: a moved point, a
   different width, a missing call. If the diff is not what the change was meant to do, the
   change is wrong.
3. Commit the definition and the fixture together, with the review record
   ([TEMPLATE.md](../review/TEMPLATE.md)) and the captures.

A fixture approved without reading the diff is a fixture that approves anything.
