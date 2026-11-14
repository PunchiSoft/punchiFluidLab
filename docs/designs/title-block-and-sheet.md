# Design: the sheet and its title block

Status: `current`. Written with the Elephant-Goldfish Model before the change was
accepted; the code lives in `feat/title-block-editor`.

## Overview

A printed pneumatic or electrical schematic has to identify itself the way an industrial
control document does in Chile: ISO 5457 / NCh 13.Of93 sheet with 20 mm on the filing edge
and 10 mm elsewhere, a 0.7 mm frame, four centring marks, optional 50 mm reference zones,
and an ISO 7200 / NCh 14.Of93 title block at the bottom right. This design records the
shape, the fields, the defaults and the files that carry them.

## Problem

The build had a 170 × 32 mm block with eight fields, painted over the drawing. It did not
match the guide's A3 template, it covered components, and the sheet opened on A4 portrait
where a schematic is normally read horizontally.

## Goals

- A block of **170 × 45 mm (industrial, A4/A5)** and **180 × 65 mm (industrial, A3+)**,
  bottom right, with the identification zone the guide bounds at 170 mm; the school and
  technical models are shorter.
- Twelve editable fields in the guide order: owner, project, equipment, title, number,
  revision, date, prepared, checked, approved, scale (`S/E`) and norms; `sheet` derived as
  `n/N` from the circuit position and the number of circuits.
- **Content models**: Escolar, Técnico and Industrial, plus **Sin cajetín**; a model chooses
  the default fields, and **Personalizar** hides or shows any of them without losing the
  typed value.
- Text heights per ISO 3098 / NCh 15: 5 mm for title/company, 3.5 mm for values and 2.5 mm
  for labels, each fitted inside its cell (the value drops one step when a scaled row cannot
  hold it).
- The block is **optional**: top right, bottom right, bottom-full or hidden (the content
  combo's Sin cajetín), chosen from the Cajetín tab or the docked editor, with the fields
  kept when it is hidden.
- **A4 landscape by default**, portrait one command away.
- **Reference zones** of 50 mm with letters and numerals from A3 upwards; absent on A4.
- The canvas and the export draw the same block; the drawing area reserves the block band
  so nothing is hidden under it.

## Non-goals

- A scale or material field: a schematic is not to scale. The guide suggests `S/E`; that is
  what the scale field defaults to and the user may change it.
- A logo image, a revision history block or the technical specifications box: the guide
  recommends them, but they are separate sheet elements and the next change.
- Enforcing the exact ISO 7200 character lengths: every field is bounded at 120 characters
  and elided on the sheet.

## Critical user journeys

1. Open the program and print the example: A4 landscape, block at the bottom right, the
   circuit clear of it.
2. **Cajetín** tab: switch Escolar / Técnico / Industrial / Sin cajetín and a position,
   then **Editar datos…** for the docked editor; every choice is one undo.
3. Open an older file: it reads with the defaults (circuit title, revision `A`, `S/E`,
   right position, industrial model) and never fails.
4. Print A3: the industrial block grows to 180 × 65 and the reference zones appear around
   the frame.

## Editor

The editor is a **docked panel on the right**, not a modal: opening it shrinks the sheet's
column and keeps the live sheet visible. It opens from **Cajetín ▸ Datos ▸ Editar datos…**
(the command says **Añadir cajetín** while the preset is none) and closes with its own
button. There is no Save: every field edit, model change, visibility change and reset is one
undo entry and marks the document modified.

- **Content combo** at the top: Técnico, Escolar, Industrial and Sin cajetín, keyboard
  accessible and labelled. Choosing a model restores its field set and, if the block was
  hidden, brings it back at the standard position in the same undo. The Cajetín ribbon tab
  shows the current model as one **Contenido: X** item that opens the editor, not four
  tiles.
- **Position combo**: top right, bottom right; a portrait sheet adds the full bottom band.
  Its dropdown delegate draws the model's real rows at the chosen corner; Sin cajetín
  disables it.
- **Fields**: only the model's fields, in print order, label and input on one row. The
  derived sheet `n/N` is shown read-only. Labels elide rather than clip and the input has
  room for the 11 px text.
- **Personalizar**: reveals a Sí/No switch per field. Hiding a field collapses its cell and
  may collapse a whole row; the block reflows and the panel says **Personalizado**. The
  value stays, so showing it again brings it back.
- **Restablecer modelo**: restores the model's field set without touching the values.
- **Cajetín ribbon tab** (index 3, before Simular): **Contenido** (one `Contenido: X` item
  that opens the editor's combo), **Posición** (top right, bottom right; portrait adds the
  bottom band; current marked, disabled under Sin cajetín) and **Datos** (Editar datos…,
  Restablecer when the block is personalized).
- **Nota**: a free multi-line note stored with the block, shown in the editor with the
  placeholder “escriba aquí su nota.” and not printed yet.

## Technical plan

- `punchi-fluid-core::title_block`: `TitleBlock` with twelve fields, `ContentModel`,
  `FieldSet`, `BlockPosition` and the `S/E` default; `TitleField` names are the file/ABI keys.
- `punchi-fluid-file::block` and `sheet`: stored model and visibility, position and model
  validation, format 11 with migration from 10 (and the earlier formats behind it).
- `punchi-fluid-doc`: one block per circuit, `set_block` as one undo entry.
- `punchi-fluid-export::block_layout`: one row/cell resolver for the three models, with
  hidden cells collapsing and empty rows shortening the block; A4 industrial 170 × 45,
  A3 industrial 180 × 65. `Page::title_rect` uses it; `drawing_area` excludes the band.
- `punchi-fluid-export::title_block`: cell-aware label/value placement; the value shrinks
  before it can cross the bottom rule.
- `punchi-fluid-ffi`: block get/set/preset, model and visibility get/set, the resolved
  layout for the canvas, ABI 10, field indices 0–11 plus sheet 12, the note and the
  reserved band.
- Qt: session model/visibility properties, the docked editor, the Cajetín ribbon tab, canvas
  block from the ABI layout, zones, `--block` capture flag.
- Docs and captures as listed in the pull request.

## Alternatives considered

- One fixed 180 × 65 block: rejected, too tall on A4.
- Keeping the two-band 32 mm block: rejected, does not carry the guide's fields.
- Forcing the block on every sheet: rejected by the owner; it is optional.
- Reference zones on every paper: rejected, A4 becomes noise; the guide recommends them for
  large sheets.

## Verification

Format migration and bounds, doc undo, export geometry per paper, model and position, every
text baseline inside its cell, the ABI round trip, model/visibility and presets, the reserved
band (example clear of the block), `--check-qml`, `--selftest` (including the five ribbon
tabs), `./scripts-dev/check.sh`, and the A4/A3 captures plus the docked editor.
