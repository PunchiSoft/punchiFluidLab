# Component cards

One **descriptive** image per catalog id, shown by **Descripción del componente** beside the
component's name and its `summary`. The card is a reference picture of the physical part, not
the schematic symbol: the sheet keeps the ISO 1219 symbol from the catalog file, and the card
never replaces it.

## Rules

- **One file per catalog id**, named `<catalog_id>.png`. The id must exist in
  [`../`](../) and the card table in `apps/punchi-fluid/src/cards.rs` must name it; a card
  without a definition is not added.
- **Normalized size: 800 × 450 px**, 16:9, white background, no border. The test
  `every_card_is_normalized` reads the PNG header and fails on another size.
- **The text is the definition's**, not a second description: the card shows the
  component's `name` and `summary` from its TOML. Do not repeat or contradict them here.
- **A licence and an origin are recorded below** for every file. A file without a row is not
  merged; the test `the_card_folder_records_its_licence` checks the row for every card in
  the table.
- **No capture of another application, manual or website enters this folder.** A reference
  image provided by the owner may guide the illustration and is recorded here; the file is
  stored only with the owner's licence.

## Normalizing an image

From the repository root, with ImageMagick:

```bash
magick "source.png" -resize 800x450! -background white -gravity center \
  -extent 800x450 -strip "assets/catalog/cards/<catalog_id>.png"
```

The source must already be 16:9 or the `!` resize will distort it; crop it to 16:9 first if
it is not. The current owner references are 2730 × 1536 (16:9), so they downscale without a
crop.

## Inventory

| File | Catalog id | Origin | Licence | Date |
|---|---|---|---|---|
| `cylinder.single_acting_20_100.png` | `cylinder.single_acting_20_100` | Provided by the project owner as a reference render, 2026-11-23; 2730 × 1536 source | GPL-3.0-or-later | 2026-11-23 |

The remaining reference images the owner keeps outside the repository do not have a row yet:
several have no catalog definition, and an image without a definition is not added.
